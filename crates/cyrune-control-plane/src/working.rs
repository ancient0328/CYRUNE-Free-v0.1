#![forbid(unsafe_code)]

use crate::memory::SourceLayer;
use cyrune_core_contract::{ContractError, CorrelationId, SlotId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const WORKING_LIMIT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WorkingSlotKind {
    Decision,
    Constraint,
    Assumption,
    Todo,
    Definition,
    Context,
    Command,
}

impl WorkingSlotKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Constraint => "constraint",
            Self::Assumption => "assumption",
            Self::Todo => "todo",
            Self::Definition => "definition",
            Self::Context => "context",
            Self::Command => "command",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkingCandidateCategory {
    PolicyConstraint,
    RequestConstraint,
    CarryForward,
    RetrievalSupport,
    TurnResult,
}

impl WorkingCandidateCategory {
    #[must_use]
    pub fn priority_band(self) -> u16 {
        match self {
            Self::PolicyConstraint => 1000,
            Self::RequestConstraint => 900,
            Self::TurnResult => 800,
            Self::CarryForward => 700,
            Self::RetrievalSupport => 600,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingCandidate {
    pub category: WorkingCandidateCategory,
    pub kind: WorkingSlotKind,
    pub text: String,
    pub source_evidence_id: String,
    pub source_layer: SourceLayer,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingSlot {
    pub slot_id: SlotId,
    pub kind: WorkingSlotKind,
    pub text: String,
    pub source_evidence_id: String,
    pub source_layer: SourceLayer,
    pub priority: u16,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingProjection {
    pub version: u8,
    pub generated_at: String,
    pub correlation_id: CorrelationId,
    pub limit: usize,
    pub slots: Vec<WorkingSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingDelta {
    pub reused_slot_ids: Vec<SlotId>,
    pub new_slot_ids: Vec<SlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingRebuildInput {
    pub generated_at: String,
    pub correlation_id: CorrelationId,
    pub prior_working: Option<WorkingProjection>,
    pub candidates: Vec<WorkingCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingRebuildOutput {
    pub projection: WorkingProjection,
    pub working_delta: WorkingDelta,
    pub working_hash: String,
}

#[derive(Debug, Error)]
pub enum WorkingError {
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error("{0}")]
    Invalid(String),
}

pub fn rebuild_working(input: &WorkingRebuildInput) -> Result<WorkingRebuildOutput, WorkingError> {
    if input.generated_at.trim().is_empty() {
        return Err(WorkingError::Invalid(
            "generated_at must not be empty".to_string(),
        ));
    }

    let mut grouped = BTreeMap::<String, Vec<WorkingCandidate>>::new();
    for candidate in &input.candidates {
        validate_candidate(candidate)?;
        grouped
            .entry(dedupe_key(candidate))
            .or_default()
            .push(candidate.clone());
    }

    let prior_slots = prior_slot_map(input.prior_working.as_ref())?;
    let mut deduped = Vec::new();
    for (key, group) in grouped {
        deduped.push(select_group_winner(&key, &group)?);
    }

    deduped.sort_by(|left, right| {
        right
            .category
            .priority_band()
            .cmp(&left.category.priority_band())
            .then_with(|| right.updated_at_unix_ms.cmp(&left.updated_at_unix_ms))
            .then_with(|| {
                source_layer_priority(right.source_layer)
                    .cmp(&source_layer_priority(left.source_layer))
            })
            .then_with(|| dedupe_key(left).cmp(&dedupe_key(right)))
    });
    deduped.truncate(WORKING_LIMIT);

    let mut reserved_slot_ids: BTreeSet<SlotId> = BTreeSet::new();
    for candidate in &deduped {
        let key = dedupe_key(candidate);
        if let Some(existing) = prior_slots.get(&key) {
            reserved_slot_ids.insert(existing.clone());
        }
    }
    let mut final_slot_ids: BTreeSet<SlotId> = BTreeSet::new();
    let mut reused_slot_ids = Vec::new();
    let mut new_slot_ids = Vec::new();
    let mut slots = Vec::new();

    for candidate in deduped {
        let key = dedupe_key(&candidate);
        let slot_id = if let Some(existing) = prior_slots.get(&key) {
            reused_slot_ids.push(existing.clone());
            existing.clone()
        } else {
            let created = next_slot_id(&reserved_slot_ids)?;
            new_slot_ids.push(created.clone());
            reserved_slot_ids.insert(created.clone());
            created
        };
        if !final_slot_ids.insert(slot_id.clone()) {
            return Err(WorkingError::Invalid(
                "slot_id duplicate detected in working projection".to_string(),
            ));
        }
        slots.push(WorkingSlot {
            slot_id,
            kind: candidate.kind,
            text: candidate.text,
            source_evidence_id: candidate.source_evidence_id,
            source_layer: candidate.source_layer,
            priority: candidate.category.priority_band(),
            updated_at: candidate.updated_at,
        });
    }

    slots.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| right.updated_at.cmp(&left.updated_at))
            .then_with(|| left.slot_id.as_str().cmp(right.slot_id.as_str()))
    });

    let projection = WorkingProjection {
        version: 1,
        generated_at: input.generated_at.clone(),
        correlation_id: input.correlation_id.clone(),
        limit: WORKING_LIMIT,
        slots,
    };
    let working_hash = format!("sha256:{}", sha256_hex(&projection.canonical_json_bytes()?));

    Ok(WorkingRebuildOutput {
        projection,
        working_delta: WorkingDelta {
            reused_slot_ids,
            new_slot_ids,
        },
        working_hash,
    })
}

impl WorkingProjection {
    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, WorkingError> {
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| WorkingError::Invalid(error.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

fn validate_candidate(candidate: &WorkingCandidate) -> Result<(), WorkingError> {
    if candidate.text.trim().is_empty() || candidate.source_evidence_id.trim().is_empty() {
        return Err(WorkingError::Invalid(
            "working candidate requires text and source_evidence_id".to_string(),
        ));
    }
    if candidate.updated_at.trim().is_empty() {
        return Err(WorkingError::Invalid(
            "working candidate requires updated_at".to_string(),
        ));
    }
    Ok(())
}

fn select_group_winner(
    key: &str,
    group: &[WorkingCandidate],
) -> Result<WorkingCandidate, WorkingError> {
    let mut best: Option<&WorkingCandidate> = None;
    let mut best_tuple: Option<(u16, u8, u64, &str)> = None;
    for candidate in group {
        let tuple = (
            candidate.category.priority_band(),
            source_layer_priority(candidate.source_layer),
            candidate.updated_at_unix_ms,
            candidate.source_evidence_id.as_str(),
        );
        match best_tuple {
            None => {
                best = Some(candidate);
                best_tuple = Some(tuple);
            }
            Some(current) => {
                if tuple > current {
                    best = Some(candidate);
                    best_tuple = Some(tuple);
                } else if tuple == current && best != Some(candidate) {
                    return Err(WorkingError::Invalid(format!(
                        "working candidate tie is not breakable for dedupe_key: {key}"
                    )));
                }
            }
        }
    }
    best.cloned().ok_or_else(|| {
        WorkingError::Invalid(format!(
            "working candidate group must not be empty for dedupe_key: {key}"
        ))
    })
}

fn prior_slot_map(
    prior: Option<&WorkingProjection>,
) -> Result<BTreeMap<String, SlotId>, WorkingError> {
    let mut map = BTreeMap::new();
    let Some(prior) = prior else {
        return Ok(map);
    };
    for slot in &prior.slots {
        let key = dedupe_key_from_slot(slot);
        if map.insert(key, slot.slot_id.clone()).is_some() {
            return Err(WorkingError::Invalid(
                "prior working contains duplicated dedupe_key".to_string(),
            ));
        }
    }
    Ok(map)
}

fn next_slot_id(used: &BTreeSet<SlotId>) -> Result<SlotId, WorkingError> {
    for next in 1..=999 {
        let slot_id = SlotId::parse(format!("W-{next:03}"))?;
        if !used.contains(&slot_id) {
            return Ok(slot_id);
        }
    }
    Err(WorkingError::Invalid("exhausted slot_id space".to_string()))
}

fn dedupe_key(candidate: &WorkingCandidate) -> String {
    format!(
        "{}\n{}",
        candidate.kind.as_str(),
        normalize_ws(&candidate.text)
    )
}

fn dedupe_key_from_slot(slot: &WorkingSlot) -> String {
    format!("{}\n{}", slot.kind.as_str(), normalize_ws(&slot.text))
}

fn normalize_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn source_layer_priority(layer: SourceLayer) -> u8 {
    match layer {
        SourceLayer::Working => 3,
        SourceLayer::Processing => 2,
        SourceLayer::Permanent => 1,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        WorkingCandidate, WorkingCandidateCategory, WorkingProjection, WorkingRebuildInput,
        WorkingSlot, WorkingSlotKind, rebuild_working,
    };
    use crate::memory::SourceLayer;
    use cyrune_core_contract::{CorrelationId, SlotId};

    fn candidate(
        category: WorkingCandidateCategory,
        kind: WorkingSlotKind,
        text: &str,
        evidence_id: &str,
        source_layer: SourceLayer,
        updated_at_unix_ms: u64,
    ) -> WorkingCandidate {
        WorkingCandidate {
            category,
            kind,
            text: text.to_string(),
            source_evidence_id: evidence_id.to_string(),
            source_layer,
            updated_at: format!("2026-03-27T15:30:{updated_at_unix_ms:02}+09:00"),
            updated_at_unix_ms,
        }
    }

    #[test]
    fn rebuild_enforces_hard_limit_12() {
        let mut candidates = Vec::new();
        for idx in 0..16 {
            candidates.push(candidate(
                WorkingCandidateCategory::RetrievalSupport,
                WorkingSlotKind::Context,
                &format!("context {idx}"),
                &format!("EVID-{idx}"),
                SourceLayer::Processing,
                idx,
            ));
        }
        let output = rebuild_working(&WorkingRebuildInput {
            generated_at: "2026-03-27T15:35:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0010").unwrap(),
            prior_working: None,
            candidates,
        })
        .unwrap();
        assert_eq!(output.projection.limit, 12);
        assert_eq!(output.projection.slots.len(), 12);
    }

    #[test]
    fn rebuild_is_deterministic_and_reuses_slot_ids() {
        let prior = WorkingProjection {
            version: 1,
            generated_at: "2026-03-27T15:30:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0011").unwrap(),
            limit: 12,
            slots: vec![WorkingSlot {
                slot_id: SlotId::parse("W-001").unwrap(),
                kind: WorkingSlotKind::Decision,
                text: "Free freezes Control Plane semantics.".to_string(),
                source_evidence_id: "EVID-1".to_string(),
                source_layer: SourceLayer::Processing,
                priority: 800,
                updated_at: "2026-03-27T15:30:00+09:00".to_string(),
            }],
        };
        let input = WorkingRebuildInput {
            generated_at: "2026-03-27T15:40:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0011").unwrap(),
            prior_working: Some(prior),
            candidates: vec![
                candidate(
                    WorkingCandidateCategory::TurnResult,
                    WorkingSlotKind::Decision,
                    "Free   freezes  Control Plane semantics.",
                    "EVID-1",
                    SourceLayer::Processing,
                    50,
                ),
                candidate(
                    WorkingCandidateCategory::RequestConstraint,
                    WorkingSlotKind::Constraint,
                    "Do not mix upper tiers into Free.",
                    "EVID-2",
                    SourceLayer::Processing,
                    51,
                ),
            ],
        };

        let left = rebuild_working(&input).unwrap();
        let right = rebuild_working(&input).unwrap();
        assert_eq!(left.projection, right.projection);
        assert_eq!(left.working_hash, right.working_hash);
        assert_eq!(left.projection.slots[1].slot_id.as_str(), "W-001");
    }
}
