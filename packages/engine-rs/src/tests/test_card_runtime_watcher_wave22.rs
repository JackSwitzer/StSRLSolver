#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn test_card_runtime_watcher_wave22_registry_promotes_pressure_points_to_typed_primary_surface() {
    let pressure_points = global_registry()
        .get("PathToVictory")
        .expect("Pressure Points should exist");
    assert_eq!(pressure_points.name, "Pressure Points");
    assert_eq!(
        pressure_points.effect_data,
        &[
            E::Simple(SE::AddStatus(T::SelectedEnemy, sid::MARK, A::Magic)),
            E::Simple(SE::TriggerMarks),
        ]
    );
    assert!(pressure_points.complex_hook.is_none());

    let pressure_points_plus = global_registry()
        .get("PathToVictory+")
        .expect("Pressure Points+ should exist");
    assert_eq!(pressure_points_plus.name, "Pressure Points+");
    assert_eq!(
        pressure_points_plus.effect_data,
        &[
            E::Simple(SE::AddStatus(T::SelectedEnemy, sid::MARK, A::Magic)),
            E::Simple(SE::TriggerMarks),
        ]
    );
    assert!(pressure_points_plus.complex_hook.is_none());
}

#[test]
fn test_card_runtime_watcher_wave22_pressure_points_triggers_mark_damage_bypassing_block() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 20, 20),
            enemy_no_intent("Cultist", 15, 15),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["PathToVictory"]);
    engine.state.enemies[0].entity.block = 4;
    engine.state.enemies[1].entity.add_status(sid::MARK, 5);

    assert!(play_on_enemy(&mut engine, "PathToVictory", 0));

    assert_eq!(engine.state.enemies[0].entity.status(sid::MARK), 8);
    assert_eq!(engine.state.enemies[0].entity.hp, 12);
    assert_eq!(engine.state.enemies[0].entity.block, 4);
    assert_eq!(engine.state.enemies[1].entity.hp, 10);
    assert_eq!(engine.state.enemies[1].entity.status(sid::MARK), 5);
}
