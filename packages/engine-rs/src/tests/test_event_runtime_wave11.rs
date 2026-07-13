use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, typed_shrine_events, EventRuntimeStatus, TypedEventDef};
use crate::run::{RunAction, RunEngine, RunPhase};
use crate::actions::Action;
use crate::cards::CardType;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{ensure_in_hand, play_on_enemy};

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/city/Colosseum.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/city/CursedTome.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

#[test]
fn colosseum_is_supported_and_uses_event_continuation_plus_two_combats() {
    let mut engine = RunEngine::new(91, 20);
    let colosseum = typed_event(2, "Colosseum");
    assert!(matches!(
        colosseum.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(colosseum);

    let intro = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(intro.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    let intro_ctx = engine.current_decision_context().event.expect("colosseum follow-up event");
    assert_eq!(intro_ctx.name, "Colosseum");
    assert_eq!(intro_ctx.options.len(), 1);

    let first_fight = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(first_fight.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    let combat = engine.get_combat_engine().expect("slavers combat");
    assert_eq!(combat.state.enemies.len(), 2);
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "SlaverBlue"));
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "SlaverRed"));

    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::Event);
    let post_ctx = engine.current_decision_context().event.expect("colosseum post combat");
    assert_eq!(post_ctx.name, "Colosseum");
    assert_eq!(post_ctx.options.len(), 2);

    let second_fight = engine.step_with_result(&RunAction::EventChoice(1));
    assert!(second_fight.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Combat);
    let combat = engine.get_combat_engine().expect("nobs combat");
    assert_eq!(combat.state.enemies.len(), 2);
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "SlaverBoss"));
    assert!(combat.state.enemies.iter().any(|enemy| enemy.id == "GremlinNob"));

    let gold_before = engine.run_state.gold;
    engine.debug_force_current_combat_outcome(true);
    engine.debug_resolve_current_combat_outcome();
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.gold, gold_before + 100);

    let screen = engine.current_reward_screen().expect("colosseum reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 2);
    assert!(screen.items.iter().all(|item| item.kind == RewardItemKind::Relic));
}

#[test]
fn cursed_tome_progresses_page_by_page_and_opens_book_reward_on_take() {
    let mut engine = RunEngine::new(93, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    let cursed_tome = typed_event(2, "Cursed Tome");
    assert!(matches!(
        cursed_tome.options[0].status,
        EventRuntimeStatus::Supported
    ));
    engine.debug_set_typed_event_state(cursed_tome);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 80);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 79);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 77);

    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::Event);
    assert_eq!(engine.run_state.current_hp, 74);

    let take = engine.step_with_result(&RunAction::EventChoice(0));
    assert!(take.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::CardReward);
    assert_eq!(engine.run_state.current_hp, 59);

    let screen = engine.current_reward_screen().expect("cursed tome reward");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::Relic);
    assert!(matches!(
        screen.items[0].label.as_str(),
        "Necronomicon" | "Enchiridion" | "Nilry's Codex" | "Circlet"
    ));
}

#[test]
fn cursed_tome_stop_reading_takes_three_and_returns_to_map_without_reward() {
    let mut engine = RunEngine::new(95, 20);
    engine.run_state.max_hp = 80;
    engine.run_state.current_hp = 80;
    engine.debug_set_typed_event_state(typed_event(2, "Cursed Tome"));

    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    engine.step_with_result(&RunAction::EventChoice(0));
    let stop = engine.step_with_result(&RunAction::EventChoice(1));

    assert!(stop.action_accepted);
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert_eq!(engine.run_state.current_hp, 71);
    assert!(engine.current_reward_screen().is_none());
}

