use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

fn equip_potion(engine: &mut crate::engine::CombatEngine, slot: usize, potion_id: &str) {
    crate::potions::equip_potion_slot(engine, slot, potion_id);
}

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<&str> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect()
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
fn declarative_potions_drop_hooks_and_apply_runtime_effects() {
    assert!(crate::potions::defs::potion_def_by_id("StancePotion")
        .unwrap()
        .complex_hook
        .is_none());
    assert!(crate::potions::defs::potion_def_by_id("PotionOfCapacity")
        .unwrap()
        .complex_hook
        .is_none());
    assert!(crate::potions::defs::potion_def_by_id("BottledMiracle")
        .unwrap()
        .complex_hook
        .is_none());
    assert!(crate::potions::defs::potion_def_by_id("CunningPotion")
        .unwrap()
        .complex_hook
        .is_none());
    assert!(crate::potions::defs::potion_def_by_id("Ambrosia")
        .unwrap()
        .complex_hook
        .is_some());

    let mut ambrosia = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    ambrosia.state.relics.push("SacredBark".to_string());
    ambrosia.state.potions = vec![String::new(); 3];
    ambrosia.state.stance = Stance::Calm;
    equip_potion(&mut ambrosia, 0, "Ambrosia");
    use_potion(&mut ambrosia, 0, -1);
    assert_eq!(ambrosia.state.stance, Stance::Divinity);
    assert!(ambrosia.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("Ambrosia")
    }));

    let mut capacity = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    capacity.state.relics.push("SacredBark".to_string());
    capacity.state.potions = vec![String::new(); 3];
    equip_potion(&mut capacity, 0, "PotionOfCapacity");
    let orb_slots_before = capacity.state.player.status(sid::ORB_SLOTS);
    use_potion(&mut capacity, 0, -1);
    assert_eq!(capacity.state.player.status(sid::ORB_SLOTS), orb_slots_before + 4);

    let mut miracle = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    miracle.state.potions = vec![String::new(); 3];
    miracle.state.hand.clear();
    equip_potion(&mut miracle, 0, "BottledMiracle");
    use_potion(&mut miracle, 0, -1);
    assert_eq!(hand_names(&miracle), vec!["Miracle", "Miracle"]);

    let mut cunning = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    cunning.state.potions = vec![String::new(); 3];
    cunning.state.hand.clear();
    equip_potion(&mut cunning, 0, "CunningPotion");
    use_potion(&mut cunning, 0, -1);
    assert_eq!(hand_names(&cunning), vec!["Shiv", "Shiv", "Shiv"]);
}

#[test]
fn random_generation_potions_pick_the_right_card_families() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases = [
        ("AttackPotion", CardType::Attack),
        ("SkillPotion", CardType::Skill),
        ("PowerPotion", CardType::Power),
    ];

    for (potion_id, expected_type) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        equip_potion(&mut engine, 0, potion_id);

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine.choice.as_ref().expect("generated potion choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion_id} should resolve one generated copy");
        assert_eq!(choice.options.len(), 3, "{potion_id} should present three generated options");
        for option in &choice.options {
            let ChoiceOption::GeneratedCard(card) = option else {
                panic!("{potion_id} should open generated-card options");
            };
            assert_eq!(
                engine.card_registry.card_def_by_id(card.def_id).card_type,
                expected_type,
                "{potion_id} should generate one card of the requested type"
            );
            assert_eq!(card.cost, 0, "{potion_id} should zero generated card cost this turn");
        }

        engine.execute_action(&Action::Choose(0));
        let generated = engine
            .state
            .hand
            .iter()
            .map(|card| engine.card_registry.card_def_by_id(card.def_id).card_type)
            .collect::<Vec<_>>();
        assert_eq!(generated, vec![expected_type], "{potion_id} should add the chosen card to hand");
    }

    engine.state.hand.clear();
    engine.state.potions = vec![String::new(); 3];
    equip_potion(&mut engine, 0, "ColorlessPotion");
    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("colorless potion choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    for option in &choice.options {
        let ChoiceOption::GeneratedCard(card) = option else {
            panic!("Colorless Potion should use generated-card options");
        };
        let generated_name = engine.card_registry.card_name(card.def_id);
        assert!(
            COLORLESS_CHOICES.contains(&generated_name),
            "Colorless Potion should generate a colorless card, got {}",
            generated_name
        );
        assert_eq!(card.cost, 0);
    }

    engine.execute_action(&Action::Choose(0));
    let hand = hand_names(&engine);
    assert_eq!(hand.len(), 1);
    assert!(COLORLESS_CHOICES.contains(&hand[0]));
}
