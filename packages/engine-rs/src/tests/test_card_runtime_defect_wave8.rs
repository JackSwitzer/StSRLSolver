#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Strike_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Defend_Blue.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Storm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Capacitor.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ForceField.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Rebound.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    end_turn, enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy,
    play_self,
};

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave8_registry_exports_match_typed_runtime_progress() {
    let reg = global_registry();

    let strike_b = reg.get("Strike").expect("Strike");
    assert_eq!(strike_b.card_type, CardType::Attack);
    assert_eq!(strike_b.target, CardTarget::Enemy);
    assert_eq!(
        strike_b.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(strike_b.uses_typed_primary_preamble());

    let defend_b = reg.get("Defend").expect("Defend");
    assert_eq!(defend_b.card_type, CardType::Skill);
    assert_eq!(defend_b.target, CardTarget::SelfTarget);
    assert_eq!(defend_b.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);
    assert!(defend_b.uses_typed_primary_preamble());

    let storm = reg.get("Storm+").expect("Storm+");
    assert_eq!(storm.card_type, CardType::Power);
    assert_eq!(
        storm.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::STORM, A::Magic))]
    );
    assert_eq!(storm.test_markers(), vec!["innate"]);

    let force_field = reg.get("Force Field+").expect("Force Field+");
    assert_eq!(
        force_field.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );
    assert!(force_field.has_test_marker("reduce_cost_per_power"));
    assert!(force_field.uses_typed_primary_preamble());

    let rebound = reg.get("Rebound+").expect("Rebound+");
    assert_eq!(
        rebound.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::AddStatus(T::Player, sid::REBOUND, A::Fixed(1))),
        ]
    );
    assert!(rebound.has_test_marker("next_card_to_top"));
    assert!(rebound.uses_typed_primary_preamble());
}

#[test]
fn defect_wave8_basic_attack_and_block_cards_follow_engine_path() {
    let mut engine = one_enemy_engine(40, 10);
    engine.state.hand = make_deck(&["Strike", "Strike+", "Defend", "Defend+"]);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 34);

    assert!(play_on_enemy(&mut engine, "Strike+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 5);

    assert!(play_self(&mut engine, "Defend+"));
    assert_eq!(engine.state.player.block, 13);
}

#[test]
fn canonical_red_green_and_blue_defends_share_source_block_values() {
    // Defend_Blue.java, Defend_Green.java, and Defend_Red.java each construct
    // a one-cost starter Skill with 5 Block; upgradeBlock(3) makes 8. Their
    // non-debug use methods queue GainBlockAction(this.block).
    let registry = global_registry();
    for (base_id, upgraded_id) in [
        ("Defend_B", "Defend_B+"),
        ("Defend_G", "Defend_G+"),
        ("Defend_R", "Defend_R+"),
    ] {
        let base = registry
            .get(base_id)
            .unwrap_or_else(|| panic!("missing {base_id}"));
        let upgraded = registry
            .get(upgraded_id)
            .unwrap_or_else(|| panic!("missing {upgraded_id}"));
        assert_eq!(base.cost, 1);
        assert_eq!(base.base_block, 5);
        assert_eq!(upgraded.cost, 1);
        assert_eq!(upgraded.base_block, 8);

        let mut engine = one_enemy_engine(40, 2);
        engine.state.hand = make_deck(&[base_id, upgraded_id]);
        assert!(play_self(&mut engine, base_id));
        assert_eq!(engine.state.player.block, 5);
        assert!(play_self(&mut engine, upgraded_id));
        assert_eq!(engine.state.player.block, 13);
    }
}

#[test]
fn defect_wave8_storm_force_field_and_rebound_follow_engine_path() {
    let mut storm = one_enemy_engine(40, 10);
    storm.init_defect_orbs(1);
    storm.state.hand = make_deck(&["Storm", "Defragment"]);
    assert!(play_self(&mut storm, "Storm"));
    assert_eq!(storm.state.player.status(sid::STORM), 1);
    assert!(play_self(&mut storm, "Defragment"));
    assert_eq!(storm.state.player.status(sid::STORM), 1);
    assert_eq!(storm.state.orb_slots.slots[0].orb_type, OrbType::Lightning);

    let mut force_field = one_enemy_engine(60, 6);
    force_field.state.hand = make_deck(&["Heatsinks", "Hello World", "Force Field+"]);
    assert!(play_self(&mut force_field, "Heatsinks"));
    assert!(play_self(&mut force_field, "Hello World"));
    force_field.state.energy = 2;
    assert!(play_self(&mut force_field, "Force Field+"));
    assert_eq!(force_field.state.player.block, 16);
    assert_eq!(force_field.state.energy, 0);

    let mut rebound = one_enemy_engine(40, 3);
    rebound.state.hand = make_deck(&["Rebound"]);
    assert!(play_on_enemy(&mut rebound, "Rebound", 0));
    assert_eq!(rebound.state.enemies[0].entity.hp, 31);
}

#[test]
fn rebound_source_skips_itself_then_puts_the_next_non_power_on_draw_top_without_rng() {
    // Rebound.java deals 9 before applying ReboundPower(1). The new power's
    // justEvoked flag ignores Rebound's own UseCardAction; on the next
    // non-Power, ReboundPower sets UseCardAction.reboundCard and reduces one
    // stack. UseCardAction.moveToDeck(card, false) adds to the top directly.
    // Sources: reference/extracted/methods/card/Rebound.java;
    // decompiled/java-src/com/megacrit/cardcrawl/powers/ReboundPower.java; and
    // decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java.
    let mut engine = one_enemy_engine(40, 3);
    engine.state.hand = make_deck(&["Rebound", "Defend"]);
    engine.state.draw_pile = make_deck(&["Strike"]);
    let card_random_before = engine.card_random_rng.counter;

    assert!(play_on_enemy(&mut engine, "Rebound", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 31);
    assert_eq!(engine.state.player.status(sid::REBOUND), 1);
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.discard_pile[0].def_id),
        "Rebound"
    );

    assert!(play_self(&mut engine, "Defend"));
    assert_eq!(engine.state.player.block, 5);
    assert_eq!(engine.state.player.status(sid::REBOUND), 0);
    assert_eq!(engine.state.draw_pile.len(), 2);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile.last().expect("draw top").def_id),
        "Defend"
    );
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(engine.card_random_rng.counter, card_random_before);
}

