//! Declarative card effect types — data-driven effect descriptions.
//!
//! ~180/200 card effects are expressed as static data (enum variants).
//! The interpreter in `interpreter.rs` walks these and routes each
//! through the proper engine methods (which handle Artifact, dex/frail,
//! auto-evoke, onCardDraw hooks, etc.).
//!
//! Only ~10 irreducible effects need fn pointers (Pressure Points,
//! Judgement, Fiend Fire, Havoc, Madness, Reboot, etc.).

use crate::ids::StatusId;
use crate::orbs::OrbType;
use crate::state::Stance;
use crate::cards::CardType;

// ===========================================================================
// Core enums
// ===========================================================================

/// Which entity to target with an effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The player.
    Player,
    /// The entity that owns the currently executing runtime effect.
    /// Used by owner-aware relic/power runtime dispatch.
    SelfEntity,
    /// The single selected enemy (from card play target_idx).
    SelectedEnemy,
    /// All living enemies.
    AllEnemies,
    /// A random living enemy (uses engine RNG).
    RandomEnemy,
}

/// A card pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pile {
    Hand,
    Draw,
    Discard,
    Exhaust,
}

/// How to resolve an integer amount at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountSource {
    /// CardDef.base_magic (floored at 1 unless specified otherwise).
    Magic,
    /// CardDef.base_block (for block pipeline).
    Block,
    /// CardDef.base_damage.
    Damage,
    /// A fixed constant.
    Fixed(i32),
    /// The X-cost value consumed when playing the card.
    XCost,
    /// X-cost value plus a constant bonus (Collect, Conjure Blade upgrades).
    XCostPlus(i32),
    /// base_magic + x_value (Doppelganger, Malaise).
    MagicPlusX,
    /// Number of living enemies.
    LivingEnemyCount,
    /// Number of channeled orbs.
    OrbCount,
    /// Number of unique orb types channeled.
    UniqueOrbCount,
    /// Number of cards in hand.
    HandSize,
    /// Number of cards in hand when the current card began resolving.
    HandSizeAtPlay,
    /// Number of cards in hand when the current card began resolving, plus N.
    HandSizeAtPlayPlus(i32),
    /// Player's current block value.
    PlayerBlock,
    /// Discard pile size.
    DiscardPileSize,
    /// Current value of a status (e.g., read Metallicize stacks).
    StatusValue(crate::ids::StatusId),
    /// Per-card mutable numeric state carried on `CardInstance.misc`.
    CardMisc,
    /// Number of cards currently in the draw pile.
    DrawPileSize,
    /// Percentage of max HP (e.g., 7 = 7% of max HP).
    PercentMaxHp(i32),
    /// Draw pile size divided by N (Aggregate: draw_pile / 4).
    DrawPileDivN(i32),
    /// Number of attacks played this turn (Finisher).
    AttacksThisTurn,
    /// Number of Skill cards in hand (Flechettes).
    SkillsInHand,
    /// Potion effective potency (base value scaled by A11 + Sacred Bark).
    /// The base potency is stored in the EntityDef; runtime resolves via
    /// `effective_potency(base, ascension, sacred_bark)`.
    PotionPotency,
    /// Total unblocked damage dealt during the current card play.
    TotalUnblockedDamage,
}

/// Boolean condition checked at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    /// Player is in this stance.
    InStance(Stance),
    /// Target enemy's intent includes damage.
    EnemyAttacking,
    /// Target enemy has this status > 0.
    EnemyHasStatus(StatusId),
    /// Last card played was this type.
    LastCardType(CardType),
    /// Player has this status > 0.
    PlayerHasStatus(StatusId),
    /// Player block == 0.
    NoBlock,
    /// An enemy was killed during the damage loop (from CardPlayContext).
    EnemyKilled,
    /// Player discarded a card this turn.
    DiscardedThisTurn,
}

/// What happens to selected card(s) after a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceAction {
    /// Discard the selected card(s).
    Discard,
    /// Discard the selected card(s) and resolve the card's post-discard effect.
    DiscardForEffect,
    /// Exhaust the selected card(s).
    Exhaust,
    /// Move selected card(s) to hand.
    MoveToHand,
    /// Put selected card(s) on top of draw pile.
    PutOnTopOfDraw,
    /// Play selected card for free.
    PlayForFree,
    /// Upgrade selected card(s).
    Upgrade,
    /// Copy selected card(s) to hand.
    CopyToHand,
    /// Put selected card(s) on top of draw pile at cost 0.
    PutOnTopAtCostZero,
    /// Put selected card(s) on bottom of draw pile at cost 0.
    PutOnBottomAtCostZero,
    /// Exhaust selected card and gain its cost as energy.
    ExhaustAndGainEnergy,
}

