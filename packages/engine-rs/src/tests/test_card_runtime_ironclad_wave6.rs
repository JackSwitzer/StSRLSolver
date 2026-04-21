#![cfg(test)]

// Java oracle references for this wave:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Cleave.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Clash.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/HeavyBlade.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/IronWave.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Carnage.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Impervious.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/PerfectedStrike.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Bludgeon.java

use crate::actions::Action;
use crate::cards::{CardTarget, CardType, global_registry};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent(enemy_id, hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

fn two_enemy_engine(
    a: (&str, i32),
    b: (&str, i32),
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent(a.0, a.1, a.1),
            enemy_no_intent(b.0, b.1, b.1),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine
}

#[test]
fn ironclad_wave6_registry_exports_honest_runtime_surface() {
    let cleave = global_registry().get("Cleave").expect("Cleave should exist");
    assert_eq!(cleave.card_type, CardType::Attack);
    assert_eq!(cleave.target, CardTarget::AllEnemy);
    assert_eq!(
        cleave.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::AllEnemies,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );

    let clash = global_registry().get("Clash").expect("Clash should exist");
    assert_eq!(clash.card_type, CardType::Attack);
    assert_eq!(clash.target, CardTarget::Enemy);
    assert!(clash.has_test_marker("only_attacks_in_hand"));
    assert_eq!(
        clash.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );

    let heavy_blade = global_registry()
        .get("Heavy Blade+")
        .expect("Heavy Blade+ should exist");
    assert_eq!(heavy_blade.base_magic, 5);
    assert!(heavy_blade.has_test_marker("heavy_blade"));
    assert_eq!(
        heavy_blade.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );

    let iron_wave = global_registry().get("Iron Wave").expect("Iron Wave should exist");
    assert_eq!(iron_wave.card_type, CardType::Attack);
    assert_eq!(iron_wave.target, CardTarget::Enemy);
    assert_eq!(iron_wave.base_damage, 5);
    assert_eq!(iron_wave.base_block, 5);
    assert_eq!(
        iron_wave.effect_data,
        &[
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::GainBlock(
                    crate::effects::declarative::AmountSource::Block,
                ),
            ),
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::DealDamage(
                    crate::effects::declarative::Target::SelectedEnemy,
                    crate::effects::declarative::AmountSource::Damage,
                ),
            ),
        ]
    );

    let carnage = global_registry().get("Carnage").expect("Carnage should exist");
    assert!(carnage.has_test_marker("ethereal"));
    assert_eq!(
        carnage.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );

    let impervious = global_registry()
        .get("Impervious+")
        .expect("Impervious+ should exist");
    assert_eq!(impervious.card_type, CardType::Skill);
    assert_eq!(impervious.target, CardTarget::SelfTarget);
    assert!(impervious.exhaust);
    assert_eq!(impervious.base_block, 40);
    assert_eq!(
        impervious.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::GainBlock(
                crate::effects::declarative::AmountSource::Block,
            ),
        )]
    );

    let perfected_strike = global_registry()
        .get("Perfected Strike")
        .expect("Perfected Strike should exist");
    assert!(perfected_strike.has_test_marker("perfected_strike"));
    assert_eq!(
        perfected_strike.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );

    let bludgeon = global_registry().get("Bludgeon").expect("Bludgeon should exist");
    assert_eq!(bludgeon.card_type, CardType::Attack);
    assert_eq!(bludgeon.target, CardTarget::Enemy);
    assert_eq!(bludgeon.base_damage, 32);
    assert_eq!(
        bludgeon.effect_data,
        &[crate::effects::declarative::Effect::Simple(
            crate::effects::declarative::SimpleEffect::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                crate::effects::declarative::AmountSource::Damage,
            ),
        )]
    );
}

