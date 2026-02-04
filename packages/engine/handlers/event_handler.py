"""
Event Handler for Slay the Spire Python Recreation

Comprehensive event system handling:
- Event selection from act-specific pools using eventRng
- Choice availability based on player state
- Outcome execution with full game state modification
- Multi-phase events (Golden Idol escape, Colosseum fights)
- Combat triggers from events

Based on decompiled game source from com.megacrit.cardcrawl.events.*
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import List, Optional, Dict, Set, Tuple, Any, Callable, TYPE_CHECKING
from copy import deepcopy

if TYPE_CHECKING:
    from ..state.run import RunState
    from ..state.rng import Random


# ============================================================================
# EVENT STATE TRACKING
# ============================================================================

class EventPhase(Enum):
    """Multi-phase event state tracking."""
    INITIAL = auto()           # First choice screen
    SECONDARY = auto()         # Second choice screen (e.g., Golden Idol escape)
    COMBAT_PENDING = auto()    # Combat triggered, waiting for resolution
    COMBAT_WON = auto()        # Combat completed successfully
    COMBAT_LOST = auto()       # Combat lost (should not happen with auto-win)
    COMPLETE = auto()          # Event finished


@dataclass
class EventState:
    """State tracking for the current event."""
    event_id: str
    phase: EventPhase = EventPhase.INITIAL

    # For multi-attempt events (Scrap Ooze, Dead Adventurer)
    attempt_count: int = 0

    # For events with escalating costs (Knowing Skull)
    hp_cost_modifier: int = 0

    # For Colosseum
    first_fight_won: bool = False

    # Generated rewards to give after combat
    pending_rewards: Dict[str, Any] = field(default_factory=dict)

    # Combat enemy to spawn
    combat_encounter: Optional[str] = None


@dataclass
class EventChoiceResult:
    """Result of making an event choice."""
    event_id: str
    choice_idx: int
    choice_name: str

    # State changes
    hp_change: int = 0
    max_hp_change: int = 0
    gold_change: int = 0

    # Card changes
    cards_gained: List[str] = field(default_factory=list)
    cards_removed: List[str] = field(default_factory=list)
    cards_upgraded: List[str] = field(default_factory=list)
    cards_transformed: List[Tuple[str, str]] = field(default_factory=list)  # (old, new)

    # Relics
    relics_gained: List[str] = field(default_factory=list)
    relics_lost: List[str] = field(default_factory=list)

    # Potions
    potions_gained: List[str] = field(default_factory=list)

    # Card selection required
    requires_card_selection: bool = False
    card_selection_type: Optional[str] = None  # "remove", "upgrade", "transform", "choose"
    card_selection_count: int = 1
    card_selection_pool: List[Any] = field(default_factory=list)  # For "choose" type

    # Combat triggered
    combat_triggered: bool = False
    combat_encounter: Optional[str] = None

    # Event state
    event_complete: bool = True
    next_phase: Optional[EventPhase] = None

    # Outcome description
    description: str = ""


# ============================================================================
# EVENT CHOICE DEFINITION
# ============================================================================

@dataclass
class EventChoice:
    """Definition of a single event choice."""
    index: int
    name: str  # Internal name
    text: str  # Display text

    # Requirements
    requires_gold: Optional[int] = None
    requires_relic: Optional[str] = None
    requires_no_relic: Optional[str] = None  # Must NOT have this relic
    requires_min_hp: Optional[int] = None
    requires_min_hp_percent: Optional[float] = None
    requires_max_hp_missing: bool = False  # HP < Max HP
    requires_upgradable_cards: bool = False
    requires_removable_cards: bool = False
    requires_transformable_cards: bool = False
    requires_curse_in_deck: bool = False
    requires_card_type: Optional[str] = None  # "ATTACK", "SKILL", "POWER"
    requires_potion: bool = False
    requires_empty_potion_slot: bool = False
    requires_non_basic_card: bool = False

    # For events where choice is conditionally hidden
    always_available: bool = True
    is_hidden: bool = False  # Some choices only appear under conditions

    # Ascension affects this choice
    ascension_modifier: bool = False


# ============================================================================
# EVENT DEFINITION
# ============================================================================

@dataclass
class EventDefinition:
    """Definition of an event."""
    id: str
    name: str
    act: int  # 1, 2, 3, or 0 for any act

    # Event requirements
    min_floor: Optional[int] = None
    max_floor: Optional[int] = None
    requires_relic: Optional[str] = None
    requires_no_relic: Optional[str] = None
    requires_curse_in_deck: bool = False

    # Is this a shrine (always appears in shrine pool)?
    is_shrine: bool = False

    # Is this a one-time special event?
    is_one_time: bool = False

    # Has ascension modifiers
    has_ascension_modifier: bool = False
    ascension_threshold: int = 15

    # Description
    description: str = ""


# ============================================================================
# EVENT HANDLER
# ============================================================================

class EventHandler:
    """
    Complete event system handler.

    Responsibilities:
    - Select events from act-appropriate pools
    - Track seen/completed events
    - Filter available choices based on player state
    - Execute choice outcomes
    - Handle multi-phase events
    """

    # Basic curse cards that can be gained from events
    CURSE_CARDS = [
        "Regret", "Doubt", "Pain", "Parasite", "Shame",
        "Decay", "Writhe", "Injury", "Normality", "Clumsy"
    ]

    # Non-removable curses
    UNREMOVABLE_CURSES = ["AscendersBane", "CurseOfTheBell", "Necronomicurse"]

    # Basic cards (for "requires_non_basic_card" check)
    BASIC_CARDS = {"Strike_P", "Defend_P", "Eruption", "Vigilance", "AscendersBane"}

    # Card type mappings (simplified - would need full card data)
    ATTACK_CARDS = {"Strike_P", "Eruption", "Tantrum", "Ragnarok", "Wallop", "CutThroughFate"}
    SKILL_CARDS = {"Defend_P", "Vigilance", "InnerPeace", "Meditate", "ThirdEye", "Evaluate"}
    POWER_CARDS = {"Rushdown", "MentalFortress", "LikeWater", "DevaForm", "Establishment"}

    # Relic pools by tier (actual game relics)
    COMMON_RELICS = [
        "Anchor", "AncientTeaSet", "ArtOfWar", "Bag of Marbles", "BagOfPreparation",
        "BloodVial", "BronzeScales", "CentennialPuzzle", "CeramicFish", "Dreamcatcher",
        "HappyFlower", "Juzu Bracelet", "Lantern", "MawBank", "MealTicket",
        "Nunchaku", "OddlySmoothStone", "Omamori", "Orichalcum", "PenNib",
        "PotionBelt", "PreservedInsect", "Regal Pillow", "SmoothStone", "Strawberry",
        "TheBoot", "TinyChest", "ToyOrnithopter", "Vajra", "WarPaint", "Whetstone",
    ]
    UNCOMMON_RELICS = [
        "BlueCandle", "BottledFlame", "BottledLightning", "BottledTornado",
        "DarkstonePeriapt", "Eternal Feather", "FrozenEgg", "GremlinHorn",
        "HornCleat", "InkBottle", "Kunai", "LetterOpener", "Matryoshka",
        "MeatOnTheBone", "MercuryHourglass", "MoltenEgg", "MummifiedHand",
        "OrnamentalFan", "Pantograph", "Pear", "QuestionCard", "Shuriken",
        "SingingBowl", "StrikeDummy", "Sundial", "TheCouncilor", "ToxicEgg",
        "WhiteBeastStatue",
    ]
    RARE_RELICS = [
        "Astrolabe", "BirdFacedUrn", "Calipers", "CaptainsWheel", "DeadBranch",
        "DuVuDoll", "EmptyCage", "FossilizedHelix", "GamblingChip", "Ginger",
        "Girya", "IceCream", "IncenseBurner", "LizardTail", "Mango",
        "OldCoin", "PeacePipe", "Pocketwatch", "PrayerWheel", "Shovel",
        "StoneCalendar", "ThreadAndNeedle", "Torii", "ToughBandages",
        "TungstenRod", "Turnip", "UnceasingTop", "WarpedTongs",
    ]

    # Card pools by rarity for Watcher
    WATCHER_COMMON_CARDS = [
        "Bowling Bash", "Consecrate", "CrushJoints", "CutThroughFate",
        "EmptyBody", "EmptyFist", "Evaluate", "Flurry of Blows",
        "Flying Sleeves", "FollowUp", "Halt", "JustLucky",
        "PressurePoints", "Prostrate", "Protect", "SashWhip",
        "Tranquility",
    ]
    WATCHER_UNCOMMON_CARDS = [
        "BattleHymn", "Carve Reality", "Conclude", "Deceive Reality",
        "EmptyMind", "FearNoEvil", "ForeignInfluence", "Indignation",
        "InnerPeace", "LikeWater", "Meditate", "MentalFortress",
        "Nirvana", "Perseverance", "ReachHeaven", "Rushdown",
        "SandsOfTime", "SignatureMove", "SimmeringFury", "Study",
        "Swivel", "TalkToTheHand", "Tantrum", "ThirdEye",
        "WaveOfTheHand", "WheelKick", "WindmillStrike", "Worship",
        "WreathOfFlame",
    ]
    WATCHER_RARE_CARDS = [
        "Alpha", "Blasphemy", "ConjureBlade", "DevaForm",
        "Establishment", "Judgement", "LessonLearned", "MasterReality",
        "Omniscience", "Ragnarok", "SpiritShield", "Vault",
        "Wish", "Scrawl",
    ]
    COLORLESS_UNCOMMON_CARDS = [
        "BandageUp", "Blind", "DarkShackles", "DeepBreath",
        "Discovery", "DramaticEntrance", "Enlightenment", "Finesse",
        "FlashOfSteel", "Forethought", "GoodInstincts", "Impatience",
        "JackOfAllTrades", "Madness", "MindBlast", "Panacea",
        "PanicButton", "Purity", "SwiftStrike", "Trip",
    ]
    COLORLESS_RARE_CARDS = [
        "Apotheosis", "Chrysalis", "HandOfGreed", "Magnetism",
        "MasterOfStrategy", "Metamorphosis", "Panache", "SadisticNature",
        "SecretTechnique", "SecretWeapon", "TheBomb", "ThinkingAhead",
        "Transmutation", "Violence",
    ]

    # Potion pool
    POTIONS = [
        "BloodPotion", "BlockPotion", "DexterityPotion", "EnergyPotion",
        "ExplosivePotion", "FearPotion", "FirePotion", "FruitJuice",
        "GamblersBrew", "LiquidBronze", "LiquidMemories", "PoisonPotion",
        "PowerPotion", "RegenPotion", "SkillPotion", "SmokeBomb",
        "SneckoOil", "SpeedPotion", "StrengthPotion", "SwiftPotion",
        "WeakPotion", "AttackPotion", "StancePotion", "AmbrosiaPot",
    ]

    def __init__(self):
        """Initialize event handler."""
        # Track seen one-time events
        self.seen_one_time_events: Set[str] = set()

        # Current event state
        self.current_event: Optional[EventState] = None

        # Event encounter tracking for Colosseum, etc.
        self._colosseum_first_fight_done = False

        # Track recent events for repetition avoidance (#12)
        self.recent_events: List[str] = []

    def _get_random_relic(self, run_state: 'RunState', rng: 'Random', tier: str = "common") -> str:
        """Get a random relic from the pool, excluding owned relics."""
        if tier == "rare":
            pool = self.RARE_RELICS
        elif tier == "uncommon":
            pool = self.UNCOMMON_RELICS
        else:
            pool = self.COMMON_RELICS
        owned = set(r.id if hasattr(r, 'id') else str(r) for r in run_state.relics)
        available = [r for r in pool if r not in owned]
        if not available:
            return "Circlet"
        return available[rng.random(len(available))]

    def _get_random_card(self, run_state: 'RunState', rng: 'Random', rarity: str = "common") -> str:
        """Get a random card from the pool for the character."""
        pool = self._get_card_pool(run_state, rarity)
        return pool[rng.random(len(pool))]

    def _get_card_pool(self, run_state: 'RunState', rarity: str) -> List[str]:
        """Get card pool by rarity."""
        if rarity == "rare":
            return self.WATCHER_RARE_CARDS
        elif rarity == "uncommon":
            return self.WATCHER_UNCOMMON_CARDS
        elif rarity == "colorless_uncommon":
            return self.COLORLESS_UNCOMMON_CARDS
        elif rarity == "colorless_rare":
            return self.COLORLESS_RARE_CARDS
        elif rarity == "colorless":
            return self.COLORLESS_UNCOMMON_CARDS + self.COLORLESS_RARE_CARDS
        else:
            return self.WATCHER_COMMON_CARDS

    def _get_random_potion(self, rng: 'Random') -> str:
        """Get a random potion."""
        return self.POTIONS[rng.random(len(self.POTIONS))]

    # =========================================================================
    # EVENT SELECTION
    # =========================================================================

    def select_event(
        self,
        run_state: 'RunState',
        event_rng: 'Random'
    ) -> Optional[EventState]:
        """
        Select a random event from the current act's pool.

        Args:
            run_state: Current run state
            event_rng: Event RNG stream

        Returns:
            EventState for the selected event, or None if no events available
        """
        # Build available event pool
        available_events = self._get_available_events(run_state)

        if not available_events:
            return None

        # Filter out recently seen events (avoid last 3)
        non_recent = [e for e in available_events if e not in self.recent_events]
        pool = non_recent if non_recent else available_events

        # Select random event
        idx = event_rng.random(len(pool) - 1)
        event_id = pool[idx]

        # Mark one-time events as seen
        event_def = self._get_event_definition(event_id)
        if event_def and event_def.is_one_time:
            self.seen_one_time_events.add(event_id)

        # Track recent events (keep last 3)
        self.recent_events.append(event_id)
        if len(self.recent_events) > 3:
            self.recent_events.pop(0)

        # Create event state
        self.current_event = EventState(event_id=event_id)
        return self.current_event

    def _get_available_events(self, run_state: 'RunState') -> List[str]:
        """Get list of available event IDs for the current act."""
        available = []

        # Get act-specific pool
        if run_state.act == 1:
            pool = list(ACT1_EVENTS.keys())
        elif run_state.act == 2:
            pool = list(ACT2_EVENTS.keys())
        elif run_state.act == 3:
            pool = list(ACT3_EVENTS.keys())
        else:
            pool = []

        # Add shrine events
        pool.extend(SHRINE_EVENTS.keys())

        # Add available one-time special events
        for event_id in SPECIAL_ONE_TIME_EVENTS.keys():
            if event_id not in self.seen_one_time_events:
                pool.append(event_id)

        # Filter by requirements
        for event_id in pool:
            event_def = self._get_event_definition(event_id)
            if event_def and self._event_is_available(event_def, run_state):
                available.append(event_id)

        return available

    def _event_is_available(
        self,
        event_def: EventDefinition,
        run_state: 'RunState'
    ) -> bool:
        """Check if an event meets its appearance conditions."""
        # Floor restrictions
        if event_def.min_floor is not None:
            if run_state.floor < event_def.min_floor:
                return False
        if event_def.max_floor is not None:
            if run_state.floor > event_def.max_floor:
                return False

        # Relic requirement
        if event_def.requires_relic is not None:
            if not run_state.has_relic(event_def.requires_relic):
                return False

        # Must not have relic
        if event_def.requires_no_relic is not None:
            if run_state.has_relic(event_def.requires_no_relic):
                return False

        # Curse requirement (Fountain of Cleansing)
        if event_def.requires_curse_in_deck:
            has_removable_curse = any(
                c.id in self.CURSE_CARDS and c.id not in self.UNREMOVABLE_CURSES
                for c in run_state.deck
            )
            if not has_removable_curse:
                return False

        return True

    def _get_event_definition(self, event_id: str) -> Optional[EventDefinition]:
        """Get event definition by ID."""
        if event_id in ACT1_EVENTS:
            return ACT1_EVENTS[event_id]
        if event_id in ACT2_EVENTS:
            return ACT2_EVENTS[event_id]
        if event_id in ACT3_EVENTS:
            return ACT3_EVENTS[event_id]
        if event_id in SHRINE_EVENTS:
            return SHRINE_EVENTS[event_id]
        if event_id in SPECIAL_ONE_TIME_EVENTS:
            return SPECIAL_ONE_TIME_EVENTS[event_id]
        return None

    # =========================================================================
    # CHOICE AVAILABILITY
    # =========================================================================

    def get_available_choices(
        self,
        event_state: EventState,
        run_state: 'RunState'
    ) -> List[EventChoice]:
        """
        Get available choices for the current event phase.

        Args:
            event_state: Current event state
            run_state: Current run state

        Returns:
            List of available EventChoice objects
        """
        event_id = event_state.event_id
        phase = event_state.phase

        # Get choices for this event/phase
        choices = self._get_event_choices(event_id, phase, event_state, run_state)

        # Filter by availability
        available = []
        for choice in choices:
            if self._choice_is_available(choice, run_state):
                available.append(choice)

        return available

    def _choice_is_available(
        self,
        choice: EventChoice,
        run_state: 'RunState'
    ) -> bool:
        """Check if a choice meets its requirements."""
        # Gold requirement (with ascension adjustment)
        if choice.requires_gold is not None:
            required = choice.requires_gold
            if choice.ascension_modifier and run_state.ascension >= 15:
                required = int(required * 1.5)  # A15+ often has higher costs
            if run_state.gold < required:
                return False

        # Relic requirement
        if choice.requires_relic is not None:
            if not run_state.has_relic(choice.requires_relic):
                return False

        # Must not have relic
        if choice.requires_no_relic is not None:
            if run_state.has_relic(choice.requires_no_relic):
                return False

        # Min HP requirement
        if choice.requires_min_hp is not None:
            if run_state.current_hp < choice.requires_min_hp:
                return False

        # Min HP percent
        if choice.requires_min_hp_percent is not None:
            hp_percent = run_state.current_hp / run_state.max_hp
            if hp_percent < choice.requires_min_hp_percent:
                return False

        # Must be missing HP
        if choice.requires_max_hp_missing:
            if run_state.current_hp >= run_state.max_hp:
                return False

        # Upgradable cards
        if choice.requires_upgradable_cards:
            if not run_state.get_upgradeable_cards():
                return False

        # Removable cards
        if choice.requires_removable_cards:
            if not run_state.get_removable_cards():
                return False

        # Transformable cards (non-basic)
        if choice.requires_transformable_cards:
            transformable = [c for c in run_state.deck if c.id not in self.BASIC_CARDS]
            if not transformable:
                return False

        # Curse in deck
        if choice.requires_curse_in_deck:
            has_curse = any(c.id in self.CURSE_CARDS for c in run_state.deck)
            if not has_curse:
                return False

        # Card type requirement
        if choice.requires_card_type is not None:
            card_type = choice.requires_card_type
            if card_type == "ATTACK":
                has_type = any(c.id in self.ATTACK_CARDS for c in run_state.deck)
            elif card_type == "SKILL":
                has_type = any(c.id in self.SKILL_CARDS for c in run_state.deck)
            elif card_type == "POWER":
                has_type = any(c.id in self.POWER_CARDS for c in run_state.deck)
            else:
                has_type = True
            if not has_type:
                return False

        # Potion requirement
        if choice.requires_potion:
            if run_state.count_potions() == 0:
                return False

        # Empty potion slot
        if choice.requires_empty_potion_slot:
            if run_state.count_empty_potion_slots() == 0:
                return False

        # Non-basic card
        if choice.requires_non_basic_card:
            has_non_basic = any(c.id not in self.BASIC_CARDS for c in run_state.deck)
            if not has_non_basic:
                return False

        return True

    # =========================================================================
    # CHOICE EXECUTION
    # =========================================================================

    def execute_choice(
        self,
        event_state: EventState,
        choice_idx: int,
        run_state: 'RunState',
        event_rng: 'Random',
        card_idx: Optional[int] = None,
        misc_rng: Optional['Random'] = None,
    ) -> EventChoiceResult:
        """
        Execute an event choice and apply outcomes.

        Args:
            event_state: Current event state
            choice_idx: Index of the chosen option
            run_state: Run state to modify
            event_rng: Event RNG stream (only for event selection)
            card_idx: Optional card index for remove/upgrade/transform
            misc_rng: Misc RNG stream for outcome randomness

        Returns:
            EventChoiceResult with all changes made
        """
        event_id = event_state.event_id

        # Dispatch to specific event handler
        handler = EVENT_HANDLERS.get(event_id)
        if handler:
            return handler(self, event_state, choice_idx, run_state, event_rng, card_idx, misc_rng=misc_rng)

        # Default: just mark event complete
        return EventChoiceResult(
            event_id=event_id,
            choice_idx=choice_idx,
            choice_name="leave",
            event_complete=True,
            description="Left without making a choice."
        )

    # =========================================================================
    # UTILITY METHODS
    # =========================================================================

    def _apply_hp_change(
        self,
        run_state: 'RunState',
        amount: int
    ) -> int:
        """Apply HP change, respecting Mark of the Bloom. Returns actual change."""
        if amount > 0:
            if run_state.has_relic("Mark of the Bloom"):
                return 0
            old_hp = run_state.current_hp
            run_state.heal(amount)
            return run_state.current_hp - old_hp
        elif amount < 0:
            old_hp = run_state.current_hp
            run_state.damage(-amount)
            return run_state.current_hp - old_hp
        return 0

    def _apply_max_hp_change(
        self,
        run_state: 'RunState',
        amount: int
    ) -> int:
        """Apply max HP change. Returns actual change."""
        if amount > 0:
            run_state.gain_max_hp(amount)
        elif amount < 0:
            run_state.lose_max_hp(-amount)
        return amount

    def _apply_gold_change(
        self,
        run_state: 'RunState',
        amount: int
    ) -> int:
        """Apply gold change. Returns actual change."""
        if amount > 0:
            run_state.add_gold(amount)
            return amount
        elif amount < 0:
            return -run_state.lose_gold(-amount)
        return 0

    def _add_curse(
        self,
        run_state: 'RunState',
        curse_id: str
    ) -> str:
        """Add a specific curse to the deck."""
        run_state.add_card(curse_id)
        return curse_id

    def _add_random_curse(
        self,
        run_state: 'RunState',
        rng: 'Random'
    ) -> str:
        """Add a random curse to the deck."""
        idx = rng.random(len(self.CURSE_CARDS) - 1)
        curse_id = self.CURSE_CARDS[idx]
        run_state.add_card(curse_id)
        return curse_id

    def _get_removable_curses(self, run_state: 'RunState') -> List[Tuple[int, Any]]:
        """Get indices and cards for removable curses."""
        result = []
        for i, card in enumerate(run_state.deck):
            if card.id in self.CURSE_CARDS and card.id not in self.UNREMOVABLE_CURSES:
                result.append((i, card))
        return result

    def _heal_percent(
        self,
        run_state: 'RunState',
        percent: float
    ) -> int:
        """Heal a percentage of max HP."""
        amount = int(run_state.max_hp * percent)
        return self._apply_hp_change(run_state, amount)

    def _damage_percent(
        self,
        run_state: 'RunState',
        percent: float,
        ascension: int = 0,
        a15_percent: Optional[float] = None
    ) -> int:
        """Take damage as percentage of max HP, with optional A15+ modifier."""
        if a15_percent is not None and ascension >= 15:
            percent = a15_percent
        amount = int(run_state.max_hp * percent)
        return self._apply_hp_change(run_state, -amount)

    def _lose_max_hp_percent(
        self,
        run_state: 'RunState',
        percent: float,
        ascension: int = 0,
        a15_percent: Optional[float] = None
    ) -> int:
        """Lose max HP as percentage, with optional A15+ modifier."""
        if a15_percent is not None and ascension >= 15:
            percent = a15_percent
        amount = int(run_state.max_hp * percent)
        return self._apply_max_hp_change(run_state, -amount)


# ============================================================================
# EVENT DEFINITIONS
# ============================================================================

# Act 1 Events
ACT1_EVENTS: Dict[str, EventDefinition] = {
    "BigFish": EventDefinition(
        id="BigFish", name="Big Fish", act=1,
        description="A large fish offers food in various forms."
    ),
    "TheCleric": EventDefinition(
        id="TheCleric", name="The Cleric", act=1,
        has_ascension_modifier=True,
        description="A cleric offers healing and card removal for gold."
    ),
    "DeadAdventurer": EventDefinition(
        id="DeadAdventurer", name="Dead Adventurer", act=1,
        has_ascension_modifier=True,
        description="Search a corpse with increasing chance of elite fight."
    ),
    "GoldenIdol": EventDefinition(
        id="GoldenIdol", name="Golden Idol", act=1,
        has_ascension_modifier=True,
        description="Take a golden idol and choose how to escape the trap."
    ),
    "WingStatue": EventDefinition(
        id="WingStatue", name="Winged Statue", act=1,
        description="A winged statue offers card removal for 7 HP, or leave."
    ),
    "WorldOfGoop": EventDefinition(
        id="WorldOfGoop", name="World of Goop", act=1,
        has_ascension_modifier=True,
        description="A puddle of slime with gold inside."
    ),
    "LivingWall": EventDefinition(
        id="LivingWall", name="Living Wall", act=1,
        description="A wall offers to remove, transform, or upgrade a card."
    ),
    "Mushrooms": EventDefinition(
        id="Mushrooms", name="Mushrooms", act=1,
        description="Fight mushroom enemies or eat them (heal + curse)."
    ),
    "ScrapOoze": EventDefinition(
        id="ScrapOoze", name="Scrap Ooze", act=1,
        has_ascension_modifier=True,
        description="Reach into ooze repeatedly for chance at relic."
    ),
    "ShiningLight": EventDefinition(
        id="ShiningLight", name="Shining Light", act=1,
        has_ascension_modifier=True,
        description="Enter light to upgrade 2 random cards, taking damage."
    ),
    "Sssserpent": EventDefinition(
        id="Sssserpent", name="Sssserpent", act=1,
        has_ascension_modifier=True,
        description="A serpent offers gold for a curse."
    ),
}

# Act 2 Events
ACT2_EVENTS: Dict[str, EventDefinition] = {
    "Addict": EventDefinition(
        id="Addict", name="Pleading Vagrant", act=2,
        description="A pleading vagrant asks for gold. Pay, refuse, or rob."
    ),
    "BackToBasics": EventDefinition(
        id="BackToBasics", name="Back to Basics", act=2,
        description="Remove or upgrade all Strikes and Defends."
    ),
    "Beggar": EventDefinition(
        id="Beggar", name="The Beggar", act=2,
        description="A beggar asks for gold."
    ),
    "Colosseum": EventDefinition(
        id="Colosseum", name="The Colosseum", act=2,
        description="Fight in the colosseum for rewards."
    ),
    "CursedTome": EventDefinition(
        id="CursedTome", name="Cursed Tome", act=2,
        has_ascension_modifier=True,
        description="Read a cursed book for a relic, taking damage each page."
    ),
    "Augmenter": EventDefinition(
        id="Augmenter", name="Augmenter", act=2,
        description="Get J.A.X., transform cards, or get Mutagenic Strength."
    ),
    "ForgottenAltar": EventDefinition(
        id="ForgottenAltar", name="Forgotten Altar", act=2,
        has_ascension_modifier=True,
        description="An altar that wants the Golden Idol."
    ),
    "Ghosts": EventDefinition(
        id="Ghosts", name="Ghosts", act=2,
        has_ascension_modifier=True,
        description="Ghosts offer Apparition cards for max HP."
    ),
    "MaskedBandits": EventDefinition(
        id="MaskedBandits", name="Masked Bandits", act=2,
        description="Bandits demand all your gold."
    ),
    "Nest": EventDefinition(
        id="Nest", name="The Nest", act=2,
        description="A nest with valuable contents."
    ),
    "TheLibrary": EventDefinition(
        id="TheLibrary", name="The Library", act=2,
        description="Choose to read (pick 1 of 20 cards) or sleep (heal to full)."
    ),
    "TheMausoleum": EventDefinition(
        id="TheMausoleum", name="The Mausoleum", act=2,
        description="Open a coffin for possible rewards."
    ),
    "Vampires": EventDefinition(
        id="Vampires", name="Vampires(?)", act=2,
        description="Vampires offer to transform you."
    ),
}

# Act 3 Events
ACT3_EVENTS: Dict[str, EventDefinition] = {
    "Falling": EventDefinition(
        id="Falling", name="Falling", act=3,
        description="Must lose a card of a specific type."
    ),
    "MindBloom": EventDefinition(
        id="MindBloom", name="Mind Bloom", act=3,
        has_ascension_modifier=True,
        description="A powerful event with major rewards and costs."
    ),
    "MoaiHead": EventDefinition(
        id="MoaiHead", name="The Moai Head", act=3,
        has_ascension_modifier=True,
        description="A giant head offers healing or wants the Golden Idol."
    ),
    "MysteriousSphere": EventDefinition(
        id="MysteriousSphere", name="Mysterious Sphere", act=3,
        description="A mysterious orb with combat inside."
    ),
    "SensoryStone": EventDefinition(
        id="SensoryStone", name="Sensory Stone", act=3,
        description="Relive memories for colorless cards."
    ),
    "TombOfLordRedMask": EventDefinition(
        id="TombOfLordRedMask", name="Tomb of Lord Red Mask", act=3,
        description="A tomb with gold, requires the Red Mask."
    ),
    "WindingHalls": EventDefinition(
        id="WindingHalls", name="Winding Halls", act=3,
        has_ascension_modifier=True,
        description="Lost in winding halls with three paths."
    ),
}

# Shrine Events (any act)
SHRINE_EVENTS: Dict[str, EventDefinition] = {
    "GoldenShrine": EventDefinition(
        id="GoldenShrine", name="Golden Shrine", act=0,
        is_shrine=True, has_ascension_modifier=True,
        description="A shrine offering gold."
    ),
    "GremlinMatchGame": EventDefinition(
        id="GremlinMatchGame", name="Gremlin Match Game", act=0,
        is_shrine=True, has_ascension_modifier=True,
        description="Memory match game for cards."
    ),
    "GremlinWheelGame": EventDefinition(
        id="GremlinWheelGame", name="Gremlin Wheel Game", act=0,
        is_shrine=True, has_ascension_modifier=True,
        description="Spin the wheel for random rewards."
    ),
    "Purifier": EventDefinition(
        id="Purifier", name="Purifier", act=0,
        is_shrine=True,
        description="A shrine that removes a card."
    ),
    "Transmogrifier": EventDefinition(
        id="Transmogrifier", name="Transmogrifier", act=0,
        is_shrine=True,
        description="A shrine that transforms a card."
    ),
    "UpgradeShrine": EventDefinition(
        id="UpgradeShrine", name="Upgrade Shrine", act=0,
        is_shrine=True,
        description="A shrine that upgrades a card."
    ),
}

# Special One-Time Events
SPECIAL_ONE_TIME_EVENTS: Dict[str, EventDefinition] = {
    "AccursedBlacksmith": EventDefinition(
        id="AccursedBlacksmith", name="Accursed Blacksmith", act=0,
        is_one_time=True,
        description="An abandoned forge."
    ),
    "BonfireElementals": EventDefinition(
        id="BonfireElementals", name="Bonfire Elementals", act=0,
        is_one_time=True,
        description="Spirits at a bonfire want offerings."
    ),
    "Designer": EventDefinition(
        id="Designer", name="Designer In-Spire", act=0,
        is_one_time=True, has_ascension_modifier=True,
        description="A fashion designer offers services for gold."
    ),
    "Duplicator": EventDefinition(
        id="Duplicator", name="Duplicator", act=0,
        is_one_time=True,
        description="A shrine that duplicates a card."
    ),
    "FaceTrader": EventDefinition(
        id="FaceTrader", name="Face Trader", act=0,
        is_one_time=True, has_ascension_modifier=True,
        description="A strange being that trades faces."
    ),
    "FountainOfCleansing": EventDefinition(
        id="FountainOfCleansing", name="Fountain of Cleansing", act=0,
        is_one_time=True, requires_curse_in_deck=True,
        description="A fountain that removes all removable curses."
    ),
    "KnowingSkull": EventDefinition(
        id="KnowingSkull", name="Knowing Skull", act=0,
        is_one_time=True,
        description="A skull offers items for HP. Costs escalate."
    ),
    "TheLab": EventDefinition(
        id="TheLab", name="The Lab", act=0,
        is_one_time=True, has_ascension_modifier=True,
        description="A laboratory with potions."
    ),
    "Nloth": EventDefinition(
        id="Nloth", name="N'loth", act=0,
        is_one_time=True,
        description="N'loth wants to trade for your relics."
    ),
    "SecretPortal": EventDefinition(
        id="SecretPortal", name="Secret Portal", act=0,
        is_one_time=True,
        description="A portal that leads directly to the boss."
    ),
    "TheJoust": EventDefinition(
        id="TheJoust", name="The Joust", act=0,
        is_one_time=True,
        description="Bet on a joust outcome."
    ),
    "WeMeetAgain": EventDefinition(
        id="WeMeetAgain", name="We Meet Again", act=0,
        is_one_time=True,
        description="A familiar face wants something in exchange for a relic."
    ),
    "WomanInBlue": EventDefinition(
        id="WomanInBlue", name="The Woman in Blue", act=0,
        is_one_time=True, has_ascension_modifier=True,
        description="A woman selling potions."
    ),
}


# ============================================================================
# EVENT CHOICE GENERATORS
# ============================================================================

def _get_event_choices_default(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Default choice generator - just Leave."""
    return [
        EventChoice(index=0, name="leave", text="[Leave]")
    ]


