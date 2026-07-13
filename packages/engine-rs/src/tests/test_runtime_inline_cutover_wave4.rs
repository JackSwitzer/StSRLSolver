use crate::actions::Action;
use crate::status_ids::sid;
use crate::tests::support::{
    end_turn, engine_with, engine_without_start, enemy_no_intent, ensure_in_hand,
    force_player_turn, make_deck,
};

#[test]
fn body_slam_engine_path_uses_direct_damage_modifier_cutover() {
    let mut engine = engine_with(make_deck(&["Body Slam"]), 60, 0);
    engine.state.player.block = 17;
    engine.state.hand = make_deck(&["Body Slam"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(crate::tests::support::play_on_enemy(&mut engine, "Body Slam", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 17);
}

#[test]
fn rage_legal_action_and_engine_path_still_work_after_inline_cutover() {
    let mut engine = engine_with(make_deck(&["Rage", "Strike"]), 50, 0);
    engine.state.hand = make_deck(&["Rage", "Strike"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    ensure_in_hand(&mut engine, "Rage");
    ensure_in_hand(&mut engine, "Strike");

    assert!(engine.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. }
        if engine.card_registry.card_name(engine.state.hand[*card_idx].def_id) == "Rage"
    )));

    assert!(crate::tests::support::play_self(&mut engine, "Rage"));
    assert!(crate::tests::support::play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.player.status(sid::RAGE), 3);
    assert_eq!(engine.state.player.block, 3);
}

#[test]
fn rage_source_stacks_once_per_attack_card_and_expires_at_turn_end() {
    // Rage.java applies 3 (5 upgraded) RagePower. RagePower.onUseCard queues
    // one block gain for an Attack regardless of its hit count, then removes
    // the entire stacked power at the end of the turn.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 100, 100)],
        10,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Rage", "Rage+", "Defend", "Pummel"]);

    assert!(crate::tests::support::play_self(&mut engine, "Rage"));
    assert!(crate::tests::support::play_self(&mut engine, "Rage+"));
    assert_eq!(engine.state.player.status(sid::RAGE), 8);

    assert!(crate::tests::support::play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 5);
    assert!(crate::tests::support::play_on_enemy(&mut engine, "Pummel", 0));
    assert_eq!(engine.state.player.block, 13);
    assert_eq!(engine.state.enemies[0].entity.hp, 92);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.status(sid::RAGE), 0);
    engine.state.hand = make_deck(&["Strike"]);
    engine.state.energy = 3;
    assert!(crate::tests::support::play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.player.block, 0);
}
