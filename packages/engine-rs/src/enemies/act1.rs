use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 1 Basic Enemies
// =========================================================================

pub(super) fn roll_jaw_worm(enemy: &mut EnemyCombatState, num: i32) {
    // Java JawWorm.getMove(int num) (decompiled monsters/exordium/JawWorm.java:146):
    //   byte 1 = CHOMP (11 dmg), byte 2 = BELLOW (+3 str, 6 block),
    //   byte 3 = THRASH (7 dmg, 5 block).
    //   if (num < 25):
    //     default CHOMP; if lastMove(CHOMP) sub-roll aiRng.randomBoolean(0.5625)
    //       -> BELLOW (56.25%) else THRASH.
    //   else if (num < 55):
    //     default THRASH; if lastTwoMoves(THRASH) sub-roll 0.357
    //       -> CHOMP (35.7%) else BELLOW.
    //   else:
    //     default BELLOW; if lastMove(BELLOW) sub-roll 0.416
    //       -> CHOMP (41.6%) else THRASH.
    //   First turn picks the default by num: 0-24 CHOMP, 25-54 THRASH, 55-99
    //   BELLOW (~25/30/45 split).
    //   Sub-roll exact parity is deferred -- we don't have a second RNG pull
    //   in the single-num API -- so the fallback picks the dominant branch.
    //   See parity register D-JW-SUBROLL.
    if num < 25 {
        if last_move(enemy, move_ids::JW_CHOMP) {
            // Deferred sub-roll: dominant branch is BELLOW (56.25%).
            enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
            enemy.add_effect(mfx::STRENGTH, 3);
        } else {
            enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
        }
    } else if num < 55 {
        if last_two_moves(enemy, move_ids::JW_THRASH) {
            // Deferred sub-roll: dominant branch is BELLOW (64.3%).
            enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
            enemy.add_effect(mfx::STRENGTH, 3);
        } else {
            enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
        }
    } else if last_move(enemy, move_ids::JW_BELLOW) {
        // Deferred sub-roll: dominant branch is THRASH (58.4%).
        enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
    } else {
        enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
        enemy.add_effect(mfx::STRENGTH, 3);
    }
}

pub(super) fn roll_cultist(enemy: &mut EnemyCombatState, _num: i32) {
    enemy.set_move(move_ids::CULT_DARK_STRIKE, 6, 1, 0);
}

