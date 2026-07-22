use crate::events::{typed_events_for_act, EventRuntimeStatus};
use crate::run::{GameAction, RunEngine, RunPhase};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn test_event_runtime_wave4_golden_wing_feed_is_runtime_supported_and_removes_a_card() {
    let mut engine = RunEngine::new(13, 20);
    let hp_before = engine.run_state.current_hp;
    let deck_before = engine.run_state.deck.len();
    let golden_wing = typed_event(1, "Golden Wing");
    assert!(matches!(
        golden_wing.options[0].status,
        EventRuntimeStatus::Supported
    ));

    engine.debug_set_typed_event_state(golden_wing);
    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    // GoldenWing.java advances INTRO -> PURGE after damage. A second dialog
    // click opens the card grid; purgeLogic removes the selected card.
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, hp_before - 7);
    assert_eq!(engine.run_state.deck.len(), deck_before);
    assert_eq!(engine.get_legal_actions(), vec![GameAction::EventChoice(0)]);
    assert!(engine.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert!(engine
        .step_game(&GameAction::SelectRewardItem(0))
        .accepted());
    assert!(engine
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.deck.len(), deck_before - 1);
}

#[test]
fn test_event_runtime_wave4_the_joust_resolves_bets_through_typed_runtime_rng() {
    let mut murderer_engine = RunEngine::new(5, 20);
    let mut owner_engine = RunEngine::new(5, 20);
    let gold_before = murderer_engine.run_state.gold;
    let joust = typed_event(2, "The Joust");
    assert!(matches!(
        joust.options[0].status,
        EventRuntimeStatus::Supported
    ));
    assert!(matches!(
        joust.options[1].status,
        EventRuntimeStatus::Supported
    ));

    murderer_engine.debug_set_typed_event_state(joust.clone());
    let murderer_step = murderer_engine.step_game(&GameAction::EventChoice(0));
    assert!(murderer_step.accepted());
    assert_eq!(murderer_engine.current_phase(), RunPhase::MapChoice);

    owner_engine.debug_set_typed_event_state(joust);
    let owner_step = owner_engine.step_game(&GameAction::EventChoice(1));
    assert!(owner_step.accepted());
    assert_eq!(owner_engine.current_phase(), RunPhase::MapChoice);

    assert_eq!(murderer_engine.run_state.gold, gold_before - 50);
    assert_eq!(owner_engine.run_state.gold, gold_before + 200);
}

#[test]
fn test_event_runtime_wave4_winding_halls_stages_intro_effect_and_leave() {
    // Source: decompiled/java-src/com/megacrit/cardcrawl/events/beyond/
    // WindingHalls.java. Screen 0 is a no-op intro, screen 1 applies the
    // selected branch, and screen 2 requires a final Leave click. Madness is
    // queued through ShowCardAndObtainEffect, not a player reward choice.
    let mut engine = RunEngine::new(11, 20);
    engine.run_state.current_hp = engine.run_state.max_hp;
    let deck_before = engine.run_state.deck.len();
    let halls = typed_event(3, "Winding Halls");
    assert!(matches!(
        halls.options[0].status,
        EventRuntimeStatus::Supported
    ));

    engine.debug_set_typed_event_state(halls);
    let intro = engine.step_game(&GameAction::EventChoice(0));
    assert!(intro.accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, engine.run_state.max_hp);
    assert_eq!(engine.run_state.deck.len(), deck_before);

    let embrace = engine.step_game(&GameAction::EventChoice(0));
    assert!(embrace.accepted());
    assert_eq!(engine.current_phase(), RunPhase::Event);
    let expected_damage = (engine.run_state.max_hp * 18 + 50) / 100;
    assert_eq!(
        engine.run_state.current_hp,
        engine.run_state.max_hp - expected_damage
    );
    assert_eq!(engine.run_state.deck.len(), deck_before + 2);
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Madness")
            .count(),
        2
    );
    assert!(engine.current_reward_screen().is_none());

    let leave = engine.step_game(&GameAction::EventChoice(0));
    assert!(leave.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);

    // A0 uses MathUtils.round(maxHealth * 0.125f), not an integer 12%
    // approximation: 68 * 0.125 = 8.5 rounds to 9. NORMAL event damage still
    // traverses Tungsten Rod's onLoseHpLast callback.
    let mut boundary = RunEngine::new(19, 0);
    boundary.run_state.max_hp = 68;
    boundary.run_state.current_hp = 68;
    boundary.debug_set_typed_event_state(typed_event(3, "Winding Halls"));
    assert!(boundary.step_game(&GameAction::EventChoice(0)).accepted());
    assert!(boundary.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(boundary.run_state.current_hp, 59);

    let mut tungsten = RunEngine::new(19, 0);
    tungsten.run_state.max_hp = 68;
    tungsten.run_state.current_hp = 68;
    tungsten.run_state.relics.push("TungstenRod".to_string());
    tungsten.debug_set_typed_event_state(typed_event(3, "Winding Halls"));
    assert!(tungsten.step_game(&GameAction::EventChoice(0)).accepted());
    assert!(tungsten.step_game(&GameAction::EventChoice(0)).accepted());
    assert_eq!(tungsten.run_state.current_hp, 60);
}

#[test]
fn test_event_runtime_wave4_winding_halls_retrace_and_press_on_use_percent_runtime_effects() {
    let mut retrace_engine = RunEngine::new(17, 20);
    retrace_engine.run_state.current_hp = 10;
    let halls = typed_event(3, "Winding Halls");
    retrace_engine.debug_set_typed_event_state(halls.clone());
    assert!(retrace_engine
        .step_game(&GameAction::EventChoice(0))
        .accepted());
    let retrace = retrace_engine.step_game(&GameAction::EventChoice(1));
    assert!(retrace.accepted());
    assert_eq!(retrace_engine.current_phase(), RunPhase::Event);
    assert_eq!(retrace_engine.run_state.current_hp, 24);
    assert!(retrace_engine
        .run_state
        .deck
        .iter()
        .any(|card| card == "Writhe"));
    assert!(retrace_engine
        .step_game(&GameAction::EventChoice(0))
        .accepted());
    assert_eq!(retrace_engine.current_phase(), RunPhase::MapChoice);

    let mut press_engine = RunEngine::new(17, 20);
    let max_hp_before = press_engine.run_state.max_hp;
    let current_hp_before = press_engine.run_state.current_hp;
    press_engine.debug_set_typed_event_state(halls);
    assert!(press_engine
        .step_game(&GameAction::EventChoice(0))
        .accepted());
    let press = press_engine.step_game(&GameAction::EventChoice(2));
    assert!(press.accepted());
    assert_eq!(press_engine.current_phase(), RunPhase::Event);
    assert_eq!(press_engine.run_state.max_hp, max_hp_before - 3);
    // AbstractCreature.decreaseMaxHealth only clamps current HP when it now
    // exceeds the reduced maximum; it does not heal an already-wounded A20
    // character. Java: core/AbstractCreature.java::decreaseMaxHealth.
    assert_eq!(
        press_engine.run_state.current_hp,
        current_hp_before.min(press_engine.run_state.max_hp)
    );
    assert!(press_engine
        .step_game(&GameAction::EventChoice(0))
        .accepted());
    assert_eq!(press_engine.current_phase(), RunPhase::MapChoice);
}
