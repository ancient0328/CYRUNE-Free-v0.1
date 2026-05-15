#![forbid(unsafe_code)]

use cyrune_core_contract::{
    ContractError, CorrelationId, Denial, DenialId, EvidenceId, ReasonKind, RuleId, RunId,
    RunOutcome, RunRejected, RunRequest,
};

pub fn validate_request(request: &RunRequest) -> Result<RunId, ContractError> {
    request.validate_shape()?;
    let run_id = RunId::for_single_run(&request.correlation_id);
    RunId::ensure_matches_single_run(&request.correlation_id, &run_id)?;
    Ok(run_id)
}

pub fn build_invalid_request_rejection(
    request: &RunRequest,
    evidence_id: EvidenceId,
    rule_id: RuleId,
    message: impl Into<String>,
    remediation: impl Into<String>,
) -> Result<RunRejected, ContractError> {
    let run_id = RunId::for_single_run(&request.correlation_id);
    let denial = Denial::new(
        DenialId::from_evidence_id(&evidence_id),
        rule_id,
        ReasonKind::InvalidRequest,
        message,
        remediation,
    )?;
    Ok(RunRejected {
        outcome: RunOutcome::Rejected,
        response_to: request.request_id.clone(),
        correlation_id: request.correlation_id.clone(),
        run_id,
        denial_id: denial.denial_id,
        evidence_id,
        rule_id: denial.rule_id,
        reason_kind: denial.reason_kind,
        message: denial.message,
        remediation: denial.remediation,
    })
}

#[must_use]
pub fn single_run_id(correlation_id: &CorrelationId) -> RunId {
    RunId::for_single_run(correlation_id)
}

#[cfg(test)]
mod tests {
    use super::{build_invalid_request_rejection, single_run_id, validate_request};
    use cyrune_core_contract::{
        CorrelationId, EvidenceId, IoMode, ReasonKind, RequestId, RuleId, RunKind, RunRequest,
    };

    fn base_request(run_kind: RunKind, adapter_id: Option<&str>) -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0001").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0001").unwrap(),
            run_kind,
            user_input: "Summarize Free boundaries.".to_string(),
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

    #[test]
    fn missing_adapter_is_rejected_for_execution_adapter() {
        let request = base_request(RunKind::ExecutionAdapter, None);
        let error = validate_request(&request).unwrap_err();
        assert_eq!(
            error,
            cyrune_core_contract::ContractError::InvalidRequest {
                message: "adapter_id is required when run_kind=execution_adapter".to_string(),
            }
        );
    }

    #[test]
    fn adapter_is_rejected_for_no_llm() {
        let request = base_request(RunKind::NoLlm, Some("local-cli-single-process.v0.1"));
        let error = validate_request(&request).unwrap_err();
        assert_eq!(
            error,
            cyrune_core_contract::ContractError::InvalidRequest {
                message: "adapter_id must be absent when run_kind=no_llm".to_string(),
            }
        );
    }

    #[test]
    fn valid_no_llm_request_derives_single_run_id() {
        let request = base_request(RunKind::NoLlm, None);
        let run_id = validate_request(&request).unwrap();
        assert_eq!(run_id.as_str(), "RUN-20260327-0001-R01");
        assert_eq!(
            single_run_id(&request.correlation_id).as_str(),
            run_id.as_str()
        );
    }

    #[test]
    fn invalid_request_rejection_uses_closed_reason_kind() {
        let request = base_request(RunKind::NoLlm, None);
        let rejection = build_invalid_request_rejection(
            &request,
            EvidenceId::new(7),
            RuleId::parse("REQ-001").unwrap(),
            "request shape is invalid",
            "request fields を修正して再実行する",
        )
        .unwrap();
        assert_eq!(rejection.reason_kind, ReasonKind::InvalidRequest);
        assert_eq!(rejection.denial_id.as_str(), "DENY-7");
    }
}
