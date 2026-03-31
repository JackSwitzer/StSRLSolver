//! Defect orb system — channel, evoke, passive triggers.
//!
//! Orbs occupy numbered slots. When a new orb is channeled and all slots
//! are full, the frontmost orb is evoked first. Passive effects fire at
//! end of turn for each orb in order.

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orb {
    pub orb_type: OrbType,
    /// Passive amount (base value before focus).
    pub passive_amount: i32,
    /// Evoke amount (base value before focus for some orbs).
    pub evoke_amount: i32,
    /// Stored value — used by Dark orbs to accumulate damage.
    pub n: i32,
}

impl Orb {
    pub fn new(orb_type: OrbType) -> Self {
        match orb_type {
            OrbType::Lightning => Self {
                orb_type,
                passive_amount: 3,
                evoke_amount: 8,
                n: 0,
            },
            OrbType::Frost => Self {
                orb_type,
                passive_amount: 2,
                evoke_amount: 5,
                n: 0,
            },
            OrbType::Dark => Self {
                orb_type,
                passive_amount: 6,
                evoke_amount: 0, // evoke uses n
                n: 6,
            },
            OrbType::Plasma => Self {
                orb_type,
                passive_amount: 1,
                evoke_amount: 2,
                n: 0,
            },
            OrbType::Empty => Self {
                orb_type,
                passive_amount: 0,
                evoke_amount: 0,
                n: 0,
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orb_type == OrbType::Empty
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

    /// Add a new orb slot (e.g. from Capacitor).
    pub fn add_slot(&mut self) {
        self.max_slots += 1;
        self.slots.push(Orb::new(OrbType::Empty));
    }

    /// Remove a slot. If all slots are occupied, evokes the front orb first.
    /// Returns any evoke effect from the removed orb.
    pub fn remove_slot(&mut self) -> EvokeEffect {
        if self.max_slots == 0 {
            return EvokeEffect::None;
        }
        self.max_slots -= 1;

        // If we have more orbs than slots, evoke the last one
        if self.slots.len() > self.max_slots {
            let orb = self.slots.pop().unwrap_or(Orb::new(OrbType::Empty));
            return Self::evoke_effect(&orb);
        }
        EvokeEffect::None
    }

    /// Channel an orb. If slots are full, evoke the frontmost orb first.
    /// `focus` is the player's current focus value, applied to the new orb.
    /// Returns any evoke effect from displacement.
    pub fn channel(&mut self, orb_type: OrbType, focus: i32) -> EvokeEffect {
        let mut evoke = EvokeEffect::None;

        // Find first empty slot
        if let Some(idx) = self.slots.iter().position(|o| o.is_empty()) {
            let mut orb = Orb::new(orb_type);
            Self::apply_focus(&mut orb, focus);
            self.slots[idx] = orb;
        } else if !self.slots.is_empty() {
            // All slots full — evoke front, shift left, place new at back
            evoke = self.evoke_front();
            let mut orb = Orb::new(orb_type);
            Self::apply_focus(&mut orb, focus);
            // After evoke_front, the front slot was removed and shifted.
            // We need to push the new orb at the end.
            if self.slots.len() < self.max_slots {
                self.slots.push(orb);
            } else if let Some(last) = self.slots.last_mut() {
                *last = orb;
            }
        }
        // If no slots at all, orb is lost (shouldn't happen in normal gameplay)

        evoke
    }

    /// Evoke the frontmost orb and remove it. Shifts remaining orbs left.
    /// Returns the evoke effect to be applied by the caller.
    pub fn evoke_front(&mut self) -> EvokeEffect {
        if self.slots.is_empty() {
            return EvokeEffect::None;
        }

        let orb = self.slots.remove(0);
        let effect = Self::evoke_effect(&orb);

        // Add empty slot at end to maintain slot count
        self.slots.push(Orb::new(OrbType::Empty));

        effect
    }

    /// Evoke all orbs. Returns a list of effects.
    pub fn evoke_all(&mut self) -> Vec<EvokeEffect> {
        let mut effects = Vec::new();
        let orbs: Vec<Orb> = self.slots.drain(..).collect();
        for orb in &orbs {
            if !orb.is_empty() {
                effects.push(Self::evoke_effect(orb));
            }
        }
        // Refill with empty slots
        self.slots = vec![Orb::new(OrbType::Empty); self.max_slots];
        effects
    }

    /// Trigger passive effects for all orbs at end of turn.
    /// `focus` is the player's current focus value.
    /// Returns a list of effects to apply.
    pub fn trigger_passives(&mut self, focus: i32) -> Vec<EvokeEffect> {
        let mut effects = Vec::new();
        for orb in &mut self.slots {
            if orb.is_empty() {
                continue;
            }
            match orb.orb_type {
                OrbType::Lightning => {
                    let damage = (orb.passive_amount + focus).max(0);
                    effects.push(EvokeEffect::LightningDamage(damage));
                }
                OrbType::Frost => {
                    let block = (orb.passive_amount + focus).max(0);
                    effects.push(EvokeEffect::FrostBlock(block));
                }
                OrbType::Dark => {
                    // Dark passively accumulates damage
                    let gain = (orb.passive_amount + focus).max(0);
                    orb.n += gain;
                    // No immediate effect
                }
                OrbType::Plasma => {
                    effects.push(EvokeEffect::PlasmaEnergy(1));
                }
                OrbType::Empty => {}
            }
        }
        effects
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn apply_focus(orb: &mut Orb, focus: i32) {
        match orb.orb_type {
            OrbType::Lightning => {
                orb.passive_amount = (orb.passive_amount + focus).max(0);
                orb.evoke_amount = (orb.evoke_amount + focus).max(0);
            }
            OrbType::Frost => {
                orb.passive_amount = (orb.passive_amount + focus).max(0);
                orb.evoke_amount = (orb.evoke_amount + focus).max(0);
            }
            OrbType::Dark => {
                orb.passive_amount = (orb.passive_amount + focus).max(0);
                orb.n = (orb.n + focus).max(0); // initial n includes focus
            }
            OrbType::Plasma => {
                // Plasma is not affected by focus
            }
            OrbType::Empty => {}
        }
    }

    fn evoke_effect(orb: &Orb) -> EvokeEffect {
        match orb.orb_type {
            OrbType::Lightning => EvokeEffect::LightningDamage(orb.evoke_amount),
            OrbType::Frost => EvokeEffect::FrostBlock(orb.evoke_amount),
            OrbType::Dark => EvokeEffect::DarkDamage(orb.n),
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
    fn evoke_all_clears_slots() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        let effects = slots.evoke_all();
        assert_eq!(effects.len(), 2);
        assert_eq!(slots.occupied_count(), 0);
    }

    #[test]
    fn dark_orb_accumulates() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Dark, 0);
        assert_eq!(slots.slots[0].n, 6); // initial

        // Trigger passives (adds passive_amount=6 each turn)
        let effects = slots.trigger_passives(0);
        assert!(effects.is_empty()); // Dark has no immediate passive effect
        assert_eq!(slots.slots[0].n, 12); // 6 + 6
    }

    #[test]
    fn focus_affects_orb_values() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 2); // focus=2
        assert_eq!(slots.slots[0].passive_amount, 5); // 3 + 2
        assert_eq!(slots.slots[0].evoke_amount, 10); // 8 + 2
    }

    #[test]
    fn add_and_remove_slot() {
        let mut slots = OrbSlots::new(2);
        assert_eq!(slots.get_slot_count(), 2);
        slots.add_slot();
        assert_eq!(slots.get_slot_count(), 3);
        assert_eq!(slots.slots.len(), 3);
        slots.remove_slot();
        assert_eq!(slots.get_slot_count(), 2);
    }

    #[test]
    fn frost_passive_gives_block() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Frost, 0);
        let effects = slots.trigger_passives(0);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], EvokeEffect::FrostBlock(2)));
    }

    #[test]
    fn plasma_passive_gives_energy() {
        let mut slots = OrbSlots::new(3);
        slots.channel(OrbType::Plasma, 0);
        let effects = slots.trigger_passives(0);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], EvokeEffect::PlasmaEnergy(1)));
    }

    #[test]
    fn orb_type_roundtrip() {
        for otype in &[OrbType::Lightning, OrbType::Frost, OrbType::Dark, OrbType::Plasma, OrbType::Empty] {
            assert_eq!(*otype, OrbType::from_str(otype.as_str()));
        }
    }
}
