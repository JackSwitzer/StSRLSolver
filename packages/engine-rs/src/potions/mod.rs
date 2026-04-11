//! Potion effects for MCTS combat simulations.
//!
//! Implements all 44 potions from Slay the Spire. Each potion has:
//! - A potency value (base values, with A11 reduced versions)
//! - Target type (self, single enemy, all enemies)
//! - Effect on use
//!
//! Ascension 11+ reduces potion effectiveness. Call `apply_potion_scaled`
//! with the run's ascension level, or use `apply_potion` for base potency.

pub mod defs;

use crate::state::CombatState;
use crate::status_ids::sid;

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
        "Fire Potion"
            | "FirePotion"
            | "Weak Potion"
            | "WeakenPotion"
            | "FearPotion"
            | "Fear Potion"
            | "Poison Potion"
            | "PoisonPotion"
    )
}

/// Return (base_potency, a11_potency) for the named potion.
/// Ascension 11 reduces most potion values. Potions not in this table
/// are unaffected by ascension.
fn potion_potency(potion_id: &str) -> Option<(i32, i32)> {
    match potion_id {
        "Fire Potion" | "FirePotion" => Some((20, 15)),
        "Explosive Potion" | "ExplosivePotion" => Some((10, 7)),
        "Block Potion" | "BlockPotion" => Some((12, 9)),
        "Strength Potion" | "StrengthPotion" => Some((2, 1)),
        "Dexterity Potion" | "DexterityPotion" => Some((2, 1)),
        "Focus Potion" | "FocusPotion" => Some((2, 1)),
        "SteroidPotion" | "Flex Potion" => Some((5, 3)),
        "SpeedPotion" => Some((5, 3)),
        "Weak Potion" | "WeakenPotion" => Some((3, 2)),
        "FearPotion" | "Fear Potion" => Some((3, 2)),
        "Poison Potion" | "PoisonPotion" => Some((6, 4)),
        "Energy Potion" | "EnergyPotion" => Some((2, 1)),
        "Swift Potion" | "SwiftPotion" => Some((3, 2)),
        "SneckoOil" => Some((5, 4)),
        "Ancient Potion" | "AncientPotion" => Some((1, 1)),
        "Regen Potion" | "RegenPotion" => Some((5, 4)),
        "EssenceOfSteel" => Some((4, 3)),
        "LiquidBronze" => Some((3, 2)),
        "CultistPotion" => Some((1, 1)),
        "HeartOfIron" => Some((6, 4)),
        "GhostInAJar" => Some((1, 1)),
        "DuplicationPotion" => Some((1, 1)),
        "Blood Potion" | "BloodPotion" => Some((20, 15)),
        "Fruit Juice" | "FruitJuice" => Some((5, 3)),
        "BottledMiracle" => Some((2, 1)),
        "CunningPotion" => Some((3, 2)),
        "PotionOfCapacity" => Some((2, 1)),
        _ => None,
    }
}

/// Get the effective potency for a potion, accounting for ascension 11+
/// and Sacred Bark.
fn effective_potency(potion_id: &str, ascension: i32, bark_mult: i32) -> i32 {
    match potion_potency(potion_id) {
        Some((base, a11)) => {
            let raw = if ascension >= 11 { a11 } else { base };
            raw * bark_mult
        }
        None => bark_mult,
    }
}

