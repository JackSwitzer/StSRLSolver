#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Adrenaline.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Blur.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Footwork.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/PiercingWail.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Prepared.java
use crate::actions::Action;
use crate::cards::global_registry;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn silent_wave7_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let adrenaline = registry.get("Adrenaline").expect("Adrenaline should be registered");
    assert_eq!(
        adrenaline.effect_data,
        &[
            E::Simple(SE::GainEnergy(A::Magic)),
            E::Simple(SE::DrawCards(A::Fixed(2))),
        ]
    );
    assert_eq!(adrenaline.base_magic, 1);

    let adrenaline_plus = registry.get("Adrenaline+").expect("Adrenaline+ should be registered");
    assert_eq!(adrenaline_plus.base_magic, 2);

    let blur = registry.get("Blur").expect("Blur should be registered");
    assert_eq!(
        blur.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BLUR, A::Fixed(1)))]
    );

    let footwork = registry.get("Footwork").expect("Footwork should be registered");
    assert_eq!(
        footwork.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic))]
    );

    let prepared = registry.get("Prepared").expect("Prepared should be registered");
    assert_eq!(prepared.effect_data[0], E::Simple(SE::DrawCards(A::Magic)));

}

#[test]
fn silent_wave7_adrenaline_blur_and_footwork_run_on_engine_path() {
    let mut adrenaline = one_enemy_engine("JawWorm", 50, 0);
    adrenaline.state.draw_pile = make_deck(&["Strike", "Defend", "Neutralize"]);
    let draw_before = adrenaline.state.draw_pile.len();
    ensure_in_hand(&mut adrenaline, "Adrenaline");
    assert!(play_self(&mut adrenaline, "Adrenaline"));
    assert_eq!(adrenaline.state.energy, 4);
    assert_eq!(adrenaline.state.draw_pile.len(), draw_before - 2);
    assert!(exhaust_prefix_count(&adrenaline, "Adrenaline") >= 1);

    let mut adrenaline_plus = one_enemy_engine("JawWorm", 50, 0);
    adrenaline_plus.state.draw_pile = make_deck(&["Strike", "Defend", "Neutralize"]);
    ensure_in_hand(&mut adrenaline_plus, "Adrenaline+");
    assert!(play_self(&mut adrenaline_plus, "Adrenaline+"));
    assert_eq!(adrenaline_plus.state.energy, 5);

    let mut blur = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut blur, "Blur");
    assert!(play_self(&mut blur, "Blur"));
    assert_eq!(blur.state.player.block, 5);
    end_turn(&mut blur);
    assert_eq!(blur.state.player.block, 5);
    assert_eq!(blur.state.player.status(sid::BLUR), 0);

    let mut footwork = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut footwork, "Footwork+");
    ensure_in_hand(&mut footwork, "Defend");
    assert!(play_self(&mut footwork, "Footwork+"));
    assert_eq!(footwork.state.player.status(sid::DEXTERITY), 3);
    assert!(play_self(&mut footwork, "Defend"));
    assert_eq!(footwork.state.player.block, 8);
}

#[test]
fn footwork_variants_stack_two_and_three_permanent_dexterity() {
    // Footwork.java applies DexterityPower(this.magicNumber), initialized to
    // two and upgraded by one. Playing both variants therefore leaves five
    // Dexterity, which raises Defend's five Block to ten.
    // Java: reference/extracted/methods/card/Footwork.java
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.hand = make_deck(&["Footwork", "Footwork+", "Defend"]);

    assert!(play_self(&mut engine, "Footwork"));
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 2);
    assert!(play_self(&mut engine, "Footwork+"));
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 5);
    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 10);
}

