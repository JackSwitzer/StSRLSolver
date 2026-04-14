#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/HeavyBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/PerfectedStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Sentinel.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent(enemy_id, hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn ironclad_wave9_registry_moves_low_risk_cards_off_empty_programs() {
    for card_id in ["Heavy Blade", "Heavy Blade+", "Perfected Strike", "Perfected Strike+"] {
        let card = global_registry()
            .get(card_id)
            .unwrap_or_else(|| panic!("{card_id} should exist"));
        assert_eq!(card.card_type, CardType::Attack);
        assert_eq!(card.target, CardTarget::Enemy);
        assert_eq!(
            card.effect_data,
            &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            "{card_id} should declare a typed primary attack",
        );
        assert!(
            card.uses_typed_primary_preamble(),
            "{card_id} should use the typed primary preamble"
        );
    }

    for (card_id, block, energy) in [("Sentinel", 5, 2), ("Sentinel+", 8, 3)] {
        let card = global_registry()
            .get(card_id)
            .unwrap_or_else(|| panic!("{card_id} should exist"));
        assert_eq!(card.card_type, CardType::Skill);
        assert_eq!(card.target, CardTarget::SelfTarget);
        assert_eq!(card.base_block, block);
        assert_eq!(card.base_magic, energy);
        assert_eq!(card.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
        assert!(
            card.uses_typed_primary_preamble(),
            "{card_id} should use the typed primary preamble"
        );
    }
}

#[test]
fn ironclad_wave9_heavy_blade_and_perfected_strike_keep_java_damage_hooks_on_typed_surface() {
    let mut heavy_blade = one_enemy_engine("JawWorm", 80);
    ensure_in_hand(&mut heavy_blade, "Heavy Blade+");
    heavy_blade.state.player.set_status(sid::STRENGTH, 2);
    assert!(play_on_enemy(&mut heavy_blade, "Heavy Blade+", 0));
    assert_eq!(heavy_blade.state.enemies[0].entity.hp, 56);

    let mut perfected_strike = one_enemy_engine("JawWorm", 80);
    perfected_strike.state.hand = make_deck(&["Perfected Strike", "Strike_R"]);
    perfected_strike.state.draw_pile = make_deck(&["Strike_R", "Strike_R"]);
    perfected_strike.state.discard_pile = make_deck(&["Strike_R"]);
    assert!(play_on_enemy(&mut perfected_strike, "Perfected Strike", 0));
    assert_eq!(perfected_strike.state.enemies[0].entity.hp, 66);
}

#[test]
fn ironclad_wave9_sentinel_primary_block_moves_to_typed_surface() {
    let mut engine = one_enemy_engine("JawWorm", 60);
    ensure_in_hand(&mut engine, "Sentinel+");

    assert!(play_self(&mut engine, "Sentinel+"));

    assert_eq!(engine.state.player.block, 8);
    assert_eq!(engine.state.energy, 2);
    assert_eq!(discard_prefix_count(&engine, "Sentinel"), 1);
}

#[test]
#[ignore = "Blocked on Java triggerOnExhaust parity for Sentinel under Corruption; current engine path does not yet fire the energy refund when the skill exhausts via Corruption"]
fn ironclad_wave9_sentinel_exhaust_energy_trigger_stays_queued_until_trigger_on_exhaust_parity_lands() {
    let mut engine = one_enemy_engine("JawWorm", 60);
    engine.state.energy = 1;
    engine.state.player.set_status(sid::CORRUPTION, 1);
    ensure_in_hand(&mut engine, "Sentinel+");

    assert!(play_self(&mut engine, "Sentinel+"));

    assert_eq!(engine.state.player.block, 8);
    assert_eq!(engine.state.energy, 3);
    assert_eq!(exhaust_prefix_count(&engine, "Sentinel"), 1);
}
