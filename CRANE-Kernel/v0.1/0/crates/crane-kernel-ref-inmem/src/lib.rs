//! In-memory reference implementation for CRANE-Kernel v0.1 bring-up.
//!
//! This crate exists to make the canonical contracts executable with minimal dependencies.

use crane_embed_null::NullEmbeddingEngine;
use crane_kernel::{
    ContentType, CorrelationId, EmbeddingEngine, EnvelopeId, ErrorKind, ForgettingPolicy,
    ForgettingVerdict, KernelError, LifecycleEngine, MemoryLayer, MemoryStore, MetricsEvent,
    MetricsHook, NoopMetricsHook, OpCtx, Operation, Query, QueryHit, QueryInput, QueryMode,
    QueryResult, Result, RetentionHints, Score, TypedEnvelope, UnixMs, Vector, VectorIndex,
};
use crane_store_inmem::InMemoryVectorIndex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct InMemoryMetricsCollector {
    events: Mutex<Vec<MetricsEvent>>,
}

impl InMemoryMetricsCollector {
    #[must_use]
    pub fn snapshot(&self) -> Vec<MetricsEvent> {
        self.events.lock().expect("metrics mutex poisoned").clone()
    }
}

impl MetricsHook for InMemoryMetricsCollector {
    fn record(&self, event: MetricsEvent) {
        self.events
            .lock()
            .expect("metrics mutex poisoned")
            .push(event);
    }
}

pub struct InMemoryKernel {
    dims: usize,
    metrics: Arc<dyn MetricsHook>,

    working: BTreeMap<EnvelopeId, TypedEnvelope>,
    processing: BTreeMap<EnvelopeId, TypedEnvelope>,
    permanent: BTreeMap<EnvelopeId, TypedEnvelope>,

    working_index: InMemoryVectorIndex,
    processing_index: InMemoryVectorIndex,
    permanent_index: InMemoryVectorIndex,

    embedder: NullEmbeddingEngine,
}

impl InMemoryKernel {
    pub fn new(dims: usize) -> Result<Self> {
        let embedder = NullEmbeddingEngine::new(dims)?;
        Ok(Self {
            dims,
            metrics: Arc::new(NoopMetricsHook),
            working: BTreeMap::new(),
            processing: BTreeMap::new(),
            permanent: BTreeMap::new(),
            working_index: InMemoryVectorIndex::new(dims)?,
            processing_index: InMemoryVectorIndex::new(dims)?,
            permanent_index: InMemoryVectorIndex::new(dims)?,
            embedder,
        })
    }

