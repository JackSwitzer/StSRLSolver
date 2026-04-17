use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::effects::trigger::Trigger;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

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

#[test]
fn display_name_generation_potions_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();

    let cases = [
        ("Attack Potion", Some(CardType::Attack), None, "AttackPotion"),
        ("Skill Potion", Some(CardType::Skill), None, "SkillPotion"),
        ("Power Potion", Some(CardType::Power), None, "PowerPotion"),
        ("Colorless Potion", None, Some(COLORLESS_CHOICES), "ColorlessPotion"),
    ];

    for (potion_name, expected_type, colorless_pool, expected_def_id) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_name.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("generated potion choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion_name} should resolve one generated card");
        assert_eq!(choice.options.len(), 3, "{potion_name} should offer three choices");
        for option in &choice.options {
            let ChoiceOption::GeneratedCard(card) = option else {
                panic!("{potion_name} should use generated-card options");
            };
            if let Some(card_type) = expected_type {
                assert_eq!(
                    engine.card_registry.card_def_by_id(card.def_id).card_type,
                    card_type,
                    "{potion_name} should generate a {card_type:?} card through runtime activation"
                );
            }
            if let Some(pool) = colorless_pool {
                let generated_name = engine.card_registry.card_name(card.def_id);
                assert!(
                    pool.contains(&generated_name),
                    "{potion_name} should draw from the colorless pool, got {generated_name}"
                );
            }
            assert_eq!(card.cost, 0, "{potion_name} should zero generated card cost");
        }

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.state.hand.len(), 1, "{potion_name} should add the chosen card");
        assert!(engine.state.potions[0].is_empty(), "{potion_name} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == Trigger::ManualActivation && record.def_id == Some(expected_def_id)
        }));
    }
}

#[test]
fn display_name_generation_potions_stay_legal_without_targets() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    for potion_name in ["Attack Potion", "Skill Potion", "Power Potion", "Colorless Potion"] {
        engine.state.potions = vec![String::new(); 3];
        engine.state.hand.clear();
        engine.state.potions[1] = potion_name.to_string();

        let actions = engine.get_legal_actions();
        assert!(actions.iter().any(|action| {
            matches!(
                action,
                Action::UsePotion {
                    potion_idx,
                    target_idx: -1
                } if *potion_idx == 1
            )
        }), "{potion_name} should enumerate as a target-free potion action");
    }
}
