#![forbid(unsafe_code)]

use crate::ipc::{IpcCommand, StreamChunkPayload, StreamKind};
use cyrune_control_plane::citation::{
    CitationMaterial, CitationMaterialClaim, ClaimKind, EvidenceRef, SimpleReasoningRecord,
};
use cyrune_control_plane::execution_registry::approved_registry_root;
use cyrune_control_plane::execution_result::NoLlmAcceptedDraft;
use cyrune_control_plane::ledger::{LedgerManifest, LedgerWriter, RejectedLedgerInput};
use cyrune_control_plane::policy::{FailureSpec, PolicyTrace};
use cyrune_control_plane::resolved_turn_context::{
    ResolvedKernelAdapters, ResolvedTurnContext, SHIPPING_BINDING_ID, TimeoutPolicy,
};
use cyrune_control_plane::resolver::{ResolverInputs, resolve_explained_policy};
use cyrune_control_plane::retrieval::RetrievalError;
use cyrune_control_plane::turn::{
    TurnError, run_approved_execution_adapter_path, run_no_llm_accepted_path,
};
use cyrune_control_plane::working::WorkingProjection;
use cyrune_core_contract::{
    CorrelationId, DenialId, RuleId, RunId, RunKind, RunRejected, RunRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const DEFAULT_POLICY_PACK_ID: &str = "cyrune-free-default";
const DEFAULT_APPROVED_ADAPTER_ID: &str = "local-cli-single-process.v0.1";
const DEFAULT_APPROVED_ADAPTER_VERSION: &str = "0.1.0";
const DEFAULT_MODEL_ID: &str = "model.local";
const DEFAULT_MODEL_DIGEST: &str = "sha256:model";
const DEFAULT_SANDBOX_REF: &str = "SANDBOX_MINIMAL_CANONICAL.md#default-profile";
const EMPTY_WORKING_HASH: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_TERMINAL_CONFIG_BYTES: &str = "local wezterm = require 'wezterm'\n\
local mux = wezterm.mux\n\
\n\
local config = {\n\
  default_prog = { 'cyr', 'shell' },\n\
}\n\
\n\
wezterm.on('gui-startup', function(_)\n\
  local workspace_tab, workspace_pane, window = mux.spawn_window {\n\
    args = { 'cyr', 'shell' },\n\
  }\n\
  workspace_tab:set_title('Workspace')\n\
  workspace_pane:split {\n\
    direction = 'Right',\n\
    args = { 'cyr', 'view', 'evidence' },\n\
  }\n\
  workspace_pane:split {\n\
    direction = 'Bottom',\n\
    args = { 'cyr', 'view', 'working', '--follow' },\n\
  }\n\
  local policy_tab = window:spawn_tab {\n\
    args = { 'cyr', 'view', 'policy' },\n\
  }\n\
  policy_tab:set_title('Policy')\n\
end)\n\
\n\
return config\n";

#[derive(Debug, Clone)]
pub struct CommandContext {
    cyrune_home: PathBuf,
    resolver_inputs: ResolverInputs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    Single(Value),
    Stream(Vec<StreamChunkPayload>),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("contract error: {0}")]
    Contract(#[from] cyrune_core_contract::ContractError),
    #[error("turn error: {0}")]
    Turn(#[from] cyrune_control_plane::turn::TurnError),
    #[error("{0}")]
    Public(String),
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelPayload {
    pub correlation_id: CorrelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TailPayload {
    pub correlation_id: CorrelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetEvidencePayload {
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ListEvidencePayload {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExplainPolicyPayload {
    pub policy_pack: Option<String>,
    pub last_denial_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: String,
    pub cyrune_home: String,
    pub distribution_root: String,
    pub bundle_root: String,
    pub terminal_config_path: String,
    pub runtime_pid_path: String,
    pub registry_ready: bool,
    pub bundle_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EvidenceView {
    manifest: Value,
    run: Value,
    policy: Value,
    citation_bundle: Option<Value>,
    rr: Option<Value>,
    working_delta: Option<Value>,
    denial: Option<Value>,
    stdout: Option<String>,
    stderr: Option<String>,
}

#[derive(Debug, Clone)]
struct BundleResolution {
    distribution_root: PathBuf,
    bundle_root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundleVerificationMode {
    HostResolution,
    StrictPreflight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PackagedRunRejection {
    MissingBinding { binding_path: PathBuf },
    MissingRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PublicUnresolvedRejectKind {
    PackagedMissingBinding { binding_path: PathBuf },
    PackagedMissingRegistry,
    ResolverResource { detail: String },
    ResolverRegistry { detail: String },
    ResolverKernelMapping { detail: String },
    ShippingMemoryBackend,
    ShippingRetrievalSource,
}

impl CommandContext {
    #[cfg(test)]
    pub fn from_parts(cyrune_home: PathBuf, resolver_inputs: ResolverInputs) -> Self {
        Self {
            cyrune_home,
            resolver_inputs,
        }
    }

    pub fn from_environment() -> Result<Self, CommandError> {
        let cyrune_home = default_cyrune_home()?;
        ensure_home_layout(&cyrune_home)?;
        let resolver_inputs = default_resolver_inputs(&cyrune_home)?;
        Ok(Self {
            cyrune_home,
            resolver_inputs,
        })
    }

    pub fn execute(
        &self,
        command: IpcCommand,
        payload: Value,
    ) -> Result<CommandResult, CommandError> {
        match command {
            IpcCommand::Run => self.handle_run(payload),
            IpcCommand::Cancel => self.handle_cancel(payload),
            IpcCommand::Tail => self.handle_tail(payload),
            IpcCommand::GetEvidence => self.handle_get_evidence(payload),
            IpcCommand::ListEvidence => self.handle_list_evidence(payload),
            IpcCommand::GetWorking => self.handle_get_working(),
            IpcCommand::ExplainPolicy => self.handle_explain_policy(payload),
            IpcCommand::Health => self.handle_health(),
        }
    }

    pub fn cyrune_home(&self) -> &Path {
        &self.cyrune_home
    }
}

impl CommandContext {
    fn handle_run(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let request: RunRequest = serde_json::from_value(payload)?;
        ensure_home_layout(&self.cyrune_home)?;
        ensure_default_terminal_config(&self.cyrune_home)?;
        let mut writer = LedgerWriter::new(self.cyrune_home.clone());
        if let Some(rejected) =
            packaged_run_rejection(&mut writer, &self.resolver_inputs, &request)?
        {
            return Ok(CommandResult::Single(serde_json::to_value(rejected)?));
        }
        ensure_default_execution_adapter_assets(
            &self.cyrune_home,
            packaged_bundle_root(&self.resolver_inputs),
        )?;

        let now_ms = now_unix_ms()?;
        let terminal_result = match request.run_kind {
            RunKind::NoLlm => {
                let draft = synthesize_no_llm_draft(&request)?;
                run_no_llm_accepted_path(
                    &mut writer,
                    &self.resolver_inputs,
                    &request,
                    &draft,
                    now_ms,
                )
            }
            RunKind::ExecutionAdapter => run_approved_execution_adapter_path(
                &mut writer,
                &self.resolver_inputs,
                &request,
                now_ms,
            ),
        };
        let terminal_result = match terminal_result {
            Ok(result) => result,
            Err(error) => {
                if let Some(rejected) =
                    resolver_turn_rejection(&mut writer, &self.resolver_inputs, &request, &error)?
                {
                    return Ok(CommandResult::Single(serde_json::to_value(rejected)?));
                }
                if let Some(rejected) = shipping_memory_turn_rejection(
                    &mut writer,
                    &self.resolver_inputs,
                    &request,
                    &error,
                )? {
                    return Ok(CommandResult::Single(serde_json::to_value(rejected)?));
                }
                return Err(CommandError::Turn(error));
            }
        };

        let payload = match terminal_result {
            Ok(accepted) => serde_json::to_value(accepted)?,
            Err(rejected) => serde_json::to_value(rejected)?,
        };
        Ok(CommandResult::Single(payload))
    }

    fn handle_cancel(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let cancel: CancelPayload = serde_json::from_value(payload)?;
        Ok(CommandResult::Single(json!({
            "accepted": false,
            "correlation_id": cancel.correlation_id,
            "message": "cancel is not implemented in single_run free v0.1"
        })))
    }

    fn handle_tail(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let tail: TailPayload = serde_json::from_value(payload)?;
        let latest = latest_evidence_for_correlation(&self.cyrune_home, &tail.correlation_id)?;
        let stdout = fs::read_to_string(latest.join("stdout.log")).unwrap_or_default();
        let stderr = fs::read_to_string(latest.join("stderr.log")).unwrap_or_default();
        let manifest: LedgerManifest =
            serde_json::from_slice(&fs::read(latest.join("manifest.json"))?)?;

        let mut sequence = 0_u64;
        let mut chunks = Vec::new();
        if !stdout.is_empty() {
            sequence += 1;
            chunks.push(StreamChunkPayload {
                stream_kind: StreamKind::Stdout,
                sequence,
                eof: false,
                data: stdout,
            });
        }
        if !stderr.is_empty() {
            sequence += 1;
            chunks.push(StreamChunkPayload {
                stream_kind: StreamKind::Stderr,
                sequence,
                eof: false,
                data: stderr,
            });
        }
        sequence += 1;
        chunks.push(StreamChunkPayload {
            stream_kind: StreamKind::Status,
            sequence,
            eof: true,
            data: match manifest.outcome {
                cyrune_control_plane::ledger::EvidenceOutcome::Accepted => "accepted",
                cyrune_control_plane::ledger::EvidenceOutcome::Rejected => "rejected",
            }
            .to_string(),
        });
        Ok(CommandResult::Stream(chunks))
    }

    fn handle_get_evidence(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let request: GetEvidencePayload = serde_json::from_value(payload)?;
        let evidence_dir = self
            .cyrune_home
            .join("ledger")
            .join("evidence")
            .join(&request.evidence_id);
        let view = EvidenceView {
            manifest: read_json_value(evidence_dir.join("manifest.json"))?,
            run: read_json_value(evidence_dir.join("run.json"))?,
            policy: read_json_value(evidence_dir.join("policy.json"))?,
            citation_bundle: read_optional_json_value(evidence_dir.join("citation_bundle.json"))?,
            rr: read_optional_json_value(evidence_dir.join("rr.json"))?,
            working_delta: read_optional_json_value(evidence_dir.join("working_delta.json"))?,
            denial: read_optional_json_value(evidence_dir.join("denial.json"))?,
            stdout: read_optional_text(evidence_dir.join("stdout.log"))?,
            stderr: read_optional_text(evidence_dir.join("stderr.log"))?,
        };
        Ok(CommandResult::Single(serde_json::to_value(view)?))
    }

    fn handle_list_evidence(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let request: ListEvidencePayload = serde_json::from_value(payload)?;
        let index_path = self
            .cyrune_home
            .join("ledger")
            .join("manifests")
            .join("index.jsonl");
        let mut items = if index_path.exists() {
            fs::read_to_string(&index_path)?
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(serde_json::from_str::<Value>)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        items.reverse();
        if let Some(cursor) = request.cursor {
            items.retain(|item| {
                item.get("evidence_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| value < cursor.as_str())
            });
        }
        if let Some(limit) = request.limit {
            items.truncate(limit);
        }
        Ok(CommandResult::Single(json!({ "items": items })))
    }

    fn handle_get_working(&self) -> Result<CommandResult, CommandError> {
        let path = self.cyrune_home.join("working").join("working.json");
        let payload = if path.exists() {
            serde_json::to_value(serde_json::from_slice::<WorkingProjection>(&fs::read(
                path,
            )?)?)?
        } else {
            json!({
                "version": 1,
                "correlation_id": null,
                "slots": [],
                "limit": 12
            })
        };
        Ok(CommandResult::Single(payload))
    }

    fn handle_explain_policy(&self, payload: Value) -> Result<CommandResult, CommandError> {
        let request: ExplainPolicyPayload = serde_json::from_value(payload)?;
        let requested_policy_pack_id = request
            .policy_pack
            .clone()
            .unwrap_or_else(|| DEFAULT_POLICY_PACK_ID.to_string());
        let resolved_policy =
            resolve_explained_policy(&requested_policy_pack_id, &self.resolver_inputs).map_err(
                |_| {
                    CommandError::Public(format!(
                        "requested policy pack is unresolved: {requested_policy_pack_id}"
                    ))
                },
            )?;
        let denial = if let Some(denial_id) = request.last_denial_id {
            find_denial_by_id(&self.cyrune_home, &denial_id)?
        } else {
            None
        };
        Ok(CommandResult::Single(json!({
            "requested_policy_pack_id": requested_policy_pack_id,
            "policy_pack_id": resolved_policy.policy.policy_pack_id,
            "policy_pack": resolved_policy.policy.policy_pack_id,
            "policy": read_json_value(resolved_policy.source_path)?,
            "last_denial": denial
        })))
    }

    fn handle_health(&self) -> Result<CommandResult, CommandError> {
        ensure_home_layout(&self.cyrune_home)?;
        ensure_default_terminal_config(&self.cyrune_home)?;
        let bundle_root = packaged_bundle_root(&self.resolver_inputs);
        let bundle_ready = if let Some(bundle_root) = bundle_root {
            let manifest = read_release_manifest(&self.resolver_inputs.distribution_root)?;
            verify_bundle_root(bundle_root, &manifest)?;
            true
        } else {
            false
        };
        ensure_default_execution_adapter_assets(&self.cyrune_home, bundle_root)?;
        let report = HealthReport {
            status: "healthy".to_string(),
            cyrune_home: self.cyrune_home.display().to_string(),
            distribution_root: if bundle_ready {
                self.resolver_inputs.distribution_root.display().to_string()
            } else {
                String::new()
            },
            bundle_root: if bundle_ready {
                self.resolver_inputs.bundle_root.display().to_string()
            } else {
                String::new()
            },
            terminal_config_path: self
                .cyrune_home
                .join("terminal")
                .join("config")
                .join("wezterm.lua")
                .display()
                .to_string(),
            runtime_pid_path: self
                .cyrune_home
                .join("runtime")
                .join("daemon.pid")
                .display()
                .to_string(),
            registry_ready: self
                .cyrune_home
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("registry.json")
                .exists(),
            bundle_ready,
        };
        Ok(CommandResult::Single(serde_json::to_value(report)?))
    }
}

pub fn packaged_launch_preflight(cyrune_home: &Path) -> Result<HealthReport, CommandError> {
    packaged_launch_preflight_with_distribution_root_override(cyrune_home, None)
}

pub fn packaged_launch_preflight_with_distribution_root_override(
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<HealthReport, CommandError> {
    ensure_home_layout_with_distribution_root_override(cyrune_home, distribution_root_override)?;
    ensure_default_terminal_config(cyrune_home)?;
    let bundle = resolve_bundle_roots(
        distribution_root_override,
        BundleVerificationMode::StrictPreflight,
    )?;
    ensure_default_execution_adapter_assets(cyrune_home, Some(&bundle.bundle_root))?;
    Ok(HealthReport {
        status: "healthy".to_string(),
        cyrune_home: cyrune_home.display().to_string(),
        distribution_root: bundle.distribution_root.display().to_string(),
        bundle_root: bundle.bundle_root.display().to_string(),
        terminal_config_path: cyrune_home
            .join("terminal")
            .join("config")
            .join("wezterm.lua")
            .display()
            .to_string(),
        runtime_pid_path: cyrune_home
            .join("runtime")
            .join("daemon.pid")
            .display()
            .to_string(),
        registry_ready: cyrune_home
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("registry.json")
            .exists(),
        bundle_ready: true,
    })
}

pub fn default_cyrune_home() -> Result<PathBuf, CommandError> {
    if let Some(path) = env::var_os("CYRUNE_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".cyrune"));
    }
    Err(CommandError::Invalid(
        "CYRUNE_HOME and HOME are both unavailable".to_string(),
    ))
}

pub fn ensure_home_layout(cyrune_home: &Path) -> Result<(), CommandError> {
    ensure_home_layout_with_distribution_root_override(cyrune_home, None)
}

fn ensure_home_layout_with_distribution_root_override(
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<(), CommandError> {
    for dir in [
        cyrune_home.to_path_buf(),
        cyrune_home.join("terminal").join("config"),
        cyrune_home.join("runtime"),
        cyrune_home.join("runtime").join("ipc"),
        cyrune_home.join("runtime").join("logs"),
        cyrune_home.join("ledger").join("manifests"),
        cyrune_home.join("ledger").join("evidence"),
        cyrune_home.join("ledger").join("quarantine"),
        cyrune_home.join("working"),
        cyrune_home.join("packs").join("policy").join("default"),
        cyrune_home
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("profiles"),
        cyrune_home.join("cache"),
        cyrune_home.join("tmp"),
    ] {
        fs::create_dir_all(dir)?;
    }
    if let Some(home_template_root) = packaged_home_template_root(distribution_root_override)? {
        materialize_home_template_into_home(&home_template_root, cyrune_home)?;
    }
    let version_path = cyrune_home.join("version.json");
    if !version_path.exists() {
        fs::write(
            version_path,
            serde_json::to_vec_pretty(&json!({
                "version": "0.1.0",
                "product": "cyrune-free"
            }))?,
        )?;
    }
    Ok(())
}

fn packaged_home_template_root(
    distribution_root_override: Option<&Path>,
) -> Result<Option<PathBuf>, CommandError> {
    let distribution_root = match distribution_root_override {
        Some(path) => match resolve_distribution_root_override(path) {
            Ok(path) => path,
            Err(_) => return Ok(None),
        },
        None => {
            let current_executable_candidate = current_executable_distribution_root_candidate()
                .ok()
                .flatten();
            if env::var_os("CYRUNE_DISTRIBUTION_ROOT").is_none()
                && current_executable_candidate.is_none()
            {
                return Ok(None);
            }
            match resolve_distribution_root() {
                Ok(path) => path,
                Err(_) => return Ok(None),
            }
        }
    };
    let manifest = match read_release_manifest(&distribution_root) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(None),
    };
    let Some(home_template_path) = manifest.get("home_template_path").and_then(Value::as_str)
    else {
        return Ok(None);
    };
    if home_template_path.trim().is_empty() {
        return Ok(None);
    }
    let home_template_root = match fs::canonicalize(distribution_root.join(home_template_path)) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    if !home_template_root.starts_with(&distribution_root) {
        return Ok(None);
    }
    Ok(Some(home_template_root))
}

fn materialize_home_template_into_home(
    source_root: &Path,
    cyrune_home: &Path,
) -> Result<(), CommandError> {
    let embedding_source_root = source_root.join("embedding");
    if !embedding_source_root.exists() {
        return Ok(());
    }
    materialize_missing_tree(&embedding_source_root, &cyrune_home.join("embedding"))
}

fn materialize_missing_tree(source_root: &Path, target_root: &Path) -> Result<(), CommandError> {
    if !source_root.is_dir() {
        return Err(CommandError::Invalid(
            "home template projection root must be a directory".to_string(),
        ));
    }
    if target_root.exists() && !target_root.is_dir() {
        return Err(CommandError::Invalid(
            "home layout projection root must be a directory".to_string(),
        ));
    }
    fs::create_dir_all(target_root)?;
    for entry in fs::read_dir(source_root)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target_root.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            materialize_missing_tree(&source_path, &target_path)?;
            continue;
        }
        if file_type.is_file() {
            if target_path.exists() {
                if target_path.is_file() {
                    continue;
                }
                return Err(CommandError::Invalid(
                    "home template projection cannot replace directory".to_string(),
                ));
            }
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &target_path)?;
            let permissions = fs::metadata(&source_path)?.permissions();
            fs::set_permissions(&target_path, permissions)?;
            continue;
        }
        return Err(CommandError::Invalid(
            "home template entry type is unsupported".to_string(),
        ));
    }
    Ok(())
}

pub fn ensure_default_terminal_config(cyrune_home: &Path) -> Result<(), CommandError> {
    let config_path = cyrune_home
        .join("terminal")
        .join("config")
        .join("wezterm.lua");
    if config_path.exists() {
        return Ok(());
    }
    fs::write(config_path, DEFAULT_TERMINAL_CONFIG_BYTES)?;
    Ok(())
}

pub fn default_resolver_inputs(cyrune_home: &Path) -> Result<ResolverInputs, CommandError> {
    default_resolver_inputs_with_distribution_root_override(cyrune_home, None)
}

fn default_resolver_inputs_with_distribution_root_override(
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<ResolverInputs, CommandError> {
    match resolve_bundle_roots(
        distribution_root_override,
        BundleVerificationMode::HostResolution,
    ) {
        Ok(bundle) => Ok(packaged_resolver_inputs(cyrune_home, &bundle)),
        Err(error) => {
            if distribution_root_override.is_some()
                || env::var_os("CYRUNE_DISTRIBUTION_ROOT").is_some()
                || current_executable_distribution_root_candidate()?.is_some()
            {
                return Err(error);
            }
            let crane_root = detect_crane_root()?;
            Ok(ResolverInputs::new(
                cyrune_home,
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("catalog"),
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            ))
        }
    }
}

fn packaged_resolver_inputs(cyrune_home: &Path, bundle: &BundleResolution) -> ResolverInputs {
    ResolverInputs::new_packaged(
        cyrune_home,
        &bundle.distribution_root,
        &bundle.bundle_root,
        bundle.bundle_root.join("adapter").join("catalog"),
        bundle
            .bundle_root
            .join("adapter")
            .join("policies")
            .join("cyrune-free-default.v0.1.json"),
        bundle
            .bundle_root
            .join("adapter")
            .join("bindings")
            .join("cyrune-free-default.v0.1.json"),
    )
}

fn packaged_bundle_root(inputs: &ResolverInputs) -> Option<&Path> {
    (inputs.bundle_root != inputs.cyrune_home).then_some(inputs.bundle_root.as_path())
}

fn resolve_distribution_root() -> Result<PathBuf, CommandError> {
    if let Some(path) = env::var_os("CYRUNE_DISTRIBUTION_ROOT") {
        return resolve_distribution_root_override(Path::new(&path));
    }

    let current_exe = env::current_exe().map_err(CommandError::Io)?;
    let Some(distribution_root) = distribution_root_from_executable_path(&current_exe) else {
        return Err(CommandError::Invalid(
            "distribution root could not be derived from executable path".to_string(),
        ));
    };
    fs::canonicalize(distribution_root).map_err(|error| {
        CommandError::Invalid(format!("distribution_root cannot be materialized: {error}"))
    })
}

fn resolve_distribution_root_override(distribution_root: &Path) -> Result<PathBuf, CommandError> {
    if !distribution_root.is_absolute() {
        return Err(CommandError::Invalid(
            "CYRUNE_DISTRIBUTION_ROOT must be an absolute path".to_string(),
        ));
    }
    fs::canonicalize(distribution_root).map_err(|error| {
        CommandError::Invalid(format!("distribution_root cannot be materialized: {error}"))
    })
}

fn current_executable_distribution_root_candidate() -> Result<Option<PathBuf>, CommandError> {
    let current_exe = env::current_exe().map_err(CommandError::Io)?;
    Ok(distribution_root_from_executable_path(&current_exe))
}

fn distribution_root_from_executable_path(current_exe: &Path) -> Option<PathBuf> {
    let bin_dir = current_exe.parent()?;
    let distribution_root = bin_dir.parent()?;
    distribution_root
        .join("RELEASE_MANIFEST.json")
        .exists()
        .then(|| distribution_root.to_path_buf())
}

fn resolve_bundle_roots(
    distribution_root_override: Option<&Path>,
    verification_mode: BundleVerificationMode,
) -> Result<BundleResolution, CommandError> {
    let distribution_root = match distribution_root_override {
        Some(path) => resolve_distribution_root_override(path)?,
        None => resolve_distribution_root()?,
    };
    let manifest = read_release_manifest(&distribution_root)?;
    let bundle_root_path = manifest
        .get("bundle_root_path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            CommandError::Invalid(
                "bundle_root_path is missing from RELEASE_MANIFEST.json".to_string(),
            )
        })?;
    let bundle_root =
        fs::canonicalize(distribution_root.join(bundle_root_path)).map_err(|error| {
            CommandError::Invalid(format!("bundle_root cannot be materialized: {error}"))
        })?;
    if verification_mode == BundleVerificationMode::StrictPreflight {
        verify_bundle_root(&bundle_root, &manifest)?;
    }
    Ok(BundleResolution {
        distribution_root,
        bundle_root,
    })
}

fn read_release_manifest(distribution_root: &Path) -> Result<Value, CommandError> {
    read_json_value(distribution_root.join("RELEASE_MANIFEST.json"))
}

fn verify_bundle_root(bundle_root: &Path, manifest: &Value) -> Result<(), CommandError> {
    let Some(bundle_root_path) = manifest.get("bundle_root_path").and_then(Value::as_str) else {
        return Err(CommandError::Invalid(
            "bundle_root_path is missing from RELEASE_MANIFEST.json".to_string(),
        ));
    };
    if bundle_root_path.trim().is_empty() {
        return Err(CommandError::Invalid(
            "bundle_root_path is missing from RELEASE_MANIFEST.json".to_string(),
        ));
    }

    for required in [
        bundle_root.join("adapter").join("catalog"),
        bundle_root
            .join("adapter")
            .join("policies")
            .join("cyrune-free-default.v0.1.json"),
        bundle_root
            .join("adapter")
            .join("bindings")
            .join("cyrune-free-default.v0.1.json"),
        bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("registry.json"),
        bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("profiles")
            .join(format!("{DEFAULT_APPROVED_ADAPTER_ID}.json")),
        bundle_root
            .join("terminal")
            .join("templates")
            .join("wezterm.lua"),
        bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh"),
    ] {
        if !required.exists() {
            return Err(CommandError::Invalid(format!(
                "required bundle resource missing: {}",
                required.display()
            )));
        }
    }
    Ok(())
}

fn packaged_run_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
) -> Result<Option<RunRejected>, CommandError> {
    let requested_binding_path = resolver_inputs.requested_binding_path(request);
    let rejection = if !requested_binding_path.exists() {
        Some(PackagedRunRejection::MissingBinding {
            binding_path: requested_binding_path,
        })
    } else if let Some(bundle_root) = packaged_bundle_root(resolver_inputs) {
        match request.run_kind {
            RunKind::ExecutionAdapter
                if !approved_registry_root(bundle_root)
                    .join("registry.json")
                    .exists() =>
            {
                Some(PackagedRunRejection::MissingRegistry)
            }
            _ => None,
        }
    } else {
        None
    };

    rejection
        .map(|kind| {
            commit_packaged_binding_unresolved_rejection(writer, resolver_inputs, request, kind)
        })
        .transpose()
}

fn commit_packaged_binding_unresolved_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    rejection: PackagedRunRejection,
) -> Result<RunRejected, CommandError> {
    let kind = match rejection {
        PackagedRunRejection::MissingBinding { binding_path } => {
            PublicUnresolvedRejectKind::PackagedMissingBinding { binding_path }
        }
        PackagedRunRejection::MissingRegistry => {
            PublicUnresolvedRejectKind::PackagedMissingRegistry
        }
    };
    commit_public_unresolved_rejection(writer, resolver_inputs, request, kind)
}

fn resolver_turn_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    error: &TurnError,
) -> Result<Option<RunRejected>, CommandError> {
    let TurnError::Resolver(resolver_error) = error else {
        return Ok(None);
    };

    let public_unresolved_kind = match resolver_error {
        cyrune_control_plane::resolver::ResolverError::Adapter(source) => {
            Some(PublicUnresolvedRejectKind::ResolverResource {
                detail: source.to_string(),
            })
        }
        cyrune_control_plane::resolver::ResolverError::Registry(source) => {
            Some(PublicUnresolvedRejectKind::ResolverRegistry {
                detail: source.to_string(),
            })
        }
        cyrune_control_plane::resolver::ResolverError::Invalid(message)
            if message.contains("capability outside closed set")
                || message.contains("adapter_id is required") =>
        {
            None
        }
        cyrune_control_plane::resolver::ResolverError::Invalid(message) => {
            Some(PublicUnresolvedRejectKind::ResolverKernelMapping {
                detail: message.clone(),
            })
        }
        cyrune_control_plane::resolver::ResolverError::Contract(_) => None,
    };

    if let Some(kind) = public_unresolved_kind {
        return commit_public_unresolved_rejection(writer, resolver_inputs, request, kind)
            .map(Some);
    }

    let failure = match resolver_error {
        cyrune_control_plane::resolver::ResolverError::Contract(source) => FailureSpec::new(
            cyrune_control_plane::policy::FailureStage::RequestValidation,
            RuleId::parse("REQ-001").map_err(CommandError::Contract)?,
            cyrune_core_contract::ReasonKind::InvalidRequest,
            format!("request validation failed: {source}"),
            "request fields を修正して再実行する",
        ),
        cyrune_control_plane::resolver::ResolverError::Invalid(message) => FailureSpec::new(
            cyrune_control_plane::policy::FailureStage::RequestValidation,
            RuleId::parse("REQ-001").map_err(CommandError::Contract)?,
            cyrune_core_contract::ReasonKind::InvalidRequest,
            format!("request validation failed: {message}"),
            "request fields を修正して再実行する",
        ),
        _ => unreachable!("public unresolved resolver rejections are handled above"),
    }
    .map_err(|failure_error| CommandError::Invalid(failure_error.to_string()))?;

    commit_unresolved_rejection(writer, resolver_inputs, request, failure).map(Some)
}

fn shipping_memory_turn_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    error: &TurnError,
) -> Result<Option<RunRejected>, CommandError> {
    if resolver_inputs.public_unresolved_binding_id(request) != SHIPPING_BINDING_ID {
        return Ok(None);
    }

    let failure = match error {
        TurnError::Memory(_) => PublicUnresolvedRejectKind::ShippingMemoryBackend,
        TurnError::Retrieval(RetrievalError::Memory(_)) => {
            PublicUnresolvedRejectKind::ShippingRetrievalSource
        }
        _ => return Ok(None),
    };

    commit_public_unresolved_rejection(writer, resolver_inputs, request, failure).map(Some)
}

fn commit_public_unresolved_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    rejection: PublicUnresolvedRejectKind,
) -> Result<RunRejected, CommandError> {
    let failure = public_unresolved_failure(resolver_inputs, request, rejection)?;
    commit_unresolved_rejection(writer, resolver_inputs, request, failure)
}

fn public_unresolved_failure(
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    rejection: PublicUnresolvedRejectKind,
) -> Result<FailureSpec, CommandError> {
    match rejection {
        PublicUnresolvedRejectKind::PackagedMissingBinding { binding_path: _ } => {
            FailureSpec::binding_unresolved(
                RuleId::parse("BND-003").map_err(CommandError::Contract)?,
                format!(
                    "requested binding is unresolved: {}",
                    resolver_inputs.public_unresolved_binding_id(request)
                ),
                "requested binding artifact を復元して再実行する",
            )
        }
        PublicUnresolvedRejectKind::PackagedMissingRegistry => FailureSpec::binding_unresolved(
            RuleId::parse("BND-004").map_err(CommandError::Contract)?,
            "packaged approved execution adapter registry is unresolved",
            "approved execution adapter registry / profile resources を復元して再実行する",
        ),
        PublicUnresolvedRejectKind::ResolverResource { detail } => FailureSpec::binding_unresolved(
            RuleId::parse("BND-006").map_err(CommandError::Contract)?,
            format!("resolver could not load requested policy/binding resources: {detail}"),
            "requested policy/binding/catalog resources を修正して再実行する",
        ),
        PublicUnresolvedRejectKind::ResolverRegistry { detail } => FailureSpec::binding_unresolved(
            RuleId::parse("BND-007").map_err(CommandError::Contract)?,
            format!("approved execution adapter registry is unresolved: {detail}"),
            "approved execution adapter registry / profile resources を修正して再実行する",
        ),
        PublicUnresolvedRejectKind::ResolverKernelMapping { detail } => {
            FailureSpec::binding_unresolved(
                RuleId::parse("BND-008").map_err(CommandError::Contract)?,
                format!("resolver produced unresolved kernel adapter mapping: {detail}"),
                "requested policy/binding resources を修正して再実行する",
            )
        }
        PublicUnresolvedRejectKind::ShippingMemoryBackend => FailureSpec::binding_unresolved(
            RuleId::parse("BND-009").map_err(CommandError::Contract)?,
            "shipping memory backend is unresolved",
            "shipping memory backend materialization を実装または修正して再実行する",
        ),
        PublicUnresolvedRejectKind::ShippingRetrievalSource => FailureSpec::binding_unresolved(
            RuleId::parse("BND-010").map_err(CommandError::Contract)?,
            "shipping memory retrieval source is unresolved",
            "shipping memory backend / retrieval source を修正して再実行する",
        ),
    }
    .map_err(|error| CommandError::Invalid(error.to_string()))
}

fn commit_unresolved_rejection(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    failure: FailureSpec,
) -> Result<RunRejected, CommandError> {
    let context = unresolved_rejection_context(request, resolver_inputs);
    commit_rejection(writer, request, failure, context)
}

fn commit_rejection(
    writer: &mut LedgerWriter,
    request: &RunRequest,
    failure: FailureSpec,
    context: ResolvedTurnContext,
) -> Result<RunRejected, CommandError> {
    let mut policy_trace = PolicyTrace::new();
    policy_trace.record_failure(&failure);
    let now = timestamp_now_rfc3339()?;
    let committed = writer
        .commit_rejected(&RejectedLedgerInput {
            request: request.clone(),
            context: context.clone(),
            created_at: now.clone(),
            started_at: now.clone(),
            finished_at: now,
            exit_status: None,
            working_hash_before: EMPTY_WORKING_HASH.to_string(),
            query_summary: None,
            failure: failure.clone(),
            policy_trace,
            stdout: String::new(),
            stderr: String::new(),
        })
        .map_err(|error| CommandError::Invalid(error.to_string()))?;
    Ok(RunRejected {
        response_to: request.request_id.clone(),
        correlation_id: context.correlation_id,
        run_id: context.run_id,
        denial_id: DenialId::from_evidence_id(&committed.evidence_id),
        evidence_id: committed.evidence_id,
        rule_id: failure.rule_id,
        reason_kind: failure.reason_kind,
        message: failure.message,
        remediation: failure.remediation,
    })
}

fn unresolved_rejection_context(
    request: &RunRequest,
    resolver_inputs: &ResolverInputs,
) -> ResolvedTurnContext {
    ResolvedTurnContext {
        version: 1,
        request_id: request.request_id.clone(),
        correlation_id: request.correlation_id.clone(),
        run_id: RunId::for_single_run(&request.correlation_id),
        policy_pack_id: request.policy_pack_id.clone(),
        requested_policy_pack_id: request.policy_pack_id.clone(),
        requested_binding_id: request.binding_id.clone(),
        binding_id: resolver_inputs.public_unresolved_binding_id(request),
        resolved_kernel_adapters: ResolvedKernelAdapters {
            working_store_adapter_id: "unresolved".to_string(),
            processing_store_adapter_id: "unresolved".to_string(),
            permanent_store_adapter_id: "unresolved".to_string(),
            vector_index_adapter_id: "unresolved".to_string(),
            embedding_engine_ref: "crane-embed-null.v0.1".to_string(),
        },
        embedding_exact_pin: None,
        memory_state_roots: None,
        allowed_capabilities: request.requested_capabilities.clone(),
        sandbox_ref: DEFAULT_SANDBOX_REF.to_string(),
        run_kind: request.run_kind.clone(),
        io_mode: request.io_mode.clone(),
        selected_execution_adapter: unresolved_selected_execution_adapter(request),
        timeout_policy: TimeoutPolicy {
            turn_timeout_s: 120,
            execution_timeout_s: 120,
        },
    }
}

fn unresolved_selected_execution_adapter(
    _request: &RunRequest,
) -> Option<cyrune_control_plane::resolved_turn_context::SelectedExecutionAdapter> {
    None
}

fn crane_root_from_ancestors(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        if ancestor.join("Adapter").join("v0.1").join("0").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn detect_crane_root() -> Result<PathBuf, CommandError> {
    if let Some(path) = env::var_os("CRANE_ROOT") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(executable_path) = env::current_exe() {
        if let Some(root) = crane_root_from_ancestors(&executable_path) {
            return Ok(root);
        }
    }
    let cwd = env::current_dir()?;
    if let Some(root) = crane_root_from_ancestors(&cwd) {
        return Ok(root);
    }
    Err(CommandError::Invalid(
        "CRANE_ROOT could not be detected from environment, executable path, or workspace"
            .to_string(),
    ))
}

fn ensure_default_execution_adapter_assets(
    cyrune_home: &Path,
    bundle_root: Option<&Path>,
) -> Result<(), CommandError> {
    if let Some(bundle_root) = bundle_root {
        let approved_dir = cyrune_home
            .join("registry")
            .join("execution-adapters")
            .join("approved");
        let profiles_dir = approved_dir.join("profiles");
        fs::create_dir_all(&profiles_dir)?;

        let source_registry = bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("registry.json");
        let source_profile = bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("profiles")
            .join(format!("{DEFAULT_APPROVED_ADAPTER_ID}.json"));
        let source_launcher = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let target_launcher = cyrune_home
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");

        for source in [&source_registry, &source_profile, &source_launcher] {
            if !source.exists() {
                return Err(CommandError::Invalid(format!(
                    "required bundle resource missing: {}",
                    source.display()
                )));
            }
        }

        fs::copy(&source_registry, approved_dir.join("registry.json"))?;
        fs::copy(
            &source_profile,
            profiles_dir.join(format!("{DEFAULT_APPROVED_ADAPTER_ID}.json")),
        )?;
        fs::copy(&source_launcher, &target_launcher)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::metadata(&source_launcher)?.permissions().mode();
            fs::set_permissions(&target_launcher, fs::Permissions::from_mode(permissions))?;
        }
        return Ok(());
    }

    let launcher_path = cyrune_home
        .join("runtime")
        .join("ipc")
        .join("local-cli-single-process.sh");
    if !launcher_path.exists() {
        let script = r#"#!/bin/sh
PIN="${CYRUNE_LAUNCHER_SHA256:-sha256:missing}"
CORR="${CYRUNE_CORRELATION_ID:-RUN-19700101-0000}"
START="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
END="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '%s\n' "{\"adapter_id\":\"local-cli-single-process.v0.1\",\"adapter_version\":\"0.1.0\",\"correlation_id\":\"${CORR}\",\"terminal_status\":\"succeeded\",\"started_at\":\"${START}\",\"finished_at\":\"${END}\",\"exit_status\":0,\"output_draft\":\"- approved adapter accepted claim\",\"stdio\":{\"stdout\":\"approved adapter stdout\",\"stderr\":\"\"},\"pin\":{\"kind\":\"launcher_sha256\",\"value\":\"${PIN}\"},\"citation_material\":{\"claims\":[{\"text\":\"approved adapter accepted claim\",\"claim_kind\":\"extractive\",\"evidence_refs\":[{\"evidence_id\":\"EVID-1\"}]}]},\"rr_material\":{\"claims\":[\"approved adapter accepted claim\"],\"decisions\":[],\"assumptions\":[],\"actions\":[],\"citations_used\":[\"EVID-1\"]},\"failure_detail\":null}"
"#;
        fs::write(&launcher_path, script)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&launcher_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&launcher_path, permissions)?;
        }
    }
    let launcher_sha256 = format!("sha256:{}", sha256_hex(&fs::read(&launcher_path)?));

    let approved_dir = cyrune_home
        .join("registry")
        .join("execution-adapters")
        .join("approved");
    let profiles_dir = approved_dir.join("profiles");
    fs::create_dir_all(&profiles_dir)?;
    fs::write(
        approved_dir.join("registry.json"),
        serde_json::to_vec_pretty(&json!({
            "registry_version": "cyrune.free.execution-adapter-registry.v1",
            "entries": [
                {
                    "adapter_id": DEFAULT_APPROVED_ADAPTER_ID,
                    "state": "approved",
                    "profile_path": format!("profiles/{DEFAULT_APPROVED_ADAPTER_ID}.json")
                }
            ]
        }))?,
    )?;
    fs::write(
        profiles_dir.join(format!("{DEFAULT_APPROVED_ADAPTER_ID}.json")),
        serde_json::to_vec_pretty(&json!({
            "adapter_id": DEFAULT_APPROVED_ADAPTER_ID,
            "adapter_version": DEFAULT_APPROVED_ADAPTER_VERSION,
            "execution_kind": "process_stdio",
            "launcher_path": launcher_path.display().to_string(),
            "launcher_sha256": launcher_sha256,
            "model_id": DEFAULT_MODEL_ID,
            "model_revision_or_digest": DEFAULT_MODEL_DIGEST,
            "allowed_capabilities": ["exec", "fs_read"],
            "default_timeout_s": 120,
            "env_allowlist": []
        }))?,
    )?;
    Ok(())
}

fn synthesize_no_llm_draft(request: &RunRequest) -> Result<NoLlmAcceptedDraft, CommandError> {
    let claims = extract_claims(&request.user_input);
    let evidence_id = format!("EVID-INPUT-{}", request.request_id.as_str());
    let output_draft = claims
        .iter()
        .map(|claim| format!("- {claim}"))
        .collect::<Vec<_>>()
        .join("\n");
    let now = timestamp_now_rfc3339()?;
    Ok(NoLlmAcceptedDraft {
        started_at: now.clone(),
        finished_at: now,
        output_draft,
        stdio: cyrune_control_plane::execution_result::StdioCapture {
            stdout: "no-llm stdout".to_string(),
            stderr: String::new(),
        },
        citation_material: CitationMaterial {
            claims: claims
                .iter()
                .map(|claim| CitationMaterialClaim {
                    text: claim.clone(),
                    claim_kind: ClaimKind::Extractive,
                    evidence_refs: vec![EvidenceRef {
                        evidence_id: evidence_id.clone(),
                    }],
                })
                .collect(),
        },
        rr_material: SimpleReasoningRecord {
            claims,
            decisions: Vec::new(),
            assumptions: Vec::new(),
            actions: Vec::new(),
            citations_used: vec![evidence_id],
        },
    })
}

fn extract_claims(input: &str) -> Vec<String> {
    let mut claims = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let stripped = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
            .or_else(|| strip_numbered(trimmed))
            .unwrap_or(trimmed)
            .trim();
        if !stripped.is_empty() {
            claims.push(stripped.to_string());
        }
    }
    if claims.is_empty() {
        split_sentences(input)
    } else {
        claims
    }
}

fn strip_numbered(value: &str) -> Option<&str> {
    let digits = value.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    value[digits..].strip_prefix(". ")
}

fn split_sentences(input: &str) -> Vec<String> {
    let mut current = String::new();
    let mut sentences = Vec::new();
    for ch in input.chars() {
        current.push(ch);
        if matches!(ch, '。' | '！' | '？' | '.' | '!' | '?') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
    if sentences.is_empty() {
        vec![input.trim().to_string()]
    } else {
        sentences
    }
}

fn latest_evidence_for_correlation(
    cyrune_home: &Path,
    correlation_id: &CorrelationId,
) -> Result<PathBuf, CommandError> {
    let evidence_root = cyrune_home.join("ledger").join("evidence");
    let mut matches = Vec::new();
    if !evidence_root.exists() {
        return Err(CommandError::Invalid(
            "no evidence is available for tail".to_string(),
        ));
    }
    for entry in fs::read_dir(&evidence_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "tmp") {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let manifest: LedgerManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        if manifest.correlation_id == *correlation_id {
            matches.push((manifest.evidence_id, path));
        }
    }
    matches.sort_by_key(|(evidence_id, _)| evidence_id.as_str().to_string());
    matches.pop().map(|(_, path)| path).ok_or_else(|| {
        CommandError::Invalid("tail target correlation_id was not found".to_string())
    })
}

fn find_denial_by_id(cyrune_home: &Path, denial_id: &str) -> Result<Option<Value>, CommandError> {
    let evidence_root = cyrune_home.join("ledger").join("evidence");
    if !evidence_root.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(&evidence_root)? {
        let entry = entry?;
        let denial_path = entry.path().join("denial.json");
        if !denial_path.exists() {
            continue;
        }
        let denial = read_json_value(&denial_path)?;
        if denial
            .get("denial_id")
            .and_then(Value::as_str)
            .is_some_and(|value| value == denial_id)
        {
            return Ok(Some(denial));
        }
    }
    Ok(None)
}

fn read_json_value(path: impl AsRef<Path>) -> Result<Value, CommandError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn read_optional_json_value(path: impl AsRef<Path>) -> Result<Option<Value>, CommandError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(read_json_value(path)?))
}

fn read_optional_text(path: impl AsRef<Path>) -> Result<Option<String>, CommandError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

fn now_unix_ms() -> Result<u64, CommandError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CommandError::Invalid(error.to_string()))?;
    Ok(duration.as_millis() as u64)
}

