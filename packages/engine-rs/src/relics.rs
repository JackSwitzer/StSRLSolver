//! Combat-relevant relic effects for MCTS simulations.
//!
//! Only implements relics that directly affect combat outcomes.
//! Non-combat relics (gold, map, etc.) are handled by the Python engine.

use crate::state::CombatState;

/// Apply relic effects at combat start.
/// Called once when combat begins, after initial draw.
pub fn apply_combat_start_relics(state: &mut CombatState) {
    for relic_id in state.relics.clone() {
        match relic_id.as_str() {
            "Vajra" => {
                // +1 Strength at combat start
                state.player.add_status("Strength", 1);
            }
            "Bag of Marbles" => {
                // Apply 1 Vulnerable to ALL enemies
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status("Vulnerable", 1);
                    }
                }
            }
            "Thread and Needle" => {
                // 4 Plated Armor at combat start
                state.player.add_status("Plated Armor", 4);
            }
            "Lantern" => {
                // +1 energy on turn 1 (tracked via counter)
                state.player.set_status("LanternReady", 1);
            }
            "Anchor" => {
                // 10 Block at combat start
                state.player.block += 10;
            }
            "Pen Nib" => {
                // Track attack counter for double damage every 10th
                // Counter starts at 0, triggers when it reaches 9 (10th attack)
                if state.player.status("PenNibCounter") == 0 {
                    state.player.set_status("PenNibCounter", 0);
                }
            }
            "Ornamental Fan" => {
                // Track cards played for block every 3
                state.player.set_status("OrnamentalFanCounter", 0);
            }
            "Violet Lotus" => {
                // +1 energy on Calm exit (Watcher-specific)
                // Handled in change_stance
                state.player.set_status("VioletLotus", 1);
            }
            "Akabeko" => {
                // 8 Vigor at combat start
                state.player.add_status("Vigor", 8);
            }
            "Bronze Scales" => {
                // 3 Thorns at combat start
                state.player.add_status("Thorns", 3);
            }
            "Clockwork Souvenir" | "ClockworkSouvenir" => {
                // 1 Artifact at combat start
                state.player.add_status("Artifact", 1);
            }
            "Fossilized Helix" | "FossilizedHelix" => {
                // 1 Buffer at combat start
                state.player.add_status("Buffer", 1);
            }
            "Data Disk" => {
                // 1 Focus at combat start
                state.player.add_status("Focus", 1);
            }
            "Mark of Pain" => {
                // 2 Wounds in draw pile
                state.draw_pile.push("Wound".to_string());
                state.draw_pile.push("Wound".to_string());
            }
            "Blood Vial" => {
                // Heal 2 HP at combat start
                state.player.hp = (state.player.hp + 2).min(state.player.max_hp);
            }
            "PureWater" => {
                // Add a Miracle card to hand at combat start
                state.hand.push("Miracle".to_string());
            }
            _ => {} // Unknown relic, ignore
        }
    }
}

/// Apply Lantern effect: +1 energy on first turn only.
/// Call this at start of player turn.
pub fn apply_lantern_turn_start(state: &mut CombatState) {
    if state.turn == 1 && state.player.status("LanternReady") > 0 {
        state.energy += 1;
        state.player.set_status("LanternReady", 0);
    }
}

/// Apply Ornamental Fan: gain 4 block after playing 3 cards.
/// Call this after each card play.
pub fn check_ornamental_fan(state: &mut CombatState) {
    if !state.has_relic("Ornamental Fan") {
        return;
    }

    let counter = state.player.status("OrnamentalFanCounter") + 1;
    if counter >= 3 {
        state.player.block += 4;
        state.player.set_status("OrnamentalFanCounter", 0);
    } else {
        state.player.set_status("OrnamentalFanCounter", counter);
    }
}

/// Check Pen Nib: every 10th attack deals double damage.
/// Call this before calculating attack damage.
/// Returns true if this attack triggers Pen Nib.
pub fn check_pen_nib(state: &mut CombatState) -> bool {
    if !state.has_relic("Pen Nib") {
        return false;
    }

    let counter = state.player.status("PenNibCounter");
    if counter >= 9 {
        // 10th attack: trigger double damage
        state.player.set_status("PenNibCounter", 0);
        true
    } else {
        state.player.set_status("PenNibCounter", counter + 1);
        false
    }
}

