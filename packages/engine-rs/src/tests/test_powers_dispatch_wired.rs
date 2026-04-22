//! Cycle 3 — dispatch-wiring integration tests for D70 / D89 / D111.
//!
//! These tests assert that power triggers defined in `buffs.rs`
//! (`process_start_of_turn` / `process_end_of_turn` / `process_end_of_round`)
//! or in PowerDefs actually fire during production turn flow. The pre-merge
//! audit (2026-04-21) confirmed all three helpers were ONLY called from
//! tests, making Equilibrium decrement (D70) and Fasting energy drain (D89)
//! silent no-ops. D111 additionally calls out pre-draw vs post-draw timing
//! mismatches for DemonForm / Brutality / NoxiousFumes.
//!
//! Coverage:
//!   1. Equilibrium (D70) — end-of-round decrement via engine.end_turn()
//!   2. Fasting    (D89) — energy drain at start of player turn
//!   3. Metallicize       — regression guard (PowerDef `TurnEnd` dispatch)
//!   4. Combust           — regression guard (PowerDef `TurnEnd` dispatch)
//!   5. Regeneration      — regression guard (inline end-of-turn at engine.rs:1522)
//!   6. Demon Form        — regression guard (PowerDef `TurnStart*` dispatch)
//!
//! Per `feedback_parity_tests_post_execute`: assertions fire on post-execute
//! state (turn-flow completed) rather than post-roll (helper-result structs).

#[cfg(test)]
mod powers_dispatch_wired {
    use crate::effects::runtime::GameEvent;
    use crate::effects::trigger::Trigger;
    use crate::engine::{CombatEngine, CombatPhase};
    use crate::state::CombatState;
    use crate::status_ids::sid;
    use crate::tests::support::*;

    /// Single dummy enemy that never attacks (move_damage 0, 0 hits). Lets
    /// us advance turn flow without player HP noise.
    fn engine_for_turn_flow(player_hp: i32) -> CombatEngine {
        let deck = make_deck_n("Strike", 10);
        // JawWorm at (50, 50) with move_damage=0 / hits=0 means no chip damage
        // once intent rolls; we still call `start_combat` so phase is PlayerTurn.
        let enemies = vec![enemy("JawWorm", 50, 50, 1, 0, 0)];
        let state = CombatState::new(player_hp, player_hp, enemies, deck, 3);
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();
        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
        engine
    }

    // =========================================================================
    // Test 1 — D70: Equilibrium decrements at end-of-round
    // =========================================================================
    //
    // Java `EquilibriumPower.atEndOfRound` (decompiled at
    // `decompiled/java-src/com/megacrit/cardcrawl/powers/EquilibriumPower.java:48-54`)
    // reduces power by 1 each round. Audit D70 / D84 confirmed
    // `decrement_equilibrium` is only called from `process_end_of_turn`,
    // which is only called from unit tests. Live engine path never decrements
    // Equilibrium, so Watcher's "retain hand for N turns" cue effectively
    // lasts forever.
    //
    // RED pre-fix: EQUILIBRIUM status stays at 3 after end_turn.
    // GREEN post-fix: EQUILIBRIUM status is 2 after one end_turn round.

    #[test]
    fn equilibrium_decrements_at_end_of_turn() {
        let mut engine = engine_for_turn_flow(80);
        engine.state.player.set_status(sid::EQUILIBRIUM, 3);

        end_turn(&mut engine);

        assert_eq!(
            engine.state.player.status(sid::EQUILIBRIUM),
            2,
            "D70: Equilibrium must decrement by 1 per round (Java \
             EquilibriumPower.atEndOfRound). Pre-fix Rust never decrements \
             because `process_end_of_round` is only called from tests.",
        );
    }

    // =========================================================================
    // Test 2 — D89: Fasting (EnergyDownPower) drains 1 energy at turn start
    // =========================================================================
    //
    // Java `EnergyDownPower.atStartOfTurn` (decompiled at
    // `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EnergyDownPower.java:52-55`)
    // calls `LoseEnergyAction(amount)` pre-draw. Audit D89: Fasting card
    // sets FASTING=1 but engine never drains the energy because
    // `process_start_of_turn` is only called from tests.
    //
    // RED pre-fix: energy equals max_energy after start_player_turn (no drain).
    // GREEN post-fix: energy == max_energy - 1 after drain fires.

    #[test]
    fn fasting_drains_energy_at_player_start_of_turn() {
        let mut engine = engine_for_turn_flow(80);
        // Apply FASTING=1 before the start-of-turn drain pass fires on turn 2.
        engine.state.player.set_status(sid::FASTING, 1);
        let max_energy = engine.state.max_energy;
        assert!(
            max_energy >= 2,
            "precondition: base max_energy must be >= 2 so drain is observable",
        );

        // Advance past the current turn. On turn 2, start_player_turn resets
        // energy to max_energy then triggers TurnStart dispatch. The Fasting
        // PowerDef (to be added in Cycle 3) should drain 1 energy.
        end_turn(&mut engine);

        assert_eq!(
            engine.phase, CombatPhase::PlayerTurn,
            "precondition: back on player turn after end_turn roundtrip",
        );
        assert_eq!(
            engine.state.energy,
            max_energy - 1,
            "D89: Fasting must drain FASTING={} energy at start-of-turn (Java \
             EnergyDownPower.atStartOfTurn). Pre-fix energy={} (no drain).",
            engine.state.player.status(sid::FASTING),
            engine.state.energy,
        );
    }

