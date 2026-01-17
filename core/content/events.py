"""
Slay the Spire Events Database
Extracted from decompiled source code for RL decision-making.
Contains all events with their options, outcomes, and conditions.
"""

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Optional, Callable


class Act(Enum):
    ACT_1 = 1  # Exordium
    ACT_2 = 2  # The City
    ACT_3 = 3  # The Beyond
    ANY = 0    # Shrines can appear in any act


class OutcomeType(Enum):
    """Types of outcomes from event choices"""
    HP_CHANGE = auto()          # Current HP gain/loss
    MAX_HP_CHANGE = auto()      # Max HP gain/loss
    GOLD_CHANGE = auto()        # Gold gain/loss
    CARD_GAIN = auto()          # Obtain card(s)
    CARD_REMOVE = auto()        # Remove card from deck
    CARD_TRANSFORM = auto()     # Transform card(s)
    CARD_UPGRADE = auto()       # Upgrade card(s)
    RELIC_GAIN = auto()         # Obtain relic
    RELIC_LOSE = auto()         # Lose relic
    POTION_GAIN = auto()        # Obtain potion(s)
    CURSE_GAIN = auto()         # Obtain curse card
    COMBAT = auto()             # Enter combat
    CARD_CHOICE = auto()        # Choose from card rewards
    NOTHING = auto()            # No effect


@dataclass
class Outcome:
    """Represents a single outcome effect"""
    type: OutcomeType
    value: Optional[int] = None         # Numeric value (damage, gold, etc.)
    value_percent: Optional[float] = None  # Percentage of max HP, etc.
    card_id: Optional[str] = None       # Specific card ID
    relic_id: Optional[str] = None      # Specific relic ID
    rarity: Optional[str] = None        # Card/relic rarity
    count: int = 1                      # Number of items
    random: bool = False                # Is this outcome random?
    description: str = ""               # Human-readable description


@dataclass
class EventChoice:
    """Represents a single choice/option in an event"""
    index: int                          # Button index (0, 1, 2, etc.)
    description: str                    # What the option says
    outcomes: list[Outcome] = field(default_factory=list)

    # Conditions for this choice to be available
    requires_gold: Optional[int] = None
    requires_relic: Optional[str] = None
    requires_min_hp: Optional[int] = None
    requires_upgradable_cards: bool = False
    requires_removable_cards: bool = False
    requires_curse: bool = False
    requires_card_type: Optional[str] = None  # "ATTACK", "SKILL", "POWER"
    requires_potion: bool = False
    requires_non_basic_card: bool = False

    # Is this choice always available?
    always_available: bool = True

    # Probability information for RNG-based outcomes
    success_chance: Optional[float] = None  # For events with chance-based outcomes


@dataclass
class Event:
    """Represents a complete event"""
    id: str                             # Event ID from game
    name: str                           # Display name
    act: Act                            # Which act(s) this can appear in
    choices: list[EventChoice] = field(default_factory=list)

    # Event-level conditions
    requires_relic: Optional[str] = None
    requires_curse_in_deck: bool = False
    min_floor: Optional[int] = None
    max_floor: Optional[int] = None

    # Ascension modifiers
    has_ascension_modifier: bool = False
    ascension_threshold: int = 15       # A15+ typically

    description: str = ""


# =============================================================================
# ACT 1 (EXORDIUM) EVENTS
# =============================================================================

BIG_FISH = Event(
    id="Big Fish",
    name="Big Fish",
    act=Act.ACT_1,
    description="Encounter with a large fish offering food.",
    choices=[
        EventChoice(
            index=0,
            description="Banana: Heal 1/3 Max HP",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=0.33, description="Heal 1/3 max HP")
            ]
        ),
        EventChoice(
            index=1,
            description="Donut: Gain 5 Max HP",
            outcomes=[
                Outcome(OutcomeType.MAX_HP_CHANGE, value=5, description="+5 Max HP")
            ]
        ),
        EventChoice(
            index=2,
            description="Box: Gain random relic, obtain Regret curse",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic (any tier)"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Regret", description="Obtain Regret curse")
            ]
        ),
    ]
)

CLERIC = Event(
    id="The Cleric",
    name="The Cleric",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="A cleric offers healing and purification services.",
    choices=[
        EventChoice(
            index=0,
            description="Heal: Pay 35 gold, heal 25% Max HP",
            requires_gold=35,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-35, description="Pay 35 gold"),
                Outcome(OutcomeType.HP_CHANGE, value_percent=0.25, description="Heal 25% max HP")
            ]
        ),
        EventChoice(
            index=1,
            description="Purify: Pay 50/75 gold, remove a card",
            requires_gold=50,  # 75 on A15+
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-50, description="Pay 50g (75g A15+)"),
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

