use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::seed::StsRandom;
use crate::status_ids::sid;

// =========================================================================
// Act 1 Basic Enemies
// =========================================================================

pub(super) fn roll_jaw_worm(enemy: &mut EnemyCombatState, num: i32, ai_rng: &mut StsRandom) {
    // Source: reference/extracted/methods/monster/JawWorm.java (`getMove`).
    // Java makes a conditional randomBoolean draw after CHOMP (<25), after two
    // THRASHes (25..54), and after BELLOW (>=55). Each advances shared aiRng.
    let chomp_damage = enemy.entity.status(sid::STARTING_DMG).max(11);
    let strength = enemy.entity.status(sid::STR_AMT).max(3) as i16;
    let bellow_block = enemy.entity.status(sid::BLOCK_AMT).max(6);
    let chomp = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::JW_CHOMP, chomp_damage, 1, 0);
    };
    let bellow = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::JW_BELLOW, 0, 0, bellow_block);
        enemy.add_effect(mfx::STRENGTH, strength);
    };
    let thrash = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
    };

    if num < 25 {
        if last_move(enemy, move_ids::JW_CHOMP) {
            if ai_rng.random_float() < 0.5625 { bellow(enemy); } else { thrash(enemy); }
        } else {
            chomp(enemy);
        }
    } else if num < 55 {
        if last_two_moves(enemy, move_ids::JW_THRASH) {
            if ai_rng.random_float() < 0.357 { chomp(enemy); } else { bellow(enemy); }
        } else {
            thrash(enemy);
        }
    } else if last_move(enemy, move_ids::JW_BELLOW) {
        if ai_rng.random_float() < 0.416 { chomp(enemy); } else { thrash(enemy); }
    } else {
        bellow(enemy);
    }
}

// Java Cultist.getMove(int num) (decompiled monsters/exordium/Cultist.java):
//   firstMove -> INCANTATION (byte 3, BUFF)   [handled at create_enemy]
//   otherwise -> DARK_STRIKE (byte 1, ATTACK, damage.get(0).base = 6), forever.
// `num` is ignored by the Java switch, but AbstractMonster.rollMove()
// (AbstractMonster.java:465-466) still consumes one aiRng.random(99) tick per
// roll — our caller roll_next_move() does the same, keeping counters in sync.
pub(super) fn roll_cultist(enemy: &mut EnemyCombatState, _num: i32) {
    enemy.set_move(move_ids::CULT_DARK_STRIKE, 6, 1, 0);
}

pub(super) fn roll_fungi_beast(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/FungiBeast.java (`getMove`).
    let strength = enemy.entity.status(sid::STR_AMT).max(3) as i16;
    if num < 60 {
        if last_two_moves(enemy, move_ids::FB_BITE) {
            enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
            enemy.add_effect(mfx::STRENGTH, strength);
        } else {
            enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
        }
    } else if last_move(enemy, move_ids::FB_GROW) {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, strength);
    }
}

pub(super) fn roll_red_louse(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/LouseNormal.java (`getMove`).
    let bite_damage = match enemy.entity.status(sid::STARTING_DMG) {
        value if value > 0 => value,
        _ => 6,
    };
    let a17 = enemy.entity.status(sid::STR_AMT) >= 4;
    let choose_grow = if num < 25 {
        if a17 { !last_move(enemy, move_ids::LOUSE_GROW) }
        else { !last_two_moves(enemy, move_ids::LOUSE_GROW) }
    } else {
        last_two_moves(enemy, move_ids::LOUSE_BITE)
    };
    if choose_grow {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, if a17 { 4 } else { 3 });
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, bite_damage, 1, 0);
    }
}

pub(super) fn roll_green_louse(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/LouseDefensive.java (`getMove`).
    let bite_damage = match enemy.entity.status(sid::STARTING_DMG) {
        value if value > 0 => value,
        _ => 6,
    };
    let a17 = enemy.entity.status(sid::STR_AMT) >= 4;
    let choose_web = if num < 25 {
        if a17 { !last_move(enemy, move_ids::LOUSE_SPIT_WEB) }
        else { !last_two_moves(enemy, move_ids::LOUSE_SPIT_WEB) }
    } else {
        last_two_moves(enemy, move_ids::LOUSE_BITE)
    };
    if choose_web {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, bite_damage, 1, 0);
    }
}

