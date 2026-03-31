//! Potion effects for MCTS combat simulations.
//!
//! Implements all 44 potions from Slay the Spire. Each potion has:
//! - A potency value (base, non-A11 values)
//! - Target type (self, single enemy, all enemies)
//! - Effect on use

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
        "Fire Potion" | "FirePotion" |
        "Weak Potion" | "WeakenPotion" |
        "FearPotion" | "Fear Potion" |
        "Poison Potion" | "PoisonPotion"
    )
}

/// Apply a potion's effect to the combat state.
/// Returns true if the potion was successfully consumed.
/// `sacred_bark` doubles potency if true.
pub fn apply_potion(state: &mut CombatState, potion_id: &str, target_idx: i32) -> bool {
    let bark = state.has_relic("SacredBark");
    let mult = if bark { 2 } else { 1 };

    match potion_id {
        // =====================================================================
        // DAMAGE POTIONS
        // =====================================================================

        "Fire Potion" | "FirePotion" => {
            // Deal 20 damage to target enemy
            let potency = 20 * mult;
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    deal_damage_to_enemy(state, target_idx as usize, potency);
                }
                true
            } else {
                false
            }
        }

        "Explosive Potion" | "ExplosivePotion" => {
            // Deal 10 damage to ALL enemies
            let potency = 10 * mult;
            let living = state.living_enemy_indices();
            for idx in living {
                deal_damage_to_enemy(state, idx, potency);
            }
            true
        }

        // =====================================================================
        // BLOCK POTIONS
        // =====================================================================

        "Block Potion" | "BlockPotion" => {
            // Gain 12 block
            state.player.block += 12 * mult;
            true
        }

        // =====================================================================
        // STAT POTIONS (permanent for combat)
        // =====================================================================

        "Strength Potion" | "StrengthPotion" => {
            // Gain 2 Strength
            state.player.add_status("Strength", 2 * mult);
            true
        }

        "Dexterity Potion" | "DexterityPotion" => {
            // Gain 2 Dexterity
            state.player.add_status("Dexterity", 2 * mult);
            true
        }

        "Focus Potion" | "FocusPotion" => {
            // Gain 2 Focus
            state.player.add_status("Focus", 2 * mult);
            true
        }

        // =====================================================================
        // TEMPORARY STAT POTIONS
        // =====================================================================

        "SteroidPotion" | "Flex Potion" => {
            // Gain 5 temporary Strength
            let potency = 5 * mult;
            state.player.add_status("Strength", potency);
            state.player.add_status("LoseStrength", potency);
            true
        }

        "SpeedPotion" => {
            // Gain 5 temporary Dexterity
            let potency = 5 * mult;
            state.player.add_status("Dexterity", potency);
            state.player.add_status("LoseDexterity", potency);
            true
        }

        // =====================================================================
        // DEBUFF POTIONS
        // =====================================================================

        "Weak Potion" | "WeakenPotion" => {
            // Apply 3 Weak to target
            let potency = 3 * mult;
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Weakened", potency);
                }
                true
            } else {
                false
            }
        }

        "FearPotion" | "Fear Potion" => {
            // Apply 3 Vulnerable to target
            let potency = 3 * mult;
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Vulnerable", potency);
                }
                true
            } else {
                false
            }
        }

        "Poison Potion" | "PoisonPotion" => {
            // Apply 6 Poison to target
            let potency = 6 * mult;
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status("Poison", potency);
                }
                true
            } else {
                false
            }
        }

        // =====================================================================
        // ENERGY / DRAW POTIONS
        // =====================================================================

        "Energy Potion" | "EnergyPotion" => {
            // Gain 2 Energy this turn
            state.energy += 2 * mult;
            true
        }

        "Swift Potion" | "SwiftPotion" => {
            // Draw 3 cards
            let potency = 3 * mult;
            state.player.set_status("PotionDraw", potency);
            true
        }

        "SneckoOil" => {
            // Draw 5 cards and randomize their costs
            let potency = 5 * mult;
            state.player.set_status("PotionDraw", potency);
            // Cost randomization handled by engine
            true
        }

        // =====================================================================
        // POWER/BUFF POTIONS
        // =====================================================================

        "Ancient Potion" | "AncientPotion" => {
            // Gain 1 Artifact
            state.player.add_status("Artifact", 1 * mult);
            true
        }

        "Regen Potion" | "RegenPotion" => {
            // Gain 5 Regeneration
            state.player.add_status("Regeneration", 5 * mult);
            true
        }

        "EssenceOfSteel" => {
            // Gain 4 Plated Armor
            state.player.add_status("Plated Armor", 4 * mult);
            true
        }

        "LiquidBronze" => {
            // Gain 3 Thorns
            state.player.add_status("Thorns", 3 * mult);
            true
        }

        "CultistPotion" => {
            // Gain 1 Ritual (Strength gain per turn)
            state.player.add_status("Ritual", 1 * mult);
            true
        }

        "HeartOfIron" => {
            // Gain 6 Metallicize
            state.player.add_status("Metallicize", 6 * mult);
            true
        }

        "GhostInAJar" => {
            // Gain 1 Intangible
            state.player.add_status("Intangible", 1 * mult);
            true
        }

        "DuplicationPotion" => {
            // Next card is played twice (1 Duplication)
            state.player.add_status("Duplication", 1 * mult);
            true
        }

        // =====================================================================
        // HEAL / HP POTIONS
        // =====================================================================

        "Blood Potion" | "BloodPotion" => {
            // Heal 20% of max HP
            let potency = 20 * mult;
            let heal = (state.player.max_hp * potency) / 100;
            state.player.hp = (state.player.hp + heal).min(state.player.max_hp);
            true
        }

        "Fruit Juice" | "FruitJuice" => {
            // Gain 5 max HP permanently
            let potency = 5 * mult;
            state.player.max_hp += potency;
            state.player.hp += potency;
            true
        }

        // =====================================================================
        // FAIRY IN A BOTTLE (auto-trigger on death)
        // =====================================================================

        "FairyPotion" | "Fairy in a Bottle" => {
            // Auto-triggers on death, not manual use.
            false
        }

        // =====================================================================
        // CARD GENERATION POTIONS
        // =====================================================================

        "BottledMiracle" => {
            // Add 2 Miracles to hand
            let potency = 2 * mult;
            for _ in 0..potency {
                if state.hand.len() < 10 {
                    state.hand.push("Miracle".to_string());
                }
            }
            true
        }

        "CunningPotion" => {
            // Add 3 Shivs to hand
            let potency = 3 * mult;
            for _ in 0..potency {
                if state.hand.len() < 10 {
                    state.hand.push("Shiv".to_string());
                }
            }
            true
        }

        // =====================================================================
        // DISCOVERY POTIONS (add card from choices)
        // =====================================================================

        "AttackPotion" => {
            // Discover an Attack card (simplified: no effect in MCTS)
            // In MCTS, we just skip
            true
        }

        "SkillPotion" => {
            // Discover a Skill card
            true
        }

        "PowerPotion" => {
            // Discover a Power card
            true
        }

        "ColorlessPotion" => {
            // Discover a Colorless card
            true
        }

        // =====================================================================
        // SPECIAL POTIONS
        // =====================================================================

        "Ambrosia" => {
            // Enter Divinity stance (Watcher)
            state.stance = crate::state::Stance::Divinity;
            true
        }

        "StancePotion" => {
            // Enter Calm or Wrath (choice; simplified to Wrath for MCTS)
            state.stance = crate::state::Stance::Wrath;
            true
        }

        "SmokeBomb" => {
            // Escape combat (non-boss). Combat ends with no reward.
            state.combat_over = true;
            state.player_won = false;
            true
        }

        "EntropicBrew" => {
            // Fill empty potion slots with random potions
            // In MCTS, just mark as used
            true
        }

        "GamblersBrew" => {
            // Discard hand and draw same number of cards
            let hand_size = state.hand.len() as i32;
            state.discard_pile.extend(state.hand.drain(..));
            state.player.set_status("PotionDraw", hand_size);
            true
        }

        "BlessingOfTheForge" => {
            // Upgrade all cards in hand (simplified: no upgrade tracking in MCTS)
            true
        }

        "Elixir" | "ElixirPotion" => {
            // Exhaust any number of cards from hand (complex; simplified)
            true
        }

        "LiquidMemories" => {
            // Put 1 card from discard into hand (complex; simplified)
            true
        }

        "DistilledChaosPotion" | "DistilledChaos" => {
            // Play 3 random cards from draw pile (complex; simplified)
            true
        }

        "EssenceOfDarkness" => {
            // Channel 1 Dark orb per orb slot (Defect; orbs Python-side)
            true
        }

        "PotionOfCapacity" => {
            // Gain 2 orb slots (Defect)
            state.player.add_status("OrbSlots", 2 * mult);
            true
        }

        _ => {
            // Unknown potion: no effect, but consume it
            true
        }
    }
}