    // =========================================================================
    // Test 3 — Metallicize regression guard (PowerDef `TurnEnd`)
    // =========================================================================
    //
    // `powers/defs/turn_end.rs::DEF_METALLICIZE` wires `TurnEnd` to
    // `GainBlock(StatusValue(sid::METALLICIZE))`. This test asserts the
    // declarative dispatch path still grants block post-execute even after
    // the D70/D89/D111 remediation touches turn-flow code.

    #[test]
    fn metallicize_block_applies_at_end_of_turn() {
        // Use direct `emit_event(TurnEnd)` so we capture block BEFORE
        // start_player_turn's block-reset path zeros it for turn 2
        // (engine.rs:1145-1167).
        let mut engine = engine_for_turn_flow(80);
        engine.state.player.set_status(sid::METALLICIZE, 5);
        engine.rebuild_effect_runtime();
        assert_eq!(
            engine.state.player.block, 0,
            "precondition: no starting block",
        );

        engine.emit_event(GameEvent::empty(Trigger::TurnEnd));

        assert_eq!(
            engine.state.player.block, 5,
            "Metallicize 5 must grant 5 block on Trigger::TurnEnd dispatch \
             (PowerDef DEF_METALLICIZE at powers/defs/turn_end.rs:15-33). \
             Actual block: {}",
            engine.state.player.block,
        );
    }

    // =========================================================================
    // Test 4 — Combust regression guard (PowerDef `TurnEnd`)
    // =========================================================================
    //
    // `powers/defs/turn_end.rs::DEF_COMBUST` fires 2 effects at `TurnEnd`:
    // (1) DealDamage(Player, Fixed(1))  — fixed 1 HP loss.
    // (2) DealDamage(AllEnemies, StatusValue(sid::COMBUST)) — N to each enemy.
    // Asserts the declarative dispatch deals the enemy damage; HP-loss side
    // is covered elsewhere.

    #[test]
    fn combust_damage_fires_at_end_of_turn() {
        let mut engine = engine_for_turn_flow(80);
        engine.state.player.set_status(sid::COMBUST, 5);
        let enemy_hp_before = engine.state.enemies[0].entity.hp;

        end_turn(&mut engine);

        let enemy_hp_after = engine.state.enemies[0].entity.hp;
        assert!(
            enemy_hp_before - enemy_hp_after >= 5,
            "Combust 5 must deal >=5 damage to enemy at end-of-turn \
             (PowerDef DEF_COMBUST at powers/defs/turn_end.rs:63-88). \
             before={} after={} delta={}",
            enemy_hp_before,
            enemy_hp_after,
            enemy_hp_before - enemy_hp_after,
        );
    }

    // =========================================================================
    // Test 5 — Regeneration regression guard (inline end-of-turn)
    // =========================================================================
    //
    // Java `RegenPower.atEndOfTurn` (decompiled at
    // `decompiled/java-src/com/megacrit/cardcrawl/powers/RegenPower.java:34-38`)
    // heals `amount` at end-of-turn. Rust currently implements this inline
    // at `engine.rs:1522-1526` (outside the PowerDef system). This test
    // locks down that inline behaviour so the Cycle 3 wiring changes don't
    // accidentally double-fire or drop the heal.

    #[test]
    fn regeneration_heals_at_end_of_turn() {
        let mut engine = engine_for_turn_flow(80);
        // Damage the player so the heal is observable (start at 50 / 80 max).
        engine.state.player.hp = 50;
        engine.state.player.set_status(sid::REGENERATION, 3);

        end_turn(&mut engine);

        assert_eq!(
            engine.state.player.hp, 53,
            "Regeneration 3 must heal 3 at end-of-turn (engine.rs:1522-1526, \
             Java RegenPower.atEndOfTurn). Got hp={}",
            engine.state.player.hp,
        );
        // Java RegenPower does NOT decrement. Rust currently does (see D74
        // register row). This test asserts the end-of-turn tick itself
        // happened — the decrement parity is tracked separately by D74.
        assert!(
            engine.state.player.status(sid::REGENERATION) <= 3,
            "REGENERATION stacks must not grow; observed {}",
            engine.state.player.status(sid::REGENERATION),
        );
    }

    // =========================================================================
    // Test 6 — Demon Form regression guard (PowerDef `TurnStartPostDraw`)
    // =========================================================================
    //
    // Java `DemonFormPower.atStartOfTurnPostDraw` (decompiled at
    // `decompiled/java-src/com/megacrit/cardcrawl/powers/DemonFormPower.java:32-36`)
    // applies StrengthPower(amount) after draw. Pre-Cycle-3 PowerDef wires
    // DemonForm on `Trigger::TurnStart` (pre-draw, D111). Cycle 3 moves it
    // to `TurnStartPostDraw` to match Java. Either trigger fires on the
    // next turn start after end_turn(), so this test locks down the net
    // outcome (STR == DEMON_FORM stacks).

    #[test]
    fn demon_form_grants_strength_at_start_of_turn() {
        let mut engine = engine_for_turn_flow(80);
        engine.state.player.set_status(sid::DEMON_FORM, 2);
        assert_eq!(
            engine.state.player.status(sid::STRENGTH),
            0,
            "precondition: no starting STR",
        );

        // end_turn triggers enemy turns (no damage with hits=0) then calls
        // start_player_turn, which emits TurnStart + TurnStartPostDraw.
        // DemonForm's PowerDef grants STR == DEMON_FORM on whichever of
        // those fires first.
        end_turn(&mut engine);

        assert_eq!(
            engine.state.player.status(sid::STRENGTH),
            2,
            "Demon Form 2 must grant 2 STR on next turn start (PowerDef \
             DEF_DEMON_FORM at powers/defs/turn_start.rs:17-37; Java \
             DemonFormPower.atStartOfTurnPostDraw). Got STR={}",
            engine.state.player.status(sid::STRENGTH),
        );
    }
}
