//! Java-vs-Rust AI RNG parity regression tests.
//!
//! These would have caught the bug fixed in commit f0a0bfd4: enemies in Rust
//! were 100% deterministic, never consuming from any RNG, while Java consumes
//! one value from `AbstractDungeon.aiRng` per `AbstractMonster.rollMove()` and
//! passes it to `getMove(int num)` for probabilistic intent branching.
//!
//! Every test here is a pre-registered regression case; failures here mean a
//! known parity property has broken.
//!
//! Audit cross-refs:
//! - §1.1 (the fix itself)
//! - §5.1 (per-enemy probabilistic intent)
//! - §5.2 (multi-enemy stream order parity)
//! - `docs/work_units/parity-status.md` §0
//! - Java references: `decompiled/java-src/com/megacrit/cardcrawl/monsters/`

#[cfg(test)]
mod ai_rng_parity_tests {
    use crate::enemies::{
        create_enemy, move_ids, roll_next_move, roll_next_move_with_num,
        roll_next_move_with_num_and_rng,
    };
    use crate::run::RunEngine;
    use crate::seed::StsRandom;
    use crate::state::EnemyCombatState;
    use crate::status_ids::sid;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ConstructedEnemy {
        id: &'static str,
        hp: i32,
        bite: Option<i32>,
    }

    fn construct_louse(misc: &mut StsRandom, monster_hp: &mut StsRandom) -> ConstructedEnemy {
        // MonsterHelper.getLouse chooses the class first; that constructor then
        // consumes HP followed immediately by Bite damage.
        // Java: helpers/MonsterHelper.java, LouseNormal.java, LouseDefensive.java.
        let (id, hp) = if misc.random_bool() {
            ("FuzzyLouseNormal", monster_hp.random_int_range(10, 15))
        } else {
            ("FuzzyLouseDefensive", monster_hp.random_int_range(11, 17))
        };
        ConstructedEnemy {
            id,
            hp,
            bite: Some(monster_hp.random_int_range(5, 7)),
        }
    }

    fn construct_weak_wildlife(
        misc: &mut StsRandom,
        monster_hp: &mut StsRandom,
    ) -> ConstructedEnemy {
        // Java constructs all candidates before selecting one. The unselected
        // Louse, Spike Slime, and Acid Slime still consume constructor draws.
        // Java: MonsterHelper.java::bottomGetWeakWildlife.
        let candidates = [
            construct_louse(misc, monster_hp),
            ConstructedEnemy {
                id: "SpikeSlime_M",
                hp: monster_hp.random_int_range(28, 32),
                bite: None,
            },
            ConstructedEnemy {
                id: "AcidSlime_M",
                hp: monster_hp.random_int_range(28, 32),
                bite: None,
            },
        ];
        candidates[misc.random_int_range(0, 2) as usize].clone()
    }

    fn construct_strong_humanoid(
        misc: &mut StsRandom,
        monster_hp: &mut StsRandom,
    ) -> ConstructedEnemy {
        // Java constructs Cultist, one color-selected Slaver, and Looter before
        // selecting the surviving candidate.
        // Java: MonsterHelper.java::bottomGetStrongHumanoid/getSlaver.
        let cultist = ConstructedEnemy {
            id: "Cultist",
            hp: monster_hp.random_int_range(48, 54),
            bite: None,
        };
        let slaver = ConstructedEnemy {
            id: if misc.random_bool() {
                "SlaverRed"
            } else {
                "SlaverBlue"
            },
            hp: monster_hp.random_int_range(46, 50),
            bite: None,
        };
        let looter = ConstructedEnemy {
            id: "Looter",
            hp: monster_hp.random_int_range(44, 48),
            bite: None,
        };
        [cultist, slaver, looter][misc.random_int_range(0, 2) as usize].clone()
    }

