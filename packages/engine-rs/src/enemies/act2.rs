use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 2 Basic Enemies
// =========================================================================

pub(super) fn roll_chosen(enemy: &mut EnemyCombatState, num: i32) {
    let used_hex = enemy.move_history.iter().any(|&m| m == move_ids::CHOSEN_HEX);

    // After first turn (Poke): use Hex
    if !used_hex {
        enemy.set_move(move_ids::CHOSEN_HEX, 0, 0, 0);
        enemy.add_effect(mfx::HEX, 1);
        return;
    }
    // Java getMove: if !lastMove(DEBILITATE) && !lastMove(DRAIN) -> debuff turn
    //   num<50 -> Debilitate(10 dmg + Vuln 2)
    //   else   -> Drain(player Weak 3, self +3 Str)
    // else -> attack turn
    //   num<40 -> Zap(18)
    //   else   -> Poke(5x2)
    if !last_move(enemy, move_ids::CHOSEN_DEBILITATE) && !last_move(enemy, move_ids::CHOSEN_DRAIN) {
        if num < 50 {
            enemy.set_move(move_ids::CHOSEN_DEBILITATE, 10, 1, 0);
            enemy.add_effect(mfx::VULNERABLE, 2);
        } else {
            enemy.set_move(move_ids::CHOSEN_DRAIN, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 3);
            enemy.add_effect(mfx::STRENGTH, 3);
        }
        return;
    }
    if num < 40 {
        enemy.set_move(move_ids::CHOSEN_ZAP, 18, 1, 0);
    } else {
        enemy.set_move(move_ids::CHOSEN_POKE, 5, 2, 0);
    }
}

pub(super) fn roll_mugger(enemy: &mut EnemyCombatState, _num: i32) {
    let turns = enemy.move_history.len();
    if turns < 2 {
        enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
    } else if turns == 2 {
        // SmokeBomb or BigSwipe. Use BigSwipe (more threatening)
        enemy.set_move(move_ids::MUGGER_BIG_SWIPE, 16, 1, 0);
    } else if last_move(enemy, move_ids::MUGGER_BIG_SWIPE) {
        enemy.set_move(move_ids::MUGGER_SMOKE_BOMB, 0, 0, 11);
    } else if last_move(enemy, move_ids::MUGGER_SMOKE_BOMB) {
        enemy.set_move(move_ids::MUGGER_ESCAPE, 0, 0, 0);
        enemy.is_escaping = true;
    } else {
        enemy.set_move(move_ids::MUGGER_MUG, 10, 1, 0);
    }
}

pub(super) fn roll_byrd(enemy: &mut EnemyCombatState, num: i32) {
    let is_flying = enemy.entity.status(sid::FLIGHT) > 0;

    if !is_flying {
        // Grounded: Headbutt after Stunned, else Fly Up
        if last_move(enemy, move_ids::BYRD_STUNNED) {
            enemy.set_move(move_ids::BYRD_HEADBUTT, 3, 1, 0);
        } else {
            enemy.set_move(move_ids::BYRD_FLY_UP, 0, 0, 0);
            enemy.entity.set_status(sid::FLIGHT, 3);
        }
        return;
    }

    // Flying Java getMove:
    //   num<50: Peck(1x5) — unless lastTwoMoves(Peck), then secondary roll
    //   num<70: Swoop(12) — unless lastMove(Swoop), then secondary roll
    //   else:   Caw (self +1 Str) — unless lastMove(Caw), then Swoop/Peck secondary roll
    // Secondary rolls consume an extra aiRng draw which we don't have; use
    // deterministic fallbacks that honor the anti-repeat predicate.
    if num < 50 {
        if last_two_moves(enemy, move_ids::BYRD_PECK) {
            // Java: 0.4f → Swoop, else Caw. Deterministic fallback: Swoop.
            enemy.set_move(move_ids::BYRD_SWOOP, 12, 1, 0);
        } else {
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
        }
    } else if num < 70 {
        if last_move(enemy, move_ids::BYRD_SWOOP) {
            // Java: 0.375f → Caw, else Peck. Deterministic fallback: Caw.
            enemy.set_move(move_ids::BYRD_CAW, 0, 0, 0);
            enemy.add_effect(mfx::STRENGTH, 1);
        } else {
            enemy.set_move(move_ids::BYRD_SWOOP, 12, 1, 0);
        }
    } else if last_move(enemy, move_ids::BYRD_CAW) {
        // Java: 0.2857f → Swoop, else Peck. Deterministic fallback: Peck.
        enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
    } else {
        enemy.set_move(move_ids::BYRD_CAW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 1);
    }
}

