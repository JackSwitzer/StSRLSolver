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
fn generation_potions_use_engine_action_path_and_consume_slot() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SkillPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/PowerPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/ColorlessPotion.java
    // - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
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
        let slot = 0;
        engine.state.potions = vec![String::new(); 3];
        engine.state.hand.clear();
        engine.state.potions[slot] = potion.to_string();

        let actions = engine.get_legal_actions();
        assert!(actions.iter().any(|action| {
            matches!(
                action,
                Action::UsePotion {
                    potion_idx,
                    target_idx: -1
                } if *potion_idx == slot
            )
        }));

        use_potion(&mut engine, slot, -1);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        let choice = engine
            .choice
            .as_ref()
            .expect("generation potion should open a discover choice");
        assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
        assert_eq!(choice.aux_count, 1, "{potion} should default to one chosen copy");
        assert_eq!(choice.options.len(), 3, "{potion} should offer three choices");
        for option in &choice.options {
            let generated = match option {
                ChoiceOption::GeneratedCard(card) => *card,
                other => panic!("{potion} should offer generated-card options, got {other:?}"),
            };
            if let Some(card_type) = expected_type {
                assert_eq!(
                    engine.card_registry.card_def_by_id(generated.def_id).card_type,
                    card_type,
                    "{potion} should offer a {card_type:?} card via the action path"
                );
            }
            if expect_colorless {
                let generated_name = engine.card_registry.card_name(generated.def_id);
                assert!(
                    COLORLESS_CHOICES.contains(&generated_name),
                    "{potion} should offer a colorless card via the action path, got {generated_name}"
                );
            }
            assert_eq!(
                generated.cost, 0,
                "{potion} generated choices should be zero-cost this turn"
            );
        }

        engine.execute_action(&Action::Choose(0));

        assert_eq!(engine.state.hand.len(), 1, "{potion} should resolve one chosen card");
        assert!(
            engine.state.potions[slot].is_empty(),
            "{potion} should consume its slot"
        );
    }
}

#[test]
fn generation_potions_emit_runtime_manual_activation_records() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases = [
        ("AttackPotion", "AttackPotion"),
        ("SkillPotion", "SkillPotion"),
        ("PowerPotion", "PowerPotion"),
        ("ColorlessPotion", "ColorlessPotion"),
    ];

    for (potion_id, def_id) in cases {
        engine.state.hand.clear();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
                && record.def_id == Some(def_id)
        }));
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::OnPotionUsed
                && record.potion_slot == 0
        }));
    }
}

#[test]
fn distilled_chaos_moves_top_draw_cards_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_B", "Defend_B", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(3);
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap"]);
    engine.state.potions[0] = "DistilledChaos".to_string();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.draw_pile.len(), 0);
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 34);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn liquid_memories_returns_discard_cards_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.discard_pile = make_deck(&["Strike_P", "Bash", "Shrug It Off"]);
    engine.state.potions[0] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Shrug It Off"]);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn entropic_brew_fills_other_empty_slots_and_then_consumes_itself() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "EntropicBrew".to_string();
    engine.state.potions[2] = "Fire Potion".to_string();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.potions[0].is_empty(), "used slot should be consumed after resolution");
    assert_eq!(engine.state.potions[1], "Block Potion");
    assert_eq!(engine.state.potions[2], "Fire Potion");
}

#[test]
fn elixir_uses_runtime_action_path_and_exhausts_hand() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike_P", "Defend_P", "Bash"]);
    engine.state.exhaust_pile.clear();
    engine.state.potions[0] = "Elixir".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.exhaust_pile.len(), 3);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("Elixir")
    }));
}

#[test]
fn blessing_of_the_forge_upgrades_hand_via_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike_P", "Defend_P"]);
    engine.state.potions[0] = "BlessingOfTheForge".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Strike_P+", "Defend_P+"]);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn bottled_miracle_and_cunning_potion_use_runtime_hooks() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.potions[0] = "BottledMiracle".to_string();
    engine.state.potions[1] = "CunningPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(hand_names(&engine), vec!["Miracle", "Miracle"]);

    use_potion(&mut engine, 1, -1);
    assert_eq!(
        hand_names(&engine),
        vec!["Miracle", "Miracle", "Shiv", "Shiv", "Shiv"]
    );
}

