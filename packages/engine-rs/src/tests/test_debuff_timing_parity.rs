//! D59 regression test -- Java-matching `justApplied` behavior for
//! enemy-applied Weak / Vulnerable / Frail.
//!
//! Java's `AbstractPower.atEndOfRound` skips the first decrement when
//! `justApplied==true`, which is set when source is an enemy. Rust used to
//! decrement unconditionally, so a 1-stack Weak landed by an enemy vanished
//! on the same turn it was applied -- dramatically under-modeling enemy
//! debuff pressure (Sentry beam, Boot Steel Pads, Sphere Slam, Time Eater
//! Ripple, etc).
//!
//! See:
//! - `packages/engine-rs/src/powers/debuffs.rs::apply_debuff_from_enemy`
//! - `decompiled/java-src/com/megacrit/cardcrawl/powers/VulnerablePower.java:32-50`
//! - `docs/work_units/parity-deviations-register.md` D59

#[cfg(test)]
mod debuff_timing_parity_tests {
    use crate::powers::{apply_debuff, apply_debuff_from_enemy, decrement_debuffs};
    use crate::state::EntityState;
    use crate::status_ids::sid;

    fn fresh_entity() -> EntityState {
        EntityState::new(80, 80)
    }

    // ------------------------------------------------------------------
    // Enemy-applied debuffs persist their first round (justApplied skip).
    // ------------------------------------------------------------------

    #[test]
    fn enemy_applied_weak_does_not_decrement_first_turn() {
        let mut player = fresh_entity();
        // Enemy hits player with 1-stack Weak.
        let applied = apply_debuff_from_enemy(&mut player, sid::WEAKENED, 1);
        assert!(applied);
        assert_eq!(player.status(sid::WEAKENED), 1);
        assert_eq!(
            player.status(sid::WEAKENED_JUST_APPLIED),
            1,
            "just_applied flag should be set"
        );

        // End of round: debuff should persist because justApplied skips the decrement.
        decrement_debuffs(&mut player);
        assert_eq!(
            player.status(sid::WEAKENED),
            1,
            "enemy-applied Weak should still be 1 after one end-of-round"
        );
        assert_eq!(
            player.status(sid::WEAKENED_JUST_APPLIED),
            0,
            "just_applied flag should be cleared after the first end-of-round"
        );

        // Second end-of-round: now the decrement actually fires.
        decrement_debuffs(&mut player);
        assert_eq!(player.status(sid::WEAKENED), 0);
    }

    #[test]
    fn enemy_applied_vulnerable_does_not_decrement_first_turn() {
        let mut player = fresh_entity();
        apply_debuff_from_enemy(&mut player, sid::VULNERABLE, 2);
        assert_eq!(player.status(sid::VULNERABLE), 2);

        decrement_debuffs(&mut player);
        assert_eq!(
            player.status(sid::VULNERABLE),
            2,
            "first end-of-round should skip decrement"
        );

        decrement_debuffs(&mut player);
        assert_eq!(player.status(sid::VULNERABLE), 1);

        decrement_debuffs(&mut player);
        assert_eq!(player.status(sid::VULNERABLE), 0);
    }

    #[test]
    fn enemy_applied_frail_does_not_decrement_first_turn() {
        let mut player = fresh_entity();
        apply_debuff_from_enemy(&mut player, sid::FRAIL, 1);
        decrement_debuffs(&mut player);
        assert_eq!(
            player.status(sid::FRAIL),
            1,
            "Frail should persist its first end-of-round when enemy-applied"
        );
    }

    // ------------------------------------------------------------------
    // Player-applied debuffs (on enemies) decrement normally -- no skip.
    // ------------------------------------------------------------------

    #[test]
    fn player_applied_weak_decrements_first_turn() {
        let mut enemy = fresh_entity();
        // Player cards use `apply_debuff` directly (no source flag).
        apply_debuff(&mut enemy, sid::WEAKENED, 2);
        assert_eq!(enemy.status(sid::WEAKENED), 2);
        assert_eq!(
            enemy.status(sid::WEAKENED_JUST_APPLIED),
            0,
            "player-applied debuffs should NOT set justApplied"
        );

        decrement_debuffs(&mut enemy);
        assert_eq!(
            enemy.status(sid::WEAKENED),
            1,
            "player-applied Weak should decrement on first end-of-round"
        );
    }

    #[test]
    fn player_applied_vulnerable_decrements_first_turn() {
        let mut enemy = fresh_entity();
        apply_debuff(&mut enemy, sid::VULNERABLE, 3);
        decrement_debuffs(&mut enemy);
        assert_eq!(enemy.status(sid::VULNERABLE), 2);
    }

    // ------------------------------------------------------------------
    // Re-stacking while justApplied is already set.
    // ------------------------------------------------------------------

    #[test]
    fn enemy_stacking_weak_on_already_just_applied_still_skips_first_decrement() {
        let mut player = fresh_entity();
        apply_debuff_from_enemy(&mut player, sid::WEAKENED, 1);
        apply_debuff_from_enemy(&mut player, sid::WEAKENED, 1);
        assert_eq!(player.status(sid::WEAKENED), 2);
        assert_eq!(player.status(sid::WEAKENED_JUST_APPLIED), 1);

        decrement_debuffs(&mut player);
        assert_eq!(
            player.status(sid::WEAKENED),
            2,
            "enemy re-applies do not burn the justApplied skip"
        );
    }

    // ------------------------------------------------------------------
    // Artifact / Ginger / Turnip block the debuff -- justApplied not set.
    // ------------------------------------------------------------------

    #[test]
    fn artifact_blocks_enemy_debuff_and_does_not_set_just_applied() {
        let mut player = fresh_entity();
        player.set_status(sid::ARTIFACT, 1);
        let applied = apply_debuff_from_enemy(&mut player, sid::VULNERABLE, 1);
        assert!(!applied, "Artifact should block the debuff");
        assert_eq!(player.status(sid::VULNERABLE), 0);
        assert_eq!(
            player.status(sid::VULNERABLE_JUST_APPLIED),
            0,
            "justApplied must not be set when the debuff is blocked"
        );
    }

    #[test]
    fn ginger_blocks_enemy_weak_and_does_not_set_just_applied() {
        let mut player = fresh_entity();
        player.set_status(sid::HAS_GINGER, 1);
        let applied = apply_debuff_from_enemy(&mut player, sid::WEAKENED, 1);
        assert!(!applied);
        assert_eq!(player.status(sid::WEAKENED), 0);
        assert_eq!(player.status(sid::WEAKENED_JUST_APPLIED), 0);
    }
}
