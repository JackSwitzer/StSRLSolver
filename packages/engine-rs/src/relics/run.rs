use crate::state::CombatState;
use crate::status_ids::sid;

// ==========================================================================
// 5. ON HP LOSS — wasHPLost
// ==========================================================================

/// Apply relic effects when player loses HP.
/// `damage` is the amount of HP lost.
pub fn on_hp_loss(state: &mut CombatState, damage: i32) {
    if damage <= 0 {
        return;
    }

    // Centennial Puzzle: first time taking damage, draw 3
    if state.has_relic("Centennial Puzzle") || state.has_relic("CentennialPuzzle") {
        if state.player.status(sid::CENTENNIAL_PUZZLE_READY) > 0 {
            state.player.set_status(sid::CENTENNIAL_PUZZLE_READY, 0);
            state.player.set_status(sid::CENTENNIAL_PUZZLE_DRAW, 3);
        }
    }

    // Self-Forming Clay: next turn gain 3 Block
    if state.has_relic("Self Forming Clay") || state.has_relic("SelfFormingClay") {
        state.player.add_status(sid::NEXT_TURN_BLOCK, 3);
    }

    // Runic Cube: draw 1 card when losing HP
    if state.has_relic("Runic Cube") || state.has_relic("RunicCube") {
        state.player.set_status(sid::RUNIC_CUBE_DRAW, 1);
    }

    // Red Skull: if now at <= 50% HP and not already active, +3 Strength
    if state.has_relic("Red Skull") {
        let active = state.player.status(sid::RED_SKULL_ACTIVE);
        if active == 0 && state.player.hp <= state.player.max_hp / 2 {
            state.player.add_status(sid::STRENGTH, 3);
            state.player.set_status(sid::RED_SKULL_ACTIVE, 1);
        }
    }

    // Emotion Chip: if took damage, trigger orb passive next turn
    if state.has_relic("Emotion Chip") || state.has_relic("EmotionChip") {
        state.player.set_status(sid::EMOTION_CHIP_TRIGGER, 1);
    }
}

// ==========================================================================
// 6. ON SHUFFLE — onShuffle
// ==========================================================================

/// Apply relic effects when draw pile is shuffled (discard into draw).
pub fn on_shuffle(state: &mut CombatState) {
    // Sundial: every 3 shuffles, +2 energy
    if state.has_relic("Sundial") {
        let counter = state.player.status(sid::SUNDIAL_COUNTER) + 1;
        if counter >= 3 {
            state.energy += 2;
            state.player.set_status(sid::SUNDIAL_COUNTER, 0);
        } else {
            state.player.set_status(sid::SUNDIAL_COUNTER, counter);
        }
    }

    // The Abacus: gain 6 Block on shuffle
    if state.has_relic("TheAbacus") {
        state.player.block += 6;
    }

    // Melange: scry 3 on shuffle (complex; Python handles)
}

// ==========================================================================
// 7. ON ENEMY DEATH — onMonsterDeath
// ==========================================================================

/// Apply relic effects when an enemy dies.
pub fn on_enemy_death(state: &mut CombatState, _dead_enemy_idx: usize) {
    // Gremlin Horn: gain 1 energy and draw 1 card on non-minion death
    if state.has_relic("Gremlin Horn") {
        // Only if other enemies still alive
        if state.enemies.iter().any(|e| e.is_alive()) {
            state.energy += 1;
            state.player.set_status(sid::GREMLIN_HORN_DRAW, 1);
        }
    }

    // The Specimen: transfer Poison from killed enemy to random alive enemy
    if state.has_relic("The Specimen") {
        let dead_poison = state.enemies[_dead_enemy_idx].entity.status(sid::POISON);
        if dead_poison > 0 {
            // Find first alive enemy
            if let Some(alive_idx) = state.enemies.iter()
                .enumerate()
                .find(|(i, e)| *i != _dead_enemy_idx && e.is_alive())
                .map(|(i, _)| i)
            {
                state.enemies[alive_idx].entity.add_status(sid::POISON, dead_poison);
            }
        }
    }
}