DEAD_ADVENTURER = Event(
    id="Dead Adventurer",
    name="Dead Adventurer",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="Search a corpse with increasing chance of elite fight.",
    choices=[
        EventChoice(
            index=0,
            description="Search: 25%/35% chance of elite fight, rewards on success",
            success_chance=0.75,  # 75%/65% chance of NOT fighting on first search
            outcomes=[
                # Success outcomes (no fight): Gold, nothing, or relic (random order)
                Outcome(OutcomeType.GOLD_CHANGE, value=30, random=True, description="30 gold (chance)"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic (chance)"),
                Outcome(OutcomeType.NOTHING, random=True, description="Nothing (chance)"),
                # Failure: Elite fight (3 Sentries, Gremlin Nob, or Lagavulin)
                Outcome(OutcomeType.COMBAT, random=True, description="Elite fight on failure")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

GOLDEN_IDOL = Event(
    id="Golden Idol",
    name="Golden Idol",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="Take a golden idol, then choose how to escape the trap.",
    choices=[
        EventChoice(
            index=0,
            description="Take Idol: Obtain Golden Idol, then choose escape",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Golden Idol", description="Obtain Golden Idol")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

# Sub-choices after taking the idol
GOLDEN_IDOL_ESCAPE_INJURY = EventChoice(
    index=0,
    description="Outrun: Obtain Injury curse",
    outcomes=[
        Outcome(OutcomeType.CURSE_GAIN, card_id="Injury", description="Obtain Injury curse")
    ]
)

GOLDEN_IDOL_ESCAPE_DAMAGE = EventChoice(
    index=1,
    description="Smash: Take 25%/35% Max HP damage",
    outcomes=[
        Outcome(OutcomeType.HP_CHANGE, value_percent=-0.25, description="Take 25% max HP damage (35% A15+)")
    ]
)

GOLDEN_IDOL_ESCAPE_MAX_HP = EventChoice(
    index=2,
    description="Hide: Lose 8%/10% Max HP permanently",
    outcomes=[
        Outcome(OutcomeType.MAX_HP_CHANGE, value_percent=-0.08, description="Lose 8% max HP (10% A15+)")
    ]
)

GOLDEN_WING = Event(
    id="Golden Wing",
    name="Golden Wing",
    act=Act.ACT_1,
    description="A bird offers to remove a card for HP, or gold if you have a 10+ damage card.",
    choices=[
        EventChoice(
            index=0,
            description="Feed: Take 7 damage, remove a card",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-7, description="Take 7 damage"),
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Offer Attack (requires 10+ damage card): Gain 50-80 gold",
            requires_card_type="ATTACK",  # Specifically needs card with 10+ base damage
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=65, random=True, description="Gain 50-80 gold")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

GOOP_PUDDLE = Event(
    id="World of Goop",
    name="World of Goop",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="A puddle of slime with gold inside.",
    choices=[
        EventChoice(
            index=0,
            description="Gather Gold: Gain 75 gold, take 11 damage",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=75, description="Gain 75 gold"),
                Outcome(OutcomeType.HP_CHANGE, value=-11, description="Take 11 damage")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave: Lose 20-50/35-75 gold",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-35, random=True,
                       description="Lose 20-50 gold (35-75 A15+)")
            ]
        ),
    ]
)

LIVING_WALL = Event(
    id="Living Wall",
    name="Living Wall",
    act=Act.ACT_1,
    description="A wall offers to modify your deck.",
    choices=[
        EventChoice(
            index=0,
            description="Forget: Remove a card",
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Change: Transform a card",
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_TRANSFORM, count=1, description="Transform a card")
            ]
        ),
        EventChoice(
            index=2,
            description="Grow: Upgrade a card",
            requires_upgradable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_UPGRADE, count=1, description="Upgrade a card")
            ]
        ),
    ]
)

MUSHROOMS = Event(
    id="Mushrooms",
    name="Mushrooms",
    act=Act.ACT_1,
    description="Fight mushroom enemies or heal with a curse.",
    choices=[
        EventChoice(
            index=0,
            description="Fight: Combat against mushroom enemies for Odd Mushroom relic",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight The Mushroom Lair"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Odd Mushroom", description="Odd Mushroom on win"),
                Outcome(OutcomeType.GOLD_CHANGE, value=25, random=True, description="20-30 gold on win")
            ]
        ),
        EventChoice(
            index=1,
            description="Eat: Heal 25% Max HP, obtain Parasite curse",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=0.25, description="Heal 25% max HP"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Parasite", description="Obtain Parasite curse")
            ]
        ),
    ]
)

