#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ReinforcedBody.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::tests::support::{enemy_no_intent, engine_without_start, ensure_in_hand, force_player_turn, play_self};

fn one_enemy_engine(energy: i32) -> crate::engine::CombatEngine {
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn xcount_wave2_reinforced_body_moves_to_typed_repeated_block_surface() {
    let reinforced_body = global_registry()
        .get("Reinforced Body+")
        .expect("Reinforced Body+");
    assert_eq!(reinforced_body.card_type, CardType::Skill);
    assert_eq!(reinforced_body.target, CardTarget::SelfTarget);
    assert_eq!(
        reinforced_body.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );
    assert!(reinforced_body.effects.is_empty());
}

#[test]
fn xcount_wave2_reinforced_body_applies_modified_block_per_energy_spent() {
    let mut engine = one_enemy_engine(5);
    engine.state.energy = 3;
    engine.state.player.add_status(crate::status_ids::sid::DEXTERITY, 2);
    ensure_in_hand(&mut engine, "Reinforced Body+");

    assert!(play_self(&mut engine, "Reinforced Body+"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.block, 33);
}

#[test]
fn xcount_wave2_expunger_moves_off_shared_status_bridge() {
    let expunger = global_registry().get("Expunger").expect("Expunger");
    assert_eq!(
        expunger.effect_data,
        &[
            E::ExtraHits(A::CardMisc),
            E::Simple(SE::DealDamage(
                crate::effects::declarative::Target::SelectedEnemy,
                A::Damage,
            )),
        ]
    );
}
