#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ForeignInfluence.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, GeneratedCardPool, GeneratedCostRule, Pile as P};
use crate::engine::{ChoiceOption, ChoiceReason, CombatPhase};
use crate::tests::support::{combat_state_with, enemy_no_intent, engine_with_state, ensure_in_hand, play_self};

fn watcher_engine() -> crate::engine::CombatEngine {
    engine_with_state(combat_state_with(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ))
}

fn first_positive_cost_choice(choice: &[crate::engine::ChoiceOption]) -> usize {
    choice
        .iter()
        .enumerate()
        .find_map(|(idx, option)| {
            let ChoiceOption::GeneratedCard(card) = option else {
                return None;
            };
            (card.cost > 0).then_some(idx)
        })
        .unwrap_or(0)
}

#[test]
fn watcher_wave17_registry_exports_foreign_influence_as_typed_generated_choice() {
    let registry = global_registry();

    let foreign = registry
        .get("ForeignInfluence")
        .expect("Foreign Influence should be registered");
    assert_eq!(
        foreign.effect_data,
        &[E::GenerateDiscoveryChoice {
            pool: GeneratedCardPool::Attack,
            option_count: 3,
            cost_rule: GeneratedCostRule::Base,
        }]
    );
    assert!(foreign.complex_hook.is_none());

    let foreign_plus = registry
        .get("ForeignInfluence+")
        .expect("Foreign Influence+ should be registered");
    assert_eq!(
        foreign_plus.effect_data,
        &[E::GenerateDiscoveryChoice {
            pool: GeneratedCardPool::Attack,
            option_count: 3,
            cost_rule: GeneratedCostRule::ZeroThisTurn,
        }]
    );
    assert!(foreign_plus.complex_hook.is_none());
}

#[test]
fn watcher_wave17_foreign_influence_preserves_base_cost_and_zeros_upgraded_cost() {
    let mut engine = watcher_engine();
    ensure_in_hand(&mut engine, "ForeignInfluence");
    assert!(play_self(&mut engine, "ForeignInfluence"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);

    let choice = engine.choice.as_ref().expect("Foreign Influence should open a generated choice");
    assert_eq!(choice.reason, ChoiceReason::DiscoverCard);
    assert_eq!(choice.options.len(), 3);
    for option in &choice.options {
        let ChoiceOption::GeneratedCard(card) = option else {
            panic!("Foreign Influence should present generated-card options");
        };
        let def = engine.card_registry.card_def_by_id(card.def_id);
        assert_eq!(def.card_type, crate::cards::CardType::Attack);
    }

    let chosen_idx = first_positive_cost_choice(&choice.options);
    let preview_cost = match &choice.options[chosen_idx] {
        ChoiceOption::GeneratedCard(card) => card.cost,
        _ => unreachable!(),
    };
    engine.execute_action(&crate::actions::Action::Choose(chosen_idx));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.state.hand[0].cost, preview_cost);

    let mut upgraded = watcher_engine();
    ensure_in_hand(&mut upgraded, "ForeignInfluence+");
    assert!(play_self(&mut upgraded, "ForeignInfluence+"));
    let upgraded_choice = upgraded
        .choice
        .as_ref()
        .expect("Foreign Influence+ should open a generated choice");
    let upgraded_idx = first_positive_cost_choice(&upgraded_choice.options);
    upgraded.execute_action(&crate::actions::Action::Choose(upgraded_idx));
    assert_eq!(upgraded.phase, CombatPhase::PlayerTurn);
    assert_eq!(upgraded.state.hand.len(), 1);
    assert_eq!(upgraded.state.hand[0].cost, 0);
}

#[test]
#[ignore = "Deus Ex Machina still needs an explicit draw-trigger card instance primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java"]
fn watcher_wave17_deus_ex_machina_stays_queued_until_draw_trigger_card_instance_exists() {}

#[test]
#[ignore = "Judgement still needs a typed enemy-HP threshold kill primitive; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java"]
fn watcher_wave17_judgement_stays_queued_until_threshold_kill_primitive_exists() {}

#[test]
fn watcher_wave17_omniscience_uses_the_typed_draw_pile_free_play_surface() {
    let omniscience = global_registry()
        .get("Omniscience")
        .expect("Omniscience should be registered");
    assert_eq!(
        omniscience.effect_data,
        &[E::ChooseCards {
            source: P::Draw,
            filter: CardFilter::All,
            action: ChoiceAction::PlayForFree,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(omniscience.complex_hook.is_none());
}

#[test]
#[ignore = "Wish still needs payload-driven option resolution for Strength, Gold, and Plated Armor branches on the canonical decision surface; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java"]
fn watcher_wave17_wish_stays_queued_until_payload_driven_option_resolution_exists() {}
