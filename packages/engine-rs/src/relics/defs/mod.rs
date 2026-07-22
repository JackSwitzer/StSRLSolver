//! Per-relic EntityDef definitions.
//!
//! Each relic's entire declarative behavior lives in one file.
//! The RELIC_DEFS registry collects all definitions for dispatch.
//! These defs are the canonical relic runtime metadata surface.

use crate::effects::entity_def::EntityDef;

// ===========================================================================
// Combat-start stat buffs (pure declarative)
// ===========================================================================
pub mod akabeko;
pub mod anchor;
pub mod bag_of_marbles;
pub mod blood_vial;
pub mod bloody_idol;
pub mod bronze_scales;
pub mod busted_crown;
pub mod calipers;
pub mod calling_bell;
pub mod chemical_x;
pub mod clockwork_souvenir;
pub mod coffee_dripper;
pub mod cursed_key;
pub mod darkstone_periapt;
pub mod data_disk;
pub mod dream_catcher;
pub mod ectoplasm;
pub mod eternal_feather;
pub mod fossilized_helix;
pub mod frozen_egg_2;
pub mod frozen_eye;
pub mod fusion_hammer;
pub mod gremlin_mask;
pub mod ice_cream;
pub mod molten_egg_2;
pub mod mutagenic_strength;
pub mod oddly_smooth_stone;
pub mod philosophers_stone;
pub mod red_mask;
pub mod thread_and_needle;
pub mod toxic_egg_2;
pub mod twisted_funnel;
pub mod vajra;

// ===========================================================================
// Counter-based relics (OnAttackPlayed/OnSkillPlayed/OnAnyCardPlayed/etc.)
// ===========================================================================
pub mod happy_flower;
pub mod incense_burner;
pub mod ink_bottle;
pub mod kunai;
pub mod letter_opener;
pub mod nunchaku;
pub mod ornamental_fan;
pub mod shuriken;
pub mod sundial;

// ===========================================================================
// Turn-based relics
// ===========================================================================
pub mod brimstone;
pub mod cloak_clasp;
pub mod damaru;
pub mod lantern;
pub mod mercury_hourglass;
pub mod orichalcum;

// ===========================================================================
// Event-triggered relics (card play, exhaust, discard, death, victory, etc.)
// ===========================================================================
pub mod bird_faced_urn;
pub mod black_blood;
pub mod black_star;
pub mod burning_blood;
pub mod charons_ashes;
pub mod dead_branch;
pub mod gremlin_horn;
pub mod melange;
pub mod self_forming_clay;
pub mod the_abacus;
pub mod tingsha;
pub mod tough_bandages;
pub mod toy_ornithopter;

// ===========================================================================
// Combat-start: stat buffs requiring complex_hook
// ===========================================================================
pub mod du_vu_doll;
pub mod girya;
pub mod preserved_insect;
pub mod slavers_collar;
pub mod sling;

// ===========================================================================
// Combat-start: flag setting
// ===========================================================================
pub mod ginger;
pub mod magic_flower;
pub mod mark_of_bloom;
pub mod snecko_eye;
pub mod turnip;

// ===========================================================================
// Combat-start: counter initialization
// ===========================================================================
pub mod art_of_war;
pub mod captains_wheel;
pub mod horn_cleat;
pub mod orange_pellets;
pub mod pocketwatch;
pub mod stone_calendar;
pub mod velvet_choker_init;

// ===========================================================================
// Combat-start: orb channeling (complex_hook)
// ===========================================================================
pub mod cracked_core;
pub mod nuclear_battery;
pub mod symbiotic_virus;

// ===========================================================================
// Combat-start: card generation (declarative)
// ===========================================================================
pub mod holy_water;
pub mod mark_of_pain;
pub mod ninja_scroll;
pub mod pure_water;

// ===========================================================================
// Combat-start: other
// ===========================================================================
pub mod neows_lament;
pub mod pantograph;

// ===========================================================================
// Complex relics (use complex_hook or stub-only)
// ===========================================================================
pub mod pen_nib;

// ===========================================================================
// Turn-start relics (draw, orb slots, counter resets)
// ===========================================================================
pub mod bag_of_prep;
pub mod inserter;
pub mod ring_of_snake;

