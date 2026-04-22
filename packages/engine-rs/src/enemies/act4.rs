use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::last_move;
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 4 — The Ending
// =========================================================================
//
// RNG-parity notes (Stage D-C):
//   Java `getMove(int num)` for every Act 4 monster IGNORES the `num`
//   parameter — there is no `num < X` threshold ladder. Randomness on Act 4
//   comes exclusively from secondary `AbstractDungeon.aiRng.randomBoolean()`
//   calls inside the `moveCount % 3` switch. Those secondary draws are
//   DEFERRED; we substitute a deterministic anti-repeat fallback that
//   preserves the 50/50 slot's lastMove invariant.
//
//   The `num` parameter is intentionally dropped for SpireShield /
//   SpireSpear (they have no branch that reads it in Java). CorruptHeart
//   keeps `num` in its signature only because the dispatcher passes it for
//   every monster; the body does not consume it.

pub(super) fn roll_spire_shield(enemy: &mut EnemyCombatState) {
    // Java SpireShield.getMove(num) — num is ignored; all randomness from
    // aiRng.randomBoolean() in slot 0. Cycle: moveCount % 3.
    //   slot 0: randomBoolean() -> Fortify ELSE Bash         (DEFERRED boolean)
    //   slot 1: !lastMove(BASH) -> Bash ELSE Fortify         (deterministic)
    //   slot 2: Smash (always)
    // Base damages: Bash 12 (A3+ 14), Smash 34 (A3+ 38), Fortify 30 block all.
    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let bash_dmg = if enemy.ascension >= 3 { 14 } else { 12 };

    match mc % 3 {
        0 => {
            // DEFERRED: Java slot 0 is aiRng.randomBoolean() -> Fortify or Bash.
            // Deterministic fallback: Bash unless last move was Bash, then Fortify.
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, bash_dmg, 1, 0);
                enemy.add_effect(mfx::STRENGTH_DOWN, 1);
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            }
        }
        1 => {
            // Deterministic anti-repeat: Bash iff !lastMove(Bash), else Fortify.
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, bash_dmg, 1, 0);
                enemy.add_effect(mfx::STRENGTH_DOWN, 1);
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            }
        }
        _ => {
            // Smash — always.
            enemy.set_move(move_ids::SHIELD_SMASH, 34, 1, 0);
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}

pub(super) fn roll_spire_spear(enemy: &mut EnemyCombatState) {
    // Java SpireSpear.getMove(num) — num is ignored; randomness only in
    // slot 2. Cycle: moveCount % 3.
    //   slot 0: !lastMove(BURN_STRIKE) -> Burn Strike ELSE Piercer (deterministic)
    //   slot 1: Skewer (10 x skewerCount)                          (deterministic)
    //   slot 2: randomBoolean() -> Piercer ELSE Burn Strike         (DEFERRED boolean)
    // Base damages: Burn Strike 5x2 (A3+ 6x2), Skewer 10, skewerCount 3 (A3+ 4).
    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let skewer_count = enemy.entity.status(sid::SKEWER_COUNT).max(3);

    match mc % 3 {
        0 => {
            // Deterministic: Burn Strike iff last wasn't Burn Strike.
            if !last_move(enemy, move_ids::SPEAR_BURN_STRIKE) {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
                enemy.add_effect(mfx::BURN, 2);
            } else {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.add_effect(mfx::STRENGTH, 2);
            }
        }
        1 => {
            // Skewer: 10 x skewerCount.
            enemy.set_move(move_ids::SPEAR_SKEWER, 10, skewer_count, 0);
        }
        _ => {
            // DEFERRED: Java slot 2 is aiRng.randomBoolean() -> Piercer or Burn Strike.
            // Deterministic fallback: Piercer iff last wasn't Piercer, else Burn Strike.
            if !last_move(enemy, move_ids::SPEAR_PIERCER) {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.add_effect(mfx::STRENGTH, 2);
            } else {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
                enemy.add_effect(mfx::BURN, 2);
            }
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}

pub(super) fn roll_corrupt_heart(enemy: &mut EnemyCombatState, num: i32) {
    // Java CorruptHeart.getMove(num) cycles `moveCount % 3`:
    //   isFirstMove -> Debilitate, isFirstMove=false, return (no increment).
    //   slot 0: randomBoolean() -> Blood Shots ELSE Echo      (D143: num<50 gate)
    //   slot 1: !lastMove(ECHO) -> Echo ELSE Blood Shots      (deterministic)
    //   slot 2: Buff (+2 Str + escalating: Artifact 2, +1 BeatOfDeath,
    //           PainfulStabs, +10 Str, +50 Str thereafter)
    //
    // Note: our spawn init sets Debilitate already (create_enemy). The first
    // `roll` call therefore treats the pre-rolled Debilitate as the isFirstMove
    // emission and proceeds to slot 0 without re-emitting Debilitate. This is
    // functionally equivalent to Java for combat-loop parity.
    let is_first = enemy.entity.status(sid::IS_FIRST_MOVE) > 0;
    if is_first {
        enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
    }

    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let blood_count = enemy.entity.status(sid::BLOOD_HIT_COUNT).max(12);
    let echo_dmg = enemy.entity.status(sid::ECHO_DMG).max(40);

    match mc % 3 {
        0 => {
            // Java `CorruptHeart.java:171-199` slot 0: `aiRng.randomBoolean()`
            // -> BLOOD_SHOTS else ECHO. The aiRng `num` passed through from
            // `roll_next_move` is used as the equivalent 50/50 gate: num < 50
            // matches Java's randomBoolean() true branch (D143).
            if num < 50 {
                enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
            } else {
                enemy.set_move(move_ids::HEART_ECHO, echo_dmg, 1, 0);
            }
        }
        1 => {
            // Deterministic anti-repeat: Echo iff last wasn't Echo, else Blood Shots.
            if !last_move(enemy, move_ids::HEART_ECHO) {
                enemy.set_move(move_ids::HEART_ECHO, echo_dmg, 1, 0);
            } else {
                enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
            }
        }
        _ => {
            // Buff: +2 Strength + escalating bonus scaling on buffCount.
            let buff_count = enemy.entity.status(sid::BUFF_COUNT);
            enemy.set_move(move_ids::HEART_BUFF, 0, 0, 0);
            enemy.add_effect(mfx::STRENGTH, 2);
            match buff_count {
                0 => { enemy.add_effect(mfx::ARTIFACT, 2); }
                1 => { enemy.add_effect(mfx::BEAT_OF_DEATH, 1); }
                2 => { enemy.add_effect(mfx::PAINFUL_STABS, 1); }
                3 => { enemy.add_effect(mfx::STRENGTH_BONUS, 10); }
                _ => { enemy.add_effect(mfx::STRENGTH_BONUS, 50); }
            }
            enemy.entity.set_status(sid::BUFF_COUNT, buff_count + 1);
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}