/// Apply a potion with ascension scaling.
/// `ascension`: the run's ascension level (0-20). At A11+ potency is reduced.
/// Returns true if the potion was successfully consumed.
pub fn apply_potion_scaled(
    state: &mut CombatState,
    potion_id: &str,
    target_idx: i32,
    ascension: i32,
) -> bool {
    let bark = state.has_relic("SacredBark");
    let bark_mult = if bark { 2 } else { 1 };

    match potion_id {
        "Fire Potion" | "FirePotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
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
            let potency = effective_potency(potion_id, ascension, bark_mult);
            let living = state.living_enemy_indices();
            for idx in living {
                deal_damage_to_enemy(state, idx, potency);
            }
            true
        }

        "Block Potion" | "BlockPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.block += potency;
            true
        }

        "Strength Potion" | "StrengthPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::STRENGTH, potency);
            true
        }

        "Dexterity Potion" | "DexterityPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::DEXTERITY, potency);
            true
        }

        "Focus Potion" | "FocusPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::FOCUS, potency);
            true
        }

        "SteroidPotion" | "Flex Potion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::STRENGTH, potency);
            state.player.add_status(sid::LOSE_STRENGTH, potency);
            true
        }

        "SpeedPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::DEXTERITY, potency);
            state.player.add_status(sid::LOSE_DEXTERITY, potency);
            true
        }

        "Weak Potion" | "WeakenPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status(sid::WEAKENED, potency);
                }
                true
            } else {
                false
            }
        }

        "FearPotion" | "Fear Potion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status(sid::VULNERABLE, potency);
                }
                true
            } else {
                false
            }
        }

        "Poison Potion" | "PoisonPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            if target_idx >= 0 && (target_idx as usize) < state.enemies.len() {
                let enemy = &mut state.enemies[target_idx as usize];
                if enemy.is_alive() {
                    enemy.entity.add_status(sid::POISON, potency);
                }
                true
            } else {
                false
            }
        }

        "Energy Potion" | "EnergyPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.energy += potency;
            true
        }

        "Swift Potion" | "SwiftPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.set_status(sid::POTION_DRAW, potency);
            true
        }

        "SneckoOil" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.set_status(sid::POTION_DRAW, potency);
            state.player.set_status(sid::CONFUSION, 1);
            true
        }

        "Ancient Potion" | "AncientPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::ARTIFACT, potency);
            true
        }

        "Regen Potion" | "RegenPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::REGENERATION, potency);
            true
        }

        "EssenceOfSteel" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::PLATED_ARMOR, potency);
            true
        }

        "LiquidBronze" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::THORNS, potency);
            true
        }

        "CultistPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::RITUAL, potency);
            true
        }

        "HeartOfIron" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::METALLICIZE, potency);
            true
        }

        "GhostInAJar" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::INTANGIBLE, potency);
            true
        }

        "DuplicationPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::DUPLICATION, potency);
            true
        }

        "Blood Potion" | "BloodPotion" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            let heal = (state.player.max_hp * potency) / 100;
            state.heal_player(heal);
            true
        }

        "Fruit Juice" | "FruitJuice" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.max_hp += potency;
            state.player.hp += potency;
            true
        }

        "FairyPotion" | "Fairy in a Bottle" => false,

        "BottledMiracle" => {
            let registry = crate::cards::global_registry();
            let potency = effective_potency(potion_id, ascension, bark_mult);
            for _ in 0..potency {
                if state.hand.len() < 10 {
                    state.hand.push(registry.make_card("Miracle"));
                }
            }
            true
        }

        "CunningPotion" => {
            let registry = crate::cards::global_registry();
            let potency = effective_potency(potion_id, ascension, bark_mult);
            for _ in 0..potency {
                if state.hand.len() < 10 {
                    state.hand.push(registry.make_card("Shiv"));
                }
            }
            true
        }

        // Discovery potions: handled below with proxy cards for MCTS

        "Ambrosia" => {
            state.stance = crate::state::Stance::Divinity;
            true
        }

        "StancePotion" => {
            use crate::state::Stance;
            match state.stance {
                Stance::Calm => { state.stance = Stance::Wrath; }
                _ => { state.stance = Stance::Calm; }
            }
            true
        }

        "SmokeBomb" => {
            state.combat_over = true;
            state.player_won = false;
            true
        }

        "BlessingOfTheForge" => {
            // Upgrade ALL cards in hand
            let registry = crate::cards::global_registry();
            for card in &mut state.hand {
                registry.upgrade_card(card);
            }
            true
        }

        "Elixir" | "ElixirPotion" => {
            // Exhaust all cards in hand
            state.exhaust_pile.extend(state.hand.drain(..));
            true
        }

        "LiquidMemories" => {
            // Return card(s) from discard to hand
            let potency = effective_potency(potion_id, ascension, bark_mult);
            for _ in 0..potency {
                if !state.discard_pile.is_empty() && state.hand.len() < 10 {
                    if let Some(card) = state.discard_pile.pop() {
                        state.hand.push(card);
                    }
                }
            }
            true
        }

        "DistilledChaosPotion" | "DistilledChaos" => {
            // Play top N cards from draw pile (MCTS: move to hand)
            let potency = effective_potency(potion_id, ascension, bark_mult);
            for _ in 0..potency {
                if !state.draw_pile.is_empty() && state.hand.len() < 10 {
                    if let Some(card) = state.draw_pile.pop() {
                        state.hand.push(card);
                    }
                }
            }
            true
        }

        "EssenceOfDarkness" => {
            // Channel Dark orbs equal to orb slot count
            let slots = state.orb_slots.get_slot_count();
            for _ in 0..slots {
                let focus = state.player.focus();
                state.orb_slots.channel(crate::orbs::OrbType::Dark, focus);
            }
            true
        }

        "EntropicBrew" => {
            // Fill empty potion slots (MCTS: Block Potion as proxy)
            for slot in &mut state.potions {
                if slot.is_empty() {
                    *slot = "Block Potion".to_string();
                }
            }
            true
        }

        "AttackPotion" => {
            let registry = crate::cards::global_registry();
            if state.hand.len() < 10 { state.hand.push(registry.make_card("Strike_P")); }
            true
        }
        "SkillPotion" => {
            let registry = crate::cards::global_registry();
            if state.hand.len() < 10 { state.hand.push(registry.make_card("Defend_P")); }
            true
        }
        "PowerPotion" => {
            let registry = crate::cards::global_registry();
            if state.hand.len() < 10 { state.hand.push(registry.make_card("Smite")); }
            true
        }
        "ColorlessPotion" => {
            let registry = crate::cards::global_registry();
            if state.hand.len() < 10 { state.hand.push(registry.make_card("Strike_P")); }
            true
        }

        "GamblersBrew" => {
            let hand_size = state.hand.len() as i32;
            state.discard_pile.extend(state.hand.drain(..));
            state.player.set_status(sid::POTION_DRAW, hand_size);
            true
        }

        "PotionOfCapacity" => {
            let potency = effective_potency(potion_id, ascension, bark_mult);
            state.player.add_status(sid::ORB_SLOTS, potency);
            true
        }

        _ => true,
    }
}