pub(super) fn roll_shelled_parasite(enemy: &mut EnemyCombatState, num: i32) {
    // First turn in Java is a 50/50 randomBoolean between DoubleStrike and LifeSuck,
    // but our create_enemy already seeded the first move (Double Strike) so we only
    // emulate the num-based getMove branches here.
    //
    // Java getMove:
    //   num<20 && !lastMove(FELL) -> Fell(18 + Frail 2)
    //   num<20 && lastMove(FELL)  -> re-roll via aiRng (deferred: needs extra aiRng)
    //   num<60 && !lastTwoMoves(DOUBLE_STRIKE) -> Double Strike(6x2)
    //   num<60 && lastTwoMoves(DS)             -> Life Suck(10)
    //   else  && !lastTwoMoves(LIFE_SUCK) -> Life Suck(10)
    //   else  && lastTwoMoves(LIFE_SUCK)  -> Double Strike
    if num < 20 {
        if !last_move(enemy, move_ids::SP_FELL) {
            enemy.set_move(move_ids::SP_FELL, 18, 1, 0);
            enemy.add_effect(mfx::FRAIL, 2);
        } else {
            // Fallback for the aiRng re-roll branch: pick the next available move
            // using a deterministic tie-break (Double Strike unless recently used).
            if !last_two_moves(enemy, move_ids::SP_DOUBLE_STRIKE) {
                enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
            } else {
                enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
                enemy.add_effect(mfx::HEAL, 10);
            }
        }
    } else if num < 60 {
        if !last_two_moves(enemy, move_ids::SP_DOUBLE_STRIKE) {
            enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
        } else {
            enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
            enemy.add_effect(mfx::HEAL, 10);
        }
    } else if !last_two_moves(enemy, move_ids::SP_LIFE_SUCK) {
        enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
        enemy.add_effect(mfx::HEAL, 10);
    } else {
        enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
    }
}

pub(super) fn roll_snake_plant(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove:
    //   num<65 && !lastTwoMoves(CHOMP) -> Chomp(7x3)
    //   num<65 && lastTwoMoves(CHOMP)  -> Spores
    //   num>=65 && !lastMove(SPORES)   -> Spores
    //   num>=65 && lastMove(SPORES)    -> Chomp
    if num < 65 {
        if last_two_moves(enemy, move_ids::SNAKE_CHOMP) {
            enemy.set_move(move_ids::SNAKE_SPORES, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::FRAIL, 2);
        } else {
            enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
        }
    } else if last_move(enemy, move_ids::SNAKE_SPORES) {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::SNAKE_SPORES, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.add_effect(mfx::FRAIL, 2);
    }
}

pub(super) fn roll_centurion(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove:
    //   num>=65 && !lastTwoMoves(PROTECT) && !lastTwoMoves(FURY):
    //     aliveCount>1 -> Protect(block to random ally)
    //     else         -> Fury(6x3)
    //   else:
    //     !lastTwoMoves(SLASH) -> Slash(12)
    //     else:
    //       aliveCount>1 -> Protect
    //       else         -> Fury
    // (aliveCount is engine state not threaded here; deferred: needs
    //  ally-count parameter. We default to Protect since Centurion almost
    //  always pairs with Mystic, matching the most common encounter.)
    if num >= 65
        && !last_two_moves(enemy, move_ids::CENT_PROTECT)
        && !last_two_moves(enemy, move_ids::CENT_FURY)
    {
        enemy.set_move(move_ids::CENT_PROTECT, 0, 0, 15);
        enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 15);
        return;
    }
    if !last_two_moves(enemy, move_ids::CENT_SLASH) {
        enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
    } else {
        enemy.set_move(move_ids::CENT_PROTECT, 0, 0, 15);
        enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 15);
    }
}