/// Bulk action applied to all matching cards in a pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkAction {
    Exhaust,
    Discard,
    Upgrade,
    SetCost(i32),
    MoveToHand,
    MoveToBottom,
}

/// Filter for selecting cards from a pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardFilter {
    All,
    Attacks,
    Skills,
    NonAttacks,
    ZeroCost,
    Upgradeable,
}

/// Boolean state flags that cards can set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolFlag {
    /// Cannot draw cards this turn.
    NoDraw,
    /// Retain entire hand at end of turn.
    RetainHand,
    /// Skip the enemy turn.
    SkipEnemyTurn,
    /// Next attack played is free.
    NextAttackFree,
    /// Die at start of next turn (Blasphemy).
    Blasphemy,
    /// All cards cost 0 this turn + no draw (Bullet Time).
    BulletTime,
}

/// Runtime card-generation source pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedCardPool {
    Attack,
    Skill,
    Power,
    Colorless,
}

/// Temporary cost override applied to generated cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedCostRule {
    Base,
    ZeroThisTurn,
    ZeroIfPositiveThisTurn,
    /// Set generated cards to cost 0 this turn and upgrade the generated copy.
    /// Used for Transmutation+ Java semantics.
    ZeroThisTurnAndUpgradeGenerated,
}

/// Upgrade rule applied to generated cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedUpgradeRule {
    Base,
    Upgrade,
}

/// Destination for generated cards outside the fixed `Effect` enum surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedDestination {
    Hand,
    Draw,
}

// ===========================================================================
// Effect enums — the core data types
// ===========================================================================