/// Apply a potion's effect to the combat state (base potency, no ascension scaling).
/// Returns true if the potion was successfully consumed.
/// Backward-compatible wrapper: passes ascension=0 (no A11 reduction).
pub fn apply_potion(state: &mut CombatState, potion_id: &str, target_idx: i32) -> bool {
    apply_potion_scaled(state, potion_id, target_idx, 0)
}

/// Deal damage to a specific enemy (used by damage potions).
/// Respects Vulnerable, Intangible, and Invincible.
fn deal_damage_to_enemy(state: &mut CombatState, idx: usize, dmg: i32) {
    let enemy = &mut state.enemies[idx];
    if !enemy.is_alive() {
        return;
    }

    let mut final_dmg = dmg as f64;

    // Vulnerable: potion damage is boosted
    if enemy.entity.is_vulnerable() {
        final_dmg *= crate::damage::VULN_MULT;
    }

    let mut final_dmg_i = final_dmg as i32;

    // Intangible: cap at 1
    if enemy.entity.status(crate::status_ids::sid::INTANGIBLE) > 0 && final_dmg_i > 1 {
        final_dmg_i = 1;
    }

    // Invincible: per-turn cap (e.g. The Heart)
    final_dmg_i = crate::powers::apply_invincible_cap_tracked(&mut enemy.entity, final_dmg_i);

    let blocked = enemy.entity.block.min(final_dmg_i);
    let hp_damage = final_dmg_i - blocked;
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
    check_fairy_revive_scaled(state, 0)
}

