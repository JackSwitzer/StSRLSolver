#[cfg(test)]
mod state_tests {
    use crate::state::*;
    use crate::status_ids::sid;

    #[test] fn stance_from_str() {
        assert_eq!(Stance::from_str("Wrath"), Stance::Wrath);
        assert_eq!(Stance::from_str("Calm"), Stance::Calm);
        assert_eq!(Stance::from_str("Divinity"), Stance::Divinity);
        assert_eq!(Stance::from_str("Neutral"), Stance::Neutral);
        assert_eq!(Stance::from_str("garbage"), Stance::Neutral);
    }
    #[test] fn stance_outgoing_mult() {
        assert_eq!(Stance::Wrath.outgoing_mult(), 2.0);
        assert_eq!(Stance::Divinity.outgoing_mult(), 3.0);
        assert_eq!(Stance::Calm.outgoing_mult(), 1.0);
        assert_eq!(Stance::Neutral.outgoing_mult(), 1.0);
    }
    #[test] fn stance_incoming_mult() {
        assert_eq!(Stance::Wrath.incoming_mult(), 2.0);
        assert_eq!(Stance::Divinity.incoming_mult(), 1.0);
        assert_eq!(Stance::Calm.incoming_mult(), 1.0);
    }
    #[test] fn entity_accessors() {
        let mut e = EntityState::new(50, 50);
        assert_eq!(e.strength(), 0);
        assert_eq!(e.dexterity(), 0);
        assert!(!e.is_weak());
        assert!(!e.is_vulnerable());
        assert!(!e.is_frail());
        assert!(!e.is_dead());
        e.set_status(sid::STRENGTH, 5);
        assert_eq!(e.strength(), 5);
    }
    #[test] fn entity_add_status() {
        let mut e = EntityState::new(50, 50);
        e.add_status(sid::STRENGTH, 3);
        e.add_status(sid::STRENGTH, 2);
        assert_eq!(e.strength(), 5);
    }
    #[test] fn entity_set_zero_removes() {
        let mut e = EntityState::new(50, 50);
        e.set_status(sid::STRENGTH, 5);
        e.set_status(sid::STRENGTH, 0);
        assert_eq!(e.status(sid::STRENGTH), 0);
    }
    #[test] fn entity_dead_at_zero() {
        let mut e = EntityState::new(50, 50);
        e.hp = 0;
        assert!(e.is_dead());
    }
    #[test] fn enemy_alive_check() {
        let e = EnemyCombatState::new("Test", 30, 30);
        assert!(e.is_alive());
    }
    #[test] fn enemy_dead_check() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.entity.hp = 0;
        assert!(!e.is_alive());
    }
    #[test] fn enemy_escaping_not_alive() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.is_escaping = true;
        assert!(!e.is_alive());
    }
    #[test] fn enemy_total_incoming() {
        let mut e = EnemyCombatState::new("Test", 30, 30);
        e.set_move(1, 5, 3, 0);
        assert_eq!(e.total_incoming_damage(), 15);
    }
    #[test] fn combat_state_victory() {
        let mut s = CombatState::new(80, 80, vec![EnemyCombatState::new("T", 0, 30)], vec![], 3);
        s.enemies[0].entity.hp = 0;
        assert!(s.is_victory());
    }
    #[test] fn combat_state_defeat() {
        let s = CombatState::new(0, 80, vec![EnemyCombatState::new("T", 30, 30)], vec![], 3);
        assert!(s.is_defeat());
    }
    #[test] fn combat_state_not_terminal() {
        let s = CombatState::new(80, 80, vec![EnemyCombatState::new("T", 30, 30)], vec![], 3);
        assert!(!s.is_terminal());
    }
    #[test] fn living_enemy_indices() {
        let mut s = CombatState::new(80, 80, vec![
            EnemyCombatState::new("A", 30, 30),
            EnemyCombatState::new("B", 0, 30),
            EnemyCombatState::new("C", 20, 20),
        ], vec![], 3);
        s.enemies[1].entity.hp = 0;
        assert_eq!(s.living_enemy_indices(), vec![0, 2]);
    }
    #[test] fn has_relic() {
        let mut s = CombatState::new(80, 80, vec![], vec![], 3);
        s.relics.push("Vajra".to_string());
        assert!(s.has_relic("Vajra"));
        assert!(!s.has_relic("Missing"));
    }
}

// =============================================================================
// Integration: engine-level combined tests
// =============================================================================

