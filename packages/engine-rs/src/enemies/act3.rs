use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 3 Basic Enemies
// =========================================================================

pub(super) fn roll_darkling(enemy: &mut EnemyCombatState, num: i32) {
    // Java: Darkling.getMove(num) — halfDead -> Reincarnate; firstMove num<50 Harden/else Nip;
    // else num<40 Chomp (anti-repeat; Java also gates on monster-list index parity which is not
    // modeled here), num<70 Harden/(Nip if lastMove=Harden), else Nip (anti-lastTwo).
    // Secondary aiRng rerolls inside num<40 and num>=70 anti-repeat fallbacks are DEFERRED;
    // those paths fall through to a deterministic fallback that keeps the anti-repeat invariant.
    // Chomp (8x2), Harden (12 block + Reanimated), Nip (8).
    if enemy.entity.hp <= 0 {
        enemy.set_move(move_ids::DARK_REINCARNATE, 0, 0, 0);
        return;
    }
    if num < 40 {
        if !last_move(enemy, move_ids::DARK_CHOMP) {
            enemy.set_move(move_ids::DARK_CHOMP, 8, 2, 0);
        } else {
            // Java retries with aiRng.random(40, 99) — DEFERRED; deterministically pick the
            // next non-repeat slot (Harden if allowed, else Nip).
            if !last_move(enemy, move_ids::DARK_HARDEN) {
                enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
            } else {
                enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
            }
        }
    } else if num < 70 {
        if !last_move(enemy, move_ids::DARK_HARDEN) {
            enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
        } else {
            // Java: if last was Harden, set Nip (byte 3).
            enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
        }
    } else if !last_two_moves(enemy, move_ids::DARK_NIP) {
        enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
    } else {
        // Java retries with aiRng.random(0, 99) — DEFERRED; fall through to a non-repeat pick.
        if !last_move(enemy, move_ids::DARK_CHOMP) {
            enemy.set_move(move_ids::DARK_CHOMP, 8, 2, 0);
        } else {
            enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
        }
    }
}

pub(super) fn roll_orb_walker(enemy: &mut EnemyCombatState, num: i32) {
    // Java: num<40 && !lastTwoMoves(CLAW=2) -> CLAW(15), else if num<40 -> LASER(10)(Burn).
    //       num>=40 && !lastTwoMoves(LASER=1) -> LASER(10)(Burn), else CLAW(15).
    if num < 40 {
        if !last_two_moves(enemy, move_ids::OW_CLAW) {
            enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
        } else {
            enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
            enemy.add_effect(mfx::BURN, 1);
        }
    } else if !last_two_moves(enemy, move_ids::OW_LASER) {
        enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
        enemy.add_effect(mfx::BURN, 1);
    } else {
        enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
    }
}

pub(super) fn roll_spiker(enemy: &mut EnemyCombatState, num: i32) {
    // Java: thornsCount>5 -> ATTACK; num<50 && !lastMove(ATTACK) -> ATTACK; else BUFF_THORNS.
    // thornsCount increments when Buff is applied in takeTurn; tracked here via sid::COUNT.
    let thorns_count = enemy.entity.status(sid::COUNT);
    if thorns_count > 5 {
        enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
        return;
    }
    if num < 50 && !last_move(enemy, move_ids::SPIKER_ATTACK) {
        enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
    } else {
        enemy.set_move(move_ids::SPIKER_BUFF, 0, 0, 0);
        let thorns = enemy.entity.status(sid::THORNS);
        enemy.entity.set_status(sid::THORNS, thorns + 2);
        enemy.entity.set_status(sid::COUNT, thorns_count + 1);
        enemy.add_effect(mfx::THORNS, 2);
    }
}

pub(super) fn roll_repulsor(enemy: &mut EnemyCombatState, num: i32) {
    // Java: num<20 && !lastMove(ATTACK=2) -> ATTACK(11), else DAZE (+2 Daze cards).
    if num < 20 && !last_move(enemy, move_ids::REPULSOR_ATTACK) {
        enemy.set_move(move_ids::REPULSOR_ATTACK, 11, 1, 0);
    } else {
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.add_effect(mfx::DAZE, 2);
    }
}

