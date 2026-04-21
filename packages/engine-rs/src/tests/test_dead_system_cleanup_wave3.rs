#![cfg(test)]

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck, make_deck_n};

#[test]
fn dead_cleanup_wave3_opening_state_relics_are_engine_path_authoritative() {
    let mut engine = engine_without_start(
        make_deck(&[
            "Strike",
            "Strike",
            "Strike",
            "Strike",
            "Strike",
            "Defend",
            "Defend",
            "Defend",
        ]),
        vec![
            enemy_no_intent("Hexaghost", 250, 250),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    );
    engine.state.player.hp = 50;
    engine.state.relics.extend([
        "TwistedFunnel".to_string(),
        "Snecko Eye".to_string(),
        "TeardropLocket".to_string(),
        "Pantograph".to_string(),
    ]);

    engine.start_combat();

    assert_eq!(engine.state.player.hp, 75);
    assert_eq!(engine.state.player.status(sid::CONFUSION), 1);
    assert_eq!(engine.state.player.status(sid::SNECKO_EYE), 1);
    assert_eq!(engine.state.player.status(sid::BAG_OF_PREP_DRAW), 2);
    assert_eq!(engine.state.stance, crate::state::Stance::Calm);
    assert!(engine
        .state
        .enemies
        .iter()
        .all(|enemy| enemy.entity.status(sid::POISON) == 4));
}

#[test]
fn dead_cleanup_wave3_flag_gated_and_zero_progress_relics_no_longer_need_helper_oracles() {
    let mut sling_engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    sling_engine.state.relics.push("Sling".to_string());
    sling_engine.start_combat();
    assert_eq!(sling_engine.state.player.strength(), 0);

    let mut du_vu_engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    du_vu_engine.state.relics.push("Du-Vu Doll".to_string());
    du_vu_engine.start_combat();
    assert_eq!(du_vu_engine.state.player.strength(), 0);

    let mut girya_engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    girya_engine.state.relics.push("Girya".to_string());
    girya_engine.start_combat();
    assert_eq!(girya_engine.state.player.strength(), 0);
}
