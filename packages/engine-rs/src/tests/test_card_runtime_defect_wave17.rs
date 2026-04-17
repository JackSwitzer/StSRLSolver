#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DoubleEnergyAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/RemoveAllBlockAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self};

fn single_enemy_engine() -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.energy = 3;
    engine
}

#[test]
fn defect_wave17_registry_exports_typed_double_energy_and_genetic_algorithm() {
    let reg = global_registry();

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert_eq!(double_energy.effect_data, &[E::Simple(SE::DoubleEnergy)]);
    assert!(double_energy.complex_hook.is_none());

    let double_energy_plus = reg.get("Double Energy+").expect("Double Energy+");
    assert_eq!(double_energy_plus.effect_data, &[E::Simple(SE::DoubleEnergy)]);
    assert!(double_energy_plus.complex_hook.is_none());
    assert_eq!(double_energy_plus.cost, 0);

    let genetic = reg.get("Genetic Algorithm").expect("Genetic Algorithm");
    assert_eq!(
        genetic.effect_data,
        &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ]
    );
    assert!(genetic.complex_hook.is_none());

    let genetic_plus = reg.get("Genetic Algorithm+").expect("Genetic Algorithm+");
    assert_eq!(
        genetic_plus.effect_data,
        &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ]
    );
    assert!(genetic_plus.complex_hook.is_none());
    assert_eq!(genetic_plus.base_block, 0);

    let blizzard = reg.get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());
    assert_eq!(blizzard.card_type, CardType::Attack);
    assert_eq!(blizzard.target, CardTarget::AllEnemy);

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
}

#[test]
fn double_energy_doubles_current_energy_and_exhausts() {
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Double Energy"]);
    engine.state.energy = 3;

    assert!(play_self(&mut engine, "Double Energy"));
    assert_eq!(engine.state.energy, 4);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    assert_eq!(engine.state.exhaust_pile[0].def_id, engine.card_registry.card_id("Double Energy"));
}

#[test]
fn genetic_algorithm_updates_the_played_copy_misc_and_future_plays_use_the_new_seed() {
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Genetic Algorithm"]);
    engine.state.player.block = 0;

    assert!(play_self(&mut engine, "Genetic Algorithm"));
    assert_eq!(engine.state.player.block, 3);

    let played = engine
        .state
        .exhaust_pile
        .pop()
        .expect("played Genetic Algorithm should exhaust");
    assert_eq!(played.misc, 3);

    engine.state.hand = vec![played];
    engine.state.player.block = 0;
    engine.state.energy = 3;

    assert!(play_self(&mut engine, "Genetic Algorithm"));
    assert_eq!(engine.state.player.block, 5);

    let updated = engine
        .state
        .exhaust_pile
        .last()
        .copied()
        .expect("played Genetic Algorithm should exhaust again");
    assert_eq!(updated.misc, 5);
}

#[test]
fn blizzard_uses_the_typed_frost_count_aoe_primitive() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());
}

#[test]
fn melter_removes_block_before_damage_on_the_typed_surface() {
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Melter"]);
    engine.state.enemies[0].entity.block = 12;

    assert!(play_on_enemy(&mut engine, "Melter", 0));
    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
}