pub(super) fn roll_exploder(enemy: &mut EnemyCombatState, _num: i32) {
    // Java: getMove ignores num. turnCount<2 -> ATTACK(9), else UNKNOWN/EXPLODE.
    // ExplosivePower handles the 30-damage detonation in Java; we collapse that into
    // an explicit EXPLODER_EXPLODE move for MCTS. turnCount is incremented in takeTurn
    // before getMove (first call sees 1, second sees 2, so explode starts on call #2).
    let count = enemy.entity.status(sid::TURN_COUNT) + 1;
    enemy.entity.set_status(sid::TURN_COUNT, count);

    if count >= 3 {
        // Explode! 30 damage and die.
        enemy.set_move(move_ids::EXPLODER_EXPLODE, 30, 1, 0);
    } else {
        enemy.set_move(move_ids::EXPLODER_ATTACK, 9, 1, 0);
    }
}

pub(super) fn roll_writhing_mass(enemy: &mut EnemyCombatState, num: i32) {
    // Java: firstMove -> num<33 MULTI_HIT / num<66 ATTACK_BLOCK / else ATTACK_DEBUFF.
    // Non-first:
    //   num<10 && !lastMove(BIG_HIT) -> BIG_HIT (else aiRng.random(10,99) — DEFERRED)
    //   num<20 && !usedMegaDebuff && !lastMove(MEGA) -> MEGA_DEBUFF
    //       (else aiRng.randomBoolean(0.1f) secondary — DEFERRED)
    //   num<40 && !lastMove(ATTACK_DEBUFF) -> ATTACK_DEBUFF (else retry — DEFERRED)
    //   num<70 && !lastMove(MULTI_HIT) -> MULTI_HIT (else retry — DEFERRED)
    //   else  && !lastMove(ATTACK_BLOCK) -> ATTACK_BLOCK (else retry — DEFERRED)
    // Deferred retry paths fall through to a deterministic anti-repeat fallback that
    // preserves Java's invariant "never repeat the same move" without spending more RNG.
    let is_first = enemy.move_history.len() <= 1;
    if is_first {
        if num < 33 {
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
        } else if num < 66 {
            enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 15);
        } else {
            enemy.set_move(move_ids::WM_ATTACK_DEBUFF, 10, 1, 0);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::VULNERABLE, 2);
        }
        return;
    }

    let used_mega = enemy.entity.status(sid::USED_MEGA_DEBUFF) != 0;

    if num < 10 {
        if !last_move(enemy, move_ids::WM_BIG_HIT) {
            enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
            return;
        }
        // DEFERRED retry fallback: prefer MULTI_HIT as a safe non-repeat.
        enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
    } else if num < 20 {
        if !used_mega && !last_move(enemy, move_ids::WM_MEGA_DEBUFF) {
            enemy.set_move(move_ids::WM_MEGA_DEBUFF, 0, 0, 0);
            enemy.add_effect(mfx::PAINFUL_STABS, 1);
            enemy.entity.set_status(sid::USED_MEGA_DEBUFF, 1);
            return;
        }
        // DEFERRED secondary roll: fall through to a non-repeat big-hit window.
        if !last_move(enemy, move_ids::WM_BIG_HIT) {
            enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
        } else {
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
        }
    } else if num < 40 {
        if !last_move(enemy, move_ids::WM_ATTACK_DEBUFF) {
            enemy.set_move(move_ids::WM_ATTACK_DEBUFF, 10, 1, 0);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::VULNERABLE, 2);
            return;
        }
        enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
    } else if num < 70 {
        if !last_move(enemy, move_ids::WM_MULTI_HIT) {
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
            return;
        }
        enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 15);
    } else if !last_move(enemy, move_ids::WM_ATTACK_BLOCK) {
        enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 15);
    } else {
        // DEFERRED retry: fall back to Multi Hit to avoid repeating Attack/Block.
        enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
    }
}

