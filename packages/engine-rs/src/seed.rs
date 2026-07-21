//! STS-compatible RNG using xorshift128+ (matches libGDX RandomXS128).
//!
//! The Java STS game uses `com.badlogic.gdx.math.RandomXS128` which is
//! the xorshift128+ algorithm. This module provides an exact port so that
//! seed-for-seed output matches the Java game.
//!
//! Also includes SeedHelper string<->long conversion (base-35 encoding
//! with 'O' mapped to '0').

use serde::{Deserialize, Serialize};

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
// RandomXs128 — private libGDX generator
// ===========================================================================

/// Native Rust port of libGDX 1.9.5's `RandomXS128`.
///
/// This type deliberately stays private: counted gameplay draws go through
/// `StsRandom`, while ambient libGDX draws go through `AmbientMathRng`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RandomXs128 {
    seed0: u64,
    seed1: u64,
}

impl RandomXs128 {
    fn new(seed: u64) -> Self {
        // RandomXS128 substitutes Long.MIN_VALUE before hashing a zero seed.
        // Source: com.badlogic.gdx.math.RandomXS128 in desktop-1.0.jar.
        let seed = if seed == 0 { i64::MIN as u64 } else { seed };
        let s0 = murmur_hash3(seed);
        let s1 = murmur_hash3(s0);
        Self {
            seed0: s0,
            seed1: s1,
        }
    }

    fn from_state(seed0: u64, seed1: u64) -> Self {
        Self { seed0, seed1 }
    }

    fn state_tuple(&self) -> (u64, u64) {
        (self.seed0, self.seed1)
    }

    fn next_long(&mut self) -> u64 {
        let mut s1 = self.seed0;
        let s0 = self.seed1;
        self.seed0 = s0;

        s1 ^= s1 << 23;
        self.seed1 = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26);

        self.seed1.wrapping_add(s0)
    }

    fn next_long_bounded(&mut self, bound: i64) -> i64 {
        assert!(bound > 0, "bound must be positive");
        loop {
            let bits = (self.next_long() >> 1) as i64;
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        self.next_long_bounded(bound as i64) as i32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_long() & 1) != 0
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_long() >> 40) as f64 * (1.0 / (1_u64 << 24) as f64)) as f32
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_long() >> 11) as f64 * (1.0 / (1_u64 << 53) as f64)
    }
}

// ===========================================================================
// AmbientMathRng — uncounted libGDX MathUtils.random owner
// ===========================================================================

/// Deterministic owner for gameplay-significant draws made through libGDX's
/// ambient `MathUtils.random` generator.
///
/// Unlike `StsRandom`, this stream has no wrapper counter and is not derived
/// from a dungeon seed implicitly. Its owner must construct, persist, and
/// restore it explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AmbientMathRng {
    inner: RandomXs128,
}

impl AmbientMathRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            inner: RandomXs128::new(seed),
        }
    }

    pub(crate) fn from_state(seed0: u64, seed1: u64) -> Self {
        Self {
            inner: RandomXs128::from_state(seed0, seed1),
        }
    }

    pub(crate) fn state_tuple(&self) -> (u64, u64) {
        self.inner.state_tuple()
    }

    pub(crate) fn restore_state(&mut self, seed0: u64, seed1: u64) {
        self.inner = RandomXs128::from_state(seed0, seed1);
    }

    /// Match libGDX `MathUtils.random(long range)` for the positive ranges used
    /// by the game. Unlike `RandomXS128.nextLong(long)`, this uses `nextDouble`.
    #[allow(dead_code)] // Required by the source-complete MathUtils oracle surface.
    fn random_long(&mut self, max_exclusive: i64) -> i64 {
        (self.inner.next_f64() * max_exclusive as f64) as i64
    }

    /// Match the signed result of libGDX `RandomXS128.nextLong()`.
    #[allow(dead_code)] // Required by the source-complete MathUtils oracle surface.
    pub(crate) fn random_long_unbounded(&mut self) -> i64 {
        self.inner.next_long() as i64
    }

    /// Match libGDX `MathUtils.random(int range)`: both endpoints are inclusive.
    pub(crate) fn random_int(&mut self, max_inclusive: i32) -> i32 {
        self.inner.next_int(max_inclusive.wrapping_add(1))
    }

    /// Match libGDX `MathUtils.randomBoolean()`.
    pub(crate) fn random_bool(&mut self) -> bool {
        self.inner.next_bool()
    }

    /// Match libGDX `MathUtils.random()`.
    pub(crate) fn random_f32(&mut self) -> f32 {
        self.inner.next_f32()
    }

    /// Match libGDX `MathUtils.random(float start, float end)`.
    pub(crate) fn random_f32_range(&mut self, start: f32, end: f32) -> f32 {
        start + self.inner.next_f32() * (end - start)
    }
}

