"""
Combat State Machine for Slay the Spire RL.

Optimized for:
1. Fast copying (for tree search)
2. Minimal memory footprint
3. Easy serialization
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Union


# =============================================================================
# Action Types
# =============================================================================


@dataclass(frozen=True)
class PlayCard:
    """Play a card from hand."""

    card_idx: int  # Index in hand
    target_idx: int = -1  # Enemy index, -1 for self/no target


@dataclass(frozen=True)
class UsePotion:
    """Use a potion."""

    potion_idx: int
    target_idx: int = -1


@dataclass(frozen=True)
class EndTurn:
    """End the current turn."""

    pass


@dataclass(frozen=True)
class SelectScryDiscard:
    """Select which scried cards to discard (indices into pending_scry_cards)."""

    discard_indices: tuple  # Tuple of indices to discard, e.g. (0, 2) to discard first and third


Action = Union[PlayCard, UsePotion, EndTurn, SelectScryDiscard]


# =============================================================================
# Entity States
# =============================================================================


@dataclass
class EntityState:
    """Minimal state for player or enemy."""

    hp: int
    max_hp: int
    block: int = 0
    # All statuses as simple dict - no objects
    statuses: Dict[str, int] = field(default_factory=dict)

    # -------------------------------------------------------------------------
    # Convenience accessors for common statuses
    # -------------------------------------------------------------------------

    @property
    def strength(self) -> int:
        return self.statuses.get("Strength", 0)

    @property
    def dexterity(self) -> int:
        return self.statuses.get("Dexterity", 0)

    @property
    def is_weak(self) -> bool:
        return self.statuses.get("Weak", 0) > 0

    @property
    def is_vulnerable(self) -> bool:
        return self.statuses.get("Vulnerable", 0) > 0

    @property
    def is_frail(self) -> bool:
        return self.statuses.get("Frail", 0) > 0

    @property
    def is_dead(self) -> bool:
        return self.hp <= 0

    @property
    def artifact(self) -> int:
        return self.statuses.get("Artifact", 0)

    @property
    def plated_armor(self) -> int:
        return self.statuses.get("Plated Armor", 0)

    @property
    def metallicize(self) -> int:
        return self.statuses.get("Metallicize", 0)

    @property
    def thorns(self) -> int:
        return self.statuses.get("Thorns", 0)

    @property
    def poison(self) -> int:
        return self.statuses.get("Poison", 0)

    def copy(self) -> EntityState:
        """Create a shallow copy with copied statuses dict."""
        return EntityState(
            hp=self.hp,
            max_hp=self.max_hp,
            block=self.block,
            statuses=self.statuses.copy(),
        )


@dataclass
class EnemyCombatState(EntityState):
    """Enemy state in combat."""

    id: str = ""
    name: str = ""  # Display name
    enemy_type: str = ""  # Enemy type for categorization (e.g., "NORMAL", "ELITE", "BOSS")
    move_id: int = -1  # Current intended move
    move_damage: int = 0
    move_hits: int = 1
    move_block: int = 0
    move_effects: Dict[str, int] = field(default_factory=dict)
    move_history: List[int] = field(default_factory=list)  # History of move IDs for AI patterns
    first_turn: bool = True  # Whether this is the enemy's first turn

    def copy(self) -> EnemyCombatState:
        """Create a shallow copy with copied dicts."""
        return EnemyCombatState(
            hp=self.hp,
            max_hp=self.max_hp,
            block=self.block,
            statuses=self.statuses.copy(),
            id=self.id,
            name=self.name,
            enemy_type=self.enemy_type,
            move_id=self.move_id,
            move_damage=self.move_damage,
            move_hits=self.move_hits,
            move_block=self.move_block,
            move_effects=self.move_effects.copy(),
            move_history=self.move_history.copy(),
            first_turn=self.first_turn,
        )

    @property
    def is_attacking(self) -> bool:
        """Returns True if the enemy's current move deals damage."""
        return self.move_damage > 0

    @property
    def total_incoming_damage(self) -> int:
        """Total damage this enemy will deal with current move."""
        return self.move_damage * self.move_hits

    @property
    def strength(self) -> int:
        """Get enemy strength from statuses."""
        return self.statuses.get("Strength", 0)

    def is_alive(self) -> bool:
        """Check if enemy is alive."""
        return self.hp > 0


# =============================================================================
# Combat State
# =============================================================================


