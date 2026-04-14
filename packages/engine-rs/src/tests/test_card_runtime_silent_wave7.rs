#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Adrenaline.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Blur.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Footwork.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/PiercingWail.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Prepared.java
use crate::actions::Action;
use crate::cards::global_registry;
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn silent_wave7_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let adrenaline = registry.get("Adrenaline").expect("Adrenaline should be registered");
    assert_eq!(
        adrenaline.effect_data,
        &[
            E::Simple(SE::GainEnergy(A::Magic)),
            E::Simple(SE::DrawCards(A::Fixed(2))),
        ]
    );
    assert_eq!(adrenaline.base_magic, 1);

    let adrenaline_plus = registry.get("Adrenaline+").expect("Adrenaline+ should be registered");
    assert_eq!(adrenaline_plus.base_magic, 2);

    let blur = registry.get("Blur").expect("Blur should be registered");
    assert_eq!(
        blur.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::BLUR, A::Fixed(1)))]
    );

    let footwork = registry.get("Footwork").expect("Footwork should be registered");
    assert_eq!(
        footwork.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic))]
    );

    let prepared = registry.get("Prepared").expect("Prepared should be registered");
    assert_eq!(prepared.effect_data[0], E::Simple(SE::DrawCards(A::Magic)));

}

#[test]
fn silent_wave7_adrenaline_blur_and_footwork_run_on_engine_path() {
    let mut adrenaline = one_enemy_engine("JawWorm", 50, 0);
    adrenaline.state.draw_pile = make_deck(&["Strike_G", "Defend_G", "Neutralize"]);
    let draw_before = adrenaline.state.draw_pile.len();
    ensure_in_hand(&mut adrenaline, "Adrenaline");
    assert!(play_self(&mut adrenaline, "Adrenaline"));
    assert_eq!(adrenaline.state.energy, 4);
    assert_eq!(adrenaline.state.draw_pile.len(), draw_before - 2);
    assert!(exhaust_prefix_count(&adrenaline, "Adrenaline") >= 1);

    let mut adrenaline_plus = one_enemy_engine("JawWorm", 50, 0);
    adrenaline_plus.state.draw_pile = make_deck(&["Strike_G", "Defend_G", "Neutralize"]);
    ensure_in_hand(&mut adrenaline_plus, "Adrenaline+");
    assert!(play_self(&mut adrenaline_plus, "Adrenaline+"));
    assert_eq!(adrenaline_plus.state.energy, 5);

    let mut blur = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut blur, "Blur");
    assert!(play_self(&mut blur, "Blur"));
    assert_eq!(blur.state.player.block, 5);
    end_turn(&mut blur);
    assert_eq!(blur.state.player.block, 5);
    assert_eq!(blur.state.player.status(sid::BLUR), 0);

    let mut footwork = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut footwork, "Footwork+");
    ensure_in_hand(&mut footwork, "Defend_G");
    assert!(play_self(&mut footwork, "Footwork+"));
    assert_eq!(footwork.state.player.status(sid::DEXTERITY), 3);
    assert!(play_self(&mut footwork, "Defend_G"));
    assert_eq!(footwork.state.player.block, 8);
}

#[test]
fn silent_wave7_prepared_uses_draw_then_discard_choice_on_engine_path() {
    let mut engine = one_enemy_engine("JawWorm", 50, 0);
    engine.state.draw_pile = make_deck(&["Strike_G", "Defend_G", "Neutralize"]);
    engine.state.hand = make_deck(&["Prepared+", "Survivor"]);

    assert!(play_self(&mut engine, "Prepared+"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let choice = engine.choice.as_ref().expect("Prepared+ discard choice");
    assert_eq!(choice.reason, ChoiceReason::DiscardFromHand);
    assert_eq!(choice.min_picks, 2);
    assert_eq!(choice.max_picks, 2);

    let mut discard_hand_indices = Vec::new();
    for option in &choice.options {
        let ChoiceOption::HandCard(hand_idx) = option else {
            continue;
        };
        discard_hand_indices.push(*hand_idx);
    }

    assert!(discard_hand_indices.len() >= 2);
    engine.execute_action(&Action::Choose(discard_hand_indices[0]));
    engine.execute_action(&Action::Choose(discard_hand_indices[1]));
    engine.execute_action(&Action::ConfirmSelection);

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.player.status(sid::DISCARDED_THIS_TURN), 2);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.discard_pile.len(), 2);
}
