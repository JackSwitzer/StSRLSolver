#[cfg(test)]
mod power_tests {
    use crate::powers::*;
    use crate::state::EntityState;

    fn entity() -> EntityState { EntityState::new(50, 50) }

    #[test] fn decrement_weak_2_to_1() {
        let mut e = entity();
        e.set_status("Weakened", 2);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 1);
    }
    #[test] fn decrement_weak_1_to_0() {
        let mut e = entity();
        e.set_status("Weakened", 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 0);
        assert!(!e.statuses.contains_key("Weakened"));
    }
    #[test] fn decrement_all_three() {
        let mut e = entity();
        e.set_status("Weakened", 3);
        e.set_status("Vulnerable", 2);
        e.set_status("Frail", 1);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 2);
        assert_eq!(e.status("Vulnerable"), 1);
        assert_eq!(e.status("Frail"), 0);
    }
    #[test] fn poison_tick_damage() {
        let mut e = entity();
        e.set_status("Poison", 7);
        let d = tick_poison(&mut e);
        assert_eq!(d, 7);
        assert_eq!(e.hp, 43);
        assert_eq!(e.status("Poison"), 6);
    }
    #[test] fn poison_tick_to_zero() {
        let mut e = entity();
        e.set_status("Poison", 1);
        tick_poison(&mut e);
        assert_eq!(e.status("Poison"), 0);
    }
    #[test] fn poison_no_poison() {
        let mut e = entity();
        assert_eq!(tick_poison(&mut e), 0);
    }
    #[test] fn metallicize_gain() {
        let mut e = entity();
        e.set_status("Metallicize", 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 4);
    }
    #[test] fn metallicize_stacks() {
        let mut e = entity();
        e.block = 3;
        e.set_status("Metallicize", 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 7);
    }
    #[test] fn plated_armor_gain() {
        let mut e = entity();
        e.set_status("PlatedArmor", 6);
        apply_plated_armor(&mut e);
        assert_eq!(e.block, 6);
    }
    #[test] fn ritual_gain() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 3);
    }
    #[test] fn ritual_stacks() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        apply_ritual(&mut e);
        apply_ritual(&mut e);
        assert_eq!(e.strength(), 6);
    }
    #[test] fn artifact_blocks_debuff() {
        let mut e = entity();
        e.set_status("Artifact", 2);
        assert!(!apply_debuff(&mut e, "Weakened", 3));
        assert_eq!(e.status("Weakened"), 0);
        assert_eq!(e.status("Artifact"), 1);
    }
    #[test] fn artifact_consumed() {
        let mut e = entity();
        e.set_status("Artifact", 1);
        apply_debuff(&mut e, "Weakened", 1);
        assert_eq!(e.status("Artifact"), 0);
    }
    #[test] fn no_artifact_applies() {
        let mut e = entity();
        assert!(apply_debuff(&mut e, "Weakened", 2));
        assert_eq!(e.status("Weakened"), 2);
    }
}

// =============================================================================
// State tests
// =============================================================================

