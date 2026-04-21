//! Cycle 1.7 — post-execute integration tests for `combat_hooks`.
//!
//! These tests drive `combat_hooks::do_enemy_turns` / `on_enemy_damaged`
//! end-to-end so assertions fire on STATE AFTER intent execution, not just
//! post-roll. Per user directive `feedback_parity_tests_post_execute`:
//!   "Assert post-execute state, not post-roll, when combat_hooks involved."
//!
//! Scope (Cycle 1.7, PR #138 remediation):
//!   1. CorruptHeart `DEBILITATE` → player WEAK/VULN/FRAIL ≥ 2
//!   2. Reptomancer `SPAWN`      → enemy list grows by 2 SnakeDagger
//!   3. CorruptHeart `BEAT_OF_DEATH` → player HP loss per card played
//!   4. Nemesis per-turn INTANGIBLE  → status set at enemy-turn start
//!   5. WrithingMass `REACTIVE`  → intent re-roll on damage (RED, D140)
//!
//! Tests 1-4 lock down existing behaviour; Test 5 is the D140 RED baseline
//! that Cycle 4c flips GREEN by wiring `writhing_mass_reactive_reroll` into
//! `combat_hooks::on_enemy_damaged`.

#[cfg(test)]
mod combat_hooks_integration {
    use crate::combat_hooks;
    use crate::combat_types::mfx;
    use crate::enemies::{self, move_ids};
    use crate::engine::CombatEngine;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::status_ids::sid;
    use crate::tests::support::*;

    /// Standardised engine factory: single enemy, 10 Strikes, 3 energy,
    /// start_combat called so CombatPhase::PlayerTurn is active.
    fn engine_with_single_enemy(enemy: EnemyCombatState) -> CombatEngine {
        let deck = make_deck_n("Strike", 10);
        let state = CombatState::new(80, 80, vec![enemy], deck, 3);
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();
        engine
    }

    // =========================================================================
    // Test 1 — CorruptHeart DEBILITATE: WEAK/VULN/FRAIL on player post-execute
    // =========================================================================

    #[test]
    fn corrupt_heart_debilitate_applies_debuffs_post_execute() {
        // CorruptHeart's opening move at `enemies/mod.rs:785-789` is
        // HEART_DEBILITATE carrying mfx::{VULNERABLE,WEAK,FRAIL} = 2 each.
        // `do_enemy_turns` routes these via `apply_debuff_from_enemy`
        // (combat_hooks.rs:287-294) so `justApplied=true` prevents the
        // same-turn end-of-turn decrement.
        let heart = enemies::create_enemy("CorruptHeart", 750, 750);
        assert_eq!(heart.move_id, move_ids::HEART_DEBILITATE);

        let mut engine = engine_with_single_enemy(heart);
        combat_hooks::do_enemy_turns(&mut engine);

        assert!(
            engine.state.player.status(sid::WEAKENED) >= 2,
            "DEBILITATE must apply Weak ≥ 2 post-execute (combat_hooks.rs:287); \
             got {}",
            engine.state.player.status(sid::WEAKENED),
        );
        assert!(
            engine.state.player.status(sid::VULNERABLE) >= 2,
            "DEBILITATE must apply Vulnerable ≥ 2 post-execute (combat_hooks.rs:290); \
             got {}",
            engine.state.player.status(sid::VULNERABLE),
        );
        assert!(
            engine.state.player.status(sid::FRAIL) >= 2,
            "DEBILITATE must apply Frail ≥ 2 post-execute (combat_hooks.rs:293); \
             got {}",
            engine.state.player.status(sid::FRAIL),
        );
    }

    // =========================================================================
    // Test 2 — Reptomancer SPAWN: enemy list grows by 2 SnakeDagger
    // =========================================================================

    #[test]
    fn reptomancer_spawn_grows_enemy_list_post_execute() {
        // Opening move is REPTO_SPAWN (enemies/mod.rs:719-722). Spawn dispatch
        // lives in combat_hooks.rs:501-504 and appends 2 SnakeDagger(22 HP).
        let repto = enemies::create_enemy("Reptomancer", 180, 180);
        assert_eq!(repto.move_id, move_ids::REPTO_SPAWN);

        let mut engine = engine_with_single_enemy(repto);
        assert_eq!(engine.state.enemies.len(), 1, "precondition: 1 enemy");

        combat_hooks::do_enemy_turns(&mut engine);

        assert_eq!(
            engine.state.enemies.len(),
            3,
            "REPTO_SPAWN must append 2 SnakeDagger post-execute \
             (combat_hooks.rs:501-504)",
        );
        for idx in 1..=2 {
            assert_eq!(
                engine.state.enemies[idx].id, "SnakeDagger",
                "spawned enemy at index {} must be SnakeDagger; got {:?}",
                idx, engine.state.enemies[idx].id,
            );
            assert_eq!(
                engine.state.enemies[idx].entity.max_hp, 22,
                "SnakeDagger max_hp must be 22 per combat_hooks.rs:503",
            );
        }
    }

