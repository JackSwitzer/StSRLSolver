//! U05 — trace oracle tests (`docs/goal/TOOLING.md` § T5, `docs/goal/UNITS.md` U05).
//!
//! No real Java golden exists yet (that's U04/U06's job), so this suite is
//! synthetic-fixture based: it generates a Rust trace in-process for a tiny
//! scripted sequence from a fixed seed, treats a clone of it as the "java"
//! side, and diffs via `sts_engine::trace::diff_records` — the exact same
//! differ `bin/trace_replay.rs` uses. It also replays any committed golden
//! under `data/traces/java/*.jsonl` against its matching script (currently
//! none — the loop below no-ops gracefully when the directory is empty or
//! missing, per T5's "part of the lib suite" requirement).
//!
//! Seed 4 / ascension 0 (Watcher) is used because floor 1's first map node
//! is naturally a `Monster` room (`RoomType::Monster`) reached by
//! `ChoosePath(0)` with no map-state hacking. Neow option 1 is Neow's
//! Lament, an immediate reward with no follow-up decision.

use std::fs;

use crate::run::GameAction;
use crate::trace::{
    diff_records, ActionScript, DivergenceStatus, ScriptStopCondition, TraceAction,
};

#[test]
fn oracle_projects_darkling_half_death_like_java() {
    // Darkling.damage marks a lethal Darkling halfDead, clears its powers,
    // calls setMove(COUNT), and queues a second SetMoveAction(COUNT). It does
    // not set AbstractMonster.isDead while another Darkling remains alive.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/beyond/Darkling.java.
    let mut run = crate::run::RunEngine::new(326, 0);
    run.debug_enter_specific_combat(&["Darkling", "Darkling", "Darkling"]);
    let opening_move = run.get_combat_engine().unwrap().state.enemies[0].move_id;
    let hp = run.get_combat_engine().unwrap().state.enemies[0].entity.hp;
    run.debug_combat_engine_mut().deal_damage_to_enemy(0, hp);
    run.debug_combat_engine_mut().deal_damage_to_enemy(0, 22);
    assert_eq!(
        run.get_combat_engine().unwrap().state.enemies[0].entity.hp,
        0,
        "AbstractMonster.damage clamps repeated half-dead damage at zero"
    );

    let oracle = crate::trace::oracle_v2::project_oracle_state(&run).unwrap();
    assert!(!oracle.enemies[0].dead);
    assert_eq!(oracle.enemies[0].intent.name, "UNKNOWN");
    assert_eq!(
        oracle.enemies[0].move_history,
        [
            opening_move,
            crate::enemies::move_ids::DARK_WAIT,
            crate::enemies::move_ids::DARK_WAIT,
        ]
    );
}

#[test]
fn run_trace_exposes_every_java_rng_counter_before_and_during_combat() {
    // Sources:
    // - decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
    // - decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java
    // Seed 4's first Exordium encounter is a single Jaw Worm, so entering
    // combat does not consume miscRng while expanding a composite encounter.
    let mut run = crate::run::RunEngine::new(4, 0);
    let expected_keys: std::collections::BTreeSet<&str> =
        crate::trace::CANONICAL_RNG_KEYS.iter().copied().collect();

    let before = run.rng_counters();
    assert_eq!(
        before
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        expected_keys
    );
    assert_eq!(before["relic"], 5);
    assert_eq!(before["misc"], 1);
    assert!(before["monster"] > 0);
    assert!(before["map"] > 0);

    assert_eq!(run.get_legal_actions(), vec![GameAction::Proceed]);
    assert!(run.step_game(&GameAction::Proceed).accepted());
    assert!(run.step_game(&GameAction::ChooseNeowOption(1)).accepted());
    assert_eq!(run.get_legal_actions(), vec![GameAction::Proceed]);
    assert!(run.step_game(&GameAction::Proceed).accepted());
    assert!(run.step_game(&GameAction::ChoosePath(0)).accepted());
    let combat = run.rng_counters();
    assert_eq!(
        combat
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        expected_keys
    );
    assert_eq!(combat["relic"], 5);
    assert_eq!(combat["misc"], 0);
    assert_eq!(combat["monsterHp"], 1);
    assert_eq!(combat["shuffle"], 1);
    assert_eq!(combat["cardRandom"], 0);
}