// ===========================================================================
// StsRandom — com.megacrit.cardcrawl.random.Random
// ===========================================================================

/// Exact native Rust equivalent of Slay the Spire's counted RNG wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StsRandom {
    inner: RandomXs128,
    pub counter: i32,
}

/// Streams that persist for the lifetime of a dungeon run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PersistentRngs {
    pub card: StsRandom,
    pub monster: StsRandom,
    pub event: StsRandom,
    pub relic: StsRandom,
    pub treasure: StsRandom,
    pub merchant: StsRandom,
    pub potion: StsRandom,
}

impl PersistentRngs {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            card: StsRandom::new(seed),
            monster: StsRandom::new(seed),
            event: StsRandom::new(seed),
            relic: StsRandom::new(seed),
            treasure: StsRandom::new(seed),
            merchant: StsRandom::new(seed),
            potion: StsRandom::new(seed),
        }
    }
}

/// Streams rebuilt from `Settings.seed + floorNum` at each room transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FloorRngs {
    pub monster_hp: StsRandom,
    pub ai: StsRandom,
    pub shuffle: StsRandom,
    pub card_random: StsRandom,
    pub misc: StsRandom,
}

impl FloorRngs {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            monster_hp: StsRandom::new(seed),
            ai: StsRandom::new(seed),
            shuffle: StsRandom::new(seed),
            card_random: StsRandom::new(seed),
            misc: StsRandom::new(seed),
        }
    }

    #[cfg(test)]
    pub(crate) fn combat_snapshot(&self, persistent: &PersistentRngs) -> CombatRngs {
        self.combat_snapshot_with_globals(
            persistent,
            AmbientMathRng::new(0),
            JavaCollectionsRng::deterministic_default(),
        )
    }

    /// Transfer the process-global Collections RNG into combat alongside the
    /// dungeon streams. `RunEngine` owns this ambient LCG and must use this path
    /// rather than reconstructing the deterministic test default per combat.
    pub(crate) fn combat_snapshot_with_globals(
        &self,
        persistent: &PersistentRngs,
        ambient_math: AmbientMathRng,
        java_collections: JavaCollectionsRng,
    ) -> CombatRngs {
        CombatRngs {
            card: persistent.card.clone(),
            monster_hp: self.monster_hp.clone(),
            shuffle: self.shuffle.clone(),
            card_random: self.card_random.clone(),
            potion: persistent.potion.clone(),
            misc: self.misc.clone(),
            ai: self.ai.clone(),
            ambient_math,
            java_collections,
        }
    }
}

/// The exact dungeon RNG ownership transferred into and out of combat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CombatRngs {
    pub card: StsRandom,
    pub monster_hp: StsRandom,
    pub shuffle: StsRandom,
    pub card_random: StsRandom,
    pub potion: StsRandom,
    pub misc: StsRandom,
    pub ai: StsRandom,
    pub ambient_math: AmbientMathRng,
    pub java_collections: JavaCollectionsRng,
}

impl CombatRngs {
    /// Return combat-owned state to the two canonical run-level groups in one
    /// operation. Java keeps `cardRng` and `potionRng` persistent while the
    /// other five streams are floor-local.
    pub(crate) fn absorb_into(
        self,
        persistent: &mut PersistentRngs,
        floor: &mut FloorRngs,
    ) -> (AmbientMathRng, JavaCollectionsRng) {
        persistent.card = self.card;
        persistent.potion = self.potion;
        floor.monster_hp = self.monster_hp;
        floor.shuffle = self.shuffle;
        floor.card_random = self.card_random;
        floor.misc = self.misc;
        floor.ai = self.ai;
        (self.ambient_math, self.java_collections)
    }
}

