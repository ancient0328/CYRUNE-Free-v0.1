//! CRANE-Kernel v0.1 (Kernel core)
//!
//! This crate is intentionally **use-case agnostic**.
//! Domain I/O, domain contracts, and operational concerns belong to distros.

pub mod api;
pub mod error;
pub mod storage;
pub mod types;

pub use api::{
    EmbeddingEngine, ForgettingPolicy, ForgettingVerdict, Kernel, LifecycleEngine, MemoryStore,
    MetricsEvent, MetricsHook, NoopMetricsHook, Operation, Query,
};
pub use error::{ErrorKind, KernelError, Result};
pub use storage::{KvStore, VectorIndex, ensure_dims};
pub use types::{
    ContentType, CorrelationId, EnvelopeId, Importance, MemoryLayer, OpCtx, QueryFilters, QueryHit,
    QueryInput, QueryMode, QueryResult, RetentionHints, Score, TypedEnvelope, TypedMetadata,
    UnixMs, Vector,
};
