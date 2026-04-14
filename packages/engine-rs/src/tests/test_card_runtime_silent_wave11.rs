#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Reflex.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Shiv.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

#[test]
fn silent_wave11_registry_exports_show_typed_primary_surface_for_shiv() {
    let registry = global_registry();

    let shiv = registry.get("Shiv").expect("Shiv should exist");
    assert_eq!(shiv.card_type, CardType::Attack);
    assert_eq!(shiv.target, CardTarget::Enemy);
    assert!(shiv.exhaust);
    assert_eq!(
        shiv.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let shiv_plus = registry.get("Shiv+").expect("Shiv+ should exist");
    assert_eq!(shiv_plus.card_type, CardType::Attack);
    assert_eq!(shiv_plus.target, CardTarget::Enemy);
    assert!(shiv_plus.exhaust);
    assert_eq!(
        shiv_plus.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let alchemize = registry.get("Alchemize").expect("Alchemize should exist");
    assert_eq!(alchemize.effect_data, &[E::Simple(SE::ObtainRandomPotion)]);
    assert!(alchemize.complex_hook.is_none());

}

#[test]
fn silent_wave11_shiv_follows_engine_path_for_primary_damage_and_exhaust() {
    let mut engine = engine_with(make_deck(&["Shiv", "Strike_G"]), 40, 0);
    engine.state.hand = make_deck(&["Shiv"]);
    let hp_before = engine.state.enemies[0].entity.hp;

    assert!(play_on_enemy(&mut engine, "Shiv", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 4);
    assert_eq!(exhaust_prefix_count(&engine, "Shiv"), 1);
}
