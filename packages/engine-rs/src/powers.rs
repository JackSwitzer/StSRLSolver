//! Power/status effect system — stubs for the core turn loop.
//!
//! The full power registry (168+ triggers in Python) is NOT ported here.
//! This module handles only the status effects that matter for the fast path:
//! - Strength / Dexterity (damage/block modifiers)
//! - Weakened / Vulnerable / Frail (debuff multipliers)
//! - Metallicize / Plated Armor (passive block gain)
//! - Poison (damage over time)
//! - Ritual (enemy strength gain)
//!
//! All other powers are handled by the Python engine when it matters.

use crate::state::EntityState;

/// Decrement turn-based debuffs at end of round.
/// Matches the atEndOfRound power trigger in Python.
///
/// Debuffs that tick down: Weakened, Vulnerable, Frail.
pub fn decrement_debuffs(entity: &mut EntityState) {
    decrement_status(entity, "Weakened");
    decrement_status(entity, "Vulnerable");
    decrement_status(entity, "Frail");
}

/// Decrement a single status by 1. Remove if it reaches 0.
fn decrement_status(entity: &mut EntityState, key: &str) {
    if let Some(val) = entity.statuses.get(key).copied() {
        if val <= 1 {
            entity.statuses.remove(key);
        } else {
            entity.statuses.insert(key.to_string(), val - 1);
        }
    }
}

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

/// Apply Metallicize block gain at end of turn.
pub fn apply_metallicize(entity: &mut EntityState) {
    let metallicize = entity.status("Metallicize");
    if metallicize > 0 {
        entity.block += metallicize;
    }
}

/// Apply Plated Armor block gain at end of turn.
pub fn apply_plated_armor(entity: &mut EntityState) {
    let plated = entity.status("Plated Armor");
    if plated > 0 {
        entity.block += plated;
    }
}

/// Apply Ritual strength gain at start of enemy turn (not first turn).
pub fn apply_ritual(entity: &mut EntityState) {
    let ritual = entity.status("Ritual");
    if ritual > 0 {
        entity.add_status("Strength", ritual);
    }
}

/// Apply a debuff, respecting Artifact (blocks debuffs).
/// Returns true if the debuff was applied, false if blocked by Artifact.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrement_debuffs() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Weakened", 2);
        entity.set_status("Vulnerable", 1);
        entity.set_status("Frail", 3);

        decrement_debuffs(&mut entity);

        assert_eq!(entity.status("Weakened"), 1);
        assert_eq!(entity.status("Vulnerable"), 0);
        assert_eq!(entity.status("Frail"), 2);
    }

    #[test]
    fn test_tick_poison() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 5);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 5);
        assert_eq!(entity.hp, 45);
        assert_eq!(entity.status("Poison"), 4);
    }

    #[test]
    fn test_tick_poison_removed_at_zero() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Poison", 1);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 1);
        assert_eq!(entity.status("Poison"), 0);
        assert!(!entity.statuses.contains_key("Poison"));
    }

    #[test]
    fn test_metallicize() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Metallicize", 4);

        apply_metallicize(&mut entity);
        assert_eq!(entity.block, 4);
    }

    #[test]
    fn test_ritual() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Ritual", 3);

        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 3);

        // Second application stacks
        apply_ritual(&mut entity);
        assert_eq!(entity.strength(), 6);
    }

    #[test]
    fn test_artifact_blocks_debuff() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status("Artifact", 1);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(!applied);
        assert_eq!(entity.status("Weakened"), 0);
        assert_eq!(entity.status("Artifact"), 0);
    }

    #[test]
    fn test_debuff_without_artifact() {
        let mut entity = EntityState::new(50, 50);

        let applied = apply_debuff(&mut entity, "Weakened", 2);
        assert!(applied);
        assert_eq!(entity.status("Weakened"), 2);
    }
}