#[test]
fn trace_preserves_java_potion_slot_placeholders_outside_and_during_combat() {
    // PotionSlot.POTION_ID is "Potion Slot", and AbstractPlayer keeps one
    // PotionSlot object for every vacant inventory position.
    // Sources:
    // - decompiled/java-src/com/megacrit/cardcrawl/potions/PotionSlot.java
    // - decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    let mut run = crate::run::RunEngine::new(4, 0);
    run.run_state.potions = vec![
        "DexterityPotion".to_string(),
        String::new(),
        "FirePotion".to_string(),
    ];

    let outside = crate::trace::build_post_state(&run);
    assert_eq!(
        outside.potions,
        ["DexterityPotion", "Potion Slot", "FirePotion"]
    );

    run.debug_enter_specific_combat(&["JawWorm"]);
    let during = crate::trace::build_post_state(&run);
    assert_eq!(
        during.potions,
        ["DexterityPotion", "Potion Slot", "FirePotion"]
    );
}

#[test]
fn trace_powers_keep_java_application_order_and_hide_ai_counters_after_restore() {
    // ApplyPowerAction appends a new power and Collections.sort uses
    // AbstractPower.priority only. Java's stable sort therefore preserves
    // application order for equal-priority Spore Cloud and Strength.
    // Sources: ApplyPowerAction.java:163-165, AbstractPower.java:370-372,
    // SporeCloudPower.java:16, StrengthPower.java:17.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let enemy = &mut run.debug_combat_engine_mut().state.enemies[0];
        enemy
            .entity
            .set_status(crate::status_ids::sid::SPORE_CLOUD, 2);
        enemy.entity.set_status(crate::status_ids::sid::STRENGTH, 3);
        assert!(enemy.entity.status(crate::status_ids::sid::STARTING_DMG) != 0);
    }

    let before = crate::trace::build_post_state(&run);
    assert_eq!(
        before.enemies[0]
            .powers
            .iter()
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [("Spore Cloud", 2), ("Strength", 3)]
    );

    let encoded = crate::checkpoint::CoreCheckpoint::capture(&run)
        .unwrap()
        .to_json()
        .unwrap();
    let restored = crate::checkpoint::CoreCheckpoint::from_json(&encoded)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(crate::trace::build_post_state(&restored), before);
}

#[test]
fn power_order_distinguishes_sorted_apply_from_unsorted_direct_add() {
    // ApplyPowerAction appends and stable-sorts by priority, while
    // AbstractCreature.addPower/powers.add append without sorting.
    // Sources: ApplyPowerAction.java:137-165; AbstractCreature.java:511-513.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let enemy = &mut run.debug_combat_engine_mut().state.enemies[0].entity;
        enemy.set_status_direct(crate::status_ids::sid::REACTIVE, 1);
        enemy.set_status_direct(crate::status_ids::sid::MALLEABLE, 3);
    }
    let direct = crate::trace::build_post_state(&run);
    assert_eq!(
        direct.enemies[0]
            .powers
            .iter()
            .map(|power| power.id.as_str())
            .collect::<Vec<_>>(),
        ["Compulsive", "Malleable"]
    );

    run.debug_combat_engine_mut().state.enemies[0]
        .entity
        .set_status(crate::status_ids::sid::STRENGTH, 2);
    let sorted = crate::trace::build_post_state(&run);
    assert_eq!(
        sorted.enemies[0]
            .powers
            .iter()
            .map(|power| power.id.as_str())
            .collect::<Vec<_>>(),
        ["Malleable", "Strength", "Compulsive"]
    );
}

