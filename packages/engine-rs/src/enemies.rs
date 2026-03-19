//! Enemy AI system — Act 1 enemies and bosses for MCTS simulations.
//!
//! Each enemy has a deterministic move pattern that mirrors the Python/Java implementations.
//! For MCTS, we use simplified AI: no RNG-based move selection, instead we use
//! the most common/expected move pattern for fast simulation.

use crate::state::EnemyCombatState;

/// Enemy move IDs — shared constants for pattern matching.
pub mod move_ids {
    // Jaw Worm
    pub const JW_CHOMP: i32 = 1;
    pub const JW_BELLOW: i32 = 2;
    pub const JW_THRASH: i32 = 3;

    // Cultist
    pub const CULT_DARK_STRIKE: i32 = 1;
    pub const CULT_INCANTATION: i32 = 3;

    // Fungi Beast
    pub const FB_BITE: i32 = 1;
    pub const FB_GROW: i32 = 2;

    // Louse (Red/Green)
    pub const LOUSE_BITE: i32 = 3;
    pub const LOUSE_GROW: i32 = 4;
    pub const LOUSE_SPIT_WEB: i32 = 4;

    // Blue Slaver
    pub const BS_STAB: i32 = 1;
    pub const BS_RAKE: i32 = 4;

    // Red Slaver
    pub const RS_STAB: i32 = 1;
    pub const RS_ENTANGLE: i32 = 2;
    pub const RS_SCRAPE: i32 = 3;

    // Acid Slime S/M/L
    pub const AS_CORROSIVE_SPIT: i32 = 1;
    pub const AS_TACKLE: i32 = 2;
    pub const AS_LICK: i32 = 4;
    pub const AS_SPLIT: i32 = 3;

    // Spike Slime S/M/L
    pub const SS_TACKLE: i32 = 1;
    pub const SS_LICK: i32 = 4; // Frail
    pub const SS_SPLIT: i32 = 3;

    // Sentry
    pub const SENTRY_BOLT: i32 = 1;
    pub const SENTRY_BEAM: i32 = 2;

    // The Guardian
    pub const GUARD_CHARGING_UP: i32 = 6;
    pub const GUARD_FIERCE_BASH: i32 = 2;
    pub const GUARD_ROLL_ATTACK: i32 = 3;
    pub const GUARD_TWIN_SLAM: i32 = 4;
    pub const GUARD_WHIRLWIND: i32 = 5;
    pub const GUARD_VENT_STEAM: i32 = 7;

    // Hexaghost
    pub const HEX_DIVIDER: i32 = 1;
    pub const HEX_TACKLE: i32 = 2;
    pub const HEX_INFLAME: i32 = 3;
    pub const HEX_SEAR: i32 = 4;
    pub const HEX_ACTIVATE: i32 = 5;
    pub const HEX_INFERNO: i32 = 6;

    // Slime Boss
    pub const SB_SLAM: i32 = 1;
    pub const SB_PREP_SLAM: i32 = 2;
    pub const SB_SPLIT: i32 = 3;
    pub const SB_STICKY: i32 = 4;
}