fn timestamp_now_rfc3339() -> Result<String, CommandError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| CommandError::Invalid(error.to_string()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        CommandContext, DEFAULT_TERMINAL_CONFIG_BYTES, ExplainPolicyPayload, ListEvidencePayload,
        TailPayload, default_resolver_inputs_with_distribution_root_override, detect_crane_root,
        distribution_root_from_executable_path, ensure_default_terminal_config, ensure_home_layout,
        ensure_home_layout_with_distribution_root_override,
        packaged_launch_preflight_with_distribution_root_override,
    };
    use crate::ipc::IpcCommand;
    use cyrune_control_plane::ledger::{EvidenceOutcome, LedgerManifest};
    use cyrune_control_plane::resolver::ResolverInputs;
    use cyrune_core_contract::{
        CorrelationId, IoMode, ReasonKind, RequestId, RunKind, RunRejected, RunRequest,
    };
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    const ALT_POLICY_JSON: &str = r#"{
  "distro_id": "cyrune-free",
  "policy_pack_id": "cyrune-free-alt",
  "version": "v0.1",
  "layers": {
    "working": { "target_items": 10, "ttl_ms": 3600000, "eviction_strategy": "priority" },
    "processing": { "target_items": 20000, "ttl_ms": 3628800000, "promotion_threshold": 0.8 },
    "permanent": { "retention_mode": "immutable" }
  },
  "fail_closed": {
    "on_capacity_out_of_range": true,
    "on_ttl_out_of_range": true,
    "on_missing_capability": true
  }
}"#;

    fn write_fixture(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn write_packaged_distribution(
        root: &std::path::Path,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let distribution_root = root.join("distribution");
        let bundle_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        let home_template_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("home-template");
        let crane_root = detect_crane_root().unwrap();
        write_fixture(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "home_template_path": "share/cyrune/home-template",
  "runtime_entry": "bin/cyr",
  "daemon_entry": "bin/cyrune-daemon"
}"#,
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("catalog")
                .join("memory-kv-inmem.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("catalog")
                    .join("memory-kv-inmem.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("policies")
                .join("cyrune-free-alt.v0.1.json"),
            ALT_POLICY_JSON,
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("bindings")
                .join("cyrune-free-default.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &bundle_root
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("registry.json"),
            r#"{
  "registry_version": "cyrune.free.execution-adapter-registry.v1",
  "entries": [
    {
      "adapter_id": "local-cli-single-process.v0.1",
      "state": "approved",
      "profile_path": "profiles/local-cli-single-process.v0.1.json"
    }
  ]
}"#,
        );
        write_fixture(
            &bundle_root
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("profiles")
                .join("local-cli-single-process.v0.1.json"),
            r#"{
  "adapter_id": "local-cli-single-process.v0.1",
  "adapter_version": "0.1.0",
  "execution_kind": "process_stdio",
  "launcher_path": "runtime/ipc/local-cli-single-process.sh",
  "launcher_sha256": "sha256:placeholder",
  "model_id": "model.local",
  "model_revision_or_digest": "sha256:model",
  "allowed_capabilities": ["exec", "fs_read"],
  "default_timeout_s": 120,
  "env_allowlist": []
}"#,
        );
        write_fixture(
            &bundle_root
                .join("terminal")
                .join("templates")
                .join("wezterm.lua"),
            DEFAULT_TERMINAL_CONFIG_BYTES,
        );
        write_fixture(
            &bundle_root
                .join("runtime")
                .join("ipc")
                .join("local-cli-single-process.sh"),
            "#!/bin/sh\nexit 0\n",
        );
        write_fixture(
            &home_template_root.join("version.json"),
            r#"{
  "version": "0.1.0",
  "product": "cyrune-free"
}"#,
        );
        (distribution_root, bundle_root)
    }

    fn write_packaged_shipping_assets(
        distribution_root: &std::path::Path,
        bundle_root: &std::path::Path,
    ) {
        let crane_root = detect_crane_root().unwrap();
        let embedding_source_root = complete_shipping_embedding_root();
        let home_template_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("home-template");
        for file_name in [
            "memory-kv-inmem.v0.1.json",
            "memory-redb-processing.v0.1.json",
            "memory-stoolap-permanent.v0.1.json",
        ] {
            write_fixture(
                &bundle_root.join("adapter").join("catalog").join(file_name),
                &fs::read_to_string(
                    crane_root
                        .join("Adapter")
                        .join("v0.1")
                        .join("0")
                        .join("catalog")
                        .join(file_name),
                )
                .unwrap(),
            );
            for relative_path in [
                "exact-pins/cyrune-free-shipping.v0.1.json",
                "artifacts/multilingual-e5-small/model.onnx",
                "artifacts/multilingual-e5-small/tokenizer.json",
                "artifacts/multilingual-e5-small/config.json",
                "artifacts/multilingual-e5-small/special_tokens_map.json",
                "artifacts/multilingual-e5-small/tokenizer_config.json",
            ] {
                let source_path = embedding_source_root.join(relative_path);
                let contents = fs::read(&source_path).unwrap();
                write_fixture(&bundle_root.join("embedding").join(relative_path), "");
                fs::write(bundle_root.join("embedding").join(relative_path), &contents).unwrap();
                write_fixture(
                    &home_template_root.join("embedding").join(relative_path),
                    "",
                );
                fs::write(
                    home_template_root.join("embedding").join(relative_path),
                    &contents,
                )
                .unwrap();
            }
        }
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("bindings")
                .join("cyrune-free-default.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &bundle_root
                .join("adapter")
                .join("bindings")
                .join("cyrune-free-shipping.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-shipping.v0.1.json"),
            )
            .unwrap(),
        );
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn complete_shipping_embedding_root() -> PathBuf {
        let mut candidates = Vec::new();
        if let Some(root) = std::env::var_os("CYRUNE_TEST_SHIPPING_HOME_ROOT") {
            candidates.push(PathBuf::from(root).join("embedding"));
        }
        candidates.push(workspace_root().join("target/public-run/home/embedding"));
        candidates.push(workspace_root().join("resources/bundle-root/embedding"));

        for candidate in candidates {
            if has_required_shipping_embedding_files(&candidate) {
                return candidate;
            }
        }

        panic!(
            "shipping daemon tests require materialized embedding artifacts; run ./scripts/prepare-public-run.sh or set CYRUNE_TEST_SHIPPING_HOME_ROOT"
        );
    }

    fn has_required_shipping_embedding_files(root: &Path) -> bool {
        [
            "exact-pins/cyrune-free-shipping.v0.1.json",
            "artifacts/multilingual-e5-small/model.onnx",
            "artifacts/multilingual-e5-small/tokenizer.json",
            "artifacts/multilingual-e5-small/config.json",
            "artifacts/multilingual-e5-small/special_tokens_map.json",
            "artifacts/multilingual-e5-small/tokenizer_config.json",
        ]
        .iter()
        .all(|relative_path| root.join(relative_path).is_file())
    }

    fn packaged_context(cyrune_home: &std::path::Path) -> CommandContext {
        let (distribution_root, bundle_root) = write_packaged_distribution(cyrune_home);
        CommandContext::from_parts(
            cyrune_home.to_path_buf(),
            ResolverInputs::new_packaged(
                cyrune_home,
                &distribution_root,
                &bundle_root,
                bundle_root.join("adapter").join("catalog"),
                bundle_root
                    .join("adapter")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
                bundle_root
                    .join("adapter")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            ),
        )
    }

    fn no_llm_request_with_binding(binding_id: Option<&str>) -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0902").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0902").unwrap(),
            run_kind: RunKind::NoLlm,
            user_input: "binding selection".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: binding_id.map(ToOwned::to_owned),
            requested_capabilities: vec!["fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: None,
            argv: None,
            cwd: None,
            env_overrides: None,
        }
    }

    fn execution_adapter_request() -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0903").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0903").unwrap(),
            run_kind: RunKind::ExecutionAdapter,
            user_input: "adapter binding selection".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: Some("local-cli-single-process.v0.1".to_string()),
            argv: None,
            cwd: None,
            env_overrides: None,
        }
    }

    #[test]
    fn distribution_root_from_executable_path_requires_release_manifest() {
        let temp = tempdir().unwrap();
        let distribution_root = temp.path().join("distribution");
        let bin_path = distribution_root.join("bin").join("cyrune-daemon");
        write_fixture(&bin_path, "");

        assert_eq!(distribution_root_from_executable_path(&bin_path), None);

        write_fixture(&distribution_root.join("RELEASE_MANIFEST.json"), "{}");
        assert_eq!(
            distribution_root_from_executable_path(&bin_path),
            Some(distribution_root)
        );
    }

    #[test]
    fn tail_preserves_sequence_order() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let evidence_dir = temp.path().join("ledger").join("evidence").join("EVID-1");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(
            evidence_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&LedgerManifest {
                evidence_id: cyrune_core_contract::EvidenceId::new(1),
                correlation_id: CorrelationId::parse("RUN-20260327-0901").unwrap(),
                run_id: cyrune_core_contract::RunId::parse("RUN-20260327-0901-R01").unwrap(),
                outcome: EvidenceOutcome::Accepted,
                created_at: "2026-03-27T09:00:00Z".to_string(),
                policy_pack_id: "cyrune-free-default".to_string(),
                working_hash_before: "sha256:before".to_string(),
                working_hash_after: "sha256:after".to_string(),
                citation_bundle_id: None,
                rr_present: false,
            })
            .unwrap(),
        )
        .unwrap();
        fs::write(evidence_dir.join("stdout.log"), "stdout\n").unwrap();
        fs::write(evidence_dir.join("stderr.log"), "stderr\n").unwrap();

        let context = packaged_context(temp.path());
        let result = context
            .execute(
                IpcCommand::Tail,
                serde_json::to_value(TailPayload {
                    correlation_id: CorrelationId::parse("RUN-20260327-0901").unwrap(),
                })
                .unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Stream(chunks) = result else {
            panic!("tail must stream");
        };
        assert_eq!(chunks[0].sequence, 1);
        assert_eq!(chunks[1].sequence, 2);
        assert_eq!(chunks[2].sequence, 3);
        assert!(chunks[2].eof);
    }

    #[test]
    fn run_rejects_missing_explicit_shipping_binding_without_fallback() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let request = no_llm_request_with_binding(Some("cyrune-free-shipping.v0.1"));

        let result = context
            .execute(IpcCommand::Run, serde_json::to_value(request).unwrap())
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("run must return single payload");
        };
        let rejection: RunRejected = serde_json::from_value(value).unwrap();

        assert_eq!(rejection.reason_kind, ReasonKind::BindingUnresolved);
        assert_eq!(rejection.rule_id.as_str(), "BND-003");
        assert!(rejection.message.contains("cyrune-free-shipping.v0.1"));
        assert!(
            !rejection
                .message
                .contains(&temp.path().display().to_string())
        );
        assert!(
            !rejection
                .remediation
                .contains(&temp.path().display().to_string())
        );
    }

    #[test]
    fn run_normalizes_missing_shipping_binding_rejection_for_shorthand_and_canonical_requests() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let canonical_request = no_llm_request_with_binding(Some("cyrune-free-shipping.v0.1"));
        let shorthand_request = no_llm_request_with_binding(Some("cyrune-free-shipping"));

        let canonical_result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(canonical_request).unwrap(),
            )
            .unwrap();
        let shorthand_result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(shorthand_request).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(canonical_value) = canonical_result else {
            panic!("canonical run must return single payload");
        };
        let crate::command::CommandResult::Single(shorthand_value) = shorthand_result else {
            panic!("shorthand run must return single payload");
        };
        let canonical_rejection: RunRejected = serde_json::from_value(canonical_value).unwrap();
        let shorthand_rejection: RunRejected = serde_json::from_value(shorthand_value).unwrap();

        assert_eq!(
            canonical_rejection.reason_kind,
            ReasonKind::BindingUnresolved
        );
        assert_eq!(
            shorthand_rejection.reason_kind,
            ReasonKind::BindingUnresolved
        );
        assert_eq!(
            canonical_rejection.rule_id.as_str(),
            shorthand_rejection.rule_id.as_str()
        );
        assert_eq!(canonical_rejection.message, shorthand_rejection.message);
        assert_eq!(
            canonical_rejection.remediation,
            shorthand_rejection.remediation
        );

        let canonical_policy_json = fs::read_to_string(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(canonical_rejection.evidence_id.as_str())
                .join("policy.json"),
        )
        .unwrap();
        let shorthand_policy_json = fs::read_to_string(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(shorthand_rejection.evidence_id.as_str())
                .join("policy.json"),
        )
        .unwrap();
        let canonical_policy: serde_json::Value =
            serde_json::from_str(&canonical_policy_json).unwrap();
        let shorthand_policy: serde_json::Value =
            serde_json::from_str(&shorthand_policy_json).unwrap();
        assert_eq!(canonical_policy["binding_id"], "cyrune-free-shipping.v0.1");
        assert_eq!(shorthand_policy["binding_id"], "cyrune-free-shipping.v0.1");
    }

    #[test]
    fn run_rejects_missing_packaged_registry_without_path_leakage() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        fs::remove_file(
            temp.path()
                .join("distribution")
                .join("share")
                .join("cyrune")
                .join("bundle-root")
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("registry.json"),
        )
        .unwrap();

        let result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(execution_adapter_request()).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("run must return single payload");
        };
        let rejection: RunRejected = serde_json::from_value(value).unwrap();

        assert_eq!(rejection.reason_kind, ReasonKind::BindingUnresolved);
        assert_eq!(rejection.rule_id.as_str(), "BND-004");
        assert_eq!(
            rejection.message,
            "packaged approved execution adapter registry is unresolved"
        );
        assert_eq!(
            rejection.remediation,
            "approved execution adapter registry / profile resources を復元して再実行する"
        );
        assert!(
            !rejection
                .message
                .contains(&temp.path().display().to_string())
        );
        assert!(
            !rejection
                .remediation
                .contains(&temp.path().display().to_string())
        );
    }

    #[test]
    fn run_rejects_missing_policy_with_evidence_instead_of_daemon_error() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        fs::remove_file(
            temp.path()
                .join("distribution")
                .join("share")
                .join("cyrune")
                .join("bundle-root")
                .join("adapter")
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
        )
        .unwrap();

        let result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(no_llm_request_with_binding(None)).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("run must return single payload");
        };
        let rejection: RunRejected = serde_json::from_value(value).unwrap();

        assert_eq!(rejection.reason_kind, ReasonKind::BindingUnresolved);
        assert_eq!(rejection.rule_id.as_str(), "BND-006");
    }

    #[test]
    fn run_rejects_shipping_binding_when_exact_pin_source_is_missing() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        write_packaged_shipping_assets(&distribution_root, &bundle_root);
        fs::remove_file(
            bundle_root
                .join("embedding")
                .join("exact-pins")
                .join("cyrune-free-shipping.v0.1.json"),
        )
        .unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            ResolverInputs::new_packaged(
                temp.path(),
                &distribution_root,
                &bundle_root,
                bundle_root.join("adapter").join("catalog"),
                bundle_root
                    .join("adapter")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
                bundle_root
                    .join("adapter")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            ),
        );
        let request = no_llm_request_with_binding(Some("cyrune-free-shipping.v0.1"));

        let result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(request.clone()).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("run must return single payload");
        };
        let rejection: RunRejected = serde_json::from_value(value).unwrap();

        assert_eq!(rejection.response_to, request.request_id);
        assert_eq!(rejection.reason_kind, ReasonKind::BindingUnresolved);
        assert_eq!(rejection.rule_id.as_str(), "BND-006");
        assert!(
            rejection
                .message
                .contains("shipping exact pin authoritative source missing")
        );
        assert!(
            !rejection
                .message
                .contains(&temp.path().display().to_string())
        );
        let policy_json = fs::read_to_string(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(rejection.evidence_id.as_str())
                .join("policy.json"),
        )
        .unwrap();
        let policy_value: serde_json::Value = serde_json::from_str(&policy_json).unwrap();
        assert_eq!(policy_value["binding_id"], "cyrune-free-shipping.v0.1");
        assert_eq!(
            policy_value["resolved_kernel_adapters"]["processing_store_adapter_id"],
            "unresolved"
        );
        assert_eq!(
            policy_value["resolved_kernel_adapters"]["permanent_store_adapter_id"],
            "unresolved"
        );
        assert!(policy_value["embedding_exact_pin"].is_null());
        assert!(policy_value["memory_state_roots"].is_null());
        println!("shipping_policy_json={policy_json}");
        println!(
            "rejected_reason_kind={}",
            serde_json::to_string(&rejection.reason_kind).unwrap()
        );
        println!("rejected_evidence_id={}", rejection.evidence_id.as_str());
    }

    #[test]
    fn run_prefers_exact_pin_source_unresolved_before_memory_backend_checks() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        fs::create_dir_all(temp.path().join("memory")).unwrap();
        fs::write(temp.path().join("memory").join("processing"), "blocked").unwrap();
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        write_packaged_shipping_assets(&distribution_root, &bundle_root);
        fs::remove_file(
            bundle_root
                .join("embedding")
                .join("exact-pins")
                .join("cyrune-free-shipping.v0.1.json"),
        )
        .unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            ResolverInputs::new_packaged(
                temp.path(),
                &distribution_root,
                &bundle_root,
                bundle_root.join("adapter").join("catalog"),
                bundle_root
                    .join("adapter")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
                bundle_root
                    .join("adapter")
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json"),
            ),
        );
        let request = no_llm_request_with_binding(Some("cyrune-free-shipping.v0.1"));

        let result = context
            .execute(
                IpcCommand::Run,
                serde_json::to_value(request.clone()).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("run must return single payload");
        };
        let rejection: RunRejected = serde_json::from_value(value).unwrap();

        assert_eq!(rejection.reason_kind, ReasonKind::BindingUnresolved);
        assert_eq!(rejection.rule_id.as_str(), "BND-006");
        assert!(
            rejection
                .message
                .contains("shipping exact pin authoritative source missing")
        );
        assert!(
            !rejection
                .message
                .contains(&temp.path().display().to_string())
        );
        let policy_json = fs::read_to_string(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(rejection.evidence_id.as_str())
                .join("policy.json"),
        )
        .unwrap();
        let policy_value: serde_json::Value = serde_json::from_str(&policy_json).unwrap();
        assert_eq!(policy_value["binding_id"], "cyrune-free-shipping.v0.1");
        assert_eq!(
            policy_value["resolved_kernel_adapters"]["processing_store_adapter_id"],
            "unresolved"
        );
        assert_eq!(
            policy_value["resolved_kernel_adapters"]["permanent_store_adapter_id"],
            "unresolved"
        );
        assert!(policy_value["embedding_exact_pin"].is_null());
        assert!(policy_value["memory_state_roots"].is_null());
        println!("rejected_policy_json={policy_json}");
        println!("rejected_evidence_id={}", rejection.evidence_id.as_str());
    }

    #[test]
    fn explain_policy_returns_current_policy_file() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let result = context
            .execute(
                IpcCommand::ExplainPolicy,
                serde_json::to_value(ExplainPolicyPayload::default()).unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("explain policy must return single payload");
        };
        assert_eq!(
            value.get("policy_pack").and_then(serde_json::Value::as_str),
            Some("cyrune-free-default")
        );
        assert_eq!(
            value
                .get("requested_policy_pack_id")
                .and_then(serde_json::Value::as_str),
            Some("cyrune-free-default")
        );
        assert_eq!(
            value
                .get("policy_pack_id")
                .and_then(serde_json::Value::as_str),
            Some("cyrune-free-default")
        );
    }

    #[test]
    fn explain_policy_uses_same_requested_policy_selection_rule_as_run() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let result = context
            .execute(
                IpcCommand::ExplainPolicy,
                serde_json::to_value(ExplainPolicyPayload {
                    policy_pack: Some("cyrune-free-alt".to_string()),
                    last_denial_id: None,
                })
                .unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("explain policy must return single payload");
        };
        assert_eq!(
            value
                .get("requested_policy_pack_id")
                .and_then(serde_json::Value::as_str),
            Some("cyrune-free-alt")
        );
        assert_eq!(
            value
                .get("policy_pack_id")
                .and_then(serde_json::Value::as_str),
            Some("cyrune-free-alt")
        );
    }

    #[test]
    fn list_evidence_returns_items_payload() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let result = context
            .execute(
                IpcCommand::ListEvidence,
                serde_json::to_value(ListEvidencePayload {
                    limit: Some(5),
                    cursor: None,
                })
                .unwrap(),
            )
            .unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("list evidence must return single payload");
        };
        assert_eq!(value, json!({"items": []}));
    }

    #[test]
    fn default_terminal_config_emits_canonical_workspace_and_policy_tabs() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        ensure_default_terminal_config(temp.path()).unwrap();

        let config_path = temp
            .path()
            .join("terminal")
            .join("config")
            .join("wezterm.lua");
        let config_bytes = fs::read(config_path).unwrap();

        assert_eq!(config_bytes, DEFAULT_TERMINAL_CONFIG_BYTES.as_bytes());
    }

    #[test]
    fn default_terminal_config_does_not_overwrite_existing_file() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let config_path = temp
            .path()
            .join("terminal")
            .join("config")
            .join("wezterm.lua");
        fs::write(&config_path, "custom-marker\n").unwrap();

        ensure_default_terminal_config(temp.path()).unwrap();

        let config = fs::read_to_string(config_path).unwrap();
        assert_eq!(config, "custom-marker\n");
    }

    #[test]
    fn ensure_home_layout_materializes_packaged_home_template_projection() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        write_packaged_shipping_assets(&distribution_root, &bundle_root);

        ensure_home_layout_with_distribution_root_override(
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap();

        let template_manifest = distribution_root
            .join("share")
            .join("cyrune")
            .join("home-template")
            .join("embedding")
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        let materialized_manifest = cyrune_home
            .join("embedding")
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        assert_eq!(
            fs::read(&materialized_manifest).unwrap(),
            fs::read(template_manifest).unwrap()
        );
        assert!(
            cyrune_home
                .join("embedding")
                .join("artifacts")
                .join("multilingual-e5-small")
                .join("model.onnx")
                .is_file()
        );
    }

    #[test]
    fn ensure_home_layout_does_not_overwrite_existing_materialized_file() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        write_packaged_shipping_assets(&distribution_root, &bundle_root);

        fs::create_dir_all(cyrune_home.join("embedding").join("exact-pins")).unwrap();
        let materialized_manifest = cyrune_home
            .join("embedding")
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        fs::write(&materialized_manifest, "{\"marker\":\"existing\"}\n").unwrap();

        ensure_home_layout_with_distribution_root_override(
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(materialized_manifest).unwrap(),
            "{\"marker\":\"existing\"}\n"
        );
        assert!(
            cyrune_home
                .join("embedding")
                .join("artifacts")
                .join("multilingual-e5-small")
                .join("model.onnx")
                .is_file()
        );
    }

    #[test]
    fn default_resolver_inputs_uses_distribution_root_override() {
        let temp = tempdir().unwrap();
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        let distribution_root = fs::canonicalize(distribution_root).unwrap();
        let bundle_root = fs::canonicalize(bundle_root).unwrap();
        let inputs = default_resolver_inputs_with_distribution_root_override(
            temp.path(),
            Some(distribution_root.as_path()),
        )
        .unwrap();
        assert_eq!(inputs.distribution_root, distribution_root);
        assert_eq!(inputs.bundle_root, bundle_root);
        assert_eq!(
            inputs.catalog_dir,
            bundle_root.join("adapter").join("catalog")
        );
        assert_eq!(
            inputs.policy_path,
            bundle_root
                .join("adapter")
                .join("policies")
                .join("cyrune-free-default.v0.1.json")
        );
        assert_eq!(
            inputs.binding_path,
            bundle_root
                .join("adapter")
                .join("bindings")
                .join("cyrune-free-default.v0.1.json")
        );
    }

    #[test]
    fn default_resolver_inputs_rejects_relative_distribution_root_override() {
        let temp = tempdir().unwrap();
        let error = default_resolver_inputs_with_distribution_root_override(
            temp.path(),
            Some(std::path::Path::new("relative/path")),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::CommandError::Invalid(message)
                if message.contains("CYRUNE_DISTRIBUTION_ROOT must be an absolute path")
        ));
    }

    #[test]
    fn default_resolver_inputs_rejects_missing_bundle_root_path() {
        let temp = tempdir().unwrap();
        let distribution_root = temp.path().join("distribution");
        write_fixture(&distribution_root.join("RELEASE_MANIFEST.json"), "{}");
        let error = default_resolver_inputs_with_distribution_root_override(
            temp.path(),
            Some(distribution_root.as_path()),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::CommandError::Invalid(message)
                if message.contains("bundle_root_path is missing from RELEASE_MANIFEST.json")
        ));
    }

    #[test]
    fn packaged_launch_preflight_reports_packaged_roots_and_generated_terminal_config() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());

        let report = packaged_launch_preflight_with_distribution_root_override(
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap();

        assert_eq!(report.status, "healthy");
        assert_eq!(
            report.distribution_root,
            fs::canonicalize(distribution_root)
                .unwrap()
                .display()
                .to_string()
        );
        assert_eq!(
            report.bundle_root,
            fs::canonicalize(bundle_root).unwrap().display().to_string()
        );
        assert_eq!(report.cyrune_home, cyrune_home.display().to_string());
        assert_eq!(
            report.terminal_config_path,
            cyrune_home
                .join("terminal")
                .join("config")
                .join("wezterm.lua")
                .display()
                .to_string()
        );
        assert!(Path::new(&report.terminal_config_path).is_file());
        assert!(report.bundle_ready);
        assert!(report.registry_ready);
    }

    #[test]
    fn packaged_launch_preflight_rejects_relative_distribution_root_override() {
        let temp = tempdir().unwrap();
        let error = packaged_launch_preflight_with_distribution_root_override(
            temp.path(),
            Some(std::path::Path::new("relative/path")),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::CommandError::Invalid(message)
                if message.contains("CYRUNE_DISTRIBUTION_ROOT must be an absolute path")
        ));
    }

    #[test]
    fn packaged_launch_preflight_requires_strict_bundle_closure() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let (distribution_root, bundle_root) = write_packaged_distribution(temp.path());
        fs::remove_file(
            bundle_root
                .join("terminal")
                .join("templates")
                .join("wezterm.lua"),
        )
        .unwrap();

        let error = packaged_launch_preflight_with_distribution_root_override(
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            super::CommandError::Invalid(message)
                if message.contains("required bundle resource missing")
        ));
    }

    #[test]
    fn handle_health_reports_bundle_root_and_bundle_ready() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        let result = context.execute(IpcCommand::Health, json!({})).unwrap();
        let crate::command::CommandResult::Single(value) = result else {
            panic!("health must return single payload");
        };
        assert_eq!(
            value.get("status").and_then(serde_json::Value::as_str),
            Some("healthy")
        );
        let distribution_root = temp.path().join("distribution");
        let bundle_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        assert_eq!(
            value
                .get("distribution_root")
                .and_then(serde_json::Value::as_str),
            Some(distribution_root.to_str().unwrap())
        );
        assert_eq!(
            value.get("bundle_root").and_then(serde_json::Value::as_str),
            Some(bundle_root.to_str().unwrap())
        );
        assert_eq!(
            value
                .get("bundle_ready")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn handle_health_rejects_missing_bundle_registry() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = packaged_context(temp.path());
        fs::remove_file(
            temp.path()
                .join("distribution")
                .join("share")
                .join("cyrune")
                .join("bundle-root")
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("registry.json"),
        )
        .unwrap();
        let error = context.execute(IpcCommand::Health, json!({})).unwrap_err();
        assert!(matches!(
            error,
            super::CommandError::Invalid(message)
                if message.contains("required bundle resource missing")
        ));
    }
}
