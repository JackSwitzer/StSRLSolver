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
    // SteamBarrier.use grants its current block, then ModifyBlockAction lowers
    // that same UUID's baseBlock by one without a zero floor. Its upgrade adds
    // two to the current baseBlock rather than replacing it with a fresh 8.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyBlockAction.java
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Steam"]);

    let mut played = engine.state.hand[0];
    for expected_block in [6, 5, 4, 3, 2, 1, 0] {
        engine.state.player.block = 0;
        assert!(play_self(&mut engine, "Steam"));
        assert_eq!(engine.state.player.block, expected_block);
        played = engine
            .state
            .discard_pile
            .pop()
            .expect("played Steam Barrier should land in discard");
        engine.state.hand.push(played);
    }

    assert_eq!(played.decrementing_misc_or(6), -1);
    engine.card_registry.upgrade_card(&mut engine.state.hand[0]);
    assert_eq!(engine.card_registry.card_name(engine.state.hand[0].def_id), "Steam+");
    engine.state.player.block = 0;
    assert!(play_self(&mut engine, "Steam+"));
    assert_eq!(engine.state.player.block, 1);
}

#[test]
fn streamline_updates_the_played_instance_cost_and_future_plays_can_reuse_the_cheaper_copy() {
    // Streamline queues 15 damage, then one UUID-targeted cost reduction; its
    // upgrade changes only damage. A same-instance Echo Form replay queues the
    // reduction again, while DamageAction clears it after a combat-ending hit.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ReduceCostAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    let mut engine = single_enemy_engine();
    engine.state.hand = make_deck(&["Streamline"]);

    assert!(play_on_enemy(&mut engine, "Streamline", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 25);

    let played = engine
        .state
        .discard_pile
        .pop()
        .expect("played Streamline should land in discard");
    assert_eq!((played.cost, played.base_cost), (1, 1));

    let mut played = played;
    engine.card_registry.upgrade_card(&mut played);
    assert_eq!(engine.card_registry.card_name(played.def_id), "Streamline+");
    played.reset_cost_for_turn();
    assert_eq!((played.cost, played.base_cost), (1, 1));

    engine.state.hand.clear();
    engine.state.hand.push(played);
    engine.state.energy = 1;

    assert!(play_on_enemy(&mut engine, "Streamline+", 0));
    assert_eq!(engine.state.enemies[0].entity.hp, 5);
    let played = engine.state.discard_pile[0];
    assert_eq!((played.cost, played.base_cost), (0, 0));

    let mut echoed = single_enemy_engine();
    echoed.state.player.set_status(crate::status_ids::sid::ECHO_FORM, 1);
    echoed.rebuild_effect_runtime();
    echoed.state.hand = make_deck(&["Streamline"]);

    assert!(play_on_enemy(&mut echoed, "Streamline", 0));
    assert_eq!(echoed.state.enemies[0].entity.hp, 10);
    let replayed = echoed.state.discard_pile[0];
    assert_eq!((replayed.cost, replayed.base_cost), (0, 0));

    let mut lethal = single_enemy_engine();
    lethal.state.enemies[0].entity.hp = 15;
    let lethal_card = lethal.card_registry.make_card("Streamline");
    lethal.runtime_played_card = Some(lethal_card);
    let lethal_def = lethal.card_registry.get("Streamline").unwrap().clone();
    lethal.execute_card_effects_with_enemy_on_use(&lethal_def, lethal_card, 0);

    assert_eq!(lethal.state.enemies[0].entity.hp, 0);
    let unmodified = lethal.runtime_played_card.expect("active lethal Streamline");
    assert_eq!((unmodified.cost, unmodified.base_cost), (-1, 2));
}
