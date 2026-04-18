#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::tests::support::{enemy, engine_without_start, ensure_in_hand, force_player_turn, make_deck, play_on_enemy, play_self};

static LESSON_LEARNED_UPGRADE_PILES: [crate::effects::declarative::Pile; 2] = [
    crate::effects::declarative::Pile::Draw,
    crate::effects::declarative::Pile::Discard,
];
static LESSON_LEARNED_KILL_BRANCH: [E; 1] = [E::Simple(SE::UpgradeRandomCardFromPiles(
    &LESSON_LEARNED_UPGRADE_PILES,
))];

fn one_enemy_engine(enemy_hp: i32, enemy_block: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy("JawWorm", enemy_hp, enemy_hp, 1, 0, 1)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.enemies[0].entity.block = enemy_block;
    engine
}

#[test]
fn watcher_wave15_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let wallop = registry.get("Wallop").expect("Wallop should be registered");
    assert_eq!(
        wallop.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
        ]
    );
    assert!(wallop.complex_hook.is_none());

    let wallop_plus = registry.get("Wallop+").expect("Wallop+ should be registered");
    assert_eq!(
        wallop_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
        ]
    );
    assert!(wallop_plus.complex_hook.is_none());

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should be registered");
    assert_eq!(
        lesson_learned.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(crate::effects::declarative::Condition::EnemyKilled, &LESSON_LEARNED_KILL_BRANCH, &[]),
        ]
    );
    assert!(lesson_learned.complex_hook.is_none());

    let lesson_learned_plus = registry
        .get("LessonLearned+")
        .expect("Lesson Learned+ should be registered");
    assert_eq!(
        lesson_learned_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(crate::effects::declarative::Condition::EnemyKilled, &LESSON_LEARNED_KILL_BRANCH, &[]),
        ]
    );
    assert!(lesson_learned_plus.complex_hook.is_none());

    let judgement = registry.get("Judgement").expect("Judgement should be registered");
    assert_eq!(
        judgement.effect_data,
        &[E::Simple(SE::Judgement(A::Magic))]
    );
    assert!(judgement.complex_hook.is_none());

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
fn watcher_wave15_wallop_and_lesson_learned_follow_engine_path() {
    let mut wallop = one_enemy_engine(40, 5);
    ensure_in_hand(&mut wallop, "Wallop");
    assert!(play_on_enemy(&mut wallop, "Wallop", 0));
    assert_eq!(wallop.state.player.block, 4);

    let mut wallop_plus = one_enemy_engine(40, 5);
    ensure_in_hand(&mut wallop_plus, "Wallop+");
    assert!(play_on_enemy(&mut wallop_plus, "Wallop+", 0));
    assert_eq!(wallop_plus.state.player.block, 7);

    let mut lesson_learned = one_enemy_engine(10, 0);
    lesson_learned.state.draw_pile = make_deck(&["Wallop"]);
    ensure_in_hand(&mut lesson_learned, "LessonLearned");
    assert!(play_on_enemy(&mut lesson_learned, "LessonLearned", 0));
    assert!(lesson_learned.state.enemies[0].entity.is_dead());
    assert!(
        lesson_learned
            .state
            .draw_pile
            .iter()
            .any(|card| lesson_learned.card_registry.card_name(card.def_id) == "Wallop+")
    );
    assert!(lesson_learned
        .state
        .exhaust_pile
        .iter()
        .any(|card| lesson_learned.card_registry.card_name(card.def_id) == "LessonLearned"));
}


#[test]
fn watcher_wave15_omniscience_uses_the_typed_draw_pile_free_play_surface() {
    let mut engine = one_enemy_engine(40, 0);
    engine.state.energy = 4;
    ensure_in_hand(&mut engine, "Omniscience");
    engine.state.draw_pile = make_deck(&["Strike", "Defend"]);

    assert!(play_self(&mut engine, "Omniscience"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().expect("choice").reason, ChoiceReason::PlayCardFreeFromDraw);

    engine.execute_action(&crate::actions::Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Strike");
    assert_eq!(engine.state.hand[0].cost, 0);
    assert_eq!(engine.state.draw_pile.len(), 1);
}
