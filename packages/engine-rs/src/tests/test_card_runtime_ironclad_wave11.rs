#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Entrench.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SwordBoomerang.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::tests::support::*;

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> crate::engine::CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
    let mut engine = crate::engine::CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn ironclad_wave11_registry_exports_promote_the_typed_surface() {
    let entrench = global_registry().get("Entrench").expect("Entrench should exist");
    assert_eq!(entrench.card_type, CardType::Skill);
    assert_eq!(entrench.target, CardTarget::SelfTarget);
    assert_eq!(entrench.effect_data, &[E::Simple(SE::GainBlock(A::PlayerBlock))]);
    assert!(entrench.complex_hook.is_none());

    let dual_wield = global_registry().get("Dual Wield").expect("Dual Wield should exist");
    assert_eq!(dual_wield.card_type, CardType::Skill);
    assert_eq!(dual_wield.target, CardTarget::None);
    assert!(dual_wield.effect_data.is_empty());
    assert!(dual_wield.complex_hook.is_some());

    let fiend_fire = global_registry().get("Fiend Fire").expect("Fiend Fire should exist");
    assert!(fiend_fire.effect_data.is_empty());
    assert!(fiend_fire.complex_hook.is_some());

    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.effect_data, &[E::Simple(SE::PlayTopCardOfDraw)]);
    assert!(havoc.complex_hook.is_none());

    let sword_boomerang = global_registry()
        .get("Sword Boomerang")
        .expect("Sword Boomerang should exist");
    assert!(sword_boomerang.effect_data.is_empty());
    assert!(sword_boomerang.complex_hook.is_some());
}

#[test]
fn ironclad_wave11_entrench_follows_the_typed_surface() {
    let mut entrench = engine_for(&["Entrench"], &[], &[], 3);
    entrench.state.player.block = 13;
    assert!(play_self(&mut entrench, "Entrench"));
    assert_eq!(entrench.state.player.block, 26);
}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave11_dual_wield_stays_explicitly_hook_backed() {
    let dual_wield = global_registry().get("Dual Wield").expect("Dual Wield should exist");
    assert!(dual_wield.effect_data.is_empty());
    assert!(dual_wield.complex_hook.is_some());
}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave11_fiend_fire_stays_explicitly_hook_backed() {
    let fiend_fire = global_registry().get("Fiend Fire").expect("Fiend Fire should exist");
    assert!(fiend_fire.effect_data.is_empty());
    assert!(fiend_fire.complex_hook.is_some());
}

#[test]
fn ironclad_wave11_havoc_uses_the_typed_play_top_card_surface() {
    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.effect_data, &[E::Simple(SE::PlayTopCardOfDraw)]);
    assert!(havoc.complex_hook.is_none());
}

#[test]
#[ignore = "Blocked on Java random-enemy multi-hit sequencing for Sword Boomerang; the current runtime still needs a richer random-hit primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SwordBoomerang.java"]
fn ironclad_wave11_sword_boomerang_stays_explicitly_hook_backed() {
    let sword_boomerang = global_registry()
        .get("Sword Boomerang")
        .expect("Sword Boomerang should exist");
    assert!(sword_boomerang.effect_data.is_empty());
    assert!(sword_boomerang.complex_hook.is_some());
}
