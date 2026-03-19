//! Potion effects for MCTS combat simulations.
//!
//! Implements the most commonly used potions. Complex discovery-based potions
//! (Attack/Skill/Power Potion) are handled by the Python engine.

use crate::state::CombatState;

/// Result of using a potion, for the engine to process.
pub struct PotionResult {
    /// Whether the potion was successfully used
    pub success: bool,
    /// Whether this potion targets an enemy (needs target_idx)
    pub requires_target: bool,
}

/// Check if a potion requires a target enemy.
pub fn potion_requires_target(potion_id: &str) -> bool {
    matches!(
        potion_id,
        "Fire Potion" | "Weak Potion" | "FearPotion" | "Poison Potion"
    )
}

/// Apply a potion's effect to the combat state.
/// Returns true if the potion was successfully consumed.
pub fn apply_potion(state: &mut CombatState, potion_id: &str, target_idx: i32) -> bool {
    match potion_id {
        "Fire Potion" => {
            // Deal 20 damage to target enemy
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    let dmg = 20;
                    let blocked = enemy.entity.block.min(dmg);
                    let hp_damage = dmg - blocked;
                    enemy.entity.block -= blocked;
                    enemy.entity.hp -= hp_damage;
                    state.total_damage_dealt += hp_damage;
                    if enemy.entity.hp <= 0 {
                        enemy.entity.hp = 0;
                    }
                }
                true
            } else {
                false
            }
        }
        "Block Potion" => {
            // Gain 12 block
            state.player.block += 12;
            true
        }
        "Strength Potion" => {
            // Gain 2 Strength
            state.player.add_status("Strength", 2);
            true
        }
        "Dexterity Potion" => {
            // Gain 2 Dexterity
            state.player.add_status("Dexterity", 2);
            true
        }
        "Energy Potion" => {
            // Gain 2 Energy this turn
            state.energy += 2;
            true
        }
        "FairyPotion" | "Fairy in a Bottle" => {
            // Fairy in a Bottle: auto-triggers on death, not manual use.
            // Mark it for auto-revive. The engine checks on death.
            // Manual use does nothing.
            false
        }
        "Weak Potion" => {
            // Apply 3 Weak to target
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Weakened", 3);
                }
                true
            } else {
                false
            }
        }
        "FearPotion" => {
            // Apply 3 Vulnerable to target
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Vulnerable", 3);
                }
                true
            } else {
                false
            }
        }
        "Poison Potion" => {
            // Apply 6 Poison to target
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Poison", 6);
                }
                true
            } else {
                false
            }
        }
        "Explosive Potion" => {
            // Deal 10 damage to ALL enemies
            let living = state.living_enemy_indices();
            for idx in living {
                let enemy = &mut state.enemies[idx];
                let dmg = 10;
                let blocked = enemy.entity.block.min(dmg);
                let hp_damage = dmg - blocked;
                enemy.entity.block -= blocked;
                enemy.entity.hp -= hp_damage;
                state.total_damage_dealt += hp_damage;
                if enemy.entity.hp <= 0 {
                    enemy.entity.hp = 0;
                }
            }
            true
        }
        "SteroidPotion" | "Flex Potion" => {
            // Gain 5 temporary Strength
            state.player.add_status("Strength", 5);
            state.player.add_status("LoseStrength", 5);
            true
        }
        "SpeedPotion" => {
            // Gain 5 temporary Dexterity
            state.player.add_status("Dexterity", 5);
            state.player.add_status("LoseDexterity", 5);
            true
        }
        "Ancient Potion" => {
            // Gain 1 Artifact
            state.player.add_status("Artifact", 1);
            true
        }
        "Regen Potion" => {
            // Gain 5 Regeneration
            state.player.add_status("Regeneration", 5);
            true
        }
        "EssenceOfSteel" => {
            // Gain 4 Plated Armor
            state.player.add_status("Plated Armor", 4);
            true
        }
        "LiquidBronze" => {
            // Gain 3 Thorns
            state.player.add_status("Thorns", 3);
            true
        }
        "CultistPotion" => {
            // Gain 1 Ritual
            state.player.add_status("Ritual", 1);
            true
        }
        "BottledMiracle" => {
            // Add 2 Miracles to hand
            for _ in 0..2 {
                if state.hand.len() < 10 {
                    state.hand.push("Miracle".to_string());
                }
            }
            true
        }
        _ => {
            // Unknown potion: no effect, but consume it
            true
        }
    }
}

/// Check if player should auto-revive (Fairy in a Bottle).
/// Returns the HP to revive to (30% of max_hp), or 0 if no fairy.
pub fn check_fairy_revive(state: &CombatState) -> i32 {
    for potion in &state.potions {
        if potion == "FairyPotion" || potion == "Fairy in a Bottle" {
            return (state.player.max_hp * 30) / 100;
        }
    }
    0
}