/// Create a pre-configured enemy with initial move set.
/// Returns (enemy, extra_state) where extra_state holds boss-specific data.
pub fn create_enemy(enemy_id: &str, hp: i32, max_hp: i32) -> EnemyCombatState {
    let mut enemy = EnemyCombatState::new(enemy_id, hp, max_hp);

    // Set initial move based on enemy type
    match enemy_id {
        "JawWorm" => {
            // First turn: always Chomp (11 damage)
            enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
        }
        "Cultist" => {
            // First turn: Incantation (buff, no damage)
            enemy.set_move(move_ids::CULT_INCANTATION, 0, 0, 0);
            enemy.move_effects.insert("ritual".to_string(), 3);
        }
        "FungiBeast" => {
            // 60% chance Bite first
            enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
            enemy.entity.set_status("SporeCloud", 2);
        }
        "FuzzyLouseNormal" | "RedLouse" => {
            // Bite with rolled damage (use midpoint 6)
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status("CurlUp", 5);
        }
        "FuzzyLouseDefensive" | "GreenLouse" => {
            // Bite with rolled damage (use midpoint 6)
            enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
            enemy.entity.set_status("CurlUp", 5);
        }
        "SlaverBlue" | "BlueSlaver" => {
            // 60% Stab first
            enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
        }
        "SlaverRed" | "RedSlaver" => {
            // First turn: always Stab
            enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
        }
        "AcidSlime_S" => {
            // Small: Tackle only (3 damage) or Lick (weak 1)
            enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
        }
        "AcidSlime_M" => {
            // Medium: Corrosive Spit (7 damage + slimed)
            enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 1);
        }
        "AcidSlime_L" => {
            // Large: Corrosive Spit (11 damage + slimed)
            enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
            enemy.move_effects.insert("slimed".to_string(), 2);
        }
        "SpikeSlime_S" => {
            // Small: Tackle only (5 damage)
            enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
        }
        "SpikeSlime_M" => {
            // Medium: Tackle (8 damage) or Lick (frail)
            enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
        }
        "SpikeSlime_L" => {
            // Large: Tackle (16 damage) or Lick (frail 2)
            enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
        }
        "Sentry" => {
            // Alternates Bolt/Beam. Start with Bolt (9 damage)
            enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
        }
        "TheGuardian" => {
            // First turn: Charging Up (9 block)
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
            // Mode Shift threshold (default 30, A9+=35, A19+=40)
            enemy.entity.set_status("ModeShift", 30);
        }
        "Hexaghost" => {
            // First turn: Activate (no effect)
            enemy.set_move(move_ids::HEX_ACTIVATE, 0, 0, 0);
        }
        "SlimeBoss" => {
            // First turn: Goop Spray (slimed cards)
            enemy.set_move(move_ids::SB_STICKY, 0, 0, 0);
            enemy.move_effects.insert("slimed".to_string(), 3);
        }
        _ => {
            // Unknown enemy: generic 6 damage attack
            enemy.set_move(1, 6, 1, 0);
        }
    }

    enemy
}

/// Advance an enemy to its next move based on move history.
/// This is a deterministic pattern for MCTS (no RNG).
pub fn roll_next_move(enemy: &mut EnemyCombatState) {
    // Record current move in history
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();

    match enemy.id.as_str() {
        "JawWorm" => roll_jaw_worm(enemy),
        "Cultist" => roll_cultist(enemy),
        "FungiBeast" => roll_fungi_beast(enemy),
        "FuzzyLouseNormal" | "RedLouse" => roll_red_louse(enemy),
        "FuzzyLouseDefensive" | "GreenLouse" => roll_green_louse(enemy),
        "SlaverBlue" | "BlueSlaver" => roll_blue_slaver(enemy),
        "SlaverRed" | "RedSlaver" => roll_red_slaver(enemy),
        "AcidSlime_S" => roll_acid_slime_s(enemy),
        "AcidSlime_M" => roll_acid_slime_m(enemy),
        "AcidSlime_L" => roll_acid_slime_l(enemy),
        "SpikeSlime_S" => roll_spike_slime_s(enemy),
        "SpikeSlime_M" => roll_spike_slime_m(enemy),
        "SpikeSlime_L" => roll_spike_slime_l(enemy),
        "Sentry" => roll_sentry(enemy),
        "TheGuardian" => roll_guardian(enemy),
        "Hexaghost" => roll_hexaghost(enemy),
        "SlimeBoss" => roll_slime_boss(enemy),
        _ => {
            // Unknown: toggle between attack and block
            if enemy.move_damage > 0 {
                enemy.set_move(2, 0, 0, 5);
            } else {
                enemy.set_move(1, 6, 1, 0);
            }
        }
    }
}

// Helper: check if last move was the given ID
fn last_move(enemy: &EnemyCombatState, move_id: i32) -> bool {
    enemy.move_history.last().copied() == Some(move_id)
}

