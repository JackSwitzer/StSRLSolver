//! 256-bit effect flags bitset for O(1) tag checking in the MCTS hot path.
//!
//! Each card effect tag maps to a bit position (0..255). Checking whether a card
//! has an effect becomes a single AND instruction instead of a linear string scan.

/// 256-bit bitset stored as 4 × u64. Each bit corresponds to one registered effect tag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectFlags(pub [u64; 4]);

impl EffectFlags {
    pub const EMPTY: Self = Self([0; 4]);

    /// Check if a specific bit is set.
    #[inline(always)]
    pub fn has(&self, bit: u8) -> bool {
        let word = (bit / 64) as usize;
        let mask = 1u64 << (bit % 64);
        self.0[word] & mask != 0
    }

    /// Set a specific bit.
    #[inline(always)]
    pub fn set(&mut self, bit: u8) {
        let word = (bit / 64) as usize;
        let mask = 1u64 << (bit % 64);
        self.0[word] |= mask;
    }

    /// Fast check: does this bitset overlap with a mask?
    /// Used for "does this card have ANY effects that fire on hook X?"
    #[inline(always)]
    pub fn intersects(&self, mask: &EffectFlags) -> bool {
        (self.0[0] & mask.0[0]) != 0
            || (self.0[1] & mask.0[1]) != 0
            || (self.0[2] & mask.0[2]) != 0
            || (self.0[3] & mask.0[3]) != 0
    }

    /// Combine two flag sets (OR).
    #[inline(always)]
    pub fn union(&self, other: &EffectFlags) -> EffectFlags {
        EffectFlags([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_has() {
        let mut flags = EffectFlags::EMPTY;
        assert!(!flags.has(0));
        assert!(!flags.has(42));
        assert!(!flags.has(255));

        flags.set(0);
        flags.set(42);
        flags.set(255);

        assert!(flags.has(0));
        assert!(flags.has(42));
        assert!(flags.has(255));
        assert!(!flags.has(1));
        assert!(!flags.has(100));
    }

    #[test]
    fn test_intersects() {
        let mut a = EffectFlags::EMPTY;
        let mut b = EffectFlags::EMPTY;

        a.set(10);
        b.set(20);
        assert!(!a.intersects(&b));

        b.set(10);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<EffectFlags>(), 32);
    }
}
