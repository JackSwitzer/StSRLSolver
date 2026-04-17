#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Feed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Condition as Cond, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

static LESSON_LEARNED_UPGRADE_PILES: [P; 2] = [P::Draw, P::Discard];
static LESSON_LEARNED_KILL_BRANCH: [E; 1] = [E::Simple(SE::UpgradeRandomCardFromPiles(
    &LESSON_LEARNED_UPGRADE_PILES,
))];

#[test]
fn test_card_runtime_post_damage_wave1_registry_documents_the_typed_post_damage_surface() {
    let registry = global_registry();

    let wallop = registry.get("Wallop").expect("Wallop should exist");
    assert_eq!(wallop.effect_data.len(), 2);
    assert!(wallop.complex_hook.is_none());

    let wallop_plus = registry.get("Wallop+").expect("Wallop+ should exist");
    assert_eq!(wallop_plus.effect_data.len(), 2);
    assert!(wallop_plus.complex_hook.is_none());

    let feed = registry.get("Feed").expect("Feed should exist");
    assert_eq!(
        feed.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilled,
                &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(feed.complex_hook.is_none());

    let feed_plus = registry.get("Feed+").expect("Feed+ should exist");
    assert_eq!(
        feed_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilled,
                &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(feed_plus.complex_hook.is_none());

    let reaper = registry.get("Reaper").expect("Reaper should exist");
    assert_eq!(
        reaper.effect_data,
        &[
            E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
            E::Simple(SE::HealHp(T::Player, A::TotalUnblockedDamage)),
        ]
    );
    assert!(reaper.complex_hook.is_none());

    let reaper_plus = registry.get("Reaper+").expect("Reaper+ should exist");
    assert_eq!(
        reaper_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
            E::Simple(SE::HealHp(T::Player, A::TotalUnblockedDamage)),
        ]
    );
    assert!(reaper_plus.complex_hook.is_none());

    let lesson_learned = registry
        .get("LessonLearned")
        .expect("Lesson Learned should exist");
    assert_eq!(
        lesson_learned.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilled,
                &LESSON_LEARNED_KILL_BRANCH,
                &[],
            ),
        ]
    );
    assert!(lesson_learned.complex_hook.is_none());
}

#[test]
fn test_card_runtime_post_damage_wave1_wallop_gain_block_uses_unblocked_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Wallop"]);
    engine.state.enemies[0].entity.block = 5;

    assert!(play_on_enemy(&mut engine, "Wallop", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 36);
    assert_eq!(engine.state.player.block, 4);
}

#[test]
fn test_card_runtime_post_damage_wave1_feed_gains_max_hp_only_on_kill() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 10, 10)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Feed"]);
    let max_hp_before = engine.state.player.max_hp;
    let hp_before = engine.state.player.hp;

    assert!(play_on_enemy(&mut engine, "Feed", 0));
    assert!(engine.state.enemies[0].entity.is_dead());
    assert_eq!(engine.state.player.max_hp, max_hp_before + 3);
    assert_eq!(engine.state.player.hp, hp_before + 3);
}

#[test]
fn test_card_runtime_post_damage_wave1_reaper_heals_for_total_unblocked_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 20, 20),
            enemy_no_intent("Cultist", 20, 20),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.player.hp = 30;
    engine.state.player.max_hp = 50;
    engine.state.hand = make_deck(&["Reaper"]);

    assert!(play_on_enemy(&mut engine, "Reaper", 0));
    assert_eq!(engine.state.player.hp, 38);
    assert_eq!(engine.state.enemies[0].entity.hp, 16);
    assert_eq!(engine.state.enemies[1].entity.hp, 16);
}