// ===========================================================================
// Turn-end relics
// ===========================================================================
pub mod frozen_core;
// stone_calendar TurnEnd trigger added to existing stone_calendar.rs

// ===========================================================================
// On-card-play relics
// ===========================================================================
pub mod mummified_hand;
pub mod velvet_choker;
pub mod yang_duality;
// pocketwatch OnAnyCardPlayed trigger added to existing pocketwatch.rs

// ===========================================================================
// On-HP-loss relics
// ===========================================================================
pub mod centennial_puzzle;
pub mod emotion_chip;
pub mod runic_cube;

// ===========================================================================
// On-enemy-death relics
// ===========================================================================
pub mod the_specimen;

// ===========================================================================
// On-victory relics
// ===========================================================================
pub mod face_of_cleric;
pub mod meat_on_the_bone;

// ===========================================================================
// Stance-change relics
// ===========================================================================
pub mod teardrop_locket;

// ===========================================================================
// Damage modifiers (called inline, not via dispatch_trigger)
// ===========================================================================
pub mod boot;
pub mod bottled_flame;
pub mod bottled_lightning;
pub mod bottled_tornado;
pub mod champion_belt;
pub mod hand_drill;
pub mod torii;
pub mod tungsten_rod;

// ===========================================================================
// Passive bonuses (called inline, not via dispatch_trigger)
// ===========================================================================
pub mod snecko_skull;
pub mod strike_dummy;
pub mod wrist_blade;

