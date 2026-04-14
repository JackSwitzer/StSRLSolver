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
    for card_id in ["Dual Wield", "Fiend Fire", "Second Wind"] {
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
        }]
    );
    assert!(
        burning_pact.complex_hook.is_some(),
        "Burning Pact still needs the deferred post-choice draw hook"
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
    assert_eq!(true_grit.effect_data, &[crate::effects::declarative::Effect::Simple(
        crate::effects::declarative::SimpleEffect::GainBlock(crate::effects::declarative::AmountSource::Block)
    )]);
    assert!(true_grit.complex_hook.is_some(), "True Grit base still needs the random exhaust primitive");
}

#[test]
#[ignore = "Blocked on Java post-choice sequencing for Burning Pact; the declarative path still needs a typed deferred-draw primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java"]
fn ironclad_wave14_burning_pact_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave14_dual_wield_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave14_fiend_fire_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java non-attack bulk exhaust sequencing for Second Wind; the current runtime still needs a typed exhaust-all-non-attacks + per-card block primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java"]
fn ironclad_wave14_second_wind_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java random-exhaust parity for base True Grit; the card still needs a random exhaust primitive rather than a choice-exhaust surface. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java"]
fn ironclad_wave14_true_grit_base_stays_explicitly_hook_backed() {}