SCRAP_OOZE = Event(
    id="Scrap Ooze",
    name="Scrap Ooze",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="Reach into ooze repeatedly for chance at relic.",
    choices=[
        EventChoice(
            index=0,
            description="Reach In: Take 3/5 damage (escalates), 25%+ chance for relic",
            success_chance=0.25,  # Starts at 25%, +10% each attempt
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-3, description="Take 3 damage (5 A15+), +1 each attempt"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic on success")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

SHINING_LIGHT = Event(
    id="Shining Light",
    name="Shining Light",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="Enter light to upgrade 2 random cards.",
    choices=[
        EventChoice(
            index=0,
            description="Enter: Take 20%/30% Max HP damage, upgrade 2 random cards",
            requires_upgradable_cards=True,
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.20, description="Take 20% max HP damage (30% A15+)"),
                Outcome(OutcomeType.CARD_UPGRADE, count=2, random=True, description="Upgrade 2 random cards")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

SSSSERPENT = Event(
    id="Liars Game",
    name="Sssserpent",
    act=Act.ACT_1,
    has_ascension_modifier=True,
    description="A serpent offers gold for a curse.",
    choices=[
        EventChoice(
            index=0,
            description="Agree: Gain 175/150 gold, obtain Doubt curse",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=175, description="Gain 175 gold (150 A15+)"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Doubt", description="Obtain Doubt curse")
            ]
        ),
        EventChoice(
            index=1,
            description="Disagree: Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)


# =============================================================================
# ACT 2 (THE CITY) EVENTS
# =============================================================================

ADDICT = Event(
    id="Addict",
    name="Addict",
    act=Act.ACT_2,
    description="A desperate person needs money or you can steal.",
    choices=[
        EventChoice(
            index=0,
            description="Pay 85 gold: Obtain random relic",
            requires_gold=85,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-85, description="Pay 85 gold"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic")
            ]
        ),
        EventChoice(
            index=1,
            description="Steal: Obtain random relic and Shame curse",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Shame", description="Obtain Shame curse")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

BACK_TO_BASICS = Event(
    id="Back to Basics",
    name="Back to Basics",
    act=Act.ACT_2,
    description="Remove all Strikes and Defends, or upgrade all Strikes and Defends.",
    choices=[
        EventChoice(
            index=0,
            description="Simplicity: Remove all Strikes and Defends",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Remove all Strikes and Defends")
            ]
        ),
        EventChoice(
            index=1,
            description="Elegance: Upgrade all Strikes and Defends",
            outcomes=[
                Outcome(OutcomeType.CARD_UPGRADE, description="Upgrade all Strikes and Defends")
            ]
        ),
    ]
)

BEGGAR = Event(
    id="Beggar",
    name="The Beggar",
    act=Act.ACT_2,
    description="A beggar asks for gold.",
    choices=[
        EventChoice(
            index=0,
            description="Give 75 gold: Obtain random relic",
            requires_gold=75,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-75, description="Pay 75 gold"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic")
            ]
        ),
        EventChoice(
            index=1,
            description="Steal: Gain ~75 gold and Doubt curse",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=75, random=True, description="Gain ~75 gold"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Doubt", description="Obtain Doubt curse")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

COLOSSEUM = Event(
    id="Colosseum",
    name="The Colosseum",
    act=Act.ACT_2,
    description="Fight in the colosseum for rewards.",
    choices=[
        EventChoice(
            index=0,
            description="Enter: Fight Slavers, then option to fight 2 Nobs for big rewards",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight Colosseum Slavers"),
                # After first fight: option to fight 2 Nobs or leave
            ]
        ),
    ]
)

# Sub-choices after first colosseum fight
COLOSSEUM_FIGHT_NOBS = EventChoice(
    index=0,
    description="Fight Nobs: Fight 2 Nobs for rare relic, uncommon relic, 100 gold",
    outcomes=[
        Outcome(OutcomeType.COMBAT, description="Fight 2 Taskmaster (Nobs)"),
        Outcome(OutcomeType.RELIC_GAIN, rarity="RARE", description="Rare relic on win"),
        Outcome(OutcomeType.RELIC_GAIN, rarity="UNCOMMON", description="Uncommon relic on win"),
        Outcome(OutcomeType.GOLD_CHANGE, value=100, description="100 gold on win")
    ]
)

COLOSSEUM_FLEE = EventChoice(
    index=1,
    description="Flee: Leave without fighting Nobs",
    outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
)

CURSED_TOME = Event(
    id="Cursed Tome",
    name="Cursed Tome",
    act=Act.ACT_2,
    has_ascension_modifier=True,
    description="Read a cursed book for a relic, taking damage each page.",
    choices=[
        EventChoice(
            index=0,
            description="Read: Take 1+2+3+10/15 damage total for a random book relic",
            outcomes=[
                # Cumulative damage: 1 + 2 + 3 + 10 = 16 (or 21 on A15+)
                Outcome(OutcomeType.HP_CHANGE, value=-16, description="Take 16 damage (21 A15+)"),
                Outcome(OutcomeType.RELIC_GAIN, random=True,
                       description="Necronomicon, Enchiridion, or Nilry's Codex")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

DRUG_DEALER = Event(
    id="Drug Dealer",
    name="Augmenter",
    act=Act.ACT_2,
    description="Get J.A.X., transform cards, or get Mutagenic Strength.",
    choices=[
        EventChoice(
            index=0,
            description="Ingest Mutagens: Obtain J.A.X. card",
            outcomes=[
                Outcome(OutcomeType.CARD_GAIN, card_id="J.A.X.", description="Obtain J.A.X.")
            ]
        ),
        EventChoice(
            index=1,
            description="Become Test Subject: Transform 2 cards",
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_TRANSFORM, count=2, description="Transform 2 cards")
            ]
        ),
        EventChoice(
            index=2,
            description="Inject Mutagens: Obtain Mutagenic Strength relic",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, relic_id="MutagenicStrength",
                       description="Obtain Mutagenic Strength")
            ]
        ),
    ]
)

FORGOTTEN_ALTAR = Event(
    id="Forgotten Altar",
    name="Forgotten Altar",
    act=Act.ACT_2,
    has_ascension_modifier=True,
    description="An altar that wants the Golden Idol.",
    choices=[
        EventChoice(
            index=0,
            description="Offer Golden Idol: Trade for Bloody Idol",
            requires_relic="Golden Idol",
            outcomes=[
                Outcome(OutcomeType.RELIC_LOSE, relic_id="Golden Idol", description="Lose Golden Idol"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Bloody Idol", description="Obtain Bloody Idol")
            ]
        ),
        EventChoice(
            index=1,
            description="Sacrifice: Gain 5 Max HP, take 25%/35% damage",
            outcomes=[
                Outcome(OutcomeType.MAX_HP_CHANGE, value=5, description="+5 Max HP"),
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.25,
                       description="Take 25% max HP damage (35% A15+)")
            ]
        ),
        EventChoice(
            index=2,
            description="Desecrate: Obtain Decay curse",
            outcomes=[
                Outcome(OutcomeType.CURSE_GAIN, card_id="Decay", description="Obtain Decay curse")
            ]
        ),
    ]
)

GHOSTS = Event(
    id="Ghosts",
    name="Ghosts",
    act=Act.ACT_2,
    has_ascension_modifier=True,
    description="Ghosts offer Apparition cards for max HP.",
    choices=[
        EventChoice(
            index=0,
            description="Accept: Lose 50% Max HP, gain 5/3 Apparition cards",
            outcomes=[
                Outcome(OutcomeType.MAX_HP_CHANGE, value_percent=-0.50, description="Lose 50% Max HP"),
                Outcome(OutcomeType.CARD_GAIN, card_id="Apparition", count=5,
                       description="Obtain 5 Apparitions (3 on A15+)")
            ]
        ),
        EventChoice(
            index=1,
            description="Refuse: Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

KNOWING_SKULL = Event(
    id="Knowing Skull",
    name="Knowing Skull",
    act=Act.ACT_2,
    description="A skull offers items for HP. Costs escalate.",
    choices=[
        EventChoice(
            index=0,
            description="Potion: Pay 6 HP (escalates) for random potion",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-6, description="Take 6+ HP damage"),
                Outcome(OutcomeType.POTION_GAIN, random=True, description="Random potion")
            ]
        ),
        EventChoice(
            index=1,
            description="Gold: Pay 6 HP (escalates) for 90 gold",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-6, description="Take 6+ HP damage"),
                Outcome(OutcomeType.GOLD_CHANGE, value=90, description="Gain 90 gold")
            ]
        ),
        EventChoice(
            index=2,
            description="Card: Pay 6 HP (escalates) for random colorless uncommon",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-6, description="Take 6+ HP damage"),
                Outcome(OutcomeType.CARD_GAIN, rarity="UNCOMMON", description="Random colorless uncommon")
            ]
        ),
        EventChoice(
            index=3,
            description="Leave: Pay 6 HP to leave",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-6, description="Take 6 HP damage to leave")
            ]
        ),
    ]
)

MASKED_BANDITS = Event(
    id="Masked Bandits",
    name="Masked Bandits",
    act=Act.ACT_2,
    description="Bandits demand all your gold.",
    choices=[
        EventChoice(
            index=0,
            description="Pay: Lose all gold",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, description="Lose all gold")
            ]
        ),
        EventChoice(
            index=1,
            description="Fight: Combat against 3 bandits for gold and relic",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight 3 Pointy (Bandits)"),
                Outcome(OutcomeType.RELIC_GAIN, rarity="COMMON", description="Common relic on win"),
                Outcome(OutcomeType.GOLD_CHANGE, value=50, random=True, description="Gold on win")
            ]
        ),
    ]
)

NEST = Event(
    id="Nest",
    name="The Nest",
    act=Act.ACT_2,
    description="A nest with valuable contents.",
    choices=[
        EventChoice(
            index=0,
            description="Take: Obtain 99 gold and Ritual Dagger card",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=99, description="Gain 99 gold"),
                Outcome(OutcomeType.CARD_GAIN, card_id="Ritual Dagger", description="Obtain Ritual Dagger")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

THE_JOUST = Event(
    id="The Joust",
    name="The Joust",
    act=Act.ACT_2,
    description="Bet on a joust outcome.",
    choices=[
        EventChoice(
            index=0,
            description="Bet on Owner (30% win): Win = 250 gold, Lose = nothing",
            success_chance=0.30,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=250, random=True, description="250 gold if win")
            ]
        ),
        EventChoice(
            index=1,
            description="Bet on Murderer (70% win): Win = 50 gold, Lose = nothing",
            success_chance=0.70,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=50, random=True, description="50 gold if win")
            ]
        ),
    ]
)

