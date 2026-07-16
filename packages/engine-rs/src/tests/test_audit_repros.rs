#![cfg(test)]

//! Intentionally failing engine-path reproducers discovered by the deep audit.
//!
//! These remain ignored until their registered `EDA-*` work units are fixed.

use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, force_player_turn,
    make_deck, play_on_enemy,
};

#[test]
fn eda_001_all_victory_relics_must_run_after_combat_is_marked_over() {
    // AbstractPlayer.onVictory iterates every relic without an early exit:
    // decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java:1949-1960
    let mut state = combat_state_with(Vec::new(), vec![enemy_no_intent("JawWorm", 1, 1)], 3);
    state.player.hp = 60;
    state.relics = vec!["Burning Blood".to_string(), "FaceOfCleric".to_string()];
    let mut engine = engine_with_state(state);
    engine.state.enemies[0].entity.hp = 0;

    engine.finalize_enemy_death(0);
    engine.check_combat_end();

    assert_eq!(engine.state.player.max_hp, 81);
    assert_eq!(engine.state.player.hp, 67);
}

#[test]
fn eda_002_rampage_growth_must_preserve_java_int_range() {
    // ModifyDamageAction increments AbstractCard.baseDamage as an int:
    // decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyDamageAction.java:21-29
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 100_000, 100_000)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Rampage"]);
    engine.state.hand[0].misc = 32_765;

    assert!(play_on_enemy(&mut engine, "Rampage", 0));

    let played = engine
        .state
        .discard_pile
        .last()
        .expect("played Rampage should land in discard");
    assert_eq!(played.misc, 32_770);
}

#[test]
fn eda_002_card_misc_round_trips_the_training_snapshot_at_java_int_width() {
    // AbstractCard.misc and CardSave.misc are Java ints. The structured
    // training snapshot must preserve the same width when a combat is cloned.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardSave.java
    let mut engine = engine_without_start(
        make_deck(&["Rampage"]),
        vec![enemy_no_intent("JawWorm", 100_000, 100_000)],
        3,
    );
    engine.state.draw_pile[0].misc = 100_000;

    let snapshot = crate::training_contract::combat_snapshot_from_combat(&engine);
    let restored = crate::training_contract::combat_engine_from_snapshot(&snapshot);

    assert_eq!(snapshot.draw_pile[0].misc, 100_000);
    assert_eq!(restored.state.draw_pile[0].misc, 100_000);
}

#[test]
#[ignore = "EDA-003: EntityState status storage truncates Java Poison amounts to i16"]
fn eda_003_catalyst_poison_must_preserve_java_int_range() {
    // TriplePoisonAction applies twice the current amount to the existing
    // PoisonPower; AbstractPower.stackPower uses Java int arithmetic. The
    // PoisonPower constructor cap is not re-applied while stacking:
    // decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TriplePoisonAction.java:20-24
    // decompiled/java-src/com/megacrit/cardcrawl/powers/AbstractPower.java:157-164
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    // PoisonPower's constructor permits 9,999. One Catalyst+ remains inside
    // i16 (29,997); the second reaches a Java-valid stacked amount beyond it.
    engine.state.enemies[0]
        .entity
        .set_status(crate::status_ids::sid::POISON, 9_999);
    engine.state.hand = make_deck(&["Catalyst+", "Catalyst+"]);

    assert!(play_on_enemy(&mut engine, "Catalyst+", 0));
    assert!(play_on_enemy(&mut engine, "Catalyst+", 0));

    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(crate::status_ids::sid::POISON),
        89_991,
    );
}

#[test]
fn eda_004_run_combat_ai_rng_must_use_the_java_per_floor_seed() {
    // AbstractDungeon.nextRoomTransition recreates aiRng from seed + floor:
    // decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1737-1741
    let seed = 42_u64;
    let floor = 7_i32;
    let mut run = crate::run::RunEngine::new(seed, 0);
    run.run_state.floor = floor;
    run.debug_enter_specific_combat(&["JawWorm"]);

    let mut actual = run.debug_combat_engine_mut().ai_rng.clone();
    let consumed = actual.counter;
    let mut expected = crate::seed::StsRandom::new(seed + floor as u64);
    for _ in 0..consumed {
        expected.random(99);
    }

    assert_eq!(actual.random(99), expected.random(99));
}