#[test]
fn ordered_dynamic_marker_powers_survive_checkpoint_and_project_java_identity() {
    // These powers carry gameplay payload in compact typed fields, but they
    // still occupy AbstractCreature.powers and must participate in its stable
    // priority ordering. Pen Nib alone has priority 6; the marker powers use
    // Java's default priority 5 and expose amount -1.
    // Sources: powers/{MinionPower,BackAttackPower,StasisPower,PenNibPower}.java.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::CONFUSION, 1);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::PEN_NIB_POWER, 1);

        let held = combat.state.draw_pile.pop().expect("fixture draw card");
        let enemy = &mut combat.state.enemies[0];
        enemy.entity.set_status(crate::status_ids::sid::STRENGTH, 2);
        enemy.set_minion(true);
        enemy.set_back_attack(true);
        enemy.set_stasis_card(held);
    }

    let before = crate::trace::build_post_state(&run);
    assert_eq!(
        before
            .player
            .powers
            .iter()
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [("Confusion", -1), ("Pen Nib", 1)]
    );
    assert_eq!(
        before.enemies[0]
            .powers
            .iter()
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [
            ("Strength", 2),
            ("Minion", -1),
            ("BackAttack", -1),
            ("Stasis", -1),
        ]
    );

    let encoded = crate::checkpoint::CoreCheckpoint::capture(&run)
        .unwrap()
        .to_json()
        .unwrap();
    let restored = crate::checkpoint::CoreCheckpoint::from_json(&encoded)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(crate::trace::build_post_state(&restored), before);
}

#[test]
fn initial_minion_and_backattack_powers_precede_philosophers_strength() {
    // AbstractPlayer.preBattlePrep queues monster pre-battle actions, then the
    // room drains them before relic atBattleStart callbacks run. Philosopher's
    // Stone therefore appends equal-priority Strength after settled markers.
    // Sources: AbstractPlayer.java:1600-1605, AbstractRoom.java:228-245,
    // GremlinLeader.java:92-97, and PhilosopherStone.java:38-42.
    let cases: &[(&[&str], usize, &str)] = &[
        (&["GremlinFat", "GremlinLeader"], 0, "Minion"),
        (&["Dagger", "Reptomancer", "Dagger"], 0, "Minion"),
        (&["SpireShield", "SpireSpear"], 0, "BackAttack"),
    ];

    for (encounter, target_idx, marker) in cases {
        let mut run = crate::run::RunEngine::new(4, 0);
        run.run_state.relics.push("Philosopher's Stone".to_string());
        run.debug_enter_specific_combat(encounter);
        let post = crate::trace::build_post_state(&run);
        let ids = post.enemies[*target_idx]
            .powers
            .iter()
            .map(|power| power.id.as_str())
            .collect::<Vec<_>>();
        let marker = ids.iter().position(|id| *id == *marker).unwrap();
        let strength = ids.iter().position(|id| *id == "Strength").unwrap();
        assert!(marker < strength, "{encounter:?} power order was {ids:?}");
    }
}

#[test]
fn shield_spear_settled_power_order_matches_java_at_a0_and_a18() {
    // MonsterGroup initializes Shield then Spear. Surrounded applies first and
    // inserts Shield BackAttack, Artifact actions resolve next, and
    // Philosopher's Stone appends Strength during relic atBattleStart.
    // Sources: SpireShield.java, SurroundedPower.java, AbstractRoom.java, and
    // PhilosopherStone.java.
    for ascension in [0, 18] {
        for has_stone in [false, true] {
            let mut run = crate::run::RunEngine::new(4, ascension);
            if has_stone {
                run.run_state.relics.push("Philosopher's Stone".to_string());
            }
            run.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
            let combat = run.get_combat_engine().expect("Shield/Spear combat");
            assert_eq!(
                combat
                    .state
                    .player
                    .status(crate::status_ids::sid::SURROUNDED_POWER),
                1
            );
            assert!(combat.state.player.power_order.contains(
                &crate::state::PowerOrderEntry::Status(crate::status_ids::sid::SURROUNDED_POWER,)
            ));
            let post = crate::trace::build_post_state(&run);
            let artifact = if ascension >= 18 { 2 } else { 1 };

            assert_eq!(
                post.player
                    .powers
                    .iter()
                    .map(|power| (power.id.as_str(), power.amt))
                    .collect::<Vec<_>>(),
                vec![("Surrounded", -1)]
            );

            let mut expected_shield = vec![("BackAttack", -1), ("Artifact", artifact)];
            let mut expected_spear = vec![("Artifact", artifact)];
            if has_stone {
                expected_shield.push(("Strength", 1));
                expected_spear.push(("Strength", 1));
            }
            assert_eq!(
                post.enemies[0]
                    .powers
                    .iter()
                    .map(|power| (power.id.as_str(), power.amt))
                    .collect::<Vec<_>>(),
                expected_shield
            );
            assert_eq!(
                post.enemies[1]
                    .powers
                    .iter()
                    .map(|power| (power.id.as_str(), power.amt))
                    .collect::<Vec<_>>(),
                expected_spear
            );
        }
    }
}

