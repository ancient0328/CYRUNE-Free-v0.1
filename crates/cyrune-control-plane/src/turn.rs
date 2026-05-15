#![forbid(unsafe_code)]

use crate::citation::{CitationMaterial, SimpleReasoningRecord, validate_citation_output};
use crate::execution_registry::materialize_launcher;
use crate::execution_result::{
    ExecutionResultEnvelope, NoLlmAcceptedDraft, TerminalStatus, execute_local_cli_single_process,
};
use crate::ledger::{
    AcceptedLedgerInput, EvidenceOutcome, LedgerError, LedgerWriter, RejectedLedgerInput,
    TERMINAL_BINDING_SCHEMA_VERSION, TerminalBindingRecord, raw_file_sha256, visible_working_hash,
    write_terminal_binding, write_working_projection,
};
use crate::memory::MemoryFacade;
use crate::policy::{
    FailureSpec, FailureStage, PolicyError, PolicyTrace, choose_first_terminal_failure,
    evaluate_precheck,
};
use crate::resolver::{ResolverError, ResolverInputs, resolve_turn_context};
use crate::retrieval::{QuerySummary, RetrievalError, RetrievalSelectionResult, select_candidates};
use crate::sandbox::{SandboxError, normalize_sandbox_spec};
use crate::working::{
    WorkingCandidate, WorkingCandidateCategory, WorkingError, WorkingProjection,
    WorkingRebuildInput, WorkingRebuildOutput, WorkingSlotKind, rebuild_working,
};
use cyrune_core_contract::{
    DenialId, ReasonKind, RuleId, RunAccepted, RunOutcome, RunRejected, RunRequest,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const RULE_LEDGER_COMMIT_FAILED: &str = "LDG-001";
const RULE_WORKING_UPDATE_FAILED: &str = "WUP-001";
const RULE_WORKING_HASH_MISMATCH: &str = "WUP-002";
const RULE_TERMINAL_BINDING_FAILED: &str = "LDG-002";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedTurnDraft {
    pub request: RunRequest,
    pub context: crate::resolved_turn_context::ResolvedTurnContext,
    pub created_at: String,
    pub started_at: String,
    pub finished_at: String,
    pub exit_status: Option<i32>,
    pub working_hash_before: String,
    pub prior_working: Option<WorkingProjection>,
    pub working_output: WorkingRebuildOutput,
    pub query_summary: QuerySummary,
    pub output_draft: String,
    pub citation_material: CitationMaterial,
    pub rr_material: SimpleReasoningRecord,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedTurnDraft {
    pub request: RunRequest,
    pub context: crate::resolved_turn_context::ResolvedTurnContext,
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

#[derive(Debug)]
struct AdapterInvocationFailure {
    started_at: String,
    finished_at: String,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
    detail: String,
}

#[derive(Debug)]
struct PriorWorkingLoadFailure {
    failure: FailureSpec,
    projection: Option<WorkingProjection>,
}

#[derive(Debug, Error)]
pub enum TurnError {
    #[error(transparent)]
    Policy(#[from] PolicyError),
    #[error(transparent)]
    Sandbox(#[from] SandboxError),
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[error(transparent)]
    Resolver(#[from] ResolverError),
    #[error(transparent)]
    ExecutionResult(#[from] crate::execution_result::ExecutionResultError),
    #[error(transparent)]
    Memory(#[from] crate::memory::MemoryError),
    #[error(transparent)]
    Retrieval(#[from] RetrievalError),
    #[error(transparent)]
    Working(#[from] WorkingError),
}

pub fn finalize_accepted_turn(
    writer: &mut LedgerWriter,
    draft: AcceptedTurnDraft,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    let precheck = evaluate_precheck(
        &draft.request,
        &draft.context,
        &draft.working_output.projection,
        &draft.working_output.working_hash,
    )?;
    if let Some(failure) = precheck.terminal_failure {
        return finalize_rejected_turn(
            writer,
            RejectedTurnDraft {
                request: draft.request,
                context: draft.context,
                created_at: draft.created_at,
                started_at: draft.started_at,
                finished_at: draft.finished_at,
                exit_status: draft.exit_status,
                working_hash_before: draft.working_hash_before,
                query_summary: Some(draft.query_summary),
                failure,
                policy_trace: precheck.policy_trace,
                stdout: draft.stdout,
                stderr: draft.stderr,
            },
        );
    }

    let mut policy_trace = precheck.policy_trace;
    let citation = match validate_citation_output(
        &draft.context.correlation_id,
        &draft.output_draft,
        &draft.citation_material,
        &draft.rr_material,
    ) {
        Ok(validated) => {
            policy_trace.record_pass(
                cyrune_core_contract::RuleId::parse("CIT-001")
                    .expect("static rule_id must be valid"),
                "citation bundle validated against accepted output",
            )?;
            validated
        }
        Err(failure) => {
            policy_trace.record_failure(&failure);
            return finalize_rejected_turn(
                writer,
                RejectedTurnDraft {
                    request: draft.request,
                    context: draft.context,
                    created_at: draft.created_at,
                    started_at: draft.started_at,
                    finished_at: draft.finished_at,
                    exit_status: draft.exit_status,
                    working_hash_before: draft.working_hash_before,
                    query_summary: Some(draft.query_summary),
                    failure,
                    policy_trace,
                    stdout: draft.stdout,
                    stderr: draft.stderr,
                },
            );
        }
    };

    let accepted_commit = writer.commit_accepted(&AcceptedLedgerInput {
        request: draft.request.clone(),
        context: draft.context.clone(),
        created_at: draft.created_at.clone(),
        started_at: draft.started_at.clone(),
        finished_at: draft.finished_at.clone(),
        exit_status: draft.exit_status,
        working_hash_before: draft.working_hash_before.clone(),
        working_output: draft.working_output.clone(),
        prior_working: draft.prior_working.clone(),
        query_summary: draft.query_summary.clone(),
        bundle: citation.bundle.clone(),
        rr: citation.rr.clone(),
        policy_trace: policy_trace.clone(),
        stdout: draft.stdout.clone(),
        stderr: draft.stderr.clone(),
    });

    let accepted_commit = match accepted_commit {
        Ok(committed) => committed,
        Err(_) => {
            let failure = FailureSpec::ledger_commit_failed(
                RuleId::parse(RULE_LEDGER_COMMIT_FAILED).expect("static rule_id must be valid"),
                "ledger commit failed before evidence became visible",
                "ledger path と atomic commit 条件を確認して再実行する",
            )?;
            policy_trace.record_failure(&failure);
            return finalize_rejected_turn(
                writer,
                RejectedTurnDraft {
                    request: draft.request,
                    context: draft.context,
                    created_at: draft.created_at,
                    started_at: draft.started_at,
                    finished_at: draft.finished_at,
                    exit_status: draft.exit_status,
                    working_hash_before: draft.working_hash_before,
                    query_summary: Some(draft.query_summary),
                    failure,
                    policy_trace,
                    stdout: draft.stdout,
                    stderr: draft.stderr,
                },
            );
        }
    };

    if let Err(error) =
        write_working_projection(writer.cyrune_home(), &draft.working_output.projection)
    {
        let failure = FailureSpec::working_update_failed(
            RuleId::parse(RULE_WORKING_UPDATE_FAILED).expect("static rule_id must be valid"),
            format!("working projection update failed after ledger commit: {error}"),
            "working/working.json を更新可能な状態にして再実行する",
        )?;
        return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
    }

    let working_json_hash = match visible_working_hash(writer.cyrune_home()) {
        Ok(hash) => hash,
        Err(error) => {
            let failure = FailureSpec::working_update_failed(
                RuleId::parse(RULE_WORKING_UPDATE_FAILED).expect("static rule_id must be valid"),
                format!("visible working hash read failed after ledger commit: {error}"),
                "working/working.json を読み取り可能な状態にして再実行する",
            )?;
            return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
        }
    };
    if working_json_hash != draft.working_output.working_hash {
        let failure = FailureSpec::working_update_failed(
            RuleId::parse(RULE_WORKING_HASH_MISMATCH).expect("static rule_id must be valid"),
            format!(
                "visible working hash mismatch after ledger commit: expected {}, got {}",
                draft.working_output.working_hash, working_json_hash
            ),
            "working/working.json を accepted response と同一 raw hash にして再実行する",
        )?;
        return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
    }

    let evidence_manifest_hash =
        match raw_file_sha256(&accepted_commit.evidence_dir.join("manifest.json")) {
            Ok(hash) => hash,
            Err(error) => {
                let failure = FailureSpec::ledger_commit_failed(
                    RuleId::parse(RULE_TERMINAL_BINDING_FAILED)
                        .expect("static rule_id must be valid"),
                    format!("terminal binding preparation failed reading manifest: {error}"),
                    "ledger/evidence の accepted manifest を読み取り可能な状態にして再実行する",
                )?;
                return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
            }
        };
    let evidence_hashes_hash =
        match raw_file_sha256(&accepted_commit.evidence_dir.join("hashes.json")) {
            Ok(hash) => hash,
            Err(error) => {
                let failure = FailureSpec::ledger_commit_failed(
                    RuleId::parse(RULE_TERMINAL_BINDING_FAILED)
                        .expect("static rule_id must be valid"),
                    format!("terminal binding preparation failed reading hashes: {error}"),
                    "ledger/evidence の accepted hashes を読み取り可能な状態にして再実行する",
                )?;
                return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
            }
        };
    let terminal_binding = TerminalBindingRecord {
        schema_version: TERMINAL_BINDING_SCHEMA_VERSION.to_string(),
        outcome: EvidenceOutcome::Accepted,
        response_to: draft.request.request_id.clone(),
        correlation_id: draft.context.correlation_id.clone(),
        run_id: draft.context.run_id.clone(),
        evidence_id: accepted_commit.evidence_id.clone(),
        policy_pack_id: draft.context.policy_pack_id.clone(),
        citation_bundle_id: citation.bundle.bundle_id.clone(),
        working_hash_after: working_json_hash.clone(),
        evidence_manifest_hash,
        evidence_hashes_hash,
        working_json_hash: working_json_hash.clone(),
        created_at: timestamp_marker_now(),
    };
    if let Err(error) = write_terminal_binding(writer.cyrune_home(), &terminal_binding) {
        let failure = FailureSpec::ledger_commit_failed(
            RuleId::parse(RULE_TERMINAL_BINDING_FAILED).expect("static rule_id must be valid"),
            format!("terminal binding marker write failed after working update: {error}"),
            "ledger/terminal-bindings を更新可能な状態にして再実行する",
        )?;
        return finalize_post_commit_rejection(writer, draft, policy_trace, failure);
    }

    Ok(Ok(RunAccepted {
        outcome: RunOutcome::Accepted,
        response_to: draft.request.request_id,
        correlation_id: draft.context.correlation_id,
        run_id: draft.context.run_id,
        evidence_id: accepted_commit.evidence_id,
        output: draft.output_draft,
        citation_bundle_id: citation.bundle.bundle_id,
        working_hash_after: working_json_hash,
        policy_pack_id: draft.context.policy_pack_id,
    }))
}

fn finalize_post_commit_rejection(
    writer: &mut LedgerWriter,
    draft: AcceptedTurnDraft,
    mut policy_trace: PolicyTrace,
    failure: FailureSpec,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    policy_trace.record_failure(&failure);
    finalize_rejected_turn(
        writer,
        RejectedTurnDraft {
            request: draft.request,
            context: draft.context,
            created_at: draft.created_at,
            started_at: draft.started_at,
            finished_at: draft.finished_at,
            exit_status: draft.exit_status,
            working_hash_before: draft.working_hash_before,
            query_summary: Some(draft.query_summary),
            failure,
            policy_trace,
            stdout: draft.stdout,
            stderr: draft.stderr,
        },
    )
}

pub fn finalize_rejected_turn(
    writer: &mut LedgerWriter,
    draft: RejectedTurnDraft,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    let committed = writer.commit_rejected(&RejectedLedgerInput {
        request: draft.request.clone(),
        context: draft.context.clone(),
        created_at: draft.created_at,
        started_at: draft.started_at,
        finished_at: draft.finished_at,
        exit_status: draft.exit_status,
        working_hash_before: draft.working_hash_before,
        query_summary: draft.query_summary,
        failure: draft.failure.clone(),
        policy_trace: draft.policy_trace,
        stdout: draft.stdout,
        stderr: draft.stderr,
    })?;
    Ok(Err(RunRejected {
        outcome: RunOutcome::Rejected,
        response_to: draft.request.request_id,
        correlation_id: draft.context.correlation_id,
        run_id: draft.context.run_id,
        denial_id: DenialId::from_evidence_id(&committed.evidence_id),
        evidence_id: committed.evidence_id,
        rule_id: draft.failure.rule_id,
        reason_kind: draft.failure.reason_kind,
        message: draft.failure.message,
        remediation: draft.failure.remediation,
    }))
}

pub fn finalize_execution_failure_turn(
    writer: &mut LedgerWriter,
    request: RunRequest,
    context: crate::resolved_turn_context::ResolvedTurnContext,
    working_hash_before: String,
    query_summary: Option<QuerySummary>,
    envelope: &ExecutionResultEnvelope,
    mut policy_trace: PolicyTrace,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    envelope.validate()?;
    if envelope.terminal_status == TerminalStatus::Succeeded {
        return Err(TurnError::ExecutionResult(
            crate::execution_result::ExecutionResultError::Invalid(
                "finalize_execution_failure_turn requires non-succeeded terminal_status"
                    .to_string(),
            ),
        ));
    }
    let failure_message = match envelope.terminal_status {
        TerminalStatus::Failed => "execution adapter reported terminal failure",
        TerminalStatus::TimedOut => "execution adapter timed out",
        TerminalStatus::Cancelled => "execution adapter cancelled",
        TerminalStatus::Succeeded => unreachable!("guarded above"),
    };
    let failure = FailureSpec::execution_failed(
        cyrune_core_contract::RuleId::parse("EXE-001").expect("static rule_id must be valid"),
        failure_message,
        "execution adapter の失敗原因、timeout、cancel 条件を修正して再実行する",
    )?;
    policy_trace.record_failure(&failure);
    finalize_rejected_turn(
        writer,
        RejectedTurnDraft {
            request,
            context,
            created_at: envelope.finished_at.clone(),
            started_at: envelope.started_at.clone(),
            finished_at: envelope.finished_at.clone(),
            exit_status: envelope.exit_status,
            working_hash_before,
            query_summary,
            failure,
            policy_trace,
            stdout: envelope.stdio.stdout.clone(),
            stderr: envelope.stdio.stderr.clone(),
        },
    )
}

pub fn first_terminal_failure_wins(failures: Vec<FailureSpec>) -> Option<FailureSpec> {
    choose_first_terminal_failure(failures)
}

pub fn run_no_llm_accepted_path(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    draft: &NoLlmAcceptedDraft,
    now_ms: u64,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    draft.validate()?;
    let context = resolve_turn_context(request, resolver_inputs)?;
    let prior_working = match load_prior_working_projection(
        writer.cyrune_home(),
        &context.correlation_id,
        &draft.started_at,
    ) {
        Ok(projection) => projection,
        Err(load_failure) => {
            let working_hash_before = working_hash_before_for_failure(
                load_failure.projection.as_ref(),
                &context.correlation_id,
                &draft.started_at,
            )?;
            return reject_invalid_prior_working(
                writer,
                request,
                context,
                &draft.started_at,
                &draft.finished_at,
                working_hash_before,
                load_failure.failure,
            );
        }
    };
    let working_hash_before = working_hash(&prior_working)?;
    let memory = MemoryFacade::new(&context)?;
    let retrieval = select_candidates(&memory, &context, now_ms, &request.user_input)?;
    let working_output = rebuild_working(&WorkingRebuildInput {
        generated_at: draft.finished_at.clone(),
        correlation_id: context.correlation_id.clone(),
        prior_working: Some(prior_working.clone()),
        candidates: build_no_llm_working_candidates(
            request,
            &retrieval,
            &prior_working,
            &draft.finished_at,
        ),
    })?;

    finalize_accepted_turn(
        writer,
        AcceptedTurnDraft {
            request: request.clone(),
            context,
            created_at: draft.finished_at.clone(),
            started_at: draft.started_at.clone(),
            finished_at: draft.finished_at.clone(),
            exit_status: Some(0),
            working_hash_before,
            prior_working: Some(prior_working),
            working_output,
            query_summary: retrieval.query_summary,
            output_draft: draft.output_draft.clone(),
            citation_material: draft.citation_material.clone(),
            rr_material: draft.rr_material.clone(),
            stdout: draft.stdio.stdout.clone(),
            stderr: draft.stdio.stderr.clone(),
        },
    )
}

pub fn run_approved_execution_adapter_path(
    writer: &mut LedgerWriter,
    resolver_inputs: &ResolverInputs,
    request: &RunRequest,
    now_ms: u64,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    let context = resolve_turn_context(request, resolver_inputs)?;
    let provisional_started_at = timestamp_marker_now();
    let prior_working = match load_prior_working_projection(
        writer.cyrune_home(),
        &context.correlation_id,
        &provisional_started_at,
    ) {
        Ok(projection) => projection,
        Err(load_failure) => {
            let working_hash_before = working_hash_before_for_failure(
                load_failure.projection.as_ref(),
                &context.correlation_id,
                &provisional_started_at,
            )?;
            return reject_invalid_prior_working(
                writer,
                request,
                context,
                &provisional_started_at,
                &provisional_started_at,
                working_hash_before,
                load_failure.failure,
            );
        }
    };
    let working_hash_before = working_hash(&prior_working)?;
    let memory = MemoryFacade::new(&context)?;
    let retrieval = select_candidates(&memory, &context, now_ms, &request.user_input)?;
    let working_output = rebuild_working(&WorkingRebuildInput {
        generated_at: provisional_started_at.clone(),
        correlation_id: context.correlation_id.clone(),
        prior_working: Some(prior_working.clone()),
        candidates: build_no_llm_working_candidates(
            request,
            &retrieval,
            &prior_working,
            &provisional_started_at,
        ),
    })?;
    let precheck = evaluate_precheck(
        request,
        &context,
        &working_output.projection,
        &working_output.working_hash,
    )?;
    if let Some(failure) = precheck.terminal_failure {
        return finalize_rejected_turn(
            writer,
            RejectedTurnDraft {
                request: request.clone(),
                context,
                created_at: provisional_started_at.clone(),
                started_at: provisional_started_at.clone(),
                finished_at: provisional_started_at,
                exit_status: None,
                working_hash_before,
                query_summary: Some(retrieval.query_summary),
                failure,
                policy_trace: precheck.policy_trace,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
    }

    let _sandbox = match normalize_sandbox_spec(&context, request, &[], &[]) {
        Ok(spec) => spec,
        Err(error) => {
            let mut policy_trace = precheck.policy_trace;
            let failure = FailureSpec::new(
                FailureStage::RequestValidation,
                RuleId::parse("REQ-003").expect("static rule_id must be valid"),
                ReasonKind::InvalidRequest,
                format!("sandbox normalization failed: {error}"),
                "absolute cwd と allowlisted env_overrides を指定して再実行する",
            )?;
            policy_trace.record_failure(&failure);
            return finalize_rejected_turn(
                writer,
                RejectedTurnDraft {
                    request: request.clone(),
                    context,
                    created_at: provisional_started_at.clone(),
                    started_at: provisional_started_at.clone(),
                    finished_at: provisional_started_at,
                    exit_status: None,
                    working_hash_before,
                    query_summary: Some(retrieval.query_summary),
                    failure,
                    policy_trace,
                    stdout: String::new(),
                    stderr: String::new(),
                },
            );
        }
    };

    let mut policy_trace = precheck.policy_trace;
    let selected = context
        .selected_execution_adapter
        .as_ref()
        .expect("execution adapter path requires selected adapter")
        .clone();

    match invoke_approved_execution_adapter(
        &resolver_inputs.bundle_root,
        &selected,
        request,
        &context,
    )? {
        Ok(envelope) => {
            envelope.validate_against_selected(&context.correlation_id, &selected)?;
            match envelope.terminal_status {
                TerminalStatus::Succeeded => finalize_accepted_turn(
                    writer,
                    AcceptedTurnDraft {
                        request: request.clone(),
                        context,
                        created_at: envelope.finished_at.clone(),
                        started_at: envelope.started_at.clone(),
                        finished_at: envelope.finished_at.clone(),
                        exit_status: envelope.exit_status,
                        working_hash_before,
                        prior_working: Some(prior_working),
                        working_output,
                        query_summary: retrieval.query_summary,
                        output_draft: envelope
                            .output_draft
                            .expect("validated succeeded envelope must carry output"),
                        citation_material: envelope
                            .citation_material
                            .expect("validated succeeded envelope must carry citations"),
                        rr_material: envelope
                            .rr_material
                            .expect("validated succeeded envelope must carry rr"),
                        stdout: envelope.stdio.stdout,
                        stderr: envelope.stdio.stderr,
                    },
                ),
                TerminalStatus::Failed | TerminalStatus::TimedOut | TerminalStatus::Cancelled => {
                    finalize_execution_failure_turn(
                        writer,
                        request.clone(),
                        context,
                        working_hash_before,
                        Some(retrieval.query_summary),
                        &envelope,
                        policy_trace,
                    )
                }
            }
        }
        Err(invocation_failure) => {
            let failure = FailureSpec::execution_failed(
                RuleId::parse("EXE-002").expect("static rule_id must be valid"),
                format!(
                    "approved execution adapter failed before validated envelope: {}",
                    invocation_failure.detail
                ),
                "approved execution adapter launcher と envelope 出力を修正して再実行する",
            )?;
            policy_trace.record_failure(&failure);
            finalize_rejected_turn(
                writer,
                RejectedTurnDraft {
                    request: request.clone(),
                    context,
                    created_at: invocation_failure.finished_at.clone(),
                    started_at: invocation_failure.started_at,
                    finished_at: invocation_failure.finished_at,
                    exit_status: invocation_failure.exit_status,
                    working_hash_before,
                    query_summary: Some(retrieval.query_summary),
                    failure,
                    policy_trace,
                    stdout: invocation_failure.stdout,
                    stderr: invocation_failure.stderr,
                },
            )
        }
    }
}

fn invoke_approved_execution_adapter(
    bundle_root: &std::path::Path,
    selected: &crate::resolved_turn_context::SelectedExecutionAdapter,
    request: &RunRequest,
    context: &crate::resolved_turn_context::ResolvedTurnContext,
) -> Result<Result<ExecutionResultEnvelope, AdapterInvocationFailure>, TurnError> {
    let started_at = timestamp_marker_now();
    let materialized = match materialize_launcher(bundle_root, selected) {
        Ok(materialized) => materialized,
        Err(error) => {
            let finished_at = timestamp_marker_now();
            return Ok(Err(AdapterInvocationFailure {
                started_at,
                finished_at,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
                detail: format!("launcher materialization failed: {error}"),
            }));
        }
    };
    Ok(Ok(execute_local_cli_single_process(
        context,
        request,
        &materialized.launcher_path,
    )))
}

fn build_no_llm_working_candidates(
    request: &RunRequest,
    retrieval: &RetrievalSelectionResult,
    prior_working: &WorkingProjection,
    updated_at: &str,
) -> Vec<WorkingCandidate> {
    let mut candidates = vec![
        WorkingCandidate {
            category: WorkingCandidateCategory::PolicyConstraint,
            kind: WorkingSlotKind::Constraint,
            text: "free fail-closed gate remains active".to_string(),
            source_evidence_id: "EVID-0".to_string(),
            source_layer: crate::memory::SourceLayer::Processing,
            updated_at: updated_at.to_string(),
            updated_at_unix_ms: 1,
        },
        WorkingCandidate {
            category: WorkingCandidateCategory::RequestConstraint,
            kind: WorkingSlotKind::Context,
            text: request.user_input.trim().to_string(),
            source_evidence_id: "EVID-0".to_string(),
            source_layer: crate::memory::SourceLayer::Processing,
            updated_at: updated_at.to_string(),
            updated_at_unix_ms: 2,
        },
    ];
    candidates.extend(carry_forward_candidates(prior_working));
    for (index, candidate) in retrieval.final_candidates.iter().enumerate() {
        candidates.push(WorkingCandidate {
            category: WorkingCandidateCategory::RetrievalSupport,
            kind: WorkingSlotKind::Context,
            text: candidate.text.clone(),
            source_evidence_id: candidate
                .source_evidence_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "EVID-0".to_string()),
            source_layer: candidate.source_layer,
            updated_at: candidate.updated_at.clone(),
            updated_at_unix_ms: candidate.updated_at_unix_ms + u64::try_from(index).unwrap_or(0),
        });
    }
    candidates
}

fn carry_forward_candidates(prior_working: &WorkingProjection) -> Vec<WorkingCandidate> {
    prior_working
        .slots
        .iter()
        .map(|slot| WorkingCandidate {
            category: WorkingCandidateCategory::CarryForward,
            kind: slot.kind,
            text: slot.text.clone(),
            source_evidence_id: slot.source_evidence_id.clone(),
            source_layer: slot.source_layer,
            updated_at: slot.updated_at.clone(),
            updated_at_unix_ms: updated_at_unix_ms(&slot.updated_at)
                .expect("validated prior working slot must keep parseable updated_at"),
        })
        .collect()
}

fn empty_working_projection(
    correlation_id: &cyrune_core_contract::CorrelationId,
    generated_at: &str,
) -> WorkingProjection {
    WorkingProjection {
        version: 1,
        generated_at: generated_at.to_string(),
        correlation_id: correlation_id.clone(),
        limit: 12,
        slots: Vec::new(),
    }
}

fn working_hash(projection: &WorkingProjection) -> Result<String, TurnError> {
    let bytes = projection.canonical_json_bytes()?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn timestamp_marker_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("current UTC time must be formattable as RFC3339")
}

fn load_prior_working_projection(
    cyrune_home: &Path,
    fallback_correlation_id: &cyrune_core_contract::CorrelationId,
    generated_at: &str,
) -> Result<WorkingProjection, Box<PriorWorkingLoadFailure>> {
    let path = cyrune_home.join("working").join("working.json");
    if !path.exists() {
        return Ok(empty_working_projection(
            fallback_correlation_id,
            generated_at,
        ));
    }
    let bytes = fs::read(&path).map_err(|error| {
        Box::new(PriorWorkingLoadFailure {
            failure: working_invalid_failure(format!(
                "failed to read prior working projection: {error}"
            )),
            projection: None,
        })
    })?;
    let projection: WorkingProjection = serde_json::from_slice(&bytes).map_err(|error| {
        Box::new(PriorWorkingLoadFailure {
            failure: working_invalid_failure(format!(
                "failed to parse prior working projection: {error}"
            )),
            projection: None,
        })
    })?;
    if let Err(failure) = validate_loaded_prior_working_projection(&projection) {
        return Err(Box::new(PriorWorkingLoadFailure {
            failure,
            projection: Some(projection),
        }));
    }
    Ok(projection)
}

fn validate_loaded_prior_working_projection(
    projection: &WorkingProjection,
) -> Result<(), FailureSpec> {
    if projection.version != 1 || projection.limit != 12 || projection.slots.len() > 12 {
        return Err(working_invalid_failure(
            "prior working projection is outside Free v0.1 constraints",
        ));
    }
    if projection.generated_at.trim().is_empty() {
        return Err(working_invalid_failure(
            "prior working projection requires generated_at",
        ));
    }
    let mut slot_ids = BTreeSet::new();
    for slot in &projection.slots {
        if slot.text.trim().is_empty() || slot.source_evidence_id.trim().is_empty() {
            return Err(working_invalid_failure(
                "prior working projection contains slot without text/source_evidence_id",
            ));
        }
        if !slot_ids.insert(slot.slot_id.clone()) {
            return Err(working_invalid_failure(
                "prior working projection contains duplicated slot_id",
            ));
        }
        updated_at_unix_ms(&slot.updated_at)?;
    }
    Ok(())
}

fn updated_at_unix_ms(updated_at: &str) -> Result<u64, FailureSpec> {
    let parsed = OffsetDateTime::parse(updated_at, &Rfc3339).map_err(|error| {
        working_invalid_failure(format!(
            "prior working projection contains non-RFC3339 updated_at: {error}"
        ))
    })?;
    u64::try_from(parsed.unix_timestamp_nanos() / 1_000_000).map_err(|_| {
        working_invalid_failure("prior working projection updated_at must be >= unix epoch")
    })
}

fn working_hash_before_for_failure(
    projection: Option<&WorkingProjection>,
    fallback_correlation_id: &cyrune_core_contract::CorrelationId,
    generated_at: &str,
) -> Result<String, TurnError> {
    match projection {
        Some(projection) => working_hash(projection),
        None => working_hash(&empty_working_projection(
            fallback_correlation_id,
            generated_at,
        )),
    }
}

fn reject_invalid_prior_working(
    writer: &mut LedgerWriter,
    request: &RunRequest,
    context: crate::resolved_turn_context::ResolvedTurnContext,
    started_at: &str,
    finished_at: &str,
    working_hash_before: String,
    failure: FailureSpec,
) -> Result<Result<RunAccepted, RunRejected>, TurnError> {
    let mut policy_trace = PolicyTrace::new();
    policy_trace.record_failure(&failure);
    finalize_rejected_turn(
        writer,
        RejectedTurnDraft {
            request: request.clone(),
            context,
            created_at: finished_at.to_string(),
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            exit_status: None,
            working_hash_before,
            query_summary: None,
            failure,
            policy_trace,
            stdout: String::new(),
            stderr: String::new(),
        },
    )
}

fn working_invalid_failure(message: impl Into<String>) -> FailureSpec {
    FailureSpec::working_invalid(
        RuleId::parse("WRK-001").expect("static rule_id must be valid"),
        message,
        "working projection を 12 slot 以下・version=1・RFC3339 updated_at 付きへ正規化して再実行する",
    )
    .expect("static failure spec must be valid")
}

#[cfg(test)]
mod tests {
    use super::{
        AcceptedTurnDraft, empty_working_projection, finalize_accepted_turn,
        first_terminal_failure_wins, run_approved_execution_adapter_path, run_no_llm_accepted_path,
    };
    use crate::citation::{
        CitationMaterial, CitationMaterialClaim, ClaimKind, EvidenceRef, SimpleReasoningRecord,
    };
    use crate::execution_result::{NoLlmAcceptedDraft, StdioCapture, TerminalStatus};
    use crate::ledger::LedgerWriter;
    use crate::policy::{FailureSpec, PolicyTrace};
    use crate::resolved_turn_context::{
        ResolvedKernelAdapters, ResolvedTurnContext, TimeoutPolicy,
    };
    use crate::resolver::ResolverInputs;
    use crate::retrieval::QuerySummary;
    use crate::working::{
        WorkingCandidate, WorkingCandidateCategory, WorkingProjection, WorkingRebuildInput,
        WorkingSlotKind, rebuild_working,
    };
    use cyrune_core_contract::{
        CorrelationId, IoMode, PathLabel, RequestId, RuleId, RunKind, RunRequest,
    };
    use serde_json::Value;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn request() -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0501").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0501").unwrap(),
            run_kind: RunKind::NoLlm,
            user_input: "turn".to_string(),
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

    fn execution_request(cwd: &std::path::Path) -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0502").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0502").unwrap(),
            run_kind: RunKind::ExecutionAdapter,
            user_input: "adapter turn".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: Some("local-cli-single-process.v0.1".to_string()),
            argv: None,
            cwd: Some(PathLabel::parse(cwd.display().to_string()).unwrap()),
            env_overrides: None,
        }
    }

    fn vendored_adapter_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for ancestor in manifest_dir.ancestors() {
            let candidate = ancestor.join("Adapter").join("v0.1").join("0");
            if candidate.join("catalog").exists()
                && candidate
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json")
                    .exists()
                && candidate
                    .join("bindings")
                    .join("cyrune-free-default.v0.1.json")
                    .exists()
            {
                return candidate;
            }
        }
        panic!("vendored Adapter/v0.1/0 could not be derived from CARGO_MANIFEST_DIR");
    }

    fn resolver_inputs_with_vendored_adapter(bundle_root: &Path) -> ResolverInputs {
        let adapter_root = vendored_adapter_root();
        ResolverInputs::new(
            bundle_root,
            adapter_root.join("catalog"),
            adapter_root
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            adapter_root
                .join("bindings")
                .join("cyrune-free-default.v0.1.json"),
        )
    }

    fn context() -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0501").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0501").unwrap(),
            run_id: cyrune_core_contract::RunId::parse("RUN-20260327-0501-R01").unwrap(),
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

    fn write_execution_registry(
        bundle_root: &std::path::Path,
        launcher_path: &str,
        launcher_sha256: &str,
    ) {
        let approved_dir = bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved");
        let profiles_dir = approved_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();
        fs::write(
            approved_dir.join("registry.json"),
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
        )
        .unwrap();
        fs::write(
            profiles_dir.join("local-cli-single-process.v0.1.json"),
            format!(
                r#"{{
  "adapter_id": "local-cli-single-process.v0.1",
  "adapter_version": "0.1.0",
  "execution_kind": "process_stdio",
  "launcher_path": "{launcher_path}",
  "launcher_sha256": "{launcher_sha256}",
  "model_id": "model.local",
  "model_revision_or_digest": "sha256:model",
  "allowed_capabilities": ["exec", "fs_read"],
  "default_timeout_s": 120,
  "env_allowlist": []
}}"#
            ),
        )
        .unwrap();
    }

    fn write_success_launcher(path: &std::path::Path) -> String {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            path,
            r#"#!/bin/sh
PIN_FILE="${0}.sha256"
PIN="$(cat "${PIN_FILE}")"
cat >/dev/null
printf '%s\n' "{\"adapter_id\":\"local-cli-single-process.v0.1\",\"adapter_version\":\"0.1.0\",\"correlation_id\":\"RUN-20260327-0502\",\"terminal_status\":\"succeeded\",\"started_at\":\"2026-03-27T12:00:00+09:00\",\"finished_at\":\"2026-03-27T12:00:01+09:00\",\"exit_status\":0,\"output_draft\":\"- adapter claim\",\"stdio\":{\"stdout\":\"adapter stdout\",\"stderr\":\"\"},\"pin\":{\"kind\":\"launcher_sha256\",\"value\":\"${PIN}\"},\"citation_material\":{\"claims\":[{\"text\":\"adapter claim\",\"claim_kind\":\"extractive\",\"evidence_refs\":[{\"evidence_id\":\"EVID-1\"}]}]},\"rr_material\":{\"claims\":[\"adapter claim\"],\"decisions\":[],\"assumptions\":[],\"actions\":[],\"citations_used\":[\"EVID-1\"]},\"failure_detail\":null}"
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
        let launcher_sha256 = format!("sha256:{}", super::sha256_hex(&fs::read(path).unwrap()));
        fs::write(
            path.with_extension("sh.sha256"),
            format!("{launcher_sha256}\n"),
        )
        .unwrap();
        launcher_sha256
    }

    fn working_output() -> crate::working::WorkingRebuildOutput {
        rebuild_working(&WorkingRebuildInput {
            generated_at: "2026-03-27T12:00:00+09:00".to_string(),
            correlation_id: CorrelationId::parse("RUN-20260327-0501").unwrap(),
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

    fn write_prior_working(cyrune_home: &std::path::Path, projection: &WorkingProjection) {
        fs::create_dir_all(cyrune_home.join("working")).unwrap();
        fs::write(
            cyrune_home.join("working").join("working.json"),
            projection.canonical_json_bytes().unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn uncited_claim_rejects() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: working_output(),
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- uncited claim".to_string(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "uncited claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: Vec::new(),
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["uncited claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: Vec::new(),
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::CitationDenied
        );
    }

    #[test]
    fn ledger_commit_failure_is_mapped_to_rejection() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::with_failures(temp.path(), 1);
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: working_output(),
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- claim".to_string(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::LedgerCommitFailed
        );
        assert_eq!(rejection.rule_id.as_str(), "LDG-001");
    }

    #[test]
    fn first_failure_helper_prefers_earlier_stage() {
        let chosen = first_terminal_failure_wins(vec![
            FailureSpec::citation_denied(RuleId::parse("CIT-001").unwrap(), "late", "fix late")
                .unwrap(),
            FailureSpec::policy_denied(RuleId::parse("POL-001").unwrap(), "early", "fix early")
                .unwrap(),
        ])
        .unwrap();
        assert_eq!(chosen.rule_id.as_str(), "POL-001");
    }

    #[test]
    fn deny_by_default_policy_reject_is_closed() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let denied_request = request();
        let mut denied_context = context();
        denied_context.policy_pack_id = "different-pack".to_string();
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: denied_request,
                context: denied_context,
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: working_output(),
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- claim".to_string(),
                stdout: String::new(),
                stderr: String::new(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::PolicyDenied
        );
    }

    #[test]
    fn binding_unresolved_reject_is_closed() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let mut adapter_request = request();
        adapter_request.run_kind = RunKind::ExecutionAdapter;
        adapter_request.adapter_id = Some("requested-adapter".to_string());
        let mut execution_context = context();
        execution_context.run_kind = RunKind::ExecutionAdapter;
        execution_context.selected_execution_adapter =
            Some(crate::resolved_turn_context::SelectedExecutionAdapter {
                adapter_id: "resolved-adapter".to_string(),
                adapter_version: "0.1.0".to_string(),
                execution_kind: "process_stdio".to_string(),
                launcher_path: "/bin/sh".to_string(),
                launcher_sha256: "sha256:launcher".to_string(),
                model_id: "model".to_string(),
                model_revision_or_digest: "sha256:model".to_string(),
                default_timeout_s: 60,
                allowed_capabilities: vec!["fs_read".to_string()],
                env_allowlist: Vec::new(),
            });
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: adapter_request,
                context: execution_context,
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: working_output(),
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- claim".to_string(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::BindingUnresolved
        );
    }

    #[test]
    fn working_limit_reject_is_closed() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let mut oversized = working_output();
        oversized.projection.slots = (1..=13)
            .map(|index| crate::working::WorkingSlot {
                slot_id: cyrune_core_contract::SlotId::parse(format!("W-{index:03}")).unwrap(),
                kind: WorkingSlotKind::Constraint,
                text: format!("slot {index}"),
                source_evidence_id: "EVID-1".to_string(),
                source_layer: crate::memory::SourceLayer::Processing,
                priority: 1000,
                updated_at: "2026-03-27T12:00:00+09:00".to_string(),
            })
            .collect();
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: oversized,
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- claim".to_string(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::WorkingInvalid
        );
    }

    #[test]
    fn execution_failed_reject_is_closed() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let result = super::finalize_execution_failure_turn(
            &mut writer,
            request(),
            context(),
            format!("sha256:{}", "0".repeat(64)),
            Some(QuerySummary {
                query_hash: format!("sha256:{}", "1".repeat(64)),
                selected_memory_ids: vec!["MEM-1".to_string()],
                rejected_reasons: Vec::new(),
            }),
            &crate::execution_result::ExecutionResultEnvelope {
                adapter_id: "local-cli-single-process.v0.1".to_string(),
                adapter_version: "0.1.0".to_string(),
                correlation_id: CorrelationId::parse("RUN-20260327-0501").unwrap(),
                terminal_status: TerminalStatus::Failed,
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(1),
                output_draft: None,
                stdio: StdioCapture {
                    stdout: String::new(),
                    stderr: "failed".to_string(),
                },
                pin: crate::execution_result::ExecutionPin {
                    kind: "launcher_sha256".to_string(),
                    value: "sha256:abc".to_string(),
                },
                citation_material: None,
                rr_material: None,
                failure_detail: Some("failed".to_string()),
            },
            PolicyTrace::new(),
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::ExecutionFailed
        );
    }

    #[test]
    fn working_update_failure_is_closed() {
        let temp = tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("working").join("working.json")).unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let result = finalize_accepted_turn(
            &mut writer,
            AcceptedTurnDraft {
                request: request(),
                context: context(),
                created_at: "2026-03-27T12:00:01+09:00".to_string(),
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                exit_status: Some(0),
                working_hash_before: format!("sha256:{}", "0".repeat(64)),
                prior_working: None,
                working_output: working_output(),
                query_summary: QuerySummary {
                    query_hash: format!("sha256:{}", "1".repeat(64)),
                    selected_memory_ids: vec!["MEM-1".to_string()],
                    rejected_reasons: Vec::new(),
                },
                output_draft: "- claim".to_string(),
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
                stdout: String::new(),
                stderr: String::new(),
            },
        )
        .unwrap();
        let rejection = result.unwrap_err();
        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::WorkingUpdateFailed
        );
    }

    #[test]
    fn no_llm_path_returns_runaccepted_and_writes_evidence() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let request = request();
        let result = run_no_llm_accepted_path(
            &mut writer,
            &resolver_inputs_with_vendored_adapter(temp.path()),
            &request,
            &NoLlmAcceptedDraft {
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                output_draft: "- claim".to_string(),
                stdio: StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
            },
            1,
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.run_id.as_str(), "RUN-20260327-0501-R01");
        assert!(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(result.evidence_id.as_str())
                .exists()
        );
        assert!(temp.path().join("working").join("working.json").exists());
    }

    #[test]
    fn no_llm_path_uses_resolved_bringup_binding() {
        let temp = tempdir().unwrap();
        let mut writer = LedgerWriter::new(temp.path());
        let request = request();
        let accepted = run_no_llm_accepted_path(
            &mut writer,
            &resolver_inputs_with_vendored_adapter(temp.path()),
            &request,
            &NoLlmAcceptedDraft {
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                output_draft: "- claim".to_string(),
                stdio: StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
            },
            1,
        )
        .unwrap()
        .unwrap();
        let policy_json = std::fs::read_to_string(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(accepted.evidence_id.as_str())
                .join("policy.json"),
        )
        .unwrap();
        assert!(policy_json.contains("\"binding_id\": \"cyrune-free-default\""));
    }

    #[test]
    fn no_llm_path_rejects_invalid_prior_working_projection() {
        let temp = tempdir().unwrap();
        let mut oversized = empty_working_projection(
            &CorrelationId::parse("RUN-20260327-0600").unwrap(),
            "2026-03-27T11:59:59+09:00",
        );
        oversized.slots = (1..=13)
            .map(|index| crate::working::WorkingSlot {
                slot_id: cyrune_core_contract::SlotId::parse(format!("W-{index:03}")).unwrap(),
                kind: WorkingSlotKind::Constraint,
                text: format!("carry {index}"),
                source_evidence_id: format!("EVID-{index}"),
                source_layer: crate::memory::SourceLayer::Processing,
                priority: 700,
                updated_at: "2026-03-27T11:59:59+09:00".to_string(),
            })
            .collect();
        write_prior_working(temp.path(), &oversized);

        let mut writer = LedgerWriter::new(temp.path());
        let rejection = run_no_llm_accepted_path(
            &mut writer,
            &resolver_inputs_with_vendored_adapter(temp.path()),
            &request(),
            &NoLlmAcceptedDraft {
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                output_draft: "- claim".to_string(),
                stdio: StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
            },
            1,
        )
        .unwrap()
        .unwrap_err();

        assert_eq!(
            rejection.reason_kind,
            cyrune_core_contract::ReasonKind::WorkingInvalid
        );
        assert_eq!(rejection.rule_id.as_str(), "WRK-001");
    }

    #[test]
    fn no_llm_path_carries_forward_prior_working_slots() {
        let temp = tempdir().unwrap();
        let mut prior = empty_working_projection(
            &CorrelationId::parse("RUN-20260327-0601").unwrap(),
            "2026-03-27T11:59:59+09:00",
        );
        prior.slots.push(crate::working::WorkingSlot {
            slot_id: cyrune_core_contract::SlotId::parse("W-007").unwrap(),
            kind: WorkingSlotKind::Definition,
            text: "carry forward definition".to_string(),
            source_evidence_id: "EVID-77".to_string(),
            source_layer: crate::memory::SourceLayer::Processing,
            priority: 700,
            updated_at: "2026-03-27T11:59:59+09:00".to_string(),
        });
        write_prior_working(temp.path(), &prior);

        let mut writer = LedgerWriter::new(temp.path());
        let accepted = run_no_llm_accepted_path(
            &mut writer,
            &resolver_inputs_with_vendored_adapter(temp.path()),
            &request(),
            &NoLlmAcceptedDraft {
                started_at: "2026-03-27T12:00:00+09:00".to_string(),
                finished_at: "2026-03-27T12:00:01+09:00".to_string(),
                output_draft: "- claim".to_string(),
                stdio: StdioCapture {
                    stdout: String::new(),
                    stderr: String::new(),
                },
                citation_material: CitationMaterial {
                    claims: vec![CitationMaterialClaim {
                        text: "claim".to_string(),
                        claim_kind: ClaimKind::Extractive,
                        evidence_refs: vec![EvidenceRef {
                            evidence_id: "EVID-1".to_string(),
                        }],
                    }],
                },
                rr_material: SimpleReasoningRecord {
                    claims: vec!["claim".to_string()],
                    decisions: Vec::new(),
                    assumptions: Vec::new(),
                    actions: Vec::new(),
                    citations_used: vec!["EVID-1".to_string()],
                },
            },
            1,
        )
        .unwrap()
        .unwrap();

        let projection: crate::working::WorkingProjection = serde_json::from_slice(
            &fs::read(temp.path().join("working").join("working.json")).unwrap(),
        )
        .unwrap();
        assert!(projection.slots.iter().any(
            |slot| slot.slot_id.as_str() == "W-007" && slot.text == "carry forward definition"
        ));
        assert!(
            temp.path()
                .join("ledger")
                .join("evidence")
                .join(accepted.evidence_id.as_str())
                .exists()
        );
    }

    #[test]
    fn approved_execution_adapter_path_returns_runaccepted_and_preserves_pin_in_evidence() {
        let temp = tempdir().unwrap();
        let launcher_path = temp
            .path()
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let launcher_sha256 = write_success_launcher(&launcher_path);
        write_execution_registry(
            temp.path(),
            "runtime/ipc/local-cli-single-process.sh",
            &launcher_sha256,
        );

        let mut writer = LedgerWriter::new(temp.path());
        let request = execution_request(temp.path());
        let accepted = run_approved_execution_adapter_path(
            &mut writer,
            &resolver_inputs_with_vendored_adapter(temp.path()),
            &request,
            1,
        )
        .unwrap()
        .unwrap();

        let evidence_dir = temp
            .path()
            .join("ledger")
            .join("evidence")
            .join(accepted.evidence_id.as_str());
        assert!(evidence_dir.exists());
        assert!(temp.path().join("working").join("working.json").exists());

        let policy_json = fs::read_to_string(evidence_dir.join("policy.json")).unwrap();
        let policy_value: Value = serde_json::from_str(&policy_json).unwrap();
        assert_eq!(
            policy_value["selected_execution_adapter"]["adapter_id"],
            Value::String("local-cli-single-process.v0.1".to_string())
        );
        assert_eq!(
            policy_value["selected_execution_adapter"]["launcher_sha256"],
            Value::String(launcher_sha256)
        );
        assert_eq!(
            policy_value["selected_execution_adapter"]["model_revision_or_digest"],
            Value::String("sha256:model".to_string())
        );
    }
}
