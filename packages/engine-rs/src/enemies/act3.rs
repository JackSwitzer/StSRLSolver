use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 3 Basic Enemies
// =========================================================================

pub(super) fn roll_darkling(enemy: &mut EnemyCombatState) {
    // Chomp (8x2), Harden (12 block + Reanimated), Nip (8).
    // If dead: Reincarnate (revive at 50% HP).
    if enemy.entity.hp <= 0 {
        enemy.set_move(move_ids::DARK_REINCARNATE, 0, 0, 0);
        return;
    }
    if last_two_moves(enemy, move_ids::DARK_NIP) {
        enemy.set_move(move_ids::DARK_CHOMP, 8, 2, 0);
    } else if last_move(enemy, move_ids::DARK_CHOMP) {
        enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
    } else if last_move(enemy, move_ids::DARK_HARDEN) {
        enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
    } else {
        enemy.set_move(move_ids::DARK_NIP, 8, 1, 0);
    }
}

pub(super) fn roll_orb_walker(enemy: &mut EnemyCombatState) {
    // Alternate: Claw (15) and Laser (10 + Burn)
    if last_two_moves(enemy, move_ids::OW_CLAW) {
        enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
        enemy.add_effect(mfx::BURN, 1);
    } else if last_two_moves(enemy, move_ids::OW_LASER) {
        enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
    } else if last_move(enemy, move_ids::OW_LASER) {
        enemy.set_move(move_ids::OW_CLAW, 15, 1, 0);
    } else {
        enemy.set_move(move_ids::OW_LASER, 10, 1, 0);
        enemy.add_effect(mfx::BURN, 1);
    }
}

pub(super) fn roll_spiker(enemy: &mut EnemyCombatState) {
    // Attack (7 dmg) or Buff (+2 Thorns). Anti-repeat.
    if last_move(enemy, move_ids::SPIKER_ATTACK) {
        enemy.set_move(move_ids::SPIKER_BUFF, 0, 0, 0);
        let thorns = enemy.entity.status(sid::THORNS);
        enemy.entity.set_status(sid::THORNS, thorns + 2);
        enemy.add_effect(mfx::THORNS, 2);
    } else {
        enemy.set_move(move_ids::SPIKER_ATTACK, 7, 1, 0);
    }
}

pub(super) fn roll_repulsor(enemy: &mut EnemyCombatState) {
    // Mostly Daze, sometimes Attack (11)
    if last_move(enemy, move_ids::REPULSOR_ATTACK) {
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.add_effect(mfx::DAZE, 2);
    } else {
        // 80% Daze, 20% Attack
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.add_effect(mfx::DAZE, 2);
    }
}

pub(super) fn roll_exploder(enemy: &mut EnemyCombatState) {
    let count = enemy.entity.status(sid::TURN_COUNT) + 1;
    enemy.entity.set_status(sid::TURN_COUNT, count);

    if count >= 3 {
        // Explode! 30 damage and die
        enemy.set_move(move_ids::EXPLODER_EXPLODE, 30, 1, 0);
    } else {
        enemy.set_move(move_ids::EXPLODER_ATTACK, 9, 1, 0);
    }
}

pub(super) fn roll_writhing_mass(enemy: &mut EnemyCombatState) {
    // Java: RNG-based move selection with anti-repeat. Has Reactive power (rerolls on hit).
    // Moves: Big Hit (32/38 A2), Multi (7/9 A2 x3), Attack+Block (15/16 A2 + block=same),
    //         Attack+Debuff (10/12 A2 + Weak 2 + Vuln 2), Mega Debuff (Parasite card, once only).
    // Deterministic MCTS: cycle through common moves, skip Mega Debuff.
    // Base damage values (non-A2): 32, 7x3, 15+15block, 10+Weak2+Vuln2
    if last_move(enemy, move_ids::WM_MULTI_HIT) {
        enemy.set_move(move_ids::WM_ATTACK_BLOCK, 15, 1, 15);
    } else if last_move(enemy, move_ids::WM_ATTACK_BLOCK) {
        enemy.set_move(move_ids::WM_ATTACK_DEBUFF, 10, 1, 0);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
    } else if last_move(enemy, move_ids::WM_ATTACK_DEBUFF) {
        enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
    } else if last_move(enemy, move_ids::WM_BIG_HIT) {
        enemy.set_move(move_ids::WM_MULTI_HIT, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::WM_BIG_HIT, 32, 1, 0);
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

pub(super) fn roll_spire_growth(enemy: &mut EnemyCombatState) {
    // Constrict then alternate Quick Tackle (16) and Smash (22)
    if last_move(enemy, move_ids::SG_CONSTRICT) || last_two_moves(enemy, move_ids::SG_SMASH) {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    } else if last_two_moves(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_CONSTRICT, 0, 0, 0);
        enemy.add_effect(mfx::CONSTRICT, 10);
    } else if last_move(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_SMASH, 22, 1, 0);
    } else {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, 16, 1, 0);
    }
}

