#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Regret.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/status/Void.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Parasite.java

use crate::cards::global_registry;
use crate::tests::support::*;

#[test]
fn support_wave1_registry_keeps_shared_support_cards_tag_driven() {
    let registry = global_registry();

    for card_id in [
        "Slimed", "Wound", "Daze", "Burn", "Burn+", "Void", "Decay", "Regret", "Doubt",
        "Shame", "AscendersBane", "Clumsy", "CurseOfTheBell", "Injury", "Necronomicurse",
        "Normality", "Pain", "Parasite", "Pride", "Writhe",
    ] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
        assert!(card.effect_data.is_empty(), "{card_id} should remain metadata-driven");
        assert!(card.complex_hook.is_none(), "{card_id} should stay non-bespoke");
    }

    assert!(registry.get("Burn").unwrap().effects.contains(&"end_turn_damage"));
    assert!(registry.get("Regret").unwrap().effects.contains(&"end_turn_regret"));
    assert!(registry.get("Doubt").unwrap().effects.contains(&"end_turn_weak"));
    assert!(registry.get("Shame").unwrap().effects.contains(&"end_turn_frail"));
    assert!(registry.get("Pain").unwrap().effects.contains(&"damage_on_draw"));
    assert!(registry.get("Parasite").unwrap().effects.contains(&"lose_max_hp_on_remove"));
    assert!(registry.get("Void").unwrap().effects.contains(&"lose_energy_on_draw"));
    assert!(registry.get("Daze").unwrap().effects.contains(&"ethereal"));
    assert!(registry.get("Wound").unwrap().effects.contains(&"unplayable"));
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
    engine.state.hand = make_deck(&["Pain", "Strike_R"]);

    let hp_before = engine.state.player.hp;
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
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
