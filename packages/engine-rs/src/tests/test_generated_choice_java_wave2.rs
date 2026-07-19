#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/JackOfAllTrades.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/InfernalBlade.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/Distraction.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java

use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::effects::declarative::GeneratedCardPool;
use crate::effects::interpreter::generated_card_pool;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, discard_prefix_count, enemy_no_intent, engine_with_state,
    exhaust_prefix_count, make_deck, make_deck_n, play_self,
};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

fn play_card(engine: &mut crate::engine::CombatEngine, card_id: &str) {
    assert!(play_self(engine, card_id), "expected {card_id} in hand");
}

const COLORLESS_CHOICES: &[&str] = &[
    "Apotheosis",
    "Bandage Up",
    "Bite",
    "Blind",
    "Chrysalis",
    "Dark Shackles",
    "Deep Breath",
    "Defend",
    "Discovery",
    "Dramatic Entrance",
    "Enlightenment",
    "Finesse",
    "Flash of Steel",
    "Forethought",
    "Ghostly",
    "Good Instincts",
    "HandOfGreed",
    "Impatience",
    "J.A.X.",
    "Jack Of All Trades",
    "Madness",
    "Magnetism",
    "Master of Strategy",
    "Mayhem",
    "Metamorphosis",
    "Mind Blast",
    "Panacea",
    "Panache",
    "PanicButton",
    "Purity",
    "RitualDagger",
    "Sadistic Nature",
    "Secret Technique",
    "Secret Weapon",
    "Strike",
    "Swift Strike",
    "The Bomb",
    "Thinking Ahead",
    "Transmutation",
    "Trip",
    "Violence",
];

#[test]
fn wish_gold_branch_credits_pending_run_gold() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Wish", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    play_card(&mut engine, "Wish");
    let choice = engine.choice.as_ref().expect("Wish should open a choice");
    let labels: Vec<&str> = choice
        .options
        .iter()
        .filter_map(|opt| match opt {
            ChoiceOption::Named(name) => Some(*name),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["Strength", "Gold", "Plated Armor"]);

    engine.execute_action(&Action::Choose(1));
    assert_eq!(engine.state.player.status(sid::STRENGTH), 0);
    assert_eq!(engine.state.pending_run_gold, 25);
}

#[test]
fn discovery_potions_open_choice_and_resolve_one_generated_copy() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Strike", "Defend", "Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();

    let cases = [
        ("AttackPotion", CardType::Attack),
        ("SkillPotion", CardType::Skill),
        ("PowerPotion", CardType::Power),
        ("ColorlessPotion", CardType::Attack),
    ];

    for (potion_id, expected_type) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("generated discovery should open a choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion_id} should resolve a single generated copy");
        assert_eq!(choice.options.len(), 3, "{potion_id} should present three discovery cards");

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.state.hand.len(), 1);
        assert!(engine.state.potions[0].is_empty());
        let generated = engine.state.hand[0];
        let generated_def = engine.card_registry.card_def_by_id(generated.def_id);
        if potion_id == "ColorlessPotion" {
            let generated_name = engine.card_registry.card_name(generated.def_id);
            assert!(
                COLORLESS_CHOICES.contains(&generated_name),
                "colorless generation should draw from the colorless pool"
            );
        } else {
            assert_eq!(generated_def.card_type, expected_type);
        }
    }
}

#[test]
fn jack_of_all_trades_uses_exact_colorless_pool_and_card_random_rng() {
    // JackOfAllTrades.java makes one cardRandomRng selection from
    // srcColorlessCardPool, excluding HEALING, and queues one base copy.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Jack Of All Trades", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let pool = generated_card_pool(&engine, GeneratedCardPool::Colorless);
    let mut oracle = engine.card_random_rng.clone();
    let expected = pool[oracle.random_int((pool.len() - 1) as i32) as usize];
    let general_before = engine.shuffle_rng.counter;
    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Jack Of All Trades");

    assert_eq!(engine.state.hand.len(), hand_before);
    let generated_name = engine.card_registry.card_name(engine.state.hand.last().unwrap().def_id);
    assert_eq!(generated_name, expected);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.shuffle_rng.counter, general_before);
    assert_eq!(exhaust_prefix_count(&engine, "Jack Of All Trades"), 1);
}

