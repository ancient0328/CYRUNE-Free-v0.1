use crate::error::{KernelError, Result};
use crate::types::{EnvelopeId, Score, Vector};

/// Minimal key-value store trait.
///
/// Concrete implementations belong to adapter crates.
pub trait KvStore {
    fn get(&self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn put(&mut self, namespace: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

    fn delete(&mut self, namespace: &str, key: &[u8]) -> Result<bool>;

    /// List key/value pairs in a deterministic order.
    fn list(
        &self,
        namespace: &str,
        prefix: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
}

/// Minimal vector index trait.
///
/// Concrete implementations belong to adapter crates.
pub trait VectorIndex {
    fn dims(&self) -> usize;

    fn upsert(&mut self, id: EnvelopeId, vector: Vector) -> Result<()>;

    fn delete(&mut self, id: EnvelopeId) -> Result<bool>;

    fn search(&self, query: &Vector, limit: usize) -> Result<Vec<(EnvelopeId, Score)>>;
}

pub fn ensure_dims(expected: usize, actual: usize) -> Result<()> {
    if expected != actual {
        return Err(KernelError::invalid_input(format!(
            "vector dimension mismatch: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn ensure_dims_accepts_equal() {
        ensure_dims(3, 3).unwrap();
    }

    #[test]
    fn ensure_dims_rejects_mismatch() {
        let e = ensure_dims(3, 2).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidInput);
    }
}