#[test]
fn surrounded_owner_death_checkpoint_preserves_corpse_backattack() {
    // SpireShield.die skips its dying owner while queuing Surrounded cleanup.
    // At the next actionable boundary the player power is gone, but the
    // corpse-owned BackAttack remains causal and keeps its exact power order.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
    assert!(run.debug_combat_engine_mut().instant_kill_enemy(0));

    let before = crate::trace::build_post_state(&run);
    assert!(before
        .player
        .powers
        .iter()
        .all(|power| power.id != "Surrounded"));
    assert!(before.enemies[0]
        .powers
        .iter()
        .any(|power| power.id == "BackAttack"));
    let restored = crate::checkpoint::CoreCheckpoint::capture(&run)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(crate::trace::build_post_state(&restored), before);
}

#[test]
fn trace_power_projection_uses_java_ids_priorities_and_compound_amounts() {
    // Sources: ConfusionPower.java, WraithFormPower.java, TimeWarpPower.java,
    // InvinciblePower.java, and AbstractPower.compareTo.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::WRAITH_FORM, 2);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::CONFUSION, 1);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::ENERGIZED, 2);
        combat
            .state
            .player
            .set_status(crate::status_ids::sid::ENERGIZED_BLUE, 3);

        let enemy = &mut combat.state.enemies[0].entity;
        enemy.set_status(crate::status_ids::sid::INVINCIBLE, 200);
        enemy.set_status(crate::status_ids::sid::INVINCIBLE_DAMAGE_TAKEN, 37);
        enemy.set_status(crate::status_ids::sid::TIME_WARP_ACTIVE, 1);
        enemy.set_status(crate::status_ids::sid::TIME_WARP, 0);
        enemy.set_status(crate::status_ids::sid::FIRST_MOVE, 1);
    }

    let post = crate::trace::build_post_state(&run);
    assert_eq!(
        post.player
            .powers
            .iter()
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [
            ("Confusion", -1),
            ("Wraith Form v2", -2),
            ("Energized", 2),
            ("EnergizedBlue", 3),
        ]
    );
    assert_eq!(
        post.enemies[0]
            .powers
            .iter()
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [("Time Warp", 0), ("Invincible", 163)]
    );
}

#[test]
fn trace_projects_checkpoint_causal_the_bomb_instances_and_ids() {
    // TheBombPower uses a process-global signed-int suffix, exposes turns as
    // amount, and keeps each damage payload independent.
    // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/TheBombPower.java:23-44.
    let mut run = crate::run::RunEngine::new(4, 0);
    run.restore_the_bomb_id_offset(41);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        combat.schedule_the_bomb(3, 40);
        combat.schedule_the_bomb(2, 50);
        assert_eq!(
            combat.state.pending_bombs.as_slice(),
            &[
                crate::state::PendingBomb {
                    java_serial: 41,
                    turns: 3,
                    damage: 40,
                },
                crate::state::PendingBomb {
                    java_serial: 42,
                    turns: 2,
                    damage: 50,
                },
            ]
        );
    }

    let post = crate::trace::build_post_state(&run);
    assert_eq!(
        post.player
            .powers
            .iter()
            .filter(|power| power.id.starts_with("TheBomb"))
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [("TheBomb41", 3), ("TheBomb42", 2)]
    );
    assert_eq!(run.the_bomb_id_offset(), 43);

    let encoded = crate::checkpoint::CoreCheckpoint::capture(&run)
        .unwrap()
        .to_json()
        .unwrap();
    let mut restored = crate::checkpoint::CoreCheckpoint::from_json(&encoded)
        .unwrap()
        .restore()
        .unwrap();
    assert_eq!(restored.the_bomb_id_offset(), 43);
    assert_eq!(crate::trace::build_post_state(&restored), post);

    assert!(restored
        .step_game(&crate::run::GameAction::CombatAction(
            crate::actions::Action::EndTurn,
        ))
        .accepted());
    let after_turn = crate::trace::build_post_state(&restored);
    assert_eq!(
        after_turn
            .player
            .powers
            .iter()
            .filter(|power| power.id.starts_with("TheBomb"))
            .map(|power| (power.id.as_str(), power.amt))
            .collect::<Vec<_>>(),
        [("TheBomb41", 2), ("TheBomb42", 1)]
    );
}