#[test]
fn cursed_tome_enchiridion_adds_one_zero_cost_watcher_power_prebattle() {
    // CursedTome.randomBook excludes books already owned, so owning the other
    // two forces Enchiridion. Enchiridion.atPreBattle consumes one cardRandom
    // roll from the Watcher's source Power pools, sets non-X cost to zero for
    // the turn, and adds the copy before the normal opening draw.
    let mut engine = RunEngine::new(97, 20);
    engine.run_state.relics.extend([
        "Necronomicon".to_string(),
        "Nilry's Codex".to_string(),
    ]);
    engine.debug_set_typed_event_state(typed_event(2, "Cursed Tome"));
    for _ in 0..4 {
        assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    }
    assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    let screen = engine.current_reward_screen().expect("forced Enchiridion reward");
    assert_eq!(screen.items[0].label, "Enchiridion");
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Enchiridion"));

    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.get_combat_engine().expect("Enchiridion combat");
    const WATCHER_POWERS: &[&str] = &[
        "BattleHymn", "DevaForm", "Devotion", "Discipline", "Establishment",
        "Fasting2", "Wireheading", "LikeWater", "MasterReality",
        "MentalFortress", "Nirvana", "Adaptation", "Study",
    ];
    let generated = combat
        .state
        .hand
        .iter()
        .find(|card| combat.card_registry.card_def_by_id(card.def_id).card_type == CardType::Power)
        .expect("one generated Power in opening hand");
    let generated_id = combat.card_registry.card_def_by_id(generated.def_id).id;
    assert!(WATCHER_POWERS.contains(&generated_id));
    assert_eq!(generated.cost, 0);
    assert_eq!(combat.state.hand.len(), 7); // five-card draw + Miracle + Enchiridion Power
    assert_eq!(combat.rng_counters()["cardRandom"], 1);
}

#[test]
fn cursed_tome_necronomicon_obtains_curse_and_replays_normal_and_x_attacks_once() {
    // CursedTome.randomBook excludes owned books, forcing Necronomicon here.
    // Necronomicon.onEquip obtains one Necronomicurse. onUseCard replays only
    // the first Attack each turn whose turn cost is at least two, or whose
    // X-cost energyOnUse is at least two, using the original target and X.
    let mut engine = RunEngine::new(99, 20);
    engine.run_state.relics.extend([
        "Enchiridion".to_string(),
        "Nilry's Codex".to_string(),
    ]);
    engine.debug_set_typed_event_state(typed_event(2, "Cursed Tome"));
    for _ in 0..5 {
        assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    }
    let screen = engine.current_reward_screen().expect("forced Necronomicon reward");
    assert_eq!(screen.items[0].label, "Necronomicon");
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Necronomicon"));
    assert_eq!(
        engine
            .run_state
            .deck
            .iter()
            .filter(|card| card.as_str() == "Necronomicurse")
            .count(),
        1
    );

    engine.run_state.deck.push("Bludgeon".to_string());
    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.debug_combat_engine_mut();
    combat.state.enemies[0].entity.hp = 200;
    combat.state.enemies[0].entity.max_hp = 200;
    ensure_in_hand(combat, "Bludgeon");
    let hp_before = combat.state.enemies[0].entity.hp;
    assert!(play_on_enemy(combat, "Bludgeon", 0));
    assert_eq!(hp_before - combat.state.enemies[0].entity.hp, 64);
    assert_eq!(combat.state.player.status(sid::NECRONOMICON_USED), 1);

    let mut x_cost = RunEngine::new(101, 0);
    x_cost.run_state.relics.push("Necronomicon".to_string());
    x_cost.run_state.deck.push("Whirlwind".to_string());
    x_cost.debug_enter_specific_combat(&["JawWorm"]);
    let combat = x_cost.debug_combat_engine_mut();
    combat.state.energy = 2;
    ensure_in_hand(combat, "Whirlwind");
    let hp_before = combat.state.enemies[0].entity.hp;
    assert!(play_on_enemy(combat, "Whirlwind", 0));
    assert_eq!(hp_before - combat.state.enemies[0].entity.hp, 20);
    assert_eq!(combat.state.player.status(sid::NECRONOMICON_USED), 1);

    combat.state.energy = 2;
    ensure_in_hand(combat, "Whirlwind");
    let hp_before_second = combat.state.enemies[0].entity.hp;
    assert!(play_on_enemy(combat, "Whirlwind", 0));
    assert_eq!(hp_before_second - combat.state.enemies[0].entity.hp, 10);

    let mut traded = RunEngine::new(103, 0);
    traded.run_state.relics = vec!["Necronomicon".to_string(), "Anchor".to_string()];
    traded.run_state.deck.push("Necronomicurse".to_string());
    let nloth = typed_shrine_events()
        .into_iter()
        .find(|event| event.name == "N'loth")
        .expect("N'loth event");
    traded.debug_set_typed_event_state(nloth);
    let trade_index = traded
        .debug_current_event()
        .expect("N'loth choices")
        .options
        .iter()
        .position(|option| option.text.contains("Necronomicon"))
        .expect("Necronomicon trade option");
    assert!(traded
        .step_with_result(&RunAction::EventChoice(trade_index))
        .action_accepted);
    assert!(!traded.run_state.relics.iter().any(|relic| relic == "Necronomicon"));
    assert!(!traded.run_state.deck.iter().any(|card| card == "Necronomicurse"));
}