# ============================================================================
# EVENT HANDLERS - Act 1
# ============================================================================

def _handle_big_fish(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Big Fish: Banana (heal 33%), Donut (+5 max HP), Box (relic + Regret curse)."""
    result = EventChoiceResult(event_id="BigFish", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Banana: Heal 1/3 max HP
        result.choice_name = "banana"
        heal = handler._heal_percent(run_state, 0.33)
        result.hp_change = heal
        result.description = f"Ate the banana. Healed {heal} HP."

    elif choice_idx == 1:
        # Donut: Gain 5 Max HP
        result.choice_name = "donut"
        handler._apply_max_hp_change(run_state, 5)
        result.max_hp_change = 5
        result.description = "Ate the donut. Gained 5 Max HP."

    elif choice_idx == 2:
        # Box: Random relic + Regret curse
        result.choice_name = "box"
        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)

        # Add Regret curse
        handler._add_curse(run_state, "Regret")
        result.cards_gained.append("Regret")
        result.description = f"Opened the box. Gained {relic} and Regret curse."

    return result


def _handle_the_cleric(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Cleric: Heal (35g for 25% HP), Purify (50/75g remove card), Leave."""
    result = EventChoiceResult(event_id="TheCleric", choice_idx=choice_idx, choice_name="")

    ascension = run_state.ascension

    if choice_idx == 0:
        # Heal: Pay 35 gold, heal 25% max HP
        result.choice_name = "heal"
        cost = 35
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        heal = handler._heal_percent(run_state, 0.25)
        result.hp_change = heal
        result.description = f"Paid {cost} gold. Healed {heal} HP."

    elif choice_idx == 1:
        # Purify: Pay 50/75 gold, remove a card
        result.choice_name = "purify"
        cost = 75 if ascension >= 15 else 50
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                result.description = f"Paid {cost} gold. Removed {removed.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = f"Paid {cost} gold. Choose a card to remove."

    elif choice_idx == 2:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the cleric."

    return result


def _handle_golden_idol(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """
    Golden Idol:
    Phase 1: Take (get relic, go to phase 2) or Leave
    Phase 2: Outrun (Injury curse), Smash (25%/35% damage), Hide (8%/10% max HP loss)
    """
    result = EventChoiceResult(event_id="GoldenIdol", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if event_state.phase == EventPhase.INITIAL:
        if choice_idx == 0:
            # Take the idol
            result.choice_name = "take"
            run_state.add_relic("GoldenIdol")
            result.relics_gained.append("GoldenIdol")

            # Move to escape phase
            result.event_complete = False
            result.next_phase = EventPhase.SECONDARY
            event_state.phase = EventPhase.SECONDARY
            result.description = "Took the Golden Idol. A trap activates! Choose how to escape."

        elif choice_idx == 1:
            # Leave
            result.choice_name = "leave"
            result.description = "Left the idol behind."

    elif event_state.phase == EventPhase.SECONDARY:
        if choice_idx == 0:
            # Outrun: Get Injury curse
            result.choice_name = "outrun"
            handler._add_curse(run_state, "Injury")
            result.cards_gained.append("Injury")
            result.description = "Outran the boulder. Gained Injury curse."

        elif choice_idx == 1:
            # Smash through: Take 25%/35% max HP damage
            result.choice_name = "smash"
            damage = handler._damage_percent(run_state, 0.25, ascension, 0.35)
            result.hp_change = -abs(damage)
            result.description = f"Smashed through the wall. Took {abs(damage)} damage."

        elif choice_idx == 2:
            # Hide: Lose 8%/10% max HP
            result.choice_name = "hide"
            loss = handler._lose_max_hp_percent(run_state, 0.08, ascension, 0.10)
            result.max_hp_change = -abs(loss)
            result.description = f"Hid in a crevice. Lost {abs(loss)} Max HP."

    return result


def _handle_world_of_goop(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """World of Goop: Gather gold (75g, take 11 damage) or Leave (lose 20-50g/35-75g)."""
    result = EventChoiceResult(event_id="WorldOfGoop", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Gather gold
        result.choice_name = "gather"
        handler._apply_gold_change(run_state, 75)
        result.gold_change = 75

        handler._apply_hp_change(run_state, -11)
        result.hp_change = -11
        result.description = "Gathered the gold. Gained 75 gold, took 11 damage."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        # Lose gold based on ascension
        if ascension >= 15:
            loss = misc_rng.random_range(35, 75)
        else:
            loss = misc_rng.random_range(20, 50)
        actual_loss = handler._apply_gold_change(run_state, -loss)
        result.gold_change = actual_loss
        result.description = f"Left the goop. Lost {abs(actual_loss)} gold."

    return result


def _handle_living_wall(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Living Wall: Forget (remove card), Change (transform card), Grow (upgrade card)."""
    result = EventChoiceResult(event_id="LivingWall", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Forget: Remove a card
        result.choice_name = "forget"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                result.description = f"The wall consumed {removed.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Choose a card to remove."

    elif choice_idx == 1:
        # Change: Transform a card
        result.choice_name = "change"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                new_card = handler._get_random_card(run_state, misc_rng, "common")
                run_state.add_card(new_card)
                result.cards_gained.append(new_card)
                result.cards_transformed.append((removed.id, new_card))
                result.description = f"The wall transformed {removed.id} into {new_card}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "transform"
            result.event_complete = False
            result.description = "Choose a card to transform."

    elif choice_idx == 2:
        # Grow: Upgrade a card
        result.choice_name = "grow"
        if card_idx is not None:
            if run_state.upgrade_card(card_idx):
                card = run_state.deck[card_idx]
                result.cards_upgraded.append(card.id)
                result.description = f"The wall upgraded {card.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "upgrade"
            result.event_complete = False
            result.description = "Choose a card to upgrade."

    return result


def _handle_scrap_ooze(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Scrap Ooze: Reach in (damage, chance for relic) or Leave."""
    result = EventChoiceResult(event_id="ScrapOoze", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Reach in
        result.choice_name = "reach"

        # Calculate damage (3/5 base + attempt count)
        base_damage = 5 if ascension >= 15 else 3
        damage = base_damage + event_state.attempt_count
        handler._apply_hp_change(run_state, -damage)
        result.hp_change = -damage

        # Calculate success chance (25% + 10% per attempt)
        success_chance = 0.25 + (event_state.attempt_count * 0.10)
        roll = misc_rng.random_float()

        if roll < success_chance:
            # Success! Get relic
            relic = handler._get_random_relic(run_state, misc_rng, "common")
            run_state.add_relic(relic)
            result.relics_gained.append(relic)
            result.description = f"Took {damage} damage. Found {relic}!"
        else:
            # Failed, can try again
            event_state.attempt_count += 1
            result.event_complete = False
            result.description = f"Took {damage} damage. Found nothing. Try again?"

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the ooze alone."

    return result


def _handle_sssserpent(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Sssserpent: Agree (gain 175/150 gold, get Doubt curse) or Disagree (leave)."""
    result = EventChoiceResult(event_id="Sssserpent", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Agree
        result.choice_name = "agree"
        gold = 150 if ascension >= 15 else 175
        handler._apply_gold_change(run_state, gold)
        result.gold_change = gold

        handler._add_curse(run_state, "Doubt")
        result.cards_gained.append("Doubt")
        result.description = f"Accepted the serpent's offer. Gained {gold} gold and Doubt curse."

    elif choice_idx == 1:
        # Disagree
        result.choice_name = "disagree"
        result.description = "Declined the serpent's offer."

    return result


def _handle_wing_statue(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Winged Statue: Purify (lose 7 HP, remove a card) or Leave."""
    result = EventChoiceResult(event_id="WingStatue", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Purify - take damage, remove a card
        result.choice_name = "purify"
        handler._apply_hp_change(run_state, -7)
        result.hp_change = -7

        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                result.description = f"Purified. Took 7 damage, removed {removed.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Purified. Took 7 damage. Choose a card to remove."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the statue."

    return result


def _handle_shining_light(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Shining Light: Enter (take damage, upgrade 2 random cards) or Leave."""
    result = EventChoiceResult(event_id="ShiningLight", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Enter the light
        result.choice_name = "enter"

        # Take damage (20%/30% max HP)
        damage = handler._damage_percent(run_state, 0.20, ascension, 0.30)
        result.hp_change = -abs(damage)

        # Upgrade 2 random upgradeable cards
        upgradeable = run_state.get_upgradeable_cards()
        upgraded_count = 0
        for _ in range(min(2, len(upgradeable))):
            if upgradeable:
                idx = misc_rng.random(len(upgradeable) - 1)
                card_idx_to_upgrade, card = upgradeable.pop(idx)
                run_state.upgrade_card(card_idx_to_upgrade)
                result.cards_upgraded.append(card.id)
                upgraded_count += 1

        result.description = f"Entered the light. Took {abs(damage)} damage, upgraded {upgraded_count} cards."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shining light."

    return result


def _handle_dead_adventurer(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Dead Adventurer: Search the body (escalating elite fight chance) or Leave."""
    result = EventChoiceResult(event_id="DeadAdventurer", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Search
        result.choice_name = "search"

        # Fight chance: 25% base + 10% per attempt (at A15+: 35% base)
        base_chance = 0.35 if ascension >= 15 else 0.25
        fight_chance = base_chance + (event_state.attempt_count * 0.10)
        roll = misc_rng.random_float()

        if roll < fight_chance:
            # Elite fight triggered
            result.combat_triggered = True
            result.combat_encounter = "DeadAdventurerElite"
            result.event_complete = False
            event_state.phase = EventPhase.COMBAT_PENDING
            event_state.combat_encounter = "DeadAdventurerElite"
            result.description = "Searched the body and awakened an elite!"
        else:
            # Got reward
            event_state.attempt_count += 1
            reward_type = event_state.attempt_count
            if reward_type == 1:
                handler._apply_gold_change(run_state, 30)
                result.gold_change = 30
                result.description = "Searched the body. Found 30 gold."
            elif reward_type == 2:
                relic = handler._get_random_relic(run_state, misc_rng, "common")
                run_state.add_relic(relic)
                result.relics_gained.append(relic)
                result.description = f"Searched the body. Found {relic}!"
            else:
                card = handler._get_random_card(run_state, misc_rng, "rare")
                run_state.add_card(card)
                result.cards_gained.append(card)
                result.description = f"Searched the body. Found {card}!"

            result.event_complete = False

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the body alone."

    return result


def _handle_mushrooms(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Mushrooms: Stomp (fight mushrooms, get Odd Mushroom) or Eat (heal, Parasite curse)."""
    result = EventChoiceResult(event_id="Mushrooms", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Stomp (fight)
        result.choice_name = "stomp"
        result.combat_triggered = True
        result.combat_encounter = "FungusBeast"
        result.event_complete = False
        event_state.phase = EventPhase.COMBAT_PENDING
        event_state.combat_encounter = "FungusBeast"
        event_state.pending_rewards = {"relic": "OddMushroom"}
        result.description = "Stomped on the mushrooms. Fight the Fungi!"

    elif choice_idx == 1:
        # Eat (heal, get Parasite)
        result.choice_name = "eat"
        heal = handler._heal_percent(run_state, 0.25 if ascension < 15 else 0.20)
        result.hp_change = heal

        handler._add_curse(run_state, "Parasite")
        result.cards_gained.append("Parasite")
        result.description = f"Ate the mushrooms. Healed {heal} HP, gained Parasite curse."

    return result


def _handle_back_to_basics(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Back to Basics: Simplicity (keep only Strikes/Defends, remove rest) or Elegance (upgrade all Strikes/Defends)."""
    result = EventChoiceResult(event_id="BackToBasics", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Simplicity - keep only strikes and defends, remove everything else
        result.choice_name = "simplicity"
        indices_to_remove = []
        for i, card in enumerate(run_state.deck):
            if card.id not in ["Strike_P", "Defend_P"]:
                indices_to_remove.append(i)
                result.cards_removed.append(card.id)

        # Remove in reverse order
        for i in reversed(indices_to_remove):
            run_state.remove_card(i)

        result.description = f"Chose simplicity. Removed {len(indices_to_remove)} non-basic cards."

    elif choice_idx == 1:
        # Elegance - upgrade all strikes and defends
        result.choice_name = "elegance"
        upgraded_count = 0
        for i, card in enumerate(run_state.deck):
            if card.id in ["Strike_P", "Defend_P"] and not card.upgraded:
                run_state.upgrade_card(i)
                result.cards_upgraded.append(card.id)
                upgraded_count += 1

        result.description = f"Chose elegance. Upgraded {upgraded_count} basic cards."

    return result


def _handle_forgotten_altar(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Forgotten Altar: Sacrifice (lose Golden Idol, get Bloody Idol) or Offer (5/7 HP = random relic) or Leave (Decay)."""
    result = EventChoiceResult(event_id="ForgottenAltar", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Sacrifice Golden Idol
        result.choice_name = "sacrifice"
        # Remove Golden Idol
        for i, relic in enumerate(run_state.relics):
            if relic.id == "GoldenIdol":
                run_state.relics.pop(i)
                result.relics_lost.append("GoldenIdol")
                break

        # Get Bloody Idol
        run_state.add_relic("BloodyIdol")
        result.relics_gained.append("BloodyIdol")
        result.description = "Sacrificed the Golden Idol. Gained Bloody Idol."

    elif choice_idx == 1:
        # Offer HP for random relic
        result.choice_name = "offer"
        damage = 7 if ascension >= 15 else 5
        handler._apply_hp_change(run_state, -damage)
        result.hp_change = -damage

        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Offered {damage} HP. Gained {relic}."

    elif choice_idx == 2:
        # Leave (get Decay curse)
        result.choice_name = "leave"
        handler._add_curse(run_state, "Decay")
        result.cards_gained.append("Decay")
        result.description = "Left the altar. Gained Decay curse."

    return result


def _handle_nest(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Nest: Smash (99 gold + random card) or Stay (Ritual Dagger)."""
    result = EventChoiceResult(event_id="Nest", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Smash and Grab - 99 gold + random card
        result.choice_name = "smash"
        handler._apply_gold_change(run_state, 99)
        result.gold_change = 99

        card = handler._get_random_card(run_state, misc_rng, "common")
        run_state.add_card(card)
        result.cards_gained.append(card)
        result.description = f"Smashed the nest. Gained 99 gold and {card}."

    elif choice_idx == 1:
        # Stay - get Ritual Dagger
        result.choice_name = "stay"
        run_state.add_card("RitualDagger")
        result.cards_gained.append("RitualDagger")
        result.description = "Stayed with the birds. Gained Ritual Dagger."

    return result


def _handle_addict(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Pleading Vagrant: Pay 85g for relic, Refuse (Shame curse), Rob (relic + Shame)."""
    result = EventChoiceResult(event_id="Addict", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Pay 85 gold for random relic
        result.choice_name = "pay"
        handler._apply_gold_change(run_state, -85)
        result.gold_change = -85

        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Paid 85 gold. Gained {relic}."

    elif choice_idx == 1:
        # Refuse - gain Shame curse
        result.choice_name = "refuse"
        handler._add_curse(run_state, "Shame")
        result.cards_gained.append("Shame")
        result.description = "Refused. Gained Shame curse."

    elif choice_idx == 2:
        # Rob - get relic + Shame curse
        result.choice_name = "rob"
        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)

        handler._add_curse(run_state, "Shame")
        result.cards_gained.append("Shame")
        result.description = f"Robbed the vagrant. Gained {relic} and Shame curse."

    return result


# ============================================================================
# EVENT HANDLERS - Act 2
# ============================================================================

def _handle_colosseum(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """
    Colosseum:
    Phase 1: Enter (fight Slavers)
    Phase 2 (after first fight): Fight Nobs (for big rewards) or Cowardice (leave)
    """
    result = EventChoiceResult(event_id="Colosseum", choice_idx=choice_idx, choice_name="")

    if event_state.phase == EventPhase.INITIAL:
        if choice_idx == 0:
            # Enter the colosseum
            result.choice_name = "enter"
            result.combat_triggered = True
            result.combat_encounter = "ColosseumSlavers"
            result.event_complete = False
            result.next_phase = EventPhase.COMBAT_PENDING
            event_state.phase = EventPhase.COMBAT_PENDING
            event_state.combat_encounter = "ColosseumSlavers"
            result.description = "Entered the colosseum. Fight the Slavers!"

    elif event_state.phase == EventPhase.COMBAT_WON:
        if not event_state.first_fight_won:
            # First fight won, offer second fight
            event_state.first_fight_won = True
            event_state.phase = EventPhase.SECONDARY
            result.event_complete = False
            result.next_phase = EventPhase.SECONDARY
            result.description = "Defeated the Slavers! The crowd demands more..."
        else:
            # Second fight won, give big rewards
            result.relics_gained.append("RareRelic")
            result.relics_gained.append("UncommonRelic")
            result.gold_change = 100
            handler._apply_gold_change(run_state, 100)
            result.description = "Victory! Gained a rare relic, uncommon relic, and 100 gold!"

    elif event_state.phase == EventPhase.SECONDARY:
        if choice_idx == 0:
            # Fight the Nobs
            result.choice_name = "fight_nobs"
            result.combat_triggered = True
            result.combat_encounter = "TwoNobs"
            result.event_complete = False
            result.next_phase = EventPhase.COMBAT_PENDING
            event_state.phase = EventPhase.COMBAT_PENDING
            event_state.combat_encounter = "TwoNobs"
            result.description = "Chose to fight the Taskmasters!"

        elif choice_idx == 1:
            # Cowardice - leave
            result.choice_name = "cowardice"
            result.description = "Fled the colosseum."

    return result


def _handle_cursed_tome(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Cursed Tome: Read (take damage, get book relic) or Leave."""
    result = EventChoiceResult(event_id="CursedTome", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Read the tome
        result.choice_name = "read"

        # Total damage: 1 + 2 + 3 + 10 = 16 (or 21 on A15+)
        total_damage = 21 if ascension >= 15 else 16
        handler._apply_hp_change(run_state, -total_damage)
        result.hp_change = -total_damage

        # Get one of the book relics
        book_relics = ["Necronomicon", "Enchiridion", "NilrysCodex"]
        relic = book_relics[misc_rng.random(len(book_relics) - 1)]
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Read the cursed tome. Took {total_damage} damage, gained {relic}."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the tome alone."

    return result


def _handle_the_library(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Library: Read (choose 1 of 20 cards) or Sleep (heal to full)."""
    result = EventChoiceResult(event_id="TheLibrary", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Read
        result.choice_name = "read"
        # Would generate 20 cards here
        result.requires_card_selection = True
        result.card_selection_type = "choose"
        result.card_selection_count = 1
        # result.card_selection_pool would be filled with 20 cards
        result.event_complete = False
        result.description = "Started reading. Choose a card from the library."

    elif choice_idx == 1:
        # Sleep
        result.choice_name = "sleep"
        heal = run_state.max_hp - run_state.current_hp
        handler._apply_hp_change(run_state, heal)
        result.hp_change = heal
        result.description = f"Slept peacefully. Healed to full HP ({heal} HP)."

    return result


def _handle_the_mausoleum(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Mausoleum: Open coffin (50% relic, 50% curse) or Leave."""
    result = EventChoiceResult(event_id="TheMausoleum", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Open
        result.choice_name = "open"

        roll = misc_rng.random_float()
        if roll < 0.5:
            # Got relic
            relic = handler._get_random_relic(run_state, misc_rng, "common")
            run_state.add_relic(relic)
            result.relics_gained.append(relic)
            result.description = f"Opened the coffin. Found {relic}!"
        else:
            # Got curse
            curse = handler._add_random_curse(run_state, misc_rng)
            result.cards_gained.append(curse)
            result.description = f"Opened the coffin. Cursed with {curse}!"

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the coffin unopened."

    return result


def _handle_masked_bandits(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Masked Bandits: Pay (lose all gold) or Fight (combat for gold + relic)."""
    result = EventChoiceResult(event_id="MaskedBandits", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Pay - lose all gold
        result.choice_name = "pay"
        lost = run_state.gold
        run_state.set_gold(0)
        result.gold_change = -lost
        result.description = f"Surrendered all gold. Lost {lost} gold."

    elif choice_idx == 1:
        # Fight
        result.choice_name = "fight"
        result.combat_triggered = True
        result.combat_encounter = "ThreeBandits"
        result.event_complete = False
        result.next_phase = EventPhase.COMBAT_PENDING
        event_state.phase = EventPhase.COMBAT_PENDING
        event_state.combat_encounter = "ThreeBandits"
        result.description = "Chose to fight the bandits!"

    return result


def _handle_knowing_skull(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Knowing Skull: Pay HP for potion/gold/card, or leave (also costs HP)."""
    result = EventChoiceResult(event_id="KnowingSkull", choice_idx=choice_idx, choice_name="")

    # Base cost is 6 HP, increases each time
    base_cost = 6 + event_state.hp_cost_modifier

    if choice_idx == 0:
        # Potion
        result.choice_name = "potion"
        handler._apply_hp_change(run_state, -base_cost)
        result.hp_change = -base_cost

        potion = handler._get_random_potion(misc_rng)
        result.potions_gained.append(potion)
        event_state.hp_cost_modifier += 2
        result.event_complete = False
        result.description = f"Paid {base_cost} HP. Gained {potion}."

    elif choice_idx == 1:
        # Gold
        result.choice_name = "gold"
        handler._apply_hp_change(run_state, -base_cost)
        result.hp_change = -base_cost

        handler._apply_gold_change(run_state, 90)
        result.gold_change = 90
        event_state.hp_cost_modifier += 2
        result.event_complete = False
        result.description = f"Paid {base_cost} HP. Gained 90 gold."

    elif choice_idx == 2:
        # Card
        result.choice_name = "card"
        handler._apply_hp_change(run_state, -base_cost)
        result.hp_change = -base_cost

        card = handler._get_random_card(run_state, misc_rng, "colorless_uncommon")
        run_state.add_card(card)
        result.cards_gained.append(card)
        event_state.hp_cost_modifier += 2
        result.event_complete = False
        result.description = f"Paid {base_cost} HP. Gained a colorless uncommon card."

    elif choice_idx == 3:
        # Leave
        result.choice_name = "leave"
        handler._apply_hp_change(run_state, -6)  # Always costs 6 to leave
        result.hp_change = -6
        result.description = "Paid 6 HP to leave."

    return result


def _handle_ghosts(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Ghosts: Accept (lose 50% max HP, gain 5/3 Apparitions) or Refuse."""
    result = EventChoiceResult(event_id="Ghosts", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Accept
        result.choice_name = "accept"

        # Lose 50% max HP
        loss = handler._lose_max_hp_percent(run_state, 0.50)
        result.max_hp_change = loss

        # Gain Apparitions (5 normally, 3 on A15+)
        apparition_count = 3 if ascension >= 15 else 5
        for _ in range(apparition_count):
            run_state.add_card("Apparition")
            result.cards_gained.append("Apparition")

        result.description = f"Accepted the ghosts' deal. Lost {abs(loss)} Max HP, gained {apparition_count} Apparitions."

    elif choice_idx == 1:
        # Refuse
        result.choice_name = "refuse"
        result.description = "Refused the ghosts' offer."

    return result


def _handle_vampires(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Vampires: Accept (remove Strikes, gain 5 Bites, -30% max HP) or Refuse (fight)."""
    result = EventChoiceResult(event_id="Vampires", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Accept
        result.choice_name = "accept"

        # Remove all Strikes
        strikes_removed = []
        indices_to_remove = []
        for i, card in enumerate(run_state.deck):
            if card.id == "Strike_P":
                indices_to_remove.append(i)
                strikes_removed.append(card.id)

        # Remove in reverse order to maintain indices
        for i in reversed(indices_to_remove):
            run_state.remove_card(i)
        result.cards_removed = strikes_removed

        # Gain 5 Bites
        for _ in range(5):
            run_state.add_card("Bite")
            result.cards_gained.append("Bite")

        # Lose 30% max HP
        loss = handler._lose_max_hp_percent(run_state, 0.30)
        result.max_hp_change = loss

        result.description = f"Became a vampire. Removed {len(strikes_removed)} Strikes, gained 5 Bites, lost {abs(loss)} Max HP."

    elif choice_idx == 1:
        # Refuse - fight
        result.choice_name = "refuse"
        result.combat_triggered = True
        result.combat_encounter = "Vampires"
        result.event_complete = False
        event_state.phase = EventPhase.COMBAT_PENDING
        event_state.combat_encounter = "Vampires"
        result.description = "Refused the vampires. They attack!"

    return result


# ============================================================================
# EVENT HANDLERS - Act 3
# ============================================================================

def _handle_falling(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Falling: Must lose a card of a specific type (Skill/Power/Attack)."""
    result = EventChoiceResult(event_id="Falling", choice_idx=choice_idx, choice_name="")

    card_type = ""
    if choice_idx == 0:
        card_type = "SKILL"
        result.choice_name = "skill"
    elif choice_idx == 1:
        card_type = "POWER"
        result.choice_name = "power"
    elif choice_idx == 2:
        card_type = "ATTACK"
        result.choice_name = "attack"

    # Find random card of that type and remove it
    if card_type == "SKILL":
        candidates = [i for i, c in enumerate(run_state.deck) if c.id in handler.SKILL_CARDS]
    elif card_type == "POWER":
        candidates = [i for i, c in enumerate(run_state.deck) if c.id in handler.POWER_CARDS]
    else:
        candidates = [i for i, c in enumerate(run_state.deck) if c.id in handler.ATTACK_CARDS]

    if candidates:
        idx = misc_rng.random(len(candidates) - 1)
        removed = run_state.remove_card(candidates[idx])
        if removed:
            result.cards_removed.append(removed.id)
            result.description = f"Landed on {card_type.lower()}. Lost {removed.id}."
    else:
        result.description = f"Landed on {card_type.lower()}. No cards of that type to lose."

    return result


def _handle_mind_bloom(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """
    Mind Bloom:
    - I am War: Fight Act 1 boss for rare relic
    - I am Awake: Upgrade all cards, get Mark of the Bloom
    - I am Rich: 999 gold, 2 Normality curses
    """
    result = EventChoiceResult(event_id="MindBloom", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # I am War
        result.choice_name = "war"
        result.combat_triggered = True
        result.combat_encounter = "Act1Boss"
        result.event_complete = False
        event_state.phase = EventPhase.COMBAT_PENDING
        event_state.combat_encounter = "Act1Boss"
        event_state.pending_rewards = {"relic": "RareRelic", "gold": 50}
        result.description = "Chose 'I am War'. Fight an Act 1 boss!"

    elif choice_idx == 1:
        # I am Awake
        result.choice_name = "awake"

        # Upgrade all cards
        upgraded_count = 0
        for i in range(len(run_state.deck)):
            if run_state.upgrade_card(i):
                upgraded_count += 1

        # Get Mark of the Bloom (can't heal)
        run_state.add_relic("Mark of the Bloom")
        result.relics_gained.append("Mark of the Bloom")
        result.description = f"Chose 'I am Awake'. Upgraded {upgraded_count} cards, gained Mark of the Bloom."

    elif choice_idx == 2:
        # I am Rich
        result.choice_name = "rich"
        handler._apply_gold_change(run_state, 999)
        result.gold_change = 999

        for _ in range(2):
            handler._add_curse(run_state, "Normality")
            result.cards_gained.append("Normality")
        result.description = "Chose 'I am Rich'. Gained 999 gold and 2 Normality curses."

    return result


def _handle_mysterious_sphere(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Mysterious Sphere: Open (fight 2 Orb Walkers for rare relic) or Leave."""
    result = EventChoiceResult(event_id="MysteriousSphere", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Open
        result.choice_name = "open"
        result.combat_triggered = True
        result.combat_encounter = "TwoOrbWalkers"
        result.event_complete = False
        event_state.phase = EventPhase.COMBAT_PENDING
        event_state.combat_encounter = "TwoOrbWalkers"
        event_state.pending_rewards = {"relic": "RareRelic"}
        result.description = "Opened the sphere. Fight 2 Orb Walkers!"

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the sphere alone."

    return result


def _handle_secret_portal(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Secret Portal: Enter (skip to Act 3 boss) or Leave."""
    result = EventChoiceResult(event_id="SecretPortal", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Enter
        result.choice_name = "enter"
        # This would teleport to boss - game logic handles this
        result.description = "Entered the portal. Teleported directly to the Act 3 boss!"

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the portal."

    return result


def _handle_sensory_stone(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Sensory Stone: Touch (gain colorless card rewards based on act number)."""
    result = EventChoiceResult(event_id="SensoryStone", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Touch - card rewards based on act number (Act 1=1, Act 2=2, Act 3=3)
        result.choice_name = "touch"

        card_count = min(run_state.act, 3)

        for _ in range(card_count):
            card = handler._get_random_card(run_state, misc_rng, "colorless")
            run_state.add_card(card)
            result.cards_gained.append(card)

        result.description = f"Touched the stone. Gained {card_count} colorless card reward(s)."

    return result


def _handle_tomb_of_lord_red_mask(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Tomb of Lord Red Mask: Don mask (get Red Mask), Offer gold (if have Red Mask), Leave."""
    result = EventChoiceResult(event_id="TombOfLordRedMask", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Don the Red Mask
        result.choice_name = "don_mask"
        run_state.add_relic("RedMask")
        result.relics_gained.append("RedMask")
        result.description = "Donned the Red Mask."

    elif choice_idx == 1:
        # Offer gold (requires Red Mask)
        result.choice_name = "offer_gold"

        # Lose all gold, gain 222 per relic
        lost_gold = run_state.gold
        run_state.set_gold(0)

        relic_count = len(run_state.relics)
        gold_gained = 222 * relic_count
        run_state.add_gold(gold_gained)

        result.gold_change = gold_gained - lost_gold
        result.description = f"Offered gold to the tomb. Gained {gold_gained} gold ({relic_count} relics x 222)."

    elif choice_idx == 2:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the tomb."

    return result


def _handle_moai_head(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Moai Head: Enter (lose max HP, heal to full), Offer Golden Idol (gain 333g), Leave."""
    result = EventChoiceResult(event_id="MoaiHead", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Enter
        result.choice_name = "enter"

        # Lose 12.5%/18% max HP
        loss = handler._lose_max_hp_percent(run_state, 0.125, ascension, 0.18)
        result.max_hp_change = loss

        # Heal to full
        heal = run_state.max_hp - run_state.current_hp
        handler._apply_hp_change(run_state, heal)
        result.hp_change = heal

        result.description = f"Entered the Moai Head. Lost {abs(loss)} Max HP, healed to full."

    elif choice_idx == 1:
        # Offer Golden Idol
        result.choice_name = "offer_idol"

        # Remove Golden Idol
        for i, relic in enumerate(run_state.relics):
            if relic.id == "GoldenIdol":
                run_state.relics.pop(i)
                result.relics_lost.append("GoldenIdol")
                break

        # Gain 333 gold
        handler._apply_gold_change(run_state, 333)
        result.gold_change = 333
        result.description = "Offered the Golden Idol. Gained 333 gold."

    elif choice_idx == 2:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the Moai Head."

    return result


def _handle_winding_halls(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Winding Halls: Embrace madness, Retrace steps, Press on."""
    result = EventChoiceResult(event_id="WindingHalls", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Embrace madness: Take damage, gain 2 Madness
        result.choice_name = "embrace"
        damage = handler._damage_percent(run_state, 0.125, ascension, 0.18)
        result.hp_change = -abs(damage)

        for _ in range(2):
            run_state.add_card("Madness")
            result.cards_gained.append("Madness")

        result.description = f"Embraced the madness. Took {abs(damage)} damage, gained 2 Madness cards."

    elif choice_idx == 1:
        # Retrace steps: Heal, gain Writhe
        result.choice_name = "retrace"
        heal_percent = 0.20 if ascension >= 15 else 0.25
        heal = handler._heal_percent(run_state, heal_percent)
        result.hp_change = heal

        handler._add_curse(run_state, "Writhe")
        result.cards_gained.append("Writhe")
        result.description = f"Retraced steps. Healed {heal} HP, gained Writhe curse."

    elif choice_idx == 2:
        # Press on: Lose 5% max HP
        result.choice_name = "press_on"
        loss = handler._lose_max_hp_percent(run_state, 0.05)
        result.max_hp_change = loss
        result.description = f"Pressed on. Lost {abs(loss)} Max HP."

    return result


# ============================================================================
# EVENT HANDLERS - Shrines
# ============================================================================

def _handle_golden_shrine(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Golden Shrine: Pray (100/50 gold), Desecrate (275g + Regret), Leave."""
    result = EventChoiceResult(event_id="GoldenShrine", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Pray
        result.choice_name = "pray"
        gold = 50 if ascension >= 15 else 100
        handler._apply_gold_change(run_state, gold)
        result.gold_change = gold
        result.description = f"Prayed at the shrine. Gained {gold} gold."

    elif choice_idx == 1:
        # Desecrate
        result.choice_name = "desecrate"
        handler._apply_gold_change(run_state, 275)
        result.gold_change = 275

        handler._add_curse(run_state, "Regret")
        result.cards_gained.append("Regret")
        result.description = "Desecrated the shrine. Gained 275 gold and Regret curse."

    elif choice_idx == 2:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shrine."

    return result


def _handle_purifier(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Purifier: Pray (remove a card) or Leave."""
    result = EventChoiceResult(event_id="Purifier", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Pray - remove a card
        result.choice_name = "pray"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                result.description = f"The shrine purified {removed.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Choose a card to remove."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shrine."

    return result


def _handle_transmogrifier(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Transmogrifier: Pray (transform a card) or Leave."""
    result = EventChoiceResult(event_id="Transmogrifier", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Pray - transform
        result.choice_name = "pray"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                new_card = handler._get_random_card(run_state, misc_rng, "common")
                run_state.add_card(new_card)
                result.cards_gained.append(new_card)
                result.cards_transformed.append((removed.id, new_card))
                result.description = f"The shrine transformed {removed.id} into {new_card}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "transform"
            result.event_complete = False
            result.description = "Choose a card to transform."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shrine."

    return result


def _handle_upgrade_shrine(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Upgrade Shrine: Pray (upgrade a card) or Leave."""
    result = EventChoiceResult(event_id="UpgradeShrine", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Pray - upgrade
        result.choice_name = "pray"
        if card_idx is not None:
            if run_state.upgrade_card(card_idx):
                card = run_state.deck[card_idx]
                result.cards_upgraded.append(card.id)
                result.description = f"The shrine upgraded {card.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "upgrade"
            result.event_complete = False
            result.description = "Choose a card to upgrade."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shrine."

    return result


def _handle_duplicator(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Duplicator: Duplicate (copy a card) or Leave."""
    result = EventChoiceResult(event_id="Duplicator", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Duplicate
        result.choice_name = "duplicate"
        if card_idx is not None:
            card = run_state.deck[card_idx]
            run_state.add_card(card.id, card.upgraded, card.misc_value)
            result.cards_gained.append(card.id)
            result.description = f"Duplicated {card.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "duplicate"
            result.event_complete = False
            result.description = "Choose a card to duplicate."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the shrine."

    return result


def _handle_fountain_of_cleansing(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Fountain of Cleansing: Drink (remove all removable curses) or Leave."""
    result = EventChoiceResult(event_id="FountainOfCleansing", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Drink
        result.choice_name = "drink"

        # Remove all removable curses
        curses_to_remove = []
        for i, card in enumerate(run_state.deck):
            if card.id in handler.CURSE_CARDS and card.id not in handler.UNREMOVABLE_CURSES:
                curses_to_remove.append((i, card.id))

        # Remove in reverse order
        for i, curse_id in reversed(curses_to_remove):
            run_state.remove_card(i)
            result.cards_removed.append(curse_id)

        result.description = f"Drank from the fountain. Removed {len(curses_to_remove)} curses."

    elif choice_idx == 1:
        # Leave
        result.choice_name = "leave"
        result.description = "Left the fountain."

    return result


def _handle_the_joust(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Joust: Bet on Owner (30% win = 250g) or Murderer (70% win = 50g)."""
    result = EventChoiceResult(event_id="TheJoust", choice_idx=choice_idx, choice_name="")

    roll = misc_rng.random_float()

    if choice_idx == 0:
        # Bet on Owner (30% chance to win)
        result.choice_name = "owner"
        if roll < 0.30:
            handler._apply_gold_change(run_state, 250)
            result.gold_change = 250
            result.description = "Bet on the Owner. Won 250 gold!"
        else:
            result.description = "Bet on the Owner. Lost the bet."

    elif choice_idx == 1:
        # Bet on Murderer (70% chance to win)
        result.choice_name = "murderer"
        if roll < 0.70:
            handler._apply_gold_change(run_state, 50)
            result.gold_change = 50
            result.description = "Bet on the Murderer. Won 50 gold!"
        else:
            result.description = "Bet on the Murderer. Lost the bet."

    return result


def _handle_the_lab(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """The Lab: Enter (gain 3/2 random potions)."""
    result = EventChoiceResult(event_id="TheLab", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Enter
        result.choice_name = "enter"
        potion_count = 2 if ascension >= 15 else 3

        for _ in range(potion_count):
            if run_state.count_empty_potion_slots() > 0:
                potion = handler._get_random_potion(misc_rng)
                result.potions_gained.append(potion)

        result.description = f"Entered the lab. Gained {len(result.potions_gained)} potions."

    return result


def _handle_woman_in_blue(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Woman in Blue: Buy 1 potion (20g), 2 potions (30g), 3 potions (40g), or Leave."""
    result = EventChoiceResult(event_id="WomanInBlue", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    prices = [20, 30, 40]
    potion_counts = [1, 2, 3]

    if choice_idx < 3:
        cost = prices[choice_idx]
        count = potion_counts[choice_idx]
        result.choice_name = f"buy_{count}"

        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        potions_gained = 0
        for _ in range(count):
            if run_state.count_empty_potion_slots() > 0:
                potion = handler._get_random_potion(misc_rng)
                result.potions_gained.append(potion)
                potions_gained += 1

        result.description = f"Paid {cost} gold. Gained {potions_gained} potions."

    elif choice_idx == 3:
        # Leave
        result.choice_name = "leave"
        if ascension >= 15:
            damage = handler._damage_percent(run_state, 0.05)
            result.hp_change = -abs(damage)
            result.description = f"Left. Took {abs(damage)} damage (A15+)."
        else:
            result.description = "Left the woman in blue."

    return result


# ============================================================================
# EVENT HANDLERS - Missing implementations
# ============================================================================

def _handle_face_trader(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Face Trader: Trade face (lose HP%, gain relic), Pay gold for Ssserpent Head, Leave."""
    result = EventChoiceResult(event_id="FaceTrader", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Trade face - lose 10% max HP, gain random relic
        result.choice_name = "trade"
        damage = handler._damage_percent(run_state, 0.10)
        result.hp_change = -abs(damage)

        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Traded face. Lost {abs(damage)} HP, gained {relic}."

    elif choice_idx == 1:
        # Pay gold for Ssserpent's Head
        result.choice_name = "pay"
        cost = 75
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        run_state.add_relic("SsserpentHead")
        result.relics_gained.append("SsserpentHead")
        result.description = f"Paid {cost} gold. Gained Ssserpent Head."

    elif choice_idx == 2:
        result.choice_name = "leave"
        result.description = "Left the Face Trader."

    return result


def _handle_designer(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Designer: Remove (pay gold), Transform (pay gold), Upgrade (pay gold), Leave."""
    result = EventChoiceResult(event_id="Designer", choice_idx=choice_idx, choice_name="")
    ascension = run_state.ascension

    if choice_idx == 0:
        # Remove a card (costs vary)
        result.choice_name = "remove"
        cost = 75 if ascension >= 15 else 50
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                result.description = f"Paid {cost} gold. Removed {removed.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = f"Paid {cost} gold. Choose a card to remove."

    elif choice_idx == 1:
        # Transform a card
        result.choice_name = "transform"
        cost = 50 if ascension >= 15 else 35
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                new_card = handler._get_random_card(run_state, misc_rng, "common")
                run_state.add_card(new_card)
                result.cards_gained.append(new_card)
                result.cards_transformed.append((removed.id, new_card))
                result.description = f"Paid {cost} gold. Transformed {removed.id} into {new_card}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "transform"
            result.event_complete = False
            result.description = f"Paid {cost} gold. Choose a card to transform."

    elif choice_idx == 2:
        # Upgrade a card
        result.choice_name = "upgrade"
        cost = 40 if ascension >= 15 else 25
        handler._apply_gold_change(run_state, -cost)
        result.gold_change = -cost

        if card_idx is not None:
            if run_state.upgrade_card(card_idx):
                card = run_state.deck[card_idx]
                result.cards_upgraded.append(card.id)
                result.description = f"Paid {cost} gold. Upgraded {card.id}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "upgrade"
            result.event_complete = False
            result.description = f"Paid {cost} gold. Choose a card to upgrade."

    elif choice_idx == 3:
        result.choice_name = "leave"
        result.description = "Left the designer."

    return result


def _handle_nloth(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """N'loth: Trade a relic for N'loth's Gift, or Leave."""
    result = EventChoiceResult(event_id="Nloth", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Trade oldest non-starter relic for N'loth's Gift
        result.choice_name = "trade"
        if len(run_state.relics) > 0:
            traded = run_state.relics.pop(0)
            traded_id = traded.id if hasattr(traded, 'id') else str(traded)
            result.relics_lost.append(traded_id)

        run_state.add_relic("NlothsGift")
        result.relics_gained.append("NlothsGift")
        result.description = "Traded a relic for N'loth's Gift."

    elif choice_idx == 1:
        result.choice_name = "leave"
        result.description = "Left N'loth."

    return result


def _handle_accursed_blacksmith(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Accursed Blacksmith: Upgrade a card (gain Parasite curse) or Leave."""
    result = EventChoiceResult(event_id="AccursedBlacksmith", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Upgrade a card, gain Parasite
        result.choice_name = "upgrade"
        if card_idx is not None:
            if run_state.upgrade_card(card_idx):
                card = run_state.deck[card_idx]
                result.cards_upgraded.append(card.id)
            handler._add_curse(run_state, "Parasite")
            result.cards_gained.append("Parasite")
            result.description = f"Upgraded a card. Gained Parasite curse."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "upgrade"
            result.event_complete = False
            result.description = "Choose a card to upgrade (will gain Parasite curse)."

    elif choice_idx == 1:
        result.choice_name = "leave"
        result.description = "Left the blacksmith."

    return result


def _handle_bonfire_elementals(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Bonfire Elementals: Offer a card (gain relic) or Leave."""
    result = EventChoiceResult(event_id="BonfireElementals", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Offer a card, gain relic
        result.choice_name = "offer"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
            relic = handler._get_random_relic(run_state, misc_rng, "common")
            run_state.add_relic(relic)
            result.relics_gained.append(relic)
            result.description = f"Offered a card. Gained {relic}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Choose a card to offer."

    elif choice_idx == 1:
        result.choice_name = "leave"
        result.description = "Left the bonfire."

    return result


def _handle_we_meet_again(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """We Meet Again: Give potion (relic), Give gold (card), Give card (potion), Leave."""
    result = EventChoiceResult(event_id="WeMeetAgain", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Give potion, get relic
        result.choice_name = "give_potion"
        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Gave a potion. Gained {relic}."

    elif choice_idx == 1:
        # Give gold, get random card
        result.choice_name = "give_gold"
        handler._apply_gold_change(run_state, -50)
        result.gold_change = -50
        card = handler._get_random_card(run_state, misc_rng, "uncommon")
        run_state.add_card(card)
        result.cards_gained.append(card)
        result.description = f"Gave 50 gold. Gained {card}."

    elif choice_idx == 2:
        # Give card, get potion
        result.choice_name = "give_card"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
            potion = handler._get_random_potion(misc_rng)
            result.potions_gained.append(potion)
            result.description = f"Gave a card. Gained {potion}."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Choose a card to give."

    elif choice_idx == 3:
        result.choice_name = "leave"
        result.description = "Left."

    return result


def _handle_augmenter(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Augmenter: Mechanical (J.A.X. + remove card), Mutagenic (Str or Dex), Transform 2."""
    result = EventChoiceResult(event_id="Augmenter", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Mechanical enhancement - get J.A.X., remove a card
        result.choice_name = "mechanical"
        run_state.add_card("J.A.X.")
        result.cards_gained.append("J.A.X.")

        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
            result.description = f"Mechanical enhancement. Gained J.A.X., removed a card."
        else:
            result.requires_card_selection = True
            result.card_selection_type = "remove"
            result.event_complete = False
            result.description = "Gained J.A.X. Choose a card to remove."

    elif choice_idx == 1:
        # Mutagenic - random Strength or Dexterity (represented as relic/buff)
        result.choice_name = "mutagenic"
        if misc_rng.random_float() < 0.5:
            run_state.add_relic("MutagenicStrength")
            result.relics_gained.append("MutagenicStrength")
            result.description = "Mutagenic enhancement. Gained Strength."
        else:
            run_state.add_relic("MutagenicDexterity")
            result.relics_gained.append("MutagenicDexterity")
            result.description = "Mutagenic enhancement. Gained Dexterity."

    elif choice_idx == 2:
        # Transform 2 cards
        result.choice_name = "transform"
        if card_idx is not None:
            removed = run_state.remove_card(card_idx)
            if removed:
                result.cards_removed.append(removed.id)
                new_card = handler._get_random_card(run_state, misc_rng, "common")
                run_state.add_card(new_card)
                result.cards_gained.append(new_card)
                result.cards_transformed.append((removed.id, new_card))
            result.description = "Transformed a card."
            # Would need state tracking for second transform
        else:
            result.requires_card_selection = True
            result.card_selection_type = "transform"
            result.card_selection_count = 2
            result.event_complete = False
            result.description = "Choose 2 cards to transform."

    return result


def _handle_beggar(
    handler: EventHandler,
    event_state: EventState,
    choice_idx: int,
    run_state: 'RunState',
    event_rng: 'Random',
    card_idx: Optional[int] = None,
    misc_rng: Optional['Random'] = None
) -> EventChoiceResult:
    """Beggar: Donate 50g (relic), Donate 100g (relic + remove curse), Leave."""
    result = EventChoiceResult(event_id="Beggar", choice_idx=choice_idx, choice_name="")

    if choice_idx == 0:
        # Donate 50g
        result.choice_name = "donate_50"
        handler._apply_gold_change(run_state, -50)
        result.gold_change = -50

        relic = handler._get_random_relic(run_state, misc_rng, "common")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)
        result.description = f"Donated 50 gold. Gained {relic}."

    elif choice_idx == 1:
        # Donate 100g - relic + remove a curse if any
        result.choice_name = "donate_100"
        handler._apply_gold_change(run_state, -100)
        result.gold_change = -100

        relic = handler._get_random_relic(run_state, misc_rng, "uncommon")
        run_state.add_relic(relic)
        result.relics_gained.append(relic)

        # Remove a curse if one exists
        curses = handler._get_removable_curses(run_state)
        if curses:
            idx, curse_card = curses[0]
            run_state.remove_card(idx)
            result.cards_removed.append(curse_card.id)
            result.description = f"Donated 100 gold. Gained {relic}, removed {curse_card.id}."
        else:
            result.description = f"Donated 100 gold. Gained {relic}."

    elif choice_idx == 2:
        result.choice_name = "leave"
        result.description = "Left the beggar."

    return result


# ============================================================================
# EVENT HANDLER REGISTRY
# ============================================================================

EVENT_HANDLERS: Dict[str, Callable] = {
    # Act 1
    "BigFish": _handle_big_fish,
    "TheCleric": _handle_the_cleric,
    "GoldenIdol": _handle_golden_idol,
    "WorldOfGoop": _handle_world_of_goop,
    "LivingWall": _handle_living_wall,
    "ScrapOoze": _handle_scrap_ooze,
    "Sssserpent": _handle_sssserpent,
    "WingStatue": _handle_wing_statue,
    "ShiningLight": _handle_shining_light,
    "DeadAdventurer": _handle_dead_adventurer,
    "Mushrooms": _handle_mushrooms,

    # Act 2
    "Colosseum": _handle_colosseum,
    "CursedTome": _handle_cursed_tome,
    "TheLibrary": _handle_the_library,
    "TheMausoleum": _handle_the_mausoleum,
    "MaskedBandits": _handle_masked_bandits,
    "KnowingSkull": _handle_knowing_skull,
    "Ghosts": _handle_ghosts,
    "Vampires": _handle_vampires,
    "BackToBasics": _handle_back_to_basics,
    "ForgottenAltar": _handle_forgotten_altar,
    "Nest": _handle_nest,
    "Addict": _handle_addict,

    # Act 3
    "Falling": _handle_falling,
    "MindBloom": _handle_mind_bloom,
    "MysteriousSphere": _handle_mysterious_sphere,
    "SecretPortal": _handle_secret_portal,
    "SensoryStone": _handle_sensory_stone,
    "TombOfLordRedMask": _handle_tomb_of_lord_red_mask,
    "MoaiHead": _handle_moai_head,
    "WindingHalls": _handle_winding_halls,

    # Shrines
    "GoldenShrine": _handle_golden_shrine,
    "Purifier": _handle_purifier,
    "Transmogrifier": _handle_transmogrifier,
    "UpgradeShrine": _handle_upgrade_shrine,
    "Duplicator": _handle_duplicator,

    # Special one-time events
    "FountainOfCleansing": _handle_fountain_of_cleansing,
    "TheJoust": _handle_the_joust,
    "TheLab": _handle_the_lab,
    "WomanInBlue": _handle_woman_in_blue,
    "FaceTrader": _handle_face_trader,
    "Designer": _handle_designer,
    "Nloth": _handle_nloth,
    "AccursedBlacksmith": _handle_accursed_blacksmith,
    "BonfireElementals": _handle_bonfire_elementals,
    "WeMeetAgain": _handle_we_meet_again,
    "Augmenter": _handle_augmenter,
    "Beggar": _handle_beggar,
}


# ============================================================================
# EVENT CHOICE REGISTRY
# ============================================================================

def _get_big_fish_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Big Fish event."""
    return [
        EventChoice(index=0, name="banana", text="[Banana] Heal 33% of Max HP"),
        EventChoice(index=1, name="donut", text="[Donut] Gain 5 Max HP"),
        EventChoice(index=2, name="box", text="[Box] Gain random relic, obtain Regret curse"),
    ]


def _get_the_cleric_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for The Cleric event."""
    ascension = run_state.ascension
    purify_cost = 75 if ascension >= 15 else 50

    return [
        EventChoice(
            index=0, name="heal", text="[Heal] Pay 35 gold, heal 25% Max HP",
            requires_gold=35, requires_max_hp_missing=True
        ),
        EventChoice(
            index=1, name="purify", text=f"[Purify] Pay {purify_cost} gold, remove a card",
            requires_gold=purify_cost, requires_removable_cards=True
        ),
        EventChoice(index=2, name="leave", text="[Leave]"),
    ]


def _get_golden_idol_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Golden Idol event."""
    if phase == EventPhase.INITIAL:
        return [
            EventChoice(index=0, name="take", text="[Take] Obtain Golden Idol"),
            EventChoice(index=1, name="leave", text="[Leave]"),
        ]
    elif phase == EventPhase.SECONDARY:
        return [
            EventChoice(index=0, name="outrun", text="[Outrun] Obtain Injury curse"),
            EventChoice(index=1, name="smash", text="[Smash] Take 25% Max HP damage"),
            EventChoice(index=2, name="hide", text="[Hide] Lose 8% Max HP"),
        ]
    return []


def _get_world_of_goop_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for World of Goop event."""
    return [
        EventChoice(index=0, name="gather", text="[Gather Gold] Gain 75 gold, take 11 damage"),
        EventChoice(index=1, name="leave", text="[Leave] Lose gold"),
    ]


def _get_living_wall_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Living Wall event."""
    return [
        EventChoice(
            index=0, name="forget", text="[Forget] Remove a card",
            requires_removable_cards=True
        ),
        EventChoice(
            index=1, name="change", text="[Change] Transform a card",
            requires_transformable_cards=True
        ),
        EventChoice(
            index=2, name="grow", text="[Grow] Upgrade a card",
            requires_upgradable_cards=True
        ),
    ]


def _get_scrap_ooze_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Scrap Ooze event."""
    base_damage = 5 if run_state.ascension >= 15 else 3
    damage = base_damage + event_state.attempt_count
    success_chance = int((0.25 + event_state.attempt_count * 0.10) * 100)

    return [
        EventChoice(
            index=0, name="reach",
            text=f"[Reach In] Take {damage} damage, {success_chance}% chance for relic"
        ),
        EventChoice(index=1, name="leave", text="[Leave]"),
    ]


def _get_colosseum_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Colosseum event."""
    if phase == EventPhase.INITIAL:
        return [
            EventChoice(index=0, name="enter", text="[Enter] Fight the Slavers"),
        ]
    elif phase == EventPhase.SECONDARY:
        return [
            EventChoice(index=0, name="fight_nobs", text="[Fight] Fight 2 Taskmasters for big rewards"),
            EventChoice(index=1, name="cowardice", text="[Cowardice] Leave"),
        ]
    return []


def _get_the_library_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for The Library event."""
    return [
        EventChoice(index=0, name="read", text="[Read] Choose 1 of 20 cards"),
        EventChoice(index=1, name="sleep", text="[Sleep] Heal to full HP"),
    ]


def _get_the_mausoleum_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for The Mausoleum event."""
    return [
        EventChoice(index=0, name="open", text="[Open] 50% relic, 50% curse"),
        EventChoice(index=1, name="leave", text="[Leave]"),
    ]


def _get_masked_bandits_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Masked Bandits event."""
    return [
        EventChoice(index=0, name="pay", text="[Pay] Lose all gold"),
        EventChoice(index=1, name="fight", text="[Fight] Fight 3 bandits for gold + relic"),
    ]


def _get_knowing_skull_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Knowing Skull event."""
    base_cost = 6 + event_state.hp_cost_modifier

    return [
        EventChoice(
            index=0, name="potion",
            text=f"[A Potion] Pay {base_cost} HP for random potion",
            requires_empty_potion_slot=True,
            requires_min_hp=base_cost + 1
        ),
        EventChoice(
            index=1, name="gold",
            text=f"[Gold] Pay {base_cost} HP for 90 gold",
            requires_min_hp=base_cost + 1
        ),
        EventChoice(
            index=2, name="card",
            text=f"[A Card] Pay {base_cost} HP for colorless uncommon",
            requires_min_hp=base_cost + 1
        ),
        EventChoice(
            index=3, name="leave",
            text="[Leave] Pay 6 HP to escape",
            requires_min_hp=7
        ),
    ]


def _get_mind_bloom_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for Mind Bloom event."""
    return [
        EventChoice(index=0, name="war", text="[I am War] Fight Act 1 boss for rare relic"),
        EventChoice(index=1, name="awake", text="[I am Awake] Upgrade all cards, get Mark of the Bloom"),
        EventChoice(index=2, name="rich", text="[I am Rich] Gain 999 gold, obtain 2 Normality curses"),
    ]


def _get_shrine_choices(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Generic shrine choices (Purifier, Transmogrifier, Upgrade Shrine)."""
    if event_id == "Purifier":
        return [
            EventChoice(index=0, name="pray", text="[Pray] Remove a card", requires_removable_cards=True),
            EventChoice(index=1, name="leave", text="[Leave]"),
        ]
    elif event_id == "Transmogrifier":
        return [
            EventChoice(index=0, name="pray", text="[Pray] Transform a card", requires_transformable_cards=True),
            EventChoice(index=1, name="leave", text="[Leave]"),
        ]
    elif event_id == "UpgradeShrine":
        return [
            EventChoice(index=0, name="pray", text="[Pray] Upgrade a card", requires_upgradable_cards=True),
            EventChoice(index=1, name="leave", text="[Leave]"),
        ]
    elif event_id == "Duplicator":
        return [
            EventChoice(index=0, name="duplicate", text="[Duplicate] Copy a card"),
            EventChoice(index=1, name="leave", text="[Leave]"),
        ]
    elif event_id == "GoldenShrine":
        return [
            EventChoice(index=0, name="pray", text="[Pray] Gain gold"),
            EventChoice(index=1, name="desecrate", text="[Desecrate] Gain 275 gold + Regret"),
            EventChoice(index=2, name="leave", text="[Leave]"),
        ]
    return [EventChoice(index=0, name="leave", text="[Leave]")]


EVENT_CHOICE_GENERATORS: Dict[str, Callable] = {
    "BigFish": _get_big_fish_choices,
    "TheCleric": _get_the_cleric_choices,
    "GoldenIdol": _get_golden_idol_choices,
    "WorldOfGoop": _get_world_of_goop_choices,
    "LivingWall": _get_living_wall_choices,
    "ScrapOoze": _get_scrap_ooze_choices,
    "Colosseum": _get_colosseum_choices,
    "TheLibrary": _get_the_library_choices,
    "TheMausoleum": _get_the_mausoleum_choices,
    "MaskedBandits": _get_masked_bandits_choices,
    "KnowingSkull": _get_knowing_skull_choices,
    "MindBloom": _get_mind_bloom_choices,
    "Purifier": _get_shrine_choices,
    "Transmogrifier": _get_shrine_choices,
    "UpgradeShrine": _get_shrine_choices,
    "Duplicator": _get_shrine_choices,
    "GoldenShrine": _get_shrine_choices,
}


def _get_event_choices_impl(
    handler: EventHandler,
    event_id: str,
    phase: EventPhase,
    event_state: EventState,
    run_state: 'RunState'
) -> List[EventChoice]:
    """Get choices for an event."""
    generator = EVENT_CHOICE_GENERATORS.get(event_id)
    if generator:
        return generator(handler, event_id, phase, event_state, run_state)
    return _get_event_choices_default(handler, event_id, phase, event_state, run_state)


# Add the method to EventHandler class
def _get_event_choices_method(self, event_id, phase, event_state, run_state):
    """Wrapper to call the implementation function."""
    return _get_event_choices_impl(self, event_id, phase, event_state, run_state)


EventHandler._get_event_choices = _get_event_choices_method
