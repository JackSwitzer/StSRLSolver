use crate::state::EntityState;

// Debuff-related power trigger functions

pub fn decrement_debuffs(entity: &mut EntityState) {
    decrement_status(entity, "Weakened");
    decrement_status(entity, "Vulnerable");
    decrement_status(entity, "Frail");
}

/// Decrement a single status by 1. Remove if it reaches 0.
pub fn decrement_status(entity: &mut EntityState, key: &str) {
    if let Some(val) = entity.statuses.get(key).copied() {
        if val <= 1 {
            entity.statuses.remove(key);
        } else {
            entity.statuses.insert(key.to_string(), val - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// Poison
// ---------------------------------------------------------------------------

/// Apply poison tick to an entity. Returns damage dealt.
/// Poison decrements by 1 each tick, removed at 0.

pub fn tick_poison(entity: &mut EntityState) -> i32 {
    let poison = entity.status("Poison");
    if poison <= 0 {
        return 0;
    }

    let damage = poison;
    entity.hp -= damage;

    let new_poison = poison - 1;
    entity.set_status("Poison", new_poison);

    damage
}

// ---------------------------------------------------------------------------
// End-of-turn triggers
// ---------------------------------------------------------------------------

/// Apply Metallicize block gain at end of turn.

pub fn apply_lose_strength(entity: &mut EntityState) {
    let lose_str = entity.status("LoseStrength");
    if lose_str > 0 {
        entity.add_status("Strength", -lose_str);
        entity.set_status("LoseStrength", 0);
    }
}

/// Apply LoseDexterity at start of turn (undo temporary Dexterity gains).

pub fn apply_lose_dexterity(entity: &mut EntityState) {
    let lose_dex = entity.status("LoseDexterity");
    if lose_dex > 0 {
        entity.add_status("Dexterity", -lose_dex);
        entity.set_status("LoseDexterity", 0);
    }
}

/// Remove Flame Barrier at start of turn (it only lasts 1 turn).

pub fn apply_wraith_form(entity: &mut EntityState) {
    let wraith = entity.status("Wraith Form");
    if wraith > 0 {
        entity.add_status("Dexterity", -wraith);
    }
}

/// Demon Form: gain N Strength at start of turn.

pub fn modify_damage_receive(entity: &EntityState, damage: f64) -> f64 {
    let mut d = damage;

    // Slow: +10% per stack
    let slow = entity.status("Slow");
    if slow > 0 {
        d *= 1.0 + (slow as f64 * 0.1);
    }

    // Intangible: cap at 1
    if entity.status("Intangible") > 0 && d > 1.0 {
        d = 1.0;
    }

    d
}

/// Modify block amount based on powers.

pub fn decrement_fading(entity: &mut EntityState) -> bool {
    let fading = entity.status("Fading");
    if fading > 0 {
        let new_val = fading - 1;
        entity.set_status("Fading", new_val);
        if new_val <= 0 {
            return true;
        }
    }
    false
}

/// Explosive countdown. Returns damage to deal when it reaches 0.

pub fn decrement_blur(entity: &mut EntityState) {
    decrement_status(entity, "Blur");
}

/// Decrement Intangible at end of turn.

pub fn decrement_intangible(entity: &mut EntityState) {
    decrement_status(entity, "Intangible");
}

/// Decrement Lock-On at end of round.

pub fn decrement_lock_on(entity: &mut EntityState) {
    decrement_status(entity, "Lock-On");
}

/// Reset Invincible at end of round (Champ).

pub fn apply_debuff(entity: &mut EntityState, status: &str, amount: i32) -> bool {
    let artifact = entity.status("Artifact");
    if artifact > 0 {
        // Artifact blocks the debuff and decrements
        entity.set_status("Artifact", artifact - 1);
        return false;
    }

    entity.add_status(status, amount);
    true
}

/// Apply a debuff with Sadistic Nature check. Returns damage to deal from Sadistic.

pub fn apply_debuff_with_sadistic(
    target: &mut EntityState,
    status: &str,
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
/// The `amount` field tracks remaining damage allowed.

pub fn apply_invincible_cap(entity: &mut EntityState, incoming_damage: i32) -> i32 {
    let inv = entity.status("Invincible");
    if inv > 0 {
        if incoming_damage > inv {
            entity.set_status("Invincible", 0);
            return inv;
        } else {
            entity.set_status("Invincible", inv - incoming_damage);
            return incoming_damage;
        }
    }
    incoming_damage
}

// ---------------------------------------------------------------------------
// ModeShift (Guardian)
// ---------------------------------------------------------------------------

/// ModeShift: track damage. Returns true if threshold reached.

pub fn apply_mode_shift_damage(entity: &mut EntityState, damage: i32) -> bool {
    let ms = entity.status("Mode Shift");
    if ms > 0 {
        let new_val = ms - damage;
        if new_val <= 0 {
            entity.set_status("Mode Shift", 0);
            return true;
        }
        entity.set_status("Mode Shift", new_val);
    }
    false
}

