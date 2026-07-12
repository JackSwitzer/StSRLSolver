use crate::decision::{DecisionAction, RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus};
use crate::run::{RunAction, RunEngine, RunPhase};
use crate::status_ids::sid;
use crate::tests::support::{ensure_in_hand, play_on_enemy};

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

fn shrine_event(name: &str) -> crate::events::TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

fn resolve_real_combat_victory(engine: &mut RunEngine) {
    let combat = engine.debug_combat_engine_mut();
    combat.state.enemies[0].entity.hp = 0;
    combat.finalize_enemy_death(0);
    combat.check_combat_end();
    assert!(combat.state.player_won);
    engine.debug_resolve_current_combat_outcome();
}

#[test]
fn library_read_has_nested_choice_without_fake_skip() {
    let mut engine = RunEngine::new(101, 20);
    let library = typed_event(2, "The Library");
    assert!(matches!(
        library.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(library);

    let open_screen = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(open_screen.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine.current_reward_screen().expect("library reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert!(screen.items[0].claimable);
    assert!(!screen.items[0].skip_allowed);

    let open_choice = engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_choice.action_accepted);
    assert_eq!(
        open_choice.legal_decision_actions,
        vec![
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 0,
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 1,
            },
            DecisionAction::PickRewardChoice {
                item_index: 0,
                choice_index: 2,
            },
        ]
    );
}

#[test]
fn drug_dealer_all_three_supported_branches_use_canonical_runtime_paths() {
    let dealer = typed_event(2, "Drug Dealer");
    assert!(dealer
        .options
        .iter()
        .all(|option| matches!(option.status, EventRuntimeStatus::Supported)));

    let mut jax_engine = RunEngine::new(17, 20);
    jax_engine.debug_set_typed_event_state(dealer.clone());
    let jax_step = jax_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(jax_step.action_accepted);
    assert_eq!(jax_engine.current_phase(), RunPhase::CardReward);
    let jax_screen = jax_engine.current_reward_screen().expect("jax reward screen");
    assert_eq!(jax_screen.items.len(), 1);
    assert_eq!(jax_screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(jax_screen.items[0].choices.len(), 1);
    let open_jax = jax_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open_jax.action_accepted);
    let choose_jax = jax_engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose_jax.action_accepted);
    assert!(jax_engine.run_state.deck.iter().any(|card| card == "J.A.X."));

    let mut transform_engine = RunEngine::new(23, 20);
    let deck_before = transform_engine.run_state.deck.len();
    transform_engine.debug_set_typed_event_state(dealer.clone());
    let transform_step = transform_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(transform_step.action_accepted);
    assert_eq!(transform_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(transform_engine.run_state.deck.len(), deck_before);

    let mut relic_engine = RunEngine::new(31, 20);
    relic_engine.debug_set_typed_event_state(dealer);
    let relic_step = relic_engine.step_with_result(&RunAction::EventChoice(2));
    assert!(relic_step.action_accepted);
    // DrugDealer.java::buttonEffect uses spawnRelicAndObtain during the event;
    // no additional reward-screen action exists.
    assert_eq!(relic_engine.current_phase(), RunPhase::MapChoice);
    assert!(relic_engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "MutagenicStrength"));
}

#[test]
fn drug_dealer_mutagenic_strength_uses_canonical_id_and_next_combat_effect() {
    // MutagenicStrength.java::atBattleStart applies 3 Strength and 3
    // LoseStrength. DrugDealer.java obtains this exact ID immediately and uses
    // Circlet instead if the relic is already owned.
    let dealer = typed_event(2, "Drug Dealer");
    let mut engine = RunEngine::new(37, 0);
    engine.debug_set_typed_event_state(dealer.clone());

    assert!(engine
        .step_with_result(&RunAction::EventChoice(2))
        .action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.debug_combat_engine_mut();
    assert_eq!(combat.state.player.status(sid::STRENGTH), 3);
    assert_eq!(combat.state.player.status(sid::LOSE_STRENGTH), 3);

    let mut duplicate = RunEngine::new(41, 0);
    duplicate
        .run_state
        .relics
        .push("MutagenicStrength".to_string());
    duplicate.debug_set_typed_event_state(dealer);
    assert!(duplicate
        .step_with_result(&RunAction::EventChoice(2))
        .action_accepted);
    assert_eq!(
        duplicate
            .run_state
            .relics
            .iter()
            .filter(|relic| relic.as_str() == "MutagenicStrength")
            .count(),
        1
    );
    assert!(duplicate
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Circlet"));
}

#[test]
fn face_trader_stages_touch_values_and_never_grants_a_face_on_touch() {
    // FaceTrader.java has a one-choice intro, then touch/trade/leave. Touch
    // deals maxHealth / 10 damage (minimum one) and grants 75 gold, or 50 at A15.
    let event = shrine_event("FaceTrader");
    assert_eq!(event.options.len(), 1);

    let mut a0 = RunEngine::new(47, 0);
    let gold_before = a0.run_state.gold;
    let relics_before = a0.run_state.relics.clone();
    a0.debug_set_typed_event_state(event.clone());
    assert!(a0.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(a0.current_phase(), RunPhase::Event);
    let main = a0.debug_current_event().expect("Face Trader main screen");
    assert_eq!(main.options.len(), 3);
    assert_eq!(main.options[0].text, "Touch (take 7 damage, gain 75 gold)");
    assert!(a0.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(a0.current_phase(), RunPhase::MapChoice);
    assert_eq!(a0.run_state.current_hp, 65);
    assert_eq!(a0.run_state.gold, gold_before + 75);
    assert_eq!(a0.run_state.relics, relics_before);

    let mut a15 = RunEngine::new(49, 15);
    a15.run_state.max_hp = 100;
    a15.run_state.current_hp = 100;
    let gold_before = a15.run_state.gold;
    a15.debug_set_typed_event_state(event);
    a15.step(&RunAction::EventChoice(0));
    a15.step(&RunAction::EventChoice(0));
    assert_eq!(a15.run_state.current_hp, 90);
    assert_eq!(a15.run_state.gold, gold_before + 50);
}

#[test]
fn face_of_cleric_trade_reaches_next_victory_and_obeys_healing_rules() {
    // FaceTrader.java filters owned faces before its miscRng-backed choice.
    // FaceOfCleric.java::onVictory calls increaseMaxHp(1, true): max HP rises
    // every victory and the accompanying one-HP heal can be blocked by Bloom.
    let event = shrine_event("FaceTrader");
    let other_faces = ["CultistMask", "GremlinMask", "NlothsMask", "SsserpentHead"];
    let mut engine = RunEngine::new(51, 0);
    engine
        .run_state
        .relics
        .extend(other_faces.iter().map(|face| (*face).to_string()));
    engine.run_state.current_hp = 40;
    engine.debug_set_typed_event_state(event.clone());
    engine.step(&RunAction::EventChoice(0));
    assert!(engine.step_with_result(&RunAction::EventChoice(1)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "FaceOfCleric"));

    let max_before = engine.run_state.max_hp;
    engine.debug_enter_specific_combat(&["JawWorm"]);
    resolve_real_combat_victory(&mut engine);
    assert_eq!(engine.run_state.max_hp, max_before + 1);
    assert_eq!(engine.run_state.current_hp, 41);

    let mut blocked = RunEngine::new(53, 0);
    blocked.run_state.relics.extend([
        "FaceOfCleric".to_string(),
        "Mark of the Bloom".to_string(),
    ]);
    blocked.run_state.relic_flags.rebuild(&blocked.run_state.relics);
    blocked.run_state.current_hp = 40;
    let max_before = blocked.run_state.max_hp;
    blocked.debug_enter_specific_combat(&["JawWorm"]);
    assert_eq!(
        blocked
            .get_combat_engine()
            .expect("blocked combat")
            .state
            .player
            .status(sid::HAS_MARK_OF_BLOOM),
        1
    );
    resolve_real_combat_victory(&mut blocked);
    assert_eq!(blocked.run_state.max_hp, max_before + 1);
    assert_eq!(blocked.run_state.current_hp, 40);
}

#[test]
fn gremlin_mask_trade_applies_one_artifact_aware_weak_in_next_combat() {
    // GremlinMask.java::atBattleStart applies WeakPower(player, 1, false).
    // FaceTrader.java filters owned faces, allowing the trade to be forced to
    // GremlinMask by owning the other four candidates.
    let mut engine = RunEngine::new(55, 0);
    engine.run_state.relics.extend(
        ["CultistMask", "FaceOfCleric", "NlothsMask", "SsserpentHead"]
            .iter()
            .map(|face| (*face).to_string()),
    );
    engine.debug_set_typed_event_state(shrine_event("FaceTrader"));
    engine.step(&RunAction::EventChoice(0));
    assert!(engine.step_with_result(&RunAction::EventChoice(1)).action_accepted);
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "GremlinMask"));

    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.debug_combat_engine_mut();
    assert_eq!(combat.state.player.status(sid::WEAKENED), 1);
    ensure_in_hand(combat, "Strike");
    let hp_before = combat.state.enemies[0].entity.hp;
    assert!(play_on_enemy(combat, "Strike", 0));
    assert_eq!(combat.state.enemies[0].entity.hp, hp_before - 4);

    let mut artifact = RunEngine::new(57, 0);
    artifact.run_state.relics.extend([
        "ClockworkSouvenir".to_string(),
        "GremlinMask".to_string(),
    ]);
    artifact.debug_enter_specific_combat(&["JawWorm"]);
    let combat = artifact.get_combat_engine().expect("artifact combat");
    assert_eq!(combat.state.player.status(sid::WEAKENED), 0);
    assert_eq!(combat.state.player.status(sid::ARTIFACT), 0);
}

#[test]
fn nest_branches_cover_direct_gold_and_specific_card_reward() {
    let nest = typed_event(2, "Nest");
    assert!(nest
        .options
        .iter()
        .all(|option| matches!(option.status, EventRuntimeStatus::Supported)));

    let mut gold_engine = RunEngine::new(37, 20);
    let gold_before = gold_engine.run_state.gold;
    gold_engine.debug_set_typed_event_state(nest.clone());
    let steal = gold_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(steal.action_accepted);
    assert_eq!(gold_engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(gold_engine.run_state.gold, gold_before + 99);

    let mut dagger_engine = RunEngine::new(41, 20);
    let hp_before = dagger_engine.run_state.current_hp;
    dagger_engine.debug_set_typed_event_state(nest);
    let join = dagger_engine.step_with_result(&RunAction::EventChoice(1));
    assert!(join.action_accepted);
    assert_eq!(dagger_engine.current_phase(), RunPhase::CardReward);
    assert_eq!(dagger_engine.run_state.current_hp, hp_before - 6);
    let screen = dagger_engine.current_reward_screen().expect("ritual dagger screen");
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].choices.len(), 1);
    let open = dagger_engine.step_with_result(&RunAction::SelectRewardItem(0));
    assert!(open.action_accepted);
    let choose = dagger_engine.step_with_result(&RunAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(choose.action_accepted);
    assert!(dagger_engine
        .run_state
        .deck
        .iter()
        .any(|card| card == "RitualDagger"));
}

#[test]
fn sensory_stone_focus_and_tomb_of_lord_red_mask_flow_through_event_rewards() {
    let mut sensory_engine = RunEngine::new(43, 20);
    sensory_engine.debug_set_typed_event_state(typed_event(3, "Sensory Stone"));
    let sensory = sensory_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(sensory.action_accepted);
    assert_eq!(sensory_engine.current_phase(), RunPhase::CardReward);
    let sensory_screen = sensory_engine.current_reward_screen().expect("sensory screen");
    assert_eq!(sensory_screen.source, RewardScreenSource::Event);
    assert_eq!(sensory_screen.items[0].kind, RewardItemKind::CardChoice);

    let mut tomb_engine = RunEngine::new(47, 20);
    tomb_engine.debug_set_typed_event_state(typed_event(3, "Tomb of Lord Red Mask"));
    let tomb = tomb_engine.step_with_result(&RunAction::EventChoice(0));
    assert!(tomb.action_accepted);
    assert_eq!(tomb_engine.current_phase(), RunPhase::CardReward);
    let tomb_screen = tomb_engine.current_reward_screen().expect("tomb screen");
    assert_eq!(tomb_screen.source, RewardScreenSource::Event);
    assert_eq!(tomb_screen.items.len(), 1);
    assert_eq!(tomb_screen.items[0].kind, RewardItemKind::Relic);
    assert_eq!(tomb_screen.items[0].label, "Red Mask");
}

#[test]
fn tomb_of_lord_red_mask_reward_is_claimable_into_run_state() {
    // TombRedMask.java grants the canonical Red Mask special relic; RedMask.java
    // declares the same ID consumed by its combat-start hook.
    let mut engine = RunEngine::new(49, 20);
    engine.debug_set_typed_event_state(typed_event(3, "Tomb of Lord Red Mask"));
    assert!(engine
        .step_with_result(&RunAction::EventChoice(0))
        .action_accepted);
    assert!(engine
        .step_with_result(&RunAction::SelectRewardItem(0))
        .action_accepted);
    assert!(engine
        .run_state
        .relics
        .iter()
        .any(|relic| relic == "Red Mask"));
}

#[test]
fn mushrooms_eat_branch_is_now_supported_heal_plus_curse() {
    let mut engine = RunEngine::new(53, 20);
    let mushrooms = typed_event(1, "Mushrooms");
    assert!(matches!(
        mushrooms.options[1].status,
        EventRuntimeStatus::Supported
    ));
    engine.run_state.current_hp = 25;
    engine.debug_set_typed_event_state(mushrooms);

    let step = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.run_state.current_hp > 25);
    assert!(engine.run_state.deck.iter().any(|card| card == "Parasite"));
}

#[test]
fn big_fish_banana_branch_applies_direct_max_hp_gain() {
    let mut engine = RunEngine::new(59, 20);
    let big_fish = typed_event(1, "Big Fish");
    let max_hp_before = engine.run_state.max_hp;
    let hp_before = engine.run_state.current_hp;
    engine.debug_set_typed_event_state(big_fish);

    let step = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(step.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.max_hp, max_hp_before + 2);
    assert_eq!(engine.run_state.current_hp, hp_before + 2);
}