@dataclass
class CombatState:
    """
    Complete combat state - everything needed to continue simulation.

    Designed for fast copying and minimal memory footprint.
    Card piles are stored as lists of card IDs (strings) for speed.
    """

    # Player state
    player: EntityState
    energy: int
    max_energy: int
    stance: str = "Neutral"

    # Card piles (as lists of card IDs for speed)
    # Note: Upgrade status can be encoded in card ID (e.g., "Strike+" or "Strike_P+")
    hand: List[str] = field(default_factory=list)
    draw_pile: List[str] = field(default_factory=list)
    discard_pile: List[str] = field(default_factory=list)
    exhaust_pile: List[str] = field(default_factory=list)

    # Enemies
    enemies: List[EnemyCombatState] = field(default_factory=list)

    # Potions (list of potion IDs, empty string = empty slot)
    potions: List[str] = field(default_factory=list)

    # Combat tracking
    turn: int = 0
    cards_played_this_turn: int = 0
    attacks_played_this_turn: int = 0
    skills_played_this_turn: int = 0
    powers_played_this_turn: int = 0
    combat_over: bool = False
    player_won: bool = False

    # Watcher-specific tracking
    mantra: int = 0
    last_card_type: str = ""  # "ATTACK", "SKILL", "POWER", or ""

    # Scry pending state (for agent selection)
    pending_scry_cards: List[str] = field(default_factory=list)
    pending_scry_selection: bool = False

    # RNG state for deterministic simulation (seed0, seed1)
    shuffle_rng_state: tuple = (0, 0)
    card_rng_state: tuple = (0, 0)
    ai_rng_state: tuple = (0, 0)

    # Combat statistics
    total_damage_dealt: int = 0
    total_damage_taken: int = 0
    total_cards_played: int = 0

    # Relic counters (only combat-relevant state)
    relic_counters: Dict[str, int] = field(default_factory=dict)

    # Relics the player has (for checking effects)
    relics: List[str] = field(default_factory=list)

    # Card costs cache (card_id -> cost, for cards with modified costs)
    card_costs: Dict[str, int] = field(default_factory=dict)

    # Defect-specific: Orb manager (lazy initialized)
    orb_manager: Any = None

    # -------------------------------------------------------------------------
    # Core Methods
    # -------------------------------------------------------------------------

    def copy(self) -> CombatState:
        """
        Create a fast copy of the combat state.

        Uses shallow copies for lists and dicts since they contain
        immutable types (strings, ints) or are explicitly copied.
        """
        return CombatState(
            # Player - needs deep copy for nested statuses
            player=self.player.copy(),
            energy=self.energy,
            max_energy=self.max_energy,
            stance=self.stance,
            # Card piles - shallow copy is fine (strings are immutable)
            hand=self.hand.copy(),
            draw_pile=self.draw_pile.copy(),
            discard_pile=self.discard_pile.copy(),
            exhaust_pile=self.exhaust_pile.copy(),
            # Enemies - need individual copies
            enemies=[e.copy() for e in self.enemies],
            # Potions - shallow copy (strings)
            potions=self.potions.copy(),
            # Combat tracking
            turn=self.turn,
            cards_played_this_turn=self.cards_played_this_turn,
            attacks_played_this_turn=self.attacks_played_this_turn,
            skills_played_this_turn=self.skills_played_this_turn,
            powers_played_this_turn=self.powers_played_this_turn,
            combat_over=self.combat_over,
            player_won=self.player_won,
            # Watcher-specific
            mantra=self.mantra,
            last_card_type=self.last_card_type,
            # Scry pending state
            pending_scry_cards=self.pending_scry_cards.copy(),
            pending_scry_selection=self.pending_scry_selection,
            # RNG state
            shuffle_rng_state=self.shuffle_rng_state,
            card_rng_state=self.card_rng_state,
            ai_rng_state=self.ai_rng_state,
            # Combat statistics
            total_damage_dealt=self.total_damage_dealt,
            total_damage_taken=self.total_damage_taken,
            total_cards_played=self.total_cards_played,
            # Relic counters - shallow copy (string keys, int values)
            relic_counters=self.relic_counters.copy(),
            relics=self.relics.copy(),
            # Card costs cache
            card_costs=self.card_costs.copy(),
            # Defect orb manager
            orb_manager=self.orb_manager.copy() if self.orb_manager else None,
        )

    def is_victory(self) -> bool:
        """Check if all enemies are dead."""
        return all(e.is_dead for e in self.enemies)

    def is_defeat(self) -> bool:
        """Check if player is dead."""
        return self.player.is_dead

    def is_terminal(self) -> bool:
        """Check if combat has ended (victory or defeat)."""
        return self.is_victory() or self.is_defeat()

    def get_living_enemies(self) -> List[EnemyCombatState]:
        """Get list of living enemies (alias for living_enemies)."""
        return [e for e in self.enemies if not e.is_dead]

    def all_enemies_dead(self) -> bool:
        """Check if all enemies are dead."""
        return all(e.is_dead for e in self.enemies)

    # -------------------------------------------------------------------------
    # Action Generation
    # -------------------------------------------------------------------------

    def get_legal_actions(
        self, card_registry: Dict[str, dict] = None
    ) -> List[Action]:
        """
        Get all legal actions from the current state.

        Args:
            card_registry: Optional dict mapping card_id -> card_data with
                          'cost', 'target', 'type' fields. If None, assumes
                          all cards cost 1 energy and target enemies.

        Returns:
            List of legal Action objects.
        """
        actions: List[Action] = []

        # If scry selection is pending, only return scry discard options
        if self.pending_scry_selection and self.pending_scry_cards:
            return self._get_scry_actions()

        # Get living enemy indices
        living_enemies = [i for i, e in enumerate(self.enemies) if not e.is_dead]

        # Card plays
        for hand_idx, card_id in enumerate(self.hand):
            card_cost = self._get_card_cost(card_id, card_registry)

            if card_cost <= self.energy:
                target_type = self._get_card_target(card_id, card_registry)

                if target_type == "enemy":
                    # Add one action per living enemy
                    for enemy_idx in living_enemies:
                        actions.append(PlayCard(card_idx=hand_idx, target_idx=enemy_idx))
                elif target_type == "all_enemies":
                    # No target needed, affects all
                    actions.append(PlayCard(card_idx=hand_idx, target_idx=-1))
                else:
                    # Self-targeting or no target
                    actions.append(PlayCard(card_idx=hand_idx, target_idx=-1))

        # Potion uses
        for pot_idx, potion_id in enumerate(self.potions):
            if potion_id:  # Non-empty slot
                pot_target = self._get_potion_target(potion_id)
                if pot_target == "enemy":
                    for enemy_idx in living_enemies:
                        actions.append(UsePotion(potion_idx=pot_idx, target_idx=enemy_idx))
                else:
                    actions.append(UsePotion(potion_idx=pot_idx, target_idx=-1))

        # End turn is always legal
        actions.append(EndTurn())

        return actions

    def _get_card_cost(
        self, card_id: str, registry: Dict[str, dict]
    ) -> int:
        """Get the cost of a card, checking cache first."""
        # Check modified cost cache first
        if card_id in self.card_costs:
            return self.card_costs[card_id]
        # Check registry
        if registry and card_id in registry:
            return registry[card_id].get("cost", 1)
        # Default cost
        return 1

    def _get_card_target(
        self, card_id: str, registry: Dict[str, dict]
    ) -> str:
        """Get targeting type for a card."""
        if registry and card_id in registry:
            return registry[card_id].get("target", "enemy")
        return "enemy"

    def _get_potion_target(self, potion_id: str) -> str:
        """Get targeting type for a potion."""
        # Common enemy-targeting potions
        enemy_target_potions = {
            "Fire Potion",
            "Explosive Potion",
            "Poison Potion",
            "CunningPotion",
            "FearPotion",
            "Weak Potion",
        }
        if potion_id in enemy_target_potions:
            return "enemy"
        return "self"

    def _get_scry_actions(self) -> List[Action]:
        """
        Get all possible scry discard selections.

        For N scried cards, generates 2^N options (all subsets of cards to discard).
        """
        from itertools import combinations

        n = len(self.pending_scry_cards)
        actions: List[Action] = []

        # Generate all possible subsets of indices to discard
        # For each possible number of cards to discard (0 to n)
        for num_discard in range(n + 1):
            # For each combination of that many indices
            for indices in combinations(range(n), num_discard):
                actions.append(SelectScryDiscard(discard_indices=indices))

        return actions

    # -------------------------------------------------------------------------
    # Utility Methods
    # -------------------------------------------------------------------------

    def living_enemies(self) -> List[EnemyCombatState]:
        """Get list of living enemies."""
        return [e for e in self.enemies if not e.is_dead]

    def total_incoming_damage(self) -> int:
        """Calculate total damage from all enemy attacks this turn."""
        return sum(
            e.total_incoming_damage for e in self.enemies if not e.is_dead and e.is_attacking
        )

    def cards_in_deck(self) -> int:
        """Total cards across all piles."""
        return (
            len(self.hand)
            + len(self.draw_pile)
            + len(self.discard_pile)
            + len(self.exhaust_pile)
        )

    def has_relic(self, relic_id: str) -> bool:
        """Check if player has a specific relic."""
        if relic_id in self.relics:
            return True
        try:
            from ..content.relics import ALL_RELICS
        except Exception:
            return False
        canonical_id = None
        if relic_id in ALL_RELICS:
            canonical_id = relic_id
        else:
            for rid, relic in ALL_RELICS.items():
                if relic.name == relic_id:
                    canonical_id = rid
                    break
        if canonical_id is None:
            return False
        if canonical_id in self.relics:
            return True
        canonical_name = ALL_RELICS[canonical_id].name
        return canonical_name in self.relics

    def get_relic_counter(self, relic_id: str, default: int = 0) -> int:
        """Get a relic's counter value."""
        return self.relic_counters.get(relic_id, default)

    def set_relic_counter(self, relic_id: str, value: int) -> None:
        """Set a relic's counter value."""
        self.relic_counters[relic_id] = value

    def __hash__(self) -> int:
        """
        Hash for state comparison in tree search.

        Note: This is a simplified hash that may have collisions.
        For exact state comparison, use full equality check.
        """
        return hash(
            (
                self.player.hp,
                self.player.block,
                self.energy,
                self.turn,
                tuple(self.hand),
                tuple((e.hp, e.move_id) for e in self.enemies),
            )
        )

    def __eq__(self, other: object) -> bool:
        """Full equality check for state comparison."""
        if not isinstance(other, CombatState):
            return False
        return (
            self.player.hp == other.player.hp
            and self.player.max_hp == other.player.max_hp
            and self.player.block == other.player.block
            and self.player.statuses == other.player.statuses
            and self.energy == other.energy
            and self.max_energy == other.max_energy
            and self.stance == other.stance
            and self.hand == other.hand
            and self.draw_pile == other.draw_pile
            and self.discard_pile == other.discard_pile
            and self.exhaust_pile == other.exhaust_pile
            and self.turn == other.turn
            and len(self.enemies) == len(other.enemies)
            and all(
                e1.hp == e2.hp
                and e1.block == e2.block
                and e1.move_id == e2.move_id
                and e1.statuses == e2.statuses
                for e1, e2 in zip(self.enemies, other.enemies)
            )
        )