/// A single atomic effect. Each variant is declarative intent — the interpreter
/// routes through the proper engine method which handles all side effects
/// (Artifact, dex/frail, auto-evoke, onCardDraw hooks, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleEffect {
    /// Add status stacks to target. Routes through apply_debuff (handles Artifact)
    /// for enemy debuffs, or add_status for player buffs.
    AddStatus(Target, StatusId, AmountSource),
    /// Set status to exact value (replaces, doesn't stack).
    SetStatus(Target, StatusId, AmountSource),
    /// Multiply existing status value (Catalyst: double/triple poison).
    MultiplyStatus(Target, StatusId, i32),
    /// Draw cards. Routes through engine.draw_cards() (handles reshuffle + onCardDraw).
    DrawCards(AmountSource),
    /// Draw up to a target hand size.
    DrawToHandSize(AmountSource),
    /// Gain energy.
    GainEnergy(AmountSource),
    /// Double current energy.
    DoubleEnergy,
    /// Gain block. Routes through engine.gain_block_player() (handles dex/frail + onGainBlock).
    GainBlock(AmountSource),
    /// Modify HP. Positive = heal, negative = lose HP.
    ModifyHp(AmountSource),
    /// Gain mantra. Routes through engine.gain_mantra() (handles Divinity at 10).
    GainMantra(AmountSource),
    /// Scry N cards. Routes through engine.do_scry() (handles onScry + Weave).
    /// NOTE: May trigger AwaitingChoice — always place last or near-last.
    Scry(AmountSource),
    /// Add a temp card to a pile. Routes through engine.temp_card() + pile push.
    AddCard(&'static str, Pile, AmountSource),
    /// Add a temp card to a pile with explicit misc state.
    AddCardWithMisc(&'static str, Pile, AmountSource, AmountSource),
    /// Copy the played card instance to a pile (Anger: copy to discard).
    CopyThisCardTo(Pile),
    /// Channel an orb. Routes through engine.channel_orb() (handles auto-evoke).
    ChannelOrb(OrbType, AmountSource),
    /// Evoke the front orb N times.
    EvokeOrb(AmountSource),
    /// Change player stance.
    ChangeStance(Stance),
    /// Set a boolean flag on combat state.
    SetFlag(BoolFlag),
    /// Shuffle discard pile into draw pile.
    ShuffleDiscardIntoDraw,
    /// Play the top card of the draw pile through the normal free-play path.
    PlayTopCardOfDraw,
    /// Deal flat damage to a target (no strength/stance modifiers).
    DealDamage(Target, AmountSource),
    /// Special-resolution attack used by Judgement:
    /// if the selected enemy is at or below threshold, deal lethal damage equal
    /// to current HP + block.
    Judgement(AmountSource),
    /// Trigger Mark damage across all enemies: each marked enemy loses HP equal
    /// to its Mark stacks, bypassing block.
    TriggerMarks,
    /// Modify the played card instance's cost by a delta.
    ModifyPlayedCardCost(AmountSource),
    /// Modify the played card instance's current block by a delta.
    ModifyPlayedCardBlock(AmountSource),
    /// Heal HP capped at max HP.
    HealHp(Target, AmountSource),
    /// Increment a counter status; fires associated effect at threshold.
    IncrementCounter(crate::ids::StatusId, i32),
    /// Modify max HP (positive = increase, negative = decrease).
    ModifyMaxHp(AmountSource),
    /// Modify max energy (positive = increase, negative = decrease).
    ModifyMaxEnergy(AmountSource),
    /// Modify gold (positive = gain, negative = lose).
    ModifyGold(AmountSource),
    /// End combat as a flee (player escapes).
    FleeCombat,
}

/// A card's effect — can be simple, conditional, choice-based, or complex.
/// All variants are const-constructible via `&'static` slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// A single atomic effect.
    Simple(SimpleEffect),

    /// Conditional: if condition then execute first slice, else execute second slice.
    /// Covers: Inner Peace, Indignation, Fear No Evil, Spot Weakness, Go for the Eyes,
    /// FollowUp, CrushJoints, SashWhip, Dropkick, Heel Hook, Bane, Sneaky Strike, etc.
    Conditional(Condition, &'static [Effect], &'static [Effect]),

    /// Player chooses card(s) from a pile. MUST be the last effect in the array
    /// (sets AwaitingChoice, interpreter stops).
    /// Covers: True Grit, Headbutt, Exhume, Hologram, Dual Wield, Armaments,
    /// Seek, Recycle, Concentrate, Purity, Secret Weapon/Technique, etc.
    ChooseCards {
        source: Pile,
        filter: CardFilter,
        action: ChoiceAction,
        min_picks: AmountSource,
        max_picks: AmountSource,
    },

    /// Apply a bulk action to all cards matching a filter in a pile.
    /// Covers: Apotheosis (upgrade all), Enlightenment (set cost 1), Second Wind,
    /// All For One, Forethought+, etc.
    ForEachInPile {
        pile: Pile,
        filter: CardFilter,
        action: BulkAction,
    },

    /// Deal extra damage hits based on a dynamic count.
    /// The damage uses the same base damage as the card's damage pipeline.
    /// Covers: Bowling Bash, Barrage, Finisher, Flechettes.
    ExtraHits(AmountSource),

    /// Present N generated cards for the player to choose from.
    /// Covers: Discovery, Foreign Influence, Wish.
    Discover(&'static [&'static str]),

    /// Present a named option menu for effects like Wish.
    ChooseNamedOptions(&'static [&'static str]),

    /// Generate random card(s) from a pool directly into hand.
    GenerateRandomCardsToHand {
        pool: GeneratedCardPool,
        count: AmountSource,
        cost_rule: GeneratedCostRule,
    },

    /// Generate random card(s) from a pool directly into the draw pile.
    GenerateRandomCardsToDraw {
        pool: GeneratedCardPool,
        count: AmountSource,
        cost_rule: GeneratedCostRule,
    },

    /// Present random cards from a pool for a Discovery-style choose-one menu.
    GenerateDiscoveryChoice {
        pool: GeneratedCardPool,
        option_count: usize,
        cost_rule: GeneratedCostRule,
    },
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_is_copy() {
        // Effect must be Copy for static arrays
        let e = Effect::Simple(SimpleEffect::DrawCards(AmountSource::Fixed(2)));
        let _e2 = e; // Copy
        let _e3 = e; // Still valid
    }

    #[test]
    fn test_recursive_static_slice() {
        // Verify that nested &'static [Effect] works
        static INNER: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(AmountSource::Magic))];
        static OUTER: [Effect; 1] = [Effect::Conditional(
            Condition::InStance(Stance::Calm),
            &INNER,
            &[],
        )];
        assert_eq!(OUTER.len(), 1);
    }

    #[test]
    fn test_effect_size() {
        // Track enum size to catch regressions
        let size = std::mem::size_of::<Effect>();
        // Effect contains &'static [Effect] (16 bytes) + discriminant + padding
        // Should be reasonable (under 48 bytes)
        assert!(size <= 48, "Effect is {} bytes, expected <= 48", size);
    }

    #[test]
    fn test_simple_effect_size() {
        let size = std::mem::size_of::<SimpleEffect>();
        // Contains &'static str (16 bytes) in AddCard variant
        assert!(size <= 32, "SimpleEffect is {} bytes, expected <= 32", size);
    }
}