/// WrithingMass: Reactive power triggers re-roll when hit. Call this when WM takes damage.
pub fn writhing_mass_reactive_reroll(enemy: &mut EnemyCombatState) {
    // Java: getMove() is called again with a new random number when hit.
    // For MCTS: advance to a different move than current.
    let current = enemy.move_id;
    // Pick the next move in cycle that isn't the current one
    let next = match current {
        x if x == move_ids::WM_BIG_HIT => move_ids::WM_MULTI_HIT,
        x if x == move_ids::WM_MULTI_HIT => move_ids::WM_ATTACK_BLOCK,
        x if x == move_ids::WM_ATTACK_BLOCK => move_ids::WM_ATTACK_DEBUFF,
        x if x == move_ids::WM_ATTACK_DEBUFF => move_ids::WM_BIG_HIT,
        _ => move_ids::WM_MULTI_HIT,
    };
    enemy.move_effects.clear();
    match next {
        x if x == move_ids::WM_BIG_HIT => {
            enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
        }
        x if x == move_ids::WM_MULTI_HIT => {
            enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
        }
        x if x == move_ids::WM_ATTACK_BLOCK => {
            enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 15);
        }
        _ => {
            enemy.set_move(move_ids::WM_ATTACK_DEBUFF, 10, 1, 0);
            enemy.add_effect(mfx::WEAK, 2);
            enemy.add_effect(mfx::VULNERABLE, 2);
        }
    }
}

pub(super) fn roll_spire_growth(enemy: &mut EnemyCombatState, num: i32) {
    // Java: num<50 && !lastTwoMoves(QUICK_TACKLE=1) -> QUICK_TACKLE(16)
    //       else if !player.hasPower(Constricted) && !lastMove(CONSTRICT=2) -> CONSTRICT
    //       else if !lastTwoMoves(SMASH=3) -> SMASH(22)
    //       else -> QUICK_TACKLE (fallback).
    // A17+ path (pre-empt with Constrict if player isn't constricted yet) DEFERRED —
    // depends on ascension + player state visible only in create_enemy.
    // "player has Constricted" is approximated via last_move(CONSTRICT) since SpireGrowth
    // only applies Constricted once itself; this is deterministic and prevents
    // re-applying the debuff mid-combat.
    let player_constricted = last_move(enemy, move_ids::SG_CONSTRICT)
        || enemy.move_history.iter().any(|&m| m == move_ids::SG_CONSTRICT);
    if num < 50 && !last_two_moves(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    } else if !player_constricted && !last_move(enemy, move_ids::SG_CONSTRICT) {
        enemy.set_move(move_ids::SG_CONSTRICT, 0, 0, 0);
        enemy.add_effect(mfx::CONSTRICT, 10);
    } else if !last_two_moves(enemy, move_ids::SG_SMASH) {
        enemy.set_move(move_ids::SG_SMASH, 22, 1, 0);
    } else {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    }
}

