#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/ChampionBelt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/StrikeDummy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/WristBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/SneckoSkull.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/ChemicalX.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/VioletLotus.java
//
// The scalar relic helpers moved into the canonical engine/card-effect paths in
// this wave. The higher-latency relic bridges remain live production callsites
// in `engine.rs`, so they stay put until a wider runtime wave owns them.

use crate::status_ids::sid;
use crate::state::Stance;
use crate::tests::support::{
    combat_state_with, enemy_no_intent, ensure_in_hand, engine_with, engine_with_state, make_deck,
    make_deck_n, play_on_enemy, play_self,
};

#[test]
fn relic_dead_helper_cleanup_wave19_scalar_bonuses_now_live_on_engine_path() {
    let mut state = combat_state_with(
        make_deck(&["Strike_R", "Shiv"]),
        vec![enemy_no_intent("JawWorm", 80, 80)],
        3,
    );
    state.relics = vec![
        "StrikeDummy".to_string(),
        "WristBlade".to_string(),
        "Snake Skull".to_string(),
        "Champion Belt".to_string(),
    ];

    let mut engine = engine_with_state(state);
    ensure_in_hand(&mut engine, "Strike_R");
    ensure_in_hand(&mut engine, "Shiv");

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Strike_R", 0));
    assert!(play_on_enemy(&mut engine, "Shiv", 0));
    assert_eq!(hp_before - engine.state.enemies[0].entity.hp, 17);

    assert!(engine.apply_player_debuff_to_enemy(0, sid::POISON, 1));
    assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 2);

    assert!(engine.apply_player_debuff_to_enemy(0, sid::VULNERABLE, 1));
    assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
}

#[test]
fn relic_dead_helper_cleanup_wave19_x_cost_and_calm_exit_bonuses_are_engine_path_only() {
    let mut collect_engine = engine_with(make_deck_n("Collect", 10), 100, 0);
    collect_engine.state.relics.push("Chemical X".to_string());
    ensure_in_hand(&mut collect_engine, "Collect");
    assert!(play_self(&mut collect_engine, "Collect"));
    assert_eq!(collect_engine.state.player.status(sid::COLLECT_MIRACLES), 5);

    let mut lotus_engine = engine_with(make_deck_n("Eruption", 10), 100, 0);
    lotus_engine.state.relics.push("Violet Lotus".to_string());
    lotus_engine.state.stance = Stance::Calm;
    let energy_before = lotus_engine.state.energy;
    ensure_in_hand(&mut lotus_engine, "Eruption");
    assert!(play_self(&mut lotus_engine, "Eruption"));
    assert_eq!(lotus_engine.state.energy, energy_before + 1);
}
