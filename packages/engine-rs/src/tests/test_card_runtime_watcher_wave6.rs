#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Consecrate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CrushJoints.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlyingSleeves.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vigilance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WindmillStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Nirvana.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LikeWater.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::actions::Action;
use crate::engine::CombatPhase;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

fn two_enemy_engine(a: (&str, i32, i32), b: (&str, i32, i32)) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy(a.0, a.1, a.1, 1, a.2, 1),
            enemy(b.0, b.1, b.1, 1, b.2, 1),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave6_registry_exports_match_declared_runtime_surface() {
    let registry = global_registry();

    let flying_sleeves = registry.get("FlyingSleeves").expect("FlyingSleeves should be registered");
    assert!(flying_sleeves.has_test_marker("retain"));
    assert_eq!(flying_sleeves.declared_extra_hits(), Some(A::Magic));

    let vigilance = registry.get("Vigilance").expect("Vigilance should be registered");
    assert_eq!(vigilance.enter_stance, Some("Calm"));
    assert_eq!(vigilance.declared_stance_change(), Some(Stance::Calm));

    let nirvana = registry.get("Nirvana").expect("Nirvana should be registered");
    assert_eq!(
        nirvana.effect_data,
        &[E::Simple(SE::AddStatus(crate::effects::declarative::Target::Player, sid::NIRVANA, A::Magic))]
    );

    let like_water = registry.get("LikeWater").expect("LikeWater should be registered");
    assert_eq!(
        like_water.effect_data,
        &[E::Simple(SE::AddStatus(crate::effects::declarative::Target::Player, sid::LIKE_WATER, A::Magic))]
    );
}

#[test]
fn watcher_wave6_bril_consecrate_and_crush_joints_follow_java_behavior() {
    let mut brilliance = one_enemy_engine("JawWorm", 80, 0);
    brilliance.state.mantra_gained = 6;
    ensure_in_hand(&mut brilliance, "Brilliance");
    assert!(play_on_enemy(&mut brilliance, "Brilliance", 0));
    assert_eq!(brilliance.state.enemies[0].entity.hp, 62);

    let mut consecrate = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut consecrate, "Consecrate+");
    assert!(play_on_enemy(&mut consecrate, "Consecrate+", 0));
    assert_eq!(consecrate.state.enemies[0].entity.hp, 42);
    assert_eq!(consecrate.state.enemies[1].entity.hp, 42);

    let mut crush_joints = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut crush_joints, "Defend_P");
    ensure_in_hand(&mut crush_joints, "CrushJoints+");
    assert!(play_self(&mut crush_joints, "Defend_P"));
    assert!(play_on_enemy(&mut crush_joints, "CrushJoints+", 0));
    assert_eq!(crush_joints.state.enemies[0].entity.hp, 40);
    assert_eq!(crush_joints.state.enemies[0].entity.status(sid::VULNERABLE), 2);
}

#[test]
fn watcher_wave6_flying_sleeves_retains_and_windmill_strike_scales_on_retain() {
    let mut flying_sleeves = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut flying_sleeves, "FlyingSleeves");
    end_turn(&mut flying_sleeves);
    assert_eq!(hand_count(&flying_sleeves, "FlyingSleeves"), 1);

    let mut attack = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut attack, "FlyingSleeves+");
    assert!(play_on_enemy(&mut attack, "FlyingSleeves+", 0));
    assert_eq!(attack.state.enemies[0].entity.hp, 38);

    let mut windmill = one_enemy_engine("JawWorm", 80, 0);
    ensure_in_hand(&mut windmill, "WindmillStrike");
    end_turn(&mut windmill);
    assert_eq!(windmill.state.player.status(sid::WINDMILL_STRIKE_BONUS), 4);
    assert!(play_on_enemy(&mut windmill, "WindmillStrike", 0));
    assert_eq!(windmill.state.enemies[0].entity.hp, 69);
}

#[test]
fn watcher_wave6_vigilance_nirvana_and_like_water_run_on_engine_path() {
    let mut vigilance = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut vigilance, "Vigilance+");
    assert!(play_self(&mut vigilance, "Vigilance+"));
    assert_eq!(vigilance.state.player.block, 12);
    assert_eq!(vigilance.state.stance, Stance::Calm);

    let mut nirvana = one_enemy_engine("JawWorm", 60, 0);
    nirvana.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
    ensure_in_hand(&mut nirvana, "Nirvana+");
    ensure_in_hand(&mut nirvana, "ThirdEye");
    assert!(play_self(&mut nirvana, "Nirvana+"));
    assert_eq!(nirvana.state.player.status(sid::NIRVANA), 4);
    assert!(play_self(&mut nirvana, "ThirdEye"));
    assert_eq!(nirvana.phase, CombatPhase::AwaitingChoice);
    let num_options = nirvana.choice.as_ref().expect("scry choice").options.len();
    for i in 0..num_options {
        nirvana.execute_action(&Action::Choose(i));
    }
    nirvana.execute_action(&Action::ConfirmSelection);
    assert_eq!(nirvana.state.player.block, 11);

    let mut like_water = one_enemy_engine("JawWorm", 60, 12);
    ensure_in_hand(&mut like_water, "LikeWater");
    ensure_in_hand(&mut like_water, "Vigilance");
    assert!(play_self(&mut like_water, "LikeWater"));
    assert_eq!(like_water.state.player.status(sid::LIKE_WATER), 5);
    assert!(play_self(&mut like_water, "Vigilance"));
    end_turn(&mut like_water);
    assert_eq!(like_water.state.player.hp, 80);
}
