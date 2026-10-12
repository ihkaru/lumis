/// High-performance deterministic dense embedding engine (128 dimensions).
/// Operates without external heavy models or Python sidecars.
pub struct DenseEmbedder;

impl DenseEmbedder {
    pub const DIMENSIONS: usize = 128;

    /// Generates a normalized 128-dim embedding vector from text.
    pub fn embed_text(text: &str) -> Vec<f32> {
        let mut vector = vec![0.0f32; Self::DIMENSIONS];
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        if tokens.is_empty() {
            return vector;
        }

        for token in &tokens {
            let bytes = token.as_bytes();
            let mut hash = 0x811c9dc5u32;
            for b in bytes {
                hash ^= *b as u32;
                hash = hash.wrapping_mul(0x01000193);
            }

            let idx = (hash as usize) % Self::DIMENSIONS;
            let sign = if (hash >> 16) % 2 == 0 { 1.0f32 } else { -1.0f32 };
            let weight = (token.len() as f32).sqrt();
            vector[idx] += sign * weight;
        }

        // Unit normalization (L2 norm)
        let norm_sq: f32 = vector.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-6 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }

        vector
    }

    /// Computes cosine similarity between two unit vectors.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        dot.clamp(-1.0, 1.0)
    }
}