// ===========================================================================
// Remaining combat relics
// ===========================================================================
pub mod ancient_tea_set;
pub mod astrolabe;
pub mod blue_candle;
pub mod cauldron;
pub mod dollys_mirror;
pub mod empty_cage;
pub mod enchiridion;
pub mod gambling_chip;
pub mod hovering_kite;
pub mod juzu_bracelet;
pub mod lizard_tail;
pub mod mango;
pub mod matryoshka;
pub mod maw_bank;
pub mod meal_ticket;
pub mod medical_kit;
pub mod membership_card;
pub mod nilrys_codex;
pub mod old_coin;
pub mod omamori;
pub mod orrery;
pub mod pandoras_box;
pub mod peace_pipe;
pub mod prismatic_shard;
pub mod red_skull;
pub mod ring_of_serpent;
pub mod runic_capacitor;
pub mod runic_dome;
pub mod runic_pyramid;
pub mod sacred_bark;
pub mod shovel;
pub mod smiling_mask;
pub mod sozu;
pub mod strange_spoon;
pub mod tiny_chest;
pub mod tiny_house;
pub mod toolbox;
pub mod unceasing_top;
pub mod violet_lotus;
pub mod waffle;
pub mod warped_tongs;
pub mod white_beast;

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
    &busted_crown::DEF,
    &calipers::DEF,
    &calling_bell::DEF,
    &coffee_dripper::DEF,
    &cursed_key::DEF,
    &darkstone_periapt::DEF,
    &dream_catcher::DEF,
    &ectoplasm::DEF,
    &eternal_feather::DEF,
    &fusion_hammer::DEF,
    &frozen_egg_2::DEF,
    &molten_egg_2::DEF,
    &toxic_egg_2::DEF,
    &ice_cream::DEF,
    &clockwork_souvenir::DEF,
    &fossilized_helix::DEF,
    &blood_vial::DEF,
    &bloody_idol::DEF,
    &twisted_funnel::DEF,
    &mutagenic_strength::DEF,
    &gremlin_mask::DEF,
    &philosophers_stone::DEF,
    &chemical_x::DEF,
    &frozen_eye::DEF,
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
    &black_star::DEF,
    &charons_ashes::DEF,
    &dead_branch::DEF,
    &tough_bandages::DEF,
    &tingsha::DEF,
    &gremlin_horn::DEF,
    &burning_blood::DEF,
    &black_blood::DEF,
    &toy_ornithopter::DEF,
    &self_forming_clay::DEF,
    &the_abacus::DEF,
    &melange::DEF,
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
    &bottled_flame::DEF,
    &bottled_lightning::DEF,
    &bottled_tornado::DEF,
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
    &nilrys_codex::DEF,
    &toolbox::DEF,
    &prismatic_shard::DEF,
    &warped_tongs::DEF,
    &gambling_chip::DEF,
    &hovering_kite::DEF,
    &lizard_tail::DEF,
    &mango::DEF,
    &matryoshka::DEF,
    &maw_bank::DEF,
    &meal_ticket::DEF,
    &juzu_bracelet::DEF,
    &waffle::DEF,
    &membership_card::DEF,
    &cauldron::DEF,
    &dollys_mirror::DEF,
    &orrery::DEF,
    &ancient_tea_set::DEF,
    &astrolabe::DEF,
    &medical_kit::DEF,
    &blue_candle::DEF,
    &strange_spoon::DEF,
    &old_coin::DEF,
    &smiling_mask::DEF,
    &peace_pipe::DEF,
    &pandoras_box::DEF,
    &shovel::DEF,
    &unceasing_top::DEF,
    &tiny_chest::DEF,
    &omamori::DEF,
    &runic_dome::DEF,
    &runic_pyramid::DEF,
    &sacred_bark::DEF,
    &sozu::DEF,
    &tiny_house::DEF,
    &empty_cage::DEF,
    &white_beast::DEF,
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
        assert!(
            RELIC_DEFS.len() >= 95,
            "Expected at least 95 relic defs, got {}",
            RELIC_DEFS.len()
        );
    }

    #[test]
    fn test_all_defs_are_relics() {
        for def in RELIC_DEFS.iter() {
            assert_eq!(
                def.kind,
                EntityKind::Relic,
                "Expected EntityKind::Relic for {}",
                def.id
            );
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
    fn test_vajra_has_combat_start_top_trigger() {
        let def = relic_def_by_id("Vajra").unwrap();
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(
            def.triggers[0].trigger,
            crate::effects::trigger::Trigger::CombatStartTop
        );
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
        assert_eq!(
            def.triggers[1].trigger,
            crate::effects::trigger::Trigger::TurnStart
        );
    }

    #[test]
    fn test_orichalcum_has_no_block_condition() {
        let def = relic_def_by_id("Orichalcum").unwrap();
        assert_eq!(
            def.triggers[0].condition,
            crate::effects::trigger::TriggerCondition::NoBlock
        );
    }

    #[test]
    fn test_lantern_has_first_turn_condition() {
        let def = relic_def_by_id("Lantern").unwrap();
        assert_eq!(
            def.triggers[0].condition,
            crate::effects::trigger::TriggerCondition::FirstTurn
        );
    }

    #[test]
    fn test_simple_defs_have_no_complex_hook() {
        // Mercury Hourglass and Orichalcum intentionally use hooks: Java
        // resolves the former as THORNS damage and queues the latter's block
        // at the top of the end-turn action queue.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Orichalcum.java
        let simple_relics = ["Vajra", "Anchor", "Kunai"];
        for name in simple_relics {
            let def = relic_def_by_id(name).unwrap();
            assert!(
                def.complex_hook.is_none(),
                "{} should not have a complex_hook",
                name
            );
        }

        assert!(relic_def_by_id("Orichalcum")
            .unwrap()
            .complex_hook
            .is_some());
    }

    #[test]
    fn test_brimstone_uses_ordered_turn_start_hook() {
        // Brimstone.atTurnStart adds one bottom action, then front-inserts the
        // player and per-monster ApplyPowerActions; a hook is required to
        // preserve that Java ordering rather than flattening two effects.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Brimstone.java
        let def = relic_def_by_id("Brimstone").unwrap();
        assert_eq!(def.triggers.len(), 1);
        assert_eq!(
            def.triggers[0].trigger,
            crate::effects::trigger::Trigger::TurnStart
        );
        assert!(def.triggers[0].effects.is_empty());
        assert!(def.complex_hook.is_some());
    }

    #[test]
    fn test_mutagenic_strength_has_two_effects() {
        let def = relic_def_by_id("MutagenicStrength").unwrap();
        assert_eq!(def.triggers[0].effects.len(), 2);
    }
}
