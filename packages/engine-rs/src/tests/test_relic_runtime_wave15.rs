#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/PureWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/HolyWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/NinjaScroll.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MarkOfPain.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/relics/MutagenicStrength.java

use crate::effects::runtime::GameEvent;
use crate::effects::trigger::Trigger;
use crate::status_ids::sid;
use crate::tests::support::{
    discard_prefix_count, draw_prefix_count, enemy_no_intent, engine_without_start, hand_count,
    make_deck_n,
};

fn fire_combat_start(engine: &mut crate::engine::CombatEngine) {
    engine.rebuild_effect_runtime();
    engine.emit_event(GameEvent::empty(Trigger::CombatStart));
}

#[test]
fn relic_wave15_pure_water_adds_one_miracle_at_combat_start() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("PureWater".to_string());

    fire_combat_start(&mut engine);

    assert_eq!(hand_count(&engine, "Miracle"), 1);
    assert_eq!(discard_prefix_count(&engine, "Miracle"), 0);
}

// Source-derived (verify relic/PureWater):
// PureWater.java atBattleStartPreDraw() -> MakeTempCardInHandAction(new Miracle(), 1, false).
// The hook is Pre-Draw: exactly one un-upgraded Miracle sits in hand alongside the
// full 5-card opening draw (it neither consumes a draw slot nor lands in draw/discard).
#[test]
fn relic_wave15_pure_water_miracle_is_added_before_the_opening_draw() {
    let mut engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics.push("PureWater".to_string());

    engine.start_combat();

    assert_eq!(hand_count(&engine, "Miracle"), 1);
    assert_eq!(hand_count(&engine, "Strike"), 5);
    assert_eq!(engine.state.hand.len(), 6);
    assert_eq!(draw_prefix_count(&engine, "Miracle"), 0);
    assert_eq!(discard_prefix_count(&engine, "Miracle"), 0);
    // MakeTempCardInHandAction receives `new Miracle()` (no upgrade call): un-upgraded.
    assert_eq!(hand_count(&engine, "Miracle+"), 0);
}

// Source-derived (verify relic/HolyWater):
// HolyWater.java queues exactly three un-upgraded Miracle instances, while
// MakeTempCardInHandAction spills only the cards beyond the ten-card hand cap.
#[test]
fn relic_wave15_holy_water_adds_three_miracles_and_spills_only_overflow() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("HolyWater".to_string());
    engine.state.hand = make_deck_n("Strike", 9);

    fire_combat_start(&mut engine);

    assert_eq!(hand_count(&engine, "Miracle"), 1);
    assert_eq!(discard_prefix_count(&engine, "Miracle"), 2);
    assert_eq!(hand_count(&engine, "Miracle+"), 0);
    assert_eq!(hand_count(&engine, "HolyWater"), 0);
    assert_eq!(discard_prefix_count(&engine, "HolyWater"), 0);
}

#[test]
fn relic_wave15_ninja_scroll_fills_hand_then_spills_overflow_to_discard() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("NinjaScroll".to_string());
    engine.state.hand = make_deck_n("Strike", 9);

    fire_combat_start(&mut engine);

    assert_eq!(hand_count(&engine, "Shiv"), 1);
    assert_eq!(discard_prefix_count(&engine, "Shiv"), 2);
}

#[test]
fn relic_wave15_mark_of_pain_adds_two_wounds_to_draw_pile_at_combat_start() {
    // Source: reference/extracted/methods/relic/MarkOfPain.java and
    // CardGroup.java::addToRandomSpot. Each Wound consumes one cardRandomRng
    // tick; the draw-pile shuffle stream is untouched.
    let mut engine = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    engine.state.relics.push("Mark of Pain".to_string());
    let before = engine.rng_counters();

    fire_combat_start(&mut engine);

    assert_eq!(draw_prefix_count(&engine, "Wound"), 2);
    assert_eq!(engine.rng_counters()["cardRandom"], before["cardRandom"] + 2);
    assert_eq!(engine.rng_counters()["card"], before["card"]);
}

#[test]
fn relic_wave15_mutagenic_strength_applies_temporary_strength_at_combat_start() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("MutagenicStrength".to_string());

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::STRENGTH), 3);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 3);
}