pub(super) fn roll_maw(enemy: &mut EnemyCombatState, num: i32) {
    // Java: ++turnCount at top; if !roared -> ROAR (byte 2).
    //       else num<50 && !lastMove(NOMNOM=5) -> NOMNOM (5 x max(1, turnCount/2))
    //       else if lastMove(SLAM=3) || lastMove(NOMNOM=5) -> DROOL (byte 4, +3 Str)
    //       else -> SLAM (byte 3, 25 dmg).
    // "roared" flag is tracked via move history: MAW_ROAR is the init move, so once
    // any non-Roar move has been rolled the flag is effectively true.
    let turn_count = enemy.entity.status(sid::TURN_COUNT) + 1;
    enemy.entity.set_status(sid::TURN_COUNT, turn_count);

    let has_roared = enemy.move_history.iter().any(|&m| m == move_ids::MAW_ROAR);
    if !has_roared {
        enemy.set_move(move_ids::MAW_ROAR, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 3);
        enemy.add_effect(mfx::FRAIL, 3);
        return;
    }

    if num < 50 && !last_move(enemy, move_ids::MAW_NOM) {
        // Java uses turnCount/2 hit count; when that would be <= 1 it still picks
        // NOMNOM (single-hit 10 dmg) instead of Slam.
        let nom_hits = (turn_count / 2).max(1);
        enemy.set_move(move_ids::MAW_NOM, 5, nom_hits, 0);
    } else if last_move(enemy, move_ids::MAW_SLAM) || last_move(enemy, move_ids::MAW_NOM) {
        enemy.set_move(move_ids::MAW_DROOL, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else {
        enemy.set_move(move_ids::MAW_SLAM, 25, 1, 0);
    }
}

pub(super) fn roll_transient(enemy: &mut EnemyCombatState, _num: i32) {
    let count = enemy.entity.status(sid::ATTACK_COUNT) + 1;
    enemy.entity.set_status(sid::ATTACK_COUNT, count);
    // Java: damage list pre-computed as startingDeathDmg + count*10
    // startingDeathDmg = 30 (A2+ = 40). count increments in takeTurn.
    let starting_dmg = enemy.entity.status(sid::STARTING_DMG);
    let base = if starting_dmg > 0 { starting_dmg } else { 30 };
    let dmg = base + count * 10;
    enemy.set_move(move_ids::TRANSIENT_ATTACK, dmg, 1, 0);
}

// =========================================================================
// Act 3 Elites
// =========================================================================

pub(super) fn roll_giant_head(enemy: &mut EnemyCombatState, num: i32) {
    // Java: count<=1 -> decrement (if > -6), set IT_IS_TIME with dmg = startingDeathDmg - count*5.
    //       else: decrement count; num<50 && !lastTwoMoves(GLARE=1) -> GLARE,
    //                              num<50 && lastTwoMoves(GLARE)    -> COUNT(13),
    //                              num>=50 && !lastTwoMoves(COUNT=3) -> COUNT(13),
    //                              else -> GLARE.
    let count = enemy.entity.status(sid::COUNT);
    let starting_death_dmg = {
        let v = enemy.entity.status(sid::STARTING_DEATH_DMG);
        if v > 0 { v } else { 30 }
    };

    if count <= 1 {
        // It Is Time mode
        let new_count = if count > -6 { count - 1 } else { count };
        enemy.entity.set_status(sid::COUNT, new_count);
        let dmg = starting_death_dmg - new_count * 5;
        enemy.set_move(move_ids::GH_IT_IS_TIME, dmg, 1, 0);
        return;
    }

    let new_count = count - 1;
    enemy.entity.set_status(sid::COUNT, new_count);

    if num < 50 {
        if !last_two_moves(enemy, move_ids::GH_GLARE) {
            enemy.set_move(move_ids::GH_GLARE, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 1);
        } else {
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        }
    } else if !last_two_moves(enemy, move_ids::GH_COUNT) {
        enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
    } else {
        enemy.set_move(move_ids::GH_GLARE, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    }
}

pub(super) fn roll_nemesis(enemy: &mut EnemyCombatState, num: i32) {
    // Java: --scytheCooldown at top.
    //   firstMove -> num<50 TRI_ATTACK else BURN.
    //   num<30:
    //     !lastMove(SCYTHE) && scytheCooldown<=0 -> SCYTHE(45), cooldown=2
    //     else randomBoolean() branch (DEFERRED — 50/50 internal roll):
    //       !lastTwoMoves(TRI_ATTACK) -> TRI_ATTACK else BURN;
    //     else !lastMove(BURN) -> BURN else TRI_ATTACK.
    //   num<65:
    //     !lastTwoMoves(TRI_ATTACK) -> TRI_ATTACK
    //     else randomBoolean() branch (DEFERRED):
    //       scytheCooldown>0 -> BURN else SCYTHE(45), cooldown=2;
    //     else BURN.
    //   else:
    //     !lastMove(BURN) -> BURN
    //     else randomBoolean() && scytheCooldown<=0 (DEFERRED) -> SCYTHE(45), cooldown=2
    //     else TRI_ATTACK.
    // Deferred randomBoolean()s above collapse to the "primary" (non-random) branch so the
    // scytheCooldown invariant is respected without a second RNG draw.
    // fireDmg default = 6 (A3+ = 7). Scythe always 45. Burn count = 3 (A18+ = 5).
    let cooldown_in = enemy.entity.status(sid::SCYTHE_COOLDOWN);
    let cooldown = cooldown_in - 1;
    enemy.entity.set_status(sid::SCYTHE_COOLDOWN, cooldown.max(0));

    let fire_dmg = 6;

    // First move
    let first_move = enemy.entity.status(sid::FIRST_MOVE) > 0;
    if first_move {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        if num < 50 {
            enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
        } else {
            enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
            enemy.add_effect(mfx::BURN, 3);
        }
        return;
    }

    if num < 30 {
        if !last_move(enemy, move_ids::NEM_SCYTHE) && cooldown <= 0 {
            enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
            enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
        } else if !last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
            // DEFERRED randomBoolean; pick the primary branch.
            enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
        } else if !last_move(enemy, move_ids::NEM_BURN) {
            enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
            enemy.add_effect(mfx::BURN, 3);
        } else {
            enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
        }
    } else if num < 65 {
        if !last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
            enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
        } else if cooldown <= 0 {
            // DEFERRED randomBoolean; prefer the rare Scythe window when cooldown allows.
            enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
            enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
        } else {
            enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
            enemy.add_effect(mfx::BURN, 3);
        }
    } else if !last_move(enemy, move_ids::NEM_BURN) {
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.add_effect(mfx::BURN, 3);
    } else if cooldown <= 0 {
        // DEFERRED randomBoolean gate; fire Scythe deterministically when off cooldown.
        enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
        enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
    } else {
        enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
    }
}

pub(super) fn roll_reptomancer(enemy: &mut EnemyCombatState, num: i32) {
    // Java: firstMove -> SPAWN (init already handles this; first real roll is non-first).
    //   num<33: !lastMove(SNAKE_STRIKE=1) -> SNAKE_STRIKE(13x2 + Weak 1)
    //           else aiRng.random(33,99) retry -> DEFERRED (fall through to BigBite fallback).
    //   num<66: !lastTwoMoves(SPAWN=2):
    //             canSpawn() -> SPAWN; else -> SNAKE_STRIKE
    //           else (spawned twice in a row) -> SNAKE_STRIKE
    //   else (num>=66): !lastMove(BIG_BITE=3) -> BIG_BITE(30)
    //                   else aiRng.random(65) retry -> DEFERRED.
    // canSpawn() depends on available dagger slots (Reptomancer + 4 snake daggers) which
    // we don't track here; assume always true. Deferred retries collapse to the next
    // non-repeat option.
    if num < 33 {
        if !last_move(enemy, move_ids::REPTO_SNAKE_STRIKE) {
            enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0);
            enemy.add_effect(mfx::WEAK, 1);
        } else {
            // DEFERRED retry -> prefer Big Bite as the only other damage move.
            if !last_move(enemy, move_ids::REPTO_BIG_BITE) {
                enemy.set_move(move_ids::REPTO_BIG_BITE, 30, 1, 0);
            } else {
                enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
            }
        }
    } else if num < 66 {
        if !last_two_moves(enemy, move_ids::REPTO_SPAWN) {
            enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
        } else {
            enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0);
            enemy.add_effect(mfx::WEAK, 1);
        }
    } else if !last_move(enemy, move_ids::REPTO_BIG_BITE) {
        enemy.set_move(move_ids::REPTO_BIG_BITE, 30, 1, 0);
    } else {
        // DEFERRED retry -> fall through to a non-BigBite option.
        if !last_move(enemy, move_ids::REPTO_SNAKE_STRIKE) {
            enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0);
            enemy.add_effect(mfx::WEAK, 1);
        } else {
            enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
        }
    }
}

