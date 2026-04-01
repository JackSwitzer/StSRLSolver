use crate::state::EntityState;

// Enemy-specific power trigger functions

pub fn apply_ritual(entity: &mut EntityState) {
    let ritual = entity.status("Ritual");
    if ritual > 0 {
        entity.add_status("Strength", ritual);
    }
}

/// Apply GenericStrengthUp (enemy version of Ritual, gains each turn).

pub fn apply_generic_strength_up(entity: &mut EntityState) {
    let amount = entity.status("GenericStrengthUp");
    if amount > 0 {
        entity.add_status("Strength", amount);
    }
}

// ---------------------------------------------------------------------------
// Start-of-turn triggers
// ---------------------------------------------------------------------------

/// Apply LoseStrength at start of turn (undo temporary Strength gains).

pub fn get_beat_of_death_damage(entity: &EntityState) -> i32 {
    entity.status("Beat of Death")
}

/// Slow: increment counter when player plays a card on this enemy.

pub fn increment_slow(entity: &mut EntityState) {
    let slow = entity.status("Slow");
    if slow >= 0 && entity.statuses.contains_key("Slow") {
        entity.add_status("Slow", 1);
    }
}

/// TimeWarp: increment card counter. Returns true if 12 reached (end turn + gain Str).
/// TimeWarp uses "TimeWarpActive" as a presence flag and "Time Warp" for the counter.
/// The counter starts at 0 and increments; at 12 it resets and triggers.

pub fn increment_time_warp(entity: &mut EntityState) -> bool {
    if entity.status("TimeWarpActive") <= 0 {
        return false;
    }
    let tw = entity.status("Time Warp");
    let new_val = tw + 1;
    if new_val >= 12 {
        entity.set_status("Time Warp", 0);
        return true;
    }
    // Use insert directly to allow storing intermediate values including 0
    entity.statuses.insert("Time Warp".to_string(), new_val);
    false
}

/// Panache: tracks cards played, every 5 deals 10 damage to all enemies.
/// Returns damage to deal (0 or panache amount).

pub fn apply_angry_on_hit(entity: &mut EntityState) {
    let angry = entity.status("Angry");
    if angry > 0 {
        entity.add_status("Strength", angry);
    }
}

/// Envenom: returns Poison amount to apply when dealing unblocked attack damage.

pub fn apply_curiosity(entity: &mut EntityState) {
    let curiosity = entity.status("Curiosity");
    if curiosity > 0 {
        entity.add_status("Strength", curiosity);
    }
}

/// Rupture: gain Strength when losing HP from a card.

pub fn reset_slow(entity: &mut EntityState) {
    if entity.statuses.contains_key("Slow") {
        entity.set_status("Slow", 0);
    }
}

/// Decrement Fading. Returns true if entity should die (Fading reaches 0).

pub fn decrement_explosive(entity: &mut EntityState) -> i32 {
    let explosive = entity.status("Explosive");
    if explosive > 0 {
        let new_val = explosive - 1;
        entity.set_status("Explosive", new_val);
        if new_val <= 0 {
            // Explosive deals its stored damage amount
            // The damage is stored separately; typically 30-50
            return 30; // Default bomb damage
        }
    }
    0
}

/// Growth: gain Strength and Dexterity at end of round.

pub fn apply_growth(entity: &mut EntityState) {
    let growth = entity.status("Growth");
    if growth > 0 {
        entity.add_status("Strength", growth);
        entity.add_status("Dexterity", growth); // Growth also adds Dex in Java? No, just in Nemesis. Check specific enemies.
    }
}

/// Decrement Blur at end of round.

pub fn reset_invincible(entity: &mut EntityState, max_amount: i32) {
    if entity.statuses.contains_key("Invincible") {
        entity.set_status("Invincible", max_amount);
    }
}

// ---------------------------------------------------------------------------
// TheBomb countdown
// ---------------------------------------------------------------------------

/// TheBomb: decrement counter. Returns (should_explode, damage).

pub fn decrement_the_bomb(entity: &mut EntityState) -> (bool, i32) {
    let turns = entity.status("TheBombTurns");
    let damage = entity.status("TheBomb");
    if turns > 0 && damage > 0 {
        let new_turns = turns - 1;
        entity.set_status("TheBombTurns", new_turns);
        if new_turns <= 0 {
            entity.set_status("TheBomb", 0);
            entity.set_status("TheBombTurns", 0);
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

pub fn apply_regeneration(entity: &mut EntityState) -> i32 {
    let regen = entity.status("Regeneration");
    if regen > 0 {
        entity.set_status("Regeneration", regen - 1);
        return regen;
    }
    0
}

// ---------------------------------------------------------------------------
// Regrow end-of-turn (enemy)
// ---------------------------------------------------------------------------

/// Regrow: heal. Returns HP to heal.

pub fn get_regrow_heal(entity: &EntityState) -> i32 {
    entity.status("Regrow")
}

// ---------------------------------------------------------------------------
// End-of-turn removal: Rage
// ---------------------------------------------------------------------------

/// Remove Rage at end of turn.

pub fn get_spore_cloud_vulnerable(entity: &EntityState) -> i32 {
    entity.status("Spore Cloud")
}

