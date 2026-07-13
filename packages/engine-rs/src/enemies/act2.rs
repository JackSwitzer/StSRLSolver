use crate::state::EnemyCombatState;
use crate::combat_types::mfx;
use super::{last_move, last_two_moves};
use super::move_ids;
use crate::status_ids::sid;

// =========================================================================
// Act 2 Basic Enemies
// =========================================================================

pub(super) fn roll_chosen(enemy: &mut EnemyCombatState, num: i32) {
    let zap_damage = enemy.entity.status(sid::STARTING_DMG).max(18);
    let debilitate_damage = enemy.entity.status(sid::STR_AMT).max(10);
    let poke_damage = enemy.entity.status(sid::SLAP_DMG).max(5);
    let high_ai = enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0;

    let poke = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::CHOSEN_POKE, poke_damage, 2, 0);
    };

    // Source: reference/extracted/methods/monster/Chosen.java (`getMove`).
    if !high_ai && enemy.entity.status(sid::FIRST_TURN) > 0 {
        enemy.entity.set_status(sid::FIRST_TURN, 0);
        poke(enemy);
        return;
    }

    if enemy.entity.status(sid::FIRST_MOVE) == 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 1);
        enemy.set_move(move_ids::CHOSEN_HEX, 0, 0, 0);
        enemy.add_effect(mfx::HEX, 1);
        return;
    }

    if !last_move(enemy, move_ids::CHOSEN_DEBILITATE)
        && !last_move(enemy, move_ids::CHOSEN_DRAIN)
    {
        if num < 50 {
            enemy.set_move(move_ids::CHOSEN_DEBILITATE, debilitate_damage, 1, 0);
            enemy.add_effect(mfx::VULNERABLE, 2);
        } else {
            enemy.set_move(move_ids::CHOSEN_DRAIN, 0, 0, 0);
            enemy.add_effect(mfx::WEAK, 3);
            enemy.add_effect(mfx::STRENGTH, 3);
        }
    } else if num < 40 {
        enemy.set_move(move_ids::CHOSEN_ZAP, zap_damage, 1, 0);
    } else {
        poke(enemy);
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

pub(super) fn roll_byrd(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    let peck_damage = enemy.entity.status(sid::STARTING_DMG).max(1);
    let peck_count = enemy.entity.status(sid::STR_AMT).max(5);
    let swoop_damage = enemy.entity.status(sid::SLASH_DMG).max(12);
    let headbutt_damage = enemy.entity.status(sid::HEAD_SLAM_DMG).max(3);

    let peck = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::BYRD_PECK, peck_damage, peck_count, 0);
    };
    let swoop = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::BYRD_SWOOP, swoop_damage, 1, 0);
    };
    let caw = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::BYRD_CAW, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, 1);
    };

    // Source: reference/extracted/methods/monster/Byrd.java (`getMove`).
    // The opening roll ignores `num` but consumes a conditional 37.5% draw.
    if enemy.entity.status(sid::FIRST_MOVE) > 0 {
        enemy.entity.set_status(sid::FIRST_MOVE, 0);
        if ai_rng.random_float() < 0.375 {
            caw(enemy);
        } else {
            peck(enemy);
        }
        return;
    }

    if enemy.entity.status(sid::FLIGHT) <= 0 {
        enemy.set_move(move_ids::BYRD_HEADBUTT, headbutt_damage, 1, 0);
    } else if num < 50 {
        if last_two_moves(enemy, move_ids::BYRD_PECK) {
            if ai_rng.random_float() < 0.4 {
                swoop(enemy);
            } else {
                caw(enemy);
            }
        } else {
            peck(enemy);
        }
    } else if num < 70 {
        if last_move(enemy, move_ids::BYRD_SWOOP) {
            if ai_rng.random_float() < 0.375 {
                caw(enemy);
            } else {
                peck(enemy);
            }
        } else {
            swoop(enemy);
        }
    } else if last_move(enemy, move_ids::BYRD_CAW) {
        if ai_rng.random_float() < 0.2857 {
            swoop(enemy);
        } else {
            peck(enemy);
        }
    } else {
        caw(enemy);
    }
}

