//! Defect orb system — channel, evoke, passive triggers.
//!
//! Orbs occupy numbered slots. When a new orb is channeled and all slots
//! are full, the frontmost orb is evoked first. Passive effects fire at
//! end of turn for each orb in order (except Plasma which fires at start
//! of turn, matching Java).
//!
//! **Focus model**: Focus is applied dynamically (not baked into the orb).
//! Each orb stores its *base* passive/evoke amounts. When computing effects,
//! the caller passes the current focus value which is added to base amounts
//! (clamped to 0). Plasma is unaffected by focus.
//!
//! **Dark orb**: `evokeAmount` starts at `baseEvokeAmount` (6) and
//! accumulates `passiveAmount + focus` each end-of-turn. Focus only
//! modifies the passive gain rate, not the stored evoke total.

use serde::{Deserialize, Serialize};

// ===========================================================================
// OrbType
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrbType {
    Lightning,
    Frost,
    Dark,
    Plasma,
    Empty,
}

impl OrbType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrbType::Lightning => "Lightning",
            OrbType::Frost => "Frost",
            OrbType::Dark => "Dark",
            OrbType::Plasma => "Plasma",
            OrbType::Empty => "Empty",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Lightning" => OrbType::Lightning,
            "Frost" => OrbType::Frost,
            "Dark" => OrbType::Dark,
            "Plasma" => OrbType::Plasma,
            _ => OrbType::Empty,
        }
    }
}

// ===========================================================================
// Orb
// ===========================================================================

/// A single orb instance in a slot.
///
/// Stores *base* amounts. Focus is applied dynamically by the caller.
/// Exception: Dark's `evoke_amount` accumulates each turn and is NOT
/// a base value — it grows by `(base_passive + focus)` per turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orb {
    pub orb_type: OrbType,
    /// Base passive amount (before focus).
    pub base_passive: i32,
    /// Base evoke amount (before focus). For Dark, this accumulates.
    pub base_evoke: i32,
    /// For Dark: the accumulated evoke damage (grows each turn).
    /// Starts equal to base_evoke (6). On evoke, this is the damage dealt.
    pub evoke_amount: i32,
}

impl Orb {
    pub fn new(orb_type: OrbType) -> Self {
        match orb_type {
            OrbType::Lightning => Self {
                orb_type,
                base_passive: 3,
                base_evoke: 8,
                evoke_amount: 8,
            },
            OrbType::Frost => Self {
                orb_type,
                base_passive: 2,
                base_evoke: 5,
                evoke_amount: 5,
            },
            OrbType::Dark => Self {
                orb_type,
                base_passive: 6,
                base_evoke: 6,
                evoke_amount: 6, // accumulates each turn
            },
            OrbType::Plasma => Self {
                orb_type,
                base_passive: 1,
                base_evoke: 2,
                evoke_amount: 2,
            },
            OrbType::Empty => Self {
                orb_type,
                base_passive: 0,
                base_evoke: 0,
                evoke_amount: 0,
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orb_type == OrbType::Empty
    }

    /// Compute the effective passive amount with focus applied.
    /// Plasma is unaffected by focus.
    pub fn passive_with_focus(&self, focus: i32) -> i32 {
        match self.orb_type {
            OrbType::Plasma => self.base_passive, // unaffected by focus
            OrbType::Empty => 0,
            _ => (self.base_passive + focus).max(0),
        }
    }

    /// Compute the effective evoke amount with focus applied.
    /// Dark uses its accumulated `evoke_amount` directly (focus already
    /// affected the accumulation rate, not the stored total).
    /// Plasma is unaffected by focus.
    pub fn evoke_with_focus(&self, focus: i32) -> i32 {
        match self.orb_type {
            OrbType::Dark => self.evoke_amount, // accumulated, no extra focus
            OrbType::Plasma => self.evoke_amount, // unaffected by focus
            OrbType::Empty => 0,
            _ => (self.base_evoke + focus).max(0),
        }
    }
}

// ===========================================================================
// OrbSlots — the orb slot manager
// ===========================================================================

/// Manages the player's orb slots (Defect mechanic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbSlots {
    pub slots: Vec<Orb>,
    pub max_slots: usize,
}

/// Result of evoking an orb, describing what effect to apply.
#[derive(Debug, Clone)]
pub enum EvokeEffect {
    /// Deal damage to a random enemy.
    LightningDamage(i32),
    /// Gain block.
    FrostBlock(i32),
    /// Deal damage to enemy with lowest HP.
    DarkDamage(i32),
    /// Gain energy.
    PlasmaEnergy(i32),
    /// No effect (empty slot).
    None,
}

/// Result of a passive trigger.
#[derive(Debug, Clone)]
pub enum PassiveEffect {
    /// Deal damage to a random enemy (Lightning).
    LightningDamage(i32),
    /// Gain block (Frost).
    FrostBlock(i32),
    /// Gain energy (Plasma — fires at start of turn).
    PlasmaEnergy(i32),
    /// No immediate effect (Dark accumulates internally).
    None,
}

impl OrbSlots {
    /// Create with a given number of empty slots.
    pub fn new(num_slots: usize) -> Self {
        let slots = vec![Orb::new(OrbType::Empty); num_slots];
        Self {
            slots,
            max_slots: num_slots,
        }
    }