#[test]
fn stacked_rebound_rebounds_the_second_copy_but_a_power_only_consumes_the_stack() {
    // ApplyPowerAction stacks a new ReboundPower into the existing instance,
    // whose justEvoked flag is already false. Thus a second Rebound is itself
    // moved to draw top: its new stack raises the amount to two, then the
    // existing power consumes one. ReboundPower excludes Power cards from the
    // destination change but still queues ReducePowerAction for them.
    // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/ReboundPower.java.
    let mut engine = one_enemy_engine(60, 5);
    engine.state.hand = make_deck(&["Rebound", "Rebound+", "Defragment"]);
    let card_random_before = engine.card_random_rng.counter;

    assert!(play_on_enemy(&mut engine, "Rebound", 0));
    assert_eq!(engine.state.player.status(sid::REBOUND), 1);

    assert!(play_on_enemy(&mut engine, "Rebound+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 39);
    assert_eq!(engine.state.player.status(sid::REBOUND), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile.last().expect("draw top").def_id),
        "Rebound+"
    );
    assert_eq!(engine.card_random_rng.counter, card_random_before);

    assert!(play_self(&mut engine, "Defragment"));
    assert_eq!(engine.state.player.focus(), 1);
    assert_eq!(engine.state.player.status(sid::REBOUND), 0);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(engine.state.discard_pile.len(), 1);
}

#[test]
fn rebound_expires_at_turn_end_and_does_not_override_exhaust() {
    // ReboundPower.atEndOfTurn removes all unused stacks. When it does trigger,
    // it only sets reboundCard on UseCardAction; UseCardAction's exhaust branch
    // still wins, so an exhausting Skill consumes Rebound but remains exhausted.
    // Sources: decompiled/java-src/com/megacrit/cardcrawl/powers/ReboundPower.java
    // and decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java.
    let mut exhaust = one_enemy_engine(40, 2);
    exhaust.state.hand = make_deck(&["Rebound", "Miracle"]);
    assert!(play_on_enemy(&mut exhaust, "Rebound", 0));
    assert!(play_self(&mut exhaust, "Miracle"));
    assert_eq!(exhaust.state.player.status(sid::REBOUND), 0);
    assert_eq!(exhaust.state.exhaust_pile.len(), 1);
    assert_eq!(
        exhaust
            .card_registry
            .card_name(exhaust.state.exhaust_pile[0].def_id),
        "Miracle"
    );

    let mut expiry = one_enemy_engine(40, 1);
    expiry.state.hand = make_deck(&["Rebound"]);
    assert!(play_on_enemy(&mut expiry, "Rebound", 0));
    assert_eq!(expiry.state.player.status(sid::REBOUND), 1);
    end_turn(&mut expiry);
    assert_eq!(expiry.state.player.status(sid::REBOUND), 0);
}

#[test]
fn capacitor_adds_two_or_three_slots_and_stops_at_the_java_cap() {
    // Capacitor.java queues IncreaseMaxOrbAction for magicNumber 2 (3 upgraded).
    // That action calls AbstractPlayer.increaseMaxOrbSlots(1, true) repeatedly;
    // increaseMaxOrbSlots refuses each call once maxOrbs reaches ten.
    let mut stacking = one_enemy_engine(40, 2);
    stacking.init_defect_orbs(3);
    stacking.state.hand = make_deck(&["Capacitor", "Capacitor+"]);
    assert!(play_self(&mut stacking, "Capacitor"));
    assert!(play_self(&mut stacking, "Capacitor+"));
    assert_eq!(stacking.state.orb_slots.max_slots, 8);
    assert_eq!(stacking.state.player.status(sid::ORB_SLOTS), 5);

    let mut capped = one_enemy_engine(40, 1);
    capped.init_defect_orbs(9);
    capped.state.hand = make_deck(&["Capacitor+"]);
    assert!(play_self(&mut capped, "Capacitor+"));
    assert_eq!(capped.state.orb_slots.max_slots, 10);
    assert_eq!(capped.state.orb_slots.slots.len(), 10);
    assert_eq!(capped.state.player.status(sid::ORB_SLOTS), 1);
}
