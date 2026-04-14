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
fn defect_wave16_registry_exports_darkness_variants_on_the_typed_channel_surface() {
    let darkness = global_registry().get("Darkness").expect("Darkness");
    assert_eq!(darkness.effect_data, &[E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1)))]);
    assert!(darkness.complex_hook.is_none());

    let darkness_plus = global_registry().get("Darkness+").expect("Darkness+");
    assert_eq!(
        darkness_plus.effect_data,
        &[
            E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
            E::Simple(SE::TriggerDarkPassive),
        ]
    );
    assert!(darkness_plus.complex_hook.is_none());
}

#[test]
fn defect_wave16_registry_keeps_only_the_remaining_java_blockers_on_partial_typed_surfaces() {
    let consume = global_registry().get("Consume").expect("Consume");
    assert_eq!(
        consume.effect_data,
        &[
            E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
            E::Simple(SE::RemoveOrbSlot),
        ]
    );
    assert!(consume.complex_hook.is_none());

    let ftl = global_registry().get("FTL").expect("FTL");
    assert_eq!(
        ftl.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(ftl.complex_hook.is_some());

    let steam = global_registry().get("Steam").expect("Steam");
    assert_eq!(
        steam.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
        ]
    );
    assert!(steam.complex_hook.is_none());

    let streamline = global_registry().get("Streamline").expect("Streamline");
    assert_eq!(
        streamline.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
        ]
    );
    assert!(streamline.complex_hook.is_none());
}

#[test]
#[ignore = "FTL still needs a cards-played-this-turn gate primitive; Java FTLAction only draws when fewer than three cards were played this turn."]
fn ftl_still_needs_a_cards_played_this_turn_gate() {}