#[test]
fn ironclad_wave6_cleave_and_bludgeon_follow_the_attack_preamble() {
    let mut cleave = two_enemy_engine(("JawWorm", 40), ("Cultist", 35));
    ensure_in_hand(&mut cleave, "Cleave");
    assert!(play_on_enemy(&mut cleave, "Cleave", 0));
    assert_eq!(cleave.state.enemies[0].entity.hp, 32);
    assert_eq!(cleave.state.enemies[1].entity.hp, 27);

    let mut bludgeon = one_enemy_engine("JawWorm", 70);
    ensure_in_hand(&mut bludgeon, "Bludgeon");
    assert!(play_on_enemy(&mut bludgeon, "Bludgeon", 0));
    assert_eq!(bludgeon.state.enemies[0].entity.hp, 38);
}

#[test]
fn ironclad_wave6_clash_and_heavy_blade_cover_legality_and_strength_scaling() {
    let mut blocked = one_enemy_engine("JawWorm", 50);
    blocked.state.hand = make_deck(&["Clash", "Defend"]);
    let clash_idx = blocked
        .state
        .hand
        .iter()
        .position(|card| blocked.card_registry.card_name(card.def_id) == "Clash")
        .expect("Clash should be in hand");
    assert!(!blocked.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard {
            card_idx,
            target_idx: 0,
        } if *card_idx == clash_idx
    )));

    let mut allowed = one_enemy_engine("JawWorm", 50);
    allowed.state.hand = make_deck(&["Clash", "Strike"]);
    assert!(play_on_enemy(&mut allowed, "Clash", 0));
    assert_eq!(allowed.state.enemies[0].entity.hp, 36);

    let mut heavy_blade = one_enemy_engine("JawWorm", 80);
    ensure_in_hand(&mut heavy_blade, "Heavy Blade+");
    heavy_blade.state.player.set_status(sid::STRENGTH, 2);
    assert!(play_on_enemy(&mut heavy_blade, "Heavy Blade+", 0));
    assert_eq!(heavy_blade.state.enemies[0].entity.hp, 56);
}

#[test]
fn ironclad_wave6_iron_wave_carnage_and_impervious_cover_block_ethereal_and_exhaust() {
    let mut iron_wave = one_enemy_engine("JawWorm", 40);
    ensure_in_hand(&mut iron_wave, "Iron Wave");
    assert!(play_on_enemy(&mut iron_wave, "Iron Wave", 0));
    assert_eq!(iron_wave.state.player.block, 5);
    assert_eq!(iron_wave.state.enemies[0].entity.hp, 35);

    let mut carnage_played = one_enemy_engine("JawWorm", 60);
    ensure_in_hand(&mut carnage_played, "Carnage");
    assert!(play_on_enemy(&mut carnage_played, "Carnage", 0));
    assert_eq!(carnage_played.state.enemies[0].entity.hp, 40);
    assert_eq!(discard_prefix_count(&carnage_played, "Carnage"), 1);

    let mut carnage_held = one_enemy_engine("JawWorm", 60);
    ensure_in_hand(&mut carnage_held, "Carnage");
    end_turn(&mut carnage_held);
    assert_eq!(exhaust_prefix_count(&carnage_held, "Carnage"), 1);
    assert_eq!(discard_prefix_count(&carnage_held, "Carnage"), 0);

    let mut impervious = one_enemy_engine("JawWorm", 60);
    ensure_in_hand(&mut impervious, "Impervious");
    assert!(play_self(&mut impervious, "Impervious"));
    assert_eq!(impervious.state.player.block, 30);
    assert_eq!(exhaust_prefix_count(&impervious, "Impervious"), 1);
}

#[test]
fn ironclad_wave6_perfected_strike_registry_stays_honest_while_engine_path_keeps_current_scope() {
    let mut engine = one_enemy_engine("JawWorm", 80);
    engine.state.hand = make_deck(&["Perfected Strike", "Strike"]);
    engine.state.draw_pile = make_deck(&["Strike", "Strike"]);
    engine.state.discard_pile = make_deck(&["Strike"]);

    assert!(play_on_enemy(&mut engine, "Perfected Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 66);
}
