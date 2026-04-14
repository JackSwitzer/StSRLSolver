#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Barrage.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/BarrageAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/RipAndTear.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewRipAndTearAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ThunderStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewThunderStrikeAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DoubleEnergyAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy};

fn one_enemy_engine(hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", hp, hp)], energy);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn defect_wave12_registry_exports_surface_barrage_rip_and_tear_and_thunder_strike() {
    let reg = global_registry();

    let barrage = reg.get("Barrage").expect("Barrage");
    assert_eq!(
        barrage.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ExtraHits(A::OrbCount),
        ]
    );
    assert!(barrage.complex_hook.is_none());
    assert_eq!(barrage.card_type, CardType::Attack);
    assert_eq!(barrage.target, CardTarget::Enemy);

    let rip = reg.get("Rip and Tear").expect("Rip and Tear");
    assert_eq!(
        rip.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::ExtraHits(A::Magic),
        ]
    );
    assert!(rip.complex_hook.is_none());
    assert_eq!(rip.card_type, CardType::Attack);
    assert_eq!(rip.target, CardTarget::AllEnemy);

    let thunder = reg.get("Thunder Strike").expect("Thunder Strike");
    assert_eq!(
        thunder.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
        ]
    );
    assert!(thunder.complex_hook.is_none());
    assert_eq!(thunder.card_type, CardType::Attack);
    assert_eq!(thunder.target, CardTarget::AllEnemy);

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());
}

#[test]
fn defect_wave12_barrage_rip_and_tear_and_thunder_strike_follow_typed_primary_paths() {
    let mut barrage = one_enemy_engine(60, 3);
    barrage.init_defect_orbs(3);
    barrage.channel_orb(OrbType::Lightning);
    barrage.channel_orb(OrbType::Frost);
    barrage.channel_orb(OrbType::Dark);
    barrage.state.hand = make_deck(&["Barrage"]);

    assert!(play_on_enemy(&mut barrage, "Barrage", 0));
    assert_eq!(barrage.state.enemies[0].entity.hp, 48);

    let mut rip = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 30, 30),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    force_player_turn(&mut rip);
    rip.state.hand = make_deck(&["Rip and Tear"]);

    let total_before = rip
        .state
        .enemies
        .iter()
        .map(|enemy| enemy.entity.hp.max(0))
        .sum::<i32>();
    assert!(play_on_enemy(&mut rip, "Rip and Tear", 0));
    let total_after = rip
        .state
        .enemies
        .iter()
        .map(|enemy| enemy.entity.hp.max(0))
        .sum::<i32>();
    assert_eq!(total_before - total_after, 14);

    let mut thunder = one_enemy_engine(60, 3);
    thunder.init_defect_orbs(3);
    thunder.channel_orb(OrbType::Lightning);
    thunder.channel_orb(OrbType::Lightning);
    thunder.channel_orb(OrbType::Lightning);
    thunder.state.hand = make_deck(&["Thunder Strike"]);

    assert!(play_on_enemy(&mut thunder, "Thunder Strike", 0));
    assert_eq!(thunder.state.enemies[0].entity.hp, 39);
}

#[test]
#[ignore = "Blocked on zero-orb Barrage parity; Java BarrageAction deals no damage when the orb list is empty"]
fn defect_wave12_barrage_zero_orb_count_still_needs_exact_no_damage_support() {
    let barrage = global_registry().get("Barrage").expect("Barrage");
    assert_eq!(
        barrage.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::ExtraHits(A::OrbCount),
        ]
    );
}

#[test]
#[ignore = "Blocked on a fresh random target per hit; Java NewRipAndTearAction selects a new enemy each time"]
fn defect_wave12_rip_and_tear_still_needs_per_hit_random_target_selection() {
    let rip = global_registry().get("Rip and Tear").expect("Rip and Tear");
    assert_eq!(
        rip.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::ExtraHits(A::Magic),
        ]
    );
}

#[test]
#[ignore = "Blocked on a fresh random target per hit and zero-lightning no-op parity; Java NewThunderStrikeAction chooses a new enemy for every lightning hit"]
fn defect_wave12_thunder_strike_still_needs_per_hit_random_target_selection() {
    let thunder = global_registry().get("Thunder Strike").expect("Thunder Strike");
    assert_eq!(
        thunder.effect_data,
        &[
            E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
            E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
        ]
    );
}

#[test]
#[ignore = "Blocked on a typed energy-doubling primitive; Java DoubleEnergyAction doubles the current energy directly"]
fn defect_wave12_double_energy_still_needs_typed_energy_doubling() {
    let double_energy = global_registry().get("Double Energy").expect("Double Energy");
    assert!(double_energy.effect_data.is_empty());
    assert!(double_energy.complex_hook.is_some());
}
