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
        make_deck(&["Strike"]),
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
fn fruit_juice_uses_increase_max_hp_healing_semantics() {
    // Source-derived (verify potion/FruitJuice): FruitJuice.use calls
    // increaseMaxHp(potency, true), getPotency always returns five, and
    // AbstractCreature.increaseMaxHp raises maxHealth before calling heal.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FruitJuice.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    let mut state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    state.player.hp = 40;
    state.player.max_hp = 80;
    state.relics.push("SacredBark".to_string());
    state.player.set_status(sid::HAS_MAGIC_FLOWER, 1);
    state.potions[0] = "FruitJuice".to_string();
    let mut engine = engine_with_state(state);

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.player.max_hp, 90);
    assert_eq!(engine.state.player.hp, 55);
    assert!(engine.state.potions[0].is_empty());

    let mut blocked_state = combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    blocked_state.player.hp = 40;
    blocked_state.player.max_hp = 80;
    blocked_state.relics.push("SacredBark".to_string());
    blocked_state.player.set_status(sid::HAS_MARK_OF_BLOOM, 1);
    blocked_state.potions[0] = "Fruit Juice".to_string();
    let mut blocked = engine_with_state(blocked_state);

    use_potion(&mut blocked, 0, -1);

    assert_eq!(blocked.state.player.max_hp, 90);
    assert_eq!(blocked.state.player.hp, 40);
    assert!(blocked.state.potions[0].is_empty());
}

#[test]
fn block_potion_grants_raw_bark_doubled_block_without_card_modifiers() {
    // Source-derived (verify potion/BlockPotion): use() queues GainBlockAction
    // for the player with potency 12. Dexterity and Frail modify card block,
    // not this raw action; Sacred Bark doubles potion potency to 24.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.player.set_status(sid::DEXTERITY, 5);
    engine.state.player.set_status(sid::FRAIL, 1);
    engine.state.potions[0] = "Block Potion".to_string();

    use_potion(&mut engine, 0, 0);

    assert_eq!(engine.state.player.block, 24);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn dexterity_potion_keeps_full_potency_and_modifies_real_card_block() {
    // Source-derived (verify potion/DexterityPotion): Java targets the player
    // and applies DexterityPower(potency 2). Sacred Bark doubles that to four.
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.potions[0] = "Dexterity Potion".to_string();

    use_potion(&mut engine, 0, 0);
    assert_eq!(engine.state.player.dexterity(), 4);
    assert!(crate::tests::support::play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 9);
}

#[test]
fn wave6_simple_targeted_potions_use_runtime_action_path() {
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
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
        make_deck(&["Strike"]),
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

#[test]
fn explosive_potion_uses_java_pure_matrix_but_keeps_normal_hit_hooks() {
    // Source-derived (verify potion/ExplosivePotion): potency is ten at every
    // ascension and Sacred Bark doubles it. createDamageMatrix(..., true)
    // bypasses receiving damage modifiers, while NORMAL onAttacked hooks still
    // run after block is applied.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/ExplosivePotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/DamageInfo.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![
            enemy_no_intent("Byrd", 50, 50),
            enemy_no_intent("JawWorm", 50, 50),
        ],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.enemies[0].entity.set_status(sid::SLOW, 5);
    engine.state.enemies[0].entity.set_status(sid::FLIGHT, 3);
    engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 3);
    engine.state.enemies[0].entity.set_status(sid::INTANGIBLE, 1);
    engine.state.enemies[1].entity.block = 6;
    engine.state.potions[0] = "Explosive Potion".to_string();

    use_potion(&mut engine, 0, -1);

    assert_eq!(engine.state.enemies[0].entity.hp, 30);
    assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 2);
    assert_eq!(engine.state.enemies[1].entity.hp, 36);
    assert_eq!(engine.state.enemies[1].entity.block, 0);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn fear_potion_keeps_three_potency_and_only_debuffs_its_target() {
    // Source-derived (verify potion/FearPotion): targetRequired is true,
    // getPotency is always three, and Sacred Bark doubles the applied
    // player-sourced VulnerablePower to six.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FearPotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/AbstractPotion.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![
            enemy_no_intent("JawWorm", 40, 40),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.potions[0] = "FearPotion".to_string();

    let legal = engine.get_legal_actions();
    assert!(legal.contains(&Action::UsePotion {
        potion_idx: 0,
        target_idx: 1,
    }));
    assert!(!legal.contains(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    }));

    use_potion(&mut engine, 0, 1);

    assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 0);
    assert_eq!(engine.state.enemies[1].entity.status(sid::VULNERABLE), 6);
    assert_eq!(engine.state.enemies[1].entity.status(sid::VULNERABLE_JUST_APPLIED), 0);
    assert!(engine.state.potions[0].is_empty());
}

#[test]
fn fire_potion_uses_target_only_thorns_damage_and_constant_potency() {
    // Source-derived (verify potion/FirePotion): getPotency is always twenty,
    // Sacred Bark doubles it, and applyEnemyPowersOnly calculates THORNS
    // damage without NORMAL-only modifiers or reactions.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FirePotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/DamageInfo.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("Byrd", 100, 100)],
        3,
    ));
    engine.state.relics.push("SacredBark".to_string());
    engine.state.enemies[0].entity.set_status(sid::SLOW, 5);
    engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 3);
    engine.state.enemies[0].entity.set_status(sid::FLIGHT, 3);
    engine.state.enemies[0].entity.set_status(sid::CURL_UP, 8);
    engine.state.enemies[0].entity.set_status(sid::MALLEABLE, 3);
    engine.state.potions[0] = "Fire Potion".to_string();

    use_potion(&mut engine, 0, 0);

    assert_eq!(engine.state.enemies[0].entity.hp, 60);
    assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CURL_UP), 8);
    assert_eq!(engine.state.enemies[0].entity.status(sid::MALLEABLE), 3);

    let mut intangible = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("Nemesis", 50, 50)],
        3,
    ));
    intangible.state.enemies[0].entity.set_status(sid::INTANGIBLE, 1);
    intangible.state.potions[0] = "Fire Potion".to_string();
    use_potion(&mut intangible, 0, 0);
    assert_eq!(intangible.state.enemies[0].entity.hp, 49);
}