#[test]
fn the_bomb_process_global_uses_java_signed_int_wrapping() {
    let mut run = crate::run::RunEngine::new(4, 0);
    run.restore_the_bomb_id_offset(i32::MAX);
    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        combat.schedule_the_bomb(3, 40);
        combat.schedule_the_bomb(3, 40);
    }
    assert_eq!(run.the_bomb_id_offset(), i32::MIN.wrapping_add(1));
    let ids = crate::trace::build_post_state(&run)
        .player
        .powers
        .into_iter()
        .filter(|power| power.id.starts_with("TheBomb"))
        .map(|power| power.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, ["TheBomb2147483647", "TheBomb-2147483648"]);
}

#[test]
fn trace_projects_constructor_and_prebattle_power_order_for_watcher_enemies() {
    // Sources: AcidSlime_L.java:93, SpikeSlime_L.java:85,
    // SlimeBoss.java:99, Transient.java:62-68, and AwakenedOne.java:138-150.
    let cases = [
        ("AcidSlime_L", "AcidSlime_L", vec![("Split", -1)]),
        ("SpikeSlime_L", "SpikeSlime_L", vec![("Split", -1)]),
        ("SlimeBoss", "SlimeBoss", vec![("Split", -1)]),
        (
            "Transient",
            "Transient",
            vec![("Fading", 5), ("Shifting", -1)],
        ),
        (
            "AwakenedOne",
            "AwakenedOne",
            vec![("Regenerate", 10), ("Curiosity", 1), ("Unawakened", -1)],
        ),
    ];

    for (encounter_id, enemy_id, expected) in cases {
        let mut run = crate::run::RunEngine::new(4, 0);
        run.debug_enter_specific_combat(&[encounter_id]);
        let post = crate::trace::build_post_state(&run);
        let enemy = post
            .enemies
            .iter()
            .find(|enemy| enemy.id == enemy_id)
            .unwrap();
        assert_eq!(
            enemy
                .powers
                .iter()
                .map(|power| (power.id.as_str(), power.amt))
                .collect::<Vec<_>>(),
            expected,
            "{enemy_id}"
        );
    }
}

#[test]
fn trace_emits_real_relic_counters_from_run_and_combat_runtime_state() {
    // TraceWriter serializes AbstractRelic.counter directly. These fixtures
    // cover the run-owned, runtime-owned, sentinel, and non-counter paths.
    // Sources:
    // - packages/harness-java/src/main/java/tracelab/TraceWriter.java
    // - decompiled/java-src/com/megacrit/cardcrawl/relics/Nunchaku.java
    // - decompiled/java-src/com/megacrit/cardcrawl/relics/Matryoshka.java
    // - decompiled/java-src/com/megacrit/cardcrawl/relics/AncientTeaSet.java
    let mut run = crate::run::RunEngine::new(4, 0);
    run.run_state.relics = vec![
        "PureWater".to_string(),
        "Nunchaku".to_string(),
        "Matryoshka".to_string(),
        "Ancient Tea Set".to_string(),
        "Lizard Tail".to_string(),
        "Circlet".to_string(),
    ];
    run.run_state.relic_flags.rebuild(&run.run_state.relics);
    run.run_state.relic_flags.init_relic_counter("Matryoshka");
    run.run_state.relic_flags.counters[crate::relic_flags::counter::ANCIENT_TEA_SET] = 1;
    run.run_state.lizard_tail_used = true;
    run.run_state
        .persisted_effect_states
        .push(crate::effects::runtime::PersistedEffectState {
            def_id: "Nunchaku".to_string(),
            values: vec![7],
        });

    let outside = crate::trace::build_post_state(&run);
    assert_eq!(
        outside
            .relics
            .iter()
            .map(|relic| (relic.id.as_str(), relic.counter))
            .collect::<Vec<_>>(),
        [
            ("PureWater", -1),
            ("Nunchaku", 7),
            ("Matryoshka", 2),
            ("Ancient Tea Set", -2),
            ("Lizard Tail", -2),
            ("Circlet", 1),
        ]
    );

    run.debug_enter_specific_combat(&["JawWorm"]);
    {
        let combat = run.debug_combat_engine_mut();
        assert!(combat.set_hidden_effect_value(
            "Nunchaku",
            crate::effects::runtime::EffectOwner::PlayerRelic { slot: 1 },
            0,
            8,
        ));
    }
    let during = crate::trace::build_post_state(&run);
    assert_eq!(during.relics[0].counter, -1);
    assert_eq!(during.relics[1].counter, 8);
    assert_eq!(during.relics[2].counter, 2);
    assert_eq!(during.relics[3].counter, -1);
    assert_eq!(during.relics[4].counter, -2);
    assert_eq!(during.relics[5].counter, 1);
}

