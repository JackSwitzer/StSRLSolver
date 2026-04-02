use crate::state::EntityState;
use crate::status_ids::sid;

// Enemy-specific power trigger functions

pub fn apply_ritual(entity: &mut EntityState) {
    let ritual = entity.status(sid::RITUAL);
    if ritual > 0 {
        entity.add_status(sid::STRENGTH, ritual);
    }
}

/// Apply GenericStrengthUp (enemy version of Ritual, gains each turn).

pub fn apply_generic_strength_up(entity: &mut EntityState) {
    let amount = entity.status(sid::GENERIC_STRENGTH_UP);
    if amount > 0 {
        entity.add_status(sid::STRENGTH, amount);
    }
}

// ---------------------------------------------------------------------------
// Start-of-turn triggers
// ---------------------------------------------------------------------------

/// Apply LoseStrength at start of turn (undo temporary Strength gains).

pub fn get_beat_of_death_damage(entity: &EntityState) -> i32 {
    entity.status(sid::BEAT_OF_DEATH)
}

/// Slow: increment counter when player plays a card on this enemy.

pub fn increment_slow(entity: &mut EntityState) {
    let slow = entity.status(sid::SLOW);
    if slow >= 0 && entity.statuses.contains_key(&sid::SLOW) {
        entity.add_status(sid::SLOW, 1);
    }
}

/// TimeWarp: increment card counter. Returns true if 12 reached (end turn + gain Str).
/// TimeWarp uses sid::TIME_WARP_ACTIVE as a presence flag and sid::TIME_WARP for the counter.
/// The counter starts at 0 and increments; at 12 it resets and triggers.

pub fn increment_time_warp(entity: &mut EntityState) -> bool {
    if entity.status(sid::TIME_WARP_ACTIVE) <= 0 {
        return false;
    }
    let tw = entity.status(sid::TIME_WARP);
    let new_val = tw + 1;
    if new_val >= 12 {
        entity.set_status(sid::TIME_WARP, 0);
        return true;
    }
    // Use insert directly to allow storing intermediate values including 0
    entity.statuses.insert(sid::TIME_WARP, new_val);
    false
}

/// Panache: tracks cards played, every 5 deals 10 damage to all enemies.
/// Returns damage to deal (0 or panache amount).

pub fn apply_angry_on_hit(entity: &mut EntityState) {
    let angry = entity.status(sid::ANGRY);
    if angry > 0 {
        entity.add_status(sid::STRENGTH, angry);
    }
}

/// Envenom: returns Poison amount to apply when dealing unblocked attack damage.

pub fn apply_curiosity(entity: &mut EntityState) {
    let curiosity = entity.status(sid::CURIOSITY);
    if curiosity > 0 {
        entity.add_status(sid::STRENGTH, curiosity);
    }
}

/// Rupture: gain Strength when losing HP from a card.

pub fn reset_slow(entity: &mut EntityState) {
    if entity.statuses.contains_key(&sid::SLOW) {
        entity.set_status(sid::SLOW, 0);
    }
}

/// Decrement Fading. Returns true if entity should die (Fading reaches 0).

pub fn decrement_explosive(entity: &mut EntityState) -> i32 {
    let explosive = entity.status(sid::EXPLOSIVE);
    if explosive > 0 {
        let new_val = explosive - 1;
        entity.set_status(sid::EXPLOSIVE, new_val);
        if new_val <= 0 {
            // Explosive deals its stored damage amount
            // The damage is stored separately; typically 30-50
            return 30; // Default bomb damage
        }
    }
    0
}

/// Growth: gain Strength and Block at end of round.
/// In the Java source, Growth adds Strength and Block (not Dexterity).

pub fn apply_growth(entity: &mut EntityState) {
    let growth = entity.status(sid::GROWTH);
    if growth > 0 {
        entity.add_status(sid::STRENGTH, growth);
        entity.block += growth;
    }
}

