//! Per-relic EntityDef definitions.
//!
//! Each relic's entire declarative behavior lives in one file.
//! The RELIC_DEFS registry collects all definitions for dispatch.
//! These defs are the canonical relic runtime metadata surface.

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
// Combat-start: stat buffs requiring complex_hook
// ===========================================================================
pub mod du_vu_doll;
pub mod girya;
pub mod sling;
pub mod preserved_insect;
pub mod slavers_collar;

// ===========================================================================
// Combat-start: flag setting
// ===========================================================================
pub mod ginger;
pub mod turnip;
pub mod mark_of_bloom;
pub mod magic_flower;
pub mod snecko_eye;

// ===========================================================================
// Combat-start: counter initialization
// ===========================================================================
pub mod velvet_choker_init;
pub mod pocketwatch;
pub mod art_of_war;
pub mod orange_pellets;
pub mod horn_cleat;
pub mod captains_wheel;
pub mod stone_calendar;

// ===========================================================================
// Combat-start: orb channeling (complex_hook)
// ===========================================================================
pub mod symbiotic_virus;
pub mod cracked_core;
pub mod nuclear_battery;

// ===========================================================================
// Combat-start: card generation (declarative)
// ===========================================================================
pub mod pure_water;
pub mod ninja_scroll;
pub mod holy_water;
pub mod mark_of_pain;

// ===========================================================================
// Combat-start: other
// ===========================================================================
pub mod pantograph;
pub mod neows_lament;

// ===========================================================================
// Complex relics (use complex_hook or stub-only)
// ===========================================================================
pub mod pen_nib;

// ===========================================================================
// Turn-start relics (draw, orb slots, counter resets)
// ===========================================================================
pub mod bag_of_prep;
pub mod ring_of_snake;
pub mod inserter;

// ===========================================================================
// Turn-end relics
// ===========================================================================
pub mod frozen_core;
// stone_calendar TurnEnd trigger added to existing stone_calendar.rs

// ===========================================================================
// On-card-play relics
// ===========================================================================
pub mod mummified_hand;
pub mod yang_duality;
pub mod velvet_choker;
// pocketwatch OnAnyCardPlayed trigger added to existing pocketwatch.rs

// ===========================================================================
// On-HP-loss relics
// ===========================================================================
pub mod centennial_puzzle;
pub mod runic_cube;
pub mod emotion_chip;

// ===========================================================================
// On-enemy-death relics
// ===========================================================================
pub mod the_specimen;

// ===========================================================================
// On-victory relics
// ===========================================================================
pub mod meat_on_the_bone;
pub mod face_of_cleric;

// ===========================================================================
// Stance-change relics
// ===========================================================================
pub mod teardrop_locket;

// ===========================================================================
// Damage modifiers (called inline, not via dispatch_trigger)
// ===========================================================================
pub mod boot;
pub mod torii;
pub mod tungsten_rod;
pub mod champion_belt;
pub mod hand_drill;

// ===========================================================================
// Passive bonuses (called inline, not via dispatch_trigger)
// ===========================================================================
pub mod strike_dummy;
pub mod wrist_blade;
pub mod snecko_skull;

