#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Consume.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Darkness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/FTL.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::orbs::OrbType;
use crate::status_ids::sid;

#[test]
fn defect_wave16_registry_exports_darkness_base_on_the_typed_channel_surface() {
    let darkness = global_registry().get("Darkness").expect("Darkness");
    assert_eq!(darkness.effect_data, &[E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1)))]);
    assert!(darkness.complex_hook.is_none());

    let darkness_plus = global_registry().get("Darkness+").expect("Darkness+");
    assert_eq!(darkness_plus.effect_data, &[E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1)))]);
    assert!(darkness_plus.complex_hook.is_some());
}

#[test]
fn defect_wave16_registry_keeps_the_remaining_java_blockers_on_partial_typed_surfaces() {
    let consume = global_registry().get("Consume").expect("Consume");
    assert_eq!(
        consume.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic))]
    );
    assert!(consume.complex_hook.is_some());

    let ftl = global_registry().get("FTL").expect("FTL");
    assert_eq!(
        ftl.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(ftl.complex_hook.is_some());

    let steam = global_registry().get("Steam").expect("Steam");
    assert_eq!(
        steam.effect_data,
        &[E::Simple(SE::GainBlock(A::Block))]
    );
    assert!(steam.complex_hook.is_some());

    let streamline = global_registry().get("Streamline").expect("Streamline");
    assert_eq!(
        streamline.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(streamline.complex_hook.is_some());
}

#[test]
#[ignore = "Consume still needs a typed orb-slot-loss primitive; Java Consume.java and DecreaseMaxOrbAction remove one orb slot after gaining Focus."]
fn consume_still_needs_a_typed_orb_slot_loss_primitive() {}

#[test]
#[ignore = "Darkness+ still needs a typed DarkImpulse primitive; Java Darkness.java queues DarkImpulseAction after the Dark channel."]
fn darkness_plus_still_needs_a_typed_dark_impulse_primitive() {}

#[test]
#[ignore = "FTL still needs a cards-played-this-turn gate primitive; Java FTLAction only draws when fewer than three cards were played this turn."]
fn ftl_still_needs_a_cards_played_this_turn_gate() {}

#[test]
#[ignore = "Steam Barrier still needs a played-instance block decay primitive; Java ModifyBlockAction(this.uuid, -1) only reduces the played copy's current block."]
fn steam_barrier_still_needs_a_played_instance_block_decay_primitive() {}

#[test]
#[ignore = "Streamline still needs a card-instance-scoped cost reduction primitive; Java ReduceCostAction(this.uuid, ...) only lowers the played instance cost."]
fn streamline_still_needs_a_card_instance_cost_reduction_primitive() {}