#[test]
fn trace_diff_reports_relic_counter_mismatches() {
    let script = tiny_fixture_script();
    let rust_records =
        crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    let mut java_records = rust_records.clone();
    java_records[0].post.relics[0].counter = 4;

    let report = diff_records(
        "relic-counter-doctored",
        &script.seed,
        &java_records,
        &rust_records,
        &[],
    );
    let first = report
        .first_divergence
        .expect("counter mismatch must diverge");
    assert_eq!(first.idx, 0);
    assert_eq!(first.path, "post.relics[0].counter");
    assert_eq!(first.java, serde_json::json!(4));
    assert_eq!(first.rust, serde_json::json!(-1));
}

fn assert_record_mutation_path(
    label: &str,
    record_index: usize,
    expected_path: &str,
    mutate: impl FnOnce(&mut crate::trace::TraceRecord, &mut crate::trace::TraceRecord),
) {
    let script = tiny_fixture_script();
    let rust_records = crate::trace::replay_script(&script).expect("fixture replay");
    let mut java_records = rust_records.clone();
    let mut comparison_records = rust_records;
    mutate(
        &mut java_records[record_index],
        &mut comparison_records[record_index],
    );

    let report = diff_records(label, &script.seed, &java_records, &comparison_records, &[]);
    let first = report.first_divergence.expect("mutation must diverge");
    assert_eq!(first.path, expected_path);
}

#[test]
fn trace_diff_covers_record_identity_and_action() {
    assert_record_mutation_path("idx", 1, "idx", |java, _| java.idx += 1);
    assert_record_mutation_path("floor", 1, "floor", |java, _| java.floor += 1);
    assert_record_mutation_path("turn", 2, "turn", |java, _| java.turn += 1);
    assert_record_mutation_path("phase", 2, "phase", |java, _| {
        java.phase = "EVENT".to_string()
    });
    assert_record_mutation_path("action", 2, "action", |java, _| {
        java.action = TraceAction::EndTurn;
    });
}

#[test]
fn trace_diff_covers_every_nested_post_state_family() {
    assert_record_mutation_path(
        "player-power",
        2,
        "post.player.powers[0].amt",
        |java, rust| {
            let power = crate::trace::PowerPostState {
                id: "Vigor".to_string(),
                amt: 8,
            };
            java.post.player.powers = vec![power.clone()];
            rust.post.player.powers = vec![power];
            java.post.player.powers[0].amt = 9;
        },
    );
    assert_record_mutation_path(
        "player-orb",
        2,
        "post.player.orbs[0].passive_amount",
        |java, rust| {
            let orb = crate::trace::OrbPostState {
                id: "Lightning".to_string(),
                evoke_amount: 8,
                passive_amount: 3,
            };
            java.post.player.orbs = vec![orb.clone()];
            rust.post.player.orbs = vec![orb];
            java.post.player.orbs[0].passive_amount = 4;
        },
    );
    assert_record_mutation_path("enemy-index", 2, "post.enemies[0].idx", |java, _| {
        java.post.enemies[0].idx += 1
    });
    assert_record_mutation_path(
        "intent-name",
        2,
        "post.enemies[0].intent.name",
        |java, _| java.post.enemies[0].intent.name.push_str("_DOCTORED"),
    );
    assert_record_mutation_path(
        "enemy-power",
        2,
        "post.enemies[0].powers[0].id",
        |java, rust| {
            let power = crate::trace::PowerPostState {
                id: "Ritual".to_string(),
                amt: 3,
            };
            java.post.enemies[0].powers = vec![power.clone()];
            rust.post.enemies[0].powers = vec![power];
            java.post.enemies[0].powers[0].id = "Strength".to_string();
        },
    );
    assert_record_mutation_path(
        "move-history",
        3,
        "post.enemies[0].move_history",
        |java, _| java.post.enemies[0].move_history.push(99),
    );
}

