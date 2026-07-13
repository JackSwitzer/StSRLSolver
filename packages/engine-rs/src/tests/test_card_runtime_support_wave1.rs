#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Decay.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Regret.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Void.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Parasite.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, OnDrawRule, WhileInHandRule};
use crate::tests::support::*;

#[test]
fn support_wave1_registry_keeps_shared_support_cards_typed_runtime_metadata() {
    let registry = global_registry();

    for card_id in [
        "Slimed", "Wound", "Dazed", "Burn", "Burn+", "Void", "Decay", "Regret", "Doubt",
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
    assert!(registry.get("Dazed").unwrap().runtime_traits().ethereal);
    assert!(registry.get("Wound").unwrap().runtime_traits().unplayable);
}

#[test]
fn curse_of_the_bell_is_unplayable_unupgradable_non_ethereal_and_unremovable() {
    // CurseOfTheBell.java constructs a cost -2 Curse with empty use/upgrade
    // and no Ethereal flag. CardGroup.getPurgeableCards explicitly excludes
    // CurseOfTheBell from removal pools.
    let registry = global_registry();
    let card = registry.get("CurseOfTheBell").expect("Curse of the Bell");
    assert_eq!(card.cost, -2);
    assert!(card.runtime_traits().unplayable);
    assert!(card.runtime_traits().unremovable);
    assert!(!card.runtime_traits().ethereal);
    assert!(registry.get("CurseOfTheBell+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["CurseOfTheBell"]);
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, .. })
    }));

    end_turn(&mut engine);

    assert_eq!(hand_count(&engine, "CurseOfTheBell"), 1);
    assert_eq!(exhaust_prefix_count(&engine, "CurseOfTheBell"), 0);
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
fn decay_has_no_magic_stat_and_deals_two_thorns_damage_at_end_of_turn() {
    // Decay.java has cost -2, empty upgrade(), and no magic-number assignment.
    // Its end-turn queue auto-plays the card, whose use() deals a literal 2
    // THORNS damage; one Block therefore leaves one HP damage.
    let registry = global_registry();
    let decay = registry.get("Decay").expect("Decay");
    assert_eq!(decay.cost, -2);
    assert_eq!(decay.base_magic, -1);
    assert!(decay.runtime_traits().unplayable);
    assert!(registry.get("Decay+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.player.block = 1;
    engine.state.hand = make_deck(&["Decay"]);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before - 1);
}

#[test]
fn burn_variants_use_thorns_block_then_buffer_order_at_end_of_turn() {
    // Burn.use queues self-targeted DamageAction with DamageType.THORNS for
    // magicNumber 2 (4 upgraded). AbstractPlayer.damage applies Intangible,
    // then block, then Buffer; Buffer consumes only if positive damage remains.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BufferPower.java
    let mut ordinary = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("Dummy", 40, 40)],
        3,
    );
    force_player_turn(&mut ordinary);
    ordinary.state.hand = make_deck(&["Burn", "Burn+"]);
    let hp_before = ordinary.state.player.hp;
    end_turn(&mut ordinary);
    assert_eq!(ordinary.state.player.hp, hp_before - 6);
    assert_eq!(ordinary.state.player.status(crate::status_ids::sid::HP_LOSS_THIS_COMBAT), 2);

    let mut mitigated = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("Dummy", 40, 40)],
        3,
    );
    force_player_turn(&mut mitigated);
    mitigated.state.hand = make_deck(&["Burn", "Burn+"]);
    mitigated.state.player.block = 3;
    mitigated.state.player.set_status(crate::status_ids::sid::BUFFER, 1);
    let hp_before = mitigated.state.player.hp;
    end_turn(&mut mitigated);
    assert_eq!(mitigated.state.player.hp, hp_before);
    assert_eq!(mitigated.state.player.block, 0);
    assert_eq!(mitigated.state.player.status(crate::status_ids::sid::BUFFER), 0);
    assert_eq!(mitigated.state.player.status(crate::status_ids::sid::HP_LOSS_THIS_COMBAT), 0);
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

#[test]
fn ascenders_bane_is_unplayable_unupgradable_and_exhausts_as_ethereal() {
    // Source: cards/curses/AscendersBane.java sets cost to -2, sets
    // isEthereal, and implements neither use() nor upgrade().
    let registry = global_registry();
    let bane = registry.get("AscendersBane").expect("Ascender's Bane is registered");
    assert_eq!(bane.cost, -2);
    assert!(bane.runtime_traits().unplayable);
    assert!(bane.runtime_traits().ethereal);
    assert!(registry.get("AscendersBane+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Pyramid".to_string());
    engine.state.hand = make_deck(&["AscendersBane", "Strike"]);

    let bane_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "AscendersBane")
        .unwrap();
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx, .. } if *card_idx == bane_idx)
    }));

    end_turn(&mut engine);

    assert_eq!(exhaust_prefix_count(&engine, "AscendersBane"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 1);
}

#[test]
fn clumsy_is_unplayable_unupgradable_and_exhausts_as_ethereal() {
    // Source: cards/curses/Clumsy.java sets cost -2 and isEthereal, queues
    // ExhaustSpecificCardAction at end of turn, and leaves upgrade() empty.
    let registry = global_registry();
    let clumsy = registry.get("Clumsy").expect("Clumsy is registered");
    assert_eq!(clumsy.cost, -2);
    assert!(clumsy.runtime_traits().unplayable);
    assert!(clumsy.runtime_traits().ethereal);
    assert!(registry.get("Clumsy+").is_none());

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Pyramid".to_string());
    engine.state.hand = make_deck(&["Clumsy", "Strike"]);

    let clumsy_idx = engine
        .state
        .hand
        .iter()
        .position(|card| engine.card_registry.card_name(card.def_id) == "Clumsy")
        .unwrap();
    assert!(!engine.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx, .. } if *card_idx == clumsy_idx)
    }));

    end_turn(&mut engine);

    assert_eq!(exhaust_prefix_count(&engine, "Clumsy"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 1);
}
