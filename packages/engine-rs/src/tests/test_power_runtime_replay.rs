use crate::status_ids::sid;
use crate::tests::support::{engine_with, ensure_in_hand, make_deck_n, play_on_enemy, play_self};

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/DoubleTapPower.java
fn double_tap_runtime_replays_next_attack_and_consumes_status() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::DOUBLE_TAP, 1);
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, 38);
    assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 0);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/BurstPower.java
fn burst_runtime_replays_next_skill_and_consumes_status() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::BURST, 1);
    ensure_in_hand(&mut engine, "Backflip");

    let hand_before = engine.state.hand.len();
    assert!(play_self(&mut engine, "Backflip"));

    assert_eq!(engine.state.player.block, 10);
    assert_eq!(engine.state.player.status(sid::BURST), 0);
    assert!(engine.state.hand.len() >= hand_before + 1);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/EchoPower.java
fn echo_form_runtime_replays_only_first_non_power_card() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::ECHO_FORM, 1);
    ensure_in_hand(&mut engine, "Strike");
    ensure_in_hand(&mut engine, "Strike");

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 38);

    assert!(play_on_enemy(&mut engine, "Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 32);
}

#[test]
// Java oracle:
// decompiled/java-src/com/megacrit/cardcrawl/powers/EchoPower.java
fn echo_form_runtime_replays_the_first_power_card_too() {
    let mut engine = engine_with(make_deck_n("Strike", 10), 50, 0);
    engine.state.player.set_status(sid::ECHO_FORM, 1);
    ensure_in_hand(&mut engine, "Inflame");

    assert!(play_self(&mut engine, "Inflame"));

    // EchoPower.onUseCard excludes only purge-on-use copies; it has no card
    // type restriction, so Inflame's 2 Strength is applied twice.
    assert_eq!(engine.state.player.status(sid::STRENGTH), 4);
}