/// The tiny scripted sequence used by both tests below: resolve Neow, take
/// the first map path into floor 1 combat (vs a lone Jaw Worm for seed 4),
/// play the first Defend in Java-shuffled opening-hand order, then end the turn.
fn tiny_fixture_script() -> ActionScript {
    ActionScript {
        v: 1,
        seed: crate::seed::seed_to_string(4),
        character: "WATCHER".to_string(),
        ascension: 0,
        stop: ScriptStopCondition {
            max_floor: None,
            max_actions: None,
        },
        actions: vec![
            TraceAction::Neow { choice: 1 },
            TraceAction::Path { choice: 0 },
            TraceAction::PlayCard {
                hand_idx: 3,
                target: -1,
                card_id: Some("Defend".to_string()),
            },
            TraceAction::EndTurn,
        ],
    }
}

#[test]
fn synthetic_self_diff_matches() {
    let script = tiny_fixture_script();
    let rust_records =
        crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert_eq!(
        rust_records.len(),
        4,
        "expected one record per scripted action"
    );
    assert_eq!(rust_records[0].action, TraceAction::Neow { choice: 1 });
    assert_eq!(rust_records[0].phase, "NEOW");
    assert_eq!(rust_records[1].phase, "COMBAT");

    // Treat an identical clone as the "java" side.
    let java_records = rust_records.clone();

    let report = diff_records(
        "synthetic-fixture",
        &script.seed,
        &java_records,
        &rust_records,
        &[],
    );
    assert_eq!(
        report.status,
        DivergenceStatus::Match,
        "identical traces must report match: {report:?}"
    );
    assert_eq!(report.matched_actions, 4);
    assert_eq!(report.total_actions, 4);
    assert!(report.first_divergence.is_none());
    assert!(report.masked.is_empty());
    assert!(report.secondary.is_empty());
}

#[test]
fn doctored_hp_and_rng_are_reported_as_first_divergence() {
    let script = tiny_fixture_script();
    let rust_records =
        crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert!(
        rust_records.len() >= 3,
        "fixture must have a play-card record to doctor"
    );

    // Doctor the "java" side at record index 2 (after PLAY_CARD): bump the
    // player's hp by 1 (an impossible value given the real replay) and the
    // `ai` rng counter by 1. hp is checked first among player fields but
    // rng counters are checked first overall (T3): the report's
    // first_divergence must land on the rng path, not hp.
    let mut java_records = rust_records.clone();
    let doctored = &mut java_records[2];
    let original_hp = doctored.post.player.hp;
    doctored.post.player.hp = original_hp + 1;
    let original_ai = *doctored
        .post
        .rng
        .get("ai")
        .expect("ai counter must be tracked");
    doctored.post.rng.insert("ai".to_string(), original_ai + 1);

    let report = diff_records(
        "synthetic-fixture-doctored",
        &script.seed,
        &java_records,
        &rust_records,
        &[],
    );

    assert_eq!(report.status, DivergenceStatus::Diverged);
    assert_eq!(
        report.matched_actions, 2,
        "NEOW and PATH records are undoctored and must still match"
    );

    let first = report
        .first_divergence
        .expect("must report a first_divergence");
    assert_eq!(first.idx, 2);
    // RNG counters are diffed first in canonical order, so the `ai` counter
    // diff must be the reported path, not `post.player.hp` (also diffed,
    // but demoted to `secondary` since it's the same divergent record).
    assert_eq!(
        first.path, "post.rng.ai",
        "rng counters must be diagnosed first: {first:?}"
    );
    assert_eq!(first.java, serde_json::json!(original_ai + 1));
    assert_eq!(first.rust, serde_json::json!(original_ai));
    assert_eq!(
        first.rng_at_divergence.java.get("ai"),
        Some(&(original_ai + 1))
    );
    assert_eq!(first.rng_at_divergence.rust.get("ai"), Some(&original_ai));

    // The hp diff at the same record must still surface, just secondary.
    let hp_secondary = report
        .secondary
        .iter()
        .find(|diff| diff.path == "post.player.hp")
        .expect("hp diff must appear in secondary diffs for the same divergent record");
    assert_eq!(hp_secondary.java, serde_json::json!(original_hp + 1));
    assert_eq!(hp_secondary.rust, serde_json::json!(original_hp));
}

