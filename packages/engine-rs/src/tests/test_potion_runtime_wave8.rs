#![cfg(test)]

use crate::actions::Action;
use crate::effects::trigger::Trigger;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, make_deck};

fn use_potion(engine: &mut crate::engine::CombatEngine, potion_idx: usize, target_idx: i32) {
    engine.execute_action(&Action::UsePotion {
        potion_idx,
        target_idx,
    });
}

#[test]
fn wave8_runtime_authority_covers_manual_activation_cutover_bundle() {
    let ids = [
        "Ambrosia",
        "BlessingOfTheForge",
        "BottledMiracle",
        "EntropicBrew",
        "EssenceOfDarkness",
        "GamblersBrew",
        "LiquidMemories",
        "PotionOfCapacity",
        "StancePotion",
        "SmokeBomb",
        "Blessing of the Forge",
        "Bottled Miracle",
        "Entropic Brew",
        "Essence of Darkness",
        "Gambler's Brew",
        "Liquid Memories",
        "Potion of Capacity",
        "Stance Potion",
        "Smoke Bomb",
    ];

    for id in ids {
        assert!(
            crate::potions::defs::potion_runtime_manual_activation_is_authoritative(id),
            "{id} should be runtime-authoritative after wave 8"
        );
    }
}

#[test]
fn wave8_blessing_bottled_and_liquid_memories_stay_slot_scoped_on_action_path() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/BlessingOfTheForge.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/BottledMiracle.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidMemories.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Bash", "Zap"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.discard_pile = make_deck(&["Bash"]);
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "BlessingOfTheForge".to_string();
    engine.state.potions[1] = "BottledMiracle".to_string();
    engine.state.potions[2] = "LiquidMemories".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);
    assert_eq!(
        engine
            .state
            .hand
            .iter()
            .map(|card| engine.card_registry.card_name(card.def_id))
            .collect::<Vec<_>>(),
        vec!["Strike+", "Defend+"]
    );
    assert!(engine.state.potions[0].is_empty());

    use_potion(&mut engine, 1, -1);
    assert_eq!(
        engine
            .state
            .hand
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Miracle")
            .count(),
        2
    );
    assert!(engine.state.potions[1].is_empty());

    use_potion(&mut engine, 2, -1);
    assert!(
        engine
            .state
            .hand
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Bash")
    );
    assert!(engine.state.potions[2].is_empty());
    assert_eq!(engine.state.discard_pile.len(), 0);
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation
            && record.def_id == Some("LiquidMemories")
            && record.potion_slot == 2
    }));
}

#[test]
fn wave8_ambrosia_entropic_darkness_capacity_and_gamblers_brew_use_runtime_path() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/Ambrosia.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/EntropicBrew.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/EssenceOfDarkness.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/GamblersBrew.java
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/PotionOfCapacity.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike", "Defend", "Zap", "Dualcast", "Bash"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.init_defect_orbs(2);
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.draw_pile = make_deck(&["Zap", "Dualcast", "Bash"]);
    engine.state.potions = vec![String::new(); 3];
    engine.state.potions[0] = "Ambrosia".to_string();
    engine.state.potions[1] = "EssenceOfDarkness".to_string();
    engine.state.potions[2] = "GamblersBrew".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.stance, Stance::Divinity);

    use_potion(&mut engine, 1, -1);
    assert!(engine
        .state
        .orb_slots
        .slots
        .iter()
        .all(|orb| orb.orb_type == crate::orbs::OrbType::Dark));

    use_potion(&mut engine, 2, -1);
    assert_eq!(engine.state.player.status(sid::POTION_DRAW), 0);
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(engine.state.discard_pile.len(), 2);

    engine.state.potions[0] = "PotionOfCapacity".to_string();
    engine.state.potions[1] = "EntropicBrew".to_string();
    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 2);
    use_potion(&mut engine, 1, -1);
    assert!(engine.state.potions[1].is_empty());
    assert!(!engine.state.potions[2].is_empty());
}

#[test]
fn wave8_smoke_bomb_flees_on_the_canonical_runtime_path() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();
    engine.clear_event_log();

    use_potion(&mut engine, 0, -1);

    assert!(engine.state.combat_over);
    assert!(engine.state.potions[0].is_empty());
    assert!(engine.event_log.iter().any(|record| {
        record.event == Trigger::ManualActivation && record.def_id == Some("SmokeBomb")
    }));
}

#[test]
fn wave8_stance_potion_matches_java_choose_one_semantics() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/StancePotion.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.stance = Stance::Neutral;
    engine.state.potions[0] = "StancePotion".to_string();

    use_potion(&mut engine, 0, -1);
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Stance Potion should open a choice");
    let labels: Vec<&str> = choice
        .options
        .iter()
        .filter_map(|opt| match opt {
            crate::engine::ChoiceOption::Named(label) => Some(*label),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["Wrath", "Calm"]);
}

#[test]
fn wave8_smoke_bomb_respects_java_can_use_restrictions() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("The Guardian", 250, 250)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();

    assert!(
        !engine.get_legal_actions().contains(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        })
    );
}

#[test]
fn wave8_smoke_bomb_stays_legal_against_normal_enemies() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();

    assert!(engine.get_legal_actions().contains(&Action::UsePotion {
        potion_idx: 0,
        target_idx: -1,
    }));
}

#[test]
fn wave8_smoke_bomb_rejects_back_attack_non_boss_enemies() {
    // Java oracle:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java
    // - decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Strike"]),
        vec![{
            let mut enemy = enemy_no_intent("JawWorm", 40, 40);
            enemy.back_attack = true;
            enemy
        }],
        3,
    ));
    engine.state.potions[0] = "Smoke Bomb".to_string();

    assert!(
        !engine.get_legal_actions().contains(&Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        })
    );
}
