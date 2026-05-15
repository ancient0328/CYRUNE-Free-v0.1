#![forbid(unsafe_code)]

use crate::resolved_turn_context::ResolvedTurnContext;
use crate::working::WorkingProjection;
use cyrune_core_contract::{ReasonKind, RuleId, RunRequest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureStage {
    RequestValidation,
    PolicyGate,
    BindingResolution,
    WorkingValidation,
    Execution,
    CitationValidation,
    LedgerCommit,
    WorkingUpdate,
    InternalInvariant,
}

impl FailureStage {
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::RequestValidation => 1,
            Self::PolicyGate => 2,
            Self::BindingResolution => 3,
            Self::WorkingValidation => 4,
            Self::Execution => 5,
            Self::CitationValidation => 6,
            Self::LedgerCommit => 7,
            Self::WorkingUpdate => 8,
            Self::InternalInvariant => 9,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureSpec {
    pub stage: FailureStage,
    pub rule_id: RuleId,
    pub reason_kind: ReasonKind,
    pub message: String,
    pub remediation: String,
}

impl FailureSpec {
    pub fn new(
        stage: FailureStage,
        rule_id: RuleId,
        reason_kind: ReasonKind,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        validate_rule_reason_pair(&rule_id, &reason_kind)?;
        let message = message.into();
        let remediation = remediation.into();
        if message.trim().is_empty() || remediation.trim().is_empty() {
            return Err(PolicyError::Invalid(
                "failure spec requires non-empty message/remediation".to_string(),
            ));
        }
        Ok(Self {
            stage,
            rule_id,
            reason_kind,
            message,
            remediation,
        })
    }

    pub fn policy_denied(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::PolicyGate,
            rule_id,
            ReasonKind::PolicyDenied,
            message,
            remediation,
        )
    }

    pub fn binding_unresolved(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::BindingResolution,
            rule_id,
            ReasonKind::BindingUnresolved,
            message,
            remediation,
        )
    }

    pub fn working_invalid(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::WorkingValidation,
            rule_id,
            ReasonKind::WorkingInvalid,
            message,
            remediation,
        )
    }

    pub fn citation_denied(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::CitationValidation,
            rule_id,
            ReasonKind::CitationDenied,
            message,
            remediation,
        )
    }

    pub fn execution_failed(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::Execution,
            rule_id,
            ReasonKind::ExecutionFailed,
            message,
            remediation,
        )
    }

    pub fn ledger_commit_failed(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::LedgerCommit,
            rule_id,
            ReasonKind::LedgerCommitFailed,
            message,
            remediation,
        )
    }

    pub fn working_update_failed(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::WorkingUpdate,
            rule_id,
            ReasonKind::WorkingUpdateFailed,
            message,
            remediation,
        )
    }

    pub fn internal_error(
        rule_id: RuleId,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        Self::new(
            FailureStage::InternalInvariant,
            rule_id,
            ReasonKind::InternalError,
            message,
            remediation,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleDecision {
    Passed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleEvaluation {
    pub rule_id: RuleId,
    pub decision: RuleDecision,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalDecision {
    Allowed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyTrace {
    pub rule_evaluations: Vec<RuleEvaluation>,
    pub final_decision: FinalDecision,
}

impl PolicyTrace {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rule_evaluations: Vec::new(),
            final_decision: FinalDecision::Allowed,
        }
    }

    pub fn record_pass(
        &mut self,
        rule_id: RuleId,
        detail: impl Into<String>,
    ) -> Result<(), PolicyError> {
        self.rule_evaluations.push(RuleEvaluation {
            rule_id,
            decision: RuleDecision::Passed,
            detail: detail.into(),
        });
        Ok(())
    }

    pub fn record_failure(&mut self, failure: &FailureSpec) {
        self.rule_evaluations.push(RuleEvaluation {
            rule_id: failure.rule_id.clone(),
            decision: RuleDecision::Rejected,
            detail: failure.message.clone(),
        });
        self.final_decision = FinalDecision::Rejected;
    }
}

impl Default for PolicyTrace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecheckOutcome {
    pub policy_trace: PolicyTrace,
    pub terminal_failure: Option<FailureSpec>,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error(transparent)]
    Contract(#[from] cyrune_core_contract::ContractError),
    #[error("{0}")]
    Invalid(String),
}

pub fn evaluate_precheck(
    request: &RunRequest,
    context: &ResolvedTurnContext,
    working: &WorkingProjection,
    working_hash: &str,
) -> Result<PrecheckOutcome, PolicyError> {
    let mut trace = PolicyTrace::new();

    if request.policy_pack_id != context.policy_pack_id {
        let failure = FailureSpec::policy_denied(
            RuleId::parse("POL-001")?,
            "requested policy pack does not match resolved policy pack",
            "policy_pack_id を resolved policy pack に合わせて再実行する",
        )?;
        trace.record_failure(&failure);
        return Ok(PrecheckOutcome {
            policy_trace: trace,
            terminal_failure: Some(failure),
        });
    }
    trace.record_pass(
        RuleId::parse("POL-001")?,
        "requested policy pack matches resolved policy pack",
    )?;

    if let Some(binding_id) = &request.binding_id {
        if binding_id != &context.binding_id {
            let failure = FailureSpec::binding_unresolved(
                RuleId::parse("BND-005")?,
                "requested binding does not match resolved binding",
                "binding_id を resolved binding に合わせて再実行する",
            )?;
            trace.record_failure(&failure);
            return Ok(PrecheckOutcome {
                policy_trace: trace,
                terminal_failure: Some(failure),
            });
        }
        trace.record_pass(
            RuleId::parse("BND-005")?,
            "requested binding matches resolved binding",
        )?;
    }

    if let Some(adapter_id) = &request.adapter_id {
        let Some(selected) = &context.selected_execution_adapter else {
            let failure = FailureSpec::binding_unresolved(
                RuleId::parse("BND-001")?,
                "execution adapter binding is unresolved",
                "approved execution adapter registry を修正して再実行する",
            )?;
            trace.record_failure(&failure);
            return Ok(PrecheckOutcome {
                policy_trace: trace,
                terminal_failure: Some(failure),
            });
        };
        if &selected.adapter_id != adapter_id {
            let failure = FailureSpec::binding_unresolved(
                RuleId::parse("BND-002")?,
                "resolved execution adapter does not match request.adapter_id",
                "adapter_id と approved registry の解決結果を一致させて再実行する",
            )?;
            trace.record_failure(&failure);
            return Ok(PrecheckOutcome {
                policy_trace: trace,
                terminal_failure: Some(failure),
            });
        }
        for capability in &request.requested_capabilities {
            if !selected
                .allowed_capabilities
                .iter()
                .any(|allowed| allowed == capability)
            {
                let failure = FailureSpec::policy_denied(
                    RuleId::parse("POL-002")?,
                    "policy pack does not allow requested capability",
                    format!(
                        "requested_capabilities から {capability} を外すか、approved adapter profile を修正して再実行する"
                    ),
                )?;
                trace.record_failure(&failure);
                return Ok(PrecheckOutcome {
                    policy_trace: trace,
                    terminal_failure: Some(failure),
                });
            }
        }
    }
    trace.record_pass(
        RuleId::parse("POL-002")?,
        "requested capabilities are within the resolved adapter capability set",
    )?;

    if let Err(failure) = validate_working_projection(context, working, working_hash) {
        trace.record_failure(&failure);
        return Ok(PrecheckOutcome {
            policy_trace: trace,
            terminal_failure: Some(failure),
        });
    }
    trace.record_pass(
        RuleId::parse("WRK-001")?,
        "working projection satisfies Free v0.1 invariants",
    )?;
    Ok(PrecheckOutcome {
        policy_trace: trace,
        terminal_failure: None,
    })
}

pub fn choose_first_terminal_failure(
    failures: impl IntoIterator<Item = FailureSpec>,
) -> Option<FailureSpec> {
    let mut chosen: Option<FailureSpec> = None;
    for failure in failures {
        match &chosen {
            None => chosen = Some(failure),
            Some(existing) if failure.stage.rank() < existing.stage.rank() => {
                chosen = Some(failure);
            }
            _ => {}
        }
    }
    chosen
}

fn validate_working_projection(
    context: &ResolvedTurnContext,
    working: &WorkingProjection,
    working_hash: &str,
) -> Result<(), FailureSpec> {
    if working.version != 1
        || working.correlation_id != context.correlation_id
        || working.limit != 12
        || working.slots.len() > 12
        || !working_hash.starts_with("sha256:")
        || working_hash.len() != 71
    {
        return Err(FailureSpec::working_invalid(
            RuleId::parse("WRK-001").expect("static rule_id must be valid"),
            "working projection is outside Free v0.1 constraints",
            "working projection を 12 slot 以下・version=1・sha256 hash 付きへ正規化して再実行する",
        )
        .expect("static failure spec must be valid"));
    }
    Ok(())
}

fn validate_rule_reason_pair(
    rule_id: &RuleId,
    reason_kind: &ReasonKind,
) -> Result<(), PolicyError> {
    let expected = match rule_id.prefix() {
        "REQ" => ReasonKind::InvalidRequest,
        "POL" => ReasonKind::PolicyDenied,
        "BND" => ReasonKind::BindingUnresolved,
        "WRK" => ReasonKind::WorkingInvalid,
        "CIT" => ReasonKind::CitationDenied,
        "EXE" => ReasonKind::ExecutionFailed,
        "LDG" => ReasonKind::LedgerCommitFailed,
        "WUP" => ReasonKind::WorkingUpdateFailed,
        "INT" => ReasonKind::InternalError,
        other => {
            return Err(PolicyError::Invalid(format!(
                "unknown rule_id prefix: {other}"
            )));
        }
    };
    if expected != *reason_kind {
        return Err(PolicyError::Invalid(format!(
            "rule_id prefix {} does not match reason_kind {:?}",
            rule_id.prefix(),
            reason_kind
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        FailureSpec, FailureStage, FinalDecision, choose_first_terminal_failure, evaluate_precheck,
    };
    use crate::resolved_turn_context::{
        ResolvedKernelAdapters, ResolvedTurnContext, SelectedExecutionAdapter, TimeoutPolicy,
    };
    use crate::working::{WorkingProjection, WorkingSlot, WorkingSlotKind};
    use cyrune_core_contract::{
        CorrelationId, IoMode, RequestId, RuleId, RunKind, RunRequest, SlotId,
    };

    fn base_request(run_kind: RunKind, adapter_id: Option<&str>) -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0101").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0101").unwrap(),
            run_kind,
            user_input: "policy".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: adapter_id.map(ToOwned::to_owned),
            argv: None,
            cwd: None,
            env_overrides: None,
        }
    }

    fn base_context(
        selected_execution_adapter: Option<SelectedExecutionAdapter>,
    ) -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0101").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0101").unwrap(),
            run_id: cyrune_core_contract::RunId::parse("RUN-20260327-0101-R01").unwrap(),
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
            run_kind: if selected_execution_adapter.is_some() {
                RunKind::ExecutionAdapter
            } else {
                RunKind::NoLlm
            },
            io_mode: IoMode::Captured,
            selected_execution_adapter,
            timeout_policy: TimeoutPolicy {
                turn_timeout_s: 120,
                execution_timeout_s: 120,
            },
        }
    }

    fn base_working() -> WorkingProjection {
        WorkingProjection {
            version: 1,
            generated_at: "2026-03-27T12:00:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0101").unwrap(),
            limit: 12,
            slots: vec![WorkingSlot {
                slot_id: SlotId::parse("W-001").unwrap(),
                kind: WorkingSlotKind::Constraint,
                text: "keep within Free".to_string(),
                source_evidence_id: "EVID-1".to_string(),
                source_layer: crate::memory::SourceLayer::Processing,
                priority: 1000,
                updated_at: "2026-03-27T12:00:00+09:00".to_string(),
            }],
        }
    }

    #[test]
    fn precheck_rejects_capability_missing_from_selected_adapter() {
        let request = base_request(
            RunKind::ExecutionAdapter,
            Some("local-cli-single-process.v0.1"),
        );
        let context = base_context(Some(SelectedExecutionAdapter {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            execution_kind: "process_stdio".to_string(),
            launcher_path: "/bin/sh".to_string(),
            launcher_sha256: "sha256:launcher".to_string(),
            model_id: "model".to_string(),
            model_revision_or_digest: "sha256:model".to_string(),
            default_timeout_s: 60,
            allowed_capabilities: vec!["exec".to_string()],
            env_allowlist: Vec::new(),
        }));
        let outcome = evaluate_precheck(
            &request,
            &context,
            &base_working(),
            &("sha256:".to_owned() + &"0".repeat(64)),
        )
        .unwrap();
        assert_eq!(outcome.policy_trace.final_decision, FinalDecision::Rejected);
        let failure = outcome.terminal_failure.unwrap();
        assert_eq!(
            failure.reason_kind,
            cyrune_core_contract::ReasonKind::PolicyDenied
        );
        assert_eq!(failure.rule_id.as_str(), "POL-002");
    }

    #[test]
    fn precheck_rejects_binding_mismatch_when_explicitly_requested() {
        let mut request = base_request(RunKind::NoLlm, None);
        request.binding_id = Some("cyrune-free-shipping.v0.1".to_string());
        let context = base_context(None);

        let outcome = evaluate_precheck(
            &request,
            &context,
            &base_working(),
            &("sha256:".to_owned() + &"0".repeat(64)),
        )
        .unwrap();

        assert_eq!(outcome.policy_trace.final_decision, FinalDecision::Rejected);
        let failure = outcome.terminal_failure.unwrap();
        assert_eq!(
            failure.reason_kind,
            cyrune_core_contract::ReasonKind::BindingUnresolved
        );
        assert_eq!(failure.rule_id.as_str(), "BND-005");
    }

    #[test]
    fn first_terminal_failure_wins_by_stage_rank() {
        let later = FailureSpec::citation_denied(
            RuleId::parse("CIT-001").unwrap(),
            "later failure",
            "fix citation",
        )
        .unwrap();
        let earlier = FailureSpec::policy_denied(
            RuleId::parse("POL-001").unwrap(),
            "earlier failure",
            "fix policy",
        )
        .unwrap();
        let chosen = choose_first_terminal_failure(vec![later, earlier]).unwrap();
        assert_eq!(chosen.stage, FailureStage::PolicyGate);
        assert_eq!(chosen.rule_id.as_str(), "POL-001");
    }
}
