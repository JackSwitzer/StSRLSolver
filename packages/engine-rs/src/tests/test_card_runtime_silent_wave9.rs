#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EndlessAgony.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/GrandFinale.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/MasterfulStab.java

use crate::actions::Action;
use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

#[test]
fn silent_wave9_registry_exports_show_clean_primary_typed_effects() {
    let registry = global_registry();

    let endless_agony = registry.get("Endless Agony").expect("Endless Agony should exist");
    assert_eq!(endless_agony.card_type, CardType::Attack);
    assert_eq!(endless_agony.target, CardTarget::Enemy);
    assert!(endless_agony.exhaust);
    assert!(endless_agony.has_test_marker("copy_on_draw"));
    assert_eq!(
        endless_agony.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let grand_finale = registry.get("Grand Finale").expect("Grand Finale should exist");
    assert_eq!(grand_finale.target, CardTarget::AllEnemy);
    assert!(grand_finale.has_test_marker("only_empty_draw"));
    assert_eq!(
        grand_finale.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let masterful_stab = registry
        .get("Masterful Stab")
        .expect("Masterful Stab should exist");
    assert!(masterful_stab.has_test_marker("cost_increase_on_hp_loss"));
    assert_eq!(
        masterful_stab.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert_eq!(alchemize.effect_data, &[E::Simple(SE::ObtainRandomPotion)]);
    assert!(alchemize.complex_hook.is_none());

}

#[test]
fn silent_wave9_primary_typed_damage_cards_follow_engine_path() {
    let mut agony_engine = engine_with(make_deck(&["Endless Agony", "Strike"]), 40, 0);
    agony_engine.state.hand = make_deck(&["Endless Agony"]);
    let hp_before = agony_engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut agony_engine, "Endless Agony", 0));
    assert_eq!(agony_engine.state.enemies[0].entity.hp, hp_before - 4);
    assert_eq!(exhaust_prefix_count(&agony_engine, "Endless Agony"), 1);

    let mut finale_engine = engine_with(Vec::new(), 70, 0);
    finale_engine.state.hand = make_deck(&["Grand Finale"]);
    finale_engine.state.draw_pile.clear();
    finale_engine.state.discard_pile.clear();
    finale_engine.state.enemies.push(enemy_no_intent("Cultist", 55, 55));
    assert!(play_self(&mut finale_engine, "Grand Finale"));
    assert_eq!(finale_engine.state.enemies[0].entity.hp, 20);
    assert_eq!(finale_engine.state.enemies[1].entity.hp, 5);

    let mut stab_engine = engine_with(make_deck(&["Masterful Stab"]), 50, 0);
    stab_engine.state.hand = make_deck(&["Masterful Stab"]);
    stab_engine.state.total_damage_taken = 0;
    let hp_before = stab_engine.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut stab_engine, "Masterful Stab", 0));
    assert_eq!(stab_engine.state.enemies[0].entity.hp, hp_before - 12);
}

#[test]
fn grand_finale_requires_an_empty_draw_pile_and_upgrade_deals_sixty_to_all() {
    // Java: reference/extracted/methods/card/GrandFinale.java
    // canUse rejects any non-empty draw pile; baseDamage is 50 and upgradeDamage(10).
    let mut blocked = engine_with(Vec::new(), 100, 0);
    blocked.state.hand = make_deck(&["Grand Finale"]);
    blocked.state.draw_pile = make_deck(&["Strike"]);
    blocked.state.discard_pile.clear();
    assert!(!blocked.get_legal_actions().iter().any(|action| {
        matches!(action, Action::PlayCard { card_idx: 0, target_idx: -1 })
    }));

    let mut upgraded = engine_with(Vec::new(), 100, 0);
    upgraded.state.hand = make_deck(&["Grand Finale+"]);
    upgraded.state.draw_pile.clear();
    upgraded.state.discard_pile.clear();
    upgraded.state.enemies.push(enemy_no_intent("Cultist", 80, 80));
    upgraded.state.energy = 2;

    assert!(play_self(&mut upgraded, "Grand Finale+"));
    assert_eq!(upgraded.state.enemies[0].entity.hp, 40);
    assert_eq!(upgraded.state.enemies[1].entity.hp, 20);
    assert_eq!(upgraded.state.energy, 2);
}

#[test]
fn silent_wave9_existing_runtime_tags_still_drive_residual_semantics() {
    // EndlessAgony.triggerWhenDrawn creates a stat-equivalent copy. With room,
    // both copies enter the hand.
    // Java: reference/extracted/methods/card/EndlessAgony.java
    let mut agony_engine = engine_without_start(
        make_deck(&["Endless Agony"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    agony_engine.state.hand.clear();
    agony_engine.draw_cards(1);
    assert_eq!(hand_count(&agony_engine, "Endless Agony"), 2);

    // If drawing the original fills the tenth slot, MakeTempCardInHandAction
    // sends the upgraded stat-equivalent copy to discard instead of deleting it.
    let mut overflow = engine_without_start(
        make_deck(&["Endless Agony+"]),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    overflow.state.hand = make_deck(&[
        "Defend", "Defend", "Defend", "Defend", "Defend",
        "Defend", "Defend", "Defend", "Defend",
    ]);

    overflow.draw_cards(1);

    assert_eq!(overflow.state.hand.len(), 10);
    assert_eq!(hand_count(&overflow, "Endless Agony+"), 1);
    assert_eq!(discard_prefix_count(&overflow, "Endless Agony+"), 1);
}
