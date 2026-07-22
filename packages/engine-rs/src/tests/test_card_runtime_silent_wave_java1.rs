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
    assert!(names
        .iter()
        .all(|name| matches!(name.as_str(), "Neutralize" | "Survivor" | "Deflect")));
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
    assert!(names
        .iter()
        .all(|name| matches!(name.as_str(), "Neutralize" | "Survivor" | "Deflect")));
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
    engine.state.enemies[0]
        .entity
        .set_status(sid::INTANGIBLE, 1);
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
    let make =
        |card_id: &str| engine_for(&[card_id], &[], vec![enemy_no_intent("JawWorm", 40, 40)], 1);

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
    artifact.state.enemies[0]
        .entity
        .set_status(sid::ARTIFACT, 1);
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
    assert_eq!(
        engine.state.enemies[0].entity.hp, 40,
        "Choke must not trigger itself"
    );
    assert_eq!(engine.state.enemies[0].entity.block, 18);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CONSTRICTED), 3);

    assert!(play_on_enemy(&mut engine, "Choke+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
    assert_eq!(engine.state.enemies[0].entity.block, 6);
    assert_eq!(engine.state.enemies[0].entity.status(sid::CONSTRICTED), 8);

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.enemies[0].entity.hp, 29);
    assert_eq!(
        engine.state.enemies[0].entity.block, 6,
        "HP_LOSS bypasses Block"
    );
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
    engine.state.enemies[0]
        .entity
        .set_status(sid::INTANGIBLE, 1);
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
    let mut engine = engine_for(&["Wound"], &[], vec![enemy_no_intent("JawWorm", 40, 40)], 0);
    engine.state.relics.push("Medical Kit".to_string());
    engine.state.enemies[0]
        .entity
        .set_status(sid::CONSTRICTED, 3);

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
    engine.state.enemies[0]
        .entity
        .set_status(sid::CONSTRICTED, 3);
    engine.state.enemies[0].entity.set_status(sid::SHIFTING, 1);
    engine.state.enemies[0].entity.set_status(sid::STRENGTH, 10);

    assert!(play_self(&mut engine, "Deflect"));
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
    assert_eq!(engine.state.enemies[0].entity.status(sid::STRENGTH), 7);
    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(sid::TEMP_STRENGTH_LOSS),
        3
    );
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
    let mut plus = engine_for(&plus_hand, &[], vec![enemy_no_intent("JawWorm", 40, 40)], 1);
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
fn night_terror_opens_a_single_card_choice_for_a_multi_card_hand() {
    let mut engine = engine_for(
        &["Night Terror", "Strike", "Defend"],
        &[],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Night Terror"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("nightmare choice");
    assert_eq!(choice.reason, ChoiceReason::DualWield);
    assert_eq!(choice.min_picks, 1);
    assert_eq!(choice.max_picks, 1);
    assert_eq!(choice.options.len(), 2);
}

#[test]
fn night_terror_delayed_copies_appear_before_next_turn_draw() {
    let mut engine = engine_for(
        &["Night Terror", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect", "Backflip", "Slice"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );

    assert!(play_self(&mut engine, "Night Terror"));
    engine.execute_action(&Action::Choose(0));
    assert_eq!(engine.state.hand.len(), 2);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Strike+"), 0);

    engine.execute_action(&Action::EndTurn);
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(
        engine.state.hand.len(),
        8,
        "NightmarePower adds three copies before the normal five-card draw"
    );
    assert_eq!(hand_count(&engine, "Strike"), 3);
    let strike_ids: std::collections::HashSet<_> = engine
        .state
        .hand
        .iter()
        .filter(|card| engine.card_registry.card_name(card.def_id) == "Strike")
        .map(|card| card.instance_id)
        .collect();
    assert_eq!(strike_ids.len(), 3, "Nightmare copies need fresh UUIDs");
}

#[test]
fn night_terror_auto_selects_a_single_card_and_keeps_each_power_independent() {
    let mut singleton = engine_for(
        &["Night Terror", "Strike"],
        &["Defend", "Defend", "Defend", "Defend", "Defend"],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    assert!(play_self(&mut singleton, "Night Terror"));
    assert_eq!(singleton.phase, CombatPhase::PlayerTurn);
    singleton.execute_action(&Action::EndTurn);
    assert_eq!(hand_count(&singleton, "Strike"), 3);

    let mut independent = engine_for(
        &["Night Terror", "Night Terror", "Strike", "Defend"],
        &["Neutralize", "Survivor", "Deflect", "Backflip", "Slice"],
        vec![enemy_no_intent("JawWorm", 80, 80)],
        6,
    );
    assert!(play_self(&mut independent, "Night Terror"));
    independent.execute_action(&Action::Choose(1)); // Strike
    assert!(play_self(&mut independent, "Night Terror"));
    independent.execute_action(&Action::Choose(1)); // Defend

    // ApplyPowerAction deliberately excludes ID "Night Terror" from its
    // same-ID stacking loop. The two NightmarePower objects therefore retain
    // distinct cards and remain in stable application order at priority five.
    // Java: actions/common/ApplyPowerAction.java and powers/NightmarePower.java.
    let pending = independent
        .nightmare_pending_copies
        .iter()
        .map(|(card, copies)| {
            (
                independent.card_registry.card_name(card.def_id),
                *copies,
                card.instance_id,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        pending
            .iter()
            .map(|(name, copies, _)| (*name, *copies))
            .collect::<Vec<_>>(),
        [("Strike", 3), ("Defend", 3)]
    );
    assert_ne!(pending[0].2, pending[1].2);
    assert_eq!(
        independent
            .state
            .player
            .power_order
            .iter()
            .filter_map(|entry| match entry {
                crate::state::PowerOrderEntry::NightTerror(instance_id) => Some(*instance_id),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [pending[0].2, pending[1].2]
    );

    // The ordered dynamic identities and their card payloads are causal state,
    // so their association must survive checkpoint serialization exactly.
    independent.validate_checkpoint_boundary().unwrap();
    let encoded = serde_json::to_string(&independent).unwrap();
    let mut restored: CombatEngine = serde_json::from_str(&encoded).unwrap();
    restored.validate_checkpoint_boundary().unwrap();
    assert_eq!(
        restored
            .nightmare_pending_copies
            .iter()
            .map(|(card, copies)| (
                restored.card_registry.card_name(card.def_id),
                *copies,
                card.instance_id,
            ))
            .collect::<Vec<_>>(),
        pending
    );

    restored.execute_action(&Action::EndTurn);

    // Each atStartOfTurn callback independently makes three copies of its own
    // stored card and then removes one same-ID power.
    // Java: powers/NightmarePower.java::atStartOfTurn.
    assert_eq!(hand_count(&restored, "Strike"), 3);
    assert_eq!(hand_count(&restored, "Defend"), 3);
    assert!(restored.nightmare_pending_copies.is_empty());
    assert!(!restored
        .state
        .player
        .power_order
        .iter()
        .any(|entry| matches!(entry, crate::state::PowerOrderEntry::NightTerror(_))));
}

#[test]
fn night_terror_instances_project_and_checkpoint_in_java_power_order() {
    // ApplyPowerAction appends both same-ID powers, and NightmarePower keeps
    // the selected stat-equivalent card as per-instance payload.
    // Java: actions/common/ApplyPowerAction.java:137-164 and
    // powers/NightmarePower.java:23-45.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        let strike = combat.card_registry.make_card("Strike");
        let defend = combat.card_registry.make_card("Defend");
        combat.store_nightmare_copies(strike, 3);
        combat.store_nightmare_copies(defend, 3);
        combat.validate_checkpoint_boundary().unwrap();
    }

    let projected = crate::trace::build_post_state(&run);
    assert_eq!(
        projected
            .player
            .powers
            .iter()
            .filter(|power| power.id == "Night Terror")
            .map(|power| power.amt)
            .collect::<Vec<_>>(),
        [3, 3]
    );

    let mut restored = crate::checkpoint::CoreCheckpoint::capture(&run)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(crate::trace::build_post_state(&restored), projected);
    let combat = restored.debug_combat_engine_mut();
    assert_eq!(
        combat
            .nightmare_pending_copies
            .iter()
            .map(|(card, _)| combat.card_registry.card_name(card.def_id))
            .collect::<Vec<_>>(),
        ["Strike", "Defend"]
    );
}
