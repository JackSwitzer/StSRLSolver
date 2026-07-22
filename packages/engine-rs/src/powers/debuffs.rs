use crate::ids::StatusId;
use crate::state::EntityState;
use crate::status_ids::sid;

// Debuff-related power trigger functions

pub fn decrement_debuffs(entity: &mut EntityState) {
    // D59 parity fix: if the debuff was applied by an enemy THIS round, Java's
    // `justApplied=true` skips the first `atEndOfRound` decrement so the debuff
    // lasts its full duration. We mirror via parallel flag statuses that
    // `apply_debuff_from_enemy` sets.
    decrement_debuff_with_just_applied(entity, sid::WEAKENED, sid::WEAKENED_JUST_APPLIED);
    decrement_debuff_with_just_applied(entity, sid::VULNERABLE, sid::VULNERABLE_JUST_APPLIED);
    decrement_debuff_with_just_applied(entity, sid::FRAIL, sid::FRAIL_JUST_APPLIED);
    decrement_debuff_with_just_applied(
        entity,
        sid::DRAW_REDUCTION,
        sid::DRAW_REDUCTION_JUST_APPLIED,
    );
}

fn decrement_debuff_with_just_applied(
    entity: &mut EntityState,
    debuff: StatusId,
    just_applied_flag: StatusId,
) {
    if entity.status(just_applied_flag) > 0 {
        // First round after enemy application: clear the flag but skip decrement.
        entity.set_status(just_applied_flag, 0);
        return;
    }
    decrement_status(entity, debuff);
}

/// Apply a debuff sourced from an enemy (as opposed to the player's own cards).
///
/// Mirrors the Java debuff power constructors' `justApplied` lifecycle.
///
/// `ApplyPowerAction` constructs a candidate power, but when the target already
/// owns that power Java calls `stackPower` on the existing instance. The
/// candidate's `justApplied` value is therefore discarded. Re-applying an
/// existing debuff must preserve its old latch instead of starting a new
/// protected round. See `ApplyPowerAction.java` plus `WeakPower.java`,
/// `VulnerablePower.java`, `FrailPower.java`, and `DrawReductionPower.java`.
pub fn apply_debuff_from_enemy(entity: &mut EntityState, status: StatusId, amount: i32) -> bool {
    let power_already_present = entity.status(status) != 0;
    let applied = apply_debuff(entity, status, amount);
    if applied && !power_already_present {
        if let Some(flag) = just_applied_flag_for(status) {
            entity.set_status(flag, 1);
        }
    }
    applied
}

fn just_applied_flag_for(status: StatusId) -> Option<StatusId> {
    if status == sid::WEAKENED {
        Some(sid::WEAKENED_JUST_APPLIED)
    } else if status == sid::VULNERABLE {
        Some(sid::VULNERABLE_JUST_APPLIED)
    } else if status == sid::FRAIL {
        Some(sid::FRAIL_JUST_APPLIED)
    } else if status == sid::DRAW_REDUCTION {
        Some(sid::DRAW_REDUCTION_JUST_APPLIED)
    } else {
        None
    }
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

    // Source: reference/extracted/methods/monster/Nemesis.java (`damage`) and
    // decompiled PoisonLoseHpAction.java. Poison constructs HP_LOSS DamageInfo,
    // so an installed Intangible power caps the tick to one before HP changes.
    let damage = if entity.status(sid::INTANGIBLE) > 0 {
        1
    } else {
        poison
    };
    entity.hp -= damage;

    let new_poison = poison - 1;
    entity.set_status(sid::POISON, new_poison);

    damage
}

/// Decrement Blur at end of turn.
#[cfg(test)]
pub fn decrement_blur(entity: &mut EntityState) {
    decrement_status(entity, sid::BLUR);
}

/// Decrement Lock-On at end of round.
#[cfg(test)]
pub fn decrement_lock_on(entity: &mut EntityState) {
    decrement_status(entity, sid::LOCK_ON);
}

/// Reset Invincible at end of round (Champ).

pub fn apply_debuff(entity: &mut EntityState, status: StatusId, amount: i32) -> bool {
    // ApplyPowerAction checks Ginger and Turnip before Artifact. Their exact
    // immunities therefore block without consuming an Artifact stack.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
    if status == sid::WEAKENED && entity.status(sid::HAS_GINGER) > 0 {
        return false;
    }
    if status == sid::FRAIL && entity.status(sid::HAS_TURNIP) > 0 {
        return false;
    }

    let artifact = entity.status(sid::ARTIFACT);
    if artifact > 0 {
        // Artifact blocks the debuff and decrements
        entity.set_status(sid::ARTIFACT, artifact - 1);
        return false;
    }

    entity.add_status(status, amount);
    true
}

// ---------------------------------------------------------------------------
// Invincible damage cap
// ---------------------------------------------------------------------------

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
    // SlowPower is installed with amount zero. Because a zero status cannot
    // keep an EntityDef active, Rust stores that installed state as sentinel
    // one and the Java amount is `status - 1`.
    // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/SlowPower.java.
    let amount = (entity.status(sid::SLOW) - 1).max(0);
    1.0 + (amount as f64 * 0.10)
}
