//! Core ID newtypes for the zero-alloc engine refactor.
//!
//! Every game entity gets a newtype wrapper around u16. This prevents
//! accidental mixing of card IDs with status IDs, etc., and enables
//! fixed-size array indexing instead of HashMap lookups.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelicId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PotionId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnemyId(pub u16);

impl CardId {
    pub const NONE: CardId = CardId(0);
    pub const UNKNOWN: CardId = CardId(u16::MAX);
}

impl PotionId {
    pub const EMPTY: PotionId = PotionId(0);
}

impl RelicId {
    pub const NONE: RelicId = RelicId(u16::MAX);
}

/// Enemy move effect indices (fixed array [i32; 32]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveEffect {
    Weak = 0,
    Vulnerable = 1,
    Frail = 2,
    Strength = 3,
    Ritual = 4,
    Entangle = 5,
    Slimed = 6,
    Daze = 7,
    Burn = 8,
    BurnUpgrade = 9,
    Hex = 10,
    Heal = 11,
    Wound = 12,
    DrawReduction = 13,
    StrengthDown = 14,
    DexterityDown = 15,
    Artifact = 16,
    Constrict = 17,
    Void = 18,
    Thorns = 19,
    PainfulStabs = 20,
    // Room for expansion up to 31
}

pub const MAX_MOVE_EFFECTS: usize = 32;

// =========================================================================
// Display helpers
// =========================================================================

impl std::fmt::Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CardId({})", self.0)
    }
}

impl std::fmt::Display for StatusId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "StatusId({})", self.0)
    }
}

impl std::fmt::Display for RelicId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RelicId({})", self.0)
    }
}

impl std::fmt::Display for PotionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PotionId({})", self.0)
    }
}

impl std::fmt::Display for EnemyId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "EnemyId({})", self.0)
    }
}
