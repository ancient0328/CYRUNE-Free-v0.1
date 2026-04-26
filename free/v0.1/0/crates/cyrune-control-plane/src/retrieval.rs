#![forbid(unsafe_code)]

use crate::memory::{
    MemoryError, MemoryFacade, RetrievedCandidateRecord, SourceLayer, ValidityState,
};
use crate::resolved_turn_context::ResolvedTurnContext;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const PROCESSING_ANCHOR_TOP_K: usize = 8;
const PROCESSING_SEMANTIC_TOP_K: usize = 8;
const PERMANENT_ANCHOR_TOP_K: usize = 8;
const PERMANENT_SEMANTIC_TOP_K: usize = 8;
const CORROBORATED_CAP: usize = 6;
const ANCHOR_ONLY_CAP: usize = 4;
const SEMANTIC_SUPPORT_CAP: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FusionClass {
    Corroborated,
    AnchorOnly,
    SemanticSupport,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFilterStatus {
    Passed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FusionResult {
    pub candidate_id: String,
    pub source_layer: SourceLayer,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub expires_at_unix_ms: Option<u64>,
    pub validity_state: ValidityState,
    pub fusion_class: FusionClass,
    pub anchor_present: bool,
    pub semantic_present: bool,
    pub anchor_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
    pub post_filter_status: PostFilterStatus,
    pub rejection_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalCandidate {
    pub candidate_id: String,
    pub source_layer: SourceLayer,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub validity_state: ValidityState,
    pub fusion_class: FusionClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySummary {
    pub query_hash: String,
    pub selected_memory_ids: Vec<String>,
    pub rejected_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalSelectionResult {
    pub final_candidates: Vec<FinalCandidate>,
    pub fusion_results: Vec<FusionResult>,
    pub query_summary: QuerySummary,
}

#[derive(Debug, Error)]
pub enum RetrievalError {
    #[error(transparent)]
    Memory(#[from] MemoryError),
    #[error("{0}")]
    Invalid(String),
}

pub fn select_candidates(
    memory: &MemoryFacade,
    context: &ResolvedTurnContext,
    now_ms: u64,
    query_text: &str,
) -> Result<RetrievalSelectionResult, RetrievalError> {
    if query_text.trim().is_empty() {
        return Err(RetrievalError::Invalid(
            "query_text must not be empty".to_string(),
        ));
    }

    let processing = build_layer_fusion(
        memory,
        context,
        now_ms,
        query_text,
        SourceLayer::Processing,
        PROCESSING_ANCHOR_TOP_K,
        PROCESSING_SEMANTIC_TOP_K,
    )?;
    let permanent = build_layer_fusion(
        memory,
        context,
        now_ms,
        query_text,
        SourceLayer::Permanent,
        PERMANENT_ANCHOR_TOP_K,
        PERMANENT_SEMANTIC_TOP_K,
    )?;

    let mut fusion_results = Vec::new();
    fusion_results.extend(processing);
    fusion_results.extend(permanent);

    shadow_permanent_duplicates(&mut fusion_results);

    let final_candidates = final_select(&fusion_results)?;
    let query_summary = QuerySummary {
        query_hash: format!("sha256:{}", sha256_hex(query_text.as_bytes())),
        selected_memory_ids: final_candidates
            .iter()
            .map(|candidate| candidate.candidate_id.clone())
            .collect(),
        rejected_reasons: collect_rejection_reasons(&fusion_results),
    };

    Ok(RetrievalSelectionResult {
        final_candidates,
        fusion_results,
        query_summary,
    })
}

fn build_layer_fusion(
    memory: &MemoryFacade,
    context: &ResolvedTurnContext,
    now_ms: u64,
    query_text: &str,
    layer: SourceLayer,
    anchor_limit: usize,
    semantic_limit: usize,
) -> Result<Vec<FusionResult>, RetrievalError> {
    let anchor = memory.lexical_search(context, now_ms, layer, query_text, anchor_limit)?;
    let semantic = memory.semantic_search(context, now_ms, layer, query_text, semantic_limit)?;

    let mut by_id = BTreeMap::<String, FusionResult>::new();
    for (rank, candidate) in anchor.iter().enumerate() {
        let entry = by_id
            .entry(candidate.candidate_id.clone())
            .or_insert_with(|| base_fusion(candidate, layer));
        entry.anchor_present = true;
        entry.anchor_rank = Some(rank + 1);
    }
    for (rank, candidate) in semantic.iter().enumerate() {
        let entry = by_id
            .entry(candidate.candidate_id.clone())
            .or_insert_with(|| base_fusion(candidate, layer));
        entry.semantic_present = true;
        entry.semantic_rank = Some(rank + 1);
    }

    let mut out = Vec::new();
    for (_, mut result) in by_id {
        result.fusion_class = classify(result.anchor_present, result.semantic_present)?;
        apply_mandatory_filters(&mut result, now_ms);
        out.push(result);
    }
    Ok(out)
}

fn base_fusion(candidate: &RetrievedCandidateRecord, layer: SourceLayer) -> FusionResult {
    FusionResult {
        candidate_id: candidate.candidate_id.clone(),
        source_layer: layer,
        payload_ref: candidate.payload_ref.clone(),
        text: candidate.text.clone(),
        source_evidence_ids: candidate.source_evidence_ids.clone(),
        updated_at: candidate.updated_at.clone(),
        updated_at_unix_ms: candidate.updated_at_unix_ms,
        expires_at_unix_ms: candidate.expires_at_unix_ms,
        validity_state: candidate.validity_state,
        fusion_class: FusionClass::Rejected,
        anchor_present: false,
        semantic_present: false,
        anchor_rank: None,
        semantic_rank: None,
        post_filter_status: PostFilterStatus::Passed,
        rejection_reasons: Vec::new(),
    }
}

fn classify(anchor_present: bool, semantic_present: bool) -> Result<FusionClass, RetrievalError> {
    match (anchor_present, semantic_present) {
        (true, true) => Ok(FusionClass::Corroborated),
        (true, false) => Ok(FusionClass::AnchorOnly),
        (false, true) => Ok(FusionClass::SemanticSupport),
        (false, false) => Err(RetrievalError::Invalid(
            "fusion class cannot be determined without anchor or semantic signal".to_string(),
        )),
    }
}

fn apply_mandatory_filters(result: &mut FusionResult, now_ms: u64) {
    if result.source_evidence_ids.is_empty() {
        reject(result, "missing_provenance");
    }
    if result.validity_state != ValidityState::Valid {
        reject(
            result,
            match result.validity_state {
                ValidityState::Valid => "valid",
                ValidityState::Superseded => "superseded",
                ValidityState::Invalidated => "invalidated",
            },
        );
    }
    if let Some(expires_at_unix_ms) = result_expiry(result) {
        if expires_at_unix_ms <= now_ms {
            reject(result, "expired");
        }
    }
}

fn result_expiry(result: &FusionResult) -> Option<u64> {
    result.expires_at_unix_ms
}

fn reject(result: &mut FusionResult, reason: &str) {
    result.fusion_class = FusionClass::Rejected;
    result.post_filter_status = PostFilterStatus::Rejected;
    if !result
        .rejection_reasons
        .iter()
        .any(|existing| existing == reason)
    {
        result.rejection_reasons.push(reason.to_string());
    }
}

fn shadow_permanent_duplicates(results: &mut [FusionResult]) {
    let mut processing_ids = BTreeSet::new();
    for result in results.iter() {
        if result.source_layer == SourceLayer::Processing
            && result.post_filter_status == PostFilterStatus::Passed
        {
            processing_ids.insert(result.candidate_id.clone());
        }
    }
    for result in results.iter_mut() {
        if result.source_layer == SourceLayer::Permanent
            && result.post_filter_status == PostFilterStatus::Passed
            && processing_ids.contains(&result.candidate_id)
        {
            reject(result, "shadowed_by_processing");
        }
    }
}

fn final_select(results: &[FusionResult]) -> Result<Vec<FinalCandidate>, RetrievalError> {
    let mut corroborated = Vec::new();
    let mut anchor_only = Vec::new();
    let mut semantic_support = Vec::new();

    for result in results {
        if result.post_filter_status != PostFilterStatus::Passed {
            continue;
        }
        match result.fusion_class {
            FusionClass::Corroborated => corroborated.push(result.clone()),
            FusionClass::AnchorOnly => anchor_only.push(result.clone()),
            FusionClass::SemanticSupport => semantic_support.push(result.clone()),
            FusionClass::Rejected => {}
        }
    }

    sort_bucket(&mut corroborated);
    sort_bucket(&mut anchor_only);
    sort_bucket(&mut semantic_support);

    let mut out = Vec::new();
    out.extend(corroborated.into_iter().take(CORROBORATED_CAP));
    out.extend(anchor_only.into_iter().take(ANCHOR_ONLY_CAP));
    out.extend(semantic_support.into_iter().take(SEMANTIC_SUPPORT_CAP));

    Ok(out
        .into_iter()
        .map(|result| FinalCandidate {
            candidate_id: result.candidate_id,
            source_layer: result.source_layer,
            payload_ref: result.payload_ref,
            text: result.text,
            source_evidence_ids: result.source_evidence_ids,
            updated_at: result.updated_at,
            updated_at_unix_ms: result.updated_at_unix_ms,
            validity_state: result.validity_state,
            fusion_class: result.fusion_class,
        })
        .collect())
}

fn sort_bucket(bucket: &mut [FusionResult]) {
    bucket.sort_by(|left, right| {
        source_layer_priority(right.source_layer)
            .cmp(&source_layer_priority(left.source_layer))
            .then_with(|| option_rank(left.anchor_rank).cmp(&option_rank(right.anchor_rank)))
            .then_with(|| option_rank(left.semantic_rank).cmp(&option_rank(right.semantic_rank)))
            .then_with(|| right.updated_at_unix_ms.cmp(&left.updated_at_unix_ms))
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
}

fn option_rank(rank: Option<usize>) -> usize {
    rank.unwrap_or(usize::MAX)
}

fn collect_rejection_reasons(results: &[FusionResult]) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    for result in results {
        for reason in &result.rejection_reasons {
            reasons.insert(reason.clone());
        }
    }
    reasons.into_iter().collect()
}

fn source_layer_priority(layer: SourceLayer) -> u8 {
    match layer {
        SourceLayer::Processing => 2,
        SourceLayer::Permanent => 1,
        SourceLayer::Working => 0,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{FusionClass, select_candidates};
    use crate::memory::{
        MemoryFacade, PermanentRecordInput, ProcessingRecordInput, RecordKeyspace,
        RelationMarkInput, SourceLayer, ValidityState,
    };
    use crate::resolved_turn_context::{
        EmbeddingExactPin, MemoryStateRoots, ResolvedKernelAdapters, ResolvedTurnContext,
        TimeoutPolicy,
    };
    use crate::resolver::shipping_embedding_engine_ref_for_pin;
    use crate::working::WorkingSlotKind;
    use cyrune_core_contract::{CorrelationId, IoMode, RequestId, RunId, RunKind};
    use serde::Deserialize;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[derive(Debug, Deserialize)]
    struct ShippingExactPinManifest {
        engine_kind: String,
        upstream_model_id: String,
        upstream_revision: String,
        artifact_set: Vec<String>,
        artifact_sha256: std::collections::BTreeMap<String, String>,
        artifact_paths: std::collections::BTreeMap<String, String>,
        dimensions: u16,
        pooling: String,
        normalization: String,
        prompt_profile: String,
        token_limit: u16,
        distance: String,
    }

    fn shipping_embedding_exact_pin() -> EmbeddingExactPin {
        let manifest_path = bundle_embedding_root()
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        let manifest: ShippingExactPinManifest =
            serde_json::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
        EmbeddingExactPin {
            engine_kind: manifest.engine_kind,
            upstream_model_id: manifest.upstream_model_id,
            upstream_revision: Some(manifest.upstream_revision),
            artifact_set: manifest.artifact_set,
            artifact_sha256: manifest.artifact_sha256,
            dimensions: manifest.dimensions,
            pooling: manifest.pooling,
            normalization: manifest.normalization,
            prompt_profile: manifest.prompt_profile,
            token_limit: manifest.token_limit,
            distance: manifest.distance,
        }
    }

    fn bundle_embedding_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/bundle-root/embedding")
    }

    fn bundle_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/bundle-root")
    }

    fn public_run_home_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/public-run/home")
    }

    fn complete_shipping_home_root() -> PathBuf {
        let mut candidates = Vec::new();
        if let Some(root) = std::env::var_os("CYRUNE_TEST_SHIPPING_HOME_ROOT") {
            candidates.push(PathBuf::from(root));
        }
        candidates.push(public_run_home_root());
        candidates.push(bundle_root());

        for candidate in candidates {
            if has_complete_shipping_embedding_home(&candidate) {
                return candidate;
            }
        }

        panic!(
            "shipping tests require materialized embedding artifacts; run ./scripts/prepare-public-run.sh or set CYRUNE_TEST_SHIPPING_HOME_ROOT"
        );
    }

    fn has_complete_shipping_embedding_home(home_root: &Path) -> bool {
        let manifest_path = home_root
            .join("embedding")
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        let Ok(bytes) = fs::read(manifest_path) else {
            return false;
        };
        let Ok(manifest) = serde_json::from_slice::<ShippingExactPinManifest>(&bytes) else {
            return false;
        };
        manifest.artifact_set.iter().all(|artifact_name| {
            manifest
                .artifact_paths
                .get(artifact_name)
                .is_some_and(|relative_path| home_root.join(relative_path).is_file())
        })
    }

    fn copy_tree(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let source = entry.path();
            let target = dst.join(entry.file_name());
            if source.is_dir() {
                copy_tree(&source, &target);
            } else {
                fs::copy(&source, &target).unwrap();
            }
        }
    }

    fn materialize_shipping_embedding_home(home_root: &Path) {
        copy_tree(
            &complete_shipping_home_root().join("embedding"),
            &home_root.join("embedding"),
        );
    }

    fn test_context() -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0004").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0004").unwrap(),
            run_id: RunId::parse("RUN-20260327-0004-R01").unwrap(),
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

    fn shipping_test_context(home_root: &std::path::Path) -> ResolvedTurnContext {
        let pin = shipping_embedding_exact_pin();
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260406-0002").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260406-0002").unwrap(),
            run_id: RunId::parse("RUN-20260406-0002-R01").unwrap(),
            requested_policy_pack_id: "cyrune-free-default".to_string(),
            requested_binding_id: Some("cyrune-free-shipping.v0.1".to_string()),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: "cyrune-free-shipping.v0.1".to_string(),
            resolved_kernel_adapters: ResolvedKernelAdapters {
                working_store_adapter_id: "memory-kv-inmem".to_string(),
                processing_store_adapter_id: "memory-redb-processing".to_string(),
                permanent_store_adapter_id: "memory-stoolap-permanent".to_string(),
                vector_index_adapter_id: "memory-kv-inmem".to_string(),
                embedding_engine_ref: shipping_embedding_engine_ref_for_pin(&pin),
            },
            embedding_exact_pin: Some(pin),
            memory_state_roots: Some(MemoryStateRoots {
                processing_state_root: home_root
                    .join("memory")
                    .join("processing")
                    .display()
                    .to_string(),
                permanent_state_root: home_root
                    .join("memory")
                    .join("permanent")
                    .display()
                    .to_string(),
            }),
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

    #[test]
    fn retrieval_selection_is_deterministic_and_respects_shadowing() {
        let context = test_context();
        let mut memory = MemoryFacade::new(&context).unwrap();

        memory
            .append_retrieval_candidate(
                &context,
                10,
                ProcessingRecordInput {
                    keyspace: RecordKeyspace::RetrievalCandidates,
                    record_id: "MEM-200".to_string(),
                    payload_ref: "processing://retrieval_candidates/MEM-200".to_string(),
                    text: "Control Plane decides final acceptance.".to_string(),
                    source_evidence_ids: vec!["EVID-200".to_string()],
                    created_at: "2026-03-27T15:30:00+09:00".to_string(),
                    created_at_unix_ms: 10,
                    updated_at: "2026-03-27T15:30:00+09:00".to_string(),
                    updated_at_unix_ms: 10,
                    expires_at: "2026-05-08T15:30:00+09:00".to_string(),
                    expires_at_unix_ms: 1_000,
                    working_kind: Some(WorkingSlotKind::Constraint),
                },
            )
            .unwrap();
        memory
            .append_knowledge_record(
                &context,
                11,
                PermanentRecordInput {
                    keyspace: RecordKeyspace::KnowledgeRecords,
                    record_id: "MEM-200".to_string(),
                    payload_ref: "permanent://knowledge_records/MEM-200".to_string(),
                    text: "Control Plane decides final acceptance.".to_string(),
                    source_evidence_ids: vec!["EVID-201".to_string()],
                    created_at: "2026-03-27T15:30:01+09:00".to_string(),
                    created_at_unix_ms: 11,
                    updated_at: "2026-03-27T15:30:01+09:00".to_string(),
                    updated_at_unix_ms: 11,
                    validity_state: ValidityState::Valid,
                    working_kind: Some(WorkingSlotKind::Constraint),
                },
            )
            .unwrap();

        let left = select_candidates(&memory, &context, 20, "Control Plane").unwrap();
        let right = select_candidates(&memory, &context, 20, "Control Plane").unwrap();
        assert_eq!(left, right);
        assert_eq!(left.final_candidates.len(), 1);
        assert_eq!(left.final_candidates[0].candidate_id, "MEM-200");
        assert_eq!(
            left.final_candidates[0].source_layer,
            crate::memory::SourceLayer::Processing
        );
        assert!(
            left.query_summary
                .rejected_reasons
                .iter()
                .any(|reason| reason == "shadowed_by_processing")
        );
    }

    #[test]
    fn semantic_support_does_not_overtake_anchor_bucket() {
        let context = test_context();
        let mut memory = MemoryFacade::new(&context).unwrap();

        memory
            .append_retrieval_candidate(
                &context,
                10,
                ProcessingRecordInput {
                    keyspace: RecordKeyspace::RetrievalCandidates,
                    record_id: "MEM-300".to_string(),
                    payload_ref: "processing://retrieval_candidates/MEM-300".to_string(),
                    text: "Anchor lexical candidate".to_string(),
                    source_evidence_ids: vec!["EVID-300".to_string()],
                    created_at: "2026-03-27T15:30:00+09:00".to_string(),
                    created_at_unix_ms: 10,
                    updated_at: "2026-03-27T15:30:00+09:00".to_string(),
                    updated_at_unix_ms: 10,
                    expires_at: "2026-05-08T15:30:00+09:00".to_string(),
                    expires_at_unix_ms: 1_000,
                    working_kind: Some(WorkingSlotKind::Context),
                },
            )
            .unwrap();
        memory
            .append_retrieval_candidate(
                &context,
                11,
                ProcessingRecordInput {
                    keyspace: RecordKeyspace::RetrievalCandidates,
                    record_id: "MEM-301".to_string(),
                    payload_ref: "processing://retrieval_candidates/MEM-301".to_string(),
                    text: "semantic drift support".to_string(),
                    source_evidence_ids: vec!["EVID-301".to_string()],
                    created_at: "2026-03-27T15:30:01+09:00".to_string(),
                    created_at_unix_ms: 11,
                    updated_at: "2026-03-27T15:30:01+09:00".to_string(),
                    updated_at_unix_ms: 11,
                    expires_at: "2026-05-08T15:30:01+09:00".to_string(),
                    expires_at_unix_ms: 1_001,
                    working_kind: Some(WorkingSlotKind::Context),
                },
            )
            .unwrap();

        let result = select_candidates(&memory, &context, 20, "Anchor").unwrap();
        assert_eq!(result.final_candidates[0].candidate_id, "MEM-300");
        assert_ne!(
            result.final_candidates[0].fusion_class,
            FusionClass::SemanticSupport
        );
    }

    #[test]
    fn shipping_selection_uses_resolved_processing_and_permanent_stores() {
        let temp = tempdir().unwrap();
        let context = shipping_test_context(temp.path());
        materialize_shipping_embedding_home(temp.path());
        {
            let mut memory = MemoryFacade::new(&context).unwrap();
            memory
                .append_retrieval_candidate(
                    &context,
                    10,
                    ProcessingRecordInput {
                        keyspace: RecordKeyspace::RetrievalCandidates,
                        record_id: "MEM-SHIP-DUP-001".to_string(),
                        payload_ref: "processing://retrieval_candidates/MEM-SHIP-DUP-001"
                            .to_string(),
                        text: "Shipping retrieval anchored candidate".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-201".to_string()],
                        created_at: "2026-04-06T23:09:47+09:00".to_string(),
                        created_at_unix_ms: 10,
                        updated_at: "2026-04-06T23:09:47+09:00".to_string(),
                        updated_at_unix_ms: 10,
                        expires_at: "2026-05-18T23:09:47+09:00".to_string(),
                        expires_at_unix_ms: 1_000,
                        working_kind: Some(WorkingSlotKind::Context),
                    },
                )
                .unwrap();
            memory
                .append_knowledge_record(
                    &context,
                    11,
                    PermanentRecordInput {
                        keyspace: RecordKeyspace::KnowledgeRecords,
                        record_id: "MEM-SHIP-DUP-001".to_string(),
                        payload_ref: "permanent://knowledge_records/MEM-SHIP-DUP-001".to_string(),
                        text: "Shipping retrieval anchored candidate".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-202".to_string()],
                        created_at: "2026-04-06T23:09:48+09:00".to_string(),
                        created_at_unix_ms: 11,
                        updated_at: "2026-04-06T23:09:48+09:00".to_string(),
                        updated_at_unix_ms: 11,
                        validity_state: ValidityState::Valid,
                        working_kind: Some(WorkingSlotKind::Context),
                    },
                )
                .unwrap();
            memory
                .append_knowledge_record(
                    &context,
                    12,
                    PermanentRecordInput {
                        keyspace: RecordKeyspace::KnowledgeRecords,
                        record_id: "MEM-SHIP-PERM-001".to_string(),
                        payload_ref: "permanent://knowledge_records/MEM-SHIP-PERM-001".to_string(),
                        text: "Shipping retrieval long-term knowledge".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-203".to_string()],
                        created_at: "2026-04-06T23:09:49+09:00".to_string(),
                        created_at_unix_ms: 12,
                        updated_at: "2026-04-06T23:09:49+09:00".to_string(),
                        updated_at_unix_ms: 12,
                        validity_state: ValidityState::Valid,
                        working_kind: Some(WorkingSlotKind::Constraint),
                    },
                )
                .unwrap();
            memory
                .append_knowledge_record(
                    &context,
                    13,
                    PermanentRecordInput {
                        keyspace: RecordKeyspace::KnowledgeRecords,
                        record_id: "MEM-SHIP-PERM-002".to_string(),
                        payload_ref: "permanent://knowledge_records/MEM-SHIP-PERM-002".to_string(),
                        text: "Shipping retrieval invalidated knowledge".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-204".to_string()],
                        created_at: "2026-04-06T23:09:50+09:00".to_string(),
                        created_at_unix_ms: 13,
                        updated_at: "2026-04-06T23:09:50+09:00".to_string(),
                        updated_at_unix_ms: 13,
                        validity_state: ValidityState::Valid,
                        working_kind: Some(WorkingSlotKind::Constraint),
                    },
                )
                .unwrap();
            memory
                .mark_invalidated(
                    &context,
                    14,
                    RelationMarkInput {
                        relation_id: "REL-SHIP-RETR-001".to_string(),
                        subject_record_id: "MEM-SHIP-PERM-002".to_string(),
                        object_record_id: None,
                        evidence_id: "EVID-SHIP-205".to_string(),
                        created_at: "2026-04-06T23:09:51+09:00".to_string(),
                    },
                )
                .unwrap();
        }

        let reopened = MemoryFacade::new(&context).unwrap();
        let result = select_candidates(&reopened, &context, 20, "Shipping retrieval").unwrap();

        assert!(result.final_candidates.iter().any(|candidate| {
            candidate.candidate_id == "MEM-SHIP-DUP-001"
                && candidate.source_layer == SourceLayer::Processing
        }));
        assert!(result.final_candidates.iter().any(|candidate| {
            candidate.candidate_id == "MEM-SHIP-PERM-001"
                && candidate.source_layer == SourceLayer::Permanent
        }));
        assert!(
            !result
                .final_candidates
                .iter()
                .any(|candidate| candidate.candidate_id == "MEM-SHIP-PERM-002")
        );
        assert!(
            result
                .query_summary
                .rejected_reasons
                .iter()
                .any(|reason| reason == "shadowed_by_processing")
        );
        assert!(
            result
                .query_summary
                .rejected_reasons
                .iter()
                .any(|reason| reason == "invalidated")
        );
        println!("binding_id={}", context.binding_id);
        println!(
            "adapter_lineage=processing:{},permanent:{},embedding:{}",
            context.resolved_kernel_adapters.processing_store_adapter_id,
            context.resolved_kernel_adapters.permanent_store_adapter_id,
            context.resolved_kernel_adapters.embedding_engine_ref,
        );
        println!(
            "selected_candidates={}",
            serde_json::to_string(
                &result
                    .final_candidates
                    .iter()
                    .map(|candidate| json!({
                        "candidate_id": candidate.candidate_id,
                        "source_layer": candidate.source_layer.as_str(),
                        "fusion_class": format!("{:?}", candidate.fusion_class).to_ascii_lowercase(),
                    }))
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        println!(
            "query_summary={}",
            serde_json::to_string(&result.query_summary).unwrap()
        );
    }
}
