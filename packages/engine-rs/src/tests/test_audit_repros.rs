#![cfg(test)]

//! Intentionally failing engine-path reproducers discovered by the deep audit.
//!
//! These remain ignored until their registered `EDA-*` work units are fixed.

use crate::tests::support::{
    combat_state_with, enemy_no_intent, engine_with_state, engine_without_start, force_player_turn,
    make_deck, play_on_enemy,
};

#[test]
#[ignore = "EDA-001: CombatVictory dispatch stops after its first handler"]
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
#[ignore = "EDA-002: CardInstance.misc truncates Java int damage state to i16"]
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
    assert_eq!(i32::from(played.misc), 32_770);
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
#[ignore = "EDA-004: per-combat aiRng is not seeded with Settings.seed + floorNum"]
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
#[ignore = "EDA-005: player Regeneration incorrectly loses one stack each turn"]
fn eda_005_regeneration_amount_must_not_decay_after_healing() {
    // RegenPower.atEndOfTurn queues RegenAction but never reduces its amount:
    // decompiled/java-src/com/megacrit/cardcrawl/powers/RegenPower.java:31-39
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 50;
    engine
        .state
        .player
        .set_status(crate::status_ids::sid::REGENERATION, 2);

    crate::tests::support::end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 52);
    assert_eq!(
        engine
            .state
            .player
            .status(crate::status_ids::sid::REGENERATION),
        2,
    );
}

#[test]
#[ignore = "EDA-006: nested runtime events are dispatched into an empty EffectRuntime"]
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
#[ignore = "EDA-007: Devotion >= 10 creates Mantra remainder when no MantraPower exists"]
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
#[ignore = "EDA-008: Necronomicon replay bypasses the normal card-use pipeline"]
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
}

#[test]
#[ignore = "EDA-009: enemy Ritual grants Strength before its first Dark Strike"]
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
#[ignore = "EDA-010: Static Discharge full-slot channel bypasses the orb evoke pipeline"]
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
#[ignore = "EDA-011: Spore Cloud debuffs the player when the final enemy dies"]
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