/// Apply Violet Lotus: +1 energy on Calm exit (beyond the normal +2).
/// Returns extra energy to add.
pub fn violet_lotus_calm_exit_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Violet Lotus") {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CombatState, EnemyCombatState};

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        CombatState::new(80, 80, vec![enemy], vec!["Strike_P".to_string(); 5], 3)
    }

    #[test]
    fn test_vajra_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_bag_of_marbles_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Bag of Marbles".to_string());

        apply_combat_start_relics(&mut state);
        assert!(state.enemies[0].entity.is_vulnerable());
    }

    #[test]
    fn test_thread_and_needle_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Thread and Needle".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("Plated Armor"), 4);
    }

    #[test]
    fn test_lantern_turn1_energy() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());
        state.turn = 0;

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("LanternReady"), 1);

        state.turn = 1;
        apply_lantern_turn_start(&mut state);
        assert_eq!(state.energy, 4); // 3 base + 1 Lantern
        assert_eq!(state.player.status("LanternReady"), 0);
    }

    #[test]
    fn test_lantern_not_turn2() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());

        apply_combat_start_relics(&mut state);

        state.turn = 2;
        apply_lantern_turn_start(&mut state);
        assert_eq!(state.energy, 3); // No bonus on turn 2
    }

    #[test]
    fn test_ornamental_fan_every_3_cards() {
        let mut state = make_test_state();
        state.relics.push("Ornamental Fan".to_string());

        apply_combat_start_relics(&mut state);

        // Play 3 cards
        check_ornamental_fan(&mut state);
        assert_eq!(state.player.block, 0);
        check_ornamental_fan(&mut state);
        assert_eq!(state.player.block, 0);
        check_ornamental_fan(&mut state);
        assert_eq!(state.player.block, 4); // Triggered!

        // Next 3 cards
        check_ornamental_fan(&mut state);
        check_ornamental_fan(&mut state);
        check_ornamental_fan(&mut state);
        assert_eq!(state.player.block, 8); // Triggered again!
    }

    #[test]
    fn test_pen_nib_every_10_attacks() {
        let mut state = make_test_state();
        state.relics.push("Pen Nib".to_string());

        apply_combat_start_relics(&mut state);

        // Attacks 1-9: no trigger
        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }

        // Attack 10: trigger!
        assert!(check_pen_nib(&mut state));

        // Resets: attacks 1-9 again
        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }
        assert!(check_pen_nib(&mut state));
    }

    #[test]
    fn test_violet_lotus_bonus() {
        let mut state = make_test_state();
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 0);

        state.relics.push("Violet Lotus".to_string());
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 1);
    }

    #[test]
    fn test_anchor_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Anchor".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.block, 10);
    }

    #[test]
    fn test_akabeko_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Akabeko".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("Vigor"), 8);
    }

    #[test]
    fn test_blood_vial_combat_start() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Blood Vial".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn test_blood_vial_does_not_exceed_max() {
        let mut state = make_test_state();
        state.player.hp = 79;
        state.relics.push("Blood Vial".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 80); // Capped at max
    }

    #[test]
    fn test_multiple_relics() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());
        state.relics.push("Bag of Marbles".to_string());
        state.relics.push("Anchor".to_string());

        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
        assert!(state.enemies[0].entity.is_vulnerable());
        assert_eq!(state.player.block, 10);
    }

    #[test]
    fn test_pure_water_adds_miracle() {
        let mut state = make_test_state();
        state.relics.push("PureWater".to_string());

        let hand_before = state.hand.len();
        apply_combat_start_relics(&mut state);

        assert_eq!(state.hand.len(), hand_before + 1);
        assert_eq!(state.hand.last().unwrap(), "Miracle");
    }

    #[test]
    fn test_mark_of_pain() {
        let mut state = make_test_state();
        state.relics.push("Mark of Pain".to_string());

        let initial_draw_size = state.draw_pile.len();
        apply_combat_start_relics(&mut state);
        assert_eq!(state.draw_pile.len(), initial_draw_size + 2);

        let wound_count = state.draw_pile.iter().filter(|c| *c == "Wound").count();
        assert_eq!(wound_count, 2);
    }
}
