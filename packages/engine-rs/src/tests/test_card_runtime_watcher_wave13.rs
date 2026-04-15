#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Expunger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SandsOfTime.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WindmillStrike.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::state::Stance;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave13_registry_exports_match_typed_surface() {
    let registry = global_registry();

    let brilliance = registry.get("Brilliance").expect("Brilliance should be registered");
    assert_eq!(
        brilliance.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
    assert!(brilliance.effects.contains(&"damage_plus_mantra"));

    let expunger = registry.get("Expunger").expect("Expunger should be registered");
    assert_eq!(
        expunger.effect_data,
        &[
            E::ExtraHits(A::CardMisc),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );
    assert!(expunger.effects.is_empty());

    let perseverance = registry.get("Perseverance").expect("Perseverance should be registered");
    assert_eq!(perseverance.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let sands_of_time = registry.get("SandsOfTime").expect("Sands of Time should be registered");
    assert_eq!(
        sands_of_time.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let tranquility = registry.get("ClearTheMind").expect("Tranquility should be registered");
    assert_eq!(
        tranquility.effect_data,
        &[E::Simple(SE::ChangeStance(Stance::Calm))]
    );

    let windmill = registry.get("WindmillStrike").expect("Windmill Strike should be registered");
    assert_eq!(
        windmill.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

}

#[test]
fn watcher_wave13_typed_surface_cards_follow_engine_path() {
    let mut brilliance = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut brilliance, "Pray");
    ensure_in_hand(&mut brilliance, "Brilliance");
    play_self(&mut brilliance, "Pray");
    play_on_enemy(&mut brilliance, "Brilliance", 0);
    assert_eq!(brilliance.state.enemies[0].entity.hp, 45);

    let mut expunger = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut expunger, "Expunger");
    if let Some(expunger_idx) = expunger
        .state
        .hand
        .iter()
        .position(|card| expunger.card_registry.card_name(card.def_id) == "Expunger")
    {
        expunger.state.hand[expunger_idx].misc = 3;
    }
    play_on_enemy(&mut expunger, "Expunger", 0);
    assert_eq!(expunger.state.enemies[0].entity.hp, 33);

    let mut perseverance = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut perseverance, "Perseverance");
    end_turn(&mut perseverance);
    assert_eq!(perseverance.state.player.status(sid::PERSEVERANCE_BONUS), 2);

    let mut sands = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut sands, "SandsOfTime");
    end_turn(&mut sands);
    let sands_card = sands
        .state
        .hand
        .iter()
        .find(|card| sands.card_registry.card_name(card.def_id) == "SandsOfTime")
        .expect("Sands of Time should stay in hand");
    assert_eq!(sands_card.cost, 3);

    let mut tranquility = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut tranquility, "ClearTheMind");
    assert!(play_self(&mut tranquility, "ClearTheMind"));
    assert_eq!(tranquility.state.stance, Stance::Calm);
    assert_eq!(exhaust_prefix_count(&tranquility, "ClearTheMind"), 1);

    let mut windmill = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut windmill, "WindmillStrike");
    end_turn(&mut windmill);
    assert_eq!(windmill.state.player.status(sid::WINDMILL_STRIKE_BONUS), 4);
}