// Helper: check if last two moves were both the given ID
fn last_two_moves(enemy: &EnemyCombatState, move_id: i32) -> bool {
    let len = enemy.move_history.len();
    if len < 2 {
        return false;
    }
    enemy.move_history[len - 1] == move_id && enemy.move_history[len - 2] == move_id
}

// =========================================================================
// Act 1 Basic Enemies
// =========================================================================

fn roll_jaw_worm(enemy: &mut EnemyCombatState) {
    // Pattern: Chomp -> Bellow/Thrash cycle (most likely path)
    // After Chomp, 45% Bellow, then alternate
    if last_move(enemy, move_ids::JW_CHOMP) {
        // After Chomp: most likely Bellow (45%)
        enemy.set_move(move_ids::JW_BELLOW, 0, 0, 6);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::JW_BELLOW) {
        // After Bellow: most likely Thrash or Chomp
        enemy.set_move(move_ids::JW_THRASH, 7, 1, 5);
    } else if last_move(enemy, move_ids::JW_THRASH) {
        // After Thrash: Chomp or Bellow
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    } else {
        // Default: Chomp
        enemy.set_move(move_ids::JW_CHOMP, 11, 1, 0);
    }
}

fn roll_cultist(enemy: &mut EnemyCombatState) {
    // After Incantation, always Dark Strike
    // Ritual is applied as a status effect, Strength grows each turn via Ritual
    enemy.set_move(move_ids::CULT_DARK_STRIKE, 6, 1, 0);
}

fn roll_fungi_beast(enemy: &mut EnemyCombatState) {
    // 60% Bite, 40% Grow. Anti-repeat.
    if last_two_moves(enemy, move_ids::FB_BITE) {
        enemy.set_move(move_ids::FB_GROW, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::FB_GROW) {
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    } else {
        // Default: Bite (60%)
        enemy.set_move(move_ids::FB_BITE, 6, 1, 0);
    }
}

fn roll_red_louse(enemy: &mut EnemyCombatState) {
    // 75% Bite, 25% Grow. Anti-repeat on Bite (3x) and Grow (2x).
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_GROW, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::LOUSE_GROW) {
        // Damage is rolled per-instance; use 6 as midpoint
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

fn roll_green_louse(enemy: &mut EnemyCombatState) {
    // 75% Bite, 25% Spit Web. Anti-repeat.
    if last_two_moves(enemy, move_ids::LOUSE_BITE) {
        enemy.set_move(move_ids::LOUSE_SPIT_WEB, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::LOUSE_SPIT_WEB) {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    } else {
        enemy.set_move(move_ids::LOUSE_BITE, 6, 1, 0);
    }
}

fn roll_blue_slaver(enemy: &mut EnemyCombatState) {
    // 60% Stab, 40% Rake. Anti-repeat on Stab (3x).
    if last_two_moves(enemy, move_ids::BS_STAB) {
        enemy.set_move(move_ids::BS_RAKE, 7, 1, 0);
        enemy.move_effects.insert("weak".to_string(), 1);
    } else if last_move(enemy, move_ids::BS_RAKE) {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    } else {
        enemy.set_move(move_ids::BS_STAB, 12, 1, 0);
    }
}

fn roll_red_slaver(enemy: &mut EnemyCombatState) {
    // After first Stab: Scrape -> Stab cycle, with Entangle once
    let used_entangle = enemy
        .move_history
        .iter()
        .any(|&m| m == move_ids::RS_ENTANGLE);

    if !used_entangle && enemy.move_history.len() >= 1 {
        // 25% Entangle on turn 2 (use it once)
        enemy.set_move(move_ids::RS_ENTANGLE, 0, 0, 0);
        enemy.move_effects.insert("entangle".to_string(), 1);
    } else if last_move(enemy, move_ids::RS_ENTANGLE)
        || last_two_moves(enemy, move_ids::RS_SCRAPE)
    {
        enemy.set_move(move_ids::RS_STAB, 13, 1, 0);
    } else {
        enemy.set_move(move_ids::RS_SCRAPE, 8, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 1);
    }
}

fn roll_acid_slime_s(enemy: &mut EnemyCombatState) {
    // Small Acid: alternates Tackle (3 dmg) and Lick (weak 1)
    if last_move(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_LICK, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 1);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 3, 1, 0);
    }
}

fn roll_acid_slime_m(enemy: &mut EnemyCombatState) {
    // Medium Acid: Corrosive Spit (40%), Tackle (40%), Lick (20%). Anti-repeat.
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 10, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    } else {
        // Default: Corrosive Spit
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 7, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 1);
    }
}

