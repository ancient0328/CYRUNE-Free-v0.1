//! In-memory adapter implementations for bring-up and tests.
//!
//! This crate intentionally keeps dependencies minimal and deterministic.

use crane_kernel::{
    EnvelopeId, KernelError, KvStore, Result, Score, Vector, VectorIndex, ensure_dims,
};
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Debug, Default, Clone)]
pub struct InMemoryKvStore {
    data: BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl KvStore for InMemoryKvStore {
    fn get(&self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(namespace).and_then(|ns| ns.get(key).cloned()))
    }

    fn put(&mut self, namespace: &str, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.data
            .entry(namespace.to_string())
            .or_default()
            .insert(key, value);
        Ok(())
    }

    fn delete(&mut self, namespace: &str, key: &[u8]) -> Result<bool> {
        let Some(ns) = self.data.get_mut(namespace) else {
            return Ok(false);
        };
        Ok(ns.remove(key).is_some())
    }

    fn list(
        &self,
        namespace: &str,
        prefix: Option<&[u8]>,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        if limit == 0 {
            return Err(KernelError::invalid_input("limit must be >= 1"));
        }
        let Some(ns) = self.data.get(namespace) else {
            return Ok(Vec::new());
        };

        let mut out = Vec::new();
        for (k, v) in ns {
            if let Some(p) = prefix
                && !k.starts_with(p)
            {
                continue;
            }
            out.push((k.clone(), v.clone()));
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryVectorIndex {
    dims: usize,
    vectors: BTreeMap<EnvelopeId, Vector>,
}

impl InMemoryVectorIndex {
    pub fn new(dims: usize) -> Result<Self> {
        if dims == 0 {
            return Err(KernelError::invalid_input("dims must be >= 1"));
        }
        Ok(Self {
            dims,
            vectors: BTreeMap::new(),
        })
    }
}

impl VectorIndex for InMemoryVectorIndex {
    fn dims(&self) -> usize {
        self.dims
    }

    fn upsert(&mut self, id: EnvelopeId, vector: Vector) -> Result<()> {
        ensure_dims(self.dims, vector.dim())?;
        self.vectors.insert(id, vector);
        Ok(())
    }

    fn delete(&mut self, id: EnvelopeId) -> Result<bool> {
        Ok(self.vectors.remove(&id).is_some())
    }

    fn search(&self, query: &Vector, limit: usize) -> Result<Vec<(EnvelopeId, Score)>> {
        if limit == 0 {
            return Err(KernelError::invalid_input("limit must be >= 1"));
        }
        ensure_dims(self.dims, query.dim())?;

        // Top-K selection via a fixed-size heap (size = limit).
        //
        // `OrdScoredHit` is ordered by our desired final order (score desc, id asc).
        // Under that ordering, the *worst* candidate is the maximum element
        // (lowest score, then highest id), so a `BinaryHeap<OrdScoredHit>` gives us
        // O(log K) eviction of the current worst item.
        let mut heap: BinaryHeap<OrdScoredHit> = BinaryHeap::with_capacity(limit + 1);

        for (id, v) in &self.vectors {
            let cos = cosine_similarity(query.as_slice(), v.as_slice());
            let score = (cos + 1.0_f32) * 0.5;
            let score = score.clamp(0.0, 1.0);
            let entry = OrdScoredHit { score, id: *id };

            if heap.len() < limit {
                heap.push(entry);
            } else if let Some(worst) = heap.peek()
                && entry < *worst
            {
                heap.pop();
                heap.push(entry);
            }
        }

        // Drain heap and apply deterministic final sort (score desc, id asc).
        let mut top: Vec<OrdScoredHit> = heap.into_iter().collect();
        top.sort_unstable();

        let mut out = Vec::with_capacity(top.len());
        for h in top {
            out.push((h.id, Score::new(h.score)?));
        }
        Ok(out)
    }
}

/// Total-ordered (score desc, id asc) wrapper for heap-based top-K selection.
#[derive(Clone, Copy)]
struct OrdScoredHit {
    score: f32,
    id: EnvelopeId,
}

impl PartialEq for OrdScoredHit {
    fn eq(&self, other: &Self) -> bool {
        self.score.to_bits() == other.score.to_bits() && self.id == other.id
    }
}

impl Eq for OrdScoredHit {}

impl PartialOrd for OrdScoredHit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdScoredHit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Score desc (higher is better), then id asc (tie-breaker).
        other
            .score
            .to_bits()
            .cmp(&self.score.to_bits())
            .then_with(|| self.id.cmp(&other.id))
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }

    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kv_list_is_deterministic() {
        let mut kv = InMemoryKvStore::default();
        kv.put("ns", b"b".to_vec(), b"2".to_vec()).unwrap();
        kv.put("ns", b"a".to_vec(), b"1".to_vec()).unwrap();

        let listed = kv.list("ns", None, 10).unwrap();
        assert_eq!(listed[0].0, b"a".to_vec());
        assert_eq!(listed[1].0, b"b".to_vec());
    }

    #[test]
    fn vector_index_orders_by_score_then_id() {
        let mut idx = InMemoryVectorIndex::new(2).unwrap();
        idx.upsert(EnvelopeId::new(1), Vector::new(vec![1.0, 0.0]).unwrap())
            .unwrap();
        idx.upsert(EnvelopeId::new(2), Vector::new(vec![1.0, 0.0]).unwrap())
            .unwrap();

        let query = Vector::new(vec![1.0, 0.0]).unwrap();
        let hits = idx.search(&query, 10).unwrap();

        // Scores are tied, so id asc must be used.
        assert_eq!(hits[0].0.as_u64(), 1);
        assert_eq!(hits[1].0.as_u64(), 2);
    }

    #[test]
    fn vector_index_selects_top_k_by_score() {
        let mut idx = InMemoryVectorIndex::new(2).unwrap();

        // Query will be [1, 0].
        idx.upsert(EnvelopeId::new(1), Vector::new(vec![1.0, 0.0]).unwrap())
            .unwrap(); // cos = 1.0 => score = 1.0
        idx.upsert(EnvelopeId::new(2), Vector::new(vec![0.0, 1.0]).unwrap())
            .unwrap(); // cos = 0.0 => score = 0.5
        idx.upsert(EnvelopeId::new(3), Vector::new(vec![-1.0, 0.0]).unwrap())
            .unwrap(); // cos = -1.0 => score = 0.0

        let query = Vector::new(vec![1.0, 0.0]).unwrap();

        let hits = idx.search(&query, 1).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.as_u64(), 1);

        let hits = idx.search(&query, 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0.as_u64(), 1);
        assert_eq!(hits[1].0.as_u64(), 2);
    }
}