pub(super) fn roll_fungi_beast(enemy: &mut EnemyCombatState, _num: i32) {
    if last_two_moves(enemy, move_ids::FB_BITE) {
        enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if last_move(enemy, move_ids::FB_GROW) {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    }
}

pub(super) fn roll_red_louse(enemy: &mut EnemyCombatState, num: i32) {
    // Java LouseNormal.getMove(int num):
    //   if (num < 25 && !lastMove(GROW)) -> GROW (+3 str)
    //   else if !lastTwoMoves(BITE) -> BITE (5-7 dmg, rolled once at combat-start
    //     via a separate aiRng.random(5,7); parity for that roll is deferred —
    //     see audit E9A1, requires a second RNG pull we don't have in this
    //     single-num API)
    //   else -> GROW
    // Bite damage of 6 here is the expected value of the Java 5-7 roll; full
    // per-combat randomised bite dmg is tracked as a follow-up.
    if num < 25 && !last_move(enemy, move_ids::LOUSE_GROW) {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if !last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    }
}

pub(super) fn roll_green_louse(enemy: &mut EnemyCombatState, num: i32) {
    // Java LouseDefensive.getMove(int num):
    //   if (num < 25 && !lastMove(SPIT_WEB)) -> SPIT_WEB (Weak 2)
    //   else if !lastTwoMoves(BITE) -> BITE (6-8 dmg, rolled once at combat-start
    //     via aiRng.random(6,8); deferred — see audit E9A1)
    //   else -> SPIT_WEB
    // Bite damage of 6 here keeps the pre-existing fixture; full randomised
    // bite dmg is tracked as a follow-up.
    if num < 25 && !last_move(enemy, move_ids::LOUSE_SPIT_WEB) {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else if !last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    }
}

pub(super) fn roll_blue_slaver(enemy: &mut EnemyCombatState, num: i32) {
    // Java SlaverBlue.getMove(int num):
    //   if (num < 40 && !lastMove(STAB) && !lastTwoMoves(STAB)) -> STAB (12 dmg)
    //   else -> RAKE (7 dmg + Weak 1)
    // The STAB branch is gated on both lastMove and lastTwoMoves guards so that
    // after two consecutive STABs (or even one STAB) the slaver must RAKE.
    if num < 40
        && !last_move(enemy, move_ids::BS_STAB)
        && !last_two_moves(enemy, move_ids::BS_STAB)
    {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    } else {
        enemy.set_move(move_ids::BS_RAKE, 7, 1, 0);
        enemy.add_effect(mfx::WEAK, 1);
    }
}

pub(super) fn roll_red_slaver(enemy: &mut EnemyCombatState, num: i32) {
    // Java SlaverRed.getMove(int num):
    //   Guard: firstMove skips ENTANGLE and uses the STAB branch directly.
    //   if (num >= 75 && !usedEntangle && !firstMove) -> ENTANGLE (usedEntangle=true)
    //   else if (num >= 55 && !lastTwoMoves(STAB)) -> STAB (13 dmg)
    //   else if (!lastTwoMoves(SCRAPE)) -> SCRAPE (8 dmg + Vuln 1)
    //   else -> STAB fallback
    //
    // In Rust the harness has already pushed the init move onto `move_history`
    // before this fn runs, so `firstMove` == `history.len() == 1`.
    let is_first_move = enemy.move_history.len() == 1;
    let used_entangle = enemy
        .move_history
        .iter()
        .any(|&m| m == move_ids::RS_ENTANGLE);

    if num >= 75 && !used_entangle && !is_first_move {
        enemy.set_move(move_ids::RS_ENTANGLE, 0, 0, 0);
        enemy.add_effect(mfx::ENTANGLE, 1);
    } else if num >= 55 && !last_two_moves(enemy, move_ids::RS_STAB) {
        enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
    } else if !last_two_moves(enemy, move_ids::RS_SCRAPE) {
        enemy.set_move(move_ids::RS_SCRAPE, 8, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 1);
    } else {
        enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
    }
}

pub(super) fn roll_acid_slime_s(enemy: &mut EnemyCombatState, num: i32) {
    // Java AcidSlime_S.getMove(int num) (A0/A16-):
    //   if (num < 40) -> TACKLE (3 dmg)
    //   else          -> LICK (Weak 1), unless lastMove==LICK in which case
    //                    force TACKLE (anti-repeat).
    // Note: A17+ flips to a strict deterministic alternation starting with
    // LICK; that ascension branch is deferred (see audit E16A1).
    if num < 40 {
        enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
    } else if last_move(enemy, move_ids::AS_LICK) {
        // Anti-repeat: can't LICK twice in a row.
        enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
    } else {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    }
}

pub(super) fn roll_acid_slime_m(enemy: &mut EnemyCombatState, num: i32) {
    // Java AcidSlime_M.getMove(int num) (A0/A16-):
    //   if (num < 30 && !lastTwoMoves(SPIT))   -> CORROSIVE_SPIT (7 dmg + Slimed 1)
    //   else if (num < 70 && !lastMove(TACKLE)) -> TACKLE (10 dmg)
    //   else if !lastTwoMoves(LICK)             -> LICK (Weak 1)
    //   else                                    -> CORROSIVE_SPIT fallback
    if num < 30 && !last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    } else if num < 70 && !last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_TACKLE, 10, 1, 0);
    } else if !last_two_moves(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    } else {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    }
}

pub(super) fn roll_acid_slime_l(enemy: &mut EnemyCombatState, num: i32) {
    // Java AcidSlime_L.getMove(int num) (A0/A16-):
    //   Same probability layout as AcidSlime_M but with bigger numbers and
    //   Slimed 2 on the SPIT branch.
    //   if (num < 30 && !lastTwoMoves(SPIT))   -> CORROSIVE_SPIT (11 dmg + Slimed 2)
    //   else if (num < 70 && !lastMove(TACKLE)) -> TACKLE (16 dmg)
    //   else if !lastTwoMoves(LICK)             -> LICK (Weak 2)
    //   else                                    -> CORROSIVE_SPIT fallback
    // (SplitPower passive — spawn two AcidSlime_M at half HP — is out of scope
    // for this stage; see audit E18A1.)
    if num < 30 && !last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.add_effect(mfx::SLIMED, 2);
    } else if num < 70 && !last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    } else if !last_two_moves(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.add_effect(mfx::SLIMED, 2);
    }
}