    /// Number of currently occupied (non-empty) orb slots.
    pub fn occupied_count(&self) -> usize {
        self.slots.iter().filter(|o| !o.is_empty()).count()
    }

    /// Total slot count.
    pub fn get_slot_count(&self) -> usize {
        self.max_slots
    }

    /// Check if there are any orbs at all.
    pub fn has_orbs(&self) -> bool {
        self.max_slots > 0
    }

    /// Add a new orb slot (e.g. from Capacitor).
    pub fn add_slot(&mut self) {
        self.max_slots += 1;
        self.slots.push(Orb::new(OrbType::Empty));
    }

    /// Remove a slot. If all slots are occupied, evokes the last orb.
    /// Returns any evoke effect from the removed orb.
    pub fn remove_slot(&mut self, focus: i32) -> EvokeEffect {
        if self.max_slots == 0 {
            return EvokeEffect::None;
        }
        self.max_slots -= 1;

        // If we have more orbs than slots, evoke the last one
        if self.slots.len() > self.max_slots {
            let orb = self.slots.pop().unwrap_or(Orb::new(OrbType::Empty));
            return Self::compute_evoke_effect(&orb, focus);
        }
        EvokeEffect::None
    }

    /// Channel an orb into the first empty slot.
    /// If slots are full, evoke the frontmost orb first, shift left, place new at back.
    /// Returns any evoke effect from displacement.
    pub fn channel(&mut self, orb_type: OrbType, focus: i32) -> EvokeEffect {
        let mut evoke = EvokeEffect::None;

        // Find first empty slot
        if let Some(idx) = self.slots.iter().position(|o| o.is_empty()) {
            self.slots[idx] = Orb::new(orb_type);
        } else if !self.slots.is_empty() {
            // All slots full — evoke front, shift left, place new at back
            evoke = self.evoke_front(focus);
            let orb = Orb::new(orb_type);
            // After evoke_front removed front and added empty at back,
            // replace that trailing empty with the new orb.
            if let Some(last) = self.slots.last_mut() {
                *last = orb;
            }
        }
        // If no slots at all, orb is lost (shouldn't happen in normal gameplay)

        evoke
    }

    /// Evoke the frontmost orb and remove it. Shifts remaining orbs left.
    /// Returns the evoke effect to be applied by the caller.
    pub fn evoke_front(&mut self, focus: i32) -> EvokeEffect {
        if self.slots.is_empty() {
            return EvokeEffect::None;
        }

        let orb = self.slots.remove(0);
        let effect = Self::compute_evoke_effect(&orb, focus);

        // Add empty slot at end to maintain slot count
        self.slots.push(Orb::new(OrbType::Empty));

        effect
    }

    /// Evoke all orbs (e.g. from Multicast). Returns a list of effects.
    pub fn evoke_all(&mut self, focus: i32) -> Vec<EvokeEffect> {
        let mut effects = Vec::new();
        let orbs: Vec<Orb> = self.slots.drain(..).collect();
        for orb in &orbs {
            if !orb.is_empty() {
                effects.push(Self::compute_evoke_effect(orb, focus));
            }
        }
        // Refill with empty slots
        self.slots = vec![Orb::new(OrbType::Empty); self.max_slots];
        effects
    }

