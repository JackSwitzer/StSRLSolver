use crate::state::EnemyCombatState;
use super::{last_move, last_two_moves};
use super::move_ids;

// =========================================================================
// Act 2 Basic Enemies
// =========================================================================

pub(super) fn roll_chosen(enemy: &mut EnemyCombatState) {
    let used_hex = enemy.move_history.iter().any(|&m| m == move_ids::CHOSEN_HEX);

    // After first turn (Poke): use Hex
    if !used_hex {
        enemy.set_move(move_ids::CHOSEN_HEX, 0, 0, 0);
        enemy.move_effects.insert("hex".to_string(), 1);
        return;
    }
    // After Hex: alternate Debilitate/Drain and Zap/Poke
    if last_move(enemy, move_ids::CHOSEN_DEBILITATE) || last_move(enemy, move_ids::CHOSEN_DRAIN) {
        // Attack turn: Zap (18) or Poke (5x2)
        enemy.set_move(move_ids::CHOSEN_ZAP, 18, 1, 0);
    } else {
        // Debuff turn: Debilitate (10 + Vuln 2) or Drain (Weak 3, +3 Str)
        enemy.set_move(move_ids::CHOSEN_DEBILITATE, 10, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    }
}

pub(super) fn roll_mugger(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_byrd(enemy: &mut EnemyCombatState) {
    let is_flying = enemy.entity.status("Flight") > 0;

    if !is_flying {
        // Grounded: Headbutt then Fly Up
        if last_move(enemy, move_ids::BYRD_STUNNED) {
            enemy.set_move(move_ids::BYRD_HEADBUTT, 3, 1, 0);
        } else {
            enemy.set_move(move_ids::BYRD_FLY_UP, 0, 0, 0);
            enemy.entity.set_status("Flight", 3);
        }
    } else {
        // Flying: alternate Peck and Swoop
        if last_two_moves(enemy, move_ids::BYRD_PECK) {
            enemy.set_move(move_ids::BYRD_SWOOP, 12, 1, 0);
        } else if last_move(enemy, move_ids::BYRD_SWOOP) {
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
        } else {
            enemy.set_move(move_ids::BYRD_PECK, 1, 5, 0);
        }
    }
}

pub(super) fn roll_shelled_parasite(enemy: &mut EnemyCombatState) {
    // Cycle: Double Strike (6x2), Life Suck (10), Fell (18 + Frail 2)
    if last_move(enemy, move_ids::SP_DOUBLE_STRIKE) {
        enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
        enemy.move_effects.insert("heal".to_string(), 10);
    } else if last_move(enemy, move_ids::SP_LIFE_SUCK) {
        enemy.set_move(move_ids::SP_FELL, 18, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
    }
}

pub(super) fn roll_snake_plant(enemy: &mut EnemyCombatState) {
    // 65% Chomp (7x3), 35% Spores (Weak 2 + Frail 2). Anti-repeat.
    if last_two_moves(enemy, move_ids::SNAKE_CHOMP) {
        enemy.set_move(move_ids::SNAKE_SPORES, 0, 0, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
        enemy.move_effects.insert("frail".to_string(), 2);
    } else if last_move(enemy, move_ids::SNAKE_SPORES) {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    }
}

pub(super) fn roll_centurion(enemy: &mut EnemyCombatState) {
    // Fury (6x3) or Slash (12), with Protect (15 block to ally) when ally alive
    if last_two_moves(enemy, move_ids::CENT_FURY) {
        enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
    } else if last_two_moves(enemy, move_ids::CENT_SLASH) {
        enemy.set_move(move_ids::CENT_FURY, 6, 3, 0);
    } else if last_move(enemy, move_ids::CENT_PROTECT) {
        enemy.set_move(move_ids::CENT_FURY, 6, 3, 0);
    } else {
        // Default: Slash
        enemy.set_move(move_ids::CENT_SLASH, 12, 1, 0);
    }
}

pub(super) fn roll_mystic(enemy: &mut EnemyCombatState) {
    // Attack (8 dmg), Heal (16 hp to ally), Buff (+2 Str to all allies)
    if last_two_moves(enemy, move_ids::MYSTIC_ATTACK) {
        enemy.set_move(move_ids::MYSTIC_BUFF, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), 2);
    } else if last_move(enemy, move_ids::MYSTIC_BUFF) || last_move(enemy, move_ids::MYSTIC_HEAL) {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::MYSTIC_ATTACK, 8, 1, 0);
    }
}

pub(super) fn roll_book_of_stabbing(enemy: &mut EnemyCombatState) {
    // Multi-stab with increasing count. Stab count increases each time multi-stab is used.
    let stab_count = enemy.entity.status("StabCount");
    if last_two_moves(enemy, move_ids::BOOK_STAB) {
        enemy.set_move(move_ids::BOOK_BIG_STAB, 21, 1, 0);
        // Increment stab count on A18+
    } else if last_move(enemy, move_ids::BOOK_BIG_STAB) {
        let new_count = stab_count + 1;
        enemy.entity.set_status("StabCount", new_count);
        enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
    } else {
        let new_count = stab_count + 1;
        enemy.entity.set_status("StabCount", new_count);
        enemy.set_move(move_ids::BOOK_STAB, 6, new_count, 0);
    }
}

pub(super) fn roll_gremlin_leader(enemy: &mut EnemyCombatState) {
    // Rally (summon), Encourage (block + Str to minions), Stab (6x3)
    if last_move(enemy, move_ids::GL_RALLY) {
        enemy.set_move(move_ids::GL_ENCOURAGE, 0, 0, 6);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::GL_ENCOURAGE) {
        enemy.set_move(move_ids::GL_STAB, 6, 3, 0);
    } else {
        // After stab: Rally if minions dead, else Encourage
        enemy.set_move(move_ids::GL_RALLY, 0, 0, 0);
    }
}

pub(super) fn roll_taskmaster(enemy: &mut EnemyCombatState) {
    // Always Scouring Whip (7 damage + Wound card to discard)
    enemy.set_move(move_ids::TASK_SCOURING_WHIP, 7, 1, 0);
    enemy.move_effects.insert("wound".to_string(), 1);
}

pub(super) fn roll_spheric_guardian(enemy: &mut EnemyCombatState) {
    // Pattern: Initial Block -> Frail Attack -> Big Attack -> Block Attack -> repeat
    if last_move(enemy, move_ids::SPHER_INITIAL_BLOCK) {
        enemy.set_move(move_ids::SPHER_FRAIL_ATTACK, 10, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 5);
    } else if last_move(enemy, move_ids::SPHER_BIG_ATTACK) {
        enemy.set_move(move_ids::SPHER_BLOCK_ATTACK, 10, 1, 15);
    } else if last_move(enemy, move_ids::SPHER_BLOCK_ATTACK) || last_move(enemy, move_ids::SPHER_FRAIL_ATTACK) {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    } else {
        enemy.set_move(move_ids::SPHER_BIG_ATTACK, 10, 2, 0);
    }
}

pub(super) fn roll_snecko(enemy: &mut EnemyCombatState) {
    // First turn: Glare. Then alternate Tail (8 + Vuln 2) and Bite (15)
    if last_move(enemy, move_ids::SNECKO_GLARE) || last_two_moves(enemy, move_ids::SNECKO_BITE) {
        enemy.set_move(move_ids::SNECKO_TAIL, 8, 1, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    } else {
        enemy.set_move(move_ids::SNECKO_BITE, 15, 1, 0);
    }
}

pub(super) fn roll_bear(enemy: &mut EnemyCombatState) {
    // Bear Hug (debuff) -> Maul (18) -> Lunge (9 + 9 block) -> cycle
    if last_move(enemy, move_ids::BEAR_HUG) {
        enemy.set_move(move_ids::BEAR_MAUL, 18, 1, 0);
    } else if last_move(enemy, move_ids::BEAR_MAUL) {
        enemy.set_move(move_ids::BEAR_LUNGE, 9, 1, 9);
    } else {
        enemy.set_move(move_ids::BEAR_HUG, 0, 0, 0);
        enemy.move_effects.insert("dexterity_down".to_string(), 2);
    }
}

pub(super) fn roll_bandit_leader(enemy: &mut EnemyCombatState) {
    // Mock -> Agonizing Slash (10 + Weak 2) -> Cross Slash (15) -> cycle
    if last_move(enemy, move_ids::BANDIT_MOCK) {
        enemy.set_move(move_ids::BANDIT_AGONIZE, 10, 1, 0);
        enemy.move_effects.insert("weak".to_string(), 2);
    } else if last_move(enemy, move_ids::BANDIT_AGONIZE) {
        enemy.set_move(move_ids::BANDIT_CROSS_SLASH, 15, 1, 0);
    } else {
        enemy.set_move(move_ids::BANDIT_MOCK, 0, 0, 0);
    }
}

// =========================================================================
// Act 2 Bosses
// =========================================================================

pub(super) fn roll_bronze_automaton(enemy: &mut EnemyCombatState) {
    // Spawn Orbs -> Flail (7x2) -> ... -> Hyper Beam (45) -> Stunned -> repeat
    if last_move(enemy, move_ids::BA_SPAWN_ORBS) || last_move(enemy, move_ids::BA_STUNNED) || last_move(enemy, move_ids::BA_BOOST) {
        enemy.set_move(move_ids::BA_FLAIL, 7, 2, 0);
    } else if last_move(enemy, move_ids::BA_FLAIL) {
        // After enough Flails, Hyper Beam
        let turns = enemy.move_history.len();
        if turns >= 4 {
            enemy.set_move(move_ids::BA_HYPER_BEAM, 45, 1, 0);
        } else {
            enemy.set_move(move_ids::BA_BOOST, 0, 0, 9);
            enemy.move_effects.insert("strength".to_string(), 3);
        }
    } else if last_move(enemy, move_ids::BA_HYPER_BEAM) {
        enemy.set_move(move_ids::BA_STUNNED, 0, 0, 0);
    } else {
        enemy.set_move(move_ids::BA_FLAIL, 7, 2, 0);
    }
}

pub(super) fn roll_bronze_orb(enemy: &mut EnemyCombatState) {
    // Stasis (first turn) -> Beam (8) / Support (12 block to Automaton)
    if last_two_moves(enemy, move_ids::BO_BEAM) {
        enemy.set_move(move_ids::BO_SUPPORT, 0, 0, 12);
    } else if last_move(enemy, move_ids::BO_SUPPORT) {
        enemy.set_move(move_ids::BO_BEAM, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::BO_BEAM, 8, 1, 0);
    }
}

pub(super) fn roll_champ(enemy: &mut EnemyCombatState) {
    let num_turns = enemy.entity.status("NumTurns") + 1;
    enemy.entity.set_status("NumTurns", num_turns);

    let str_amt = enemy.entity.status("StrAmt").max(2);
    let _forge_amt = enemy.entity.status("ForgeAmt").max(5);
    let _block_amt = enemy.entity.status("BlockAmt").max(15);
    let slash_dmg = enemy.entity.status("SlashDmg").max(16);
    let slap_dmg = enemy.entity.status("SlapDmg").max(12);

    let threshold_reached_now = enemy.entity.hp <= enemy.entity.max_hp / 2;

    // Phase 2 trigger: Anger (remove debuffs, gain 3*strAmt Str)
    if threshold_reached_now && enemy.entity.status("ThresholdReached") == 0 {
        enemy.entity.set_status("ThresholdReached", 1);
        enemy.set_move(move_ids::CHAMP_ANGER, 0, 0, 0);
        // Java: Anger gives 3*strAmt Strength (not strAmt)
        enemy.move_effects.insert("strength".to_string(), str_amt * 3);
        enemy.move_effects.insert("remove_debuffs".to_string(), 1);
        return;
    }

    // Phase 2: Execute spam
    if enemy.entity.status("ThresholdReached") > 0 {
        // Java: Execute (10x2) every turn if threshold reached.
        // Uses lastMove and lastMoveBefore to check.
        if !last_move(enemy, move_ids::CHAMP_EXECUTE) {
            enemy.set_move(move_ids::CHAMP_EXECUTE, 10, 2, 0);
        } else {
            enemy.set_move(move_ids::CHAMP_EXECUTE, 10, 2, 0);
        }
        return;
    }

    // Phase 1: Java uses numTurns==4 for Taunt, then RNG-based selection.
    // Deterministic MCTS: simplified cycle.
    if num_turns == 4 {
        // Taunt at turn 4 (Java)
        enemy.set_move(move_ids::CHAMP_TAUNT, 0, 0, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
        enemy.move_effects.insert("weak".to_string(), 2);
        enemy.entity.set_status("NumTurns", 0);
        return;
    }

    if last_move(enemy, move_ids::CHAMP_FACE_SLAP) || last_move(enemy, move_ids::CHAMP_TAUNT) {
        enemy.set_move(move_ids::CHAMP_HEAVY_SLASH, slash_dmg, 1, 0);
    } else if last_move(enemy, move_ids::CHAMP_HEAVY_SLASH) {
        // Gloat (gain strAmt Str)
        enemy.set_move(move_ids::CHAMP_GLOAT, 0, 0, 0);
        enemy.move_effects.insert("strength".to_string(), str_amt);
    } else if last_move(enemy, move_ids::CHAMP_GLOAT) || last_move(enemy, move_ids::CHAMP_DEFENSIVE) {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        // Java: Face Slap gives Frail 2 + Vulnerable 2
        enemy.move_effects.insert("frail".to_string(), 2);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    } else {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        enemy.move_effects.insert("frail".to_string(), 2);
        enemy.move_effects.insert("vulnerable".to_string(), 2);
    }
}

pub(super) fn roll_collector(enemy: &mut EnemyCombatState) {
    // Spawn -> Mega Debuff -> Fireball (18) cycle with Buff (+3 Str, 15 block) and Revive
    let turns = enemy.move_history.len();
    if turns == 1 {
        // After Spawn: Mega Debuff (Vuln 3, Weak 3, Frail 3)
        enemy.set_move(move_ids::COLL_MEGA_DEBUFF, 0, 0, 0);
        enemy.move_effects.insert("vulnerable".to_string(), 3);
        enemy.move_effects.insert("weak".to_string(), 3);
        enemy.move_effects.insert("frail".to_string(), 3);
    } else if last_two_moves(enemy, move_ids::COLL_FIREBALL) {
        enemy.set_move(move_ids::COLL_BUFF, 0, 0, 15);
        enemy.move_effects.insert("strength".to_string(), 3);
    } else if last_move(enemy, move_ids::COLL_BUFF) {
        enemy.set_move(move_ids::COLL_FIREBALL, 18, 1, 0);
    } else {
        enemy.set_move(move_ids::COLL_FIREBALL, 18, 1, 0);
    }
}

