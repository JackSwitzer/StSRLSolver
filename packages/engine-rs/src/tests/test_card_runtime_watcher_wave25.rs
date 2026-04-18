#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::tests::support::{enemy_no_intent, engine_with, engine_without_start, force_player_turn, exhaust_prefix_count, hand_count, make_deck, play_self};

static LESSON_LEARNED_UPGRADE_PILES: [crate::effects::declarative::Pile; 2] = [
    crate::effects::declarative::Pile::Draw,
    crate::effects::declarative::Pile::Discard,
];
static LESSON_LEARNED_KILL_BRANCH: [crate::effects::declarative::Effect; 1] = [
    crate::effects::declarative::Effect::Simple(
        crate::effects::declarative::SimpleEffect::UpgradeRandomCardFromPiles(
            &LESSON_LEARNED_UPGRADE_PILES,
        ),
    ),
];

#[test]
fn watcher_wave25_deus_ex_machina_stays_engine_path_covered_on_draw() {
    let engine = engine_with(crate::tests::support::make_deck(&["DeusExMachina"]), 50, 0);
    assert_eq!(hand_count(&engine, "Miracle"), 2);
    assert_eq!(hand_count(&engine, "DeusExMachina"), 0);
    assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina"), 1);
}

#[test]
fn watcher_wave25_registry_exports_match_current_surface_for_blocked_cards() {
    let registry = global_registry();

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should be registered");
    assert_eq!(
        lesson_learned.effect_data,
        &[
            crate::effects::declarative::Effect::Simple(
                crate::effects::declarative::SimpleEffect::DealDamage(
                    crate::effects::declarative::Target::SelectedEnemy,
                    crate::effects::declarative::AmountSource::Damage,
                ),
            ),
            crate::effects::declarative::Effect::Conditional(
                crate::effects::declarative::Condition::EnemyKilled,
                &LESSON_LEARNED_KILL_BRANCH,
                &[],
            ),
        ]
    );
    assert!(lesson_learned.complex_hook.is_none());

    let omniscience = registry
        .get("Omniscience")
        .expect("Omniscience should be registered");
    assert_eq!(
        omniscience.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::PlayForFree,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(omniscience.complex_hook.is_none());
}

#[test]
fn omniscience_uses_the_typed_draw_pile_free_play_surface() {
    let mut engine = engine_without_start(
        make_deck(&["Omniscience", "Strike", "Defend"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        4,
    );
    force_player_turn(&mut engine);
    engine.state.energy = 4;
    engine.state.hand = make_deck(&["Omniscience"]);
    engine.state.draw_pile = make_deck(&["Strike", "Defend"]);

    assert!(play_self(&mut engine, "Omniscience"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(
        engine.choice.as_ref().expect("Omniscience should open a choice").reason,
        ChoiceReason::PlayCardFreeFromDraw
    );

    engine.execute_action(&crate::actions::Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Strike");
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.draw_pile.len(), 1);
}