    /// Evoke the front orb N times (e.g. Multicast channels N evokes).
    pub fn evoke_front_n(&mut self, n: usize, focus: i32) -> Vec<EvokeEffect> {
        let mut effects = Vec::new();
        for _ in 0..n {
            if self.occupied_count() == 0 {
                break;
            }
            effects.push(self.evoke_front(focus));
        }
        effects
    }

    /// Trigger end-of-turn passive effects for all orbs.
    /// Dark accumulates damage. Lightning/Frost produce effects.
    /// Plasma is NOT included here — it fires at start of turn.
    pub fn trigger_end_of_turn_passives(&mut self, focus: i32) -> Vec<PassiveEffect> {
        let mut effects = Vec::new();
        for orb in &mut self.slots {
            if orb.is_empty() {
                continue;
            }
            match orb.orb_type {
                OrbType::Lightning => {
                    let damage = orb.passive_with_focus(focus);
                    effects.push(PassiveEffect::LightningDamage(damage));
                }
                OrbType::Frost => {
                    let block = orb.passive_with_focus(focus);
                    effects.push(PassiveEffect::FrostBlock(block));
                }
                OrbType::Dark => {
                    // Dark accumulates: evokeAmount += passiveAmount (with focus)
                    let gain = orb.passive_with_focus(focus);
                    orb.evoke_amount += gain;
                    // No immediate effect
                    effects.push(PassiveEffect::None);
                }
                OrbType::Plasma => {
                    // Plasma passive fires at start of turn, not here
                }
                OrbType::Empty => {}
            }
        }
        effects
    }