pub(super) fn roll_blue_slaver(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/SlaverBlue.java (`getMove`).
    let stab = enemy.entity.status(sid::STARTING_DMG).max(12);
    let rake = enemy.entity.status(sid::STR_AMT).max(7);
    let weak = enemy.entity.status(sid::BLOCK_AMT).max(1) as i16;
    let choose_stab = if num >= 40 && !last_two_moves(enemy, move_ids::BS_STAB) {
        true
    } else if weak >= 2 {
        last_move(enemy, move_ids::BS_RAKE)
    } else {
        last_two_moves(enemy, move_ids::BS_RAKE)
    };
    if choose_stab {
        enemy.set_move(move_ids::BS_STAB, stab, 1, 0);
    } else {
        enemy.set_move(move_ids::BS_RAKE, rake, 1, 0);
        enemy.add_effect(mfx::WEAK, weak);
    }
}

pub(super) fn roll_red_slaver(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/SlaverRed.java (`getMove`).
    let stab = enemy.entity.status(sid::STARTING_DMG).max(13);
    let scrape = enemy.entity.status(sid::STR_AMT).max(8);
    let vulnerable = enemy.entity.status(sid::BLOCK_AMT).max(1) as i16;
    if enemy.entity.status(sid::IS_FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
        enemy.set_move(move_ids::RS_STAB, stab, 1, 0);
        return;
    }
    let used_entangle = enemy
        .move_history
        .iter()
        .any(|&m| m == move_ids::RS_ENTANGLE);

    if num >= 75 && !used_entangle {
        enemy.set_move(move_ids::RS_ENTANGLE, 0, 0, 0);
        enemy.add_effect(mfx::ENTANGLE, 1);
    } else if num >= 55 && used_entangle && !last_two_moves(enemy, move_ids::RS_STAB) {
        enemy.set_move(move_ids::RS_STAB, stab, 1, 0);
    } else if vulnerable >= 2 {
        if !last_move(enemy, move_ids::RS_SCRAPE) {
            enemy.set_move(move_ids::RS_SCRAPE, scrape, 1, 0);
            enemy.add_effect(mfx::VULNERABLE, vulnerable);
        } else {
            enemy.set_move(move_ids::RS_STAB, stab, 1, 0);
        }
    } else if !last_two_moves(enemy, move_ids::RS_SCRAPE) {
        enemy.set_move(move_ids::RS_SCRAPE, scrape, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, vulnerable);
    } else {
        enemy.set_move(move_ids::RS_STAB, stab, 1, 0);
    }
}

