#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/HeavyBlade.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/PerfectedStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Sentinel.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent(enemy_id, hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn ironclad_wave9_registry_moves_low_risk_cards_off_empty_programs() {
    for card_id in [
        "Heavy Blade",
        "Heavy Blade+",
        "Perfected Strike",
        "Perfected Strike+",
    ] {
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
    perfected_strike.state.hand = make_deck(&["Perfected Strike", "Strike"]);
    perfected_strike.state.draw_pile = make_deck(&["Strike", "Strike"]);
    perfected_strike.state.discard_pile = make_deck(&["Strike"]);
    assert!(play_on_enemy(&mut perfected_strike, "Perfected Strike", 0));
    assert_eq!(perfected_strike.state.enemies[0].entity.hp, 64);
}

#[test]
fn perfected_strike_counts_tagged_live_piles_including_self_but_not_exhaust() {
    // PerfectedStrike.countCards scans hand, draw, and discard for STRIKE tags.
    // calculateCardDamage runs while the ordinary played copy is still in hand,
    // and the exhaust pile is never scanned. Upgraded magic is three.
    let mut engine = one_enemy_engine("JawWorm", 100);
    engine.state.hand = make_deck(&["Perfected Strike+", "Pommel Strike", "Defend"]);
    engine.state.draw_pile = make_deck(&["Wild Strike", "Sneaky Strike", "Meteor Strike"]);
    engine.state.discard_pile = make_deck(&["WindmillStrike", "Swift Strike"]);
    engine.state.exhaust_pile = make_deck(&["Strike", "Perfected Strike"]);

    assert!(play_on_enemy(&mut engine, "Perfected Strike+", 0));

    // Seven live-pile Strike tags: the played card, Pommel, Wild, Sneaky,
    // Meteor, Windmill, and Swift. Damage = 6 + 7 * 3 = 27.
    assert_eq!(engine.state.enemies[0].entity.hp, 73);
}

#[test]
fn heavy_blade_multiplies_positive_and_negative_strength_by_three_or_five() {
    // HeavyBlade temporarily multiplies StrengthPower.amount before delegating
    // to AbstractCard damage calculation; the upgrade changes only 3x to 5x.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/HeavyBlade.java
    for (card_id, strength, expected_damage) in [
        ("Heavy Blade", 0, 14),
        ("Heavy Blade", 2, 20),
        ("Heavy Blade", -2, 8),
        ("Heavy Blade+", 2, 24),
        ("Heavy Blade+", -2, 4),
        ("Heavy Blade+", -3, 0),
    ] {
        let mut engine = one_enemy_engine("JawWorm", 100);
        engine.state.hand = make_deck(&[card_id]);
        engine.state.player.set_status(sid::STRENGTH, strength);

        assert!(play_on_enemy(&mut engine, card_id, 0));
        assert_eq!(
            engine.state.enemies[0].entity.hp,
            100 - expected_damage,
            "{card_id} with {strength} Strength"
        );
    }
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
fn ironclad_wave9_sentinel_exhaust_energy_trigger_fires_under_corruption() {
    let mut engine = one_enemy_engine("JawWorm", 60);
    engine.state.energy = 1;
    engine.state.player.set_status(sid::CORRUPTION, 1);
    ensure_in_hand(&mut engine, "Sentinel+");

    assert!(play_self(&mut engine, "Sentinel+"));

    assert_eq!(engine.state.player.block, 8);
    assert_eq!(engine.state.energy, 4);
    assert_eq!(exhaust_prefix_count(&engine, "Sentinel"), 1);
}

#[test]
fn sentinel_exhaust_energy_triggers_without_corruption() {
    // Sentinel.triggerOnExhaust has no Corruption check: Second Wind exhausts
    // both non-Attacks, so the base and upgraded copies grant 2 + 3 energy.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Sentinel.java
    let mut engine = one_enemy_engine("JawWorm", 60);
    engine.state.energy = 1;
    engine.state.hand = make_deck(&["Second Wind+", "Sentinel", "Sentinel+", "Strike"]);

    assert!(play_self(&mut engine, "Second Wind+"));

    assert_eq!(engine.state.energy, 5);
    assert_eq!(exhaust_prefix_count(&engine, "Sentinel"), 2);
    assert_eq!(hand_prefix_count(&engine, "Strike"), 1);
}
