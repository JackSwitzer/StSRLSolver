"""
Run State Tracker - Complete state of a Slay the Spire run in progress.

Tracks everything needed to:
1. Simulate game decisions deterministically
2. Save/restore exact game state
3. Provide complete information for RL agent decisions

The RunState is the ground truth for what the agent knows about the current run.
"""

from dataclasses import dataclass, field
from typing import List, Dict, Set, Optional, Tuple
from copy import deepcopy

from ..generation.map import MapRoomNode, RoomType, MapGenerator, MapGeneratorConfig, get_map_seed_offset
from ..generation.rewards import (
    RewardState,
    generate_card_rewards,
    generate_relic_reward,
    generate_potion_reward,
    RelicTier,
)
from ..content.cards import ALL_CARDS, CardType
from .rng import Random, GameRNG, seed_to_long, long_to_seed


@dataclass
class CardInstance:
    """
    A card in the deck (may be upgraded).

    Cards are tracked by instance, not just ID, because:
    - Upgrades are per-card
    - Removal targets specific cards
    - Some effects care about "this card" vs "a card"
    """
    id: str
    upgraded: bool = False

    # For tracking specific cards (e.g., Searing Blow upgrade count)
    misc_value: int = 0

    def __repr__(self) -> str:
        suffix = "+" if self.upgraded else ""
        if self.id == "SearingBlow" and self.misc_value > 0:
            suffix = f"+{self.misc_value}"
        return f"{self.id}{suffix}"

    def copy(self) -> 'CardInstance':
        """Create a deep copy of this card instance."""
        return CardInstance(
            id=self.id,
            upgraded=self.upgraded,
            misc_value=self.misc_value
        )


@dataclass
class RelicInstance:
    """
    A relic with its counter state.

    Many relics track counters:
    - Pen Nib: Attacks until 10x damage
    - Ink Bottle: Cards until draw
    - Happy Flower: Turns until energy
    - Nunchaku: Attacks until energy
    - etc.

    counter = -1 means the relic has no counter.
    """
    id: str
    counter: int = -1  # -1 = no counter

    # Some relics need additional state
    triggered_this_combat: bool = False  # e.g., Lizard Tail
    triggered_this_turn: bool = False    # e.g., Orichalcum
    card_id: Optional[str] = None        # e.g., bottled card ID (with + if upgraded)

    def __repr__(self) -> str:
        if self.counter >= 0:
            return f"{self.id}({self.counter})"
        return self.id

    def copy(self) -> 'RelicInstance':
        """Create a deep copy of this relic instance."""
        return RelicInstance(
            id=self.id,
            counter=self.counter,
            triggered_this_combat=self.triggered_this_combat,
            triggered_this_turn=self.triggered_this_turn,
            card_id=self.card_id,
        )


@dataclass
class PotionSlot:
    """
    A potion slot (may be empty).

    Watcher starts with 2 potion slots (3 at A11+).
    Potion Belt adds 2 more slots.
    """
    potion_id: Optional[str] = None

    def is_empty(self) -> bool:
        return self.potion_id is None

    def __repr__(self) -> str:
        return self.potion_id if self.potion_id else "[empty]"

    def copy(self) -> 'PotionSlot':
        """Create a deep copy of this potion slot."""
        return PotionSlot(potion_id=self.potion_id)


@dataclass
class MapPosition:
    """Current position on the map."""
    x: int = -1  # -1 = not on map yet (at start of act)
    y: int = -1

    def is_at_start(self) -> bool:
        """Check if we're at the start (haven't entered map yet)."""
        return self.x == -1 and self.y == -1

    def __repr__(self) -> str:
        if self.is_at_start():
            return "Start"
        return f"({self.x}, {self.y})"