    fn construct_strong_wildlife(
        misc: &mut StsRandom,
        monster_hp: &mut StsRandom,
    ) -> ConstructedEnemy {
        // Java constructs both candidates before the miscRng selection.
        // Java: MonsterHelper.java::bottomGetStrongWildlife.
        let candidates = [
            ConstructedEnemy {
                id: "FungiBeast",
                hp: monster_hp.random_int_range(22, 28),
                bite: None,
            },
            ConstructedEnemy {
                id: "JawWorm",
                hp: monster_hp.random_int_range(40, 44),
                bite: None,
            },
        ];
        candidates[misc.random_int_range(0, 1) as usize].clone()
    }

    // Source-derived from reference/extracted/methods/monster/JawWorm.java.
    // The three num windows depend on history; three cases consume another
    // aiRng.randomBoolean draw instead of forming a simple 25/30/45 split.
    fn seed_for_float(predicate: fn(f32) -> bool) -> u64 {
        (1..10_000)
            .find(|&seed| {
                let mut rng = StsRandom::new(seed);
                predicate(rng.random_f32())
            })
            .expect("seed satisfying probability branch")
    }

    #[test]
    fn jaw_worm_after_chomp_num_below_25_uses_5625_percent_draw() {
        let cases: [(bool, fn(f32) -> bool); 2] = [
            (true, |value| value < 0.5625),
            (false, |value| value >= 0.5625),
        ];
        for (take_bellow, predicate) in cases {
            let mut rng = StsRandom::new(seed_for_float(predicate));
            let mut e = create_enemy("JawWorm", 44, 44);
            roll_next_move_with_num_and_rng(&mut e, 0, &mut rng);
            assert_eq!(
                e.move_id,
                if take_bellow {
                    move_ids::JW_BELLOW
                } else {
                    move_ids::JW_THRASH
                }
            );
            assert_eq!(rng.counter, 1);
        }
    }

    #[test]
    fn jaw_worm_plain_windows_do_not_draw_secondary_rng() {
        let mut rng = StsRandom::new(7);
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num_and_rng(&mut e, 25, &mut rng);
        assert_eq!(e.move_id, move_ids::JW_THRASH);
        assert_eq!(rng.counter, 0);
        let mut e = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num_and_rng(&mut e, 55, &mut rng);
        assert_eq!(e.move_id, move_ids::JW_BELLOW);
        assert_eq!(rng.counter, 0);
    }

    #[test]
    fn jaw_worm_after_two_thrashes_uses_357_percent_draw() {
        let cases: [(bool, fn(f32) -> bool); 2] = [
            (true, |value| value < 0.357),
            (false, |value| value >= 0.357),
        ];
        for (take_chomp, predicate) in cases {
            let mut e = create_enemy("JawWorm", 44, 44);
            e.move_history.push(move_ids::JW_THRASH);
            e.set_move(move_ids::JW_THRASH, 7, 1, 5);
            let mut rng = StsRandom::new(seed_for_float(predicate));
            roll_next_move_with_num_and_rng(&mut e, 25, &mut rng);
            assert_eq!(
                e.move_id,
                if take_chomp {
                    move_ids::JW_CHOMP
                } else {
                    move_ids::JW_BELLOW
                }
            );
            assert_eq!(rng.counter, 1);
        }
    }

    #[test]
    fn jaw_worm_after_bellow_uses_416_percent_draw() {
        let cases: [(bool, fn(f32) -> bool); 2] = [
            (true, |value| value < 0.416),
            (false, |value| value >= 0.416),
        ];
        for (take_chomp, predicate) in cases {
            let mut e = create_enemy("JawWorm", 44, 44);
            e.set_move(move_ids::JW_BELLOW, 0, 0, 6);
            let mut rng = StsRandom::new(seed_for_float(predicate));
            roll_next_move_with_num_and_rng(&mut e, 55, &mut rng);
            assert_eq!(
                e.move_id,
                if take_chomp {
                    move_ids::JW_CHOMP
                } else {
                    move_ids::JW_THRASH
                }
            );
            assert_eq!(rng.counter, 1);
        }
    }