// ===========================================================================
// Remaining combat relics
// ===========================================================================
pub mod runic_capacitor;
pub mod ring_of_serpent;
pub mod violet_lotus;
pub mod red_skull;
pub mod enchiridion;
pub mod warped_tongs;
pub mod gambling_chip;
pub mod hovering_kite;
pub mod lizard_tail;
pub mod ancient_tea_set;
pub mod medical_kit;
pub mod blue_candle;
pub mod strange_spoon;

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
    // Combat-start: stat buffs (complex_hook)
    &du_vu_doll::DEF,
    &girya::DEF,
    &sling::DEF,
    &preserved_insect::DEF,
    &slavers_collar::DEF,
    // Combat-start: flag setting
    &ginger::DEF,
    &turnip::DEF,
    &mark_of_bloom::DEF,
    &magic_flower::DEF,
    &snecko_eye::DEF,
    // Combat-start: counter initialization
    &velvet_choker_init::DEF,
    &pocketwatch::DEF,
    &art_of_war::DEF,
    &orange_pellets::DEF,
    &horn_cleat::DEF,
    &captains_wheel::DEF,
    &stone_calendar::DEF,
    // Combat-start: orb channeling
    &symbiotic_virus::DEF,
    &cracked_core::DEF,
    &nuclear_battery::DEF,
    // Combat-start: card generation
    &pure_water::DEF,
    &ninja_scroll::DEF,
    &holy_water::DEF,
    &mark_of_pain::DEF,
    // Combat-start: other
    &pantograph::DEF,
    &neows_lament::DEF,
    // Complex relics
    &pen_nib::DEF,
    // Turn-start relics
    &bag_of_prep::DEF,
    &ring_of_snake::DEF,
    &inserter::DEF,
    // Turn-end relics
    &frozen_core::DEF,
    // On-card-play relics
    &mummified_hand::DEF,
    &yang_duality::DEF,
    &velvet_choker::DEF,
    // On-HP-loss relics
    &centennial_puzzle::DEF,
    &runic_cube::DEF,
    &emotion_chip::DEF,
    // On-enemy-death relics
    &the_specimen::DEF,
    // On-victory relics
    &meat_on_the_bone::DEF,
    &face_of_cleric::DEF,
    // Stance-change relics
    &teardrop_locket::DEF,
    // Damage modifiers
    &boot::DEF,
    &torii::DEF,
    &tungsten_rod::DEF,
    &champion_belt::DEF,
    &hand_drill::DEF,
    // Passive bonuses
    &strike_dummy::DEF,
    &wrist_blade::DEF,
    &snecko_skull::DEF,
    // Remaining combat relics
    &runic_capacitor::DEF,
    &ring_of_serpent::DEF,
    &violet_lotus::DEF,
    &red_skull::DEF,
    &enchiridion::DEF,
    &warped_tongs::DEF,
    &gambling_chip::DEF,
    &hovering_kite::DEF,
    &lizard_tail::DEF,
    &ancient_tea_set::DEF,
    &medical_kit::DEF,
    &blue_candle::DEF,
    &strange_spoon::DEF,
];

pub fn relic_def_by_id(id: &str) -> Option<&'static EntityDef> {
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
        assert!(RELIC_DEFS.len() >= 95, "Expected at least 95 relic defs, got {}", RELIC_DEFS.len());
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
    fn test_relic_def_by_id() {
        assert!(relic_def_by_id("Vajra").is_some());
        assert!(relic_def_by_id("Nonexistent").is_none());
    }

    #[test]
    fn test_vajra_has_combat_start_trigger() {
        let def = relic_def_by_id("Vajra").unwrap();
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(def.triggers[0].trigger, crate::effects::trigger::Trigger::CombatStart);
    }

    #[test]
    fn test_ornamental_fan_has_counter() {
        let def = relic_def_by_id("Ornamental Fan").unwrap();
        assert_eq!(def.triggers.len(), 2); // OnAttackPlayed counter + TurnStart reset
        let te = &def.triggers[0];
        assert_eq!(te.trigger, crate::effects::trigger::Trigger::OnAttackPlayed);
        assert!(te.counter.is_some());
        let (counter_id, threshold) = te.counter.unwrap();
        assert_eq!(counter_id, crate::status_ids::sid::ORNAMENTAL_FAN_COUNTER);
        assert_eq!(threshold, 3);
        // Second trigger resets counter at turn start
        assert_eq!(def.triggers[1].trigger, crate::effects::trigger::Trigger::TurnStart);
    }

    #[test]
    fn test_orichalcum_has_no_block_condition() {
        let def = relic_def_by_id("Orichalcum").unwrap();
        assert_eq!(def.triggers[0].condition, crate::effects::trigger::TriggerCondition::NoBlock);
    }

    #[test]
    fn test_lantern_has_first_turn_condition() {
        let def = relic_def_by_id("Lantern").unwrap();
        assert_eq!(def.triggers[0].condition, crate::effects::trigger::TriggerCondition::FirstTurn);
    }

    #[test]
    fn test_simple_defs_have_no_complex_hook() {
        let simple_relics = ["Vajra", "Anchor", "Orichalcum", "Mercury Hourglass", "Kunai"];
        for name in simple_relics {
            let def = relic_def_by_id(name).unwrap();
            assert!(def.complex_hook.is_none(), "{} should not have a complex_hook", name);
        }
    }

    #[test]
    fn test_brimstone_has_two_effects() {
        let def = relic_def_by_id("Brimstone").unwrap();
        assert_eq!(def.triggers[0].effects.len(), 2);
    }

    #[test]
    fn test_mutagenic_strength_has_two_effects() {
        let def = relic_def_by_id("MutagenicStrength").unwrap();
        assert_eq!(def.triggers[0].effects.len(), 2);
    }
}
