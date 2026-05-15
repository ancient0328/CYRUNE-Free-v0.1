#![forbid(unsafe_code)]

use crate::resolved_turn_context::{EmbeddingExactPin, ResolvedTurnContext, SHIPPING_BINDING_ID};
use crate::resolver::shipping_embedding_engine_ref_for_pin;
use crate::working::WorkingSlotKind;
use crane_embed_null::NullEmbeddingEngine;
use crane_kernel::{
    ContentType, CorrelationId as KernelCorrelationId, EnvelopeId, Importance, KernelError,
    MemoryLayer, MemoryStore, OpCtx, Query, QueryFilters, QueryInput, QueryMode, RetentionHints,
    TypedEnvelope, TypedMetadata, UnixMs,
};
use crane_kernel_ref_inmem::InMemoryKernel;
use redb::{Database as RedbDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use stoolap::Database as StoolapDatabase;
use thiserror::Error;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};
use tract_onnx::prelude::*;

const BRINGUP_MEMORY_ADAPTER_ID: &str = "memory-kv-inmem";
const BRINGUP_EMBEDDING_ENGINE_REF: &str = "crane-embed-null.v0.1";
const SHIPPING_PROCESSING_ADAPTER_ID: &str = "memory-redb-processing";
const SHIPPING_PERMANENT_ADAPTER_ID: &str = "memory-stoolap-permanent";
const PROCESSING_TTL_MS: u64 = 3_628_800_000;
const PROCESSING_DATABASE_FILE: &str = "processing.redb";
const PERMANENT_DATABASE_FILE: &str = "permanent.stoolap.db";
const MATERIALIZED_SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH: &str =
    "embedding/exact-pins/cyrune-free-shipping.v0.1.json";
const MATERIALIZED_SHIPPING_ARTIFACT_PREFIX: &str = "embedding/artifacts";
const PROCESSING_RECORDS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("processing_records");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLayer {
    Working,
    Processing,
    Permanent,
}

impl SourceLayer {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Processing => "processing",
            Self::Permanent => "permanent",
        }
    }

    #[must_use]
    pub fn to_kernel(self) -> MemoryLayer {
        match self {
            Self::Working => MemoryLayer::Working,
            Self::Processing => MemoryLayer::Processing,
            Self::Permanent => MemoryLayer::Permanent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidityState {
    Valid,
    Superseded,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKeyspace {
    RunArtifacts,
    RetrievalCandidates,
    WorkingCandidates,
    RetentionIndex,
    KnowledgeRecords,
    Relations,
    LexicalIndex,
    ProvenanceIndex,
    ValidityIndex,
}

impl RecordKeyspace {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RunArtifacts => "run_artifacts",
            Self::RetrievalCandidates => "retrieval_candidates",
            Self::WorkingCandidates => "working_candidates",
            Self::RetentionIndex => "retention_index",
            Self::KnowledgeRecords => "knowledge_records",
            Self::Relations => "relations",
            Self::LexicalIndex => "lexical_index",
            Self::ProvenanceIndex => "provenance_index",
            Self::ValidityIndex => "validity_index",
        }
    }

    #[must_use]
    pub fn is_allowed_for(self, layer: SourceLayer) -> bool {
        match layer {
            SourceLayer::Working => false,
            SourceLayer::Processing => matches!(
                self,
                Self::RunArtifacts
                    | Self::RetrievalCandidates
                    | Self::WorkingCandidates
                    | Self::RetentionIndex
            ),
            SourceLayer::Permanent => matches!(
                self,
                Self::KnowledgeRecords
                    | Self::Relations
                    | Self::LexicalIndex
                    | Self::ProvenanceIndex
                    | Self::ValidityIndex
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Supersedes,
    Invalidates,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredRecord {
    pub record_id: String,
    pub source_layer: SourceLayer,
    pub keyspace: RecordKeyspace,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub created_at: String,
    pub created_at_unix_ms: u64,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub expires_at: Option<String>,
    pub expires_at_unix_ms: Option<u64>,
    pub validity_state: ValidityState,
    pub working_kind: Option<WorkingSlotKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationRecord {
    pub relation_id: String,
    pub relation_type: RelationType,
    pub source_evidence_ids: Vec<String>,
    pub subject_record_id: String,
    pub object_record_id: Option<String>,
    pub created_at: String,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationMarkInput {
    pub relation_id: String,
    pub subject_record_id: String,
    pub object_record_id: Option<String>,
    pub evidence_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessingRecordInput {
    pub keyspace: RecordKeyspace,
    pub record_id: String,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub created_at: String,
    pub created_at_unix_ms: u64,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub expires_at: String,
    pub expires_at_unix_ms: u64,
    pub working_kind: Option<WorkingSlotKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermanentRecordInput {
    pub keyspace: RecordKeyspace,
    pub record_id: String,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub created_at: String,
    pub created_at_unix_ms: u64,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub validity_state: ValidityState,
    pub working_kind: Option<WorkingSlotKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievedCandidateRecord {
    pub candidate_id: String,
    pub source_layer: SourceLayer,
    pub payload_ref: String,
    pub text: String,
    pub source_evidence_ids: Vec<String>,
    pub updated_at: String,
    pub updated_at_unix_ms: u64,
    pub expires_at_unix_ms: Option<u64>,
    pub validity_state: ValidityState,
    pub working_kind: Option<WorkingSlotKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RecordLocator {
    source_layer: SourceLayer,
    record_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "payload_kind", rename_all = "snake_case")]
enum StoredEnvelopePayload {
    Record(StoredRecord),
    Relation(RelationRecord),
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error(transparent)]
    Kernel(#[from] KernelError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(String),
}

struct RedbProcessingStore {
    db: RedbDatabase,
}

impl RedbProcessingStore {
    fn open(root: &Path) -> Result<Self, MemoryError> {
        fs::create_dir_all(root)?;
        let db_path = root.join(PROCESSING_DATABASE_FILE);
        let db = if db_path.exists() {
            RedbDatabase::open(&db_path).map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to open processing redb at {}: {error}",
                    db_path.display()
                ))
            })?
        } else {
            RedbDatabase::create(&db_path).map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to create processing redb at {}: {error}",
                    db_path.display()
                ))
            })?
        };
        let write_txn = db.begin_write().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to initialize processing redb transaction at {}: {error}",
                db_path.display()
            ))
        })?;
        {
            write_txn
                .open_table(PROCESSING_RECORDS_TABLE)
                .map_err(|error| {
                    MemoryError::Invalid(format!(
                        "failed to initialize processing redb table at {}: {error}",
                        db_path.display()
                    ))
                })?;
        }
        write_txn.commit().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to commit processing redb initialization at {}: {error}",
                db_path.display()
            ))
        })?;
        Ok(Self { db })
    }

    fn insert_record(&self, record: &StoredRecord) -> Result<(), MemoryError> {
        let payload = serde_json::to_vec(record)?;
        let write_txn = self.db.begin_write().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to begin processing redb write transaction: {error}"
            ))
        })?;
        {
            let mut table = write_txn
                .open_table(PROCESSING_RECORDS_TABLE)
                .map_err(|error| {
                    MemoryError::Invalid(format!(
                        "failed to open processing redb table for write: {error}"
                    ))
                })?;
            table
                .insert(record.record_id.as_str(), payload.as_slice())
                .map_err(|error| {
                    MemoryError::Invalid(format!(
                        "failed to insert processing record {} into redb: {error}",
                        record.record_id
                    ))
                })?;
        }
        write_txn.commit().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to commit processing redb write transaction: {error}"
            ))
        })?;
        Ok(())
    }

    fn get_record(&self, record_id: &str) -> Result<Option<StoredRecord>, MemoryError> {
        let read_txn = self.db.begin_read().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to begin processing redb read transaction: {error}"
            ))
        })?;
        let table = read_txn
            .open_table(PROCESSING_RECORDS_TABLE)
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to open processing redb table for read: {error}"
                ))
            })?;
        let Some(value) = table.get(record_id).map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to read processing record {record_id} from redb: {error}"
            ))
        })?
        else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_slice(value.value())?))
    }

    fn list_records(&self) -> Result<Vec<StoredRecord>, MemoryError> {
        let read_txn = self.db.begin_read().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to begin processing redb read transaction: {error}"
            ))
        })?;
        let table = read_txn
            .open_table(PROCESSING_RECORDS_TABLE)
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to open processing redb table for iteration: {error}"
                ))
            })?;
        let mut out = Vec::new();
        for entry in table.iter().map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to iterate processing redb records: {error}"
            ))
        })? {
            let (_, value) = entry.map_err(|error| {
                MemoryError::Invalid(format!("failed to decode processing redb row: {error}"))
            })?;
            out.push(serde_json::from_slice(value.value())?);
        }
        Ok(out)
    }
}

struct StoolapPermanentStore {
    db: StoolapDatabase,
}

