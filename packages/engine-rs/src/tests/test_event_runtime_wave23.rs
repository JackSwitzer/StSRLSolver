use crate::checkpoint::CoreCheckpoint;
use crate::events::{
    typed_events_for_act, typed_shrine_events, DeadAdventurerReward, EventProgramOp, TypedEventDef,
};
use crate::run::{GameAction, RunEngine, RunPhase};
use crate::seed::{java_util_shuffle, StsRandom};

fn act_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing event {name}"))
}

fn shrine_event(name: &str) -> TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing shrine {name}"))
}

fn misc_counter(engine: &RunEngine) -> i64 {
    engine.rng_counters()["misc"]
}

fn dead_adventurer_page_contract(
    event: &TypedEventDef,
) -> (DeadAdventurerReward, usize, String, &TypedEventDef) {
    let EventProgramOp::RandomOutcomeTable { outcomes } = &event.options[0].program.ops[0] else {
        panic!("expected Dead Adventurer risk table");
    };
    let fight_count = outcomes
        .iter()
        .filter(|program| {
            matches!(
                program.ops.first(),
                Some(EventProgramOp::PrepareCombatBranch { .. })
            )
        })
        .count();
    let enemy = outcomes
        .iter()
        .find_map(|program| match program.ops.first() {
            Some(EventProgramOp::PrepareCombatBranch { enemies, .. }) => enemies.first().cloned(),
            _ => None,
        })
        .expect("expected frozen elite identity");
    let reward_program = outcomes
        .iter()
        .find(|program| {
            !matches!(
                program.ops.first(),
                Some(EventProgramOp::PrepareCombatBranch { .. })
            )
        })
        .expect("expected successful search outcome");
    let reward = match reward_program.ops.first() {
        Some(EventProgramOp::AdjustGold { amount: 30 }) => DeadAdventurerReward::Gold,
        Some(EventProgramOp::Nothing) => DeadAdventurerReward::Nothing,
        Some(EventProgramOp::ObtainRandomScreenlessRelic) => DeadAdventurerReward::Relic,
        other => panic!("unexpected Dead Adventurer reward op: {other:?}"),
    };
    let next = reward_program
        .ops
        .iter()
        .find_map(|op| match op {
            EventProgramOp::ContinueEvent { event } => Some(event.as_ref()),
            _ => None,
        })
        .expect("expected next Dead Adventurer page");
    (reward, fight_count, enemy, next)
}

#[test]
fn dead_adventurer_constructor_and_recursive_risk_pages_match_java() {
    // The constructor shuffles GOLD/NOTHING/RELIC with one misc randomLong,
    // then chooses Sentries/Nob/Lagavulin with one misc random(2). The exact
    // order and elite persist through the 25/50/75 percent search loop.
    // Java: events/exordium/DeadAdventurer.java::<init>/buttonEffect.
    let mut engine = RunEngine::new(797, 0);
    let (seed0, seed1, counter) = engine.debug_floor_rng_states()[4];
    let mut oracle = StsRandom::from_state(seed0, seed1, counter);
    let mut expected_rewards = [
        DeadAdventurerReward::Gold,
        DeadAdventurerReward::Nothing,
        DeadAdventurerReward::Relic,
    ];
    let shuffle_seed = oracle.random_long_unbounded();
    java_util_shuffle(&mut expected_rewards, shuffle_seed);
    let expected_enemy =
        ["3 Sentries", "Gremlin Nob", "Lagavulin Event"][oracle.random_int(2) as usize];

    engine.debug_set_typed_event_state(act_event(1, "Dead Adventurer"));
    assert_eq!(engine.debug_floor_rng_states()[4], oracle.state_tuple());

    let mut page = engine.debug_current_event().unwrap();
    for (index, expected_risk) in [25, 50, 75].into_iter().enumerate() {
        let (reward, fight_count, enemy, next) = dead_adventurer_page_contract(&page);
        assert_eq!(reward, expected_rewards[index]);
        assert_eq!(fight_count, expected_risk);
        assert_eq!(enemy, expected_enemy);
        assert!(page.options[0]
            .text
            .contains(&format!("{expected_risk}% elite fight")));
        page = next.clone();
    }
    assert_eq!(page.options.len(), 1);
    assert_eq!(page.options[0].text, "Leave");
}

