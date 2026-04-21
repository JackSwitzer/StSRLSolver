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
    end_turn, enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self,
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
    let mut auto_shields_open = one_enemy_engine(40);
    auto_shields_open.state.hand = make_deck(&["Auto Shields+"]);
    assert!(play_self(&mut auto_shields_open, "Auto Shields+"));
    assert_eq!(auto_shields_open.state.player.block, 15);

    let mut auto_shields_blocked = one_enemy_engine(40);
    auto_shields_blocked.state.player.block = 3;
    auto_shields_blocked.state.hand = make_deck(&["Auto Shields+"]);
    assert!(play_self(&mut auto_shields_blocked, "Auto Shields+"));
    assert_eq!(auto_shields_blocked.state.player.block, 3);

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
    assert_eq!(
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