pub(super) fn roll_spike_slime_s(enemy: &mut EnemyCombatState, _num: i32) {
    enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
}

pub(super) fn roll_spike_slime_m(enemy: &mut EnemyCombatState, _num: i32) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::FRAIL, 1);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    }
}

pub(super) fn roll_spike_slime_l(enemy: &mut EnemyCombatState, _num: i32) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    }
}

pub(super) fn roll_looter(enemy: &mut EnemyCombatState, num: i32) {
    // Java Looter.getMove(int num):
    //   Turns 1-2: always MUG (10 dmg + steal gold).
    //   Turn 3: 50/50 split via `num < 50`:
    //     num < 50  -> SMOKE_BOMB (11 block, preps escape)
    //     num >= 50 -> MUG-variant (LUNGE: 12 dmg, preps escape)
    //   Turn 4+: ESCAPE.
    // Using LOOTER_SMOKE_BOMB for the smoke branch and re-using LOOTER_MUG for
    // the stab-12 branch keeps the move-id space stable; the Lunge move-id
    // (distinct in Java) is tracked as follow-up since it's out of scope here.
    let turns = enemy.move_history.len();
    if turns < 2 {
        enemy.set_move(move_ids::LOOTER_MUG, 10, 1, 0);
    } else if turns == 2 {
        if num < 50 {
            enemy.set_move(move_ids::LOOTER_SMOKE_BOMB, 0, 0, 11);
        } else {
            // Lunge variant: 12 dmg stab, also preps escape.
            enemy.set_move(move_ids::LOOTER_MUG, 12, 1, 0);
        }
    } else {
        enemy.set_move(move_ids::LOOTER_ESCAPE, 0, 0, 0);
        enemy.is_escaping = true;
    }
}

/// GremlinTsundere / GremlinSneaky per-turn roll.
///
/// Java `GremlinTsundere.getMove(int num)`:
///   if there are other live gremlins in this combat ->
///       PROTECT (gain 7 block for a random ally; 6 block at A7+, 11 at A17+)
///   else ->
///       PUNCTURE (9 dmg solo-bash).
///
/// This function models the "is alone" decision via a caller-supplied boolean
/// because `EnemyCombatState` has no view of other enemies. The match arm in
/// `mod.rs:853` is currently empty (`GremlinTsundere | GremlinSneaky => {}`)
/// and is OUTSIDE THIS STAGE'S EDIT SCOPE — so calling this helper from the
/// dispatch is a follow-up wire-up. Until then, a tsundere still acts as a
/// no-op in combat.
///
/// Block amount defaults to 7 (A0 value); ascension scaling (6 at A7+, 11 at
/// A17+) is deferred, see audit E27A1.
pub fn roll_gremlin_tsundere(enemy: &mut EnemyCombatState, _num: i32, has_other_allies: bool) {
    if has_other_allies {
        // PROTECT — buff that grants block to a random ally; we model it as a
        // Buff intent keyed on GREMLIN_PROTECT so the dispatch/animation code
        // can branch on move_id. The actual ally-block application is a
        // separate combat-hooks concern (out of scope here).
        enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
    } else {
        // PUNCTURE — 9 dmg solo attack. Reuse GREMLIN_ATTACK move_id to stay
        // inside the existing Gremlin move-id table.
        enemy.set_move(move_ids::GREMLIN_ATTACK, 9, 1, 0);
    }
}

pub(super) fn roll_gremlin_simple(enemy: &mut EnemyCombatState, dmg: i32, weak: i32) {
    enemy.set_move(move_ids::GREMLIN_ATTACK, dmg, 1, 0);
    if weak > 0 {
        enemy.add_effect(mfx::WEAK, weak as i16);
    }
}