#[test]
fn eda_004_room_reset_streams_and_persistent_potion_rng_follow_java() {
    // AbstractDungeon.nextRoomTransition resets monsterHpRng, aiRng,
    // shuffleRng, cardRandomRng, and miscRng from seed + floor, but leaves
    // dungeon-owned potionRng untouched.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:397,422,1737-1741
    let seed = 42_u64;
    let floor = 7_i32;
    let floor_seed = seed + floor as u64;
    let mut run = crate::run::RunEngine::new(seed, 0);
    run.run_state.floor = floor;
    run.run_state.potions = vec![
        "EntropicBrew".to_string(),
        "SmokeBomb".to_string(),
        String::new(),
    ];
    run.debug_enter_specific_combat(&["JawWorm"]);

    let combat = run.debug_combat_engine_mut();
    let mut expected_hp = crate::seed::StsRandom::new(floor_seed);
    assert_eq!(
        combat.state.enemies[0].entity.max_hp,
        expected_hp.random_range(40, 44),
    );

    let mut expected_shuffle = crate::seed::StsRandom::new(floor_seed);
    expected_shuffle.random_long();
    assert_eq!(combat.rng.state_tuple(), expected_shuffle.state_tuple());

    let mut expected_ai = crate::seed::StsRandom::new(floor_seed);
    for _ in 0..combat.ai_rng.counter {
        expected_ai.random(99);
    }
    assert_eq!(combat.ai_rng.state_tuple(), expected_ai.state_tuple());
    assert_eq!(
        combat.card_random_rng.state_tuple(),
        crate::seed::StsRandom::new(floor_seed).state_tuple(),
    );
    assert_eq!(
        combat.misc_rng.state_tuple(),
        crate::seed::StsRandom::new(floor_seed).state_tuple(),
    );
    assert_eq!(
        combat.potion_rng.state_tuple(),
        crate::seed::StsRandom::new(seed).state_tuple(),
    );

    let result = run.step_with_result(&crate::run::RunAction::CombatAction(
        crate::actions::Action::UsePotion {
            potion_idx: 0,
            target_idx: -1,
        },
    ));
    assert!(result.action_accepted);
    let potion_counter = run.rng_counters()["potion"];
    assert!(potion_counter > 0);

    let result = run.step_with_result(&crate::run::RunAction::CombatAction(
        crate::actions::Action::UsePotion {
            potion_idx: 1,
            target_idx: -1,
        },
    ));
    assert!(result.action_accepted);
    assert_eq!(run.current_phase(), crate::run::RunPhase::MapChoice);

    run.run_state.floor += 1;
    run.debug_enter_specific_combat(&["JawWorm"]);
    assert_eq!(
        run.debug_combat_engine_mut().potion_rng.counter as u64,
        potion_counter,
    );
}

#[test]
#[ignore = "EDA-005: run encounters rotate hard-coded pools instead of Java's seeded weighted queue"]
fn eda_005_first_weak_encounter_must_follow_monster_rng() {
    // Exordium.generateWeakEnemies normalizes four equal weights, then
    // populateMonsterList rolls monsterRng seeded with Settings.seed. Seed 4's
    // first RandomXS128 float is 0.45369965, selecting Jaw Worm from the stable
    // [Cultist, Jaw Worm, 2 Louse, Small Slimes] order.
    // decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:118-125
    // decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:390,1054-1084
    // decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterInfo.java:27-47
    let mut run = crate::run::RunEngine::new(4, 0);
    assert!(
        run.step_with_result(&crate::run::RunAction::ChooseNeowOption(0))
            .action_accepted
    );
    assert!(
        run.step_with_result(&crate::run::RunAction::ChoosePath(0))
            .action_accepted
    );

    assert_eq!(run.debug_current_enemy_ids(), vec!["JawWorm".to_string()]);
}

