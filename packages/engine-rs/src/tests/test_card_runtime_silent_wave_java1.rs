#![cfg(test)]

// Java oracle:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CalculatedGambleAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/unique/NightmareAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/powers/NightmarePower.java

use crate::actions::Action;
use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, force_player_turn, hand_count, make_deck,
    play_on_enemy, play_self, TEST_SEED,
};

fn engine_for(
    hand: &[&str],
    draw: &[&str],
    enemies: Vec<crate::state::EnemyCombatState>,
    energy: i32,
) -> CombatEngine {
    let mut state = combat_state_with(make_deck(draw), enemies, energy);
    state.hand = make_deck(hand);
    let mut engine = CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

fn hand_names(engine: &CombatEngine) -> Vec<String> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn calculated_gamble_discards_the_remaining_hand_then_draws_the_same_count() {
    let mut engine = engine_for(
        &["Calculated Gamble", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Calculated Gamble"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| matches!(
        name.as_str(),
        "Neutralize" | "Survivor" | "Deflect"
    )));
    assert_eq!(hand_count(&engine, "Calculated Gamble"), 0);
    assert_eq!(hand_count(&engine, "Strike"), 0);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Strike"));
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Defend"));
    assert!(engine
        .state
        .exhaust_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Calculated Gamble"));
}

#[test]
fn calculated_gamble_plus_only_removes_exhaust_and_draws_the_same_count() {
    // CalculatedGamble.upgrade only sets exhaust=false, and use() still constructs
    // CalculatedGambleAction(false). The action's false branch draws exactly `count`.
    // Java: cards/green/CalculatedGamble.java and
    // actions/unique/CalculatedGambleAction.java.
    let mut engine = engine_for(
        &["Calculated Gamble+", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Calculated Gamble+"));

    let names = hand_names(&engine);
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| matches!(
        name.as_str(),
        "Neutralize" | "Survivor" | "Deflect"
    )));
    assert_eq!(hand_count(&engine, "Strike"), 0);
    assert_eq!(hand_count(&engine, "Defend"), 0);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(engine.state.discard_pile.len(), 3);
    assert!(engine
        .state
        .discard_pile
        .iter()
        .any(|card| engine.card_registry.card_name(card.def_id) == "Calculated Gamble+"));
}

#[test]
fn caltrops_stacks_and_retaliates_after_buffer_with_thorns_damage_rules() {
    // Caltrops.java applies ThornsPower for magicNumber 3 (5 upgraded).
    // AbstractPlayer.damage calls BufferPower.onAttackedToChangeDamage before
    // ThornsPower.onAttacked, so Buffer can zero the hit without suppressing
    // retaliation. IntangiblePower caps the queued THORNS DamageAction at one.
    let mut engine = engine_for(
        &["Caltrops", "Caltrops+"],
        &[],
        vec![enemy("JawWorm", 20, 20, 1, 7, 1)],
        2,
    );

    assert!(play_self(&mut engine, "Caltrops"));
    assert!(play_self(&mut engine, "Caltrops+"));
    assert_eq!(engine.state.player.status(sid::THORNS), 8);
    assert_eq!(engine.state.energy, 0);

    engine.state.player.set_status(sid::BUFFER, 1);
    engine.state.enemies[0].entity.set_status(sid::INTANGIBLE, 1);
    let player_hp = engine.state.player.hp;
    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, player_hp);
    assert_eq!(engine.state.player.status(sid::BUFFER), 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 19);
}

#[test]
fn catalyst_applies_only_the_extra_poison_through_apply_power_action() {
    // DoublePoisonAction applies the current Poison amount again; upgraded
    // TriplePoisonAction applies twice the current amount. Both use
    // ApplyPowerAction, whose constructor adds Snecko Skull's +1 and whose
    // update consumes Artifact instead of stacking the Poison debuff.
    let make = |card_id: &str| {
        engine_for(
            &[card_id],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        )
    };

    let mut base = make("Catalyst");
    base.state.enemies[0].entity.set_status(sid::POISON, 5);
    assert!(play_on_enemy(&mut base, "Catalyst", 0));
    assert_eq!(base.state.enemies[0].entity.status(sid::POISON), 10);

    let mut skull = make("Catalyst+");
    skull.state.relics.push("Snake Skull".to_string());
    skull.state.enemies[0].entity.set_status(sid::POISON, 5);
    assert!(play_on_enemy(&mut skull, "Catalyst+", 0));
    assert_eq!(skull.state.enemies[0].entity.status(sid::POISON), 16);

    let mut artifact = make("Catalyst");
    artifact.state.enemies[0].entity.set_status(sid::POISON, 5);
    artifact.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
    assert!(play_on_enemy(&mut artifact, "Catalyst", 0));
    assert_eq!(artifact.state.enemies[0].entity.status(sid::POISON), 5);
    assert_eq!(artifact.state.enemies[0].entity.status(sid::ARTIFACT), 0);
}

#[test]
fn choke_uses_only_preexisting_stacks_then_stacks_the_upgraded_amount() {
    // Choke.use queues its 12-damage action before ApplyPowerAction(ChokePower),
    // while UseCardAction constructs the existing ChokePower LoseHPAction after
    // those card actions. ChokePower is 3 base and Choke.upgrade adds 2 magic.
    // Java: cards/green/Choke.java, powers/ChokePower.java, and
    // actions/utility/UseCardAction.java.
    let mut engine = engine_for(
        &["Choke", "Choke+", "Defend"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        5,
    );
    engine.state.enemies[0].entity.block = 30;

    assert!(play_on_enemy(&mut engine, "Choke", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 40, "Choke must not trigger itself");
    assert_eq!(engine.state.enemies[0].entity.block, 18);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CONSTRICTED), 3);

    assert!(play_on_enemy(&mut engine, "Choke+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
    assert_eq!(engine.state.enemies[0].entity.block, 6);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CONSTRICTED), 8);

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.enemies[0].entity.hp, 29);
    assert_eq!(engine.state.enemies[0].entity.block, 6, "HP_LOSS bypasses Block");
}