pub(super) fn roll_gremlin_wizard(enemy: &mut EnemyCombatState) {
    // Java GremlinWizard.java L42 initializes `currentCharge = 1`, L66-96 case
    // CHARGE(2) increments it and only emits ULTIMATE_BLAST when it reaches 3.
    // That yields a 3-turn cycle CHARGE -> CHARGE -> ULTIMATE_BLAST, not the
    // 2-turn alternation the pre-fix Rust implemented.
    //
    // In Rust the turn-1 opener is pre-seeded as GREMLIN_PROTECT in
    // `create_enemy_with_ascension`; `roll_next_move_with_num` pushes the
    // executed move onto `move_history` before dispatching here. Two
    // consecutive PROTECT moves in `move_history` mark the end of a charge
    // cycle, so the next intent is ULTIMATE_BLAST. After ULTIMATE_BLAST
    // executes, last_two_moves(PROTECT) is false and we go back to PROTECT.
    if last_two_moves(enemy, move_ids::GREMLIN_PROTECT) {
        // Ultimate Blast after two charge turns.
        enemy.set_move(move_ids::GREMLIN_ATTACK, 25, 1, 0);
    } else {
        // Charge up (turn 1 or turn 2 of a cycle).
        enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
    }
}

pub(super) fn roll_gremlin_nob(enemy: &mut EnemyCombatState, num: i32) {
    // Java GremlinNob.getMove(int num):
    //   Turn 1 (intent already set to BELLOW at construction).
    //   Turn 2 (first getMove call): num < 33 -> SKULL_BASH (6 dmg + Vuln 2),
    //                                else     -> RUSH (14 dmg).
    //   Turn 3+: always RUSH.
    // In Rust, the harness has already pushed the init move (BELLOW) onto
    // `move_history` before this fn runs. So "turn 2" == `history.len() == 1`
    // with the only entry being BELLOW.
    let just_bellowed = enemy.move_history.len() == 1
        && last_move(enemy, move_ids::NOB_BELLOW);
    if just_bellowed {
        if num < 33 {
            enemy.set_move(move_ids::NOB_SKULL_BASH, 6, 1, 0);
            enemy.add_effect(mfx::VULNERABLE, 2);
        } else {
            enemy.set_move(move_ids::NOB_RUSH, 14, 1, 0);
        }
    } else {
        // Turn 3+: always RUSH.
        enemy.set_move(move_ids::NOB_RUSH, 14, 1, 0);
    }
}

pub(super) fn roll_lagavulin(enemy: &mut EnemyCombatState) {
    let sleep_turns = enemy.entity.status(sid::SLEEP_TURNS);

    if sleep_turns > 0 {
        enemy.entity.set_status(sid::SLEEP_TURNS, sleep_turns - 1);
        if sleep_turns - 1 <= 0 {
            enemy.entity.set_status(sid::METALLICIZE, 0);
            enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
        } else {
            enemy.set_move(move_ids::LAGA_SLEEP, 0, 0, 0);
        }
    } else {
        // Awake — Java Lagavulin.java L209-223 getMove:
        //   isOut && debuffTurnCount < 2:
        //     lastTwoMoves(STRONG_ATK=3) -> DEBUFF, else STRONG_ATK.
        //   else -> DEBUFF.
        //
        // `debuffTurnCount` increments on STRONG_ATK and resets on DEBUFF,
        // yielding the cycle STRONG_ATK -> STRONG_ATK -> SIPHON_SOUL -> ...
        // Rust mirrors that with `last_two_moves(LAGA_ATTACK) -> SIPHON`,
        // which is equivalent because the counter only exceeds 2 precisely
        // when the two most recent moves were both STRONG_ATK.
        if last_two_moves(enemy, move_ids::LAGA_ATTACK) {
            enemy.set_move(move_ids::LAGA_SIPHON, 0, 0, 0);
            enemy.add_effect(mfx::SIPHON_STR, 1);
            enemy.add_effect(mfx::SIPHON_DEX, 1);
        } else {
            enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
        }
    }
}

/// Wake Lagavulin early (e.g. when player deals damage to it while sleeping).
pub fn lagavulin_wake_up(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status(sid::SLEEP_TURNS, 0);
    enemy.entity.set_status(sid::METALLICIZE, 0);
    enemy.set_move(move_ids::LAGA_ATTACK, 18, 1, 0);
}

