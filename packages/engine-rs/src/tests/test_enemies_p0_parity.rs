// Cycle 4b-rest: Act 1 enemy-AI P0 parity tests (D152 / D154 / D155 / D156).
//
// Java references (decompiled sources on disk):
//   decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/GremlinWizard.java
//     L42      — `currentCharge = 1` at construction
//     L66-104  — takeTurn(): case CHARGE(2) increments currentCharge; if ==3
//                setMove to ULTIMATE_BLAST (MOVES[1], byte 1); else setMove CHARGE.
//                case ULTIMATE_BLAST(1) resets currentCharge=0 and setMove CHARGE.
//     L141-144 — getMove(int): always setMove CHARGE (MOVES[0], byte 2).
//     => 3-turn cycle CHARGE -> CHARGE -> ULTIMATE_BLAST -> CHARGE -> CHARGE -> ...
//
//   decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/Lagavulin.java
//     L67      — `debuffTurnCount = 0` at construction
//     L112-123 — takeTurn(): case DEBUFF(1) resets debuffTurnCount=0;
//                case STRONG_ATK(3) increments debuffTurnCount.
//     L209-223 — getMove(int): if isOut && debuffTurnCount < 2:
//                  if lastTwoMoves(STRONG_ATK) -> DEBUFF else STRONG_ATK.
//                else -> DEBUFF.
//     => awake cycle STRONG_ATK -> STRONG_ATK -> DEBUFF -> STRONG_ATK -> ...
//
//   decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/Sentry.java
//     L48-49   — BOLT byte 3 (Dazed card inserter, no damage),
//                BEAM byte 4 (9 dmg attack, no daze).
//     L54      — `firstMove = true` at construction.
//     L132-147 — getMove(int): if firstMove: index % 2 == 0 -> BOLT else BEAM.
//                else: lastMove(BEAM) -> BOLT else BEAM.
//     => idx 0 (even) opens BOLT; idx 1 (odd) opens BEAM; strict alternation after.
//
// ---------------------------------------------------------------------
// Rust label convention
// ---------------------------------------------------------------------
// Rust's `move_ids::SENTRY_BOLT` and `SENTRY_BEAM` are semantically swapped
// relative to Java: the Rust `SENTRY_BEAM` move carries DAZE (= Java BOLT's
// Dazed card insertion), and the Rust `SENTRY_BOLT` move is pure 9-damage
// (= Java BEAM). That inversion is pre-existing and out of scope for this
// cycle (fixing semantics would cascade through SpireMonitor labels, test
// names, and the run.rs:1249 stagger hook). These tests therefore assert
// the Rust-label positioning that matches Java *by effect*:
//   Java idx 0 -> BOLT/daze      == Rust idx 0 -> SENTRY_BEAM (carries DAZE)
//   Java idx 1 -> BEAM/attack    == Rust idx 1 -> SENTRY_BOLT (pure damage)
//
// D155 covers the BOLT/BEAM alternation and D156 covers the first-move
// positional opener; both read the same Java file (Sentry.java L132-147).

#[cfg(test)]
mod act1_enemy_p0_parity_tests {
    use crate::combat_types::mfx;
    use crate::enemies::move_ids;
    use crate::enemies::{create_enemy, roll_next_move_with_num};
    use crate::state::EnemyCombatState;

    // ---------------------------------------------------------------
    // D152 — GremlinWizard 3-turn cycle (CHARGE, CHARGE, ULTIMATE_BLAST)
    // ---------------------------------------------------------------

    #[test]
    fn gremlin_wizard_turns_1_2_charge() {
        // Java GremlinWizard.java L66-96 takeTurn case CHARGE(2):
        //   turn 1 action executes CHARGE, ++currentCharge -> 2 (still != 3),
        //   so the NEXT intent is CHARGE again.
        // In Rust, `create_enemy` pre-sets the turn-1 intent to GREMLIN_PROTECT,
        // then `roll_next_move_with_num` (called after turn 1 executes) must
        // produce the turn-2 intent. Turn 2 must still be CHARGE per Java.
        let mut wizard = create_enemy("GremlinWizard", 25, 25);
        assert_eq!(
            wizard.move_id,
            move_ids::GREMLIN_PROTECT,
            "turn-1 opening must be CHARGE per Java L66-96"
        );
        // Roll after turn 1 action (turn-2 intent).
        roll_next_move_with_num(&mut wizard, 0);
        assert_eq!(
            wizard.move_id,
            move_ids::GREMLIN_PROTECT,
            "turn-2 intent must still be CHARGE (currentCharge=2 != 3)"
        );
    }