pub(super) fn roll_shelled_parasite(enemy: &mut EnemyCombatState, _num: i32) {
    // Cycle: Double Strike (6x2), Life Suck (10), Fell (18 + Frail 2)
    if last_move(enemy, move_ids::SP_DOUBLE_STRIKE) {
        enemy.set_move(move_ids::SP_LIFE_SUCK, 10, 1, 0);
        enemy.add_effect(mfx::HEAL, 10);
    } else if last_move(enemy, move_ids::SP_LIFE_SUCK) {
        enemy.set_move(move_ids::SP_FELL, 18, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    } else {
        enemy.set_move(move_ids::SP_DOUBLE_STRIKE, 6, 2, 0);
    }
}

pub(super) fn roll_snake_plant(enemy: &mut EnemyCombatState, _num: i32) {
    // 65% Chomp (7x3), 35% Spores (Weak 2 + Frail 2). Anti-repeat.
    if last_two_moves(enemy, move_ids::SNAKE_CHOMP) {
        enemy.set_move(move_ids::SNAKE_SPORES, 0, 0, 0);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.add_effect(mfx::FRAIL, 2);
    } else if last_move(enemy, move_ids::SNAKE_SPORES) {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    } else {
        enemy.set_move(move_ids::SNAKE_CHOMP, 7, 3, 0);
    }
}

pub(super) fn roll_centurion(enemy: &mut EnemyCombatState, num: i32) {
    let slash_damage = enemy.entity.status(sid::STARTING_DMG).max(12);
    let fury_damage = enemy.entity.status(sid::STR_AMT).max(6);
    let fury_hits = enemy.entity.status(sid::ATTACK_COUNT).max(3);
    let block = enemy.entity.status(sid::BLOCK_AMT).max(15);
    let has_ally = enemy.entity.status(sid::COUNT) > 1;

    let slash = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::CENT_SLASH, slash_damage, 1, 0);
    };
    let protect_or_fury = |enemy: &mut EnemyCombatState| {
        if has_ally {
            enemy.set_move(move_ids::CENT_PROTECT, 0, 0, 0);
            enemy.add_effect(mfx::BLOCK_RANDOM_OTHER, block as i16);
        } else {
            enemy.set_move(move_ids::CENT_FURY, fury_damage, fury_hits, 0);
        }
    };

    // Source: reference/extracted/methods/monster/Centurion.java (`getMove`).
    if num >= 65
        && !last_two_moves(enemy, move_ids::CENT_PROTECT)
        && !last_two_moves(enemy, move_ids::CENT_FURY)
    {
        protect_or_fury(enemy);
    } else if !last_two_moves(enemy, move_ids::CENT_SLASH) {
        slash(enemy);
    } else {
        protect_or_fury(enemy);
    }
}