    #[must_use]
    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsHook>) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
    pub fn ctx(correlation_id: &str, now_ms: u64) -> OpCtx {
        OpCtx::new(
            CorrelationId::new(correlation_id).expect("correlation_id must be non-empty"),
            UnixMs::new(now_ms),
        )
    }

    fn layer_store(&self, layer: MemoryLayer) -> &BTreeMap<EnvelopeId, TypedEnvelope> {
        match layer {
            MemoryLayer::Working => &self.working,
            MemoryLayer::Processing => &self.processing,
            MemoryLayer::Permanent => &self.permanent,
        }
    }

    fn layer_store_mut(&mut self, layer: MemoryLayer) -> &mut BTreeMap<EnvelopeId, TypedEnvelope> {
        match layer {
            MemoryLayer::Working => &mut self.working,
            MemoryLayer::Processing => &mut self.processing,
            MemoryLayer::Permanent => &mut self.permanent,
        }
    }

    fn layer_index(&self, layer: MemoryLayer) -> &InMemoryVectorIndex {
        match layer {
            MemoryLayer::Working => &self.working_index,
            MemoryLayer::Processing => &self.processing_index,
            MemoryLayer::Permanent => &self.permanent_index,
        }
    }

    fn layer_index_mut(&mut self, layer: MemoryLayer) -> &mut InMemoryVectorIndex {
        match layer {
            MemoryLayer::Working => &mut self.working_index,
            MemoryLayer::Processing => &mut self.processing_index,
            MemoryLayer::Permanent => &mut self.permanent_index,
        }
    }

    fn record(
        &self,
        ctx: &OpCtx,
        operation: Operation,
        layer: Option<MemoryLayer>,
        success: bool,
        error_kind: Option<ErrorKind>,
        took_ms: u64,
    ) {
        self.metrics.record(MetricsEvent {
            correlation_id: ctx.correlation_id.clone(),
            operation,
            layer,
            success,
            error_kind,
            took_ms,
        });
    }

    fn should_index(envelope: &TypedEnvelope) -> bool {
        matches!(envelope.content_type, ContentType::Text | ContentType::Json)
    }

    fn payload_text(envelope: &TypedEnvelope) -> Option<&str> {
        std::str::from_utf8(&envelope.payload).ok()
    }

    fn passes_filters(envelope: &TypedEnvelope, input: &QueryInput) -> bool {
        if let Some(ct) = &input.filters.content_types
            && !ct.contains(&envelope.content_type)
        {
            return false;
        }

        if let Some(imp) = &input.filters.importance
            && !imp.contains(&envelope.importance)
        {
            return false;
        }

        if let Some(tags_all) = &input.filters.tags_all {
            for t in tags_all {
                if !envelope.metadata.tags.contains(t) {
                    return false;
                }
            }
        }

        if let Some((from, to)) = input.filters.created_range {
            let created = envelope.created_at;
            if created < from || created > to {
                return false;
            }
        }

        true
    }

    fn selected_layers(filters_layers: Option<&BTreeSet<MemoryLayer>>) -> BTreeSet<MemoryLayer> {
        match filters_layers {
            Some(s) if !s.is_empty() => s.clone(),
            _ => BTreeSet::from([
                MemoryLayer::Working,
                MemoryLayer::Processing,
                MemoryLayer::Permanent,
            ]),
        }
    }

    fn structured_hits(&self, input: &QueryInput) -> Vec<QueryHit> {
        let query = input
            .query_text
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let layers = Self::selected_layers(input.filters.layers.as_ref());
        let mut out = Vec::new();

        for layer in layers {
            for envelope in self.layer_store(layer).values() {
                if !Self::passes_filters(envelope, input) {
                    continue;
                }

                let Some(text) = Self::payload_text(envelope) else {
                    continue;
                };
                if !text.to_ascii_lowercase().contains(&query) {
                    continue;
                }

                out.push(QueryHit {
                    id: envelope.id,
                    score: Score::new(1.0).expect("1.0 within [0,1]"),
                    metadata: Some(envelope.metadata.clone()),
                });
            }
        }

        // Deterministic ordering: score desc, id asc.
        out.sort_by(|a, b| {
            b.score
                .as_f32()
                .partial_cmp(&a.score.as_f32())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        out
    }

    fn vector_hits(&self, ctx: &OpCtx, input: &QueryInput) -> Result<Vec<QueryHit>> {
        let owned;
        let qv: &Vector = if let Some(v) = &input.query_vector {
            v
        } else if let Some(qt) = input.query_text.as_deref() {
            owned = self.embed(ctx, qt)?;
            &owned
        } else {
            return Err(KernelError::invalid_input(
                "vector/hybrid query requires query_vector or query_text",
            ));
        };

        let layers = Self::selected_layers(input.filters.layers.as_ref());
        let mut out = Vec::new();

        // Overfetch to allow post-filtering while keeping code simple.
        let search_limit = input.limit.saturating_mul(8).max(input.limit);

        for layer in layers {
            let hits = self.layer_index(layer).search(qv, search_limit)?;
            for (id, score) in hits {
                let Some(envelope) = self.layer_store(layer).get(&id) else {
                    continue;
                };
                if !Self::passes_filters(envelope, input) {
                    continue;
                }

                out.push(QueryHit {
                    id,
                    score,
                    metadata: Some(envelope.metadata.clone()),
                });
            }
        }

        out.sort_by(|a, b| {
            b.score
                .as_f32()
                .partial_cmp(&a.score.as_f32())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(out)
    }

    fn merge_hybrid(
        structured: Vec<QueryHit>,
        vector: Vec<QueryHit>,
        limit: usize,
    ) -> Vec<QueryHit> {
        let mut by_id: BTreeMap<EnvelopeId, QueryHit> = BTreeMap::new();
        for h in structured {
            by_id.insert(h.id, h);
        }

        for h in vector {
            match by_id.get_mut(&h.id) {
                Some(existing) => {
                    if h.score.as_f32() > existing.score.as_f32() {
                        existing.score = h.score;
                    }
                    if existing.metadata.is_none() {
                        existing.metadata = h.metadata;
                    }
                }
                None => {
                    by_id.insert(h.id, h);
                }
            }
        }

        let mut out: Vec<QueryHit> = by_id.into_values().collect();
        out.sort_by(|a, b| {
            b.score
                .as_f32()
                .partial_cmp(&a.score.as_f32())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        out.truncate(limit);
        out
    }
}

impl MemoryStore for InMemoryKernel {
    fn put(&mut self, ctx: &OpCtx, layer: MemoryLayer, envelope: TypedEnvelope) -> Result<()> {
        let start = Instant::now();
        let id = envelope.id;
        let result: Result<()> = (|| {
            let embed_text = if Self::should_index(&envelope) {
                Self::payload_text(&envelope).map(str::to_string)
            } else {
                None
            };
            self.layer_store_mut(layer).insert(id, envelope);
            if let Some(text) = &embed_text {
                let v = self.embed(ctx, text)?;
                self.layer_index_mut(layer).upsert(id, v)?;
            }
            Ok(())
        })();
        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        match &result {
            Ok(()) => self.record(
                ctx,
                Operation::MemoryStorePut,
                Some(layer),
                true,
                None,
                took,
            ),
            Err(e) => self.record(
                ctx,
                Operation::MemoryStorePut,
                Some(layer),
                false,
                Some(e.kind()),
                took,
            ),
        }

        result
    }

    fn get(
        &self,
        ctx: &OpCtx,
        layer: MemoryLayer,
        id: EnvelopeId,
    ) -> Result<Option<TypedEnvelope>> {
        let start = Instant::now();
        let result = Ok(self.layer_store(layer).get(&id).cloned());
        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.record(
            ctx,
            Operation::MemoryStoreGet,
            Some(layer),
            true,
            None,
            took,
        );
        result
    }

    fn delete(&mut self, ctx: &OpCtx, layer: MemoryLayer, id: EnvelopeId) -> Result<bool> {
        let start = Instant::now();
        let existed = self.layer_store_mut(layer).remove(&id).is_some();
        if existed {
            let _ = self.layer_index_mut(layer).delete(id);
        }
        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.record(
            ctx,
            Operation::MemoryStoreDelete,
            Some(layer),
            true,
            None,
            took,
        );
        Ok(existed)
    }

    fn list(&self, ctx: &OpCtx, layer: MemoryLayer, limit: usize) -> Result<Vec<TypedEnvelope>> {
        let start = Instant::now();
        let result = (|| {
            if limit == 0 {
                return Err(KernelError::invalid_input("limit must be >= 1"));
            }
            Ok(self
                .layer_store(layer)
                .values()
                .take(limit)
                .cloned()
                .collect())
        })();
        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        match &result {
            Ok(_) => self.record(
                ctx,
                Operation::MemoryStoreList,
                Some(layer),
                true,
                None,
                took,
            ),
            Err(e) => self.record(
                ctx,
                Operation::MemoryStoreList,
                Some(layer),
                false,
                Some(e.kind()),
                took,
            ),
        }

        result
    }
}

impl EmbeddingEngine for InMemoryKernel {
    fn dims(&self) -> usize {
        self.dims
    }

    fn embed(&self, ctx: &OpCtx, text: &str) -> Result<Vector> {
        let start = Instant::now();
        let result = self.embedder.embed(ctx, text);
        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(_) => self.record(ctx, Operation::Embedding, None, true, None, took),
            Err(e) => self.record(ctx, Operation::Embedding, None, false, Some(e.kind()), took),
        }
        result
    }
}

impl ForgettingPolicy for InMemoryKernel {
    fn evaluate(&self, ctx: &OpCtx, envelope: &TypedEnvelope) -> Result<ForgettingVerdict> {
        let start = Instant::now();
        let result: Result<ForgettingVerdict> = (|| {
            let RetentionHints {
                forgettable,
                forgetting_exempt,
                ttl_ms,
            } = envelope.retention;

            if forgetting_exempt {
                return Ok(ForgettingVerdict::Keep);
            }
            if !forgettable {
                return Ok(ForgettingVerdict::Keep);
            }

            if let Some(ttl) = ttl_ms {
                let age_ms = ctx
                    .now
                    .as_u64()
                    .saturating_sub(envelope.created_at.as_u64());
                if age_ms > ttl {
                    return Ok(ForgettingVerdict::Forget);
                }
            }

            Ok(ForgettingVerdict::Keep)
        })();

        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(_) => self.record(ctx, Operation::ForgettingEvaluate, None, true, None, took),
            Err(e) => self.record(
                ctx,
                Operation::ForgettingEvaluate,
                None,
                false,
                Some(e.kind()),
                took,
            ),
        }

        result
    }
}

impl InMemoryKernel {
    fn move_between_layers(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        from: MemoryLayer,
        to: MemoryLayer,
        operation: Operation,
    ) -> Result<()> {
        let start = Instant::now();
        let result = (|| {
            let Some(env) = store.get(ctx, from, id)? else {
                return Err(KernelError::not_found("envelope not found"));
            };
            let _ = store.delete(ctx, from, id)?;
            store.put(ctx, to, env)?;
            Ok(())
        })();

        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(()) => self.record(ctx, operation, Some(from), true, None, took),
            Err(e) => self.record(ctx, operation, Some(from), false, Some(e.kind()), took),
        }

        result
    }
}

impl LifecycleEngine for InMemoryKernel {
    fn promote(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        from: MemoryLayer,
        to: MemoryLayer,
    ) -> Result<()> {
        self.move_between_layers(ctx, store, id, from, to, Operation::LifecyclePromote)
    }

    fn demote(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        from: MemoryLayer,
        to: MemoryLayer,
    ) -> Result<()> {
        self.move_between_layers(ctx, store, id, from, to, Operation::LifecycleDemote)
    }

    fn evict(
        &self,
        ctx: &OpCtx,
        store: &mut dyn MemoryStore,
        id: EnvelopeId,
        layer: MemoryLayer,
    ) -> Result<()> {
        let start = Instant::now();
        let result = (|| {
            let existed = store.delete(ctx, layer, id)?;
            if !existed {
                return Err(KernelError::not_found("envelope not found"));
            }
            Ok(())
        })();

        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(()) => self.record(
                ctx,
                Operation::LifecycleEvict,
                Some(layer),
                true,
                None,
                took,
            ),
            Err(e) => self.record(
                ctx,
                Operation::LifecycleEvict,
                Some(layer),
                false,
                Some(e.kind()),
                took,
            ),
        }

        result
    }
}

impl Query for InMemoryKernel {
    fn query(&self, ctx: &OpCtx, input: &QueryInput) -> Result<QueryResult> {
        let start = Instant::now();
        let result: Result<QueryResult> = (|| {
            input.validate()?;

            let hits = match input.mode {
                QueryMode::Structured => {
                    let mut out = self.structured_hits(input);
                    out.truncate(input.limit);
                    out
                }
                QueryMode::Vector => {
                    let mut out = self.vector_hits(ctx, input)?;
                    out.truncate(input.limit);
                    out
                }
                QueryMode::Hybrid => {
                    let structured = self.structured_hits(input);
                    let vector = self.vector_hits(ctx, input)?;
                    Self::merge_hybrid(structured, vector, input.limit)
                }
            };

            let took_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            Ok(QueryResult { hits, took_ms })
        })();

        let took = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let op = Operation::from_query_mode(input.mode);
        let layer = match &input.filters.layers {
            Some(s) if s.len() == 1 => s.iter().next().copied(),
            _ => None,
        };

        match &result {
            Ok(_) => self.record(ctx, op, layer, true, None, took),
            Err(e) => self.record(ctx, op, layer, false, Some(e.kind()), took),
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crane_kernel::{QueryFilters, TypedMetadata};

    fn env(id: u64, text: &str) -> TypedEnvelope {
        let mut md = TypedMetadata::default();
        md.tags.insert("t1".to_string());

        TypedEnvelope {
            id: EnvelopeId::new(id),
            payload: text.as_bytes().to_vec(),
            content_type: ContentType::Text,
            importance: crane_kernel::Importance::Medium,
            created_at: UnixMs::new(1000),
            updated_at: UnixMs::new(1000),
            last_accessed_at: UnixMs::new(1000),
            metadata: md,
            retention: RetentionHints::default(),
        }
    }

    fn env_with_retention(id: u64, text: &str, retention: RetentionHints) -> TypedEnvelope {
        let mut e = env(id, text);
        e.retention = retention;
        e
    }

    #[test]
    fn structured_query_is_deterministic_and_tie_breaks_by_id() {
        let metrics = Arc::new(InMemoryMetricsCollector::default());
        let mut k = InMemoryKernel::new(8)
            .unwrap()
            .with_metrics(metrics.clone());
        let ctx = InMemoryKernel::ctx("c1", 2000);

        k.put(&ctx, MemoryLayer::Working, env(2, "hello world"))
            .unwrap();
        k.put(&ctx, MemoryLayer::Working, env(1, "hello world"))
            .unwrap();

        let input = QueryInput {
            mode: QueryMode::Structured,
            query_text: Some("hello".to_string()),
            query_vector: None,
            filters: QueryFilters::default(),
            limit: 10,
        };

        let r1 = k.query(&ctx, &input).unwrap();
        let r2 = k.query(&ctx, &input).unwrap();
        assert_eq!(r1.hits.len(), 2);
        assert_eq!(r1.hits, r2.hits);
        assert_eq!(r1.hits[0].id.as_u64(), 1);
        assert_eq!(r1.hits[1].id.as_u64(), 2);

        // Metrics must include correlation_id.
        let ev = metrics.snapshot();
        assert!(ev.iter().any(|e| e.correlation_id.as_str() == "c1"));
    }

    #[test]
    fn vector_query_prefers_exact_match_text() {
        let mut k = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        k.put(&ctx, MemoryLayer::Working, env(1, "alpha")).unwrap();
        k.put(&ctx, MemoryLayer::Working, env(2, "beta")).unwrap();

        let input = QueryInput {
            mode: QueryMode::Vector,
            query_text: Some("alpha".to_string()),
            query_vector: None,
            filters: QueryFilters::default(),
            limit: 10,
        };

        let r = k.query(&ctx, &input).unwrap();
        assert!(!r.hits.is_empty());
        assert_eq!(r.hits[0].id.as_u64(), 1);
        assert!((0.0..=1.0).contains(&r.hits[0].score.as_f32()));
    }

    #[test]
    fn invalid_query_returns_invalid_input() {
        let k = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let input = QueryInput {
            mode: QueryMode::Structured,
            query_text: None,
            query_vector: None,
            filters: QueryFilters::default(),
            limit: 10,
        };

        let e = k.query(&ctx, &input).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn memory_store_put_get_delete_roundtrip() {
        let mut k = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let e = env(1, "hello");
        k.put(&ctx, MemoryLayer::Working, e.clone()).unwrap();
        assert_eq!(
            k.get(&ctx, MemoryLayer::Working, e.id).unwrap(),
            Some(e.clone())
        );
        assert!(k.delete(&ctx, MemoryLayer::Working, e.id).unwrap());
        assert_eq!(k.get(&ctx, MemoryLayer::Working, e.id).unwrap(), None);
        assert!(!k.delete(&ctx, MemoryLayer::Working, e.id).unwrap());
    }

    #[test]
    fn memory_store_list_limit_zero_returns_invalid_input_and_is_recorded() {
        let metrics = Arc::new(InMemoryMetricsCollector::default());
        let k = InMemoryKernel::new(8)
            .unwrap()
            .with_metrics(metrics.clone());
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let e = k.list(&ctx, MemoryLayer::Working, 0).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);

        let ev = metrics.snapshot();
        assert!(ev.iter().any(|e| {
            e.correlation_id.as_str() == "c1"
                && e.operation == Operation::MemoryStoreList
                && e.layer == Some(MemoryLayer::Working)
                && !e.success
                && e.error_kind == Some(ErrorKind::InvalidInput)
        }));
    }

    #[test]
    fn forgetting_policy_ttl_and_exempt_are_respected() {
        let k = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let e = env_with_retention(
            1,
            "hello",
            RetentionHints {
                forgettable: true,
                forgetting_exempt: false,
                ttl_ms: Some(500),
            },
        );
        let v = k.evaluate(&ctx, &e).unwrap();
        assert_eq!(v, ForgettingVerdict::Forget);

        let e = env_with_retention(
            2,
            "hello",
            RetentionHints {
                forgettable: true,
                forgetting_exempt: true,
                ttl_ms: Some(0),
            },
        );
        let v = k.evaluate(&ctx, &e).unwrap();
        assert_eq!(v, ForgettingVerdict::Keep);
    }

    #[test]
    fn lifecycle_promote_moves_between_layers_and_is_recorded() {
        let metrics = Arc::new(InMemoryMetricsCollector::default());
        let engine = InMemoryKernel::new(8)
            .unwrap()
            .with_metrics(metrics.clone());

        let mut store = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let e = env(1, "hello");
        store.put(&ctx, MemoryLayer::Working, e.clone()).unwrap();

        engine
            .promote(
                &ctx,
                &mut store,
                e.id,
                MemoryLayer::Working,
                MemoryLayer::Permanent,
            )
            .unwrap();

        assert_eq!(store.get(&ctx, MemoryLayer::Working, e.id).unwrap(), None);
        assert_eq!(
            store.get(&ctx, MemoryLayer::Permanent, e.id).unwrap(),
            Some(e)
        );

        let ev = metrics.snapshot();
        assert!(ev.iter().any(|e| {
            e.correlation_id.as_str() == "c1"
                && e.operation == Operation::LifecyclePromote
                && e.layer == Some(MemoryLayer::Working)
                && e.success
                && e.error_kind.is_none()
        }));
    }

    #[test]
    fn lifecycle_evict_missing_returns_not_found() {
        let engine = InMemoryKernel::new(8).unwrap();
        let mut store = InMemoryKernel::new(8).unwrap();
        let ctx = InMemoryKernel::ctx("c1", 2000);

        let e = engine
            .evict(&ctx, &mut store, EnvelopeId::new(1), MemoryLayer::Working)
            .unwrap_err();
        assert_eq!(e.kind(), ErrorKind::NotFound);
    }
}
