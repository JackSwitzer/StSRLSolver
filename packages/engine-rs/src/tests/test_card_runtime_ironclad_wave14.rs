#![cfg(test)]

// Java oracle sources for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

#[test]
fn ironclad_wave14_registry_keeps_the_remaining_blockers_explicit() {
    for card_id in ["Dual Wield", "Fiend Fire"] {
        let card = global_registry()
            .get(card_id)
            .unwrap_or_else(|| panic!("{card_id} should exist"));
        assert!(card.effect_data.is_empty(), "{card_id} should stay blocked");
        assert!(card.complex_hook.is_some(), "{card_id} should remain hook-backed");
    }

    let burning_pact = global_registry().get("Burning Pact").expect("Burning Pact");
    assert_eq!(
        burning_pact.effect_data,
        &[E::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::Exhaust,
            min_picks: crate::effects::declarative::AmountSource::Fixed(1),
            max_picks: crate::effects::declarative::AmountSource::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Magic,
        }]
    );
    assert!(
        burning_pact.complex_hook.is_none(),
        "Burning Pact should now be fully typed"
    );

    let headbutt = global_registry().get("Headbutt").expect("Headbutt");
    assert_eq!(
        headbutt.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ChooseCards {
                source: crate::effects::declarative::Pile::Discard,
                filter: crate::effects::declarative::CardFilter::All,
                action: crate::effects::declarative::ChoiceAction::PutOnTopOfDraw,
                min_picks: crate::effects::declarative::AmountSource::Fixed(1),
                max_picks: crate::effects::declarative::AmountSource::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ]
    );
    assert!(headbutt.complex_hook.is_none(), "Headbutt should now be fully typed");

    let havoc = global_registry().get("Havoc").expect("Havoc");
    assert_eq!(
        havoc.effect_data,
        &[E::Simple(SE::PlayTopCardOfDraw)]
    );
    assert!(havoc.complex_hook.is_none(), "Havoc should now be fully typed");

    let true_grit = global_registry().get("True Grit").expect("True Grit");
    assert_eq!(
        true_grit.effect_data,
        &[
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::GainBlock(
                    crate::effects::declarative::AmountSource::Block,
                ),
            ),
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::ExhaustRandomCardFromHand,
            ),
        ]
    );
    assert!(true_grit.complex_hook.is_none(), "True Grit base should now be fully typed");
}

#[test]
fn ironclad_wave14_burning_pact_keeps_choice_body_with_choice_owned_draw_follow_up() {
    let burning_pact = global_registry().get("Burning Pact").expect("Burning Pact");
    assert_eq!(
        burning_pact.effect_data,
        &[E::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::Exhaust,
            min_picks: crate::effects::declarative::AmountSource::Fixed(1),
            max_picks: crate::effects::declarative::AmountSource::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Magic,
        }]
    );
    assert!(burning_pact.complex_hook.is_none());
}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave14_dual_wield_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave14_fiend_fire_stays_explicitly_hook_backed() {}

#[test]
fn ironclad_wave14_second_wind_uses_the_typed_bulk_exhaust_and_count_return_surface() {
    let second_wind = global_registry().get("Second Wind").expect("Second Wind");
    assert_eq!(
        second_wind.effect_data,
        &[
            E::ForEachInPile {
                pile: crate::effects::declarative::Pile::Hand,
                filter: crate::effects::declarative::CardFilter::NonAttacks,
                action: crate::effects::declarative::BulkAction::Exhaust,
            },
            E::Simple(SE::GainBlock(A::LastBulkCountTimesBlock)),
        ]
    );
    assert!(second_wind.complex_hook.is_none());
}

#[test]
fn ironclad_wave14_true_grit_base_uses_the_typed_random_exhaust_surface() {
    let true_grit = global_registry().get("True Grit").expect("True Grit");
    assert_eq!(
        true_grit.effect_data,
        &[
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::GainBlock(
                    crate::effects::declarative::AmountSource::Block,
                ),
            ),
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::ExhaustRandomCardFromHand,
            ),
        ]
    );
    assert!(true_grit.complex_hook.is_none());
}
