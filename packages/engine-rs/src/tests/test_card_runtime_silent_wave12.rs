#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Reflex.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java

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
    assert!(nightmare.effect_data.is_empty());
    assert!(nightmare.complex_hook.is_some());

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

#[test]
#[ignore = "Nightmare still needs a delayed next-turn copy/install primitive; Java /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/NightmareAction.java define the delayed copy path, with /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/NightmarePower.java carrying the next-turn install."]
fn silent_wave12_nightmare_needs_delayed_copy_install_primitive() {}
