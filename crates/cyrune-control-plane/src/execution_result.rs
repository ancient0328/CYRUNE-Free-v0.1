#![forbid(unsafe_code)]

use crate::citation::{CitationMaterial, SimpleReasoningRecord};
use crate::resolved_turn_context::{ResolvedTurnContext, SelectedExecutionAdapter};
use cyrune_core_contract::{CorrelationId, RunKind, RunRequest};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalStatus {
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdioCapture {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPin {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResultEnvelope {
    pub adapter_id: String,
    pub adapter_version: String,
    pub correlation_id: CorrelationId,
    pub terminal_status: TerminalStatus,
    pub started_at: String,
    pub finished_at: String,
    pub exit_status: Option<i32>,
    pub output_draft: Option<String>,
    pub stdio: StdioCapture,
    pub pin: ExecutionPin,
    pub citation_material: Option<CitationMaterial>,
    pub rr_material: Option<SimpleReasoningRecord>,
    pub failure_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoLlmAcceptedDraft {
    pub started_at: String,
    pub finished_at: String,
    pub output_draft: String,
    pub stdio: StdioCapture,
    pub citation_material: CitationMaterial,
    pub rr_material: SimpleReasoningRecord,
}

#[derive(Debug, Error)]
pub enum ExecutionResultError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Serialize)]
struct LauncherRequestPayload<'a> {
    request_id: &'a cyrune_core_contract::RequestId,
    correlation_id: &'a CorrelationId,
    run_id: &'a cyrune_core_contract::RunId,
    user_input: &'a str,
    policy_pack_id: &'a str,
    requested_capabilities: &'a [String],
    io_mode: &'a cyrune_core_contract::IoMode,
    argv: &'a [String],
    cwd: Option<&'a str>,
    env_overrides: &'a BTreeMap<String, String>,
    adapter_id: &'a str,
    model_id: &'a str,
    model_revision_or_digest: &'a str,
    launcher_sha256: &'a str,
    execution_timeout_s: u64,
}

impl ExecutionResultEnvelope {
    pub fn validate(&self) -> Result<(), ExecutionResultError> {
        if self.adapter_id.trim().is_empty()
            || self.adapter_version.trim().is_empty()
            || self.started_at.trim().is_empty()
            || self.finished_at.trim().is_empty()
            || self.pin.kind.trim().is_empty()
            || self.pin.value.trim().is_empty()
        {
            return Err(ExecutionResultError::Invalid(
                "execution result envelope is missing required fields".to_string(),
            ));
        }
        match self.terminal_status {
            TerminalStatus::Succeeded => {
                if self
                    .output_draft
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                    || self.citation_material.is_none()
                    || self.rr_material.is_none()
                {
                    return Err(ExecutionResultError::Invalid(
                        "succeeded envelope requires output_draft, citation_material, and rr_material"
                            .to_string(),
                    ));
                }
                if self.failure_detail.is_some() {
                    return Err(ExecutionResultError::Invalid(
                        "succeeded envelope must not contain failure_detail".to_string(),
                    ));
                }
            }
            TerminalStatus::Failed | TerminalStatus::TimedOut | TerminalStatus::Cancelled => {
                if self
                    .failure_detail
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(ExecutionResultError::Invalid(
                        "failed/timed_out/cancelled envelope requires failure_detail".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn validate_against_selected(
        &self,
        correlation_id: &CorrelationId,
        selected: &SelectedExecutionAdapter,
    ) -> Result<(), ExecutionResultError> {
        self.validate()?;
        if &self.correlation_id != correlation_id {
            return Err(ExecutionResultError::Invalid(
                "execution result envelope correlation_id mismatch".to_string(),
            ));
        }
        if self.adapter_id != selected.adapter_id
            || self.adapter_version != selected.adapter_version
        {
            return Err(ExecutionResultError::Invalid(
                "execution result envelope adapter identity mismatch".to_string(),
            ));
        }
        if self.pin.kind != "launcher_sha256" || self.pin.value != selected.launcher_sha256 {
            return Err(ExecutionResultError::Invalid(
                "execution result envelope pin does not match selected execution adapter"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

impl NoLlmAcceptedDraft {
    pub fn validate(&self) -> Result<(), ExecutionResultError> {
        if self.started_at.trim().is_empty()
            || self.finished_at.trim().is_empty()
            || self.output_draft.trim().is_empty()
        {
            return Err(ExecutionResultError::Invalid(
                "no-llm accepted draft requires started_at, finished_at, and output_draft"
                    .to_string(),
            ));
        }
        if self.citation_material.claims.is_empty() || self.rr_material.claims.is_empty() {
            return Err(ExecutionResultError::Invalid(
                "no-llm accepted draft requires citation_material and rr_material".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn execute_local_cli_single_process(
    context: &ResolvedTurnContext,
    request: &RunRequest,
    launcher_path: &Path,
) -> ExecutionResultEnvelope {
    let started_at = timestamp_now_rfc3339();
    if request.run_kind != RunKind::ExecutionAdapter {
        return failure_envelope(
            context,
            TerminalStatus::Failed,
            started_at.clone(),
            started_at,
            None,
            StdioCapture {
                stdout: String::new(),
                stderr: String::new(),
            },
            "execution adapter launcher requires run_kind=execution_adapter",
        );
    }
    let selected = context.selected_execution_adapter.as_ref().ok_or_else(|| {
        ExecutionResultError::Invalid(
            "selected execution adapter is required for process_stdio launch".to_string(),
        )
    });
    let selected = match selected {
        Ok(selected) => selected,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                started_at,
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                error.to_string(),
            );
        }
    };
    if selected.execution_kind != "process_stdio" {
        return failure_envelope(
            context,
            TerminalStatus::Failed,
            started_at.clone(),
            started_at,
            None,
            StdioCapture {
                stdout: String::new(),
                stderr: String::new(),
            },
            format!("unsupported execution_kind: {}", selected.execution_kind),
        );
    }

    let normalized_cwd = match normalize_optional_cwd(request.cwd.as_ref().map(|cwd| cwd.as_str()))
    {
        Ok(cwd) => cwd,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                timestamp_now_rfc3339(),
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                error.to_string(),
            );
        }
    };
    let env_overrides = match normalize_env_overrides(request.env_overrides.as_ref(), selected) {
        Ok(env_overrides) => env_overrides,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                timestamp_now_rfc3339(),
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                error.to_string(),
            );
        }
    };
    let argv = request.argv.clone().unwrap_or_default();
    let payload = LauncherRequestPayload {
        request_id: &request.request_id,
        correlation_id: &context.correlation_id,
        run_id: &context.run_id,
        user_input: &request.user_input,
        policy_pack_id: &context.policy_pack_id,
        requested_capabilities: &request.requested_capabilities,
        io_mode: &request.io_mode,
        argv: &argv,
        cwd: request.cwd.as_ref().map(|cwd| cwd.as_str()),
        env_overrides: &env_overrides,
        adapter_id: &selected.adapter_id,
        model_id: &selected.model_id,
        model_revision_or_digest: &selected.model_revision_or_digest,
        launcher_sha256: &selected.launcher_sha256,
        execution_timeout_s: context.timeout_policy.execution_timeout_s,
    };

    let mut command = Command::new(launcher_path);
    command.args(&argv);
    if let Some(cwd) = &normalized_cwd {
        command.current_dir(cwd);
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env_clear();
    for (key, value) in &env_overrides {
        command.env(key, value);
    }
    command.env("CYRUNE_REQUEST_ID", request.request_id.as_str());
    command.env("CYRUNE_CORRELATION_ID", context.correlation_id.as_str());
    command.env("CYRUNE_RUN_ID", context.run_id.as_str());
    command.env("CYRUNE_LAUNCHER_SHA256", &selected.launcher_sha256);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                timestamp_now_rfc3339(),
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                format!("approved execution adapter launch failed: {error}"),
            );
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let payload_bytes = match serde_json::to_vec(&payload) {
            Ok(payload_bytes) => payload_bytes,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return failure_envelope(
                    context,
                    TerminalStatus::Failed,
                    started_at.clone(),
                    timestamp_now_rfc3339(),
                    None,
                    StdioCapture {
                        stdout: String::new(),
                        stderr: String::new(),
                    },
                    format!(
                        "run request cannot be serialized for approved execution adapter: {error}"
                    ),
                );
            }
        };
        let write_result = stdin
            .write_all(&payload_bytes)
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush());
        if let Err(error) = write_result {
            let _ = child.kill();
            let _ = child.wait();
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                timestamp_now_rfc3339(),
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                format!("approved execution adapter stdin transport failed: {error}"),
            );
        }
        drop(stdin);
    }

    let output = match collect_process_output(child, context.timeout_policy.execution_timeout_s) {
        Ok(output) => output,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at.clone(),
                timestamp_now_rfc3339(),
                None,
                StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                format!("approved execution adapter process collection failed: {error}"),
            );
        }
    };
    if output.timed_out {
        return failure_envelope(
            context,
            TerminalStatus::TimedOut,
            started_at,
            output.finished_at,
            output.exit_status,
            output.stdio,
            "approved execution adapter timed out",
        );
    }

    let raw_stdout = output.stdio.stdout.clone();
    let raw_stderr = output.stdio.stderr.clone();
    let mut envelope = match serde_json::from_str::<ExecutionResultEnvelope>(&raw_stdout) {
        Ok(envelope) => envelope,
        Err(error) => {
            return failure_envelope(
                context,
                TerminalStatus::Failed,
                started_at,
                output.finished_at,
                output.exit_status,
                output.stdio,
                format!(
                    "launcher stdout did not decode as execution result envelope: {error}; stderr={raw_stderr}"
                ),
            );
        }
    };
    if envelope.exit_status != output.exit_status {
        return failure_envelope(
            context,
            TerminalStatus::Failed,
            envelope.started_at.clone(),
            envelope.finished_at.clone(),
            output.exit_status,
            envelope.stdio.clone(),
            "execution result envelope exit_status does not match launcher exit status",
        );
    }
    if envelope.stdio.stderr.is_empty() && !raw_stderr.is_empty() {
        envelope.stdio.stderr = raw_stderr;
    }
    if let Err(error) = envelope.validate_against_selected(&context.correlation_id, selected) {
        return failure_envelope(
            context,
            TerminalStatus::Failed,
            envelope.started_at.clone(),
            envelope.finished_at.clone(),
            envelope.exit_status,
            envelope.stdio.clone(),
            format!("approved execution adapter envelope validation failed: {error}"),
        );
    }
    if envelope.terminal_status == TerminalStatus::Succeeded && envelope.exit_status != Some(0) {
        return failure_envelope(
            context,
            TerminalStatus::Failed,
            envelope.started_at.clone(),
            envelope.finished_at.clone(),
            envelope.exit_status,
            envelope.stdio.clone(),
            "approved execution adapter returned succeeded with non-zero exit_status",
        );
    }
    envelope
}

fn normalize_optional_cwd(raw: Option<&str>) -> Result<Option<PathBuf>, ExecutionResultError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let path = Path::new(raw);
    if !path.is_absolute() {
        return Err(ExecutionResultError::Invalid(
            "execution adapter cwd must be absolute when provided".to_string(),
        ));
    }
    Ok(Some(std::fs::canonicalize(path)?))
}

fn normalize_env_overrides(
    overrides: Option<&BTreeMap<String, String>>,
    selected: &SelectedExecutionAdapter,
) -> Result<BTreeMap<String, String>, ExecutionResultError> {
    let mut normalized = BTreeMap::new();
    for key in &selected.env_allowlist {
        validate_env_key(key)?;
        if let Some(value) = overrides.and_then(|entries| entries.get(key)) {
            normalized.insert(key.clone(), value.clone());
        } else if let Ok(value) = std::env::var(key) {
            normalized.insert(key.clone(), value);
        }
    }
    if let Some(entries) = overrides {
        for key in entries.keys() {
            validate_env_key(key)?;
            if !selected.env_allowlist.iter().any(|allowed| allowed == key) {
                return Err(ExecutionResultError::Invalid(format!(
                    "env override key is not allowlisted for selected execution adapter: {key}"
                )));
            }
        }
    }
    Ok(normalized)
}

fn validate_env_key(key: &str) -> Result<(), ExecutionResultError> {
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(ExecutionResultError::Invalid(format!(
            "env key is outside closed set: {key}"
        )));
    }
    Ok(())
}

#[derive(Debug)]
struct CollectedProcessOutput {
    exit_status: Option<i32>,
    stdio: StdioCapture,
    finished_at: String,
    timed_out: bool,
}

fn collect_process_output(
    mut child: Child,
    timeout_s: u64,
) -> Result<CollectedProcessOutput, ExecutionResultError> {
    let timeout = Duration::from_secs(timeout_s);
    let poll_interval = Duration::from_millis(10);
    let started = Instant::now();
    let stdout_reader = spawn_pipe_reader(child.stdout.take());
    let stderr_reader = spawn_pipe_reader(child.stderr.take());
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(CollectedProcessOutput {
                exit_status: status.code(),
                stdio: StdioCapture {
                    stdout: join_pipe_reader(stdout_reader)?,
                    stderr: join_pipe_reader(stderr_reader)?,
                },
                finished_at: timestamp_now_rfc3339(),
                timed_out: false,
            });
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let status = child.wait()?;
            return Ok(CollectedProcessOutput {
                exit_status: status.code(),
                stdio: StdioCapture {
                    stdout: join_pipe_reader(stdout_reader)?,
                    stderr: join_pipe_reader(stderr_reader)?,
                },
                finished_at: timestamp_now_rfc3339(),
                timed_out: true,
            });
        }
        thread::sleep(poll_interval);
    }
}

