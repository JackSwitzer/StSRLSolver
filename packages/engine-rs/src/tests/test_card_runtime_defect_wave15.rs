#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::orbs::OrbType;
use crate::status_ids::sid;
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
fn defect_wave15_registry_exports_typed_and_blocked_cards_honestly() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());

    let blizzard_plus = global_registry().get("Blizzard+").expect("Blizzard+");
    assert_eq!(
        blizzard_plus.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard_plus.complex_hook.is_none());

    let double_energy = global_registry().get("Double Energy").expect("Double Energy");
    assert_eq!(double_energy.effect_data, &[E::Simple(SE::DoubleEnergy)]);
    assert!(double_energy.complex_hook.is_none());

    let genetic = global_registry().get("Genetic Algorithm").expect("Genetic Algorithm");
    assert_eq!(
        genetic.effect_data,
        &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ]
    );
    assert!(genetic.complex_hook.is_none());

    let melter = global_registry().get("Melter").expect("Melter");
    assert_eq!(
        melter.effect_data,
        &[
            E::Simple(SE::RemoveEnemyBlock(T::SelectedEnemy)),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
    assert!(melter.complex_hook.is_none());
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
fn blizzard_typed_runtime_damages_all_enemies_when_frost_has_been_channeled() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 35, 35),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Frost);
    engine.channel_orb(OrbType::Frost);
    engine.state.hand = make_deck(&["Blizzard"]);
    let hp_before = total_enemy_hp(&engine);

    assert!(play_self(&mut engine, "Blizzard"));
    assert_eq!(hp_before - total_enemy_hp(&engine), 8);
}

#[test]
fn blizzard_typed_registry_surface_is_present() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());
}