fn roll_acid_slime_l(enemy: &mut EnemyCombatState) {
    // Large Acid: Corrosive Spit (30%), Tackle (40%), Lick (30%). Anti-repeat. Splits at 50%.
    if last_two_moves(enemy, move_ids::AS_CORROSIVE_SPIT) {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    } else if last_two_moves(enemy, move_ids::AS_TACKLE) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 2);
    } else if last_move(enemy, move_ids::AS_LICK) {
        enemy.set_move(move_ids::AS_CORROSIVE_SPIT, 11, 1, 0);
        enemy.move_effects.insert("slimed".to_string(), 2);
    } else {
        enemy.set_move(move_ids::AS_TACKLE, 16, 1, 0);
    }
}

fn roll_spike_slime_s(enemy: &mut EnemyCombatState) {
    // Small Spike: Tackle only (5 damage)
    enemy.set_move(move_ids::SS_TACKLE, 5, 1, 0);
}

fn roll_spike_slime_m(enemy: &mut EnemyCombatState) {
    // Medium Spike: Tackle (30%) or Lick/Flame Tackle. Anti-repeat.
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.move_effects.insert("frail".to_string(), 1);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 8, 1, 0);
    }
}

fn roll_spike_slime_l(enemy: &mut EnemyCombatState) {
    // Large Spike: Tackle (30%) or Lick (frail 2). Anti-repeat. Splits at 50%.
    if last_two_moves(enemy, move_ids::SS_TACKLE) {
        enemy.set_move(move_ids::SS_LICK, 0, 0, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else if last_move(enemy, move_ids::SS_LICK) {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    } else {
        enemy.set_move(move_ids::SS_TACKLE, 16, 1, 0);
    }
}

fn roll_sentry(enemy: &mut EnemyCombatState) {
    // Alternates: Bolt (9 damage) -> Beam (9 damage + Daze)
    if last_move(enemy, move_ids::SENTRY_BOLT) {
        enemy.set_move(move_ids::SENTRY_BEAM, 9, 1, 0);
        enemy.move_effects.insert("daze".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SENTRY_BOLT, 9, 1, 0);
    }
}

// =========================================================================
// Act 1 Bosses
// =========================================================================

fn roll_guardian(enemy: &mut EnemyCombatState) {
    let is_defensive = enemy.entity.status("SharpHide") > 0;

    if is_defensive {
        // Defensive mode: Roll Attack -> Twin Slam -> Roll Attack ...
        if last_move(enemy, move_ids::GUARD_ROLL_ATTACK) {
            enemy.set_move(move_ids::GUARD_TWIN_SLAM, 8, 2, 0);
        } else {
            enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        }
    } else {
        // Offensive mode: Charging Up -> Fierce Bash -> Vent Steam -> Whirlwind -> repeat
        if last_move(enemy, move_ids::GUARD_CHARGING_UP) {
            enemy.set_move(move_ids::GUARD_FIERCE_BASH, 32, 1, 0);
        } else if last_move(enemy, move_ids::GUARD_FIERCE_BASH) {
            enemy.set_move(move_ids::GUARD_VENT_STEAM, 0, 0, 0);
            enemy.move_effects.insert("weak".to_string(), 2);
            enemy.move_effects.insert("vulnerable".to_string(), 2);
        } else if last_move(enemy, move_ids::GUARD_VENT_STEAM) {
            enemy.set_move(move_ids::GUARD_WHIRLWIND, 5, 4, 0);
        } else {
            // After Whirlwind or start: Charging Up
            enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
        }
    }
}

/// Check if Guardian should switch to defensive mode after taking damage.
/// Returns true if mode shifted. Caller should apply SharpHide status.
pub fn guardian_check_mode_shift(enemy: &mut EnemyCombatState, damage_dealt: i32) -> bool {
    let threshold = enemy.entity.status("ModeShift");
    if threshold <= 0 {
        return false;
    }

    // Track cumulative damage this mode
    let current_taken = enemy.entity.status("DamageTakenThisMode") + damage_dealt;
    enemy.entity.set_status("DamageTakenThisMode", current_taken);

    if current_taken >= threshold {
        // Switch to defensive mode
        enemy.entity.set_status("SharpHide", 3);
        enemy.entity.set_status("DamageTakenThisMode", 0);
        // Increase threshold by 10 for next mode shift
        enemy.entity.set_status("ModeShift", threshold + 10);
        // Set next move to defensive pattern
        enemy.set_move(move_ids::GUARD_ROLL_ATTACK, 9, 1, 0);
        enemy.move_history.clear(); // Reset history for new mode
        true
    } else {
        false
    }
}

/// Switch Guardian back to offensive mode. Called after Twin Slam.
pub fn guardian_switch_to_offensive(enemy: &mut EnemyCombatState) {
    enemy.entity.set_status("SharpHide", 0);
    enemy.entity.set_status("DamageTakenThisMode", 0);
    enemy.set_move(move_ids::GUARD_CHARGING_UP, 0, 0, 9);
    enemy.move_history.clear();
}

fn roll_hexaghost(enemy: &mut EnemyCombatState) {
    // Turn-based pattern: Activate -> Divider -> Sear -> Tackle -> Sear -> Inflame -> Tackle -> Sear -> Inferno
    // move_history has all previous moves at this point (current move was just pushed)
    let moves_done = enemy.move_history.len(); // Number of completed moves

    match moves_done {
        // After Activate (1 move done): next is Divider
        1 => {
            // Divider (6 hits, damage based on player HP / 12 + 1)
            // Use 7 as default (80hp / 12 + 1 = 7.67, floored)
            enemy.set_move(move_ids::HEX_DIVIDER, 7, 6, 0);
        }
        _ => {
            // After Divider: cycle through 7-move pattern
            let pattern_turn = (moves_done - 2) % 7;
            match pattern_turn {
                0 => {
                    // Sear: 6 damage + 1 Burn
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                1 => {
                    // Tackle: 5x2
                    enemy.set_move(move_ids::HEX_TACKLE, 5, 2, 0);
                }
                2 => {
                    // Sear
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                3 => {
                    // Inflame: +2 Str, 12 block
                    enemy.set_move(move_ids::HEX_INFLAME, 0, 0, 12);
                    enemy.move_effects.insert("strength".to_string(), 2);
                }
                4 => {
                    // Tackle: 5x2
                    enemy.set_move(move_ids::HEX_TACKLE, 5, 2, 0);
                }
                5 => {
                    // Sear
                    enemy.set_move(move_ids::HEX_SEAR, 6, 1, 0);
                    enemy.move_effects.insert("burn".to_string(), 1);
                }
                _ => {
                    // Inferno: 2x6 + 3 Burns
                    enemy.set_move(move_ids::HEX_INFERNO, 2, 6, 0);
                    enemy
                        .move_effects
                        .insert("burn+".to_string(), 3);
                }
            }
        }
    }
}

fn roll_slime_boss(enemy: &mut EnemyCombatState) {
    // Pattern: Sticky -> Prep -> Slam -> Sticky -> Prep -> Slam ...
    if last_move(enemy, move_ids::SB_STICKY) {
        enemy.set_move(move_ids::SB_PREP_SLAM, 0, 0, 0);
    } else if last_move(enemy, move_ids::SB_PREP_SLAM) {
        enemy.set_move(move_ids::SB_SLAM, 35, 1, 0);
    } else {
        // After Slam: Sticky
        enemy.set_move(move_ids::SB_STICKY, 0, 0, 0);
        enemy.move_effects.insert("slimed".to_string(), 3);
    }
}

/// Check if Slime Boss should split (HP <= 50%).
pub fn slime_boss_should_split(enemy: &EnemyCombatState) -> bool {
    enemy.entity.hp > 0 && enemy.entity.hp <= enemy.entity.max_hp / 2
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_jaw_worm() {
        let enemy = create_enemy("JawWorm", 44, 44);
        assert_eq!(enemy.id, "JawWorm");
        assert_eq!(enemy.entity.hp, 44);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
        assert_eq!(enemy.move_damage, 11);
    }

    #[test]
    fn test_jaw_worm_pattern() {
        let mut enemy = create_enemy("JawWorm", 44, 44);
        // Turn 1: Chomp
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);

        // Roll next: after Chomp -> Bellow
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_BELLOW);
        assert_eq!(enemy.move_effects.get("strength"), Some(&3));

        // After Bellow -> Thrash
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_THRASH);
        assert_eq!(enemy.move_damage, 7);
        assert_eq!(enemy.move_block, 5);

        // After Thrash -> Chomp
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::JW_CHOMP);
    }

    #[test]
    fn test_cultist_pattern() {
        let mut enemy = create_enemy("Cultist", 50, 50);
        // Turn 1: Incantation (buff)
        assert_eq!(enemy.move_id, move_ids::CULT_INCANTATION);
        assert_eq!(enemy.move_damage, 0);
        assert_eq!(enemy.move_effects.get("ritual"), Some(&3));

        // After: always Dark Strike
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
        assert_eq!(enemy.move_damage, 6);

        // Continues Dark Strike
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::CULT_DARK_STRIKE);
    }

    #[test]
    fn test_fungi_beast_anti_repeat() {
        let mut enemy = create_enemy("FungiBeast", 24, 24);
        // Turn 1: Bite
        assert_eq!(enemy.move_id, move_ids::FB_BITE);

        roll_next_move(&mut enemy); // Bite -> Bite (likely)
        roll_next_move(&mut enemy); // Two Bites -> must Grow
        if enemy.move_history.len() >= 2
            && enemy.move_history[enemy.move_history.len() - 1] == move_ids::FB_BITE
            && enemy.move_history[enemy.move_history.len() - 2] == move_ids::FB_BITE
        {
            assert_eq!(enemy.move_id, move_ids::FB_GROW);
        }
    }

    #[test]
    fn test_sentry_alternating() {
        let mut enemy = create_enemy("Sentry", 38, 38);
        // Turn 1: Bolt
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);

        // Turn 2: Beam
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BEAM);
        assert_eq!(enemy.move_effects.get("daze"), Some(&2));

        // Turn 3: Bolt
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SENTRY_BOLT);
    }

    #[test]
    fn test_slime_boss_pattern() {
        let mut enemy = create_enemy("SlimeBoss", 140, 140);
        // Turn 1: Sticky
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);

        // Turn 2: Prep
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_PREP_SLAM);

        // Turn 3: Slam
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_SLAM);
        assert_eq!(enemy.move_damage, 35);

        // Turn 4: Sticky again
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SB_STICKY);
    }

    #[test]
    fn test_slime_boss_split_check() {
        let mut enemy = create_enemy("SlimeBoss", 140, 140);
        assert!(!slime_boss_should_split(&enemy));

        enemy.entity.hp = 70; // Exactly 50%
        assert!(slime_boss_should_split(&enemy));

        enemy.entity.hp = 69; // Below 50%
        assert!(slime_boss_should_split(&enemy));
    }

    #[test]
    fn test_guardian_offensive_pattern() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        // Turn 1: Charging Up
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);

        // Turn 2: Fierce Bash
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_FIERCE_BASH);
        assert_eq!(enemy.move_damage, 32);

        // Turn 3: Vent Steam
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_VENT_STEAM);
        assert_eq!(enemy.move_effects.get("weak"), Some(&2));
        assert_eq!(enemy.move_effects.get("vulnerable"), Some(&2));

        // Turn 4: Whirlwind
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_WHIRLWIND);
        assert_eq!(enemy.move_damage, 5);
        assert_eq!(enemy.move_hits, 4);
    }

    #[test]
    fn test_guardian_mode_shift() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        assert_eq!(enemy.entity.status("ModeShift"), 30);

        // Deal 30 damage -> mode shift
        let shifted = guardian_check_mode_shift(&mut enemy, 30);
        assert!(shifted);
        assert_eq!(enemy.entity.status("SharpHide"), 3);
        assert_eq!(enemy.entity.status("ModeShift"), 40); // Increased by 10

        // Defensive mode: Roll Attack
        assert_eq!(enemy.move_id, move_ids::GUARD_ROLL_ATTACK);

        // Next: Twin Slam
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::GUARD_TWIN_SLAM);
    }

    #[test]
    fn test_hexaghost_pattern() {
        let mut enemy = create_enemy("Hexaghost", 250, 250);
        // Turn 1: Activate
        assert_eq!(enemy.move_id, move_ids::HEX_ACTIVATE);

        // Turn 2: Divider
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_DIVIDER);
        assert_eq!(enemy.move_hits, 6);

        // Turn 3: Sear
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_SEAR);
        assert_eq!(enemy.move_effects.get("burn"), Some(&1));

        // Turn 4: Tackle
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::HEX_TACKLE);
        assert_eq!(enemy.move_hits, 2);
    }

    #[test]
    fn test_blue_slaver_pattern() {
        let mut enemy = create_enemy("SlaverBlue", 48, 48);
        assert_eq!(enemy.move_id, move_ids::BS_STAB);
        assert_eq!(enemy.move_damage, 12);

        // After Stab: Stab again (60% chance)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BS_STAB);

        // After two Stabs: must Rake
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::BS_RAKE);
        assert_eq!(enemy.move_effects.get("weak"), Some(&1));
    }

    #[test]
    fn test_red_slaver_pattern() {
        let mut enemy = create_enemy("SlaverRed", 48, 48);
        // Turn 1: Stab
        assert_eq!(enemy.move_id, move_ids::RS_STAB);
        assert_eq!(enemy.move_damage, 13);

        // Turn 2: Entangle (first use)
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::RS_ENTANGLE);
        assert_eq!(enemy.move_effects.get("entangle"), Some(&1));

        // Turn 3: Scrape or Stab
        roll_next_move(&mut enemy);
        assert!(
            enemy.move_id == move_ids::RS_SCRAPE || enemy.move_id == move_ids::RS_STAB
        );
    }

    #[test]
    fn test_acid_slime_s_pattern() {
        let mut enemy = create_enemy("AcidSlime_S", 10, 10);
        assert_eq!(enemy.move_id, move_ids::AS_TACKLE);

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_LICK);
        assert_eq!(enemy.move_effects.get("weak"), Some(&1));

        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::AS_TACKLE);
    }

    #[test]
    fn test_spike_slime_m_pattern() {
        let mut enemy = create_enemy("SpikeSlime_M", 28, 28);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);
        assert_eq!(enemy.move_damage, 8);

        // After Tackle: Tackle again
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SS_TACKLE);

        // After two Tackles: Lick
        roll_next_move(&mut enemy);
        assert_eq!(enemy.move_id, move_ids::SS_LICK);
        assert_eq!(enemy.move_effects.get("frail"), Some(&1));
    }

    #[test]
    fn test_louse_curl_up() {
        let enemy = create_enemy("RedLouse", 12, 12);
        assert_eq!(enemy.entity.status("CurlUp"), 5);
    }

    #[test]
    fn test_guardian_switch_to_offensive() {
        let mut enemy = create_enemy("TheGuardian", 240, 240);
        // Force into defensive mode
        guardian_check_mode_shift(&mut enemy, 30);
        assert_eq!(enemy.entity.status("SharpHide"), 3);

        // Switch back
        guardian_switch_to_offensive(&mut enemy);
        assert_eq!(enemy.entity.status("SharpHide"), 0);
        assert_eq!(enemy.move_id, move_ids::GUARD_CHARGING_UP);
    }
}
