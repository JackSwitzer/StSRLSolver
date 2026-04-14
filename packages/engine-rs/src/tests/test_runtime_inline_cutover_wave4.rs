use crate::actions::Action;
use crate::status_ids::sid;
use crate::tests::support::{engine_with, ensure_in_hand, make_deck};

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
fn manual_discard_engine_path_applies_reflex_and_tactician_without_registry_dispatch() {
    let mut engine = engine_with(make_deck(&["Reflex", "Tactician", "Strike_G"]), 40, 0);
    engine.state.hand = make_deck(&["Reflex", "Tactician"]);
    engine.state.draw_pile = make_deck(&["Strike_G", "Strike_G", "Strike_G"]);
    engine.state.discard_pile.clear();
    engine.state.energy = 1;

    let reflex = engine.state.hand.remove(0);
    engine.state.discard_pile.push(reflex);
    engine.on_card_discarded(reflex);

    assert_eq!(engine.state.hand.len(), 3);
    assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 1);

    let tactician = engine.state.hand.remove(
        engine
            .state
            .hand
            .iter()
            .position(|card| engine.card_registry.card_name(card.def_id) == "Tactician")
            .expect("Tactician should still be in hand after Reflex draw"),
    );
    engine.state.discard_pile.push(tactician);
    engine.on_card_discarded(tactician);

    assert_eq!(engine.state.energy, 2);
    assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 2);
}

#[test]
fn rage_legal_action_and_engine_path_still_work_after_inline_cutover() {
    let mut engine = engine_with(make_deck(&["Rage", "Strike_R"]), 50, 0);
    engine.state.hand = make_deck(&["Rage", "Strike_R"]);
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    ensure_in_hand(&mut engine, "Rage");
    ensure_in_hand(&mut engine, "Strike_R");

    assert!(engine.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. }
        if engine.card_registry.card_name(engine.state.hand[*card_idx].def_id) == "Rage"
    )));

    assert!(crate::tests::support::play_self(&mut engine, "Rage"));
    assert!(crate::tests::support::play_on_enemy(&mut engine, "Strike_R", 0));

    assert_eq!(engine.state.player.status(sid::RAGE), 3);
    assert_eq!(engine.state.player.block, 3);
}
