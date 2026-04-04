use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 4 — The Ending
// =========================================================================

pub(super) fn roll_spire_shield(enemy: &mut EnemyCombatState) {
    // Java: moveCount % 3 cycle. moveCount post-incremented.
    // Bash: A3+ = 14, else 12. Smash: A3+ = 38, else 34.
    // Fortify: 30 block to ALL monsters. Smash: A18 gains 99 block, else damage-dealt block.
    let mc = enemy.entity.status(sid::MOVE_COUNT);

    match mc % 3 {
        0 => {
            // 50/50 Fortify or Bash. Deterministic: Bash if not last, else Fortify.
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, 12, 1, 0);
                enemy.add_effect(mfx::STRENGTH_DOWN, 1);
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            }
        }
        1 => {
            // The other of Bash/Fortify
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, 12, 1, 0);
                enemy.add_effect(mfx::STRENGTH_DOWN, 1);
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
            }
        }
        _ => {
            // Smash (34 dmg + block)
            enemy.set_move(move_ids::SHIELD_SMASH, 34, 1, 0);
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}

pub(super) fn roll_spire_spear(enemy: &mut EnemyCombatState) {
    // Java: moveCount % 3, post-incremented.
    // A3+: burnStrikeDmg=6, skewerCount=4. Else 5, 3. Skewer always 10 per hit.
    // Burn Strike: A18 adds burns to draw pile, else discard.
    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let skewer_count = enemy.entity.status(sid::SKEWER_COUNT).max(3);

    match mc % 3 {
        0 => {
            // Burn Strike or Piercer
            if !last_move(enemy, move_ids::SPEAR_BURN_STRIKE) {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, 5, 2, 0);
                enemy.add_effect(mfx::BURN, 2);
            } else {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.add_effect(mfx::STRENGTH, 2);
            }
        }
        1 => {
            // Skewer: 10 x skewerCount
            enemy.set_move(move_ids::SPEAR_SKEWER, 10, skewer_count, 0);
        }
        _ => {
            // 50/50 Piercer or Burn Strike
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

pub(super) fn roll_corrupt_heart(enemy: &mut EnemyCombatState) {
    // Java: isFirstMove handled separately. Then moveCount % 3 cycle.
    // moveCount incremented AFTER getMove (post-increment).
    let is_first = enemy.entity.status(sid::IS_FIRST_MOVE) > 0;
    if is_first {
        // After Debilitate: moveCount starts at 0
        enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
    }

    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let blood_count = enemy.entity.status(sid::BLOOD_HIT_COUNT).max(12);
    let echo_dmg = enemy.entity.status(sid::ECHO_DMG).max(40);

    // Java: 3-move cycle. moveCount % 3:
    // 0: 50/50 Blood Shots or Echo
    // 1: whichever wasn't used in slot 0 (anti-repeat)
    // 2: Buff (+2 Str + escalating buff based on buffCount)
    match mc % 3 {
        0 => {
            // Deterministic: Blood Shots first
            enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
        }
        1 => {
            // Use the other attack
            if !last_move(enemy, move_ids::HEART_ECHO) {
                enemy.set_move(move_ids::HEART_ECHO, echo_dmg, 1, 0);
            } else {
                enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
            }
        }
        _ => {
            // Buff: +2 Str + escalating buff (Artifact 2, +1 BeatOfDeath, PainfulStabs, +10 Str, +50 Str)
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

