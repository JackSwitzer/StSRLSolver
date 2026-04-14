#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretTechnique.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Headbutt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java

use crate::effects::declarative::{
    AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE,
    Target as T,
};
use crate::engine::{ChoiceReason, CombatEngine, CombatPhase};
use crate::tests::support::{
    combat_state_with, enemy_no_intent, force_player_turn, make_deck, play_on_enemy, play_self,
    set_stance, TEST_SEED,
};
use crate::state::Stance;

fn engine_for(hand: &[&str], draw: &[&str], discard: &[&str], energy: i32) -> CombatEngine {
    let mut state = combat_state_with(
        make_deck(draw),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        energy,
    );
    state.hand = make_deck(hand);
    state.discard_pile = make_deck(discard);
    let mut engine = CombatEngine::new(state, TEST_SEED);
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn headbutt_now_exports_typed_primary_damage_while_kept_hook_backed_for_discard_choice() {
    let registry = crate::cards::global_registry();
    let headbutt = registry.get("Headbutt").expect("Headbutt should exist");
    assert_eq!(
        headbutt.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(headbutt.complex_hook.is_some());
}

#[test]
fn headbutt_still_deals_damage_and_opens_discard_pick_choice() {
    let mut engine = engine_for(&["Headbutt"], &[], &["Strike_R", "Defend_R"], 3);

    assert!(play_on_enemy(&mut engine, "Headbutt", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 51);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    let choice = engine.choice.as_ref().expect("headbutt choice");
    assert_eq!(choice.reason, ChoiceReason::PickFromDiscard);
    assert_eq!(choice.options.len(), 2);
}

#[test]
fn secret_technique_still_uses_declarative_skill_search_and_finds_only_skills() {
    let registry = crate::cards::global_registry();
    let card = registry
        .get("Secret Technique")
        .expect("Secret Technique should be registered");
    assert!(card.complex_hook.is_none());
    assert_eq!(
        card.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::Skills,
            action: ChoiceAction::MoveToHand,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );

    let mut engine = engine_for(&["Secret Technique"], &["Strike_R", "Shrug It Off", "Bash"], &[], 3);
    assert!(play_self(&mut engine, "Secret Technique"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("secret technique choice");
    assert_eq!(choice.reason, ChoiceReason::SearchDrawPile);
    assert_eq!(choice.options.len(), 1);
}

#[test]
fn tranquility_remains_honestly_on_shared_enter_stance_metadata_in_this_slice() {
    let registry = crate::cards::global_registry();
    let tranquility = registry
        .get("ClearTheMind")
        .expect("Tranquility should exist under its Java id");
    assert_eq!(tranquility.enter_stance, Some("Calm"));
    assert!(tranquility.effect_data.is_empty());

    let mut engine = engine_for(&["ClearTheMind+"], &[], &[], 3);
    set_stance(&mut engine, Stance::Wrath);
    assert!(play_self(&mut engine, "ClearTheMind+"));
    assert_eq!(engine.state.stance, Stance::Calm);
}

#[test]
#[ignore = "Secret Technique can_use legality still needs the shared can_play surface; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/SecretTechnique.java"]
fn secret_technique_is_illegal_when_draw_pile_has_no_skills() {}

#[test]
#[ignore = "Violence still needs a typed capped filtered draw-to-hand primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java"]
fn violence_remains_hook_backed_until_capped_attack_fetch_is_typed() {}

#[test]
#[ignore = "Burning Pact still needs post-choice follow-up sequencing on the declarative path; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java"]
fn burning_pact_remains_hook_backed_until_draw_after_choice_is_typed() {}

#[test]
#[ignore = "Tranquility still uses shared enter_stance metadata; moving it cleanly needs the coordinated metadata cleanup path in engine.rs and registry tests. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java"]
fn tranquility_typed_stance_migration_remains_queued() {}
