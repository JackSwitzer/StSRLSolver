use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 1 Basic Enemies
// =========================================================================

pub(super) fn roll_jaw_worm(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::JW_CHOMP) {
        enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if last_move(enemy, move_ids::JW_BELLOW) {
        enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
    } else if last_move(enemy, move_ids::JW_THRASH) {
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    } else {
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    }
}

pub(super) fn roll_cultist(enemy: &mut EnemyCombatState) {
    enemy.set_move(move_ids::CULT_DARK_STRIKE, 6, 1, 0);
}

pub(super) fn roll_fungi_beast(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::FB_BITE) {
        enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if last_move(enemy, move_ids::FB_GROW) {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    }
}

pub(super) fn roll_red_louse(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if last_move(enemy, move_ids::LOUSE_GROW) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

pub(super) fn roll_green_louse(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
    } else if last_move(enemy, move_ids::LOUSE_SPIT_WEB) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

pub(super) fn roll_blue_slaver(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::BS_STAB) {
        enemy.set_move(move_ids::BS_RAKE, 7, 1, 0);
        enemy.add_effect(mfx::WEAK, 1);
    } else if last_move(enemy, move_ids::BS_RAKE) {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    } else {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    }
}

pub(super) fn roll_red_slaver(enemy: &mut EnemyCombatState) {
    let used_entangle = enemy
        .move_history
        .iter()
        .any(|&m| m == move_ids::RS_ENTANGLE);

    if !used_entangle && !enemy.move_history.is_empty() {
        enemy.set_move(move_ids::RS_ENTANGLE, 0, 0, 0);
        enemy.add_effect(mfx::ENTANGLE, 1);
    } else if last_move(enemy, move_ids::RS_ENTANGLE)
        || last_two_moves(enemy, move_ids::RS_SCRAPE)
    {
        enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
    } else {
        enemy.set_move(move_ids::RS_SCRAPE, 8, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 1);
    }
}

pub(super) fn roll_acid_slime_s(enemy: &mut EnemyCombatState) {
    if last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 1);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
    }
}

pub(super) fn roll_acid_slime_m(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 10, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    } else {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.add_effect(mfx::SLIMED, 1);
    }
}

pub(super) fn roll_acid_slime_l(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.add_effect(mfx::SLIMED, 2);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.add_effect(mfx::SLIMED, 2);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    }
}

pub(super) fn roll_spike_slime_s(enemy: &mut EnemyCombatState) {
    enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
}

pub(super) fn roll_spike_slime_m(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::FRAIL, 1);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    }
}

pub(super) fn roll_spike_slime_l(enemy: &mut EnemyCombatState) {
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    }
}

pub(super) fn roll_looter(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_gremlin_nob(enemy: &mut EnemyCombatState) {
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