impl StoolapPermanentStore {
    fn open(root: &Path) -> Result<Self, MemoryError> {
        fs::create_dir_all(root)?;
        let db_path = root.join(PERMANENT_DATABASE_FILE);
        let dsn = format!("file://{}", db_path.display());
        let db = StoolapDatabase::open(&dsn).map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to open permanent stoolap at {}: {error}",
                db_path.display()
            ))
        })?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS permanent_records (
                row_id INTEGER PRIMARY KEY AUTO_INCREMENT,
                record_id TEXT NOT NULL UNIQUE,
                keyspace TEXT NOT NULL,
                payload_ref TEXT NOT NULL,
                text TEXT NOT NULL,
                source_evidence_ids TEXT NOT NULL,
                created_at TEXT NOT NULL,
                created_at_unix_ms INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                updated_at_unix_ms INTEGER NOT NULL,
                validity_state TEXT NOT NULL,
                working_kind TEXT
            )",
            (),
        )
        .map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to initialize permanent stoolap records table at {}: {error}",
                db_path.display()
            ))
        })?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS permanent_relations (
                row_id INTEGER PRIMARY KEY AUTO_INCREMENT,
                relation_id TEXT NOT NULL UNIQUE,
                relation_type TEXT NOT NULL,
                source_evidence_ids TEXT NOT NULL,
                subject_record_id TEXT NOT NULL,
                object_record_id TEXT,
                created_at TEXT NOT NULL,
                created_at_unix_ms INTEGER NOT NULL
            )",
            (),
        )
        .map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to initialize permanent stoolap relations table at {}: {error}",
                db_path.display()
            ))
        })?;
        Ok(Self { db })
    }

    fn insert_record(&self, record: &StoredRecord) -> Result<(), MemoryError> {
        self.db
            .execute(
                "INSERT INTO permanent_records (
                    record_id,
                    keyspace,
                    payload_ref,
                    text,
                    source_evidence_ids,
                    created_at,
                    created_at_unix_ms,
                    updated_at,
                    updated_at_unix_ms,
                    validity_state,
                    working_kind
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                (
                    record.record_id.as_str(),
                    serde_json::to_string(&record.keyspace)?,
                    record.payload_ref.as_str(),
                    record.text.as_str(),
                    serde_json::to_string(&record.source_evidence_ids)?,
                    record.created_at.as_str(),
                    record.created_at_unix_ms as i64,
                    record.updated_at.as_str(),
                    record.updated_at_unix_ms as i64,
                    serde_json::to_string(&record.validity_state)?,
                    serde_json::to_string(&record.working_kind)?,
                ),
            )
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to insert permanent record {} into stoolap: {error}",
                    record.record_id
                ))
            })?;
        Ok(())
    }

    fn insert_relation(&self, relation: &RelationRecord) -> Result<(), MemoryError> {
        self.db
            .execute(
                "INSERT INTO permanent_relations (
                    relation_id,
                    relation_type,
                    source_evidence_ids,
                    subject_record_id,
                    object_record_id,
                    created_at,
                    created_at_unix_ms
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                (
                    relation.relation_id.as_str(),
                    serde_json::to_string(&relation.relation_type)?,
                    serde_json::to_string(&relation.source_evidence_ids)?,
                    relation.subject_record_id.as_str(),
                    relation.object_record_id.as_deref(),
                    relation.created_at.as_str(),
                    relation.created_at_unix_ms as i64,
                ),
            )
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to insert permanent relation {} into stoolap: {error}",
                    relation.relation_id
                ))
            })?;
        Ok(())
    }

    fn get_record(&self, record_id: &str) -> Result<Option<StoredRecord>, MemoryError> {
        let mut rows = self
            .db
            .query(
                "SELECT
                    record_id,
                    keyspace,
                    payload_ref,
                    text,
                    source_evidence_ids,
                    created_at,
                    created_at_unix_ms,
                    updated_at,
                    updated_at_unix_ms,
                    validity_state,
                    working_kind
                 FROM permanent_records
                 WHERE record_id = $1",
                (record_id,),
            )
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to query permanent record {record_id} from stoolap: {error}"
                ))
            })?;
        let Some(row) = rows.next() else {
            return Ok(None);
        };
        permanent_record_from_row(&row.map_err(|error| {
            MemoryError::Invalid(format!(
                "failed to decode permanent record row {record_id} from stoolap: {error}"
            ))
        })?)
        .map(Some)
    }

    fn relation_records_for(&self, record_id: &str) -> Result<Vec<RelationRecord>, MemoryError> {
        let rows = self
            .db
            .query(
                "SELECT
                    relation_id,
                    relation_type,
                    source_evidence_ids,
                    subject_record_id,
                    object_record_id,
                    created_at,
                    created_at_unix_ms
                 FROM permanent_relations
                 WHERE subject_record_id = $1 OR object_record_id = $1
                 ORDER BY relation_id, created_at_unix_ms",
                (record_id,),
            )
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to query permanent relations for {record_id} from stoolap: {error}"
                ))
            })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(permanent_relation_from_row(&row.map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to decode permanent relation row for {record_id} from stoolap: {error}"
                ))
            })?)?);
        }
        Ok(out)
    }

    fn list_records(&self) -> Result<Vec<StoredRecord>, MemoryError> {
        let rows = self
            .db
            .query(
                "SELECT
                    record_id,
                    keyspace,
                    payload_ref,
                    text,
                    source_evidence_ids,
                    created_at,
                    created_at_unix_ms,
                    updated_at,
                    updated_at_unix_ms,
                    validity_state,
                    working_kind
                 FROM permanent_records
                 ORDER BY updated_at_unix_ms DESC, record_id",
                (),
            )
            .map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to query permanent records from stoolap: {error}"
                ))
            })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(permanent_record_from_row(&row.map_err(|error| {
                MemoryError::Invalid(format!(
                    "failed to decode permanent record row from stoolap: {error}"
                ))
            })?)?);
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy)]
enum EmbeddingTextRole {
    Query,
    Passage,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct MaterializedEmbeddingExactPinManifest {
    binding_id: String,
    engine_kind: String,
    upstream_model_id: String,
    upstream_revision: String,
    artifact_set: Vec<String>,
    artifact_sha256: BTreeMap<String, String>,
    artifact_paths: BTreeMap<String, String>,
    dimensions: u16,
    pooling: String,
    normalization: String,
    prompt_profile: String,
    token_limit: u16,
    distance: String,
}

type ShippingEmbedFn = dyn Fn(EmbeddingTextRole, &str) -> Result<Vec<f32>, MemoryError>;

struct ShippingEmbeddingRuntime {
    expected_pin: EmbeddingExactPin,
    expected_engine_ref: String,
    home_root: PathBuf,
    embed_fn: RefCell<Option<Box<ShippingEmbedFn>>>,
}

impl ShippingEmbeddingRuntime {
    fn new(context: &ResolvedTurnContext) -> Result<Self, MemoryError> {
        let expected_pin = context.embedding_exact_pin.clone().ok_or_else(|| {
            MemoryError::Invalid(
                "shipping exact pin is required before semantic runtime open".to_string(),
            )
        })?;
        let expected_engine_ref = shipping_embedding_engine_ref_for_pin(&expected_pin);
        if context.resolved_kernel_adapters.embedding_engine_ref != expected_engine_ref {
            return Err(MemoryError::Invalid(
                "shipping semantic runtime requires source-driven non-null embedding engine alignment"
                    .to_string(),
            ));
        }
        Ok(Self {
            expected_pin,
            expected_engine_ref,
            home_root: shipping_home_root(context)?,
            embed_fn: RefCell::new(None),
        })
    }

    fn embed(&self, role: EmbeddingTextRole, text: &str) -> Result<Vec<f32>, MemoryError> {
        if self.embed_fn.borrow().is_none() {
            let loaded = load_shipping_embedding_runtime(
                &self.home_root,
                &self.expected_pin,
                &self.expected_engine_ref,
            )?;
            *self.embed_fn.borrow_mut() = Some(loaded);
        }
        let borrow = self.embed_fn.borrow();
        let embed = borrow.as_ref().ok_or_else(|| {
            MemoryError::Invalid("shipping exact pin materialized runtime is invalid".to_string())
        })?;
        embed(role, text)
    }
}

pub struct MemoryFacade {
    kernel: Option<InMemoryKernel>,
    next_envelope_id: u64,
    record_index: BTreeMap<RecordLocator, EnvelopeId>,
    processing_store: Option<RedbProcessingStore>,
    permanent_store: Option<StoolapPermanentStore>,
    shipping_embedding_runtime: Option<ShippingEmbeddingRuntime>,
}

impl MemoryFacade {
    pub fn new(context: &ResolvedTurnContext) -> Result<Self, MemoryError> {
        if context.is_shipping_binding() {
            let roots = context.memory_state_roots.as_ref().ok_or_else(|| {
                MemoryError::Invalid(
                    "shipping binding requires memory_state_roots before memory open".to_string(),
                )
            })?;
            validate_shipping_binding(context)?;
            return Ok(Self {
                kernel: None,
                next_envelope_id: 1,
                record_index: BTreeMap::new(),
                processing_store: Some(RedbProcessingStore::open(Path::new(
                    &roots.processing_state_root,
                ))?),
                permanent_store: Some(StoolapPermanentStore::open(Path::new(
                    &roots.permanent_state_root,
                ))?),
                shipping_embedding_runtime: Some(ShippingEmbeddingRuntime::new(context)?),
            });
        }
        validate_bringup_binding(context)?;
        Ok(Self {
            kernel: Some(InMemoryKernel::new(NullEmbeddingEngine::DEFAULT_DIMS)?),
            next_envelope_id: 1,
            record_index: BTreeMap::new(),
            processing_store: None,
            permanent_store: None,
            shipping_embedding_runtime: None,
        })
    }

    pub fn append_run_artifact(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: ProcessingRecordInput,
    ) -> Result<(), MemoryError> {
        if input.keyspace != RecordKeyspace::RunArtifacts {
            return Err(MemoryError::Invalid(
                "append_run_artifact requires run_artifacts keyspace".to_string(),
            ));
        }
        self.insert_processing_record(context, now_ms, input)
    }

    pub fn append_retrieval_candidate(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: ProcessingRecordInput,
    ) -> Result<(), MemoryError> {
        if input.keyspace != RecordKeyspace::RetrievalCandidates {
            return Err(MemoryError::Invalid(
                "append_retrieval_candidate requires retrieval_candidates keyspace".to_string(),
            ));
        }
        self.insert_processing_record(context, now_ms, input)
    }

    pub fn append_working_candidate(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: ProcessingRecordInput,
    ) -> Result<(), MemoryError> {
        if input.keyspace != RecordKeyspace::WorkingCandidates {
            return Err(MemoryError::Invalid(
                "append_working_candidate requires working_candidates keyspace".to_string(),
            ));
        }
        self.insert_processing_record(context, now_ms, input)
    }

    pub fn append_processing_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: ProcessingRecordInput,
    ) -> Result<(), MemoryError> {
        self.insert_processing_record(context, now_ms, input)
    }