/// Deal damage to a specific enemy (used by damage potions).
fn deal_damage_to_enemy(state: &mut CombatState, idx: usize, dmg: i32) {
    let enemy = &mut state.enemies[idx];
    if !enemy.is_alive() {
        return;
    }
    let blocked = enemy.entity.block.min(dmg);
    let hp_damage = dmg - blocked;
    enemy.entity.block -= blocked;
    enemy.entity.hp -= hp_damage;
    state.total_damage_dealt += hp_damage;
    if enemy.entity.hp <= 0 {
        enemy.entity.hp = 0;
    }
}

/// Check if player should auto-revive (Fairy in a Bottle).
/// Returns the HP to revive to (30% of max_hp), or 0 if no fairy.
pub fn check_fairy_revive(state: &CombatState) -> i32 {
    let bark = state.has_relic("SacredBark");
    let potency = if bark { 60 } else { 30 };
    for potion in &state.potions {
        if potion == "FairyPotion" || potion == "Fairy in a Bottle" {
            return (state.player.max_hp * potency) / 100;
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

// ==========================================================================
// TESTS
// ==========================================================================

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

    fn make_two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 44, 44);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        let mut state = CombatState::new(80, 80, vec![e1, e2], vec!["Strike_P".to_string(); 5], 3);
        state.potions = vec!["".to_string(); 3];
        state
    }

    // --- Damage potions ---

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
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 12);
        assert_eq!(state.enemies[0].entity.block, 0);
    }

    #[test]
    fn test_fire_potion_invalid_target() {
        let mut state = make_test_state();
        let success = apply_potion(&mut state, "Fire Potion", 5);
        assert!(!success);
    }

    #[test]
    fn test_explosive_potion_all_enemies() {
        let mut state = make_two_enemy_state();
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        apply_potion(&mut state, "Explosive Potion", -1);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 10);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 10);
    }

    // --- Block potions ---

    #[test]
    fn test_block_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Block Potion", -1);
        assert_eq!(state.player.block, 12);
    }

    // --- Stat potions ---

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
    fn test_focus_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Focus Potion", -1);
        assert_eq!(state.player.status("Focus"), 2);
    }

    // --- Temporary stat potions ---

    #[test]
    fn test_flex_potion_temporary_strength() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SteroidPotion", -1);
        assert_eq!(state.player.strength(), 5);
        assert_eq!(state.player.status("LoseStrength"), 5);
    }

    #[test]
    fn test_speed_potion_temporary_dexterity() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SpeedPotion", -1);
        assert_eq!(state.player.dexterity(), 5);
        assert_eq!(state.player.status("LoseDexterity"), 5);
    }

    // --- Debuff potions ---

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

    // --- Energy / draw potions ---

    #[test]
    fn test_energy_potion() {
        let mut state = make_test_state();
        let initial_energy = state.energy;
        apply_potion(&mut state, "Energy Potion", -1);
        assert_eq!(state.energy, initial_energy + 2);
    }

    #[test]
    fn test_swift_potion_draw() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Swift Potion", -1);
        assert_eq!(state.player.status("PotionDraw"), 3);
    }

    // --- Power/buff potions ---

    #[test]
    fn test_ancient_potion_artifact() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Ancient Potion", -1);
        assert_eq!(state.player.status("Artifact"), 1);
    }

    #[test]
    fn test_regen_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Regen Potion", -1);
        assert_eq!(state.player.status("Regeneration"), 5);
    }

    #[test]
    fn test_essence_of_steel() {
        let mut state = make_test_state();
        apply_potion(&mut state, "EssenceOfSteel", -1);
        assert_eq!(state.player.status("Plated Armor"), 4);
    }

    #[test]
    fn test_liquid_bronze() {
        let mut state = make_test_state();
        apply_potion(&mut state, "LiquidBronze", -1);
        assert_eq!(state.player.status("Thorns"), 3);
    }

    #[test]
    fn test_cultist_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "CultistPotion", -1);
        assert_eq!(state.player.status("Ritual"), 1);
    }

    #[test]
    fn test_heart_of_iron() {
        let mut state = make_test_state();
        apply_potion(&mut state, "HeartOfIron", -1);
        assert_eq!(state.player.status("Metallicize"), 6);
    }

    #[test]
    fn test_ghost_in_a_jar() {
        let mut state = make_test_state();
        apply_potion(&mut state, "GhostInAJar", -1);
        assert_eq!(state.player.status("Intangible"), 1);
    }

    #[test]
    fn test_duplication_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "DuplicationPotion", -1);
        assert_eq!(state.player.status("Duplication"), 1);
    }

    // --- Heal / HP potions ---

    #[test]
    fn test_blood_potion() {
        let mut state = make_test_state();
        state.player.hp = 60;
        apply_potion(&mut state, "Blood Potion", -1);
        // 20% of 80 = 16
        assert_eq!(state.player.hp, 76);
    }

    #[test]
    fn test_fruit_juice() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Fruit Juice", -1);
        assert_eq!(state.player.max_hp, 85);
        assert_eq!(state.player.hp, 85);
    }

    // --- Fairy in a Bottle ---

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
    fn test_fairy_manual_use_fails() {
        let mut state = make_test_state();
        let success = apply_potion(&mut state, "FairyPotion", -1);
        assert!(!success); // Can't use manually
    }

    // --- Card generation potions ---

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
    fn test_cunning_potion() {
        let mut state = make_test_state();
        state.hand.clear();
        apply_potion(&mut state, "CunningPotion", -1);
        assert_eq!(state.hand.len(), 3);
        assert!(state.hand.iter().all(|c| c == "Shiv"));
    }

    // --- Special potions ---

    #[test]
    fn test_ambrosia() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Ambrosia", -1);
        assert_eq!(state.stance, crate::state::Stance::Divinity);
    }

    #[test]
    fn test_smoke_bomb() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SmokeBomb", -1);
        assert!(state.combat_over);
        assert!(!state.player_won);
    }

    #[test]
    fn test_gamblers_brew() {
        let mut state = make_test_state();
        state.hand = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        apply_potion(&mut state, "GamblersBrew", -1);
        assert!(state.hand.is_empty());
        assert_eq!(state.discard_pile.len(), 3);
        assert_eq!(state.player.status("PotionDraw"), 3);
    }

    #[test]
    fn test_potion_of_capacity() {
        let mut state = make_test_state();
        apply_potion(&mut state, "PotionOfCapacity", -1);
        assert_eq!(state.player.status("OrbSlots"), 2);
    }

    // --- Sacred Bark doubles potions ---

    #[test]
    fn test_sacred_bark_doubles_fire() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        let hp = state.enemies[0].entity.hp;
        apply_potion(&mut state, "Fire Potion", 0);
        assert_eq!(state.enemies[0].entity.hp, hp - 40); // 20 * 2
    }

    #[test]
    fn test_sacred_bark_doubles_block() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        apply_potion(&mut state, "Block Potion", -1);
        assert_eq!(state.player.block, 24); // 12 * 2
    }

    #[test]
    fn test_sacred_bark_doubles_strength() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        apply_potion(&mut state, "Strength Potion", -1);
        assert_eq!(state.player.strength(), 4); // 2 * 2
    }

    #[test]
    fn test_sacred_bark_fairy_revive() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        state.potions[0] = "FairyPotion".to_string();
        let revive = check_fairy_revive(&state);
        assert_eq!(revive, 48); // 60% of 80 = 48
    }

    // --- Target requirements ---

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
}
