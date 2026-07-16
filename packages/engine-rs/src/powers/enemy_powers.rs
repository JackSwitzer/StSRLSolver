use crate::state::EntityState;
use crate::status_ids::sid;

/// Apply GenericStrengthUp (enemy version of Ritual, gains each turn).
pub fn apply_generic_strength_up(entity: &mut EntityState) {
    let amount = entity.status(sid::GENERIC_STRENGTH_UP);
    if amount > 0 {
        entity.add_status(sid::STRENGTH, amount);
    }
}

/// TimeWarp: increment card counter. Returns true if 12 reached.
pub fn increment_time_warp(entity: &mut EntityState) -> bool {
    if entity.status(sid::TIME_WARP_ACTIVE) <= 0 {
        return false;
    }
    let new_val = entity.status(sid::TIME_WARP) + 1;
    if new_val >= 12 {
        entity.set_status(sid::TIME_WARP, 0);
        return true;
    }
    entity.set_status(sid::TIME_WARP, new_val);
    false
}

pub fn reset_slow(entity: &mut EntityState) {
    if entity.status(sid::SLOW) != 0 {
        // SlowPower.atEndOfRound resets amount to zero without removing the
        // power. Sentinel 1 represents that installed zero amount in Rust.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/powers/SlowPower.java.
        entity.set_status(sid::SLOW, 1);
    }
}