// ==========================================================================
// 8. COMBAT END — onVictory
// ==========================================================================

/// Apply relic effects when combat is won.
/// Returns HP to heal (0 if none).
pub fn on_victory(state: &mut CombatState) -> i32 {
    let mut heal = 0;

    // Black Blood: heal 12 on victory (replaces Burning Blood)
    if state.has_relic("Black Blood") {
        heal += 12;
    } else if state.has_relic("Burning Blood") {
        // Burning Blood: heal 6 on victory (skipped if Black Blood present)
        heal += 6;
    }

    // Meat on the Bone: if HP <= 50%, heal 12
    if state.has_relic("Meat on the Bone") || state.has_relic("MeatOnTheBone") {
        if state.player.hp <= state.player.max_hp / 2 {
            heal += 12;
        }
    }

    // Face of Cleric: +1 max HP on victory
    if state.has_relic("FaceOfCleric") {
        state.player.max_hp += 1;
    }

    heal
}

// ==========================================================================
// 9. DAMAGE MODIFIERS
// ==========================================================================

/// Boot: if unblocked damage is > 0 and < 5, set to 5.
pub fn apply_boot(state: &CombatState, unblocked_damage: i32) -> i32 {
    if state.has_relic("Boot") && unblocked_damage > 0 && unblocked_damage < 5 {
        5
    } else {
        unblocked_damage
    }
}

/// Torii: if unblocked attack damage is > 1 and <= 5, reduce to 1.
/// (Does NOT apply to HP_LOSS or THORNS damage types.)
pub fn apply_torii(state: &CombatState, unblocked_damage: i32) -> i32 {
    if state.has_relic("Torii") && unblocked_damage > 1 && unblocked_damage <= 5 {
        1
    } else {
        unblocked_damage
    }
}

/// Tungsten Rod: reduce all HP loss by 1 (minimum 0).
pub fn apply_tungsten_rod(state: &CombatState, damage: i32) -> i32 {
    if state.has_relic("TungstenRod") && damage > 0 {
        (damage - 1).max(0)
    } else {
        damage
    }
}

/// Champion's Belt: whenever applying Vulnerable, also apply 1 Weak.
pub fn champion_belt_on_vulnerable(state: &CombatState) -> bool {
    state.has_relic("Champion Belt")
}

/// Charon's Ashes: deal 3 damage to all enemies whenever a card is exhausted.
pub fn charons_ashes_on_exhaust(state: &mut CombatState) {
    if !state.has_relic("Charon's Ashes") && !state.has_relic("CharonsAshes") {
        return;
    }
    let living = state.living_enemy_indices();
    for idx in living {
        let enemy = &mut state.enemies[idx];
        let dmg = 3;
        let blocked = enemy.entity.block.min(dmg);
        enemy.entity.block -= blocked;
        let hp_dmg = dmg - blocked;
        enemy.entity.hp -= hp_dmg;
        state.total_damage_dealt += hp_dmg;
        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }
    }
}

/// Dead Branch: when a card is exhausted, add a random card to hand.
/// Returns true if Dead Branch should trigger (actual card generation by engine).
pub fn dead_branch_on_exhaust(state: &CombatState) -> bool {
    state.has_relic("Dead Branch")
}

/// Tough Bandages: gain 3 Block whenever a card is discarded manually.
pub fn tough_bandages_on_discard(state: &mut CombatState) {
    if state.has_relic("Tough Bandages") || state.has_relic("ToughBandages") {
        state.player.block += 3;
    }
}

/// Tingsha: deal 3 damage to random enemy when card is discarded manually.
pub fn tingsha_on_discard(state: &mut CombatState) {
    if !state.has_relic("Tingsha") {
        return;
    }
    let living = state.living_enemy_indices();
    if let Some(&idx) = living.first() {
        let enemy = &mut state.enemies[idx];
        let dmg = 3;
        let blocked = enemy.entity.block.min(dmg);
        enemy.entity.block -= blocked;
        let hp_dmg = dmg - blocked;
        enemy.entity.hp -= hp_dmg;
        state.total_damage_dealt += hp_dmg;
        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }
    }
}

