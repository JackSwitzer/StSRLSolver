#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Condition as Cond, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::tests::support::*;

fn one_enemy_engine(enemy_hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", enemy_hp, enemy_hp)],
        energy,
    );
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn ironclad_wave10_registry_exports_promote_the_typed_primary_surface() {
    for card_id in ["Rampage", "Rampage+"] {
        let card = global_registry()
            .get(card_id)
            .unwrap_or_else(|| panic!("{card_id} should exist"));
        assert_eq!(card.card_type, CardType::Attack);
        assert_eq!(card.target, CardTarget::Enemy);
        assert_eq!(
            card.effect_data,
            &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
            ],
            "{card_id} should declare a typed primary attack",
        );
        assert!(
            card.uses_typed_primary_preamble(),
            "{card_id} should use the typed primary preamble"
        );
        assert!(card.effects.contains(&"rampage"));
        assert!(card.complex_hook.is_none(), "{card_id} should be fully typed");
    }

    let true_grit = global_registry()
        .get("True Grit")
        .expect("True Grit should exist");
    assert_eq!(true_grit.card_type, CardType::Skill);
    assert_eq!(true_grit.target, CardTarget::SelfTarget);
    assert_eq!(
        true_grit.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ExhaustRandomCardFromHand),
        ]
    );
    assert!(true_grit.uses_typed_primary_preamble());
    assert!(true_grit.complex_hook.is_none());

    let true_grit_plus = global_registry()
        .get("True Grit+")
        .expect("True Grit+ should exist");
    assert_eq!(
        true_grit_plus.effect_data,
        &[E::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
}

#[test]
fn ironclad_wave10_rampage_and_true_grit_follow_the_typed_primary_surface() {
    let mut rampage = one_enemy_engine(40, 3);
    ensure_in_hand(&mut rampage, "Rampage+");
    assert!(play_on_enemy(&mut rampage, "Rampage+", 0));
    assert_eq!(rampage.state.enemies[0].entity.hp, 32);
    let played_once = rampage
        .state
        .discard_pile
        .last()
        .expect("played Rampage+ should be in discard");
    assert_eq!(played_once.misc, 16, "Rampage+ should store its next damage on the played copy");

    let mut true_grit = one_enemy_engine(40, 3);
    true_grit.state.hand = make_deck(&["True Grit", "Strike_R", "Defend_R"]);
    assert!(play_self(&mut true_grit, "True Grit"));
    assert_eq!(true_grit.state.player.block, 7);
    assert_eq!(true_grit.state.exhaust_pile.len(), 1);
    assert_eq!(discard_prefix_count(&true_grit, "True Grit"), 1);
}

#[test]
fn ironclad_wave10_burning_pact_uses_choice_owned_deferred_draw_follow_up() {
    let burning_pact = global_registry()
        .get("Burning Pact")
        .expect("Burning Pact should exist");
    assert_eq!(
        burning_pact.effect_data,
        &[E::ChooseCards {
            source: crate::effects::declarative::Pile::Hand,
            filter: crate::effects::declarative::CardFilter::All,
            action: crate::effects::declarative::ChoiceAction::Exhaust,
            min_picks: A::Fixed(1),
            max_picks: A::Fixed(1),
        }]
    );
    assert!(burning_pact.complex_hook.is_some());
}

#[test]
fn ironclad_wave10_feed_and_reaper_follow_the_typed_attack_surface() {
    let feed = global_registry().get("Feed").expect("Feed should exist");
    assert_eq!(
        feed.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilled,
                &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(feed.complex_hook.is_none());

    let reaper = global_registry().get("Reaper").expect("Reaper should exist");
    assert_eq!(
        reaper.effect_data,
        &[
            E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
            E::Simple(SE::HealHp(T::Player, A::TotalUnblockedDamage)),
        ]
    );
    assert!(reaper.complex_hook.is_none());
}

#[test]
fn ironclad_wave10_true_grit_base_uses_the_typed_random_exhaust_surface() {
    let true_grit = global_registry()
        .get("True Grit")
        .expect("True Grit should exist");
    assert_eq!(
        true_grit.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ExhaustRandomCardFromHand),
        ]
    );
    assert!(true_grit.complex_hook.is_none());
}
