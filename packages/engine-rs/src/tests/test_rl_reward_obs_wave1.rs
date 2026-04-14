#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/rooms/TreasureRoom.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/Matryoshka.java

use crate::decision::{RewardItem, RewardItemKind, RewardItemState, RewardScreen, RewardScreenSource};
use crate::obs::{get_observation, RUN_DECISION_TAIL_OFFSET};
use crate::run::RunEngine;

#[test]
fn treasure_reward_screen_is_explicitly_visible_in_rl_observation() {
    let mut engine = RunEngine::new(42, 20);
    engine.run_state.relics.push("Matryoshka".to_string());
    engine.run_state.relic_flags.rebuild(&engine.run_state.relics);
    engine.run_state.relic_flags.init_relic_counter("Matryoshka");
    engine.debug_build_treasure_reward_screen();

    let screen = engine
        .current_reward_screen()
        .expect("treasure reward screen should exist");
    assert_eq!(screen.items.len(), 3, "Matryoshka should add a second relic");

    let obs = get_observation(&engine);
    assert_eq!(obs[RUN_DECISION_TAIL_OFFSET + 6], 1.0);
    assert_eq!(obs[RUN_DECISION_TAIL_OFFSET + 7], 0.0);
    assert_eq!(obs[RUN_DECISION_TAIL_OFFSET + 8], 0.6);
}

#[test]
fn unknown_non_treasure_reward_screens_do_not_fabricate_treasure_source() {
    let mut engine = RunEngine::new(42, 20);
    engine.debug_set_reward_screen(RewardScreen {
        source: RewardScreenSource::Unknown,
        ordered: true,
        active_item: None,
        items: vec![RewardItem {
            index: 0,
            kind: RewardItemKind::CardChoice,
            state: RewardItemState::Available,
            label: "card_reward".to_string(),
            claimable: true,
            active: false,
            skip_allowed: true,
            skip_label: Some("Skip".to_string()),
            choices: Vec::new(),
        }],
    });

    let obs = get_observation(&engine);
    assert_eq!(obs[RUN_DECISION_TAIL_OFFSET + 6], 0.0);
    assert_eq!(obs[RUN_DECISION_TAIL_OFFSET + 8], 0.2);
}