#[test]
fn truncated_trace_is_never_reported_as_match() {
    // A "java" trace that stopped early (e.g. the golden mint truncated,
    // or a script only partially replayed on one side) must never be
    // silently reported as "match" even though every record it does have
    // matches exactly.
    let script = tiny_fixture_script();
    let rust_records =
        crate::trace::replay_script(&script).expect("fixture script must replay cleanly");
    assert!(rust_records.len() > 1);

    let java_records = rust_records[..1].to_vec();
    let report = diff_records(
        "synthetic-fixture-truncated",
        &script.seed,
        &java_records,
        &rust_records,
        &[],
    );

    assert_eq!(
        report.status,
        DivergenceStatus::Diverged,
        "a length mismatch must never report match: {report:?}"
    );
    assert_eq!(report.matched_actions, 1);
    assert_eq!(report.total_actions, rust_records.len() as u64);
    assert!(report.first_divergence.is_some());
}

/// If `data/traces/java/*.jsonl` goldens exist (minted by U04/U06), replay
/// each against its matching `data/traces/scripts/<stem>.json` and diff —
/// per T5, "part of the lib suite". Today the directory doesn't exist yet,
/// so this must no-op cleanly rather than fail.
#[test]
fn replays_committed_goldens_if_any_exist() {
    let repo_root = repo_root();
    let goldens_dir = repo_root.join("data/traces/java");
    let scripts_dir = repo_root.join("data/traces/scripts");

    let Ok(entries) = fs::read_dir(&goldens_dir) else {
        // No goldens minted yet (U04/U06 not landed) — graceful no-op.
        return;
    };

    let mut checked = 0usize;
    for entry in entries {
        let entry = entry.expect("readable dir entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("golden filename must have a stem")
            .to_string();
        let script_path = scripts_dir.join(format!("{stem}.json"));
        assert!(
            script_path.exists(),
            "golden {path:?} has no matching script at {script_path:?}"
        );

        let script_text = fs::read_to_string(&script_path).expect("script must be readable");
        let script: ActionScript = serde_json::from_str(&script_text).expect("script must parse");
        script
            .check_version()
            .expect("script version must be supported");

        let java_text = fs::read_to_string(&path).expect("golden must be readable");
        let mut lines = java_text.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().expect("golden must have a header line");
        let header: crate::trace::TraceHeader =
            serde_json::from_str(header_line).expect("golden header must parse");
        header
            .check_version()
            .expect("golden header version must be supported");
        let java_records: Vec<crate::trace::TraceRecord> = lines
            // Skip meta records: header/end sentinels carry a `kind`, data records do not.
            .filter(|line| {
                serde_json::from_str::<serde_json::Value>(line)
                    .ok()
                    .and_then(|v| v.get("kind").cloned())
                    .is_none()
            })
            .map(|line| serde_json::from_str(line).expect("golden record must parse"))
            .collect();

        let rust_records = crate::trace::replay_script(&script)
            .unwrap_or_else(|err| panic!("script {stem} failed to replay: {err}"));

        // Divergences against real goldens are U08+'s backlog (per U06's
        // acceptance criteria) — this test only asserts the pipeline runs
        // end-to-end without panicking, not that it matches yet.
        let _report = diff_records(&stem, &script.seed, &java_records, &rust_records, &[]);
        checked += 1;
    }

    let _ = checked; // informational; zero is a valid outcome today
}

fn repo_root() -> std::path::PathBuf {
    // src/tests/test_trace_oracle.rs -> packages/engine-rs/src/tests -> ../../../.. = repo root
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR must have packages/ and repo root ancestors")
        .to_path_buf()
}
