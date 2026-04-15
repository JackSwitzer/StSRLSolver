#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/RemoveAllBlockAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy};

#[test]
fn test_card_runtime_defect_wave13_registry_exports_blocked_melter_surface() {
    let reg = global_registry();

    let melter = reg.get("Melter").expect("Melter");
    assert_eq!(
        melter.effect_data,
        &[
            E::Simple(SE::RemoveEnemyBlock(T::SelectedEnemy)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
    assert!(melter.complex_hook.is_none());
    assert_eq!(melter.card_type, CardType::Attack);
    assert_eq!(melter.target, CardTarget::Enemy);

    let blizzard = reg.get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());

    let genetic = reg
        .get("Genetic Algorithm")
        .expect("Genetic Algorithm");
    assert_eq!(
        genetic.effect_data,
        &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ]
    );
    assert!(genetic.complex_hook.is_none());

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert_eq!(double_energy.effect_data, &[E::Simple(SE::DoubleEnergy)]);
    assert!(double_energy.complex_hook.is_none());
}

#[test]
fn test_card_runtime_defect_wave13_blizzard_uses_the_typed_frost_count_damage_scaling_surface() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());
}

#[test]
fn test_card_runtime_defect_wave13_melter_uses_the_typed_pre_damage_block_removal_surface() {
    let melter = global_registry().get("Melter").expect("Melter");
    assert_eq!(
        melter.effect_data,
        &[
            E::Simple(SE::RemoveEnemyBlock(T::SelectedEnemy)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
}

#[test]
fn test_card_runtime_defect_wave13_melter_removes_block_before_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Melter"]);
    engine.state.enemies[0].entity.block = 12;

    assert!(play_on_enemy(&mut engine, "Melter", 0));
    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
}
