#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Feed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

fn one_enemy_engine(enemy_hp: i32, enemy_block: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", enemy_hp, enemy_hp.max(1))],
        energy,
    );
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.state.enemies[0].entity.block = enemy_block;
    engine
}

fn two_enemy_engine(
    enemy0_hp: i32,
    enemy0_block: i32,
    enemy1_hp: i32,
    enemy1_block: i32,
    energy: i32,
) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", enemy0_hp, enemy0_hp.max(1)),
            enemy_no_intent("Cultist", enemy1_hp, enemy1_hp.max(1)),
        ],
        energy,
    );
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.state.enemies[0].entity.block = enemy0_block;
    engine.state.enemies[1].entity.block = enemy1_block;
    engine
}

#[test]
fn ironclad_wave13_registry_exports_promote_feed_and_reaper_to_typed_primary_surface() {
    let feed = global_registry().get("Feed").expect("Feed should exist");
    assert_eq!(feed.card_type, CardType::Attack);
    assert_eq!(feed.target, CardTarget::Enemy);
    assert_eq!(
        feed.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(feed.complex_hook.is_some());

    let reaper = global_registry().get("Reaper").expect("Reaper should exist");
    assert_eq!(reaper.card_type, CardType::Attack);
    assert_eq!(reaper.target, CardTarget::AllEnemy);
    assert_eq!(
        reaper.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );
    assert!(reaper.complex_hook.is_some());
}

#[test]
fn ironclad_wave13_feed_and_reaper_follow_the_typed_primary_surface() {
    let mut feed = one_enemy_engine(3, 0, 3);
    feed.state.player.hp = 40;
    feed.state.player.max_hp = 60;
    ensure_in_hand(&mut feed, "Feed");
    assert!(play_on_enemy(&mut feed, "Feed", 0));
    assert_eq!(feed.state.player.max_hp, 63);
    assert_eq!(feed.state.player.hp, 43);

    let mut reaper = two_enemy_engine(8, 3, 7, 0, 3);
    reaper.state.player.hp = 30;
    reaper.state.player.max_hp = 60;
    ensure_in_hand(&mut reaper, "Reaper");
    assert!(play_on_enemy(&mut reaper, "Reaper", 0));
    assert_eq!(reaper.state.player.hp, 35);
}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave13_dual_wield_stays_explicitly_hook_backed() {
    let dual_wield = global_registry().get("Dual Wield").expect("Dual Wield should exist");
    assert!(dual_wield.effect_data.is_empty());
    assert!(dual_wield.complex_hook.is_some());
}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave13_fiend_fire_stays_explicitly_hook_backed() {
    let fiend_fire = global_registry().get("Fiend Fire").expect("Fiend Fire should exist");
    assert!(fiend_fire.effect_data.is_empty());
    assert!(fiend_fire.complex_hook.is_some());
}

#[test]
#[ignore = "Blocked on Java top-of-draw play sequencing for Havoc; the current runtime still needs a dedicated play-top-card primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java"]
fn ironclad_wave13_havoc_stays_explicitly_hook_backed() {
    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert!(havoc.effect_data.is_empty());
    assert!(havoc.complex_hook.is_some());
}

#[test]
#[ignore = "Blocked on Java non-attack bulk exhaust sequencing for Second Wind; the current runtime still needs a typed exhaust-all-non-attacks + per-card block primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SecondWind.java"]
fn ironclad_wave13_second_wind_stays_explicitly_hook_backed() {
    let second_wind = global_registry().get("Second Wind").expect("Second Wind should exist");
    assert!(second_wind.effect_data.is_empty());
    assert!(second_wind.complex_hook.is_some());
}
