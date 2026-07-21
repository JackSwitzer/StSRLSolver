#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Apotheosis.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApotheosisAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java

use crate::cards::global_registry;
use crate::effects::declarative::{BulkAction, Effect as E, CardFilter, Pile as P};
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self,
};

#[test]
fn colorless_wave2_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let apotheosis = registry.get("Apotheosis").expect("Apotheosis should exist");
    assert_eq!(
        apotheosis.effect_data,
        &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::All,
                action: BulkAction::Upgrade,
            },
            E::ForEachInPile {
                pile: P::Draw,
                filter: CardFilter::All,
                action: BulkAction::Upgrade,
            },
            E::ForEachInPile {
                pile: P::Discard,
                filter: CardFilter::All,
                action: BulkAction::Upgrade,
            },
            E::ForEachInPile {
                pile: P::Exhaust,
                filter: CardFilter::All,
                action: BulkAction::Upgrade,
            },
        ]
    );
    assert!(apotheosis.complex_hook.is_none());

    let apotheosis_plus = registry.get("Apotheosis+").expect("Apotheosis+ should exist");
    assert_eq!(apotheosis_plus.effect_data, apotheosis.effect_data);
    assert!(apotheosis_plus.complex_hook.is_none());
}

#[test]
fn apotheosis_upgrades_all_cards_across_all_piles() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        10,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Apotheosis", "Dramatic Entrance"]);
    engine.state.draw_pile = make_deck(&["Good Instincts"]);
    engine.state.discard_pile = make_deck(&["Swift Strike"]);
    engine.state.exhaust_pile = make_deck(&["Magnetism"]);

    assert!(play_self(&mut engine, "Apotheosis"));

    assert!(
        engine.state.hand.iter().any(|card| {
            engine.card_registry.card_name(card.def_id) == "Dramatic Entrance+"
        }),
        "hand card should be upgraded"
    );
    assert!(
        engine.state.draw_pile.iter().all(|card| card.is_upgraded()),
        "draw pile cards should be upgraded"
    );
    assert!(
        engine.state.discard_pile.iter().all(|card| card.is_upgraded()),
        "discard pile cards should be upgraded"
    );
    assert!(
        engine
            .state
            .exhaust_pile
            .iter()
            .any(|card| engine.card_registry.card_name(card.def_id) == "Magnetism+"),
        "exhaust pile cards should be upgraded"
    );
}

#[test]
fn apotheosis_upgrade_changes_only_cost_and_does_not_upgrade_itself_in_limbo() {
    // Apotheosis.java costs 2, exhausts, and its upgrade changes only base
    // cost to 1. ApotheosisAction upgrades hand/draw/discard/exhaust groups;
    // the currently played card is in limbo and therefore cannot upgrade itself.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Apotheosis.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ApotheosisAction.java
    for (card_id, cost) in [("Apotheosis", 2), ("Apotheosis+", 1)] {
        let mut engine = engine_without_start(
            Vec::new(),
            vec![enemy_no_intent("JawWorm", 40, 40)],
            cost,
        );
        force_player_turn(&mut engine);
        engine.state.hand = make_deck(&[card_id, "Dramatic Entrance"]);
        engine.state.draw_pile = make_deck(&["Good Instincts"]);
        engine.state.discard_pile = make_deck(&["Swift Strike"]);
        engine.state.exhaust_pile = make_deck(&["Magnetism"]);
        engine.state.energy = cost;

        assert!(play_self(&mut engine, card_id));
        assert_eq!(engine.state.energy, 0);
        assert!(engine.state.hand.iter().all(|card| card.is_upgraded()));
        assert!(engine.state.draw_pile.iter().all(|card| card.is_upgraded()));
        assert!(engine.state.discard_pile.iter().all(|card| card.is_upgraded()));
        assert!(engine.state.exhaust_pile.iter().any(|card| {
            engine.card_registry.card_name(card.def_id) == "Magnetism+"
        }));
        assert!(engine.state.exhaust_pile.iter().any(|card| {
            engine.card_registry.card_name(card.def_id) == card_id
        }));
        if card_id == "Apotheosis" {
            assert!(engine.state.exhaust_pile.iter().any(|card| {
                engine.card_registry.card_name(card.def_id) == "Apotheosis"
                    && !card.is_upgraded()
            }));
        }
    }
}