#[test]
fn infernal_blade_uses_card_random_attack_pool_and_zeroes_turn_cost() {
    // InfernalBlade.java selects exactly once through cardRandomRng from
    // returnTrulyRandomCardInCombat(ATTACK), then sets costForTurn to zero.
    // Upgrade changes only the played card's base cost from 1 to 0.
    // Java: cards/red/InfernalBlade.java and dungeons/AbstractDungeon.java.
    for (card_id, expected_energy) in [("Infernal Blade", 2), ("Infernal Blade+", 3)] {
        let mut engine = engine_with_state(combat_state_with(
            make_deck(&[card_id, "Strike", "Defend", "Strike", "Defend"]),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        ));
        let attack_pool = generated_card_pool(&engine, GeneratedCardPool::Attack);
        let mut oracle = engine.card_random_rng.clone();
        let expected = attack_pool[oracle.random_int((attack_pool.len() - 1) as i32) as usize];
        let general_before = engine.shuffle_rng.counter;
        let hand_before = engine.state.hand.len();

        play_card(&mut engine, card_id);

        assert_eq!(engine.state.hand.len(), hand_before);
        assert_eq!(engine.state.energy, expected_energy);
        assert_eq!(engine.card_random_rng.counter, oracle.counter);
        assert_eq!(engine.shuffle_rng.counter, general_before);
        assert_eq!(exhaust_prefix_count(&engine, "Infernal Blade"), 1);
        let generated = *engine.state.hand.last().unwrap();
        assert_eq!(engine.card_registry.card_name(generated.def_id), expected);
        assert_eq!(generated.cost, 0);
    }
}

#[test]
fn distraction_uses_random_skill_pool_and_zeroes_cost() {
    // Distraction.java selects exactly one current-class Skill, sets its
    // costForTurn to -99 (free), and exhausts. Upgrade changes cost 1 to 0.
    for (card_id, expected_energy) in [("Distraction", 2), ("Distraction+", 3)] {
        let mut engine = engine_with_state(combat_state_with(
            make_deck(&[card_id, "Strike", "Defend", "Strike", "Defend"]),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        ));

        let hand_before = engine.state.hand.len();
        let rng_before = engine.card_random_rng.counter;
        play_card(&mut engine, card_id);
        assert_eq!(engine.state.hand.len(), hand_before);
        assert_eq!(engine.state.energy, expected_energy);
        assert_eq!(engine.card_random_rng.counter, rng_before + 1);
        assert_eq!(exhaust_prefix_count(&engine, "Distraction"), 1);
        let generated = *engine.state.hand.last().unwrap();
        let generated_def = engine.card_registry.card_def_by_id(generated.def_id);
        assert_eq!(generated_def.card_type, CardType::Skill);
        assert_eq!(generated.cost, 0);
    }
}

#[test]
fn jack_of_all_trades_plus_generates_two_colorless_cards() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Jack Of All Trades+", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    // The two selections happen independently during use(), before their two
    // queued one-card MakeTempCardInHandActions. At nine remaining cards, the
    // first copy enters hand and the second spills to discard, but both rolls
    // consume cardRandomRng.
    engine.state.hand = make_deck_n("Defend", 9);
    engine.state.hand.push(engine.card_registry.make_card("Jack Of All Trades+"));
    engine.state.discard_pile.clear();
    let pool = generated_card_pool(&engine, GeneratedCardPool::Colorless);
    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<_> = (0..2)
        .map(|_| pool[oracle.random_int((pool.len() - 1) as i32) as usize])
        .collect();
    let general_before = engine.shuffle_rng.counter;

    play_card(&mut engine, "Jack Of All Trades+");

    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(discard_prefix_count(&engine, expected[1]), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.hand.last().unwrap().def_id),
        expected[0]
    );
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.shuffle_rng.counter, general_before);
    assert_eq!(exhaust_prefix_count(&engine, "Jack Of All Trades"), 1);
}

#[test]
fn chrysalis_needs_random_upgraded_skill_generation() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Chrysalis", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    play_card(&mut engine, "Chrysalis");
    assert_eq!(engine.state.draw_pile.len(), 3);
    assert!(engine.state.draw_pile.iter().all(|card| {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        def.card_type == CardType::Skill && card.cost <= 0
    }));
}

#[test]
fn metamorphosis_needs_random_upgraded_attack_generation() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Metamorphosis", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    play_card(&mut engine, "Metamorphosis");
    assert_eq!(engine.state.draw_pile.len(), 3);
    assert!(engine.state.draw_pile.iter().all(|card| {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        def.card_type == CardType::Attack && card.cost <= 0
    }));
}

#[test]
fn transmutation_uses_x_cost_random_generation_action() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation", "Strike", "Defend", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Transmutation");
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.hand.len(), hand_before + 2);
    let zero_cost_generated = engine
        .state
        .hand
        .iter()
        .filter(|card| {
            let name = engine.card_registry.card_name(card.def_id);
            COLORLESS_CHOICES.contains(&name) && card.cost == 0
        })
        .count();
    assert_eq!(zero_cost_generated, 3);
}
