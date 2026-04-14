#![cfg(test)]

// Java oracle references for the shared post-damage context slice:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Feed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java

use crate::tests::support::*;

fn one_enemy_engine(enemy_hp: i32, enemy_block: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", enemy_hp, enemy_hp.max(1))],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.enemies[0].entity.block = enemy_block;
    engine
}

#[test]
fn typed_primary_attack_context_reaches_feed_reaper_wallop_and_lesson_learned() {
    let mut feed = one_enemy_engine(3, 0);
    feed.state.player.hp = 40;
    feed.state.player.max_hp = 60;
    ensure_in_hand(&mut feed, "Feed");
    assert!(play_on_enemy(&mut feed, "Feed", 0));
    assert_eq!(feed.state.player.max_hp, 63);
    assert_eq!(feed.state.player.hp, 43);

    let mut reaper = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 8, 8),
            enemy_no_intent("Cultist", 7, 7),
        ],
        3,
    );
    force_player_turn(&mut reaper);
    reaper.state.enemies[0].entity.block = 3;
    reaper.state.player.hp = 30;
    reaper.state.player.max_hp = 60;
    ensure_in_hand(&mut reaper, "Reaper");
    assert!(play_on_enemy(&mut reaper, "Reaper", 0));
    assert_eq!(reaper.state.player.hp, 35);

    let mut wallop = one_enemy_engine(20, 3);
    wallop.state.player.block = 0;
    ensure_in_hand(&mut wallop, "Wallop");
    assert!(play_on_enemy(&mut wallop, "Wallop", 0));
    assert_eq!(wallop.state.player.block, 6);

    let mut lesson_learned = one_enemy_engine(3, 0);
    lesson_learned.state.draw_pile = make_deck(&["Wallop"]);
    lesson_learned.state.player.hp = 35;
    lesson_learned.state.player.max_hp = 60;
    ensure_in_hand(&mut lesson_learned, "LessonLearned");
    assert!(play_on_enemy(&mut lesson_learned, "LessonLearned", 0));
    assert!(
        lesson_learned
            .state
            .draw_pile
            .iter()
            .any(|card| card.is_upgraded()),
        "Lesson Learned should upgrade a random eligible card after a kill",
    );
}