    pub fn append_knowledge_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: PermanentRecordInput,
    ) -> Result<(), MemoryError> {
        if input.keyspace != RecordKeyspace::KnowledgeRecords {
            return Err(MemoryError::Invalid(
                "append_knowledge_record requires knowledge_records keyspace".to_string(),
            ));
        }
        self.insert_permanent_record(context, now_ms, input)
    }

    pub fn append_permanent_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: PermanentRecordInput,
    ) -> Result<(), MemoryError> {
        self.insert_permanent_record(context, now_ms, input)
    }

    pub fn append_relation(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        relation: RelationRecord,
    ) -> Result<(), MemoryError> {
        if relation.relation_id.trim().is_empty() {
            return Err(MemoryError::Invalid(
                "relation_id must not be empty".to_string(),
            ));
        }
        if relation.source_evidence_ids.is_empty() {
            return Err(MemoryError::Invalid(
                "relation requires at least one source_evidence_id".to_string(),
            ));
        }
        if relation.subject_record_id.trim().is_empty() {
            return Err(MemoryError::Invalid(
                "relation subject_record_id must not be empty".to_string(),
            ));
        }
        if self.is_shipping_mode() {
            return self.shipping_permanent_store()?.insert_relation(&relation);
        }
        let payload = StoredEnvelopePayload::Relation(relation.clone());
        let envelope_id = self.allocate_envelope_id();
        let envelope = build_envelope(envelope_id, SourceLayer::Permanent, now_ms, &payload)?;
        let ctx = op_ctx(context, now_ms)?;
        self.kernel_mut()?
            .put(&ctx, MemoryLayer::Permanent, envelope)?;
        Ok(())
    }

    pub fn mark_superseded(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: RelationMarkInput,
    ) -> Result<(), MemoryError> {
        self.append_relation(
            context,
            now_ms,
            RelationRecord {
                relation_id: input.relation_id,
                relation_type: RelationType::Supersedes,
                source_evidence_ids: vec![input.evidence_id],
                subject_record_id: input.subject_record_id,
                object_record_id: input.object_record_id,
                created_at: input.created_at,
                created_at_unix_ms: now_ms,
            },
        )
    }

    pub fn mark_invalidated(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: RelationMarkInput,
    ) -> Result<(), MemoryError> {
        self.append_relation(
            context,
            now_ms,
            RelationRecord {
                relation_id: input.relation_id,
                relation_type: RelationType::Invalidates,
                source_evidence_ids: vec![input.evidence_id],
                subject_record_id: input.subject_record_id,
                object_record_id: input.object_record_id,
                created_at: input.created_at,
                created_at_unix_ms: now_ms,
            },
        )
    }

    pub fn query_recent_candidates(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        limit: usize,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if self.is_shipping_mode() {
            let mut out = self.shipping_processing_records()?;
            out.retain(|record| {
                matches!(
                    record.keyspace,
                    RecordKeyspace::RetrievalCandidates | RecordKeyspace::WorkingCandidates
                )
            });
            out.sort_by(|left, right| {
                right
                    .updated_at_unix_ms
                    .cmp(&left.updated_at_unix_ms)
                    .then_with(|| left.record_id.cmp(&right.record_id))
            });
            return Ok(out
                .into_iter()
                .take(limit)
                .map(effective_candidate)
                .collect());
        }
        let mut out = self.records_for_layer(context, now_ms, SourceLayer::Processing, limit)?;
        out.retain(|record| {
            matches!(
                record.keyspace,
                RecordKeyspace::RetrievalCandidates | RecordKeyspace::WorkingCandidates
            )
        });
        out.sort_by(|left, right| {
            right
                .updated_at_unix_ms
                .cmp(&left.updated_at_unix_ms)
                .then_with(|| left.record_id.cmp(&right.record_id))
        });
        Ok(out
            .into_iter()
            .take(limit)
            .map(effective_candidate)
            .collect())
    }

    pub fn list_expiring(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
    ) -> Result<Vec<StoredRecord>, MemoryError> {
        if self.is_shipping_mode() {
            let mut out = self.shipping_processing_records()?;
            out.retain(|record| {
                record
                    .expires_at_unix_ms
                    .is_some_and(|expires_at| expires_at <= now_ms)
            });
            out.sort_by(|left, right| {
                left.expires_at_unix_ms
                    .cmp(&right.expires_at_unix_ms)
                    .then_with(|| left.record_id.cmp(&right.record_id))
            });
            return Ok(out);
        }
        let mut out =
            self.records_for_layer(context, now_ms, SourceLayer::Processing, usize::MAX)?;
        out.retain(|record| {
            record
                .expires_at_unix_ms
                .is_some_and(|expires_at| expires_at <= now_ms)
        });
        out.sort_by(|left, right| {
            left.expires_at_unix_ms
                .cmp(&right.expires_at_unix_ms)
                .then_with(|| left.record_id.cmp(&right.record_id))
        });
        Ok(out)
    }

    pub fn fetch_working_candidates(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if self.is_shipping_mode() {
            let mut out = self.shipping_processing_records()?;
            out.retain(|record| record.keyspace == RecordKeyspace::WorkingCandidates);
            out.sort_by(|left, right| {
                right
                    .updated_at_unix_ms
                    .cmp(&left.updated_at_unix_ms)
                    .then_with(|| left.record_id.cmp(&right.record_id))
            });
            return Ok(out.into_iter().map(effective_candidate).collect());
        }
        let mut out =
            self.records_for_layer(context, now_ms, SourceLayer::Processing, usize::MAX)?;
        out.retain(|record| record.keyspace == RecordKeyspace::WorkingCandidates);
        out.sort_by(|left, right| {
            right
                .updated_at_unix_ms
                .cmp(&left.updated_at_unix_ms)
                .then_with(|| left.record_id.cmp(&right.record_id))
        });
        Ok(out.into_iter().map(effective_candidate).collect())
    }

    pub fn get_record(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        record_id: &str,
    ) -> Result<Option<StoredRecord>, MemoryError> {
        if self.is_shipping_mode() {
            return match layer {
                SourceLayer::Processing => self.shipping_processing_record(record_id),
                SourceLayer::Permanent => self
                    .shipping_permanent_record(record_id)?
                    .map_or(Ok(None), |record| {
                        self.with_effective_validity(context, now_ms, record)
                    }),
                SourceLayer::Working => Err(MemoryError::Invalid(
                    "working layer is projection-only for P4".to_string(),
                )),
            };
        }
        let Some(envelope_id) = self
            .record_index
            .get(&RecordLocator {
                source_layer: layer,
                record_id: record_id.to_string(),
            })
            .copied()
        else {
            return Ok(None);
        };
        let ctx = op_ctx(context, now_ms)?;
        let envelope = self
            .kernel_ref()?
            .get(&ctx, layer.to_kernel(), envelope_id)?;
        let Some(envelope) = envelope else {
            return Ok(None);
        };
        let Some(record) = parse_record_envelope(&envelope)? else {
            return Ok(None);
        };
        self.with_effective_validity(context, now_ms, record)
    }

    pub fn lexical_search(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if self.is_shipping_mode() {
            return self.shipping_search(
                context,
                now_ms,
                layer,
                query_text,
                limit,
                QueryMode::Structured,
            );
        }
        self.search(
            context,
            now_ms,
            layer,
            query_text,
            limit,
            QueryMode::Structured,
        )
    }

    pub fn semantic_search(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if self.is_shipping_mode() {
            return self.shipping_search(
                context,
                now_ms,
                layer,
                query_text,
                limit,
                QueryMode::Vector,
            );
        }
        self.search(context, now_ms, layer, query_text, limit, QueryMode::Vector)
    }

    pub fn relation_traverse(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        record_id: &str,
    ) -> Result<Vec<RelationRecord>, MemoryError> {
        if self.is_shipping_mode() {
            return self
                .shipping_permanent_store()?
                .relation_records_for(record_id);
        }
        let mut relations = self.relation_records_for(context, now_ms)?;
        relations.retain(|relation| {
            relation.subject_record_id == record_id
                || relation.object_record_id.as_deref() == Some(record_id)
        });
        relations.sort_by(|left, right| {
            left.relation_id
                .cmp(&right.relation_id)
                .then_with(|| left.created_at_unix_ms.cmp(&right.created_at_unix_ms))
        });
        Ok(relations)
    }

    fn insert_processing_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: ProcessingRecordInput,
    ) -> Result<(), MemoryError> {
        if !input.keyspace.is_allowed_for(SourceLayer::Processing) {
            return Err(MemoryError::Invalid(format!(
                "keyspace {} is not allowed for processing",
                input.keyspace.as_str()
            )));
        }
        if input.source_evidence_ids.is_empty() {
            return Err(MemoryError::Invalid(
                "processing record requires source_evidence_ids".to_string(),
            ));
        }
        let record = StoredRecord {
            record_id: input.record_id,
            source_layer: SourceLayer::Processing,
            keyspace: input.keyspace,
            payload_ref: input.payload_ref,
            text: input.text,
            source_evidence_ids: input.source_evidence_ids,
            created_at: input.created_at,
            created_at_unix_ms: input.created_at_unix_ms,
            updated_at: input.updated_at,
            updated_at_unix_ms: input.updated_at_unix_ms,
            expires_at: Some(input.expires_at),
            expires_at_unix_ms: Some(input.expires_at_unix_ms),
            validity_state: ValidityState::Valid,
            working_kind: input.working_kind,
        };
        if self.is_shipping_mode() {
            validate_record(&record)?;
            return self.shipping_processing_store()?.insert_record(&record);
        }
        self.insert_record(context, now_ms, record)
    }

    fn insert_permanent_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        input: PermanentRecordInput,
    ) -> Result<(), MemoryError> {
        if !input.keyspace.is_allowed_for(SourceLayer::Permanent) {
            return Err(MemoryError::Invalid(format!(
                "keyspace {} is not allowed for permanent",
                input.keyspace.as_str()
            )));
        }
        if input.source_evidence_ids.is_empty() {
            return Err(MemoryError::Invalid(
                "permanent record requires source_evidence_ids".to_string(),
            ));
        }
        let record = StoredRecord {
            record_id: input.record_id,
            source_layer: SourceLayer::Permanent,
            keyspace: input.keyspace,
            payload_ref: input.payload_ref,
            text: input.text,
            source_evidence_ids: input.source_evidence_ids,
            created_at: input.created_at,
            created_at_unix_ms: input.created_at_unix_ms,
            updated_at: input.updated_at,
            updated_at_unix_ms: input.updated_at_unix_ms,
            expires_at: None,
            expires_at_unix_ms: None,
            validity_state: input.validity_state,
            working_kind: input.working_kind,
        };
        if self.is_shipping_mode() {
            validate_record(&record)?;
            return self.shipping_permanent_store()?.insert_record(&record);
        }
        self.insert_record(context, now_ms, record)
    }

    fn insert_record(
        &mut self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        record: StoredRecord,
    ) -> Result<(), MemoryError> {
        validate_record(&record)?;
        let layer = record.source_layer;
        let envelope_id = self.allocate_envelope_id();
        let payload = StoredEnvelopePayload::Record(record.clone());
        let envelope = build_envelope(envelope_id, layer, now_ms, &payload)?;
        let ctx = op_ctx(context, now_ms)?;
        self.kernel_mut()?.put(&ctx, layer.to_kernel(), envelope)?;
        self.record_index.insert(
            RecordLocator {
                source_layer: layer,
                record_id: record.record_id.clone(),
            },
            envelope_id,
        );
        Ok(())
    }

    fn records_for_layer(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        limit: usize,
    ) -> Result<Vec<StoredRecord>, MemoryError> {
        let ctx = op_ctx(context, now_ms)?;
        let envelopes = self
            .kernel_ref()?
            .list(&ctx, layer.to_kernel(), limit.max(1))?;
        let mut out = Vec::new();
        for envelope in envelopes {
            if let Some(record) = parse_record_envelope(&envelope)? {
                if let Some(record) = self.with_effective_validity(context, now_ms, record)? {
                    out.push(record);
                }
            }
        }
        Ok(out)
    }

    fn relation_records_for(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
    ) -> Result<Vec<RelationRecord>, MemoryError> {
        let ctx = op_ctx(context, now_ms)?;
        let envelopes = self
            .kernel_ref()?
            .list(&ctx, MemoryLayer::Permanent, usize::MAX)?;
        let mut out = Vec::new();
        for envelope in envelopes {
            if let Some(relation) = parse_relation_envelope(&envelope)? {
                out.push(relation);
            }
        }
        Ok(out)
    }

    fn with_effective_validity(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        mut record: StoredRecord,
    ) -> Result<Option<StoredRecord>, MemoryError> {
        if record.source_layer != SourceLayer::Permanent {
            return Ok(Some(record));
        }
        let relations = self.relation_traverse(context, now_ms, &record.record_id)?;
        for relation in relations {
            match relation.relation_type {
                RelationType::Invalidates => {
                    record.validity_state = ValidityState::Invalidated;
                }
                RelationType::Supersedes => {
                    if relation.subject_record_id == record.record_id {
                        record.validity_state = ValidityState::Superseded;
                    }
                }
            }
        }
        Ok(Some(record))
    }

    fn search(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        query_text: &str,
        limit: usize,
        mode: QueryMode,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if query_text.trim().is_empty() {
            return Err(MemoryError::Invalid(
                "query_text must not be empty".to_string(),
            ));
        }
        if matches!(layer, SourceLayer::Working) {
            return Err(MemoryError::Invalid(
                "working is not a retrieval source of truth".to_string(),
            ));
        }
        let ctx = op_ctx(context, now_ms)?;
        let mut layers = BTreeSet::new();
        layers.insert(layer.to_kernel());
        let input = QueryInput {
            mode,
            query_text: Some(query_text.to_string()),
            query_vector: None,
            filters: QueryFilters {
                layers: Some(layers),
                ..QueryFilters::default()
            },
            limit,
        };
        input.validate()?;
        let result = self.kernel_ref()?.query(&ctx, &input)?;
        let mut out = Vec::new();
        for hit in result.hits {
            let Some(record) = self.record_for_envelope(context, now_ms, layer, hit.id)? else {
                continue;
            };
            if !searchable_keyspaces(layer).contains(&record.keyspace) {
                continue;
            }
            out.push(effective_candidate(record));
        }
        out.sort_by(|left, right| {
            right
                .updated_at_unix_ms
                .cmp(&left.updated_at_unix_ms)
                .then_with(|| left.candidate_id.cmp(&right.candidate_id))
        });
        Ok(out)
    }

    fn shipping_search(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        query_text: &str,
        limit: usize,
        mode: QueryMode,
    ) -> Result<Vec<RetrievedCandidateRecord>, MemoryError> {
        if query_text.trim().is_empty() {
            return Err(MemoryError::Invalid(
                "query_text must not be empty".to_string(),
            ));
        }
        if matches!(layer, SourceLayer::Working) {
            return Err(MemoryError::Invalid(
                "working is not a retrieval source of truth".to_string(),
            ));
        }
        let records = self.shipping_search_records(context, now_ms, layer)?;
        let candidates = match mode {
            QueryMode::Structured => shipping_structured_hits(records, query_text),
            QueryMode::Vector => {
                let runtime = self.shipping_embedding_runtime()?;
                shipping_vector_hits(runtime, records, query_text)?
            }
            QueryMode::Hybrid => {
                return Err(MemoryError::Invalid(
                    "hybrid query mode is not used by CYRUNE retrieval selection".to_string(),
                ));
            }
        };

        Ok(candidates
            .into_iter()
            .take(limit)
            .map(effective_candidate)
            .collect())
    }

    fn record_for_envelope(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
        envelope_id: EnvelopeId,
    ) -> Result<Option<StoredRecord>, MemoryError> {
        let ctx = op_ctx(context, now_ms)?;
        let envelope = self
            .kernel_ref()?
            .get(&ctx, layer.to_kernel(), envelope_id)?;
        let Some(envelope) = envelope else {
            return Ok(None);
        };
        let Some(record) = parse_record_envelope(&envelope)? else {
            return Ok(None);
        };
        self.with_effective_validity(context, now_ms, record)
    }

    fn allocate_envelope_id(&mut self) -> EnvelopeId {
        let next = self.next_envelope_id;
        self.next_envelope_id = self.next_envelope_id.saturating_add(1);
        EnvelopeId::new(next)
    }

    fn kernel_ref(&self) -> Result<&InMemoryKernel, MemoryError> {
        self.kernel.as_ref().ok_or_else(|| {
            MemoryError::Invalid(
                "in-memory kernel is unavailable for shipping processing mode".to_string(),
            )
        })
    }

    fn kernel_mut(&mut self) -> Result<&mut InMemoryKernel, MemoryError> {
        self.kernel.as_mut().ok_or_else(|| {
            MemoryError::Invalid(
                "in-memory kernel is unavailable for shipping processing mode".to_string(),
            )
        })
    }

    fn shipping_processing_store(&self) -> Result<&RedbProcessingStore, MemoryError> {
        self.processing_store.as_ref().ok_or_else(|| {
            MemoryError::Invalid(
                "shipping processing store is unavailable outside shipping mode".to_string(),
            )
        })
    }

    fn shipping_permanent_store(&self) -> Result<&StoolapPermanentStore, MemoryError> {
        self.permanent_store.as_ref().ok_or_else(|| {
            MemoryError::Invalid(
                "shipping permanent store is unavailable outside shipping mode".to_string(),
            )
        })
    }

    fn shipping_embedding_runtime(&self) -> Result<&ShippingEmbeddingRuntime, MemoryError> {
        self.shipping_embedding_runtime.as_ref().ok_or_else(|| {
            MemoryError::Invalid(
                "shipping semantic runtime is unavailable outside shipping mode".to_string(),
            )
        })
    }

    fn shipping_processing_records(&self) -> Result<Vec<StoredRecord>, MemoryError> {
        self.shipping_processing_store()?.list_records()
    }

    fn shipping_processing_record(
        &self,
        record_id: &str,
    ) -> Result<Option<StoredRecord>, MemoryError> {
        self.shipping_processing_store()?.get_record(record_id)
    }

    fn shipping_permanent_record(
        &self,
        record_id: &str,
    ) -> Result<Option<StoredRecord>, MemoryError> {
        self.shipping_permanent_store()?.get_record(record_id)
    }

    fn shipping_search_records(
        &self,
        context: &ResolvedTurnContext,
        now_ms: u64,
        layer: SourceLayer,
    ) -> Result<Vec<StoredRecord>, MemoryError> {
        let mut records = match layer {
            SourceLayer::Processing => self.shipping_processing_records()?,
            SourceLayer::Permanent => {
                let raw = self.shipping_permanent_store()?.list_records()?;
                let mut out = Vec::new();
                for record in raw {
                    if let Some(record) = self.with_effective_validity(context, now_ms, record)? {
                        out.push(record);
                    }
                }
                out
            }
            SourceLayer::Working => {
                return Err(MemoryError::Invalid(
                    "working is not a retrieval source of truth".to_string(),
                ));
            }
        };
        records.retain(|record| searchable_keyspaces(layer).contains(&record.keyspace));
        Ok(records)
    }

    fn is_shipping_mode(&self) -> bool {
        self.processing_store.is_some() || self.permanent_store.is_some()
    }
}

