//! STS-compatible RNG using xorshift128+ (matches libGDX RandomXS128).
//!
//! The Java STS game uses `com.badlogic.gdx.math.RandomXS128` which is
//! the xorshift128+ algorithm. This module provides an exact port so that
//! seed-for-seed output matches the Java game.
//!
//! Also includes SeedHelper string<->long conversion (base-34 encoding
//! with 'O' mapped to '0').

use rand::RngCore;

// ===========================================================================
// Murmur hash for seeding (matches libGDX RandomXS128 constructor)
// ===========================================================================

/// MurmurHash3 finalizer — used by libGDX to derive seed0/seed1 from a
/// single long seed. This is the 64-bit finalizer from MurmurHash3_x64_128.
fn murmur_hash3(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}

// ===========================================================================
// StsRandom — xorshift128+ RNG
// ===========================================================================

/// STS-compatible RNG using xorshift128+ (matches libGDX RandomXS128).
#[derive(Debug, Clone)]
pub struct StsRandom {
    seed0: u64,
    seed1: u64,
    pub counter: i32,
}

impl StsRandom {
    /// Create a new RNG from a single seed.
    /// Matches libGDX: `new RandomXS128(seed)` which calls
    /// `setState(murmurHash3(seed), murmurHash3(seed0))`.
    pub fn new(seed: u64) -> Self {
        let mut s0 = murmur_hash3(seed);
        let s1 = murmur_hash3(s0);
        // Guard: murmur_hash3(0)==0, making both seeds 0 -- an absorbing state
        // for xorshift128+. Use fallback to avoid degenerate all-zero output.
        if s0 == 0 && s1 == 0 {
            s0 = 1;
        }
        Self {
            seed0: s0,
            seed1: s1,
            counter: 0,
        }
    }

    /// Create from explicit state (for copy/restore).
    pub fn from_state(seed0: u64, seed1: u64, counter: i32) -> Self {
        Self {
            seed0,
            seed1,
            counter,
        }
    }

    /// Export the internal RNG state for deterministic snapshot/replay flows.
    pub fn state_tuple(&self) -> (u64, u64, i32) {
        (self.seed0, self.seed1, self.counter)
    }

    /// Deep copy with identical state.
    pub fn copy(&self) -> Self {
        Self {
            seed0: self.seed0,
            seed1: self.seed1,
            counter: self.counter,
        }
    }

    // -----------------------------------------------------------------------
    // Core xorshift128+ step
    // -----------------------------------------------------------------------

    /// One step of xorshift128+. Returns 64 random bits.
    ///
    /// Matches libGDX RandomXS128.nextLong() exactly:
    /// ```java
    /// long s1 = seed0;         // note: s1 reads from seed0
    /// long s0 = seed1;         // note: s0 reads from seed1
    /// seed0 = s0;              // swap
    /// s1 ^= s1 << 23;
    /// seed1 = s1 ^ s0 ^ (s1 >>> 17) ^ (s0 >>> 26);
    /// return seed1 + s0;
    /// ```
    pub fn next_long(&mut self) -> u64 {
        let mut s1 = self.seed0;
        let s0 = self.seed1;
        self.seed0 = s0;

        s1 ^= s1 << 23;
        self.seed1 = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26);

        self.seed1.wrapping_add(s0)
    }

    /// Generate the next i32 in [0, bound) — matches java.util.Random.nextInt(int)
    /// with rejection sampling to eliminate modulo bias.
    pub fn next_int(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0, "bound must be positive");
        let bound = bound as u64;

        // Power-of-2 bounds have no modulo bias
        if bound & (bound - 1) == 0 {
            let bits = (self.next_long() >> 33) as u64;
            return (bits & (bound - 1)) as i32;
        }

        // Rejection sampling: reject values where modular reduction is biased.
        // Mirrors java.util.Random.nextInt(int bound) logic.
        loop {
            let bits = (self.next_long() >> 33) as u64;
            let val = bits % bound;
            // Reject if bits - val + (bound - 1) would overflow 31-bit range
            if bits.wrapping_sub(val).wrapping_add(bound - 1) < (1u64 << 31) {
                return val as i32;
            }
        }
    }

    /// Generate i32 in [start, end] (inclusive both ends).
    /// Matches STS Random.random(start, end): `start + nextInt(end - start + 1)`.
    pub fn next_int_range(&mut self, start: i32, end: i32) -> i32 {
        self.counter += 1;
        start + self.next_int(end - start + 1)
    }

    /// Generate a random bool. Matches libGDX nextBoolean().
    pub fn next_boolean(&mut self) -> bool {
        self.counter += 1;
        (self.next_long() & 1) != 0
    }

    /// Match STS Random.random(range): returns int in [0, range] (inclusive!).
    /// This is the most-used method in STS: `random.random(range)` = `nextInt(range + 1)`.
    pub fn random(&mut self, range: i32) -> i32 {
        self.counter += 1;
        self.next_int(range + 1)
    }

    /// Match STS Random.random(start, end): returns int in [start, end] (inclusive!).
    pub fn random_range(&mut self, start: i32, end: i32) -> i32 {
        self.counter += 1;
        start + self.next_int(end - start + 1)
    }

    /// Random long — matches STS Random.randomLong().
    pub fn random_long(&mut self) -> u64 {
        self.counter += 1;
        self.next_long()
    }

    /// Random bool — matches STS Random.randomBoolean().
    pub fn random_boolean(&mut self) -> bool {
        // next_boolean already increments counter
        self.next_boolean()
    }

    /// Random float in [0, 1) — matches libGDX nextFloat().
    pub fn next_float(&mut self) -> f32 {
        // libGDX: (nextLong() >>> 40) as f64 / (1L << 24) as f64
        let bits = self.next_long() >> 40;
        (bits as f32) / ((1u64 << 24) as f32)
    }

    /// Random float in [0, 1) with counter increment.
    pub fn random_float(&mut self) -> f32 {
        self.counter += 1;
        self.next_float()
    }
}

