#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RitualDaggerAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::engine::CombatPhase;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy};

#[test]
fn colorless_wave6_registry_exports_match_typed_surface_for_mind_blast() {
    let registry = global_registry();

    let mind_blast = registry.get("Mind Blast").expect("Mind Blast should exist");
    assert_eq!(
        mind_blast.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::DrawPileSize))]
    );
    assert!(mind_blast.complex_hook.is_none());
}

#[test]
fn mind_blast_uses_draw_pile_size_for_attack_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Mind Blast"]);
    engine.state.draw_pile = make_deck(&["Strike_R", "Defend_R", "Strike_R"]);

    assert!(play_on_enemy(&mut engine, "Mind Blast", 0));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.enemies[0].entity.hp, 37);
}

#[test]
#[ignore = "Forethought still needs the single-card auto-resolve primitive; Java moves the only card directly without opening the hand-select screen."]
fn forethought_still_needs_single_card_auto_resolve_primitive() {}

#[test]
#[ignore = "Impatience still needs a no-attacks-in-hand primitive; Java checks the current hand contents before drawing."]
fn impatience_still_needs_no_attacks_in_hand_primitive() {}

#[test]
#[ignore = "Madness still needs a random-hand-card zero-cost primitive; Java repeatedly samples the hand until it finds a card that can be reduced."]
fn madness_still_needs_random_hand_card_zero_cost_primitive() {}

#[test]
#[ignore = "Violence still needs a typed draw-pile attack fetch primitive; Java pulls random attacks from the draw pile into hand."]
fn violence_still_needs_draw_pile_attack_fetch_primitive() {}