fn validate_bringup_binding(context: &ResolvedTurnContext) -> Result<(), MemoryError> {
    let adapters = &context.resolved_kernel_adapters;
    if adapters.working_store_adapter_id != BRINGUP_MEMORY_ADAPTER_ID
        || adapters.processing_store_adapter_id != BRINGUP_MEMORY_ADAPTER_ID
        || adapters.permanent_store_adapter_id != BRINGUP_MEMORY_ADAPTER_ID
        || adapters.vector_index_adapter_id != BRINGUP_MEMORY_ADAPTER_ID
    {
        return Err(MemoryError::Invalid(
            "P4 bring-up baseline requires memory-kv-inmem across all kernel adapters".to_string(),
        ));
    }
    if adapters.embedding_engine_ref != BRINGUP_EMBEDDING_ENGINE_REF {
        return Err(MemoryError::Invalid(format!(
            "unexpected embedding engine ref: {}",
            adapters.embedding_engine_ref
        )));
    }
    Ok(())
}

fn validate_shipping_binding(context: &ResolvedTurnContext) -> Result<(), MemoryError> {
    let adapters = &context.resolved_kernel_adapters;
    if adapters.working_store_adapter_id != BRINGUP_MEMORY_ADAPTER_ID {
        return Err(MemoryError::Invalid(format!(
            "shipping binding requires working adapter {} but got {}",
            BRINGUP_MEMORY_ADAPTER_ID, adapters.working_store_adapter_id
        )));
    }
    if adapters.processing_store_adapter_id != SHIPPING_PROCESSING_ADAPTER_ID {
        return Err(MemoryError::Invalid(format!(
            "shipping binding requires processing adapter {} but got {}",
            SHIPPING_PROCESSING_ADAPTER_ID, adapters.processing_store_adapter_id
        )));
    }
    if adapters.permanent_store_adapter_id != SHIPPING_PERMANENT_ADAPTER_ID {
        return Err(MemoryError::Invalid(format!(
            "shipping binding requires permanent adapter {} but got {}",
            SHIPPING_PERMANENT_ADAPTER_ID, adapters.permanent_store_adapter_id
        )));
    }
    let pin = context.embedding_exact_pin.as_ref().ok_or_else(|| {
        MemoryError::Invalid(
            "shipping binding requires embedding_exact_pin before memory open".to_string(),
        )
    })?;
    let expected_engine_ref = shipping_embedding_engine_ref_for_pin(pin);
    if adapters.embedding_engine_ref != expected_engine_ref
        || adapters.embedding_engine_ref == BRINGUP_EMBEDDING_ENGINE_REF
    {
        return Err(MemoryError::Invalid(
            "shipping binding requires source-driven non-null embedding engine alignment"
                .to_string(),
        ));
    }
    Ok(())
}

