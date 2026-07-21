use super::last_move;
use super::move_ids;
use crate::combat_types::{mfx, Intent};
use crate::seed::StsRandom;
use crate::state::EnemyCombatState;
use crate::status_ids::sid;

// =========================================================================
// Act 4 — The Ending
// =========================================================================

pub(super) fn roll_spire_shield(enemy: &mut EnemyCombatState, ai_rng: &mut StsRandom) {
    // Source: reference/extracted/methods/monster/SpireShield.java (`getMove`).
    // Slot zero consumes a conditional randomBoolean in addition to the
    // AbstractMonster.rollMove integer; slots one and two consume no extra RNG.
    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let bash_damage = enemy.entity.status(sid::STARTING_DMG).max(12);
    let smash_damage = enemy.entity.status(sid::STR_AMT).max(34);

    match mc % 3 {
        0 => {
            if ai_rng.random_bool() {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
                enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 30);
            } else {
                enemy.set_move(move_ids::SHIELD_BASH, bash_damage, 1, 0);
                enemy.intent = Intent::AttackDebuff {
                    damage: bash_damage as i16,
                    hits: 1,
                    effects: 0,
                };
            }
        }
        1 => {
            if !last_move(enemy, move_ids::SHIELD_BASH) {
                enemy.set_move(move_ids::SHIELD_BASH, bash_damage, 1, 0);
                enemy.intent = Intent::AttackDebuff {
                    damage: bash_damage as i16,
                    hits: 1,
                    effects: 0,
                };
            } else {
                enemy.set_move(move_ids::SHIELD_FORTIFY, 0, 0, 30);
                enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 30);
            }
        }
        _ => {
            enemy.set_move(move_ids::SHIELD_SMASH, smash_damage, 1, 0);
            // Java's ATTACK_DEFEND intent does not encode the later dynamic
            // GainBlockAction amount; execution derives it from DamageInfo.output.
            enemy.intent = Intent::AttackBlock {
                damage: smash_damage as i16,
                hits: 1,
                block: 0,
                effects: 0,
            };
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}

pub(super) fn roll_spire_spear(enemy: &mut EnemyCombatState, ai_rng: &mut StsRandom) {
    // Source: reference/extracted/methods/monster/SpireSpear.java (`getMove`).
    // Only slot two consumes a conditional randomBoolean in addition to the
    // AbstractMonster.rollMove integer.
    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let burn_damage = enemy.entity.status(sid::STARTING_DMG).max(5);
    let skewer_count = enemy.entity.status(sid::SKEWER_COUNT).max(3);

    match mc % 3 {
        0 => {
            if !last_move(enemy, move_ids::SPEAR_BURN_STRIKE) {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, burn_damage, 2, 0);
                enemy.intent = Intent::AttackDebuff {
                    damage: burn_damage as i16,
                    hits: 2,
                    effects: 0,
                };
                enemy.add_effect(mfx::BURN, 2);
            } else {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.add_effect(mfx::STRENGTH, 2);
                enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 2);
            }
        }
        1 => {
            enemy.set_move(move_ids::SPEAR_SKEWER, 10, skewer_count, 0);
        }
        _ => {
            if ai_rng.random_bool() {
                enemy.set_move(move_ids::SPEAR_PIERCER, 0, 0, 0);
                enemy.add_effect(mfx::STRENGTH, 2);
                enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 2);
            } else {
                enemy.set_move(move_ids::SPEAR_BURN_STRIKE, burn_damage, 2, 0);
                enemy.intent = Intent::AttackDebuff {
                    damage: burn_damage as i16,
                    hits: 2,
                    effects: 0,
                };
                enemy.add_effect(mfx::BURN, 2);
            }
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}

pub(super) fn roll_corrupt_heart(enemy: &mut EnemyCombatState, _num: i32, ai_rng: &mut StsRandom) {
    // Source: reference/extracted/methods/monster/CorruptHeart.java (`getMove`).
    // AbstractMonster.rollMove has already consumed `num`. The first call
    // returns Debilitate without advancing moveCount; cycle slot zero consumes
    // one additional aiRng boolean to choose between the two attacks.
    if enemy.entity.status(sid::IS_FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::IS_FIRST_MOVE, 0);
        enemy.set_move(move_ids::HEART_DEBILITATE, 0, 0, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.add_effect(mfx::FRAIL, 2);
        enemy.add_effect(mfx::HEART_STATUS_CARDS, 1);
        return;
    }

    let mc = enemy.entity.status(sid::MOVE_COUNT);
    let blood_count = enemy.entity.status(sid::BLOOD_HIT_COUNT).max(12);
    let echo_dmg = enemy.entity.status(sid::ECHO_DMG).max(40);

    match mc % 3 {
        0 => {
            if ai_rng.random_bool() {
                enemy.set_move(move_ids::HEART_BLOOD_SHOTS, 2, blood_count, 0);
            } else {
                enemy.set_move(move_ids::HEART_ECHO, echo_dmg, 1, 0);
            }
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
            // The Strength cleanup, escalation, and buffCount increment happen
            // in takeTurn, not while this intent is selected.
            enemy.set_move(move_ids::HEART_BUFF, 0, 0, 0);
        }
    }
    enemy.entity.set_status(sid::MOVE_COUNT, mc + 1);
}
