#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::{enemy, engine_without_start, ensure_in_hand, force_player_turn, make_deck, play_on_enemy};

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
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(wallop.complex_hook.is_some());

    let wallop_plus = registry.get("Wallop+").expect("Wallop+ should be registered");
    assert_eq!(
        wallop_plus.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(wallop_plus.complex_hook.is_some());

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should be registered");
    assert_eq!(
        lesson_learned.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(lesson_learned.complex_hook.is_some());

    let lesson_learned_plus = registry
        .get("LessonLearned+")
        .expect("Lesson Learned+ should be registered");
    assert_eq!(
        lesson_learned_plus.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(lesson_learned_plus.complex_hook.is_some());

    let judgement = registry.get("Judgement").expect("Judgement should be registered");
    assert!(judgement.effect_data.is_empty());
    assert!(judgement.complex_hook.is_some());

    let omniscience = registry
        .get("Omniscience")
        .expect("Omniscience should be registered");
    assert!(omniscience.effect_data.is_empty());
    assert!(omniscience.complex_hook.is_some());
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
#[ignore = "Judgement still needs a typed enemy-HP threshold kill primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java"]
fn watcher_wave15_judgement_stays_queued_until_threshold_kill_primitive_exists() {}

#[test]
#[ignore = "Omniscience still needs a typed draw-pile choice / play-twice primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java"]
fn watcher_wave15_omniscience_stays_queued_until_choice_primitive_exists() {}