#[test]
fn eda_006_runtime_caused_enemy_death_must_dispatch_death_relics() {
    // MercuryHourglass.atTurnStart queues source-less THORNS damage. When that
    // kills a non-final enemy, AbstractMonster.die synchronously calls every
    // relic's onMonsterDeath; Gremlin Horn then queues one energy and one draw:
    // reference/extracted/methods/relic/MercuryHourglass.java:2-6
    // decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java:925-937
    // reference/extracted/methods/relic/GremlinHorn.java:2-8
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 3, 3),
            enemy_no_intent("Cultist", 30, 30),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.relics = vec!["Mercury Hourglass".to_string(), "Gremlin Horn".to_string()];
    engine.state.draw_pile = make_deck(&["Strike"]);
    engine.state.hand.clear();
    engine.rebuild_effect_runtime();

    engine.emit_event(crate::effects::runtime::GameEvent::empty(
        crate::effects::trigger::Trigger::TurnStart,
    ));

    assert!(engine.state.enemies[0].entity.is_dead());
    assert!(engine.state.enemies[1].is_alive());
    assert_eq!(
        engine.state.energy, 4,
        "Gremlin Horn must see the nested death"
    );
    assert_eq!(
        engine.state.hand.len(),
        1,
        "Gremlin Horn must draw one card"
    );
}

#[test]
fn eda_007_large_devotion_without_mantra_enters_divinity_without_remainder() {
    // DevotionPower special-cases a missing MantraPower: at amount >= 10 it
    // queues only ChangeStanceAction and never constructs MantraPower. Thus a
    // 12-stack Devotion enters Divinity with no residual Mantra:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevotionPower.java:35-41
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::DEVOTION, 12);
    engine.state.mantra = 0;
    engine.rebuild_effect_runtime();

    engine.emit_event(crate::effects::runtime::GameEvent::empty(
        crate::effects::trigger::Trigger::TurnStartPostDraw,
    ));

    assert_eq!(engine.state.stance, crate::state::Stance::Divinity);
    assert_eq!(engine.state.mantra, 0);
}

#[test]
fn eda_008_necronomicon_replay_must_repeat_card_counters_and_use_hooks() {
    // Necronomicon queues a CardQueueItem copy. GameActionManager processes
    // that copy as another card: it repeats onPlayCard hooks and increments the
    // played-card collections before AbstractPlayer.useCard runs. After Image
    // consequently triggers once for the original and once for the replay:
    // decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java:56-73
    // decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java:188-243
    // decompiled/java-src/com/megacrit/cardcrawl/powers/AfterImagePower.java:31-36
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 200, 200)], 3);
    force_player_turn(&mut engine);
    engine.state.relics.push("Necronomicon".to_string());
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::AFTER_IMAGE, 1);
    engine.state.hand = make_deck(&["Bludgeon"]);
    engine.rebuild_effect_runtime();

    assert!(play_on_enemy(&mut engine, "Bludgeon", 0));

    assert_eq!(engine.state.cards_played_this_turn, 2);
    assert_eq!(engine.state.attacks_played_this_turn, 2);
    assert_eq!(engine.state.total_cards_played, 2);
    assert_eq!(engine.state.player.block, 2);
    assert_eq!(engine.state.enemies[0].entity.hp, 136);
    assert_eq!(
        engine
            .state
            .discard_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Bludgeon")
            .count(),
        1,
        "the purge-on-use replay must not create a second pile card"
    );
}

#[test]
fn eda_008_necronomicon_x_replay_reuses_energy_on_use_and_chemical_x() {
    // Necronomicon copies the original CardQueueItem.energyOnUse and marks the
    // replay free/autoplay. Each Whirlwind use independently receives Chemical
    // X's +2 effect while only the original spends EnergyPanel.totalCount.
    // decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java:56-73
    // decompiled/java-src/com/megacrit/cardcrawl/cards/CardQueueItem.java:41-46
    // decompiled/java-src/com/megacrit/cardcrawl/relics/ChemicalX.java
    let mut engine =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 200, 200)], 2);
    force_player_turn(&mut engine);
    engine.state.relics.extend([
        "Necronomicon".to_string(),
        "Chemical X".to_string(),
    ]);
    engine.state.hand = make_deck(&["Whirlwind"]);
    engine.rebuild_effect_runtime();

    assert!(play_on_enemy(&mut engine, "Whirlwind", 0));

    assert_eq!(engine.state.energy, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 160);
    assert_eq!(engine.state.cards_played_this_turn, 2);
    assert_eq!(engine.state.attacks_played_this_turn, 2);
    assert_eq!(engine.state.total_cards_played, 2);
    assert_eq!(engine.state.discard_pile.len(), 1);
}