#[test]
fn dead_adventurer_failed_search_returns_fight_decision_with_frozen_rewards() {
    // A failed Search first spends the risk roll and 25..35 gold roll, then
    // returns the same event as an explicit Fight decision. Relic identity is
    // fixed when Fight starts, so restoring that decision cannot diverge.
    // Java: events/exordium/DeadAdventurer.java::buttonEffect.
    let seed = (0..10_000_u64)
        .find(|seed| {
            let run = RunEngine::new(*seed, 0);
            let (seed0, seed1, counter) = run.debug_floor_rng_states()[4];
            let mut oracle = StsRandom::from_state(seed0, seed1, counter);
            let _shuffle_seed = oracle.random_long_unbounded();
            let _enemy = oracle.random_int(2);
            oracle.random_int(99) < 25
        })
        .expect("expected a seed whose first search finds the elite");
    let mut original = RunEngine::new(seed, 0);
    let misc_before = misc_counter(&original);
    original.debug_set_typed_event_state(act_event(1, "Dead Adventurer"));
    let relic_before = original.rng_counters()["relic"];

    assert!(original.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(original.current_phase(), RunPhase::Event);
    assert_eq!(misc_counter(&original), misc_before + 4);
    assert_eq!(original.rng_counters()["relic"], relic_before);
    assert_eq!(
        original.get_legal_actions(),
        vec![GameAction::EventChoice(0)]
    );
    assert_eq!(
        original.debug_current_event().unwrap().options[0].text,
        "Fight"
    );

    let checkpoint = CoreCheckpoint::capture(&original).unwrap();
    let mut restored = checkpoint.restore().unwrap();
    for run in [&mut original, &mut restored] {
        assert!(run.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(run.current_phase(), RunPhase::Combat);
        assert!(run.rng_counters()["relic"] > relic_before);
        let post_start_rng = run.rng_state_tuples();
        run.debug_force_current_combat_outcome(true);
        run.debug_resolve_current_combat_outcome();
        assert_eq!(run.current_phase(), RunPhase::CardReward);
        assert_eq!(run.rng_state_tuples(), post_start_rng);
        assert!(run
            .current_reward_screen()
            .unwrap()
            .items
            .iter()
            .all(|item| item.label != "random relic"));
    }
    assert_eq!(
        CoreCheckpoint::capture(&restored).unwrap(),
        CoreCheckpoint::capture(&original).unwrap()
    );
}

#[test]
fn world_of_goop_rolls_and_persists_the_constructor_gold_loss() {
    // GoopPuddle.goldLoss consumes miscRng.random(20, 50) at A0 when the
    // event is constructed, caps the result by current gold, and displays the
    // same persisted amount that the leave action removes.
    // Java: events/exordium/GoopPuddle.java::<init>/buttonEffect.
    let mut engine = RunEngine::new(811, 0);
    engine.run_state.gold = 41;
    let before = misc_counter(&engine);
    engine.debug_set_typed_event_state(act_event(1, "World of Goop"));
    assert_eq!(misc_counter(&engine), before + 1);

    let text = engine.debug_current_event().unwrap().options[1]
        .text
        .clone();
    let loss = text
        .split_whitespace()
        .nth(2)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert!((20..=41).contains(&loss));
    assert!(engine.step_game(&GameAction::EventChoice(1)).accepted());
    assert_eq!(engine.run_state.gold, 41 - loss);
}

#[test]
fn shining_light_uses_java_rounding_and_one_seeded_upgrade_shuffle() {
    // ShiningLight rounds 20% max HP at A0, consumes one misc randomLong to
    // seed java.util.Random, and upgrades the first two shuffled eligible
    // master-deck instances.
    // Java: events/exordium/ShiningLight.java::upgradeCards.
    let mut engine = RunEngine::new(823, 0);
    engine.run_state.deck = vec![
        "Eruption".to_string(),
        "Vigilance".to_string(),
        "ThirdEye".to_string(),
        "DevaForm".to_string(),
    ];
    engine.run_state.current_hp = 72;
    engine.run_state.max_hp = 72;
    let before = misc_counter(&engine);
    engine.debug_set_typed_event_state(act_event(1, "Shining Light"));
    assert_eq!(misc_counter(&engine), before);
    assert!(engine.debug_current_event().unwrap().options[0]
        .text
        .contains("14 damage"));

    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(engine.run_state.current_hp, 58);
    assert_eq!(misc_counter(&engine), before + 1);
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.ends_with('+'))
            .count(),
        2
    );
}