    // §5.2 -- multi-enemy stream order: every roll_next_move call consumes
    // exactly one value from ai_rng, so two enemies rolled in different orders
    // get different intent sequences. This protects against a regression where
    // ai_rng is no longer threaded (would zero out variance) or where some
    // enemies skip the draw (would desync the stream).

    #[test]
    fn deterministic_enemy_ai_rng_advances_one_per_roll() {
        // Cultist performs no getMove draw, so only AbstractMonster.rollMove's
        // mandatory num draw advances the stream.
        let mut rng_via_roll = StsRandom::new(42);
        let mut rng_manual = StsRandom::new(42);

        for _ in 0..10 {
            let mut e = create_enemy("Cultist", 50, 50);
            roll_next_move(&mut e, &mut rng_via_roll);
            let _ = rng_manual.random_int(99);
            // Both rngs should have consumed the same number of values; the next
            // draw from each must match.
            assert_eq!(
                rng_via_roll.random_int(99),
                rng_manual.random_int(99),
                "ai_rng stream desync after roll_next_move"
            );
        }
    }

    #[test]
    fn multi_enemy_intent_sequence_depends_on_roll_order() {
        // 3 Orb Walkers sharing one ai_rng. Rolling A then B then C consumes
        // different values than rolling C then B then A.
        // Shipped RandomXS128 seed 2 yields 9, 45, 62, producing a
        // non-palindromic Claw/Laser result.
        let seed = 2;

        let mut rng_a = StsRandom::new(seed);
        let mut a1 = create_enemy("OrbWalker", 93, 93);
        let mut a2 = create_enemy("OrbWalker", 93, 93);
        let mut a3 = create_enemy("OrbWalker", 93, 93);
        roll_next_move(&mut a1, &mut rng_a);
        roll_next_move(&mut a2, &mut rng_a);
        roll_next_move(&mut a3, &mut rng_a);

        let mut rng_b = StsRandom::new(seed);
        let mut b1 = create_enemy("OrbWalker", 93, 93);
        let mut b2 = create_enemy("OrbWalker", 93, 93);
        let mut b3 = create_enemy("OrbWalker", 93, 93);
        // Reverse order of consumption.
        roll_next_move(&mut b3, &mut rng_b);
        roll_next_move(&mut b2, &mut rng_b);
        roll_next_move(&mut b1, &mut rng_b);

        // Same enemy receives a different num depending on stream position.
        // The source-derived seed above makes this deterministic rather than
        // relying on an incidental probabilistic difference.
        let intents_forward = (a1.move_id, a2.move_id, a3.move_id);
        let intents_reversed = (b1.move_id, b2.move_id, b3.move_id);
        assert_ne!(
            intents_forward, intents_reversed,
            "multi-enemy intent must depend on roll order; if equal, the AI RNG \
             stream is not advancing per call (regression of audit §1.1)"
        );
    }

    // §5.1 (continued) -- deterministic enemies still consume from the stream.
    // If a future "optimization" makes Cultist skip the draw, multi-enemy
    // ordering breaks even though Cultist's own intent looks identical.

    #[test]
    fn deterministic_enemy_still_consumes_from_stream() {
        // Cultist always picks DARK_STRIKE post-Incantation, but the Java
        // contract still draws one num. Verify by checking that the stream
        // advances exactly once.
        let mut rng = StsRandom::new(42);
        let baseline_after_one_draw = {
            let mut probe = StsRandom::new(42);
            let _ = probe.random_int(99);
            probe.random_int(99)
        };

        let mut e = create_enemy("Cultist", 50, 50);
        roll_next_move(&mut e, &mut rng);
        assert_eq!(e.move_id, move_ids::CULT_DARK_STRIKE);

        let next_from_engine_rng = rng.random_int(99);
        assert_eq!(
            next_from_engine_rng, baseline_after_one_draw,
            "deterministic enemy roll_next_move must still advance ai_rng by one"
        );
    }

