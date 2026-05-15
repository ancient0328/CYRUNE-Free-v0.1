use crate::error::{KernelError, Result};
use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvelopeId(u64);

impl EnvelopeId {
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EnvelopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixMs(u64);

impl UnixMs {
    #[must_use]
    pub fn new(ms: u64) -> Self {
        Self(ms)
    }

    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let v = value.into();
        if v.is_empty() {
            return Err(KernelError::invalid_input(
                "correlation_id must be non-empty",
            ));
        }
        Ok(Self(v))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpCtx {
    pub correlation_id: CorrelationId,
    pub now: UnixMs,
}

impl OpCtx {
    #[must_use]
    pub fn new(correlation_id: CorrelationId, now: UnixMs) -> Self {
        Self {
            correlation_id,
            now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryLayer {
    Working,
    Processing,
    Permanent,
}

impl MemoryLayer {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Processing => "processing",
            Self::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContentType {
    Text,
    Json,
    Binary,
}

impl ContentType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Binary => "binary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Importance {
    Low,
    Medium,
    High,
    Critical,
}

impl Importance {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedMetadata {
    pub kv: BTreeMap<String, String>,
    pub tags: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionHints {
    pub forgettable: bool,
    pub forgetting_exempt: bool,
    pub ttl_ms: Option<u64>,
}

impl Default for RetentionHints {
    fn default() -> Self {
        Self {
            forgettable: true,
            forgetting_exempt: false,
            ttl_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedEnvelope {
    pub id: EnvelopeId,
    pub payload: Vec<u8>,
    pub content_type: ContentType,
    pub importance: Importance,
    pub created_at: UnixMs,
    pub updated_at: UnixMs,
    pub last_accessed_at: UnixMs,
    pub metadata: TypedMetadata,
    pub retention: RetentionHints,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    values: Box<[f32]>,
}

impl Vector {
    pub fn new(values: Vec<f32>) -> Result<Self> {
        if values.is_empty() {
            return Err(KernelError::invalid_input("vector must be non-empty"));
        }
        if values.iter().any(|v| v.is_nan()) {
            return Err(KernelError::invalid_input("vector must not contain NaN"));
        }
        Ok(Self {
            values: values.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn dim(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score(f32);

impl Score {
    pub fn new(value: f32) -> Result<Self> {
        if value.is_nan() {
            return Err(KernelError::invalid_input("score must not be NaN"));
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(KernelError::invalid_input(
                "score must be within [0.0, 1.0]",
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_f32(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryMode {
    Structured,
    Vector,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryFilters {
    pub layers: Option<BTreeSet<MemoryLayer>>,
    pub content_types: Option<BTreeSet<ContentType>>,
    pub importance: Option<BTreeSet<Importance>>,
    pub tags_all: Option<BTreeSet<String>>,
    pub created_range: Option<(UnixMs, UnixMs)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryInput {
    pub mode: QueryMode,
    pub query_text: Option<String>,
    pub query_vector: Option<Vector>,
    pub filters: QueryFilters,
    pub limit: usize,
}

impl QueryInput {
    pub fn validate(&self) -> Result<()> {
        if self.limit == 0 {
            return Err(KernelError::invalid_input("limit must be >= 1"));
        }

        match self.mode {
            QueryMode::Structured => {
                if self.query_text.as_deref().unwrap_or_default().is_empty() {
                    return Err(KernelError::invalid_input(
                        "structured query requires non-empty query_text",
                    ));
                }
            }
            QueryMode::Vector => {
                let has_text = !self.query_text.as_deref().unwrap_or_default().is_empty();
                if self.query_vector.is_none() && !has_text {
                    return Err(KernelError::invalid_input(
                        "vector query requires query_vector or non-empty query_text",
                    ));
                }
            }
            QueryMode::Hybrid => {
                if self.query_text.as_deref().unwrap_or_default().is_empty()
                    && self.query_vector.is_none()
                {
                    return Err(KernelError::invalid_input(
                        "hybrid query requires query_text and/or query_vector",
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryHit {
    pub id: EnvelopeId,
    pub score: Score,
    pub metadata: Option<TypedMetadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub hits: Vec<QueryHit>,
    pub took_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn correlation_id_rejects_empty() {
        let e = CorrelationId::new("").unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn vector_rejects_empty_and_nan() {
        let e = Vector::new(Vec::new()).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);

        let e = Vector::new(vec![f32::NAN]).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn score_accepts_bounds_and_rejects_invalid() {
        assert!(Score::new(0.0).is_ok());
        assert!(Score::new(1.0).is_ok());

        let e = Score::new(-0.1).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);

        let e = Score::new(1.1).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);

        let e = Score::new(f32::NAN).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn query_input_validation_is_mode_dependent() {
        let base = QueryInput {
            mode: QueryMode::Structured,
            query_text: None,
            query_vector: None,
            filters: QueryFilters::default(),
            limit: 1,
        };

        // limit
        let e = QueryInput {
            limit: 0,
            ..base.clone()
        }
        .validate()
        .unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);

        // structured
        let e = base.validate().unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
        QueryInput {
            query_text: Some("q".to_string()),
            ..base.clone()
        }
        .validate()
        .unwrap();

        // vector
        let base_vector = QueryInput {
            mode: QueryMode::Vector,
            ..base.clone()
        };
        let e = base_vector.validate().unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
        QueryInput {
            query_text: Some("q".to_string()),
            ..base_vector.clone()
        }
        .validate()
        .unwrap();
        QueryInput {
            query_text: None,
            query_vector: Some(Vector::new(vec![0.0]).unwrap()),
            ..base_vector
        }
        .validate()
        .unwrap();

        // hybrid
        let base_hybrid = QueryInput {
            mode: QueryMode::Hybrid,
            ..base
        };
        let e = base_hybrid.validate().unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
        QueryInput {
            query_text: Some("q".to_string()),
            ..base_hybrid
        }
        .validate()
        .unwrap();
    }
}