@dataclass
class RunState:
    """
    Complete state of a run in progress.

    This is the master state object that tracks everything about
    the current run. It should contain all information needed to:
    1. Make optimal decisions
    2. Simulate future states
    3. Save/restore game state exactly
    """

    # ==================== SEED & RNG ====================
    seed: int                    # Numeric seed value
    seed_string: str             # String representation (e.g., "ABC123")

    # ==================== RUN PROGRESS ====================
    act: int = 1                 # Current act (1-4)
    floor: int = 0               # Current floor within act
    ascension: int = 20          # Ascension level (0-20)
    character: str = "Watcher"   # Character class

    # ==================== RESOURCES ====================
    current_hp: int = 72         # Current HP
    max_hp: int = 72             # Maximum HP
    gold: int = 99               # Current gold
    gold_blocked: int = 0        # Gold blocked by Ectoplasm

    # ==================== DECK ====================
    deck: List[CardInstance] = field(default_factory=list)

    # ==================== RELICS ====================
    relics: List[RelicInstance] = field(default_factory=list)

    # ==================== POTIONS ====================
    potion_slots: List[PotionSlot] = field(default_factory=list)

    # ==================== MAP STATE ====================
    # Maps for each act (generated lazily)
    act_maps: Dict[int, List[List[MapRoomNode]]] = field(default_factory=dict)

    # Current position
    map_position: MapPosition = field(default_factory=MapPosition)

    # Path history (for tracking which nodes we've visited)
    visited_nodes: List[Tuple[int, int, int]] = field(default_factory=list)  # (act, x, y)

    # ==================== REWARD POOL TRACKING ====================
    # These affect what cards/relics can appear in rewards
    seen_cards: Set[str] = field(default_factory=set)
    seen_relics: Set[str] = field(default_factory=set)

    # Cards obtained this act (for Prismatic Shard logic)
    cards_obtained_this_act: List[str] = field(default_factory=list)

    # ==================== KEYS (ACT 4 ACCESS) ====================
    has_ruby_key: bool = False      # From rest site (skip heal/upgrade)
    has_emerald_key: bool = False   # From burning elite
    has_sapphire_key: bool = False  # From chest (skip relic)

    # ==================== RNG COUNTERS ====================
    # For save/load determinism - tracks how many times each RNG was called
    rng_counters: Dict[str, int] = field(default_factory=dict)

    # ==================== BLIZZARD MODIFIERS (PITY TIMERS) ====================
    # Card rarity blizzard: increases rare card chance
    # Starts at 5, decrements on non-rare, resets to 5 on rare
    card_blizzard: int = 5

    # Potion drop blizzard: increases potion drop chance
    # Starts at 0, increments by 10 on no drop, resets to 0 on drop
    potion_blizzard: int = 0

    # ==================== COMBAT TRACKING ====================
    # Elites killed this act (for elite pool management)
    elites_killed_this_act: int = 0

    # Total floors climbed (for some relic effects)
    floors_climbed: int = 0

    # Combats won (for achievement/stats tracking)
    combats_won: int = 0
    elites_killed: int = 0
    bosses_killed: int = 0

    # Perfect floors (no damage taken) - for some event logic
    perfect_floors: int = 0

    # Shop purge count (card removals - affects future shop prices)
    purge_count: int = 0

    # ==================== SPECIAL FLAGS ====================
    # Neow's Lament counter (if obtained)
    neow_lament_count: int = 0

    # Question Card counter (if have relic)
    question_card_charges: int = 0

    # ==================== METHODS ====================

    # ----- DECK MANAGEMENT -----

    _EGG_RELIC_BY_TYPE = {
        CardType.POWER: "Frozen Egg 2",
        CardType.ATTACK: "Molten Egg 2",
        CardType.SKILL: "Toxic Egg 2",
    }

    def _apply_on_obtain_card_upgrades(
        self,
        card_id: str,
        upgraded: bool,
    ) -> Tuple[Optional[object], bool]:
        """Apply upgrade modifiers for card acquisition (Egg relics)."""
        card_def = ALL_CARDS.get(card_id)
        if card_def is None or upgraded:
            return card_def, upgraded

        egg_relic = self._EGG_RELIC_BY_TYPE.get(card_def.card_type)
        if egg_relic and self.has_relic(egg_relic):
            upgraded = True

        return card_def, upgraded

    def _apply_on_obtain_card_side_effects(
        self,
        card_id: str,
        card_def: Optional[object],
    ) -> None:
        """Apply non-upgrade relic effects from card acquisition."""
        if self.has_relic("CeramicFish"):
            self.add_gold(9)

        if card_def is not None and getattr(card_def, "card_type", None) == CardType.CURSE:
            if self.has_relic("Darkstone Periapt"):
                self.gain_max_hp(6)

    def add_card(self, card_id: str, upgraded: bool = False, misc_value: int = 0) -> CardInstance:
        """
        Add a card to the deck.

        Args:
            card_id: Base card ID (e.g., "Strike_P" for Watcher Strike)
            upgraded: Whether the card is upgraded
            misc_value: Special value (e.g., Searing Blow upgrade count)

        Returns:
            The created CardInstance
        """
        card_def, upgraded = self._apply_on_obtain_card_upgrades(card_id, upgraded)

        card = CardInstance(id=card_id, upgraded=upgraded, misc_value=misc_value)
        self.deck.append(card)
        self.cards_obtained_this_act.append(card_id)
        self.seen_cards.add(card_id)

        self._apply_on_obtain_card_side_effects(card_id, card_def)

        return card

    def remove_card(self, card_idx: int) -> Optional[CardInstance]:
        """
        Remove a card from the deck by index.

        Args:
            card_idx: Index of card to remove

        Returns:
            The removed card, or None if index invalid
        """
        if 0 <= card_idx < len(self.deck):
            return self.deck.pop(card_idx)
        return None

    def remove_card_by_id(self, card_id: str, upgraded: Optional[bool] = None) -> Optional[CardInstance]:
        """
        Remove first card matching the ID (and optionally upgrade status).

        Args:
            card_id: Card ID to remove
            upgraded: If specified, must also match upgrade status

        Returns:
            The removed card, or None if not found
        """
        for i, card in enumerate(self.deck):
            if card.id == card_id:
                if upgraded is None or card.upgraded == upgraded:
                    return self.deck.pop(i)
        return None

    def upgrade_card(self, card_idx: int) -> bool:
        """
        Upgrade a card by index.

        Args:
            card_idx: Index of card to upgrade

        Returns:
            True if upgraded, False if invalid or already upgraded
        """
        if 0 <= card_idx < len(self.deck):
            card = self.deck[card_idx]
            if not card.upgraded:
                card.upgraded = True
                return True
            # Searing Blow can be upgraded multiple times
            if card.id == "SearingBlow":
                card.misc_value += 1
                return True
        return False

    def get_deck_card_ids(self) -> List[str]:
        """
        Get list of card IDs for combat initialization.

        Returns:
            List of card IDs (with + suffix if upgraded)
        """
        result = []
        for card in self.deck:
            card_id = card.id
            if card.upgraded:
                card_id += "+"
            result.append(card_id)
        return result

    def count_card(self, card_id: str, upgraded_only: bool = False) -> int:
        """Count how many copies of a card are in deck."""
        count = 0
        for card in self.deck:
            if card.id == card_id:
                if not upgraded_only or card.upgraded:
                    count += 1
        return count

    def get_upgradeable_cards(self) -> List[Tuple[int, CardInstance]]:
        """Get indices and cards that can be upgraded."""
        return [
            (i, card) for i, card in enumerate(self.deck)
            if not card.upgraded or card.id == "SearingBlow"
        ]

    def get_removable_cards(self) -> List[Tuple[int, CardInstance]]:
        """Get indices and cards that can be removed (all of them)."""
        return list(enumerate(self.deck))

    def get_transformable_cards(self) -> List[Tuple[int, CardInstance]]:
        """Get indices and cards that can be transformed (non-basic)."""
        basic_cards = {"Strike_P", "Defend_P", "Eruption", "Vigilance", "AscendersBane"}
        return [
            (i, card) for i, card in enumerate(self.deck)
            if card.id not in basic_cards
        ]

    # ----- REWARD HELPERS -----

    def get_card_reward_count(self) -> int:
        """
        Get cards shown in a single card reward after relic modifiers.

        Note: Prayer Wheel normally adds an additional reward roll, not +1 card
        to a single roll. This helper follows current tests, where it contributes
        +1 to the per-reward choice count.
        """
        count = 3
        if self.has_relic("Prayer Wheel"):
            count += 1
        if self.has_relic("Question Card"):
            count += 1
        if self.has_relic("BustedCrown"):
            count -= 2
        return max(1, count)

    def skip_card_reward(self) -> None:
        """Apply effects when the player skips a card reward."""
        if self.has_relic("Singing Bowl"):
            self.gain_max_hp(2)

    # ----- RELIC HELPERS -----

    _RNG_STREAM_OFFSETS = {
        "misc": 4000,
        "card": 5000,
        "potion": 7000,
        "relic": 8000,
    }

    def _get_rng(self, stream: str, override: Optional[Random] = None) -> Random:
        """Resolve an RNG stream, preferring overrides and cached RNGs."""
        if override is not None:
            return override
        rng_attr = f"{stream}_rng"
        rng = getattr(self, rng_attr, None)
        if rng is not None:
            return rng
        offset = self._RNG_STREAM_OFFSETS.get(stream, 0)
        counter = self.get_rng_counter(stream, 0)
        return Random(self.seed + offset, counter)

    def _sync_rng_counter(self, stream: str, rng: Random) -> None:
        """Persist RNG counter after local consumption."""
        self.rng_counters[stream] = int(rng.counter)

    def _shuffle_indices(self, indices: List[int], rng: Random) -> None:
        """Shuffle indices in-place using game RNG."""
        for i in range(len(indices) - 1, 0, -1):
            j = rng.random(i)
            indices[i], indices[j] = indices[j], indices[i]

    def _upgrade_random_cards(self, card_type: CardType, count: int, rng: Random) -> int:
        """Upgrade up to count cards of the given type. Returns upgrades performed."""
        eligible = []
        for i, card in enumerate(self.deck):
            card_def = ALL_CARDS.get(card.id)
            if not card_def or card_def.card_type != card_type:
                continue
            if not card.upgraded or card.id == "SearingBlow":
                eligible.append(i)
        if not eligible:
            return 0
        self._shuffle_indices(eligible, rng)
        upgrades = 0
        for idx in eligible[:count]:
            if self.upgrade_card(idx):
                upgrades += 1
        return upgrades

    def _roll_random_reward_card(self, card_rng: Random) -> Optional[str]:
        """Roll a random card from reward logic and return its ID."""
        reward_state = RewardState()
        cards = generate_card_rewards(
            card_rng,
            reward_state,
            act=self.act,
            player_class=self.character,
            ascension=self.ascension,
            room_type="normal",
            num_cards=1,
        )
        if cards:
            return cards[0].id
        return None

    def _select_random_index(self, indices: List[int], rng: Optional[Random]) -> Optional[int]:
        """Select a random index from a list, or the first if RNG missing."""
        if not indices:
            return None
        if rng is None:
            return indices[0]
        return indices[rng.random(len(indices) - 1)]

    def get_bottled_cards(self) -> Dict[str, str]:
        """Get bottled card mapping for combat start."""
        bottled = {}
        for relic in self.relics:
            if relic.id in ("Bottled Flame", "Bottled Lightning", "Bottled Tornado"):
                if relic.card_id:
                    bottled[relic.id] = relic.card_id
        return bottled

    # ----- RELIC MANAGEMENT -----

    _RELIC_KEY_ALIASES = {
        # Java/content variants: "Mark of the Bloom" vs "MarkOfBloom"
        "markofthebloom": "markofbloom",
    }

    @staticmethod
    def _relic_lookup_key(relic_id: str) -> str:
        """Normalize relic IDs for alias-safe comparisons."""
        key = "".join(ch.lower() for ch in relic_id if ch.isalnum())
        return RunState._RELIC_KEY_ALIASES.get(key, key)

    def add_relic(
        self,
        relic_id: str,
        counter: int = -1,
        *,
        misc_rng: Optional[Random] = None,
        card_rng: Optional[Random] = None,
        relic_rng: Optional[Random] = None,
        potion_rng: Optional[Random] = None,
        selection_card_indices: Optional[List[int]] = None,
    ) -> RelicInstance:
        """
        Add a relic.

        Args:
            relic_id: Relic ID
            counter: Initial counter value (-1 for no counter)

        Returns:
            The created RelicInstance
        """
        # Java behavior: duplicate relics become Circlet.
        if relic_id != "Circlet" and self.has_relic(relic_id):
            relic_id = "Circlet"

        relic = RelicInstance(id=relic_id, counter=counter)
        self.relics.append(relic)
        self.seen_relics.add(relic_id)

        # Handle immediate effects
        self._on_relic_obtained(
            relic,
            misc_rng=misc_rng,
            card_rng=card_rng,
            relic_rng=relic_rng,
            potion_rng=potion_rng,
            selection_card_indices=selection_card_indices,
        )

        return relic

    def _on_relic_obtained(
        self,
        relic: RelicInstance,
        *,
        misc_rng: Optional[Random] = None,
        card_rng: Optional[Random] = None,
        relic_rng: Optional[Random] = None,
        potion_rng: Optional[Random] = None,
        selection_card_indices: Optional[List[int]] = None,
    ):
        """Handle effects that trigger when obtaining a relic."""
        relic_id = relic.id
        used_misc = None
        used_card = None
        used_relic = None
        used_potion = None

        # Max HP changes
        if relic_id == "MarkOfPain":
            self.gain_max_hp(12)  # A15+: also gain 2 HP per combat
        elif relic_id == "BustedCrown":
            self.gain_max_hp(8)
        elif relic_id == "PhilosopherStone":
            self.gain_max_hp(10)
        elif relic_id == "Sozu":
            self.gain_max_hp(10)
        elif relic_id == "SneckoEye":
            self.gain_max_hp(8)
        elif relic_id == "VelvetChoker":
            self.gain_max_hp(8)
        elif relic_id == "Runic Dome":
            self.gain_max_hp(8)
        elif relic_id in ("CursedKey", "Cursed Key"):
            self.gain_max_hp(10)
        elif relic_id == "FusionHammer":
            self.gain_max_hp(12)
        elif relic_id == "Coffee Dripper":
            self.gain_max_hp(12)
        elif relic_id in ("BlackStar", "Black Star"):
            self.gain_max_hp(8)
        elif relic_id == "SacredBark":
            self.gain_max_hp(12)
        elif relic_id == "DuVuDoll":
            self.gain_max_hp(8)
        elif relic_id == "Strawberry":
            self.gain_max_hp(7)
        elif relic_id == "Pear":
            self.gain_max_hp(10)
        elif relic_id == "Mango":
            self.gain_max_hp(14)
        elif relic_id == "VioletLotus":
            pass  # Watcher-specific energy on exit Calm
        elif relic_id == "Potion Belt":
            # Add 2 more potion slots
            self.potion_slots.append(PotionSlot())
            self.potion_slots.append(PotionSlot())

        # Counter setup for specific relics
        if relic_id == "Tiny Chest":
            relic.counter = 0
        elif relic_id == "Matryoshka":
            relic.counter = 2
        elif relic_id == "MawBank":
            relic.counter = 0
        elif relic_id in ("NlothsMask", "N'loth's Mask", "Nloth's Hungry Face"):
            # Java: NlothsMask starts at 1 and consumes itself after first non-boss chest.
            relic.counter = 1

        # War Paint / Whetstone (misc RNG)
        if relic_id == "War Paint":
            used_misc = self._get_rng("misc", misc_rng)
            self._upgrade_random_cards(CardType.SKILL, 2, used_misc)
        elif relic_id == "Whetstone":
            used_misc = self._get_rng("misc", misc_rng)
            self._upgrade_random_cards(CardType.ATTACK, 2, used_misc)

        # Bottled relics: choose a card (deterministic first eligible if unset)
        if relic_id in ("Bottled Flame", "Bottled Lightning", "Bottled Tornado"):
            if relic.card_id is None:
                if relic_id == "Bottled Flame":
                    eligible = [i for i, c in enumerate(self.deck)
                                if ALL_CARDS.get(c.id) and ALL_CARDS[c.id].card_type == CardType.ATTACK]
                elif relic_id == "Bottled Lightning":
                    eligible = [i for i, c in enumerate(self.deck)
                                if ALL_CARDS.get(c.id) and ALL_CARDS[c.id].card_type == CardType.SKILL]
                else:
                    eligible = [i for i, c in enumerate(self.deck)
                                if ALL_CARDS.get(c.id) and ALL_CARDS[c.id].card_type == CardType.POWER]
                if eligible:
                    chosen = self.deck[eligible[0]]
                    relic.card_id = f"{chosen.id}+" if chosen.upgraded else chosen.id

        # Astrolabe: transform and upgrade up to 3 purgeable cards
        if relic_id == "Astrolabe":
            unpurgeable = {"AscendersBane", "CurseOfTheBell", "Necronomicurse"}
            transformable = [
                (idx, card)
                for idx, card in self.get_removable_cards()
                if card.id not in unpurgeable
            ]
            if transformable:
                used_misc = self._get_rng("misc", misc_rng)
                transformable_indices = [idx for idx, _ in transformable]
                max_count = min(3, len(transformable_indices))

                if selection_card_indices is not None:
                    selected: List[int] = []
                    allowed = set(transformable_indices)
                    for idx in selection_card_indices:
                        if idx in allowed and idx not in selected:
                            selected.append(idx)
                        if len(selected) == max_count:
                            break
                    indices = selected if len(selected) == max_count else transformable_indices[:max_count]
                else:
                    indices = transformable_indices[:max_count]

                # Remove highest indices first to avoid shifting
                for idx in sorted(indices, reverse=True):
                    self.remove_card(idx)
                    new_card_id = self._roll_random_reward_card(used_misc)
                    if new_card_id:
                        self.add_card(new_card_id, upgraded=True)

        # Empty Cage: remove 2 cards
        if relic_id == "Empty Cage":
            removable = self.get_removable_cards()
            if removable:
                removable_indices = [idx for idx, _ in removable]
                max_count = min(2, len(removable_indices))
                if selection_card_indices is not None:
                    selected: List[int] = []
                    allowed = set(removable_indices)
                    for idx in selection_card_indices:
                        if idx in allowed and idx not in selected:
                            selected.append(idx)
                        if len(selected) == max_count:
                            break
                    indices = selected if len(selected) == max_count else removable_indices[:max_count]
                else:
                    indices = removable_indices[:max_count]
                for idx in sorted(indices, reverse=True):
                    self.remove_card(idx)

        # Calling Bell: gain 3 relics (C/U/R) + Curse of the Bell
        if relic_id == "Calling Bell":
            used_relic = self._get_rng("relic", relic_rng)
            reward_state = RewardState(owned_relics=set(self.get_relic_ids()))
            for tier in (RelicTier.COMMON, RelicTier.UNCOMMON, RelicTier.RARE):
                relic_reward = generate_relic_reward(
                    used_relic, tier, reward_state, self.character, self.act
                )
                if relic_reward:
                    self.add_relic(
                        relic_reward.id,
                        relic_rng=used_relic,
                        card_rng=card_rng,
                        misc_rng=misc_rng,
                        potion_rng=potion_rng,
                    )
            self.add_card("CurseOfTheBell")

        # Tiny House: gold, max HP, potion, card, and upgrade
        if relic_id == "Tiny House":
            self.add_gold(50)
            self.gain_max_hp(5)
            self.heal(5)

            used_potion = self._get_rng("potion", potion_rng)
            potion = generate_potion_reward(used_potion, self.character)
            if potion:
                self.add_potion(potion.id)

            used_card = self._get_rng("card", card_rng)
            cards = generate_card_rewards(
                used_card,
                RewardState(),
                act=self.act,
                player_class=self.character,
                ascension=self.ascension,
                room_type="normal",
                num_cards=1,
            )
            if cards:
                card = cards[0]
                self.add_card(card.id, upgraded=card.upgraded)

            upgradeable = self.get_upgradeable_cards()
            if upgradeable:
                self.upgrade_card(upgradeable[0][0])

        # Cauldron: gain 5 random potions
        if relic_id == "Cauldron":
            used_potion = self._get_rng("potion", potion_rng)
            for _ in range(5):
                potion = generate_potion_reward(used_potion, self.character)
                if potion is None:
                    continue
                if self.count_empty_potion_slots() == 0:
                    break
                self.add_potion(potion.id)

        # Dolly's Mirror: duplicate a card in deck
        if relic_id == "DollysMirror":
            if self.deck:
                chosen = self.deck[0]
                self.add_card(chosen.id, upgraded=chosen.upgraded, misc_value=chosen.misc_value)

        # Orrery: add 5 cards (auto-pick first from each reward)
        if relic_id == "Orrery":
            used_card = self._get_rng("card", card_rng)
            for _ in range(5):
                cards = generate_card_rewards(
                    used_card,
                    RewardState(),
                    act=self.act,
                    player_class=self.character,
                    ascension=self.ascension,
                    room_type="normal",
                    num_cards=3,
                )
                if cards:
                    card = cards[0]
                    self.add_card(card.id, upgraded=card.upgraded)

        # Pandora's Box: transform all Strikes and Defends
        if relic_id == "Pandora's Box":
            strike_defend_indices = []
            for i, card in enumerate(self.deck):
                if card.id.startswith("Strike_") or card.id.startswith("Defend_"):
                    strike_defend_indices.append(i)
            if strike_defend_indices:
                used_card = self._get_rng("card", card_rng)
                for idx in sorted(strike_defend_indices, reverse=True):
                    self.remove_card(idx)
                    new_card_id = self._roll_random_reward_card(used_card)
                    if new_card_id:
                        self.add_card(new_card_id, upgraded=False)

        # Old Coin: gain 300 gold (respects Ectoplasm)
        if relic_id == "Old Coin":
            self.add_gold(300)

        # Lee's Waffle: +7 max HP, heal to full
        if relic_id == "Lee's Waffle":
            self.gain_max_hp(7)
            self.current_hp = self.max_hp

        # Sync RNG counters if we used any streams locally
        if used_misc is not None:
            self._sync_rng_counter("misc", used_misc)
        if used_card is not None:
            self._sync_rng_counter("card", used_card)
        if used_relic is not None:
            self._sync_rng_counter("relic", used_relic)
        if used_potion is not None:
            self._sync_rng_counter("potion", used_potion)

    def has_relic(self, relic_id: str) -> bool:
        """Check if we have a specific relic."""
        key = self._relic_lookup_key(relic_id)
        return any(self._relic_lookup_key(r.id) == key for r in self.relics)

    def get_relic(self, relic_id: str) -> Optional[RelicInstance]:
        """Get a relic by ID if we have it."""
        key = self._relic_lookup_key(relic_id)
        for relic in self.relics:
            if self._relic_lookup_key(relic.id) == key:
                return relic
        return None

    def get_relic_ids(self) -> List[str]:
        """Get list of relic IDs."""
        return [r.id for r in self.relics]

    def get_starter_relic(self) -> Optional[str]:
        """
        Get the starter relic ID for this character.

        Returns:
            The starter relic ID if still owned, or None if removed/swapped
        """
        # Starter relics by character
        starter_relics = {
            "Watcher": "PureWater",
            "Ironclad": "BurningBlood",
            "Silent": "RingOfTheSnake",
            "Defect": "CrackedCore",
        }
        starter = starter_relics.get(self.character)
        if starter and self.has_relic(starter):
            return starter
        return None

    def remove_relic(self, relic_id: str) -> bool:
        """
        Remove a relic by ID.

        Args:
            relic_id: ID of relic to remove

        Returns:
            True if removed, False if not found
        """
        key = self._relic_lookup_key(relic_id)
        for i, relic in enumerate(self.relics):
            if self._relic_lookup_key(relic.id) == key:
                self.relics.pop(i)
                return True
        return False

    def get_relic_counter(self, relic_id: str) -> int:
        """Get counter value for a relic (-1 if not found or no counter)."""
        relic = self.get_relic(relic_id)
        return relic.counter if relic else -1

    def set_relic_counter(self, relic_id: str, value: int) -> bool:
        """Set counter value for a relic."""
        relic = self.get_relic(relic_id)
        if relic:
            relic.counter = value
            return True
        return False

    def increment_relic_counter(self, relic_id: str, amount: int = 1) -> int:
        """Increment counter for a relic, return new value."""
        relic = self.get_relic(relic_id)
        if relic and relic.counter >= 0:
            relic.counter += amount
            return relic.counter
        return -1

    # ----- POTION MANAGEMENT -----

    def add_potion(self, potion_id: str) -> bool:
        """
        Add a potion to first empty slot.

        Returns:
            True if added, False if no empty slots
        """
        for slot in self.potion_slots:
            if slot.is_empty():
                slot.potion_id = potion_id
                return True
        return False

    def use_potion(self, slot_idx: int) -> Optional[str]:
        """
        Use/discard potion from slot.

        Returns:
            Potion ID that was used, or None if slot empty/invalid
        """
        if 0 <= slot_idx < len(self.potion_slots):
            potion_id = self.potion_slots[slot_idx].potion_id
            self.potion_slots[slot_idx].potion_id = None
            return potion_id
        return None

    def has_potion(self, potion_id: str) -> bool:
        """Check if we have a specific potion."""
        return any(s.potion_id == potion_id for s in self.potion_slots)

    def get_potions(self) -> List[str]:
        """Get list of current potions (non-empty slots)."""
        return [s.potion_id for s in self.potion_slots if not s.is_empty()]

    def count_empty_potion_slots(self) -> int:
        """Count empty potion slots."""
        return sum(1 for s in self.potion_slots if s.is_empty())

    def count_potions(self) -> int:
        """Count potions held."""
        return sum(1 for s in self.potion_slots if not s.is_empty())

    # ----- RESOURCE MANAGEMENT -----

    def add_gold(self, amount: int):
        """Add gold (affected by Ectoplasm and Golden Idol)."""
        if self.has_relic("Ectoplasm"):
            if amount > 0:
                self.gold_blocked += amount
            return  # Can't gain gold

        # Golden Idol increases gold gain by 25%
        if self.has_relic("Golden Idol") and amount > 0:
            amount = int(amount * 1.25)

        self.gold += amount

        # Bloody Idol heals on gold gain
        if self.has_relic("Bloody Idol") and amount > 0:
            self.heal(5)

    def lose_gold(self, amount: int) -> int:
        """
        Lose gold (can't go below 0).

        Returns:
            Actual amount lost
        """
        actual = min(self.gold, amount)
        self.gold -= actual
        return actual

    def spend_gold(self, amount: int) -> int:
        """
        Spend gold (shop purchases) and trigger onSpendGold relic effects.

        Returns:
            Actual amount spent
        """
        spent = self.lose_gold(amount)
        if spent > 0:
            # Maw Bank deactivates after spending gold
            if self.has_relic("MawBank"):
                relic = self.get_relic("MawBank")
                if relic:
                    relic.counter = -2
        return spent

    def set_gold(self, amount: int):
        """Set gold directly (for events that set specific amounts)."""
        self.gold = max(0, amount)

    def heal(self, amount: int):
        """
        Heal HP.

        Args:
            amount: Amount to heal (affected by relics)
        """
        if self.has_relic("Mark of the Bloom"):
            return  # Can't heal

        # Magic Flower increases healing by 50%
        if self.has_relic("Magic Flower"):
            amount = int(amount * 1.5)

        self.current_hp = min(self.current_hp + amount, self.max_hp)

    def damage(self, amount: int):
        """Take damage (outside of combat)."""
        self.current_hp = max(0, self.current_hp - amount)

    def lose_max_hp(self, amount: int):
        """
        Lose max HP (current HP capped to new max).

        Args:
            amount: Amount of max HP to lose
        """
        self.max_hp = max(1, self.max_hp - amount)
        self.current_hp = min(self.current_hp, self.max_hp)

    def gain_max_hp(self, amount: int):
        """
        Gain max HP.

        Args:
            amount: Amount of max HP to gain
        """
        self.max_hp += amount

        # Optional: also heal to new max (some effects do this)
        # self.current_hp = self.max_hp

    def heal_to_full(self):
        """Heal to full HP (respects Mark of the Bloom)."""
        self.heal(self.max_hp - self.current_hp)

    def hp_percent(self) -> float:
        """Get current HP as percentage of max."""
        return self.current_hp / self.max_hp if self.max_hp > 0 else 0

    # ----- MAP MANAGEMENT -----

    def get_current_map(self) -> Optional[List[List[MapRoomNode]]]:
        """Get the map for the current act."""
        return self.act_maps.get(self.act)

    def generate_map_for_act(self, act_num: int) -> List[List[MapRoomNode]]:
        """
        Generate (or return cached) map for an act.

        Args:
            act_num: Act number (1-4)

        Returns:
            2D list of MapRoomNodes
        """
        if act_num in self.act_maps:
            return self.act_maps[act_num]

        # Generate map using seed
        config = MapGeneratorConfig(ascension_level=self.ascension)
        map_seed = self.seed + get_map_seed_offset(act_num)
        map_rng = Random(map_seed)
        generator = MapGenerator(map_rng, config)

        self.act_maps[act_num] = generator.generate()
        return self.act_maps[act_num]

    def get_available_paths(self) -> List[MapRoomNode]:
        """
        Get available nodes to travel to from current position.

        Returns:
            List of reachable MapRoomNodes
        """
        current_map = self.get_current_map()
        if not current_map:
            return []

        # At start of act - can travel to any node in row 0 with edges
        if self.map_position.is_at_start():
            return [node for node in current_map[0] if node.has_edges()]

        # Get current node and return connected nodes
        current_node = current_map[self.map_position.y][self.map_position.x]
        next_nodes = []

        for edge in current_node.edges:
            if edge.is_boss:
                # Boss node is special - create a synthetic boss node
                boss_node = MapRoomNode(x=3, y=current_node.y + 2, room_type=RoomType.BOSS)
                next_nodes.append(boss_node)
            else:
                next_nodes.append(current_map[edge.dst_y][edge.dst_x])

        return next_nodes

    def move_to(self, x: int, y: int):
        """Move to a position on the map."""
        self.map_position.x = x
        self.map_position.y = y
        self.visited_nodes.append((self.act, x, y))

    def advance_act(self):
        """Move to the next act.

        Java: AbstractDungeon.java:2562-2566
        Between-act healing:
        - A0-A4: Full heal
        - A5+: Heal 75% of missing HP (rounded)
        """
        # Between-act heal (Java: AbstractDungeon.java:2562-2566)
        if self.ascension >= 5:
            missing = self.max_hp - self.current_hp
            heal_amount = round(missing * 0.75)
            self.current_hp = min(self.current_hp + heal_amount, self.max_hp)
        else:
            self.current_hp = self.max_hp

        self.act += 1
        self.floor = 0
        self.map_position = MapPosition()
        self.elites_killed_this_act = 0
        self.cards_obtained_this_act.clear()

        # Generate map for new act
        if self.act <= 4:
            self.generate_map_for_act(self.act)

    def advance_floor(self):
        """Increment floor counter."""
        self.floor += 1
        self.floors_climbed += 1

    # ----- RNG COUNTER HELPERS -----

    def sync_rng_counters(self, counters: Dict[str, int]) -> None:
        """Replace RNG counters with the latest values."""
        self.rng_counters = {k: int(v) for k, v in counters.items()}

    def get_rng_counter(self, key: str, default: int = 0) -> int:
        """Get a stored RNG counter value."""
        return int(self.rng_counters.get(key, default))

    # ----- KEY MANAGEMENT -----

    def has_all_keys(self) -> bool:
        """Check if we have all three keys for Act 4."""
        return self.has_ruby_key and self.has_emerald_key and self.has_sapphire_key

    def obtain_ruby_key(self):
        """Obtain the ruby key (from rest site)."""
        self.has_ruby_key = True

    def obtain_emerald_key(self):
        """Obtain the emerald key (from burning elite)."""
        self.has_emerald_key = True

    def obtain_sapphire_key(self):
        """Obtain the sapphire key (from chest)."""
        self.has_sapphire_key = True

    # ----- BLIZZARD/PITY MANAGEMENT -----

    def on_card_reward_taken(self, is_rare: bool):
        """Update card blizzard counter after taking a card."""
        if is_rare:
            self.card_blizzard = 5
        else:
            self.card_blizzard = max(0, self.card_blizzard - 1)

    def on_potion_drop_check(self, got_potion: bool):
        """Update potion blizzard counter after drop check."""
        if got_potion:
            self.potion_blizzard = 0
        else:
            self.potion_blizzard += 10

    def get_rare_card_chance(self) -> float:
        """Get current rare card chance (affected by blizzard)."""
        # Base: 3% rare, increases by blizzard modifier
        base_rare = 0.03
        blizzard_bonus = (5 - self.card_blizzard) * 0.01
        return min(base_rare + blizzard_bonus, 1.0)

    def get_potion_drop_chance(self) -> float:
        """Get current potion drop chance (affected by blizzard)."""
        # Base: 40% chance
        base_chance = 0.40
        return min(base_chance + (self.potion_blizzard / 100), 1.0)

    # ----- STATE MANAGEMENT -----

    def copy(self) -> 'RunState':
        """Create a deep copy of the run state (for simulation)."""
        return deepcopy(self)

    def to_dict(self) -> dict:
        """Serialize to dictionary (for saving)."""
        return {
            "seed": self.seed,
            "seed_string": self.seed_string,
            "act": self.act,
            "floor": self.floor,
            "ascension": self.ascension,
            "character": self.character,
            "current_hp": self.current_hp,
            "max_hp": self.max_hp,
            "gold": self.gold,
            "gold_blocked": self.gold_blocked,
            "deck": [{"id": c.id, "upgraded": c.upgraded, "misc": c.misc_value} for c in self.deck],
            "relics": [
                {"id": r.id, "counter": r.counter, "card_id": r.card_id}
                for r in self.relics
            ],
            "potions": [s.potion_id for s in self.potion_slots],
            "map_position": {"x": self.map_position.x, "y": self.map_position.y},
            "visited_nodes": self.visited_nodes,
            "seen_cards": list(self.seen_cards),
            "seen_relics": list(self.seen_relics),
            "has_ruby_key": self.has_ruby_key,
            "has_emerald_key": self.has_emerald_key,
            "has_sapphire_key": self.has_sapphire_key,
            "rng_counters": self.rng_counters,
            "card_blizzard": self.card_blizzard,
            "potion_blizzard": self.potion_blizzard,
            "elites_killed_this_act": self.elites_killed_this_act,
            "floors_climbed": self.floors_climbed,
            "combats_won": self.combats_won,
            "elites_killed": self.elites_killed,
            "bosses_killed": self.bosses_killed,
        }

    @classmethod
    def from_dict(cls, data: dict) -> 'RunState':
        """Deserialize from dictionary (for loading)."""
        state = cls(
            seed=data["seed"],
            seed_string=data["seed_string"],
            act=data.get("act", 1),
            floor=data.get("floor", 0),
            ascension=data.get("ascension", 20),
            character=data.get("character", "Watcher"),
            current_hp=data.get("current_hp", 72),
            max_hp=data.get("max_hp", 72),
            gold=data.get("gold", 99),
            gold_blocked=data.get("gold_blocked", 0),
        )

        # Restore deck
        for card_data in data.get("deck", []):
            state.deck.append(CardInstance(
                id=card_data["id"],
                upgraded=card_data.get("upgraded", False),
                misc_value=card_data.get("misc", 0)
            ))

        # Restore relics
        for relic_data in data.get("relics", []):
            state.relics.append(RelicInstance(
                id=relic_data["id"],
                counter=relic_data.get("counter", -1),
                card_id=relic_data.get("card_id")
            ))

        # Restore potions
        potion_count = len(data.get("potions", []))
        state.potion_slots = [PotionSlot(potion_id=p) for p in data.get("potions", [])]

        # Restore map position
        pos_data = data.get("map_position", {"x": -1, "y": -1})
        state.map_position = MapPosition(x=pos_data["x"], y=pos_data["y"])

        # Restore visited nodes
        state.visited_nodes = data.get("visited_nodes", [])

        # Restore sets
        state.seen_cards = set(data.get("seen_cards", []))
        state.seen_relics = set(data.get("seen_relics", []))

        # Restore keys
        state.has_ruby_key = data.get("has_ruby_key", False)
        state.has_emerald_key = data.get("has_emerald_key", False)
        state.has_sapphire_key = data.get("has_sapphire_key", False)

        # Restore RNG counters
        state.rng_counters = data.get("rng_counters", {})

        # Restore blizzard
        state.card_blizzard = data.get("card_blizzard", 5)
        state.potion_blizzard = data.get("potion_blizzard", 0)

        # Restore stats
        state.elites_killed_this_act = data.get("elites_killed_this_act", 0)
        state.floors_climbed = data.get("floors_climbed", 0)
        state.combats_won = data.get("combats_won", 0)
        state.elites_killed = data.get("elites_killed", 0)
        state.bosses_killed = data.get("bosses_killed", 0)

        return state

    def __repr__(self) -> str:
        return (
            f"RunState(seed={self.seed_string}, "
            f"A{self.ascension} {self.character}, "
            f"Act{self.act} F{self.floor}, "
            f"HP:{self.current_hp}/{self.max_hp}, "
            f"Gold:{self.gold}, "
            f"Deck:{len(self.deck)} cards, "
            f"Relics:{len(self.relics)})"
        )