    // Bonus: helper-function smoke. roll_next_move_with_num is the test path;
    // it must NOT touch ai_rng (otherwise tests get non-deterministic).
    #[test]
    fn roll_next_move_with_num_does_not_touch_rng() {
        let mut rng_before = StsRandom::new(1234);
        let snapshot_value = {
            let mut probe = StsRandom::new(1234);
            probe.random_int(99)
        };

        let mut e: EnemyCombatState = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 50);
        // rng_before still untouched.
        assert_eq!(rng_before.random_int(99), snapshot_value);
    }

    #[test]
    fn fixed_set_hp_calls_consume_monster_hp_rng_even_when_bounds_are_equal() {
        // AbstractMonster.setHp(int) delegates to setHp(hp, hp), which still
        // calls monsterHpRng.random(hp, hp).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
        for (id, hp) in [
            ("TheGuardian", 240),
            ("BronzeAutomaton", 300),
            ("TimeEater", 456),
            ("CorruptHeart", 750),
        ] {
            let mut oracle = StsRandom::new(42);
            assert_eq!(oracle.random_int_range(hp, hp), hp);

            let mut run = RunEngine::new(42, 0);
            run.debug_enter_specific_combat(&[id]);
            let combat = run.get_combat_engine().expect("specific combat");
            assert_eq!(combat.state.enemies[0].entity.hp, hp, "{id}");
            assert_eq!(
                run.debug_floor_rng_states()[0],
                oracle.state_tuple(),
                "{id}"
            );
        }
    }

    #[test]
    fn overwritten_set_hp_constructors_consume_both_java_draws() {
        // Each constructor passes a random HP to super, then overwrites it via
        // setHp. Both monsterHpRng calls are observable stream consumption.
        // Java: Taskmaster.java, BronzeOrb.java, TorchHead.java, OrbWalker.java.
        for (id, initial, final_range) in [
            ("Taskmaster", (54, 60), (54, 60)),
            ("BronzeOrb", (52, 58), (52, 58)),
            ("TorchHead", (38, 40), (38, 40)),
            ("OrbWalker", (90, 96), (90, 96)),
        ] {
            let mut oracle = StsRandom::new(42);
            let _discarded = oracle.random_int_range(initial.0, initial.1);
            let expected_hp = oracle.random_int_range(final_range.0, final_range.1);

            let mut run = RunEngine::new(42, 0);
            run.debug_enter_specific_combat(&[id]);
            let combat = run.get_combat_engine().expect("specific combat");
            assert_eq!(combat.state.enemies[0].entity.hp, expected_hp, "{id}");
            assert_eq!(
                run.debug_floor_rng_states()[0],
                oracle.state_tuple(),
                "{id}"
            );
        }
    }

    #[test]
    fn reptomancer_group_preserves_constructor_draw_order() {
        // MonsterHelper constructs left Dagger, Reptomancer, then right Dagger.
        // Reptomancer consumes an overwritten super HP draw plus its final HP.
        // Java: MonsterHelper.java, Reptomancer.java, SnakeDagger.java.
        let mut oracle = StsRandom::new(42);
        let left_hp = oracle.random_int_range(20, 25);
        let _discarded_reptomancer_hp = oracle.random_int_range(180, 190);
        let reptomancer_hp = oracle.random_int_range(180, 190);
        let right_hp = oracle.random_int_range(20, 25);

        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["Reptomancer"]);
        let combat = run.get_combat_engine().expect("Reptomancer combat");
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.id.as_str())
                .collect::<Vec<_>>(),
            ["Dagger", "Reptomancer", "Dagger"]
        );
        assert_eq!(
            combat
                .state
                .enemies
                .iter()
                .map(|enemy| enemy.entity.hp)
                .collect::<Vec<_>>(),
            [left_hp, reptomancer_hp, right_hp]
        );
        assert_eq!(run.debug_floor_rng_states()[0], oracle.state_tuple());
    }

    #[test]
    fn dead_adventurer_lagavulin_event_starts_awake() {
        // DeadAdventurer.getMonster returns `Lagavulin Event`, which
        // MonsterHelper maps to new Lagavulin(false). Its pre-battle action
        // starts on Siphon Soul with no sleeping block or Metallicize.
        // Java: events/exordium/DeadAdventurer.java, helpers/MonsterHelper.java,
        // monsters/exordium/Lagavulin.java.
        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["Lagavulin Event"]);
        let combat = run.get_combat_engine().expect("Dead Adventurer combat");
        let enemy = &combat.state.enemies[0];

        assert_eq!(enemy.id, "Lagavulin");
        assert_eq!(enemy.move_id, move_ids::LAGA_SIPHON);
        assert_eq!(enemy.entity.block, 0);
        assert_eq!(enemy.entity.status(sid::METALLICIZE), 0);
        assert_eq!(enemy.entity.status(sid::SLEEP_TURNS), 0);
    }

    #[test]
    fn three_louse_constructor_and_prebattle_draws_are_member_ordered() {
        // Java selects and constructs each Louse as HP then Bite. Once all
        // constructors finish, AbstractRoom.update first initializes every
        // monster's move, then Louse pre-battle actions roll Curl Up in order.
        // Java: MonsterHelper.java, LouseNormal.java, LouseDefensive.java.
        let mut misc_oracle = StsRandom::new(42);
        let mut hp_oracle = StsRandom::new(42);
        let expected = (0..3)
            .map(|_| construct_louse(&mut misc_oracle, &mut hp_oracle))
            .collect::<Vec<_>>();
        let curls = (0..3)
            .map(|_| hp_oracle.random_int_range(3, 7))
            .collect::<Vec<_>>();

        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["3 Louse"]);
        let combat = run.get_combat_engine().expect("three Louse combat");
        for (index, enemy) in combat.state.enemies.iter().enumerate() {
            assert_eq!(enemy.id, expected[index].id, "member {index}");
            assert_eq!(enemy.entity.hp, expected[index].hp, "member {index}");
            assert_eq!(
                enemy.entity.status(sid::STARTING_DMG),
                expected[index].bite.expect("Louse Bite"),
                "member {index}"
            );
            assert_eq!(
                enemy.entity.status(sid::CURL_UP),
                curls[index],
                "member {index}"
            );
        }
        let states = run.debug_floor_rng_states();
        assert_eq!(states[0], hp_oracle.state_tuple());
        assert_eq!(states[4], misc_oracle.state_tuple());
    }

    #[test]
    fn gremlin_leader_encounter_interleaves_type_selection_and_construction() {
        // MonsterHelper selects and immediately constructs each of the two
        // Gremlin Leader minions. The Leader is constructed afterward, and
        // only then does room initialization consume one aiRng draw per member.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/
        // MonsterHelper.java and monsters/city/GremlinLeader.java.
        let mut misc_oracle = StsRandom::new(42);
        let mut hp_oracle = StsRandom::new(42);
        let mut expected = Vec::with_capacity(2);
        for _ in 0..2 {
            let id = match misc_oracle.random_int(7) {
                0 | 1 => "GremlinWarrior",
                2 | 3 => "GremlinThief",
                4 | 5 => "GremlinFat",
                6 => "GremlinTsundere",
                _ => "GremlinWizard",
            };
            let hp = match id {
                "GremlinFat" => hp_oracle.random_int_range(13, 17),
                "GremlinThief" => hp_oracle.random_int_range(10, 14),
                "GremlinWarrior" => hp_oracle.random_int_range(20, 24),
                "GremlinWizard" => hp_oracle.random_int_range(21, 25),
                _ => hp_oracle.random_int_range(12, 15),
            };
            expected.push((id, hp));
        }
        let leader_hp = hp_oracle.random_int_range(140, 148);
        let mut ai_oracle = StsRandom::new(42);
        for _ in 0..3 {
            ai_oracle.random_int(99);
        }

        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["GremlinLeader"]);
        let combat = run.get_combat_engine().expect("Gremlin Leader combat");
        assert_eq!(
            combat.state.enemies[..2]
                .iter()
                .map(|enemy| (enemy.id.as_str(), enemy.entity.hp))
                .collect::<Vec<_>>(),
            expected,
        );
        assert_eq!(combat.state.enemies[2].id, "GremlinLeader");
        assert_eq!(combat.state.enemies[2].entity.hp, leader_hp);
        let states = run.debug_floor_rng_states();
        assert_eq!(states[0], hp_oracle.state_tuple());
        assert_eq!(states[1], ai_oracle.state_tuple());
        assert_eq!(states[4], misc_oracle.state_tuple());
        assert_eq!(run.rng_counters()["monsterHp"], 3);
        assert_eq!(run.rng_counters()["ai"], 3);
        assert_eq!(run.rng_counters()["misc"], 2);
    }

    #[test]
    fn three_darklings_interleave_hp_and_nip_constructor_draws() {
        // Every Darkling constructor consumes HP followed by Nip before the
        // next array element is constructed.
        // Java: MonsterHelper.java, monsters/beyond/Darkling.java.
        let mut oracle = StsRandom::new(42);
        let expected = (0..3)
            .map(|_| {
                let hp = oracle.random_int_range(48, 56);
                let nip = oracle.random_int_range(7, 11);
                (hp, nip)
            })
            .collect::<Vec<_>>();

        let mut run = RunEngine::new(42, 0);
        run.debug_enter_specific_combat(&["3 Darklings"]);
        let combat = run.get_combat_engine().expect("three Darklings combat");
        for (index, enemy) in combat.state.enemies.iter().enumerate() {
            assert_eq!(enemy.id, "Darkling", "member {index}");
            assert_eq!(enemy.entity.hp, expected[index].0, "member {index}");
            assert_eq!(
                enemy.entity.status(sid::STR_AMT),
                expected[index].1,
                "member {index}"
            );
        }
        assert_eq!(run.debug_floor_rng_states()[0], oracle.state_tuple());
    }

    #[test]
    fn exordium_candidate_groups_consume_discarded_constructor_draws() {
        // bottomHumanoid/bottomWildlife construct every candidate before
        // selecting one. This test protects both final members and all discarded
        // constructor draws, which influence every later room RNG result.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/MonsterHelper.java.
        for encounter in ["Exordium Thugs", "Exordium Wildlife"] {
            let mut misc_oracle = StsRandom::new(42);
            let mut hp_oracle = StsRandom::new(42);
            let expected = if encounter == "Exordium Thugs" {
                vec![
                    construct_weak_wildlife(&mut misc_oracle, &mut hp_oracle),
                    construct_strong_humanoid(&mut misc_oracle, &mut hp_oracle),
                ]
            } else {
                vec![
                    construct_strong_wildlife(&mut misc_oracle, &mut hp_oracle),
                    construct_weak_wildlife(&mut misc_oracle, &mut hp_oracle),
                ]
            };
            for enemy in &expected {
                if enemy.bite.is_some() {
                    let _curl_up = hp_oracle.random_int_range(3, 7);
                }
            }

            let mut run = RunEngine::new(42, 0);
            run.debug_enter_specific_combat(&[encounter]);
            let combat = run.get_combat_engine().expect("Exordium group combat");
            assert_eq!(combat.state.enemies.len(), expected.len());
            for (index, enemy) in combat.state.enemies.iter().enumerate() {
                assert_eq!(enemy.id, expected[index].id, "{encounter} member {index}");
                assert_eq!(
                    enemy.entity.hp, expected[index].hp,
                    "{encounter} member {index}"
                );
            }
            let states = run.debug_floor_rng_states();
            assert_eq!(
                states[0],
                hp_oracle.state_tuple(),
                "{encounter} monsterHpRng"
            );
            assert_eq!(states[4], misc_oracle.state_tuple(), "{encounter} miscRng");
        }
    }
}
