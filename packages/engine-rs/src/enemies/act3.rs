use crate::state::EnemyCombatState;
use crate::combat_types::{fx, mfx, Intent};
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::seed::StsRandom;
use crate::status_ids::sid;

// =========================================================================
// Act 3 Basic Enemies
// =========================================================================

pub(super) fn roll_darkling(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    // Source: reference/extracted/methods/monster/Darkling.java (`getMove`).
    if enemy.entity.status(sid::REBIRTH_PENDING) > 0 {
        enemy.set_move(move_ids::DARK_REINCARNATE, 0, 0, 0);
        return;
    }

    let chomp = enemy.entity.status(sid::STARTING_DMG).max(8);
    let nip = enemy.entity.status(sid::STR_AMT).max(7);
    let harden = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::DARK_HARDEN, 0, 0, 12);
        if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
            enemy.add_effect(mfx::STRENGTH, 2);
        }
    };

    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        if num < 50 {
            harden(enemy);
        } else {
            enemy.set_move(move_ids::DARK_NIP, nip, 1, 0);
        }
        return;
    }

    let mut roll = num;
    loop {
        if roll < 40 {
            if !last_move(enemy, move_ids::DARK_CHOMP)
                && enemy.entity.status(sid::COUNT) % 2 == 0
            {
                enemy.set_move(move_ids::DARK_CHOMP, chomp, 2, 0);
                return;
            }
            // Odd-position Darklings and repeat Chomps recurse only into the
            // 40..=99 portion of the table.
            roll = ai_rng.random_range(40, 99);
        } else if roll < 70 {
            if !last_move(enemy, move_ids::DARK_HARDEN) {
                harden(enemy);
            } else {
                enemy.set_move(move_ids::DARK_NIP, nip, 1, 0);
            }
            return;
        } else if !last_two_moves(enemy, move_ids::DARK_NIP) {
            enemy.set_move(move_ids::DARK_NIP, nip, 1, 0);
            return;
        } else {
            roll = ai_rng.random(99);
        }
    }
}

pub(super) fn roll_orb_walker(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/OrbWalker.java (`getMove`).
    let laser_damage = enemy.entity.status(sid::STARTING_DMG).max(10);
    let claw_damage = enemy.entity.status(sid::STR_AMT).max(15);
    let laser = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::OW_LASER, laser_damage, 1, 0);
        enemy.add_effect(mfx::BURN_DRAW_DISCARD, 1);
        enemy.intent = Intent::AttackDebuff {
            damage: laser_damage as i16,
            hits: 1,
            effects: fx::BURN,
        };
    };
    let claw = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::OW_CLAW, claw_damage, 1, 0);
    };

    if num < 40 {
        if !last_two_moves(enemy, move_ids::OW_CLAW) { claw(enemy); }
        else { laser(enemy); }
    } else if !last_two_moves(enemy, move_ids::OW_LASER) {
        laser(enemy);
    } else {
        claw(enemy);
    }
}

pub(super) fn roll_spiker(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/Spiker.java (`getMove`).
    // `COUNT` is thornsCount: it increments only when a buff turn executes.
    let damage = enemy.entity.status(sid::STARTING_DMG).max(7);
    if enemy.entity.status(sid::COUNT) > 5
        || (num < 50 && !last_move(enemy, move_ids::SPIKER_ATTACK))
    {
        enemy.set_move(move_ids::SPIKER_ATTACK, damage, 1, 0);
    } else {
        enemy.set_move(move_ids::SPIKER_BUFF, 0, 0, 0);
        enemy.add_effect(mfx::THORNS, 2);
    }
}

pub(super) fn roll_repulsor(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/Repulsor.java (`getMove`).
    if num < 20 && !last_move(enemy, move_ids::REPULSOR_ATTACK) {
        let damage = enemy.entity.status(sid::STARTING_DMG).max(11);
        enemy.set_move(move_ids::REPULSOR_ATTACK, damage, 1, 0);
    } else {
        enemy.set_move(move_ids::REPULSOR_DAZE, 0, 0, 0);
        enemy.add_effect(mfx::DAZE_DRAW, 2);
        enemy.intent = Intent::Debuff { effects: fx::DAZE };
    }
}

