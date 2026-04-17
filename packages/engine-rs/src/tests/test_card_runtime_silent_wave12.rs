#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java

use crate::cards::global_registry;
use crate::effects::declarative::{
    AmountSource as A, BulkAction, CardFilter, Effect as E, Pile as P, SimpleEffect as SE,
};
#[test]
fn silent_wave12_registry_documents_the_remaining_silent_blockers() {
    let registry = global_registry();

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert_eq!(alchemize.effect_data, &[E::Simple(SE::ObtainRandomPotion)]);
    assert!(alchemize.complex_hook.is_none());

    let nightmare = registry.get("Nightmare").expect("Nightmare should exist");
    assert_eq!(
        nightmare.effect_data,
        &[crate::effects::declarative::Effect::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::StoreCardForNextTurnCopies,
            min_picks: crate::effects::declarative::AmountSource::Fixed(1),
            max_picks: crate::effects::declarative::AmountSource::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(nightmare.complex_hook.is_none());

    let storm_of_steel = registry
        .get("Storm of Steel")
        .expect("Storm of Steel should exist");
    assert_eq!(
        storm_of_steel.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Discard,
            },
            E::Simple(SE::AddCard("Shiv", P::Hand, A::HandSizeAtPlay)),
        ]
    );
    assert!(storm_of_steel.complex_hook.is_none());

}
