#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Conclude.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Omega.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
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
fn watcher_wave12_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let conclude = registry.get("Conclude").expect("Conclude should be registered");
    assert_eq!(conclude.card_type, CardType::Attack);
    assert_eq!(conclude.target, CardTarget::AllEnemy);
    assert_eq!(conclude.effect_data, &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]);
    assert!(conclude.effects.contains(&"end_turn"));

    let holy_water = registry.get("HolyWater").expect("Holy Water should be registered");
    assert_eq!(holy_water.card_type, CardType::Skill);
    assert_eq!(holy_water.target, CardTarget::SelfTarget);
    assert_eq!(holy_water.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(holy_water.effects.contains(&"retain"));

    let omega = registry.get("Omega").expect("Omega should be registered");
    assert_eq!(omega.card_type, CardType::Power);
    assert_eq!(omega.target, CardTarget::SelfTarget);
    assert_eq!(omega.effect_data, &[E::Simple(SE::AddStatus(T::Player, sid::OMEGA, A::Magic))]);

    let judgement = registry.get("Judgement").expect("Judgement should be registered");
    assert!(judgement.effect_data.is_empty());
    assert!(judgement.complex_hook.is_some());
}

#[test]
fn watcher_wave12_conclude_holy_water_and_omega_follow_engine_path() {
    let mut conclude = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
    ensure_in_hand(&mut conclude, "Conclude");
    let turn_before = conclude.state.turn;
    assert!(play_on_enemy(&mut conclude, "Conclude", 0));
    assert_eq!(conclude.state.turn, turn_before + 1);
    assert_eq!(conclude.state.enemies[0].entity.hp, 38);
    assert_eq!(conclude.state.enemies[1].entity.hp, 38);

    let mut holy_water = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut holy_water, "HolyWater");
    assert!(play_self(&mut holy_water, "HolyWater"));
    assert_eq!(holy_water.state.player.block, 5);
    assert_eq!(exhaust_prefix_count(&holy_water, "HolyWater"), 1);

    let mut omega = one_enemy_engine("JawWorm", 90, 0);
    ensure_in_hand(&mut omega, "Omega+");
    assert!(play_self(&mut omega, "Omega+"));
    assert_eq!(omega.state.player.status(sid::OMEGA), 60);
    end_turn(&mut omega);
    assert_eq!(omega.state.enemies[0].entity.hp, 30);
}