pub(super) fn roll_sentry(enemy: &mut EnemyCombatState) {
    // Java Sentry.java L142-146: after first move, lastMove(BEAM) ? BOLT : BEAM.
    // Rust labels are semantically inverted (see sentry_fix_first_moves docs),
    // but the alternation is label-symmetric: whichever move was just played,
    // flip to the other. The positional opener is handled separately in
    // `sentry_fix_first_moves`.
    if last_move(enemy, move_ids::SENTRY_BOLT) {
        enemy.set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
        enemy.add_effect(mfx::DAZE, 2);
    } else {
        enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
    }
}

/// Post-process Sentry openers to match Java's positional first-move logic.
///
/// Java `Sentry.getMove(int)` L132-141 reads
/// `AbstractDungeon.getMonsters().monsters.lastIndexOf(this) % 2`:
///   * even-index Sentry -> BOLT (byte 3, Dazed card inserter, no damage)
///   * odd-index  Sentry -> BEAM (byte 4, 9-damage attack)
///
/// In Rust the move_id labels are inverted relative to Java — our
/// `SENTRY_BEAM` carries the DAZE payload (= Java's BOLT) and our
/// `SENTRY_BOLT` is the 9-damage attack (= Java's BEAM). This helper
/// rewrites each Sentry's opener by its position in the enemies vector so
/// the player-visible behavior matches Java:
///   * idx 0, 2, 4, ... -> SENTRY_BEAM opener (daze cards, Java BOLT effect)
///   * idx 1, 3, 5, ... -> SENTRY_BOLT opener (9-damage, Java BEAM effect)
///
/// Called from `CombatEngine::start_combat` after enemy construction so the
/// fix-up runs once per combat and applies to every encounter shape that
/// includes Sentries (not just the hard-coded 3-Sentry sprite in
/// `run.rs`).
pub fn sentry_fix_first_moves(enemies: &mut [EnemyCombatState]) {
    for (idx, enemy) in enemies.iter_mut().enumerate() {
        if enemy.id != "Sentry" {
            continue;
        }
        // Only touch enemies whose opener is still pristine (no moves have
        // been rolled yet). `create_enemy_with_ascension` pushes the opener
        // via `set_move` but leaves `move_history` empty; if a caller has
        // already rolled, respect their state.
        if !enemy.move_history.is_empty() {
            continue;
        }
        if idx % 2 == 0 {
            enemy.set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
            enemy.add_effect(mfx::DAZE, 2);
        } else {
            enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
        }
    }
}

// =========================================================================
// Act 1 Bosses
// =========================================================================

pub(super) fn roll_guardian(enemy: &mut EnemyCombatState) {
    let is_defensive = enemy.entity.status(sid::SHARP_HIDE) > 0;

    if is_defensive {
        if last_move(enemy, move_ids::GUARD_ROLL_ATTACK) {
            enemy.set_move(move_ids::GUARD_TWIN_SLAM, 8, 2, 0);
        } else {
            enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        }
    } else {
        if last_move(enemy, move_ids::GUARD_CHARGING_UP) {
            let fb = { let v = enemy.entity.status(sid::FIERCE_BASH_DMG); if v > 0 { v } else { 32 } };
            enemy.set_move(move_ids::GUARD_FIERCE_BASH, fb, 1, 0);
        } else if last_move(enemy, move_ids::GUARD_FIERCE_BASH) {
            enemy.set_move(move_ids::GUARD_VENT_STEAM, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::VULNERABLE, 2);
        } else if last_move(enemy, move_ids::GUARD_VENT_STEAM) {
            enemy.set_move(move_ids::GUARD_WHIRLWIND, 5, 4, 0);
        } else {
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }
    }
}

/// Check if Guardian should switch to defensive mode after taking damage.
pub fn guardian_check_mode_shift(enemy: &mut EnemyCombatState, damage_dealt: i32) -> bool {
    let threshold = enemy.entity.status(sid::MODE_SHIFT);
    if threshold <= 0 { return false; }

    let current_taken = enemy.entity.status(sid::DAMAGE_TAKEN_THIS_MODE) + damage_dealt;
    enemy.entity.set_status(sid::DAMAGE_TAKEN_THIS_MODE, current_taken);

    if current_taken >= threshold {
        let sha = if threshold >= 40 { 4 } else { 3 };
        enemy.entity.set_status(sid::SHARP_HIDE, sha);
        enemy.entity.set_status(sid::DAMAGE_TAKEN_THIS_MODE, 0);
        enemy.entity.set_status(sid::MODE_SHIFT, threshold + 10);
        enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        enemy.move_history.clear();
        true
    } else {
        false
    }
}

