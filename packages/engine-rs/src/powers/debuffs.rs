use crate::ids::StatusId;
use crate::state::EntityState;
use crate::status_ids::sid;

// Debuff-related power trigger functions

pub fn decrement_debuffs(entity: &mut EntityState) {
    decrement_status(entity, sid::WEAKENED);
    decrement_status(entity, sid::VULNERABLE);
    decrement_status(entity, sid::FRAIL);
}

/// Decrement a single status by 1. Remove if it reaches 0.
pub fn decrement_status(entity: &mut EntityState, key: StatusId) {
    let val = entity.status(key);
    if val > 0 {
        entity.set_status(key, val - 1);
    }
}

// ---------------------------------------------------------------------------
// Poison
// ---------------------------------------------------------------------------

/// Apply poison tick to an entity. Returns damage dealt.
/// Poison decrements by 1 each tick, removed at 0.

pub fn tick_poison(entity: &mut EntityState) -> i32 {
    let poison = entity.status(sid::POISON);
    if poison <= 0 {
        return 0;
    }

    let damage = poison;
    entity.hp -= damage;

    let new_poison = poison - 1;
    entity.set_status(sid::POISON, new_poison);

    damage
}

// ---------------------------------------------------------------------------
// End-of-turn triggers
// ---------------------------------------------------------------------------

/// Remove temporary Strength (LoseStrength) at end of turn.

pub fn apply_lose_strength(entity: &mut EntityState) {
    let lose_str = entity.status(sid::LOSE_STRENGTH);
    if lose_str > 0 {
        entity.add_status(sid::STRENGTH, -lose_str);
        entity.set_status(sid::LOSE_STRENGTH, 0);
    }
}

/// Apply LoseDexterity at start of turn (undo temporary Dexterity gains).

pub fn apply_lose_dexterity(entity: &mut EntityState) {
    let lose_dex = entity.status(sid::LOSE_DEXTERITY);
    if lose_dex > 0 {
        entity.add_status(sid::DEXTERITY, -lose_dex);
        entity.set_status(sid::LOSE_DEXTERITY, 0);
    }
}

/// Wraith Form: lose N Dexterity per stack each turn.

pub fn apply_wraith_form(entity: &mut EntityState) {
    let wraith = entity.status(sid::WRAITH_FORM);
    if wraith > 0 {
        entity.add_status(sid::DEXTERITY, -wraith);
    }
}

/// Modify incoming damage based on Slow and Intangible.

pub fn modify_damage_receive(entity: &EntityState, damage: f64) -> f64 {
    let mut d = damage;

    // Slow: +10% per stack
    let slow = entity.status(sid::SLOW);
    if slow > 0 {
        d *= 1.0 + (slow as f64 * 0.1);
    }

    // Intangible: cap at 1
    if entity.status(sid::INTANGIBLE) > 0 && d > 1.0 {
        d = 1.0;
    }

    d
}

/// Decrement Fading by 1. Returns true if entity should die (fading reached 0).

pub fn decrement_fading(entity: &mut EntityState) -> bool {
    let fading = entity.status(sid::FADING);
    if fading > 0 {
        let new_val = fading - 1;
        entity.set_status(sid::FADING, new_val);
        if new_val <= 0 {
            return true;
        }
    }
    false
}

/// Decrement Blur at end of turn.

pub fn decrement_blur(entity: &mut EntityState) {
    decrement_status(entity, sid::BLUR);
}

/// Decrement Intangible at end of turn.

pub fn decrement_intangible(entity: &mut EntityState) {
    decrement_status(entity, sid::INTANGIBLE);
}

/// Decrement Lock-On at end of round.

pub fn decrement_lock_on(entity: &mut EntityState) {
    decrement_status(entity, sid::LOCK_ON);
}

/// Reset Invincible at end of round (Champ).

pub fn apply_debuff(entity: &mut EntityState, status: StatusId, amount: i32) -> bool {
    let artifact = entity.status(sid::ARTIFACT);
    if artifact > 0 {
        // Artifact blocks the debuff and decrements
        entity.set_status(sid::ARTIFACT, artifact - 1);
        return false;
    }

    // Ginger blocks Weak
    if status == sid::WEAKENED && entity.status(sid::HAS_GINGER) > 0 {
        return false;
    }

    // Turnip blocks Frail
    if status == sid::FRAIL && entity.status(sid::HAS_TURNIP) > 0 {
        return false;
    }

    entity.add_status(status, amount);
    true
}

/// Apply a debuff with Sadistic Nature check. Returns damage to deal from Sadistic.

pub fn apply_debuff_with_sadistic(
    target: &mut EntityState,
    status: StatusId,
    amount: i32,
    source_sadistic: i32,
) -> (bool, i32) {
    let applied = apply_debuff(target, status, amount);
    if applied && source_sadistic > 0 {
        (true, source_sadistic)
    } else {
        (applied, 0)
    }
}

// ---------------------------------------------------------------------------
// Invincible damage cap
// ---------------------------------------------------------------------------

/// Invincible: cap total damage this turn. Returns capped damage.
/// The `Invincible` status value tracks remaining damage allowed this turn.
/// Call `reset_invincible` at start of each turn to restore the cap.

pub fn apply_invincible_cap(entity: &mut EntityState, incoming_damage: i32) -> i32 {
    let inv = entity.status(sid::INVINCIBLE);
    if inv > 0 {
        if incoming_damage > inv {
            entity.set_status(sid::INVINCIBLE, 0);
            return inv;
        } else {
            entity.set_status(sid::INVINCIBLE, inv - incoming_damage);
            return incoming_damage;
        }
    }
    incoming_damage
}

/// Invincible: per-turn cap using a separate damage-taken tracker.
/// Leaves the INVINCIBLE cap itself unchanged so it persists across turns.
/// Reset via `reset_invincible_damage_taken` at start of each turn.
pub fn apply_invincible_cap_tracked(entity: &mut EntityState, raw_damage: i32) -> i32 {
    let cap = entity.status(sid::INVINCIBLE);
    if cap <= 0 {
        return raw_damage;
    }
    let taken_this_turn = entity.status(sid::INVINCIBLE_DAMAGE_TAKEN);
    let remaining = (cap - taken_this_turn).max(0);
    let capped = raw_damage.min(remaining);
    entity.set_status(sid::INVINCIBLE_DAMAGE_TAKEN, taken_this_turn + capped);
    capped
}

/// Reset Invincible per-turn damage tracking. Call at start of each turn.
pub fn reset_invincible_damage_taken(entity: &mut EntityState) {
    entity.set_status(sid::INVINCIBLE_DAMAGE_TAKEN, 0);
}

// ---------------------------------------------------------------------------
// Slow damage multiplier
// ---------------------------------------------------------------------------

/// Slow: returns the damage multiplier for an entity with Slow stacks.
/// Each stack adds +10% damage taken.
pub fn slow_damage_multiplier(entity: &EntityState) -> f64 {
    let slow = entity.status(sid::SLOW);
    if slow > 0 {
        1.0 + (slow as f64 * 0.10)
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------
// ModeShift (Guardian)
// ---------------------------------------------------------------------------

/// ModeShift: track damage. Returns true if threshold reached.

pub fn apply_mode_shift_damage(entity: &mut EntityState, damage: i32) -> bool {
    let ms = entity.status(sid::MODE_SHIFT);
    if ms > 0 {
        let new_val = ms - damage;
        if new_val <= 0 {
            entity.set_status(sid::MODE_SHIFT, 0);
            return true;
        }
        entity.set_status(sid::MODE_SHIFT, new_val);
    }
    false
}

