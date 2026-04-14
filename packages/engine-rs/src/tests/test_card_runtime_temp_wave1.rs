#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Safety.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/ThroughViolence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Shiv.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Omega.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

#[test]
fn temp_wave1_registry_exports_typed_surface_for_live_temp_cards() {
    let registry = global_registry();

    let safety = registry.get("Safety").expect("Safety should exist");
    assert_eq!(safety.card_type, CardType::Skill);
    assert_eq!(safety.target, CardTarget::SelfTarget);
    assert!(safety.exhaust);
    assert!(safety.effects.contains(&"retain"));
    assert_eq!(safety.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let through_violence = registry
        .get("ThroughViolence")
        .expect("Through Violence should exist");
    assert_eq!(through_violence.card_type, CardType::Attack);
    assert_eq!(through_violence.target, CardTarget::Enemy);
    assert!(through_violence.exhaust);
    assert!(through_violence.effects.contains(&"retain"));
    assert_eq!(
        through_violence.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let shiv = registry.get("Shiv").expect("Shiv should exist");
    assert_eq!(shiv.card_type, CardType::Attack);
    assert_eq!(shiv.target, CardTarget::Enemy);
    assert!(shiv.exhaust);
    assert_eq!(
        shiv.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let omega = registry.get("Omega").expect("Omega should exist");
    assert_eq!(omega.card_type, CardType::Power);
    assert_eq!(omega.target, CardTarget::SelfTarget);
    assert_eq!(
        omega.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::OMEGA, A::Magic))]
    );
}

#[test]
fn temp_wave1_safety_through_violence_and_shiv_follow_engine_path() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Safety+");
    assert!(play_self(&mut engine, "Safety+"));
    assert_eq!(engine.state.player.block, 16);
    assert_eq!(exhaust_prefix_count(&engine, "Safety"), 1);

    ensure_in_hand(&mut engine, "ThroughViolence");
    assert!(play_on_enemy(&mut engine, "ThroughViolence", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 40);
    assert_eq!(exhaust_prefix_count(&engine, "ThroughViolence"), 1);

    engine.state.player.set_status(sid::ACCURACY, 4);
    ensure_in_hand(&mut engine, "Shiv");
    assert!(play_on_enemy(&mut engine, "Shiv", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 32);
    assert_eq!(exhaust_prefix_count(&engine, "Shiv"), 1);
}

#[test]
fn temp_wave1_omega_installs_runtime_status_and_deals_turn_end_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 90, 90)],
        10,
    );
    force_player_turn(&mut engine);

    ensure_in_hand(&mut engine, "Omega+");
    assert!(play_self(&mut engine, "Omega+"));
    assert_eq!(engine.state.player.status(sid::OMEGA), 60);

    end_turn(&mut engine);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
}

#[test]
#[ignore = "Blocked on typed X-count / repeated-hit temp-card semantics; Java Expunger uses setX(amount) and repeats damage magicNumber times before preserving that state on copies"]
fn temp_wave1_expunger_waits_for_typed_x_count_surface() {
    let registry = global_registry();
    let expunger = registry.get("Expunger").expect("Expunger should exist");
    assert_eq!(expunger.effect_data.len(), 1, "queued once Expunger leaves tag-backed multi_hit");
}
