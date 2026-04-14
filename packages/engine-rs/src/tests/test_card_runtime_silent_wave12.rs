#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Concentrate.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Expertise.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Reflex.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java

use crate::cards::global_registry;
use crate::tests::support::*;

#[test]
fn silent_wave12_registry_documents_the_remaining_silent_blockers() {
    let registry = global_registry();

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert!(alchemize.effect_data.is_empty());
    assert!(alchemize.complex_hook.is_some());

    let calculated_gamble = registry
        .get("Calculated Gamble")
        .expect("Calculated Gamble should exist");
    assert!(calculated_gamble.effect_data.is_empty());
    assert!(calculated_gamble.complex_hook.is_some());

    let concentrate = registry.get("Concentrate").expect("Concentrate should exist");
    assert!(concentrate.effect_data.is_empty());
    assert!(concentrate.complex_hook.is_some());

    let expertise = registry.get("Expertise").expect("Expertise should exist");
    assert!(expertise.effect_data.is_empty());
    assert!(expertise.complex_hook.is_some());

    let nightmare = registry.get("Nightmare").expect("Nightmare should exist");
    assert!(nightmare.effect_data.is_empty());
    assert!(nightmare.complex_hook.is_some());

    let reflex = registry.get("Reflex").expect("Reflex should exist");
    assert!(reflex.effect_data.is_empty());
    assert!(reflex.complex_hook.is_none());
    assert!(reflex.effects.contains(&"draw_on_discard"));

    let storm_of_steel = registry
        .get("Storm of Steel")
        .expect("Storm of Steel should exist");
    assert!(storm_of_steel.effect_data.is_empty());
    assert!(storm_of_steel.complex_hook.is_some());

    let tactician = registry.get("Tactician").expect("Tactician should exist");
    assert!(tactician.effect_data.is_empty());
    assert!(tactician.complex_hook.is_none());
    assert!(tactician.effects.contains(&"energy_on_discard"));
}

#[test]
fn silent_wave12_runtime_backed_residuals_still_follow_their_discard_hooks() {
    let mut reflex_engine = engine_without_start(
        make_deck(&["Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    let reflex = reflex_engine.card_registry.make_card("Reflex+");
    reflex_engine.state.discard_pile.push(reflex);
    reflex_engine.on_card_discarded(reflex);
    assert_eq!(reflex_engine.state.hand.len(), 3);

    let mut tactician_engine = engine_without_start(
        make_deck(&["Strike_G"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        1,
    );
    let tactician = tactician_engine.card_registry.make_card("Tactician+");
    tactician_engine.state.discard_pile.push(tactician);
    tactician_engine.on_card_discarded(tactician);
    assert_eq!(tactician_engine.state.energy, 3);
}

#[test]
#[ignore = "Alchemize still needs a typed random-potion generation effect on the canonical runtime path; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java"]
fn silent_wave12_alchemize_needs_typed_random_potion_generation() {}

#[test]
#[ignore = "Calculated Gamble still needs a typed discard-then-draw-count primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/CalculatedGamble.java"]
fn silent_wave12_calculated_gamble_needs_typed_discard_then_draw_count() {}

#[test]
#[ignore = "Concentrate still needs a typed discard-then-gain-energy post-choice primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Concentrate.java"]
fn silent_wave12_concentrate_needs_typed_discard_then_gain_energy() {}

#[test]
#[ignore = "Expertise still needs a draw-to-N runtime primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Expertise.java"]
fn silent_wave12_expertise_needs_draw_to_n_runtime_primitive() {}

#[test]
#[ignore = "Nightmare still needs a delayed next-turn copy/install primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java"]
fn silent_wave12_nightmare_needs_delayed_copy_install_primitive() {}

#[test]
#[ignore = "Storm of Steel still needs a typed discard-hand-then-create-Shiv-per-card primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/StormOfSteel.java"]
fn silent_wave12_storm_of_steel_needs_discard_then_spawn_shivs() {}