fn spawn_pipe_reader<R>(pipe: Option<R>) -> JoinHandle<Result<String, ExecutionResultError>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || read_pipe(pipe))
}

fn join_pipe_reader(
    handle: JoinHandle<Result<String, ExecutionResultError>>,
) -> Result<String, ExecutionResultError> {
    handle.join().map_err(|_| {
        ExecutionResultError::Invalid("execution adapter pipe reader panicked".to_string())
    })?
}

fn read_pipe<R: Read>(pipe: Option<R>) -> Result<String, ExecutionResultError> {
    let Some(mut pipe) = pipe else {
        return Ok(String::new());
    };
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn failure_envelope(
    context: &ResolvedTurnContext,
    terminal_status: TerminalStatus,
    started_at: String,
    finished_at: String,
    exit_status: Option<i32>,
    stdio: StdioCapture,
    failure_detail: impl Into<String>,
) -> ExecutionResultEnvelope {
    let selected = context
        .selected_execution_adapter
        .as_ref()
        .expect("execution adapter failure envelope requires selected adapter");
    ExecutionResultEnvelope {
        adapter_id: selected.adapter_id.clone(),
        adapter_version: selected.adapter_version.clone(),
        correlation_id: context.correlation_id.clone(),
        terminal_status,
        started_at,
        finished_at,
        exit_status,
        output_draft: None,
        stdio,
        pin: ExecutionPin {
            kind: "launcher_sha256".to_string(),
            value: selected.launcher_sha256.clone(),
        },
        citation_material: None,
        rr_material: None,
        failure_detail: Some(failure_detail.into()),
    }
}

fn timestamp_now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionPin, ExecutionResultEnvelope, NoLlmAcceptedDraft, StdioCapture, TerminalStatus,
        execute_local_cli_single_process,
    };
    use crate::citation::{
        CitationMaterial, CitationMaterialClaim, ClaimKind, EvidenceRef, SimpleReasoningRecord,
    };
    use crate::resolved_turn_context::{
        ResolvedKernelAdapters, ResolvedTurnContext, SelectedExecutionAdapter, TimeoutPolicy,
    };
    use cyrune_core_contract::{
        CorrelationId, IoMode, PathLabel, RequestId, RunId, RunKind, RunRequest,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn succeeded_requires_citation_and_rr() {
        let envelope = ExecutionResultEnvelope {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0301").unwrap(),
            terminal_status: TerminalStatus::Succeeded,
            started_at: "2026-03-27T12:00:00+09:00".to_string(),
            finished_at: "2026-03-27T12:00:01+09:00".to_string(),
            exit_status: Some(0),
            output_draft: Some("ok".to_string()),
            stdio: StdioCapture {
                stdout: String::new(),
                stderr: String::new(),
            },
            pin: ExecutionPin {
                kind: "launcher_sha256".to_string(),
                value: "sha256:abc".to_string(),
            },
            citation_material: None,
            rr_material: None,
            failure_detail: None,
        };
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn succeeded_with_required_material_is_valid() {
        let envelope = ExecutionResultEnvelope {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0302").unwrap(),
            terminal_status: TerminalStatus::Succeeded,
            started_at: "2026-03-27T12:00:00+09:00".to_string(),
            finished_at: "2026-03-27T12:00:01+09:00".to_string(),
            exit_status: Some(0),
            output_draft: Some("- claim".to_string()),
            stdio: StdioCapture {
                stdout: "stdout".to_string(),
                stderr: String::new(),
            },
            pin: ExecutionPin {
                kind: "launcher_sha256".to_string(),
                value: "sha256:abc".to_string(),
            },
            citation_material: Some(CitationMaterial {
                claims: vec![CitationMaterialClaim {
                    text: "claim".to_string(),
                    claim_kind: ClaimKind::Extractive,
                    evidence_refs: vec![EvidenceRef {
                        evidence_id: "EVID-1".to_string(),
                    }],
                }],
            }),
            rr_material: Some(SimpleReasoningRecord {
                claims: vec!["claim".to_string()],
                decisions: Vec::new(),
                assumptions: Vec::new(),
                actions: Vec::new(),
                citations_used: vec!["EVID-1".to_string()],
            }),
            failure_detail: None,
        };
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn no_llm_draft_requires_citation_and_rr() {
        let draft = NoLlmAcceptedDraft {
            started_at: "2026-03-27T12:00:00+09:00".to_string(),
            finished_at: "2026-03-27T12:00:01+09:00".to_string(),
            output_draft: "- claim".to_string(),
            stdio: StdioCapture {
                stdout: String::new(),
                stderr: String::new(),
            },
            citation_material: CitationMaterial { claims: Vec::new() },
            rr_material: SimpleReasoningRecord {
                claims: Vec::new(),
                decisions: Vec::new(),
                assumptions: Vec::new(),
                actions: Vec::new(),
                citations_used: Vec::new(),
            },
        };
        assert!(draft.validate().is_err());
    }

    #[test]
    fn envelope_must_match_selected_execution_adapter_pin() {
        let envelope = ExecutionResultEnvelope {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0303").unwrap(),
            terminal_status: TerminalStatus::Succeeded,
            started_at: "2026-03-27T12:00:00+09:00".to_string(),
            finished_at: "2026-03-27T12:00:01+09:00".to_string(),
            exit_status: Some(0),
            output_draft: Some("- claim".to_string()),
            stdio: StdioCapture {
                stdout: "stdout".to_string(),
                stderr: String::new(),
            },
            pin: ExecutionPin {
                kind: "launcher_sha256".to_string(),
                value: "sha256:wrong".to_string(),
            },
            citation_material: Some(CitationMaterial {
                claims: vec![CitationMaterialClaim {
                    text: "claim".to_string(),
                    claim_kind: ClaimKind::Extractive,
                    evidence_refs: vec![EvidenceRef {
                        evidence_id: "EVID-1".to_string(),
                    }],
                }],
            }),
            rr_material: Some(SimpleReasoningRecord {
                claims: vec!["claim".to_string()],
                decisions: Vec::new(),
                assumptions: Vec::new(),
                actions: Vec::new(),
                citations_used: vec!["EVID-1".to_string()],
            }),
            failure_detail: None,
        };
        let selected = SelectedExecutionAdapter {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            execution_kind: "process_stdio".to_string(),
            launcher_path: "/bin/sh".to_string(),
            launcher_sha256: "sha256:expected".to_string(),
            model_id: "model.local".to_string(),
            model_revision_or_digest: "sha256:model".to_string(),
            default_timeout_s: 120,
            allowed_capabilities: vec!["exec".to_string()],
            env_allowlist: Vec::new(),
        };
        assert!(
            envelope
                .validate_against_selected(
                    &CorrelationId::parse("RUN-20260327-0303").unwrap(),
                    &selected
                )
                .is_err()
        );
    }

    #[test]
    #[cfg(unix)]
    fn local_cli_single_process_returns_execution_envelope() {
        let temp = tempdir().unwrap();
        let launcher_path = temp.path().join("launcher.sh");
        fs::write(
            &launcher_path,
            r#"#!/bin/sh
cat >/dev/null
cat <<'JSON'
{"adapter_id":"local-cli-single-process.v0.1","adapter_version":"0.1.0","correlation_id":"RUN-20260327-0304","terminal_status":"succeeded","started_at":"2026-03-27T12:00:00+09:00","finished_at":"2026-03-27T12:00:01+09:00","exit_status":0,"output_draft":"- claim","stdio":{"stdout":"adapter stdout","stderr":""},"pin":{"kind":"launcher_sha256","value":"sha256:expected"},"citation_material":{"claims":[{"text":"claim","claim_kind":"extractive","evidence_refs":[{"evidence_id":"EVID-1"}]}]},"rr_material":{"claims":["claim"],"decisions":[],"assumptions":[],"actions":[],"citations_used":["EVID-1"]},"failure_detail":null}
JSON
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&launcher_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&launcher_path, permissions).unwrap();

        let context = ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0304").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0304").unwrap(),
            run_id: RunId::parse("RUN-20260327-0304-R01").unwrap(),
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
            allowed_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            sandbox_ref: "SANDBOX_MINIMAL_CANONICAL.md#default-profile".to_string(),
            run_kind: RunKind::ExecutionAdapter,
            io_mode: IoMode::Captured,
            selected_execution_adapter: Some(SelectedExecutionAdapter {
                adapter_id: "local-cli-single-process.v0.1".to_string(),
                adapter_version: "0.1.0".to_string(),
                execution_kind: "process_stdio".to_string(),
                launcher_path: launcher_path.display().to_string(),
                launcher_sha256: "sha256:expected".to_string(),
                model_id: "model.local".to_string(),
                model_revision_or_digest: "sha256:model".to_string(),
                default_timeout_s: 120,
                allowed_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
                env_allowlist: vec!["SAFE_VAR".to_string()],
            }),
            timeout_policy: TimeoutPolicy {
                turn_timeout_s: 120,
                execution_timeout_s: 120,
            },
        };
        let request = RunRequest {
            request_id: RequestId::parse("REQ-20260327-0304").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0304").unwrap(),
            run_kind: RunKind::ExecutionAdapter,
            user_input: "execute".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: Some("local-cli-single-process.v0.1".to_string()),
            argv: Some(vec!["--once".to_string()]),
            cwd: Some(PathLabel::parse(temp.path().display().to_string()).unwrap()),
            env_overrides: Some(BTreeMap::from([("SAFE_VAR".to_string(), "1".to_string())])),
        };

        let envelope = execute_local_cli_single_process(&context, &request, &launcher_path);
        assert_eq!(envelope.adapter_id, "local-cli-single-process.v0.1");
        assert_eq!(envelope.stdio.stdout, "adapter stdout");
        assert!(
            envelope
                .validate_against_selected(
                    &context.correlation_id,
                    context.selected_execution_adapter.as_ref().unwrap()
                )
                .is_ok()
        );
    }
}
