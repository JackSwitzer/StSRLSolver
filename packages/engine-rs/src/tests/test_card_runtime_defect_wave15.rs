#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java

use crate::cards::global_registry;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self};

fn total_enemy_hp(engine: &crate::engine::CombatEngine) -> i32 {
    engine
        .state
        .enemies
        .iter()
        .map(|enemy| enemy.entity.hp.max(0))
        .sum()
}

#[test]
fn defect_wave15_registry_exports_blocked_cards_as_oracle_backed_entries() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert!(blizzard.effect_data.is_empty());
    assert!(blizzard.complex_hook.is_some());

    let blizzard_plus = global_registry().get("Blizzard+").expect("Blizzard+");
    assert!(blizzard_plus.effect_data.is_empty());
    assert!(blizzard_plus.complex_hook.is_some());

    let double_energy = global_registry().get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());

    let genetic = global_registry().get("Genetic Algorithm").expect("Genetic Algorithm");
    assert!(genetic.effect_data.is_empty());
    assert!(genetic.complex_hook.is_some());

    let melter = global_registry().get("Melter").expect("Melter");
    assert!(melter.effect_data.is_empty());
    assert!(melter.complex_hook.is_some());
}

#[test]
fn blizzard_does_nothing_without_frost_channeled_this_combat() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Blizzard"]);
    let hp_before = total_enemy_hp(&engine);

    assert!(play_self(&mut engine, "Blizzard"));
    assert_eq!(hp_before - total_enemy_hp(&engine), 0);
}

#[test]
#[ignore = "Blizzard still needs a typed frost-scale AoE primitive; Java Blizzard.java uses per-combat Frost Channeled counting and the typed runtime proof does not yet reproduce it."]
fn blizzard_still_needs_typed_frost_scale_aoe() {}

#[test]
#[ignore = "Double Energy still needs a typed energy-doubling primitive; Java DoubleEnergyAction doubles the current energy directly."]
fn double_energy_still_needs_typed_energy_doubling() {}

#[test]
#[ignore = "Genetic Algorithm still needs card-owned current-block seeding plus replay-state mutation; Java IncreaseMiscAction mutates the played copy before future plays."]
fn genetic_algorithm_still_needs_card_owned_misc_seeding() {}

#[test]
#[ignore = "Melter still needs a pre-damage enemy block removal primitive; Java RemoveAllBlockAction clears the target's block before damage."]
fn melter_still_needs_pre_damage_enemy_block_removal() {}