pub(super) fn roll_maw(enemy: &mut EnemyCombatState) {
    let turn_count = enemy.entity.status(sid::TURN_COUNT) + 1;
    enemy.entity.set_status(sid::TURN_COUNT, turn_count);

    // Roar (first turn), then cycle: NomNom / Slam / Drool(Str)
    if last_move(enemy, move_ids::MAW_SLAM) || last_move(enemy, move_ids::MAW_NOM) {
        enemy.set_move(move_ids::MAW_DROOL, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
    } else if last_move(enemy, move_ids::MAW_DROOL) || last_move(enemy, move_ids::MAW_ROAR) {
        // NomNom: 5 x (turnCount/2) or Slam: 25
        let nom_hits = turn_count / 2;
        if nom_hits >= 2 {
            enemy.set_move(move_ids::MAW_NOM, 5, nom_hits, 0);
        } else {
            enemy.set_move(move_ids::MAW_SLAM, 25, 1, 0);
        }
    } else {
        enemy.set_move(move_ids::MAW_SLAM, 25, 1, 0);
    }
}

pub(super) fn roll_transient(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_giant_head(enemy: &mut EnemyCombatState) {
    // Java: count starts at 5 (A18: 4). Decremented in getMove each call.
    // When count <= 1: It Is Time mode. Damage = startingDeathDmg - count*5
    // (count goes negative: -1, -2, etc., capped at -6).
    // Before count <= 1: alternate Glare (Weak 1) and Count (13 dmg).
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
    } else {
        let new_count = count - 1;
        enemy.entity.set_status(sid::COUNT, new_count);
        // Alternate Glare and Count with anti-repeat (lastTwoMoves)
        if last_two_moves(enemy, move_ids::GH_GLARE) {
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        } else if last_two_moves(enemy, move_ids::GH_COUNT) {
            enemy.set_move(move_ids::GH_GLARE, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 1);
        } else if last_move(enemy, move_ids::GH_GLARE) {
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        } else {
            // Default: Count (attack)
            enemy.set_move(move_ids::GH_COUNT, 13, 1, 0);
        }
    }
}

pub(super) fn roll_nemesis(enemy: &mut EnemyCombatState) {
    // Java: scytheCooldown decremented FIRST in getMove, then pattern checked.
    // Intangible applied every turn in takeTurn if not already present (not just Scythe).
    // fireDmg default = 6 (A3+ = 7). Scythe always 45.
    // Burn count: 3 (A18+ = 5).
    let cooldown = enemy.entity.status(sid::SCYTHE_COOLDOWN) - 1;
    enemy.entity.set_status(sid::SCYTHE_COOLDOWN, cooldown.max(0));

    let fire_dmg = 6; // base; caller should adjust for A3+ (7)

    // Java getMove: first move handled separately
    let first_move = enemy.entity.status(sid::FIRST_MOVE) > 0;
    if first_move {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        // 50/50: Tri Attack or Burn. Deterministic: Tri Attack.
        enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
        return;
    }

    // Deterministic MCTS pattern matching Java probabilities:
    // Scythe when off cooldown and haven't used recently,
    // otherwise alternate Tri Attack and Burn with anti-repeat.
    if cooldown <= 0 && !last_move(enemy, move_ids::NEM_SCYTHE) {
        enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
        enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
    } else if last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.add_effect(mfx::BURN, 3);
    } else if last_move(enemy, move_ids::NEM_BURN) {
        enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
    } else if last_move(enemy, move_ids::NEM_SCYTHE) {
        // After Scythe: prefer Burn or Tri Attack
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.add_effect(mfx::BURN, 3);
    } else {
        enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
    }
}

pub(super) fn roll_reptomancer(enemy: &mut EnemyCombatState) {
    // Spawn -> Snake Strike (13x2 + Weak) -> Big Bite (30) -> cycle
    if last_move(enemy, move_ids::REPTO_SPAWN) {
        enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, 13, 2, 0);
        enemy.add_effect(mfx::WEAK, 1);
    } else if last_move(enemy, move_ids::REPTO_SNAKE_STRIKE) {
        enemy.set_move(move_ids::REPTO_BIG_BITE, 30, 1, 0);
    } else {
        // After Big Bite: Spawn more daggers if slots open
        enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
    }
}

