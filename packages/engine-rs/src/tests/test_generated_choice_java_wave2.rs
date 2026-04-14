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
use crate::engine::ChoiceOption;
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
#[ignore = "Java Wish grants Gold through run-state plumbing; the current combat runtime cannot mutate run gold yet."]
fn wish_gold_branch_is_blocked_by_run_gold_plumbing() {
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
}

#[test]
#[ignore = "Java DiscoveryAction is potency-sensitive and returns N generated cards; the current runtime only exposes the base one-copy generated-choice shape."]
fn discovery_potions_need_potency_sensitive_generate_n_copies() {
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

        assert_eq!(engine.state.hand.len(), 1);
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
#[ignore = "Java Chrysalis generates random upgraded Skill cards into the draw pile; the current runtime still uses a fixed Deflect-style approximation."]
fn chrysalis_needs_random_upgraded_skill_generation() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Chrysalis", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let draw_before = engine.state.draw_pile.len();
    play_card(&mut engine, "Chrysalis");
    assert!(engine.state.draw_pile.len() >= draw_before);
}

#[test]
#[ignore = "Java Metamorphosis generates random upgraded Attack cards into the draw pile; the current runtime still uses a fixed Smite-style approximation."]
fn metamorphosis_needs_random_upgraded_attack_generation() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Metamorphosis", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let draw_before = engine.state.draw_pile.len();
    play_card(&mut engine, "Metamorphosis");
    assert!(engine.state.draw_pile.len() >= draw_before);
}

#[test]
#[ignore = "Java Transmutation converts X energy into random card generation; the current runtime still uses a fixed Smite-style approximation."]
fn transmutation_needs_x_cost_random_generation_action() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let hand_before = engine.state.hand.len();
    play_card(&mut engine, "Transmutation");
    assert!(engine.state.hand.len() >= hand_before);
}
