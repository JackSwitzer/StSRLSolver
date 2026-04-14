#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Chrysalis.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Metamorphosis.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Transmutation.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TransmutationAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ForeignInfluence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java

use crate::cards::{global_registry, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, GeneratedCardPool, GeneratedCostRule};
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, play_self};

#[test]
fn chrysalis_moves_to_typed_generate_to_draw_surface() {
    let registry = global_registry();
    let chrysalis = registry.get("Chrysalis").expect("Chrysalis should exist");
    let chrysalis_plus = registry.get("Chrysalis+").expect("Chrysalis+ should exist");

    assert_eq!(
        chrysalis.effect_data,
        &[E::GenerateRandomCardsToDraw {
            pool: GeneratedCardPool::Skill,
            count: A::Magic,
            cost_rule: GeneratedCostRule::ZeroIfPositiveThisTurn,
        }]
    );
    assert_eq!(chrysalis_plus.effect_data, chrysalis.effect_data);
    assert!(chrysalis.complex_hook.is_none());
    assert!(chrysalis_plus.complex_hook.is_none());
}

#[test]
fn chrysalis_and_metamorphosis_generate_zero_cost_cards_into_draw_pile() {
    let mut engine = engine_with_state(combat_state_with(
        vec![],
        vec![enemy_no_intent("JawWorm", 40, 40)],
        10,
    ));
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    engine.state.hand.clear();
    engine.state.hand.push(engine.card_registry.make_card("Chrysalis"));
    engine.state.hand.push(engine.card_registry.make_card("Metamorphosis+"));

    assert!(play_self(&mut engine, "Chrysalis"));
    assert_eq!(engine.state.draw_pile.len(), 3);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Skill);
        assert!(
            card.cost <= 0,
            "Chrysalis should zero out positive-cost generated skills, got {}",
            card.cost
        );
    }

    engine.state.draw_pile.clear();
    assert!(play_self(&mut engine, "Metamorphosis+"));
    assert_eq!(engine.state.draw_pile.len(), 5);
    for card in &engine.state.draw_pile {
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, CardType::Attack);
        assert!(
            card.cost <= 0,
            "Metamorphosis should zero out positive-cost generated attacks, got {}",
            card.cost
        );
    }
}