#[test]
fn choke_hp_loss_obeys_intangible_buffer_and_expires_at_enemy_turn_start() {
    // AbstractMonster.damage caps HP_LOSS with Intangible before Buffer's
    // onAttackedToChangeDamage, and ChokePower removes itself at start of turn.
    // Java: monsters/AbstractMonster.java, powers/BufferPower.java, and
    // powers/ChokePower.java.
    let mut engine = engine_for(
        &["Choke+", "Defend", "Deflect"],
        &["Strike", "Strike", "Strike", "Strike", "Strike"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        4,
    );

    assert!(play_on_enemy(&mut engine, "Choke+", 0));
    engine.state.enemies[0].entity.set_status(sid::INTANGIBLE, 1);
    engine.state.enemies[0].entity.set_status(sid::BUFFER, 1);
    let hp_after_choke = engine.state.enemies[0].entity.hp;

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_after_choke);
    assert_eq!(engine.state.enemies[0].entity.status(sid::BUFFER), 0);
    assert!(play_self(&mut engine, "Deflect"));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_after_choke - 1);

    end_turn(&mut engine);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CONSTRICTED), 0);
}

#[test]
fn choke_triggers_when_medical_kit_plays_a_status_card() {
    // MedicalKit.onUseCard marks a played Status for exhaust, but it is still a
    // normal UseCardAction and therefore still visits enemy ChokePower.onUseCard.
    // Java: relics/MedicalKit.java and actions/utility/UseCardAction.java.
    let mut engine = engine_for(
        &["Wound"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        0,
    );
    engine.state.relics.push("Medical Kit".to_string());
    engine.state.enemies[0].entity.set_status(sid::CONSTRICTED, 3);

    assert!(play_self(&mut engine, "Wound"));
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
}

#[test]
fn choke_hp_loss_reaches_shifting_on_attacked() {
    // AbstractMonster.damage invokes target powers' onAttacked even for
    // HP_LOSS. ShiftingPower applies an equal temporary Strength loss after a
    // positive hit, represented here by Strength plus its restoration counter.
    // Java: monsters/AbstractMonster.java and powers/ShiftingPower.java.
    let mut engine = engine_for(
        &["Deflect"],
        &[],
        vec![enemy_no_intent("Transient", 40, 40)],
        0,
    );
    engine.state.enemies[0].entity.set_status(sid::CONSTRICTED, 3);
    engine.state.enemies[0].entity.set_status(sid::SHIFTING, 1);
    engine.state.enemies[0].entity.set_status(sid::STRENGTH, 10);

    assert!(play_self(&mut engine, "Deflect"));
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
    assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), 7);
    assert_eq!(engine.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS), 3);
}

#[test]
fn cloak_and_dagger_creates_source_count_shivs_with_master_reality_and_overflow() {
    // CloakAndDagger.use gains 6 Block before MakeTempCardInHandAction creates
    // one Shiv; upgradeMagicNumber(1) creates two. The played card has already
    // left a ten-card hand when the action resolves, leaving one free slot; the
    // second Shiv spills to discard. Master Reality upgrades both copies.
    // Java: cards/green/CloakAndDagger.java and
    // actions/common/MakeTempCardInHandAction.java.
    let mut base = engine_for(
        &["Cloak and Dagger"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    assert!(play_self(&mut base, "Cloak and Dagger"));
    assert_eq!(base.state.player.block, 6);
    assert_eq!(hand_count(&base, "Shiv"), 1);

    let mut plus_hand = vec!["Cloak and Dagger+"];
    plus_hand.extend(std::iter::repeat("Defend").take(9));
    let mut plus = engine_for(
        &plus_hand,
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    plus.state.player.set_status(sid::MASTER_REALITY, 1);

    assert!(play_self(&mut plus, "Cloak and Dagger+"));
    assert_eq!(plus.state.player.block, 6);
    assert_eq!(plus.state.hand.len(), 10);
    assert_eq!(hand_count(&plus, "Shiv+"), 1);
    assert_eq!(
        plus.state
            .discard_pile
            .iter()
            .filter(|card| plus.card_registry.card_name(card.def_id) == "Shiv+")
            .count(),
        1
    );
    assert_eq!(plus.state.energy, 0);
}

#[test]
fn nightmare_opens_a_single_card_choice_but_delayed_next_turn_copies_need_a_runtime_primitive() {
    let mut engine = engine_for(
        &["Nightmare", "Strike", "Defend"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Nightmare"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("nightmare choice");
    assert_eq!(choice.reason, ChoiceReason::DualWield);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 1);
    assert_eq!(choice.options.len(), 2);
}

#[test]
fn nightmare_delayed_copies_should_appear_next_turn_not_immediately() {
    let mut engine = engine_for(
        &["Nightmare", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Nightmare"));
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Strike+"), 0);

    engine.execute_action(&Action::EndTurn);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine.state.hand.len(),
        8,
        "Java Nightmare would add the copies at start of turn before the normal draw"
    );
}
