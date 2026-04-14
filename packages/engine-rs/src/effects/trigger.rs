//! Trigger types for the unified entity effect system.
//!
//! Triggers define WHEN an effect fires (combat start, turn start, on card play, etc.).
//! TriggerConditions define additional guards (first turn only, stance check, etc.).
//! TriggerContext carries per-invocation data (card type, target, etc.).

use crate::cards::CardType;
use crate::ids::StatusId;
use crate::state::Stance;

// ===========================================================================
// Trigger — when an effect fires
// ===========================================================================

/// When a triggered effect should fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// At the start of combat (before initial draw).
    CombatStart,
    /// At combat start, before the pre-draw phase.
    CombatStartPreDraw,
    /// At the start of each player turn (before draw).
    TurnStart,
    /// At the start of each player turn (after draw).
    TurnStartPostDraw,
    /// Late start of turn, after post-draw power/setup effects have resolved.
    TurnStartPostDrawLate,
    /// At the end of each player turn.
    TurnEnd,
    /// Late end of turn, after orb passives and Loop have resolved.
    TurnEndPostOrbs,
    /// When combat is won.
    CombatVictory,
    /// Before a card is played (can modify).
    OnCardPlayedPre,
    /// Java-style `onPlayCard`: card has been committed to play and counted.
    OnPlayCard,
    /// Java-style `onUseCard`: fires before the card's effects resolve.
    OnUseCard,
    /// After a card is played.
    OnCardPlayedPost,
    /// Java-style `onAfterUseCard`: card effects resolved, before replay window.
    OnAfterUseCard,
    /// Java-style `onAfterCardPlayed`: card fully played for trigger purposes.
    OnAfterCardPlayed,
    /// After an Attack card is played.
    OnAttackPlayed,
    /// After a Skill card is played.
    OnSkillPlayed,
    /// After a Power card is played.
    OnPowerPlayed,
    /// After any card is played (regardless of type).
    OnAnyCardPlayed,
    /// When a card is exhausted.
    OnCardExhaust,
    /// When a card is discarded.
    OnCardDiscard,
    /// When the player loses HP.
    OnPlayerHpLoss,
    /// When an enemy dies.
    OnEnemyDeath,
    /// When the draw pile is shuffled.
    OnShuffle,
    /// When the player changes stance.
    OnStanceChange,
    /// When a potion is used.
    OnPotionUsed,
    /// At the start of the enemy turn.
    EnemyTurnStart,
    /// Only fires when manually activated (e.g. potion use).
    ManualActivation,
    /// When unblocked damage is resolved (for min-damage / reduction relics).
    /// Called from the damage pipeline; not dispatched via dispatch_trigger.
    DamageResolved,
    /// When a debuff is applied (for Champion Belt: Vuln -> also apply Weak).
    /// Called inline; not dispatched via dispatch_trigger.
    OnDebuffApplied,
    /// When an enemy's block is broken by an attack.
    /// Called inline; not dispatched via dispatch_trigger.
    OnBlockBroken,
    /// When damage is calculated (for passive damage bonuses like Strike Dummy).
    /// Called from the damage pipeline; not dispatched via dispatch_trigger.
    DamageCalculation,
    /// When Poison is applied (for Snecko Skull bonus).
    /// Called inline; not dispatched via dispatch_trigger.
    OnPoisonApplied,
}

// ===========================================================================
// TriggerCondition — additional guard on a triggered effect
// ===========================================================================

/// Additional condition that must be true for a triggered effect to fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerCondition {
    /// Always fires (no additional condition).
    Always,
    /// Only on the first turn of combat.
    FirstTurn,
    /// Only on turns after the first.
    NotFirstTurn,
    /// Only when in a specific stance.
    InStance(Stance),
    /// Only when a counter status reaches its threshold.
    CounterReached,
    /// Only when the player has a specific status > 0.
    HasStatus(StatusId),
    /// Only when player HP is below this percentage (0-100).
    HpBelow(u8),
    /// Only when the player has no block.
    NoBlock,
    /// Only when the player's hand is empty.
    HandEmpty,
    /// Only when the triggering card is this type.
    CardTypeIs(CardType),
    /// Only in boss fights.
    IsBossFight,
    /// Only in elite fights.
    IsEliteFight,
    /// Only in elite or boss fights.
    IsEliteOrBossFight,
}

// ===========================================================================
// TriggerContext — per-invocation data
// ===========================================================================

/// Runtime context passed when evaluating a trigger.
#[derive(Debug, Clone, Copy)]
pub struct TriggerContext {
    /// The type of card that caused this trigger (if any).
    pub card_type: Option<CardType>,
    /// Whether this is the first turn of combat.
    pub is_first_turn: bool,
    /// Target enemy index (-1 if no target).
    pub target_idx: i32,
}

impl TriggerContext {
    /// Create a default context with no card and no target.
    pub fn empty() -> Self {
        Self {
            card_type: None,
            is_first_turn: false,
            target_idx: -1,
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_is_copy() {
        let t = Trigger::TurnStart;
        let _t2 = t;
        let _t3 = t; // still valid — Copy
    }

    #[test]
    fn test_trigger_condition_is_copy() {
        let c = TriggerCondition::InStance(Stance::Wrath);
        let _c2 = c;
        let _c3 = c;
    }

    #[test]
    fn test_trigger_context_is_copy() {
        let ctx = TriggerContext::empty();
        let _ctx2 = ctx;
        let _ctx3 = ctx;
    }

    #[test]
    fn test_trigger_context_empty() {
        let ctx = TriggerContext::empty();
        assert_eq!(ctx.card_type, None);
        assert!(!ctx.is_first_turn);
        assert_eq!(ctx.target_idx, -1);
    }

    #[test]
    fn test_trigger_size() {
        let size = std::mem::size_of::<Trigger>();
        assert!(size <= 4, "Trigger is {} bytes, expected <= 4", size);
    }

    #[test]
    fn test_trigger_condition_size() {
        let size = std::mem::size_of::<TriggerCondition>();
        // Contains StatusId (2 bytes) or Stance enum — should be small
        assert!(size <= 8, "TriggerCondition is {} bytes, expected <= 8", size);
    }
}