pub(super) fn roll_snake_dagger(enemy: &mut EnemyCombatState, _num: i32) {
    // Wound (9 + Wound card) -> Explode (25 dmg, dies)
    if last_move(enemy, move_ids::SD_WOUND) {
        enemy.set_move(move_ids::SD_EXPLODE, 25, 1, 0);
    } else {
        enemy.set_move(move_ids::SD_WOUND, 9, 1, 0);
        enemy.add_effect(mfx::WOUND, 1);
    }
}

// =========================================================================
// Act 3 Bosses
// =========================================================================

pub(super) fn roll_awakened_one(enemy: &mut EnemyCombatState, num: i32) {
    // Java AwakenedOne.getMove(num) — two forms:
    //   form1: firstTurn -> SLASH(20);
    //          num<25 && !lastMove(SOUL_STRIKE=2) -> SOUL_STRIKE(6x4);
    //          num<25 && lastMove(SOUL_STRIKE) -> SLASH(20);
    //          !lastTwoMoves(SLASH=1) -> SLASH(20) else -> SOUL_STRIKE(6x4).
    //   form2: firstTurn (of P2) -> DARK_ECHO(40);
    //          num<50 && !lastTwoMoves(SLUDGE=6) -> SLUDGE(18, +Void card);
    //          num<50 && lastTwoMoves(SLUDGE) -> TACKLE(10x3);
    //          !lastTwoMoves(TACKLE=8) -> TACKLE(10x3) else -> SLUDGE(18).
    let phase = enemy.entity.status(sid::PHASE);

    if phase == 1 {
        if num < 25 {
            if !last_move(enemy, move_ids::AO_SOUL_STRIKE) {
                enemy.set_move(move_ids::AO_SOUL_STRIKE, 6, 4, 0);
            } else {
                enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
            }
        } else if !last_two_moves(enemy, move_ids::AO_SLASH) {
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
        } else {
            enemy.set_move(move_ids::AO_SOUL_STRIKE, 6, 4, 0);
        }
    } else {
        if num < 50 {
            if !last_two_moves(enemy, move_ids::AO_SLUDGE) {
                enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
                enemy.add_effect(mfx::VOID, 1);
            } else {
                enemy.set_move(move_ids::AO_TACKLE, 10, 3, 0);
            }
        } else if !last_two_moves(enemy, move_ids::AO_TACKLE) {
            enemy.set_move(move_ids::AO_TACKLE, 10, 3, 0);
        } else {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.add_effect(mfx::VOID, 1);
        }
    }
}