# ==================== STARTING STATE ====================

BASE_STARTING_GOLD = 99

# Watcher starting deck
WATCHER_STARTING_DECK = [
    ("Strike_P", False),  # 4 Strikes
    ("Strike_P", False),
    ("Strike_P", False),
    ("Strike_P", False),
    ("Defend_P", False),  # 4 Defends
    ("Defend_P", False),
    ("Defend_P", False),
    ("Defend_P", False),
    ("Eruption", False),  # Starter attack (1 cost, 9 damage, enter Wrath)
    ("Vigilance", False), # Starter skill (2 cost, 8 block, enter Calm)
]

# Watcher starting relic
WATCHER_STARTING_RELIC = "PureWater"  # At start of combat, add Miracle to hand

# Ironclad starting deck
IRONCLAD_STARTING_DECK = [
    ("Strike_R", False),  # 5 Strikes
    ("Strike_R", False),
    ("Strike_R", False),
    ("Strike_R", False),
    ("Strike_R", False),
    ("Defend_R", False),  # 4 Defends
    ("Defend_R", False),
    ("Defend_R", False),
    ("Defend_R", False),
    ("Bash", False),      # Starter attack
]

# Ironclad starting relic
IRONCLAD_STARTING_RELIC = "Burning Blood"

# Silent starting deck
SILENT_STARTING_DECK = [
    ("Strike_G", False),  # 5 Strikes
    ("Strike_G", False),
    ("Strike_G", False),
    ("Strike_G", False),
    ("Strike_G", False),
    ("Defend_G", False),  # 5 Defends
    ("Defend_G", False),
    ("Defend_G", False),
    ("Defend_G", False),
    ("Defend_G", False),
    ("Neutralize", False),
    ("Survivor", False),
]

