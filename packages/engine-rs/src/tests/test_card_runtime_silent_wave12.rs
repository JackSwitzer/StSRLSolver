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
use crate::tests::support::*;

#[test]
fn silent_wave12_registry_documents_the_remaining_silent_blockers() {
    let registry = global_registry();

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert_eq!(alchemize.effect_data, &[E::Simple(SE::ObtainRandomPotion)]);
    assert!(alchemize.complex_hook.is_none());

    let nightmare = registry.get("Nightmare").expect("Nightmare should exist");
    assert!(nightmare.effect_data.is_empty());
    assert!(nightmare.complex_hook.is_some());

    let reflex = registry.get("Reflex").expect("Reflex should exist");
    assert!(reflex.effect_data.is_empty());
    assert!(reflex.complex_hook.is_none());
    assert!(reflex.effects.contains(&"draw_on_discard"));

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

    let tactician = registry.get("Tactician").expect("Tactician should exist");
    assert!(tactician.effect_data.is_empty());
    assert!(tactician.complex_hook.is_none());
    assert!(tactician.effects.contains(&"energy_on_discard"));
}

#[test]
fn silent_wave12_runtime_backed_residuals_still_follow_their_discard_hooks() {
    let mut reflex_engine = engine_without_start(
        make_deck(&["Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    let reflex = reflex_engine.card_registry.make_card("Reflex+");
    reflex_engine.state.discard_pile.push(reflex);
    reflex_engine.on_card_discarded(reflex);
    assert_eq!(reflex_engine.state.hand.len(), 3);

    let mut tactician_engine = engine_without_start(
        make_deck(&["Strike_G"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    let tactician = tactician_engine.card_registry.make_card("Tactician+");
    tactician_engine.state.discard_pile.push(tactician);
    tactician_engine.on_card_discarded(tactician);
    assert_eq!(tactician_engine.state.energy, 3);
}

#[test]
#[ignore = "Nightmare still needs a delayed next-turn copy/install primitive; Java /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/NightmareAction.java define the delayed copy path, with /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/NightmarePower.java carrying the next-turn install."]
fn silent_wave12_nightmare_needs_delayed_copy_install_primitive() {}
