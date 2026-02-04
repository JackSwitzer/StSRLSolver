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

    # Upgrade-time flag changes (None = no change on upgrade)
    upgrade_retain: Optional[bool] = None
    upgrade_innate: Optional[bool] = None
    upgrade_exhaust: Optional[bool] = None
    upgrade_ethereal: Optional[bool] = None

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
            upgrade_innate=self.upgrade_innate, upgrade_retain=self.upgrade_retain,
            upgrade_ethereal=self.upgrade_ethereal, upgrade_exhaust=self.upgrade_exhaust,
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
    base_magic=2, upgrade_magic=1,  # Scry amount (2 base, 3 upgraded)
    effects=["scry", "draw_1"],
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
    cost=1, base_damage=4, upgrade_damage=2,
    retain=True,
    effects=["damage_twice"],  # Hits twice (hardcoded)
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
    base_block=2, upgrade_block=1,  # Also gains block
    base_magic=1, upgrade_magic=1,  # Scry amount
    effects=["scry", "gain_block"],  # Scry, Block, Damage
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
    base_magic=1, upgrade_magic=1,  # Weak amount
    effects=["if_last_card_attack_weak"],
)

TRANQUILITY = Card(
    id="ClearTheMind", name="Tranquility", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    # Java cardID is "ClearTheMind", not "Tranquility"
    target=CardTarget.SELF, cost=1, upgrade_cost=0, retain=True, exhaust=True,
    enter_stance="Calm",
)

CRESCENDO = Card(
    id="Crescendo", name="Crescendo", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, upgrade_cost=0, retain=True, exhaust=True,
    enter_stance="Wrath",  # Upgraded: not exhaust
)

CONSECRATE = Card(
    id="Consecrate", name="Consecrate", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    target=CardTarget.ALL_ENEMY, cost=0, base_damage=5, upgrade_damage=3,
)

CRUSH_JOINTS = Card(
    id="CrushJoints", name="Crush Joints", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    cost=1, base_damage=8, upgrade_damage=2,
    base_magic=1, upgrade_magic=1,  # Vulnerable amount
    effects=["if_last_card_skill_vulnerable"],
)


# === COMMON SKILLS ===

EMPTY_BODY = Card(
    id="EmptyBody", name="Empty Body", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, base_block=7, upgrade_block=3, exit_stance=True,
)

EMPTY_MIND = Card(
    id="EmptyMind", name="Empty Mind", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
    exit_stance=True, effects=["draw_cards"],
)

EVALUATE = Card(
    id="Evaluate", name="Evaluate", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    target=CardTarget.SELF, cost=1, base_block=6, upgrade_block=4,
    effects=["add_insight_to_draw"],  # Put Insight on top of draw pile
)

INNER_PEACE = Card(
    id="InnerPeace", name="Inner Peace", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1,
    base_magic=3, upgrade_magic=1,  # Draw amount (3 base, 4 upgraded)
    effects=["if_calm_draw_else_calm"],
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
    target=CardTarget.SELF, cost=0, base_block=4,
    base_magic=2, upgrade_magic=1, effects=["gain_mantra"],  # Mantra amount
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
    cost=2, base_damage=9, upgrade_damage=3,
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
    target=CardTarget.SELF, cost=-1,  # X cost
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
    target=CardTarget.NONE, cost=1, base_magic=3, upgrade_magic=1,
    effects=["scry_each_turn"],
)

INDIGNATION = Card(
    id="Indignation", name="Indignation", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.NONE, cost=1, base_magic=3, upgrade_magic=2,
    effects=["if_wrath_gain_mantra_else_wrath"],
)

MEDITATE = Card(
    id="Meditate", name="Meditate", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.NONE, cost=1, base_magic=1, upgrade_magic=1,
    enter_stance="Calm",  # Critical: enters Calm stance
    effects=["put_cards_from_discard_to_hand", "enter_calm", "end_turn"],
)

PERSEVERANCE = Card(
    id="Perseverance", name="Perseverance", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=2, base_magic=2, upgrade_magic=1, retain=True,
    effects=["gains_block_when_retained"],  # +2/+3 block when retained
)

PRAY = Card(
    id="Pray", name="Pray", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=1,
    effects=["gain_mantra_add_insight"],  # Gain 3 Mantra, shuffle Insight into draw pile
)

SANCTITY = Card(
    id="Sanctity", name="Sanctity", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=1, base_block=6, upgrade_block=3, base_magic=2,
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
    target=CardTarget.SELF, cost=2, base_magic=5, upgrade_magic=0, retain=False,
    upgrade_retain=True,
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
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_innate=True,
    effects=["add_smite_each_turn"],  # Upgraded: becomes Innate
)

ESTABLISHMENT = Card(
    id="Establishment", name="Establishment", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, base_magic=1, upgrade_innate=True,
    effects=["retained_cards_cost_less"],  # Upgraded: becomes Innate
)

LIKE_WATER = Card(
    id="LikeWater", name="Like Water", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.NONE, cost=1, base_magic=5, upgrade_magic=2,
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
    target=CardTarget.SELF, cost=1, upgrade_cost=0,
    base_magic=2,  # Draw 2 when entering Wrath
    effects=["on_wrath_draw"],
)

