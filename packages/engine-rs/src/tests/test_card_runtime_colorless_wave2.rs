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
#[ignore = "Impatience still needs a no-attacks-in-hand primitive; Java checks the current hand contents before drawing."]
fn impatience_still_needs_no_attacks_in_hand_primitive() {}

#[test]
#[ignore = "Mind Blast still needs a draw-pile-size attack scaling primitive on the typed primary attack path; Java resolves damage from the current draw pile size."]
fn mind_blast_still_needs_draw_pile_size_attack_scaling() {}

#[test]
#[ignore = "Ritual Dagger still needs kill-context and card-owned misc scaling propagation; Java updates the played copy after a kill and carries the dagger's misc state forward."]
fn ritual_dagger_still_needs_kill_context_and_misc_scaling() {}