#[test]
fn eda_009_cultist_ritual_must_buff_after_the_second_round_attack() {
    // RitualPower.skipFirst is cleared at the end of the Incantation round.
    // The first Dark Strike therefore deals its unbuffed six damage, and only
    // that round's atEndOfRound grants three Strength:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/RitualPower.java:19,46-54
    // reference/extracted/methods/monster/Cultist.java:2-17
    let mut run = crate::run::RunEngine::new(99, 0);
    run.debug_enter_specific_combat(&["Cultist"]);
    let engine = run.debug_combat_engine_mut();
    engine.state.player.max_hp = 999;
    engine.state.player.hp = 999;

    crate::tests::support::end_turn(engine); // Incantation, then player turn 2.
    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(crate::status_ids::sid::STRENGTH),
        0
    );

    engine.state.player.block = 0;
    let hp_before_dark_strike = engine.state.player.hp;
    crate::tests::support::end_turn(engine);

    assert_eq!(hp_before_dark_strike - engine.state.player.hp, 6);
    assert_eq!(
        engine.state.enemies[0]
            .entity
            .status(crate::status_ids::sid::STRENGTH),
        3
    );
}

#[test]
fn eda_010_static_discharge_auto_evoke_must_honor_electrodynamics_and_channel_count() {
    // StaticDischargePower queues an ordinary ChannelAction. Filling the last
    // slot auto-evokes the front Lightning; with Electrodynamics, Lightning's
    // onEvoke damages every enemy. The newly channeled Lightning also counts
    // for Thunder Strike just like every other ChannelAction:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/StaticDischargePower.java:29-34
    // decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ChannelAction.java:28-36
    // decompiled/java-src/com/megacrit/cardcrawl/orbs/Lightning.java:63-68
    let mut engine = engine_without_start(
        Vec::new(),
        vec![
            crate::tests::support::enemy("JawWorm", 40, 40, 1, 5, 1),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    );
    force_player_turn(&mut engine);
    engine.init_defect_orbs(1);
    engine.channel_orb(crate::orbs::OrbType::Lightning);
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::STATIC_DISCHARGE, 1);
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::ELECTRODYNAMICS, 1);
    engine.rebuild_effect_runtime();

    crate::tests::support::end_turn(&mut engine);

    // End-of-turn passive (3) plus the displaced Lightning evoke (8) hits both.
    assert_eq!(engine.state.enemies[0].entity.hp, 29);
    assert_eq!(engine.state.enemies[1].entity.hp, 29);
    assert_eq!(
        engine
            .state
            .player
            .status(crate::status_ids::sid::LIGHTNING_CHANNELED),
        2,
    );
}

#[test]
fn eda_011_final_enemy_spore_cloud_death_must_not_apply_vulnerable() {
    // SporeCloudPower.onDeath returns when AbstractRoom.isBattleEnding;
    // isBattleEnding is true as soon as the monster group is basically dead.
    // The final enemy's Spore Cloud therefore queues no VulnerablePower:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/SporeCloudPower.java:36-43
    // decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java:640-647
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("FungiBeast", 1, 1)], 3);
    force_player_turn(&mut engine);
    engine.state.enemies[0]
        .entity
        .set_status(crate::status_ids::sid::SPORE_CLOUD, 2);

    engine.deal_damage_to_enemy(0, 1);

    assert!(engine.state.enemies[0].entity.is_dead());
    assert_eq!(
        engine
            .state
            .player
            .status(crate::status_ids::sid::VULNERABLE),
        0,
    );
}