    #[test]
    fn gremlin_wizard_turn_3_ultimate_blast() {
        // Java L75-80: after CHARGE executes with currentCharge incrementing to 3,
        // setMove to ULTIMATE_BLAST (MOVES[1], byte 1, damage.get(0).base).
        // Opening CHARGE -> roll -> CHARGE -> roll -> ULTIMATE_BLAST (attack, 25 dmg).
        let mut wizard = create_enemy("GremlinWizard", 25, 25);
        roll_next_move_with_num(&mut wizard, 0); // turn-2 intent
        roll_next_move_with_num(&mut wizard, 0); // turn-3 intent
        assert_eq!(
            wizard.move_id,
            move_ids::GREMLIN_ATTACK,
            "turn-3 intent must be ULTIMATE_BLAST (Java L75-80)"
        );
        assert_eq!(wizard.move_damage(), 25);
        assert_eq!(wizard.move_hits(), 1);
    }

    #[test]
    fn gremlin_wizard_full_cycle_repeats() {
        // Java L84-96 case ULTIMATE_BLAST: resets currentCharge=0 and setMove CHARGE.
        // Full cycle: CHARGE, CHARGE, ULTIMATE_BLAST, CHARGE, CHARGE, ULTIMATE_BLAST, ...
        let mut wizard = create_enemy("GremlinWizard", 25, 25);
        let expected = [
            move_ids::GREMLIN_PROTECT, // turn 1 (opener)
            move_ids::GREMLIN_PROTECT, // turn 2
            move_ids::GREMLIN_ATTACK,  // turn 3 (ULTIMATE)
            move_ids::GREMLIN_PROTECT, // turn 4
            move_ids::GREMLIN_PROTECT, // turn 5
            move_ids::GREMLIN_ATTACK,  // turn 6 (ULTIMATE)
        ];
        assert_eq!(wizard.move_id, expected[0]);
        for (i, &want) in expected.iter().enumerate().skip(1) {
            roll_next_move_with_num(&mut wizard, 0);
            assert_eq!(
                wizard.move_id, want,
                "GremlinWizard turn {} must be {}",
                i + 1,
                if want == move_ids::GREMLIN_ATTACK {
                    "ULTIMATE_BLAST"
                } else {
                    "CHARGE"
                }
            );
        }
    }

    // ---------------------------------------------------------------
    // D154 — Lagavulin 2:1 attack:debuff ratio
    // ---------------------------------------------------------------

    fn wake_lagavulin() -> EnemyCombatState {
        // `create_enemy` starts Lagavulin asleep with SLEEP_TURNS=3. Roll three
        // times to drain sleep and land on the awake LAGA_ATTACK opener.
        let mut laga = create_enemy("Lagavulin", 109, 109);
        roll_next_move_with_num(&mut laga, 0); // sleep 3 -> 2
        roll_next_move_with_num(&mut laga, 0); // sleep 2 -> 1
        roll_next_move_with_num(&mut laga, 0); // sleep 1 -> 0, set LAGA_ATTACK
        assert_eq!(laga.move_id, move_ids::LAGA_ATTACK);
        laga
    }

    #[test]
    fn lagavulin_attack_attack_debuff_cycle() {
        // Java Lagavulin.java L209-223 getMove:
        //   debuffTurnCount starts at 0, increments on STRONG_ATK, resets on DEBUFF.
        //   Awake enemy with debuffTurnCount < 2: lastTwoMoves(STRONG_ATK) ? DEBUFF : STRONG_ATK.
        // Sequence from first awake intent: STRONG_ATK, STRONG_ATK, DEBUFF, STRONG_ATK, STRONG_ATK, DEBUFF, ...
        let mut laga = wake_lagavulin();

        // Turn-2 intent: last move was LAGA_ATTACK (only one), so STRONG_ATK again.
        roll_next_move_with_num(&mut laga, 0);
        assert_eq!(
            laga.move_id,
            move_ids::LAGA_ATTACK,
            "after single ATTACK, next intent must still be ATTACK (Java L211-216 !lastTwoMoves)"
        );

        // Turn-3 intent: last two were both LAGA_ATTACK -> SIPHON_SOUL.
        roll_next_move_with_num(&mut laga, 0);
        assert_eq!(
            laga.move_id,
            move_ids::LAGA_SIPHON,
            "after two ATTACKs, next intent must be SIPHON_SOUL (Java L212-213)"
        );

        // Turn-4 intent: reset — last move SIPHON, not STRONG_ATK -> STRONG_ATK.
        roll_next_move_with_num(&mut laga, 0);
        assert_eq!(laga.move_id, move_ids::LAGA_ATTACK);

        // Turn-5 intent: one ATTACK since SIPHON -> STRONG_ATK again.
        roll_next_move_with_num(&mut laga, 0);
        assert_eq!(laga.move_id, move_ids::LAGA_ATTACK);

        // Turn-6 intent: two ATTACKs since SIPHON -> SIPHON again.
        roll_next_move_with_num(&mut laga, 0);
        assert_eq!(laga.move_id, move_ids::LAGA_SIPHON);
    }

