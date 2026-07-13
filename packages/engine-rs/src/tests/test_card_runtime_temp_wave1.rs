#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Safety.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/ThroughViolence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Shiv.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/AccuracyPower.java
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
    assert!(safety.has_test_marker("retain"));
    assert_eq!(safety.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let through_violence = registry
        .get("ThroughViolence")
        .expect("Through Violence should exist");
    assert_eq!(through_violence.card_type, CardType::Attack);
    assert_eq!(through_violence.target, CardTarget::Enemy);
    assert!(through_violence.exhaust);
    assert!(through_violence.has_test_marker("retain"));
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
fn safety_plus_self_retains_then_gains_modified_block_and_exhausts() {
    // Safety.java is a one-cost, self-retaining, exhausting Skill with 12 Block;
    // upgradeBlock(4) makes Safety+ grant 16 before ordinary card block powers.
    // GainBlockAction receives `this.block`, so two Dexterity raises the live
    // block gain to 18 after the retained card is eventually played.
    // Java: reference/extracted/methods/card/Safety.java
    let registry = global_registry();
    let base = registry.get("Safety").expect("Safety should exist");
    let upgraded = registry.get("Safety+").expect("Safety+ should exist");
    assert_eq!((base.cost, base.base_block), (1, 12));
    assert_eq!((upgraded.cost, upgraded.base_block), (1, 16));

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Safety+"]);
    engine.state.player.set_status(sid::DEXTERITY, 2);

    end_turn(&mut engine);
    assert_eq!(hand_count(&engine, "Safety+"), 1);
    assert_eq!(discard_prefix_count(&engine, "Safety"), 0);

    assert!(play_self(&mut engine, "Safety+"));
    assert_eq!(engine.state.player.block, 18);
    assert_eq!(exhaust_prefix_count(&engine, "Safety"), 1);
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
fn shiv_variants_use_accuracy_damage_for_free_then_exhaust() {
    // Shiv.java constructs a 0-cost 4-damage Attack, exhausts it on use, and
    // upgrades only by 2 damage. Its constructor and AccuracyPower's existing-
    // Shiv refresh make five Accuracy produce 9 and 11 base damage.
    let registry = global_registry();
    let shiv = registry.get("Shiv").expect("Shiv");
    let shiv_plus = registry.get("Shiv+").expect("Shiv+");
    assert_eq!((shiv.cost, shiv.base_damage), (0, 4));
    assert_eq!((shiv_plus.cost, shiv_plus.base_damage), (0, 6));
    assert!(shiv.exhaust && shiv_plus.exhaust);

    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        0,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 0;
    engine.state.hand = make_deck(&["Shiv", "Shiv+"]);
    engine.state.player.set_status(sid::ACCURACY, 5);

    assert!(play_on_enemy(&mut engine, "Shiv", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 41);
    assert!(play_on_enemy(&mut engine, "Shiv+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
    assert_eq!(engine.state.energy, 0);
    assert_eq!(exhaust_prefix_count(&engine, "Shiv"), 2);
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
fn omega_stacks_and_deals_source_less_thorns_damage_to_every_enemy() {
    // Omega.java stacks 50 (60 upgraded). OmegaPower.java uses a pure damage
    // matrix with DamageType.THORNS, so block and Intangible apply while
    // NORMAL-only Slow, Flight, Curl Up, and Malleable do not.
    let mut grounded = enemy_no_intent("JawWorm", 200, 200);
    grounded.entity.block = 10;
    grounded.entity.set_status(sid::SLOW, 5);
    grounded.entity.set_status(sid::FLIGHT, 3);
    grounded.entity.set_status(sid::CURL_UP, 12);
    grounded.entity.set_status(sid::MALLEABLE, 3);
    let mut intangible = enemy_no_intent("Cultist", 200, 200);
    intangible.entity.set_status(sid::INTANGIBLE, 1);

    let mut engine = engine_without_start(Vec::new(), vec![grounded, intangible], 10);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Omega", "Omega+"]);

    assert!(play_self(&mut engine, "Omega"));
    assert!(play_self(&mut engine, "Omega+"));
    assert_eq!(engine.state.player.status(sid::OMEGA), 110);

    end_turn(&mut engine);

    assert_eq!(engine.state.enemies[0].entity.hp, 100);
    assert_eq!(engine.state.enemies[0].entity.block, 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CURL_UP), 12);
    assert_eq!(engine.state.enemies[0].entity.status(sid::MALLEABLE), 3);
    assert_eq!(engine.state.enemies[1].entity.hp, 199);
}

#[test]
fn temp_wave1_expunger_exports_typed_x_count_surface() {
    let registry = global_registry();
    let expunger = registry.get("Expunger").expect("Expunger should exist");
    assert_eq!(expunger.card_type, CardType::Attack);
    assert_eq!(expunger.target, CardTarget::Enemy);
    assert_eq!(
        expunger.effect_data,
        &[
            E::ExtraHits(A::CardMisc),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );

    let expunger_plus = registry.get("Expunger+").expect("Expunger+ should exist");
    assert_eq!(expunger_plus.card_type, CardType::Attack);
    assert_eq!(expunger_plus.target, CardTarget::Enemy);
    assert_eq!(
        expunger_plus.effect_data,
        &[
            E::ExtraHits(A::CardMisc),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
}
