#![cfg(test)]

use crate::actions::Action;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<&str> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect()
}

#[test]
fn runtime_manual_activation_registry_covers_wave4_potions() {
    let ids = [
        "Ambrosia",
        "StancePotion",
        "BlessingOfTheForge",
        "LiquidMemories",
        "DistilledChaos",
        "EntropicBrew",
    ];

    for id in ids {
        assert!(
            crate::potions::defs::potion_uses_runtime_manual_activation(id),
            "{id} should be runtime-backed for manual activation"
        );
    }
}

#[test]
fn stance_family_potions_surface_no_target_legal_actions_and_consume_slots() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "Ambrosia".to_string();
    engine.state.potions[1] = "StancePotion".to_string();

    let legal = engine.get_legal_actions();
    assert!(legal.contains(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    }));
    assert!(legal.contains(&Action::UsePotion {
        potion_idx: 1,
        target_idx: -1,
    }));
    assert!(!legal.iter().any(|action| matches!(
        action,
        Action::UsePotion { potion_idx: 0 | 1, target_idx } if *target_idx >= 0
    )));

    engine.state.stance = Stance::Neutral;
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.stance, Stance::Divinity);
    assert!(engine.state.potions[0].is_empty());

    engine.state.stance = Stance::Calm;
    use_potion(&mut engine, 1, -1);
    assert!(matches!(engine.phase, crate::engine::CombatPhase::AwaitingChoice));
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.stance, Stance::Wrath);
    assert!(engine.state.potions[1].is_empty());
}

#[test]
fn blessing_and_liquid_memories_respect_hand_limit_and_sacred_bark_via_engine_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off", "Inflame", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand = make_deck(&[
        "Strike_P",
        "Defend_P",
        "Bash",
        "Shrug It Off",
        "Inflame",
        "Zap",
        "Dualcast",
        "Strike_P",
        "Defend_P",
    ]);
    engine.state.discard_pile = make_deck(&["Strike_P", "Defend_P", "Bash"]);
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "BlessingOfTheForge".to_string();
    engine.state.potions[1] = "LiquidMemories".to_string();

    use_potion(&mut engine, 0, -1);
    assert!(hand_names(&engine)
        .into_iter()
        .all(|name| name.ends_with('+')));

    use_potion(&mut engine, 1, -1);
    assert!(matches!(engine.phase, crate::engine::CombatPhase::AwaitingChoice));
    engine.execute_action(&Action::Choose(2));
    engine.execute_action(&Action::Choose(1));
    engine.execute_action(&Action::ConfirmSelection);
    assert_eq!(engine.state.hand.len(), 10, "hand limit should cap Liquid Memories");
    assert_eq!(engine.state.discard_pile.len(), 2, "Sacred Bark should attempt two returns");
    assert_eq!(hand_names(&engine).last().copied(), Some("Bash"));
}

#[test]
fn distilled_chaos_and_entropic_brew_chain_through_runtime_slots() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off", "Inflame", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.draw_pile = make_deck(&[
        "Strike_P",
        "Defend_P",
        "Bash",
        "Shrug It Off",
        "Inflame",
        "Zap",
    ]);
    engine.state.hand.clear();
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "DistilledChaos".to_string();
    engine.state.potions[1] = "EntropicBrew".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.hand.len(), 1, "Shrug It Off should leave its drawn Bash in hand");
    assert_eq!(engine.state.draw_pile.len(), 0, "Sacred Bark should let Distilled Chaos exhaust the whole pile here");
    assert_eq!(hand_names(&engine), vec!["Bash"]);
    assert_eq!(engine.state.player.block, 13);
    assert_eq!(engine.state.enemies[0].entity.hp, 32);
    assert_eq!(engine.state.orb_slots.occupied_count(), 0);
    assert!(engine.state.potions[0].is_empty());

    engine.state.hand.clear();
    engine.state.potions[0] = String::new();
    engine.state.potions[1] = "EntropicBrew".to_string();
    engine.state.potions[2] = String::new();
    use_potion(&mut engine, 1, -1);

    assert!(engine.state.potions[1].is_empty(), "used Entropic Brew slot should be consumed");
    assert_eq!(engine.state.potions[0], "Block Potion");
    assert_eq!(engine.state.potions[2], "Block Potion");

    let legal = engine.get_legal_actions();
    assert!(legal.contains(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    }));
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.block, 37);
}

#[test]
fn wave4_runtime_potions_emit_manual_activation_records() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P", "Defend_P", "Bash", "Shrug It Off"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];
    engine.state.discard_pile = make_deck(&["Strike_P", "Defend_P"]);
    engine.state.draw_pile = make_deck(&["Bash", "Shrug It Off"]);

    let cases = [
        ("Ambrosia", 0usize),
        ("StancePotion", 1usize),
        ("LiquidMemories", 2usize),
    ];

    for (potion_id, slot) in cases {
        engine.clear_event_log();
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[slot] = potion_id.to_string();
        use_potion(&mut engine, slot, -1);

        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::ManualActivation
                && record.def_id == Some(potion_id)
        }));
        assert!(engine.event_log.iter().any(|record| {
            record.event == crate::effects::trigger::Trigger::OnPotionUsed
                && record.potion_slot == slot as i32
        }));
    }

    engine.state.player.set_status(sid::ORB_SLOTS, 0);
}