# Silent starting relic
SILENT_STARTING_RELIC = "Ring of the Snake"

# Defect starting deck
DEFECT_STARTING_DECK = [
    ("Strike_B", False),  # 4 Strikes
    ("Strike_B", False),
    ("Strike_B", False),
    ("Strike_B", False),
    ("Defend_B", False),  # 4 Defends
    ("Defend_B", False),
    ("Defend_B", False),
    ("Defend_B", False),
    ("Zap", False),
    ("Dualcast", False),
]

# Defect starting relic
DEFECT_STARTING_RELIC = "Cracked Core"

# Base stats
WATCHER_BASE_HP = 72
IRONCLAD_BASE_HP = 80
SILENT_BASE_HP = 70
DEFECT_BASE_HP = 75


def _create_character_run(
    seed: str,
    ascension: int,
    character: str,
    base_hp: int,
    starting_deck: List[Tuple[str, bool]],
    starting_relic: str,
) -> RunState:
    """Create a new run with character-specific starting state."""
    # Parse seed
    if isinstance(seed, str):
        seed_string = seed.upper()
        seed_long = seed_to_long(seed_string)
    else:
        seed_long = seed
        seed_string = long_to_seed(seed_long)

    # Calculate starting HP based on ascension
    if ascension >= 14:
        # A14+: -4 max HP
        max_hp = base_hp - 4
    else:
        max_hp = base_hp

    # A6+: start at 90% current HP (applied after max HP reduction)
    if ascension >= 6:
        current_hp = round(max_hp * 0.9)
    else:
        current_hp = max_hp

    # Starting gold is always base (99) -- A15 only affects events, not starting gold
    gold = BASE_STARTING_GOLD

    # Calculate potion slots based on ascension
    if ascension >= 11:
        # A11+: only 2 potion slots
        potion_slot_count = 2
    else:
        # A0-10: 3 potion slots
        potion_slot_count = 3

    # Create run state
    state = RunState(
        seed=seed_long,
        seed_string=seed_string,
        ascension=ascension,
        character=character,
        current_hp=current_hp,
        max_hp=max_hp,
        gold=gold,
    )

    # Initialize potion slots
    state.potion_slots = [PotionSlot() for _ in range(potion_slot_count)]

    # Add starting deck
    for card_id, upgraded in starting_deck:
        state.deck.append(CardInstance(id=card_id, upgraded=upgraded))

    # Add Ascender's Bane at A10+
    if ascension >= 10:
        state.deck.append(CardInstance(id="AscendersBane", upgraded=False))

    # Add starting relic
    state.add_relic(starting_relic)

    # Generate Act 1 map
    state.generate_map_for_act(1)

    return state


