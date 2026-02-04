"""
Slay the Spire - Complete Reward System Handler

Implements all combat and boss reward mechanics from the decompiled game source:

Combat Rewards (Monster/Elite):
- Gold: Base amount by room type, modified by relics (Golden Idol, Gold Tooth)
- Potions: 40% base chance with blizzard modifier, relic effects (White Beast Statue, Sozu)
- Cards: 3 choices (modified by relics), rarity with pity timer
- Relics: Elite-only, tier rolled from relicRng

Boss Rewards:
- 3 boss relic choices
- Gold reward
- Optional potion

Reward Actions:
- PickCard: Select a card from reward
- SkipCard: Skip card reward (Singing Bowl option for +2 max HP)
- TakeGold: Auto-collected but tracked
- TakePotion: Add to slot if available
- SkipPotion: Leave potion
- TakeRelic: Add relic, trigger onPickup effects
- TakeBossRelic: Select one of 3 boss relics
- ProceedFromRewards: Done collecting rewards, return to map
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Tuple, Union, TYPE_CHECKING
from enum import Enum, auto

from ..state.run import RunState, CardInstance
from ..state.rng import Random
from ..generation.rewards import (
    RewardState, CardBlizzardState, PotionBlizzardState,
    generate_card_rewards, generate_gold_reward,
    generate_relic_reward, generate_elite_relic_reward, generate_boss_relics,
    check_potion_drop, generate_potion_reward,
    RelicTier, get_relic,
    GOLD_REWARDS,
)
from ..content.cards import Card, CardRarity
from ..content.potions import Potion
from ..content.relics import Relic, ALL_RELICS


# ============================================================================
# CONSTANTS - Gold Reward Modifiers
# ============================================================================

# Gold Tooth: +15 gold per enemy killed (tracked per combat)
GOLD_TOOTH_BONUS = 15

# Golden Idol: +25% gold
GOLDEN_IDOL_MULTIPLIER = 1.25


# ============================================================================
# REWARD DATA CLASSES
# ============================================================================

class RewardType(Enum):
    """Types of rewards available after combat."""
    GOLD = auto()
    POTION = auto()
    CARD = auto()
    RELIC = auto()
    EMERALD_KEY = auto()


@dataclass
class GoldReward:
    """A gold reward."""
    amount: int
    claimed: bool = False

    def __repr__(self) -> str:
        status = " (claimed)" if self.claimed else ""
        return f"Gold: {self.amount}{status}"


@dataclass
class PotionReward:
    """A potion reward."""
    potion: Potion
    claimed: bool = False
    skipped: bool = False

    def __repr__(self) -> str:
        status = " (claimed)" if self.claimed else " (skipped)" if self.skipped else ""
        return f"Potion: {self.potion.name}{status}"


@dataclass
class CardReward:
    """A card reward with multiple choices."""
    cards: List[Card]
    claimed_index: Optional[int] = None
    skipped: bool = False
    singing_bowl_used: bool = False  # Took +2 max HP instead

    @property
    def is_resolved(self) -> bool:
        return self.claimed_index is not None or self.skipped or self.singing_bowl_used

    def __repr__(self) -> str:
        if self.claimed_index is not None:
            return f"Card: {self.cards[self.claimed_index].name} (claimed)"
        if self.singing_bowl_used:
            return "Card: Singing Bowl (+2 Max HP)"
        if self.skipped:
            return "Card: (skipped)"
        card_names = [c.name for c in self.cards]
        return f"Card choices: {card_names}"


@dataclass
class RelicReward:
    """A relic reward."""
    relic: Relic
    claimed: bool = False

    def __repr__(self) -> str:
        status = " (claimed)" if self.claimed else ""
        return f"Relic: {self.relic.name}{status}"


@dataclass
class EmeraldKeyReward:
    """An emerald key reward (from burning elite)."""
    claimed: bool = False

    def __repr__(self) -> str:
        status = " (claimed)" if self.claimed else ""
        return f"Emerald Key{status}"


@dataclass
class BossRelicChoices:
    """Boss relic choices (pick 1 of 3)."""
    relics: List[Relic]
    chosen_index: Optional[int] = None  # -1 means skipped

    @property
    def is_resolved(self) -> bool:
        return self.chosen_index is not None

    @property
    def is_skipped(self) -> bool:
        return self.chosen_index == -1

    def __repr__(self) -> str:
        if self.chosen_index == -1:
            return "Boss Relic: (skipped)"
        if self.chosen_index is not None and 0 <= self.chosen_index < len(self.relics):
            return f"Boss Relic: {self.relics[self.chosen_index].name} (chosen)"
        names = [r.name for r in self.relics]
        return f"Boss Relic choices: {names}"


@dataclass
class CombatRewards:
    """
    All rewards available after a combat victory.

    Tracks what rewards are available and which have been claimed/skipped.
    """
    room_type: str  # "monster", "elite", or "boss"
    gold: Optional[GoldReward] = None
    potion: Optional[PotionReward] = None
    card_rewards: List[CardReward] = field(default_factory=list)  # Can be 2 with Prayer Wheel
    relic: Optional[RelicReward] = None
    second_relic: Optional[RelicReward] = None  # Black Star second elite relic
    emerald_key: Optional[EmeraldKeyReward] = None
    boss_relics: Optional[BossRelicChoices] = None

    # Track enemy kills for Gold Tooth
    enemies_killed: int = 0

    @property
    def all_resolved(self) -> bool:
        """Check if all rewards have been claimed or skipped."""
        # Gold is auto-claimed
        if self.gold and not self.gold.claimed:
            return False

        # Potion must be claimed or skipped
        if self.potion and not self.potion.claimed and not self.potion.skipped:
            return False

        # All card rewards must be resolved
        for card_reward in self.card_rewards:
            if not card_reward.is_resolved:
                return False

        # Relic must be claimed (can't skip elite relic)
        if self.relic and not self.relic.claimed:
            return False

        # Emerald key is optional
        # Boss relics must be chosen
        if self.boss_relics and not self.boss_relics.is_resolved:
            return False

        return True

    def get_unclaimed_rewards(self) -> List[str]:
        """Get list of unclaimed reward types."""
        unclaimed = []
        if self.gold and not self.gold.claimed:
            unclaimed.append("gold")
        if self.potion and not self.potion.claimed and not self.potion.skipped:
            unclaimed.append("potion")
        for i, card in enumerate(self.card_rewards):
            if not card.is_resolved:
                unclaimed.append(f"card_{i}")
        if self.relic and not self.relic.claimed:
            unclaimed.append("relic")
        if self.emerald_key and not self.emerald_key.claimed:
            unclaimed.append("emerald_key")
        if self.boss_relics and not self.boss_relics.is_resolved:
            unclaimed.append("boss_relic")
        return unclaimed


# ============================================================================
# REWARD ACTION TYPES
# ============================================================================

@dataclass(frozen=True)
class ClaimGoldAction:
    """Claim gold reward (usually auto-claimed)."""
    pass


@dataclass(frozen=True)
class ClaimPotionAction:
    """Claim potion reward."""
    pass


@dataclass(frozen=True)
class SkipPotionAction:
    """Skip potion reward."""
    pass


@dataclass(frozen=True)
class PickCardAction:
    """Pick a card from the card reward."""
    card_reward_index: int  # Which card reward (0 or 1 with Prayer Wheel)
    card_index: int  # Which card to pick


@dataclass(frozen=True)
class SkipCardAction:
    """Skip a card reward."""
    card_reward_index: int


@dataclass(frozen=True)
class SingingBowlAction:
    """Use Singing Bowl to gain +2 max HP instead of card."""
    card_reward_index: int


@dataclass(frozen=True)
class ClaimRelicAction:
    """Claim relic reward (elite only)."""
    pass


@dataclass(frozen=True)
class ClaimEmeraldKeyAction:
    """Claim emerald key from burning elite."""
    pass


@dataclass(frozen=True)
class SkipEmeraldKeyAction:
    """Skip emerald key (take relic instead)."""
    pass


@dataclass(frozen=True)
class PickBossRelicAction:
    """Pick a boss relic (1 of 3)."""
    relic_index: int


@dataclass(frozen=True)
class SkipBossRelicAction:
    """Skip boss relic selection."""
    pass


@dataclass(frozen=True)
class ProceedFromRewardsAction:
    """Done with rewards, return to map."""
    pass


RewardAction = Union[
    ClaimGoldAction, ClaimPotionAction, SkipPotionAction,
    PickCardAction, SkipCardAction, SingingBowlAction,
    ClaimRelicAction, ClaimEmeraldKeyAction, SkipEmeraldKeyAction,
    PickBossRelicAction, SkipBossRelicAction, ProceedFromRewardsAction
]


# ============================================================================
# REWARD HANDLER
# ============================================================================

class RewardHandler:
    """
    Handles all combat reward generation and collection.

    This handler manages:
    1. Generating rewards based on room type and run state
    2. Applying relic modifiers to rewards
    3. Processing reward actions
    4. Tracking pity timers (card blizzard, potion blizzard)
    """

    @staticmethod
    def generate_combat_rewards(
        run_state: RunState,
        room_type: str,
        card_rng: Random,
        treasure_rng: Random,
        potion_rng: Random,
        relic_rng: Random,
        enemies_killed: int = 1,
        is_burning_elite: bool = False,
    ) -> CombatRewards:
        """
        Generate all rewards for a combat victory.

        Args:
            run_state: Current run state (for relics, blizzard modifiers)
            room_type: "monster", "elite", or "boss"
            card_rng: Card RNG stream
            treasure_rng: Treasure RNG stream (for gold)
            potion_rng: Potion RNG stream
            relic_rng: Relic RNG stream
            enemies_killed: Number of enemies killed (for Gold Tooth)
            is_burning_elite: Whether this is a burning elite (emerald key)

        Returns:
            CombatRewards with all available rewards
        """
        rewards = CombatRewards(room_type=room_type, enemies_killed=enemies_killed)

        # Build reward state from run state
        reward_state = RewardState()
        reward_state.card_blizzard.offset = run_state.card_blizzard
        reward_state.potion_blizzard.modifier = run_state.potion_blizzard
        reward_state.owned_relics = set(run_state.get_relic_ids())

        # --- GOLD REWARD ---
        rewards.gold = RewardHandler._generate_gold_reward(
            run_state, room_type, treasure_rng, enemies_killed
        )

        # --- POTION REWARD ---
        rewards.potion = RewardHandler._generate_potion_reward(
            run_state, room_type, potion_rng, reward_state
        )

        # --- CARD REWARDS ---
        num_card_rewards = 1
        if room_type == "monster" and run_state.has_relic("Prayer Wheel"):
            num_card_rewards = 2

        for _ in range(num_card_rewards):
            card_reward = RewardHandler._generate_card_reward(
                run_state, room_type, card_rng, reward_state
            )
            if card_reward:
                rewards.card_rewards.append(card_reward)

        # --- RELIC REWARD (Elite only) ---
        if room_type == "elite":
            relic = generate_elite_relic_reward(
                relic_rng,
                reward_state,
                run_state.character,
                run_state.act
            )
            if relic:
                rewards.relic = RelicReward(relic=relic)

            # Second relic with Black Star
            if run_state.has_relic("Black Star"):
                second_relic = generate_elite_relic_reward(
                    relic_rng,
                    reward_state,  # Already includes first relic
                    run_state.character,
                    run_state.act
                )
                if second_relic:
                    # Wrap in RelicReward so it appears on the reward screen
                    rewards.second_relic = RelicReward(relic=second_relic)

        # --- EMERALD KEY (Burning Elite) ---
        if is_burning_elite and not run_state.has_emerald_key:
            rewards.emerald_key = EmeraldKeyReward()

        # Update run state blizzard from reward state
        run_state.card_blizzard = reward_state.card_blizzard.offset
        run_state.potion_blizzard = reward_state.potion_blizzard.modifier

        return rewards

    @staticmethod
    def generate_boss_rewards(
        run_state: RunState,
        card_rng: Random,
        treasure_rng: Random,
        potion_rng: Random,
        relic_rng: Random,
    ) -> CombatRewards:
        """
        Generate boss rewards (gold, potion, boss relic choice).

        Args:
            run_state: Current run state
            card_rng: Card RNG stream
            treasure_rng: Treasure RNG stream
            potion_rng: Potion RNG stream
            relic_rng: Relic RNG stream

        Returns:
            CombatRewards with boss-specific rewards
        """
        rewards = CombatRewards(room_type="boss")

        # Build reward state
        reward_state = RewardState()
        reward_state.owned_relics = set(run_state.get_relic_ids())
        reward_state.potion_blizzard.modifier = run_state.potion_blizzard

        # --- GOLD REWARD ---
        rewards.gold = RewardHandler._generate_gold_reward(
            run_state, "boss", treasure_rng, 1
        )

        # --- POTION REWARD ---
        rewards.potion = RewardHandler._generate_potion_reward(
            run_state, "boss", potion_rng, reward_state
        )

        # --- BOSS RELIC CHOICES ---
        boss_relics = generate_boss_relics(
            relic_rng,
            reward_state,
            run_state.character,
            run_state.act,
            num_choices=3
        )
        if boss_relics:
            rewards.boss_relics = BossRelicChoices(relics=boss_relics)

        # Update potion blizzard
        run_state.potion_blizzard = reward_state.potion_blizzard.modifier

        return rewards

    @staticmethod
    def _generate_gold_reward(
        run_state: RunState,
        room_type: str,
        treasure_rng: Random,
        enemies_killed: int,
    ) -> GoldReward:
        """Generate gold reward with all modifiers."""
        # Base gold from room type
        base_gold = generate_gold_reward(
            treasure_rng,
            room_type,
            run_state.ascension,
            has_golden_idol=run_state.has_relic("Golden Idol")
        )

        # Gold Tooth: +15 per enemy killed
        if run_state.has_relic("Gold Tooth"):
            base_gold += GOLD_TOOTH_BONUS * enemies_killed

        return GoldReward(amount=base_gold)

    @staticmethod
    def _generate_potion_reward(
        run_state: RunState,
        room_type: str,
        potion_rng: Random,
        reward_state: RewardState,
    ) -> Optional[PotionReward]:
        """Generate potion reward with blizzard modifier and relics."""
        # Sozu prevents potions entirely - don't touch blizzard counter
        if run_state.has_relic("Sozu"):
            return None

        # Check for drop
        dropped, potion = check_potion_drop(
            potion_rng,
            reward_state,
            room_type,
            has_white_beast_statue=run_state.has_relic("White Beast Statue"),
            has_sozu=run_state.has_relic("Sozu"),
        )

        if dropped and potion:
            return PotionReward(potion=potion)

        return None

    @staticmethod
    def _generate_card_reward(
        run_state: RunState,
        room_type: str,
        card_rng: Random,
        reward_state: RewardState,
    ) -> Optional[CardReward]:
        """Generate card reward with all modifiers."""
        # Determine number of cards
        num_cards = 3

        # Question Card: +1 card choice
        if run_state.has_relic("Question Card"):
            num_cards += 1

        # Busted Crown: -2 card choices
        if run_state.has_relic("Busted Crown"):
            num_cards -= 2

        num_cards = max(1, num_cards)

        # Generate cards
        cards = generate_card_rewards(
            card_rng,
            reward_state,
            act=run_state.act,
            player_class=run_state.character,
            ascension=run_state.ascension,
            room_type=room_type,
            num_cards=num_cards,
            has_prismatic_shard=run_state.has_relic("PrismaticShard"),
            has_busted_crown=run_state.has_relic("Busted Crown"),
            has_question_card=run_state.has_relic("Question Card"),
            has_nloth_gift=run_state.has_relic("Nloth's Gift"),
        )

        if cards:
            return CardReward(cards=cards)

        return None

    # =========================================================================
    # REWARD ACTIONS
    # =========================================================================

    @staticmethod
    def get_available_actions(
        run_state: RunState,
        rewards: CombatRewards,
    ) -> List[RewardAction]:
        """
        Get all valid reward actions for the current state.

        Args:
            run_state: Current run state
            rewards: Current combat rewards

        Returns:
            List of valid RewardAction objects
        """
        actions: List[RewardAction] = []

        # Gold is auto-claimed, but we include it for completeness
        if rewards.gold and not rewards.gold.claimed:
            actions.append(ClaimGoldAction())

        # Potion actions
        if rewards.potion and not rewards.potion.claimed and not rewards.potion.skipped:
            # Can claim if have empty slot
            if run_state.count_empty_potion_slots() > 0:
                actions.append(ClaimPotionAction())
            actions.append(SkipPotionAction())

        # Card actions
        for i, card_reward in enumerate(rewards.card_rewards):
            if not card_reward.is_resolved:
                # Can pick any card
                for j in range(len(card_reward.cards)):
                    actions.append(PickCardAction(card_reward_index=i, card_index=j))

                # Can skip (always allowed, but especially with Question Card)
                actions.append(SkipCardAction(card_reward_index=i))

                # Singing Bowl option
                if run_state.has_relic("Singing Bowl"):
                    actions.append(SingingBowlAction(card_reward_index=i))

        # Relic actions (elite)
        if rewards.relic and not rewards.relic.claimed:
            actions.append(ClaimRelicAction())

        # Emerald key actions
        if rewards.emerald_key and not rewards.emerald_key.claimed:
            actions.append(ClaimEmeraldKeyAction())
            actions.append(SkipEmeraldKeyAction())

        # Boss relic actions
        if rewards.boss_relics and not rewards.boss_relics.is_resolved:
            for i in range(len(rewards.boss_relics.relics)):
                actions.append(PickBossRelicAction(relic_index=i))
            # Allow explicit skip
            actions.append(SkipBossRelicAction())

        # Can proceed if all mandatory rewards resolved
        # Mandatory: gold (auto), card (pick/skip), relic (claim), boss relic (pick)
        # Optional: potion, emerald key
        mandatory_resolved = True

        if rewards.gold and not rewards.gold.claimed:
            mandatory_resolved = False

        for card_reward in rewards.card_rewards:
            if not card_reward.is_resolved:
                mandatory_resolved = False
                break

        if rewards.relic and not rewards.relic.claimed:
            mandatory_resolved = False

        if rewards.boss_relics and not rewards.boss_relics.is_resolved:
            mandatory_resolved = False

        if mandatory_resolved:
            actions.append(ProceedFromRewardsAction())

        return actions

    @staticmethod
    def handle_action(
        action: RewardAction,
        run_state: RunState,
        rewards: CombatRewards,
    ) -> Dict[str, Any]:
        """
        Process a reward action and update state.

        Args:
            action: The action to process
            run_state: Run state to modify
            rewards: Rewards to modify

        Returns:
            Dict with action result details
        """
        result: Dict[str, Any] = {"success": True, "action_type": type(action).__name__}

        if isinstance(action, ClaimGoldAction):
            if rewards.gold and not rewards.gold.claimed:
                run_state.add_gold(rewards.gold.amount)
                rewards.gold.claimed = True
                result["gold_gained"] = rewards.gold.amount
                result["new_total"] = run_state.gold
            else:
                result["success"] = False
                result["error"] = "No gold to claim"

        elif isinstance(action, ClaimPotionAction):
            if rewards.potion and not rewards.potion.claimed and not rewards.potion.skipped:
                if run_state.count_empty_potion_slots() > 0:
                    run_state.add_potion(rewards.potion.potion.id)
                    rewards.potion.claimed = True
                    result["potion_id"] = rewards.potion.potion.id
                    result["potion_name"] = rewards.potion.potion.name
                else:
                    result["success"] = False
                    result["error"] = "No empty potion slots"
            else:
                result["success"] = False
                result["error"] = "No potion to claim"

        elif isinstance(action, SkipPotionAction):
            if rewards.potion and not rewards.potion.claimed and not rewards.potion.skipped:
                rewards.potion.skipped = True
                result["potion_skipped"] = rewards.potion.potion.name
            else:
                result["success"] = False
                result["error"] = "No potion to skip"

        elif isinstance(action, PickCardAction):
            idx = action.card_reward_index
            if idx < len(rewards.card_rewards):
                card_reward = rewards.card_rewards[idx]
                if not card_reward.is_resolved and action.card_index < len(card_reward.cards):
                    card = card_reward.cards[action.card_index]
                    run_state.add_card(card.id, card.upgraded)
                    card_reward.claimed_index = action.card_index

                    # Update card blizzard
                    run_state.on_card_reward_taken(card.rarity == CardRarity.RARE)

                    result["card_id"] = card.id
                    result["card_name"] = card.name
                    result["card_upgraded"] = card.upgraded
                    result["card_rarity"] = card.rarity.name
                else:
                    result["success"] = False
                    result["error"] = "Invalid card index or already resolved"
            else:
                result["success"] = False
                result["error"] = "Invalid card reward index"

        elif isinstance(action, SkipCardAction):
            idx = action.card_reward_index
            if idx < len(rewards.card_rewards):
                card_reward = rewards.card_rewards[idx]
                if not card_reward.is_resolved:
                    card_reward.skipped = True
                    result["cards_skipped"] = [c.name for c in card_reward.cards]
                else:
                    result["success"] = False
                    result["error"] = "Card reward already resolved"
            else:
                result["success"] = False
                result["error"] = "Invalid card reward index"

        elif isinstance(action, SingingBowlAction):
            if not run_state.has_relic("Singing Bowl"):
                result["success"] = False
                result["error"] = "Don't have Singing Bowl"
            else:
                idx = action.card_reward_index
                if idx < len(rewards.card_rewards):
                    card_reward = rewards.card_rewards[idx]
                    if not card_reward.is_resolved:
                        run_state.gain_max_hp(2)
                        card_reward.singing_bowl_used = True
                        result["max_hp_gained"] = 2
                        result["new_max_hp"] = run_state.max_hp
                    else:
                        result["success"] = False
                        result["error"] = "Card reward already resolved"
                else:
                    result["success"] = False
                    result["error"] = "Invalid card reward index"

        elif isinstance(action, ClaimRelicAction):
            if rewards.relic and not rewards.relic.claimed:
                run_state.add_relic(rewards.relic.relic.id)
                rewards.relic.claimed = True
                result["relic_id"] = rewards.relic.relic.id
                result["relic_name"] = rewards.relic.relic.name
            else:
                result["success"] = False
                result["error"] = "No relic to claim"

        elif isinstance(action, ClaimEmeraldKeyAction):
            if rewards.emerald_key and not rewards.emerald_key.claimed:
                run_state.obtain_emerald_key()
                rewards.emerald_key.claimed = True
                result["emerald_key_obtained"] = True
            else:
                result["success"] = False
                result["error"] = "No emerald key to claim"

        elif isinstance(action, SkipEmeraldKeyAction):
            if rewards.emerald_key and not rewards.emerald_key.claimed:
                # Just mark it as claimed (skipped)
                rewards.emerald_key.claimed = True  # Mark resolved but didn't take
                result["emerald_key_skipped"] = True
            else:
                result["success"] = False
                result["error"] = "No emerald key to skip"

        elif isinstance(action, PickBossRelicAction):
            if rewards.boss_relics and not rewards.boss_relics.is_resolved:
                idx = action.relic_index
                if 0 <= idx < len(rewards.boss_relics.relics):
                    relic = rewards.boss_relics.relics[idx]

                    # Handle starter relic replacement
                    starter_replacement = RewardHandler._handle_boss_relic_pickup(
                        run_state, relic
                    )

                    run_state.add_relic(relic.id)
                    rewards.boss_relics.chosen_index = idx

                    result["relic_id"] = relic.id
                    result["relic_name"] = relic.name
                    if starter_replacement:
                        result["replaced_starter"] = starter_replacement
                else:
                    result["success"] = False
                    result["error"] = "Invalid boss relic index"
            else:
                result["success"] = False
                result["error"] = "No boss relic to pick"

        elif isinstance(action, SkipBossRelicAction):
            if rewards.boss_relics and not rewards.boss_relics.is_resolved:
                # Mark boss relic as skipped (use -1 to indicate skip)
                rewards.boss_relics.chosen_index = -1
                result["boss_relic_skipped"] = True
            else:
                result["success"] = False
                result["error"] = "No boss relic to skip"

        elif isinstance(action, ProceedFromRewardsAction):
            result["proceeding_to_map"] = True

        return result

    # Alias for API compatibility
    execute_action = handle_action

    @staticmethod
    def _handle_boss_relic_pickup(
        run_state: RunState,
        relic: Relic,
    ) -> Optional[str]:
        """
        Handle special boss relic effects on pickup.

        Some boss relics replace starter relics:
        - Black Blood replaces Burning Blood
        - Ring of the Serpent replaces Ring of the Snake
        - Frozen Core replaces Cracked Core
        - Holy Water replaces Pure Water

        Returns:
            ID of replaced relic if any
        """
        replacements = {
            "BlackBlood": "BurningBlood",
            "RingOfTheSerpent": "RingOfTheSnake",
            "FrozenCore": "CrackedCore",
            "HolyWater": "PureWater",
        }

        if relic.id in replacements:
            starter_id = replacements[relic.id]
            if run_state.has_relic(starter_id):
                # Remove the starter relic
                for i, r in enumerate(run_state.relics):
                    if r.id == starter_id:
                        run_state.relics.pop(i)
                        return starter_id

        return None

    @staticmethod
    def auto_claim_gold(run_state: RunState, rewards: CombatRewards) -> int:
        """
        Auto-claim gold reward (gold is always auto-collected in StS).

        Returns:
            Amount of gold claimed
        """
        if rewards.gold and not rewards.gold.claimed:
            run_state.add_gold(rewards.gold.amount)
            rewards.gold.claimed = True
            return rewards.gold.amount
        return 0


# ============================================================================
# REWARD RESULT DATACLASS (for game runner integration)
# ============================================================================

@dataclass
class RewardResult:
    """Result of processing a reward action."""
    success: bool
    action_type: str
    details: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> 'RewardResult':
        success = d.pop("success", True)
        action_type = d.pop("action_type", "unknown")
        return cls(success=success, action_type=action_type, details=d)


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    from ..state.run import create_watcher_run
    from ..state.rng import seed_to_long

    print("=== Reward Handler Test ===\n")

    # Create test run
    seed_str = "REWARDTEST"
    seed = seed_to_long(seed_str)
    run = create_watcher_run(seed_str, ascension=20)

    print(f"Created run: {run}")
    print(f"Starting gold: {run.gold}")
    print(f"Starting HP: {run.current_hp}/{run.max_hp}")
    print(f"Potion slots: {run.count_empty_potion_slots()}\n")

    # Initialize RNG streams
    card_rng = Random(seed)
    treasure_rng = Random(seed + 1)
    potion_rng = Random(seed + 2)
    relic_rng = Random(seed + 3)

    # Test monster combat rewards
    print("--- Monster Combat Rewards ---")
    rewards = RewardHandler.generate_combat_rewards(
        run, "monster", card_rng, treasure_rng, potion_rng, relic_rng,
        enemies_killed=2
    )

    print(f"Gold: {rewards.gold}")
    print(f"Potion: {rewards.potion}")
    print(f"Card rewards: {len(rewards.card_rewards)}")
    for i, cr in enumerate(rewards.card_rewards):
        print(f"  Card reward {i}: {cr}")
    print(f"Relic: {rewards.relic}")

    # Get available actions
    print("\n--- Available Actions ---")
    actions = RewardHandler.get_available_actions(run, rewards)
    for action in actions[:10]:  # Show first 10
        print(f"  {action}")
    print(f"  ... ({len(actions)} total)")

    # Process some actions
    print("\n--- Processing Actions ---")

    # Auto-claim gold
    gold_claimed = RewardHandler.auto_claim_gold(run, rewards)
    print(f"Auto-claimed gold: {gold_claimed} (total: {run.gold})")

    # Pick first card
    if rewards.card_rewards:
        action = PickCardAction(card_reward_index=0, card_index=0)
        result = RewardHandler.handle_action(action, run, rewards)
        print(f"Picked card: {result}")

    # Skip potion if available
    if rewards.potion and not rewards.potion.claimed:
        action = SkipPotionAction()
        result = RewardHandler.handle_action(action, run, rewards)
        print(f"Skipped potion: {result}")

    print(f"\nFinal gold: {run.gold}")
    print(f"Deck size: {len(run.deck)}")
    print(f"All resolved: {rewards.all_resolved}")

    # Test elite rewards
    print("\n--- Elite Combat Rewards ---")
    rewards = RewardHandler.generate_combat_rewards(
        run, "elite", card_rng, treasure_rng, potion_rng, relic_rng,
        enemies_killed=1
    )

    print(f"Gold: {rewards.gold}")
    print(f"Potion: {rewards.potion}")
    print(f"Card rewards: {len(rewards.card_rewards)}")
    print(f"Relic: {rewards.relic}")

    # Test boss rewards
    print("\n--- Boss Rewards ---")
    rewards = RewardHandler.generate_boss_rewards(
        run, card_rng, treasure_rng, potion_rng, relic_rng
    )

    print(f"Gold: {rewards.gold}")
    print(f"Potion: {rewards.potion}")
    print(f"Boss relics: {rewards.boss_relics}")

    print("\n=== All tests passed ===")