pub(super) fn roll_exploder(enemy: &mut EnemyCombatState, _num: i32) {
    // Source: reference/extracted/methods/monster/Exploder.java (`getMove`).
    // turnCount increments in takeTurn, before the queued RollMoveAction.
    if enemy.entity.status(sid::TURN_COUNT) >= 2 {
        enemy.set_move(move_ids::EXPLODER_EXPLODE, 0, 0, 0);
    } else {
        let damage = enemy.entity.status(sid::STARTING_DMG).max(9);
        enemy.set_move(move_ids::EXPLODER_ATTACK, damage, 1, 0);
    }
}

pub(super) fn roll_writhing_mass(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut StsRandom,
) {
    // Source: reference/extracted/methods/monster/WrithingMass.java (`getMove`).
    let big = enemy.entity.status(sid::STARTING_DMG).max(32);
    let multi = enemy.entity.status(sid::STR_AMT).max(7);
    let attack_block = enemy.entity.status(sid::BLOCK_AMT).max(15);
    let attack_debuff = enemy.entity.status(sid::HEAD_SLAM_DMG).max(10);

    let set_big = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::WM_BIG_HIT, big, 1, 0);
    };
    let set_multi = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::WM_MULTI_HIT, multi, 3, 0);
    };
    let set_attack_block = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::WM_ATTACK_BLOCK, attack_block, 1, attack_block);
    };
    let set_attack_debuff = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::WM_ATTACK_DEBUFF, attack_debuff, 1, 0);
        enemy.intent = crate::combat_types::Intent::AttackDebuff {
            damage: attack_debuff as i16,
            hits: 1,
            effects: 0,
        };
        enemy.add_effect(mfx::WEAK, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
    };

    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        if num < 33 {
            set_multi(enemy);
        } else if num < 66 {
            set_attack_block(enemy);
        } else {
            set_attack_debuff(enemy);
        }
        return;
    }

    if num < 10 {
        if !last_move(enemy, move_ids::WM_BIG_HIT) {
            set_big(enemy);
        } else {
            let reroll = ai_rng.random_range(10, 99);
            roll_writhing_mass(enemy, reroll, ai_rng);
        }
    } else if num < 20 {
        if enemy.entity.status(sid::USED_MEGA_DEBUFF) == 0
            && !last_move(enemy, move_ids::WM_MEGA_DEBUFF)
        {
            enemy.set_move(move_ids::WM_MEGA_DEBUFF, 0, 0, 0);
            enemy.intent = crate::combat_types::Intent::Debuff { effects: 0 };
        } else if ai_rng.random_float() < 0.1 {
            set_big(enemy);
        } else {
            let reroll = ai_rng.random_range(20, 99);
            roll_writhing_mass(enemy, reroll, ai_rng);
        }
    } else if num < 40 {
        if !last_move(enemy, move_ids::WM_ATTACK_DEBUFF) {
            set_attack_debuff(enemy);
        } else if ai_rng.random_float() < 0.4 {
            let reroll = ai_rng.random(19);
            roll_writhing_mass(enemy, reroll, ai_rng);
        } else {
            let reroll = ai_rng.random_range(40, 99);
            roll_writhing_mass(enemy, reroll, ai_rng);
        }
    } else if num < 70 {
        if !last_move(enemy, move_ids::WM_MULTI_HIT) {
            set_multi(enemy);
        } else if ai_rng.random_float() < 0.3 {
            set_attack_block(enemy);
        } else {
            let reroll = ai_rng.random(39);
            roll_writhing_mass(enemy, reroll, ai_rng);
        }
    } else if !last_move(enemy, move_ids::WM_ATTACK_BLOCK) {
        set_attack_block(enemy);
    } else {
        let reroll = ai_rng.random(69);
        roll_writhing_mass(enemy, reroll, ai_rng);
    }
}

/// ReactivePower queues RollMoveAction without recording the unexecuted intent.
pub fn writhing_mass_reactive_reroll(
    enemy: &mut EnemyCombatState,
    ai_rng: &mut StsRandom,
) {
    let num = ai_rng.random(99);
    roll_writhing_mass(enemy, num, ai_rng);
}

