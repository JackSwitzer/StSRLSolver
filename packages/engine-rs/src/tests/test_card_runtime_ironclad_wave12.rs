#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Havoc.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/SwordBoomerang.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
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

fn hand_names(engine: &crate::engine::CombatEngine) -> Vec<String> {
    engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id).to_string())
        .collect()
}

#[test]
fn ironclad_wave12_registry_exports_promote_the_typed_surface_where_supported() {
    let burning_pact = global_registry()
        .get("Burning Pact")
        .expect("Burning Pact should exist");
    assert_eq!(burning_pact.card_type, CardType::Skill);
    assert_eq!(burning_pact.target, CardTarget::None);
    assert_eq!(
        burning_pact.effect_data,
        &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(burning_pact.complex_hook.is_some());

    let sword_boomerang = global_registry()
        .get("Sword Boomerang")
        .expect("Sword Boomerang should exist");
    assert_eq!(sword_boomerang.card_type, CardType::Attack);
    assert_eq!(sword_boomerang.target, CardTarget::AllEnemy);
    assert_eq!(
        sword_boomerang.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::ExtraHits(A::Magic),
        ]
    );
    assert!(sword_boomerang.complex_hook.is_none());

    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.card_type, CardType::Skill);
    assert_eq!(havoc.target, CardTarget::None);
    assert_eq!(havoc.effect_data, &[E::Simple(SE::PlayTopCardOfDraw)]);
    assert!(havoc.complex_hook.is_none());

    let true_grit = global_registry().get("True Grit").expect("True Grit should exist");
    assert_eq!(
        true_grit.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ExhaustRandomCardFromHand),
        ]
    );
    assert!(true_grit.complex_hook.is_none());

    let dual_wield = global_registry().get("Dual Wield").expect("Dual Wield should exist");
    assert!(dual_wield.effect_data.is_empty());
    assert!(dual_wield.complex_hook.is_some());

    let fiend_fire = global_registry().get("Fiend Fire").expect("Fiend Fire should exist");
    assert!(fiend_fire.effect_data.is_empty());
    assert!(fiend_fire.complex_hook.is_some());

    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.effect_data, &[E::Simple(SE::PlayTopCardOfDraw)]);
    assert!(havoc.complex_hook.is_none());
}

#[test]
fn ironclad_wave12_burning_pact_uses_declarative_choice_and_deferred_draw() {
    let mut engine = engine_for(
        &["Burning Pact", "Strike_R"],
        &["Defend_R", "Bash"],
        &[],
        3,
    );

    assert!(play_self(&mut engine, "Burning Pact"));
    assert_eq!(engine.phase, crate::engine::CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().unwrap().reason, crate::engine::ChoiceReason::ExhaustFromHand);

    engine.execute_action(&crate::actions::Action::Choose(0));

    let names = hand_names(&engine);
    assert_eq!(engine.phase, crate::engine::CombatPhase::PlayerTurn);
    assert_eq!(engine.state.exhaust_pile.len(), 1);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Defend_R".to_string()));
    assert!(names.contains(&"Bash".to_string()));
}

#[test]
fn ironclad_wave12_sword_boomerang_uses_typed_random_enemy_extra_hits() {
    let mut engine = engine_for(&["Sword Boomerang"], &[], &[], 3);
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Sword Boomerang", 0));

    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 9);
    assert_eq!(engine.state.discard_pile.len(), 1);
    assert_eq!(
        engine
            .card_registry
            .card_name(engine.state.discard_pile.last().expect("discarded sword boomerang").def_id),
        "Sword Boomerang"
    );
}

#[test]
fn ironclad_wave12_true_grit_base_uses_the_typed_random_exhaust_surface() {
    let true_grit = global_registry().get("True Grit").expect("True Grit");
    assert_eq!(
        true_grit.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ExhaustRandomCardFromHand),
        ]
    );
    assert!(true_grit.complex_hook.is_none());
}

#[test]
#[ignore = "Blocked on Java attack-or-power union filtering for Dual Wield; the current declarative filter surface cannot express the card's option set. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/DualWield.java"]
fn ironclad_wave12_dual_wield_stays_explicitly_hook_backed() {}

#[test]
#[ignore = "Blocked on Java exhaust/per-hit sequencing for Fiend Fire; the current hook still owns the hand-exhaust + per-card damage loop. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/FiendFire.java"]
fn ironclad_wave12_fiend_fire_stays_explicitly_hook_backed() {}

#[test]
fn ironclad_wave12_havoc_uses_the_typed_play_top_card_surface() {
    let havoc = global_registry().get("Havoc").expect("Havoc should exist");
    assert_eq!(havoc.effect_data, &[E::Simple(SE::PlayTopCardOfDraw)]);
    assert!(havoc.complex_hook.is_none());
}
