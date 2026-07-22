#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ForeignInfluence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java

use crate::cards::{global_registry, CardType};
use crate::effects::declarative::{
    AmountSource as A, Effect as E, GeneratedCardPool, GeneratedCostRule,
};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self,
};

const JAVA_WATCHER_SKILL_POOL: &[&str] = &[
    "Prostrate",
    "Evaluate",
    "PathToVictory",
    "EmptyBody",
    "ClearTheMind",
    "Crescendo",
    "ThirdEye",
    "Protect",
    "Halt",
    "Pray",
    "EmptyMind",
    "Worship",
    "Swivel",
    "Perseverance",
    "Meditate",
    "WaveOfTheHand",
    "DeceiveReality",
    "InnerPeace",
    "Collect",
    "WreathOfFlame",
    "ForeignInfluence",
    "Indignation",
    "Sanctity",
    "Vengeance",
    "Judgement",
    "ConjureBlade",
    "Blasphemy",
    "Scrawl",
    "Vault",
    "Alpha",
    "Omniscience",
    "SpiritShield",
    "DeusExMachina",
];

const JAVA_WATCHER_ATTACK_POOL: &[&str] = &[
    "EmptyFist",
    "CrushJoints",
    "FollowUp",
    "CutThroughFate",
    "SashWhip",
    "FlurryOfBlows",
    "JustLucky",
    "FlyingSleeves",
    "BowlingBash",
    "Consecrate",
    "SignatureMove",
    "Weave",
    "Tantrum",
    "Conclude",
    "SandsOfTime",
    "FearNoEvil",
    "ReachHeaven",
    "Wallop",
    "CarveReality",
    "WindmillStrike",
    "TalkToTheHand",
    "WheelKick",
    "Brilliance",
    "Ragnarok",
];

#[test]
fn chrysalis_moves_to_typed_generate_to_draw_surface() {
    let registry = global_registry();
    let chrysalis = registry.get("Chrysalis").expect("Chrysalis should exist");
    let chrysalis_plus = registry.get("Chrysalis+").expect("Chrysalis+ should exist");

    assert_eq!(
        chrysalis.effect_data,
        &[E::GenerateRandomCardsToDraw {
            pool: GeneratedCardPool::Skill,
            count: A::Magic,
            cost_rule: GeneratedCostRule::ZeroIfPositiveThisTurn,
        }]
    );
    assert_eq!(chrysalis_plus.effect_data, chrysalis.effect_data);
    assert!(chrysalis.complex_hook.is_none());
    assert!(chrysalis_plus.complex_hook.is_none());
}

#[test]
fn chrysalis_and_metamorphosis_generate_zero_cost_cards_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        vec![],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        10,
    ));
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.hand.clear();
    engine
        .state
        .hand
        .push(engine.card_registry.make_card("Chrysalis"));
    engine
        .state
        .hand
        .push(engine.card_registry.make_card("Metamorphosis+"));

    assert!(play_self(&mut engine, "Chrysalis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Skill);
        assert!(
            card.cost <= 0,
            "Chrysalis should zero out positive-cost generated skills, got {}",
            card.cost
        );
    }

    engine.state.draw_pile.clear();
    assert!(play_self(&mut engine, "Metamorphosis+"));
    assert_eq!(engine.state.draw_pile.len(), 5);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Attack);
        assert!(
            card.cost <= 0,
            "Metamorphosis should zero out positive-cost generated attacks, got {}",
            card.cost
        );
    }
}

#[test]
fn chrysalis_plus_matches_watcher_pool_rng_placement_and_master_reality() {
    // Chrysalis selects five Skills with cardRandomRng before any queued
    // MakeTempCardInDrawPileAction resolves. Each action then inserts at a
    // random non-top position, and Master Reality upgrades the inserted copy.
    // returnTrulyRandomCardInCombat(SKILL) uses only current-character normal
    // rarity cards and excludes HEALING-tagged Wish.
    // Java: cards/colorless/Chrysalis.java, dungeons/AbstractDungeon.java,
    // actions/common/MakeTempCardInDrawPileAction.java, and cards/CardGroup.java.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine
        .state
        .hand
        .push(engine.card_registry.make_card("Chrysalis+"));
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Strike"]);
    engine.state.player.set_status(sid::MASTER_REALITY, 1);

    let general_before = engine.shuffle_rng.counter;
    let mut oracle = engine.card_random_rng.clone();
    let selected: Vec<&str> = (0..5)
        .map(|_| {
            let idx = oracle.random_int((JAVA_WATCHER_SKILL_POOL.len() - 1) as i32) as usize;
            JAVA_WATCHER_SKILL_POOL[idx]
        })
        .collect();
    let mut expected_draw: Vec<String> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    for id in &selected {
        let idx = oracle.random_int((expected_draw.len() - 1) as i32) as usize;
        expected_draw.insert(idx, format!("{id}+"));
    }

    assert!(play_self(&mut engine, "Chrysalis+"));

    let actual_draw: Vec<String> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    assert_eq!(actual_draw, expected_draw);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.shuffle_rng.counter, general_before);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    for card in &engine.state.draw_pile {
        let name = engine.card_registry.card_name(card.def_id);
        if name.ends_with('+') && selected.iter().any(|id| name == format!("{id}+")) {
            let base = engine
                .card_registry
                .get(name.trim_end_matches('+'))
                .expect("base card");
            if base.cost > 0 {
                assert_eq!(
                    card.cost, 0,
                    "positive-cost {name} should be zero this combat"
                );
            }
        }
    }
}

#[test]
fn metamorphosis_plus_matches_watcher_attack_rng_placement_and_master_reality() {
    // Metamorphosis.java selects all five Attacks through cardRandomRng before
    // its queued MakeTempCardInDrawPileActions resolve. Positive costs become
    // zero, each action inserts randomly, and Master Reality upgrades the copy.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine
        .state
        .hand
        .push(engine.card_registry.make_card("Metamorphosis+"));
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Strike"]);
    engine.state.player.set_status(sid::MASTER_REALITY, 1);

    let general_before = engine.shuffle_rng.counter;
    let mut oracle = engine.card_random_rng.clone();
    let selected: Vec<&str> = (0..5)
        .map(|_| {
            let idx = oracle.random_int((JAVA_WATCHER_ATTACK_POOL.len() - 1) as i32) as usize;
            JAVA_WATCHER_ATTACK_POOL[idx]
        })
        .collect();
    let mut expected_draw: Vec<String> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    for id in &selected {
        let idx = oracle.random_int((expected_draw.len() - 1) as i32) as usize;
        expected_draw.insert(idx, format!("{id}+"));
    }

    assert!(play_self(&mut engine, "Metamorphosis+"));

    let actual_draw: Vec<String> = engine
        .state
        .draw_pile
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect();
    assert_eq!(actual_draw, expected_draw);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.shuffle_rng.counter, general_before);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    for card in &engine.state.draw_pile {
        let name = engine.card_registry.card_name(card.def_id);
        if selected.iter().any(|id| name == format!("{id}+")) {
            let base = engine
                .card_registry
                .get(name.trim_end_matches('+'))
                .expect("base card");
            if base.cost > 0 {
                assert_eq!(
                    card.cost, 0,
                    "positive-cost {name} should be zero this combat"
                );
            }
        }
    }
}
