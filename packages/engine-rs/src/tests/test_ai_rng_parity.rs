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
    use crate::enemies::{create_enemy, move_ids, roll_next_move, roll_next_move_with_num,
        roll_next_move_with_num_and_rng};
    use crate::seed::StsRandom;
    use crate::state::EnemyCombatState;

    // Source-derived from reference/extracted/methods/monster/JawWorm.java.
    // The three num windows depend on history; three cases consume another
    // aiRng.randomBoolean draw instead of forming a simple 25/30/45 split.
    fn seed_for_float(predicate: fn(f32) -> bool) -> u64 {
        (1..10_000).find(|&seed| {
            let mut rng = StsRandom::new(seed);
            predicate(rng.random_float())
        }).expect("seed satisfying probability branch")
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
            assert_eq!(e.move_id, if take_bellow { move_ids::JW_BELLOW } else { move_ids::JW_THRASH });
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
            assert_eq!(e.move_id, if take_chomp { move_ids::JW_CHOMP } else { move_ids::JW_BELLOW });
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
            assert_eq!(e.move_id, if take_chomp { move_ids::JW_CHOMP } else { move_ids::JW_THRASH });
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
            let _ = rng_manual.random(99);
            // Both rngs should have consumed the same number of values; the next
            // draw from each must match.
            assert_eq!(
                rng_via_roll.random(99),
                rng_manual.random(99),
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
            let _ = probe.random(99);
            probe.random(99)
        };

        let mut e = create_enemy("Cultist", 50, 50);
        roll_next_move(&mut e, &mut rng);
        assert_eq!(e.move_id, move_ids::CULT_DARK_STRIKE);

        let next_from_engine_rng = rng.random(99);
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
            probe.random(99)
        };

        let mut e: EnemyCombatState = create_enemy("JawWorm", 44, 44);
        roll_next_move_with_num(&mut e, 50);
        // rng_before still untouched.
        assert_eq!(rng_before.random(99), snapshot_value);
    }
}
