//! Relic support for the owner-aware combat runtime.
//!
//! `defs` contains the canonical runtime schema for relic behavior. The old
//! helper dispatchers in `run.rs` are kept only as internal parity oracles
//! while engine production paths go through `effects::runtime`.

mod combat;
mod run;
pub mod defs;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use run::on_hp_loss;
pub(crate) use run::champion_belt_on_vulnerable;
#[cfg(test)]
pub(crate) use run::charons_ashes_on_exhaust;
#[cfg(test)]
pub(crate) use run::dead_branch_on_exhaust;
#[cfg(test)]
pub(crate) use run::tough_bandages_on_discard;
#[cfg(test)]
pub(crate) use run::tingsha_on_discard;
#[cfg(test)]
pub(crate) use run::toy_ornithopter_on_potion;
#[cfg(test)]
pub(crate) use run::hand_drill_on_block_break;
pub(crate) use run::strike_dummy_bonus;
pub(crate) use run::wrist_blade_bonus;
pub(crate) use run::snecko_skull_bonus;
pub(crate) use run::chemical_x_bonus;
#[cfg(test)]
pub(crate) use run::gold_plated_cables_active;
pub(crate) use run::violet_lotus_calm_exit_bonus;
pub(crate) use run::unceasing_top_should_draw;
pub(crate) use run::has_runic_pyramid;
pub(crate) use run::calipers_block_retention;
pub(crate) use run::has_ice_cream;
#[cfg(test)]
pub(crate) use run::has_sacred_bark;
pub(crate) use run::necronomicon_should_trigger;
pub(crate) use run::necronomicon_mark_used;
pub(crate) use run::necronomicon_reset;

// ==========================================================================
// TESTS
// ==========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status_ids::sid;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::make_deck_n;

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3)
    }

    fn make_two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 44, 44);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        CombatState::new(80, 80, vec![e1, e2], make_deck_n("Strike_P", 5), 3)
    }

    // --- Card play / turn / end tests ---

    // --- On HP loss tests ---

    // --- On shuffle tests ---

    // --- On enemy death tests ---

    // --- Combat end tests ---

    #[test]
    fn test_charons_ashes() {
        let mut state = make_two_enemy_state();
        state.relics.push("Charon's Ashes".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn test_tough_bandages() {
        let mut state = make_test_state();
        state.relics.push("Tough Bandages".to_string());
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn test_violet_lotus_bonus() {
        let mut state = make_test_state();
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 0);
        state.relics.push("Violet Lotus".to_string());
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 1);
    }

    #[test]
    fn test_calipers_block_retention() {
        let mut state = make_test_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 20), 15);
        assert_eq!(calipers_block_retention(&state, 10), 10);
    }

    #[test]
    fn test_chemical_x_bonus() {
        let mut state = make_test_state();
        assert_eq!(chemical_x_bonus(&state), 0);
        state.relics.push("Chemical X".to_string());
        assert_eq!(chemical_x_bonus(&state), 2);
    }

    #[test]
    fn test_unceasing_top() {
        let mut state = make_test_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.clear();
        assert!(unceasing_top_should_draw(&state));
        { let reg = crate::cards::global_registry(); state.hand.push(reg.make_card("Strike")); };
        assert!(!unceasing_top_should_draw(&state));
    }

    #[test]
    fn test_necronomicon() {
        let mut state = make_test_state();
        state.relics.push("Necronomicon".to_string());
        assert!(necronomicon_should_trigger(&state, 2, true));
        assert!(!necronomicon_should_trigger(&state, 1, true));
        assert!(!necronomicon_should_trigger(&state, 2, false));
        necronomicon_mark_used(&mut state);
        assert!(!necronomicon_should_trigger(&state, 2, true));
    }

    #[test]
    fn test_toy_ornithopter() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Toy Ornithopter".to_string());
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn test_hand_drill() {
        let mut state = make_test_state();
        state.relics.push("HandDrill".to_string());
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status(sid::VULNERABLE), 2);
    }




    #[test]
    fn test_tingsha() {
        let mut state = make_test_state();
        state.relics.push("Tingsha".to_string());
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp - 3);
    }

    // --- Stone Calendar fires only once ---

}
