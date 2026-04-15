#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Capacitor.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ReinforcedBody.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self};

fn one_enemy_engine(energy: i32) -> crate::engine::CombatEngine {
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave9_capacitor_moves_to_typed_orb_slot_effect() {
    let capacitor = global_registry().get("Capacitor+").expect("Capacitor+");
    assert_eq!(capacitor.card_type, CardType::Power);
    assert_eq!(capacitor.target, CardTarget::SelfTarget);
    assert_eq!(
        capacitor.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::ORB_SLOTS, A::Magic))]
    );
    assert!(capacitor.effects.is_empty());
}

#[test]
fn defect_wave9_capacitor_engine_path_adds_orb_slots_from_typed_effect() {
    let mut engine = one_enemy_engine(3);
    engine.init_defect_orbs(1);
    engine.state.hand = make_deck(&["Capacitor+"]);

    assert_eq!(engine.state.orb_slots.max_slots, 1);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 0);

    assert!(play_self(&mut engine, "Capacitor+"));

    assert_eq!(engine.state.orb_slots.max_slots, 4);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 3);
}

#[test]
fn defect_wave9_reinforced_body_uses_the_typed_xcost_block_surface() {
    let reinforced_body = global_registry()
        .get("Reinforced Body+")
        .expect("Reinforced Body+");
    assert_eq!(reinforced_body.cost, -1);
    assert_eq!(reinforced_body.base_block, 9);
    assert_eq!(reinforced_body.card_type, CardType::Skill);
    assert_eq!(reinforced_body.target, CardTarget::SelfTarget);
    assert_eq!(
        reinforced_body.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );
    assert!(reinforced_body.effects.is_empty());
}
