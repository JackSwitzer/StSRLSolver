use super::debuffs::decrement_status;
#[cfg(test)]
use super::debuffs::{decrement_blur, decrement_debuffs, decrement_lock_on};
#[cfg(test)]
use super::enemy_powers::reset_slow;
use crate::state::EntityState;
use crate::status_ids::sid;

/// NextTurnBlock: return the block to gain, then remove the power.
pub fn consume_next_turn_block(entity: &mut EntityState) -> i32 {
    let amount = entity.status(sid::NEXT_TURN_BLOCK);
    if amount > 0 {
        entity.set_status(sid::NEXT_TURN_BLOCK, 0);
    }
    amount
}

/// EquilibriumPower loses one stack at end of round.
pub fn decrement_equilibrium(entity: &mut EntityState) {
    decrement_status(entity, sid::EQUILIBRIUM);
}

/// Test support for source-derived end-of-round power assertions.
#[cfg(test)]
pub fn process_end_of_round(entity: &mut EntityState) {
    decrement_equilibrium(entity);
    decrement_debuffs(entity);
    decrement_blur(entity);
    decrement_lock_on(entity);
    reset_slow(entity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::powers::{apply_debuff, increment_time_warp, tick_poison};

    #[test]
    fn test_decrement_debuffs() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::WEAKENED, 2);
        entity.set_status(sid::VULNERABLE, 1);
        entity.set_status(sid::FRAIL, 3);

        decrement_debuffs(&mut entity);

        assert_eq!(entity.status(sid::WEAKENED), 1);
        assert_eq!(entity.status(sid::VULNERABLE), 0);
        assert_eq!(entity.status(sid::FRAIL), 2);
    }

    #[test]
    fn test_tick_poison() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::POISON, 5);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 5);
        assert_eq!(entity.hp, 45);
        assert_eq!(entity.status(sid::POISON), 4);
    }

    #[test]
    fn test_tick_poison_removed_at_zero() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::POISON, 1);

        let dmg = tick_poison(&mut entity);
        assert_eq!(dmg, 1);
        assert_eq!(entity.status(sid::POISON), 0);
    }

    #[test]
    fn test_artifact_blocks_debuff() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::ARTIFACT, 1);

        let applied = apply_debuff(&mut entity, sid::WEAKENED, 2);
        assert!(!applied);
        assert_eq!(entity.status(sid::WEAKENED), 0);
        assert_eq!(entity.status(sid::ARTIFACT), 0);
    }

    #[test]
    fn test_debuff_without_artifact() {
        let mut entity = EntityState::new(50, 50);

        let applied = apply_debuff(&mut entity, sid::WEAKENED, 2);
        assert!(applied);
        assert_eq!(entity.status(sid::WEAKENED), 2);
    }

    #[test]
    fn test_time_warp_countdown() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::TIME_WARP_ACTIVE, 1);

        for _ in 0..11 {
            assert!(!increment_time_warp(&mut entity));
        }
        assert!(increment_time_warp(&mut entity));
        assert_eq!(entity.status(sid::TIME_WARP), 0);
    }

    #[test]
    fn test_process_end_of_round() {
        let mut entity = EntityState::new(50, 50);
        entity.set_status(sid::WEAKENED, 2);
        entity.set_status(sid::VULNERABLE, 1);
        entity.set_status(sid::BLUR, 1);
        entity.set_status(sid::SLOW, 5);
        entity.set_status(sid::LOCK_ON, 2);

        process_end_of_round(&mut entity);

        assert_eq!(entity.status(sid::WEAKENED), 1);
        assert_eq!(entity.status(sid::VULNERABLE), 0);
        assert_eq!(entity.status(sid::BLUR), 0);
        assert_eq!(entity.status(sid::SLOW), 1);
        assert_eq!(entity.status(sid::LOCK_ON), 1);
    }
}