STUDY = Card(
    id="Study", name="Study", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    target=CardTarget.SELF, cost=2, upgrade_cost=1,
    base_magic=1,  # Shuffles 1 Insight at end of turn
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
    id="Judgement", name="Judgement", card_type=CardType.SKILL, rarity=CardRarity.RARE,
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

DEUS_EX_MACHINA = Card(
    id="DeusExMachina", name="Deus Ex Machina", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=-2, base_magic=2, upgrade_magic=1,
    exhaust=True,
    effects=["on_draw_add_miracles_and_exhaust"],  # When drawn: add 2 Miracles, exhaust
)

ALPHA = Card(
    id="Alpha", name="Alpha", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=1, innate=False,
    upgrade_innate=True,
    exhaust=True, effects=["shuffle_beta_into_draw"],
)

BLASPHEMY = Card(
    id="Blasphemy", name="Blasphemy", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=1, retain=False, exhaust=True,
    upgrade_retain=True,
    enter_stance="Divinity",  # Critical: enters Divinity stance
    effects=["enter_divinity", "die_next_turn"],
)

CONJURE_BLADE = Card(
    id="ConjureBlade", name="Conjure Blade", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=-1, exhaust=True,  # X cost
    effects=["add_expunger_to_hand"],  # Expunger does Xx9 damage
)

FOREIGN_INFLUENCE = Card(
    id="ForeignInfluence", name="Foreign Influence", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    target=CardTarget.NONE, cost=0, exhaust=True,
    effects=["choose_attack_from_any_class"],
)

OMNISCIENCE = Card(
    id="Omniscience", name="Omniscience", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=4, upgrade_cost=3, exhaust=True,
    effects=["play_card_from_draw_twice"],
)

SCRAWL = Card(
    id="Scrawl", name="Scrawl", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=1, upgrade_cost=0, exhaust=True,
    effects=["draw_until_hand_full"],  # Draw until hand = 10
)

SPIRIT_SHIELD = Card(
    id="SpiritShield", name="Spirit Shield", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=2, base_magic=3, upgrade_magic=1,
    effects=["gain_block_per_card_in_hand"],  # 3/4 block per card
)

UNRAVELING = Card(
    id="Unraveling", name="Unraveling", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=2, upgrade_cost=1, exhaust=True,
    effects=["scry_draw_pile_discard_for_block"],  # Scry entire draw pile, discard any, gain 1 block per discard
)

VAULT = Card(
    id="Vault", name="Vault", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.ALL, cost=3, upgrade_cost=2, exhaust=True,
    effects=["take_extra_turn"],
)

WISH = Card(
    id="Wish", name="Wish", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=3, exhaust=True,
    base_damage=3,      # For BecomeAlmighty (Strength)
    base_block=6,       # For LiveForever (Plated Armor)
    base_magic=25,      # For FameAndFortune (Gold)
    upgrade_damage=1,   # 3→4 Strength
    upgrade_block=2,    # 6→8 Plated Armor
    upgrade_magic=5,    # 25→30 Gold (was incorrect: 1)
    effects=["choose_plated_armor_or_strength_or_gold"],
)


# === RARE POWERS ===

DEVA_FORM = Card(
    id="DevaForm", name="Deva Form", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.SELF, cost=3, ethereal=True, base_magic=1,
    upgrade_ethereal=False,
    effects=["gain_energy_each_turn_stacking"],
)

DEVOTION = Card(
    id="Devotion", name="Devotion", card_type=CardType.POWER, rarity=CardRarity.RARE,
    target=CardTarget.NONE, cost=1, base_magic=2, upgrade_magic=1,
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


# ============ IRONCLAD CARDS ============

# === BASIC CARDS ===

STRIKE_R = Card(
    id="Strike_R", name="Strike", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    color=CardColor.RED, cost=1, base_damage=6, upgrade_damage=3,
)

DEFEND_R = Card(
    id="Defend_R", name="Defend", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.RED, target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
)

BASH = Card(
    id="Bash", name="Bash", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    color=CardColor.RED, cost=2, base_damage=8, base_magic=2,
    upgrade_damage=2, upgrade_magic=1,  # Apply Vulnerable
    effects=["apply_vulnerable"],
)


# === COMMON ATTACKS ===

ANGER = Card(
    id="Anger", name="Anger", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=0, base_damage=6, upgrade_damage=2,
    effects=["add_copy_to_discard"],
)

BODY_SLAM = Card(
    id="Body Slam", name="Body Slam", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=0, upgrade_cost=0,
    effects=["damage_equals_block"],
)

CLASH = Card(
    id="Clash", name="Clash", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=0, base_damage=14, upgrade_damage=4,
    effects=["only_attacks_in_hand"],
)

CLEAVE = Card(
    id="Cleave", name="Cleave", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=1,
    base_damage=8, upgrade_damage=3,
)

CLOTHESLINE = Card(
    id="Clothesline", name="Clothesline", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=2, base_damage=12, base_magic=2,
    upgrade_damage=2, upgrade_magic=1,
    effects=["apply_weak"],
)

HEADBUTT = Card(
    id="Headbutt", name="Headbutt", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=9, upgrade_damage=3,
    effects=["put_card_from_discard_on_draw"],
)

HEAVY_BLADE = Card(
    id="Heavy Blade", name="Heavy Blade", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=2, base_damage=14, base_magic=3,
    upgrade_magic=2,  # Strength multiplier: 3 -> 5
    effects=["strength_multiplier"],
)

IRON_WAVE = Card(
    id="Iron Wave", name="Iron Wave", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=5, base_block=5,
    upgrade_damage=2, upgrade_block=2,
)

PERFECTED_STRIKE = Card(
    id="Perfected Strike", name="Perfected Strike", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=2, base_damage=6, base_magic=2,
    upgrade_magic=1,  # +2/+3 damage per Strike card
    effects=["damage_per_strike"],
)

POMMEL_STRIKE = Card(
    id="Pommel Strike", name="Pommel Strike", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=9, base_magic=1,
    upgrade_damage=1, upgrade_magic=1,  # Draw 1/2
    effects=["draw_cards"],
)

SWORD_BOOMERANG = Card(
    id="Sword Boomerang", name="Sword Boomerang", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=1,
    base_damage=3, base_magic=3, upgrade_magic=1,  # 3/4 hits
    effects=["random_enemy_x_times"],
)

THUNDERCLAP = Card(
    id="Thunderclap", name="Thunderclap", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=1,
    base_damage=4, upgrade_damage=3,
    effects=["apply_vulnerable_1_all"],
)

TWIN_STRIKE = Card(
    id="Twin Strike", name="Twin Strike", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=5, upgrade_damage=2,
    base_magic=2,  # Hits 2 times
    effects=["damage_x_times"],
)

WILD_STRIKE = Card(
    id="Wild Strike", name="Wild Strike", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.RED, cost=1, base_damage=12, upgrade_damage=5,
    effects=["shuffle_wound_into_draw"],
)


# === COMMON SKILLS ===

ARMAMENTS = Card(
    id="Armaments", name="Armaments", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=5,
    effects=["upgrade_card_in_hand"],  # Upgraded: all cards
)

FLEX = Card(
    id="Flex", name="Flex", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=2,  # Temp strength 2/4
    effects=["gain_temp_strength"],
)

HAVOC = Card(
    id="Havoc", name="Havoc", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    effects=["play_top_card"],
)

SHRUG_IT_OFF = Card(
    id="Shrug It Off", name="Shrug It Off", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=8, upgrade_block=3,
    effects=["draw_1"],
)

TRUE_GRIT = Card(
    id="True Grit", name="True Grit", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=7, upgrade_block=2,
    effects=["exhaust_random_card"],  # Upgraded: choose card
)

WARCRY = Card(
    id="Warcry", name="Warcry", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=1, upgrade_magic=1,  # Draw 1/2, put card on draw
    exhaust=True,
    effects=["draw_then_put_on_draw"],
)


# === UNCOMMON ATTACKS ===

BLOOD_FOR_BLOOD = Card(
    id="Blood for Blood", name="Blood for Blood", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=4, base_damage=18, upgrade_damage=4,
    upgrade_cost=3,  # Base cost 4/3, reduces by 1 when taking damage
    effects=["cost_reduces_when_damaged"],
)

CARNAGE = Card(
    id="Carnage", name="Carnage", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=2, base_damage=20, upgrade_damage=8,
    ethereal=True,
)

DROPKICK = Card(
    id="Dropkick", name="Dropkick", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=1, base_damage=5, upgrade_damage=3,
    effects=["if_vulnerable_draw_and_energy"],
)

HEMOKINESIS = Card(
    id="Hemokinesis", name="Hemokinesis", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=1, base_damage=15, base_magic=2,
    upgrade_damage=5,  # Lose 2 HP
    effects=["lose_hp"],
)

PUMMEL = Card(
    id="Pummel", name="Pummel", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=1, base_damage=2, base_magic=4,
    upgrade_magic=1,  # 4/5 hits
    exhaust=True,
    effects=["damage_x_times"],
)

RAMPAGE = Card(
    id="Rampage", name="Rampage", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=1, base_damage=8, base_magic=5,
    upgrade_magic=3,  # +5/+8 damage each use
    effects=["increase_damage_on_use"],
)

RECKLESS_CHARGE = Card(
    id="Reckless Charge", name="Reckless Charge", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=0, base_damage=7, upgrade_damage=3,
    effects=["shuffle_dazed_into_draw"],
)

SEARING_BLOW = Card(
    id="Searing Blow", name="Searing Blow", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=2, base_damage=12, upgrade_damage=4,  # Can upgrade multiple times
    effects=["can_upgrade_unlimited"],
)

SEVER_SOUL = Card(
    id="Sever Soul", name="Sever Soul", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=2, base_damage=16, upgrade_damage=6,
    effects=["exhaust_all_non_attacks"],
)

UPPERCUT = Card(
    id="Uppercut", name="Uppercut", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, cost=2, base_damage=13,
    base_magic=1, upgrade_magic=1,  # Weak and Vulnerable 1/2
    effects=["apply_weak_and_vulnerable"],
)

WHIRLWIND = Card(
    id="Whirlwind", name="Whirlwind", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=-1,  # X cost
    base_damage=5, upgrade_damage=3,
    effects=["damage_all_x_times"],
)


# === UNCOMMON SKILLS ===

BATTLE_TRANCE = Card(
    id="Battle Trance", name="Battle Trance", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=0,
    base_magic=3, upgrade_magic=1,  # Draw 3/4, can't draw more this turn
    effects=["draw_then_no_draw"],
)

BLOODLETTING = Card(
    id="Bloodletting", name="Bloodletting", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=1,  # Lose 3 HP, gain 2/3 energy
    effects=["lose_hp_gain_energy"],
)

BURNING_PACT = Card(
    id="Burning Pact", name="Burning Pact", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=1,
    base_magic=2, upgrade_magic=1,  # Exhaust 1, draw 2/3
    effects=["exhaust_to_draw"],
)

DISARM = Card(
    id="Disarm", name="Disarm", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.ENEMY, cost=1,
    base_magic=2, upgrade_magic=1,  # -2/-3 strength
    exhaust=True,
    effects=["reduce_enemy_strength"],
)

DUAL_WIELD = Card(
    id="Dual Wield", name="Dual Wield", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=1,
    base_magic=1, upgrade_magic=1,  # 1/2 copies
    effects=["copy_attack_or_power"],
)

ENTRENCH = Card(
    id="Entrench", name="Entrench", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=2, upgrade_cost=1,
    effects=["double_block"],
)

FLAME_BARRIER = Card(
    id="Flame Barrier", name="Flame Barrier", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=2,
    base_block=12, base_magic=4, upgrade_block=4, upgrade_magic=2,
    effects=["when_attacked_deal_damage"],
)

GHOSTLY_ARMOR = Card(
    id="Ghostly Armor", name="Ghostly Armor", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=10, upgrade_block=3,
    ethereal=True,
)

INFERNAL_BLADE = Card(
    id="Infernal Blade", name="Infernal Blade", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    exhaust=True,
    effects=["add_random_attack_cost_0"],
)

INTIMIDATE = Card(
    id="Intimidate", name="Intimidate", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=0,
    base_magic=1, upgrade_magic=1,  # Weak 1/2 to all
    exhaust=True,
    effects=["apply_weak_all"],
)

POWER_THROUGH = Card(
    id="Power Through", name="Power Through", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=15, upgrade_block=5,
    effects=["add_wounds_to_hand"],
)

RAGE = Card(
    id="Rage", name="Rage", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=3, upgrade_magic=2,  # Block 3/5 per attack this turn
    effects=["gain_block_per_attack"],
)

SECOND_WIND = Card(
    id="Second Wind", name="Second Wind", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=5, upgrade_block=2,  # Block per non-attack exhausted
    effects=["exhaust_non_attacks_gain_block"],
)

SEEING_RED = Card(
    id="Seeing Red", name="Seeing Red", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    exhaust=True,
    effects=["gain_2_energy"],
)

SENTINEL = Card(
    id="Sentinel", name="Sentinel", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_block=5, upgrade_block=3,
    effects=["gain_energy_on_exhaust_2_3"],  # 2/3 energy when exhausted
)

SHOCKWAVE = Card(
    id="Shockwave", name="Shockwave", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=2,
    base_magic=3, upgrade_magic=2,  # Weak and Vulnerable 3/5 all
    exhaust=True,
    effects=["apply_weak_and_vulnerable_all"],
)

SPOT_WEAKNESS = Card(
    id="Spot Weakness", name="Spot Weakness", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF_AND_ENEMY, cost=1,
    base_magic=3, upgrade_magic=1,  # Strength 3/4 if enemy attacking
    effects=["gain_strength_if_enemy_attacking"],
)


# === UNCOMMON POWERS ===

COMBUST = Card(
    id="Combust", name="Combust", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=5, upgrade_magic=2,  # Lose 1 HP, deal 5/7 damage to all
    effects=["end_turn_damage_all_lose_hp"],
)

DARK_EMBRACE = Card(
    id="Dark Embrace", name="Dark Embrace", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=2, upgrade_cost=1,
    effects=["draw_on_exhaust"],
)

EVOLVE = Card(
    id="Evolve", name="Evolve", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1,  # Draw 1/2 when Status drawn
    effects=["draw_on_status"],
)

FEEL_NO_PAIN = Card(
    id="Feel No Pain", name="Feel No Pain", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=3, upgrade_magic=1,  # Block 3/4 when exhaust
    effects=["block_on_exhaust"],
)

FIRE_BREATHING = Card(
    id="Fire Breathing", name="Fire Breathing", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=6, upgrade_magic=4,  # Deal 6/10 when Status/Curse drawn
    effects=["damage_on_status_curse"],
)

INFLAME = Card(
    id="Inflame", name="Inflame", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=2, upgrade_magic=1,  # Gain 2/3 Strength
    effects=["gain_strength"],
)

METALLICIZE = Card(
    id="Metallicize", name="Metallicize", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=3, upgrade_magic=1,  # End turn gain 3/4 block
    effects=["end_turn_gain_block"],
)

RUPTURE = Card(
    id="Rupture", name="Rupture", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1,  # +1/+2 Strength when lose HP from card
    effects=["gain_strength_on_hp_loss"],
)


# === RARE ATTACKS ===

BLUDGEON = Card(
    id="Bludgeon", name="Bludgeon", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.RED, cost=3, base_damage=32, upgrade_damage=10,
)

FEED = Card(
    id="Feed", name="Feed", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.RED, cost=1, base_damage=10, base_magic=3,
    upgrade_damage=2, upgrade_magic=1,  # If fatal, gain 3/4 max HP
    exhaust=True,
    effects=["if_fatal_gain_max_hp"],
)

FIEND_FIRE = Card(
    id="Fiend Fire", name="Fiend Fire", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.RED, cost=2, base_damage=7, upgrade_damage=3,
    exhaust=True,
    effects=["exhaust_hand_damage_per_card"],
)

IMMOLATE = Card(
    id="Immolate", name="Immolate", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=2,
    base_damage=21, upgrade_damage=7,
    effects=["add_burn_to_discard"],
)

REAPER = Card(
    id="Reaper", name="Reaper", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.ALL_ENEMY, cost=2,
    base_damage=4, upgrade_damage=1,  # Heal unblocked damage dealt
    exhaust=True,
    effects=["damage_all_heal_unblocked"],
)


# === RARE SKILLS ===

DOUBLE_TAP = Card(
    id="Double Tap", name="Double Tap", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1,  # Play next 1/2 attacks twice
    effects=["play_attacks_twice"],
)

EXHUME = Card(
    id="Exhume", name="Exhume", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    exhaust=True,
    effects=["return_exhausted_card_to_hand"],
)

IMPERVIOUS = Card(
    id="Impervious", name="Impervious", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=2,
    base_block=30, upgrade_block=10,
    exhaust=True,
)

LIMIT_BREAK = Card(
    id="Limit Break", name="Limit Break", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=1,
    exhaust=True,  # Upgraded: no exhaust
    effects=["double_strength"],
)

OFFERING = Card(
    id="Offering", name="Offering", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=3, upgrade_magic=2,  # Lose 6 HP, gain 2 energy, draw 3/5
    exhaust=True,
    effects=["lose_hp_gain_energy_draw"],
)


# === RARE POWERS ===

BARRICADE = Card(
    id="Barricade", name="Barricade", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=3, upgrade_cost=2,
    effects=["block_not_lost"],
)

BERSERK = Card(
    id="Berserk", name="Berserk", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=-1,  # Vulnerable 2/1, +1 energy each turn
    effects=["gain_vulnerable_gain_energy_per_turn"],
)

BRUTALITY = Card(
    id="Brutality", name="Brutality", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=0,
    innate=False,  # Upgraded: Innate
    effects=["start_turn_lose_hp_draw"],
)

CORRUPTION = Card(
    id="Corruption", name="Corruption", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=3, upgrade_cost=2,
    effects=["skills_cost_0_exhaust"],
)

DEMON_FORM = Card(
    id="Demon Form", name="Demon Form", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.NONE, cost=3,
    base_magic=2, upgrade_magic=1,  # Gain 2/3 Strength each turn
    effects=["gain_strength_each_turn"],
)

JUGGERNAUT = Card(
    id="Juggernaut", name="Juggernaut", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.RED, target=CardTarget.SELF, cost=2,
    base_magic=5, upgrade_magic=2,  # Deal 5/7 damage to random when gain block
    effects=["damage_random_on_block"],
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


# ============ DEFECT CARDS ============

# === BASIC CARDS ===

STRIKE_D = Card(
    id="Strike_B", name="Strike", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    color=CardColor.BLUE, cost=1, base_damage=6, upgrade_damage=3,
)

DEFEND_D = Card(
    id="Defend_B", name="Defend", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
)

ZAP = Card(
    id="Zap", name="Zap", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    base_magic=1, effects=["channel_lightning"],
)

DUALCAST = Card(
    id="Dualcast", name="Dualcast", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    effects=["evoke_orb_twice"],
)


# === COMMON ATTACKS ===

BALL_LIGHTNING = Card(
    id="Ball Lightning", name="Ball Lightning", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=1, base_damage=7, upgrade_damage=3,
    base_magic=1, effects=["channel_lightning"],
)

BARRAGE = Card(
    id="Barrage", name="Barrage", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=1, base_damage=4, upgrade_damage=2,
    effects=["damage_per_orb"],
)

BEAM_CELL = Card(
    id="Beam Cell", name="Beam Cell", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=0, base_damage=3, upgrade_damage=1,
    base_magic=1, upgrade_magic=1, effects=["apply_vulnerable"],
)

CLAW = Card(
    id="Claw", name="Claw", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=0, base_damage=3, upgrade_damage=2,
    base_magic=2, effects=["increase_all_claw_damage"],
)

COLD_SNAP = Card(
    id="Cold Snap", name="Cold Snap", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=1, base_damage=6, upgrade_damage=3,
    base_magic=1, effects=["channel_frost"],
)

COMPILE_DRIVER = Card(
    id="Compile Driver", name="Compile Driver", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=1, base_damage=7, upgrade_damage=3,
    base_magic=1, effects=["draw_per_unique_orb"],
)

GO_FOR_THE_EYES = Card(
    id="Go for the Eyes", name="Go for the Eyes", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=0, base_damage=3, upgrade_damage=1,
    base_magic=1, upgrade_magic=1, effects=["if_attacking_apply_weak"],
)

REBOUND = Card(
    id="Rebound", name="Rebound", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=1, base_damage=9, upgrade_damage=3,
    effects=["next_card_on_top_of_draw"],
)

STREAMLINE = Card(
    id="Streamline", name="Streamline", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, cost=2, base_damage=15, upgrade_damage=5,
    base_magic=1, effects=["reduce_cost_permanently"],
)

SWEEPING_BEAM = Card(
    id="Sweeping Beam", name="Sweeping Beam", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=1, base_damage=6, upgrade_damage=3,
    base_magic=1, effects=["draw_1"],
)


# === COMMON SKILLS ===

CHARGE_BATTERY = Card(
    id="Conserve Battery", name="Charge Battery", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=7, upgrade_block=3,
    effects=["gain_1_energy_next_turn"],
)

COOLHEADED = Card(
    id="Coolheaded", name="Coolheaded", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["channel_frost", "draw_cards"],
)

HOLOGRAM = Card(
    id="Hologram", name="Hologram", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=3, upgrade_block=2,
    exhaust=True, effects=["return_card_from_discard"],
)

LEAP_D = Card(
    id="Leap", name="Leap", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=9, upgrade_block=3,
)

RECURSION = Card(
    id="Redo", name="Recursion", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    effects=["evoke_then_channel_same_orb"],
)

STACK = Card(
    id="Stack", name="Stack", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=0,
    upgrade_block=3, effects=["block_equals_discard_size"],
)

STEAM_BARRIER = Card(
    id="Steam", name="Steam Barrier", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0, base_block=6, upgrade_block=2,
    effects=["lose_1_block_permanently"],
)

TURBO = Card(
    id="Turbo", name="Turbo", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=1, effects=["gain_energy_magic", "add_void_to_discard"],
)


# === UNCOMMON ATTACKS ===

BLIZZARD = Card(
    id="Blizzard", name="Blizzard", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=1, base_damage=0,
    base_magic=2, upgrade_magic=1, effects=["damage_per_frost_channeled"],
)

DOOM_AND_GLOOM = Card(
    id="Doom and Gloom", name="Doom and Gloom", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=2, base_damage=10, upgrade_damage=4,
    base_magic=1, effects=["channel_dark"],
)

FTL = Card(
    id="FTL", name="FTL", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, cost=0, base_damage=5, upgrade_damage=1,
    base_magic=3, upgrade_magic=1, effects=["if_played_less_than_x_draw"],
)

LOCKON = Card(
    id="Lockon", name="Lock-On", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, cost=1, base_damage=8, upgrade_damage=3,
    base_magic=2, upgrade_magic=1, effects=["apply_lockon"],
)

MELTER = Card(
    id="Melter", name="Melter", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, cost=1, base_damage=10, upgrade_damage=4,
    effects=["remove_enemy_block"],
)

RIP_AND_TEAR = Card(
    id="Rip and Tear", name="Rip and Tear", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=1, base_damage=7, upgrade_damage=2,
    base_magic=2, effects=["damage_random_enemy_twice"],
)

SCRAPE = Card(
    id="Scrape", name="Scrape", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, cost=1, base_damage=7, upgrade_damage=3,
    base_magic=4, upgrade_magic=1, effects=["draw_discard_non_zero_cost"],
)

SUNDER = Card(
    id="Sunder", name="Sunder", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, cost=3, base_damage=24, upgrade_damage=8,
    effects=["if_fatal_gain_3_energy"],
)


# === UNCOMMON SKILLS ===

AGGREGATE = Card(
    id="Aggregate", name="Aggregate", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=4, upgrade_magic=-1, effects=["gain_energy_per_x_cards_in_draw"],
)

AUTO_SHIELDS = Card(
    id="Auto Shields", name="Auto-Shields", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=11, upgrade_block=4,
    effects=["only_if_no_block"],
)

BOOT_SEQUENCE = Card(
    id="BootSequence", name="Boot Sequence", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0, base_block=10, upgrade_block=3,
    innate=True, exhaust=True,
)

CHAOS = Card(
    id="Chaos", name="Chaos", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["channel_random_orb"],
)

CHILL = Card(
    id="Chill", name="Chill", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0,
    base_magic=1, exhaust=True, effects=["channel_frost_per_enemy"],
)

CONSUME = Card(
    id="Consume", name="Consume", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2,
    base_magic=2, upgrade_magic=1, effects=["gain_focus_lose_orb_slot"],
)

DARKNESS_D = Card(
    id="Darkness", name="Darkness", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, effects=["channel_dark"],
)

DOUBLE_ENERGY = Card(
    id="Double Energy", name="Double Energy", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    exhaust=True, effects=["double_energy"],
)

EQUILIBRIUM_D = Card(
    id="Undo", name="Equilibrium", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2, base_block=13, upgrade_block=3,
    base_magic=1, effects=["retain_hand"],
)

FORCE_FIELD = Card(
    id="Force Field", name="Force Field", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=4, base_block=12, upgrade_block=4,
    effects=["cost_reduces_per_power_played"],
)

FUSION = Card(
    id="Fusion", name="Fusion", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2, upgrade_cost=1,
    base_magic=1, effects=["channel_plasma"],
)

GENETIC_ALGORITHM = Card(
    id="Genetic Algorithm", name="Genetic Algorithm", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, base_block=1,
    base_magic=2, upgrade_magic=1, exhaust=True, effects=["block_increases_permanently"],
)

GLACIER = Card(
    id="Glacier", name="Glacier", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2, base_block=7, upgrade_block=3,
    base_magic=2, effects=["channel_2_frost"],
)

OVERCLOCK = Card(
    id="Steam Power", name="Overclock", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0,
    base_magic=2, upgrade_magic=1, effects=["draw_cards", "add_burn_to_discard"],
)

RECYCLE = Card(
    id="Recycle", name="Recycle", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    effects=["exhaust_card_gain_energy"],
)

REINFORCED_BODY = Card(
    id="Reinforced Body", name="Reinforced Body", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=-1, base_block=7, upgrade_block=2,
    effects=["block_x_times"],
)

REPROGRAM = Card(
    id="Reprogram", name="Reprogram", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=1,
    base_magic=1, upgrade_magic=1, effects=["lose_focus_gain_strength_dex"],
)

SKIM = Card(
    id="Skim", name="Skim", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=1,
    base_magic=3, upgrade_magic=1, effects=["draw_cards"],
)

TEMPEST = Card(
    id="Tempest", name="Tempest", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=-1,
    exhaust=True, effects=["channel_x_lightning"],
)

WHITE_NOISE = Card(
    id="White Noise", name="White Noise", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    exhaust=True, effects=["add_random_power_to_hand_cost_0"],
)


# === UNCOMMON POWERS ===

CAPACITOR = Card(
    id="Capacitor", name="Capacitor", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=2, upgrade_magic=1, effects=["increase_orb_slots"],
)

DEFRAGMENT = Card(
    id="Defragment", name="Defragment", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["gain_focus"],
)

HEATSINKS = Card(
    id="Heatsinks", name="Heatsinks", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["draw_on_power_play"],
)

HELLO_WORLD = Card(
    id="Hello World", name="Hello World", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    effects=["add_common_card_each_turn"],
)

LOOP_D = Card(
    id="Loop", name="Loop", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["trigger_orb_passive_extra"],
)

SELF_REPAIR = Card(
    id="Self Repair", name="Self Repair", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=7, upgrade_magic=3, effects=["heal_at_end_of_combat"],
)

STATIC_DISCHARGE = Card(
    id="Static Discharge", name="Static Discharge", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["channel_lightning_on_damage"],
)

STORM_D = Card(
    id="Storm", name="Storm", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, effects=["channel_lightning_on_power_play"],
)


# === RARE ATTACKS ===

ALL_FOR_ONE = Card(
    id="All For One", name="All For One", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.BLUE, cost=2, base_damage=10, upgrade_damage=4,
    effects=["return_all_0_cost_from_discard"],
)

CORE_SURGE = Card(
    id="Core Surge", name="Core Surge", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.BLUE, cost=1, base_damage=11, upgrade_damage=4,
    base_magic=1, exhaust=True, effects=["gain_artifact"],
)

HYPERBEAM = Card(
    id="Hyperbeam", name="Hyperbeam", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=2, base_damage=26, upgrade_damage=8,
    base_magic=3, effects=["lose_focus"],
)

METEOR_STRIKE = Card(
    id="Meteor Strike", name="Meteor Strike", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.BLUE, cost=5, base_damage=24, upgrade_damage=6,
    base_magic=3, effects=["channel_3_plasma"],
)

THUNDER_STRIKE = Card(
    id="Thunder Strike", name="Thunder Strike", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.ALL_ENEMY, cost=3, base_damage=7, upgrade_damage=2,
    effects=["damage_per_lightning_channeled"],
)


# === RARE SKILLS ===

AMPLIFY = Card(
    id="Amplify", name="Amplify", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1, effects=["next_power_plays_twice"],
)

FISSION = Card(
    id="Fission", name="Fission", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=0,
    base_magic=1, exhaust=True, effects=["remove_orbs_gain_energy_and_draw"],
)

MULTI_CAST = Card(
    id="Multi-Cast", name="Multi-Cast", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=-1,
    effects=["evoke_first_orb_x_times"],
)

RAINBOW = Card(
    id="Rainbow", name="Rainbow", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2,
    exhaust=True, effects=["channel_lightning_frost_dark"],
)

REBOOT = Card(
    id="Reboot", name="Reboot", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=0,
    base_magic=4, upgrade_magic=2, exhaust=True, effects=["shuffle_hand_and_discard_draw"],
)

SEEK = Card(
    id="Seek", name="Seek", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.NONE, cost=0,
    base_magic=1, upgrade_magic=1, exhaust=True, effects=["search_draw_pile"],
)


# === RARE POWERS ===

BIASED_COGNITION = Card(
    id="Biased Cognition", name="Biased Cognition", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=4, upgrade_magic=1, effects=["gain_focus_lose_focus_each_turn"],
)

BUFFER = Card(
    id="Buffer", name="Buffer", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2,
    base_magic=1, upgrade_magic=1, effects=["prevent_next_hp_loss"],
)

CREATIVE_AI = Card(
    id="Creative AI", name="Creative AI", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=3, upgrade_cost=2,
    base_magic=1, effects=["add_random_power_each_turn"],
)

ECHO_FORM = Card(
    id="Echo Form", name="Echo Form", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=3,
    ethereal=True, effects=["play_first_card_twice"],
)

ELECTRODYNAMICS = Card(
    id="Electrodynamics", name="Electrodynamics", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=2,
    base_magic=2, upgrade_magic=1, effects=["lightning_hits_all", "channel_lightning_magic"],
)

MACHINE_LEARNING = Card(
    id="Machine Learning", name="Machine Learning", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.BLUE, target=CardTarget.SELF, cost=1,
    base_magic=1, effects=["draw_extra_each_turn"],
)


# ============ CARD REGISTRY ============

# All Defect cards by ID
DEFECT_CARDS: Dict[str, Card] = {
    # Basic
    "Strike_B": STRIKE_D,
    "Defend_B": DEFEND_D,
    "Zap": ZAP,
    "Dualcast": DUALCAST,
    # Common Attacks
    "Ball Lightning": BALL_LIGHTNING,
    "Barrage": BARRAGE,
    "Beam Cell": BEAM_CELL,
    "Claw": CLAW,
    "Cold Snap": COLD_SNAP,
    "Compile Driver": COMPILE_DRIVER,
    "Go for the Eyes": GO_FOR_THE_EYES,
    "Rebound": REBOUND,
    "Streamline": STREAMLINE,
    "Sweeping Beam": SWEEPING_BEAM,
    # Common Skills
    "Conserve Battery": CHARGE_BATTERY,
    "Coolheaded": COOLHEADED,
    "Hologram": HOLOGRAM,
    "Leap": LEAP_D,
    "Redo": RECURSION,
    "Stack": STACK,
    "Steam": STEAM_BARRIER,
    "Turbo": TURBO,
    # Uncommon Attacks
    "Blizzard": BLIZZARD,
    "Doom and Gloom": DOOM_AND_GLOOM,
    "FTL": FTL,
    "Lockon": LOCKON,
    "Melter": MELTER,
    "Rip and Tear": RIP_AND_TEAR,
    "Scrape": SCRAPE,
    "Sunder": SUNDER,
    # Uncommon Skills
    "Aggregate": AGGREGATE,
    "Auto Shields": AUTO_SHIELDS,
    "BootSequence": BOOT_SEQUENCE,
    "Chaos": CHAOS,
    "Chill": CHILL,
    "Consume": CONSUME,
    "Darkness": DARKNESS_D,
    "Double Energy": DOUBLE_ENERGY,
    "Undo": EQUILIBRIUM_D,
    "Force Field": FORCE_FIELD,
    "Fusion": FUSION,
    "Genetic Algorithm": GENETIC_ALGORITHM,
    "Glacier": GLACIER,
    "Steam Power": OVERCLOCK,
    "Recycle": RECYCLE,
    "Reinforced Body": REINFORCED_BODY,
    "Reprogram": REPROGRAM,
    "Skim": SKIM,
    "Tempest": TEMPEST,
    "White Noise": WHITE_NOISE,
    # Uncommon Powers
    "Capacitor": CAPACITOR,
    "Defragment": DEFRAGMENT,
    "Heatsinks": HEATSINKS,
    "Hello World": HELLO_WORLD,
    "Loop": LOOP_D,
    "Self Repair": SELF_REPAIR,
    "Static Discharge": STATIC_DISCHARGE,
    "Storm": STORM_D,
    # Rare Attacks
    "All For One": ALL_FOR_ONE,
    "Core Surge": CORE_SURGE,
    "Hyperbeam": HYPERBEAM,
    "Meteor Strike": METEOR_STRIKE,
    "Thunder Strike": THUNDER_STRIKE,
    # Rare Skills
    "Amplify": AMPLIFY,
    "Fission": FISSION,
    "Multi-Cast": MULTI_CAST,
    "Rainbow": RAINBOW,
    "Reboot": REBOOT,
    "Seek": SEEK,
    # Rare Powers
    "Biased Cognition": BIASED_COGNITION,
    "Buffer": BUFFER,
    "Creative AI": CREATIVE_AI,
    "Echo Form": ECHO_FORM,
    "Electrodynamics": ELECTRODYNAMICS,
    "Machine Learning": MACHINE_LEARNING,
}


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
    "DeusExMachina": DEUS_EX_MACHINA,
    "Alpha": ALPHA,
    "Blasphemy": BLASPHEMY,
    "ConjureBlade": CONJURE_BLADE,
    "ForeignInfluence": FOREIGN_INFLUENCE,
    "Omniscience": OMNISCIENCE,
    "Scrawl": SCRAWL,
    "SpiritShield": SPIRIT_SHIELD,
    "Unraveling": UNRAVELING,
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


# Ironclad cards by ID
IRONCLAD_CARDS: Dict[str, Card] = {
    # Basic
    "Strike_R": STRIKE_R,
    "Defend_R": DEFEND_R,
    "Bash": BASH,
    # Common Attacks
    "Anger": ANGER,
    "Body Slam": BODY_SLAM,
    "Clash": CLASH,
    "Cleave": CLEAVE,
    "Clothesline": CLOTHESLINE,
    "Headbutt": HEADBUTT,
    "Heavy Blade": HEAVY_BLADE,
    "Iron Wave": IRON_WAVE,
    "Perfected Strike": PERFECTED_STRIKE,
    "Pommel Strike": POMMEL_STRIKE,
    "Sword Boomerang": SWORD_BOOMERANG,
    "Thunderclap": THUNDERCLAP,
    "Twin Strike": TWIN_STRIKE,
    "Wild Strike": WILD_STRIKE,
    # Common Skills
    "Armaments": ARMAMENTS,
    "Flex": FLEX,
    "Havoc": HAVOC,
    "Shrug It Off": SHRUG_IT_OFF,
    "True Grit": TRUE_GRIT,
    "Warcry": WARCRY,
    # Uncommon Attacks
    "Blood for Blood": BLOOD_FOR_BLOOD,
    "Carnage": CARNAGE,
    "Dropkick": DROPKICK,
    "Hemokinesis": HEMOKINESIS,
    "Pummel": PUMMEL,
    "Rampage": RAMPAGE,
    "Reckless Charge": RECKLESS_CHARGE,
    "Searing Blow": SEARING_BLOW,
    "Sever Soul": SEVER_SOUL,
    "Uppercut": UPPERCUT,
    "Whirlwind": WHIRLWIND,
    # Uncommon Skills
    "Battle Trance": BATTLE_TRANCE,
    "Bloodletting": BLOODLETTING,
    "Burning Pact": BURNING_PACT,
    "Disarm": DISARM,
    "Dual Wield": DUAL_WIELD,
    "Entrench": ENTRENCH,
    "Flame Barrier": FLAME_BARRIER,
    "Ghostly Armor": GHOSTLY_ARMOR,
    "Infernal Blade": INFERNAL_BLADE,
    "Intimidate": INTIMIDATE,
    "Power Through": POWER_THROUGH,
    "Rage": RAGE,
    "Second Wind": SECOND_WIND,
    "Seeing Red": SEEING_RED,
    "Sentinel": SENTINEL,
    "Shockwave": SHOCKWAVE,
    "Spot Weakness": SPOT_WEAKNESS,
    # Uncommon Powers
    "Combust": COMBUST,
    "Dark Embrace": DARK_EMBRACE,
    "Evolve": EVOLVE,
    "Feel No Pain": FEEL_NO_PAIN,
    "Fire Breathing": FIRE_BREATHING,
    "Inflame": INFLAME,
    "Metallicize": METALLICIZE,
    "Rupture": RUPTURE,
    # Rare Attacks
    "Bludgeon": BLUDGEON,
    "Feed": FEED,
    "Fiend Fire": FIEND_FIRE,
    "Immolate": IMMOLATE,
    "Reaper": REAPER,
    # Rare Skills
    "Double Tap": DOUBLE_TAP,
    "Exhume": EXHUME,
    "Impervious": IMPERVIOUS,
    "Limit Break": LIMIT_BREAK,
    "Offering": OFFERING,
    # Rare Powers
    "Barricade": BARRICADE,
    "Berserk": BERSERK,
    "Brutality": BRUTALITY,
    "Corruption": CORRUPTION,
    "Demon Form": DEMON_FORM,
    "Juggernaut": JUGGERNAUT,
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


# ============ SILENT (GREEN) CARDS ============

# === BASIC CARDS ===

STRIKE_S = Card(
    id="Strike_G", name="Strike", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    color=CardColor.GREEN, cost=1, base_damage=6, upgrade_damage=3,
)

DEFEND_S = Card(
    id="Defend_G", name="Defend", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
)

NEUTRALIZE = Card(
    id="Neutralize", name="Neutralize", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
    color=CardColor.GREEN, cost=0, base_damage=3, upgrade_damage=1,
    base_magic=1, upgrade_magic=1,  # Weak amount
    effects=["apply_weak"],
)

SURVIVOR_S = Card(
    id="Survivor", name="Survivor", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=8, upgrade_block=3,
    effects=["discard_1"],
)


# === COMMON ATTACKS ===

BANE = Card(
    id="Bane", name="Bane", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=7, upgrade_damage=3,
    effects=["double_damage_if_poisoned"],
)

DAGGER_SPRAY = Card(
    id="Dagger Spray", name="Dagger Spray", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=1, base_damage=4, upgrade_damage=2,
    base_magic=2,  # Hits twice
    effects=["damage_all_x_times"],
)

DAGGER_THROW = Card(
    id="Dagger Throw", name="Dagger Throw", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=9, upgrade_damage=3,
    effects=["draw_1", "discard_1"],
)

FLYING_KNEE = Card(
    id="Flying Knee", name="Flying Knee", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=8, upgrade_damage=3,
    effects=["gain_energy_next_turn_1"],
)

POISONED_STAB = Card(
    id="Poisoned Stab", name="Poisoned Stab", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=6, upgrade_damage=2,
    base_magic=3, upgrade_magic=1,  # Poison amount
    effects=["apply_poison"],
)

QUICK_SLASH = Card(
    id="Quick Slash", name="Quick Slash", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=8, upgrade_damage=4,
    effects=["draw_1"],
)

SLICE = Card(
    id="Slice", name="Slice", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=0, base_damage=6, upgrade_damage=3,
)

SNEAKY_STRIKE = Card(
    id="Underhanded Strike", name="Sneaky Strike", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    # Java cardID is "Underhanded Strike"
    color=CardColor.GREEN, cost=2, base_damage=12, upgrade_damage=4,
    effects=["refund_2_energy_if_discarded_this_turn"],
)

SUCKER_PUNCH = Card(
    id="Sucker Punch", name="Sucker Punch", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, cost=1, base_damage=7, upgrade_damage=2,
    base_magic=1, upgrade_magic=1,  # Weak amount
    effects=["apply_weak"],
)


# === COMMON SKILLS ===

ACROBATICS = Card(
    id="Acrobatics", name="Acrobatics", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1,
    base_magic=3, upgrade_magic=1,  # Draw amount
    effects=["draw_x", "discard_1"],
)

BACKFLIP = Card(
    id="Backflip", name="Backflip", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
    effects=["draw_2"],
)

BLADE_DANCE = Card(
    id="Blade Dance", name="Blade Dance", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1,
    base_magic=3, upgrade_magic=1,  # Number of Shivs
    effects=["add_shivs_to_hand"],
)

CLOAK_AND_DAGGER = Card(
    id="Cloak And Dagger", name="Cloak and Dagger", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=6,
    base_magic=1, upgrade_magic=1,  # Number of Shivs
    effects=["add_shivs_to_hand"],
)

DEADLY_POISON = Card(
    id="Deadly Poison", name="Deadly Poison", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=1,
    base_magic=5, upgrade_magic=2,  # Poison amount
    effects=["apply_poison"],
)

DEFLECT = Card(
    id="Deflect", name="Deflect", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=0, base_block=4, upgrade_block=3,
)

DODGE_AND_ROLL = Card(
    id="Dodge and Roll", name="Dodge and Roll", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=4, upgrade_block=2,
    effects=["block_next_turn"],
)

OUTMANEUVER = Card(
    id="Outmaneuver", name="Outmaneuver", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1,
    base_magic=2, upgrade_magic=1,  # Energy next turn (shown in description change)
    effects=["gain_energy_next_turn"],
)

PIERCING_WAIL = Card(
    id="PiercingWail", name="Piercing Wail", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=1, exhaust=True,
    base_magic=6, upgrade_magic=2,  # Strength reduction amount (temp)
    effects=["reduce_strength_all_enemies"],
)

PREPARED = Card(
    id="Prepared", name="Prepared", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=0,
    base_magic=1, upgrade_magic=1,  # Draw/discard amount
    effects=["draw_x", "discard_x"],
)


# === UNCOMMON ATTACKS ===

ALL_OUT_ATTACK = Card(
    id="All Out Attack", name="All-Out Attack", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=1, base_damage=10, upgrade_damage=4,
    effects=["discard_random_1"],
)

BACKSTAB = Card(
    id="Backstab", name="Backstab", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=0, base_damage=11, upgrade_damage=4,
    innate=True, exhaust=True,
)

CHOKE = Card(
    id="Choke", name="Choke", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=2, base_damage=12,
    base_magic=3, upgrade_magic=2,  # Choke amount (damage per card played)
    effects=["apply_choke"],
)

DASH_S = Card(
    id="Dash", name="Dash", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=2, base_damage=10, upgrade_damage=3,
    base_block=10, upgrade_block=3,
)

ENDLESS_AGONY = Card(
    id="Endless Agony", name="Endless Agony", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=0, base_damage=4, upgrade_damage=2, exhaust=True,
    effects=["copy_to_hand_when_drawn"],
)

EVISCERATE = Card(
    id="Eviscerate", name="Eviscerate", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=3, base_damage=7, upgrade_damage=2,
    base_magic=3,  # Hits 3 times
    effects=["cost_reduces_per_discard", "damage_x_times"],
)

FINISHER = Card(
    id="Finisher", name="Finisher", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=1, base_damage=6, upgrade_damage=2,
    effects=["damage_per_attack_this_turn"],
)

FLECHETTES = Card(
    id="Flechettes", name="Flechettes", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=1, base_damage=4, upgrade_damage=2,
    effects=["damage_per_skill_in_hand"],
)

HEEL_HOOK = Card(
    id="Heel Hook", name="Heel Hook", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=1, base_damage=5, upgrade_damage=3,
    effects=["if_target_weak_gain_energy_draw"],
)

MASTERFUL_STAB = Card(
    id="Masterful Stab", name="Masterful Stab", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=0, base_damage=12, upgrade_damage=4,
    effects=["cost_increases_when_damaged"],
)

PREDATOR = Card(
    id="Predator", name="Predator", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=2, base_damage=15, upgrade_damage=5,
    effects=["draw_2_next_turn"],
)

RIDDLE_WITH_HOLES = Card(
    id="Riddle With Holes", name="Riddle with Holes", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=2, base_damage=3, upgrade_damage=1,
    base_magic=5,  # Hit count
    effects=["damage_x_times"],
)

SKEWER = Card(
    id="Skewer", name="Skewer", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, cost=-1, base_damage=7, upgrade_damage=3,  # X cost
    effects=["damage_x_times_energy"],
)


# === UNCOMMON SKILLS ===

BLUR = Card(
    id="Blur", name="Blur", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
    effects=["block_not_removed_next_turn"],
)

BOUNCING_FLASK = Card(
    id="Bouncing Flask", name="Bouncing Flask", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=2,
    base_magic=3, upgrade_magic=1,  # Poison per bounce (3 bounces)
    effects=["apply_poison_random_3_times"],
)

CALCULATED_GAMBLE = Card(
    id="Calculated Gamble", name="Calculated Gamble", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=0, exhaust=True,
    # Upgraded: no longer exhausts
    effects=["discard_hand_draw_same"],
)

CATALYST = Card(
    id="Catalyst", name="Catalyst", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=1, exhaust=True,
    # Base: double poison, Upgraded: triple poison
    effects=["double_poison"],
)

CONCENTRATE = Card(
    id="Concentrate", name="Concentrate", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=0,
    base_magic=3, upgrade_magic=-1,  # Discard amount (reduced on upgrade)
    effects=["discard_x", "gain_energy_2"],
)

CRIPPLING_POISON = Card(
    id="Crippling Poison", name="Crippling Poison", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=2, exhaust=True,
    base_magic=4, upgrade_magic=3,  # Poison amount
    effects=["apply_poison_all", "apply_weak_2_all"],
)

DISTRACTION = Card(
    id="Distraction", name="Distraction", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1, upgrade_cost=0, exhaust=True,
    effects=["add_random_skill_cost_0"],
)

ESCAPE_PLAN = Card(
    id="Escape Plan", name="Escape Plan", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=0, base_block=3, upgrade_block=2,
    effects=["draw_1", "if_skill_drawn_gain_block"],
)

EXPERTISE = Card(
    id="Expertise", name="Expertise", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=6, upgrade_magic=1,  # Hand size to draw to
    effects=["draw_to_x_cards"],
)

LEG_SWEEP = Card(
    id="Leg Sweep", name="Leg Sweep", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=2, base_block=11, upgrade_block=3,
    base_magic=2, upgrade_magic=1,  # Weak amount
    effects=["apply_weak"],
)

REFLEX = Card(
    id="Reflex", name="Reflex", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=-2,  # Unplayable
    base_magic=2, upgrade_magic=1,  # Draw amount
    effects=["unplayable", "when_discarded_draw"],
)

SETUP_S = Card(
    id="Setup", name="Setup", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1, upgrade_cost=0,
    effects=["put_card_on_draw_pile_cost_0"],
)

TACTICIAN = Card(
    id="Tactician", name="Tactician", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=-2,  # Unplayable
    base_magic=1, upgrade_magic=1,  # Energy gain
    effects=["unplayable", "when_discarded_gain_energy"],
)

TERROR = Card(
    id="Terror", name="Terror", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=1, upgrade_cost=0, exhaust=True,
    base_magic=99,  # Vulnerable amount
    effects=["apply_vulnerable"],
)


# === UNCOMMON POWERS ===

ACCURACY = Card(
    id="Accuracy", name="Accuracy", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=4, upgrade_magic=2,  # Bonus Shiv damage
    effects=["shivs_deal_more_damage"],
)

CALTROPS = Card(
    id="Caltrops", name="Caltrops", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=3, upgrade_magic=2,  # Thorns amount
    effects=["gain_thorns"],
)

FOOTWORK = Card(
    id="Footwork", name="Footwork", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=2, upgrade_magic=1,  # Dexterity amount
    effects=["gain_dexterity"],
)

INFINITE_BLADES = Card(
    id="Infinite Blades", name="Infinite Blades", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    upgrade_innate=True,
    effects=["add_shiv_each_turn"],
)

NOXIOUS_FUMES = Card(
    id="Noxious Fumes", name="Noxious Fumes", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=2, upgrade_magic=1,  # Poison amount per turn
    effects=["apply_poison_all_each_turn"],
)

WELL_LAID_PLANS = Card(
    id="Well Laid Plans", name="Well-Laid Plans", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1,
    base_magic=1, upgrade_magic=1,  # Cards to retain
    effects=["retain_cards_each_turn"],
)


# === RARE ATTACKS ===

DIE_DIE_DIE = Card(
    id="Die Die Die", name="Die Die Die", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=1, base_damage=13, upgrade_damage=4,
    exhaust=True,
)

GLASS_KNIFE = Card(
    id="Glass Knife", name="Glass Knife", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.GREEN, cost=1, base_damage=8, upgrade_damage=4,
    base_magic=2,  # Hits twice
    effects=["damage_x_times", "reduce_damage_by_2"],
)

GRAND_FINALE = Card(
    id="Grand Finale", name="Grand Finale", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.ALL_ENEMY, cost=0, base_damage=50, upgrade_damage=10,
    effects=["only_playable_if_draw_pile_empty"],
)

UNLOAD = Card(
    id="Unload", name="Unload", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
    color=CardColor.GREEN, cost=1, base_damage=14, upgrade_damage=4,
    effects=["discard_non_attacks"],
)


# === RARE SKILLS ===

ADRENALINE = Card(
    id="Adrenaline", name="Adrenaline", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=0, exhaust=True,
    base_magic=1, upgrade_magic=1,  # Energy gain (shown in description)
    effects=["gain_energy", "draw_2"],
)

ALCHEMIZE = Card(
    id="Venomology", name="Alchemize", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    # Java cardID is "Venomology"
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, upgrade_cost=0, exhaust=True,
    effects=["obtain_random_potion"],
)

BULLET_TIME = Card(
    id="Bullet Time", name="Bullet Time", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=3, upgrade_cost=2,
    effects=["no_draw_this_turn", "cards_cost_0_this_turn"],
)

BURST = Card(
    id="Burst", name="Burst", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    base_magic=1, upgrade_magic=1,  # Number of skills to double
    effects=["double_next_skills"],
)

CORPSE_EXPLOSION = Card(
    id="Corpse Explosion", name="Corpse Explosion", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=2,
    base_magic=6, upgrade_magic=3,  # Poison amount
    effects=["apply_poison", "apply_corpse_explosion"],
)

DOPPELGANGER = Card(
    id="Doppelganger", name="Doppelganger", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=-1, exhaust=True,  # X cost
    # Upgraded: draw X+1 and gain X+1 energy next turn
    effects=["draw_x_next_turn", "gain_x_energy_next_turn"],
)

MALAISE = Card(
    id="Malaise", name="Malaise", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.ENEMY, cost=-1, exhaust=True,  # X cost
    # Upgraded: applies X+1 weak and X+1 strength down
    effects=["apply_weak_x", "apply_strength_down_x"],
)

NIGHTMARE = Card(
    id="Night Terror", name="Nightmare", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    # Java cardID is "Night Terror"
    color=CardColor.GREEN, target=CardTarget.NONE, cost=3, upgrade_cost=2, exhaust=True,
    base_magic=3,  # Copies made
    effects=["copy_card_to_hand_next_turn"],
)

PHANTASMAL_KILLER = Card(
    id="Phantasmal Killer", name="Phantasmal Killer", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    effects=["double_damage_next_turn"],
)

STORM_OF_STEEL = Card(
    id="Storm of Steel", name="Storm of Steel", card_type=CardType.SKILL, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.NONE, cost=1,
    # Upgraded: Shivs are upgraded
    effects=["discard_hand", "add_shivs_equal_to_discarded"],
)


# === RARE POWERS ===

AFTER_IMAGE = Card(
    id="After Image", name="After Image", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1,
    upgrade_innate=True,
    effects=["gain_1_block_per_card_played"],
)

A_THOUSAND_CUTS = Card(
    id="A Thousand Cuts", name="A Thousand Cuts", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=2,
    base_magic=1, upgrade_magic=1,  # Damage per card played
    effects=["deal_damage_per_card_played"],
)

ENVENOM = Card(
    id="Envenom", name="Envenom", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=2, upgrade_cost=1,
    effects=["attacks_apply_poison"],
)

TOOLS_OF_THE_TRADE = Card(
    id="Tools of the Trade", name="Tools of the Trade", card_type=CardType.POWER, rarity=CardRarity.RARE,
    color=CardColor.GREEN, target=CardTarget.SELF, cost=1, upgrade_cost=0,
    effects=["draw_1_discard_1_each_turn"],
)

WRAITH_FORM = Card(
    id="Wraith Form v2", name="Wraith Form", card_type=CardType.POWER, rarity=CardRarity.RARE,
    # Java cardID is "Wraith Form v2"
    color=CardColor.GREEN, target=CardTarget.SELF, cost=3,
    base_magic=2, upgrade_magic=1,  # Intangible turns
    effects=["gain_intangible", "lose_1_dexterity_each_turn"],
)


# === SPECIAL CARDS ===

SHIV = Card(
    id="Shiv", name="Shiv", card_type=CardType.ATTACK, rarity=CardRarity.SPECIAL,
    color=CardColor.COLORLESS, cost=0, base_damage=4, upgrade_damage=2,
    exhaust=True,
)


# Silent cards by ID
SILENT_CARDS: Dict[str, Card] = {
    # Basic
    "Strike_G": STRIKE_S,
    "Defend_G": DEFEND_S,
    "Neutralize": NEUTRALIZE,
    "Survivor": SURVIVOR_S,
    # Common Attacks
    "Bane": BANE,
    "Dagger Spray": DAGGER_SPRAY,
    "Dagger Throw": DAGGER_THROW,
    "Flying Knee": FLYING_KNEE,
    "Poisoned Stab": POISONED_STAB,
    "Quick Slash": QUICK_SLASH,
    "Slice": SLICE,
    "Underhanded Strike": SNEAKY_STRIKE,  # SneakyStrike.java
    "Sucker Punch": SUCKER_PUNCH,
    # Common Skills
    "Acrobatics": ACROBATICS,
    "Backflip": BACKFLIP,
    "Blade Dance": BLADE_DANCE,
    "Cloak And Dagger": CLOAK_AND_DAGGER,
    "Deadly Poison": DEADLY_POISON,
    "Deflect": DEFLECT,
    "Dodge and Roll": DODGE_AND_ROLL,
    "Outmaneuver": OUTMANEUVER,
    "PiercingWail": PIERCING_WAIL,
    "Prepared": PREPARED,
    # Uncommon Attacks
    "All Out Attack": ALL_OUT_ATTACK,
    "Backstab": BACKSTAB,
    "Choke": CHOKE,
    "Dash": DASH_S,
    "Endless Agony": ENDLESS_AGONY,
    "Eviscerate": EVISCERATE,
    "Finisher": FINISHER,
    "Flechettes": FLECHETTES,
    "Heel Hook": HEEL_HOOK,
    "Masterful Stab": MASTERFUL_STAB,
    "Predator": PREDATOR,
    "Riddle With Holes": RIDDLE_WITH_HOLES,
    "Skewer": SKEWER,
    # Uncommon Skills
    "Blur": BLUR,
    "Bouncing Flask": BOUNCING_FLASK,
    "Calculated Gamble": CALCULATED_GAMBLE,
    "Catalyst": CATALYST,
    "Concentrate": CONCENTRATE,
    "Crippling Poison": CRIPPLING_POISON,
    "Distraction": DISTRACTION,
    "Escape Plan": ESCAPE_PLAN,
    "Expertise": EXPERTISE,
    "Leg Sweep": LEG_SWEEP,
    "Reflex": REFLEX,
    "Setup": SETUP_S,
    "Tactician": TACTICIAN,
    "Terror": TERROR,
    # Uncommon Powers
    "Accuracy": ACCURACY,
    "Caltrops": CALTROPS,
    "Footwork": FOOTWORK,
    "Infinite Blades": INFINITE_BLADES,
    "Noxious Fumes": NOXIOUS_FUMES,
    "Well Laid Plans": WELL_LAID_PLANS,
    # Rare Attacks
    "Die Die Die": DIE_DIE_DIE,
    "Glass Knife": GLASS_KNIFE,
    "Grand Finale": GRAND_FINALE,
    "Unload": UNLOAD,
    # Rare Skills
    "Adrenaline": ADRENALINE,
    "Venomology": ALCHEMIZE,  # Alchemize.java has ID "Venomology"
    "Bullet Time": BULLET_TIME,
    "Burst": BURST,
    "Corpse Explosion": CORPSE_EXPLOSION,
    "Doppelganger": DOPPELGANGER,
    "Malaise": MALAISE,
    "Night Terror": NIGHTMARE,  # Nightmare.java has ID "Night Terror"
    "Phantasmal Killer": PHANTASMAL_KILLER,
    "Storm of Steel": STORM_OF_STEEL,
    # Rare Powers
    "After Image": AFTER_IMAGE,
    "A Thousand Cuts": A_THOUSAND_CUTS,
    "Envenom": ENVENOM,
    "Tools of the Trade": TOOLS_OF_THE_TRADE,
    "Wraith Form v2": WRAITH_FORM,  # WraithForm.java has ID "Wraith Form v2"
    # Special
    "Shiv": SHIV,
}


# All cards combined
ALL_CARDS: Dict[str, Card] = {
    **WATCHER_CARDS,
    **IRONCLAD_CARDS,
    **SILENT_CARDS,
    **DEFECT_CARDS,
    **COLORLESS_CARDS,
    **CURSE_CARDS,
    **STATUS_CARDS,
}

# Modern-name aliases for legacy Java IDs.
# Keep canonical IDs in ALL_CARDS; use aliases for lookups only.
CARD_ID_ALIASES = {
    "Rushdown": "Adaptation",
    "Foresight": "Wireheading",
    "Wraith Form": "Wraith Form v2",
    "WraithForm": "Wraith Form v2",
}


def resolve_card_id(card_id: str) -> str:
    """Resolve modern display IDs to canonical Java IDs."""
    return CARD_ID_ALIASES.get(card_id, card_id)


def get_card(card_id: str, upgraded: bool = False) -> Card:
    """Get a copy of a card by ID."""
    resolved_id = resolve_card_id(card_id)
    if resolved_id not in ALL_CARDS:
        raise ValueError(f"Unknown card: {card_id}")
    card = ALL_CARDS[resolved_id].copy()
    if upgraded:
        card.upgrade()
    return card


def normalize_card_id(java_id: str) -> tuple:
    """
    Normalize a card ID from mod JSONL format to engine format.

    Handles the '+' suffix convention for upgraded cards.
    Returns (base_id, upgraded) tuple. If base_id is not in ALL_CARDS,
    returns as-is and lets the caller handle it.
    """
    upgraded = False
    base_id = java_id
    if "+" in java_id:
        base_id = java_id.rsplit("+", 1)[0]
        upgraded = True
    base_id = resolve_card_id(base_id)
    return (base_id, upgraded)


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
