#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Alchemize.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Reaper.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EscapePlan.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java

use crate::cards::global_registry;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::tests::support::{
    engine_without_start, enemy_no_intent, force_player_turn, make_deck, play_on_enemy, play_self,
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
#[ignore = "Enlightenment base still needs a turn-only cost-reduction lifetime primitive; Java updates costForTurn for the turn and only permanently reduces upgraded cards. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java"]
fn shared_primitive_wave1_enlightenment_base_stays_explicitly_blocked() {}

#[test]
#[ignore = "Omniscience still needs a draw-pile card selection plus play-twice primitive; Java selects a card from the draw pile and queues it multiple times via OmniscienceAction. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java"]
fn shared_primitive_wave1_omniscience_stays_explicitly_blocked() {}