pub(super) fn roll_snake_dagger(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_awakened_one(enemy: &mut EnemyCombatState) {
    let phase = enemy.entity.status(sid::PHASE);

    if phase == 1 {
        // Phase 1: Java getMove uses RNG < 25 for Soul Strike, else Slash.
        // Anti-repeat: can't use Soul Strike twice in a row, can't Slash 3 in a row.
        // Deterministic MCTS: alternate Slash and Soul Strike.
        if last_move(enemy, move_ids::AO_SLASH) {
            enemy.set_move(move_ids::AO_SOUL_STRIKE, 6, 4, 0);
        } else if last_move(enemy, move_ids::AO_SOUL_STRIKE) || last_two_moves(enemy, move_ids::AO_SLASH) {
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
        } else {
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
        }
    } else {
        // Phase 2: Dark Echo (40), Sludge (18 + Void card), Tackle (10x3).
        // Java: firstTurn of P2 = Dark Echo. Then RNG < 50 for Sludge, else Tackle.
        // Anti-repeat: Sludge can't be used 3 in a row, Tackle can't be used 3 in a row.
        // Sludge adds a Void card to draw pile (not Slimed!).
        if last_move(enemy, move_ids::AO_DARK_ECHO) {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.add_effect(mfx::VOID, 1);
        } else if last_two_moves(enemy, move_ids::AO_SLUDGE) {
            enemy.set_move(move_ids::AO_TACKLE, 10, 3, 0);
        } else if last_two_moves(enemy, move_ids::AO_TACKLE) {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.add_effect(mfx::VOID, 1);
        } else if last_move(enemy, move_ids::AO_SLUDGE) {
            enemy.set_move(move_ids::AO_TACKLE, 10, 3, 0);
        } else if last_move(enemy, move_ids::AO_TACKLE) {
            enemy.set_move(move_ids::AO_SLUDGE, 18, 1, 0);
            enemy.add_effect(mfx::VOID, 1);
        } else {
            enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
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
            let name = crate::status_ids::status_name(sid);
            if crate::powers::registry::is_debuff(name)
            {
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

pub(super) fn roll_donu(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_deca(enemy: &mut EnemyCombatState) {
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

pub(super) fn roll_time_eater(enemy: &mut EnemyCombatState) {
    // Java: Haste triggered when HP < maxHP/2 (once only).
    // Haste: remove debuffs, heal to 50%, A19 also gains headSlamDmg block.
    // Reverberate (reverbDmg x3), Head Slam (headSlamDmg + draw reduction, A19 + 2 Slimed),
    // Ripple (20 block + Vuln 1 + Weak 1, A19 also Frail 1).
    let reverb_dmg = {
        let v = enemy.entity.status(sid::REVERB_DMG);
        if v > 0 { v } else { 7 }
    };
    let head_slam_dmg = {
        let v = enemy.entity.status(sid::HEAD_SLAM_DMG);
        if v > 0 { v } else { 26 }
    };

    // Check for Haste trigger
    if enemy.entity.hp < enemy.entity.max_hp / 2 && enemy.entity.status(sid::USED_HASTE) == 0 {
        enemy.entity.set_status(sid::USED_HASTE, 1);
        enemy.set_move(move_ids::TE_HASTE, 0, 0, 0);
        enemy.add_effect(mfx::REMOVE_DEBUFFS, 1);
        enemy.add_effect(mfx::HEAL_TO_HALF, 1);
        return;
    }

    // Pattern: RNG-based in Java, deterministic for MCTS.
    // Reverberate can't be used 3 in a row, Head Slam can't repeat, Ripple can't repeat.
    if last_move(enemy, move_ids::TE_HASTE) || last_two_moves(enemy, move_ids::TE_REVERBERATE) {
        enemy.set_move(move_ids::TE_HEAD_SLAM, head_slam_dmg, 1, 0);
        // Head Slam: draw reduction (not Slimed). A19 also adds 2 Slimed.
        enemy.add_effect(mfx::DRAW_REDUCTION, 1);
    } else if last_move(enemy, move_ids::TE_HEAD_SLAM) {
        enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
        enemy.add_effect(mfx::VULNERABLE, 1);
        enemy.add_effect(mfx::WEAK, 1);
    } else if last_move(enemy, move_ids::TE_RIPPLE) {
        enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
    } else {
        enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
    }
}