THE_LIBRARY = Event(
    id="The Library",
    name="The Library",
    act=Act.ACT_2,
    description="Choose between reading or sleeping.",
    choices=[
        EventChoice(
            index=0,
            description="Read: Choose 1 of 20 cards to obtain",
            outcomes=[
                Outcome(OutcomeType.CARD_CHOICE, count=20, description="Choose 1 of 20 cards")
            ]
        ),
        EventChoice(
            index=1,
            description="Sleep: Heal to full HP",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, description="Heal to full HP")
            ]
        ),
    ]
)

THE_MAUSOLEUM = Event(
    id="The Mausoleum",
    name="The Mausoleum",
    act=Act.ACT_2,
    description="Open a coffin for possible rewards.",
    choices=[
        EventChoice(
            index=0,
            description="Open: 50% chance for relic, 50% chance for curse",
            success_chance=0.50,
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic (50%)"),
                Outcome(OutcomeType.CURSE_GAIN, random=True, description="Random curse (50%)")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

VAMPIRES = Event(
    id="Vampires",
    name="Vampires(?)",
    act=Act.ACT_2,
    description="Vampires offer to transform you.",
    choices=[
        EventChoice(
            index=0,
            description="Accept: Remove all Strikes, gain 5 Bites, lose 30% Max HP",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Remove all Strikes"),
                Outcome(OutcomeType.CARD_GAIN, card_id="Bite", count=5, description="Obtain 5 Bites"),
                Outcome(OutcomeType.MAX_HP_CHANGE, value_percent=-0.30, description="Lose 30% Max HP")
            ]
        ),
        EventChoice(
            index=1,
            description="Refuse: Fight 3 Spikers and 2 Vampires",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight vampires")
            ]
        ),
    ]
)


# =============================================================================
# ACT 3 (THE BEYOND) EVENTS
# =============================================================================

FALLING = Event(
    id="Falling",
    name="Falling",
    act=Act.ACT_3,
    description="Must lose a card of a specific type.",
    choices=[
        EventChoice(
            index=0,
            description="Land on Skill: Lose random Skill card",
            requires_card_type="SKILL",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Lose random Skill")
            ]
        ),
        EventChoice(
            index=1,
            description="Land on Power: Lose random Power card",
            requires_card_type="POWER",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Lose random Power")
            ]
        ),
        EventChoice(
            index=2,
            description="Land on Attack: Lose random Attack card",
            requires_card_type="ATTACK",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Lose random Attack")
            ]
        ),
    ]
)

MIND_BLOOM = Event(
    id="MindBloom",
    name="Mind Bloom",
    act=Act.ACT_3,
    has_ascension_modifier=True,
    description="A powerful event with major rewards and costs.",
    choices=[
        EventChoice(
            index=0,
            description="I am War: Fight random Act 1 boss for rare relic",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight Act 1 boss"),
                Outcome(OutcomeType.RELIC_GAIN, rarity="RARE", description="Rare relic on win"),
                Outcome(OutcomeType.GOLD_CHANGE, value=50, description="50 gold on win (25 A13+)")
            ]
        ),
        EventChoice(
            index=1,
            description="I am Awake: Upgrade all cards, obtain Mark of the Bloom",
            outcomes=[
                Outcome(OutcomeType.CARD_UPGRADE, description="Upgrade ALL cards"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Mark of the Bloom",
                       description="Obtain Mark of the Bloom (can't heal)")
            ]
        ),
        EventChoice(
            index=2,
            description="I am Rich (floors 1-40): Gain 999 gold, obtain 2 Normality curses",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=999, description="Gain 999 gold"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Normality", count=2,
                       description="Obtain 2 Normality curses")
            ]
        ),
        # Alternative option on floors 41-50
        EventChoice(
            index=2,
            description="I am Healthy (floors 41-50): Full heal, obtain Doubt curse",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, description="Heal to full"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Doubt", description="Obtain Doubt curse")
            ]
        ),
    ]
)