#[test]
fn mausoleum_always_consumes_boolean_and_only_forces_writhe_at_a15() {
    // TheMausoleum rolls miscRng.randomBoolean before the A15 override. A0
    // therefore has both outcomes; A15 always obtains Writhe but still spends
    // exactly one wrapper call.
    // Java: events/city/TheMausoleum.java::buttonEffect.
    let mut saw_curse = false;
    let mut saw_clean = false;
    for seed in 0..32 {
        let mut engine = RunEngine::new(seed, 0);
        let before = misc_counter(&engine);
        engine.debug_set_typed_event_state(act_event(2, "The Mausoleum"));
        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(misc_counter(&engine), before + 1);
        let cursed = engine.run_state.deck.iter().any(|card| card == "Writhe");
        saw_curse |= cursed;
        saw_clean |= !cursed;
    }
    assert!(saw_curse && saw_clean);

    let mut asc15 = RunEngine::new(829, 15);
    let before = misc_counter(&asc15);
    asc15.debug_set_typed_event_state(act_event(2, "The Mausoleum"));
    assert!(asc15.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(misc_counter(&asc15), before + 1);
    assert!(asc15.run_state.deck.iter().any(|card| card == "Writhe"));
}

#[test]
fn fountain_removes_every_removable_curse_without_rng_or_deck_size_cap() {
    // Fountain iterates the master deck in reverse and removes every curse
    // except the three unpurgeable identities. It can reduce a small deck and
    // never samples a random card.
    // Java: events/shrines/FountainOfCurseRemoval.java::buttonEffect.
    let mut engine = RunEngine::new(839, 0);
    engine.run_state.deck = vec![
        "Regret".to_string(),
        "Eruption".to_string(),
        "Doubt".to_string(),
        "AscendersBane".to_string(),
        "CurseOfTheBell".to_string(),
        "Necronomicurse".to_string(),
    ];
    let before = engine.rng_state_tuples();
    engine.debug_set_typed_event_state(shrine_event("Fountain of Cleansing"));
    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(
        engine.run_state.deck,
        vec![
            "Eruption",
            "AscendersBane",
            "CurseOfTheBell",
            "Necronomicurse"
        ]
    );
    assert_eq!(engine.rng_state_tuples(), before);
}

#[test]
fn falling_persists_constructor_selected_card_instances_through_checkpoint() {
    // Falling.setCards consumes misc draws in ATTACK, SKILL, POWER order and
    // persists those exact instances. The dialog order is SKILL, POWER,
    // ATTACK, and choosing one removes that instance without another RNG draw.
    // Java: events/beyond/Falling.java and helpers/CardHelper.java.
    let mut original = RunEngine::new(853, 0);
    original.run_state.deck = vec![
        "Eruption".to_string(),
        "Vigilance".to_string(),
        "DevaForm".to_string(),
    ];
    let before = misc_counter(&original);
    original.debug_set_typed_event_state(act_event(3, "Falling"));
    assert_eq!(misc_counter(&original), before + 3);
    assert!(original.step_game(&GameAction::EventChoice(0)).accepted());
    let choice = original.debug_current_event().unwrap();
    assert_eq!(choice.options[0].text, "Lose skill: Vigilance");
    assert_eq!(choice.options[1].text, "Lose power: DevaForm");
    assert_eq!(choice.options[2].text, "Lose attack: Eruption");

    let checkpoint = CoreCheckpoint::capture(&original).unwrap();
    let mut restored = checkpoint.restore().unwrap();
    for engine in [&mut original, &mut restored] {
        assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
        assert_eq!(engine.run_state.deck, vec!["Eruption", "DevaForm"]);
        assert_eq!(misc_counter(engine), before + 3);
        assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    }
    assert_eq!(
        CoreCheckpoint::capture(&restored).unwrap(),
        CoreCheckpoint::capture(&original).unwrap()
    );
}

#[test]
fn sensory_stone_exposes_three_costed_choices_and_colorless_reward_items() {
    // SensoryStone first reveals the three choices. Every choice consumes one
    // misc randomLong for the shuffled memory text, then creates exactly one,
    // two, or three colorless RewardItems and applies 0/5/10 HP loss.
    // Java: events/beyond/SensoryStone.java::buttonEffect/getRandomMemory/reward.
    let mut engine = RunEngine::new(857, 0);
    engine.run_state.current_hp = 72;
    let before = misc_counter(&engine);
    engine.debug_set_typed_event_state(act_event(3, "Sensory Stone"));
    assert_eq!(engine.get_legal_actions(), vec![GameAction::EventChoice(0)]);
    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(engine.event_option_count(), 3);
    assert!(engine.step_game(&GameAction::EventChoice(2)).accepted());
    assert_eq!(engine.run_state.current_hp, 62);
    assert_eq!(misc_counter(&engine), before + 1);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    let rewards = engine.current_reward_screen().unwrap();
    assert_eq!(rewards.items.len(), 3);
    assert!(rewards
        .items
        .iter()
        .all(|item| item.label == "event_colorless_card_reward" && item.choices.len() == 3));
}

#[test]
fn designer_freezes_variants_and_full_service_orders_remove_before_upgrade() {
    // Designer's constructor consumes two booleans. The intro then reveals
    // A0 prices 40/60/90 and Full Service removes the chosen instance before
    // consuming one misc randomLong to choose a remaining upgrade target.
    // Java: events/shrines/Designer.java::<init>/buttonEffect/update.
    let mut engine = RunEngine::new(863, 0);
    engine.run_state.gold = 200;
    engine.run_state.deck = vec![
        "Eruption".to_string(),
        "Vigilance".to_string(),
        "ThirdEye".to_string(),
        "DevaForm".to_string(),
    ];
    let before = misc_counter(&engine);
    engine.debug_set_typed_event_state(shrine_event("Designer"));
    assert_eq!(misc_counter(&engine), before + 2);
    assert_eq!(engine.get_legal_actions(), vec![GameAction::EventChoice(0)]);
    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    let main = engine.debug_current_event().unwrap();
    assert_eq!(main.options.len(), 4);
    assert!(main.options[0].text.contains("40 gold"));
    assert!(main.options[1].text.contains("60 gold"));
    assert!(main.options[2].text.contains("90 gold"));
    assert!(main.options[3].text.contains("lose 3 HP"));

    let checkpoint = CoreCheckpoint::capture(&engine).unwrap();
    let mut restored = checkpoint.restore().unwrap();
    for run in [&mut engine, &mut restored] {
        assert!(run.step_game(&GameAction::EventChoice(2)).accepted());
        assert_eq!(run.run_state.gold, 110);
        assert_eq!(run.current_phase(), RunPhase::CardReward);
        assert!(run.step_game(&GameAction::SelectRewardItem(0)).accepted());
        assert!(run
            .step_game(&GameAction::ChooseRewardOption {
                item_index: 0,
                choice_index: 0,
            })
            .accepted());
        assert_eq!(run.run_state.deck.len(), 3);
        assert_eq!(
            run.run_state
                .deck
                .iter()
                .filter(|card| card.ends_with('+'))
                .count(),
            1
        );
        assert_eq!(misc_counter(run), before + 3);
    }
    assert_eq!(
        CoreCheckpoint::capture(&restored).unwrap(),
        CoreCheckpoint::capture(&engine).unwrap()
    );
}

#[test]
fn we_meet_again_freezes_all_three_offers_and_pays_an_immediate_relic() {
    // Constructor order is potion shuffle, gold roll, card shuffle. The three
    // exact offers persist, and every paid branch removes that offered object
    // before immediately obtaining a screenless relic.
    // Java: events/shrines/WeMeetAgain.java.
    let mut original = RunEngine::new(881, 0);
    original.run_state.gold = 120;
    original.run_state.potions = vec![
        "Block Potion".to_string(),
        String::new(),
        "StancePotion".to_string(),
    ];
    original.run_state.deck = vec![
        "Strike_P".to_string(),
        "ThirdEye".to_string(),
        "DevaForm".to_string(),
        "Vigilance".to_string(),
    ];
    let misc_before = misc_counter(&original);
    original.debug_set_typed_event_state(shrine_event("WeMeetAgain"));
    assert_eq!(misc_counter(&original), misc_before + 3);
    let event = original.debug_current_event().unwrap();
    assert!(event.options[0].text.starts_with("Give potion: "));
    assert!(event.options[1].text.starts_with("Give "));
    assert!(event.options[1].text.ends_with(" gold"));
    assert!(event.options[2].text.starts_with("Give card: "));
    assert_eq!(event.options[3].text, "Leave");

    let offered_potion = event.options[0]
        .text
        .strip_prefix("Give potion: ")
        .unwrap()
        .to_string();
    let offered_gold = event.options[1]
        .text
        .strip_prefix("Give ")
        .and_then(|text| text.strip_suffix(" gold"))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let offered_card = event.options[2]
        .text
        .strip_prefix("Give card: ")
        .unwrap()
        .to_string();
    let checkpoint = CoreCheckpoint::capture(&original).unwrap();

    let mut potion = checkpoint.restore().unwrap();
    let relics_before = potion.run_state.relics.len();
    assert!(potion.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(potion.current_phase(), RunPhase::MapChoice);
    assert_eq!(potion.run_state.relics.len(), relics_before + 1);
    assert_eq!(
        potion
            .run_state
            .potions
            .iter()
            .filter(|id| id.as_str() == offered_potion)
            .count(),
        original
            .run_state
            .potions
            .iter()
            .filter(|id| id.as_str() == offered_potion)
            .count()
            - 1
    );
    assert_eq!(misc_counter(&potion), misc_before + 3);

    let mut gold = checkpoint.restore().unwrap();
    let relics_before = gold.run_state.relics.len();
    assert!(gold.step_game(&GameAction::EventChoice(1)).accepted());
    assert_eq!(gold.run_state.gold, 120 - offered_gold);
    assert_eq!(gold.run_state.relics.len(), relics_before + 1);
    assert_eq!(misc_counter(&gold), misc_before + 3);

    let mut card = checkpoint.restore().unwrap();
    let copies_before = card
        .run_state
        .deck
        .iter()
        .filter(|id| id.as_str() == offered_card)
        .count();
    let relics_before = card.run_state.relics.len();
    assert!(card.step_game(&GameAction::EventChoice(2)).accepted());
    assert_eq!(
        card.run_state
            .deck
            .iter()
            .filter(|id| id.as_str() == offered_card)
            .count(),
        copies_before - 1
    );
    assert_eq!(card.run_state.relics.len(), relics_before + 1);
    assert_eq!(misc_counter(&card), misc_before + 3);
}