def create_watcher_run(seed: str, ascension: int = 20) -> RunState:
    """
    Create a new Watcher run with starting deck/relic.

    Args:
        seed: Seed string (e.g., "ABC123") or numeric seed
        ascension: Ascension level (0-20)

    Returns:
        RunState initialized for a new Watcher run
    """
    return _create_character_run(
        seed=seed,
        ascension=ascension,
        character="Watcher",
        base_hp=WATCHER_BASE_HP,
        starting_deck=WATCHER_STARTING_DECK,
        starting_relic=WATCHER_STARTING_RELIC,
    )


def create_ironclad_run(seed: str, ascension: int = 20) -> RunState:
    """Create a new Ironclad run with starting deck/relic."""
    return _create_character_run(
        seed=seed,
        ascension=ascension,
        character="Ironclad",
        base_hp=IRONCLAD_BASE_HP,
        starting_deck=IRONCLAD_STARTING_DECK,
        starting_relic=IRONCLAD_STARTING_RELIC,
    )


def create_silent_run(seed: str, ascension: int = 20) -> RunState:
    """Create a new Silent run with starting deck/relic."""
    return _create_character_run(
        seed=seed,
        ascension=ascension,
        character="Silent",
        base_hp=SILENT_BASE_HP,
        starting_deck=SILENT_STARTING_DECK,
        starting_relic=SILENT_STARTING_RELIC,
    )


