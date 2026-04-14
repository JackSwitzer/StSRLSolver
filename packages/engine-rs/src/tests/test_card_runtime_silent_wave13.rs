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
        make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Expertise"]);
    engine.state.draw_pile = make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]);

    assert!(play_self(&mut engine, "Expertise"));
    assert_eq!(engine.state.hand.len(), 6);
    assert_eq!(discard_prefix_count(&engine, "Expertise"), 1);
}
