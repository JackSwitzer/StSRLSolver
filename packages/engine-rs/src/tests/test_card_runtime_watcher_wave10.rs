#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Protect.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Study.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LikeWater.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MentalFortress.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MasterReality.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Smite.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Safety.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/ThroughViolence.java

use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave10_registry_exports_typed_surface_for_live_cards() {
    let registry = global_registry();

    let protect = registry.get("Protect").expect("Protect should be registered");
    assert_eq!(protect.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let study = registry.get("Study").expect("Study should be registered");
    assert_eq!(study.effect_data, &[E::Simple(SE::AddStatus(T::Player, sid::STUDY, A::Magic))]);

    let like_water = registry.get("LikeWater").expect("LikeWater should be registered");
    assert_eq!(
        like_water.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::LIKE_WATER, A::Magic))]
    );

    let mental_fortress = registry.get("MentalFortress").expect("MentalFortress should be registered");
    assert_eq!(
        mental_fortress.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::MENTAL_FORTRESS, A::Magic))]
    );

    let master_reality = registry.get("MasterReality").expect("MasterReality should be registered");
    assert_eq!(
        master_reality.effect_data,
        &[E::Simple(SE::SetStatus(T::Player, sid::MASTER_REALITY, A::Fixed(1)))]
    );

    let smite = registry.get("Smite").expect("Smite should be registered");
    assert_eq!(smite.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);
}

#[test]
fn watcher_wave10_protect_smite_and_power_installs_follow_engine_path() {
    let mut protect = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut protect, "Protect+");
    assert!(play_self(&mut protect, "Protect+"));
    assert_eq!(protect.state.player.block, 16);

    let mut smite = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut smite, "Smite+");
    assert!(play_on_enemy(&mut smite, "Smite+", 0));
    assert_eq!(smite.state.enemies[0].entity.hp, 44);

    let mut study = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut study, "Study");
    assert!(play_self(&mut study, "Study"));
    assert_eq!(study.state.player.status(sid::STUDY), 1);
    end_turn(&mut study);
    let total_insights = hand_prefix_count(&study, "Insight")
        + draw_prefix_count(&study, "Insight")
        + discard_prefix_count(&study, "Insight");
    assert_eq!(total_insights, 1);

    let mut like_water = one_enemy_engine("JawWorm", 60, 10);
    ensure_in_hand(&mut like_water, "LikeWater+");
    ensure_in_hand(&mut like_water, "Vigilance");
    assert!(play_self(&mut like_water, "LikeWater+"));
    assert_eq!(like_water.state.player.status(sid::LIKE_WATER), 7);
    assert!(play_self(&mut like_water, "Vigilance"));
    end_turn(&mut like_water);
    assert_eq!(like_water.state.player.hp, 80);
}

#[test]
fn watcher_wave10_mental_fortress_master_reality_and_tranquility_behave_like_java() {
    let mut mental_fortress = one_enemy_engine("JawWorm", 80, 0);
    set_stance(&mut mental_fortress, Stance::Wrath);
    ensure_in_hand(&mut mental_fortress, "MentalFortress+");
    ensure_in_hand(&mut mental_fortress, "EmptyBody");
    assert!(play_self(&mut mental_fortress, "MentalFortress+"));
    assert_eq!(mental_fortress.state.player.status(sid::MENTAL_FORTRESS), 6);
    let block_before = mental_fortress.state.player.block;
    assert!(play_self(&mut mental_fortress, "EmptyBody"));
    assert_eq!(mental_fortress.state.stance, Stance::Neutral);
    assert_eq!(mental_fortress.state.player.block, block_before + 13);

    let mut master_reality = one_enemy_engine("JawWorm", 80, 0);
    ensure_in_hand(&mut master_reality, "MasterReality");
    ensure_in_hand(&mut master_reality, "CarveReality");
    assert!(play_self(&mut master_reality, "MasterReality"));
    assert_eq!(master_reality.state.player.status(sid::MASTER_REALITY), 1);
    assert!(play_on_enemy(&mut master_reality, "CarveReality", 0));
    assert_eq!(hand_count(&master_reality, "Smite+"), 1);

    let mut tranquility = one_enemy_engine("JawWorm", 80, 0);
    set_stance(&mut tranquility, Stance::Wrath);
    ensure_in_hand(&mut tranquility, "ClearTheMind+");
    assert!(play_self(&mut tranquility, "ClearTheMind+"));
    assert_eq!(tranquility.state.stance, Stance::Calm);
    assert!(tranquility
        .state
        .exhaust_pile
        .iter()
        .any(|card| tranquility.card_registry.card_name(card.def_id) == "ClearTheMind+"));
}

#[test]
fn watcher_wave10_temp_card_bundle_is_still_owned_by_temp_registry() {
    let registry = global_registry();

    let safety = registry.get("Safety").expect("Safety should be registered");
    assert_eq!(safety.effect_data, &[E::Simple(SE::GainBlock(A::Block))]);

    let through_violence = registry.get("ThroughViolence").expect("ThroughViolence should be registered");
    assert_eq!(
        through_violence.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );
}