def create_defect_run(seed: str, ascension: int = 20) -> RunState:
    """Create a new Defect run with starting deck/relic."""
    return _create_character_run(
        seed=seed,
        ascension=ascension,
        character="Defect",
        base_hp=DEFECT_BASE_HP,
        starting_deck=DEFECT_STARTING_DECK,
        starting_relic=DEFECT_STARTING_RELIC,
    )


def create_run_from_save(save_data: dict) -> RunState:
    """
    Create a RunState from game save data.

    This is used when loading a save file from the actual game.
    """
    return RunState.from_dict(save_data)


# ==================== TESTING ====================

if __name__ == "__main__":
    # Test creating a new Watcher run
    print("=== Testing RunState ===\n")

    run = create_watcher_run("ABC123", ascension=20)
    print(f"Created run: {run}")
    print(f"\nStarting deck ({len(run.deck)} cards):")
    for i, card in enumerate(run.deck):
        print(f"  {i}: {card}")

    print(f"\nStarting relics:")
    for relic in run.relics:
        print(f"  {relic}")

    print(f"\nPotion slots: {len(run.potion_slots)}")
    print(f"Starting gold: {run.gold}")
    print(f"Starting HP: {run.current_hp}/{run.max_hp}")

    # Test deck operations
    print("\n=== Testing Deck Operations ===")
    run.add_card("Tantrum", upgraded=True)
    print(f"After adding Tantrum+: {len(run.deck)} cards")

    run.upgrade_card(0)  # Upgrade first Strike
    print(f"Upgraded Strike: {run.deck[0]}")

    removed = run.remove_card_by_id("Strike_P")
    print(f"Removed: {removed}, deck now has {len(run.deck)} cards")

    # Test relic operations
    print("\n=== Testing Relic Operations ===")
    run.add_relic("PenNib", counter=0)
    print(f"Added Pen Nib: {run.get_relic('PenNib')}")

    run.increment_relic_counter("PenNib", 5)
    print(f"After incrementing: {run.get_relic('PenNib')}")

    # Test potion operations
    print("\n=== Testing Potion Operations ===")
    run.add_potion("FirePotion")
    print(f"Added Fire Potion: {run.get_potions()}")

    run.add_potion("BlockPotion")
    print(f"Added Block Potion: {run.get_potions()}")

    used = run.use_potion(0)
    print(f"Used: {used}, remaining: {run.get_potions()}")

    # Test resource operations
    print("\n=== Testing Resources ===")
    run.damage(20)
    print(f"After 20 damage: {run.current_hp}/{run.max_hp}")

    run.heal(10)
    print(f"After 10 heal: {run.current_hp}/{run.max_hp}")

    run.add_gold(50)
    print(f"After +50 gold: {run.gold}")

    run.lose_gold(30)
    print(f"After -30 gold: {run.gold}")

    # Test serialization
    print("\n=== Testing Serialization ===")
    save_dict = run.to_dict()
    loaded_run = RunState.from_dict(save_dict)
    print(f"Loaded run matches: {loaded_run.seed == run.seed}")
    print(f"Deck preserved: {len(loaded_run.deck) == len(run.deck)}")

    # Test map
    print("\n=== Testing Map ===")
    current_map = run.get_current_map()
    print(f"Act 1 map generated: {current_map is not None}")

    available = run.get_available_paths()
    print(f"Available starting paths: {len(available)}")
    for node in available:
        print(f"  Floor 0, Column {node.x}: {node.room_type}")

    print("\n=== All tests passed ===")