#[test]
fn bottled_miracle_and_cunning_potion_respect_sacred_bark_and_hand_limit_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand = make_deck(&[
        "Strike_P",
        "Defend_P",
        "Bash",
        "Zap",
        "Dualcast",
        "Inflame",
        "Shrug It Off",
        "Defend_P",
    ]);
    engine.state.potions[0] = "BottledMiracle".to_string();
    engine.state.potions[1] = "CunningPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.hand.len(), 10);
    assert_eq!(
        hand_names(&engine)[8..],
        ["Miracle", "Miracle"]
    );

    use_potion(&mut engine, 1, -1);
    assert_eq!(engine.state.hand.len(), 10);
    let shiv_count = hand_names(&engine)
        .into_iter()
        .filter(|name| *name == "Shiv")
        .count();
    assert_eq!(shiv_count, 0, "full hand should block extra Shiv generation");
}

#[test]
fn gamblers_brew_discards_then_redraws_through_engine_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off", "Inflame", "Zap", "Dualcast"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike_P", "Defend_P", "Bash"]);
    engine.state.draw_pile = make_deck(&["Shrug It Off", "Inflame", "Zap", "Dualcast"]);
    engine.state.discard_pile.clear();
    engine.state.potions[0] = "GamblersBrew".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(hand_names(&engine), vec!["Dualcast", "Zap", "Inflame"]);
    assert_eq!(engine.state.discard_pile.len(), 3);
    assert_eq!(engine.state.player.status(sid::POTION_DRAW), 0);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn snecko_oil_runtime_action_path_draws_and_applies_confusion() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off", "Inflame", "Zap", "Dualcast"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off"]);
    engine.state.potions[0] = "SneckoOil".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.player.status(sid::CONFUSION), 1);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("SneckoOil")
    }));
}

#[test]
fn temporary_effect_potions_apply_statuses_through_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases: [(&str, crate::ids::StatusId, i32, Option<crate::ids::StatusId>); 4] = [
        ("DuplicationPotion", sid::DUPLICATION, 1, None),
        ("GhostInAJar", sid::INTANGIBLE, 1, None),
        ("SteroidPotion", sid::STRENGTH, 5, Some(sid::LOSE_STRENGTH)),
        ("SpeedPotion", sid::DEXTERITY, 5, Some(sid::LOSE_DEXTERITY)),
    ];

    for (potion_id, primary_status, primary_amount, trailing_status) in cases {
        engine.state.player.set_status(sid::DUPLICATION, 0);
        engine.state.player.set_status(sid::INTANGIBLE, 0);
        engine.state.player.set_status(sid::STRENGTH, 0);
        engine.state.player.set_status(sid::DEXTERITY, 0);
        engine.state.player.set_status(sid::LOSE_STRENGTH, 0);
        engine.state.player.set_status(sid::LOSE_DEXTERITY, 0);
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 0, -1);

        assert_eq!(engine.state.player.status(primary_status), primary_amount, "{potion_id} primary status");
        if let Some(trailing_status) = trailing_status {
            assert_eq!(engine.state.player.status(trailing_status), primary_amount, "{potion_id} trailing status");
        }
        assert!(engine.state.potions[0].is_empty(), "{potion_id} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
        }));
    }
}

#[test]
fn stance_potion_opens_choose_one_and_sets_stance_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.stance = Stance::Calm;
    engine.state.potions[0] = "StancePotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Stance Potion should open a choice");
    let labels: Vec<&str> = choice
        .options
        .iter()
        .filter_map(|opt| match opt {
            crate::engine::ChoiceOption::Named(label) => Some(*label),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["Wrath", "Calm"]);
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert!(engine.state.potions[0].is_empty());

    engine.state.potions[0] = "StancePotion".to_string();
    use_potion(&mut engine, 0, -1);
    engine.execute_action(&Action::Choose(1));
    assert_eq!(engine.state.stance, Stance::Calm);
}

#[test]
fn ambrosia_essence_of_darkness_and_capacity_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(2);
    engine.state.stance = Stance::Neutral;

    engine.state.potions[0] = "Ambrosia".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.stance, Stance::Divinity);

    engine.state.potions[0] = "EssenceOfDarkness".to_string();
    use_potion(&mut engine, 0, -1);
    assert!(engine
        .state
        .orb_slots
        .slots
        .iter()
        .all(|orb| orb.orb_type == crate::orbs::OrbType::Dark));

    engine.state.potions[0] = "PotionOfCapacity".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 2);
}

#[test]
fn smoke_bomb_uses_runtime_action_path_and_consumes_its_slot() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.combat_over);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == crate::effects::trigger::Trigger::ManualActivation
            && record.def_id == Some("SmokeBomb")
    }));
}
