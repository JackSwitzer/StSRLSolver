#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ConjureBlade.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::tests::support::{enemy_no_intent, engine_without_start, ensure_in_hand, force_player_turn, play_on_enemy, play_self};

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn xcount_wave3_expunger_and_conjure_blade_use_card_owned_xcount_surface() {
    let registry = global_registry();

    let expunger = registry.get("Expunger").expect("Expunger");
    assert_eq!(expunger.card_type, CardType::Attack);
    assert_eq!(expunger.target, CardTarget::Enemy);
    assert_eq!(
        expunger.effect_data,
        &[
            E::ExtraHits(A::CardMisc),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );

    let conjure_blade_plus = registry.get("ConjureBlade+").expect("ConjureBlade+");
    assert_eq!(
        conjure_blade_plus.effect_data,
        &[E::Simple(SE::AddCardWithMisc(
            "Expunger",
            P::Draw,
            A::Fixed(1),
            A::XCostPlus(1),
        ))]
    );
    assert!(conjure_blade_plus.complex_hook.is_none());
}

#[test]
fn xcount_wave3_conjure_blade_stamps_generated_expunger_misc() {
    let mut engine = one_enemy_engine(80, 5);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "ConjureBlade+");

    assert!(play_self(&mut engine, "ConjureBlade+"));

    let expunger = engine
        .state
        .draw_pile
        .iter()
        .rev()
        .find(|card| {
            let name = engine.card_registry.card_name(card.def_id);
            name == "Expunger" || name == "Expunger+"
        })
        .copied()
        .expect("generated Expunger");

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.player.status(crate::status_ids::sid::EXPUNGER_HITS), 0);
    assert_eq!(expunger.misc, 4);
}

#[test]
fn xcount_wave3_expunger_copy_preserves_card_owned_hit_count() {
    let mut engine = one_enemy_engine(120, 5);
    engine.state.energy = 3;
    ensure_in_hand(&mut engine, "ConjureBlade+");
    assert!(play_self(&mut engine, "ConjureBlade+"));

    let generated = engine
        .state
        .draw_pile
        .iter()
        .rev()
        .find(|card| {
            let name = engine.card_registry.card_name(card.def_id);
            name == "Expunger" || name == "Expunger+"
        })
        .copied()
        .expect("generated Expunger");
    assert_eq!(generated.misc, 4);

    engine.state.draw_pile.retain(|card| {
        !(engine.card_registry.card_name(card.def_id) == "Expunger" && card.misc == generated.misc)
    });
    engine.state.hand.push(generated);
    engine.state.hand.push(generated);
    engine.state.energy = 2;

    assert!(play_on_enemy(&mut engine, "Expunger", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 84);

    assert!(play_on_enemy(&mut engine, "Expunger", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 48);
}