pub(super) fn roll_spire_growth(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/SpireGrowth.java (`getMove`).
    let tackle = enemy.entity.status(sid::STARTING_DMG).max(16);
    let smash = enemy.entity.status(sid::STR_AMT).max(22);
    let constrict = enemy.entity.status(sid::BLOCK_AMT).max(10) as i16;
    let player_constricted = enemy.entity.status(sid::COUNT) > 0;
    let high_ascension = enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0;
    let set_constrict = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::SG_CONSTRICT, 0, 0, 0);
        enemy.add_effect(mfx::CONSTRICT, constrict);
        enemy.intent = Intent::Debuff { effects: 0 };
    };

    if high_ascension && !player_constricted
        && !last_move(enemy, move_ids::SG_CONSTRICT)
    {
        set_constrict(enemy);
    } else if num < 50 && !last_two_moves(enemy, move_ids::SG_QUICK_TACKLE) {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, tackle, 1, 0);
    } else if !player_constricted && !last_move(enemy, move_ids::SG_CONSTRICT) {
        set_constrict(enemy);
    } else if !last_two_moves(enemy, move_ids::SG_SMASH) {
        enemy.set_move(move_ids::SG_SMASH, smash, 1, 0);
    } else {
        enemy.set_move(move_ids::SG_QUICK_TACKLE, tackle, 1, 0);
    }
}

pub(super) fn roll_maw(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/Maw.java (`getMove`).
    // Java increments turnCount on every selection, including the opening
    // roll that always chooses Roar while `roared` is false.
    let turn_count = enemy.entity.status(sid::TURN_COUNT) + 1;
    enemy.entity.set_status(sid::TURN_COUNT, turn_count);
    let slam_damage = enemy.entity.status(sid::STARTING_DMG).max(25);
    let strength = enemy.entity.status(sid::STR_AMT).max(3) as i16;
    let terrify = enemy.entity.status(sid::BLOCK_AMT).max(3) as i16;

    // FIRST_MOVE mirrors the constructor's `roared` boolean: zero until the
    // Roar intent executes, then one for the rest of combat.
    if enemy.entity.status(sid::FIRST_MOVE) == 0 {
        enemy.set_move(move_ids::MAW_ROAR, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, terrify);
        enemy.add_effect(mfx::FRAIL, terrify);
    } else if num < 50 && !last_move(enemy, move_ids::MAW_NOM) {
        enemy.set_move(move_ids::MAW_NOM, 5, (turn_count / 2).max(1), 0);
    } else if last_move(enemy, move_ids::MAW_SLAM)
        || last_move(enemy, move_ids::MAW_NOM)
    {
        enemy.set_move(move_ids::MAW_DROOL, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, strength);
    } else {
        enemy.set_move(move_ids::MAW_SLAM, slam_damage, 1, 0);
    }
}

pub(super) fn roll_transient(enemy: &mut EnemyCombatState, _num: i32) {
    let count = enemy.entity.status(sid::ATTACK_COUNT);
    // Source: reference/extracted/methods/monster/Transient.java (`getMove`).
    // Selection only reads count; takeTurn increments it and uses SetMoveAction.
    let starting_dmg = enemy.entity.status(sid::STARTING_DMG);
    let base = if starting_dmg > 0 { starting_dmg } else { 30 };
    let dmg = base + count * 10;
    enemy.set_move(move_ids::TRANSIENT_ATTACK, dmg, 1, 0);
}

// =========================================================================
// Act 3 Elites
// =========================================================================

pub(super) fn roll_giant_head(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/GiantHead.java (`getMove`).
    // Count starts at 5 (A18: 4) and decrements on every selection. Above one,
    // num splits Glare/Count at 50 with only a two-in-a-row guard.
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
}

