#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Regret.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Void.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Parasite.java

use crate::cards::global_registry;
use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, OnDrawRule, WhileInHandRule};
use crate::tests::support::*;

#[test]
fn support_wave1_registry_keeps_shared_support_cards_typed_runtime_metadata() {
    let registry = global_registry();

    for card_id in [
        "Slimed", "Wound", "Daze", "Burn", "Burn+", "Void", "Decay", "Regret", "Doubt",
        "Shame", "AscendersBane", "Clumsy", "CurseOfTheBell", "Injury", "Necronomicurse",
        "Normality", "Pain", "Parasite", "Pride", "Writhe",
    ] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
        assert!(card.complex_hook.is_none(), "{card_id} should stay non-bespoke");
    }

    let burn = registry.get("Burn").unwrap();
    assert!(burn.runtime_traits().unplayable);
    assert!(burn
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Damage))));

    let regret = registry.get("Regret").unwrap();
    assert!(regret.runtime_traits().unplayable);
    assert!(regret
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Regret))));

    let doubt = registry.get("Doubt").unwrap();
    assert!(doubt.runtime_traits().unplayable);
    assert!(doubt
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Weak))));

    let shame = registry.get("Shame").unwrap();
    assert!(shame.runtime_traits().unplayable);
    assert!(shame
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::Frail))));

    let pain = registry.get("Pain").unwrap();
    assert!(pain.runtime_traits().unplayable);
    assert!(pain
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed))));

    let void = registry.get("Void").unwrap();
    assert!(void.runtime_traits().unplayable);
    assert!(void.runtime_traits().ethereal);
    assert!(void
        .runtime_triggers()
        .iter()
        .any(|trigger| matches!(trigger, CardRuntimeTrigger::OnDraw(OnDrawRule::LoseEnergy))));

    assert!(registry.get("Parasite").unwrap().runtime_traits().unplayable);
    assert!(registry.get("Daze").unwrap().runtime_traits().ethereal);
    assert!(registry.get("Wound").unwrap().runtime_traits().unplayable);
}

#[test]
fn support_wave1_end_turn_curse_and_status_hooks_fire_on_the_runtime_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Burn", "Regret", "Doubt", "Shame", "Pride"]);

    let hp_before = engine.state.player.hp;
    end_turn(&mut engine);

    assert_eq!(hp_before - engine.state.player.hp, 7);
    assert_eq!(draw_prefix_count(&engine, "Pride"), 1);
}

#[test]
fn support_wave1_pain_triggers_when_any_other_card_is_played() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Pain", "Strike"]);

    let hp_before = engine.state.player.hp;
    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(hp_before - engine.state.player.hp, 1);
    assert_eq!(hand_count(&engine, "Pain"), 1);
}

#[test]
fn support_wave1_void_loses_energy_when_drawn() {
    let mut engine = engine_without_start(
        make_deck(&["Void"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    let energy_before = engine.state.energy;
    engine.draw_cards(1);

    assert_eq!(engine.state.energy, energy_before - 1);
    assert_eq!(hand_count(&engine, "Void"), 1);
}
