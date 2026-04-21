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

fn equip_potion(engine: &mut crate::engine::CombatEngine, slot: usize, potion_id: &str) {
    crate::potions::equip_potion_slot(engine, slot, potion_id);
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
fn test_potion_runtime_wave3_registry_exports_are_canonical() {
    let ids = [
        "Ambrosia",
        "StancePotion",
        "PotionOfCapacity",
        "BottledMiracle",
        "CunningPotion",
        "BlessingOfTheForge",
        "Elixir",
        "LiquidMemories",
        "GamblersBrew",
        "DistilledChaos",
    ];

    for id in ids {
        assert!(
            crate::potions::defs::potion_uses_runtime_manual_activation(id),
            "{id} should advertise runtime manual activation"
        );
    }
}

#[test]
fn test_potion_runtime_wave3_slot_scoped_activation_rebuilds_runtime_on_each_equip() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];

    equip_potion(&mut engine, 0, "Ambrosia");
    engine.state.stance = Stance::Calm;
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.stance, Stance::Divinity);
    assert!(engine.state.potions[0].is_empty());

    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];
    equip_potion(&mut engine, 0, "PotionOfCapacity");

    engine.state.stance = Stance::Calm;
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 2);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn test_potion_runtime_wave3_generated_cards_and_upgrade_behaviors() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];

    equip_potion(&mut engine, 0, "BottledMiracle");
    engine.state.hand.clear();
    use_potion(&mut engine, 0, -1);
    assert_eq!(hand_names(&engine), vec!["Miracle", "Miracle"]);
    assert!(engine.state.potions[0].is_empty());

    engine.state.hand.clear();
    engine.state.discard_pile.clear();
    equip_potion(&mut engine, 0, "CunningPotion");
    use_potion(&mut engine, 0, -1);
    assert_eq!(hand_names(&engine), vec!["Shiv", "Shiv", "Shiv"]);

    engine.state.hand = make_deck(&["Strike", "Defend"]);
    equip_potion(&mut engine, 0, "BlessingOfTheForge");
    use_potion(&mut engine, 0, -1);
    assert_eq!(hand_names(&engine), vec!["Strike+", "Defend+"]);

    engine.state.hand = make_deck(&["Strike", "Defend", "Bash"]);
    equip_potion(&mut engine, 0, "Elixir");
    use_potion(&mut engine, 0, -1);
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.exhaust_pile.len(), 3);
}

#[test]
fn test_potion_runtime_wave3_discard_draw_and_randomized_draw_behaviors() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions = vec![String::new(); 3];

    engine.state.discard_pile = make_deck(&["Strike", "Defend", "Bash"]);
    engine.state.hand.clear();
    equip_potion(&mut engine, 0, "LiquidMemories");
    use_potion(&mut engine, 0, -1);
    engine.execute_action(&Action::Choose(2));
    assert_eq!(hand_names(&engine), vec!["Bash"]);
    assert_eq!(engine.state.discard_pile.len(), 2);

    engine.state.hand = make_deck(&["Strike", "Defend", "Bash"]);
    equip_potion(&mut engine, 0, "GamblersBrew");
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.hand.len(), 3);
    assert!(engine.state.discard_pile.is_empty());
    assert_eq!(engine.state.player.status(sid::POTION_DRAW), 0);

    engine.init_defect_orbs(3);
    engine.state.hand.clear();
    engine.state.discard_pile.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Zap"]);
    equip_potion(&mut engine, 0, "DistilledChaos");
    use_potion(&mut engine, 0, -1);
    assert!(engine.state.hand.is_empty());
    assert_eq!(engine.state.draw_pile.len(), 0);
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.enemies[0].entity.hp, 34);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
}
