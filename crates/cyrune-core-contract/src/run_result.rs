#![forbid(unsafe_code)]

use crate::denial::{ReasonKind, RuleId};
use crate::id::{CitationBundleId, CorrelationId, DenialId, EvidenceId, RequestId, RunId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunAccepted {
    pub outcome: RunOutcome,
    pub response_to: RequestId,
    pub correlation_id: CorrelationId,
    pub run_id: RunId,
    pub evidence_id: EvidenceId,
    pub output: String,
    pub citation_bundle_id: CitationBundleId,
    pub working_hash_after: String,
    pub policy_pack_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRejected {
    pub outcome: RunOutcome,
    pub response_to: RequestId,
    pub correlation_id: CorrelationId,
    pub run_id: RunId,
    pub denial_id: DenialId,
    pub evidence_id: EvidenceId,
    pub rule_id: RuleId,
    pub reason_kind: ReasonKind,
    pub message: String,
    pub remediation: String,
}
