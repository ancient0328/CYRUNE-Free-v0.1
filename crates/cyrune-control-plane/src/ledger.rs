#![forbid(unsafe_code)]

use crate::citation::{CitationBundle, SimpleReasoningRecord};
use crate::policy::{FailureSpec, PolicyTrace};
use crate::resolved_turn_context::ResolvedTurnContext;
use crate::retrieval::QuerySummary;
use crate::working::{WorkingProjection, WorkingRebuildOutput};
use cyrune_core_contract::{EvidenceId, RunRequest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const TERMINAL_BINDING_SCHEMA_VERSION: &str = "cyrune.free.terminal-binding.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOutcome {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerManifest {
    pub evidence_id: EvidenceId,
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub run_id: cyrune_core_contract::RunId,
    pub outcome: EvidenceOutcome,
    pub created_at: String,
    pub policy_pack_id: String,
    pub working_hash_before: String,
    pub working_hash_after: String,
    pub citation_bundle_id: Option<cyrune_core_contract::CitationBundleId>,
    pub rr_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunLedgerRecord {
    pub request_id: cyrune_core_contract::RequestId,
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub run_id: cyrune_core_contract::RunId,
    pub run_kind: cyrune_core_contract::RunKind,
    pub adapter_id: Option<String>,
    pub cwd: Option<String>,
    pub argv: Option<Vec<String>>,
    pub requested_capabilities: Vec<String>,
    pub started_at: String,
    pub finished_at: String,
    pub exit_status: Option<i32>,
    pub query_summary: Option<QuerySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyLedgerRecord {
    pub requested_policy_pack_id: String,
    pub requested_binding_id: Option<String>,
    pub policy_pack_id: String,
    pub binding_id: String,
    pub resolved_kernel_adapters: crate::resolved_turn_context::ResolvedKernelAdapters,
    pub embedding_exact_pin: Option<crate::resolved_turn_context::EmbeddingExactPin>,
    pub memory_state_roots: Option<crate::resolved_turn_context::MemoryStateRoots>,
    pub selected_execution_adapter: Option<crate::resolved_turn_context::SelectedExecutionAdapter>,
    pub allowed_capabilities: Vec<String>,
    pub rule_evaluations: Vec<crate::policy::RuleEvaluation>,
    pub final_decision: crate::policy::FinalDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingDeltaLedgerRecord {
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub added_slots: Vec<cyrune_core_contract::SlotId>,
    pub removed_slots: Vec<cyrune_core_contract::SlotId>,
    pub resulting_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenialLedgerRecord {
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub denial_id: cyrune_core_contract::DenialId,
    pub run_id: cyrune_core_contract::RunId,
    pub rule_id: cyrune_core_contract::RuleId,
    pub reason_kind: cyrune_core_contract::ReasonKind,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashesRecord {
    pub files: BTreeMap<String, String>,
    pub prev_evidence_id: Option<EvidenceId>,
    pub prev_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalBindingRecord {
    pub schema_version: String,
    pub outcome: EvidenceOutcome,
    pub response_to: cyrune_core_contract::RequestId,
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub run_id: cyrune_core_contract::RunId,
    pub evidence_id: EvidenceId,
    pub policy_pack_id: String,
    pub citation_bundle_id: cyrune_core_contract::CitationBundleId,
    pub working_hash_after: String,
    pub evidence_manifest_hash: String,
    pub evidence_hashes_hash: String,
    pub working_json_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedLedgerInput {
    pub request: RunRequest,
    pub context: ResolvedTurnContext,
    pub created_at: String,
    pub started_at: String,
    pub finished_at: String,
    pub exit_status: Option<i32>,
    pub working_hash_before: String,
    pub working_output: WorkingRebuildOutput,
    pub prior_working: Option<WorkingProjection>,
    pub query_summary: QuerySummary,
    pub bundle: CitationBundle,
    pub rr: SimpleReasoningRecord,
    pub policy_trace: PolicyTrace,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedLedgerInput {
    pub request: RunRequest,
    pub context: ResolvedTurnContext,
    pub created_at: String,
    pub started_at: String,
    pub finished_at: String,
    pub exit_status: Option<i32>,
    pub working_hash_before: String,
    pub query_summary: Option<QuerySummary>,
    pub failure: FailureSpec,
    pub policy_trace: PolicyTrace,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerCommitOutput {
    pub evidence_id: EvidenceId,
    pub manifest: LedgerManifest,
    pub evidence_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

pub struct LedgerWriter {
    cyrune_home: PathBuf,
    remaining_failures_before_rename: usize,
}

impl LedgerWriter {
    #[must_use]
    pub fn new(cyrune_home: impl Into<PathBuf>) -> Self {
        Self {
            cyrune_home: cyrune_home.into(),
            remaining_failures_before_rename: 0,
        }
    }

    #[must_use]
    pub fn with_failures(cyrune_home: impl Into<PathBuf>, failures_before_rename: usize) -> Self {
        Self {
            cyrune_home: cyrune_home.into(),
            remaining_failures_before_rename: failures_before_rename,
        }
    }

    pub fn commit_accepted(
        &mut self,
        input: &AcceptedLedgerInput,
    ) -> Result<LedgerCommitOutput, LedgerError> {
        let evidence_id = self.next_evidence_id()?;
        let (prev_evidence_id, prev_hash) = self.previous_hash_chain()?;
        let manifest = LedgerManifest {
            evidence_id: evidence_id.clone(),
            correlation_id: input.context.correlation_id.clone(),
            run_id: input.context.run_id.clone(),
            outcome: EvidenceOutcome::Accepted,
            created_at: input.created_at.clone(),
            policy_pack_id: input.context.policy_pack_id.clone(),
            working_hash_before: input.working_hash_before.clone(),
            working_hash_after: input.working_output.working_hash.clone(),
            citation_bundle_id: Some(input.bundle.bundle_id.clone()),
            rr_present: true,
        };
        let run = RunLedgerRecord {
            request_id: input.request.request_id.clone(),
            correlation_id: input.context.correlation_id.clone(),
            run_id: input.context.run_id.clone(),
            run_kind: input.request.run_kind.clone(),
            adapter_id: input.request.adapter_id.clone(),
            cwd: input
                .request
                .cwd
                .as_ref()
                .map(|cwd| cwd.as_str().to_string()),
            argv: input.request.argv.clone(),
            requested_capabilities: input.request.requested_capabilities.clone(),
            started_at: input.started_at.clone(),
            finished_at: input.finished_at.clone(),
            exit_status: input.exit_status,
            query_summary: Some(input.query_summary.clone()),
        };
        let policy = policy_record(&input.context, &input.policy_trace);
        let working_delta = working_delta_record(
            &input.context.correlation_id,
            input.prior_working.as_ref(),
            &input.working_output,
        );
        let files = vec![
            ("manifest.json", canonical_json_bytes(&manifest)?),
            ("run.json", canonical_json_bytes(&run)?),
            ("policy.json", canonical_json_bytes(&policy)?),
            (
                "citation_bundle.json",
                input.bundle.canonical_json_bytes().map_err(to_invalid)?,
            ),
            (
                "rr.json",
                input.rr.canonical_json_bytes().map_err(to_invalid)?,
            ),
            ("working_delta.json", canonical_json_bytes(&working_delta)?),
            ("stdout.log", canonical_log_bytes(&input.stdout)),
            ("stderr.log", canonical_log_bytes(&input.stderr)),
        ];
        self.commit_evidence(evidence_id, manifest, files, prev_evidence_id, prev_hash)
    }

    pub fn commit_rejected(
        &mut self,
        input: &RejectedLedgerInput,
    ) -> Result<LedgerCommitOutput, LedgerError> {
        let evidence_id = self.next_evidence_id()?;
        let (prev_evidence_id, prev_hash) = self.previous_hash_chain()?;
        let manifest = LedgerManifest {
            evidence_id: evidence_id.clone(),
            correlation_id: input.context.correlation_id.clone(),
            run_id: input.context.run_id.clone(),
            outcome: EvidenceOutcome::Rejected,
            created_at: input.created_at.clone(),
            policy_pack_id: input.context.policy_pack_id.clone(),
            working_hash_before: input.working_hash_before.clone(),
            working_hash_after: input.working_hash_before.clone(),
            citation_bundle_id: None,
            rr_present: false,
        };
        let run = RunLedgerRecord {
            request_id: input.request.request_id.clone(),
            correlation_id: input.context.correlation_id.clone(),
            run_id: input.context.run_id.clone(),
            run_kind: input.request.run_kind.clone(),
            adapter_id: input.request.adapter_id.clone(),
            cwd: input
                .request
                .cwd
                .as_ref()
                .map(|cwd| cwd.as_str().to_string()),
            argv: input.request.argv.clone(),
            requested_capabilities: input.request.requested_capabilities.clone(),
            started_at: input.started_at.clone(),
            finished_at: input.finished_at.clone(),
            exit_status: input.exit_status,
            query_summary: input.query_summary.clone(),
        };
        let policy = policy_record(&input.context, &input.policy_trace);
        let denial = DenialLedgerRecord {
            correlation_id: input.context.correlation_id.clone(),
            denial_id: cyrune_core_contract::DenialId::from_evidence_id(&evidence_id),
            run_id: input.context.run_id.clone(),
            rule_id: input.failure.rule_id.clone(),
            reason_kind: input.failure.reason_kind.clone(),
            message: input.failure.message.clone(),
            remediation: input.failure.remediation.clone(),
        };
        let files = vec![
            ("manifest.json", canonical_json_bytes(&manifest)?),
            ("run.json", canonical_json_bytes(&run)?),
            ("policy.json", canonical_json_bytes(&policy)?),
            ("denial.json", canonical_json_bytes(&denial)?),
        ];
        self.commit_evidence(evidence_id, manifest, files, prev_evidence_id, prev_hash)
    }

    #[must_use]
    pub fn cyrune_home(&self) -> &Path {
        &self.cyrune_home
    }

    fn commit_evidence(
        &mut self,
        evidence_id: EvidenceId,
        manifest: LedgerManifest,
        mut files: Vec<(&'static str, Vec<u8>)>,
        prev_evidence_id: Option<EvidenceId>,
        prev_hash: Option<String>,
    ) -> Result<LedgerCommitOutput, LedgerError> {
        self.ensure_root_layout()?;
        let evidence_root = self.cyrune_home.join("ledger").join("evidence");
        let finalized_dir = evidence_root.join(evidence_id.as_str());
        let tmp_dir = evidence_root.join(format!("{}.tmp", evidence_id.as_str()));
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        fs::create_dir_all(&tmp_dir)?;
        sync_dir(&evidence_root)?;

        let mut file_hashes = BTreeMap::new();
        for (file_name, bytes) in &files {
            write_bytes(&tmp_dir.join(file_name), bytes)?;
            file_hashes.insert(
                (*file_name).to_string(),
                format!("sha256:{}", sha256_hex(bytes)),
            );
        }

        let hashes = HashesRecord {
            files: file_hashes,
            prev_evidence_id,
            prev_hash,
        };
        let hashes_bytes = canonical_json_bytes(&hashes)?;
        write_bytes(&tmp_dir.join("hashes.json"), &hashes_bytes)?;
        files.push(("hashes.json", hashes_bytes));

        sync_dir(&tmp_dir)?;
        if self.remaining_failures_before_rename > 0 {
            self.remaining_failures_before_rename -= 1;
            return Err(LedgerError::Invalid(
                "simulated ledger commit failure before rename".to_string(),
            ));
        }
        fs::rename(&tmp_dir, &finalized_dir)?;
        sync_dir(&evidence_root)?;
        self.append_index(&manifest)?;

        Ok(LedgerCommitOutput {
            evidence_id,
            manifest,
            evidence_dir: finalized_dir,
        })
    }

    fn append_index(&self, manifest: &LedgerManifest) -> Result<(), LedgerError> {
        let manifests_dir = self.cyrune_home.join("ledger").join("manifests");
        fs::create_dir_all(&manifests_dir)?;
        let index_path = manifests_dir.join("index.jsonl");
        let mut line = canonical_json_bytes(manifest)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&index_path)?;
        file.write_all(&line)?;
        file.sync_all()?;
        sync_dir(&manifests_dir)?;
        line.clear();
        Ok(())
    }

    fn ensure_root_layout(&self) -> Result<(), LedgerError> {
        for path in [
            self.cyrune_home.clone(),
            self.cyrune_home.join("ledger"),
            self.cyrune_home.join("ledger").join("evidence"),
            self.cyrune_home.join("ledger").join("manifests"),
            self.cyrune_home.join("ledger").join("quarantine"),
            self.cyrune_home.join("working"),
        ] {
            fs::create_dir_all(&path)?;
            sync_dir(&path)?;
        }
        Ok(())
    }

    fn next_evidence_id(&self) -> Result<EvidenceId, LedgerError> {
        let evidence_root = self.cyrune_home.join("ledger").join("evidence");
        if !evidence_root.exists() {
            return Ok(EvidenceId::new(1));
        }
        let mut max_id = 0_u64;
        for entry in fs::read_dir(&evidence_root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".tmp") {
                continue;
            }
            if let Some(rest) = name.strip_prefix("EVID-") {
                let parsed = rest.parse::<u64>().map_err(|error| {
                    LedgerError::Invalid(format!("invalid evidence directory name {name}: {error}"))
                })?;
                max_id = max_id.max(parsed);
            }
        }
        Ok(EvidenceId::new(max_id + 1))
    }

    fn previous_hash_chain(&self) -> Result<(Option<EvidenceId>, Option<String>), LedgerError> {
        let evidence_root = self.cyrune_home.join("ledger").join("evidence");
        if !evidence_root.exists() {
            return Ok((None, None));
        }
        let mut finalized = Vec::new();
        for entry in fs::read_dir(&evidence_root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".tmp") {
                continue;
            }
            if let Some(rest) = name.strip_prefix("EVID-") {
                let parsed = rest.parse::<u64>().map_err(|error| {
                    LedgerError::Invalid(format!("invalid evidence directory name {name}: {error}"))
                })?;
                finalized.push((parsed, entry.path()));
            }
        }
        finalized.sort_by_key(|(id, _)| *id);
        let Some((last_id, last_path)) = finalized.last() else {
            return Ok((None, None));
        };
        let hashes_bytes = fs::read(last_path.join("hashes.json"))?;
        Ok((
            Some(EvidenceId::new(*last_id)),
            Some(format!("sha256:{}", sha256_hex(&hashes_bytes))),
        ))
    }
}

pub fn write_working_projection(
    cyrune_home: &Path,
    projection: &WorkingProjection,
) -> Result<(), LedgerError> {
    let working_dir = cyrune_home.join("working");
    fs::create_dir_all(&working_dir)?;
    sync_dir(&working_dir)?;
    let tmp_path = working_dir.join("working.json.tmp");
    let final_path = working_dir.join("working.json");
    let bytes = projection
        .canonical_json_bytes()
        .map_err(|error| LedgerError::Invalid(error.to_string()))?;
    write_bytes(&tmp_path, &bytes)?;
    fs::rename(&tmp_path, &final_path)?;
    sync_dir(&working_dir)?;
    Ok(())
}

#[must_use]
pub fn terminal_binding_path(cyrune_home: &Path, evidence_id: &EvidenceId) -> PathBuf {
    cyrune_home
        .join("ledger")
        .join("terminal-bindings")
        .join(format!("{}.json", evidence_id.as_str()))
}

pub fn write_terminal_binding(
    cyrune_home: &Path,
    record: &TerminalBindingRecord,
) -> Result<PathBuf, LedgerError> {
    let binding_dir = cyrune_home.join("ledger").join("terminal-bindings");
    fs::create_dir_all(&binding_dir)?;
    sync_dir(&binding_dir)?;
    let final_path = terminal_binding_path(cyrune_home, &record.evidence_id);
    let tmp_path = binding_dir.join(format!("{}.json.tmp", record.evidence_id.as_str()));
    let bytes = canonical_json_bytes(record)?;
    write_bytes(&tmp_path, &bytes)?;
    fs::rename(&tmp_path, &final_path)?;
    let _ = sync_dir(&binding_dir);
    Ok(final_path)
}

pub fn raw_file_sha256(path: &Path) -> Result<String, LedgerError> {
    let bytes = fs::read(path)?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

pub fn visible_working_hash(cyrune_home: &Path) -> Result<String, LedgerError> {
    raw_file_sha256(&cyrune_home.join("working").join("working.json"))
}

fn policy_record(context: &ResolvedTurnContext, trace: &PolicyTrace) -> PolicyLedgerRecord {
    PolicyLedgerRecord {
        requested_policy_pack_id: context.requested_policy_pack_id.clone(),
        requested_binding_id: context.requested_binding_id.clone(),
        policy_pack_id: context.policy_pack_id.clone(),
        binding_id: context.binding_id.clone(),
        resolved_kernel_adapters: context.resolved_kernel_adapters.clone(),
        embedding_exact_pin: context.embedding_exact_pin.clone(),
        memory_state_roots: context.memory_state_roots.clone(),
        selected_execution_adapter: context.selected_execution_adapter.clone(),
        allowed_capabilities: context.allowed_capabilities.clone(),
        rule_evaluations: trace.rule_evaluations.clone(),
        final_decision: trace.final_decision,
    }
}

fn working_delta_record(
    correlation_id: &cyrune_core_contract::CorrelationId,
    prior_working: Option<&WorkingProjection>,
    working_output: &WorkingRebuildOutput,
) -> WorkingDeltaLedgerRecord {
    let previous_ids = prior_working
        .map(|projection| {
            projection
                .slots
                .iter()
                .map(|slot| slot.slot_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let current_ids = working_output
        .projection
        .slots
        .iter()
        .map(|slot| slot.slot_id.clone())
        .collect::<Vec<_>>();
    let current_id_set = current_ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let removed_slots = previous_ids
        .into_iter()
        .filter(|slot_id| !current_id_set.contains(slot_id))
        .collect();
    WorkingDeltaLedgerRecord {
        correlation_id: correlation_id.clone(),
        added_slots: working_output.working_delta.new_slot_ids.clone(),
        removed_slots,
        resulting_hash: working_output.working_hash.clone(),
    }
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, LedgerError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_log_bytes(value: &str) -> Vec<u8> {
    let mut text = value.replace("\r\n", "\n").replace('\r', "\n");
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.into_bytes()
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), LedgerError> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), LedgerError> {
    let dir = File::open(path)?;
    dir.sync_all()?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn to_invalid(error: impl std::fmt::Display) -> LedgerError {
    LedgerError::Invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{AcceptedLedgerInput, LedgerWriter, RejectedLedgerInput, write_working_projection};
    use crate::citation::{
        CitationBundle, CitationClaim, ClaimKind, EvidenceRef, SimpleReasoningRecord,
    };
    use crate::policy::{FailureSpec, PolicyTrace};
    use crate::resolved_turn_context::{
        ResolvedKernelAdapters, ResolvedTurnContext, TimeoutPolicy,
    };
    use crate::retrieval::QuerySummary;
    use crate::working::{
        WorkingCandidate, WorkingCandidateCategory, WorkingRebuildInput, WorkingSlotKind,
        rebuild_working,
    };
    use cyrune_core_contract::{CorrelationId, IoMode, RequestId, RuleId, RunKind, RunRequest};
    use serde_json::Value;
    use tempfile::tempdir;

    fn request() -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0401").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0401").unwrap(),
            run_kind: RunKind::NoLlm,
            user_input: "ledger".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: None,
            argv: None,
            cwd: None,
            env_overrides: None,
        }
    }

    fn context() -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0401").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0401").unwrap(),
            run_id: cyrune_core_contract::RunId::parse("RUN-20260327-0401-R01").unwrap(),
            requested_policy_pack_id: "cyrune-free-default".to_string(),
            requested_binding_id: None,
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: "cyrune-free-default".to_string(),
            resolved_kernel_adapters: ResolvedKernelAdapters {
                working_store_adapter_id: "memory-kv-inmem".to_string(),
                processing_store_adapter_id: "memory-kv-inmem".to_string(),
                permanent_store_adapter_id: "memory-kv-inmem".to_string(),
                vector_index_adapter_id: "memory-kv-inmem".to_string(),
                embedding_engine_ref: "crane-embed-null.v0.1".to_string(),
            },
            embedding_exact_pin: None,
            memory_state_roots: None,
            allowed_capabilities: vec!["fs_read".to_string()],
            sandbox_ref: "SANDBOX_MINIMAL_CANONICAL.md#default-profile".to_string(),
            run_kind: RunKind::NoLlm,
            io_mode: IoMode::Captured,
            selected_execution_adapter: None,
            timeout_policy: TimeoutPolicy {
                turn_timeout_s: 120,
                execution_timeout_s: 120,
            },
        }
    }

    fn working_output() -> crate::working::WorkingRebuildOutput {
        rebuild_working(&WorkingRebuildInput {
            generated_at: "2026-03-27T12:00:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0401").unwrap(),
            prior_working: None,
            candidates: vec![WorkingCandidate {
                category: WorkingCandidateCategory::PolicyConstraint,
                kind: WorkingSlotKind::Constraint,
                text: "keep deterministic".to_string(),
                source_evidence_id: "EVID-1".to_string(),
                source_layer: crate::memory::SourceLayer::Processing,
                updated_at: "2026-03-27T12:00:00+09:00".to_string(),
                updated_at_unix_ms: 1,
            }],
        })
        .unwrap()
    }

    #[test]
    fn commit_accepted_writes_required_files() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let output = working_output();
        let committed = writer
            .commit_accepted(&AcceptedLedgerInput {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:02+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:02+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                working_output: output.clone(),
                prior_working: None,
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                bundle: CitationBundle {
                    bundle_id: cyrune_core_contract::CitationBundleId::from_correlation_id(
                        &CorrelationId::parse("RUN-20260327-0401").unwrap(),
                    ),
                    correlation_id: CorrelationId::parse("RUN-20260327-0401").unwrap(),
                    claims: vec![CitationClaim {
                        claim_id: cyrune_core_contract::ClaimId::parse("CLM-001").unwrap(),
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
                policy_trace: PolicyTrace::new(),
                stdout: "stdout".to_string(),
                stderr: String::new(),
            })
            .unwrap();
        assert!(committed.evidence_dir.join("manifest.json").exists());
        assert!(committed.evidence_dir.join("citation_bundle.json").exists());
        assert!(committed.evidence_dir.join("rr.json").exists());
        assert!(committed.evidence_dir.join("working_delta.json").exists());
        assert!(committed.evidence_dir.join("stdout.log").exists());
        assert!(committed.evidence_dir.join("stderr.log").exists());
        let policy_json =
            std::fs::read_to_string(committed.evidence_dir.join("policy.json")).unwrap();
        let policy_value: Value = serde_json::from_str(&policy_json).unwrap();
        assert_eq!(
            policy_value["requested_policy_pack_id"],
            "cyrune-free-default"
        );
        assert!(policy_value["requested_binding_id"].is_null());
        assert_eq!(policy_value["binding_id"], "cyrune-free-default");
        assert_eq!(
            policy_value["resolved_kernel_adapters"]["processing_store_adapter_id"],
            "memory-kv-inmem"
        );
        assert!(policy_value["memory_state_roots"].is_null());
        println!("ledger_policy_json={policy_json}");
    }

    #[test]
    fn write_working_projection_persists_projection() {
        let temp = tempdir().unwrap();
        let output = working_output();
        write_working_projection(temp.path(), &output.projection).unwrap();
        assert!(temp.path().join("working").join("working.json").exists());
    }

    #[test]
    fn commit_rejected_writes_denial() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let committed = writer
            .commit_rejected(&RejectedLedgerInput {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:02+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:02+09:00".to_string(),
                exit_status: None,
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                query_summary: None,
                failure: FailureSpec::citation_denied(
                    RuleId::parse("CIT-001").unwrap(),
                    "citation failed",
                    "fix citation",
                )
                .unwrap(),
                policy_trace: PolicyTrace::new(),
                stdout: String::new(),
                stderr: String::new(),
            })
            .unwrap();
        assert!(committed.evidence_dir.join("denial.json").exists());
    }
}
