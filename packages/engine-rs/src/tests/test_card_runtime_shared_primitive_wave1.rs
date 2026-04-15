#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EscapePlan.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::engine::{ChoiceReason, CombatPhase};
use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self,
};

#[test]
fn shared_primitive_wave1_registry_exports_cover_alchemize_and_reaper() {
    let registry = global_registry();

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert_eq!(alchemize.effect_data, &[E::Simple(SE::ObtainRandomPotion)]);
    assert!(alchemize.complex_hook.is_none());

    let reaper = registry.get("Reaper").expect("Reaper should exist");
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
fn shared_primitive_wave1_alchemize_obtains_a_random_potion_and_exhausts() {
    let mut engine = crate::tests::support::engine_with(make_deck(&["Alchemize"]), 50, 0);

    assert!(play_self(&mut engine, "Alchemize"));
    assert_eq!(crate::tests::support::exhaust_prefix_count(&engine, "Alchemize"), 1);
    assert!(
        engine.state.potions.iter().any(|p| !p.is_empty()),
        "Alchemize should obtain a potion into the first empty slot"
    );
}

#[test]
fn shared_primitive_wave1_reaper_heals_for_total_unblocked_damage() {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 20, 20),
            enemy_no_intent("Cultist", 20, 20),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.player.hp = 30;
    engine.state.player.max_hp = 50;
    engine.state.hand = make_deck(&["Reaper"]);

    assert!(play_on_enemy(&mut engine, "Reaper", 0));
    assert_eq!(engine.state.player.hp, 38);
    assert_eq!(engine.state.enemies[0].entity.hp, 16);
    assert_eq!(engine.state.enemies[1].entity.hp, 16);
}

#[test]
fn shared_primitive_wave1_omniscience_uses_the_typed_draw_pile_free_play_surface() {
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
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }]
    );
    assert!(omniscience.complex_hook.is_none());

    let mut engine = engine_with_state(combat_state_with(
        make_deck(&["Omniscience", "Strike_P", "Defend_P"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    ));
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine.state.energy = 4;
    engine.state.hand = make_deck(&["Omniscience"]);
    engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P"]);

    assert!(play_self(&mut engine, "Omniscience"));
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.choice.as_ref().expect("choice").reason, ChoiceReason::PlayCardFreeFromDraw);

    engine.execute_action(&crate::actions::Action::Choose(0));

    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    assert_eq!(engine.state.hand.len(), 1);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Strike_P");
    assert_eq!(engine.state.hand[0].cost, 0);
}
