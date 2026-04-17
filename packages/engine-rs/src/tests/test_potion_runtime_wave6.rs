#![cfg(test)]

use crate::actions::Action;
use crate::effects::trigger::Trigger;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

#[test]
fn runtime_authority_covers_wave6_simple_combat_potions() {
    let ids = [
        "BlockPotion",
        "DexterityPotion",
        "ExplosivePotion",
        "FearPotion",
        "FirePotion",
        "PoisonPotion",
        "StrengthPotion",
        "WeakenPotion",
    ];

    for id in ids {
        assert!(
            crate::potions::defs::potion_runtime_manual_activation_is_authoritative(id),
            "{id} should use the runtime-only production path"
        );
    }
}

#[test]
fn wave6_simple_self_buff_potions_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases = [
        ("Block Potion", sid::ARTIFACT, 0, 12, "BlockPotion"),
        ("Strength Potion", sid::STRENGTH, 2, 0, "StrengthPotion"),
        ("Dexterity Potion", sid::DEXTERITY, 2, 0, "DexterityPotion"),
    ];

    for (potion_id, status_id, status_amount, block_amount, def_id) in cases {
        engine.state.player.block = 0;
        engine.state.player.set_status(sid::STRENGTH, 0);
        engine.state.player.set_status(sid::DEXTERITY, 0);
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[0] = potion_id.to_string();
        engine.clear_event_log();

        let legal = engine.get_legal_actions();
        assert!(legal.contains(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        }));

        use_potion(&mut engine, 0, -1);

        if block_amount > 0 {
            assert_eq!(engine.state.player.block, block_amount, "{potion_id} should grant block");
        } else {
            assert_eq!(
                engine.state.player.status(status_id),
                status_amount,
                "{potion_id} should add the expected status"
            );
        }
        assert!(engine.state.potions[0].is_empty(), "{potion_id} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == Trigger::ManualActivation && record.def_id == Some(def_id)
        }));
    }
}

#[test]
fn wave6_simple_targeted_potions_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Louse", 30, 30)],
        3,
    ));

    let cases = [
        ("Fire Potion", "FirePotion"),
        ("Weak Potion", "WeakenPotion"),
        ("Fear Potion", "FearPotion"),
        ("Poison Potion", "PoisonPotion"),
    ];

    for (potion_id, def_id) in cases {
        engine.state.enemies[0].entity.hp = engine.state.enemies[0].entity.max_hp;
        engine.state.enemies[0].entity.set_status(sid::WEAKENED, 0);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 0);
        engine.state.enemies[0].entity.set_status(sid::POISON, 0);
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[1] = potion_id.to_string();
        engine.clear_event_log();

        let legal = engine.get_legal_actions();
        assert!(legal.contains(&Action::UsePotion {
            potion_idx: 1,
            target_idx: 0,
        }), "{potion_id} should enumerate targeted use");
        assert!(!legal.contains(&Action::UsePotion {
            potion_idx: 1,
            target_idx: -1,
        }), "{potion_id} should require a target");

        use_potion(&mut engine, 1, 0);

        match potion_id {
            "Fire Potion" => assert_eq!(engine.state.enemies[0].entity.hp, 20),
            "Weak Potion" => assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 3),
            "Fear Potion" => assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 3),
            "Poison Potion" => assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 6),
            _ => unreachable!(),
        }

        assert!(engine.state.potions[1].is_empty(), "{potion_id} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == Trigger::ManualActivation
                && record.def_id == Some(def_id)
                && record.potion_slot == 1
        }));
    }
}

#[test]
fn wave6_simple_all_enemy_potions_respect_sacred_bark_via_runtime_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Louse", 30, 30)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "Explosive Potion".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.enemies[0].entity.hp, 20);
    assert_eq!(engine.state.enemies[1].entity.hp, 10);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation && record.def_id == Some("ExplosivePotion")
    }));
}
