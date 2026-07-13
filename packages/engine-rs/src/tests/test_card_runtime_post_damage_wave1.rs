#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Feed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/HandOfGreed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GreedAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Condition as Cond, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

static LESSON_LEARNED_KILL_BRANCH: [E; 1] =
    [E::Simple(SE::UpgradeRandomMasterDeckCard)];

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
                Cond::EnemyKilledNonMinion,
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
                Cond::EnemyKilledNonMinion,
                &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(feed_plus.complex_hook.is_none());

    let hand_of_greed = registry.get("HandOfGreed").expect("Hand of Greed should exist");
    assert_eq!(
        hand_of_greed.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilledNonMinion,
                &[E::Simple(SE::ModifyGold(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(hand_of_greed.complex_hook.is_none());

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
                Cond::EnemyKilledNonMinion,
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
    // FeedAction excludes half-dead targets and enemies with MinionPower.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FeedAction.java
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

    let mut minion = enemy_no_intent("TorchHead", 10, 10);
    minion.is_minion = true;
    let mut minion_engine = engine_without_start(Vec::new(), vec![minion], 3);
    force_player_turn(&mut minion_engine);
    minion_engine.state.hand = make_deck(&["Feed"]);
    let minion_max_hp = minion_engine.state.player.max_hp;

    assert!(play_on_enemy(&mut minion_engine, "Feed", 0));
    assert!(minion_engine.state.enemies[0].entity.is_dead());
    assert_eq!(minion_engine.state.player.max_hp, minion_max_hp);
}

#[test]
fn hand_of_greed_gains_gold_only_when_its_hit_kills_a_non_minion() {
    // GreedAction damages first, then gains magicNumber gold only if the target
    // is dead, is not half-dead, and does not have MinionPower.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/GreedAction.java
    let mut nonlethal = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 26, 26)],
        3,
    );
    force_player_turn(&mut nonlethal);
    nonlethal.state.hand = make_deck(&["HandOfGreed+"]);
    nonlethal.state.run_gold = 100;

    assert!(play_on_enemy(&mut nonlethal, "HandOfGreed+", 0));
    assert_eq!(nonlethal.state.enemies[0].entity.hp, 1);
    assert_eq!(nonlethal.state.run_gold, 100);
    assert_eq!(nonlethal.state.pending_run_gold, 0);

    let mut lethal = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 25, 25)],
        3,
    );
    force_player_turn(&mut lethal);
    lethal.state.hand = make_deck(&["HandOfGreed+"]);
    lethal.state.run_gold = 100;

    assert!(play_on_enemy(&mut lethal, "HandOfGreed+", 0));
    assert!(lethal.state.enemies[0].entity.is_dead());
    assert_eq!(lethal.state.run_gold, 125);
    assert_eq!(lethal.state.pending_run_gold, 25);

    let mut minion = enemy_no_intent("TorchHead", 20, 20);
    minion.is_minion = true;
    let mut minion_engine = engine_without_start(Vec::new(), vec![minion], 3);
    force_player_turn(&mut minion_engine);
    minion_engine.state.hand = make_deck(&["HandOfGreed"]);
    minion_engine.state.run_gold = 100;

    assert!(play_on_enemy(&mut minion_engine, "HandOfGreed", 0));
    assert!(minion_engine.state.enemies[0].entity.is_dead());
    assert_eq!(minion_engine.state.run_gold, 100);
    assert_eq!(minion_engine.state.pending_run_gold, 0);

    // GreedAction calls AbstractPlayer.gainGold, so the canonical Ectoplasm
    // guard and Bloody Idol callback apply to this reward too.
    let mut ectoplasm = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 20, 20)],
        3,
    );
    force_player_turn(&mut ectoplasm);
    ectoplasm.state.hand = make_deck(&["HandOfGreed"]);
    ectoplasm.state.relics.push("Ectoplasm".to_string());
    ectoplasm.state.run_gold = 100;
    assert!(play_on_enemy(&mut ectoplasm, "HandOfGreed", 0));
    assert_eq!(ectoplasm.state.run_gold, 100);
    assert_eq!(ectoplasm.state.pending_run_gold, 0);

    let mut bloody_idol = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 20, 20)],
        3,
    );
    force_player_turn(&mut bloody_idol);
    bloody_idol.state.hand = make_deck(&["HandOfGreed"]);
    bloody_idol.state.relics.push("Bloody Idol".to_string());
    bloody_idol.state.player.hp = 50;
    bloody_idol.state.run_gold = 100;
    assert!(play_on_enemy(&mut bloody_idol, "HandOfGreed", 0));
    assert_eq!(bloody_idol.state.run_gold, 120);
    assert_eq!(bloody_idol.state.player.hp, 55);
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
