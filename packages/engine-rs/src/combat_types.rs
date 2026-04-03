//! Core combat types for engine v2.
//! Entity-based model: player and enemies share the same Entity struct.
//! All types are Copy-friendly for MCTS cloning.

use smallvec::SmallVec;
use serde::{Serialize, Deserialize};

// ---------------------------------------------------------------------------
// Entity — universal character (player AND enemy)
// ---------------------------------------------------------------------------

/// Universal character. entities[0] = player, entities[1..] = enemies.
#[derive(Clone, Debug)]
pub struct Entity {
    pub hp: i16,
    pub max_hp: i16,
    pub block: i16,
    /// Fixed array indexed by StatusId. Zero = absent.
    pub statuses: [i16; 64],
}

impl Entity {
    pub fn new(hp: i16, max_hp: i16) -> Self {
        Self { hp, max_hp, block: 0, statuses: [0; 64] }
    }

    pub fn status(&self, id: u8) -> i16 { self.statuses[id as usize] }
    pub fn set_status(&mut self, id: u8, val: i16) { self.statuses[id as usize] = val; }
    pub fn add_status(&mut self, id: u8, amt: i16) { self.statuses[id as usize] += amt; }
    pub fn is_alive(&self) -> bool { self.hp > 0 }
    pub fn is_dead(&self) -> bool { self.hp <= 0 }

    // Convenience (StatusId numbers match status_ids.rs)
    pub fn strength(&self) -> i16 { self.statuses[0] }
    pub fn dexterity(&self) -> i16 { self.statuses[1] }
    pub fn focus(&self) -> i16 { self.statuses[2] }
    pub fn is_weak(&self) -> bool { self.statuses[6] > 0 }
    pub fn is_vulnerable(&self) -> bool { self.statuses[5] > 0 }
    pub fn is_frail(&self) -> bool { self.statuses[7] > 0 }
}

// ---------------------------------------------------------------------------
// CardInstance — 4 bytes per card, Copy
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstance {
    /// Index into static CardDef table.
    pub def_id: u16,
    /// Cost for this turn. -1 = use base cost from CardDef.
    pub cost: i8,
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
        Self { def_id, cost: -1, flags: 0 }
    }
    pub fn with_cost(mut self, cost: i8) -> Self { self.cost = cost; self }
    pub fn upgraded(mut self) -> Self { self.flags |= Self::FLAG_UPGRADED; self }

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
// Stance
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StanceV2 {
    Neutral = 0,
    Wrath = 1,
    Calm = 2,
    Divinity = 3,
}

// ---------------------------------------------------------------------------
// EnemyMeta — AI state separate from Entity stats
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyMeta {
    pub enemy_id: u16,
    pub intent: Intent,
    pub move_history: SmallVec<[u8; 8]>,
    pub first_turn: bool,
    pub escaping: bool,
}

// ---------------------------------------------------------------------------
// Combat — full snapshot cloned for MCTS
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Combat {
    // Entities: [0] = player, [1..] = enemies
    pub entities: SmallVec<[Entity; 6]>,
    pub enemy_meta: SmallVec<[EnemyMeta; 5]>,

    // Card piles
    pub hand: SmallVec<[CardInstance; 10]>,
    pub draw_pile: Vec<CardInstance>,
    pub discard_pile: Vec<CardInstance>,
    pub exhaust_pile: Vec<CardInstance>,

    // Resources
    pub energy: i8,
    pub max_energy: i8,
    pub stance: StanceV2,
    pub mantra: i16,

    // Relics + potions (compact)
    pub relics: [u64; 3],   // 192-bit bitfield
    pub potions: [u8; 5],   // potion IDs, 0=empty

    // Orbs (Defect)
    pub orb_types: [u8; 5],
    pub orb_values: [i16; 5],
    pub orb_count: u8,
    pub orb_max: u8,

    // Turn tracking
    pub turn: u16,
    pub cards_played: u8,
    pub attacks_played: u8,

    // Flags
    pub combat_over: bool,
    pub player_won: bool,
    pub skip_enemy_turn: bool,
    pub blasphemy: bool,
}

// ---------------------------------------------------------------------------
// CombatLine — MCTS solver output
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct CombatLine {
    pub actions: SmallVec<[u8; 20]>,
    pub expected_hp_remaining: i16,
    pub expected_hp_loss: i16,
    pub potions_used: u8,
    pub enemies_killed: u8,
    pub turns: u8,
    pub confidence: f32,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_basics() {
        let mut e = Entity::new(72, 72);
        assert_eq!(e.hp, 72);
        assert!(e.is_alive());
        e.set_status(0, 3);
        assert_eq!(e.strength(), 3);
        e.add_status(0, 2);
        assert_eq!(e.strength(), 5);
    }

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
        assert_eq!(std::mem::size_of::<CardInstance>(), 4);
    }

    #[test]
    fn entity_status_array_size() {
        assert_eq!(std::mem::size_of::<[i16; 64]>(), 128);
    }

    #[test]
    fn intent_is_copy() {
        let i = Intent::Attack { damage: 12, hits: 3, effects: fx::WEAK | fx::VULNERABLE };
        let i2 = i; // Copy
        assert_eq!(i, i2);
    }

    #[test]
    fn combat_clone() {
        let mut c = Combat {
            entities: SmallVec::new(),
            enemy_meta: SmallVec::new(),
            hand: SmallVec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            energy: 3,
            max_energy: 3,
            stance: StanceV2::Neutral,
            mantra: 0,
            relics: [0; 3],
            potions: [0; 5],
            orb_types: [0; 5],
            orb_values: [0; 5],
            orb_count: 0,
            orb_max: 0,
            turn: 1,
            cards_played: 0,
            attacks_played: 0,
            combat_over: false,
            player_won: false,
            skip_enemy_turn: false,
            blasphemy: false,
        };
        c.entities.push(Entity::new(72, 72));
        let c2 = c.clone();
        assert_eq!(c2.entities[0].hp, 72);
        assert_eq!(c2.energy, 3);
    }
}
