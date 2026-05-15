use crate::error::{ErrorKind, Result};
use crate::types::{
    CorrelationId, EnvelopeId, MemoryLayer, OpCtx, QueryInput, QueryMode, QueryResult,
    TypedEnvelope, Vector,
};

pub trait MemoryStore {
    fn put(&mut self, ctx: &OpCtx, layer: MemoryLayer, envelope: TypedEnvelope) -> Result<()>;

    fn get(&self, ctx: &OpCtx, layer: MemoryLayer, id: EnvelopeId)
    -> Result<Option<TypedEnvelope>>;

    fn delete(&mut self, ctx: &OpCtx, layer: MemoryLayer, id: EnvelopeId) -> Result<bool>;

    fn list(&self, ctx: &OpCtx, layer: MemoryLayer, limit: usize) -> Result<Vec<TypedEnvelope>>;
}

pub trait Query {
    fn query(&self, ctx: &OpCtx, input: &QueryInput) -> Result<QueryResult>;
}

pub trait EmbeddingEngine {
    fn dims(&self) -> usize;

    fn embed(&self, ctx: &OpCtx, text: &str) -> Result<Vector>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgettingVerdict {
    Keep,
    Forget,
}

pub trait ForgettingPolicy {
    fn evaluate(&self, ctx: &OpCtx, envelope: &TypedEnvelope) -> Result<ForgettingVerdict>;
}

pub trait LifecycleEngine {
    fn promote(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        from: MemoryLayer,
        to: MemoryLayer,
    ) -> Result<()>;

    fn demote(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        from: MemoryLayer,
        to: MemoryLayer,
    ) -> Result<()>;

    fn evict(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        layer: MemoryLayer,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    MemoryStorePut,
    MemoryStoreGet,
    MemoryStoreDelete,
    MemoryStoreList,
    QueryStructured,
    QueryVector,
    QueryHybrid,
    Embedding,
    ForgettingEvaluate,
    LifecyclePromote,
    LifecycleDemote,
    LifecycleEvict,
}

impl Operation {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MemoryStorePut => "memory_store_put",
            Self::MemoryStoreGet => "memory_store_get",
            Self::MemoryStoreDelete => "memory_store_delete",
            Self::MemoryStoreList => "memory_store_list",
            Self::QueryStructured => "query_structured",
            Self::QueryVector => "query_vector",
            Self::QueryHybrid => "query_hybrid",
            Self::Embedding => "embedding",
            Self::ForgettingEvaluate => "forgetting_evaluate",
            Self::LifecyclePromote => "lifecycle_promote",
            Self::LifecycleDemote => "lifecycle_demote",
            Self::LifecycleEvict => "lifecycle_evict",
        }
    }

    #[must_use]
    pub fn from_query_mode(mode: QueryMode) -> Self {
        match mode {
            QueryMode::Structured => Self::QueryStructured,
            QueryMode::Vector => Self::QueryVector,
            QueryMode::Hybrid => Self::QueryHybrid,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsEvent {
    pub correlation_id: CorrelationId,
    pub operation: Operation,
    pub layer: Option<MemoryLayer>,
    pub success: bool,
    pub error_kind: Option<ErrorKind>,
    pub took_ms: u64,
}

pub trait MetricsHook {
    fn record(&self, event: MetricsEvent);
}

#[derive(Debug, Default)]
pub struct NoopMetricsHook;

impl MetricsHook for NoopMetricsHook {
    fn record(&self, _event: MetricsEvent) {}
}

pub trait Kernel {
    type Memory: MemoryStore;
    type Query: Query;
    type Embedding: EmbeddingEngine;
    type Forgetting: ForgettingPolicy;
    type Lifecycle: LifecycleEngine;
    type Metrics: MetricsHook;

    fn memory_store(&self) -> &Self::Memory;
    fn memory_store_mut(&mut self) -> &mut Self::Memory;
    fn query(&self) -> &Self::Query;
    fn embedding_engine(&self) -> &Self::Embedding;
    fn forgetting_policy(&self) -> &Self::Forgetting;
    fn lifecycle_engine(&self) -> &Self::Lifecycle;
    fn metrics_hook(&self) -> &Self::Metrics;
}
