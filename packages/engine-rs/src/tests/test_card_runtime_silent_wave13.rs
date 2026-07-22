#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Expertise.java

use crate::cards::{global_registry, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::tests::support::*;

#[test]
fn silent_wave13_expertise_moves_to_the_declarative_draw_to_n_surface() {
    let registry = global_registry();
    let expertise = registry.get("Expertise").expect("Expertise should exist");
    assert_eq!(
        expertise.effect_data,
        &[E::Simple(SE::DrawToHandSize(A::Magic))]
    );
    assert_eq!(expertise.card_type, CardType::Skill);
    assert!(expertise.complex_hook.is_none());

    let mut engine = engine_without_start(
        make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Expertise"]);
    engine.state.draw_pile =
        make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]);

    assert!(play_self(&mut engine, "Expertise"));
    assert_eq!(engine.state.hand.len(), 6);
    assert_eq!(discard_prefix_count(&engine, "Expertise"), 1);
}

#[test]
fn expertise_draw_is_not_reduced_by_draw_reduction_power() {
    // ExpertiseAction queues an ordinary DrawCardAction for the hand-size
    // shortfall. DrawReductionPower only lowers gameHandSize by one; its
    // remaining duration does not subtract from card-effect draws.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ExpertiseAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawReductionPower.java
    let mut expertise =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 50, 50)], 1);
    force_player_turn(&mut expertise);
    expertise.state.hand = make_deck(&["Expertise", "Defend", "Defend"]);
    expertise.state.draw_pile = make_deck_n("Strike", 10);
    expertise
        .state
        .player
        .set_status(crate::status_ids::sid::DRAW_REDUCTION, 3);

    assert!(play_self(&mut expertise, "Expertise"));
    assert_eq!(expertise.state.hand.len(), 6);
    assert_eq!(expertise.state.draw_pile.len(), 6);

    // The same three-turn power reduces normal turn draw from five to four,
    // not from five to two.
    let mut turn_draw = engine_without_start(
        make_deck_n("Strike", 10),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut turn_draw);
    turn_draw
        .state
        .player
        .set_status(crate::status_ids::sid::DRAW_REDUCTION, 3);
    end_turn(&mut turn_draw);
    assert_eq!(turn_draw.state.hand.len(), 4);
}
