#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyDamageAction.java

use crate::cards::{global_registry, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy};

fn single_enemy_engine() -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.energy = 3;
    engine
}

#[test]
fn scaling_wave1_registry_exports_typed_played_instance_damage_mutation_surface() {
    let reg = global_registry();

    let rampage = reg.get("Rampage").expect("Rampage");
    assert_eq!(
        rampage.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
        ]
    );
    assert!(rampage.complex_hook.is_none());
    assert_eq!(rampage.card_type, CardType::Attack);

    let rampage_plus = reg.get("Rampage+").expect("Rampage+");
    assert_eq!(
        rampage_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
        ]
    );
    assert!(rampage_plus.complex_hook.is_none());

    let glass_knife = reg.get("Glass Knife").expect("Glass Knife");
    assert_eq!(
        glass_knife.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
        ]
    );
    assert!(glass_knife.complex_hook.is_none());

    let glass_knife_plus = reg.get("Glass Knife+").expect("Glass Knife+");
    assert_eq!(
        glass_knife_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
        ]
    );
    assert!(glass_knife_plus.complex_hook.is_none());
}

#[test]
fn rampage_and_glass_knife_update_the_played_instance_damage_seed_for_future_plays() {
    let mut rampage = single_enemy_engine();
    rampage.state.hand = make_deck(&["Rampage"]);

    assert!(play_on_enemy(&mut rampage, "Rampage", 0));
    assert_eq!(rampage.state.enemies[0].entity.hp, 32);

    let played = rampage
        .state
        .discard_pile
        .pop()
        .expect("played Rampage should land in discard");
    assert_eq!(played.misc, 13);

    rampage.state.hand.clear();
    rampage.state.hand.push(played);
    rampage.state.energy = 1;

    assert!(play_on_enemy(&mut rampage, "Rampage", 0));
    assert_eq!(rampage.state.enemies[0].entity.hp, 19);

    let mut glass_knife = single_enemy_engine();
    glass_knife.state.hand = make_deck(&["Glass Knife"]);

    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, 24);

    let played = glass_knife
        .state
        .discard_pile
        .pop()
        .expect("played Glass Knife should land in discard");
    assert_eq!(played.misc, 6);

    glass_knife.state.hand.clear();
    glass_knife.state.hand.push(played);
    glass_knife.state.energy = 1;

    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, 12);
}