#[test]
fn cursed_tome_nilrys_codex_pauses_end_turn_for_three_card_draw_pile_choice() {
    // CursedTome.randomBook excludes the two owned books and therefore offers
    // Nilry's Codex. NilrysCodex.onPlayerEndTurn queues CodexAction, which
    // consumes cardRandom rolls until it has three distinct source-pool cards;
    // the selected copy is then inserted at a random draw-pile position.
    let mut engine = RunEngine::new(105, 20);
    engine.run_state.relics.extend([
        "Necronomicon".to_string(),
        "Enchiridion".to_string(),
    ]);
    engine.debug_set_typed_event_state(typed_event(2, "Cursed Tome"));
    for _ in 0..5 {
        assert!(engine.step_with_result(&RunAction::EventChoice(0)).action_accepted);
    }
    let screen = engine.current_reward_screen().expect("forced Codex reward");
    assert_eq!(screen.items[0].label, "Nilry's Codex");
    assert!(engine.step_with_result(&RunAction::SelectRewardItem(0)).action_accepted);

    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.debug_combat_engine_mut();
    let total_cards_before = combat.state.hand.len()
        + combat.state.draw_pile.len()
        + combat.state.discard_pile.len()
        + combat.state.exhaust_pile.len();
    let card_random_before = combat.rng_counters()["cardRandom"];
    combat.execute_action(&Action::EndTurn);
    assert_eq!(combat.phase, CombatPhase::AwaitingChoice);
    let choice = combat.choice.as_ref().expect("Codex choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    let offered = choice
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::GeneratedCard(card) => card.def_id,
            _ => panic!("Codex must offer generated cards"),
        })
        .collect::<Vec<_>>();
    assert_ne!(offered[0], offered[1]);
    assert_ne!(offered[0], offered[2]);
    assert_ne!(offered[1], offered[2]);

    let selected = offered[0];
    combat.execute_action(&Action::Choose(0));
    assert_eq!(combat.phase, CombatPhase::PlayerTurn);
    let total_cards_after = combat.state.hand.len()
        + combat.state.draw_pile.len()
        + combat.state.discard_pile.len()
        + combat.state.exhaust_pile.len();
    assert_eq!(total_cards_after, total_cards_before + 1);
    assert!(combat
        .state
        .hand
        .iter()
        .chain(combat.state.draw_pile.iter())
        .chain(combat.state.discard_pile.iter())
        .chain(combat.state.exhaust_pile.iter())
        .any(|card| card.def_id == selected));
    assert!(combat.rng_counters()["cardRandom"] >= card_random_before + 4);
}
