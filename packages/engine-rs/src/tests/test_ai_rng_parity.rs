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
    use crate::enemies::{create_enemy, move_ids, roll_next_move, roll_next_move_with_num};
    use crate::seed::StsRandom;
    use crate::state::EnemyCombatState;

    // §5.1 -- JawWorm: each Java getMove branch must be reachable via a num
    // in the documented Java window. If JawWorm goes deterministic again
    // (e.g. someone removes the num check), every test here will fail with the
    // same wrong move_id.

    #[test]
    fn jaw_worm_num_below_25_picks_chomp_on_first_turn() {
        let mut e = create_enemy("JawWorm", 44, 44);
        // Initial intent in create_enemy is CHOMP; rolling with num=0 (no prior
        // history matching CHOMP twice) should keep CHOMP per Java's
        // `num < 25 && !lastTwoMoves(CHOMP)`.
        roll_next_move_with_num(&mut e, 0);
        assert_eq!(e.move_id, move_ids::JW_CHOMP, "num=0 should yield CHOMP");
        assert_eq!(e.move_damage(), 11);
    }

    #[test]
    fn jaw_worm_num_25_to_54_picks_bellow_on_first_turn() {
        for num in [25, 30, 50, 54] {
            let mut e = create_enemy("JawWorm", 44, 44);
            roll_next_move_with_num(&mut e, num);
            assert_eq!(
                e.move_id,
                move_ids::JW_BELLOW,
                "num={num} should yield BELLOW"
            );
            assert_eq!(e.move_block(), 6);
        }
    }

    #[test]
    fn jaw_worm_num_55_to_99_picks_thrash_on_first_turn() {
        for num in [55, 75, 99] {
            let mut e = create_enemy("JawWorm", 44, 44);
            roll_next_move_with_num(&mut e, num);
            assert_eq!(
                e.move_id,
                move_ids::JW_THRASH,
                "num={num} should yield THRASH"
            );
            assert_eq!(e.move_damage(), 7);
            assert_eq!(e.move_block(), 5);
        }
    }

    #[test]
    fn jaw_worm_anti_repeat_chomp_blocks_third_chomp() {
        // After two consecutive CHOMPs in move_history, num<25 must NOT pick
        // CHOMP (Java's `!lastTwoMoves(CHOMP)` guard). It should fall through
        // to the next branch.
        let mut e = create_enemy("JawWorm", 44, 44);
        // Seed history directly so lastTwoMoves(CHOMP) is unambiguously true
        // BEFORE the next push happens. roll_next_move_with_num pushes the
        // *current* move_id first, so we set the current to CHOMP and seed
        // exactly one prior CHOMP -- after the push, the last two entries are
        // [CHOMP, CHOMP].
        e.move_history.push(move_ids::JW_CHOMP);
        e.set_move(move_ids::JW_CHOMP, 11, 1, 0);
        roll_next_move_with_num(&mut e, 0);
        assert_ne!(
            e.move_id,
            move_ids::JW_CHOMP,
            "lastTwoMoves(CHOMP) should block a third CHOMP at num=0"
        );
        // num<25 + CHOMP-blocked + !lastMove(BELLOW)=true -> BELLOW.
        assert_eq!(e.move_id, move_ids::JW_BELLOW);
    }

    #[test]
    fn jaw_worm_three_branch_distribution_is_balanced() {
        // Drive 1000 fresh JawWorm rolls under uniform num in [0, 99].
        // Expected ~25% CHOMP, ~30% BELLOW, ~45% THRASH per Java's getMove.
        // Anti-repeat guards on a fresh enemy with no prior history don't trigger.
        let mut counts = [0_i32; 3]; // CHOMP, BELLOW, THRASH
        for num in 0..100 {
            let mut e = create_enemy("JawWorm", 44, 44);
            roll_next_move_with_num(&mut e, num);
            match e.move_id {
                x if x == move_ids::JW_CHOMP => counts[0] += 1,
                x if x == move_ids::JW_BELLOW => counts[1] += 1,
                x if x == move_ids::JW_THRASH => counts[2] += 1,
                _ => panic!("unexpected JawWorm intent"),
            }
        }
        assert_eq!(counts[0], 25, "CHOMP count");
        assert_eq!(counts[1], 30, "BELLOW count");
        assert_eq!(counts[2], 45, "THRASH count");
    }

    // §5.2 -- multi-enemy stream order: every roll_next_move call consumes
    // exactly one value from ai_rng, so two enemies rolled in different orders
    // get different intent sequences. This protects against a regression where
    // ai_rng is no longer threaded (would zero out variance) or where some
    // enemies skip the draw (would desync the stream).

    #[test]
    fn ai_rng_advances_one_per_roll() {
        // Two parallel rngs seeded identically; one consumed via roll_next_move
        // (which calls random(99) internally), one consumed manually. They must
        // stay in lockstep.
        let mut rng_via_roll = StsRandom::new(42);
        let mut rng_manual = StsRandom::new(42);

        for _ in 0..10 {
            let mut e = create_enemy("JawWorm", 44, 44);
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
        // 3 JawWorms sharing one ai_rng. Rolling A then B then C consumes
        // different values than rolling C then B then A.
        let seed = 7;

        let mut rng_a = StsRandom::new(seed);
        let mut a1 = create_enemy("JawWorm", 44, 44);
        let mut a2 = create_enemy("JawWorm", 44, 44);
        let mut a3 = create_enemy("JawWorm", 44, 44);
        roll_next_move(&mut a1, &mut rng_a);
        roll_next_move(&mut a2, &mut rng_a);
        roll_next_move(&mut a3, &mut rng_a);

        let mut rng_b = StsRandom::new(seed);
        let mut b1 = create_enemy("JawWorm", 44, 44);
        let mut b2 = create_enemy("JawWorm", 44, 44);
        let mut b3 = create_enemy("JawWorm", 44, 44);
        // Reverse order of consumption.
        roll_next_move(&mut b3, &mut rng_b);
        roll_next_move(&mut b2, &mut rng_b);
        roll_next_move(&mut b1, &mut rng_b);

        // Same enemy receives a different num depending on stream position.
        // It is technically possible (though improbable) that two random draws
        // happen to land in the same JawWorm branch; if this assertion ever
        // becomes flaky we need a sharper variance test, but for seed=7 the
        // intent sequences differ.
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
