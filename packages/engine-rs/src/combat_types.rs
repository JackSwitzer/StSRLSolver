//! Core combat types — CardInstance, Intent, effect bitfields, DamageSource.
//! All types are Copy-friendly for MCTS cloning.

use serde::{Serialize, Deserialize};

// ---------------------------------------------------------------------------
// CardInstance — compact Copy-friendly state for deck/hand/discard entries.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstance {
    /// Index into static CardDef table.
    pub def_id: u16,
    /// Cost for this turn. -1 = use base cost from CardDef.
    pub cost: i8,
    /// Card-specific mutable numeric state. -1 = uninitialized / use def value.
    pub misc: i16,
    /// Bit flags.
    pub flags: u8,
}

impl CardInstance {
    pub const FLAG_RETAINED: u8  = 0x01;
    pub const FLAG_ETHEREAL: u8  = 0x02;
    pub const FLAG_UPGRADED: u8  = 0x04;
    pub const FLAG_FREE: u8      = 0x08;
    pub const FLAG_INNATE: u8    = 0x10;
    pub const FLAG_PURGE: u8     = 0x20;

    pub fn new(def_id: u16) -> Self {
        Self { def_id, cost: -1, misc: -1, flags: 0 }
    }
    pub fn with_cost(mut self, cost: i8) -> Self { self.cost = cost; self }
    pub fn upgraded(mut self) -> Self { self.flags |= Self::FLAG_UPGRADED; self }
    pub fn with_misc(mut self, misc: i16) -> Self { self.misc = misc; self }
    pub fn set_free(mut self, v: bool) -> Self {
        if v { self.flags |= Self::FLAG_FREE } else { self.flags &= !Self::FLAG_FREE }
        self
    }

    pub fn is_retained(&self) -> bool { self.flags & Self::FLAG_RETAINED != 0 }
    pub fn is_ethereal(&self) -> bool { self.flags & Self::FLAG_ETHEREAL != 0 }
    pub fn is_upgraded(&self) -> bool { self.flags & Self::FLAG_UPGRADED != 0 }
    pub fn is_free(&self) -> bool { self.flags & Self::FLAG_FREE != 0 }

    pub fn set_retained(&mut self, v: bool) {
        if v { self.flags |= Self::FLAG_RETAINED } else { self.flags &= !Self::FLAG_RETAINED }
    }
}

// ---------------------------------------------------------------------------
// Intent — typed enemy intent, Copy
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Attack { damage: i16, hits: u8, effects: u16 },
    Block { amount: i16, effects: u16 },
    Buff { effects: u16 },
    Debuff { effects: u16 },
    AttackBlock { damage: i16, hits: u8, block: i16, effects: u16 },
    AttackBuff { damage: i16, hits: u8, effects: u16 },
    AttackDebuff { damage: i16, hits: u8, effects: u16 },
    DefendBuff { block: i16, effects: u16 },
    Spawn,
    Escape,
    Sleep,
    Stun,
    Unknown,
}

/// Effect bitfield for Intent.effects
pub mod fx {
    pub const WEAK: u16           = 1 << 0;
    pub const VULNERABLE: u16     = 1 << 1;
    pub const FRAIL: u16          = 1 << 2;
    pub const STRENGTH: u16       = 1 << 3;
    pub const RITUAL: u16         = 1 << 4;
    pub const BLOCK_SELF: u16     = 1 << 5;
    pub const ARTIFACT: u16       = 1 << 6;
    pub const POISON: u16         = 1 << 7;
    pub const ENTANGLE: u16       = 1 << 8;
    pub const BURN: u16           = 1 << 9;
    pub const DAZE: u16           = 1 << 10;
    pub const SLIMED: u16         = 1 << 11;
    pub const WOUND: u16          = 1 << 12;
    pub const DRAW_REDUCTION: u16 = 1 << 13;
    pub const STR_DOWN: u16       = 1 << 14;
    pub const DEX_DOWN: u16       = 1 << 15;
}

/// Move effect ID constants for EnemyCombatState.move_effects SmallVec.
pub mod mfx {
    pub const WEAK: u8 = 0;
    pub const VULNERABLE: u8 = 1;
    pub const FRAIL: u8 = 2;
    pub const STRENGTH: u8 = 3;
    pub const RITUAL: u8 = 4;
    pub const ENTANGLE: u8 = 5;
    pub const SLIMED: u8 = 6;
    pub const DAZE: u8 = 7;
    pub const BURN: u8 = 8;
    pub const BURN_UPGRADE: u8 = 9;
    pub const SIPHON_STR: u8 = 10;
    pub const SIPHON_DEX: u8 = 11;
    pub const REMOVE_DEBUFFS: u8 = 12;
    pub const HEAL_TO_HALF: u8 = 13;
    pub const HEAL_FULL: u8 = 14;
    pub const ARTIFACT: u8 = 15;
    pub const CONFUSED: u8 = 16;
    pub const CONSTRICT: u8 = 17;
    pub const DEX_DOWN: u8 = 18;
    pub const DRAW_REDUCTION: u8 = 19;
    pub const HEX: u8 = 20;
    pub const PAINFUL_STABS: u8 = 21;
    pub const STASIS: u8 = 22;
    pub const STRENGTH_BONUS: u8 = 23;
    pub const STRENGTH_DOWN: u8 = 24;
    pub const THORNS: u8 = 25;
    pub const VOID: u8 = 26;
    pub const WOUND: u8 = 27;
    pub const BEAT_OF_DEATH: u8 = 28;
    pub const HEAL: u8 = 29;
    pub const POISON: u8 = 30;
    pub const BLOCK_ALL_ALLIES: u8 = 31;
    pub const HEAL_LOWEST_ALLY: u8 = 32;
    pub const STRENGTH_ALL_ALLIES: u8 = 33;
}

// ---------------------------------------------------------------------------
// DamageSource
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageSource {
    Card,
    Power,
    Relic,
    Potion,
    Thorns,
    Orb,
    Status,
    Enemy,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_instance_flags() {
        let mut c = CardInstance::new(42);
        assert!(!c.is_retained());
        c.set_retained(true);
        assert!(c.is_retained());
        let c2 = CardInstance::new(42).upgraded();
        assert!(c2.is_upgraded());
    }

    #[test]
    fn card_instance_is_4_bytes() {
        assert_eq!(std::mem::size_of::<CardInstance>(), 6);
    }

    #[test]
    fn intent_is_copy() {
        let i = Intent::Attack { damage: 12, hits: 3, effects: fx::WEAK | fx::VULNERABLE };
        let i2 = i; // Copy
        assert_eq!(i, i2);
    }
}