    /// Trigger start-of-turn passive effects (Plasma only).
    pub fn trigger_start_of_turn_passives(&self) -> Vec<PassiveEffect> {
        let mut effects = Vec::new();
        for orb in &self.slots {
            if orb.orb_type == OrbType::Plasma {
                effects.push(PassiveEffect::PlasmaEnergy(orb.base_passive));
            }
        }
        effects
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn compute_evoke_effect(orb: &Orb, focus: i32) -> EvokeEffect {
        match orb.orb_type {
            OrbType::Lightning => EvokeEffect::LightningDamage(orb.evoke_with_focus(focus)),
            OrbType::Frost => EvokeEffect::FrostBlock(orb.evoke_with_focus(focus)),
            OrbType::Dark => EvokeEffect::DarkDamage(orb.evoke_amount), // accumulated value
            OrbType::Plasma => EvokeEffect::PlasmaEnergy(orb.evoke_amount),
            OrbType::Empty => EvokeEffect::None,
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Basic channel/evoke --

    #[test]
    fn new_orb_slots_are_empty() {
        let slots = OrbSlots::new(3);
        assert_eq!(slots.get_slot_count(), 3);
        assert_eq!(slots.occupied_count(), 0);
        assert!(slots.slots.iter().all(|o| o.is_empty()));
    }

    #[test]
    fn channel_fills_empty_slot() {
        let mut slots = OrbSlots::new(3);
        let effect = slots.channel(OrbType::Lightning, 0);
        assert!(matches!(effect, EvokeEffect::None));
        assert_eq!(slots.occupied_count(), 1);
        assert_eq!(slots.slots[0].orb_type, OrbType::Lightning);
    }

    #[test]
    fn channel_fills_first_empty_preserves_order() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        assert_eq!(slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(slots.slots[1].orb_type, OrbType::Frost);
        assert_eq!(slots.slots[2].orb_type, OrbType::Empty);
    }

    #[test]
    fn channel_when_full_evokes_front() {
        let mut slots = OrbSlots::new(2);
        slots.channel(OrbType::Frost, 0);
        slots.channel(OrbType::Lightning, 0);
        // Now full. Channel Dark -> should evoke Frost (front)
        let effect = slots.channel(OrbType::Dark, 0);
        assert!(matches!(effect, EvokeEffect::FrostBlock(5)));
        // Remaining: Lightning, Dark
        assert_eq!(slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(slots.slots[1].orb_type, OrbType::Dark);
    }

    #[test]
    fn channel_when_full_single_slot() {
        let mut slots = OrbSlots::new(1);
        slots.channel(OrbType::Lightning, 0);
        // Channel Frost -> evoke Lightning first
        let effect = slots.channel(OrbType::Frost, 0);
        assert!(matches!(effect, EvokeEffect::LightningDamage(8)));
        assert_eq!(slots.slots[0].orb_type, OrbType::Frost);
    }

    // -- Evoke --

    #[test]
    fn evoke_front_removes_and_shifts() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        let effect = slots.evoke_front(0);
        assert!(matches!(effect, EvokeEffect::LightningDamage(8)));
        assert_eq!(slots.slots[0].orb_type, OrbType::Frost);
        assert_eq!(slots.occupied_count(), 1);
        assert_eq!(slots.slots.len(), 3); // still 3 slots total
    }

    #[test]
    fn evoke_all_clears_slots() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        let effects = slots.evoke_all(0);
        assert_eq!(effects.len(), 2);
        assert_eq!(slots.occupied_count(), 0);
        assert_eq!(slots.slots.len(), 3);
    }

    #[test]
    fn evoke_front_n_multiple() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        slots.channel(OrbType::Dark, 0);
        let effects = slots.evoke_front_n(2, 0);
        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], EvokeEffect::LightningDamage(8)));
        assert!(matches!(effects[1], EvokeEffect::FrostBlock(5)));
        assert_eq!(slots.occupied_count(), 1);
        assert_eq!(slots.slots[0].orb_type, OrbType::Dark);
    }

    // -- Focus --

    #[test]
    fn focus_affects_lightning_passive_and_evoke() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        // Passive with focus=2: 3+2 = 5
        let effects = slots.trigger_end_of_turn_passives(2);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], PassiveEffect::LightningDamage(5)));
        // Evoke with focus=2: 8+2 = 10
        let evoke = slots.evoke_front(2);
        assert!(matches!(evoke, EvokeEffect::LightningDamage(10)));
    }

    #[test]
    fn focus_affects_frost_passive_and_evoke() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Frost, 0);
        let effects = slots.trigger_end_of_turn_passives(3);
        assert_eq!(effects.len(), 1);
        // passive: 2+3 = 5
        assert!(matches!(effects[0], PassiveEffect::FrostBlock(5)));
        // evoke: 5+3 = 8
        let evoke = slots.evoke_front(3);
        assert!(matches!(evoke, EvokeEffect::FrostBlock(8)));
    }

    #[test]
    fn negative_focus_clamps_to_zero() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        // Focus = -10 -> passive: max(0, 3-10) = 0
        let effects = slots.trigger_end_of_turn_passives(-10);
        assert!(matches!(effects[0], PassiveEffect::LightningDamage(0)));
        // Evoke: max(0, 8-10) = 0
        let evoke = slots.evoke_front(-10);
        assert!(matches!(evoke, EvokeEffect::LightningDamage(0)));
    }

    #[test]
    fn focus_does_not_affect_plasma() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Plasma, 0);
        // Start-of-turn passive: always 1, regardless of focus
        let effects = slots.trigger_start_of_turn_passives();
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], PassiveEffect::PlasmaEnergy(1)));
        // End-of-turn should produce nothing for Plasma
        let eot = slots.trigger_end_of_turn_passives(5);
        assert!(eot.is_empty());
        // Evoke: always 2
        let evoke = slots.evoke_front(5);
        assert!(matches!(evoke, EvokeEffect::PlasmaEnergy(2)));
    }

    // -- Dark accumulation --

    #[test]
    fn dark_orb_accumulates_evoke_amount() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Dark, 0);
        assert_eq!(slots.slots[0].evoke_amount, 6); // initial

        // End of turn 1: accumulate passive_amount (6+0 focus = 6)
        let effects = slots.trigger_end_of_turn_passives(0);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], PassiveEffect::None)); // Dark has no immediate passive
        assert_eq!(slots.slots[0].evoke_amount, 12); // 6 + 6

        // End of turn 2: accumulate again
        slots.trigger_end_of_turn_passives(0);
        assert_eq!(slots.slots[0].evoke_amount, 18); // 12 + 6
    }

    #[test]
    fn dark_orb_focus_affects_accumulation_rate() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Dark, 0);
        assert_eq!(slots.slots[0].evoke_amount, 6);

        // With focus=3: passive = max(0, 6+3) = 9
        slots.trigger_end_of_turn_passives(3);
        assert_eq!(slots.slots[0].evoke_amount, 15); // 6 + 9
    }

    #[test]
    fn dark_orb_evoke_uses_accumulated_value() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Dark, 0);
        // Accumulate for 2 turns
        slots.trigger_end_of_turn_passives(0); // 6 + 6 = 12
        slots.trigger_end_of_turn_passives(0); // 12 + 6 = 18
        // Evoke: uses accumulated value, NOT affected by focus
        let evoke = slots.evoke_front(5); // focus doesn't matter for Dark evoke
        assert!(matches!(evoke, EvokeEffect::DarkDamage(18)));
    }

    // -- Plasma timing --

    #[test]
    fn plasma_passive_fires_at_start_of_turn() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Plasma, 0);
        // Start-of-turn triggers Plasma
        let sot = slots.trigger_start_of_turn_passives();
        assert_eq!(sot.len(), 1);
        assert!(matches!(sot[0], PassiveEffect::PlasmaEnergy(1)));
        // End-of-turn does NOT trigger Plasma
        let eot = slots.trigger_end_of_turn_passives(0);
        assert!(eot.is_empty());
    }

    // -- Slot management --

    #[test]
    fn add_and_remove_slot() {
        let mut slots = OrbSlots::new(2);
        assert_eq!(slots.get_slot_count(), 2);
        slots.add_slot();
        assert_eq!(slots.get_slot_count(), 3);
        assert_eq!(slots.slots.len(), 3);
        let effect = slots.remove_slot(0);
        assert!(matches!(effect, EvokeEffect::None));
        assert_eq!(slots.get_slot_count(), 2);
    }

    #[test]
    fn remove_slot_evokes_if_full() {
        let mut slots = OrbSlots::new(2);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        // Both slots occupied, remove one -> evokes last
        let effect = slots.remove_slot(0);
        assert!(matches!(effect, EvokeEffect::FrostBlock(5)));
        assert_eq!(slots.get_slot_count(), 1);
        assert_eq!(slots.occupied_count(), 1);
    }

    // -- Mixed orbs --

    #[test]
    fn mixed_orbs_end_of_turn() {
        let mut slots = OrbSlots::new(4);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        slots.channel(OrbType::Dark, 0);
        slots.channel(OrbType::Plasma, 0);

        let effects = slots.trigger_end_of_turn_passives(0);
        // Lightning -> damage, Frost -> block, Dark -> accumulate (None), Plasma -> skipped
        assert_eq!(effects.len(), 3);
        assert!(matches!(effects[0], PassiveEffect::LightningDamage(3)));
        assert!(matches!(effects[1], PassiveEffect::FrostBlock(2)));
        assert!(matches!(effects[2], PassiveEffect::None)); // Dark

        let sot_effects = slots.trigger_start_of_turn_passives();
        assert_eq!(sot_effects.len(), 1);
        assert!(matches!(sot_effects[0], PassiveEffect::PlasmaEnergy(1)));
    }

    // -- OrbType roundtrip --

    #[test]
    fn orb_type_roundtrip() {
        for otype in &[OrbType::Lightning, OrbType::Frost, OrbType::Dark, OrbType::Plasma, OrbType::Empty] {
            assert_eq!(*otype, OrbType::from_str(otype.as_str()));
        }
    }

    // -- Edge cases --

    #[test]
    fn zero_slots_channel_does_nothing() {
        let mut slots = OrbSlots::new(0);
        let effect = slots.channel(OrbType::Lightning, 0);
        assert!(matches!(effect, EvokeEffect::None));
        assert_eq!(slots.occupied_count(), 0);
    }

    #[test]
    fn evoke_empty_returns_none() {
        let mut slots = OrbSlots::new(3);
        let effect = slots.evoke_front(0);
        assert!(matches!(effect, EvokeEffect::None));
    }

    #[test]
    fn has_orbs_reflects_max_slots() {
        let slots = OrbSlots::new(0);
        assert!(!slots.has_orbs());
        let slots = OrbSlots::new(3);
        assert!(slots.has_orbs());
    }
}
