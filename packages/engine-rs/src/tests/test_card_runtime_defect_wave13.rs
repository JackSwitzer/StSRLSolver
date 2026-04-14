#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java

use crate::cards::{global_registry, CardTarget, CardType};

#[test]
fn test_card_runtime_defect_wave13_registry_exports_blocked_melter_surface() {
    let reg = global_registry();

    let melter = reg.get("Melter").expect("Melter");
    assert!(melter.effect_data.is_empty());
    assert!(melter.complex_hook.is_some(), "Melter still needs the Java pre-damage block-removal hook");
    assert_eq!(melter.card_type, CardType::Attack);
    assert_eq!(melter.target, CardTarget::Enemy);

    let blizzard = reg.get("Blizzard").expect("Blizzard");
    assert!(blizzard.effect_data.is_empty());
    assert!(blizzard.complex_hook.is_some());

    let genetic = reg
        .get("Genetic Algorithm")
        .expect("Genetic Algorithm");
    assert!(genetic.effect_data.is_empty());
    assert!(genetic.complex_hook.is_some());

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());
}

#[test]
#[ignore = "Blocked on a typed frost-count damage primitive; Java Blizzard scales by frost channeled this combat before dealing AoE damage"]
fn test_card_runtime_defect_wave13_blizzard_still_needs_typed_frost_count_damage_scaling() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert!(blizzard.effect_data.is_empty());
    assert!(blizzard.effects.contains(&"damage_per_frost_channeled"));
}

#[test]
#[ignore = "Blocked on card-owned current-block seeding plus replay-state mutation; Java Genetic Algorithm updates misc before future plays"]
fn test_card_runtime_defect_wave13_genetic_algorithm_still_needs_card_owned_misc_seeding() {
    let genetic = global_registry()
        .get("Genetic Algorithm")
        .expect("Genetic Algorithm");
    assert!(genetic.effect_data.is_empty());
    assert!(genetic.effects.contains(&"genetic_algorithm"));
}

#[test]
#[ignore = "Blocked on a typed pre-damage enemy block removal primitive; Java Melter removes all block before damage"]
fn test_card_runtime_defect_wave13_melter_still_needs_pre_damage_block_removal_primitive() {
    let melter = global_registry().get("Melter").expect("Melter");
    assert!(melter.effect_data.is_empty());
    assert!(melter.effects.contains(&"remove_enemy_block"));
}

#[test]
#[ignore = "Blocked on a typed energy-doubling primitive; Java DoubleEnergyAction doubles the current energy directly"]
fn test_card_runtime_defect_wave13_double_energy_still_needs_typed_energy_doubling() {
    let double_energy = global_registry().get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());
}
