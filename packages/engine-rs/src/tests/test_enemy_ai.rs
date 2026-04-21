#[cfg(test)]
mod enemy_ai_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/exordium/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/city/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/beyond/*.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/monsters/ending/*.java

    use crate::enemies::*;
    use crate::combat_types::mfx;
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
        expect_one_of(&e, &[move_ids::AS_TACKLE, move_ids::AS_LICK]);
        match e.move_id {
            x if x == move_ids::AS_TACKLE => expect_move(&e, move_ids::AS_TACKLE, 3, 1, 0, &[]),
            _ => expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 1)]),
        }

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
        expect_move(&e, move_ids::SS_TACKLE, 8, 1, 0, &[]);

        let e = make("SpikeSlime_L", 65);
        expect_move(&e, move_ids::SS_TACKLE, 16, 1, 0, &[]);

        let e = make("Looter", 44);
        expect_move(&e, move_ids::LOOTER_MUG, 10, 1, 0, &[]);

        let e = make("GremlinFat", 18);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 4, 1, 0, &[(mfx::WEAK, 1)]);

        let e = make("GremlinThief", 13);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 9, 1, 0, &[]);

        let e = make("GremlinWarrior", 11);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 4, 1, 0, &[]);

        let e = make("GremlinWizard", 20);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);

        let e = make("GremlinTsundere", 13);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);

        let e = make("GremlinNob", 106);
        expect_move(&e, move_ids::NOB_BELLOW, 0, 0, 0, &[]);
        expect_status(&e, sid::ENRAGE, 2);

        let e = make("Lagavulin", 109);
        expect_move(&e, move_ids::LAGA_SLEEP, 0, 0, 0, &[]);
        expect_status(&e, sid::METALLICIZE, 8);
        expect_status(&e, sid::SLEEP_TURNS, 3);

        let e = make("Sentry", 38);
        expect_move(&e, move_ids::SENTRY_BOLT, 9, 1, 0, &[]);
    }

    #[test]
    fn act1_patterns_match_java() {
        // JawWorm is probabilistic (post-AI-RNG-fix); test each branch via num.
        let mut e = make("JawWorm", 44);
        roll_with_num(&mut e, 30); // 25..55 -> BELLOW
        expect_move(&e, move_ids::JW_BELLOW, 0, 0, 6, &[(mfx::STRENGTH, 3)]);
        roll_with_num(&mut e, 80); // >=55 and !lastTwoMoves(THRASH) -> THRASH
        expect_move(&e, move_ids::JW_THRASH, 7, 1, 5, &[]);
        roll_with_num(&mut e, 10); // <25 and !lastTwoMoves(CHOMP) -> CHOMP
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
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOUSE_GROW, 0, 0, 0, &[(mfx::STRENGTH, 3)]);

        let mut e = make("GreenLouse", 14);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOUSE_BITE, 6, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOUSE_SPIT_WEB, 0, 0, 0, &[(mfx::WEAK, 2)]);

        let mut e = make("SlaverBlue", 46);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BS_STAB, 12, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BS_RAKE, 7, 1, 0, &[(mfx::WEAK, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BS_STAB, 12, 1, 0, &[]);

        let mut e = make("SlaverRed", 46);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::RS_ENTANGLE, 0, 0, 0, &[(mfx::ENTANGLE, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::RS_STAB, 13, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::RS_SCRAPE, 8, 1, 0, &[(mfx::VULNERABLE, 1)]);

        let mut e = make("AcidSlime_S", 8);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::AS_LICK, 0, 0, 0, &[(mfx::WEAK, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::AS_TACKLE, 3, 1, 0, &[]);

        let mut e = make("AcidSlime_M", 28);
        roll_times(&mut e, 1);
        let first_move = e.move_id;
        assert!(matches!(
            first_move,
            move_ids::AS_CORROSIVE_SPIT | move_ids::AS_TACKLE | move_ids::AS_LICK
        ));
        roll_times(&mut e, 1);
        assert_ne!(e.move_id, first_move, "AcidSlime_M should not immediately repeat its first move");

        let mut e = make("AcidSlime_L", 65);
        expect_one_of(&e, &[move_ids::AS_CORROSIVE_SPIT, move_ids::AS_TACKLE, move_ids::AS_LICK]);

        e.move_id = move_ids::AS_TACKLE;
        e.move_history = vec![move_ids::AS_TACKLE, move_ids::AS_TACKLE];
        roll_times(&mut e, 1);
        assert_ne!(e.move_id, move_ids::AS_TACKLE, "AcidSlime_L should not use Normal Tackle three times in a row");

        e.move_id = move_ids::AS_CORROSIVE_SPIT;
        e.move_history = vec![move_ids::AS_CORROSIVE_SPIT, move_ids::AS_CORROSIVE_SPIT];
        roll_times(&mut e, 1);
        assert_ne!(
            e.move_id,
            move_ids::AS_CORROSIVE_SPIT,
            "AcidSlime_L should not use Corrosive Spit three times in a row"
        );

        let mut e = make("SpikeSlime_M", 28);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SS_TACKLE, 8, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SS_TACKLE, 8, 1, 0, &[]);

        let mut e = make("SpikeSlime_L", 65);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SS_TACKLE, 16, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SS_LICK, 0, 0, 0, &[(mfx::FRAIL, 2)]);

        let mut e = make("Looter", 44);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOOTER_MUG, 10, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOOTER_SMOKE_BOMB, 0, 0, 11, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LOOTER_ESCAPE, 0, 0, 0, &[]);
        assert!(e.is_escaping);

        let mut e = make("GremlinFat", 18);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 4, 1, 0, &[(mfx::WEAK, 1)]);
        let mut e = make("GremlinWizard", 20);
        e.move_id = move_ids::GREMLIN_PROTECT;
        e.move_history = vec![move_ids::GREMLIN_PROTECT, move_ids::GREMLIN_PROTECT];
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_ATTACK, 25, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);
        let mut e = make("GremlinTsundere", 13);
        roll_times(&mut e, 3);
        expect_move(&e, move_ids::GREMLIN_PROTECT, 0, 0, 0, &[]);

        let mut e = make("GremlinNob", 106);
        roll_times(&mut e, 1);
        assert!(
            matches!(e.move_id, move_ids::NOB_SKULL_BASH | move_ids::NOB_RUSH),
            "GremlinNob should follow Bellow with either Skull Bash or Rush"
        );
        e.move_id = move_ids::NOB_RUSH;
        e.move_history = vec![move_ids::NOB_RUSH, move_ids::NOB_RUSH];
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::NOB_SKULL_BASH, 6, 1, 0, &[(mfx::VULNERABLE, 2)]);

        let mut e = make("Lagavulin", 109);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LAGA_SLEEP, 0, 0, 0, &[]);
        expect_status(&e, sid::SLEEP_TURNS, 2);
        roll_times(&mut e, 1);
        expect_status(&e, sid::SLEEP_TURNS, 1);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::LAGA_ATTACK, 18, 1, 0, &[]);
        lagavulin_wake_up(&mut e);
        expect_move(&e, move_ids::LAGA_ATTACK, 18, 1, 0, &[]);
        expect_status(&e, sid::SLEEP_TURNS, 0);

        let mut e = make("Sentry", 38);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SENTRY_BEAM, 9, 1, 0, &[(mfx::DAZE, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SENTRY_BOLT, 9, 1, 0, &[]);
    }

    #[test]
    fn act2_initial_states_match_java() {
        let e = make("Chosen", 95);
        expect_move(&e, move_ids::CHOSEN_POKE, 5, 2, 0, &[]);

        let e = make("Mugger", 48);
        expect_move(&e, move_ids::MUGGER_MUG, 10, 1, 0, &[]);

        let e = make("Byrd", 25);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);
        expect_status(&e, sid::FLIGHT, 3);

        let e = make("ShelledParasite", 68);
        expect_move(&e, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);
        expect_status(&e, sid::PLATED_ARMOR, 14);

        let e = make("SnakePlant", 75);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);
        expect_status(&e, sid::MALLEABLE, 1);

        let e = make("Centurion", 76);
        expect_move(&e, move_ids::CENT_FURY, 6, 3, 0, &[]);

        let e = make("Mystic", 48);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[]);

        let e = make("Healer", 48);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[]);

        let e = make("BookOfStabbing", 160);
        expect_move(&e, move_ids::BOOK_STAB, 6, 2, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 2);

        let e = make("GremlinLeader", 140);
        expect_move(&e, move_ids::GL_RALLY, 0, 0, 0, &[]);

        let e = make("Taskmaster", 60);
        expect_move(&e, move_ids::TASK_SCOURING_WHIP, 7, 1, 0, &[(mfx::WOUND, 1)]);

        let e = make("SphericGuardian", 135);
        expect_move(&e, move_ids::SPHER_INITIAL_BLOCK, 0, 0, 40, &[]);

        let e = make("Snecko", 114);
        expect_move(&e, move_ids::SNECKO_GLARE, 0, 0, 0, &[(mfx::CONFUSED, 1)]);

        let e = make("BanditBear", 40);
        expect_move(&e, move_ids::BEAR_HUG, 0, 0, 0, &[(mfx::DEX_DOWN, 2)]);

        let e = make("BanditLeader", 50);
        expect_move(&e, move_ids::BANDIT_MOCK, 0, 0, 0, &[]);

        let e = make("BanditPointy", 35);
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
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CHOSEN_HEX, 0, 0, 0, &[(mfx::HEX, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CHOSEN_DEBILITATE, 10, 1, 0, &[(mfx::VULNERABLE, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CHOSEN_ZAP, 18, 1, 0, &[]);

        let mut e = make("Mugger", 48);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MUGGER_MUG, 10, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MUGGER_BIG_SWIPE, 16, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MUGGER_SMOKE_BOMB, 0, 0, 11, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MUGGER_ESCAPE, 0, 0, 0, &[]);
        assert!(e.is_escaping);

        let mut e = make("Byrd", 25);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BYRD_SWOOP, 12, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BYRD_PECK, 1, 5, 0, &[]);

        let mut e = make("ShelledParasite", 68);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SP_LIFE_SUCK, 10, 1, 0, &[(mfx::HEAL, 10)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SP_FELL, 18, 1, 0, &[(mfx::FRAIL, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SP_DOUBLE_STRIKE, 6, 2, 0, &[]);

        let mut e = make("SnakePlant", 75);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNAKE_SPORES, 0, 0, 0, &[(mfx::WEAK, 2), (mfx::FRAIL, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNAKE_CHOMP, 7, 3, 0, &[]);

        // Centurion: Fury -> Slash -> Protect -> Fury -> ...
        let mut e = make("Centurion", 76);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CENT_SLASH, 12, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CENT_PROTECT, 0, 0, 15, &[(mfx::BLOCK_ALL_ALLIES, 15)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::CENT_FURY, 6, 3, 0, &[]);

        // Mystic: Attack -> Attack -> Heal -> Attack -> Attack -> Buff -> ...
        let mut e = make("Mystic", 48);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MYSTIC_HEAL, 0, 0, 0, &[(mfx::HEAL_LOWEST_ALLY, 16)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[]);

        let mut e = make("Healer", 48);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MYSTIC_ATTACK, 8, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MYSTIC_HEAL, 0, 0, 0, &[(mfx::HEAL_LOWEST_ALLY, 16)]);

        let mut e = make("BookOfStabbing", 160);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BOOK_STAB, 6, 3, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 3);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BOOK_BIG_STAB, 21, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BOOK_STAB, 6, 4, 0, &[]);
        expect_status(&e, sid::STAB_COUNT, 4);

        let mut e = make("GremlinLeader", 140);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GL_ENCOURAGE, 0, 0, 6, &[(mfx::STRENGTH_ALL_ALLIES, 3), (mfx::BLOCK_ALL_ALLIES, 6)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GL_STAB, 6, 3, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::GL_RALLY, 0, 0, 0, &[]);

        let mut e = make("Taskmaster", 60);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::TASK_SCOURING_WHIP, 7, 1, 0, &[(mfx::WOUND, 1)]);

        let mut e = make("SphericGuardian", 135);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPHER_FRAIL_ATTACK, 10, 1, 0, &[(mfx::FRAIL, 5)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPHER_BIG_ATTACK, 10, 2, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPHER_BLOCK_ATTACK, 10, 1, 15, &[]);

        let mut e = make("Snecko", 114);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNECKO_TAIL, 8, 1, 0, &[(mfx::VULNERABLE, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNECKO_BITE, 15, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SNECKO_BITE, 15, 1, 0, &[]);

        let mut e = make("BanditBear", 40);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BEAR_MAUL, 18, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BEAR_LUNGE, 9, 1, 9, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BEAR_HUG, 0, 0, 0, &[(mfx::DEX_DOWN, 2)]);

        let mut e = make("BanditLeader", 50);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BANDIT_AGONIZE, 10, 1, 0, &[(mfx::WEAK, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BANDIT_CROSS_SLASH, 15, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BANDIT_MOCK, 0, 0, 0, &[]);

        let mut e = make("BanditPointy", 35);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::POINTY_STAB, 5, 2, 0, &[]);

        let mut e = make("BronzeAutomaton", 300);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_FLAIL, 7, 2, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_BOOST, 0, 0, 9, &[(mfx::STRENGTH, 3)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_FLAIL, 7, 2, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BA_HYPER_BEAM, 45, 1, 0, &[]);

        let mut e = make("BronzeOrb", 35);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BO_BEAM, 8, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BO_BEAM, 8, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::BO_SUPPORT, 0, 0, 12, &[]);

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

        let e = make("OrbWalker", 90);
        expect_one_of(&e, &[move_ids::OW_LASER, move_ids::OW_CLAW]);
        match e.move_id {
            x if x == move_ids::OW_LASER => expect_move(&e, move_ids::OW_LASER, 10, 1, 0, &[(mfx::BURN, 1)]),
            _ => expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]),
        }

        let e = make("Spiker", 170);
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);
        expect_status(&e, sid::THORNS, 3);

        let e = make("Repulsor", 29);
        expect_move(&e, move_ids::REPULSOR_DAZE, 0, 0, 0, &[(mfx::DAZE, 2)]);

        let e = make("Exploder", 30);
        expect_move(&e, move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        expect_status(&e, sid::TURN_COUNT, 0);

        let e = make("WrithingMass", 160);
        expect_move(&e, move_ids::WM_MULTI_HIT, 7, 3, 0, &[]);
        expect_status(&e, sid::REACTIVE, 1);
        expect_status(&e, sid::MALLEABLE, 1);

        let e = make("SpireGrowth", 170);
        expect_one_of(&e, &[move_ids::SG_QUICK_TACKLE, move_ids::SG_CONSTRICT]);
        match e.move_id {
            x if x == move_ids::SG_QUICK_TACKLE => expect_move(&e, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]),
            _ => expect_move(&e, move_ids::SG_CONSTRICT, 0, 0, 0, &[(mfx::CONSTRICT, 10)]),
        }

        let e = make("Maw", 300);
        expect_move(&e, move_ids::MAW_ROAR, 0, 0, 0, &[(mfx::WEAK, 3), (mfx::FRAIL, 3)]);
        expect_status(&e, sid::TURN_COUNT, 1);

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

        let e = make("Reptomancer", 190);
        expect_move(&e, move_ids::REPTO_SPAWN, 0, 0, 0, &[]);

        let e = make("SnakeDagger", 20);
        expect_move(&e, move_ids::SD_WOUND, 9, 1, 0, &[(mfx::WOUND, 1)]);
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
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::DARK_REINCARNATE, 0, 0, 0, &[]);

        let mut e = make("OrbWalker", 90);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::OW_LASER, 10, 1, 0, &[(mfx::BURN, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::OW_CLAW, 15, 1, 0, &[]);

        let mut e = make("Spiker", 170);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPIKER_BUFF, 0, 0, 0, &[(mfx::THORNS, 2)]);
        expect_status(&e, sid::THORNS, 5);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPIKER_ATTACK, 7, 1, 0, &[]);

        let mut e = make("Repulsor", 29);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::REPULSOR_DAZE, 0, 0, 0, &[(mfx::DAZE, 2)]);

        let mut e = make("Exploder", 30);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::EXPLODER_ATTACK, 9, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::EXPLODER_EXPLODE, 30, 1, 0, &[]);

        let mut e = make("WrithingMass", 160);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_ATTACK_BLOCK, 15, 1, 15, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_ATTACK_DEBUFF, 10, 1, 0, &[(mfx::WEAK, 2), (mfx::VULNERABLE, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::WM_BIG_HIT, 32, 1, 0, &[]);
        writhing_mass_reactive_reroll(&mut e);
        assert_ne!(e.move_id, move_ids::WM_BIG_HIT);

        let mut e = make("SpireGrowth", 170);
        e.move_id = move_ids::SG_QUICK_TACKLE;
        e.move_history = vec![move_ids::SG_QUICK_TACKLE];
        roll_times(&mut e, 1);
        assert!(
            matches!(e.move_id, move_ids::SG_QUICK_TACKLE | move_ids::SG_CONSTRICT),
            "Java SpireGrowth should not go straight from one Quick Tackle to Smash when the player is not already Constricted"
        );

        e.move_id = move_ids::SG_CONSTRICT;
        e.move_history = vec![move_ids::SG_CONSTRICT];
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]);

        e.move_id = move_ids::SG_SMASH;
        e.move_history = vec![move_ids::SG_SMASH, move_ids::SG_SMASH];
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SG_QUICK_TACKLE, 16, 1, 0, &[]);

        let mut e = make("Maw", 300);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MAW_SLAM, 25, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::MAW_DROOL, 0, 0, 0, &[(mfx::STRENGTH, 3)]);
        roll_times(&mut e, 1);
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
        e.entity.set_status(sid::FIRST_MOVE, 0);
        e.entity.set_status(sid::SCYTHE_COOLDOWN, 0);
        e.move_history = vec![move_ids::NEM_TRI_ATTACK];
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::NEM_SCYTHE, 45, 1, 0, &[]);
        expect_status(&e, sid::SCYTHE_COOLDOWN, 2);

        e.move_id = move_ids::NEM_SCYTHE;
        e.move_history = vec![move_ids::NEM_SCYTHE];
        e.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::NEM_BURN, 0, 0, 0, &[(mfx::BURN, 3)]);

        e.move_id = move_ids::NEM_TRI_ATTACK;
        e.move_history = vec![move_ids::NEM_TRI_ATTACK, move_ids::NEM_TRI_ATTACK];
        e.entity.set_status(sid::SCYTHE_COOLDOWN, 1);
        roll_times(&mut e, 1);
        assert!(
            matches!(e.move_id, move_ids::NEM_BURN | move_ids::NEM_SCYTHE),
            "Nemesis should not use Tri Attack three times in a row once Scythe is available again"
        );

        let mut e = make("Reptomancer", 190);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0, &[(mfx::WEAK, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::REPTO_BIG_BITE, 30, 1, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::REPTO_SPAWN, 0, 0, 0, &[]);

        let mut e = make("SnakeDagger", 20);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SD_EXPLODE, 25, 1, 0, &[]);
    }

    #[test]
    fn act4_initial_states_match_java() {
        let e = make("SpireShield", 200);
        expect_move(&e, move_ids::SHIELD_BASH, 12, 1, 0, &[(mfx::STRENGTH_DOWN, 1)]);
        expect_status(&e, sid::MOVE_COUNT, 0);

        let e = make("SpireSpear", 200);
        expect_move(&e, move_ids::SPEAR_BURN_STRIKE, 5, 2, 0, &[(mfx::BURN, 2)]);
        expect_status(&e, sid::MOVE_COUNT, 0);
        expect_status(&e, sid::SKEWER_COUNT, 3);
    }

    #[test]
    fn act4_patterns_match_java() {
        let mut e = make("SpireShield", 200);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SHIELD_FORTIFY, 0, 0, 30, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SHIELD_BASH, 12, 1, 0, &[(mfx::STRENGTH_DOWN, 1)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SHIELD_SMASH, 34, 1, 0, &[]);

        let mut e = make("SpireSpear", 200);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_PIERCER, 0, 0, 0, &[(mfx::STRENGTH, 2)]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_SKEWER, 10, 3, 0, &[]);
        roll_times(&mut e, 1);
        expect_move(&e, move_ids::SPEAR_PIERCER, 0, 0, 0, &[(mfx::STRENGTH, 2)]);
    }

    #[test]
    fn run_engine_exposes_ascension_hp_tables() {
        let act1_weak = enter_forced_combat(1, 0, RoomType::Monster, 0);
        let combat = act1_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Cultist");
        assert_eq!(combat.state.enemies[0].entity.hp, 48);

        let act1_weak_a20 = enter_forced_combat(1, 20, RoomType::Monster, 0);
        let combat = act1_weak_a20.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Cultist");
        assert_eq!(combat.state.enemies[0].entity.hp, 50);

        let act1_strong = enter_forced_combat(1, 0, RoomType::Monster, 3);
        let combat = act1_strong.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "BlueSlaver");
        assert_eq!(combat.state.enemies[0].entity.hp, 46);

        let act1_elite = enter_forced_combat(1, 20, RoomType::Elite, 0);
        let combat = act1_elite.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "GremlinNob");
        assert_eq!(combat.state.enemies[0].entity.hp, 110);

        let act2_weak = enter_forced_combat(2, 0, RoomType::Monster, 0);
        let combat = act2_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Byrd");
        assert_eq!(combat.state.enemies[0].entity.hp, 25);

        let act2_strong = enter_forced_combat(2, 20, RoomType::Monster, 3);
        let combat = act2_strong.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "SnakePlant");
        assert_eq!(combat.state.enemies[0].entity.hp, 79);

        let act2_elite = enter_forced_combat(2, 20, RoomType::Elite, 0);
        let combat = act2_elite.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "GremlinLeader");
        assert_eq!(combat.state.enemies[0].entity.hp, 162);

        let act3_weak = enter_forced_combat(3, 0, RoomType::Monster, 0);
        let combat = act3_weak.get_combat_engine().expect("combat engine");
        assert_eq!(combat.state.enemies[0].id, "Darkling");
        assert_eq!(combat.state.enemies[0].entity.hp, 48);

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
        assert_eq!(combat.state.enemies[0].entity.hp, 220);
    }
}
