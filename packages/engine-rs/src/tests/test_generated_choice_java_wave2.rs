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
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self,
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
    "Defend_R",
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
    "Strike_R",
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
        make_deck(&["Wish", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
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
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
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
fn jack_of_all_trades_uses_random_colorless_pool() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Jack Of All Trades", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Jack Of All Trades");
    assert_eq!(engine.state.hand.len(), hand_before);
    let generated_name = engine.card_registry.card_name(engine.state.hand.last().unwrap().def_id);
    assert!(
        COLORLESS_CHOICES.contains(&generated_name),
        "Jack Of All Trades should use the Java colorless pool"
    );
}

#[test]
fn infernal_blade_uses_random_attack_pool_and_zeroes_cost() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Infernal Blade", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Infernal Blade");
    assert_eq!(engine.state.hand.len(), hand_before);
    let generated = *engine.state.hand.last().unwrap();
    let generated_def = engine.card_registry.card_def_by_id(generated.def_id);
    assert_eq!(generated_def.card_type, CardType::Attack);
    assert_eq!(generated.cost, 0);
}

#[test]
fn distraction_uses_random_skill_pool_and_zeroes_cost() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Distraction", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Distraction");
    assert_eq!(engine.state.hand.len(), hand_before);
    let generated = *engine.state.hand.last().unwrap();
    let generated_def = engine.card_registry.card_def_by_id(generated.def_id);
    assert_eq!(generated_def.card_type, CardType::Skill);
    assert_eq!(generated.cost, 0);
}

#[test]
fn jack_of_all_trades_plus_generates_two_colorless_cards() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Jack Of All Trades+", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Jack Of All Trades+");
    assert_eq!(engine.state.hand.len(), hand_before + 1);
    for card in &engine.state.hand[hand_before - 1..] {
        let generated_name = engine.card_registry.card_name(card.def_id);
        assert!(COLORLESS_CHOICES.contains(&generated_name));
    }
}

#[test]
fn chrysalis_needs_random_upgraded_skill_generation() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Chrysalis", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
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
        make_deck(&["Metamorphosis", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
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
        make_deck(&["Transmutation", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
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
