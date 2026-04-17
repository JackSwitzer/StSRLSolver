#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java

use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck, play_self};

const COLORLESS_CHOICES: &[&str] = &[
    "Apotheosis", "Bandage Up", "Bite", "Blind", "Chrysalis", "Dark Shackles", "Deep Breath",
    "Defend_R", "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse", "Flash of Steel",
    "Forethought", "Ghostly", "Good Instincts", "HandOfGreed", "Impatience", "J.A.X.",
    "Jack Of All Trades", "Madness", "Magnetism", "Master of Strategy", "Mayhem",
    "Metamorphosis", "Mind Blast", "Panacea", "Panache", "PanicButton", "Purity",
    "RitualDagger", "Sadistic Nature", "Secret Technique", "Secret Weapon", "Strike_R",
    "Swift Strike", "The Bomb", "Thinking Ahead", "Transmutation", "Trip", "Violence",
];

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

#[test]
fn chrysalis_generates_zero_cost_skills_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Chrysalis"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Chrysalis"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Chrysalis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Skill);
        assert!(card.cost <= 0, "generated Chrysalis cards should be free this turn");
    }
}

#[test]
fn metamorphosis_generates_zero_cost_attacks_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Metamorphosis"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Metamorphosis"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    assert!(play_self(&mut engine, "Metamorphosis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Attack);
        assert!(card.cost <= 0, "generated Metamorphosis cards should be free this turn");
    }
}

#[test]
fn transmutation_generates_x_zero_cost_colorless_cards_to_hand() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Transmutation"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 3;

    assert!(play_self(&mut engine, "Transmutation"));
    assert_eq!(engine.state.hand.len(), 3);
    for card in &engine.state.hand {
        let name = engine.card_registry.card_name(card.def_id);
        assert!(COLORLESS_CHOICES.contains(&name));
        assert_eq!(card.cost, 0);
    }
}

#[test]
fn transmutation_plus_upgrades_generated_cards() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Transmutation+"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        2,
    ));
    engine.state.hand = make_deck(&["Transmutation+"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.energy = 2;

    assert!(play_self(&mut engine, "Transmutation+"));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.is_upgraded()));
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}

#[test]
fn discovery_potions_open_java_style_choice_and_track_copy_count() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();

    let cases = [
        ("AttackPotion", Some(CardType::Attack), false),
        ("SkillPotion", Some(CardType::Skill), false),
        ("PowerPotion", Some(CardType::Power), false),
        ("ColorlessPotion", None, true),
    ];

    for (potion, expected_type, expect_colorless) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion.to_string();

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("potion should open a discovery choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion} should default to one generated copy");
        assert_eq!(choice.options.len(), 3);
        for option in &choice.options {
            let ChoiceOption::GeneratedCard(card) = option else {
                panic!("{potion} should offer generated cards");
            };
            if let Some(card_type) = expected_type {
                assert_eq!(engine.card_registry.card_def_by_id(card.def_id).card_type, card_type);
            }
            if expect_colorless {
                let name = engine.card_registry.card_name(card.def_id);
                assert!(COLORLESS_CHOICES.contains(&name));
            }
            assert_eq!(card.cost, 0);
        }

        engine.execute_action(&Action::Choose(0));
        assert_eq!(engine.state.hand.len(), 1);
        assert!(engine.state.potions[0].is_empty());
    }
}

#[test]
fn master_reality_upgrades_resolved_generated_discovery_card() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.player.set_status(sid::MASTER_REALITY, 1);
    engine.state.potions[0] = "AttackPotion".to_string();

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Attack Potion should open a choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);

    engine.execute_action(&Action::Choose(0));

    assert_eq!(engine.state.hand.len(), 1);
    assert!(
        engine.state.hand[0].is_upgraded(),
        "Master Reality should upgrade the resolved generated card copy"
    );
}

#[test]
fn sacred_bark_discovery_choice_needs_copy_count_resolution() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "SkillPotion".to_string();

    use_potion(&mut engine, 0, -1);

    let choice = engine.choice.as_ref().expect("Skill Potion should open a choice");
    assert_eq!(choice.aux_count, 2);
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert!(engine.state.hand.iter().all(|card| card.cost == 0));
}
