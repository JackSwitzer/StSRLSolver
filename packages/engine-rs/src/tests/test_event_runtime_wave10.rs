use crate::decision::{RewardItemKind, RewardScreenSource};
use crate::events::{typed_events_for_act, typed_shrine_events, EventProgramOp, EventRuntimeStatus};
use crate::run::{GameAction, RunEngine, RunPhase};
use crate::status_ids::sid;

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/events/shrines/Bonfire.java
// - decompiled/java-src/com/megacrit/cardcrawl/events/beyond/SpireHeart.java

fn typed_event(act: i32, name: &str) -> crate::events::TypedEventDef {
    typed_events_for_act(act)
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
}

fn typed_shrine_event(name: &str) -> crate::events::TypedEventDef {
    typed_shrine_events()
        .into_iter()
        .find(|event| event.name == name)
        .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
}

#[test]
fn bonfire_offer_branch_is_supported_and_uses_shared_deck_selection() {
    let bonfire = typed_shrine_event("Bonfire Elementals");
    assert!(matches!(bonfire.options[0].status, EventRuntimeStatus::Supported));
    assert!(matches!(
        bonfire.options[0].program.ops.as_slice(),
        [EventProgramOp::DeckSelection { label }] if label == "deck_selection_bonfire_offer"
    ));
}

#[test]
fn bonfire_choice_opens_canonical_event_reward_selection() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.deck = vec![
        "Defend".to_string(),
        "Wallop".to_string(),
        "Scrawl".to_string(),
    ];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));

    let step = engine.step_game(&GameAction::EventChoice(0));
    assert!(step.accepted());
    assert_eq!(engine.current_phase(), RunPhase::CardReward);

    let screen = engine.current_reward_screen().expect("bonfire reward screen");
    assert_eq!(screen.source, RewardScreenSource::Event);
    assert_eq!(screen.items.len(), 1);
    assert_eq!(screen.items[0].kind, RewardItemKind::CardChoice);
    assert_eq!(screen.items[0].label, "deck_selection_bonfire_offer");
    assert!(screen.items[0].claimable);
    assert!(!screen.items[0].skip_allowed);
    assert_eq!(screen.items[0].choices.len(), 3);
}

#[test]
fn bonfire_common_offer_purges_card_and_heals_five() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.max_hp = 70;
    engine.run_state.current_hp = 51;
    engine.run_state.deck = vec!["Deflect".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));

    engine.step_game(&GameAction::EventChoice(0));
    let choose = engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(choose.accepted());
    let resolve = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });

    assert!(resolve.accepted());
    assert_eq!(engine.current_phase(), RunPhase::MapChoice);
    assert!(engine.run_state.deck.is_empty());
    assert_eq!(engine.run_state.current_hp, 56);
}

#[test]
fn bonfire_uncommon_offer_heals_to_full() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.max_hp = 70;
    engine.run_state.current_hp = 23;
    engine.run_state.deck = vec!["Wallop".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));

    engine.step_game(&GameAction::EventChoice(0));
    engine.step_game(&GameAction::SelectRewardItem(0));
    let resolve = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });

    assert!(resolve.accepted());
    assert_eq!(engine.run_state.current_hp, 70);
    assert!(engine.run_state.deck.is_empty());
}

#[test]
fn bonfire_rare_offer_increases_max_hp_and_heals_to_full() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.max_hp = 70;
    engine.run_state.current_hp = 19;
    engine.run_state.deck = vec!["Scrawl".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));

    engine.step_game(&GameAction::EventChoice(0));
    engine.step_game(&GameAction::SelectRewardItem(0));
    let resolve = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });

    assert!(resolve.accepted());
    assert_eq!(engine.run_state.max_hp, 80);
    assert_eq!(engine.run_state.current_hp, 80);
    assert!(engine.run_state.deck.is_empty());
}

#[test]
fn bonfire_curse_offer_grants_spirit_poop_then_circlet_if_repeated() {
    let mut engine = RunEngine::new(73, 20);
    engine.run_state.deck = vec!["Pain".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));

    engine.step_game(&GameAction::EventChoice(0));
    engine.step_game(&GameAction::SelectRewardItem(0));
    let first = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(first.accepted());
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Spirit Poop"));

    engine.run_state.deck = vec!["Doubt".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));
    engine.step_game(&GameAction::EventChoice(0));
    engine.step_game(&GameAction::SelectRewardItem(0));
    let second = engine.step_game(&GameAction::ChooseRewardOption {
        item_index: 0,
        choice_index: 0,
    });
    assert!(second.accepted());
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Circlet"));
}

#[test]
fn spirit_poop_from_bonfire_has_no_gameplay_hook() {
    // SpiritPoop.java contains only construction, description, and makeCopy;
    // it has no equip, combat-start, turn, or reward callback.
    let mut engine = RunEngine::new(77, 0);
    engine.run_state.deck = vec!["Pain".to_string()];
    engine.debug_set_typed_event_state(typed_shrine_event("Bonfire Elementals"));
    engine.step_game(&GameAction::EventChoice(0));
    engine.step_game(&GameAction::SelectRewardItem(0));
    assert!(engine
        .step_game(&GameAction::ChooseRewardOption {
            item_index: 0,
            choice_index: 0,
        })
        .accepted());
    assert!(engine.run_state.relics.iter().any(|relic| relic == "Spirit Poop"));

    engine.run_state.deck = [
        "Strike", "Strike", "Strike", "Strike", "Defend", "Defend", "Defend", "Defend",
        "Eruption", "Vigilance",
    ]
    .iter()
    .map(|card| (*card).to_string())
    .collect();
    engine
        .run_state
        .relics
        .retain(|relic| matches!(relic.as_str(), "PureWater" | "Spirit Poop"));
    engine.debug_enter_specific_combat(&["JawWorm"]);
    let combat = engine.get_combat_engine().expect("Spirit Poop combat");
    assert_eq!(combat.state.energy, 3);
    assert_eq!(combat.state.hand.len(), 6); // ordinary draw plus Pure Water only
    assert_eq!(combat.state.player.status(sid::STRENGTH), 0);
    assert_eq!(combat.state.player.status(sid::DEXTERITY), 0);
}

#[test]
fn spire_heart_uses_the_canonical_final_act_operation() {
    let spire_heart = typed_event(3, "Spire Heart");
    assert!(matches!(
        spire_heart.options[0].program.ops.as_slice(),
        [EventProgramOp::ResolveFinalAct]
    ));
    assert!(matches!(
        spire_heart.options[0].status,
        EventRuntimeStatus::Supported
    ));
}
