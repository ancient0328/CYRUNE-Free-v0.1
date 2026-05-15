//! Null embedding engine for tests and lightweight bring-up.
//!
//! This implementation is deterministic and intentionally does not depend on heavy runtimes.

use crane_kernel::{EmbeddingEngine, KernelError, OpCtx, Result, Vector};

#[derive(Debug, Clone)]
pub struct NullEmbeddingEngine {
    dims: usize,
}

impl NullEmbeddingEngine {
    pub const DEFAULT_DIMS: usize = 384;

    pub fn new(dims: usize) -> Result<Self> {
        if dims == 0 {
            return Err(KernelError::invalid_input("dims must be >= 1"));
        }
        Ok(Self { dims })
    }
}

impl Default for NullEmbeddingEngine {
    fn default() -> Self {
        // Safe: constant is non-zero.
        Self {
            dims: Self::DEFAULT_DIMS,
        }
    }
}

impl EmbeddingEngine for NullEmbeddingEngine {
    fn dims(&self) -> usize {
        self.dims
    }

    fn embed(&self, _ctx: &OpCtx, text: &str) -> Result<Vector> {
        let mut out = Vec::with_capacity(self.dims);
        let mut h = fnv1a64(text.as_bytes());

        // Deterministically expand a 64-bit hash into `dims` floats.
        for i in 0..self.dims {
            h ^= (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h = h.wrapping_mul(0x1000_0000_01B3);

            // Map to [-1.0, 1.0) without NaNs.
            //
            // Avoid integer->float casts to keep clippy (pedantic) quiet.
            let mantissa = (h >> 41) as u32; // 23 bits
            let bits = 0x3F80_0000u32 | (mantissa & 0x007F_FFFF); // 1.mantissa
            let unit = f32::from_bits(bits) - 1.0; // [0, 1)
            out.push(unit * 2.0 - 1.0);
        }

        Vector::new(out)
    }
}

pub fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    let mut h = OFFSET;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crane_kernel::{CorrelationId, UnixMs};

    #[test]
    fn embed_is_deterministic() {
        let e = NullEmbeddingEngine::new(8).unwrap();
        let ctx = OpCtx::new(CorrelationId::new("test").unwrap(), UnixMs::new(0));
        let v1 = e.embed(&ctx, "hello").unwrap();
        let v2 = e.embed(&ctx, "hello").unwrap();
        assert_eq!(v1.as_slice(), v2.as_slice());
        assert_eq!(v1.dim(), 8);
    }

    #[test]
    fn different_text_produces_different_vectors() {
        let e = NullEmbeddingEngine::new(8).unwrap();
        let ctx = OpCtx::new(CorrelationId::new("test").unwrap(), UnixMs::new(0));
        let v1 = e.embed(&ctx, "a").unwrap();
        let v2 = e.embed(&ctx, "b").unwrap();
        assert_ne!(v1.as_slice(), v2.as_slice());
    }
}
