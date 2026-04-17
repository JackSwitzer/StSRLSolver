#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java

use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::ChoiceOption;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with,
    enemy_no_intent,
    engine_with_state,
    make_deck,
    play_self,
};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
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

// Java parity note:
// - Discovery potions still need a shared "generate N copies" primitive for Sacred Bark.
// This file locks the honest base-path behavior we can already execute.

#[test]
fn wish_uses_named_choice_flow_for_its_java_menu() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Wish", "Strike_P", "Defend_P", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    assert!(play_self(&mut engine, "Wish"));
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);

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

    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.player.status(sid::STRENGTH), 3);
}

#[test]
fn discovery_potions_open_zero_cost_generated_choice_and_consume_the_slot() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();

    let cases = [
        ("AttackPotion", CardType::Attack, false),
        ("SkillPotion", CardType::Skill, false),
        ("PowerPotion", CardType::Power, false),
        ("ColorlessPotion", CardType::Attack, true),
    ];

    for (potion_id, expected_type, is_colorless) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert!(engine.state.potions[0].is_empty(), "{potion_id} should consume its slot");
        assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("potion should open a generated choice");
        assert_eq!(choice.options.len(), 3, "{potion_id} should offer three choices");
        let mut seen = std::collections::HashSet::new();
        for option in &choice.options {
            let crate::engine::ChoiceOption::GeneratedCard(card) = option else {
                panic!("{potion_id} should use generated-card options");
            };
            let generated_def = engine.card_registry.card_def_by_id(card.def_id);
            assert_eq!(card.cost, 0, "{potion_id} should set generated cards to 0 cost for the turn");
            if is_colorless {
                let generated_name = engine.card_registry.card_name(card.def_id);
                assert!(
                    COLORLESS_CHOICES.contains(&generated_name),
                    "{potion_id} should generate a colorless card, got {generated_name}"
                );
                assert!(seen.insert(generated_name));
            } else {
                assert_eq!(
                    generated_def.card_type,
                    expected_type,
                    "{potion_id} should generate the requested card family"
                );
                assert!(seen.insert(engine.card_registry.card_name(card.def_id)));
            }
        }
        engine.execute_action(&Action::Choose(0));
        assert_eq!(engine.state.hand.len(), 1, "{potion_id} should add the chosen card to hand");
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
                && record.def_id.is_some()
        }));
    }
}
