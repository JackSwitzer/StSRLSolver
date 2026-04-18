#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/RedMask.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/PhilosopherStone.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/BagOfPreparation.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/RingOfTheSnake.java

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

#[test]
fn relic_wave16_start_combat_buffs_and_temp_cards_match_canonical_runtime() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 12),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 44, 44)],
        3,
    );
    engine.state.relics = vec![
        "Oddly Smooth Stone".to_string(),
        "Red Mask".to_string(),
        "Philosopher's Stone".to_string(),
        "PureWater".to_string(),
    ];

    engine.start_combat();

    assert_eq!(engine.state.player.dexterity(), 1);
    assert!(engine.state.enemies.iter().all(|enemy| enemy.entity.is_weak()));
    assert!(engine.state.enemies.iter().all(|enemy| enemy.entity.strength() == 1));
    assert_eq!(
        engine
            .state
            .hand
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Miracle")
            .count(),
        1
    );
    assert_eq!(engine.state.player.status(sid::BAG_OF_PREP_DRAW), 0);
}

#[test]
fn relic_wave16_opening_draw_relics_match_canonical_runtime() {
    let mut bag = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    bag.state.relics.push("Bag of Preparation".to_string());
    bag.start_combat();
    assert_eq!(bag.state.hand.len(), 7);

    let mut ring = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    ring.state.relics.push("Ring of the Snake".to_string());
    ring.start_combat();
    assert_eq!(ring.state.hand.len(), 7);
}

#[test]
fn relic_wave16_runtime_mutagenic_strength_and_ninja_scroll_stay_authoritative() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics = vec!["MutagenicStrength".to_string(), "NinjaScroll".to_string()];

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::STRENGTH), 3);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 3);
    assert_eq!(
        engine
            .state
            .hand
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Shiv")
            .count(),
        3
    );
    assert_eq!(engine.state.discard_pile.iter().filter(|card| engine.card_registry.card_name(card.def_id) == "Shiv").count(), 0);
}
