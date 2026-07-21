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
fn defect_wave12_chaos_uses_java_orb_order_and_card_random_rng() {
    // Chaos.java calls AbstractOrb.getRandomOrb(true) once, or twice upgraded.
    // AbstractOrb.java builds [Dark, Frost, Lightning, Plasma] and selects with
    // cardRandomRng.random(3), so each orb consumes exactly one cardRandom tick.
    let mut engine = one_enemy_engine(60, 3);
    engine.init_defect_orbs(3);
    engine.state.hand = make_deck(&["Chaos", "Chaos+"]);

    let java_orbs = [OrbType::Dark, OrbType::Frost, OrbType::Lightning, OrbType::Plasma];
    let mut oracle = engine.card_random_rng.clone();
    let expected: Vec<OrbType> = (0..3)
        .map(|_| java_orbs[oracle.random_int(3) as usize])
        .collect();
    let generic_before = engine.rng_counters()["shuffle"];

    assert!(play_self(&mut engine, "Chaos"));
    assert!(play_self(&mut engine, "Chaos+"));

    let actual: Vec<OrbType> = engine.state.orb_slots.slots[0..3]
        .iter()
        .map(|orb| orb.orb_type)
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(engine.card_random_rng.counter, oracle.counter);
    assert_eq!(engine.rng_counters()["shuffle"], generic_before);
}

#[test]
fn defect_wave12_barrage_zero_orb_count_deals_no_damage() {
    let mut barrage = one_enemy_engine(60, 3);
    barrage.state.hand = make_deck(&["Barrage"]);

    assert!(play_on_enemy(&mut barrage, "Barrage", 0));
    assert_eq!(barrage.state.enemies[0].entity.hp, 60);
}

#[test]
fn barrage_hit_count_is_exactly_the_number_of_non_empty_orbs() {
    // Sources: Barrage.java sets damage 4 and upgradeDamage(2);
    // BarrageAction.java queues one DamageAction for each non-EmptyOrbSlot.
    for (card_id, per_hit) in [("Barrage", 4), ("Barrage+", 6)] {
        for orb_count in 0..=3 {
            let mut engine = one_enemy_engine(60, 3);
            engine.init_defect_orbs(3);
            for _ in 0..orb_count {
                engine.channel_orb(OrbType::Frost);
            }
            engine.state.hand = make_deck(&[card_id]);

            assert!(play_on_enemy(&mut engine, card_id, 0));

            assert_eq!(engine.state.enemies[0].entity.hp, 60 - per_hit * orb_count);
            assert_eq!(engine.state.energy, 2);
        }
    }
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
fn rip_and_tear_plus_consumes_one_card_random_tick_per_hit_with_one_target() {
    // RipAndTear.java queues two NewRipAndTearActions. Each superclass update
    // calls MonsterGroup.getRandomMonster through cardRandomRng, whose
    // random(0, 0) still consumes a counter tick. The card's ALL_ENEMY target
    // requires no selected target, and upgradeDamage(2) makes 9 + 2 Strength
    // damage per hit.
    // Sources: cards/blue/RipAndTear.java,
    // actions/defect/NewRipAndTearAction.java,
    // actions/common/AttackDamageRandomEnemyAction.java, and
    // monsters/MonsterGroup.java.
    let mut engine = one_enemy_engine(50, 3);
    engine.state.hand = make_deck(&["Rip and Tear+"]);
    engine.state.player.set_status(sid::STRENGTH, 2);
    let card_random_before = engine.rng_counters()["cardRandom"];

    assert!(play_self(&mut engine, "Rip and Tear+"));

    assert_eq!(engine.state.enemies[0].entity.hp, 28);
    assert_eq!(engine.rng_counters()["cardRandom"], card_random_before + 2);
    assert_eq!(engine.state.energy, 2);
}
#[test]
fn defect_wave12_thunder_strike_deals_no_damage_with_zero_lightning() {
    let mut thunder = one_enemy_engine(60, 3);
    thunder.state.hand = make_deck(&["Thunder Strike"]);

    assert!(play_on_enemy(&mut thunder, "Thunder Strike", 0));
    assert_eq!(thunder.state.enemies[0].entity.hp, 60);
}

#[test]
fn thunder_strike_plus_uses_strike_tag_and_card_random_once_per_lightning() {
    // ThunderStrike.java counts Lightning instances channeled this combat,
    // queues one NewThunderStrikeAction per instance, and carries STRIKE.
    // AttackDamageRandomEnemyAction selects through cardRandomRng even when
    // only one enemy is alive; StrikeDummy adds three to every tagged hit.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ThunderStrike.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewThunderStrikeAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/AttackDamageRandomEnemyAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/StrikeDummy.java
    let mut engine = one_enemy_engine(100, 3);
    engine.init_defect_orbs(3);
    engine.channel_orb(OrbType::Lightning);
    engine.channel_orb(OrbType::Lightning);
    engine.state.relics.push("StrikeDummy".to_string());
    engine.state.player.set_status(sid::STRENGTH, 2);
    engine.state.hand = make_deck(&["Thunder Strike+"]);
    let card_random_before = engine.rng_counters()["cardRandom"];
    let generic_before = engine.rng_counters()["shuffle"];

    assert!(play_self(&mut engine, "Thunder Strike+"));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 72);
    assert_eq!(engine.rng_counters()["cardRandom"], card_random_before + 2);
    assert_eq!(engine.rng_counters()["shuffle"], generic_before);
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
