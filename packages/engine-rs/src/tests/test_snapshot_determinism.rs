//! Snapshot determinism regression tests — merge-blockers F1/F2/F3.
//!
//! These tests are currently RED. They fail because:
//!   F1 / D162: CombatSnapshotV1 drops ai_rng state (`training_contract.rs:523-555`)
//!   F2 / D163: EnemySnapshotV1 drops move_history (`training_contract.rs:506-521`)
//!   F3 / D164: combat_state_hash ignores RNG streams (`search.rs:1087-1138`)
//!
//! They will flip GREEN in Cycle 2 once `training_contract.rs` serializes
//! `engine.ai_rng.state_tuple()` + `enemy.move_history`, and `search.rs`
//! hashes both RNG streams.
//!
//! Cross-refs:
//! - `docs/work_units/parity-deviations-register.md` D162-D165
//! - `docs/work_units/audit-reports/pre-merge-triage-2026-04-21.md` §F1-F4
//! - Plan: Cycle 1.1 (red baseline) → Cycle 2 (flip green)

#[cfg(test)]
mod snapshot_determinism {
    use crate::enemies::roll_next_move;
    use crate::search::combat_state_hash;
    use crate::tests::support::{engine_with, make_deck};
    use crate::training_contract::{
        combat_engine_from_snapshot, combat_snapshot_from_combat, CombatSnapshotV1,
    };

    /// F1 / D162 — ai_rng state_tuple must survive snapshot→JSON→restore.
    #[test]
    fn ai_rng_state_survives_snapshot_roundtrip() {
        let mut engine = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        // JawWorm is probabilistic: one roll consumes one num from ai_rng.
        roll_next_move(&mut engine.state.enemies[0], &mut engine.ai_rng);
        let original_state = engine.ai_rng.state_tuple();
        assert_ne!(
            original_state,
            (0, 0, 0),
            "precondition: ai_rng should have advanced past default",
        );

        // Roundtrip via JSON to catch any serde field omission.
        let snapshot = combat_snapshot_from_combat(&engine);
        let json = serde_json::to_string(&snapshot).expect("snapshot serialize");
        let decoded: CombatSnapshotV1 =
            serde_json::from_str(&json).expect("snapshot deserialize");
        let restored = combat_engine_from_snapshot(&decoded);

        assert_eq!(
            restored.ai_rng.state_tuple(),
            original_state,
            "ai_rng state_tuple must survive snapshot→JSON→restore roundtrip \
             (F1 / D162 — training_contract.rs:523-555 missing ai_rng fields)",
        );
    }

    /// F2 / D163 — enemy.move_history must survive snapshot→restore.
    #[test]
    fn enemy_move_history_survives_snapshot_roundtrip() {
        let mut engine = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        // Three consecutive rolls populate move_history; JawWorm's getMove
        // consults `last_move` / `last_two_moves` so history is load-bearing.
        for _ in 0..3 {
            roll_next_move(&mut engine.state.enemies[0], &mut engine.ai_rng);
        }
        let original_history = engine.state.enemies[0].move_history.clone();
        assert!(
            original_history.len() >= 3,
            "precondition: history should be populated after 3 rolls, got {}",
            original_history.len(),
        );

        let snapshot = combat_snapshot_from_combat(&engine);
        let restored = combat_engine_from_snapshot(&snapshot);

        assert_eq!(
            restored.state.enemies[0].move_history, original_history,
            "enemy move_history must survive snapshot roundtrip \
             (F2 / D163 — training_contract.rs:506-521 missing move_history field)",
        );
    }

    /// F1+F2 integration — a snapshot→restore followed by one more roll
    /// must produce the same move_id as an untouched twin engine.
    #[test]
    fn snapshot_continuation_produces_same_next_move() {
        let mut original = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        let mut twin = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);

        // Advance both engines one step to populate ai_rng + move_history.
        roll_next_move(&mut original.state.enemies[0], &mut original.ai_rng);
        roll_next_move(&mut twin.state.enemies[0], &mut twin.ai_rng);
        assert_eq!(
            original.state.enemies[0].move_id, twin.state.enemies[0].move_id,
            "sanity: original and twin are identical after first roll",
        );

        // Roundtrip original through a snapshot; twin is untouched.
        let snapshot = combat_snapshot_from_combat(&original);
        let mut restored = combat_engine_from_snapshot(&snapshot);

        // Continue: one more roll on each. Restored should match twin exactly.
        roll_next_move(&mut restored.state.enemies[0], &mut restored.ai_rng);
        roll_next_move(&mut twin.state.enemies[0], &mut twin.ai_rng);

        assert_eq!(
            restored.state.enemies[0].move_id, twin.state.enemies[0].move_id,
            "move_id after continuation must match untouched twin \
             (F1+F2 / D162,D163 — ai_rng+move_history both drive intent)",
        );
        assert_eq!(
            restored.state.enemies[0].move_history,
            twin.state.enemies[0].move_history,
            "move_history after continuation must match untouched twin (F2 / D163)",
        );
        assert_eq!(
            restored.ai_rng.state_tuple(),
            twin.ai_rng.state_tuple(),
            "ai_rng state_tuple after continuation must match untouched twin (F1 / D162)",
        );
    }

    /// F3 / D164 — `combat_state_hash` must hash RNG streams. Two engines
    /// with identical gameplay surface but divergent ai_rng states must
    /// hash differently, otherwise MCTS transposition table conflates
    /// genuinely distinct futures.
    #[test]
    fn combat_state_hash_distinguishes_ai_rng_divergence() {
        let engine_a = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        let mut engine_b = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        // Advance engine_b's ai_rng WITHOUT changing any other gameplay surface:
        // no enemy rolls, no card plays — only the ai_rng counter moves.
        for _ in 0..5 {
            let _ = engine_b.ai_rng.random(99);
        }
        assert_ne!(
            engine_a.ai_rng.state_tuple(),
            engine_b.ai_rng.state_tuple(),
            "precondition: engine_b's ai_rng should have advanced past engine_a's",
        );

        let hash_a = combat_state_hash(&engine_a);
        let hash_b = combat_state_hash(&engine_b);
        assert_ne!(
            hash_a, hash_b,
            "combat_state_hash must differ when ai_rng state_tuple differs \
             (F3 / D164 — search.rs:1087-1138 ignores engine.ai_rng + engine.rng)",
        );
    }

    /// F3 / D164 (companion) — the combat `rng` stream also must be hashed.
    /// Draw/shuffle RNG divergence without gameplay-surface divergence must
    /// hash differently.
    #[test]
    fn combat_state_hash_distinguishes_combat_rng_divergence() {
        let engine_a = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        let mut engine_b = engine_with(make_deck(&["Strike", "Defend"]), 44, 11);
        for _ in 0..5 {
            let _ = engine_b.rng.random(99);
        }
        assert_ne!(
            engine_a.rng.state_tuple(),
            engine_b.rng.state_tuple(),
            "precondition: engine_b's rng should have advanced past engine_a's",
        );

        let hash_a = combat_state_hash(&engine_a);
        let hash_b = combat_state_hash(&engine_b);
        assert_ne!(
            hash_a, hash_b,
            "combat_state_hash must differ when rng state_tuple differs \
             (F3 / D164 — search.rs:1087-1138 ignores engine.rng state)",
        );
    }
}