fn shipping_home_root(context: &ResolvedTurnContext) -> Result<PathBuf, MemoryError> {
    let roots = context.memory_state_roots.as_ref().ok_or_else(|| {
        MemoryError::Invalid(
            "shipping binding requires memory_state_roots before runtime alignment".to_string(),
        )
    })?;
    let processing_root = PathBuf::from(&roots.processing_state_root);
    let memory_root = processing_root.parent().ok_or_else(|| {
        MemoryError::Invalid(
            "shipping memory_state_roots must resolve under CYRUNE_HOME/memory".to_string(),
        )
    })?;
    if memory_root.file_name() != Some(OsStr::new("memory")) {
        return Err(MemoryError::Invalid(
            "shipping memory_state_roots must resolve under CYRUNE_HOME/memory".to_string(),
        ));
    }
    memory_root.parent().map(Path::to_path_buf).ok_or_else(|| {
        MemoryError::Invalid(
            "shipping memory_state_roots must resolve under CYRUNE_HOME/memory".to_string(),
        )
    })
}

fn load_shipping_embedding_runtime(
    home_root: &Path,
    expected_pin: &EmbeddingExactPin,
    expected_engine_ref: &str,
) -> Result<Box<ShippingEmbedFn>, MemoryError> {
    let manifest = load_materialized_shipping_manifest(home_root)?;
    validate_materialized_shipping_manifest(
        home_root,
        &manifest,
        expected_pin,
        expected_engine_ref,
    )?;

    let tokenizer_path = materialized_artifact_path(home_root, &manifest, "tokenizer.json")?;
    let model_path = materialized_artifact_path(home_root, &manifest, "model.onnx")?;
    let special_tokens_path =
        materialized_artifact_path(home_root, &manifest, "special_tokens_map.json")?;
    let special_tokens = load_special_tokens_map(&special_tokens_path)?;

    let mut tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|_| {
        MemoryError::Invalid("shipping exact pin materialized runtime is unreadable".to_string())
    })?;
    let pad_token = special_tokens
        .pad_token
        .unwrap_or_else(|| "<pad>".to_string());
    let pad_id = tokenizer.token_to_id(&pad_token).ok_or_else(|| {
        MemoryError::Invalid("shipping exact pin materialized runtime is invalid".to_string())
    })?;
    tokenizer
        .with_truncation(Some(TruncationParams {
            max_length: usize::from(expected_pin.token_limit),
            ..TruncationParams::default()
        }))
        .map_err(|_| {
            MemoryError::Invalid("shipping exact pin materialized runtime is invalid".to_string())
        })?;
    tokenizer.with_padding(Some(PaddingParams {
        strategy: PaddingStrategy::Fixed(usize::from(expected_pin.token_limit)),
        pad_id,
        pad_type_id: 0,
        pad_token,
        ..PaddingParams::default()
    }));

    let runnable = tract_onnx::onnx()
        .model_for_path(&model_path)
        .and_then(|model| model.into_optimized())
        .and_then(|model| model.into_runnable())
        .map_err(|_| {
            MemoryError::Invalid(
                "shipping exact pin materialized runtime is unreadable".to_string(),
            )
        })?;
    let dimensions = usize::from(expected_pin.dimensions);
    let token_limit = usize::from(expected_pin.token_limit);

    Ok(Box::new(move |role, text| {
        embed_with_shipping_model(&tokenizer, &runnable, role, text, dimensions, token_limit)
    }))
}

fn load_materialized_shipping_manifest(
    home_root: &Path,
) -> Result<MaterializedEmbeddingExactPinManifest, MemoryError> {
    let manifest_path = home_root.join(MATERIALIZED_SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH);
    let bytes = fs::read(&manifest_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            MemoryError::Invalid("shipping exact pin materialized source is missing".to_string())
        } else {
            MemoryError::Invalid("shipping exact pin materialized source is unreadable".to_string())
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        MemoryError::Invalid("shipping exact pin materialized source is invalid".to_string())
    })
}

fn validate_materialized_shipping_manifest(
    home_root: &Path,
    manifest: &MaterializedEmbeddingExactPinManifest,
    expected_pin: &EmbeddingExactPin,
    expected_engine_ref: &str,
) -> Result<(), MemoryError> {
    if manifest.binding_id != SHIPPING_BINDING_ID {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }
    let manifest_pin = EmbeddingExactPin {
        engine_kind: manifest.engine_kind.clone(),
        upstream_model_id: manifest.upstream_model_id.clone(),
        upstream_revision: Some(manifest.upstream_revision.clone()),
        artifact_set: manifest.artifact_set.clone(),
        artifact_sha256: manifest.artifact_sha256.clone(),
        dimensions: manifest.dimensions,
        pooling: manifest.pooling.clone(),
        normalization: manifest.normalization.clone(),
        prompt_profile: manifest.prompt_profile.clone(),
        token_limit: manifest.token_limit,
        distance: manifest.distance.clone(),
    };
    if &manifest_pin != expected_pin {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }
    if shipping_embedding_engine_ref_for_pin(expected_pin) != expected_engine_ref {
        return Err(MemoryError::Invalid(
            "shipping semantic runtime requires source-driven non-null embedding engine alignment"
                .to_string(),
        ));
    }

    let artifact_set: BTreeSet<&str> = manifest.artifact_set.iter().map(String::as_str).collect();
    if artifact_set.len() != manifest.artifact_set.len()
        || artifact_set
            != manifest
                .artifact_paths
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
        || artifact_set
            != manifest
                .artifact_sha256
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
    {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }

    for artifact_name in &manifest.artifact_set {
        let relative_path = manifest.artifact_paths.get(artifact_name).ok_or_else(|| {
            MemoryError::Invalid("shipping exact pin materialized source is invalid".to_string())
        })?;
        let expected_hash = manifest.artifact_sha256.get(artifact_name).ok_or_else(|| {
            MemoryError::Invalid("shipping exact pin materialized source is invalid".to_string())
        })?;
        if expected_hash.len() != 64 || !expected_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(MemoryError::Invalid(
                "shipping exact pin materialized source is invalid".to_string(),
            ));
        }
        let artifact_path = validated_materialized_artifact_relative_path(relative_path)?;
        let bytes = fs::read(home_root.join(&artifact_path)).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                MemoryError::Invalid(
                    "shipping exact pin materialized source is missing".to_string(),
                )
            } else {
                MemoryError::Invalid(
                    "shipping exact pin materialized source is unreadable".to_string(),
                )
            }
        })?;
        if sha256_hex(&bytes) != *expected_hash {
            return Err(MemoryError::Invalid(
                "shipping exact pin materialized source is invalid".to_string(),
            ));
        }
    }
    Ok(())
}

fn validated_materialized_artifact_relative_path(
    relative_path: &str,
) -> Result<PathBuf, MemoryError> {
    if relative_path.trim().is_empty() {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }
    let path = PathBuf::from(relative_path);
    if path.is_absolute() {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }
    let mut components = path.components();
    match components.next() {
        Some(std::path::Component::Normal(first)) if first == OsStr::new("embedding") => {}
        _ => {
            return Err(MemoryError::Invalid(
                "shipping exact pin materialized source is invalid".to_string(),
            ));
        }
    }
    match components.next() {
        Some(std::path::Component::Normal(second)) if second == OsStr::new("artifacts") => {}
        _ => {
            return Err(MemoryError::Invalid(
                "shipping exact pin materialized source is invalid".to_string(),
            ));
        }
    }
    for component in components {
        match component {
            std::path::Component::Normal(_) => {}
            _ => {
                return Err(MemoryError::Invalid(
                    "shipping exact pin materialized source is invalid".to_string(),
                ));
            }
        }
    }
    if !relative_path.starts_with(MATERIALIZED_SHIPPING_ARTIFACT_PREFIX) {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized source is invalid".to_string(),
        ));
    }
    Ok(path)
}

fn materialized_artifact_path(
    home_root: &Path,
    manifest: &MaterializedEmbeddingExactPinManifest,
    artifact_name: &str,
) -> Result<PathBuf, MemoryError> {
    let relative_path = manifest.artifact_paths.get(artifact_name).ok_or_else(|| {
        MemoryError::Invalid("shipping exact pin materialized source is invalid".to_string())
    })?;
    Ok(
        home_root.join(validated_materialized_artifact_relative_path(
            relative_path,
        )?),
    )
}

#[derive(Debug, Deserialize)]
struct MaterializedSpecialTokensMap {
    pad_token: Option<String>,
}

fn load_special_tokens_map(path: &Path) -> Result<MaterializedSpecialTokensMap, MemoryError> {
    let bytes = fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            MemoryError::Invalid("shipping exact pin materialized source is missing".to_string())
        } else {
            MemoryError::Invalid("shipping exact pin materialized source is unreadable".to_string())
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        MemoryError::Invalid("shipping exact pin materialized source is invalid".to_string())
    })
}

fn role_prefixed_text(role: EmbeddingTextRole, text: &str) -> String {
    match role {
        EmbeddingTextRole::Query => format!("query: {text}"),
        EmbeddingTextRole::Passage => format!("passage: {text}"),
    }
}

type ShippingRunnableModel =
    RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