/// Switch Guardian back to offensive mode.
pub fn guardian_switch_to_offensive(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status(sid::SHARP_HIDE, 0);
    enemy.entity.set_status(sid::DAMAGE_TAKEN_THIS_MODE, 0);
    enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
    enemy.move_history.clear();
}

pub(super) fn roll_hexaghost(enemy: &mut EnemyCombatState) {
    let moves_done = enemy.move_history.len();

    // Java: orbActiveCount tracks the cycle position (0-6).
    // After Activate (first turn): Divider on turn 2, then orbActiveCount-based cycle.
    // Cycle: Sear(0), Tackle(1), Sear(2), Inflame(3), Tackle(4), Sear(5), Inferno(6->reset).
    // orbActiveCount resets to 0 after Inferno (Deactivate all orbs).
    // A4+: fireTackleDmg=6, infernoDmg=3. Else 5, 2.
    // A19: strAmount=3, searBurnCount=2. Else 2, 1.
    match moves_done {
        1 => {
            // After Activate: Divider. Damage = player_hp / 12 + 1 (integer division), hit 6 times.
            // Use 7 as default (80hp / 12 + 1 = 7.67 -> 7)
            enemy.set_move(move_ids::HEX_DIVIDER, 7, 6, 0);
        }
        _ => {
            // orbActiveCount-based: starts at 0 after Divider, increments with each orb-activating move
            let orb_count = (moves_done - 2) % 7;
            match orb_count {
                0 | 2 | 5 => {
                    // Sear: 6 damage + burn cards (searBurnCount, default 1)
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    let sbc = { let v = enemy.entity.status(sid::SEAR_BURN_COUNT); if v > 0 { v } else { 1 } };
                    enemy.add_effect(mfx::BURN, sbc as i16);
                }
                1 | 4 => {
                    // Fire Tackle: fireTackleDmg x2 (A4+ = 6, else 5)
                    let ftd = { let v = enemy.entity.status(sid::FIRE_TACKLE_DMG); if v > 0 { v } else { 5 } };
                    enemy.set_move(move_ids::HEX_TACKLE, ftd, 2, 0);
                }
                3 => {
                    // Inflame: 12 block + strAmount Str (A19 = 3, else 2)
                    enemy.set_move(move_ids::HEX_INFLAME, 0, 0, 12);
                    let sa = { let v = enemy.entity.status(sid::STR_AMT); if v > 0 { v } else { 2 } };
                    enemy.add_effect(mfx::STRENGTH, sa as i16);
                }
                _ => {
                    // Inferno: infernoDmg x6 (A4+ = 3, else 2) + upgrade all burns
                    let idmg = { let v = enemy.entity.status(sid::INFERNO_DMG); if v > 0 { v } else { 2 } };
                    enemy.set_move(move_ids::HEX_INFERNO, idmg, 6, 0);
                    enemy.add_effect(mfx::BURN_UPGRADE, 1);
                }
            }
        }
    }
}

/// Set Hexaghost Divider damage based on player HP.
/// Java formula: `d = AbstractDungeon.player.currentHealth / 12 + 1`
/// This is integer division, no ceiling. Per hit = player_hp / 12 + 1, 6 hits.
pub fn hexaghost_set_divider(enemy: &mut EnemyCombatState, player_hp: i32) {
    let per_hit = player_hp / 12 + 1;
    enemy.set_move(move_ids::HEX_DIVIDER, per_hit, 6, 0);
}

pub(super) fn roll_slime_boss(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::SB_STICKY) {
        enemy.set_move(move_ids::SB_PREP_SLAM, 0, 0, 0);
    } else if last_move(enemy, move_ids::SB_PREP_SLAM) {
        enemy.set_move(move_ids::SB_SLAM, 35, 1, 0);
    } else {
        enemy.set_move(move_ids::SB_STICKY, 0, 0, 0);
        enemy.add_effect(mfx::SLIMED, 3);
    }
}

/// Check if Slime Boss should split (HP <= 50%).
pub fn slime_boss_should_split(enemy: &EnemyCombatState) -> bool {
    enemy.entity.hp > 0 && enemy.entity.hp <= enemy.entity.max_hp / 2
}

