#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{
    AmountSource as A, CardFilter, ChoiceAction, Condition as Cond, Effect as E, Pile as P,
    SimpleEffect as SE, Target as T,
};
use crate::engine::CombatPhase;
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_self,
};

#[test]
fn colorless_wave3_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let forethought_plus = registry
        .get("Forethought+")
        .expect("Forethought+ should exist");
    assert_eq!(
        forethought_plus.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::PutOnBottomFreeIfCostly,
            min_picks: A::Fixed(0),
            max_picks: A::Fixed(99),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(forethought_plus.complex_hook.is_none());

    let ritual_dagger = registry
        .get("RitualDagger")
        .expect("RitualDagger should exist");
    assert_eq!(
        ritual_dagger.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilledNonMinion,
                &[E::Simple(SE::ModifyPlayedCardDamage(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(ritual_dagger.complex_hook.is_none());
}

#[test]
fn forethought_plus_puts_selected_positive_cost_cards_free_on_bottom() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Forethought+", "Strike", "Defend"]);

    assert!(play_self(&mut engine, "Forethought+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    engine.execute_action(&Action::Choose(0));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.draw_pile[0].def_id),
        "Strike"
    );
    assert_eq!(engine.state.draw_pile[0].cost, -1);
    assert!(engine.state.draw_pile[0].is_free());
    assert_eq!(
        engine.card_registry.card_name(engine.state.hand[0].def_id),
        "Defend"
    );
}
