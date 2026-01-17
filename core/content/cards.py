"""
Watcher Card Definitions - Extracted from decompiled card classes.

Card structure matches AbstractCard fields:
- baseDamage, baseBlock, baseMagicNumber
- damage, block, magicNumber (calculated values after modifiers)
- cost, costForTurn
- type (ATTACK, SKILL, POWER, STATUS, CURSE)
- rarity (BASIC, COMMON, UNCOMMON, RARE, SPECIAL, CURSE)
- target (ENEMY, ALL_ENEMY, SELF, NONE, SELF_AND_ENEMY, ALL)

Special flags:
- exhaust: Removes from deck for combat
- ethereal: Exhausts if in hand at end of turn
- retain: Doesn't discard at end of turn
- innate: Starts in opening hand
- shuffleBackIntoDrawPile: Goes to draw pile instead of discard

Upgrade effects vary per card but follow patterns:
- upgradeBaseCost(n): Reduce cost
- upgradeDamage(n): Increase base damage
- upgradeBlock(n): Increase base block
- upgradeMagicNumber(n): Increase magic number
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Callable
from enum import Enum


class CardType(Enum):
    """Card types matching AbstractCard.CardType."""
    ATTACK = "ATTACK"
    SKILL = "SKILL"
    POWER = "POWER"
    STATUS = "STATUS"
    CURSE = "CURSE"


class CardRarity(Enum):
    """Card rarities matching AbstractCard.CardRarity."""
    BASIC = "BASIC"
    COMMON = "COMMON"
    UNCOMMON = "UNCOMMON"
    RARE = "RARE"
    SPECIAL = "SPECIAL"
    CURSE = "CURSE"


class CardTarget(Enum):
    """Card targeting modes."""
    ENEMY = "ENEMY"  # Single enemy
    ALL_ENEMY = "ALL_ENEMY"  # All enemies
    SELF = "SELF"  # Player only
    NONE = "NONE"  # No target
    SELF_AND_ENEMY = "SELF_AND_ENEMY"  # Affects both
    ALL = "ALL"  # Everything


class CardColor(Enum):
    """Card colors (character classes)."""
    RED = "RED"  # Ironclad
    GREEN = "GREEN"  # Silent
    BLUE = "BLUE"  # Defect
    PURPLE = "PURPLE"  # Watcher
    COLORLESS = "COLORLESS"
    CURSE = "CURSE"


@dataclass
class CardEffect:
    """Effect that a card applies."""
    effect_type: str  # damage, block, draw, etc.
    value: int = 0
    target: str = "enemy"  # enemy, self, all_enemies, random_enemy
    hits: int = 1
    extra: Dict[str, Any] = field(default_factory=dict)


@dataclass
class Card:
    """A card definition."""
    id: str
    name: str
    card_type: CardType
    rarity: CardRarity
    color: CardColor = CardColor.PURPLE
    target: CardTarget = CardTarget.ENEMY

    # Base stats
    cost: int = 1
    base_damage: int = -1
    base_block: int = -1
    base_magic: int = -1  # Magic number (hits, draw amount, etc.)

    # Upgraded stats (deltas)
    upgrade_cost: Optional[int] = None  # New cost after upgrade (None = no change)
    upgrade_damage: int = 0
    upgrade_block: int = 0
    upgrade_magic: int = 0

    # Flags
    exhaust: bool = False
    ethereal: bool = False
    retain: bool = False
    innate: bool = False
    shuffle_back: bool = False

    # Stance effects
    enter_stance: Optional[str] = None  # "Wrath", "Calm", "Divinity", "Neutral"
    exit_stance: bool = False

    # Special effects
    effects: List[str] = field(default_factory=list)  # List of effect names

    # Current state (for combat)
    upgraded: bool = False
    cost_for_turn: Optional[int] = None

    @property
    def damage(self) -> int:
        """Current damage value."""
        if self.base_damage < 0:
            return -1
        return self.base_damage + (self.upgrade_damage if self.upgraded else 0)

    @property
    def block(self) -> int:
        """Current block value."""
        if self.base_block < 0:
            return -1
        return self.base_block + (self.upgrade_block if self.upgraded else 0)

    @property
    def magic_number(self) -> int:
        """Current magic number value."""
        if self.base_magic < 0:
            return -1
        return self.base_magic + (self.upgrade_magic if self.upgraded else 0)

    @property
    def current_cost(self) -> int:
        """Current energy cost."""
        if self.cost_for_turn is not None:
            return self.cost_for_turn
        if self.upgraded and self.upgrade_cost is not None:
            return self.upgrade_cost
        return self.cost

    def can_upgrade(self) -> bool:
        """Check if this card can be upgraded (not already upgraded)."""
        return not self.upgraded

    def upgrade(self):
        """Upgrade this card."""
        self.upgraded = True

    def copy(self) -> 'Card':
        """Create a copy of this card."""
        return Card(
            id=self.id, name=self.name, card_type=self.card_type,
            rarity=self.rarity, color=self.color, target=self.target,
            cost=self.cost, base_damage=self.base_damage,
            base_block=self.base_block, base_magic=self.base_magic,
            upgrade_cost=self.upgrade_cost, upgrade_damage=self.upgrade_damage,
            upgrade_block=self.upgrade_block, upgrade_magic=self.upgrade_magic,
            exhaust=self.exhaust, ethereal=self.ethereal, retain=self.retain,
            innate=self.innate, shuffle_back=self.shuffle_back,
            enter_stance=self.enter_stance, exit_stance=self.exit_stance,
            effects=self.effects.copy(), upgraded=self.upgraded,
        )


# ============ WATCHER CARDS ============

# === BASIC CARDS ===

STRIKE_W = Card(
    id="Strike_P", name="Strike", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    cost=1, base_damage=6, upgrade_damage=3,
)

DEFEND_W = Card(
    id="Defend_P", name="Defend", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
)

ERUPTION = Card(
    id="Eruption", name="Eruption", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    cost=2, base_damage=9, upgrade_cost=1, enter_stance="Wrath",
)

VIGILANCE = Card(
    id="Vigilance", name="Vigilance", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    target=CardTarget.SELF, cost=2, base_block=8, upgrade_block=4, enter_stance="Calm",
)

MIRACLE = Card(
    id="Miracle", name="Miracle", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    retain=True, exhaust=True, effects=["gain_1_energy"],
    upgrade_magic=1,  # Upgraded gives 2 energy
)


# === COMMON ATTACKS ===

BOWLING_BASH = Card(
    id="BowlingBash", name="Bowling Bash", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=7, upgrade_damage=3,
    effects=["damage_per_enemy"],  # Deals damage equal to # of enemies
)

CUT_THROUGH_FATE = Card(
    id="CutThroughFate", name="Cut Through Fate", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=7, upgrade_damage=2,
    effects=["scry_2", "draw_1"],
    upgrade_magic=1,  # Scry 3 when upgraded
)

EMPTY_FIST = Card(
    id="EmptyFist", name="Empty Fist", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=9, upgrade_damage=5, exit_stance=True,
)

FLURRY_OF_BLOWS = Card(
    id="FlurryOfBlows", name="Flurry of Blows", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=0, base_damage=4, upgrade_damage=2,
    effects=["on_stance_change_play_from_discard"],
)

FLYING_SLEEVES = Card(
    id="FlyingSleeves", name="Flying Sleeves", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=4, base_magic=2, upgrade_damage=2,
    retain=True,
    effects=["damage_x_times"],  # Hits magic_number times
)

FOLLOW_UP = Card(
    id="FollowUp", name="Follow-Up", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=7, upgrade_damage=4,
    effects=["if_last_card_attack_gain_energy"],
)

HALT = Card(
    id="Halt", name="Halt", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=0, base_block=3, upgrade_block=1,
    effects=["if_in_wrath_extra_block_6"],  # +6 more block in Wrath
)

JUST_LUCKY = Card(
    id="JustLucky", name="Just Lucky", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=0, base_damage=3, upgrade_damage=1,
    base_magic=1, upgrade_magic=1,
    effects=["scry_1", "gain_block_2"],  # Scry, Block, Damage
)

PRESSURE_POINTS = Card(
    id="PathToVictory", name="Pressure Points", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    # Java cardID is "PathToVictory", not "PressurePoints"
    cost=1, base_magic=8, upgrade_magic=3,
    effects=["apply_mark", "trigger_all_marks"],  # Mark damage
)

SASH_WHIP = Card(
    id="SashWhip", name="Sash Whip", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=8, upgrade_damage=2,
    effects=["if_last_card_attack_weak_1"],
)

TRANQUILITY = Card(
    id="ClearTheMind", name="Tranquility", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    # Java cardID is "ClearTheMind", not "Tranquility"
    target=CardTarget.SELF, cost=0, retain=True, exhaust=True,
    enter_stance="Calm",
)

CRESCENDO = Card(
    id="Crescendo", name="Crescendo", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=0, retain=True, exhaust=True,
    enter_stance="Wrath", upgrade_magic=1,  # Upgraded: not exhaust
)

CONSECRATE = Card(
    id="Consecrate", name="Consecrate", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    target=CardTarget.ALL_ENEMY, cost=0, base_damage=5, upgrade_damage=3,
)

CRUSH_JOINTS = Card(
    id="CrushJoints", name="Crush Joints", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=8, upgrade_damage=2,
    effects=["if_last_card_skill_vulnerable_1"],
)


# === COMMON SKILLS ===

EMPTY_BODY = Card(
    id="EmptyBody", name="Empty Body", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, base_block=7, upgrade_block=5, exit_stance=True,
)

EMPTY_MIND = Card(
    id="EmptyMind", name="Empty Mind", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
    exit_stance=True, effects=["draw_cards"],
)

EVALUATE = Card(
    id="Evaluate", name="Evaluate", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, base_magic=6, upgrade_magic=4,
    effects=["add_insight_to_draw"],  # Put Insight on top of draw pile
)

INNER_PEACE = Card(
    id="InnerPeace", name="Inner Peace", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1,
    effects=["if_calm_draw_3_else_calm"],
    upgrade_magic=1,  # Draw 4 when upgraded
)

PROTECT = Card(
    id="Protect", name="Protect", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=2, base_block=12, upgrade_block=4, retain=True,
)

THIRD_EYE = Card(
    id="ThirdEye", name="Third Eye", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, base_block=7, upgrade_block=2,
    base_magic=3, upgrade_magic=2, effects=["scry"],
)

PROSTRATE = Card(
    id="Prostrate", name="Prostrate", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=0, base_block=4, upgrade_block=2,
    base_magic=2, upgrade_magic=1, effects=["gain_mantra"],
)


# === UNCOMMON ATTACKS ===

TANTRUM = Card(
    id="Tantrum", name="Tantrum", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=1, base_damage=3, base_magic=3, upgrade_magic=1,
    shuffle_back=True, enter_stance="Wrath",
    effects=["damage_x_times"],
)

FEAR_NO_EVIL = Card(
    id="FearNoEvil", name="Fear No Evil", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=1, base_damage=8, upgrade_damage=3,
    effects=["if_enemy_attacking_enter_calm"],
)

REACH_HEAVEN = Card(
    id="ReachHeaven", name="Reach Heaven", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=2, base_damage=10, upgrade_damage=5,
    effects=["add_through_violence_to_draw"],
)

SANDS_OF_TIME = Card(
    id="SandsOfTime", name="Sands of Time", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=4, base_damage=20, upgrade_damage=6, retain=True,
    effects=["cost_reduces_each_turn"],  # -1 cost at end of turn while retained
)

SIGNATURE_MOVE = Card(
    id="SignatureMove", name="Signature Move", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=2, base_damage=30, upgrade_damage=10,
    effects=["only_attack_in_hand"],  # Only usable if this is only attack in hand
)

TALK_TO_THE_HAND = Card(
    id="TalkToTheHand", name="Talk to the Hand", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=1, base_damage=5, upgrade_damage=2,
    base_magic=2, upgrade_magic=1, exhaust=True,
    effects=["apply_block_return"],  # Target gives you block when attacking
)

WALLOP = Card(
    id="Wallop", name="Wallop", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=2, base_damage=9, upgrade_damage=5,
    effects=["gain_block_equal_unblocked_damage"],
)

WEAVE = Card(
    id="Weave", name="Weave", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=0, base_damage=4, upgrade_damage=2,
    effects=["on_scry_play_from_discard"],
)

WHEEL_KICK = Card(
    id="WheelKick", name="Wheel Kick", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=2, base_damage=15, upgrade_damage=5,
    effects=["draw_2"],
)

WINDMILL_STRIKE = Card(
    id="WindmillStrike", name="Windmill Strike", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=2, base_damage=7, upgrade_damage=3, retain=True,
    effects=["gain_damage_when_retained_4"],  # +4 damage each turn retained
)

CONCLUDE = Card(
    id="Conclude", name="Conclude", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    target=CardTarget.ALL_ENEMY, cost=1, base_damage=12, upgrade_damage=4,
    effects=["end_turn"],  # Ends your turn after playing
)

CARVE_REALITY = Card(
    id="CarveReality", name="Carve Reality", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    cost=1, base_damage=6, upgrade_damage=4,
    effects=["add_smite_to_hand"],
)


# === UNCOMMON SKILLS ===

COLLECT = Card(
    id="Collect", name="Collect", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=0, base_magic=1, upgrade_magic=1,  # X cost
    exhaust=True, effects=["put_x_miracles_on_draw"],
)

DECEIVE_REALITY = Card(
    id="DeceiveReality", name="Deceive Reality", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_block=4, upgrade_block=3,
    effects=["add_safety_to_hand"],
)

FORESIGHT = Card(
    id="Wireheading", name="Foresight", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    # Java cardID is "Wireheading", not "Foresight"
    target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=1,
    effects=["scry_each_turn"],
)

INDIGNATION = Card(
    id="Indignation", name="Indignation", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=2,
    effects=["if_wrath_gain_mantra_else_wrath"],
)

MEDITATE = Card(
    id="Meditate", name="Meditate", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_magic=1,
    effects=["put_cards_from_discard_to_hand", "enter_calm", "end_turn"],
)

PERSEVERANCE = Card(
    id="Perseverance", name="Perseverance", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=2, retain=True,
    effects=["gains_block_when_retained"],  # +2/+3 block when retained
)

PRAY = Card(
    id="Pray", name="Pray", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=1,
    effects=["gain_mantra_add_insight"],  # Gain 3 Mantra, shuffle Insight into draw pile
)

SANCTITY = Card(
    id="Sanctity", name="Sanctity", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_block=6, upgrade_block=3,
    effects=["if_last_skill_draw_2"],
)

SWIVEL = Card(
    id="Swivel", name="Swivel", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=2, base_block=8, upgrade_block=3,
    effects=["free_attack_next_turn"],  # Next attack costs 0
)

WAVE_OF_THE_HAND = Card(
    id="WaveOfTheHand", name="Wave of the Hand", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_magic=1,
    effects=["block_gain_applies_weak"],  # When you gain block, apply weak
)

# Note: SimmeringFury's internal ID is "Vengeance" in the game
SIMMERING_FURY = Card(
    id="Vengeance", name="Simmering Fury", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.NONE, cost=1, base_magic=2, upgrade_magic=1,
    effects=["wrath_next_turn_draw_next_turn"],  # Next turn enter Wrath and draw 2 cards
)

WORSHIP = Card(
    id="Worship", name="Worship", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=2, base_magic=5, upgrade_magic=3, retain=True,
    effects=["gain_mantra"],
)

WREATH_OF_FLAME = Card(
    id="WreathOfFlame", name="Wreath of Flame", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=5, upgrade_magic=3,
    effects=["next_attack_plus_damage"],
)


# === UNCOMMON POWERS ===

BATTLE_HYMN = Card(
    id="BattleHymn", name="Battle Hymn", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_magic=1,
    effects=["add_smite_each_turn"],
)

ESTABLISHMENT = Card(
    id="Establishment", name="Establishment", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_magic=0,
    upgrade_cost=0, effects=["retained_cards_cost_less"],
)

LIKE_WATER = Card(
    id="LikeWater", name="Like Water", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=5, upgrade_magic=2,
    effects=["if_calm_end_turn_gain_block"],
)

MENTAL_FORTRESS = Card(
    id="MentalFortress", name="Mental Fortress", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=4, upgrade_magic=2,
    effects=["on_stance_change_gain_block"],
)

NIRVANA = Card(
    id="Nirvana", name="Nirvana", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=1,
    effects=["on_scry_gain_block"],
)

RUSHDOWN = Card(
    id="Adaptation", name="Rushdown", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    # Java cardID is "Adaptation", not "Rushdown"
    target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
    effects=["on_wrath_draw"],
)

STUDY = Card(
    id="Study", name="Study", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=2, upgrade_cost=1,
    effects=["add_insight_end_turn"],
)


# === RARE ATTACKS ===

BRILLIANCE = Card(
    id="Brilliance", name="Brilliance", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    cost=1, base_damage=12, upgrade_damage=4,
    effects=["damage_plus_mantra_gained"],
)

CONCLUDE_PLUS = Card(  # Already defined above
    id="Conclude+", name="Conclude+", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    target=CardTarget.ALL_ENEMY, cost=1, base_damage=16, upgraded=True,
    effects=["end_turn"],
)

JUDGMENT = Card(
    id="Judgement", name="Judgement", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    cost=1, base_magic=30, upgrade_magic=10,
    effects=["if_enemy_hp_below_kill"],  # Kill if HP < magic_number
)

LESSON_LEARNED = Card(
    id="LessonLearned", name="Lesson Learned", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    cost=2, base_damage=10, upgrade_damage=3, exhaust=True,
    effects=["if_fatal_upgrade_random_card"],
)

RAGNAROK = Card(
    id="Ragnarok", name="Ragnarok", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    target=CardTarget.ALL_ENEMY, cost=3, base_damage=5, base_magic=5,
    upgrade_damage=1, upgrade_magic=1,
    effects=["damage_random_x_times"],  # 5x5 (6x6 upgraded) to random enemies
)


# === RARE SKILLS ===

ALPHA = Card(
    id="Alpha", name="Alpha", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, upgrade_cost=0, innate=True,
    exhaust=True, effects=["shuffle_beta_into_draw"],
)

BLASPHEMY = Card(
    id="Blasphemy", name="Blasphemy", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, upgrade_cost=0, retain=True,
    effects=["enter_divinity", "die_next_turn"],
)

CONJURE_BLADE = Card(
    id="ConjureBlade", name="Conjure Blade", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=0,  # X cost
    effects=["add_expunger_to_hand"],  # Expunger does Xx9 damage
)

FOREIGN_INFLUENCE = Card(
    id="ForeignInfluence", name="Foreign Influence", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=0, exhaust=True,
    effects=["choose_attack_from_any_class"],
    upgrade_magic=1,  # Choose 2 when upgraded
)

OMNISCIENCE = Card(
    id="Omniscience", name="Omniscience", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=4, upgrade_cost=3, exhaust=True,
    effects=["play_card_from_draw_twice"],
)

SCRAWL = Card(
    id="Scrawl", name="Scrawl", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, upgrade_cost=0, exhaust=True,
    effects=["draw_until_hand_full"],  # Draw until hand = 10
)

SPIRIT_SHIELD = Card(
    id="SpiritShield", name="Spirit Shield", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=2, base_magic=3, upgrade_magic=1,
    effects=["gain_block_per_card_in_hand"],  # 3/4 block per card
)

VAULT = Card(
    id="Vault", name="Vault", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=3, upgrade_cost=2, exhaust=True,
    effects=["take_extra_turn"],
)

WISH = Card(
    id="Wish", name="Wish", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=3, exhaust=True,
    effects=["choose_plated_armor_or_strength_or_gold"],
    upgrade_magic=1,  # Higher values when upgraded
)


# === RARE POWERS ===

DEVA_FORM = Card(
    id="DevaForm", name="Deva Form", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=3, ethereal=True,
    effects=["gain_energy_each_turn_stacking"],
)

DEVOTION = Card(
    id="Devotion", name="Devotion", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
    effects=["gain_mantra_each_turn"],
)

FASTING = Card(
    id="Fasting2", name="Fasting", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    # Java cardID is "Fasting2", not "Fasting"
    target=CardTarget.SELF, cost=2, base_magic=3, upgrade_magic=1,
    effects=["gain_strength_and_dex_lose_focus"],  # Doesn't have focus for watcher
)

MASTER_REALITY = Card(
    id="MasterReality", name="Master Reality", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, upgrade_cost=0,
    effects=["created_cards_upgraded"],
)


# === SPECIAL CARDS (Generated during combat) ===

INSIGHT = Card(
    id="Insight", name="Insight", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF,
    cost=0, retain=True, exhaust=True,
    base_magic=2, upgrade_magic=1, effects=["draw_cards"],
)

SMITE = Card(
    id="Smite", name="Smite", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, cost=1, base_damage=12, upgrade_damage=4,
    retain=True, exhaust=True,
)

SAFETY = Card(
    id="Safety", name="Safety", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF,
    cost=1, base_block=12, upgrade_block=4, retain=True, exhaust=True,
)

THROUGH_VIOLENCE = Card(
    id="ThroughViolence", name="Through Violence", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, cost=0, base_damage=20, upgrade_damage=10,
    retain=True, exhaust=True,
)

EXPUNGER = Card(
    id="Expunger", name="Expunger", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, cost=1, base_damage=9, base_magic=1,
    effects=["hits_x_times"],  # Hits = X from Conjure Blade
)

BETA = Card(
    id="Beta", name="Beta", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=2, upgrade_cost=1,
    exhaust=True, effects=["shuffle_omega_into_draw"],
)

OMEGA = Card(
    id="Omega", name="Omega", card_type=CardType.POWER, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=3,
    effects=["deal_50_damage_end_turn"],
)


# ============ COLORLESS CARDS ============

# === UNCOMMON COLORLESS ===

BANDAGE_UP = Card(
    id="Bandage Up", name="Bandage Up", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=4, upgrade_magic=2, exhaust=True,
    effects=["heal_magic_number"],
)

BLIND = Card(
    id="Blind", name="Blind", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=0,
    base_magic=2,  # Weak amount
    effects=["apply_weak"],
    # Upgraded: targets ALL enemies instead
)

DARK_SHACKLES = Card(
    id="Dark Shackles", name="Dark Shackles", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=0,
    base_magic=9, upgrade_magic=6, exhaust=True,
    effects=["apply_temp_strength_down"],  # -9/-15 strength this turn only
)

DEEP_BREATH = Card(
    id="Deep Breath", name="Deep Breath", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=1, upgrade_magic=1,
    effects=["shuffle_discard_into_draw", "draw_cards"],
)

DISCOVERY = Card(
    id="Discovery", name="Discovery", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=1,
    exhaust=True,  # Upgraded: no exhaust
    effects=["discover_card"],
)

DRAMATIC_ENTRANCE = Card(
    id="Dramatic Entrance", name="Dramatic Entrance", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ALL_ENEMY, cost=0,
    base_damage=8, upgrade_damage=4, innate=True, exhaust=True,
)

ENLIGHTENMENT = Card(
    id="Enlightenment", name="Enlightenment", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    effects=["reduce_hand_cost_to_1"],  # Upgraded: permanent for combat
)

FINESSE = Card(
    id="Finesse", name="Finesse", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_block=2, upgrade_block=2,
    effects=["draw_1"],
)

FLASH_OF_STEEL = Card(
    id="Flash of Steel", name="Flash of Steel", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=0,
    base_damage=3, upgrade_damage=3,
    effects=["draw_1"],
)

FORETHOUGHT = Card(
    id="Forethought", name="Forethought", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    effects=["put_card_on_bottom_of_draw_cost_0"],  # Upgraded: any number of cards
)

GOOD_INSTINCTS = Card(
    id="Good Instincts", name="Good Instincts", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_block=6, upgrade_block=3,
)

IMPATIENCE = Card(
    id="Impatience", name="Impatience", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    base_magic=2, upgrade_magic=1,
    effects=["draw_if_no_attacks_in_hand"],
)

JACK_OF_ALL_TRADES = Card(
    id="Jack Of All Trades", name="Jack of All Trades", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    base_magic=1, upgrade_magic=1, exhaust=True,
    effects=["add_random_colorless_to_hand"],
)

MADNESS = Card(
    id="Madness", name="Madness", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=1,
    upgrade_cost=0, exhaust=True,
    effects=["reduce_random_card_cost_to_0"],
)

MIND_BLAST = Card(
    id="Mind Blast", name="Mind Blast", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=2,
    upgrade_cost=1, base_damage=0, innate=True,
    effects=["damage_equals_draw_pile_size"],
)

PANACEA = Card(
    id="Panacea", name="Panacea", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=1, upgrade_magic=1, exhaust=True,
    effects=["gain_artifact"],
)

PANIC_BUTTON = Card(
    id="PanicButton", name="Panic Button", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_block=30, upgrade_block=10, base_magic=2, exhaust=True,
    effects=["gain_no_block_next_2_turns"],
)

PURITY = Card(
    id="Purity", name="Purity", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    base_magic=3, upgrade_magic=2, exhaust=True,
    effects=["exhaust_up_to_x_cards"],
)

SWIFT_STRIKE = Card(
    id="Swift Strike", name="Swift Strike", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=0,
    base_damage=7, upgrade_damage=3,
)

TRIP = Card(
    id="Trip", name="Trip", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=0,
    base_magic=2,  # Vulnerable amount
    effects=["apply_vulnerable"],
    # Upgraded: targets ALL enemies instead
)


# === RARE COLORLESS ===

APOTHEOSIS = Card(
    id="Apotheosis", name="Apotheosis", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=2,
    upgrade_cost=1, exhaust=True,
    effects=["upgrade_all_cards_in_combat"],
)

CHRYSALIS = Card(
    id="Chrysalis", name="Chrysalis", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=2,
    base_magic=3, upgrade_magic=2, exhaust=True,
    effects=["add_random_skills_to_draw_cost_0"],  # Adds 3/5 random skills with cost 0
)

HAND_OF_GREED = Card(
    id="HandOfGreed", name="Hand of Greed", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=2,
    base_damage=20, upgrade_damage=5, base_magic=20, upgrade_magic=5,
    effects=["if_fatal_gain_gold"],  # Gain gold on kill
)

MAGNETISM = Card(
    id="Magnetism", name="Magnetism", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=2,
    upgrade_cost=1, base_magic=1,
    effects=["add_random_colorless_each_turn"],
)

MASTER_OF_STRATEGY = Card(
    id="Master of Strategy", name="Master of Strategy", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    base_magic=3, upgrade_magic=1, exhaust=True,
    effects=["draw_cards"],
)

MAYHEM = Card(
    id="Mayhem", name="Mayhem", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=2,
    upgrade_cost=1, base_magic=1,
    effects=["auto_play_top_card_each_turn"],
)

METAMORPHOSIS = Card(
    id="Metamorphosis", name="Metamorphosis", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=2,
    base_magic=3, upgrade_magic=2, exhaust=True,
    effects=["add_random_attacks_to_draw_cost_0"],  # Adds 3/5 random attacks with cost 0
)

PANACHE = Card(
    id="Panache", name="Panache", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=10, upgrade_magic=4,
    effects=["every_5_cards_deal_damage_to_all"],  # Every 5 cards, deal 10/14 damage to all
)

SADISTIC_NATURE = Card(
    id="Sadistic Nature", name="Sadistic Nature", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=5, upgrade_magic=2,
    effects=["on_debuff_deal_damage"],  # Deal 5/7 damage when applying debuff
)

SECRET_TECHNIQUE = Card(
    id="Secret Technique", name="Secret Technique", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    exhaust=True,  # Upgraded: no exhaust
    effects=["search_draw_for_skill"],
)

SECRET_WEAPON = Card(
    id="Secret Weapon", name="Secret Weapon", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    exhaust=True,  # Upgraded: no exhaust
    effects=["search_draw_for_attack"],
)

THE_BOMB = Card(
    id="The Bomb", name="The Bomb", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=2,
    base_magic=40, upgrade_magic=10,
    effects=["deal_damage_to_all_after_3_turns"],
)

THINKING_AHEAD = Card(
    id="Thinking Ahead", name="Thinking Ahead", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    exhaust=True,  # Upgraded: no exhaust
    effects=["draw_2_put_1_on_top_of_draw"],
)

TRANSMUTATION = Card(
    id="Transmutation", name="Transmutation", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=-1,  # X cost
    exhaust=True,
    effects=["add_x_random_colorless_cost_0"],  # Upgraded: cards are upgraded
)

VIOLENCE = Card(
    id="Violence", name="Violence", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=0,
    base_magic=3, upgrade_magic=1, exhaust=True,
    effects=["put_attacks_from_draw_into_hand"],
)


# === SPECIAL COLORLESS (Event/Relic rewards) ===

APPARITION = Card(
    id="Ghostly", name="Apparition", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=1,
    exhaust=True, ethereal=True,  # Upgraded: not ethereal
    effects=["gain_intangible_1"],
)

BITE = Card(
    id="Bite", name="Bite", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=1,
    base_damage=7, upgrade_damage=1, base_magic=2, upgrade_magic=1,
    effects=["heal_magic_number"],
)

JAX = Card(
    id="J.A.X.", name="J.A.X.", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=1,
    effects=["lose_3_hp_gain_strength"],
)

RITUAL_DAGGER = Card(
    id="RitualDagger", name="Ritual Dagger", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, target=CardTarget.ENEMY, cost=1,
    base_damage=15, base_magic=3, upgrade_magic=2, exhaust=True,
    effects=["if_fatal_permanently_increase_damage"],  # +3/+5 damage permanently on kill
)


# ============ CURSE CARDS ============

ASCENDERS_BANE = Card(
    id="AscendersBane", name="Ascender's Bane", card_type=CardType.CURSE, rarity=CardRarity.SPECIAL,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    ethereal=True,
    effects=["unplayable", "cannot_be_removed"],
)

CLUMSY = Card(
    id="Clumsy", name="Clumsy", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    ethereal=True,
    effects=["unplayable"],
)

CURSE_OF_THE_BELL = Card(
    id="CurseOfTheBell", name="Curse of the Bell", card_type=CardType.CURSE, rarity=CardRarity.SPECIAL,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "cannot_be_removed"],
)

DECAY = Card(
    id="Decay", name="Decay", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "end_of_turn_take_2_damage"],
)

DOUBT = Card(
    id="Doubt", name="Doubt", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "end_of_turn_gain_weak_1"],
)

INJURY = Card(
    id="Injury", name="Injury", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable"],
)

NECRONOMICURSE = Card(
    id="Necronomicurse", name="Necronomicurse", card_type=CardType.CURSE, rarity=CardRarity.SPECIAL,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "returns_when_exhausted_or_removed"],
)

NORMALITY = Card(
    id="Normality", name="Normality", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "limit_3_cards_per_turn"],
)

PAIN = Card(
    id="Pain", name="Pain", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "lose_1_hp_when_other_card_played"],
)

PARASITE = Card(
    id="Parasite", name="Parasite", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "lose_3_max_hp_when_removed"],
)

PRIDE = Card(
    id="Pride", name="Pride", card_type=CardType.CURSE, rarity=CardRarity.SPECIAL,
    color=CardColor.CURSE, target=CardTarget.SELF, cost=1,
    innate=True, exhaust=True,
    effects=["end_of_turn_add_copy_to_draw"],
)

REGRET = Card(
    id="Regret", name="Regret", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "end_of_turn_lose_hp_equal_to_hand_size"],
)

SHAME = Card(
    id="Shame", name="Shame", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable", "end_of_turn_gain_frail_1"],
)

WRITHE = Card(
    id="Writhe", name="Writhe", card_type=CardType.CURSE, rarity=CardRarity.CURSE,
    color=CardColor.CURSE, target=CardTarget.NONE, cost=-2,  # Unplayable
    innate=True,
    effects=["unplayable"],
)


# ============ STATUS CARDS ============

BURN = Card(
    id="Burn", name="Burn", card_type=CardType.STATUS, rarity=CardRarity.COMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=-2,  # Unplayable
    base_magic=2, upgrade_magic=2,  # Burn+ deals 4 damage
    effects=["unplayable", "end_of_turn_take_damage"],
)

DAZED = Card(
    id="Dazed", name="Dazed", card_type=CardType.STATUS, rarity=CardRarity.COMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=-2,  # Unplayable
    ethereal=True,
    effects=["unplayable"],
)

SLIMED = Card(
    id="Slimed", name="Slimed", card_type=CardType.STATUS, rarity=CardRarity.COMMON,
    color=CardColor.COLORLESS, target=CardTarget.SELF, cost=1,
    exhaust=True,
    effects=[],  # Does nothing when played, just costs energy
)

VOID = Card(
    id="Void", name="Void", card_type=CardType.STATUS, rarity=CardRarity.COMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=-2,  # Unplayable
    ethereal=True,
    effects=["unplayable", "lose_1_energy_when_drawn"],
)

WOUND = Card(
    id="Wound", name="Wound", card_type=CardType.STATUS, rarity=CardRarity.COMMON,
    color=CardColor.COLORLESS, target=CardTarget.NONE, cost=-2,  # Unplayable
    effects=["unplayable"],
)


# ============ CARD REGISTRY ============

# All Watcher cards by ID
WATCHER_CARDS: Dict[str, Card] = {
    # Basic
    "Strike_P": STRIKE_W,
    "Defend_P": DEFEND_W,
    "Eruption": ERUPTION,
    "Vigilance": VIGILANCE,
    "Miracle": MIRACLE,
    # Common Attacks
    "BowlingBash": BOWLING_BASH,
    "CutThroughFate": CUT_THROUGH_FATE,
    "EmptyFist": EMPTY_FIST,
    "FlurryOfBlows": FLURRY_OF_BLOWS,
    "FlyingSleeves": FLYING_SLEEVES,
    "FollowUp": FOLLOW_UP,
    "Halt": HALT,
    "JustLucky": JUST_LUCKY,
    "PathToVictory": PRESSURE_POINTS,  # Java ID for PressurePoints
    "SashWhip": SASH_WHIP,
    "ClearTheMind": TRANQUILITY,  # Java ID for Tranquility
    "Crescendo": CRESCENDO,
    "Consecrate": CONSECRATE,
    "CrushJoints": CRUSH_JOINTS,
    # Common Skills
    "EmptyBody": EMPTY_BODY,
    "EmptyMind": EMPTY_MIND,
    "Evaluate": EVALUATE,
    "InnerPeace": INNER_PEACE,
    "Protect": PROTECT,
    "ThirdEye": THIRD_EYE,
    "Prostrate": PROSTRATE,
    # Uncommon Attacks
    "Tantrum": TANTRUM,
    "FearNoEvil": FEAR_NO_EVIL,
    "ReachHeaven": REACH_HEAVEN,
    "SandsOfTime": SANDS_OF_TIME,
    "SignatureMove": SIGNATURE_MOVE,
    "TalkToTheHand": TALK_TO_THE_HAND,
    "Wallop": WALLOP,
    "Weave": WEAVE,
    "WheelKick": WHEEL_KICK,
    "WindmillStrike": WINDMILL_STRIKE,
    "Conclude": CONCLUDE,
    "CarveReality": CARVE_REALITY,
    # Uncommon Skills
    "Collect": COLLECT,
    "DeceiveReality": DECEIVE_REALITY,
    "Wireheading": FORESIGHT,  # Java ID for Foresight
    "Indignation": INDIGNATION,
    "Meditate": MEDITATE,
    "Perseverance": PERSEVERANCE,
    "Pray": PRAY,
    "Sanctity": SANCTITY,
    "Swivel": SWIVEL,
    "Vengeance": SIMMERING_FURY,  # SimmeringFury has ID "Vengeance" in game
    "WaveOfTheHand": WAVE_OF_THE_HAND,
    "Worship": WORSHIP,
    "WreathOfFlame": WREATH_OF_FLAME,
    # Uncommon Powers
    "BattleHymn": BATTLE_HYMN,
    "Establishment": ESTABLISHMENT,
    "LikeWater": LIKE_WATER,
    "MentalFortress": MENTAL_FORTRESS,
    "Nirvana": NIRVANA,
    "Adaptation": RUSHDOWN,  # Java ID for Rushdown
    "Study": STUDY,
    # Rare Attacks
    "Brilliance": BRILLIANCE,
    "Judgement": JUDGMENT,
    "LessonLearned": LESSON_LEARNED,
    "Ragnarok": RAGNAROK,
    # Rare Skills
    "Alpha": ALPHA,
    "Blasphemy": BLASPHEMY,
    "ConjureBlade": CONJURE_BLADE,
    "ForeignInfluence": FOREIGN_INFLUENCE,
    "Omniscience": OMNISCIENCE,
    "Scrawl": SCRAWL,
    "SpiritShield": SPIRIT_SHIELD,
    "Vault": VAULT,
    "Wish": WISH,
    # Rare Powers
    "DevaForm": DEVA_FORM,
    "Devotion": DEVOTION,
    "Fasting2": FASTING,  # Java ID is "Fasting2", not "Fasting"
    "MasterReality": MASTER_REALITY,
    # Special
    "Insight": INSIGHT,
    "Smite": SMITE,
    "Safety": SAFETY,
    "ThroughViolence": THROUGH_VIOLENCE,
    "Expunger": EXPUNGER,
    "Beta": BETA,
    "Omega": OMEGA,
}


# Colorless cards by ID
COLORLESS_CARDS: Dict[str, Card] = {
    # Uncommon
    "Bandage Up": BANDAGE_UP,
    "Blind": BLIND,
    "Dark Shackles": DARK_SHACKLES,
    "Deep Breath": DEEP_BREATH,
    "Discovery": DISCOVERY,
    "Dramatic Entrance": DRAMATIC_ENTRANCE,
    "Enlightenment": ENLIGHTENMENT,
    "Finesse": FINESSE,
    "Flash of Steel": FLASH_OF_STEEL,
    "Forethought": FORETHOUGHT,
    "Good Instincts": GOOD_INSTINCTS,
    "Impatience": IMPATIENCE,
    "Jack Of All Trades": JACK_OF_ALL_TRADES,
    "Madness": MADNESS,
    "Mind Blast": MIND_BLAST,
    "Panacea": PANACEA,
    "PanicButton": PANIC_BUTTON,
    "Purity": PURITY,
    "Swift Strike": SWIFT_STRIKE,
    "Trip": TRIP,
    # Rare
    "Apotheosis": APOTHEOSIS,
    "Chrysalis": CHRYSALIS,
    "HandOfGreed": HAND_OF_GREED,
    "Magnetism": MAGNETISM,
    "Master of Strategy": MASTER_OF_STRATEGY,
    "Mayhem": MAYHEM,
    "Metamorphosis": METAMORPHOSIS,
    "Panache": PANACHE,
    "Sadistic Nature": SADISTIC_NATURE,
    "Secret Technique": SECRET_TECHNIQUE,
    "Secret Weapon": SECRET_WEAPON,
    "The Bomb": THE_BOMB,
    "Thinking Ahead": THINKING_AHEAD,
    "Transmutation": TRANSMUTATION,
    "Violence": VIOLENCE,
    # Special (Event/Relic rewards)
    "Ghostly": APPARITION,
    "Bite": BITE,
    "J.A.X.": JAX,
    "RitualDagger": RITUAL_DAGGER,
}


# Curse cards by ID
CURSE_CARDS: Dict[str, Card] = {
    "AscendersBane": ASCENDERS_BANE,
    "Clumsy": CLUMSY,
    "CurseOfTheBell": CURSE_OF_THE_BELL,
    "Decay": DECAY,
    "Doubt": DOUBT,
    "Injury": INJURY,
    "Necronomicurse": NECRONOMICURSE,
    "Normality": NORMALITY,
    "Pain": PAIN,
    "Parasite": PARASITE,
    "Pride": PRIDE,
    "Regret": REGRET,
    "Shame": SHAME,
    "Writhe": WRITHE,
}


# Status cards by ID
STATUS_CARDS: Dict[str, Card] = {
    "Burn": BURN,
    "Dazed": DAZED,
    "Slimed": SLIMED,
    "Void": VOID,
    "Wound": WOUND,
}


# All cards combined
ALL_CARDS: Dict[str, Card] = {
    **WATCHER_CARDS,
    **COLORLESS_CARDS,
    **CURSE_CARDS,
    **STATUS_CARDS,
}


def get_card(card_id: str, upgraded: bool = False) -> Card:
    """Get a copy of a card by ID."""
    if card_id not in ALL_CARDS:
        raise ValueError(f"Unknown card: {card_id}")
    card = ALL_CARDS[card_id].copy()
    if upgraded:
        card.upgrade()
    return card


def get_starting_deck() -> List[Card]:
    """Get Watcher's starting deck."""
    deck = []
    # 4x Strike
    for _ in range(4):
        deck.append(get_card("Strike_P"))
    # 4x Defend
    for _ in range(4):
        deck.append(get_card("Defend_P"))
    # 1x Eruption
    deck.append(get_card("Eruption"))
    # 1x Vigilance
    deck.append(get_card("Vigilance"))
    return deck


# ============ TESTING ============

if __name__ == "__main__":
    print("=== Watcher Card Tests ===\n")

    # Test card lookup
    eruption = get_card("Eruption")
    print(f"Eruption: Cost {eruption.cost}, Damage {eruption.damage}")
    print(f"  Enters: {eruption.enter_stance}")

    eruption.upgrade()
    print(f"Eruption+: Cost {eruption.current_cost}, Damage {eruption.damage}")

    # Test starting deck
    print(f"\nStarting deck ({len(get_starting_deck())} cards):")
    deck = get_starting_deck()
    card_counts = {}
    for c in deck:
        card_counts[c.name] = card_counts.get(c.name, 0) + 1
    for name, count in card_counts.items():
        print(f"  {count}x {name}")

    # Test Tantrum
    tantrum = get_card("Tantrum")
    print(f"\nTantrum: {tantrum.damage} damage x{tantrum.magic_number} hits")
    tantrum.upgrade()
    print(f"Tantrum+: {tantrum.damage} damage x{tantrum.magic_number} hits")