pub(super) fn roll_mystic(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove (with aliveCount HP delta gating the heal):
    //   needToHeal>15 && !lastTwoMoves(HEAL) -> Heal  (ally HP delta; not threaded here)
    //   num>=40 && !lastTwoMoves(ATTACK)     -> Attack(8 + Frail 2)
    //   !lastTwoMoves(BUFF)                   -> Buff(+2 Str to all allies)
    //   else                                  -> Attack(8 + Frail 2)
    //
    // needToHeal requires access to ally HP which the roll signature doesn't
    // provide. We keep the existing Rust convention of using MYSTIC_HEAL_USED
    // as a one-shot proxy for the first heal, then fall back to the num-based
    // Attack/Buff tree for subsequent turns.
    let used_heal = enemy.entity.status(sid::MYSTIC_HEAL_USED);
    if used_heal == 0
        && last_two_moves(enemy, move_ids::MYSTIC_ATTACK)
        && !last_two_moves(enemy, move_ids::MYSTIC_HEAL)
    {
        enemy.set_move(move_ids::MYSTIC_HEAL, 0, 0, 0);
        enemy.add_effect(mfx::HEAL_LOWEST_ALLY, 16);
        enemy.entity.set_status(sid::MYSTIC_HEAL_USED, 1);
        return;
    }
    if num >= 40 && !last_two_moves(enemy, move_ids::MYSTIC_ATTACK) {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
        return;
    }
    if !last_two_moves(enemy, move_ids::MYSTIC_BUFF) {
        enemy.set_move(move_ids::MYSTIC_BUFF, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 2);
    } else {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    }
}

pub(super) fn roll_book_of_stabbing(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove:
    //   num<15 && lastMove(BIG_STAB)  -> Stab++
    //   num<15 && !lastMove(BIG_STAB) -> BigStab (stabCount increments only on A18+)
    //   num>=15 && lastTwoMoves(STAB) -> BigStab
    //   num>=15 && !lastTwoMoves(STAB)-> Stab++
    let stab_count = enemy.entity.status(sid::STAB_COUNT);
    if num < 15 {
        if last_move(enemy, move_ids::BOOK_BIG_STAB) {
            let new_count = stab_count + 1;
            enemy.entity.set_status(sid::STAB_COUNT, new_count);
            enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
        } else {
            enemy.set_move(move_ids::BOOK_BIG_STAB, 21, 1, 0);
        }
    } else if last_two_moves(enemy, move_ids::BOOK_STAB) {
        enemy.set_move(move_ids::BOOK_BIG_STAB, 21, 1, 0);
    } else {
        let new_count = stab_count + 1;
        enemy.entity.set_status(sid::STAB_COUNT, new_count);
        enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
    }
}

pub(super) fn roll_gremlin_leader(enemy: &mut EnemyCombatState, _num: i32) {
    // Rally (summon), Encourage (block + Str to all allies), Stab (6x3)
    if last_move(enemy, move_ids::GL_RALLY) {
        enemy.set_move(move_ids::GL_ENCOURAGE, 0, 0, 6);
        enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 3);
        enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 6);
    } else if last_move(enemy, move_ids::GL_ENCOURAGE) {
        enemy.set_move(move_ids::GL_STAB, 6, 3, 0);
    } else {
        enemy.set_move(move_ids::GL_RALLY, 0, 0, 0);
    }
}

pub(super) fn roll_taskmaster(enemy: &mut EnemyCombatState, _num: i32) {
    // Always Scouring Whip (7 damage + Wound card to discard)
    enemy.set_move(move_ids::TASK_SCOURING_WHIP, 7, 1, 0);
    enemy.add_effect(mfx::WOUND, 1);
}