fn embed_with_shipping_model(
    tokenizer: &Tokenizer,
    runnable: &ShippingRunnableModel,
    role: EmbeddingTextRole,
    text: &str,
    dimensions: usize,
    token_limit: usize,
) -> Result<Vec<f32>, MemoryError> {
    let prefixed = role_prefixed_text(role, text);
    let encoding = tokenizer.encode(prefixed, true).map_err(|_| {
        MemoryError::Invalid("shipping exact pin materialized runtime is invalid".to_string())
    })?;
    let input_ids = encoding
        .get_ids()
        .iter()
        .copied()
        .map(i64::from)
        .collect::<Vec<_>>();
    let attention_mask = encoding
        .get_attention_mask()
        .iter()
        .copied()
        .map(i64::from)
        .collect::<Vec<_>>();
    let token_type_ids = encoding
        .get_type_ids()
        .iter()
        .copied()
        .map(i64::from)
        .collect::<Vec<_>>();
    if input_ids.len() != token_limit
        || attention_mask.len() != token_limit
        || token_type_ids.len() != token_limit
    {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized runtime is invalid".to_string(),
        ));
    }

    let shape = [1usize, token_limit];
    let outputs = runnable
        .run(tvec!(
            Tensor::from_shape(&shape, &input_ids)
                .map_err(|_| MemoryError::Invalid(
                    "shipping exact pin materialized runtime is unreadable".to_string()
                ))?
                .into_tvalue(),
            Tensor::from_shape(&shape, &attention_mask)
                .map_err(|_| MemoryError::Invalid(
                    "shipping exact pin materialized runtime is unreadable".to_string()
                ))?
                .into_tvalue(),
            Tensor::from_shape(&shape, &token_type_ids)
                .map_err(|_| MemoryError::Invalid(
                    "shipping exact pin materialized runtime is unreadable".to_string()
                ))?
                .into_tvalue()
        ))
        .map_err(|_| {
            MemoryError::Invalid(
                "shipping exact pin materialized runtime is unreadable".to_string(),
            )
        })?;

    let output = outputs.first().ok_or_else(|| {
        MemoryError::Invalid("shipping exact pin materialized runtime is invalid".to_string())
    })?;
    let output = output.to_array_view::<f32>().map_err(|_| {
        MemoryError::Invalid("shipping exact pin materialized runtime is unreadable".to_string())
    })?;
    let shape = output.shape();
    if shape.len() != 3 || shape[0] != 1 || shape[1] != token_limit || shape[2] != dimensions {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized runtime is invalid".to_string(),
        ));
    }
    let flat = output.as_slice().ok_or_else(|| {
        MemoryError::Invalid("shipping exact pin materialized runtime is unreadable".to_string())
    })?;
    let mut pooled = vec![0.0f32; dimensions];
    let mut count = 0usize;
    for (token_idx, mask) in attention_mask.iter().enumerate().take(token_limit) {
        if *mask == 0 {
            continue;
        }
        let base = token_idx * dimensions;
        for dim in 0..dimensions {
            pooled[dim] += flat[base + dim];
        }
        count += 1;
    }
    if count == 0 {
        return Err(MemoryError::Invalid(
            "shipping exact pin materialized runtime is invalid".to_string(),
        ));
    }
    for value in &mut pooled {
        *value /= count as f32;
    }
    let norm = pooled.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm <= f32::EPSILON {
        return Err(MemoryError::Invalid(
            "semantic retrieval requires non-zero embeddings".to_string(),
        ));
    }
    for value in &mut pooled {
        *value /= norm;
    }
    Ok(pooled)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn permanent_record_from_row(row: &stoolap::ResultRow) -> Result<StoredRecord, MemoryError> {
    Ok(StoredRecord {
        record_id: row.get(0).map_err(stoolap_row_error)?,
        source_layer: SourceLayer::Permanent,
        keyspace: serde_json::from_str(&row.get::<String>(1).map_err(stoolap_row_error)?)?,
        payload_ref: row.get(2).map_err(stoolap_row_error)?,
        text: row.get(3).map_err(stoolap_row_error)?,
        source_evidence_ids: serde_json::from_str(
            &row.get::<String>(4).map_err(stoolap_row_error)?,
        )?,
        created_at: row.get(5).map_err(stoolap_row_error)?,
        created_at_unix_ms: row
            .get::<i64>(6)
            .map_err(stoolap_row_error)?
            .try_into()
            .map_err(|_| {
                MemoryError::Invalid(
                    "permanent created_at_unix_ms must be non-negative".to_string(),
                )
            })?,
        updated_at: row.get(7).map_err(stoolap_row_error)?,
        updated_at_unix_ms: row
            .get::<i64>(8)
            .map_err(stoolap_row_error)?
            .try_into()
            .map_err(|_| {
                MemoryError::Invalid(
                    "permanent updated_at_unix_ms must be non-negative".to_string(),
                )
            })?,
        expires_at: None,
        expires_at_unix_ms: None,
        validity_state: serde_json::from_str(&row.get::<String>(9).map_err(stoolap_row_error)?)?,
        working_kind: serde_json::from_str(&row.get::<String>(10).map_err(stoolap_row_error)?)?,
    })
}

fn permanent_relation_from_row(row: &stoolap::ResultRow) -> Result<RelationRecord, MemoryError> {
    Ok(RelationRecord {
        relation_id: row.get(0).map_err(stoolap_row_error)?,
        relation_type: serde_json::from_str(&row.get::<String>(1).map_err(stoolap_row_error)?)?,
        source_evidence_ids: serde_json::from_str(
            &row.get::<String>(2).map_err(stoolap_row_error)?,
        )?,
        subject_record_id: row.get(3).map_err(stoolap_row_error)?,
        object_record_id: row.get::<Option<String>>(4).map_err(stoolap_row_error)?,
        created_at: row.get(5).map_err(stoolap_row_error)?,
        created_at_unix_ms: row
            .get::<i64>(6)
            .map_err(stoolap_row_error)?
            .try_into()
            .map_err(|_| {
                MemoryError::Invalid(
                    "permanent relation created_at_unix_ms must be non-negative".to_string(),
                )
            })?,
    })
}

fn stoolap_row_error(error: stoolap::Error) -> MemoryError {
    MemoryError::Invalid(format!("failed to decode stoolap row: {error}"))
}

fn searchable_keyspaces(layer: SourceLayer) -> &'static [RecordKeyspace] {
    match layer {
        SourceLayer::Working => &[],
        SourceLayer::Processing => &[
            RecordKeyspace::RetrievalCandidates,
            RecordKeyspace::WorkingCandidates,
        ],
        SourceLayer::Permanent => &[RecordKeyspace::KnowledgeRecords],
    }
}

fn shipping_structured_hits(records: Vec<StoredRecord>, query_text: &str) -> Vec<StoredRecord> {
    let query = query_text.to_ascii_lowercase();
    let mut out: Vec<_> = records
        .into_iter()
        .filter(|record| record.text.to_ascii_lowercase().contains(&query))
        .collect();
    out.sort_by(|left, right| {
        right
            .updated_at_unix_ms
            .cmp(&left.updated_at_unix_ms)
            .then_with(|| left.record_id.cmp(&right.record_id))
    });
    out
}

fn shipping_vector_hits(
    runtime: &ShippingEmbeddingRuntime,
    records: Vec<StoredRecord>,
    query_text: &str,
) -> Result<Vec<StoredRecord>, MemoryError> {
    let query_vector = runtime.embed(EmbeddingTextRole::Query, query_text)?;
    let mut scored = Vec::new();
    for record in records {
        let record_vector = runtime.embed(EmbeddingTextRole::Passage, &record.text)?;
        scored.push((cosine_similarity(&query_vector, &record_vector)?, record));
    }
    scored.sort_by(|(left_score, left_record), (right_score, right_record)| {
        right_score
            .partial_cmp(left_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right_record
                    .updated_at_unix_ms
                    .cmp(&left_record.updated_at_unix_ms)
            })
            .then_with(|| left_record.record_id.cmp(&right_record.record_id))
    });
    Ok(scored.into_iter().map(|(_, record)| record).collect())
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f32, MemoryError> {
    if left.len() != right.len() {
        return Err(MemoryError::Invalid(
            "query and record embeddings must share the same dimensions".to_string(),
        ));
    }
    let mut dot = 0.0f32;
    let mut left_norm = 0.0f32;
    let mut right_norm = 0.0f32;
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }
    if left_norm <= f32::EPSILON || right_norm <= f32::EPSILON {
        return Err(MemoryError::Invalid(
            "semantic retrieval requires non-zero embeddings".to_string(),
        ));
    }
    let cosine = dot / (left_norm.sqrt() * right_norm.sqrt());
    Ok(((cosine + 1.0) / 2.0).clamp(0.0, 1.0))
}