pub(super) fn roll_healer(enemy: &mut EnemyCombatState, num: i32) {
    // Source: reference/extracted/methods/monster/Healer.java (`getMove`).
    // COUNT mirrors the summed missing HP of every living monster.
    let damage = enemy.entity.status(sid::STARTING_DMG).max(8);
    let strength = enemy.entity.status(sid::STR_AMT).max(2) as i16;
    let heal = enemy.entity.status(sid::BLOCK_AMT).max(16) as i16;
    let high_ai = enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0;
    let need_to_heal = enemy.entity.status(sid::COUNT);

    if need_to_heal > if high_ai { 20 } else { 15 }
        && !last_two_moves(enemy, move_ids::MYSTIC_HEAL)
    {
        enemy.set_move(move_ids::MYSTIC_HEAL, 0, 0, 0);
        enemy.add_effect(mfx::HEAL_ALL, heal);
    } else if num >= 40
        && if high_ai {
            !last_move(enemy, move_ids::MYSTIC_ATTACK)
        } else {
            !last_two_moves(enemy, move_ids::MYSTIC_ATTACK)
        }
    {
        enemy.set_move(move_ids::MYSTIC_ATTACK, damage, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    } else if !last_two_moves(enemy, move_ids::MYSTIC_BUFF) {
        enemy.set_move(move_ids::MYSTIC_BUFF, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, strength);
        enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, strength);
    } else {
        enemy.set_move(move_ids::MYSTIC_ATTACK, damage, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
    }
}

pub(super) fn roll_book_of_stabbing(enemy: &mut EnemyCombatState, num: i32) {
    // Java: reference/extracted/methods/monster/BookOfStabbing.java (`getMove`).
    let stab_count = enemy.entity.status(sid::STAB_COUNT);
    let a18 = enemy.entity.status(sid::BLOCK_AMT) > 0;
    let stab_damage = enemy.entity.status(sid::STARTING_DMG);
    let big_stab_damage = enemy.entity.status(sid::STR_AMT);
    if num < 15 && last_move(enemy, move_ids::BOOK_BIG_STAB) {
        let new_count = stab_count + 1;
        enemy.entity.set_status(sid::STAB_COUNT, new_count);
        enemy.set_move(move_ids::BOOK_STAB, stab_damage, new_count, 0);
    } else if num < 15 || last_two_moves(enemy, move_ids::BOOK_STAB) {
        if a18 {
            enemy.entity.set_status(sid::STAB_COUNT, stab_count + 1);
        }
        enemy.set_move(move_ids::BOOK_BIG_STAB, big_stab_damage, 1, 0);
    } else {
        let new_count = stab_count + 1;
        enemy.entity.set_status(sid::STAB_COUNT, new_count);
        enemy.set_move(move_ids::BOOK_STAB, stab_damage, new_count, 0);
    }
}

pub(super) fn roll_gremlin_leader(
    enemy: &mut EnemyCombatState,
    num: i32,
    ai_rng: &mut crate::seed::StsRandom,
) {
    // Source: reference/extracted/methods/monster/GremlinLeader.java
    // (`getMove`). COUNT mirrors numAliveGremlins for this selection.
    let alive = enemy.entity.status(sid::COUNT);
    let strength = enemy.entity.status(sid::STR_AMT).max(3) as i16;
    let block = enemy.entity.status(sid::BLOCK_AMT).max(6) as i16;
    let rally = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::GL_RALLY, 0, 0, 0);
    };
    let encourage = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::GL_ENCOURAGE, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, strength);
        enemy.add_effect(mfx::STRENGTH_ALL_ALLIES, strength);
        enemy.add_effect(mfx::BLOCK_ALL_ALLIES, block);
    };
    let stab = |enemy: &mut EnemyCombatState| {
        enemy.set_move(move_ids::GL_STAB, 6, 3, 0);
    };

    if alive == 0 {
        if num < 75 {
            if !last_move(enemy, move_ids::GL_RALLY) { rally(enemy); } else { stab(enemy); }
        } else if !last_move(enemy, move_ids::GL_STAB) {
            stab(enemy);
        } else {
            rally(enemy);
        }
    } else if alive == 1 {
        if num < 50 {
            if !last_move(enemy, move_ids::GL_RALLY) {
                rally(enemy);
            } else {
                let retry = ai_rng.random_range(50, 99);
                roll_gremlin_leader(enemy, retry, ai_rng);
            }
        } else if num < 80 {
            if !last_move(enemy, move_ids::GL_ENCOURAGE) { encourage(enemy); } else { stab(enemy); }
        } else if !last_move(enemy, move_ids::GL_STAB) {
            stab(enemy);
        } else {
            let retry = ai_rng.random_range(0, 80);
            roll_gremlin_leader(enemy, retry, ai_rng);
        }
    } else if num < 66 {
        if !last_move(enemy, move_ids::GL_ENCOURAGE) { encourage(enemy); } else { stab(enemy); }
    } else if !last_move(enemy, move_ids::GL_STAB) {
        stab(enemy);
    } else {
        encourage(enemy);
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

pub(super) fn roll_snecko(enemy: &mut EnemyCombatState, _num: i32) {
    // First turn: Glare. Then alternate Tail (8 + Vuln 2) and Bite (15)
    if last_move(enemy, move_ids::SNECKO_GLARE) || last_two_moves(enemy, move_ids::SNECKO_BITE) {
        enemy.set_move(move_ids::SNECKO_TAIL, 8, 1, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
    } else {
        enemy.set_move(move_ids::SNECKO_BITE, 15, 1, 0);
    }
}

pub(super) fn roll_bear(enemy: &mut EnemyCombatState, _num: i32) {
    // BanditBear.getMove always selects the one-time Bear Hug opener. Later
    // intents are installed directly by takeTurn and never call rollMove.
    // Java: reference/extracted/methods/monster/BanditBear.java
    enemy.set_move(move_ids::BEAR_HUG, 0, 0, 0);
    enemy.add_effect(
        mfx::DEX_DOWN,
        enemy.entity.status(sid::BLOCK_AMT) as i16,
    );
}

pub(crate) fn advance_bear_after_turn(enemy: &mut EnemyCombatState) {
    // takeTurn case 2 sets Lunge, then cases 3 and 1 alternate Lunge/Maul.
    // SetMoveAction does not consume aiRng.
    // Java: reference/extracted/methods/monster/BanditBear.java
    let completed_move = enemy.move_id;
    enemy.move_history.push(completed_move);
    enemy.move_effects.clear();
    if completed_move == move_ids::BEAR_LUNGE {
        enemy.set_move(
            move_ids::BEAR_MAUL,
            enemy.entity.status(sid::STARTING_DMG),
            1,
            0,
        );
    } else {
        enemy.set_move(
            move_ids::BEAR_LUNGE,
            enemy.entity.status(sid::STR_AMT),
            1,
            9,
        );
    }
}

pub(super) fn roll_bandit_pointy(enemy: &mut EnemyCombatState, _num: i32) {
    // BanditPointy.getMove always selects its two-hit attack.
    // Java: reference/extracted/methods/monster/BanditPointy.java
    enemy.set_move(
        move_ids::POINTY_STAB,
        enemy.entity.status(sid::STARTING_DMG),
        2,
        0,
    );
}

pub(crate) fn advance_bandit_pointy_after_turn(enemy: &mut EnemyCombatState) {
    // takeTurn repeats the same intent with SetMoveAction, not RollMoveAction.
    // Java: reference/extracted/methods/monster/BanditPointy.java
    enemy.move_history.push(enemy.move_id);
    enemy.move_effects.clear();
    enemy.set_move(
        move_ids::POINTY_STAB,
        enemy.entity.status(sid::STARTING_DMG),
        2,
        0,
    );
}

pub(super) fn roll_bandit_leader(enemy: &mut EnemyCombatState, _num: i32) {
    // BanditLeader.getMove always selects the one-time Mock opener. All later
    // intents are installed by takeTurn through SetMoveAction.
    // Java: reference/extracted/methods/monster/BanditLeader.java
    enemy.set_move(move_ids::BANDIT_MOCK, 0, 0, 0);
}

pub(crate) fn advance_bandit_leader_after_turn(enemy: &mut EnemyCombatState) {
    // Mock -> Agonizing Slash -> Cross Slash. Below A17, Cross returns to
    // Agonizing Slash; at A17, it repeats once unless the last two were Cross.
    // SetMoveAction does not consume aiRng.
    // Java: reference/extracted/methods/monster/BanditLeader.java
    let completed_move = enemy.move_id;
    enemy.move_history.push(completed_move);
    enemy.move_effects.clear();
    match completed_move {
        move_ids::BANDIT_MOCK | move_ids::BANDIT_CROSS_SLASH
            if completed_move == move_ids::BANDIT_MOCK
                || enemy.entity.status(sid::BLOCK_AMT) < 3
                || last_two_moves(enemy, move_ids::BANDIT_CROSS_SLASH) =>
        {
            enemy.set_move(
                move_ids::BANDIT_AGONIZE,
                enemy.entity.status(sid::STR_AMT),
                1,
                0,
            );
            enemy.add_effect(
                mfx::WEAK,
                enemy.entity.status(sid::BLOCK_AMT) as i16,
            );
        }
        _ => enemy.set_move(
            move_ids::BANDIT_CROSS_SLASH,
            enemy.entity.status(sid::STARTING_DMG),
            1,
            0,
        ),
    }
}

// =========================================================================
// Act 2 Bosses
// =========================================================================

pub(super) fn roll_bronze_automaton(enemy: &mut EnemyCombatState, _num: i32) {
    // Java: reference/extracted/methods/monster/BronzeAutomaton.java (`getMove`).
    let flail = enemy.entity.status(sid::FLAIL_DMG);
    let beam = enemy.entity.status(sid::BEAM_DMG);
    let strength = enemy.entity.status(sid::STR_AMT);
    let block = enemy.entity.status(sid::BLOCK_AMT);
    if enemy.entity.status(sid::FIRST_TURN) > 0 {
        enemy.entity.set_status(sid::FIRST_TURN, 0);
        enemy.set_move(move_ids::BA_SPAWN_ORBS, 0, 0, 0);
        return;
    }
    let num_turns = enemy.entity.status(sid::NUM_TURNS);
    if num_turns == 4 {
        enemy.entity.set_status(sid::NUM_TURNS, 0);
        enemy.set_move(move_ids::BA_HYPER_BEAM, beam, 1, 0);
        return;
    }
    if last_move(enemy, move_ids::BA_HYPER_BEAM) {
        if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
            enemy.set_move(move_ids::BA_BOOST, 0, 0, block);
            enemy.add_effect(mfx::STRENGTH, strength as i16);
        } else {
            enemy.set_move(move_ids::BA_STUNNED, 0, 0, 0);
        }
        return;
    }
    if last_move(enemy, move_ids::BA_STUNNED)
        || last_move(enemy, move_ids::BA_BOOST)
        || last_move(enemy, move_ids::BA_SPAWN_ORBS)
    {
        enemy.set_move(move_ids::BA_FLAIL, flail, 2, 0);
    } else {
        enemy.set_move(move_ids::BA_BOOST, 0, 0, block);
        enemy.add_effect(mfx::STRENGTH, strength as i16);
    }
    enemy.entity.set_status(sid::NUM_TURNS, num_turns + 1);
}

pub(super) fn roll_bronze_orb(enemy: &mut EnemyCombatState, num: i32) {
    // Java: reference/extracted/methods/monster/BronzeOrb.java (`getMove`).
    if enemy.entity.status(sid::FIRST_MOVE) == 0 && num >= 25 {
        enemy.entity.set_status(sid::FIRST_MOVE, 1);
        enemy.set_move(move_ids::BO_STASIS, 0, 0, 0);
        enemy.add_effect(mfx::STASIS, 1);
    } else if num >= 70 && !last_two_moves(enemy, move_ids::BO_SUPPORT) {
        enemy.set_move(move_ids::BO_SUPPORT, 0, 0, 12);
    } else if !last_two_moves(enemy, move_ids::BO_BEAM) {
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

    // Source: reference/extracted/methods/monster/Champ.java (`getMove`).
    let threshold_reached_now = enemy.entity.hp < enemy.entity.max_hp / 2;

    if threshold_reached_now && enemy.entity.status(sid::THRESHOLD_REACHED) == 0 {
        enemy.entity.set_status(sid::THRESHOLD_REACHED, 1);
        enemy.set_move(move_ids::CHAMP_ANGER, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, (str_amt * 3) as i16);
        enemy.add_effect(mfx::REMOVE_DEBUFFS, 1);
        return;
    }

    let history_len = enemy.move_history.len();
    let last_move_before_execute = history_len >= 2
        && enemy.move_history[history_len - 2] == move_ids::CHAMP_EXECUTE;
    if enemy.entity.status(sid::THRESHOLD_REACHED) > 0
        && !last_move(enemy, move_ids::CHAMP_EXECUTE)
        && !last_move_before_execute
    {
        enemy.set_move(move_ids::CHAMP_EXECUTE, 10, 2, 0);
        return;
    }

    if num_turns == 4 && enemy.entity.status(sid::THRESHOLD_REACHED) == 0 {
        enemy.set_move(move_ids::CHAMP_TAUNT, 0, 0, 0);
        enemy.add_effect(mfx::VULNERABLE, 2);
        enemy.add_effect(mfx::WEAK, 2);
        enemy.entity.set_status(sid::NUM_TURNS, 0);
        return;
    }

    let forge_times = enemy.entity.status(sid::FORGE_TIMES);
    let forge_roll_max = if enemy.entity.status(sid::HIGH_ASCENSION_AI) > 0 {
        30
    } else {
        15
    };
    if !last_move(enemy, move_ids::CHAMP_DEFENSIVE)
        && forge_times < 2
        && num <= forge_roll_max
    {
        enemy.entity.set_status(sid::FORGE_TIMES, forge_times + 1);
        enemy.set_move(move_ids::CHAMP_DEFENSIVE, 0, 0, block_amt);
        enemy.add_effect(mfx::METALLICIZE, forge_amt as i16);
    } else if !last_move(enemy, move_ids::CHAMP_GLOAT)
        && !last_move(enemy, move_ids::CHAMP_DEFENSIVE)
        && num <= 30
    {
        enemy.set_move(move_ids::CHAMP_GLOAT, 0, 0, 0);
        enemy.add_effect(mfx::STRENGTH, str_amt as i16);
    } else if !last_move(enemy, move_ids::CHAMP_FACE_SLAP) && num <= 55 {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
    } else if !last_move(enemy, move_ids::CHAMP_HEAVY_SLASH) {
        enemy.set_move(move_ids::CHAMP_HEAVY_SLASH, slash_dmg, 1, 0);
    } else {
        enemy.set_move(move_ids::CHAMP_FACE_SLAP, slap_dmg, 1, 0);
        enemy.add_effect(mfx::FRAIL, 2);
        enemy.add_effect(mfx::VULNERABLE, 2);
    }
}

pub(super) fn roll_collector(enemy: &mut EnemyCombatState, _num: i32) {
    let fd = { let v = enemy.entity.status(sid::FIREBALL_DMG); if v > 0 { v } else { 18 } };
    let sa = { let v = enemy.entity.status(sid::STR_AMT); if v > 0 { v } else { 3 } };
    let ba = { let v = enemy.entity.status(sid::BLOCK_AMT); if v > 0 { v } else { 15 } };
    let turns = enemy.move_history.len();
    if turns == 4 && !enemy.move_history.iter().any(|&m| m == move_ids::COLL_MEGA_DEBUFF) {
        enemy.set_move(move_ids::COLL_MEGA_DEBUFF, 0, 0, 0);
        enemy.add_effect(mfx::VULNERABLE, 3);
        enemy.add_effect(mfx::WEAK, 3);
        enemy.add_effect(mfx::FRAIL, 3);
    } else if last_two_moves(enemy, move_ids::COLL_FIREBALL) {
        enemy.set_move(move_ids::COLL_BUFF, 0, 0, ba);
        enemy.add_effect(mfx::STRENGTH, sa as i16);
    } else if last_move(enemy, move_ids::COLL_BUFF) || last_move(enemy, move_ids::COLL_MEGA_DEBUFF) {
        enemy.set_move(move_ids::COLL_FIREBALL, fd, 1, 0);
    } else {
        enemy.set_move(move_ids::COLL_FIREBALL, fd, 1, 0);
    }
}