pub(super) fn roll_nemesis(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    // Source: reference/extracted/methods/monster/Nemesis.java (`getMove`).
    // scytheCooldown decrements before every branch, including the opener.
    let cooldown = enemy.entity.status(sid::SCYTHE_COOLDOWN) - 1;
    enemy.entity.set_status(sid::SCYTHE_COOLDOWN, cooldown);
    let fire_dmg = enemy.entity.status(sid::STARTING_DMG).max(6);
    let burn_count = enemy.entity.status(sid::BLOCK_AMT).max(3) as i16;
    let tri_attack = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::NEM_TRI_ATTACK, fire_dmg, 3, 0);
    };
    let burn = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::NEM_BURN, 0, 0, 0);
        enemy.add_effect(mfx::BURN, burn_count);
    };
    let scythe = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::NEM_SCYTHE, 45, 1, 0);
        enemy.entity.set_status(sid::SCYTHE_COOLDOWN, 2);
    };

    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        if num < 50 { tri_attack(enemy); } else { burn(enemy); }
        return;
    }

    if num < 30 {
        if !last_move(enemy, move_ids::NEM_SCYTHE) && cooldown <= 0 {
            scythe(enemy);
        } else if ai_rng.random_boolean() {
            if !last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
                tri_attack(enemy);
            } else {
                burn(enemy);
            }
        } else if !last_move(enemy, move_ids::NEM_BURN) {
            burn(enemy);
        } else {
            tri_attack(enemy);
        }
    } else if num < 65 {
        if !last_two_moves(enemy, move_ids::NEM_TRI_ATTACK) {
            tri_attack(enemy);
        } else if ai_rng.random_boolean() {
            if cooldown > 0 {
                burn(enemy);
            } else {
                scythe(enemy);
            }
        } else {
            burn(enemy);
        }
    } else if !last_move(enemy, move_ids::NEM_BURN) {
        burn(enemy);
    } else if ai_rng.random_boolean() && cooldown <= 0 {
        scythe(enemy);
    } else {
        tri_attack(enemy);
    }
}

pub(super) fn roll_reptomancer(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    // Source: reference/extracted/methods/monster/Reptomancer.java (`getMove`).
    let strike_damage = enemy.entity.status(sid::STARTING_DMG).max(13);
    let bite_damage = enemy.entity.status(sid::STR_AMT).max(30);
    let strike = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::REPTO_SNAKE_STRIKE, strike_damage, 2, 0);
        enemy.add_effect(mfx::WEAK, 1);
        enemy.intent = Intent::AttackDebuff {
            damage: strike_damage as i16,
            hits: 2,
            effects: fx::WEAK,
        };
    };
    let spawn = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::REPTO_SPAWN, 0, 0, 0);
        enemy.intent = Intent::Unknown;
    };

    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        spawn(enemy);
        return;
    }

    let mut roll = num;
    loop {
        if roll < 33 {
            if !last_move(enemy, move_ids::REPTO_SNAKE_STRIKE) {
                strike(enemy);
                return;
            }
            roll = ai_rng.random_range(33, 99);
        } else if roll < 66 {
            if !last_two_moves(enemy, move_ids::REPTO_SPAWN) {
                if enemy.entity.status(sid::COUNT) <= 3 {
                    spawn(enemy);
                } else {
                    strike(enemy);
                }
            } else {
                strike(enemy);
            }
            return;
        } else if !last_move(enemy, move_ids::REPTO_BIG_BITE) {
            enemy.set_move(move_ids::REPTO_BIG_BITE, bite_damage, 1, 0);
            return;
        } else {
            roll = ai_rng.random(65);
        }
    }
}

pub(super) fn roll_snake_dagger(enemy: &mut EnemyCombatState, _num: i32) {
    // Source: reference/extracted/methods/monster/SnakeDagger.java (`getMove`).
    // Its initialized first move is always Wound; every later roll is Explode.
    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        enemy.set_move(move_ids::SD_WOUND, 9, 1, 0);
        enemy.add_effect(mfx::WOUND, 1);
    } else {
        enemy.set_move(move_ids::SD_EXPLODE, 25, 1, 0);
    }
}

// =========================================================================
// Act 3 Bosses
// =========================================================================

