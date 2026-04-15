//! Typed effect structs and context types for the card effect registry.
//!
//! Hook functions receive context and return typed effect structs.
//! The engine applies effects after dispatch — hooks never mutate state directly
//! (except complex on_play hooks which get &mut CombatEngine).

use serde::{Deserialize, Serialize};

use crate::combat_types::CardInstance;
use crate::cards::CardDef;
use crate::engine::CombatEngine;

pub type ComplexCardHook = fn(&mut CombatEngine, &CardPlayContext);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRuntimeTraits {
    pub innate: bool,
    pub retain: bool,
    pub ethereal: bool,
    pub unplayable: bool,
    pub limit_cards_per_turn: bool,
    pub unremovable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanPlayRule {
    OnlyAttackInHand,
    OnlyAttacksInHand,
    OnlyEmptyDraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostModifierRule {
    ReduceOnHpLoss,
    ReducePerPower,
    ReduceOnDiscard,
    IncreaseOnHpLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageModifierRule {
    HeavyBlade,
    DamageEqualsBlock,
    DamagePlusMantra,
    PerfectedStrike,
    Rampage,
    GlassKnife,
    RitualDagger,
    SearingBlow,
    DamageRandomXTimes,
    WindmillStrike,
    ClawScaling,
    DamagePerLightning,
    DamageFromDrawPile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnDrawRule {
    LoseEnergy,
    CopySelf,
    DeusExMachina,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnDiscardRule {
    DrawCards,
    GainEnergy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnRetainRule {
    ReduceCost,
    GrowBlock,
    GrowDamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnExhaustRule {
    GainEnergy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostPlayRule {
    ShuffleIntoDraw,
    EndTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndTurnHandRule {
    Damage,
    Regret,
    Weak,
    Frail,
    AddCopy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhileInHandRule {
    PainOnOtherCardPlayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardRuntimeTrigger {
    CanPlay(CanPlayRule),
    ModifyCost(CostModifierRule),
    ModifyDamage(DamageModifierRule),
    OnDraw(OnDrawRule),
    OnDiscard(OnDiscardRule),
    OnRetain(OnRetainRule),
    OnExhaust(OnExhaustRule),
    PostPlay(PostPlayRule),
    EndTurnInHand(EndTurnHandRule),
    WhileInHand(WhileInHandRule),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardBlockHint {
    XTimes,
    IfSkill,
    IfNoBlock,
    BulkCountTimesBaseBlock,
    UsesCardMisc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardEvokeHint {
    Fixed(u8),
    XCost,
    XCostPlus(u8),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardPlayHints {
    pub draws_cards: bool,
    pub discards_cards: bool,
    pub x_cost: bool,
    pub multi_hit: bool,
    pub block_hint: Option<CardBlockHint>,
    pub evoke_hint: Option<CardEvokeHint>,
    pub channel_evoked_orb: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardMetadata {
    pub runtime_traits: CardRuntimeTraits,
    pub runtime_triggers: Box<[CardRuntimeTrigger]>,
    pub play_hints: CardPlayHints,
}

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
    /// Hand size after the played card has been removed from hand.
    pub hand_size_at_play: usize,
    /// Count recorded by the most recent bulk pile operation in this card play.
    pub last_bulk_count: i32,
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