/// Trigger Awakened One rebirth (Phase 1 -> Phase 2).
/// Heals to full, removes all debuffs, enters Phase 2.
pub fn awakened_one_rebirth(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status(sid::PHASE, 2);
    enemy.entity.set_status(sid::CURIOSITY, 0);
    // Remove all debuffs using power registry
    for i in 0..256 {
        if enemy.entity.statuses[i] != 0 {
            let sid = crate::ids::StatusId(i as u16);
            if crate::powers::registry::status_is_debuff(sid) {
                enemy.entity.statuses[i] = 0;
            }
        }
    }
    // Heal to full (second form HP)
    enemy.entity.hp = enemy.entity.max_hp;
    enemy.move_history.clear();
    // First move of Phase 2: Dark Echo
    enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
}

pub(super) fn roll_donu(enemy: &mut EnemyCombatState, _num: i32) {
    // Java: isAttacking flag toggles. Donu starts with isAttacking=false.
    // Circle -> isAttacking=true -> Beam -> isAttacking=false -> repeat.
    // beamDmg: A4+ = 12, else 10. Artifact: A19 = 3, else 2.
    if last_move(enemy, move_ids::DONU_CIRCLE) {
        let bd = { let v = enemy.entity.status(sid::BEAM_DMG); if v > 0 { v } else { 10 } };
        enemy.set_move(move_ids::DONU_BEAM, bd, 2, 0);
    } else {
        enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    }
}