pub(super) fn roll_awakened_one(enemy: &mut EnemyCombatState, num: i32) {
    let phase = enemy.entity.status(sid::PHASE);

    if phase == 1 {
        // Java getMove: first intent is always Slash. Thereafter num < 25
        // selects Soul Strike unless it was just used; num >= 25 selects Slash
        // unless that would be the third consecutive Slash.
        // Java: reference/extracted/methods/monster/AwakenedOne.java
        if enemy.move_history.is_empty() {
            enemy.set_move(move_ids::AO_SLASH, 20, 1, 0);
        } else if num < 25 {
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
        // Java getMove: first phase-two intent is Dark Echo. Thereafter num <
        // 50 chooses Sludge unless it would be the third; num >= 50 chooses
        // Tackle unless it would be the third.
        // Java: reference/extracted/methods/monster/AwakenedOne.java
        if enemy.move_history.is_empty() {
            enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
        } else if num < 50 {
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
    enemy.entity.set_status(sid::TEMP_STRENGTH_LOSS, 0);
    if enemy.entity.strength() < 0 {
        enemy.entity.set_status(sid::STRENGTH, 0);
    }
    // Heal to full (second form HP)
    enemy.entity.hp = enemy.entity.max_hp;
    enemy.move_history.clear();
    // First move of Phase 2: Dark Echo
    enemy.set_move(move_ids::AO_DARK_ECHO, 40, 1, 0);
}

pub(super) fn roll_donu(enemy: &mut EnemyCombatState, _num: i32) {
    // Source: reference/extracted/methods/monster/Donu.java (`getMove` and
    // `takeTurn`). Donu starts with Circle and alternates after execution.
    if last_move(enemy, move_ids::DONU_CIRCLE) {
        let bd = { let v = enemy.entity.status(sid::BEAM_DMG); if v > 0 { v } else { 10 } };
        enemy.set_move(move_ids::DONU_BEAM, bd, 2, 0);
    } else {
        enemy.set_move(move_ids::DONU_CIRCLE, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 3);
        enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, 3);
    }
}

pub(super) fn roll_deca(enemy: &mut EnemyCombatState, _num: i32) {
    // Source: reference/extracted/methods/monster/Deca.java (`getMove` and
    // `takeTurn`). Deca starts attacking and alternates after each execution.
    if last_move(enemy, move_ids::DECA_BEAM) {
        enemy.set_move(move_ids::DECA_SQUARE, 0, 0, 16);
        enemy.add_effect(mfx::BLOCK_ALL_ALLIES, 16);
        if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
            enemy.add_effect(mfx::PLATED_ARMOR_ALL, 3);
        }
    } else {
        let bd = { let v = enemy.entity.status(sid::BEAM_DMG); if v > 0 { v } else { 10 } };
        enemy.set_move(move_ids::DECA_BEAM, bd, 2, 0);
        enemy.add_effect(mfx::DAZE, 2);
    }
}

pub(super) fn roll_time_eater(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut StsRandom,
) {
    // Source: reference/extracted/methods/monster/TimeEater.java (`getMove`).
    // Repeated Reverberate/Ripple branches recursively draw replacement nums;
    // repeated Head Slam consumes a 0.66 probability float.
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
        enemy.intent = Intent::Buff { effects: 0 };
        enemy.add_effect(mfx::REMOVE_DEBUFFS, 1);
        enemy.add_effect(mfx::HEAL_TO_HALF, 1);
        return;
    }

    if num < 45 {
        if last_two_moves(enemy, move_ids::TE_REVERBERATE) {
            let reroll = ai_rng.random_range(50, 99);
            roll_time_eater(enemy, reroll, ai_rng);
            return;
        }
        enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
        return;
    }

    if num < 80 {
        if !last_move(enemy, move_ids::TE_HEAD_SLAM) {
            enemy.set_move(move_ids::TE_HEAD_SLAM, head_slam_dmg, 1, 0);
            enemy.intent = Intent::AttackDebuff {
                damage: head_slam_dmg as i16,
                hits: 1,
                effects: 0,
            };
            enemy.add_effect(mfx::DRAW_REDUCTION, 1);
            if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
                enemy.add_effect(mfx::SLIMED, 2);
            }
        } else if ai_rng.random_float() < 0.66 {
            enemy.set_move(move_ids::TE_REVERBERATE, reverb_dmg, 3, 0);
        } else {
            set_time_eater_ripple(enemy);
        }
        return;
    }

    if !last_move(enemy, move_ids::TE_RIPPLE) {
        set_time_eater_ripple(enemy);
    } else {
        let reroll = ai_rng.random(74);
        roll_time_eater(enemy, reroll, ai_rng);
    }
}

fn set_time_eater_ripple(enemy: &mut EnemyCombatState) {
    enemy.set_move(move_ids::TE_RIPPLE, 0, 0, 20);
    enemy.add_effect(mfx::VULNERABLE, 1);
    enemy.add_effect(mfx::WEAK, 1);
    if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
        enemy.add_effect(mfx::FRAIL, 1);
    }
}
