#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustSpecificCardAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/NewQueueCardAction.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn ironclad_wave15_registry_promotes_havoc_and_headbutt_to_the_typed_surface() {
    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.card_type, CardType::Skill);
    assert_eq!(havoc.target, CardTarget::None);
    assert_eq!(
        havoc.effect_data,
        &[E::Simple(SE::PlayTopCardOfDraw)]
    );
    assert!(havoc.complex_hook.is_none(), "Havoc should now be fully typed");

    let headbutt = global_registry().get("Headbutt").expect("Headbutt should exist");
    assert_eq!(headbutt.card_type, CardType::Attack);
    assert_eq!(headbutt.target, CardTarget::Enemy);
    assert_eq!(
        headbutt.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ChooseCards {
                source: P::Discard,
                filter: CardFilter::All,
                action: ChoiceAction::PutOnTopOfDraw,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
            },
        ]
    );
    assert!(headbutt.complex_hook.is_none(), "Headbutt should now be fully typed");
}

#[test]
fn ironclad_wave15_havoc_plays_the_top_card_of_draw_pile_through_the_normal_free_path() {
    let mut engine = engine_for(&["Havoc"], &["Defend_R", "Strike_R"], &[], 3);

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_self(&mut engine, "Havoc"));

    assert_eq!(engine.state.energy, 2);
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 6);
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.draw_pile.last().expect("remaining draw card").def_id),
        "Defend_R"
    );
    assert_eq!(engine.state.discard_pile.len(), 2);
    assert_eq!(
        engine.card_registry.card_name(engine.state.discard_pile.last().expect("top discard").def_id),
        "Havoc"
    );
}

#[test]
fn ironclad_wave15_headbutt_moves_a_discard_card_to_the_top_of_draw() {
    let mut engine = engine_for(&["Headbutt"], &[], &["Defend_R", "Strike_R"], 3);

    let hp_before = engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut engine, "Headbutt", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 9);
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, crate::engine::ChoiceReason::PickFromDiscard);

    engine.execute_action(&Action::Choose(1));

    assert_eq!(engine.phase, crate::engine::CombatPhase::PlayerTurn);
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.discard_pile.last().expect("remaining discard card").def_id),
        "Defend_R"
    );
    assert_eq!(engine.state.draw_pile.len(), 1);
    assert_eq!(
        engine.card_registry.card_name(engine.state.draw_pile.last().expect("top draw card").def_id),
        "Strike_R"
    );
}