#[test]
fn piercing_wail_reduces_all_unprotected_attacks_then_restores_strength() {
    // PiercingWail.java applies StrengthPower(-8) to every monster, but its
    // second loop skips GainStrengthPower when Artifact was present. The
    // unprotected monster therefore attacks at reduced Strength before
    // GainStrengthPower.java restores it at the end of the monster turn.
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy("JawWorm", 50, 50, 1, 10, 1),
            enemy("Cultist", 50, 50, 1, 10, 1),
        ],
        1,
    );
    force_player_turn(&mut engine);
    for enemy in &mut engine.state.enemies {
        enemy.entity.set_status(sid::STRENGTH, 4);
    }
    engine.state.enemies[1].entity.set_status(sid::ARTIFACT, 1);
    engine.state.hand = make_deck(&["Piercing Wail+"]);

    assert!(play_self(&mut engine, "Piercing Wail+"));
    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), -4);
    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(sid::TEMP_STRENGTH_LOSS),
        8
    );
    assert_eq!(engine.state.enemies[1].entity.status(sid::ARTIFACT), 0);
    assert_eq!(engine.state.enemies[1].entity.status(sid::STRENGTH), 4);
    assert_eq!(
        engine.state.enemies[1]
            .entity
            .status(sid::TEMP_STRENGTH_LOSS),
        0
    );
    assert_eq!(exhaust_prefix_count(&engine, "Piercing Wail"), 1);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 60,
        "unprotected enemy deals 10-4 while Artifact enemy deals 10+4");
    assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), 4);
    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(sid::TEMP_STRENGTH_LOSS),
        0
    );
}

#[test]
fn poisoned_stab_deals_damage_before_applying_source_poison() {
    // PoisonedStab.java queues 6 Damage then 3 Poison; upgradeDamage(2) and
    // upgradeMagicNumber(1) make those 8 and 4. ApplyPowerAction means Artifact
    // blocks only Poison, and DamageAction cancels the queued power on lethal.
    let mut stacked = one_enemy_engine("JawWorm", 40, 0);
    stacked.state.hand = make_deck(&["Poisoned Stab", "Poisoned Stab+"]);
    assert!(play_on_enemy(&mut stacked, "Poisoned Stab", 0));
    assert!(play_on_enemy(&mut stacked, "Poisoned Stab+", 0));
    assert_eq!(stacked.state.enemies[0].entity.hp, 26);
    assert_eq!(stacked.state.enemies[0].entity.status(sid::POISON), 7);
    assert_eq!(stacked.state.energy, 1);

    let mut artifact = one_enemy_engine("JawWorm", 20, 0);
    artifact.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
    artifact.state.hand = make_deck(&["Poisoned Stab+"]);
    assert!(play_on_enemy(&mut artifact, "Poisoned Stab+", 0));
    assert_eq!(artifact.state.enemies[0].entity.hp, 12);
    assert_eq!(artifact.state.enemies[0].entity.status(sid::ARTIFACT), 0);
    assert_eq!(artifact.state.enemies[0].entity.status(sid::POISON), 0);

    let mut lethal = one_enemy_engine("JawWorm", 6, 0);
    lethal.state.hand = make_deck(&["Poisoned Stab"]);
    assert!(play_on_enemy(&mut lethal, "Poisoned Stab", 0));
    assert_eq!(lethal.state.enemies[0].entity.hp, 0);
    assert_eq!(lethal.state.enemies[0].entity.status(sid::POISON), 0,
        "lethal DamageAction clears the queued ApplyPowerAction");
}

#[test]
fn blur_does_not_decrement_during_vaults_skipped_enemy_round() {
    // Sources: Blur.java installs one BlurPower; GameActionManager.java skips
    // monsters.applyEndOfTurnPowers() under Vault but still checks Blur before
    // start-of-turn block loss.
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.energy = 10;
    engine.state.hand = make_deck(&["Blur", "Vault"]);

    assert!(play_self(&mut engine, "Blur"));
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.player.status(sid::BLUR), 1);
    assert!(play_self(&mut engine, "Vault"));

    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.player.status(sid::BLUR), 1);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.player.status(sid::BLUR), 0);

    end_turn(&mut engine);
    assert_eq!(engine.state.player.block, 0);
}

#[test]
fn silent_wave7_prepared_uses_draw_then_discard_choice_on_engine_path() {
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.draw_pile = make_deck(&["Strike", "Defend", "Neutralize"]);
    engine.state.hand = make_deck(&["Prepared+", "Survivor"]);

    assert!(play_self(&mut engine, "Prepared+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Prepared+ discard choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.min_picks, 2);
    assert_eq!(choice.max_picks, 2);

    let mut discard_hand_indices = Vec::new();
    for option in &choice.options {
        let ChoiceOption::HandCard(hand_idx) = option else {
            continue;
        };
        discard_hand_indices.push(*hand_idx);
    }

    assert!(discard_hand_indices.len() >= 2);
    engine.execute_action(&Action::Choose(discard_hand_indices[0]));
    engine.execute_action(&Action::Choose(discard_hand_indices[1]));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 2);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.discard_pile.len(), 3);
}
