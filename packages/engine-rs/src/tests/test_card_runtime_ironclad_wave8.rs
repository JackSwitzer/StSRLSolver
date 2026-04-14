#![cfg(test)]

// Java oracle references for this wave:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Bludgeon.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/BloodForBlood.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Carnage.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Clash.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Impervious.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/IronWave.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/SearingBlow.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent(enemy_id, hp, hp)], 10);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn ironclad_wave8_registry_exports_use_typed_primary_ops() {
    let registry = global_registry();

    for card_id in [
        "Bludgeon",
        "Bludgeon+",
        "Carnage",
        "Carnage+",
        "Clash",
        "Clash+",
        "Blood for Blood",
        "Blood for Blood+",
        "Searing Blow",
        "Searing Blow+",
    ] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
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

    for card_id in ["Impervious", "Impervious+"] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
        assert_eq!(card.card_type, CardType::Skill);
        assert_eq!(card.target, CardTarget::SelfTarget);
        assert_eq!(card.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
        assert!(
            card.uses_typed_primary_preamble(),
            "{card_id} should use the typed primary preamble"
        );
    }

    for (card_id, block, damage) in [("Iron Wave", 5, 5), ("Iron Wave+", 7, 7)] {
        let card = registry.get(card_id).unwrap_or_else(|| panic!("{card_id} should exist"));
        assert_eq!(card.card_type, CardType::Attack);
        assert_eq!(card.base_block, block);
        assert_eq!(card.base_damage, damage);
        assert_eq!(
            card.effect_data,
            &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ]
        );
        assert!(
            card.uses_typed_primary_preamble(),
            "{card_id} should use the typed primary preamble"
        );
    }
}

#[test]
fn ironclad_wave8_typed_primary_attack_cards_follow_engine_path() {
    let mut engine = one_enemy_engine("JawWorm", 160);

    ensure_in_hand(&mut engine, "Clash");
    assert!(play_on_enemy(&mut engine, "Clash", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 146);

    ensure_in_hand(&mut engine, "Carnage");
    assert!(play_on_enemy(&mut engine, "Carnage", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 126);

    ensure_in_hand(&mut engine, "Blood for Blood+");
    assert!(play_on_enemy(&mut engine, "Blood for Blood+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 104);

    ensure_in_hand(&mut engine, "Bludgeon+");
    assert!(play_on_enemy(&mut engine, "Bludgeon+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 62);
}

#[test]
fn ironclad_wave8_typed_block_and_mixed_cards_follow_engine_path() {
    let mut engine = one_enemy_engine("JawWorm", 80);

    ensure_in_hand(&mut engine, "Impervious");
    assert!(play_self(&mut engine, "Impervious"));
    assert_eq!(engine.state.player.block, 30);
    assert_eq!(exhaust_prefix_count(&engine, "Impervious"), 1);

    ensure_in_hand(&mut engine, "Iron Wave+");
    assert!(play_on_enemy(&mut engine, "Iron Wave+", 0));
    assert_eq!(engine.state.player.block, 37);
    assert_eq!(engine.state.enemies[0].entity.hp, 73);
}

#[test]
fn ironclad_wave8_searing_blow_and_ethereal_rules_still_hold_on_typed_surface() {
    let mut searing = one_enemy_engine("JawWorm", 40);
    ensure_in_hand(&mut searing, "Searing Blow+");
    assert!(play_on_enemy(&mut searing, "Searing Blow+", 0));
    assert_eq!(searing.state.enemies[0].entity.hp, 24);

    let mut carnage = one_enemy_engine("JawWorm", 40);
    ensure_in_hand(&mut carnage, "Carnage+");
    end_turn(&mut carnage);
    assert_eq!(exhaust_prefix_count(&carnage, "Carnage"), 1);
}