MOAI_HEAD = Event(
    id="The Moai Head",
    name="The Moai Head",
    act=Act.ACT_3,
    has_ascension_modifier=True,
    description="A giant head offers healing or wants the Golden Idol.",
    choices=[
        EventChoice(
            index=0,
            description="Enter: Lose 12.5%/18% Max HP, heal to full",
            outcomes=[
                Outcome(OutcomeType.MAX_HP_CHANGE, value_percent=-0.125,
                       description="Lose 12.5% max HP (18% A15+)"),
                Outcome(OutcomeType.HP_CHANGE, description="Heal to full HP")
            ]
        ),
        EventChoice(
            index=1,
            description="Offer Golden Idol: Gain 333 gold",
            requires_relic="Golden Idol",
            outcomes=[
                Outcome(OutcomeType.RELIC_LOSE, relic_id="Golden Idol", description="Lose Golden Idol"),
                Outcome(OutcomeType.GOLD_CHANGE, value=333, description="Gain 333 gold")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

MYSTERIOUS_SPHERE = Event(
    id="Mysterious Sphere",
    name="Mysterious Sphere",
    act=Act.ACT_3,
    description="A mysterious orb with combat inside.",
    choices=[
        EventChoice(
            index=0,
            description="Open: Fight 2 Orb Walkers for rare relic",
            outcomes=[
                Outcome(OutcomeType.COMBAT, description="Fight 2 Orb Walkers"),
                Outcome(OutcomeType.RELIC_GAIN, rarity="RARE", description="Rare relic on win")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

SECRET_PORTAL = Event(
    id="Secret Portal",
    name="Secret Portal",
    act=Act.ACT_3,
    description="A portal that leads directly to the boss.",
    choices=[
        EventChoice(
            index=0,
            description="Enter: Skip to Act 3 boss immediately",
            outcomes=[
                Outcome(OutcomeType.NOTHING, description="Go directly to Act 3 boss")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Continue normal path")]
        ),
    ]
)

SENSORY_STONE = Event(
    id="Sensory Stone",
    name="Sensory Stone",
    act=Act.ACT_3,
    description="Relive memories for colorless cards.",
    choices=[
        EventChoice(
            index=0,
            description="Touch: Obtain 1-3 random colorless cards based on relics",
            outcomes=[
                # 1 card base, +1 for each of Circlet, Enchiridion, Nilry's Codex
                Outcome(OutcomeType.CARD_GAIN, count=1, description="1 colorless card (+1 per memory relic)")
            ]
        ),
    ]
)

TOMB_OF_LORD_RED_MASK = Event(
    id="Tomb of Lord Red Mask",
    name="Tomb of Lord Red Mask",
    act=Act.ACT_3,
    description="A tomb with gold but requires the Red Mask.",
    choices=[
        EventChoice(
            index=0,
            description="Don the Red Mask: Obtain Red Mask relic",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Red Mask", description="Obtain Red Mask")
            ]
        ),
        EventChoice(
            index=1,
            description="Offer Gold (requires Red Mask): Lose all gold, gain 222 gold per relic",
            requires_relic="Red Mask",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, description="Lose all gold"),
                Outcome(OutcomeType.GOLD_CHANGE, description="Gain 222 gold per relic")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

WINDING_HALLS = Event(
    id="Winding Halls",
    name="Winding Halls",
    act=Act.ACT_3,
    has_ascension_modifier=True,
    description="Lost in winding halls with three paths.",
    choices=[
        EventChoice(
            index=0,
            description="Embrace Madness: Take 12.5%/18% damage, gain 2 Madness cards",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.125,
                       description="Take 12.5% max HP damage (18% A15+)"),
                Outcome(OutcomeType.CARD_GAIN, card_id="Madness", count=2, description="Obtain 2 Madness")
            ]
        ),
        EventChoice(
            index=1,
            description="Retrace Steps: Heal 25%/20%, gain Writhe curse",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=0.25, description="Heal 25% (20% A15+)"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Writhe", description="Obtain Writhe curse")
            ]
        ),
        EventChoice(
            index=2,
            description="Press On: Lose 5% Max HP",
            outcomes=[
                Outcome(OutcomeType.MAX_HP_CHANGE, value_percent=-0.05, description="Lose 5% Max HP")
            ]
        ),
    ]
)


# =============================================================================
# SHRINE EVENTS (ANY ACT)
# =============================================================================

ACCURSED_BLACKSMITH = Event(
    id="Accursed Blacksmith",
    name="Accursed Blacksmith",
    act=Act.ANY,
    description="An abandoned forge.",
    choices=[
        EventChoice(
            index=0,
            description="Forge: Upgrade a card",
            requires_upgradable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_UPGRADE, count=1, description="Upgrade a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Rummage: Obtain Warped Tongs relic and Pain curse",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Warped Tongs", description="Obtain Warped Tongs"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Pain", description="Obtain Pain curse")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

BONFIRE_ELEMENTALS = Event(
    id="Bonfire Elementals",
    name="Bonfire Elementals",
    act=Act.ANY,
    description="Spirits at a bonfire want offerings.",
    choices=[
        EventChoice(
            index=0,
            description="Offer card based on rarity:",
            requires_removable_cards=True,
            outcomes=[
                # Curse: Spirit Poop relic
                # Basic: Nothing
                # Common/Special: Heal 5
                # Uncommon: Heal to full
                # Rare: Heal to full + 10 Max HP
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card"),
                Outcome(OutcomeType.HP_CHANGE, description="Heal based on rarity"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Spirit Poop",
                       description="Spirit Poop if curse offered")
            ]
        ),
    ]
)

DESIGNER = Event(
    id="Designer",
    name="Designer In-Spire",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="A fashion designer offers services for gold.",
    choices=[
        EventChoice(
            index=0,
            description="Adjustments (40/50 gold): Upgrade 1 card OR 2 random cards",
            requires_gold=40,  # 50 on A15+
            requires_upgradable_cards=True,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-40, description="Pay 40 gold (50 A15+)"),
                Outcome(OutcomeType.CARD_UPGRADE, count=1, random=True,
                       description="Upgrade 1 card or 2 random")
            ]
        ),
        EventChoice(
            index=1,
            description="Clean Up (60/75 gold): Remove 1 card OR Transform 2 cards",
            requires_gold=60,  # 75 on A15+
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-60, description="Pay 60 gold (75 A15+)"),
                Outcome(OutcomeType.CARD_REMOVE, count=1, random=True,
                       description="Remove 1 or transform 2")
            ]
        ),
        EventChoice(
            index=2,
            description="Full Service (90/110 gold): Remove 1 card, upgrade 1 random card",
            requires_gold=90,  # 110 on A15+
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-90, description="Pay 90 gold (110 A15+)"),
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove 1 card"),
                Outcome(OutcomeType.CARD_UPGRADE, count=1, random=True, description="Upgrade 1 random")
            ]
        ),
        EventChoice(
            index=3,
            description="Punch: Take 3/5 damage, leave",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value=-3, description="Take 3 damage (5 A15+)")
            ]
        ),
    ]
)

DUPLICATOR = Event(
    id="Duplicator",
    name="Duplicator",
    act=Act.ANY,
    description="A shrine that duplicates a card.",
    choices=[
        EventChoice(
            index=0,
            description="Duplicate: Copy a card from your deck",
            outcomes=[
                Outcome(OutcomeType.CARD_GAIN, description="Duplicate a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

FACE_TRADER = Event(
    id="FaceTrader",
    name="Face Trader",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="A strange being that trades faces.",
    choices=[
        EventChoice(
            index=0,
            description="Touch: Take 10% Max HP damage, gain 50/75 gold",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.10, description="Take 10% max HP damage"),
                Outcome(OutcomeType.GOLD_CHANGE, value=75, description="Gain 75 gold (50 A15+)")
            ]
        ),
        EventChoice(
            index=1,
            description="Trade: Obtain random face relic",
            outcomes=[
                Outcome(OutcomeType.RELIC_GAIN, random=True,
                       description="Random mask relic (Cultist, Cleric, Gremlin, N'loth, Sssserpent)")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

FOUNTAIN_OF_CURSE_REMOVAL = Event(
    id="Fountain of Cleansing",
    name="Fountain of Cleansing",
    act=Act.ANY,
    requires_curse_in_deck=True,
    description="A fountain that removes all removable curses.",
    choices=[
        EventChoice(
            index=0,
            description="Drink: Remove all curses (except Ascender's Bane, CurseOfTheBell, Necronomicurse)",
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Remove all removable curses")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

GOLD_SHRINE = Event(
    id="Golden Shrine",
    name="Golden Shrine",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="A shrine offering gold.",
    choices=[
        EventChoice(
            index=0,
            description="Pray: Gain 100/50 gold",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=100, description="Gain 100 gold (50 A15+)")
            ]
        ),
        EventChoice(
            index=1,
            description="Desecrate: Gain 275 gold, obtain Regret curse",
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=275, description="Gain 275 gold"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Regret", description="Obtain Regret curse")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

GREMLIN_MATCH_GAME = Event(
    id="Match and Keep!",
    name="Gremlin Match Game",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="Memory match game for cards.",
    choices=[
        EventChoice(
            index=0,
            description="Play: Match pairs in 5 attempts, keep matched cards",
            outcomes=[
                # 12 cards (6 pairs): Rare, Uncommon, Common, Colorless Uncommon (or Curse on A15+), Curse, Starting card
                Outcome(OutcomeType.CARD_GAIN, random=True,
                       description="Keep matched pairs (Rare, Uncommon, Common, etc.)")
            ]
        ),
    ]
)

GREMLIN_WHEEL_GAME = Event(
    id="Wheel of Change",
    name="Gremlin Wheel Game",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="Spin the wheel for random rewards.",
    choices=[
        EventChoice(
            index=0,
            description="Spin: Random outcome (gold, relic, heal, curse, card remove, or damage)",
            outcomes=[
                # Equal chance of 6 outcomes:
                Outcome(OutcomeType.GOLD_CHANGE, random=True, description="Gold (100/200/300 by act)"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic"),
                Outcome(OutcomeType.HP_CHANGE, description="Heal to full"),
                Outcome(OutcomeType.CURSE_GAIN, card_id="Decay", description="Obtain Decay curse"),
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card"),
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.10,
                       description="Take 10% max HP damage (15% A15+)")
            ]
        ),
    ]
)

LAB = Event(
    id="Lab",
    name="The Lab",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="A laboratory with potions.",
    choices=[
        EventChoice(
            index=0,
            description="Enter: Obtain 3/2 random potions",
            outcomes=[
                Outcome(OutcomeType.POTION_GAIN, count=3, random=True,
                       description="Obtain 3 random potions (2 on A15+)")
            ]
        ),
    ]
)

NLOTH = Event(
    id="N'loth",
    name="N'loth",
    act=Act.ANY,
    description="N'loth wants to trade for your relics.",
    choices=[
        EventChoice(
            index=0,
            description="Trade Relic 1: Lose relic, gain N'loth's Gift",
            outcomes=[
                Outcome(OutcomeType.RELIC_LOSE, description="Lose random relic 1"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Nloth's Gift", description="Obtain N'loth's Gift")
            ]
        ),
        EventChoice(
            index=1,
            description="Trade Relic 2: Lose relic, gain N'loth's Gift",
            outcomes=[
                Outcome(OutcomeType.RELIC_LOSE, description="Lose random relic 2"),
                Outcome(OutcomeType.RELIC_GAIN, relic_id="Nloth's Gift", description="Obtain N'loth's Gift")
            ]
        ),
        EventChoice(
            index=2,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

NOTE_FOR_YOURSELF = Event(
    id="NoteForYourself",
    name="Note For Yourself",
    act=Act.ANY,
    description="A note from a previous run with a card.",
    choices=[
        EventChoice(
            index=0,
            description="Take: Take the card, leave a card from your deck for next run",
            outcomes=[
                Outcome(OutcomeType.CARD_GAIN, description="Obtain saved card"),
                Outcome(OutcomeType.CARD_REMOVE, description="Save a card for next run")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

PURIFICATION_SHRINE = Event(
    id="Purifier",
    name="Purifier",
    act=Act.ANY,
    description="A shrine that removes a card.",
    choices=[
        EventChoice(
            index=0,
            description="Pray: Remove a card",
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, count=1, description="Remove a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

TRANSMOGRIFIER = Event(
    id="Transmorgrifier",
    name="Transmogrifier",
    act=Act.ANY,
    description="A shrine that transforms a card.",
    choices=[
        EventChoice(
            index=0,
            description="Pray: Transform a card",
            requires_removable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_TRANSFORM, count=1, description="Transform a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

UPGRADE_SHRINE = Event(
    id="Upgrade Shrine",
    name="Upgrade Shrine",
    act=Act.ANY,
    description="A shrine that upgrades a card.",
    choices=[
        EventChoice(
            index=0,
            description="Pray: Upgrade a card",
            requires_upgradable_cards=True,
            outcomes=[
                Outcome(OutcomeType.CARD_UPGRADE, count=1, description="Upgrade a card")
            ]
        ),
        EventChoice(
            index=1,
            description="Leave",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

WE_MEET_AGAIN = Event(
    id="WeMeetAgain",
    name="We Meet Again",
    act=Act.ANY,
    description="A familiar face wants something in exchange for a relic.",
    choices=[
        EventChoice(
            index=0,
            description="Give Potion: Lose a potion, gain random relic",
            requires_potion=True,
            outcomes=[
                Outcome(OutcomeType.POTION_GAIN, count=-1, description="Lose a potion"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic")
            ]
        ),
        EventChoice(
            index=1,
            description="Give Gold: Lose 50-150 gold, gain random relic",
            requires_gold=50,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, random=True, description="Lose 50-150 gold"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic")
            ]
        ),
        EventChoice(
            index=2,
            description="Give Card: Lose non-basic card, gain random relic",
            requires_non_basic_card=True,
            outcomes=[
                Outcome(OutcomeType.CARD_REMOVE, description="Lose random non-basic card"),
                Outcome(OutcomeType.RELIC_GAIN, random=True, description="Random relic")
            ]
        ),
        EventChoice(
            index=3,
            description="Attack: Leave (with dramatic effect)",
            outcomes=[Outcome(OutcomeType.NOTHING, description="Leave")]
        ),
    ]
)

WOMAN_IN_BLUE = Event(
    id="The Woman in Blue",
    name="The Woman in Blue",
    act=Act.ANY,
    has_ascension_modifier=True,
    description="A woman selling potions.",
    choices=[
        EventChoice(
            index=0,
            description="Buy 1 Potion: Pay 20 gold",
            requires_gold=20,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-20, description="Pay 20 gold"),
                Outcome(OutcomeType.POTION_GAIN, count=1, random=True, description="1 random potion")
            ]
        ),
        EventChoice(
            index=1,
            description="Buy 2 Potions: Pay 30 gold",
            requires_gold=30,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-30, description="Pay 30 gold"),
                Outcome(OutcomeType.POTION_GAIN, count=2, random=True, description="2 random potions")
            ]
        ),
        EventChoice(
            index=2,
            description="Buy 3 Potions: Pay 40 gold",
            requires_gold=40,
            outcomes=[
                Outcome(OutcomeType.GOLD_CHANGE, value=-40, description="Pay 40 gold"),
                Outcome(OutcomeType.POTION_GAIN, count=3, random=True, description="3 random potions")
            ]
        ),
        EventChoice(
            index=3,
            description="Leave: Take 5% Max HP damage (A15+ only) or free leave",
            outcomes=[
                Outcome(OutcomeType.HP_CHANGE, value_percent=-0.05,
                       description="Take 5% max HP damage on A15+ (free otherwise)")
            ]
        ),
    ]
)


# =============================================================================
# NEOW EVENT (Start of run)
# =============================================================================

@dataclass
class NeowBonus:
    """Represents a Neow bonus option"""
    type: str
    description: str
    category: int  # 0=Small benefits, 1=Medium benefits, 2=With drawback, 3=Boss relic swap
    drawback: Optional[str] = None

    # Specific values
    hp_bonus: Optional[int] = None
    gold_bonus: Optional[int] = None
    card_count: int = 0
    is_rare: bool = False


# Category 0: Small benefits (no drawback)
NEOW_THREE_CARDS = NeowBonus("THREE_CARDS", "Choose a card to obtain", 0, card_count=3)
NEOW_ONE_RANDOM_RARE = NeowBonus("ONE_RANDOM_RARE_CARD", "Obtain a random rare card", 0, is_rare=True)
NEOW_REMOVE_CARD = NeowBonus("REMOVE_CARD", "Remove a card from your deck", 0)
NEOW_UPGRADE_CARD = NeowBonus("UPGRADE_CARD", "Upgrade a card", 0)
NEOW_TRANSFORM_CARD = NeowBonus("TRANSFORM_CARD", "Transform a card", 0)
NEOW_RANDOM_COLORLESS = NeowBonus("RANDOM_COLORLESS", "Obtain a random colorless card", 0)

# Category 1: Medium benefits (no drawback)
NEOW_THREE_POTIONS = NeowBonus("THREE_SMALL_POTIONS", "Obtain 3 random potions", 1)
NEOW_RANDOM_COMMON_RELIC = NeowBonus("RANDOM_COMMON_RELIC", "Obtain a random common relic", 1)
NEOW_TEN_PERCENT_HP = NeowBonus("TEN_PERCENT_HP_BONUS", "Gain 10% max HP", 1)
NEOW_THREE_ENEMY_KILL = NeowBonus("THREE_ENEMY_KILL", "Neow's Lament (first 3 combats deal 1 damage)", 1)
NEOW_HUNDRED_GOLD = NeowBonus("HUNDRED_GOLD", "Gain 100 gold", 1, gold_bonus=100)

# Category 2: Large benefits (with drawback)
NEOW_RANDOM_COLORLESS_2 = NeowBonus("RANDOM_COLORLESS_2", "Choose 1 of 3 rare colorless cards", 2)
NEOW_REMOVE_TWO = NeowBonus("REMOVE_TWO", "Remove 2 cards", 2)
NEOW_ONE_RARE_RELIC = NeowBonus("ONE_RARE_RELIC", "Obtain a random rare relic", 2)
NEOW_THREE_RARE_CARDS = NeowBonus("THREE_RARE_CARDS", "Choose 1 of 3 rare cards", 2, is_rare=True)
NEOW_TWO_FIFTY_GOLD = NeowBonus("TWO_FIFTY_GOLD", "Gain 250 gold", 2, gold_bonus=250)
NEOW_TRANSFORM_TWO = NeowBonus("TRANSFORM_TWO_CARDS", "Transform 2 cards", 2)
NEOW_TWENTY_PERCENT_HP = NeowBonus("TWENTY_PERCENT_HP_BONUS", "Gain 20% max HP", 2)

# Category 3: Boss relic swap
NEOW_BOSS_SWAP = NeowBonus("BOSS_RELIC", "Swap starting relic for random boss relic", 3)

# Drawbacks for Category 2
NEOW_DRAWBACK_10_PERCENT_HP_LOSS = "Lose 10% max HP"
NEOW_DRAWBACK_NO_GOLD = "Lose all gold"
NEOW_DRAWBACK_CURSE = "Obtain a random curse"
NEOW_DRAWBACK_PERCENT_DAMAGE = "Take 30% current HP damage"


# =============================================================================
# EVENT LOOKUP DICTIONARIES
# =============================================================================

EXORDIUM_EVENTS = {
    "Big Fish": BIG_FISH,
    "The Cleric": CLERIC,
    "Dead Adventurer": DEAD_ADVENTURER,
    "Golden Idol": GOLDEN_IDOL,
    "Golden Wing": GOLDEN_WING,
    "World of Goop": GOOP_PUDDLE,
    "Living Wall": LIVING_WALL,
    "Mushrooms": MUSHROOMS,
    "Scrap Ooze": SCRAP_OOZE,
    "Shining Light": SHINING_LIGHT,
    "Liars Game": SSSSERPENT,
}

CITY_EVENTS = {
    "Addict": ADDICT,
    "Back to Basics": BACK_TO_BASICS,
    "Beggar": BEGGAR,
    "Colosseum": COLOSSEUM,
    "Cursed Tome": CURSED_TOME,
    "Drug Dealer": DRUG_DEALER,
    "Forgotten Altar": FORGOTTEN_ALTAR,
    "Ghosts": GHOSTS,
    "Knowing Skull": KNOWING_SKULL,
    "Masked Bandits": MASKED_BANDITS,
    "Nest": NEST,
    "The Joust": THE_JOUST,
    "The Library": THE_LIBRARY,
    "The Mausoleum": THE_MAUSOLEUM,
    "Vampires": VAMPIRES,
}

BEYOND_EVENTS = {
    "Falling": FALLING,
    "MindBloom": MIND_BLOOM,
    "The Moai Head": MOAI_HEAD,
    "Mysterious Sphere": MYSTERIOUS_SPHERE,
    "Secret Portal": SECRET_PORTAL,
    "Sensory Stone": SENSORY_STONE,
    "Tomb of Lord Red Mask": TOMB_OF_LORD_RED_MASK,
    "Winding Halls": WINDING_HALLS,
}

SHRINE_EVENTS = {
    "Accursed Blacksmith": ACCURSED_BLACKSMITH,
    "Bonfire Elementals": BONFIRE_ELEMENTALS,
    "Designer": DESIGNER,
    "Duplicator": DUPLICATOR,
    "FaceTrader": FACE_TRADER,
    "Fountain of Cleansing": FOUNTAIN_OF_CURSE_REMOVAL,
    "Golden Shrine": GOLD_SHRINE,
    "Match and Keep!": GREMLIN_MATCH_GAME,
    "Wheel of Change": GREMLIN_WHEEL_GAME,
    "Lab": LAB,
    "N'loth": NLOTH,
    "NoteForYourself": NOTE_FOR_YOURSELF,
    "Purifier": PURIFICATION_SHRINE,
    "Transmorgrifier": TRANSMOGRIFIER,
    "Upgrade Shrine": UPGRADE_SHRINE,
    "WeMeetAgain": WE_MEET_AGAIN,
    "The Woman in Blue": WOMAN_IN_BLUE,
}

ALL_EVENTS = {
    **EXORDIUM_EVENTS,
    **CITY_EVENTS,
    **BEYOND_EVENTS,
    **SHRINE_EVENTS,
}


def get_event(event_id: str) -> Optional[Event]:
    """Get an event by its ID"""
    return ALL_EVENTS.get(event_id)


def get_events_for_act(act: Act) -> dict[str, Event]:
    """Get all events that can appear in a specific act"""
    if act == Act.ACT_1:
        return {**EXORDIUM_EVENTS, **SHRINE_EVENTS}
    elif act == Act.ACT_2:
        return {**CITY_EVENTS, **SHRINE_EVENTS}
    elif act == Act.ACT_3:
        return {**BEYOND_EVENTS, **SHRINE_EVENTS}
    else:
        return SHRINE_EVENTS


def calculate_outcome_value(outcome: Outcome, player_max_hp: int, player_current_hp: int,
                           ascension_level: int = 0) -> int:
    """Calculate the actual numeric value of an outcome given player state"""
    if outcome.value is not None:
        # Apply ascension modifiers for certain events
        if outcome.type == OutcomeType.HP_CHANGE and outcome.value < 0:
            # Damage is often higher on A15+
            if ascension_level >= 15:
                return int(outcome.value * 1.4)  # Rough approximation
        return outcome.value

    if outcome.value_percent is not None:
        return int(player_max_hp * outcome.value_percent)

    return 0