// ===========================================================================
// Implement rand::RngCore so StsRandom works with .shuffle(), .gen_range(), etc.
// ===========================================================================

impl RngCore for StsRandom {
    fn next_u32(&mut self) -> u32 {
        self.next_long() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.next_long()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut i = 0;
        while i < dest.len() {
            let val = self.next_long();
            let bytes = val.to_le_bytes();
            let remaining = dest.len() - i;
            let to_copy = remaining.min(8);
            dest[i..i + to_copy].copy_from_slice(&bytes[..to_copy]);
            i += to_copy;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

// ===========================================================================
// SeedHelper — base-34 string <-> u64 conversion
// ===========================================================================

const CHARACTERS: &str = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ";
const BASE: u64 = 34;

/// Convert a seed string to u64. Matches STS SeedHelper.getLong().
/// 'O' is mapped to '0'. Case insensitive.
pub fn seed_from_string(s: &str) -> u64 {
    let s = s.to_uppercase().replace('O', "0");
    let mut total: u64 = 0;
    for ch in s.chars() {
        let idx = CHARACTERS.find(ch).unwrap_or(0) as u64;
        total = total.wrapping_mul(BASE).wrapping_add(idx);
    }
    total
}

/// Convert a u64 seed to display string. Matches STS SeedHelper.getString().
/// Uses base-34 encoding (skipping 'O').
pub fn seed_to_string(seed: u64) -> String {
    if seed == 0 {
        return "0".to_string();
    }

    // Java uses BigInteger for unsigned division since Java long is signed.
    // We use u128 to handle the full u64 range correctly.
    let chars: Vec<u8> = CHARACTERS.as_bytes().to_vec();
    let mut result = Vec::new();
    let mut leftover = seed as u128;
    let base = BASE as u128;

    while leftover > 0 {
        let remainder = (leftover % base) as usize;
        leftover /= base;
        result.push(chars[remainder]);
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn murmur_hash3_basic() {
        // murmur_hash3(0) == 0 because 0 is a fixed point of the finalizer.
        assert_eq!(murmur_hash3(0), 0);
        // Non-zero seed should produce a non-zero deterministic result.
        let h = murmur_hash3(42);
        assert_ne!(h, 0);
        // Same input always same output
        assert_eq!(h, murmur_hash3(42));
    }

    #[test]
    fn sts_random_deterministic() {
        let mut rng1 = StsRandom::new(42);
        let mut rng2 = StsRandom::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_long(), rng2.next_long());
        }
    }

    #[test]
    fn sts_random_different_seeds() {
        let mut rng1 = StsRandom::new(42);
        let mut rng2 = StsRandom::new(123);
        // Overwhelmingly likely to differ
        assert_ne!(rng1.next_long(), rng2.next_long());
    }

    #[test]
    fn sts_random_copy() {
        let mut rng = StsRandom::new(42);
        // Advance a few steps
        for _ in 0..10 {
            rng.next_long();
        }
        let mut copy = rng.copy();
        // Should produce same sequence
        for _ in 0..100 {
            assert_eq!(rng.next_long(), copy.next_long());
        }
    }

    #[test]
    fn next_int_in_range() {
        let mut rng = StsRandom::new(42);
        for _ in 0..1000 {
            let val = rng.next_int(10);
            assert!(val >= 0 && val < 10, "next_int(10) produced {}", val);
        }
    }

    #[test]
    fn random_inclusive_range() {
        let mut rng = StsRandom::new(42);
        let mut seen_zero = false;
        let mut seen_five = false;
        for _ in 0..1000 {
            let val = rng.random(5);
            assert!(val >= 0 && val <= 5, "random(5) produced {}", val);
            if val == 0 {
                seen_zero = true;
            }
            if val == 5 {
                seen_five = true;
            }
        }
        assert!(seen_zero, "random(5) never produced 0");
        assert!(seen_five, "random(5) never produced 5");
    }

    #[test]
    fn random_range_inclusive() {
        let mut rng = StsRandom::new(42);
        let mut seen_3 = false;
        let mut seen_7 = false;
        for _ in 0..1000 {
            let val = rng.random_range(3, 7);
            assert!(val >= 3 && val <= 7, "random_range(3,7) produced {}", val);
            if val == 3 {
                seen_3 = true;
            }
            if val == 7 {
                seen_7 = true;
            }
        }
        assert!(seen_3, "random_range(3,7) never produced 3");
        assert!(seen_7, "random_range(3,7) never produced 7");
    }

    #[test]
    fn counter_tracks_calls() {
        let mut rng = StsRandom::new(42);
        assert_eq!(rng.counter, 0);
        rng.random(5);
        assert_eq!(rng.counter, 1);
        rng.random_boolean();
        assert_eq!(rng.counter, 2);
        rng.random_range(1, 10);
        assert_eq!(rng.counter, 3);
    }

    #[test]
    fn seed_zero_not_absorbing() {
        // Seed 0 should not produce all-zero output
        let mut rng = StsRandom::new(0);
        let mut all_zero = true;
        for _ in 0..10 {
            if rng.next_long() != 0 {
                all_zero = false;
                break;
            }
        }
        assert!(
            !all_zero,
            "Seed 0 produced all-zero output (absorbing state)"
        );
    }

    #[test]
    fn next_int_rejection_sampling_uniformity() {
        // Test that next_int with non-power-of-2 bound is reasonably uniform
        let mut rng = StsRandom::new(42);
        let bound = 7;
        let mut counts = [0u32; 7];
        let n = 7000;
        for _ in 0..n {
            let val = rng.next_int(bound);
            assert!(
                val >= 0 && val < bound,
                "next_int({}) produced {}",
                bound,
                val
            );
            counts[val as usize] += 1;
        }
        // Each bucket should get roughly n/7 = 1000. Allow 30% deviation.
        let expected = n as f64 / bound as f64;
        for (i, &count) in counts.iter().enumerate() {
            let ratio = count as f64 / expected;
            assert!(
                ratio > 0.7 && ratio < 1.3,
                "Bucket {} has {} (expected ~{:.0}), ratio {:.2}",
                i,
                count,
                expected,
                ratio
            );
        }
    }

    #[test]
    fn next_int_power_of_2_fast_path() {
        let mut rng = StsRandom::new(42);
        for _ in 0..1000 {
            let val = rng.next_int(8); // power of 2
            assert!(val >= 0 && val < 8);
        }
    }

    #[test]
    fn seed_string_roundtrip() {
        // Test several known seeds
        for seed in &[0u64, 1, 42, 1000, 12345678, u64::MAX / 2, u64::MAX] {
            if *seed == 0 {
                // 0 encodes to "0", decodes back to 0
                let s = seed_to_string(*seed);
                assert_eq!(s, "0");
                assert_eq!(seed_from_string(&s), 0);
            } else {
                let s = seed_to_string(*seed);
                let decoded = seed_from_string(&s);
                assert_eq!(
                    *seed, decoded,
                    "Roundtrip failed for seed {}: encoded as '{}', decoded as {}",
                    seed, s, decoded
                );
            }
        }
    }

    #[test]
    fn seed_string_o_maps_to_zero() {
        // 'O' should be treated as '0'
        let with_o = seed_from_string("1O");
        let with_zero = seed_from_string("10");
        assert_eq!(with_o, with_zero);
    }

    #[test]
    fn seed_string_case_insensitive() {
        let upper = seed_from_string("ABC123");
        let lower = seed_from_string("abc123");
        assert_eq!(upper, lower);
    }

    #[test]
    fn rng_core_trait_works() {
        use rand::Rng;
        let mut rng = StsRandom::new(42);
        // Should be able to use rand trait methods
        let _: f64 = rng.gen();
        let _: i32 = rng.gen_range(0..10);
        let _: bool = rng.gen();
    }

    #[test]
    fn shuffle_works() {
        use rand::seq::SliceRandom;
        let mut rng = StsRandom::new(42);
        let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let original = data.clone();
        data.shuffle(&mut rng);
        // Extremely unlikely to remain in order
        assert_ne!(data, original, "Shuffle didn't change order");
    }
}