pub(super) fn roll_deca(enemy: &mut EnemyCombatState, _num: i32) {
    // Java: Deca starts with isAttacking=true, alternates.
    // Beam (beamDmg x2 + 2 Daze) then Square (16 block, A19 also +3 Plated Armor).
    // beamDmg: A4+ = 12, else 10. Artifact: A19 = 3, else 2.
    if last_move(enemy, move_ids::DECA_BEAM) {
        enemy.set_move(move_ids::DECA_SQUARE, 0, 0, 16);
    } else {
        let bd = { let v = enemy.entity.status(sid::BEAM_DMG); if v > 0 { v } else { 10 } };
        enemy.set_move(move_ids::DECA_BEAM, bd, 2, 0);
        enemy.add_effect(mfx::DAZE, 2);
    }
}

pub(super) fn roll_time_eater(enemy: &mut EnemyCombatState, num: i32) {
    // Java: HP<maxHP/2 && !usedHaste -> HASTE(5). Else:
    //   num<45: !lastTwoMoves(REVERB=2) -> REVERB (reverbDmg x3);
    //           else aiRng.random(50,99) retry -> DEFERRED.
    //   num<80: !lastMove(HEAD_SLAM=4) -> HEAD_SLAM (headSlamDmg);
    //           else aiRng.randomBoolean(0.66f) -> REVERB (DEFERRED, 66/34 split);
    //                                          else -> RIPPLE(3, 20 block + debuffs).
    //   else:   !lastMove(RIPPLE=3) -> RIPPLE;
    //           else aiRng.random(74) retry -> DEFERRED.
    // Deferred secondary rolls collapse to the branch that best preserves the anti-repeat
    // invariant used by Java (Reverb 3x cap, no repeated HeadSlam/Ripple).
    let reverb_dmg = {
        let v = enemy.entity.status(sid::REVERB_DMG);
        if v > 0 { v } else { 7 }
    };
    let head_slam_dmg = {
        let v = enemy.entity.status(sid::HEAD_SLAM_DMG);
        if v > 0 { v } else { 26 }
    };

    if enemy.entity.hp < enemy.entity.max_hp / 2 && enemy.entity.status(sid::USED_HASTE) == 0 {
        enemy.entity.set_status(sid::USED_HASTE, 1);
        enemy.set_move(move_ids::TE_HASTE, 0, 0, 0);
        enemy.add_effect(mfx::REMOVE_DEBUFFS, 1);
        enemy.add_effect(mfx::HEAL_TO_HALF, 1);
        return;
    }

    if num < 45 {
        if !last_two_moves(enemy, move_ids::TE_REVERBERATE) {
            enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
        } else {
            // DEFERRED retry: fall into the num<80 window's primary branch.
            if !last_move(enemy, move_ids::TE_HEAD_SLAM) {
                enemy.set_move(move_ids::TE_HEAD_SLAM, head_slam_dmg, 1, 0);
                enemy.add_effect(mfx::DRAW_REDUCTION, 1);
            } else {
                enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
                enemy.add_effect(mfx::VULNERABLE, 1);
                enemy.add_effect(mfx::WEAK, 1);
            }
        }
    } else if num < 80 {
        if !last_move(enemy, move_ids::TE_HEAD_SLAM) {
            enemy.set_move(move_ids::TE_HEAD_SLAM, head_slam_dmg, 1, 0);
            enemy.add_effect(mfx::DRAW_REDUCTION, 1);
        } else {
            // DEFERRED randomBoolean(0.66f): lean Reverb if non-repeat, else Ripple.
            if !last_two_moves(enemy, move_ids::TE_REVERBERATE) {
                enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
            } else {
                enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
                enemy.add_effect(mfx::VULNERABLE, 1);
                enemy.add_effect(mfx::WEAK, 1);
            }
        }
    } else if !last_move(enemy, move_ids::TE_RIPPLE) {
        enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
        enemy.add_effect(mfx::VULNERABLE, 1);
        enemy.add_effect(mfx::WEAK, 1);
    } else {
        // DEFERRED retry: fall back to Reverb if possible, else HeadSlam.
        if !last_two_moves(enemy, move_ids::TE_REVERBERATE) {
            enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
        } else {
            enemy.set_move(move_ids::TE_HEAD_SLAM, head_slam_dmg, 1, 0);
            enemy.add_effect(mfx::DRAW_REDUCTION, 1);
        }
    }
}
