#[cfg(test)]
mod enemy_ai_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/exordium/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/city/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/ending/*.java

    use crate::enemies::*;
    use crate::combat_types::{mfx, Intent};
    use crate::status_ids::sid;
    use crate::enemies::move_ids;
    use crate::map::{DungeonMap, MapNode, RoomType};
    use crate::run::{RunAction, RunEngine, RunPhase};
    use crate::state::EnemyCombatState;
    use crate::tests::support::run_engine;
    use crate::tests::support::TEST_SEED;

    fn make(id: &str, hp: i32) -> EnemyCombatState {
        create_enemy(id, hp, hp)
    }

    fn roll_times(enemy: &mut EnemyCombatState, times: usize) {
        // Default num=0 keeps deterministic enemies (Cultist, Slime, etc.) on their
        // canonical branch; probabilistic enemies (JawWorm, ...) should use
        // `roll_with_num` below to assert specific Java branches.
        for _ in 0..times {
            roll_next_move_with_num(enemy, 0);
        }
    }

    fn roll_with_num(enemy: &mut EnemyCombatState, num: i32) {
        roll_next_move_with_num(enemy, num);
    }

    fn expect_move(
        enemy: &EnemyCombatState,
        move_id: i32,
        damage: i32,
        hits: i32,
        block: i32,
        effects: &[(u8, i16)],
    ) {
        assert_eq!(enemy.move_id, move_id, "{} move_id", enemy.id);
        assert_eq!(enemy.move_damage(), damage, "{} move_damage", enemy.id);
        assert_eq!(enemy.move_hits(), hits, "{} move_hits", enemy.id);
        assert_eq!(enemy.move_block(), block, "{} move_block", enemy.id);
        assert_eq!(enemy.move_effects.len(), effects.len(), "{} effect count", enemy.id);
        for (key, value) in effects {
            assert_eq!(
                enemy.effect(*key),
                Some(*value),
                "{} effect {:?}",
                enemy.id,
                key
            );
        }
    }

    fn expect_status(enemy: &EnemyCombatState, key: crate::ids::StatusId, value: i32) {
        let name = crate::status_ids::status_name(key);
        assert_eq!(enemy.entity.status(key), value, "{} status {}", enemy.id, name);
    }

    fn expect_one_of(enemy: &EnemyCombatState, allowed: &[i32]) {
        assert!(
            allowed.contains(&enemy.move_id),
            "{} move_id {} was not one of {:?}",
            enemy.id,
            enemy.move_id,
            allowed
        );
    }

    fn forced_map(room_type: RoomType) -> DungeonMap {
        DungeonMap {
            rows: vec![vec![MapNode {
                x: 0,
                y: 0,
                room_type,
                has_edges: true,
                edges: Vec::new(),
                parents: Vec::new(),
                has_emerald_key: false,
            }]],
            height: 1,
            width: 1,
        }
    }

    fn forced_run_engine(act: i32, ascension: i32, room_type: RoomType, floor_before: i32) -> RunEngine {
        let mut engine = run_engine(TEST_SEED, ascension);
        engine.map = forced_map(room_type);
        engine.run_state.act = act;
        engine.run_state.floor = floor_before;
        engine.run_state.map_x = -1;
        engine.run_state.map_y = -1;
        engine.phase = RunPhase::MapChoice;
        engine
    }

    fn enter_forced_combat(act: i32, ascension: i32, room_type: RoomType, floor_before: i32) -> RunEngine {
        let mut engine = forced_run_engine(act, ascension, room_type, floor_before);
        let (reward, done) = engine.step(&RunAction::ChoosePath(0));
        assert!(!done, "forced combat entry should not end the run");
        assert!(reward >= 0.0);
        assert_eq!(engine.current_phase(), RunPhase::Combat);
        engine
    }

    #[test]
    fn act1_initial_states_match_java() {
        let e = make("JawWorm", 44);
        expect_move(&e, move_ids::JW_CHOMP, 11, 1, 0, &[]);

        let e = make("Cultist", 50);
        expect_move(&e, move_ids::CULT_INCANTATION, 0, 0, 0, &[(mfx::RITUAL, 3)]);

        let e = make("FungiBeast", 22);
        expect_move(&e, move_ids::FB_BITE, 6, 1, 0, &[]);
        expect_status(&e, sid::SPORE_CLOUD, 2);

        let e = make("RedLouse", 12);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        expect_status(&e, sid::CURL_UP, 5);

        let e = make("GreenLouse", 14);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        expect_status(&e, sid::CURL_UP, 5);

        let e = make("SlaverBlue", 46);
        expect_move(&e, move_ids::BS_STAB, 12, 1, 0, &[]);

        let e = make("SlaverRed", 46);
        expect_move(&e, move_ids::RS_STAB, 13, 1, 0, &[]);

        let e = make("AcidSlime_S", 8);
        expect_move(&e, move_ids::AS_S_TACKLE, 3, 1, 0, &[]);

        let e = make("AcidSlime_M", 28);
        expect_one_of(&e, &[move_ids::AS_CORROSIVE_SPIT, move_ids::AS_TACKLE, move_ids::AS_LICK]);
        match e.move_id {
            x if x == move_ids::AS_CORROSIVE_SPIT => {
                expect_move(&e, move_ids::AS_CORROSIVE_SPIT, 7, 1, 0, &[(mfx::SLIMED, 1)])
            }
            x if x == move_ids::AS_TACKLE => expect_move(&e, move_ids::AS_TACKLE, 10, 1, 0, &[]),
            _ => expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 1)]),
        }

        let e = make("AcidSlime_L", 65);
        expect_one_of(&e, &[move_ids::AS_CORROSIVE_SPIT, move_ids::AS_TACKLE, move_ids::AS_LICK]);
        match e.move_id {
            x if x == move_ids::AS_CORROSIVE_SPIT => {
                expect_move(&e, move_ids::AS_CORROSIVE_SPIT, 11, 1, 0, &[(mfx::SLIMED, 2)])
            }
            x if x == move_ids::AS_TACKLE => expect_move(&e, move_ids::AS_TACKLE, 16, 1, 0, &[]),
            _ => expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 2)]),
        }

        let e = make("SpikeSlime_S", 11);
        expect_move(&e, move_ids::SS_TACKLE, 5, 1, 0, &[]);

        let e = make("SpikeSlime_M", 28);
        // Source: reference/extracted/methods/monster/SpikeSlime_M.java
        // (`takeTurn` case TACKLE adds one Slimed to the discard pile).
        expect_move(&e, move_ids::SS_TACKLE, 8, 1, 0, &[(mfx::SLIMED, 1)]);

        let e = make("SpikeSlime_L", 65);
        // Source: reference/extracted/methods/monster/SpikeSlime_L.java
        // (`takeTurn` case FLAME_TACKLE adds two Slimed).
        expect_move(&e, move_ids::SS_TACKLE, 16, 1, 0, &[(mfx::SLIMED, 2)]);

        let e = make("Looter", 44);
        expect_move(&e, move_ids::LOOTER_MUG, 10, 1, 0, &[]);

        let e = make("GremlinFat", 18);
        // Source: reference/extracted/methods/monster/GremlinFat.java.
        expect_move(&e, move_ids::GREMLIN_FAT_SMASH, 4, 1, 0, &[(mfx::WEAK, 1)]);

        let e = make("GremlinThief", 13);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 9, 1, 0, &[]);

        let e = make("GremlinWarrior", 11);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 4, 1, 0, &[]);

        let e = make("GremlinWizard", 20);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);

        let e = make("GremlinTsundere", 13);
        // Source: reference/extracted/methods/monster/GremlinTsundere.java.
        expect_move(&e, move_ids::GREMLIN_TSUNDERE_PROTECT, 0, 0, 0,
            &[(mfx::BLOCK_RANDOM_OTHER, 7)]);

        let e = make("GremlinNob", 106);
        // Source: reference/extracted/methods/monster/GremlinNob.java: Bellow
        // applies Enrage during takeTurn, not during construction.
        expect_move(&e, move_ids::NOB_BELLOW, 0, 0, 0, &[(mfx::ENRAGE, 2)]);
        expect_status(&e, sid::ENRAGE, 0);

        let e = make("Lagavulin", 109);
        expect_move(&e, move_ids::LAGA_SLEEP, 0, 0, 0, &[]);
        expect_status(&e, sid::METALLICIZE, 8);
        expect_status(&e, sid::COUNT, 0);
        assert_eq!(e.entity.block, 8);

        let e = make("Sentry", 38);
        // Source: reference/extracted/methods/monster/Sentry.java: Bolt (3)
        // adds Dazed; Beam (4) is the damaging move.
        expect_move(&e, move_ids::SENTRY_BOLT, 0, 0, 0, &[(mfx::DAZE, 2)]);
        expect_status(&e, sid::ARTIFACT, 1);
    }

    #[test]
    fn act1_patterns_match_java() {
        // Source: reference/extracted/methods/monster/JawWorm.java (`getMove`).
        let mut e = make("JawWorm", 44);
        roll_with_num(&mut e, 30); // CHOMP + 25..54 -> THRASH
        expect_move(&e, move_ids::JW_THRASH, 7, 1, 5, &[]);
        roll_with_num(&mut e, 80); // THRASH + >=55 -> BELLOW
        expect_move(&e, move_ids::JW_BELLOW, 0, 0, 6, &[(mfx::STRENGTH, 3)]);
        roll_with_num(&mut e, 30);
        roll_with_num(&mut e, 10); // THRASH + <25 -> CHOMP
        expect_move(&e, move_ids::JW_CHOMP, 11, 1, 0, &[]);

        let mut e = make("Cultist", 50);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CULT_DARK_STRIKE, 6, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CULT_DARK_STRIKE, 6, 1, 0, &[]);

        let mut e = make("FungiBeast", 22);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::FB_BITE, 6, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::FB_GROW, 0, 0, 0, &[(mfx::STRENGTH, 3)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::FB_BITE, 6, 1, 0, &[]);

        let mut e = make("RedLouse", 12);
        // LouseNormal.java: initial num=25 selects Bite; after two Bites the
        // >=25 branch forces Grow.
        roll_initial_move_with_num_and_rng(
            &mut e, 25, &mut crate::seed::StsRandom::new(0));
        roll_with_num(&mut e, 25);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        roll_with_num(&mut e, 25);
        expect_move(&e, move_ids::LOUSE_GROW, 0, 0, 0, &[(mfx::STRENGTH, 3)]);

        let mut e = make("GreenLouse", 14);
        // LouseDefensive.java has the same history rule, forcing Web instead.
        roll_initial_move_with_num_and_rng(
            &mut e, 25, &mut crate::seed::StsRandom::new(0));
        roll_with_num(&mut e, 25);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        roll_with_num(&mut e, 25);
        expect_move(&e, move_ids::LOUSE_SPIT_WEB, 0, 0, 0, &[(mfx::WEAK, 2)]);

        let mut e = make("SlaverBlue", 46);
        // SlaverBlue.java: >=40 selects Stab until two consecutive Stabs force Rake.
        roll_initial_move_with_num_and_rng(
            &mut e, 40, &mut crate::seed::StsRandom::new(0));
        roll_with_num(&mut e, 40);
        expect_move(&e, move_ids::BS_STAB, 12, 1, 0, &[]);
        roll_with_num(&mut e, 40);
        expect_move(&e, move_ids::BS_RAKE, 7, 1, 0, &[(mfx::WEAK, 1)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::BS_RAKE, 7, 1, 0, &[(mfx::WEAK, 1)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::BS_STAB, 12, 1, 0, &[]);

        let mut e = make("SlaverRed", 46);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::RS_STAB, 13, 1, 0, &[]);
        roll_with_num(&mut e, 75);
        expect_move(&e, move_ids::RS_ENTANGLE, 0, 0, 0, &[(mfx::ENTANGLE, 1)]);
        roll_with_num(&mut e, 60);
        expect_move(&e, move_ids::RS_STAB, 13, 1, 0, &[]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::RS_SCRAPE, 8, 1, 0, &[(mfx::VULNERABLE, 1)]);

        let mut e = make("AcidSlime_S", 8);
        advance_acid_slime_s_after_turn(&mut e);
        expect_move(&e, move_ids::AS_S_LICK, 0, 0, 0, &[(mfx::WEAK, 1)]);
        advance_acid_slime_s_after_turn(&mut e);
        expect_move(&e, move_ids::AS_S_TACKLE, 3, 1, 0, &[]);

        let mut e = make("AcidSlime_M", 28);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        expect_move(&e, move_ids::AS_CORROSIVE_SPIT, 7, 1, 0, &[(mfx::SLIMED, 1)]);
        roll_with_num(&mut e, 40);
        expect_move(&e, move_ids::AS_TACKLE, 10, 1, 0, &[]);
        roll_with_num(&mut e, 70);
        expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 1)]);

        let mut e = make("AcidSlime_L", 65);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        expect_move(&e, move_ids::AS_CORROSIVE_SPIT, 11, 1, 0, &[(mfx::SLIMED, 2)]);
        roll_with_num(&mut e, 40);
        expect_move(&e, move_ids::AS_TACKLE, 16, 1, 0, &[]);
        roll_with_num(&mut e, 70);
        expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 2)]);

        let mut e = make("SpikeSlime_M", 28);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        expect_move(&e, move_ids::SS_TACKLE, 8, 1, 0, &[(mfx::SLIMED, 1)]);
        roll_with_num(&mut e, 30);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 1)]);
        roll_with_num(&mut e, 30);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 1)]);

        let mut e = make("SpikeSlime_L", 65);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        expect_move(&e, move_ids::SS_TACKLE, 16, 1, 0, &[(mfx::SLIMED, 2)]);
        roll_with_num(&mut e, 30);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 2)]);
        roll_with_num(&mut e, 30);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 2)]);

        let mut e = make("Looter", 44);
        let seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let _ = rng.random_float();
            rng.random_float() < 0.5
        }).unwrap();
        let mut rng = crate::seed::StsRandom::new(seed);
        act1::advance_looter_after_turn(&mut e, &mut rng);
        expect_move(&e, move_ids::LOOTER_MUG, 10, 1, 0, &[]);
        act1::advance_looter_after_turn(&mut e, &mut rng);
        expect_move(&e, move_ids::LOOTER_SMOKE_BOMB, 0, 0, 6, &[]);
        act1::advance_looter_after_turn(&mut e, &mut rng);
        expect_move(&e, move_ids::LOOTER_ESCAPE, 0, 0, 0, &[]);
        assert!(!e.is_escaping);
        act1::advance_looter_after_turn(&mut e, &mut rng);
        assert!(e.is_escaping);

        let mut e = make("GremlinFat", 18);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_FAT_SMASH, 4, 1, 0, &[(mfx::WEAK, 1)]);
        let mut e = make("GremlinWizard", 20);
        // Source: reference/extracted/methods/monster/GremlinWizard.java.
        act1::advance_gremlin_wizard_after_turn(&mut e);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);
        act1::advance_gremlin_wizard_after_turn(&mut e);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 25, 1, 0, &[]);
        act1::advance_gremlin_wizard_after_turn(&mut e);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);
        let mut e = make("GremlinTsundere", 13);
        act1::advance_gremlin_tsundere_after_turn(&mut e, 2);
        expect_move(&e, move_ids::GREMLIN_TSUNDERE_PROTECT, 0, 0, 0,
            &[(mfx::BLOCK_RANDOM_OTHER, 7)]);
        act1::advance_gremlin_tsundere_after_turn(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_TSUNDERE_BASH, 6, 1, 0, &[]);
        act1::advance_gremlin_tsundere_after_turn(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_TSUNDERE_BASH, 6, 1, 0, &[]);

        let mut e = make("GremlinNob", 106);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(1));
        roll_with_num(&mut e, 33);
        expect_move(&e, move_ids::NOB_RUSH, 14, 1, 0, &[]);
        e.move_id = move_ids::NOB_RUSH;
        e.move_history = vec![move_ids::NOB_RUSH, move_ids::NOB_RUSH];
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::NOB_SKULL_BASH, 6, 1, 0, &[(mfx::VULNERABLE, 2)]);

        let mut e = make("Lagavulin", 109);
        let mut laga_rng = crate::seed::StsRandom::new(1);
        act1::advance_lagavulin_after_turn(&mut e, &mut laga_rng);
        expect_move(&e, move_ids::LAGA_SLEEP, 0, 0, 0, &[]);
        expect_status(&e, sid::COUNT, 1);
        act1::advance_lagavulin_after_turn(&mut e, &mut laga_rng);
        expect_status(&e, sid::COUNT, 2);
        act1::advance_lagavulin_after_turn(&mut e, &mut laga_rng);
        expect_move(&e, move_ids::LAGA_ATTACK, 18, 1, 0, &[]);
        assert_eq!(laga_rng.counter, 2);
        lagavulin_wake_up(&mut e);
        expect_move(&e, move_ids::LAGA_STUN, 0, 0, 0, &[]);
        expect_status(&e, sid::SLEEP_TURNS, 0);

        let mut e = make("Sentry", 38);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(1));
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SENTRY_BEAM, 9, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SENTRY_BOLT, 0, 0, 0, &[(mfx::DAZE, 2)]);
    }

    #[test]
    fn act2_initial_states_match_java() {
        let e = make("Chosen", 95);
        expect_move(&e, move_ids::CHOSEN_POKE, 5, 2, 0, &[]);

        let e = make("Mugger", 48);
        expect_move(&e, move_ids::MUGGER_MUG, 10, 1, 0, &[]);
        expect_status(&e, sid::STARTING_DMG, 10);
        expect_status(&e, sid::STR_AMT, 16);
        expect_status(&e, sid::BLOCK_AMT, 11);
        expect_status(&e, sid::TURN_COUNT, 15);

        let e = make("Byrd", 25);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);
        expect_status(&e, sid::FLIGHT, 3);

        let e = make("Shelled Parasite", 68);
        expect_move(&e, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);
        expect_status(&e, sid::PLATED_ARMOR, 14);
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::STARTING_DMG, 6);
        expect_status(&e, sid::STR_AMT, 18);
        expect_status(&e, sid::BLOCK_AMT, 10);
        assert_eq!(e.entity.block, 14);

        let e = make("SnakePlant", 75);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);
        expect_status(&e, sid::STARTING_DMG, 7);
        expect_status(&e, sid::MALLEABLE, 3);
        expect_status(&e, sid::BLOCK_AMT, 3);

        let e = make("Centurion", 76);
        expect_move(&e, move_ids::CENT_SLASH, 12, 1, 0, &[]);

        let e = make("Mystic", 48);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[(mfx::FRAIL, 2)]);

        let e = make("Healer", 48);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[(mfx::FRAIL, 2)]);
        expect_status(&e, sid::STARTING_DMG, 8);
        expect_status(&e, sid::STR_AMT, 2);
        expect_status(&e, sid::BLOCK_AMT, 16);
        expect_status(&e, sid::HIGH_ASCENSION_AI, 0);

        let e = make("BookOfStabbing", 160);
        expect_move(&e, move_ids::BOOK_STAB, 6, 1, 0, &[]);
        expect_status(&e, sid::PAINFUL_STABS, 1);
        expect_status(&e, sid::STAB_COUNT, 0);

        let e = make("GremlinLeader", 140);
        expect_move(&e, move_ids::GL_RALLY, 0, 0, 0, &[]);
        expect_status(&e, sid::STR_AMT, 3);
        expect_status(&e, sid::BLOCK_AMT, 6);
        expect_status(&e, sid::COUNT, 0);

        let e = make("SlaverBoss", 60);
        expect_move(&e, move_ids::TASK_SCOURING_WHIP, 7, 1, 0, &[(mfx::WOUND, 1)]);
        assert!(matches!(e.intent, crate::combat_types::Intent::AttackDebuff { .. }));
        expect_status(&e, sid::STARTING_DMG, 7);
        expect_status(&e, sid::BLOCK_AMT, 1);

        let e = make("SphericGuardian", 20);
        expect_move(&e, move_ids::SPHER_INITIAL_BLOCK, 0, 0, 25, &[]);
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::FIRST_TURN, 1);
        expect_status(&e, sid::STARTING_DMG, 10);
        expect_status(&e, sid::BLOCK_AMT, 25);
        expect_status(&e, sid::BARRICADE, 1);
        expect_status(&e, sid::ARTIFACT, 3);
        assert_eq!(e.entity.block, 40);

        let e = make("Snecko", 114);
        expect_move(&e, move_ids::SNECKO_GLARE, 0, 0, 0, &[(mfx::CONFUSED, 1)]);
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::STARTING_DMG, 15);
        expect_status(&e, sid::STR_AMT, 8);

        let e = make("BanditBear", 40);
        expect_move(&e, move_ids::BEAR_HUG, 0, 0, 0, &[(mfx::DEX_DOWN, 2)]);

        let e = make("BanditLeader", 50);
        expect_move(&e, move_ids::BANDIT_MOCK, 0, 0, 0, &[]);

        let e = make("BanditChild", 30);
        expect_move(&e, move_ids::POINTY_STAB, 5, 2, 0, &[]);

        let e = make("BronzeAutomaton", 300);
        expect_move(&e, move_ids::BA_SPAWN_ORBS, 0, 0, 0, &[]);

        let e = make("BronzeOrb", 35);
        expect_move(&e, move_ids::BO_STASIS, 0, 0, 0, &[(mfx::STASIS, 1)]);

        let e = make("TorchHead", 35);
        expect_move(&e, move_ids::TORCH_TACKLE, 7, 1, 0, &[]);
    }

    #[test]
    fn act2_patterns_match_java() {
        let mut e = make("Chosen", 95);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::CHOSEN_POKE, 5, 2, 0, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::CHOSEN_HEX, 0, 0, 0, &[(mfx::HEX, 1)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::CHOSEN_DEBILITATE, 10, 1, 0, &[(mfx::VULNERABLE, 2)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::CHOSEN_ZAP, 18, 1, 0, &[]);

        let mut e = make("Mugger", 48);
        let smoke_seed = (1..10_000).find(|&seed| {
            let mut rng = crate::seed::StsRandom::new(seed);
            let _ = rng.random(2);
            let _ = rng.random(2);
            let _ = rng.random_float();
            rng.random_float() < 0.5
        }).unwrap();
        let mut mugger_rng = crate::seed::StsRandom::new(smoke_seed);
        act2::advance_mugger_after_turn(&mut e, &mut mugger_rng);
        expect_move(&e, move_ids::MUGGER_MUG, 10, 1, 0, &[]);
        assert_eq!(mugger_rng.counter, 1);
        act2::advance_mugger_after_turn(&mut e, &mut mugger_rng);
        expect_move(&e, move_ids::MUGGER_SMOKE_BOMB, 0, 0, 11, &[]);
        assert_eq!(mugger_rng.counter, 4);
        act2::advance_mugger_after_turn(&mut e, &mut mugger_rng);
        expect_move(&e, move_ids::MUGGER_ESCAPE, 0, 0, 0, &[]);
        assert!(!e.is_escaping);
        act2::advance_mugger_after_turn(&mut e, &mut mugger_rng);
        assert!(e.is_escaping);

        let mut e = make("Byrd", 25);
        e.entity.set_status(sid::FIRST_MOVE, 0);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);
        roll_with_num(&mut e, 60);
        expect_move(&e, move_ids::BYRD_SWOOP, 12, 1, 0, &[]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);

        // Source: reference/extracted/methods/monster/ShelledParasite.java.
        let true_seed = (1..10_000).find(|&seed|
            crate::seed::StsRandom::new(seed).random_boolean()).unwrap();
        let mut e = make("Shelled Parasite", 68);
        let mut true_rng = crate::seed::StsRandom::new(true_seed);
        roll_initial_move_with_num_and_rng(&mut e, 99, &mut true_rng);
        expect_move(&e, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);
        assert_eq!(true_rng.counter, 1,
            "pre-A17 opener consumes aiRng.randomBoolean after rollMove num");

        let false_seed = (1..10_000).find(|&seed|
            !crate::seed::StsRandom::new(seed).random_boolean()).unwrap();
        let mut life = make("Shelled Parasite", 68);
        let mut false_rng = crate::seed::StsRandom::new(false_seed);
        roll_initial_move_with_num_and_rng(&mut life, 0, &mut false_rng);
        expect_move(&life, move_ids::SP_LIFE_SUCK, 10, 1, 0,
            &[(mfx::HEAL, 10)]);
        assert_eq!(false_rng.counter, 1);

        let mut a17 = make("Shelled Parasite", 70);
        a17.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        let mut no_boolean = crate::seed::StsRandom::new(0);
        roll_initial_move_with_num_and_rng(&mut a17, 99, &mut no_boolean);
        expect_move(&a17, move_ids::SP_FELL, 18, 1, 0, &[(mfx::FRAIL, 2)]);
        assert_eq!(no_boolean.counter, 0,
            "A17 forced Fell consumes no conditional boolean");

        let reroll_seed = (1..10_000).find(|&seed| {
            let roll = crate::seed::StsRandom::new(seed).random_range(20, 99);
            roll < 60
        }).unwrap();
        a17.move_id = move_ids::SP_FELL;
        a17.move_history.clear();
        let mut reroll_rng = crate::seed::StsRandom::new(reroll_seed);
        roll_next_move_with_num_and_rng(&mut a17, 0, &mut reroll_rng);
        expect_move(&a17, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);
        assert_eq!(reroll_rng.counter, 1,
            "repeated Fell rerolls with aiRng.random(20, 99)");

        a17.move_id = move_ids::SP_DOUBLE_STRIKE;
        a17.move_history = vec![move_ids::SP_DOUBLE_STRIKE];
        roll_with_num(&mut a17, 20);
        expect_move(&a17, move_ids::SP_LIFE_SUCK, 10, 1, 0,
            &[(mfx::HEAL, 10)]);

        a17.move_id = move_ids::SP_LIFE_SUCK;
        a17.move_history = vec![move_ids::SP_LIFE_SUCK];
        roll_with_num(&mut a17, 60);
        expect_move(&a17, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);

        // Source: reference/extracted/methods/monster/SnakePlant.java.
        let mut e = make("SnakePlant", 75);
        roll_initial_move_with_num_and_rng(
            &mut e, 64, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);
        roll_with_num(&mut e, 64);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);
        roll_with_num(&mut e, 64);
        expect_move(&e, move_ids::SNAKE_SPORES, 0, 0, 0, &[(mfx::WEAK, 2), (mfx::FRAIL, 2)]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);

        let mut pre_a17 = make("SnakePlant", 75);
        pre_a17.move_id = move_ids::SNAKE_CHOMP;
        pre_a17.move_history = vec![move_ids::SNAKE_SPORES];
        roll_with_num(&mut pre_a17, 99);
        expect_move(&pre_a17, move_ids::SNAKE_SPORES, 0, 0, 0,
            &[(mfx::FRAIL, 2), (mfx::WEAK, 2)]);

        let mut a17 = make("SnakePlant", 78);
        a17.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        a17.entity.set_status(sid::STARTING_DMG, 8);
        a17.move_id = move_ids::SNAKE_CHOMP;
        a17.move_history = vec![move_ids::SNAKE_SPORES];
        roll_with_num(&mut a17, 99);
        expect_move(&a17, move_ids::SNAKE_CHOMP, 8, 3, 0, &[]);

        // Centurion.java: low rolls Slash until two consecutive Slashes, then
        // Protects one random ally (or uses Fury when alone).
        let mut e = make("Centurion", 76);
        roll_initial_move_with_num_and_rng(
            &mut e, 64, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::CENT_SLASH, 12, 1, 0, &[]);
        roll_with_num(&mut e, 64);
        expect_move(&e, move_ids::CENT_SLASH, 12, 1, 0, &[]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::CENT_PROTECT, 0, 0, 0,
            &[(mfx::BLOCK_RANDOM_OTHER, 15)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::CENT_SLASH, 12, 1, 0, &[]);

        // Healer.java prioritizes group healing, then a >=40 attack roll,
        // then group Strength. The legacy Mystic ID remains an alias only.
        let mut e = make("Healer", 48);
        e.entity.set_status(sid::COUNT, 16);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::MYSTIC_HEAL, 0, 0, 0, &[(mfx::HEAL_ALL, 16)]);
        e.entity.set_status(sid::COUNT, 0);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[(mfx::FRAIL, 2)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::MYSTIC_BUFF, 0, 0, 0,
            &[(mfx::STRENGTH, 2), (mfx::STRENGTH_ALL_ALLIES, 2)]);

        let mut e = make("BookOfStabbing", 160);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::BOOK_STAB, 6, 1, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 1);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::BOOK_STAB, 6, 2, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 2);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::BOOK_BIG_STAB, 21, 1, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 2);
        roll_with_num(&mut e, 14);
        expect_move(&e, move_ids::BOOK_STAB, 6, 3, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 3);

        // A18 increments stabCount even when BIG_STAB is selected.
        let mut a18 = make("BookOfStabbing", 168);
        a18.entity.set_status(sid::STARTING_DMG, 7);
        a18.entity.set_status(sid::STR_AMT, 24);
        a18.entity.set_status(sid::BLOCK_AMT, 1);
        roll_initial_move_with_num_and_rng(
            &mut a18, 14, &mut crate::seed::StsRandom::new(0));
        expect_move(&a18, move_ids::BOOK_BIG_STAB, 24, 1, 0, &[]);
        expect_status(&a18, sid::STAB_COUNT, 1);
        roll_with_num(&mut a18, 14);
        expect_move(&a18, move_ids::BOOK_STAB, 7, 2, 0, &[]);
        roll_with_num(&mut a18, 99);
        expect_move(&a18, move_ids::BOOK_STAB, 7, 3, 0, &[]);
        roll_with_num(&mut a18, 99);
        expect_move(&a18, move_ids::BOOK_BIG_STAB, 24, 1, 0, &[]);
        expect_status(&a18, sid::STAB_COUNT, 4);

        let mut e = make("GremlinLeader", 140);
        e.entity.set_status(sid::COUNT, 0);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::GL_STAB, 6, 3, 0, &[]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::GL_RALLY, 0, 0, 0, &[]);

        e.entity.set_status(sid::COUNT, 2);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::GL_ENCOURAGE, 0, 0, 0,
            &[(mfx::STRENGTH, 3), (mfx::STRENGTH_ALL_ALLIES, 3),
                (mfx::BLOCK_ALL_ALLIES, 6)]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::GL_STAB, 6, 3, 0, &[]);

        // Source: reference/extracted/methods/monster/Taskmaster.java.
        let mut e = make("SlaverBoss", 60);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::TASK_SCOURING_WHIP, 7, 1, 0, &[(mfx::WOUND, 1)]);

        // Source: reference/extracted/methods/monster/SphericGuardian.java.
        let mut e = make("SphericGuardian", 20);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::SPHER_INITIAL_BLOCK, 0, 0, 25, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SPHER_FRAIL_ATTACK, 10, 1, 0, &[(mfx::FRAIL, 5)]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SPHER_BIG_ATTACK, 10, 2, 0, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SPHER_BLOCK_ATTACK, 10, 1, 15, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SPHER_BIG_ATTACK, 10, 2, 0, &[]);

        // Source: reference/extracted/methods/monster/Snecko.java.
        let mut e = make("Snecko", 114);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::SNECKO_GLARE, 0, 0, 0, &[(mfx::CONFUSED, 1)]);
        roll_with_num(&mut e, 39);
        expect_move(&e, move_ids::SNECKO_TAIL, 8, 1, 0, &[(mfx::VULNERABLE, 2)]);
        roll_with_num(&mut e, 40);
        expect_move(&e, move_ids::SNECKO_BITE, 15, 1, 0, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SNECKO_BITE, 15, 1, 0, &[]);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SNECKO_TAIL, 8, 1, 0, &[(mfx::VULNERABLE, 2)]);

        let mut a17 = make("Snecko", 120);
        a17.entity.set_status(sid::FIRST_MOVE, 0);
        a17.entity.set_status(sid::STARTING_DMG, 18);
        a17.entity.set_status(sid::STR_AMT, 10);
        a17.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        roll_with_num(&mut a17, 0);
        expect_move(&a17, move_ids::SNECKO_TAIL, 10, 1, 0,
            &[(mfx::WEAK, 2), (mfx::VULNERABLE, 2)]);

        // Source: reference/extracted/methods/monster/BanditBear.java.
        // Only the opener uses getMove; takeTurn then installs Lunge and Maul
        // directly, alternating those two attacks without another Bear Hug.
        let mut e = make("BanditBear", 40);
        act2::advance_bear_after_turn(&mut e);
        expect_move(&e, move_ids::BEAR_LUNGE, 9, 1, 9, &[]);
        act2::advance_bear_after_turn(&mut e);
        expect_move(&e, move_ids::BEAR_MAUL, 18, 1, 0, &[]);
        act2::advance_bear_after_turn(&mut e);
        expect_move(&e, move_ids::BEAR_LUNGE, 9, 1, 9, &[]);

        let mut e = make("BanditLeader", 50);
        act2::advance_bandit_leader_after_turn(&mut e);
        expect_move(&e, move_ids::BANDIT_AGONIZE, 10, 1, 0, &[(mfx::WEAK, 2)]);
        act2::advance_bandit_leader_after_turn(&mut e);
        expect_move(&e, move_ids::BANDIT_CROSS_SLASH, 15, 1, 0, &[]);
        act2::advance_bandit_leader_after_turn(&mut e);
        expect_move(&e, move_ids::BANDIT_AGONIZE, 10, 1, 0, &[(mfx::WEAK, 2)]);

        let mut e = make("BanditChild", 30);
        act2::advance_bandit_pointy_after_turn(&mut e);
        expect_move(&e, move_ids::POINTY_STAB, 5, 2, 0, &[]);

        let mut e = make("BronzeAutomaton", 300);
        roll_initial_move_with_num_and_rng(
            &mut e, 0, &mut crate::seed::StsRandom::new(0));
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_FLAIL, 7, 2, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_BOOST, 0, 0, 9, &[(mfx::STRENGTH, 3)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_FLAIL, 7, 2, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_BOOST, 0, 0, 9, &[(mfx::STRENGTH, 3)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_HYPER_BEAM, 45, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_STUNNED, 0, 0, 0, &[]);

        let mut a19 = make("BronzeAutomaton", 320);
        a19.entity.set_status(sid::FLAIL_DMG, 8);
        a19.entity.set_status(sid::BEAM_DMG, 50);
        a19.entity.set_status(sid::STR_AMT, 4);
        a19.entity.set_status(sid::BLOCK_AMT, 12);
        a19.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        roll_initial_move_with_num_and_rng(
            &mut a19, 0, &mut crate::seed::StsRandom::new(0));
        for _ in 0..5 {
            roll_times(&mut a19, 1);
        }
        expect_move(&a19, move_ids::BA_HYPER_BEAM, 50, 1, 0, &[]);
        roll_times(&mut a19, 1);
        expect_move(&a19, move_ids::BA_BOOST, 0, 0, 12, &[(mfx::STRENGTH, 4)]);

        // Source: reference/extracted/methods/monster/BronzeOrb.java.
        let mut e = make("BronzeOrb", 52);
        roll_initial_move_with_num_and_rng(
            &mut e, 24, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::BO_BEAM, 8, 1, 0, &[]);
        roll_with_num(&mut e, 25);
        expect_move(&e, move_ids::BO_STASIS, 0, 0, 0, &[(mfx::STASIS, 1)]);
        expect_status(&e, sid::FIRST_MOVE, 1);
        roll_with_num(&mut e, 70);
        expect_move(&e, move_ids::BO_SUPPORT, 0, 0, 12, &[]);
        roll_with_num(&mut e, 70);
        expect_move(&e, move_ids::BO_SUPPORT, 0, 0, 12, &[]);
        roll_with_num(&mut e, 70);
        expect_move(&e, move_ids::BO_BEAM, 8, 1, 0, &[]);

        let e = make("TorchHead", 35);
        expect_move(&e, move_ids::TORCH_TACKLE, 7, 1, 0, &[]);
    }

    #[test]
    fn act3_initial_states_match_java() {
        let e = make("Darkling", 48);
        expect_one_of(&e, &[move_ids::DARK_HARDEN, move_ids::DARK_NIP]);
        match e.move_id {
            x if x == move_ids::DARK_HARDEN => expect_move(&e, move_ids::DARK_HARDEN, 0, 0, 12, &[]),
            _ => expect_move(&e, move_ids::DARK_NIP, 8, 1, 0, &[]),
        }
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::REGROW, 1);

        let e = make("OrbWalker", 90);
        expect_one_of(&e, &[move_ids::OW_LASER, move_ids::OW_CLAW]);
        match e.move_id {
            x if x == move_ids::OW_LASER => expect_move(&e, move_ids::OW_LASER, 10, 1, 0, &[(mfx::BURN_DRAW_DISCARD, 1)]),
            _ => expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]),
        }
        expect_status(&e, sid::STARTING_DMG, 10);
        expect_status(&e, sid::STR_AMT, 15);
        expect_status(&e, sid::GENERIC_STRENGTH_UP, 3);

        let e = make("Spiker", 42);
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);
        expect_status(&e, sid::THORNS, 3);
        expect_status(&e, sid::STARTING_DMG, 7);
        expect_status(&e, sid::COUNT, 0);

        let e = make("Repulsor", 29);
        expect_move(&e, move_ids::REPULSOR_DAZE, 0, 0, 0,
            &[(mfx::DAZE_DRAW, 2)]);
        assert!(matches!(e.intent, crate::combat_types::Intent::Debuff { .. }));
        expect_status(&e, sid::STARTING_DMG, 11);

        let e = make("Exploder", 30);
        expect_move(&e, move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        expect_status(&e, sid::TURN_COUNT, 0);
        expect_status(&e, sid::STARTING_DMG, 9);
        expect_status(&e, sid::EXPLOSIVE, 3);

        let e = make("WrithingMass", 160);
        expect_move(&e, move_ids::WM_MULTI_HIT, 7, 3, 0, &[]);
        expect_status(&e, sid::REACTIVE, 1);
        expect_status(&e, sid::MALLEABLE, 1);

        let e = make("Serpent", 170);
        expect_move(&e, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]);
        expect_status(&e, sid::STARTING_DMG, 16);
        expect_status(&e, sid::STR_AMT, 22);
        expect_status(&e, sid::BLOCK_AMT, 10);

        let e = make("Maw", 300);
        expect_move(&e, move_ids::MAW_ROAR, 0, 0, 0, &[(mfx::WEAK, 3), (mfx::FRAIL, 3)]);
        expect_status(&e, sid::TURN_COUNT, 1);
        expect_status(&e, sid::STARTING_DMG, 25);
        expect_status(&e, sid::STR_AMT, 3);
        expect_status(&e, sid::BLOCK_AMT, 3);

        let e = make("Transient", 999);
        expect_move(&e, move_ids::TRANSIENT_ATTACK, 30, 1, 0, &[]);
        expect_status(&e, sid::ATTACK_COUNT, 0);
        expect_status(&e, sid::STARTING_DMG, 30);
        expect_status(&e, sid::SHIFTING, 1);

        let e = make("GiantHead", 500);
        expect_move(&e, move_ids::GH_COUNT, 13, 1, 0, &[]);
        expect_status(&e, sid::COUNT, 5);
        expect_status(&e, sid::SLOW, 1);

        let e = make("Nemesis", 185);
        expect_move(&e, move_ids::NEM_TRI_ATTACK, 6, 3, 0, &[]);
        expect_status(&e, sid::SCYTHE_COOLDOWN, 0);
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::STARTING_DMG, 6);
        expect_status(&e, sid::BLOCK_AMT, 3);

        let e = make("Reptomancer", 190);
        expect_move(&e, move_ids::REPTO_SPAWN, 0, 0, 0, &[]);
        assert!(matches!(e.intent, crate::combat_types::Intent::Unknown));
        expect_status(&e, sid::FIRST_MOVE, 1);
        expect_status(&e, sid::STARTING_DMG, 13);
        expect_status(&e, sid::STR_AMT, 30);
        expect_status(&e, sid::BLOCK_AMT, 1);

        let e = make("SnakeDagger", 20);
        expect_move(&e, move_ids::SD_WOUND, 9, 1, 0, &[(mfx::WOUND, 1)]);
        expect_status(&e, sid::FIRST_MOVE, 1);
    }

    #[test]
    fn act3_patterns_match_java() {
        let mut e = make("Darkling", 48);
        roll_times(&mut e, 1);
        let first_move = e.move_id;
        assert!(matches!(first_move, move_ids::DARK_HARDEN | move_ids::DARK_NIP));
        roll_times(&mut e, 1);
        assert_ne!(e.move_id, first_move, "Darkling should not immediately repeat its opening move");
        e.entity.hp = 0;
        e.entity.set_status(sid::REBIRTH_PENDING, 1);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::DARK_REINCARNATE, 0, 0, 0, &[]);

        let mut e = make("OrbWalker", 90);
        roll_initial_move_with_num_and_rng(
            &mut e, 39, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]);
        roll_with_num(&mut e, 39);
        expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]);
        roll_with_num(&mut e, 39);
        expect_move(&e, move_ids::OW_LASER, 10, 1, 0,
            &[(mfx::BURN_DRAW_DISCARD, 1)]);

        let mut high = make("OrbWalker", 90);
        roll_initial_move_with_num_and_rng(
            &mut high, 40, &mut crate::seed::StsRandom::new(0));
        expect_move(&high, move_ids::OW_LASER, 10, 1, 0,
            &[(mfx::BURN_DRAW_DISCARD, 1)]);
        roll_with_num(&mut high, 40);
        expect_move(&high, move_ids::OW_LASER, 10, 1, 0,
            &[(mfx::BURN_DRAW_DISCARD, 1)]);
        roll_with_num(&mut high, 40);
        expect_move(&high, move_ids::OW_CLAW, 15, 1, 0, &[]);

        // Source: reference/extracted/methods/monster/Spiker.java.
        let mut e = make("Spiker", 42);
        roll_initial_move_with_num_and_rng(
            &mut e, 49, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);
        roll_with_num(&mut e, 49);
        expect_move(&e, move_ids::SPIKER_BUFF, 0, 0, 0, &[(mfx::THORNS, 2)]);
        assert_eq!(e.entity.status(sid::THORNS), 3,
            "choosing the buff must not execute ApplyPowerAction");
        roll_with_num(&mut e, 49);
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);
        roll_with_num(&mut e, 50);
        expect_move(&e, move_ids::SPIKER_BUFF, 0, 0, 0, &[(mfx::THORNS, 2)]);
        e.entity.set_status(sid::COUNT, 6);
        roll_with_num(&mut e, 99);
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);

        let mut e = make("Repulsor", 29);
        roll_initial_move_with_num_and_rng(
            &mut e, 19, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::REPULSOR_ATTACK, 11, 1, 0, &[]);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::REPULSOR_DAZE, 0, 0, 0,
            &[(mfx::DAZE_DRAW, 2)]);
        roll_with_num(&mut e, 19);
        expect_move(&e, move_ids::REPULSOR_ATTACK, 11, 1, 0, &[]);
        roll_with_num(&mut e, 20);
        expect_move(&e, move_ids::REPULSOR_DAZE, 0, 0, 0,
            &[(mfx::DAZE_DRAW, 2)]);

        let mut e = make("Exploder", 30);
        e.entity.set_status(sid::TURN_COUNT, 1);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        e.entity.set_status(sid::TURN_COUNT, 2);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::EXPLODER_EXPLODE, 0, 0, 0, &[]);

        let mut e = make("WrithingMass", 160);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_ATTACK_BLOCK, 15, 1, 15, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_ATTACK_DEBUFF, 10, 1, 0, &[(mfx::WEAK, 2), (mfx::VULNERABLE, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_BIG_HIT, 32, 1, 0, &[]);
        writhing_mass_reactive_reroll(&mut e);
        assert_ne!(e.move_id, move_ids::WM_BIG_HIT);

        // Source: reference/extracted/methods/monster/SpireGrowth.java. COUNT
        // mirrors player.hasPower("Constricted") for this context-free AI.
        let mut e = make("Serpent", 170);
        roll_initial_move_with_num_and_rng(
            &mut e, 49, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]);

        let mut high = make("Serpent", 170);
        roll_initial_move_with_num_and_rng(
            &mut high, 50, &mut crate::seed::StsRandom::new(0));
        expect_move(&high, move_ids::SG_CONSTRICT, 0, 0, 0,
            &[(mfx::CONSTRICT, 10)]);
        assert!(matches!(high.intent, crate::combat_types::Intent::Debuff { .. }));

        high.entity.set_status(sid::COUNT, 1);
        roll_with_num(&mut high, 99);
        expect_move(&high, move_ids::SG_SMASH, 22, 1, 0, &[]);
        roll_with_num(&mut high, 99);
        expect_move(&high, move_ids::SG_SMASH, 22, 1, 0, &[]);
        roll_with_num(&mut high, 99);
        expect_move(&high, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]);

        let mut repeated_tackle = make("Serpent", 170);
        repeated_tackle.move_id = move_ids::SG_QUICK_TACKLE;
        repeated_tackle.move_history = vec![move_ids::SG_QUICK_TACKLE];
        roll_with_num(&mut repeated_tackle, 0);
        expect_move(&repeated_tackle, move_ids::SG_CONSTRICT, 0, 0, 0,
            &[(mfx::CONSTRICT, 10)]);

        let mut a17 = make("Serpent", 190);
        a17.entity.set_status(sid::HIGH_ASCENSION_AI, 1);
        a17.entity.set_status(sid::BLOCK_AMT, 12);
        roll_initial_move_with_num_and_rng(
            &mut a17, 0, &mut crate::seed::StsRandom::new(0));
        expect_move(&a17, move_ids::SG_CONSTRICT, 0, 0, 0,
            &[(mfx::CONSTRICT, 12)]);

        let mut e = make("Maw", 300);
        crate::enemies::roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::MAW_ROAR, 0, 0, 0, &[(mfx::WEAK, 3), (mfx::FRAIL, 3)]);
        expect_status(&e, sid::TURN_COUNT, 2);

        // Source: Maw.java getMove chooses Nom below 50 immediately after
        // Roar; Nom/Slam repetition and Drool are not a fixed cycle.
        e.entity.set_status(sid::FIRST_MOVE, 1);
        crate::enemies::roll_next_move_with_num(&mut e, 49);
        expect_move(&e, move_ids::MAW_NOM, 5, 1, 0, &[]);
        crate::enemies::roll_next_move_with_num(&mut e, 99);
        expect_move(&e, move_ids::MAW_DROOL, 0, 0, 0, &[(mfx::STRENGTH, 3)]);
        crate::enemies::roll_next_move_with_num(&mut e, 49);
        expect_move(&e, move_ids::MAW_NOM, 5, 2, 0, &[]);

        let mut e = make("Transient", 999);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::TRANSIENT_ATTACK, 40, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::TRANSIENT_ATTACK, 50, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::TRANSIENT_ATTACK, 60, 1, 0, &[]);

        let mut e = make("GiantHead", 500);
        e.move_id = move_ids::GH_GLARE;
        e.move_history = vec![move_ids::GH_GLARE, move_ids::GH_GLARE];
        e.entity.set_status(sid::COUNT, 5);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GH_COUNT, 13, 1, 0, &[]);
        expect_status(&e, sid::COUNT, 4);

        e.move_id = move_ids::GH_COUNT;
        e.move_history = vec![move_ids::GH_COUNT, move_ids::GH_COUNT];
        e.entity.set_status(sid::COUNT, 4);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GH_GLARE, 0, 0, 0, &[(mfx::WEAK, 1)]);
        expect_status(&e, sid::COUNT, 3);

        e.entity.set_status(sid::COUNT, 1);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GH_IT_IS_TIME, 30, 1, 0, &[]);

        let mut e = make("Nemesis", 185);
        roll_initial_move_with_num_and_rng(
            &mut e, 49, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::NEM_TRI_ATTACK, 6, 3, 0, &[]);
        expect_status(&e, sid::SCYTHE_COOLDOWN, -1);
        roll_with_num(&mut e, 29);
        expect_move(&e, move_ids::NEM_SCYTHE, 45, 1, 0, &[]);
        expect_status(&e, sid::SCYTHE_COOLDOWN, 2);

        let true_seed = (1..10_000).find(|&seed| {
            crate::seed::StsRandom::new(seed).random_boolean()
        }).unwrap();
        let mut true_rng = crate::seed::StsRandom::new(true_seed);
        crate::enemies::roll_next_move_with_num_and_rng(&mut e, 29, &mut true_rng);
        expect_move(&e, move_ids::NEM_TRI_ATTACK, 6, 3, 0, &[]);
        assert_eq!(true_rng.counter, 1,
            "low window after Scythe consumes its conditional randomBoolean");

        let false_seed = (1..10_000).find(|&seed| {
            !crate::seed::StsRandom::new(seed).random_boolean()
        }).unwrap();
        let mut middle = make("Nemesis", 185);
        middle.entity.set_status(sid::FIRST_MOVE, 0);
        middle.entity.set_status(sid::SCYTHE_COOLDOWN, 1);
        middle.move_history = vec![move_ids::NEM_TRI_ATTACK];
        let mut false_rng = crate::seed::StsRandom::new(false_seed);
        crate::enemies::roll_next_move_with_num_and_rng(&mut middle, 30, &mut false_rng);
        expect_move(&middle, move_ids::NEM_BURN, 0, 0, 0, &[(mfx::BURN, 3)]);
        assert_eq!(false_rng.counter, 1);

        // Source: reference/extracted/methods/monster/Reptomancer.java. Its
        // opener ignores `num`; later low/high repeats recursively consume one
        // additional aiRng draw, while the middle window checks canSpawn and
        // prevents a third consecutive summon.
        let mut e = make("Reptomancer", 190);
        roll_initial_move_with_num_and_rng(
            &mut e, 99, &mut crate::seed::StsRandom::new(0));
        expect_move(&e, move_ids::REPTO_SPAWN, 0, 0, 0, &[]);
        expect_status(&e, sid::FIRST_MOVE, 0);
        roll_with_num(&mut e, 0);
        expect_move(&e, move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0, &[(mfx::WEAK, 1)]);

        let middle_seed = (1..10_000).find(|&seed| {
            (33..66).contains(
                &crate::seed::StsRandom::new(seed).random_range(33, 99))
        }).unwrap();
        let mut middle_rng = crate::seed::StsRandom::new(middle_seed);
        e.entity.set_status(sid::COUNT, 2);
        roll_next_move_with_num_and_rng(&mut e, 0, &mut middle_rng);
        expect_move(&e, move_ids::REPTO_SPAWN, 0, 0, 0, &[]);
        assert_eq!(middle_rng.counter, 1,
            "repeated low Snake Strike rerolls with aiRng.random(33, 99)");

        e.move_id = move_ids::REPTO_SPAWN;
        e.move_history = vec![move_ids::REPTO_SPAWN];
        roll_with_num(&mut e, 33);
        expect_move(&e, move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0, &[(mfx::WEAK, 1)]);

        e.move_id = move_ids::REPTO_BIG_BITE;
        e.move_history.clear();
        e.entity.set_status(sid::COUNT, 4);
        roll_with_num(&mut e, 33);
        expect_move(&e, move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0, &[(mfx::WEAK, 1)]);

        e.move_id = move_ids::REPTO_BIG_BITE;
        e.move_history.clear();
        e.entity.set_status(sid::COUNT, 2);
        let low_seed = (1..10_000).find(|&seed| {
            crate::seed::StsRandom::new(seed).random(65) < 33
        }).unwrap();
        let mut low_rng = crate::seed::StsRandom::new(low_seed);
        roll_next_move_with_num_and_rng(&mut e, 99, &mut low_rng);
        expect_move(&e, move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0, &[(mfx::WEAK, 1)]);
        assert_eq!(low_rng.counter, 1,
            "repeated Big Bite rerolls with aiRng.random(65)");

        let mut e = make("SnakeDagger", 20);
        crate::enemies::roll_initial_move(
            &mut e, &mut crate::seed::StsRandom::new(0));
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SD_EXPLODE, 25, 1, 0, &[]);
    }

    #[test]
    fn exploder_stats_timer_unknown_intent_and_thorns_suicide_match_java() {
        // Sources: reference/extracted/methods/monster/Exploder.java and
        // decompiled/java-src/com/megacrit/cardcrawl/powers/ExplosivePower.java.
        // HP changes at A7, attack at A2;
        // the power ticks after takeTurn and suicides before dealing 30
        // DamageInfo.THORNS when its amount reaches one.
        let mut hp_values = std::collections::BTreeSet::new();
        for seed in 1..=256 {
            let mut run = run_engine(seed, 7);
            run.debug_enter_specific_combat(&["Exploder"]);
            let combat = run.get_combat_engine().expect("Exploder combat");
            hp_values.insert(combat.state.enemies[0].entity.hp);
            assert_eq!(combat.state.enemies[0].move_damage(), 11);
        }
        assert_eq!(hp_values, (30..=35).collect());

        let mut a0 = run_engine(7, 0);
        a0.debug_enter_specific_combat(&["Exploder"]);
        let combat = a0.debug_combat_engine_mut();
        combat.state.player.hp = 500;
        combat.state.player.max_hp = 500;
        assert_eq!(combat.state.enemies[0].entity.hp, 30);
        expect_move(&combat.state.enemies[0], move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        expect_status(&combat.state.enemies[0], sid::TURN_COUNT, 0);
        expect_status(&combat.state.enemies[0], sid::EXPLOSIVE, 3);
        assert_eq!(combat.ai_rng.counter, 1);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 491);
        expect_status(&combat.state.enemies[0], sid::TURN_COUNT, 1);
        expect_status(&combat.state.enemies[0], sid::EXPLOSIVE, 2);
        expect_move(&combat.state.enemies[0], move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        assert_eq!(combat.ai_rng.counter, 2);

        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 482);
        expect_status(&combat.state.enemies[0], sid::TURN_COUNT, 2);
        expect_status(&combat.state.enemies[0], sid::EXPLOSIVE, 1);
        expect_move(&combat.state.enemies[0], move_ids::EXPLODER_EXPLODE, 0, 0, 0, &[]);
        assert_eq!(combat.ai_rng.counter, 3);

        // Barricade retains this block through EndTurn. ExplosivePower's
        // THORNS damage consumes it, while the UNKNOWN intent deals no attack.
        combat.state.player.set_status(sid::BARRICADE, 1);
        combat.state.player.block = 10;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.hp, 462);
        assert_eq!(combat.state.player.block, 0);
        assert_eq!(combat.state.enemies[0].entity.hp, 0);
        expect_status(&combat.state.enemies[0], sid::EXPLOSIVE, 0);
        assert_eq!(combat.ai_rng.counter, 4);
        assert!(combat.state.combat_over);
        assert!(combat.state.player_won);
    }

    #[test]
    fn act4_initial_states_match_java() {
        let e = make("SpireShield", 200);
        // Source: reference/extracted/methods/monster/SpireShield.java.
        expect_move(&e, move_ids::SHIELD_BASH, 12, 1, 0, &[]);
        expect_status(&e, sid::MOVE_COUNT, 0);
        expect_status(&e, sid::STARTING_DMG, 12);
        expect_status(&e, sid::STR_AMT, 34);
        expect_status(&e, sid::ARTIFACT, 1);

        let e = make("SpireSpear", 200);
        expect_move(&e, move_ids::SPEAR_BURN_STRIKE, 5, 2, 0, &[(mfx::BURN, 2)]);
        expect_status(&e, sid::MOVE_COUNT, 0);
        expect_status(&e, sid::SKEWER_COUNT, 3);
    }

    #[test]
    fn act4_patterns_match_java() {
        let mut e = make("SpireShield", 200);
        // Source: reference/extracted/methods/monster/SpireShield.java
        // (`getMove`). Slot zero consumes one conditional boolean, slot one
        // selects the other opening move, and slot two is always Smash.
        let false_seed = (1..10_000).find(|&seed|
            !crate::seed::StsRandom::new(seed).random_boolean()).unwrap();
        let mut rng = crate::seed::StsRandom::new(false_seed);
        roll_initial_move_with_num_and_rng(&mut e, 99, &mut rng);
        expect_move(&e, move_ids::SHIELD_BASH, 12, 1, 0, &[]);
        assert_eq!(rng.counter, 1);
        roll_next_move_with_num_and_rng(&mut e, 99, &mut rng);
        expect_move(&e, move_ids::SHIELD_FORTIFY, 0, 0, 30,
            &[(mfx::BLOCK_ALL_ALLIES, 30)]);
        assert_eq!(rng.counter, 1);
        roll_next_move_with_num_and_rng(&mut e, 99, &mut rng);
        expect_move(&e, move_ids::SHIELD_SMASH, 34, 1, 0, &[]);
        assert_eq!(rng.counter, 1);

        e.entity.set_status(sid::MOVE_COUNT, 0);
        let true_seed = (1..10_000).find(|&seed|
            crate::seed::StsRandom::new(seed).random_boolean()).unwrap();
        let mut rng = crate::seed::StsRandom::new(true_seed);
        roll_initial_move_with_num_and_rng(&mut e, 0, &mut rng);
        expect_move(&e, move_ids::SHIELD_FORTIFY, 0, 0, 30,
            &[(mfx::BLOCK_ALL_ALLIES, 30)]);
        assert_eq!(rng.counter, 1);

        let mut e = make("SpireSpear", 200);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_PIERCER, 0, 0, 0, &[(mfx::STRENGTH, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_SKEWER, 10, 3, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_PIERCER, 0, 0, 0, &[(mfx::STRENGTH, 2)]);
    }

    #[test]
    fn spire_shield_stats_powers_moves_and_surrounded_match_java() {
        // Source: reference/extracted/methods/monster/SpireShield.java and
        // decompiled AbstractMonster.java (`applyBackAttack`).
        for (ascension, hp, bash, smash, artifact, high_ai) in [
            (0, 110, 12, 34, 1, 0),
            (3, 110, 14, 38, 1, 0),
            (8, 125, 14, 38, 1, 0),
            (18, 125, 14, 38, 2, 1),
        ] {
            let mut run = run_engine(42, ascension);
            run.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
            let combat = run.get_combat_engine().expect("Shield and Spear combat");
            let shield = &combat.state.enemies[0];
            assert_eq!((shield.entity.hp, shield.entity.max_hp), (hp, hp));
            assert_eq!(shield.entity.status(sid::STARTING_DMG), bash);
            assert_eq!(shield.entity.status(sid::STR_AMT), smash);
            assert_eq!(shield.entity.status(sid::ARTIFACT), artifact);
            assert_eq!(shield.entity.status(sid::HIGH_ASCENSION_AI), high_ai);
            assert!(shield.back_attack);
            assert!(!combat.state.enemies[1].back_attack);
            assert!(matches!(shield.move_id,
                move_ids::SHIELD_BASH | move_ids::SHIELD_FORTIFY));
            assert_eq!(combat.ai_rng.counter, 2,
                "rollMove integer plus slot-zero randomBoolean");
        }

        // Fortify queues GainBlockAction(30) for every monster, including self.
        let mut fortify = run_engine(43, 0);
        fortify.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
        let combat = fortify.debug_combat_engine_mut();
        combat.state.enemies[0].set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
        combat.state.enemies[0].add_effect(mfx::BLOCK_ALL_ALLIES, 30);
        combat.state.enemies[1].move_id = -1;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies[0].entity.block, 30);
        assert_eq!(combat.state.enemies[1].entity.block, 30);

        // With an occupied orb Bash conditionally consumes one aiRng boolean
        // and applies -1 Focus. Artifact blocks the selected negative power.
        let focus_seed = (1..10_000).find(|&seed|
            crate::seed::StsRandom::new(seed).random_boolean()).unwrap();
        let mut bash = run_engine(44, 0);
        bash.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
        let combat = bash.debug_combat_engine_mut();
        combat.ai_rng = crate::seed::StsRandom::new(focus_seed);
        combat.state.orb_slots = crate::orbs::OrbSlots::new(1);
        combat.state.orb_slots.channel(crate::orbs::OrbType::Lightning, 0);
        combat.state.player.set_status(sid::ARTIFACT, 1);
        combat.state.enemies[0].set_move(move_ids::SHIELD_BASH, 12, 1, 0);
        combat.state.enemies[0].intent = Intent::AttackDebuff {
            damage: 12, hits: 1, effects: 0,
        };
        combat.state.enemies[1].entity.hp = 0;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.player.focus(), 0);
        assert_eq!(combat.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(combat.ai_rng.counter, 2,
            "Bash boolean followed by RollMove integer");

        // Pre-A18 Smash gains its modified DamageInfo.output, not HP lost;
        // initial Back Attack turns 34 into 51 even through full player Block.
        let mut smash = run_engine(45, 0);
        smash.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
        let combat = smash.debug_combat_engine_mut();
        combat.state.player.set_status(sid::BARRICADE, 1);
        combat.state.player.block = 100;
        combat.state.enemies[0].set_move(move_ids::SHIELD_SMASH, 34, 1, 0);
        combat.state.enemies[0].intent = Intent::AttackBlock {
            damage: 34, hits: 1, block: 0, effects: 0,
        };
        combat.state.enemies[1].entity.hp = 0;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies[0].entity.block, 51);
        assert_eq!(combat.state.player.hp, combat.state.player.max_hp);

        let mut high_smash = run_engine(46, 18);
        high_smash.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
        let combat = high_smash.debug_combat_engine_mut();
        combat.state.player.set_status(sid::BARRICADE, 1);
        combat.state.player.block = 100;
        combat.state.enemies[0].set_move(move_ids::SHIELD_SMASH, 38, 1, 0);
        combat.state.enemies[0].intent = Intent::AttackBlock {
            damage: 38, hits: 1, block: 0, effects: 0,
        };
        combat.state.enemies[1].entity.hp = 0;
        combat.execute_action(&crate::actions::Action::EndTurn);
        assert_eq!(combat.state.enemies[0].entity.block, 99);

        // A targeted card turns the player toward that monster. Killing either
        // partner removes Surrounded/BackAttack from the survivor.
        let mut facing = run_engine(47, 0);
        facing.debug_enter_specific_combat(&["SpireShield", "SpireSpear"]);
        let combat = facing.debug_combat_engine_mut();
        combat.state.hand = vec![combat.card_registry.make_card("Strike")];
        combat.state.energy = 3;
        combat.execute_action(&crate::actions::Action::PlayCard {
            card_idx: 0,
            target_idx: 0,
        });
        assert!(!combat.state.enemies[0].back_attack);
        assert!(combat.state.enemies[1].back_attack);
        assert!(combat.instant_kill_enemy(0));
        assert!(!combat.state.enemies[1].back_attack);
    }

    #[test]
    fn run_engine_exposes_ascension_hp_tables() {
        // Rewritten per Cultist.java ctor (verify monster/Cultist): setHp(48, 54)
        // below ascension 7, setHp(50, 56) at 7+ — a uniform roll, not a fixed
        // value. The old assertions (hp == 48 / == 50) baked in the pre-fix
        // fixed-HP behavior.
        let act1_weak = enter_forced_combat(1, 0, RoomType::Monster, 0);
        let combat = act1_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Cultist");
        assert!((48..=54).contains(&combat.state.enemies[0].entity.hp));

        let act1_weak_a20 = enter_forced_combat(1, 20, RoomType::Monster, 0);
        let combat = act1_weak_a20.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Cultist");
        assert!((50..=56).contains(&combat.state.enemies[0].entity.hp));
        // Cultist.java: ritualAmount = asc >= 2 ? 4 : 3, +1 applied at asc >= 17.
        assert_eq!(combat.state.enemies[0].effect(mfx::RITUAL), Some(5));

        let act1_strong = enter_forced_combat(1, 0, RoomType::Monster, 3);
        let combat = act1_strong.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "BlueSlaver");
        // SlaverBlue.java uses inclusive setHp(46, 50), not a fixed 46.
        assert!((46..=50).contains(&combat.state.enemies[0].entity.hp));

        let act1_elite = enter_forced_combat(1, 20, RoomType::Elite, 0);
        let combat = act1_elite.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "GremlinNob");
        // Source: reference/extracted/methods/monster/GremlinNob.java:
        // ascension 8+ uses inclusive setHp(85, 90), not fixed 110 HP.
        assert!((85..=90).contains(&combat.state.enemies[0].entity.hp));

        let act2_weak = enter_forced_combat(2, 0, RoomType::Monster, 0);
        let combat = act2_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Byrd");
        // Source: reference/extracted/methods/monster/Byrd.java:
        // the constructor uses inclusive setHp(25, 31), not fixed 25 HP.
        assert!((25..=31).contains(&combat.state.enemies[0].entity.hp));

        let act2_strong = enter_forced_combat(2, 20, RoomType::Monster, 3);
        let combat = act2_strong.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "SnakePlant");
        assert!((78..=82).contains(&combat.state.enemies[0].entity.hp));

        let act2_elite = enter_forced_combat(2, 20, RoomType::Elite, 0);
        let combat = act2_elite.get_combat_engine().expect("combat engine");
        // Sources: MonsterHelper.java and GremlinLeader.java construct two
        // random gremlins followed by a 145..=155 HP Leader at A8+.
        assert_eq!(combat.state.enemies.len(), 3);
        assert!(combat.state.enemies[..2].iter().all(|enemy| enemy.is_minion));
        let leader = combat.state.enemies.iter()
            .find(|enemy| enemy.id == "GremlinLeader").expect("Gremlin Leader");
        assert!((145..=155).contains(&leader.entity.hp));

        let act3_weak = enter_forced_combat(3, 0, RoomType::Monster, 0);
        let combat = act3_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Darkling");
        // Source: reference/extracted/methods/monster/Darkling.java uses an
        // inclusive 48..=56 constructor HP roll below ascension 7.
        assert!((48..=56).contains(&combat.state.enemies[0].entity.hp));

        let act3_strong = enter_forced_combat(3, 20, RoomType::Monster, 3);
        let combat = act3_strong.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "WrithingMass");
        assert_eq!(combat.state.enemies[0].entity.hp, 175);

        let act3_elite = enter_forced_combat(3, 20, RoomType::Elite, 0);
        let combat = act3_elite.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "GiantHead");
        assert_eq!(combat.state.enemies[0].entity.hp, 520);

        let act4_elite = enter_forced_combat(4, 20, RoomType::Elite, 0);
        let combat = act4_elite.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "SpireShield");
        // Source: reference/extracted/methods/monster/SpireShield.java: fixed
        // 125 HP at ascension 8+, rather than the draft's shared 220 table.
        assert_eq!(combat.state.enemies[0].entity.hp, 125);
    }
}