/// Check fairy revive with ascension scaling.
/// A11+ reduces revive from 30% to 20% max HP.
pub fn check_fairy_revive_scaled(state: &CombatState, ascension: i32) -> i32 {
    let bark = state.has_relic("SacredBark");
    let base_pct = if ascension >= 11 { 20 } else { 30 };
    let potency = if bark { base_pct * 2 } else { base_pct };
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
    use crate::cards::CardRegistry;
    use crate::state::{CombatState, EnemyCombatState};
    use crate::tests::support::{make_deck, make_deck_n};

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        let mut state =
            CombatState::new(80, 80, vec![enemy], make_deck_n("Strike_P", 5), 3);
        state.potions = vec!["".to_string(); 3];
        state
    }

    fn make_two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 44, 44);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        let mut state =
            CombatState::new(80, 80, vec![e1, e2], make_deck_n("Strike_P", 5), 3);
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
    fn test_focus_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Focus Potion", -1);
        assert_eq!(state.player.status(sid::FOCUS), 2);
    }

    #[test]
    fn test_flex_potion_temporary_strength() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SteroidPotion", -1);
        assert_eq!(state.player.strength(), 5);
        assert_eq!(state.player.status(sid::LOSE_STRENGTH), 5);
    }

    #[test]
    fn test_speed_potion_temporary_dexterity() {
        let mut state = make_test_state();
        apply_potion(&mut state, "SpeedPotion", -1);
        assert_eq!(state.player.dexterity(), 5);
        assert_eq!(state.player.status(sid::LOSE_DEXTERITY), 5);
    }

    #[test]
    fn test_weak_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Weak Potion", 0);
        assert_eq!(state.enemies[0].entity.status(sid::WEAKENED), 3);
    }

    #[test]
    fn test_fear_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "FearPotion", 0);
        assert_eq!(state.enemies[0].entity.status(sid::VULNERABLE), 3);
    }

    #[test]
    fn test_poison_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Poison Potion", 0);
        assert_eq!(state.enemies[0].entity.status(sid::POISON), 6);
    }

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
        assert_eq!(state.player.status(sid::POTION_DRAW), 3);
    }

    #[test]
    fn test_ancient_potion_artifact() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Ancient Potion", -1);
        assert_eq!(state.player.status(sid::ARTIFACT), 1);
    }

    #[test]
    fn test_regen_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Regen Potion", -1);
        assert_eq!(state.player.status(sid::REGENERATION), 5);
    }

    #[test]
    fn test_essence_of_steel() {
        let mut state = make_test_state();
        apply_potion(&mut state, "EssenceOfSteel", -1);
        assert_eq!(state.player.status(sid::PLATED_ARMOR), 4);
    }

    #[test]
    fn test_liquid_bronze() {
        let mut state = make_test_state();
        apply_potion(&mut state, "LiquidBronze", -1);
        assert_eq!(state.player.status(sid::THORNS), 3);
    }

    #[test]
    fn test_cultist_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "CultistPotion", -1);
        assert_eq!(state.player.status(sid::RITUAL), 1);
    }

    #[test]
    fn test_heart_of_iron() {
        let mut state = make_test_state();
        apply_potion(&mut state, "HeartOfIron", -1);
        assert_eq!(state.player.status(sid::METALLICIZE), 6);
    }

    #[test]
    fn test_ghost_in_a_jar() {
        let mut state = make_test_state();
        apply_potion(&mut state, "GhostInAJar", -1);
        assert_eq!(state.player.status(sid::INTANGIBLE), 1);
    }

    #[test]
    fn test_duplication_potion() {
        let mut state = make_test_state();
        apply_potion(&mut state, "DuplicationPotion", -1);
        assert_eq!(state.player.status(sid::DUPLICATION), 1);
    }

    #[test]
    fn test_blood_potion() {
        let mut state = make_test_state();
        state.player.hp = 60;
        apply_potion(&mut state, "Blood Potion", -1);
        assert_eq!(state.player.hp, 76);
    }

    #[test]
    fn test_fruit_juice() {
        let mut state = make_test_state();
        apply_potion(&mut state, "Fruit Juice", -1);
        assert_eq!(state.player.max_hp, 85);
        assert_eq!(state.player.hp, 85);
    }

    #[test]
    fn test_fairy_revive_check() {
        let mut state = make_test_state();
        assert_eq!(check_fairy_revive(&state), 0);
        state.potions[0] = "FairyPotion".to_string();
        assert_eq!(check_fairy_revive(&state), 24);
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
        assert!(!success);
    }

    #[test]
    fn test_bottled_miracle() {
        let mut state = make_test_state();
        state.hand.clear();
        apply_potion(&mut state, "BottledMiracle", -1);
        let reg = crate::cards::global_registry();
        assert_eq!(state.hand.len(), 2);
        assert_eq!(reg.card_name(state.hand[0].def_id), "Miracle");
        assert_eq!(reg.card_name(state.hand[1].def_id), "Miracle");
    }

    #[test]
    fn test_cunning_potion() {
        let mut state = make_test_state();
        state.hand.clear();
        apply_potion(&mut state, "CunningPotion", -1);
        let reg = crate::cards::global_registry();
        assert_eq!(state.hand.len(), 3);
        assert!(state.hand.iter().all(|c| reg.card_name(c.def_id) == "Shiv"));
    }

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
        state.hand = make_deck(&["A", "B", "C"]);
        apply_potion(&mut state, "GamblersBrew", -1);
        assert!(state.hand.is_empty());
        assert_eq!(state.discard_pile.len(), 3);
        assert_eq!(state.player.status(sid::POTION_DRAW), 3);
    }

    #[test]
    fn test_potion_of_capacity() {
        let mut state = make_test_state();
        apply_potion(&mut state, "PotionOfCapacity", -1);
        assert_eq!(state.player.status(sid::ORB_SLOTS), 2);
    }

    #[test]
    fn test_sacred_bark_doubles_fire() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        let hp = state.enemies[0].entity.hp;
        apply_potion(&mut state, "Fire Potion", 0);
        assert_eq!(state.enemies[0].entity.hp, hp - 40);
    }

    #[test]
    fn test_sacred_bark_doubles_block() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        apply_potion(&mut state, "Block Potion", -1);
        assert_eq!(state.player.block, 24);
    }

    #[test]
    fn test_sacred_bark_doubles_strength() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        apply_potion(&mut state, "Strength Potion", -1);
        assert_eq!(state.player.strength(), 4);
    }

    #[test]
    fn test_sacred_bark_fairy_revive() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        state.potions[0] = "FairyPotion".to_string();
        let revive = check_fairy_revive(&state);
        assert_eq!(revive, 48);
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

    // --- Ascension 11 reduced potency tests ---

    #[test]
    fn test_a11_fire_potion_reduced() {
        let mut state = make_test_state();
        let initial_hp = state.enemies[0].entity.hp;
        apply_potion_scaled(&mut state, "Fire Potion", 0, 11);
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 15);
    }

    #[test]
    fn test_a11_block_potion_reduced() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Block Potion", -1, 11);
        assert_eq!(state.player.block, 9);
    }

    #[test]
    fn test_a11_strength_potion_reduced() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Strength Potion", -1, 11);
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_a11_weak_potion_reduced() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Weak Potion", 0, 11);
        assert_eq!(state.enemies[0].entity.status(sid::WEAKENED), 2);
    }

    #[test]
    fn test_a11_poison_potion_reduced() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Poison Potion", 0, 11);
        assert_eq!(state.enemies[0].entity.status(sid::POISON), 4);
    }

    #[test]
    fn test_a11_energy_potion_reduced() {
        let mut state = make_test_state();
        let initial = state.energy;
        apply_potion_scaled(&mut state, "Energy Potion", -1, 11);
        assert_eq!(state.energy, initial + 1);
    }

    #[test]
    fn test_a11_fruit_juice_reduced() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Fruit Juice", -1, 11);
        assert_eq!(state.player.max_hp, 83);
    }

    #[test]
    fn test_a11_fairy_revive_reduced() {
        let mut state = make_test_state();
        state.potions[0] = "FairyPotion".to_string();
        let revive = check_fairy_revive_scaled(&state, 11);
        assert_eq!(revive, 16);
    }

    #[test]
    fn test_a10_no_reduction() {
        let mut state = make_test_state();
        let initial_hp = state.enemies[0].entity.hp;
        apply_potion_scaled(&mut state, "Fire Potion", 0, 10);
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 20);
    }

    #[test]
    fn test_a11_sacred_bark_stacks() {
        let mut state = make_test_state();
        state.relics.push("SacredBark".to_string());
        let initial_hp = state.enemies[0].entity.hp;
        apply_potion_scaled(&mut state, "Fire Potion", 0, 11);
        assert_eq!(state.enemies[0].entity.hp, initial_hp - 30);
    }

    #[test]
    fn test_a20_potency_same_as_a11() {
        let mut state = make_test_state();
        apply_potion_scaled(&mut state, "Block Potion", -1, 20);
        assert_eq!(state.player.block, 9);
    }
}
