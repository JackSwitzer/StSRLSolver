#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/AutoShields.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/BootSequence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Buffer.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Heatsinks.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/HelloWorld.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Leap.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Loop.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::effects::types::{CardRuntimeTraits, CardBlockHint};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    discard_prefix_count, end_turn, enemy, enemy_no_intent, engine_with, engine_without_start,
    exhaust_prefix_count, force_player_turn, hand_count, make_deck, make_deck_n, play_self,
};

fn one_enemy_engine(hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave7_registry_exports_match_typed_runtime_progress() {
    let reg = global_registry();

    let auto_shields = reg.get("Auto Shields+").expect("Auto Shields+ should exist");
    assert_eq!(auto_shields.card_type, CardType::Skill);
    assert_eq!(auto_shields.target, CardTarget::SelfTarget);
    assert_eq!(auto_shields.base_block, 15);
    assert_eq!(
        auto_shields.effect_data,
        &[E::Conditional(
            crate::effects::declarative::Condition::NoBlock,
            &[E::Simple(SE::GainBlock(A::Block))],
            &[],
        )]
    );
    assert_eq!(
        auto_shields.play_hints().block_hint,
        Some(CardBlockHint::IfNoBlock)
    );

    let boot_sequence = reg.get("BootSequence+").expect("BootSequence+ should exist");
    assert!(boot_sequence.runtime_traits().innate);
    assert!(boot_sequence.exhaust);
    assert_eq!(boot_sequence.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let buffer = reg.get("Buffer+").expect("Buffer+ should exist");
    assert_eq!(
        buffer.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BUFFER, A::Magic))]
    );
    assert_eq!(buffer.runtime_traits(), CardRuntimeTraits::default());

    let heatsinks = reg.get("Heatsinks+").expect("Heatsinks+ should exist");
    assert_eq!(
        heatsinks.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::HEATSINK, A::Magic))]
    );
    assert_eq!(heatsinks.runtime_traits(), CardRuntimeTraits::default());

    let hello_world = reg.get("Hello World+").expect("Hello World+ should exist");
    assert_eq!(
        hello_world.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::HELLO_WORLD, A::Magic))]
    );
    assert!(hello_world.runtime_traits().innate);

    let leap = reg.get("Leap+").expect("Leap+ should exist");
    assert_eq!(leap.base_block, 12);
    assert_eq!(leap.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let loop_card = reg.get("Loop+").expect("Loop+ should exist");
    assert_eq!(
        loop_card.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::LOOP, A::Magic))]
    );
    assert_eq!(loop_card.runtime_traits(), CardRuntimeTraits::default());
}

#[test]
fn defect_wave7_auto_shields_boot_sequence_and_leap_follow_engine_path() {
    // Source: AutoShields.java checks currentBlock == 0 before queuing its
    // GainBlockAction, and upgrades its 11 base block by 4 without changing cost.
    let mut auto_shields_open = one_enemy_engine(40);
    auto_shields_open.state.player.set_status(sid::DEXTERITY, 2);
    auto_shields_open.state.hand = make_deck(&["Auto Shields"]);
    assert!(play_self(&mut auto_shields_open, "Auto Shields"));
    assert_eq!(auto_shields_open.state.player.block, 13);
    assert_eq!(auto_shields_open.state.energy, 2);

    let mut auto_shields_blocked = one_enemy_engine(40);
    auto_shields_blocked.state.player.block = 3;
    auto_shields_blocked.state.hand = make_deck(&["Auto Shields+"]);
    assert!(play_self(&mut auto_shields_blocked, "Auto Shields+"));
    assert_eq!(auto_shields_blocked.state.player.block, 3);
    assert_eq!(auto_shields_blocked.state.energy, 2);

    let mut boot_sequence = one_enemy_engine(40);
    boot_sequence.state.hand = make_deck(&["BootSequence+"]);
    assert!(play_self(&mut boot_sequence, "BootSequence+"));
    assert_eq!(boot_sequence.state.player.block, 13);
    assert_eq!(boot_sequence.state.exhaust_pile.len(), 1);
    assert_eq!(
        boot_sequence.card_registry.card_name(boot_sequence.state.exhaust_pile[0].def_id),
        "BootSequence+"
    );

    let mut leap = one_enemy_engine(40);
    leap.state.hand = make_deck(&["Leap+"]);
    assert!(play_self(&mut leap, "Leap+"));
    assert_eq!(leap.state.player.block, 12);
}

#[test]
fn leap_source_gains_modified_nine_or_twelve_block_for_one_energy() {
    // Leap.java uses this.block in one GainBlockAction. Base Block is 9 and
    // upgradeBlock(3) makes 12; Dexterity and Frail therefore modify each play
    // independently through AbstractCard.applyPowers.
    let mut engine = one_enemy_engine(40);
    engine.state.hand = make_deck(&["Leap", "Leap+"]);
    engine.state.energy = 2;
    engine.state.player.set_status(sid::DEXTERITY, 1);
    engine.state.player.set_status(sid::FRAIL, 1);

    assert!(play_self(&mut engine, "Leap"));
    assert_eq!(engine.state.player.block, 7); // floor((9 + 1) * 0.75)
    assert!(play_self(&mut engine, "Leap+"));
    assert_eq!(engine.state.player.block, 16); // + floor((12 + 1) * 0.75)
    assert_eq!(engine.state.energy, 0);
    assert_eq!(discard_prefix_count(&engine, "Leap"), 2);
}

