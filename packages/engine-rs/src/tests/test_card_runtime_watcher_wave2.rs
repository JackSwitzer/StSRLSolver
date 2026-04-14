use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::engine::CombatPhase;
use crate::state::Stance;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

fn two_enemy_engine(
    a: (&str, i32, i32),
    b: (&str, i32, i32),
) -> crate::engine::CombatEngine {
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
fn bowling_bash_and_empty_fist_export_declarative_effect_data() {
    let registry = global_registry();

    let bowling_bash = registry.get("BowlingBash").expect("Bowling Bash should be registered");
    assert_eq!(bowling_bash.effect_data, &[E::ExtraHits(A::LivingEnemyCount)]);
    assert!(bowling_bash.complex_hook.is_none());

    let bowling_bash_plus = registry
        .get("BowlingBash+")
        .expect("Bowling Bash+ should be registered");
    assert_eq!(bowling_bash_plus.effect_data, &[E::ExtraHits(A::LivingEnemyCount)]);
    assert!(bowling_bash_plus.complex_hook.is_none());

    let empty_fist = registry.get("EmptyFist").expect("Empty Fist should be registered");
    assert_eq!(empty_fist.enter_stance, None);
    assert_eq!(
        empty_fist.effect_data,
        &[E::Simple(SE::ChangeStance(Stance::Neutral))]
    );

    let empty_fist_plus = registry
        .get("EmptyFist+")
        .expect("Empty Fist+ should be registered");
    assert_eq!(empty_fist_plus.enter_stance, None);
    assert_eq!(
        empty_fist_plus.effect_data,
        &[E::Simple(SE::ChangeStance(Stance::Neutral))]
    );
}

#[test]
fn bowling_bash_hits_once_per_living_enemy_and_empty_fist_exits_stance() {
    let mut bowling_bash = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut bowling_bash, "BowlingBash");
    assert!(play_on_enemy(&mut bowling_bash, "BowlingBash", 0));
    assert_eq!(bowling_bash.state.enemies[0].entity.hp, 36);

    let mut empty_fist = one_enemy_engine("JawWorm", 50, 0);
    set_stance(&mut empty_fist, Stance::Wrath);
    ensure_in_hand(&mut empty_fist, "EmptyFist");
    assert!(play_on_enemy(&mut empty_fist, "EmptyFist", 0));
    assert_eq!(empty_fist.state.enemies[0].entity.hp, 32);
    assert_eq!(empty_fist.state.stance, Stance::Neutral);
}

#[test]
fn fear_no_evil_and_follow_up_cover_stance_sensitive_branching() {
    let mut fear_no_evil = one_enemy_engine("JawWorm", 50, 12);
    set_stance(&mut fear_no_evil, Stance::Wrath);
    ensure_in_hand(&mut fear_no_evil, "FearNoEvil");
    assert!(play_on_enemy(&mut fear_no_evil, "FearNoEvil", 0));
    assert_eq!(fear_no_evil.state.stance, Stance::Calm);

    let mut fear_no_evil_off = one_enemy_engine("JawWorm", 50, 0);
    set_stance(&mut fear_no_evil_off, Stance::Wrath);
    ensure_in_hand(&mut fear_no_evil_off, "FearNoEvil");
    assert!(play_on_enemy(&mut fear_no_evil_off, "FearNoEvil", 0));
    assert_eq!(fear_no_evil_off.state.stance, Stance::Wrath);

    let mut follow_up = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut follow_up, "Strike_P");
    ensure_in_hand(&mut follow_up, "FollowUp");
    assert!(play_on_enemy(&mut follow_up, "Strike_P", 0));
    assert!(play_on_enemy(&mut follow_up, "FollowUp", 0));
    assert_eq!(follow_up.state.energy, 2);
}

#[test]
fn cut_through_fate_third_eye_and_wheel_kick_cover_draw_and_scry_amounts() {
    let mut cut_through_fate = one_enemy_engine("JawWorm", 50, 0);
    cut_through_fate.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship", "Protect"]);
    ensure_in_hand(&mut cut_through_fate, "CutThroughFate");
    let hand_before = cut_through_fate.state.hand.len();
    assert!(play_on_enemy(&mut cut_through_fate, "CutThroughFate", 0));
    assert_eq!(cut_through_fate.phase, CombatPhase::AwaitingChoice);
    assert_eq!(cut_through_fate.choice.as_ref().unwrap().options.len(), 2);
    for i in 0..2 {
        cut_through_fate.execute_action(&Action::Choose(i));
    }
    cut_through_fate.execute_action(&Action::ConfirmSelection);
    assert_eq!(cut_through_fate.state.hand.len(), hand_before + 1);

    let mut third_eye = one_enemy_engine("JawWorm", 50, 0);
    third_eye.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship", "Protect"]);
    ensure_in_hand(&mut third_eye, "ThirdEye");
    assert!(play_self(&mut third_eye, "ThirdEye"));
    assert_eq!(third_eye.phase, CombatPhase::AwaitingChoice);
    assert_eq!(third_eye.choice.as_ref().unwrap().options.len(), 3);
    for i in 0..3 {
        third_eye.execute_action(&Action::Choose(i));
    }
    third_eye.execute_action(&Action::ConfirmSelection);
    assert_eq!(third_eye.state.discard_pile.len(), 3);

    let mut wheel_kick = one_enemy_engine("JawWorm", 50, 0);
    wheel_kick.state.draw_pile = make_deck(&["Strike_P", "Defend_P"]);
    ensure_in_hand(&mut wheel_kick, "WheelKick");
    assert!(play_on_enemy(&mut wheel_kick, "WheelKick", 0));
    assert_eq!(wheel_kick.state.hand.len(), 2);
}

#[test]
fn sash_whip_weakens_after_a_previous_attack() {
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut engine, "Strike_P");
    ensure_in_hand(&mut engine, "SashWhip");
    assert!(play_on_enemy(&mut engine, "Strike_P", 0));
    assert!(play_on_enemy(&mut engine, "SashWhip", 0));
    assert_eq!(engine.state.enemies[0].entity.status(crate::status_ids::sid::WEAKENED), 1);
}