pub(super) fn roll_acid_slime_s(
    enemy: &mut EnemyCombatState,
    _num: i32,
    ai_rng: &mut StsRandom,
) {
    // Source: reference/extracted/methods/monster/AcidSlime_S.java (`getMove`).
    let damage = enemy.entity.status(sid::STARTING_DMG).max(3);
    let tackle = if enemy.entity.status(sid::STR_AMT) >= 17 {
        last_two_moves(enemy, move_ids::AS_S_TACKLE)
    } else {
        ai_rng.random_boolean()
    };
    if tackle {
        enemy.set_move(move_ids::AS_S_TACKLE, damage, 1, 0);
    } else {
        enemy.set_move(move_ids::AS_S_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    }
}

pub(crate) fn advance_acid_slime_s_after_turn(enemy: &mut EnemyCombatState) {
    // AcidSlime_S.takeTurn calls setMove directly and never queues RollMoveAction.
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();
    if enemy.move_id == move_ids::AS_S_TACKLE {
        enemy.set_move(move_ids::AS_S_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    } else {
        let damage = enemy.entity.status(sid::STARTING_DMG).max(3);
        enemy.set_move(move_ids::AS_S_TACKLE, damage, 1, 0);
    }
}

pub(super) fn roll_acid_slime_m(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut StsRandom,
) {
    // Source: reference/extracted/methods/monster/AcidSlime_M.java (`getMove`).
    let wound_damage = enemy.entity.status(sid::STARTING_DMG).max(7);
    let normal_damage = enemy.entity.status(sid::STR_AMT).max(10);
    let a17 = enemy.entity.status(sid::BLOCK_AMT) >= 17;
    let wound = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, wound_damage, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    };
    let normal = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::AS_TACKLE, normal_damage, 1, 0);
    };
    let lick = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    };

    if a17 {
        if num < 40 {
            if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
                if ai_rng.random_boolean() { normal(enemy); } else { lick(enemy); }
            } else { wound(enemy); }
        } else if num < 80 {
            if last_two_moves(enemy, move_ids::AS_TACKLE) {
                if ai_rng.random_float() < 0.5 { wound(enemy); } else { lick(enemy); }
            } else { normal(enemy); }
        } else if last_move(enemy, move_ids::AS_LICK) {
            if ai_rng.random_float() < 0.4 { wound(enemy); } else { normal(enemy); }
        } else { lick(enemy); }
    } else if num < 30 {
        if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
            if ai_rng.random_boolean() { normal(enemy); } else { lick(enemy); }
        } else { wound(enemy); }
    } else if num < 70 {
        if last_move(enemy, move_ids::AS_TACKLE) {
            if ai_rng.random_float() < 0.4 { wound(enemy); } else { lick(enemy); }
        } else { normal(enemy); }
    } else if last_two_moves(enemy, move_ids::AS_LICK) {
        if ai_rng.random_float() < 0.4 { wound(enemy); } else { normal(enemy); }
    } else {
        lick(enemy);
    }
}

pub(super) fn roll_acid_slime_l(enemy: &mut EnemyCombatState, _num: i32) {
    // Cycle: Tackle -> Spit -> Lick -> Tackle -> ...
    if last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.add_effect(mfx::SLIMED, 2);
    } else if last_move(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
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

pub(super) fn roll_looter(enemy: &mut EnemyCombatState, _num: i32) {
    let turns = enemy.move_history.len();
    if turns < 2 {
        // Mug twice
        enemy.set_move(move_ids::LOOTER_MUG, 10, 1, 0);
    } else if turns == 2 {
        // Smoke Bomb (block + prepare escape)
        enemy.set_move(move_ids::LOOTER_SMOKE_BOMB, 0, 0, 11);
    } else {
        // Escape
        enemy.set_move(move_ids::LOOTER_ESCAPE, 0, 0, 0);
        enemy.is_escaping = true;
    }
}

pub(super) fn roll_gremlin_simple(enemy: &mut EnemyCombatState, dmg: i32, weak: i32) {
    enemy.set_move(move_ids::GREMLIN_ATTACK, dmg, 1, 0);
    if weak > 0 {
        enemy.add_effect(mfx::WEAK, weak as i16);
    }
}

pub(super) fn roll_gremlin_wizard(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::GREMLIN_PROTECT) {
        // Ultimate Blast after charging
        enemy.set_move(move_ids::GREMLIN_ATTACK, 25, 1, 0);
    } else {
        // Charge up again
        enemy.set_move(move_ids::GREMLIN_PROTECT, 0, 0, 0);
    }
}

pub(super) fn roll_gremlin_nob(enemy: &mut EnemyCombatState, _num: i32) {
    if last_move(enemy, move_ids::NOB_BELLOW) || last_move(enemy, move_ids::NOB_SKULL_BASH) {
        enemy.set_move(move_ids::NOB_RUSH, 14, 1, 0);
    } else {
        enemy.set_move(move_ids::NOB_SKULL_BASH, 6, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
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
        // Awake: alternate Attack and Siphon Soul
        if last_move(enemy, move_ids::LAGA_ATTACK) {
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
    if last_move(enemy, move_ids::SENTRY_BOLT) {
        enemy.set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
        enemy.add_effect(mfx::DAZE, 2);
    } else {
        enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
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
