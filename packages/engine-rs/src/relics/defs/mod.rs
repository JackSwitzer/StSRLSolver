//! Per-relic EntityDef definitions.
//!
//! Each relic's entire declarative behavior lives in one file.
//! The RELIC_DEFS registry collects all definitions for dispatch.
//!
//! Phase 1: definitions only -- old dispatch in combat.rs/run.rs
//! stays alongside these defs until the trigger interpreter is wired.

use crate::effects::entity_def::EntityDef;

// ===========================================================================
// Combat-start stat buffs (pure declarative)
// ===========================================================================
pub mod vajra;
pub mod oddly_smooth_stone;
pub mod data_disk;
pub mod akabeko;
pub mod anchor;
pub mod bag_of_marbles;
pub mod red_mask;
pub mod thread_and_needle;
pub mod bronze_scales;
pub mod clockwork_souvenir;
pub mod fossilized_helix;
pub mod blood_vial;
pub mod twisted_funnel;
pub mod mutagenic_strength;
pub mod philosophers_stone;

// ===========================================================================
// Counter-based relics (OnAttackPlayed/OnSkillPlayed/OnAnyCardPlayed/etc.)
// ===========================================================================
pub mod ornamental_fan;
pub mod kunai;
pub mod shuriken;
pub mod nunchaku;
pub mod letter_opener;
pub mod ink_bottle;
pub mod happy_flower;
pub mod incense_burner;
pub mod sundial;

// ===========================================================================
// Turn-based relics
// ===========================================================================
pub mod mercury_hourglass;
pub mod orichalcum;
pub mod lantern;
pub mod brimstone;
pub mod cloak_clasp;
pub mod damaru;

// ===========================================================================
// Event-triggered relics (card play, exhaust, discard, death, victory, etc.)
// ===========================================================================
pub mod bird_faced_urn;
pub mod charons_ashes;
pub mod tough_bandages;
pub mod tingsha;
pub mod gremlin_horn;
pub mod burning_blood;
pub mod black_blood;
pub mod toy_ornithopter;
pub mod self_forming_clay;
pub mod the_abacus;

// ===========================================================================
// Complex relics (use complex_hook or stub-only)
// ===========================================================================
pub mod pen_nib;

// ===========================================================================
// Registry — static array of all relic EntityDefs
// ===========================================================================

/// All relic definitions. The dispatch loop iterates this to find matching
/// triggers. Order does not matter -- all matching triggers fire.
pub static RELIC_DEFS: &[&EntityDef] = &[
    // Combat-start stat buffs
    &vajra::DEF,
    &oddly_smooth_stone::DEF,
    &data_disk::DEF,
    &akabeko::DEF,
    &anchor::DEF,
    &bag_of_marbles::DEF,
    &red_mask::DEF,
    &thread_and_needle::DEF,
    &bronze_scales::DEF,
    &clockwork_souvenir::DEF,
    &fossilized_helix::DEF,
    &blood_vial::DEF,
    &twisted_funnel::DEF,
    &mutagenic_strength::DEF,
    &philosophers_stone::DEF,
    // Counter-based relics
    &ornamental_fan::DEF,
    &kunai::DEF,
    &shuriken::DEF,
    &nunchaku::DEF,
    &letter_opener::DEF,
    &ink_bottle::DEF,
    &happy_flower::DEF,
    &incense_burner::DEF,
    &sundial::DEF,
    // Turn-based relics
    &mercury_hourglass::DEF,
    &orichalcum::DEF,
    &lantern::DEF,
    &brimstone::DEF,
    &cloak_clasp::DEF,
    &damaru::DEF,
    // Event-triggered relics
    &bird_faced_urn::DEF,
    &charons_ashes::DEF,
    &tough_bandages::DEF,
    &tingsha::DEF,
    &gremlin_horn::DEF,
    &burning_blood::DEF,
    &black_blood::DEF,
    &toy_ornithopter::DEF,
    &self_forming_clay::DEF,
    &the_abacus::DEF,
    // Complex relics
    &pen_nib::DEF,
];

// ===========================================================================
// Lookup helper
// ===========================================================================

/// Find a relic definition by ID string.
/// Returns None if the relic has no EntityDef yet (still in old dispatch).
pub fn find_relic_def(id: &str) -> Option<&'static EntityDef> {
    RELIC_DEFS.iter().find(|def| def.id == id).copied()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::entity_def::EntityKind;

    #[test]
    fn test_relic_defs_count() {
        assert!(RELIC_DEFS.len() >= 30, "Expected at least 30 relic defs, got {}", RELIC_DEFS.len());
    }

    #[test]
    fn test_all_defs_are_relics() {
        for def in RELIC_DEFS.iter() {
            assert_eq!(def.kind, EntityKind::Relic, "Expected EntityKind::Relic for {}", def.id);
        }
    }

    #[test]
    fn test_no_duplicate_ids() {
        let mut ids: Vec<&str> = RELIC_DEFS.iter().map(|d| d.id).collect();
        ids.sort();
        for window in ids.windows(2) {
            assert_ne!(window[0], window[1], "Duplicate relic ID: {}", window[0]);
        }
    }

    #[test]
    fn test_find_relic_def() {
        assert!(find_relic_def("Vajra").is_some());
        assert!(find_relic_def("Nonexistent").is_none());
    }

    #[test]
    fn test_vajra_has_combat_start_trigger() {
        let def = find_relic_def("Vajra").unwrap();
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(def.triggers[0].trigger, crate::effects::trigger::Trigger::CombatStart);
    }

    #[test]
    fn test_ornamental_fan_has_counter() {
        let def = find_relic_def("Ornamental Fan").unwrap();
        assert_eq!(def.triggers.len(), 1);
        let te = &def.triggers[0];
        assert_eq!(te.trigger, crate::effects::trigger::Trigger::OnAttackPlayed);
        assert!(te.counter.is_some());
        let (counter_id, threshold) = te.counter.unwrap();
        assert_eq!(counter_id, crate::status_ids::sid::ORNAMENTAL_FAN_COUNTER);
        assert_eq!(threshold, 3);
    }

    #[test]
    fn test_orichalcum_has_no_block_condition() {
        let def = find_relic_def("Orichalcum").unwrap();
        assert_eq!(def.triggers[0].condition, crate::effects::trigger::TriggerCondition::NoBlock);
    }

    #[test]
    fn test_lantern_has_first_turn_condition() {
        let def = find_relic_def("Lantern").unwrap();
        assert_eq!(def.triggers[0].condition, crate::effects::trigger::TriggerCondition::FirstTurn);
    }

    #[test]
    fn test_simple_defs_have_no_complex_hook() {
        let simple_relics = ["Vajra", "Anchor", "Orichalcum", "Mercury Hourglass", "Kunai"];
        for name in simple_relics {
            let def = find_relic_def(name).unwrap();
            assert!(def.complex_hook.is_none(), "{} should not have a complex_hook", name);
        }
    }

    #[test]
    fn test_brimstone_has_two_effects() {
        let def = find_relic_def("Brimstone").unwrap();
        assert_eq!(def.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_mutagenic_strength_has_two_effects() {
        let def = find_relic_def("MutagenicStrength").unwrap();
        assert_eq!(def.triggers[0].effects.len(), 2);
    }
}