/// Decrement Blur at end of round.

pub fn reset_invincible(entity: &mut EntityState, max_amount: i32) {
    if entity.statuses.contains_key(&sid::INVINCIBLE) {
        entity.set_status(sid::INVINCIBLE, max_amount);
    }
}

// ---------------------------------------------------------------------------
// TheBomb countdown
// ---------------------------------------------------------------------------

/// TheBomb: decrement counter. Returns (should_explode, damage).

pub fn decrement_the_bomb(entity: &mut EntityState) -> (bool, i32) {
    let turns = entity.status(sid::THE_BOMB_TURNS);
    let damage = entity.status(sid::THE_BOMB);
    if turns > 0 && damage > 0 {
        let new_turns = turns - 1;
        entity.set_status(sid::THE_BOMB_TURNS, new_turns);
        if new_turns <= 0 {
            entity.set_status(sid::THE_BOMB, 0);
            entity.set_status(sid::THE_BOMB_TURNS, 0);
            return (true, damage);
        }
    }
    (false, 0)
}

// ---------------------------------------------------------------------------
// Combust end-of-turn
// ---------------------------------------------------------------------------

/// Combust: lose 1 HP, deal N damage to all enemies.
/// Returns (hp_loss, damage_per_enemy).

/// Regeneration: heal HP and decrement stacks. Returns amount healed.
/// The simple variant returns the heal amount without applying it.
pub fn apply_regeneration(entity: &mut EntityState) -> i32 {
    let regen = entity.status(sid::REGENERATION);
    if regen > 0 {
        entity.set_status(sid::REGENERATION, regen - 1);
        return regen;
    }
    0
}

/// Enemy Regeneration: heal HP capped at max_hp, decrement stacks.
pub fn apply_enemy_regeneration(entity: &mut EntityState, max_hp: i32) {
    let regen = entity.status(sid::REGENERATION);
    if regen > 0 {
        entity.hp = (entity.hp + regen).min(max_hp);
        entity.add_status(sid::REGENERATION, -1);
    }
}

// ---------------------------------------------------------------------------
// Regrow end-of-turn (enemy)
// ---------------------------------------------------------------------------

/// Regrow: heal. Returns HP to heal.

pub fn get_regrow_heal(entity: &EntityState) -> i32 {
    entity.status(sid::REGROW)
}

// ---------------------------------------------------------------------------
// End-of-turn removal: Rage
// ---------------------------------------------------------------------------

/// Remove Rage at end of turn.

pub fn get_spore_cloud_vulnerable(entity: &EntityState) -> i32 {
    entity.status(sid::SPORE_CLOUD)
}

// ---------------------------------------------------------------------------
// Fading (enemy death countdown)
// ---------------------------------------------------------------------------

/// Fading: decrement at end of turn. Returns true if entity should die.
pub fn tick_fading(entity: &mut EntityState) -> bool {
    let fading = entity.status(sid::FADING);
    if fading > 0 {
        let new_val = fading - 1;
        entity.set_status(sid::FADING, new_val);
        if new_val <= 0 {
            return true; // enemy should die
        }
    }
    false
}

// ---------------------------------------------------------------------------
// TheBomb countdown (standalone helper)
// ---------------------------------------------------------------------------

/// TheBomb: decrement counter. Returns damage to deal to player (0 if not yet).
pub fn tick_the_bomb(entity: &mut EntityState) -> i32 {
    let bomb = entity.status(sid::THE_BOMB);
    if bomb > 0 {
        let turns = entity.status(sid::THE_BOMB_TURNS);
        if turns > 1 {
            entity.set_status(sid::THE_BOMB_TURNS, turns - 1);
            0
        } else {
            entity.set_status(sid::THE_BOMB, 0);
            entity.set_status(sid::THE_BOMB_TURNS, 0);
            bomb // return damage to deal to player
        }
    } else {
        0
    }
}

