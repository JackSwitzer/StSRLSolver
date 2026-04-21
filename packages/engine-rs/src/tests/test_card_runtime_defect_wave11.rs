#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Blizzard.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/FTL.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/FTLAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Melter.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Streamline.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ReduceCostAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/RemoveAllBlockAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Condition as Cond, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self};

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave11_registry_exports_promote_ftl_steam_and_streamline_to_typed_primary_effects() {
    let reg = global_registry();

    let ftl = reg.get("FTL").expect("FTL");
    assert_eq!(
        ftl.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::CardsPlayedThisTurnLessThan(3),
                &[E::Simple(SE::DrawCards(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(ftl.complex_hook.is_none());
    assert_eq!(ftl.card_type, CardType::Attack);
    assert_eq!(ftl.target, CardTarget::Enemy);

    let steam = reg.get("Steam").expect("Steam");
    assert_eq!(
        steam.effect_data,
        &[
            E::Simple(SE::GainBlock(A::Block)),
            E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
        ]
    );
    assert!(steam.complex_hook.is_none(), "Steam Barrier is now on the played-instance block decrement surface");
    assert_eq!(steam.card_type, CardType::Skill);
    assert_eq!(steam.target, CardTarget::SelfTarget);

    let streamline = reg.get("Streamline").expect("Streamline");
    assert_eq!(
        streamline.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
        ]
    );
    assert!(streamline.complex_hook.is_none(), "Streamline is now on the played-card cost mutation surface");

    let blizzard = reg.get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());

    let genetic = reg
        .get("Genetic Algorithm")
        .expect("Genetic Algorithm");
    assert_eq!(
        genetic.effect_data,
        &[
            E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            E::Simple(SE::GainBlock(A::Block)),
        ]
    );
    assert!(genetic.complex_hook.is_none());
}

#[test]
fn defect_wave11_ftl_draws_under_threshold_and_stops_at_threshold() {
    let mut draws = one_enemy_engine(40, 3);
    draws.state.hand = make_deck(&["FTL+"]);
    draws.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Dualcast"]);

    assert!(play_on_enemy(&mut draws, "FTL+", 0));
    assert_eq!(draws.state.hand.len(), 4);
    assert_eq!(draws.state.enemies[0].entity.hp, 34);

    let mut gated = one_enemy_engine(40, 3);
    gated.state.cards_played_this_turn = 3;
    gated.state.hand = make_deck(&["FTL"]);
    gated.state.draw_pile = make_deck(&["Strike", "Defend"]);

    assert!(play_on_enemy(&mut gated, "FTL", 0));
    assert_eq!(gated.state.hand.len(), 0);
    assert_eq!(gated.state.enemies[0].entity.hp, 35);
}

#[test]
fn defect_wave11_steam_and_streamline_follow_typed_primary_effects_and_keep_instance_state_local() {
    let mut steam = one_enemy_engine(40, 3);
    steam.state.hand = vec![
        steam.card_registry.make_card("Steam"),
        steam.card_registry.make_card("Steam"),
    ];
    steam.state.draw_pile = vec![steam.card_registry.make_card("Steam")];
    steam.state.discard_pile = vec![steam.card_registry.make_card("Steam")];

    assert!(play_self(&mut steam, "Steam"));
    assert_eq!(steam.state.player.block, 6);
    let played_steam = steam
        .state
        .discard_pile
        .last()
        .copied()
        .expect("played Steam Barrier should land in discard");
    assert_eq!(played_steam.misc, 5);
    assert_eq!(steam.state.draw_pile[0].misc, -1);
    assert_eq!(steam.state.discard_pile[0].misc, -1);

    let mut streamline = one_enemy_engine(40, 3);
    streamline.state.hand = vec![streamline.card_registry.make_card("Streamline")];
    streamline.state.draw_pile = vec![streamline.card_registry.make_card("Streamline")];
    streamline.state.discard_pile = vec![streamline.card_registry.make_card("Streamline")];

    assert!(play_on_enemy(&mut streamline, "Streamline", 0));
    assert_eq!(streamline.state.enemies[0].entity.hp, 25);
    let played_streamline = streamline
        .state
        .discard_pile
        .last()
        .copied()
        .expect("played Streamline should land in discard");
    assert_eq!(played_streamline.cost, 1);
    assert_eq!(streamline.state.draw_pile[0].cost, -1);
    assert_eq!(streamline.state.discard_pile[0].cost, -1);
}

#[test]
fn defect_wave11_blizzard_uses_the_typed_frost_count_damage_scaling_surface() {
    let blizzard = global_registry().get("Blizzard").expect("Blizzard");
    assert_eq!(
        blizzard.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::StatusValueTimesMagic(sid::FROST_CHANNELED)))]
    );
    assert!(blizzard.complex_hook.is_none());
}