    #[test]
    fn lagavulin_siphon_carries_str_and_dex_debuff() {
        // Java L115-122 takeTurn case DEBUFF(1): ApplyPower Dexterity(debuff=-1),
        // ApplyPower Strength(debuff=-1). At A0-A17 debuff=-1; Rust mirrors via
        // SIPHON_STR / SIPHON_DEX move effects with magnitude 1.
        let mut laga = wake_lagavulin();
        roll_next_move_with_num(&mut laga, 0); // ATTACK
        roll_next_move_with_num(&mut laga, 0); // SIPHON
        assert_eq!(laga.move_id, move_ids::LAGA_SIPHON);
        assert_eq!(laga.effect(mfx::SIPHON_STR), Some(1));
        assert_eq!(laga.effect(mfx::SIPHON_DEX), Some(1));
    }

    // ---------------------------------------------------------------
    // D155 / D156 — Sentry positional opening + BOLT/BEAM alternation
    // ---------------------------------------------------------------
    //
    // Java Sentry.java L132-141 keys the first move on
    // `monsters.lastIndexOf(this) % 2`. In Rust, the engine post-processes
    // Sentries at start_combat using their index inside `state.enemies`.
    // Tests construct a multi-Sentry combat via `CombatEngine::new` +
    // `engine.start_combat()` and inspect each slot's opener.

    use crate::engine::CombatEngine;
    use crate::state::CombatState;

    fn combat_with_sentries(count: usize) -> CombatEngine {
        let enemies: Vec<EnemyCombatState> = (0..count)
            .map(|_| create_enemy("Sentry", 38, 38))
            .collect();
        let state = CombatState::new(80, 80, enemies, Vec::new(), 3);
        let mut engine = CombatEngine::new(state, 42);
        engine.start_combat();
        engine
    }

    #[test]
    fn sentry_position_0_opens_beam() {
        // Java L133-137: monsters.lastIndexOf(this) % 2 == 0 -> BOLT (byte 3,
        // Dazed card inserter). In Rust-label terms, that is SENTRY_BEAM
        // (carries DAZE effect, 0 damage in Java terms). This asserts the
        // D156 positional opener for slot 0.
        let engine = combat_with_sentries(3);
        let idx0 = &engine.state.enemies[0];
        assert_eq!(
            idx0.move_id,
            move_ids::SENTRY_BEAM,
            "Sentry slot 0 must open on SENTRY_BEAM (Java BOLT/Dazed, Rust-label inverted)"
        );
        assert_eq!(idx0.effect(mfx::DAZE), Some(2));
    }

    #[test]
    fn sentry_position_1_opens_bolt() {
        // Java L133-137: monsters.lastIndexOf(this) % 2 != 0 -> BEAM (byte 4,
        // 9-damage attack). Rust-label equivalent is SENTRY_BOLT (pure damage).
        // D155/D156 stack on this: idx 1 is odd, opener is the attack move.
        let engine = combat_with_sentries(3);
        let idx1 = &engine.state.enemies[1];
        assert_eq!(
            idx1.move_id,
            move_ids::SENTRY_BOLT,
            "Sentry slot 1 must open on SENTRY_BOLT (Java BEAM/attack, Rust-label inverted)"
        );
        assert_eq!(idx1.move_damage(), 9);
        assert_eq!(idx1.effect(mfx::DAZE), None);
    }

    #[test]
    fn sentry_position_2_opens_beam() {
        // Third Sentry has `lastIndexOf % 2 == 0` (even), mirroring slot 0.
        let engine = combat_with_sentries(3);
        let idx2 = &engine.state.enemies[2];
        assert_eq!(
            idx2.move_id,
            move_ids::SENTRY_BEAM,
            "Sentry slot 2 must open on SENTRY_BEAM (even-index Java BOLT)"
        );
        assert_eq!(idx2.effect(mfx::DAZE), Some(2));
    }

    #[test]
    fn sentry_position_0_turn_2_is_bolt() {
        // Java L142-146: after first move (BEAM-daze in Rust), lastMove(BEAM) ?
        // BOLT : BEAM. So Sentry idx 0 must alternate BEAM -> BOLT -> BEAM ...
        let mut engine = combat_with_sentries(3);
        roll_next_move_with_num(&mut engine.state.enemies[0], 0);
        assert_eq!(
            engine.state.enemies[0].move_id,
            move_ids::SENTRY_BOLT,
            "Sentry slot 0 turn 2 must flip to SENTRY_BOLT (Java attack)"
        );
        assert_eq!(engine.state.enemies[0].move_damage(), 9);
    }

    #[test]
    fn sentry_position_1_turn_2_is_beam() {
        // Sentry idx 1 opens on BOLT, next roll must flip to BEAM.
        let mut engine = combat_with_sentries(3);
        roll_next_move_with_num(&mut engine.state.enemies[1], 0);
        assert_eq!(
            engine.state.enemies[1].move_id,
            move_ids::SENTRY_BEAM,
            "Sentry slot 1 turn 2 must flip to SENTRY_BEAM (Java Dazed)"
        );
        assert_eq!(engine.state.enemies[1].effect(mfx::DAZE), Some(2));
    }
}