pub(super) fn roll_spheric_guardian(enemy: &mut EnemyCombatState) {
    // Pattern: Initial Block -> Frail Attack -> Big Attack -> Block Attack -> repeat
    if last_move(enemy, move_ids::SPHER_INITIAL_BLOCK) {
        enemy.set_move(move_ids::SPHER_FRAIL_ATTACK, 10, 1, 0);
        enemy.add_effect(mfx::FRAIL, 5);
    } else if last_move(enemy, move_ids::SPHER_BIG_ATTACK) {
        enemy.set_move(move_ids::SPHER_BLOCK_ATTACK, 10, 1, 15);
    } else if last_move(enemy, move_ids::SPHER_BLOCK_ATTACK) || last_move(enemy, move_ids::SPHER_FRAIL_ATTACK) {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    } else {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    }
}

pub(super) fn roll_snecko(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove (first-turn Glare handled at create_enemy):
    //   num<40 -> Tail(8 + Vuln 2)
    //   else   -> if lastTwoMoves(BITE) Tail, else Bite(15)
    if num < 40 {
        enemy.set_move(move_ids::SNECKO_TAIL, 8, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
        return;
    }
    if last_two_moves(enemy, move_ids::SNECKO_BITE) {
        enemy.set_move(move_ids::SNECKO_TAIL, 8, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
    } else {
        enemy.set_move(move_ids::SNECKO_BITE, 15, 1, 0);
    }
}

pub(super) fn roll_bear(enemy: &mut EnemyCombatState, _num: i32) {
    // Bear Hug (debuff) -> Maul (18) -> Lunge (9 + 9 block) -> cycle
    if last_move(enemy, move_ids::BEAR_HUG) {
        enemy.set_move(move_ids::BEAR_MAUL, 18, 1, 0);
    } else if last_move(enemy, move_ids::BEAR_MAUL) {
        enemy.set_move(move_ids::BEAR_LUNGE, 9, 1, 9);
    } else {
        enemy.set_move(move_ids::BEAR_HUG, 0, 0, 0);
        enemy.add_effect(mfx::DEX_DOWN, 2);
    }
}

pub(super) fn roll_bandit_leader(enemy: &mut EnemyCombatState, _num: i32) {
    // Mock -> Agonizing Slash (10 + Weak 2) -> Cross Slash (15) -> cycle
    if last_move(enemy, move_ids::BANDIT_MOCK) {
        enemy.set_move(move_ids::BANDIT_AGONIZE, 10, 1, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else if last_move(enemy, move_ids::BANDIT_AGONIZE) {
        enemy.set_move(move_ids::BANDIT_CROSS_SLASH, 15, 1, 0);
    } else {
        enemy.set_move(move_ids::BANDIT_MOCK, 0, 0, 0);
    }
}

// =========================================================================
// Act 2 Bosses
// =========================================================================

pub(super) fn roll_bronze_automaton(enemy: &mut EnemyCombatState, _num: i32) {
    let fd = { let v = enemy.entity.status(sid::FLAIL_DMG); if v > 0 { v } else { 7 } };
    let bd = { let v = enemy.entity.status(sid::BEAM_DMG); if v > 0 { v } else { 45 } };
    let sa = { let v = enemy.entity.status(sid::STR_AMT); if v > 0 { v } else { 3 } };
    let ba = { let v = enemy.entity.status(sid::BLOCK_AMT); if v > 0 { v } else { 9 } };
    if last_move(enemy, move_ids::BA_SPAWN_ORBS) || last_move(enemy, move_ids::BA_STUNNED) || last_move(enemy, move_ids::BA_BOOST) {
        enemy.set_move(move_ids::BA_FLAIL, fd, 2, 0);
    } else if last_move(enemy, move_ids::BA_FLAIL) {
        let turns = enemy.move_history.len();
        if turns >= 4 {
            enemy.set_move(move_ids::BA_HYPER_BEAM, bd, 1, 0);
        } else {
            enemy.set_move(move_ids::BA_BOOST, 0, 0, ba);
            enemy.add_effect(mfx::STRENGTH, sa as i16);
        }
    } else if last_move(enemy, move_ids::BA_HYPER_BEAM) {
        enemy.set_move(move_ids::BA_STUNNED, 0, 0, 0);
    } else {
        enemy.set_move(move_ids::BA_FLAIL, fd, 2, 0);
    }
}

pub(super) fn roll_bronze_orb(enemy: &mut EnemyCombatState, num: i32) {
    // Java getMove:
    //   !usedStasis && num>=25 -> Stasis(one-shot); usedStasis := true
    //   num>=70 && !lastTwoMoves(SUPPORT) -> Support(12 block to Automaton)
    //   !lastTwoMoves(BEAM) -> Beam(8)
    //   else -> Support
    let used_stasis = enemy.move_history.iter().any(|&m| m == move_ids::BO_STASIS);
    if !used_stasis && num >= 25 {
        enemy.set_move(move_ids::BO_STASIS, 0, 0, 0);
        return;
    }
    if num >= 70 && !last_two_moves(enemy, move_ids::BO_SUPPORT) {
        enemy.set_move(move_ids::BO_SUPPORT, 0, 0, 12);
        return;
    }
    if !last_two_moves(enemy, move_ids::BO_BEAM) {
        enemy.set_move(move_ids::BO_BEAM, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::BO_SUPPORT, 0, 0, 12);
    }
}

pub(super) fn roll_champ(enemy: &mut EnemyCombatState, num: i32) {
    let num_turns = enemy.entity.status(sid::NUM_TURNS) + 1;
    enemy.entity.set_status(sid::NUM_TURNS, num_turns);

    let str_amt = enemy.entity.status(sid::STR_AMT).max(2);
    let forge_amt = enemy.entity.status(sid::FORGE_AMT).max(5);
    let block_amt = enemy.entity.status(sid::BLOCK_AMT).max(15);
    let slash_dmg = enemy.entity.status(sid::SLASH_DMG).max(16);
    let slap_dmg = enemy.entity.status(sid::SLAP_DMG).max(12);

    let threshold_reached_now = enemy.entity.hp <= enemy.entity.max_hp / 2;

    // Phase 2 trigger: Anger (remove debuffs, gain 3*strAmt Str)
    if threshold_reached_now && enemy.entity.status(sid::THRESHOLD_REACHED) == 0 {
        enemy.entity.set_status(sid::THRESHOLD_REACHED, 1);
        enemy.set_move(move_ids::CHAMP_ANGER, 0, 0, 0);
        // Java: Anger gives 3*strAmt Strength (not strAmt)
        enemy.add_effect(mfx::STRENGTH, (str_amt * 3) as i16);
        enemy.add_effect(mfx::REMOVE_DEBUFFS, 1);
        return;
    }

    // Phase 2: Java getMove alternates Execute/FaceSlap/HeavySlash/Gloat via RNG.
    // Uses lastMove + lastMoveBefore so Execute triggers at least every other turn.
    if enemy.entity.status(sid::THRESHOLD_REACHED) > 0 {
        // len()>=2 lets us peek at the second-to-last entry cheaply.
        let hist = &enemy.move_history;
        let last_before = if hist.len() >= 2 { hist[hist.len() - 2] } else { -1 };
        // Java: if !lastMove(EXECUTE) && !lastMoveBefore(EXECUTE) -> Execute
        if !last_move(enemy, move_ids::CHAMP_EXECUTE)
            && last_before != move_ids::CHAMP_EXECUTE
        {
            enemy.set_move(move_ids::CHAMP_EXECUTE, 10, 2, 0);
            return;
        }
        // Otherwise fall through to the Phase 1 RNG tree.
    }

    // Phase 1: Taunt at numTurns==4 (reset cycle).
    if num_turns == 4 && enemy.entity.status(sid::THRESHOLD_REACHED) == 0 {
        enemy.set_move(move_ids::CHAMP_TAUNT, 0, 0, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.entity.set_status(sid::NUM_TURNS, 0);
        return;
    }

    // Java getMove:
    //   (num<=15 A0..A18 | num<=30 A19+) && !lastMove(DEFENSIVE) && forgeTimes<threshold
    //     -> Defensive Stance (byte 2, block + Metallicize)
    //   num<=30 && !lastMove(GLOAT) && !lastMove(DEFENSIVE)
    //     -> Gloat (byte 5, +strAmt Str)
    //   num<=55 && !lastMove(FACE_SLAP)
    //     -> Face Slap (byte 4, slapDmg + Frail 2 + Vuln 2)
    //   !lastMove(HEAVY_SLASH) -> Heavy Slash (byte 1, slashDmg)
    //   else                   -> Face Slap
    //
    // forgeTimes/forgeThreshold/ascensionLevel aren't modeled; we gate Defensive
    // on !lastMove(DEFENSIVE) only and use the A0..A18 num<=15 threshold.
    if num <= 15 && !last_move(enemy, move_ids::CHAMP_DEFENSIVE) {
        // Defensive Stance: blockAmt + Metallicize forge_amt (Metallicize
        // effect not threaded through move_effects; deferred until mfx adds it).
        enemy.set_move(move_ids::CHAMP_DEFENSIVE, 0, 0, block_amt);
        let _ = forge_amt; // reserved for Metallicize plumbing
        // Java: Defensive Stance increments forgeTimes (used by A19+ forge threshold).
        let forge_times = enemy.entity.status(sid::FORGE_TIMES);
        enemy.entity.set_status(sid::FORGE_TIMES, forge_times + 1);
        return;
    }
    if num <= 30
        && !last_move(enemy, move_ids::CHAMP_GLOAT)
        && !last_move(enemy, move_ids::CHAMP_DEFENSIVE)
    {
        enemy.set_move(move_ids::CHAMP_GLOAT, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, str_amt as i16);
        return;
    }
    if num <= 55 && !last_move(enemy, move_ids::CHAMP_FACE_SLAP) {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
        return;
    }
    if !last_move(enemy, move_ids::CHAMP_HEAVY_SLASH) {
        enemy.set_move(move_ids::CHAMP_HEAVY_SLASH, slash_dmg, 1, 0);
    } else {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
    }
}

pub(super) fn roll_collector(enemy: &mut EnemyCombatState, num: i32) {
    let fd = { let v = enemy.entity.status(sid::FIREBALL_DMG); if v > 0 { v } else { 18 } };
    let sa = { let v = enemy.entity.status(sid::STR_AMT); if v > 0 { v } else { 3 } };
    let ba = { let v = enemy.entity.status(sid::BLOCK_AMT); if v > 0 { v } else { 15 } };
    let turns = enemy.move_history.len();
    let ult_used = enemy.move_history.iter().any(|&m| m == move_ids::COLL_MEGA_DEBUFF);

    // Java getMove:
    //   initialSpawn                          -> Spawn (handled at create_enemy)
    //   turnsTaken>=3 && !ultUsed             -> MegaDebuff
    //   num<=25 && isMinionDead && !lastMove(5) -> Revive (deferred: needs minion-dead signal)
    //   num<=70 && !lastTwoMoves(FIREBALL)    -> Fireball
    //   !lastMove(BUFF)                        -> Buff
    //   else                                   -> Fireball
    if turns == 4 && !ult_used {
        enemy.set_move(move_ids::COLL_MEGA_DEBUFF, 0, 0, 0);
        enemy.add_effect(mfx::VULNERABLE, 3);
        enemy.add_effect(mfx::WEAK, 3);
        enemy.add_effect(mfx::FRAIL, 3);
        return;
    }
    if num <= 70 && !last_two_moves(enemy, move_ids::COLL_FIREBALL) {
        enemy.set_move(move_ids::COLL_FIREBALL, fd, 1, 0);
        return;
    }
    if !last_move(enemy, move_ids::COLL_BUFF) {
        enemy.set_move(move_ids::COLL_BUFF, 0, 0, ba);
        enemy.add_effect(mfx::STRENGTH, sa as i16);
    } else {
        enemy.set_move(move_ids::COLL_FIREBALL, fd, 1, 0);
    }
}