#[test]
fn eda_012_combat_card_group_shuffle_matches_java_order_and_one_outer_tick() {
    // CardGroup.shuffle consumes exactly one shuffleRng.randomLong even for
    // empty/singleton groups, then delegates the actual permutation to a new
    // java.util.Random seeded from that value.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java:550-555
    let cases: &[&[&str]] = &[
        &[],
        &["Strike"],
        &["Strike", "Defend"],
        &["Strike", "Defend", "Bash", "Inflame", "Anger", "Flex"],
    ];

    for &card_ids in cases {
        let mut engine =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
        engine.rng = crate::seed::StsRandom::new(73);
        engine.state.draw_pile = make_deck(card_ids);

        let mut expected = engine.state.draw_pile.clone();
        let mut expected_rng = crate::seed::StsRandom::new(73);
        let java_seed = expected_rng.random_long();
        crate::seed::java_util_shuffle(&mut expected, java_seed);

        engine.shuffle_draw_pile();

        assert_eq!(engine.state.draw_pile, expected, "case: {card_ids:?}");
        assert_eq!(engine.rng.counter, 1, "case: {card_ids:?}");
    }
}

#[test]
fn eda_012_opening_draw_and_empty_deck_reshuffle_share_the_java_contract() {
    // Combat start calls CardGroup.shuffle before drawing, and
    // EmptyDeckShuffleAction shuffles the discard group before moving it into
    // draw. Both paths must use the same one-tick Java permutation.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java:550-555
    // decompiled/java-src/com/megacrit/cardcrawl/actions/common/EmptyDeckShuffleAction.java:38-43
    let card_ids = ["Strike", "Defend", "Bash", "Inflame", "Anger", "Flex"];
    let cards = make_deck(&card_ids);

    let mut expected_opening = cards.clone();
    let mut expected_opening_rng = crate::seed::StsRandom::new(91);
    let opening_seed = expected_opening_rng.random_long();
    crate::seed::java_util_shuffle(&mut expected_opening, opening_seed);
    let expected_opening_hand = (0..5)
        .map(|_| expected_opening.pop().expect("six-card opening deck"))
        .collect::<Vec<_>>();

    let state = combat_state_with(
        cards.clone(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    let mut opening = crate::engine::CombatEngine::new(state, 91);
    opening.start_combat();
    assert_eq!(opening.state.hand, expected_opening_hand);
    assert_eq!(opening.state.draw_pile, expected_opening);
    assert_eq!(opening.rng.counter, 1);

    let mut expected_reshuffle = cards.clone();
    let mut expected_reshuffle_rng = crate::seed::StsRandom::new(91);
    let reshuffle_seed = expected_reshuffle_rng.random_long();
    crate::seed::java_util_shuffle(&mut expected_reshuffle, reshuffle_seed);
    let expected_reshuffle_hand = (0..3)
        .map(|_| expected_reshuffle.pop().expect("six-card discard pile"))
        .collect::<Vec<_>>();

    let mut reshuffle =
        engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut reshuffle);
    reshuffle.rng = crate::seed::StsRandom::new(91);
    reshuffle.state.hand.clear();
    reshuffle.state.draw_pile.clear();
    reshuffle.state.discard_pile = cards;
    reshuffle.draw_cards(3);
    assert_eq!(reshuffle.state.hand, expected_reshuffle_hand);
    assert_eq!(reshuffle.state.draw_pile, expected_reshuffle);
    assert_eq!(reshuffle.rng.counter, 1);
}

#[test]
fn eda_013_player_poison_must_tick_after_enemy_turn_at_owner_turn_start() {
    // GameActionManager runs monster turns before player atStartOfTurn powers.
    // PoisonPower then queues PoisonLoseHpAction at the poisoned owner's turn
    // start. Buffer must therefore absorb the enemy hit before Poison resolves.
    // decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java:297-359
    // decompiled/java-src/com/megacrit/cardcrawl/powers/PoisonPower.java:58-64
    let mut engine = engine_without_start(
        Vec::new(),
        vec![crate::tests::support::enemy("JawWorm", 100, 100, 1, 6, 1)],
        3,
    );
    force_player_turn(&mut engine);
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::POISON, 5);
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::BUFFER, 1);
    let hp_before = engine.state.player.hp;

    crate::tests::support::end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before - 5);
    assert_eq!(
        engine.state.player.status(crate::status_ids::sid::POISON),
        4,
    );
    assert_eq!(
        engine.state.player.status(crate::status_ids::sid::BUFFER),
        0,
    );
}