impl StsRandom {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: RandomXs128::new(seed),
            counter: 0,
        }
    }

    /// Model Java's no-argument `Random()` constructor, including its two
    /// process-global `MathUtils.random` draws in Java's left-to-right order.
    ///
    /// Source: decompiled/java-src/com/megacrit/cardcrawl/random/Random.java:22-24
    /// and the shipped libGDX `MathUtils.random(long/int)` overloads.
    #[allow(dead_code)] // Java API parity; no faithful live call site currently constructs this way.
    fn from_ambient(ambient: &mut AmbientMathRng) -> Self {
        let seed = ambient.random_long(9_999);
        let counter = ambient.random_int(99);
        Self::with_counter(seed as u64, counter)
    }

    /// Java's `Random(Long seed, int counter)` constructor advances by calling
    /// `random(999)` exactly `counter` times.
    pub fn with_counter(seed: u64, counter: i32) -> Self {
        let mut rng = Self::new(seed);
        for _ in 0..counter.max(0) {
            rng.random_int(999);
        }
        rng
    }

    pub fn from_state(seed0: u64, seed1: u64, counter: i32) -> Self {
        Self {
            inner: RandomXs128::from_state(seed0, seed1),
            counter,
        }
    }

    pub fn state_tuple(&self) -> (u64, u64, i32) {
        let (seed0, seed1) = self.inner.state_tuple();
        (seed0, seed1, self.counter)
    }

    /// Match Java `Random.copy()`: constructing the temporary copy consumes
    /// two ambient draws before its local state and counter are overwritten.
    ///
    /// Source: decompiled/java-src/com/megacrit/cardcrawl/random/Random.java:37-41.
    #[allow(dead_code)] // Java API parity; retained for fixtures and future exact copy call sites.
    pub(crate) fn copy(&self, ambient: &mut AmbientMathRng) -> Self {
        let mut copied = Self::from_ambient(ambient);
        copied.inner = self.inner.clone();
        copied.counter = self.counter;
        copied
    }

    /// Match Java `Random.random(int range)`: both endpoints are inclusive.
    pub fn random_int(&mut self, max_inclusive: i32) -> i32 {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_int(max_inclusive.wrapping_add(1))
    }

    /// Match Java `Random.random(int start, int end)`: both endpoints are inclusive.
    pub fn random_int_range(&mut self, start: i32, end_inclusive: i32) -> i32 {
        self.counter = self.counter.wrapping_add(1);
        let width = end_inclusive.wrapping_sub(start).wrapping_add(1);
        start.wrapping_add(self.inner.next_int(width))
    }

    /// Match Java `Random.random(long range)`, which uses `nextDouble` rather than
    /// `RandomXS128.nextLong(long)` and therefore has an exclusive upper bound.
    pub fn random_long(&mut self, max_exclusive: i64) -> i64 {
        self.counter = self.counter.wrapping_add(1);
        (self.inner.next_f64() * max_exclusive as f64) as i64
    }

    /// Match Java `Random.random(long start, long end)`. Java's upper endpoint is
    /// exclusive because the implementation multiplies a `[0, 1)` double.
    pub fn random_long_range(&mut self, start: i64, end_exclusive: i64) -> i64 {
        self.counter = self.counter.wrapping_add(1);
        let width = end_exclusive.wrapping_sub(start);
        start.wrapping_add((self.inner.next_f64() * width as f64) as i64)
    }

    pub fn random_long_unbounded(&mut self) -> i64 {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_long() as i64
    }

    pub fn random_bool(&mut self) -> bool {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_bool()
    }

    pub fn random_bool_chance(&mut self, chance: f32) -> bool {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_f32() < chance
    }

    pub fn random_f32(&mut self) -> f32 {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_f32()
    }

    pub fn random_f32_scaled(&mut self, range: f32) -> f32 {
        self.counter = self.counter.wrapping_add(1);
        self.inner.next_f32() * range
    }

    pub fn random_f32_range(&mut self, start: f32, end: f32) -> f32 {
        self.counter = self.counter.wrapping_add(1);
        start + self.inner.next_f32() * (end - start)
    }

    pub(crate) fn random_index(&mut self, len: usize) -> usize {
        assert!(len > 0 && len <= i32::MAX as usize, "invalid random index length");
        self.random_int(len as i32 - 1) as usize
    }

    /// Java's room assignment passes the wrapped `RandomXS128` directly to
    /// `Collections.shuffle`. This advances the inner state for every swap but
    /// intentionally does not increment `com.megacrit...Random.counter`.
    pub(crate) fn shuffle_with_inner<T>(&mut self, values: &mut [T]) {
        for len in (2..=values.len()).rev() {
            let other = self.inner.next_int(len as i32) as usize;
            values.swap(len - 1, other);
        }
    }

    /// Match Java's forward-only `setCounter`. Requests at or below the current
    /// counter are intentionally a no-op, matching the game's logged error path.
    pub fn set_counter(&mut self, target: i32) {
        if self.counter < target {
            // Java computes this distance as a signed int. An overflowing
            // positive-looking interval therefore becomes negative and the
            // loop performs no draws.
            let count = target.wrapping_sub(self.counter);
            for _ in 0..count {
                self.random_bool();
            }
        }
    }
}

/// Shuffle a slice with `java.util.Random`, seeded the same way as
/// `Collections.shuffle`. Slay the Spire's `CardGroup.shuffle` consumes one
/// `StsRandom.randomLong()` and passes that value to this distinct RNG.
///
/// Source: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct JavaUtilRandom {
    seed: u64,
}

