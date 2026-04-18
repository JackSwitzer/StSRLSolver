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

#[test]
fn relic_wave15_holy_water_fills_hand_then_spills_overflow_to_discard() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("HolyWater".to_string());
    engine.state.hand = make_deck_n("Strike", 9);

    fire_combat_start(&mut engine);

    assert_eq!(hand_count(&engine, "HolyWater"), 1);
    assert_eq!(discard_prefix_count(&engine, "HolyWater"), 2);
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
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("Mark of Pain".to_string());

    fire_combat_start(&mut engine);

    assert_eq!(draw_prefix_count(&engine, "Wound"), 2);
}

#[test]
fn relic_wave15_mutagenic_strength_applies_temporary_strength_at_combat_start() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.state.relics.push("MutagenicStrength".to_string());

    engine.start_combat();

    assert_eq!(engine.state.player.status(sid::STRENGTH), 3);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 3);
}
