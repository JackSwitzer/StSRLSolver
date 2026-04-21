#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java

use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_on_enemy, play_self};

fn effective_cost(engine: &crate::engine::CombatEngine, card: crate::combat_types::CardInstance) -> i32 {
    if card.cost >= 0 {
        card.cost as i32
    } else {
        engine.card_registry.card_def_by_id(card.def_id).cost
    }
}

fn effective_misc_or(
    engine: &crate::engine::CombatEngine,
    card: crate::combat_types::CardInstance,
    base: i32,
) -> i32 {
    if card.misc >= 0 {
        card.misc as i32
    } else if base >= 0 {
        base
    } else {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        if def.base_damage >= 0 {
            def.base_damage
        } else {
            def.base_block
        }
    }
}

#[test]
fn streamline_reduces_the_played_instance_cost_only() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Streamline")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Streamline")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Streamline")];

    assert!(play_on_enemy(&mut engine, "Streamline", 0));

    let discard_last = engine.state.discard_pile.last().copied().expect("played card should discard");
    assert_eq!(effective_cost(&engine, discard_last), 1);
    assert_eq!(effective_cost(&engine, engine.state.draw_pile[0]), 2);
    assert_eq!(effective_cost(&engine, engine.state.discard_pile[0]), 2);
}

#[test]
fn rampage_only_scales_the_played_copy() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Rampage")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Rampage")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Rampage")];

    assert!(play_on_enemy(&mut engine, "Rampage", 0));

    let played = engine.state.discard_pile.last().copied().expect("played Rampage should discard");
    assert_eq!(effective_misc_or(&engine, played, 8), 13);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn steam_barrier_only_reduces_the_played_copy_block() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Steam")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Steam")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Steam")];

    assert!(play_self(&mut engine, "Steam"));

    let played = engine.state.discard_pile.last().copied().expect("played Steam Barrier should discard");
    assert_eq!(effective_misc_or(&engine, played, 6), 5);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn glass_knife_only_reduces_the_played_copy_damage() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    ));
    engine.state.hand = vec![engine.card_registry.make_card("Glass Knife")];
    engine.state.draw_pile = vec![engine.card_registry.make_card("Glass Knife")];
    engine.state.discard_pile = vec![engine.card_registry.make_card("Glass Knife")];

    assert!(play_on_enemy(&mut engine, "Glass Knife", 0));

    let played = engine.state.discard_pile.last().copied().expect("played Glass Knife should discard");
    assert_eq!(effective_misc_or(&engine, played, 8), 6);
    assert_eq!(engine.state.draw_pile[0].misc, -1);
    assert_eq!(engine.state.discard_pile[0].misc, -1);
}

#[test]
fn genetic_algorithm_and_ritual_dagger_only_scale_the_played_copy() {
    let mut genetic_engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    genetic_engine.state.hand = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];
    genetic_engine.state.draw_pile = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];
    genetic_engine.state.discard_pile = vec![genetic_engine.card_registry.make_card("Genetic Algorithm")];

    assert!(play_self(&mut genetic_engine, "Genetic Algorithm"));

    let played_genetic = genetic_engine
        .state
        .exhaust_pile
        .last()
        .copied()
        .expect("played Genetic Algorithm should exhaust");
    assert_eq!(effective_misc_or(&genetic_engine, played_genetic, 1), 3);
    assert_eq!(genetic_engine.state.draw_pile[0].misc, -1);
    assert_eq!(genetic_engine.state.discard_pile[0].misc, -1);

    let mut ritual_engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 15, 15)],
        3,
    ));
    ritual_engine.state.hand = vec![ritual_engine.card_registry.make_card("RitualDagger")];
    ritual_engine.state.draw_pile = vec![ritual_engine.card_registry.make_card("RitualDagger")];
    ritual_engine.state.discard_pile = vec![ritual_engine.card_registry.make_card("RitualDagger")];

    assert!(play_on_enemy(&mut ritual_engine, "RitualDagger", 0));

    let played_ritual = ritual_engine
        .state
        .exhaust_pile
        .last()
        .copied()
        .expect("played Ritual Dagger should exhaust");
    assert_eq!(effective_misc_or(&ritual_engine, played_ritual, 15), 18);
    assert_eq!(ritual_engine.state.draw_pile[0].misc, -1);
    assert_eq!(ritual_engine.state.discard_pile[0].misc, -1);
}