#[test]
fn boot_sequence_plus_starts_in_hand_blocks_for_free_and_exhausts() {
    // Source: BootSequence.java sets Innate and Exhaust, costs 0, starts at 10
    // block, and upgradeBlock(3) makes the upgraded block 13.
    let mut deck = make_deck_n("Defend", 9);
    deck.push(global_registry().make_card("BootSequence+"));
    let mut engine = engine_with(deck, 40, 0);
    engine.state.player.set_status(sid::DEXTERITY, 2);

    assert_eq!(hand_count(&engine, "BootSequence+"), 1);
    let energy_before = engine.state.energy;
    assert!(play_self(&mut engine, "BootSequence+"));

    assert_eq!(engine.state.player.block, 15);
    assert_eq!(engine.state.energy, energy_before);
    assert_eq!(exhaust_prefix_count(&engine, "BootSequence"), 1);
}

#[test]
fn defect_wave7_buffer_heatsinks_hello_world_and_loop_follow_engine_path() {
    let mut buffer = one_enemy_engine(40);
    buffer.state.hand = make_deck(&["Buffer+"]);
    assert!(play_self(&mut buffer, "Buffer+"));
    assert_eq!(buffer.state.player.status(sid::BUFFER), 2);

    let mut heatsinks = one_enemy_engine(50);
    heatsinks.state.hand = make_deck(&["Heatsinks+", "Buffer"]);
    heatsinks.state.draw_pile = make_deck(&["Strike", "Defend"]);
    assert!(play_self(&mut heatsinks, "Heatsinks+"));
    assert_eq!(heatsinks.state.player.status(sid::HEATSINK), 2);
    assert!(play_self(&mut heatsinks, "Buffer"));
    assert_eq!(heatsinks.state.player.status(sid::BUFFER), 1);
    assert_eq!(heatsinks.state.hand.len(), 2);

    let mut hello_world = one_enemy_engine(40);
    hello_world.state.hand = make_deck(&["Hello World+"]);
    hello_world.state.draw_pile.clear();
    assert!(play_self(&mut hello_world, "Hello World+"));
    assert_eq!(hello_world.state.player.status(sid::HELLO_WORLD), 1);
    end_turn(&mut hello_world);
    assert_eq!(hello_world.state.hand.len(), 1);
    // Java: powers/HelloPower.java draws from commonCardPool, which excludes
    // the BASIC Strike that the old approximation generated.
    assert_ne!(
        hello_world.card_registry.card_name(hello_world.state.hand[0].def_id),
        "Strike"
    );

    let mut loop_card = one_enemy_engine(60);
    loop_card.init_defect_orbs(1);
    loop_card.channel_orb(OrbType::Lightning);
    loop_card.state.hand = make_deck(&["Loop+"]);
    assert!(play_self(&mut loop_card, "Loop+"));
    assert_eq!(loop_card.state.player.status(sid::LOOP), 2);
    end_turn(&mut loop_card);
    assert_eq!(loop_card.state.enemies[0].entity.hp, 54);
}

#[test]
fn heatsinks_triggers_with_preexisting_stacks_before_the_new_power_resolves() {
    // HeatsinkPower.onUseCard queues a draw using its current amount before the
    // played Power's use() action resolves. Therefore Heatsinks does not draw
    // for itself, and a second Heatsinks draws using only the old stack count.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/HeatsinkPower.java
    let mut engine = one_enemy_engine(50);
    engine.state.hand = make_deck(&["Heatsinks", "Heatsinks+"]);
    engine.state.draw_pile = make_deck(&[
        "Strike",
        "Defend",
        "Defend",
        "Defend",
        "Buffer",
    ]);
    engine.state.energy = 5;

    assert!(play_self(&mut engine, "Heatsinks"));
    assert_eq!(engine.state.player.status(sid::HEATSINK), 1);
    assert_eq!(engine.state.draw_pile.len(), 5, "first Heatsinks cannot trigger itself");

    assert!(play_self(&mut engine, "Heatsinks+"));
    assert_eq!(engine.state.player.status(sid::HEATSINK), 3);
    assert_eq!(engine.state.draw_pile.len(), 4, "second Heatsinks draws from the old one stack");
    assert_eq!(hand_count(&engine, "Buffer"), 1);

    assert!(play_self(&mut engine, "Buffer"));
    assert_eq!(engine.state.player.status(sid::BUFFER), 1);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(engine.state.hand.len(), 3, "three Heatsink stacks draw three cards");
}

#[test]
fn buffer_stacks_and_only_consumes_after_block_leaves_positive_damage() {
    // Buffer.java applies magicNumber stacks (1, upgraded to 2). In
    // AbstractPlayer.damage, block is decremented before
    // BufferPower.onAttackedToChangeDamage; Buffer queues one stack reduction
    // only when that per-hit damageAmount is still positive.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Buffer.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BufferPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy("Dummy", 40, 40, 1, 5, 2)],
        4,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Buffer", "Buffer+"]);

    assert!(play_self(&mut engine, "Buffer"));
    assert!(play_self(&mut engine, "Buffer+"));
    assert_eq!(engine.state.player.status(sid::BUFFER), 3);
    assert_eq!(engine.state.energy, 0);

    engine.state.player.block = 5;
    let hp_before = engine.state.player.hp;
    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(engine.state.player.block, 0);
    assert_eq!(engine.state.player.status(sid::BUFFER), 2);
}
