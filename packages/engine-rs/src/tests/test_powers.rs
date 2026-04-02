#[cfg(test)]
mod power_tests {
    use crate::powers::*;
    use crate::status_ids::sid;
    use crate::state::EntityState;

    fn entity() -> EntityState { EntityState::new(50, 50) }

    #[test] fn decrement_weak_2_to_1() {
        let mut e = entity();
        e.set_status(sid::WEAKENED, 2);
        decrement_debuffs(&mut e);
        assert_eq!(e.status(sid::WEAKENED), 1);
    }
    #[test] fn decrement_weak_1_to_0() {
        let mut e = entity();
        e.set_status(sid::WEAKENED, 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status(sid::WEAKENED), 0);
        assert!(!e.statuses.contains_key(&sid::WEAKENED));
    }
    #[test] fn decrement_all_three() {
        let mut e = entity();
        e.set_status(sid::WEAKENED, 3);
        e.set_status(sid::VULNERABLE, 2);
        e.set_status(sid::FRAIL, 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status(sid::WEAKENED), 2);
        assert_eq!(e.status(sid::VULNERABLE), 1);
        assert_eq!(e.status(sid::FRAIL), 0);
    }
    #[test] fn poison_tick_damage() {
        let mut e = entity();
        e.set_status(sid::POISON, 7);
        let d = tick_poison(&mut e);
        assert_eq!(d, 7);
        assert_eq!(e.hp, 43);
        assert_eq!(e.status(sid::POISON), 6);
    }
    #[test] fn poison_tick_to_zero() {
        let mut e = entity();
        e.set_status(sid::POISON, 1);
        tick_poison(&mut e);
        assert_eq!(e.status(sid::POISON), 0);
    }
    #[test] fn poison_no_poison() {
        let mut e = entity();
        assert_eq!(tick_poison(&mut e), 0);
    }
    #[test] fn metallicize_gain() {
        let mut e = entity();
        e.set_status(sid::METALLICIZE, 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 4);
    }
    #[test] fn metallicize_stacks() {
        let mut e = entity();
        e.block = 3;
        e.set_status(sid::METALLICIZE, 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 7);
    }
    #[test] fn plated_armor_gain() {
        let mut e = entity();
        e.set_status(sid::PLATED_ARMOR, 6);
        apply_plated_armor(&mut e);
        assert_eq!(e.block, 6);
    }
    #[test] fn ritual_gain() {
        let mut e = entity();
        e.set_status(sid::RITUAL, 3);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 3);
    }
    #[test] fn ritual_stacks() {
        let mut e = entity();
        e.set_status(sid::RITUAL, 3);
        apply_ritual(&mut e);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 6);
    }
    #[test] fn artifact_blocks_debuff() {
        let mut e = entity();
        e.set_status(sid::ARTIFACT, 2);
        assert!(!apply_debuff(&mut e, sid::WEAKENED, 3));
        assert_eq!(e.status(sid::WEAKENED), 0);
        assert_eq!(e.status(sid::ARTIFACT), 1);
    }
    #[test] fn artifact_consumed() {
        let mut e = entity();
        e.set_status(sid::ARTIFACT, 1);
        apply_debuff(&mut e, sid::WEAKENED, 1);
        assert_eq!(e.status(sid::ARTIFACT), 0);
    }
    #[test] fn no_artifact_applies() {
        let mut e = entity();
        assert!(apply_debuff(&mut e, sid::WEAKENED, 2));
        assert_eq!(e.status(sid::WEAKENED), 2);
    }
}

// =============================================================================
// State tests
// =============================================================================