fn validate_record(record: &StoredRecord) -> Result<(), MemoryError> {
    if record.record_id.trim().is_empty() {
        return Err(MemoryError::Invalid(
            "record_id must not be empty".to_string(),
        ));
    }
    if !record.keyspace.is_allowed_for(record.source_layer) {
        return Err(MemoryError::Invalid(format!(
            "keyspace {} is not allowed for {}",
            record.keyspace.as_str(),
            record.source_layer.as_str()
        )));
    }
    if record.payload_ref.trim().is_empty() || record.text.trim().is_empty() {
        return Err(MemoryError::Invalid(
            "payload_ref and text must not be empty".to_string(),
        ));
    }
    if record.source_evidence_ids.is_empty() {
        return Err(MemoryError::Invalid(
            "record requires source_evidence_ids".to_string(),
        ));
    }
    match record.source_layer {
        SourceLayer::Working => {
            return Err(MemoryError::Invalid(
                "working layer is projection-only for P4".to_string(),
            ));
        }
        SourceLayer::Processing => {
            if record.expires_at.is_none() || record.expires_at_unix_ms.is_none() {
                return Err(MemoryError::Invalid(
                    "processing record requires expires_at".to_string(),
                ));
            }
            if record.expires_at_unix_ms <= Some(record.updated_at_unix_ms) {
                return Err(MemoryError::Invalid(
                    "processing expires_at must be later than updated_at".to_string(),
                ));
            }
        }
        SourceLayer::Permanent => {
            if record.expires_at.is_some() || record.expires_at_unix_ms.is_some() {
                return Err(MemoryError::Invalid(
                    "permanent record must not carry expires_at".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn build_envelope(
    id: EnvelopeId,
    layer: SourceLayer,
    now_ms: u64,
    payload: &StoredEnvelopePayload,
) -> Result<TypedEnvelope, MemoryError> {
    let payload_bytes = serde_json::to_vec(payload)?;
    let mut kv = BTreeMap::new();
    let mut tags = BTreeSet::new();
    match payload {
        StoredEnvelopePayload::Record(record) => {
            kv.insert("record_id".to_string(), record.record_id.clone());
            kv.insert("keyspace".to_string(), record.keyspace.as_str().to_string());
            kv.insert("payload_ref".to_string(), record.payload_ref.clone());
            kv.insert(
                "validity_state".to_string(),
                format!("{:?}", record.validity_state).to_ascii_lowercase(),
            );
            kv.insert(
                "updated_at_unix_ms".to_string(),
                record.updated_at_unix_ms.to_string(),
            );
            for evidence_id in &record.source_evidence_ids {
                tags.insert(format!("evidence:{evidence_id}"));
            }
        }
        StoredEnvelopePayload::Relation(relation) => {
            kv.insert("relation_id".to_string(), relation.relation_id.clone());
            kv.insert(
                "relation_type".to_string(),
                format!("{:?}", relation.relation_type).to_ascii_lowercase(),
            );
            kv.insert(
                "subject_record_id".to_string(),
                relation.subject_record_id.clone(),
            );
            if let Some(object_record_id) = &relation.object_record_id {
                kv.insert("object_record_id".to_string(), object_record_id.clone());
            }
            for evidence_id in &relation.source_evidence_ids {
                tags.insert(format!("evidence:{evidence_id}"));
            }
        }
    }
    tags.insert(format!("layer:{}", layer.as_str()));

    Ok(TypedEnvelope {
        id,
        payload: payload_bytes,
        content_type: ContentType::Json,
        importance: Importance::High,
        created_at: UnixMs::new(now_ms),
        updated_at: UnixMs::new(now_ms),
        last_accessed_at: UnixMs::new(now_ms),
        metadata: TypedMetadata { kv, tags },
        retention: retention_for_payload(layer, payload),
    })
}

fn retention_for_payload(layer: SourceLayer, payload: &StoredEnvelopePayload) -> RetentionHints {
    match (layer, payload) {
        (SourceLayer::Processing, StoredEnvelopePayload::Record(_)) => RetentionHints {
            ttl_ms: Some(PROCESSING_TTL_MS),
            ..RetentionHints::default()
        },
        (SourceLayer::Processing, StoredEnvelopePayload::Relation(_)) => RetentionHints {
            ttl_ms: Some(PROCESSING_TTL_MS),
            ..RetentionHints::default()
        },
        (SourceLayer::Permanent, _) => RetentionHints {
            forgettable: false,
            forgetting_exempt: true,
            ttl_ms: None,
        },
        (SourceLayer::Working, _) => RetentionHints::default(),
    }
}

fn parse_record_envelope(envelope: &TypedEnvelope) -> Result<Option<StoredRecord>, MemoryError> {
    let payload: StoredEnvelopePayload = serde_json::from_slice(&envelope.payload)?;
    match payload {
        StoredEnvelopePayload::Record(record) => Ok(Some(record)),
        StoredEnvelopePayload::Relation(_) => Ok(None),
    }
}

fn parse_relation_envelope(
    envelope: &TypedEnvelope,
) -> Result<Option<RelationRecord>, MemoryError> {
    let payload: StoredEnvelopePayload = serde_json::from_slice(&envelope.payload)?;
    match payload {
        StoredEnvelopePayload::Record(_) => Ok(None),
        StoredEnvelopePayload::Relation(relation) => Ok(Some(relation)),
    }
}

fn effective_candidate(record: StoredRecord) -> RetrievedCandidateRecord {
    RetrievedCandidateRecord {
        candidate_id: record.record_id,
        source_layer: record.source_layer,
        payload_ref: record.payload_ref,
        text: record.text,
        source_evidence_ids: record.source_evidence_ids,
        updated_at: record.updated_at,
        updated_at_unix_ms: record.updated_at_unix_ms,
        expires_at_unix_ms: record.expires_at_unix_ms,
        validity_state: record.validity_state,
        working_kind: record.working_kind,
    }
}

fn op_ctx(context: &ResolvedTurnContext, now_ms: u64) -> Result<OpCtx, MemoryError> {
    Ok(OpCtx::new(
        KernelCorrelationId::new(context.correlation_id.as_str().to_string())?,
        UnixMs::new(now_ms),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        MaterializedEmbeddingExactPinManifest, MemoryFacade, PROCESSING_TTL_MS,
        PermanentRecordInput, ProcessingRecordInput, RecordKeyspace, RelationMarkInput,
        SourceLayer, StoredEnvelopePayload, StoredRecord, ValidityState, retention_for_payload,
    };
    use crate::resolved_turn_context::{
        EmbeddingExactPin, MemoryStateRoots, ResolvedKernelAdapters, ResolvedTurnContext,
        TimeoutPolicy,
    };
    use crate::resolver::shipping_embedding_engine_ref_for_pin;
    use crate::working::WorkingSlotKind;
    use cyrune_core_contract::{CorrelationId, IoMode, RequestId, RunId, RunKind};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn shipping_embedding_exact_pin() -> EmbeddingExactPin {
        let manifest_path = bundle_embedding_root()
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        let manifest: MaterializedEmbeddingExactPinManifest =
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
        let Ok(manifest) = serde_json::from_slice::<MaterializedEmbeddingExactPinManifest>(&bytes)
        else {
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
            request_id: RequestId::parse("REQ-20260327-0003").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0003").unwrap(),
            run_id: RunId::parse("RUN-20260327-0003-R01").unwrap(),
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
            request_id: RequestId::parse("REQ-20260406-0001").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260406-0001").unwrap(),
            run_id: RunId::parse("RUN-20260406-0001-R01").unwrap(),
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
    fn processing_records_are_expiring_and_fetchable() {
        let context = test_context();
        let mut memory = MemoryFacade::new(&context).unwrap();
        memory
            .append_working_candidate(
                &context,
                10,
                ProcessingRecordInput {
                    keyspace: RecordKeyspace::WorkingCandidates,
                    record_id: "MEM-001".to_string(),
                    payload_ref: "processing://working_candidates/MEM-001".to_string(),
                    text: "Runtime is projection-only.".to_string(),
                    source_evidence_ids: vec!["EVID-10".to_string()],
                    created_at: "2026-03-27T15:30:00+09:00".to_string(),
                    created_at_unix_ms: 10,
                    updated_at: "2026-03-27T15:30:00+09:00".to_string(),
                    updated_at_unix_ms: 10,
                    expires_at: "2026-05-08T15:30:00+09:00".to_string(),
                    expires_at_unix_ms: 20,
                    working_kind: Some(WorkingSlotKind::Definition),
                },
            )
            .unwrap();

        let fetched = memory.fetch_working_candidates(&context, 10).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].candidate_id, "MEM-001");
        assert_eq!(fetched[0].source_layer, SourceLayer::Processing);

        let expiring = memory.list_expiring(&context, 20).unwrap();
        assert_eq!(expiring.len(), 1);
    }

    #[test]
    fn permanent_validity_is_relation_driven() {
        let context = test_context();
        let mut memory = MemoryFacade::new(&context).unwrap();
        memory
            .append_knowledge_record(
                &context,
                100,
                PermanentRecordInput {
                    keyspace: RecordKeyspace::KnowledgeRecords,
                    record_id: "MEM-100".to_string(),
                    payload_ref: "permanent://knowledge_records/MEM-100".to_string(),
                    text: "Free keeps Permanent as long-term knowledge.".to_string(),
                    source_evidence_ids: vec!["EVID-100".to_string()],
                    created_at: "2026-03-27T15:30:00+09:00".to_string(),
                    created_at_unix_ms: 100,
                    updated_at: "2026-03-27T15:30:00+09:00".to_string(),
                    updated_at_unix_ms: 100,
                    validity_state: ValidityState::Valid,
                    working_kind: Some(WorkingSlotKind::Definition),
                },
            )
            .unwrap();
        memory
            .mark_invalidated(
                &context,
                110,
                RelationMarkInput {
                    relation_id: "REL-001".to_string(),
                    subject_record_id: "MEM-100".to_string(),
                    object_record_id: None,
                    evidence_id: "EVID-110".to_string(),
                    created_at: "2026-03-27T15:31:50+09:00".to_string(),
                },
            )
            .unwrap();

        let record = memory
            .get_record(&context, 110, SourceLayer::Permanent, "MEM-100")
            .unwrap()
            .unwrap();
        assert_eq!(record.validity_state, ValidityState::Invalidated);
    }

    #[test]
    fn shipping_processing_records_materialize_to_redb_root() {
        let temp = tempdir().unwrap();
        let context = shipping_test_context(temp.path());

        {
            let mut memory = MemoryFacade::new(&context).unwrap();
            memory
                .append_working_candidate(
                    &context,
                    10,
                    ProcessingRecordInput {
                        keyspace: RecordKeyspace::WorkingCandidates,
                        record_id: "MEM-SHIP-001".to_string(),
                        payload_ref: "processing://working_candidates/MEM-SHIP-001".to_string(),
                        text: "Shipping Processing backend is persistent.".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-10".to_string()],
                        created_at: "2026-04-06T22:50:40+09:00".to_string(),
                        created_at_unix_ms: 10,
                        updated_at: "2026-04-06T22:50:40+09:00".to_string(),
                        updated_at_unix_ms: 10,
                        expires_at: "2026-05-18T22:50:40+09:00".to_string(),
                        expires_at_unix_ms: 20,
                        working_kind: Some(WorkingSlotKind::Definition),
                    },
                )
                .unwrap();
        }

        let reopened = MemoryFacade::new(&context).unwrap();
        let fetched = reopened.fetch_working_candidates(&context, 10).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].candidate_id, "MEM-SHIP-001");
        assert_eq!(fetched[0].source_layer, SourceLayer::Processing);

        let expiring = reopened.list_expiring(&context, 20).unwrap();
        assert_eq!(expiring.len(), 1);

        let record = reopened
            .get_record(&context, 20, SourceLayer::Processing, "MEM-SHIP-001")
            .unwrap()
            .unwrap();
        assert_eq!(
            record.payload_ref,
            "processing://working_candidates/MEM-SHIP-001"
        );
        println!("correlation_id={}", context.correlation_id.as_str());
        println!("binding_id={}", context.binding_id);
        println!(
            "processing_state_root={}",
            context
                .memory_state_roots
                .as_ref()
                .unwrap()
                .processing_state_root
        );
        println!(
            "processing_adapter={}",
            context.resolved_kernel_adapters.processing_store_adapter_id
        );
        println!("reopened_candidate_id={}", fetched[0].candidate_id);
        println!("expiring_count={}", expiring.len());
        println!("reopened_payload_ref={}", record.payload_ref);
    }

    #[test]
    fn shipping_permanent_records_materialize_to_stoolap_root() {
        let temp = tempdir().unwrap();
        let context = shipping_test_context(temp.path());
        {
            let mut memory = MemoryFacade::new(&context).unwrap();
            memory
                .append_knowledge_record(
                    &context,
                    100,
                    PermanentRecordInput {
                        keyspace: RecordKeyspace::KnowledgeRecords,
                        record_id: "MEM-SHIP-PERM-001".to_string(),
                        payload_ref: "permanent://knowledge_records/MEM-SHIP-PERM-001".to_string(),
                        text: "Permanent is materialized in SMB-I2.".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-100".to_string()],
                        created_at: "2026-04-06T22:57:04+09:00".to_string(),
                        created_at_unix_ms: 100,
                        updated_at: "2026-04-06T22:57:04+09:00".to_string(),
                        updated_at_unix_ms: 100,
                        validity_state: ValidityState::Valid,
                        working_kind: Some(WorkingSlotKind::Definition),
                    },
                )
                .unwrap();
            memory
                .mark_invalidated(
                    &context,
                    110,
                    RelationMarkInput {
                        relation_id: "REL-SHIP-001".to_string(),
                        subject_record_id: "MEM-SHIP-PERM-001".to_string(),
                        object_record_id: None,
                        evidence_id: "EVID-SHIP-110".to_string(),
                        created_at: "2026-04-06T22:57:05+09:00".to_string(),
                    },
                )
                .unwrap();
        }

        let reopened = MemoryFacade::new(&context).unwrap();
        let record = reopened
            .get_record(&context, 110, SourceLayer::Permanent, "MEM-SHIP-PERM-001")
            .unwrap()
            .unwrap();
        assert_eq!(record.validity_state, ValidityState::Invalidated);

        let relations = reopened
            .relation_traverse(&context, 110, "MEM-SHIP-PERM-001")
            .unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].relation_id, "REL-SHIP-001");
        assert!(record.expires_at.is_none());
        assert!(record.expires_at_unix_ms.is_none());
        println!("correlation_id={}", context.correlation_id.as_str());
        println!("binding_id={}", context.binding_id);
        println!(
            "permanent_state_root={}",
            context
                .memory_state_roots
                .as_ref()
                .unwrap()
                .permanent_state_root
        );
        println!(
            "permanent_adapter={}",
            context.resolved_kernel_adapters.permanent_store_adapter_id
        );
        println!("reopened_record_id={}", record.record_id);
        println!("reopened_payload_ref={}", record.payload_ref);
        println!(
            "validity_state={}",
            serde_json::to_string(&record.validity_state).unwrap()
        );
        println!("relation_count={}", relations.len());
        println!("non_expiring={}", record.expires_at.is_none());
    }

    #[test]
    fn shipping_retrieval_searches_resolved_processing_store_after_smb_i3() {
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
                        record_id: "MEM-SHIP-SEARCH-001".to_string(),
                        payload_ref: "processing://retrieval_candidates/MEM-SHIP-SEARCH-001"
                            .to_string(),
                        text: "Shipping retrieval is resolved in SMB-I3.".to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-SEARCH-10".to_string()],
                        created_at: "2026-04-06T22:57:04+09:00".to_string(),
                        created_at_unix_ms: 10,
                        updated_at: "2026-04-06T22:57:04+09:00".to_string(),
                        updated_at_unix_ms: 10,
                        expires_at: "2026-05-18T22:57:04+09:00".to_string(),
                        expires_at_unix_ms: 20,
                        working_kind: Some(WorkingSlotKind::Definition),
                    },
                )
                .unwrap();
        }

        let memory = MemoryFacade::new(&context).unwrap();
        let lexical = memory
            .lexical_search(&context, 10, SourceLayer::Processing, "resolved", 4)
            .unwrap();
        let semantic = memory
            .semantic_search(&context, 10, SourceLayer::Processing, "resolved", 4)
            .unwrap();

        assert_eq!(lexical.len(), 1);
        assert_eq!(semantic.len(), 1);
        assert_eq!(lexical[0].candidate_id, "MEM-SHIP-SEARCH-001");
        assert_eq!(semantic[0].candidate_id, "MEM-SHIP-SEARCH-001");
    }

    #[test]
    fn shipping_retention_hints_fix_processing_ttl_and_permanent_non_expiring() {
        let payload = StoredEnvelopePayload::Record(StoredRecord {
            record_id: "MEM-SHIP-TTL-001".to_string(),
            source_layer: SourceLayer::Processing,
            keyspace: RecordKeyspace::WorkingCandidates,
            payload_ref: "processing://working_candidates/MEM-SHIP-TTL-001".to_string(),
            text: "Processing retention stays pinned to the shipping TTL contract.".to_string(),
            source_evidence_ids: vec!["EVID-SHIP-TTL-10".to_string()],
            created_at: "2026-04-09T14:00:00+09:00".to_string(),
            created_at_unix_ms: 10,
            updated_at: "2026-04-09T14:00:00+09:00".to_string(),
            updated_at_unix_ms: 10,
            expires_at: Some("2026-05-21T14:00:00+09:00".to_string()),
            expires_at_unix_ms: Some(20),
            validity_state: ValidityState::Valid,
            working_kind: Some(WorkingSlotKind::Definition),
        });

        let processing = retention_for_payload(SourceLayer::Processing, &payload);
        let permanent = retention_for_payload(SourceLayer::Permanent, &payload);

        assert_eq!(processing.ttl_ms, Some(PROCESSING_TTL_MS));
        assert_eq!(permanent.ttl_ms, None);
        assert!(!permanent.forgettable);
        assert!(permanent.forgetting_exempt);

        println!("processing_ttl_ms={}", processing.ttl_ms.unwrap());
        println!("permanent_ttl_ms=null");
        println!("permanent_non_expiring={}", permanent.ttl_ms.is_none());
        println!(
            "permanent_forgetting_exempt={}",
            permanent.forgetting_exempt
        );
    }

    #[test]
    fn shipping_promotion_path_writes_non_expiring_record_to_resolved_permanent_store() {
        let temp = tempdir().unwrap();
        let context = shipping_test_context(temp.path());
        {
            let mut memory = MemoryFacade::new(&context).unwrap();
            memory
                .append_processing_record(
                    &context,
                    10,
                    ProcessingRecordInput {
                        keyspace: RecordKeyspace::WorkingCandidates,
                        record_id: "MEM-SHIP-PROMOTE-001".to_string(),
                        payload_ref: "processing://working_candidates/MEM-SHIP-PROMOTE-001"
                            .to_string(),
                        text: "Promoted knowledge stays in the resolved Permanent store."
                            .to_string(),
                        source_evidence_ids: vec!["EVID-SHIP-PROM-10".to_string()],
                        created_at: "2026-04-07T00:00:09+09:00".to_string(),
                        created_at_unix_ms: 10,
                        updated_at: "2026-04-07T00:00:09+09:00".to_string(),
                        updated_at_unix_ms: 10,
                        expires_at: "2026-05-19T00:00:09+09:00".to_string(),
                        expires_at_unix_ms: 20,
                        working_kind: Some(WorkingSlotKind::Definition),
                    },
                )
                .unwrap();
        }

        let reopened = MemoryFacade::new(&context).unwrap();
        let source = reopened
            .get_record(
                &context,
                20,
                SourceLayer::Processing,
                "MEM-SHIP-PROMOTE-001",
            )
            .unwrap()
            .unwrap();
        assert!(source.expires_at.is_some());
        assert!(source.expires_at_unix_ms.is_some());

        {
            let mut memory = reopened;
            memory
                .append_knowledge_record(
                    &context,
                    30,
                    PermanentRecordInput {
                        keyspace: RecordKeyspace::KnowledgeRecords,
                        record_id: "MEM-SHIP-KNOW-001".to_string(),
                        payload_ref: "permanent://knowledge_records/MEM-SHIP-KNOW-001".to_string(),
                        text: source.text.clone(),
                        source_evidence_ids: source.source_evidence_ids.clone(),
                        created_at: "2026-04-07T00:00:30+09:00".to_string(),
                        created_at_unix_ms: 30,
                        updated_at: "2026-04-07T00:00:30+09:00".to_string(),
                        updated_at_unix_ms: 30,
                        validity_state: ValidityState::Valid,
                        working_kind: source.working_kind,
                    },
                )
                .unwrap();
        }

        let reopened = MemoryFacade::new(&context).unwrap();
        let promoted = reopened
            .get_record(&context, 30, SourceLayer::Permanent, "MEM-SHIP-KNOW-001")
            .unwrap()
            .unwrap();
        assert_eq!(
            promoted.text,
            "Promoted knowledge stays in the resolved Permanent store."
        );
        assert_eq!(
            promoted.source_evidence_ids,
            vec!["EVID-SHIP-PROM-10".to_string()]
        );
        assert!(promoted.expires_at.is_none());
        assert!(promoted.expires_at_unix_ms.is_none());
        println!("correlation_id={}", context.correlation_id.as_str());
        println!("binding_id={}", context.binding_id);
        println!(
            "permanent_state_root={}",
            context
                .memory_state_roots
                .as_ref()
                .unwrap()
                .permanent_state_root
        );
        println!(
            "permanent_adapter={}",
            context.resolved_kernel_adapters.permanent_store_adapter_id
        );
        println!("promotion_source_record_id={}", source.record_id);
        println!("source_expires_at_present={}", source.expires_at.is_some());
        println!(
            "source_expires_at_unix_ms={}",
            source.expires_at_unix_ms.unwrap()
        );
        println!("promoted_record_id={}", promoted.record_id);
        println!(
            "promoted_source_evidence_ids={}",
            serde_json::to_string(&promoted.source_evidence_ids).unwrap()
        );
        println!(
            "promoted_expires_at_present={}",
            promoted.expires_at.is_some()
        );
        println!("promoted_non_expiring={}", promoted.expires_at.is_none());
    }
}
