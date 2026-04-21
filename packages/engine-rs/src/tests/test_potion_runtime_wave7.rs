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
fn runtime_authority_covers_wave7_targetless_potions() {
    let ids = [
        "SwiftPotion",
        "EnergyPotion",
        "AncientPotion",
        "RegenPotion",
        "EssenceOfSteel",
        "LiquidBronze",
        "CultistPotion",
        "GhostInAJar",
        "DuplicationPotion",
        "SmokeBomb",
        "Swift Potion",
        "Energy Potion",
        "Ancient Potion",
        "Regen Potion",
        "Essence of Steel",
        "Liquid Bronze",
        "Cultist Potion",
        "Ghost in a Jar",
        "Duplication Potion",
        "Smoke Bomb",
    ];

    for id in ids {
        assert!(
            crate::potions::defs::potion_runtime_manual_activation_is_authoritative(id),
            "{id} should use the runtime-only production path"
        );
    }
}

#[test]
fn wave7_draw_and_energy_potions_use_action_path_with_runtime_potency() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Zap", "Dualcast"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand.clear();
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Zap", "Dualcast"]);

    engine.state.potions[0] = "Swift Potion".to_string();
    engine.clear_event_log();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.hand.len(), 3);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation && record.def_id == Some("SwiftPotion")
    }));

    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand.clear();
    engine.state.energy = 1;
    engine.state.potions[0] = "Energy Potion".to_string();
    engine.clear_event_log();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.energy, 5);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation && record.def_id == Some("EnergyPotion")
    }));
}

#[test]
fn wave7_status_potions_apply_expected_statuses_via_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));

    let cases = [
        ("Ancient Potion", sid::ARTIFACT, 1, "AncientPotion"),
        ("Regen Potion", sid::REGENERATION, 5, "RegenPotion"),
        ("EssenceOfSteel", sid::PLATED_ARMOR, 4, "EssenceOfSteel"),
        ("LiquidBronze", sid::THORNS, 3, "LiquidBronze"),
        ("CultistPotion", sid::RITUAL, 1, "CultistPotion"),
        ("GhostInAJar", sid::INTANGIBLE, 1, "GhostInAJar"),
        ("DuplicationPotion", sid::DUPLICATION, 1, "DuplicationPotion"),
    ];

    for (potion_id, status_id, status_amount, def_id) in cases {
        engine.state.player.set_status(sid::ARTIFACT, 0);
        engine.state.player.set_status(sid::REGENERATION, 0);
        engine.state.player.set_status(sid::PLATED_ARMOR, 0);
        engine.state.player.set_status(sid::THORNS, 0);
        engine.state.player.set_status(sid::RITUAL, 0);
        engine.state.player.set_status(sid::INTANGIBLE, 0);
        engine.state.player.set_status(sid::DUPLICATION, 0);
        engine.state.potions = vec![String::new(); 3];
        engine.state.potions[1] = potion_id.to_string();
        engine.clear_event_log();

        use_potion(&mut engine, 1, -1);

        assert_eq!(
            engine.state.player.status(status_id),
            status_amount,
            "{potion_id} should set the expected status via the runtime action path"
        );
        assert!(engine.state.potions[1].is_empty(), "{potion_id} should consume its slot");
        assert!(engine.event_log.iter().any(|record| {
            record.event == Trigger::ManualActivation
                && record.def_id == Some(def_id)
                && record.potion_slot == 1
        }));
    }
}

#[test]
fn wave7_smoke_bomb_flees_combat_via_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions[2] = "Smoke Bomb".to_string();
    engine.clear_event_log();

    let legal = engine.get_legal_actions();
    assert!(legal.contains(&Action::UsePotion {
        potion_idx: 2,
        target_idx: -1,
    }));

    use_potion(&mut engine, 2, -1);

    assert!(engine.state.combat_over, "Smoke Bomb should flee combat");
    assert!(engine.state.potions[2].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation
            && record.def_id == Some("SmokeBomb")
            && record.potion_slot == 2
    }));
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::OnPotionUsed && record.potion_slot == 2
    }));
}

#[test]
fn wave7_smoke_bomb_boss_legality_matches_java_can_use() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("The Guardian", 250, 250)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();

    let legal = engine.get_legal_actions();
    assert!(
        !legal.contains(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        }),
        "Smoke Bomb should be illegal against bosses like the Java canUse() path"
    );
}
