//! Typed effect structs and context types for the card effect registry.
//!
//! Hook functions receive context and return typed effect structs.
//! The engine applies effects after dispatch — hooks never mutate state directly
//! (except complex on_play hooks which get &mut CombatEngine).

use crate::combat_types::CardInstance;
use crate::cards::CardDef;

/// Context passed to card effect hooks during play.
/// Contains all pre-computed values from the damage preamble.
#[derive(Debug, Clone)]
pub struct CardPlayContext<'a> {
    pub card: &'a CardDef,
    pub card_inst: CardInstance,
    pub target_idx: i32,
    pub x_value: i32,
    pub pen_nib_active: bool,
    pub vigor: i32,
    /// Total unblocked damage dealt during the damage loop (for Wallop, Reaper).
    pub total_unblocked_damage: i32,
    /// Whether an enemy was killed during the damage loop (for Sunder, Ritual Dagger, Feed).
    pub enemy_killed: bool,
}

/// Damage modifier returned by modify_damage hooks.
/// Merged across all active modifiers before the generic damage loop.
#[derive(Debug, Clone)]
pub struct DamageModifier {
    /// Override base damage entirely (Body Slam = player block). -1 = no override.
    pub base_damage_override: i32,
    /// Additive bonus to base damage (Perfected Strike, Brilliance, scaling).
    pub base_damage_bonus: i32,
    /// Strength multiplier (Heavy Blade: 3 or 5). 1 = normal.
    pub strength_multiplier: i32,
    /// Skip generic damage entirely (damage_random_x_times handles own loop).
    pub skip_generic_damage: bool,
}

impl Default for DamageModifier {
    fn default() -> Self {
        Self {
            base_damage_override: -1,
            base_damage_bonus: 0,
            strength_multiplier: 1,
            skip_generic_damage: false,
        }
    }
}

impl DamageModifier {
    pub fn merge(&mut self, other: Self) {
        if other.base_damage_override >= 0 {
            self.base_damage_override = other.base_damage_override;
        }
        self.base_damage_bonus += other.base_damage_bonus;
        if other.strength_multiplier > 1 {
            self.strength_multiplier = self.strength_multiplier.max(other.strength_multiplier);
        }
        self.skip_generic_damage = self.skip_generic_damage || other.skip_generic_damage;
    }
}

/// Effect returned by on_discard hooks.
#[derive(Debug, Default, Clone)]
pub struct OnDiscardEffect {
    pub draw: i32,
    pub energy: i32,
}

impl OnDiscardEffect {
    pub fn merge(&mut self, other: Self) {
        self.draw += other.draw;
        self.energy += other.energy;
    }
}

/// Where a card goes after being played.
#[derive(Debug, Clone, PartialEq)]
pub enum PostPlayDestination {
    /// Normal: discard (or exhaust if card.exhaust)
    Normal,
    /// Shuffle back into draw pile (Tantrum)
    ShuffleIntoDraw,
    /// End the player's turn (Conclude/Meditate)
    EndTurn,
}

impl Default for PostPlayDestination {
    fn default() -> Self {
        Self::Normal
    }
}