/// Consume the Fairy in a Bottle potion slot after reviving.
pub fn consume_fairy(state: &mut CombatState) {
    for slot in &mut state.potions {
        if slot == "FairyPotion" || slot == "Fairy in a Bottle" {
            *slot = String::new();
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CombatState, EnemyCombatState};

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        let mut state = CombatState::new(80, 80, vec![enemy], vec!["Strike_P".to_string(); 5], 3);
        state.potions = vec!["".to_string(); 3];
        state
    }

    #[test]
    fn test_fire_potion_damage() {
        let mut state = make_test_state();
        let initial_hp = state.enemies[0].entity.hp;

        let success = apply_potion(&mut state, "Fire Potion", 0);
        assert!(success);
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 20);
        assert_eq!(state.total_damage_dealt, 20);
    }

    #[test]
    fn test_fire_potion_through_block() {
        let mut state = make_test_state();
        state.enemies[0].entity.block = 8;
        let initial_hp = state.enemies[0].entity.hp;

        apply_potion(&mut state, "Fire Potion", 0);
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 12); // 20 - 8 block
        assert_eq!(state.enemies[0].entity.block, 0);
    }

    #[test]
    fn test_fire_potion_invalid_target() {
        let mut state = make_test_state();
        let success = apply_potion(&mut state, "Fire Potion", 5);
        assert!(!success);
    }

    #[test]
    fn test_block_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Block Potion", -1);
        assert_eq!(state.player.block, 12);
    }

    #[test]
    fn test_strength_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Strength Potion", -1);
        assert_eq!(state.player.strength(), 2);
    }

    #[test]
    fn test_dexterity_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Dexterity Potion", -1);
        assert_eq!(state.player.dexterity(), 2);
    }

    #[test]
    fn test_energy_potion() {
        let mut state = make_test_state();
        let initial_energy = state.energy;
        apply_potion(&mut state, "Energy Potion", -1);
        assert_eq!(state.energy, initial_energy + 2);
    }

    #[test]
    fn test_weak_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Weak Potion", 0);
        assert_eq!(state.enemies[0].entity.status("Weakened"), 3);
    }

    #[test]
    fn test_fear_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "FearPotion", 0);
        assert_eq!(state.enemies[0].entity.status("Vulnerable"), 3);
    }

    #[test]
    fn test_poison_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Poison Potion", 0);
        assert_eq!(state.enemies[0].entity.status("Poison"), 6);
    }

    #[test]
    fn test_explosive_potion_all_enemies() {
        let mut state = make_test_state();
        // Add a second enemy
        state
            .enemies
            .push(EnemyCombatState::new("Cultist", 50, 50));

        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;

        apply_potion(&mut state, "Explosive Potion", -1);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 10);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 10);
    }

    #[test]
    fn test_flex_potion_temporary_strength() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SteroidPotion", -1);
        assert_eq!(state.player.strength(), 5);
        assert_eq!(state.player.status("LoseStrength"), 5);
    }

    #[test]
    fn test_ancient_potion_artifact() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Ancient Potion", -1);
        assert_eq!(state.player.status("Artifact"), 1);
    }

    #[test]
    fn test_fairy_revive_check() {
        let mut state = make_test_state();
        assert_eq!(check_fairy_revive(&state), 0);

        state.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive(&state), 24); // 30% of 80 = 24
    }

    #[test]
    fn test_fairy_consume() {
        let mut state = make_test_state();
        state.potions[1] = "FairyPotion".to_string();

        consume_fairy(&mut state);
        assert!(state.potions[1].is_empty());
    }

    #[test]
    fn test_potion_requires_target() {
        assert!(potion_requires_target("Fire Potion"));
        assert!(potion_requires_target("Weak Potion"));
        assert!(potion_requires_target("FearPotion"));
        assert!(potion_requires_target("Poison Potion"));
        assert!(!potion_requires_target("Block Potion"));
        assert!(!potion_requires_target("Strength Potion"));
        assert!(!potion_requires_target("Energy Potion"));
    }

    #[test]
    fn test_bottled_miracle() {
        let mut state = make_test_state();
        state.hand.clear();

        apply_potion(&mut state, "BottledMiracle", -1);
        assert_eq!(state.hand.len(), 2);
        assert_eq!(state.hand[0], "Miracle");
        assert_eq!(state.hand[1], "Miracle");
    }

    #[test]
    fn test_essence_of_steel() {
        let mut state = make_test_state();
        apply_potion(&mut state, "EssenceOfSteel", -1);
        assert_eq!(state.player.status("Plated Armor"), 4);
    }

    #[test]
    fn test_regen_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Regen Potion", -1);
        assert_eq!(state.player.status("Regeneration"), 5);
    }
}
