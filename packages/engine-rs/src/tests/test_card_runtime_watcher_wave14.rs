#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SpiritShield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Ragnarok.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Scrawl.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, ensure_in_hand, force_player_turn, make_deck, play_on_enemy, play_self};

fn one_enemy_engine(enemy_hp: i32, enemy_block: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Judgement"]),
        vec![enemy_no_intent("JawWorm", enemy_hp, enemy_hp.max(1))],
        3,
    ));
    force_player_turn(&mut engine);
    engine.state.enemies[0].entity.block = enemy_block;
    engine
}

#[test]
fn watcher_wave14_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let pressure_points = registry
        .get("PathToVictory")
        .expect("Pressure Points should be registered");
    assert_eq!(
        pressure_points.effect_data,
        &[
            E::Simple(SE::AddStatus(T::SelectedEnemy, sid::MARK, A::Magic)),
            E::Simple(SE::TriggerMarks),
        ]
    );
    assert!(pressure_points.complex_hook.is_none());

    let spirit_shield = registry
        .get("SpiritShield")
        .expect("Spirit Shield should be registered");
    assert_eq!(
        spirit_shield.effect_data,
        &[
            E::Simple(SE::GainBlock(A::HandSize)),
            E::Simple(SE::GainBlock(A::HandSize)),
            E::Simple(SE::GainBlock(A::HandSize)),
        ]
    );
    assert!(spirit_shield.complex_hook.is_none());

    let spirit_shield_plus = registry
        .get("SpiritShield+")
        .expect("Spirit Shield+ should be registered");
    assert_eq!(
        spirit_shield_plus.effect_data,
        &[
            E::Simple(SE::GainBlock(A::HandSize)),
            E::Simple(SE::GainBlock(A::HandSize)),
            E::Simple(SE::GainBlock(A::HandSize)),
            E::Simple(SE::GainBlock(A::HandSize)),
        ]
    );
    assert!(spirit_shield_plus.complex_hook.is_none());

    let ragnarok = registry.get("Ragnarok").expect("Ragnarok should be registered");
    assert_eq!(
        ragnarok.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
        ]
    );
    assert!(ragnarok.complex_hook.is_none());

    let scrawl = registry.get("Scrawl").expect("Scrawl should be registered");
    assert_eq!(scrawl.effect_data, &[E::Simple(SE::DrawCards(A::Fixed(10)))]);
    assert!(scrawl.complex_hook.is_none());

    let judgement = registry.get("Judgement").expect("Judgement should be registered");
    assert_eq!(judgement.effect_data, &[E::Simple(SE::Judgement(A::Magic))]);
    assert!(judgement.complex_hook.is_none());
}

#[test]
fn watcher_wave14_pressure_points_spirit_shield_ragnarok_and_scrawl_follow_engine_path() {
    let mut pressure_points = one_enemy_engine(30, 20);
    ensure_in_hand(&mut pressure_points, "Pray");
    ensure_in_hand(&mut pressure_points, "PathToVictory");
    assert!(play_on_enemy(&mut pressure_points, "PathToVictory", 0));
    assert_eq!(pressure_points.state.enemies[0].entity.status(sid::MARK), 8);
    assert_eq!(pressure_points.state.enemies[0].entity.hp, 22);
    assert_eq!(pressure_points.state.enemies[0].entity.block, 20);

    let mut spirit_shield = engine_with_state(combat_state_with(
        make_deck(&["SpiritShield", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    force_player_turn(&mut spirit_shield);
    spirit_shield.state.hand = make_deck(&["SpiritShield", "Strike_P", "Defend_P", "Strike_P"]);
    assert!(play_self(&mut spirit_shield, "SpiritShield"));
    assert_eq!(spirit_shield.state.player.block, 9);

    let mut spirit_shield_plus = engine_with_state(combat_state_with(
        make_deck(&["SpiritShield+", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    force_player_turn(&mut spirit_shield_plus);
    spirit_shield_plus.state.hand = make_deck(&["SpiritShield+", "Strike_P", "Defend_P", "Strike_P"]);
    assert!(play_self(&mut spirit_shield_plus, "SpiritShield+"));
    assert_eq!(spirit_shield_plus.state.player.block, 12);

    let mut ragnarok = engine_with_state(combat_state_with(
        make_deck(&["Ragnarok"]),
        vec![
            enemy_no_intent("JawWorm", 50, 50),
            enemy_no_intent("Cultist", 50, 50),
            enemy_no_intent("Louse", 50, 50),
        ],
        3,
    ));
    force_player_turn(&mut ragnarok);
    ragnarok.state.hand = make_deck(&["Ragnarok"]);
    let hp_before: i32 = ragnarok.state.enemies.iter().map(|enemy| enemy.entity.hp).sum();
    assert!(play_on_enemy(&mut ragnarok, "Ragnarok", 0));
    let hp_after: i32 = ragnarok.state.enemies.iter().map(|enemy| enemy.entity.hp).sum();
    assert_eq!(hp_before - hp_after, 25);

    let mut ragnarok_plus = engine_with_state(combat_state_with(
        make_deck(&["Ragnarok+"]),
        vec![
            enemy_no_intent("JawWorm", 60, 60),
            enemy_no_intent("Cultist", 60, 60),
            enemy_no_intent("Louse", 60, 60),
        ],
        3,
    ));
    force_player_turn(&mut ragnarok_plus);
    ragnarok_plus.state.hand = make_deck(&["Ragnarok+"]);
    let hp_before_plus: i32 = ragnarok_plus.state.enemies.iter().map(|enemy| enemy.entity.hp).sum();
    assert!(play_on_enemy(&mut ragnarok_plus, "Ragnarok+", 0));
    let hp_after_plus: i32 = ragnarok_plus.state.enemies.iter().map(|enemy| enemy.entity.hp).sum();
    assert_eq!(hp_before_plus - hp_after_plus, 36);

    let mut scrawl = engine_with_state(combat_state_with(
        make_deck(&[
            "Scrawl", "Strike_R", "Defend_R", "Strike_R", "Defend_R", "Strike_R", "Defend_R",
            "Strike_R", "Defend_R", "Strike_R", "Defend_R",
        ]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        10,
    ));
    force_player_turn(&mut scrawl);
    scrawl.state.hand = make_deck(&["Scrawl", "Strike_R", "Defend_R", "Strike_R", "Defend_R"]);
    scrawl.state.draw_pile = make_deck(&["Strike_R", "Defend_R", "Strike_R", "Defend_R", "Strike_R", "Defend_R"]);
    assert!(play_self(&mut scrawl, "Scrawl"));
    assert_eq!(scrawl.state.hand.len(), 10);
    assert!(scrawl
        .state
        .exhaust_pile
        .iter()
        .any(|card| scrawl.card_registry.card_name(card.def_id) == "Scrawl"));
}
