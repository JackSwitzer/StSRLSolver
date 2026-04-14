#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyBlockAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ReduceCostAction.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self};

fn single_enemy_engine() -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn instance_mutation_wave1_registry_exports_steam_and_streamline_on_typed_played_instance_mutation_surfaces() {
    let steam = global_registry().get("Steam").expect("Steam");
    assert_eq!(
        steam.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
        ]
    );
    assert!(steam.complex_hook.is_none());

    let steam_plus = global_registry().get("Steam+").expect("Steam+");
    assert_eq!(
        steam_plus.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
        ]
    );
    assert!(steam_plus.complex_hook.is_none());

    let streamline = global_registry().get("Streamline").expect("Streamline");
    assert_eq!(
        streamline.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
        ]
    );
    assert!(streamline.complex_hook.is_none());

    let streamline_plus = global_registry().get("Streamline+").expect("Streamline+");
    assert_eq!(
        streamline_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
        ]
    );
    assert!(streamline_plus.complex_hook.is_none());
}

#[test]
fn steam_barrier_updates_the_played_instance_block_and_future_plays_see_the_reduced_block() {
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Steam", "Steam"]);

    assert!(play_self(&mut engine, "Steam"));
    assert_eq!(engine.state.player.block, 6);

    let played = engine
        .state
        .discard_pile
        .pop()
        .expect("played Steam Barrier should land in discard");
    assert_eq!(played.misc, 5);

    engine.state.hand.clear();
    engine.state.hand.push(played);

    assert!(play_self(&mut engine, "Steam"));
    assert_eq!(engine.state.player.block, 11);
}

#[test]
fn streamline_updates_the_played_instance_cost_and_future_plays_can_reuse_the_cheaper_copy() {
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Streamline"]);

    assert!(play_on_enemy(&mut engine, "Streamline", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);

    let played = engine
        .state
        .discard_pile
        .pop()
        .expect("played Streamline should land in discard");
    assert_eq!(played.cost, 1);

    engine.state.hand.clear();
    engine.state.hand.push(played);
    engine.state.energy = 1;

    assert!(play_on_enemy(&mut engine, "Streamline", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 10);
}
