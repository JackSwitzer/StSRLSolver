#![cfg(test)]

// Java sources:
// - decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
// - decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
// - decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/{GamblingChip,WarpedTongs,
//   MercuryHourglass,Brimstone,HornCleat,Toolbox,Lantern,
//   MutagenicStrength}.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ChooseOneColorless.java
// - decompiled/java-src/com/megacrit/cardcrawl/powers/{MayhemPower,
//   ToolsOfTheTradePower}.java
// - decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/ForesightPower.java

use crate::actions::Action;
use crate::effects::declarative::GeneratedCardPool;
use crate::engine::{ChoiceOption, ChoiceReason, CombatEngine, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck_n,
};

fn opening_with(relics: &[&str]) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine_with_state(state)
}

fn has_upgraded_hand_card(engine: &crate::engine::CombatEngine) -> bool {
    engine
        .state
        .hand
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id).ends_with('+'))
}

#[test]
fn gambling_then_warped_pauses_before_upgrade_and_resumes_it() {
    let mut engine = opening_with(&["Gambling Chip", "WarpedTongs"]);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert!(!has_upgraded_hand_card(&engine));
    engine.execute_action(&Action::ConfirmSelection);
    assert!(has_upgraded_hand_card(&engine));
}

#[test]
fn warped_then_gambling_upgrades_before_discard_choice() {
    let engine = opening_with(&["WarpedTongs", "Gambling Chip"]);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert!(has_upgraded_hand_card(&engine));
}

#[test]
fn opening_gambling_resumes_into_foresight_without_losing_callback() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics.push("Gambling Chip".to_string());
    state.player.set_status(sid::FORESIGHT, 1);
    let mut engine = engine_with_state(state);
    assert_eq!(
        engine.choice.as_ref().unwrap().reason,
        ChoiceReason::DiscardFromHand
    );
    engine.execute_action(&Action::ConfirmSelection);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::Scry);
}

#[test]
fn foresight_resumes_into_tools_in_power_order_on_later_turn() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 30),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.player.set_status(sid::FORESIGHT, 1);
    state.player.set_status(sid::TOOLS_OF_THE_TRADE, 1);
    let mut engine = engine_with_state(state);
    // Opening invokes atStartOfTurn powers but not post-draw powers.
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::Scry);
    engine.execute_action(&Action::ConfirmSelection);
    end_turn(&mut engine);
    assert_eq!(engine.choice.as_ref().unwrap().reason, ChoiceReason::Scry);
    engine.execute_action(&Action::ConfirmSelection);
    assert_eq!(
        engine.choice.as_ref().unwrap().reason,
        ChoiceReason::DiscardFromHand
    );
}

#[test]
fn mercury_then_brimstone_collects_top_strength_before_lethal_bottom_damage() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 3, 3)],
        3,
    );
    state.relics = vec!["Mercury Hourglass".to_string(), "Brimstone".to_string()];
    let engine = engine_with_state(state);
    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.status(sid::STRENGTH), 2);
    assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), 1);
}

#[test]
fn mercury_lethal_preserves_later_horn_cleat_block_action() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 6, 6)],
        3,
    );
    state.relics = vec!["Mercury Hourglass".to_string(), "HornCleat".to_string()];
    let mut engine = engine_with_state(state);
    assert_eq!(engine.state.enemies[0].entity.hp, 3);
    end_turn(&mut engine);
    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.block, 14);
}

#[test]
fn opening_toolbox_choice_observes_lantern_and_master_energy() {
    let engine = opening_with(&["Toolbox", "Lantern"]);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.energy, 4);
}

#[test]
fn opening_toolbox_choice_observes_brimstone_top_action() {
    let engine = opening_with(&["Toolbox", "Brimstone"]);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.player.status(sid::STRENGTH), 2);
}

#[test]
fn opening_ice_cream_starts_at_master_energy_not_double() {
    let engine = opening_with(&["Ice Cream"]);
    assert_eq!(engine.state.energy, 3);
}

#[test]
fn opening_does_not_invoke_post_draw_power_callbacks() {
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.player.set_status(sid::DEMON_FORM, 2);
    let engine = engine_with_state(state);
    assert_eq!(engine.state.player.status(sid::STRENGTH), 0);
}

#[test]
fn toolbox_rolls_after_callback_time_card_random_consumers() {
    // Toolbox only constructs ChooseOneColorless during atBattleStartPreDraw;
    // that action generates its three choices later in update(). HelloPower
    // chooses its random common card synchronously during the intervening
    // atStartOfTurn callback, so its cardRandom draw must occur first.
    // Java: relics/Toolbox.java, actions/utility/ChooseOneColorless.java,
    // and powers/HelloPower.java.
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics.push("Toolbox".to_string());
    state.player.set_status(sid::HELLO_WORLD, 1);

    let mut engine = CombatEngine::new(state, crate::tests::support::TEST_SEED);
    let mut oracle = engine.clone();
    crate::effects::interpreter::generate_random_card(&mut oracle, GeneratedCardPool::DefectCommon)
        .expect("Hello World source pool is nonempty");
    let expected = crate::effects::interpreter::generate_unique_random_cards(
        &mut oracle,
        GeneratedCardPool::Colorless,
        3,
    )
    .into_iter()
    .map(|card| oracle.card_registry.card_name(card.def_id).to_string())
    .collect::<Vec<_>>();

    engine.start_combat();
    let actual = engine
        .choice
        .as_ref()
        .expect("Toolbox opens its queued choice")
        .options
        .iter()
        .map(|option| match option {
            ChoiceOption::GeneratedCard(card) => {
                engine.card_registry.card_name(card.def_id).to_string()
            }
            other => panic!("unexpected Toolbox option {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(
        engine.card_random_rng.counter,
        oracle.card_random_rng.counter
    );
}

#[test]
fn combat_start_top_preserves_multi_effect_callback_drain_order() {
    // MutagenicStrength queues Strength with addToTop, then LoseStrength with
    // addToTop. The LIFO manager therefore applies LoseStrength first and
    // Strength second. Front-inserting a preordered Rust effect batch must not
    // reverse that callback a second time.
    // Java: relics/MutagenicStrength.java::atBattleStart.
    let engine = opening_with(&["MutagenicStrength"]);
    let ordered = engine
        .state
        .player
        .power_order
        .iter()
        .filter_map(|entry| match entry {
            crate::state::PowerOrderEntry::Status(status)
                if matches!(*status, sid::LOSE_STRENGTH | sid::STRENGTH) =>
            {
                Some(*status)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ordered, [sid::LOSE_STRENGTH, sid::STRENGTH]);
}
