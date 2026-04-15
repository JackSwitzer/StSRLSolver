#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Barrage.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/BarrageAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/RipAndTear.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewRipAndTearAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ThunderStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewThunderStrikeAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Chaos.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/DoubleEnergy.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/DoubleEnergyAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::engine::CombatEngine;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self};

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

    let chaos = reg.get("Chaos").expect("Chaos");
    assert_eq!(
        chaos.effect_data,
        &[E::Simple(SE::ChannelRandomOrb(A::Magic))]
    );
    assert!(chaos.complex_hook.is_none());
    assert_eq!(chaos.card_type, CardType::Skill);
    assert_eq!(chaos.target, CardTarget::SelfTarget);

    let double_energy = reg.get("Double Energy").expect("Double Energy");
    assert_eq!(double_energy.effect_data, &[E::Simple(SE::DoubleEnergy)]);
    assert!(double_energy.complex_hook.is_none());
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
fn defect_wave12_chaos_channels_random_orbs_deterministically_for_identical_seeds() {
    let mut left = one_enemy_engine(60, 3);
    left.init_defect_orbs(3);
    left.state.hand = make_deck(&["Chaos", "Chaos+"]);

    let mut right = one_enemy_engine(60, 3);
    right.init_defect_orbs(3);
    right.state.hand = make_deck(&["Chaos", "Chaos+"]);

    assert!(play_self(&mut left, "Chaos"));
    assert!(play_self(&mut right, "Chaos"));
    assert_eq!(left.state.orb_slots.occupied_count(), 1);
    assert_eq!(right.state.orb_slots.occupied_count(), 1);
    assert_eq!(left.state.orb_slots.front_orb_type(), right.state.orb_slots.front_orb_type());

    assert!(play_self(&mut left, "Chaos+"));
    assert!(play_self(&mut right, "Chaos+"));
    assert_eq!(left.state.orb_slots.occupied_count(), 3);
    assert_eq!(right.state.orb_slots.occupied_count(), 3);
    assert_eq!(left.state.orb_slots.front_orb_type(), right.state.orb_slots.front_orb_type());
}

#[test]
fn defect_wave12_barrage_zero_orb_count_deals_no_damage() {
    let mut barrage = one_enemy_engine(60, 3);
    barrage.state.hand = make_deck(&["Barrage"]);

    assert!(play_on_enemy(&mut barrage, "Barrage", 0));
    assert_eq!(barrage.state.enemies[0].entity.hp, 60);
}
#[test]
fn defect_wave12_rip_and_tear_chooses_a_fresh_random_target_for_each_hit() {
    let seed = (0u64..128)
        .find(|seed| {
            let state = crate::tests::support::combat_state_with(
                make_deck(&["Rip and Tear"]),
                vec![
                    enemy_no_intent("JawWorm", 30, 30),
                    enemy_no_intent("Cultist", 30, 30),
                ],
                3,
            );
            let mut engine = CombatEngine::new(state, *seed);
            force_player_turn(&mut engine);
            engine.state.hand = make_deck(&["Rip and Tear"]);
            if !play_on_enemy(&mut engine, "Rip and Tear", 0) {
                return false;
            }
            let hp0 = engine.state.enemies[0].entity.hp;
            let hp1 = engine.state.enemies[1].entity.hp;
            hp0 < 30 && hp1 < 30 && hp0 + hp1 == 46
        })
        .expect("expected a seed that splits Rip and Tear across both enemies");

    let state = crate::tests::support::combat_state_with(
        make_deck(&["Rip and Tear"]),
        vec![
            enemy_no_intent("JawWorm", 30, 30),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    let mut engine = CombatEngine::new(state, seed);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Rip and Tear"]);
    assert!(play_on_enemy(&mut engine, "Rip and Tear", 0));
    assert_eq!(engine.state.enemies[0].entity.hp + engine.state.enemies[1].entity.hp, 46);
    assert!(engine.state.enemies[0].entity.hp < 30);
    assert!(engine.state.enemies[1].entity.hp < 30);
}
#[test]
fn defect_wave12_thunder_strike_deals_no_damage_with_zero_lightning() {
    let mut thunder = one_enemy_engine(60, 3);
    thunder.state.hand = make_deck(&["Thunder Strike"]);

    assert!(play_on_enemy(&mut thunder, "Thunder Strike", 0));
    assert_eq!(thunder.state.enemies[0].entity.hp, 60);
}

#[test]
fn defect_wave12_thunder_strike_chooses_a_fresh_random_target_for_each_lightning_hit() {
    let seed = (0u64..128)
        .find(|seed| {
            let mut state = crate::tests::support::combat_state_with(
                make_deck(&["Thunder Strike"]),
                vec![
                    enemy_no_intent("JawWorm", 30, 30),
                    enemy_no_intent("Cultist", 30, 30),
                ],
                3,
            );
            state.player.set_status(sid::LIGHTNING_CHANNELED, 3);
            let mut engine = CombatEngine::new(state, *seed);
            force_player_turn(&mut engine);
            engine.state.hand = make_deck(&["Thunder Strike"]);
            if !play_on_enemy(&mut engine, "Thunder Strike", 0) {
                return false;
            }
            let hp0 = engine.state.enemies[0].entity.hp;
            let hp1 = engine.state.enemies[1].entity.hp;
            hp0 < 30 && hp1 < 30 && hp0 + hp1 == 39
        })
        .expect("expected a seed that splits Thunder Strike across both enemies");

    let mut state = crate::tests::support::combat_state_with(
        make_deck(&["Thunder Strike"]),
        vec![
            enemy_no_intent("JawWorm", 30, 30),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    state.player.set_status(sid::LIGHTNING_CHANNELED, 3);
    let mut engine = CombatEngine::new(state, seed);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Thunder Strike"]);
    assert!(play_on_enemy(&mut engine, "Thunder Strike", 0));
    assert_eq!(engine.state.enemies[0].entity.hp + engine.state.enemies[1].entity.hp, 39);
    assert!(engine.state.enemies[0].entity.hp < 30);
    assert!(engine.state.enemies[1].entity.hp < 30);
}