    // =========================================================================
    // Test 3 — CorruptHeart BEAT_OF_DEATH: player HP loss per card played
    // =========================================================================

    #[test]
    fn beat_of_death_damages_player_per_card_played() {
        // DEF_BEAT_OF_DEATH (powers/defs/card_play.rs:146-164) fires on
        // Trigger::OnAnyCardPlayed, effect DealDamage with
        // AmountSource::StatusValue(sid::BEAT_OF_DEATH).
        // At base (CorruptHeart HP 750), BEAT_OF_DEATH = 1 per mod.rs:800.
        let heart = enemies::create_enemy("CorruptHeart", 750, 750);
        assert_eq!(
            heart.entity.status(sid::BEAT_OF_DEATH),
            1,
            "precondition: A0 CorruptHeart has BEAT_OF_DEATH = 1",
        );

        let mut engine = engine_with_single_enemy(heart);
        engine.state.energy = 10; // ensure 3 Strikes are playable.

        let hp_before = engine.state.player.hp;
        let mut cards_played = 0;
        for _ in 0..3 {
            if play_on_enemy(&mut engine, "Strike", 0) {
                cards_played += 1;
            }
        }
        assert!(
            cards_played >= 1,
            "precondition: at least 1 Strike must have been played",
        );
        let hp_lost = hp_before - engine.state.player.hp;

        assert!(
            hp_lost >= cards_played,
            "BEAT_OF_DEATH(1) must remove ≥ 1 hp per card played; \
             cards_played={} hp_lost={}",
            cards_played,
            hp_lost,
        );
    }

    // =========================================================================
    // Test 4 — Nemesis gains INTANGIBLE at start of enemy turn
    // =========================================================================

    #[test]
    fn nemesis_gains_intangible_at_turn_start_post_execute() {
        // combat_hooks.rs:32-36 applies INTANGIBLE=1 at start of enemy turn
        // iff Nemesis doesn't already carry it. Models Java takeTurn's
        // per-turn power re-application.
        let nemesis = enemies::create_enemy("Nemesis", 185, 185);
        assert_eq!(
            nemesis.entity.status(sid::INTANGIBLE),
            0,
            "precondition: Nemesis lacks INTANGIBLE at creation",
        );

        let mut engine = engine_with_single_enemy(nemesis);
        combat_hooks::do_enemy_turns(&mut engine);

        assert_eq!(
            engine.state.enemies[0].entity.status(sid::INTANGIBLE),
            1,
            "Nemesis must gain INTANGIBLE=1 at enemy-turn start \
             (combat_hooks.rs:32-36)",
        );
    }

    // =========================================================================
    // Test 5 — WrithingMass REACTIVE: intent re-roll on damage (RED, D140)
    // =========================================================================

    #[test]
    fn writhing_mass_reactive_reroll_fires_on_damage() {
        // Java WrithingMass.damage() re-calls rollMove() when Reactive
        // absorbs damage. `writhing_mass_reactive_reroll`
        // (enemies/act3.rs:186-215) implements the re-roll but `on_enemy_damaged`
        // (combat_hooks.rs:528-584) dispatches only Guardian/Lagavulin/
        // SlimeBoss/AwakenedOne/Champ — WrithingMass is absent.
        // D140: WrithingMass ReactivePower never wired.
        //
        // RED until Cycle 4c lands the missing dispatch arm.
        let wm = enemies::create_enemy("WrithingMass", 160, 160);
        let initial_move = wm.move_id;
        assert_eq!(
            initial_move,
            move_ids::WM_MULTI_HIT,
            "precondition: WrithingMass opening move is MULTI_HIT \
             (enemies/mod.rs:674)",
        );

        let mut engine = engine_with_single_enemy(wm);
        engine.deal_damage_to_enemy(0, 5);

        assert_ne!(
            engine.state.enemies[0].move_id, initial_move,
            "WrithingMass Reactive must re-roll intent when damaged \
             (D140 — combat_hooks.rs:528-584 missing WrithingMass arm; \
             `writhing_mass_reactive_reroll` exists at act3.rs:186 but is \
             never called)",
        );
    }
}
