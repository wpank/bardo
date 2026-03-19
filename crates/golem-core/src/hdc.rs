//! 10,240-bit HDC primitive stub.

use core::convert::TryFrom;
use uuid::Uuid;

/// 10,240-bit binary sparse distributed vector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HdcVector {
    bits: [u64; 160],
}

impl HdcVector {
    /// Returns an all-zero vector.
    #[must_use]
    pub const fn zeros() -> Self {
        Self { bits: [0; 160] }
    }

    /// Returns a pseudo-random vector.
    #[must_use]
    pub fn random() -> Self {
        fn splitmix64(state: &mut u64) -> u64 {
            *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = *state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }

        let seed = Uuid::new_v4().as_u128();
        let seed_bytes = seed.to_le_bytes();
        let mut low_bytes = [0u8; 8];
        low_bytes.copy_from_slice(&seed_bytes[..8]);
        let mut high_bytes = [0u8; 8];
        high_bytes.copy_from_slice(&seed_bytes[8..]);
        let mut state = u64::from_le_bytes(low_bytes) ^ u64::from_le_bytes(high_bytes);
        if state == 0 {
            state = 0xA5A5_A5A5_5A5A_5A5A;
        }

        let mut bits = [0u64; 160];
        for word in &mut bits {
            *word = splitmix64(&mut state);
        }
        Self { bits }
    }

    /// Binds two vectors using XOR.
    #[must_use]
    pub fn bind(&self, other: &Self) -> Self {
        let mut bits = [0u64; 160];
        for (slot, (left, right)) in bits.iter_mut().zip(self.bits.iter().zip(other.bits.iter())) {
            *slot = left ^ right;
        }
        Self { bits }
    }

    /// Bundles vectors using majority vote.
    #[must_use]
    pub fn bundle(vectors: &[&Self]) -> Self {
        if vectors.is_empty() {
            return Self::zeros();
        }

        let len = vectors.len();
        let mut bits = [0u64; 160];
        for (word_index, slot) in bits.iter_mut().enumerate() {
            let mut word = 0u64;
            for bit_index in 0..64 {
                let mut ones = 0usize;
                for vector in vectors {
                    ones += ((vector.bits[word_index] >> bit_index) & 1) as usize;
                }
                if ones * 2 > len {
                    word |= 1u64 << bit_index;
                }
            }
            *slot = word;
        }
        Self { bits }
    }

    /// Rotates bits left by `n` positions.
    #[must_use]
    pub fn permute(&self, n: usize) -> Self {
        let bits_len = self.bits.len() * 64;
        let n = n % bits_len;
        if n == 0 {
            return self.clone();
        }

        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut bits = [0u64; 160];

        for (index, slot) in bits.iter_mut().enumerate() {
            let src0 = (index + 160 - word_shift) % 160;
            *slot = if bit_shift == 0 {
                self.bits[src0]
            } else {
                let src1 = (src0 + 159) % 160;
                (self.bits[src0] << bit_shift) | (self.bits[src1] >> (64 - bit_shift))
            };
        }

        Self { bits }
    }

    /// Returns the Hamming similarity in the range `[0, 1]`.
    pub fn similarity(&self, other: &Self) -> f32 {
        let mut differing_bits = 0u32;
        for (left, right) in self.bits.iter().zip(other.bits.iter()) {
            differing_bits += (left ^ right).count_ones();
        }
        let differing_bits = u16::try_from(differing_bits).unwrap_or(u16::MAX);
        1.0_f32 - (f32::from(differing_bits) / 10_240.0_f32)
    }
}

#[cfg(test)]
mod tests {
    use super::HdcVector;

    #[test]
    fn hdc_bind_involution() {
        let a = HdcVector::random();
        let b = HdcVector::random();
        let recovered = a.bind(&b).bind(&b);
        assert!((recovered.similarity(&a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hdc_similarity_self() {
        let vector = HdcVector::random();
        assert!((vector.similarity(&vector) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hdc_bundle_tie_rule() {
        let mut a = HdcVector::zeros();
        let mut b = HdcVector::zeros();
        a.bits[0] = 1;
        b.bits[0] = 0;
        let bundled = HdcVector::bundle(&[&a, &b]);
        assert_eq!(bundled.bits[0], 0);
    }
}
