#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::engine::{ChoiceReason, ChoiceOption};
use crate::effects::declarative::{Effect as E};
use crate::tests::support::*;

#[test]
fn test_card_runtime_watcher_wave19_registry_promotes_wish_to_typed_named_choice() {
    let wish = global_registry().get("Wish").expect("Wish should exist");
    assert_eq!(wish.effect_data, &[E::ChooseNamedOptions(&["Strength", "Gold", "Plated Armor"])]);
    assert!(wish.complex_hook.is_none());

    let wish_plus = global_registry().get("Wish+").expect("Wish+ should exist");
    assert_eq!(wish_plus.effect_data, &[E::ChooseNamedOptions(&["Strength", "Gold", "Plated Armor"])]);
    assert!(wish_plus.complex_hook.is_none());
}

#[test]
fn test_card_runtime_watcher_wave19_wish_named_choice_resolves_strength_branch() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Wish"]);
    engine.state.player.set_status(crate::status_ids::sid::STRENGTH, 0);
    engine.state.player.block = 0;

    assert!(play_self(&mut engine, "Wish"));
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Wish should open a choice");
    assert_eq!(choice.reason, ChoiceReason::PickOption);
    assert_eq!(choice.options.len(), 3);
    assert!(matches!(choice.options[0], ChoiceOption::Named("Strength")));

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.state.player.status(crate::status_ids::sid::STRENGTH), 3);
    assert_eq!(engine.phase, crate::engine::CombatPhase::PlayerTurn);
    assert!(engine.choice.is_none());
}

#[test]
#[ignore = "Blocked on Java mark-triggered damage primitive for Pressure Points; the current runtime can apply Mark but cannot yet resolve enemy-specific mark damage that bypasses block. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java"]
fn test_card_runtime_watcher_wave19_pressure_points_needs_mark_damage_primitive() {}

#[test]
#[ignore = "Blocked on Java enemy-hp-threshold kill primitive for Judgement; the current runtime can target and damage enemies but cannot yet perform a declarative kill-if-hp-below-threshold resolution. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java"]
fn test_card_runtime_watcher_wave19_judgement_needs_threshold_kill_primitive() {}
