#![cfg(test)]

use crate::actions::Action;
use crate::effects::trigger::Trigger;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy_no_intent, engine_with_state, make_deck, play_self,
};

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
fn energy_potion_keeps_two_potency_and_sacred_bark_doubles_it() {
    // Source-derived (verify potion/EnergyPotion): getPotency returns two with
    // no ascension branch; AbstractPotion doubles that value for Sacred Bark.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/EnergyPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/AbstractPotion.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.energy = 1;
    engine.state.potions[0] = "Energy Potion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.energy, 3);

    engine.state.energy = 1;
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "Energy Potion".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.energy, 5);
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
fn ancient_potion_targets_player_and_uses_sacred_bark_potency() {
    // Source-derived (verify potion/AncientPotion): Java overwrites `target`
    // with the player and applies ArtifactPower(potency). Base potency is one.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "Ancient Potion".to_string();

    use_potion(&mut engine, 0, 0);

    assert_eq!(engine.state.player.status(sid::ARTIFACT), 2);
    assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn liquid_bronze_keeps_three_potency_and_retaliates_per_attack_hit() {
    // Source-derived (verify potion/LiquidBronze): getPotency always returns
    // three and use applies ThornsPower to the player. Sacred Bark doubles the
    // applied amount, while ThornsPower retaliates once for every qualifying
    // onAttacked call.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidBronze.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ThornsPower.java
    let mut attacker = enemy_no_intent("JawWorm", 40, 40);
    attacker.set_move(1, 1, 2, 0);
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![attacker],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "LiquidBronze".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.player.status(sid::THORNS), 6);
    end_turn(&mut engine);
    assert_eq!(engine.state.enemies[0].entity.hp, 28);
}

#[test]
fn essence_of_steel_keeps_four_potency_and_targets_the_player() {
    // Source-derived (verify potion/EssenceOfSteel): Java overwrites target
    // with the player, applies four Plated Armor at every ascension, and
    // AbstractPotion doubles the potency for Sacred Bark.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/EssenceOfSteel.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/AbstractPotion.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "EssenceOfSteel".to_string();

    use_potion(&mut engine, 0, 0);

    assert_eq!(engine.state.player.status(sid::PLATED_ARMOR), 8);
    assert_eq!(engine.state.enemies[0].entity.status(sid::PLATED_ARMOR), 0);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn cultist_potion_ritual_grows_strength_at_player_turn_end_only() {
    // Source-derived (verify potion/CultistPotion): the potion applies
    // RitualPower(player, potency, true). RitualPower gains that Strength at
    // each player turn end; Sacred Bark doubles base potency one to two.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "CultistPotion".to_string();

    use_potion(&mut engine, 0, 0);
    assert_eq!(engine.state.player.status(sid::RITUAL), 2);
    assert_eq!(engine.state.player.strength(), 0);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.strength(), 2);
    assert_eq!(engine.state.player.status(sid::RITUAL), 2);
}

#[test]
fn duplication_potion_replays_each_non_purge_card_and_consumes_one_charge() {
    // Source-derived (verify potion/DuplicationPotion): base potency is one,
    // Sacred Bark doubles it to two, and DuplicationPower copies every card
    // type once while consuming one charge per original card.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/DuplicationPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DuplicationPower.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Defend", "Inflame"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Defend", "Inflame"]);
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "DuplicationPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::DUPLICATION), 2);

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 10);
    assert_eq!(engine.state.player.status(sid::DUPLICATION), 1);

    assert!(play_self(&mut engine, "Inflame"));
    assert_eq!(engine.state.player.strength(), 4);
    assert_eq!(engine.state.player.status(sid::DUPLICATION), 0);
}

#[test]
fn unused_duplication_charge_expires_at_end_of_round() {
    // DuplicationPower.atEndOfRound reduces the power by exactly one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DuplicationPower.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "DuplicationPotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::DUPLICATION), 2);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::DUPLICATION), 1);
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
