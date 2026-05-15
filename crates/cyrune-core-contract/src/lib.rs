#![forbid(unsafe_code)]

pub mod denial;
pub mod error;
pub mod id;
pub mod path_label;
pub mod run_request;
pub mod run_result;

pub use denial::{Denial, ReasonKind, RuleId};
pub use error::ContractError;
pub use id::{
    CitationBundleId, ClaimId, CorrelationId, DenialId, EvidenceId, RequestId, RunId, SlotId,
};
pub use path_label::PathLabel;
pub use run_request::{IoMode, RunKind, RunRequest};
pub use run_result::{RunAccepted, RunOutcome, RunRejected};

pub const CRATE_IDENTITY: &str = "cyrune-core-contract";

#[must_use]
pub fn crate_identity() -> &'static str {
    CRATE_IDENTITY
}

#[cfg(test)]
mod tests {
    use super::{
        CitationBundleId, CorrelationId, DenialId, EvidenceId, RequestId, RunId, SlotId,
        crate_identity,
    };

    #[test]
    fn crate_identity_is_stable() {
        assert_eq!(crate_identity(), "cyrune-core-contract");
    }

    #[test]
    fn canonical_id_derivations_are_stable() {
        let request_id = RequestId::parse("REQ-20260327-0001").unwrap();
        let correlation_id = CorrelationId::parse("RUN-20260327-0001").unwrap();
        let run_id = RunId::for_single_run(&correlation_id);
        let evidence_id = EvidenceId::new(42);
        let denial_id = DenialId::from_evidence_id(&evidence_id);
        let bundle_id = CitationBundleId::from_correlation_id(&correlation_id);
        let slot_id = SlotId::parse("W-001").unwrap();

        assert_eq!(request_id.as_str(), "REQ-20260327-0001");
        assert_eq!(run_id.as_str(), "RUN-20260327-0001-R01");
        assert_eq!(denial_id.as_str(), "DENY-42");
        assert_eq!(bundle_id.as_str(), "CB-20260327-0001");
        assert_eq!(slot_id.as_str(), "W-001");
    }
}
