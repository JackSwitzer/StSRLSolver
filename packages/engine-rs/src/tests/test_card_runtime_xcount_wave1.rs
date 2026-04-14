#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ReinforcedBody.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Tempest.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Skewer.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, Pile as P, SimpleEffect as SE};
use crate::orbs::OrbType;
use crate::tests::support::*;

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn xcount_wave1_registry_exports_typed_surface_for_skewer_tempest_plus_and_conjure_blade_plus() {
    let registry = global_registry();

    let skewer = registry.get("Skewer+").expect("Skewer+");
    assert_eq!(skewer.card_type, CardType::Attack);
    assert_eq!(skewer.target, CardTarget::Enemy);
    assert_eq!(skewer.effect_data, &[E::ExtraHits(A::XCost)]);

    let tempest_plus = registry.get("Tempest+").expect("Tempest+");
    assert_eq!(tempest_plus.card_type, CardType::Skill);
    assert_eq!(tempest_plus.target, CardTarget::SelfTarget);
    assert_eq!(
        tempest_plus.effect_data,
        &[
            E::Simple(SE::ChannelOrb(OrbType::Lightning, A::XCost)),
            E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
        ]
    );
    assert!(tempest_plus.complex_hook.is_none());

    let conjure_blade_plus = registry.get("ConjureBlade+").expect("ConjureBlade+");
    assert_eq!(
        conjure_blade_plus.effect_data,
        &[E::Simple(SE::AddCard("Expunger", P::Hand, A::Fixed(1)))]
    );
    assert!(conjure_blade_plus.complex_hook.is_some());
}

#[test]
fn xcount_wave1_skewer_uses_declared_x_hits_and_consumes_all_energy() {
    let mut engine = one_enemy_engine(80, 5);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "Skewer+");

    assert!(play_on_enemy(&mut engine, "Skewer+", 0));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 50);
}

#[test]
fn xcount_wave1_tempest_plus_channels_x_plus_one_lightning_orbs() {
    let mut engine = one_enemy_engine(50, 5);
    engine.init_defect_orbs(4);
    engine.state.energy = 2;
    ensure_in_hand(&mut engine, "Tempest+");

    assert!(play_self(&mut engine, "Tempest+"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.orb_slots.occupied_count(), 3);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    assert_eq!(engine.state.orb_slots.slots[1].orb_type, OrbType::Lightning);
    assert_eq!(engine.state.orb_slots.slots[2].orb_type, OrbType::Lightning);
    assert_eq!(exhaust_prefix_count(&engine, "Tempest"), 1);
}

#[test]
fn xcount_wave1_conjure_blade_plus_sets_expunger_hits_to_x_plus_one() {
    let mut engine = one_enemy_engine(80, 5);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "ConjureBlade+");

    assert!(play_self(&mut engine, "ConjureBlade+"));

    assert_eq!(engine.state.energy, 0);
    let expunger = engine
        .state
        .hand
        .iter()
        .find(|card| engine.card_registry.card_name(card.def_id) == "Expunger")
        .expect("generated Expunger");
    assert_eq!(expunger.misc, 4);
    assert!(hand_count(&engine, "Expunger") >= 1);
}