/// Toy Ornithopter: heal 5 HP whenever a potion is used.
pub fn toy_ornithopter_on_potion(state: &mut CombatState) {
    if state.has_relic("Toy Ornithopter") || state.has_relic("ToyOrnithopter") {
        state.heal_player(5);
    }
}

/// Hand Drill: if attack breaks enemy Block, apply 2 Vulnerable.
pub fn hand_drill_on_block_break(state: &mut CombatState, enemy_idx: usize) {
    if state.has_relic("HandDrill") && enemy_idx < state.enemies.len() {
        state.enemies[enemy_idx].entity.add_status(sid::VULNERABLE, 2);
    }
}

/// Strike Dummy: +3 damage on Strikes (simplified passive).
pub fn strike_dummy_bonus(state: &CombatState) -> i32 {
    if state.has_relic("StrikeDummy") {
        3
    } else {
        0
    }
}

/// Wrist Blade: +4 damage on 0-cost attacks.
pub fn wrist_blade_bonus(state: &CombatState) -> i32 {
    if state.has_relic("WristBlade") {
        4
    } else {
        0
    }
}

/// Snecko Skull: +1 Poison when applying Poison.
pub fn snecko_skull_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Snake Skull") || state.has_relic("SneckoSkull") {
        1
    } else {
        0
    }
}

/// Chemical X: +2 to X-cost effects.
pub fn chemical_x_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Chemical X") || state.has_relic("ChemicalX") {
        2
    } else {
        0
    }
}

/// Gold Plated Cables: if HP is full, orbs passive trigger extra.
pub fn gold_plated_cables_active(state: &CombatState) -> bool {
    state.has_relic("Cables") && state.player.hp == state.player.max_hp
}

/// Apply Violet Lotus: +1 energy on Calm exit.
pub fn violet_lotus_calm_exit_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Violet Lotus") || state.has_relic("VioletLotus") {
        1
    } else {
        0
    }
}

/// Unceasing Top: if hand is empty, draw 1.
pub fn unceasing_top_should_draw(state: &CombatState) -> bool {
    (state.has_relic("Unceasing Top") || state.has_relic("UnceasingTop"))
        && state.hand.is_empty()
        && (!state.draw_pile.is_empty() || !state.discard_pile.is_empty())
}

/// Runic Pyramid: don't discard hand at end of turn.
pub fn has_runic_pyramid(state: &CombatState) -> bool {
    state.has_relic("Runic Pyramid") || state.has_relic("RunicPyramid")
}

/// Calipers: retain up to 15 Block between turns.
pub fn calipers_block_retention(state: &CombatState, current_block: i32) -> i32 {
    if state.has_relic("Calipers") {
        current_block.min(15).max(0)
    } else {
        0
    }
}

/// Ice Cream: energy carries over between turns.
pub fn has_ice_cream(state: &CombatState) -> bool {
    state.has_relic("Ice Cream") || state.has_relic("IceCream")
}

/// Sacred Bark: double potion effectiveness.
pub fn has_sacred_bark(state: &CombatState) -> bool {
    state.has_relic("SacredBark")
}

/// Necronomicon: first 2+-cost attack per turn plays twice.
pub fn necronomicon_should_trigger(state: &CombatState, card_cost: i32, is_attack: bool) -> bool {
    if !state.has_relic("Necronomicon") {
        return false;
    }
    is_attack && card_cost >= 2 && state.player.status(sid::NECRONOMICON_USED) == 0
}

/// Mark Necronomicon as used for this turn.
pub fn necronomicon_mark_used(state: &mut CombatState) {
    state.player.set_status(sid::NECRONOMICON_USED, 1);
}

/// Reset Necronomicon at turn start.
pub fn necronomicon_reset(state: &mut CombatState) {
    if state.has_relic("Necronomicon") {
        state.player.set_status(sid::NECRONOMICON_USED, 0);
    }
}