impl JavaUtilRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1_u64 << 48) - 1;

    fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn from_internal_state(seed: u64) -> Self {
        Self {
            seed: seed & Self::MASK,
        }
    }

    fn internal_state(&self) -> u64 {
        self.seed
    }

    fn next_bits(&mut self, bits: u32) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as u32
    }

    fn next_int(&mut self, bound: usize) -> usize {
        assert!(bound > 0 && bound <= i32::MAX as usize, "invalid Java int bound");
        if bound.is_power_of_two() {
            return ((bound as u64 * self.next_bits(31) as u64) >> 31) as usize;
        }
        loop {
            let bits = self.next_bits(31) as usize;
            let value = bits % bound;
            if bits - value + (bound - 1) < (1_usize << 31) {
                return value;
            }
        }
    }

    #[cfg(test)]
    fn next_i32(&mut self) -> i32 {
        self.next_bits(32) as i32
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for len in (2..=values.len()).rev() {
            let other = self.next_int(len);
            values.swap(len - 1, other);
        }
    }
}

/// Transferable state for the static default `Random` used by no-argument
/// `Collections.shuffle`.
///
/// Java initializes that generator from process/time state, not the dungeon
/// seed. Standalone simulation therefore uses Java seed `0` as an explicit,
/// deterministic boundary default. Exact trace replay must inject the captured
/// 48-bit internal state instead of pretending it can be derived from a run
/// seed.
///
/// Source: JDK 8 `java.util.Collections.shuffle(List)` and
/// `actions/common/DiscardAtEndOfTurnAction.java`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JavaCollectionsRng {
    inner: JavaUtilRandom,
}

impl JavaCollectionsRng {
    const DETERMINISTIC_SIMULATOR_SEED: i64 = 0;

    pub(crate) fn deterministic_default() -> Self {
        Self {
            inner: JavaUtilRandom::new(Self::DETERMINISTIC_SIMULATOR_SEED),
        }
    }

    pub(crate) fn from_state(state: u64) -> Self {
        Self {
            inner: JavaUtilRandom::from_internal_state(state),
        }
    }

    pub(crate) fn state(&self) -> u64 {
        self.inner.internal_state()
    }

    pub(crate) fn restore_state(&mut self, state: u64) {
        *self = Self::from_state(state);
    }

    pub(crate) fn shuffle<T>(&mut self, values: &mut [T]) {
        self.inner.shuffle(values);
    }
}

pub(crate) fn java_util_shuffle<T>(values: &mut [T], random_seed: i64) {
    JavaUtilRandom::new(random_seed).shuffle(values);
}

/// Match `CardGroup.shuffle(Random)`: consume one outer stream tick, then use
/// that value to seed the independent `java.util.Random` permutation.
///
/// Source: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java:550-555
pub(crate) fn card_group_shuffle<T>(values: &mut [T], rng: &mut StsRandom) {
    let random_seed = rng.random_long_unbounded();
    java_util_shuffle(values, random_seed);
}

// ===========================================================================
// SeedHelper — base-35 string <-> u64 conversion
// ===========================================================================

const CHARACTERS: &str = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ";
const BASE: u64 = 35;

/// Convert a seed string to u64. Matches STS SeedHelper.getLong().
/// 'O' is mapped to '0'. Case insensitive.
pub fn seed_from_string(s: &str) -> u64 {
    let s = s.to_uppercase().replace('O', "0");
    let mut total: u64 = 0;
    for code_unit in s.encode_utf16() {
        // Java iterates UTF-16 code units. SeedHelper.getLong logs invalid
        // input but still folds indexOf == -1 into the returned long.
        let idx = CHARACTERS
            .as_bytes()
            .iter()
            .position(|candidate| u16::from(*candidate) == code_unit)
            .map(|index| index as i64)
            .unwrap_or(-1);
        total = total.wrapping_mul(BASE).wrapping_add(idx as u64);
    }
    total
}