# =============================================================================
# Factory Functions
# =============================================================================


def create_player(hp: int, max_hp: int = None) -> EntityState:
    """Create a new player entity state."""
    return EntityState(hp=hp, max_hp=max_hp or hp)


def create_enemy(
    id: str,
    hp: int,
    max_hp: int = None,
    name: str = None,
    enemy_type: str = "NORMAL",
    move_id: int = -1,
    move_damage: int = 0,
    move_hits: int = 1,
    move_block: int = 0,
) -> EnemyCombatState:
    """Create a new enemy combat state."""
    return EnemyCombatState(
        hp=hp,
        max_hp=max_hp or hp,
        id=id,
        name=name or id,  # Use id as name if not provided
        enemy_type=enemy_type,
        move_id=move_id,
        move_damage=move_damage,
        move_hits=move_hits,
        move_block=move_block,
    )


def create_combat(
    player_hp: int,
    player_max_hp: int,
    enemies: List[EnemyCombatState],
    deck: List[str],
    energy: int = 3,
    max_energy: int = 3,
    relics: List[str] = None,
    potions: List[str] = None,
) -> CombatState:
    """
    Create a new combat state with initial setup.

    The deck is placed in the draw pile (should be shuffled before calling).
    """
    return CombatState(
        player=create_player(player_hp, player_max_hp),
        energy=energy,
        max_energy=max_energy,
        hand=[],
        draw_pile=deck.copy(),
        discard_pile=[],
        exhaust_pile=[],
        enemies=enemies,
        relics=relics or [],
        potions=potions or ["", "", ""],  # 3 empty potion slots by default
    )