/// Convert a u64 seed to display string. Matches STS SeedHelper.getString().
/// Uses base-35 encoding (skipping 'O').
pub fn seed_to_string(seed: u64) -> String {
    if seed == 0 {
        return String::new();
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
            assert_eq!(rng1.random_long_unbounded(), rng2.random_long_unbounded());
        }
        assert_eq!(rng1.counter, 100);
        assert_eq!(rng1.state_tuple(), rng2.state_tuple());
    }

    #[test]
    fn sts_random_different_seeds() {
        let mut rng1 = StsRandom::new(42);
        let mut rng2 = StsRandom::new(123);
        // Overwhelmingly likely to differ
        assert_ne!(rng1.random_long_unbounded(), rng2.random_long_unbounded());
    }

    #[test]
    fn sts_random_clone_preserves_stream_state() {
        let mut rng = StsRandom::new(42);
        for _ in 0..10 {
            rng.random_long_unbounded();
        }
        let mut copy = rng.clone();
        for _ in 0..100 {
            assert_eq!(rng.random_long_unbounded(), copy.random_long_unbounded());
        }
        assert_eq!(rng.state_tuple(), copy.state_tuple());
    }

    #[test]
    fn next_int_in_range() {
        let mut rng = RandomXs128::new(42);
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
            let val = rng.random_int(5);
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
            let val = rng.random_int_range(3, 7);
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
        rng.random_int(5);
        assert_eq!(rng.counter, 1);
        rng.random_bool();
        assert_eq!(rng.counter, 2);
        rng.random_int_range(1, 10);
        assert_eq!(rng.counter, 3);
    }

    #[test]
    fn seed_zero_not_absorbing() {
        let mut rng = RandomXs128::new(0);
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
    fn combat_rng_transfer_preserves_java_collections_state() {
        let mut persistent = PersistentRngs::new(42);
        let mut floor = FloorRngs::new(43);
        let injected = JavaCollectionsRng::from_state(0x1234_5678_9ABC);
        let ambient = AmbientMathRng::from_state(0x1111, 0x2222);

        let combat = floor.combat_snapshot_with_globals(&persistent, ambient, injected);
        assert_eq!(combat.java_collections.state(), 0x1234_5678_9ABC);
        assert_eq!(combat.ambient_math.state_tuple(), (0x1111, 0x2222));

        let (ambient, collections) = combat.absorb_into(&mut persistent, &mut floor);
        assert_eq!(ambient.state_tuple(), (0x1111, 0x2222));
        assert_eq!(collections.state(), 0x1234_5678_9ABC);
    }

    #[test]
    fn random_xs128_bounded_ints_match_shipped_java_class() {
        // Oracle generated directly from the shipped desktop-1.0.jar
        // com.badlogic.gdx.math.RandomXS128 class.
        let cases = [
            (0, [72, 60, 52, 92, 31, 68, 42, 24]),
            (1, [55, 5, 88, 32, 21, 19, 63, 84]),
            (4, [19, 33, 83, 6, 31, 43, 57, 53]),
            (42, [24, 41, 71, 88, 61, 27, 25, 23]),
            (57_554_006_466, [56, 22, 0, 1, 20, 77, 89, 72]),
        ];

        for (seed, expected) in cases {
            let mut rng = RandomXs128::new(seed);
            let actual = std::array::from_fn(|_| rng.next_int(100));
            assert_eq!(actual, expected, "seed {seed}");
        }
    }

    #[test]
    fn random_xs128_zero_seed_long_sequence_matches_shipped_java_class() {
        let mut rng = RandomXs128::new(0);
        assert_eq!(rng.next_long() as i64, 2_940_871_956_904_845_945);
        assert_eq!(rng.next_long() as i64, -1_645_442_809_927_433_695);
        assert_eq!(rng.next_long() as i64, -890_117_169_686_220_111);
    }

    #[test]
    fn next_int_rejection_sampling_uniformity() {
        // Test that next_int with non-power-of-2 bound is reasonably uniform
        let mut rng = RandomXs128::new(42);
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
    fn next_int_power_of_two_bound() {
        let mut rng = RandomXs128::new(42);
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
                // SeedHelper.getString(0) returns an empty string.
                let s = seed_to_string(*seed);
                assert_eq!(s, "");
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
    fn seed_helper_matches_java_base_35_vectors() {
        // Oracle: decompiled/java-src/com/megacrit/cardcrawl/helpers/SeedHelper.java:62-91.
        // Re-verified: the alphabet has 35 entries and treats a Java long as unsigned.
        assert_eq!(seed_from_string("WATCHER"), 57_554_006_466);
        assert_eq!(seed_to_string(57_554_006_466), "WATCHER");
        assert_eq!(seed_to_string(0), "");
        assert_eq!(seed_from_string("1O"), seed_from_string("10"));
        assert_eq!(seed_to_string(u64::MAX), "5G24A25UXKXFF");
        assert_eq!(seed_to_string(i64::MIN as u64), "2QIJMIKEYSYQ8");
        assert_eq!(seed_from_string("😀"), (-36_i64) as u64);
        assert_eq!(seed_from_string("?"), (-1_i64) as u64);
    }

    #[test]
    fn random_xs128_seed_boundaries_match_shipped_java_class() {
        let cases = [
            (0_u64, (0x8f78_0810_af31_a493, 0xd1f9_a22a_f8e8_3383)),
            (1_u64, (0xb456_bcfc_34c2_cb2c, 0x7d6e_4ac3_8b2b_1be2)),
            (u64::MAX, (0x64b5_720b_4b82_5f21, 0xfa60_5f44_aea3_667d)),
            (i64::MIN as u64, (0x8f78_0810_af31_a493, 0xd1f9_a22a_f8e8_3383)),
            (i64::MAX as u64, (0xabb9_3df0_a930_edea, 0xe723_0606_8b6e_596a)),
        ];
        for (seed, expected) in cases {
            assert_eq!(RandomXs128::new(seed).state_tuple(), expected, "seed {seed}");
        }
    }

    #[test]
    fn ambient_math_rng_zero_seed_state_and_first_draw_match_libgdx() {
        // Oracle: shipped libGDX 1.9.5 RandomXS128.
        let mut rng = AmbientMathRng::new(0);
        assert_eq!(
            rng.state_tuple(),
            (0x8f78_0810_af31_a493, 0xd1f9_a22a_f8e8_3383)
        );
        assert_eq!(rng.random_long_unbounded(), 2_940_871_956_904_845_945);
        assert_eq!(
            rng.state_tuple(),
            (0xd1f9_a22a_f8e8_3383, 0x56d6_714b_a850_6ef6)
        );
    }

    #[test]
    fn ambient_math_rng_unbounded_long_preserves_libgdx_signed_result() {
        // Oracle: shipped libGDX 1.9.5 RandomXS128.nextLong().
        let cases = [
            (42, 3_553_440_125_194_606_449_i64),
            (u64::MAX, -7_651_268_203_606_709_133_i64),
        ];
        for (seed, expected) in cases {
            assert_eq!(AmbientMathRng::new(seed).random_long_unbounded(), expected);
        }
    }

    #[test]
    fn ambient_math_rng_inclusive_int_matches_math_utils() {
        // Oracle: shipped libGDX 1.9.5 MathUtils.random(int range).
        assert_eq!(AmbientMathRng::new(0).random_int(9), 2);
        assert_eq!(AmbientMathRng::new(1).random_int(9), 5);
    }

    #[test]
    fn ambient_math_rng_exact_state_clones_serializes_and_restores() {
        let initial = (0x8f78_0810_af31_a493, 0xd1f9_a22a_f8e8_3383);
        let original = AmbientMathRng::from_state(initial.0, initial.1);
        let encoded = serde_json::to_string(&original).unwrap();
        let mut restored: AmbientMathRng = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored, original);

        let mut cloned = original.clone();
        assert_eq!(restored.random_int(9), cloned.random_int(9));
        assert_eq!(restored.state_tuple(), cloned.state_tuple());

        restored.restore_state(initial.0, initial.1);
        assert_eq!(restored, original);
        assert_eq!(restored.random_int(9), 2);
    }

    #[test]
    fn no_arg_sts_random_consumes_ambient_seed_then_counter() {
        // Oracle: Random.java:22-24. With shipped MathUtils/RandomXS128 seeded
        // to 42, random(long 9999) returns 1926 before random(int 99) returns 41.
        let mut ambient = AmbientMathRng::new(42);
        let constructed = StsRandom::from_ambient(&mut ambient);

        assert_eq!(
            constructed.state_tuple(),
            (0xacc1_a75d_2515_14a3, 0x42fa_4c3f_20af_7e63, 41),
        );
        assert_eq!(
            ambient.state_tuple(),
            (0x0042_8d49_06e5_8ed6, 0xe60f_ca85_4041_3235),
        );
    }

    #[test]
    fn sts_random_copy_consumes_ambient_constructor_draws_before_overwrite() {
        // Oracle: Random.java:37-41 constructs `new Random()` first, then
        // replaces that temporary's RandomXS128 state and wrapper counter.
        let mut source = StsRandom::new(73);
        source.random_int(999);
        source.random_bool();
        assert_eq!(
            source.state_tuple(),
            (0xe119_9a87_b502_5022, 0xe038_c969_1899_ff69, 2),
        );

        let mut ambient = AmbientMathRng::new(42);
        let copied = source.copy(&mut ambient);

        assert_eq!(copied, source);
        assert_eq!(
            ambient.state_tuple(),
            (0x0042_8d49_06e5_8ed6, 0xe60f_ca85_4041_3235),
        );
    }

    #[test]
    fn signed_seed_boundaries_match_java_first_draws() {
        let cases = [
            (0_u64, 2_940_871_956_904_845_945_i64),
            (42_u64, 3_553_440_125_194_606_449_i64),
            (u64::MAX, -7_651_268_203_606_709_133_i64),
            (i64::MIN as u64, 2_940_871_956_904_845_945_i64),
            (i64::MAX as u64, -7_209_822_827_912_203_100_i64),
        ];
        for (seed, expected) in cases {
            assert_eq!(StsRandom::new(seed).random_long_unbounded(), expected);
        }
    }

    #[test]
    fn random_xs128_primitives_match_shipped_java_class() {
        let mut floats = RandomXs128::new(42);
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|_| floats.next_f32().to_bits()),
            [0x3e45_4168, 0x3f66_5257, 0x3e23_78c4, 0x3e24_c118]
        );

        let mut doubles = RandomXs128::new(42);
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|_| doubles.next_f64().to_bits()),
            [
                0x3fc8_a82d_7bc4_6c08,
                0x3fec_ca4a_f9c8_e4d8,
                0x3fc4_6f18_ebc8_e92c,
                0x3fc4_9823_2dcf_ac84,
            ]
        );

        let mut booleans = RandomXs128::new(42);
        assert_eq!(
            std::array::from_fn::<_, 8, _>(|_| booleans.next_bool()),
            [true, true, false, true, false, true, true, false]
        );
    }

    #[test]
    fn bounded_rejection_consumes_raw_state_without_extra_wrapper_ticks() {
        let mut rng = StsRandom::from_state(0x07e0_7ff0_3fff_e000, 0, 0);
        assert_eq!(rng.random_int(i32::MAX - 1), 2_147_483_584_u32 as i32);
        assert_eq!(rng.counter, 1);
        assert_eq!(rng.state_tuple(), (u64::MAX, 0xffff_ffc0_0000_0000, 1));
    }

    #[test]
    fn invalid_bounds_increment_counter_without_advancing_inner_state() {
        let mut rng = StsRandom::new(42);
        let before = rng.state_tuple();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rng.random_int(-1);
        }))
        .is_err());
        assert_eq!(rng.counter, 1);
        assert_eq!((rng.state_tuple().0, rng.state_tuple().1), (before.0, before.1));

        rng.counter = i32::MAX;
        assert_eq!(rng.random_int(0), 0);
        assert_eq!(rng.counter, i32::MIN);
    }

    #[test]
    fn sts_random_mixed_overloads_match_java_derived_fixture() {
        // Oracle: Random.java:55-103 and libGDX 1.9.5 RandomXS128.java.
        // Re-verified: integer overloads are inclusive, while the long overload
        // multiplies nextDouble and therefore excludes its upper endpoint.
        let mut rng = StsRandom::new(42);
        assert_eq!(rng.random_int(999), 224);
        assert_eq!(rng.random_int_range(3, 9), 8);
        assert_eq!(rng.random_long_unbounded(), 2_944_846_008_281_095_542);
        assert!(rng.random_bool());
        assert_eq!(rng.random_f32().to_bits(), 0x3f73_e8e4);
        assert!(!rng.random_bool_chance(0.33));
        assert_eq!(rng.random_f32_range(2.0, 5.0).to_bits(), 0x403c_2f36);
        assert_eq!(rng.random_long(1_000_000), 634_725);
        assert_eq!(rng.counter, 8);
        assert_eq!(
            rng.state_tuple(),
            (16_212_654_256_975_937_342, 13_942_703_727_853_910_520, 8)
        );
    }

    #[test]
    fn each_sts_random_overload_matches_an_independent_java_fixture() {
        // Each assertion starts from Random(Long.valueOf(42)). Fixtures were
        // generated from the shipped Random.java and libGDX RandomXS128.
        assert_eq!(StsRandom::new(42).random_int(999), 224);
        assert_eq!(StsRandom::new(42).random_int_range(3, 9), 5);
        assert_eq!(StsRandom::new(42).random_long(1_000_000), 192_632);
        assert_eq!(StsRandom::new(42).random_long_range(-100, 100), -62);
        assert_eq!(
            StsRandom::new(42).random_long_unbounded(),
            3_553_440_125_194_606_449,
        );
        assert!(StsRandom::new(42).random_bool());
        assert!(StsRandom::new(42).random_bool_chance(0.33));
        assert_eq!(StsRandom::new(42).random_f32().to_bits(), 0x3e45_4168);
        assert_eq!(
            StsRandom::new(42).random_f32_scaled(5.0).to_bits(),
            0x3f76_91c2,
        );
        assert_eq!(
            StsRandom::new(42).random_f32_range(2.0, 5.0).to_bits(),
            0x4024_fc44,
        );
    }

    #[test]
    fn every_sts_draw_overload_ticks_the_wrapper_counter_once() {
        macro_rules! assert_one_tick {
            ($call:expr) => {{
                let mut rng = StsRandom::new(42);
                rng.counter = i32::MAX;
                $call(&mut rng);
                assert_eq!(rng.counter, i32::MIN);
            }};
        }

        assert_one_tick!(|rng: &mut StsRandom| { rng.random_int(5) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_int_range(-3, 7) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_long(5) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_long_range(-3, 7) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_long_unbounded() });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_bool() });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_bool_chance(0.5) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_f32() });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_f32_scaled(5.0) });
        assert_one_tick!(|rng: &mut StsRandom| { rng.random_f32_range(-3.0, 7.0) });
    }

    #[test]
    fn rng_state_serializes_and_restores_without_losing_signed_bits() {
        let original = StsRandom::from_state(
            i64::MIN as u64,
            (-7_i64) as u64,
            i32::MIN,
        );
        let encoded = serde_json::to_string(&original).unwrap();
        let mut restored: StsRandom = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored, original);

        let mut direct = original.clone();
        for _ in 0..8 {
            assert_eq!(restored.random_long_unbounded(), direct.random_long_unbounded());
        }

        let all_zero = StsRandom::from_state(0, 0, i32::MAX);
        assert_eq!(
            serde_json::from_str::<StsRandom>(&serde_json::to_string(&all_zero).unwrap()).unwrap(),
            all_zero,
        );
    }

    #[test]
    fn constructor_counter_and_set_counter_follow_java_advance_methods() {
        // Oracle: Random.java:30-35,44-49.
        // Re-verified: constructor replay uses random(999), while setCounter
        // advances with randomBoolean; equal/lower targets do not rewind.
        let mut constructed = StsRandom::with_counter(42, 3);
        assert_eq!(
            constructed.state_tuple(),
            (16_577_691_427_031_560_757, 4_813_898_654_959_086_401, 3)
        );
        assert!(constructed.random_bool());

        let mut advanced = StsRandom::new(42);
        advanced.set_counter(5);
        assert_eq!(
            advanced.state_tuple(),
            (16_600_794_932_517_003_392, 974_753_361_020_776_730, 5)
        );
        advanced.set_counter(3);
        assert_eq!(advanced.counter, 5);
        assert_eq!(advanced.random_int(999), 527);

        let negative = StsRandom::with_counter(42, -2);
        assert_eq!(negative.state_tuple(), StsRandom::new(42).state_tuple());

        let mut overflowed = StsRandom::from_state(1, 2, i32::MIN);
        let before = overflowed.state_tuple();
        overflowed.set_counter(0);
        assert_eq!(overflowed.state_tuple(), before);
    }

    #[test]
    fn java_util_random_shuffle_matches_collections_fixture() {
        // Oracle: CardGroup.java:550-555 and JDK 8 Collections.shuffle.
        // Re-verified: the signed randomLong bit pattern seeds a distinct
        // 48-bit java.util.Random before the backwards Fisher-Yates pass.
        let mut data: Vec<i32> = (0..10).collect();
        java_util_shuffle(&mut data, -1_645_442_809_927_433_695);
        assert_eq!(data, [1, 2, 8, 5, 7, 9, 6, 0, 4, 3]);
    }

    #[test]
    fn java_util_random_rejection_sampling_matches_jdk_fixture() {
        let mut random = JavaUtilRandom::new(0);
        assert_eq!(random.next_int(1_073_741_825), 516_548_029);
        assert_eq!(random.next_i32(), -1_690_734_402);
    }

    #[test]
    fn java_util_random_seed_boundaries_and_state_restore_match_jdk8() {
        // Oracle: JDK 8 java.util.Random.setSeed/next and the JRE bundled with
        // Slay the Spire. Constructor seeding keeps only the low 48 bits.
        let cases = [
            (
                0_i64,
                0x0005_deec_e66d,
                -1_155_484_576,
                0xbb20_b460_0a74,
                -723_955_400,
            ),
            (
                42_i64,
                0x0005_deec_e647,
                -1_170_105_035,
                0xba41_9d35_d646,
                234_785_527,
            ),
            (
                -42_i64,
                0xfffa_2113_19bb,
                1_170_874_531,
                0x45ca_20a3_f6aa,
                1_757_677_092,
            ),
            (
                i64::MIN,
                0x0005_deec_e66d,
                -1_155_484_576,
                0xbb20_b460_0a74,
                -723_955_400,
            ),
            (
                i64::MAX,
                0xfffa_2113_1992,
                1_155_099_827,
                0x44d9_6cb3_0f35,
                1_887_904_451,
            ),
        ];

        for (seed, initial_state, first, state_after_first, second) in cases {
            let mut random = JavaUtilRandom::new(seed);
            assert_eq!(random.internal_state(), initial_state, "seed {seed}");
            assert_eq!(random.next_i32(), first, "seed {seed}");
            assert_eq!(random.internal_state(), state_after_first, "seed {seed}");

            let mut restored = JavaUtilRandom::from_internal_state(state_after_first);
            assert_eq!(restored.next_i32(), second, "restored seed {seed}");
        }
    }

    #[test]
    fn direct_inner_shuffle_preserves_wrapper_counter_and_matches_java_state() {
        let mut rng = StsRandom::new(42);
        let mut values = [0, 1, 2, 3, 4, 5];
        rng.shuffle_with_inner(&mut values);
        assert_eq!(values, [0, 2, 5, 3, 1, 4]);
        assert_eq!(
            rng.state_tuple(),
            (0xe661_df09_4dc7_e080, 0x0d87_0504_7342_f11a, 0),
        );

        let state = rng.state_tuple();
        rng.shuffle_with_inner::<u8>(&mut []);
        rng.shuffle_with_inner(&mut [1]);
        assert_eq!(rng.state_tuple(), state);
    }
}
