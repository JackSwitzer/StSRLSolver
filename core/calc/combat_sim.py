"""
Combat Simulator - Turn-by-turn combat simulation with optimal play search.

This module provides a combat simulator that:
1. Simulates combat turn-by-turn with exact game mechanics
2. Uses immutable state for tree search (no mutation)
3. Supports various policies (random, greedy, custom)
4. Tracks detailed combat results for analysis

Usage:
    sim = CombatSimulator(card_data, enemy_data)
    state = sim.setup_combat(deck, enemies, player_hp, ...)

    # Manual play
    actions = sim.get_legal_actions(state)
    new_state = sim.execute_action(state, actions[0])

    # Full simulation
    result = sim.simulate_full_combat(state, sim.greedy_policy)
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple, Callable, Any, Set
from enum import Enum
from copy import deepcopy
import math

# Import from core modules
from ..state.rng import Random
from ..content.cards import Card, CardType, CardTarget, get_card, ALL_CARDS
from ..content.enemies import Enemy, Intent, MoveInfo, EnemyState, EnemyType
from ..content.stances import StanceID, StanceManager, STANCES
# Use old damage module for DamageCombatState/Power classes (legacy compatibility)
from ..damage import calculate_card_damage, calculate_block, DamageType, CombatState as DamageCombatState, Power


# =============================================================================
# ACTION TYPES
# =============================================================================

class ActionType(Enum):
    """Types of actions the player can take."""
    PLAY_CARD = "PLAY_CARD"
    USE_POTION = "USE_POTION"
    END_TURN = "END_TURN"


@dataclass(frozen=True)
class Action:
    """
    An action the player can take.

    Immutable for use as dict keys and in sets.
    """
    action_type: ActionType
    card_index: int = -1  # Index in hand for PLAY_CARD
    target_index: int = 0  # Target enemy index
    potion_index: int = -1  # Potion slot for USE_POTION

    def __repr__(self) -> str:
        if self.action_type == ActionType.PLAY_CARD:
            return f"Play(card={self.card_index}, target={self.target_index})"
        elif self.action_type == ActionType.USE_POTION:
            return f"Potion({self.potion_index}, target={self.target_index})"
        else:
            return "EndTurn"


# =============================================================================
# COMBAT STATE (Immutable)
# =============================================================================

@dataclass
class EnemyCombatState:
    """Enemy state within combat (copyable)."""
    id: str
    name: str
    enemy_type: EnemyType
    current_hp: int
    max_hp: int
    block: int = 0
    strength: int = 0
    powers: Dict[str, int] = field(default_factory=dict)
    move_history: List[int] = field(default_factory=list)
    next_move: Optional[MoveInfo] = None
    first_turn: bool = True

    def copy(self) -> EnemyCombatState:
        return EnemyCombatState(
            id=self.id,
            name=self.name,
            enemy_type=self.enemy_type,
            current_hp=self.current_hp,
            max_hp=self.max_hp,
            block=self.block,
            strength=self.strength,
            powers=dict(self.powers),
            move_history=list(self.move_history),
            next_move=self.next_move,
            first_turn=self.first_turn,
        )

    def is_alive(self) -> bool:
        return self.current_hp > 0


@dataclass
class PlayerCombatState:
    """Player state within combat (copyable)."""
    current_hp: int
    max_hp: int
    energy: int = 3
    max_energy: int = 3
    block: int = 0

    # Card piles (stored as card IDs with upgrade status)
    draw_pile: List[Tuple[str, bool]] = field(default_factory=list)  # (card_id, upgraded)
    hand: List[Tuple[str, bool]] = field(default_factory=list)
    discard_pile: List[Tuple[str, bool]] = field(default_factory=list)
    exhaust_pile: List[Tuple[str, bool]] = field(default_factory=list)

    # Stance
    stance: StanceID = StanceID.NEUTRAL
    mantra: int = 0

    # Powers (id -> amount)
    powers: Dict[str, int] = field(default_factory=dict)

    # Combat tracking
    cards_played_this_turn: int = 0
    attacks_played_this_turn: int = 0
    last_card_type: Optional[CardType] = None

    def copy(self) -> PlayerCombatState:
        return PlayerCombatState(
            current_hp=self.current_hp,
            max_hp=self.max_hp,
            energy=self.energy,
            max_energy=self.max_energy,
            block=self.block,
            draw_pile=list(self.draw_pile),
            hand=list(self.hand),
            discard_pile=list(self.discard_pile),
            exhaust_pile=list(self.exhaust_pile),
            stance=self.stance,
            mantra=self.mantra,
            powers=dict(self.powers),
            cards_played_this_turn=self.cards_played_this_turn,
            attacks_played_this_turn=self.attacks_played_this_turn,
            last_card_type=self.last_card_type,
        )

    def has_power(self, power_id: str) -> bool:
        return power_id in self.powers and self.powers[power_id] > 0

    def get_power(self, power_id: str) -> int:
        return self.powers.get(power_id, 0)

    def is_alive(self) -> bool:
        return self.current_hp > 0


@dataclass
class SimCombatState:
    """
    Complete combat state for simulation.

    Designed to be copied cheaply for tree search.
    """
    player: PlayerCombatState
    enemies: List[EnemyCombatState]

    turn: int = 1
    combat_over: bool = False
    player_won: bool = False

    # RNG state (for deterministic simulation)
    shuffle_rng_state: Tuple[int, int] = (0, 0)  # (seed0, seed1)
    card_rng_state: Tuple[int, int] = (0, 0)
    ai_rng_state: Tuple[int, int] = (0, 0)

    # Relic flags
    has_violet_lotus: bool = False
    has_barricade: bool = False
    has_runic_pyramid: bool = False

    # Tracking
    total_damage_dealt: int = 0
    total_damage_taken: int = 0
    total_cards_played: int = 0

    def copy(self) -> SimCombatState:
        return SimCombatState(
            player=self.player.copy(),
            enemies=[e.copy() for e in self.enemies],
            turn=self.turn,
            combat_over=self.combat_over,
            player_won=self.player_won,
            shuffle_rng_state=self.shuffle_rng_state,
            card_rng_state=self.card_rng_state,
            ai_rng_state=self.ai_rng_state,
            has_violet_lotus=self.has_violet_lotus,
            has_barricade=self.has_barricade,
            has_runic_pyramid=self.has_runic_pyramid,
            total_damage_dealt=self.total_damage_dealt,
            total_damage_taken=self.total_damage_taken,
            total_cards_played=self.total_cards_played,
        )

    def get_living_enemies(self) -> List[EnemyCombatState]:
        return [e for e in self.enemies if e.is_alive()]

    def all_enemies_dead(self) -> bool:
        return all(not e.is_alive() for e in self.enemies)


# =============================================================================
# COMBAT RESULT
# =============================================================================

@dataclass
class CombatResult:
    """Result of a completed combat simulation."""
    victory: bool
    hp_remaining: int
    hp_lost: int
    turns: int
    cards_played: int
    damage_dealt: int
    damage_taken: int

    # Detailed tracking
    cards_played_sequence: List[str] = field(default_factory=list)
    stance_changes: int = 0
    energy_spent: int = 0


# =============================================================================
# COMBAT SIMULATOR
# =============================================================================

class CombatSimulator:
    """
    Simulates combat and finds optimal plays.

    Key design principles:
    1. State is never mutated - all methods return new states
    2. RNG is fully deterministic given initial state
    3. Supports tree search via state copying
    """

    def __init__(
        self,
        card_data: Optional[Dict[str, Card]] = None,
        enemy_data: Optional[Dict] = None,
    ):
        """
        Initialize simulator with game data.

        Args:
            card_data: Card definitions (uses ALL_CARDS if None)
            enemy_data: Enemy definitions (not currently used, enemies passed directly)
        """
        self.card_data = card_data or ALL_CARDS
        self.enemy_data = enemy_data or {}

    def setup_combat(
        self,
        deck: List[str],
        enemies: List[Enemy],
        player_hp: int,
        player_max_hp: int,
        relics: List[str] = None,
        potions: List[str] = None,
        ascension: int = 20,
        shuffle_rng: Random = None,
        card_rng: Random = None,
        ai_rng: Random = None,
    ) -> SimCombatState:
        """
        Initialize combat state.

        Args:
            deck: List of card IDs (e.g., ["Strike_P", "Defend_P", ...])
            enemies: List of Enemy objects
            player_hp: Current HP
            player_max_hp: Maximum HP
            relics: List of relic IDs
            potions: List of potion IDs
            ascension: Ascension level
            shuffle_rng: RNG for deck shuffling
            card_rng: RNG for card effects
            ai_rng: RNG for enemy AI

        Returns:
            Initial SimCombatState
        """
        relics = relics or []
        potions = potions or []

        # Initialize RNG if not provided
        if shuffle_rng is None:
            shuffle_rng = Random(12345)
        if card_rng is None:
            card_rng = Random(12346)
        if ai_rng is None:
            ai_rng = Random(12347)

        # Build draw pile from deck
        draw_pile = []
        for card_id in deck:
            # Check if card ID includes upgrade marker
            upgraded = card_id.endswith("+")
            base_id = card_id.rstrip("+")
            draw_pile.append((base_id, upgraded))

        # Shuffle draw pile
        draw_pile = self._shuffle_pile(draw_pile, shuffle_rng)

        # Check relic flags
        has_violet_lotus = "VioletLotus" in relics
        has_barricade = "Barricade" in relics or any("Barricade" in r for r in relics)
        has_runic_pyramid = "Runic Pyramid" in relics

        # Determine base energy
        base_energy = 3
        for relic in relics:
            relic_lower = relic.lower()
            if any(r in relic_lower for r in ["coffee dripper", "busted crown", "cursed key",
                                               "ectoplasm", "fusion hammer", "philosopher",
                                               "runic dome", "sozu", "velvet choker"]):
                base_energy = 4
                break

        # Create player state
        player = PlayerCombatState(
            current_hp=player_hp,
            max_hp=player_max_hp,
            energy=base_energy,
            max_energy=base_energy,
            draw_pile=draw_pile,
        )

        # Convert enemies to combat state
        enemy_states = []
        for enemy in enemies:
            # Roll initial move
            enemy.roll_move()

            enemy_combat = EnemyCombatState(
                id=enemy.ID,
                name=enemy.NAME,
                enemy_type=enemy.TYPE,
                current_hp=enemy.state.current_hp,
                max_hp=enemy.state.max_hp,
                block=enemy.state.block,
                strength=enemy.state.strength,
                powers=dict(enemy.state.powers),
                move_history=list(enemy.state.move_history),
                next_move=enemy.state.next_move,
                first_turn=enemy.state.first_turn,
            )
            enemy_states.append(enemy_combat)

        # Create initial state
        state = SimCombatState(
            player=player,
            enemies=enemy_states,
            shuffle_rng_state=(shuffle_rng.rng.seed0, shuffle_rng.rng.seed1),
            card_rng_state=(card_rng.rng.seed0, card_rng.rng.seed1),
            ai_rng_state=(ai_rng.rng.seed0, ai_rng.rng.seed1),
            has_violet_lotus=has_violet_lotus,
            has_barricade=has_barricade,
            has_runic_pyramid=has_runic_pyramid,
        )

        # Draw starting hand
        state = self._draw_cards(state, 5)

        return state

    def _shuffle_pile(
        self,
        pile: List[Tuple[str, bool]],
        rng: Random,
    ) -> List[Tuple[str, bool]]:
        """Shuffle a card pile using Fisher-Yates."""
        result = list(pile)
        n = len(result)
        for i in range(n - 1, 0, -1):
            j = rng.random(i)
            result[i], result[j] = result[j], result[i]
        return result

    def _draw_cards(self, state: SimCombatState, count: int) -> SimCombatState:
        """Draw cards from draw pile to hand."""
        state = state.copy()

        for _ in range(count):
            if not state.player.draw_pile:
                # Shuffle discard into draw
                if not state.player.discard_pile:
                    break

                # Create RNG from state
                rng = Random.__new__(Random)
                rng.rng = type(rng).__new__(type(rng))

                # Copy discard to draw and shuffle
                state.player.draw_pile = list(state.player.discard_pile)
                state.player.discard_pile = []

                # Simple deterministic shuffle based on state
                n = len(state.player.draw_pile)
                for i in range(n - 1, 0, -1):
                    # Use a deterministic index based on position and turn
                    j = (state.shuffle_rng_state[0] + i * 7 + state.turn) % (i + 1)
                    state.player.draw_pile[i], state.player.draw_pile[j] = \
                        state.player.draw_pile[j], state.player.draw_pile[i]

            if state.player.draw_pile:
                card = state.player.draw_pile.pop()
                state.player.hand.append(card)

        return state

    def _get_card_from_tuple(self, card_tuple: Tuple[str, bool]) -> Card:
        """Get a Card object from a (card_id, upgraded) tuple."""
        card_id, upgraded = card_tuple
        card = get_card(card_id, upgraded)
        return card

    def get_legal_actions(self, state: SimCombatState) -> List[Action]:
        """
        Get all legal actions from current state.

        Returns list of Action objects representing:
        - Playing each playable card
        - Ending the turn
        """
        if state.combat_over:
            return []

        actions = []

        # Check each card in hand
        for i, card_tuple in enumerate(state.player.hand):
            card = self._get_card_from_tuple(card_tuple)

            if self._can_play_card(state, card, i):
                # Determine targets
                if card.target == CardTarget.ENEMY:
                    # Must target a specific enemy
                    for j, enemy in enumerate(state.enemies):
                        if enemy.is_alive():
                            actions.append(Action(
                                action_type=ActionType.PLAY_CARD,
                                card_index=i,
                                target_index=j,
                            ))
                else:
                    # No target or self-target
                    actions.append(Action(
                        action_type=ActionType.PLAY_CARD,
                        card_index=i,
                        target_index=0,
                    ))

        # Always can end turn
        actions.append(Action(action_type=ActionType.END_TURN))

        return actions

    def _can_play_card(self, state: SimCombatState, card: Card, hand_index: int) -> bool:
        """Check if a card can be played."""
        # Energy check
        if card.current_cost > state.player.energy:
            return False

        # Unplayable check (curses, statuses)
        if card.cost == -2 or "unplayable" in card.effects:
            return False

        # Signature Move check
        if "only_attack_in_hand" in card.effects:
            attacks_in_hand = sum(
                1 for ct in state.player.hand
                if self._get_card_from_tuple(ct).card_type == CardType.ATTACK
            )
            if attacks_in_hand > 1:
                return False

        # Entangled check
        if state.player.has_power("Entangled") and card.card_type == CardType.ATTACK:
            return False

        return True

    def execute_action(self, state: SimCombatState, action: Action) -> SimCombatState:
        """
        Execute an action and return new state.

        IMPORTANT: Does NOT mutate the input state.
        """
        if action.action_type == ActionType.END_TURN:
            return self._end_player_turn(state)
        elif action.action_type == ActionType.PLAY_CARD:
            return self._play_card(state, action.card_index, action.target_index)

        return state.copy()

    def _play_card(
        self,
        state: SimCombatState,
        hand_index: int,
        target_index: int,
    ) -> SimCombatState:
        """Play a card from hand."""
        state = state.copy()

        if hand_index >= len(state.player.hand):
            return state

        card_tuple = state.player.hand[hand_index]
        card = self._get_card_from_tuple(card_tuple)

        if not self._can_play_card(state, card, hand_index):
            return state

        # Pay energy
        state.player.energy -= card.current_cost

        # Remove from hand
        state.player.hand.pop(hand_index)

        # Track card play
        state.player.cards_played_this_turn += 1
        state.player.last_card_type = card.card_type
        state.total_cards_played += 1

        if card.card_type == CardType.ATTACK:
            state.player.attacks_played_this_turn += 1

        # Get target enemy
        target_enemy = None
        if target_index < len(state.enemies) and state.enemies[target_index].is_alive():
            target_enemy = state.enemies[target_index]

        # Apply card effects
        state = self._apply_card_effects(state, card, target_index)

        # Card destination
        if card.exhaust:
            state.player.exhaust_pile.append(card_tuple)
        elif card.shuffle_back:
            # Insert at random position in draw pile
            pos = (state.shuffle_rng_state[0] + state.turn) % (len(state.player.draw_pile) + 1)
            state.player.draw_pile.insert(pos, card_tuple)
        else:
            state.player.discard_pile.append(card_tuple)

        # Check for end turn effect
        if "end_turn" in card.effects:
            state = self._end_player_turn(state)

        # Check combat end
        state = self._check_combat_end(state)

        return state

    def _apply_card_effects(
        self,
        state: SimCombatState,
        card: Card,
        target_index: int,
    ) -> SimCombatState:
        """Apply a card's effects."""
        # Build damage calculation state
        damage_state = self._build_damage_state(state, target_index)

        # Damage
        if card.damage > 0:
            hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
            per_hit_damage = calculate_card_damage(card.damage, damage_state, card.id)

            for _ in range(hits):
                if target_index < len(state.enemies) and state.enemies[target_index].is_alive():
                    enemy = state.enemies[target_index]

                    # Apply damage to enemy
                    blocked = min(enemy.block, per_hit_damage)
                    hp_damage = per_hit_damage - blocked
                    enemy.block -= blocked
                    enemy.current_hp -= hp_damage

                    state.total_damage_dealt += hp_damage

                    if enemy.current_hp <= 0:
                        enemy.current_hp = 0

        # Block
        if card.block > 0:
            block_gained = calculate_block(card.block, damage_state)
            state.player.block += block_gained

        # Stance changes
        if card.enter_stance:
            state = self._change_stance(state, StanceID(card.enter_stance.upper()))

        if card.exit_stance:
            state = self._change_stance(state, StanceID.NEUTRAL)

        # Draw effects
        if "draw_1" in card.effects:
            state = self._draw_cards(state, 1)
        if "draw_2" in card.effects:
            state = self._draw_cards(state, 2)
        if "draw_cards" in card.effects and card.magic_number > 0:
            state = self._draw_cards(state, card.magic_number)

        # Mantra
        if "gain_mantra" in card.effects and card.magic_number > 0:
            state = self._add_mantra(state, card.magic_number)

        # Apply powers
        if card.card_type == CardType.POWER:
            state = self._apply_power_card(state, card)

        return state

    def _build_damage_state(
        self,
        state: SimCombatState,
        target_index: int,
    ) -> DamageCombatState:
        """Build damage calculation state from combat state."""
        player_powers = []

        # Strength
        str_amt = state.player.get_power("Strength")
        if str_amt != 0:
            player_powers.append(Power("Strength", str_amt))

        # Weak
        if state.player.has_power("Weak") or state.player.has_power("Weakened"):
            weak_amt = state.player.get_power("Weak") or state.player.get_power("Weakened")
            player_powers.append(Power("Weak", weak_amt))

        # Vigor
        vigor = state.player.get_power("Vigor")
        if vigor > 0:
            player_powers.append(Power("Vigor", vigor))

        # Dexterity
        dex = state.player.get_power("Dexterity")
        if dex != 0:
            player_powers.append(Power("Dexterity", dex))

        # Frail
        if state.player.has_power("Frail"):
            player_powers.append(Power("Frail", 1))

        target_powers = []
        if target_index < len(state.enemies):
            enemy = state.enemies[target_index]
            if enemy.powers.get("Vulnerable", 0) > 0:
                target_powers.append(Power("Vulnerable", 1))

        # Stance multiplier
        stance_effect = STANCES[state.player.stance]
        stance_mult = stance_effect.damage_give_multiplier
        stance_incoming_mult = stance_effect.damage_receive_multiplier

        return DamageCombatState(
            player_powers=player_powers,
            stance_damage_mult=stance_mult,
            stance_incoming_mult=stance_incoming_mult,
            target_powers=target_powers,
        )

    def _change_stance(
        self,
        state: SimCombatState,
        new_stance: StanceID,
    ) -> SimCombatState:
        """Change stance and handle effects."""
        old_stance = state.player.stance

        if old_stance == new_stance:
            return state

        # Exit effects
        if old_stance == StanceID.CALM:
            # Gain energy
            energy_gain = 3 if state.has_violet_lotus else 2
            state.player.energy += energy_gain

        # Enter effects
        if new_stance == StanceID.DIVINITY:
            state.player.energy += 3

        state.player.stance = new_stance

        # Mental Fortress trigger
        if state.player.has_power("MentalFortress"):
            state.player.block += state.player.get_power("MentalFortress")

        # Rushdown trigger
        if new_stance == StanceID.WRATH and state.player.has_power("Rushdown"):
            state = self._draw_cards(state, state.player.get_power("Rushdown"))

        # Flurry of Blows trigger
        flurries = [(i, ct) for i, ct in enumerate(state.player.discard_pile)
                    if ct[0] == "FlurryOfBlows"]
        for i, ct in reversed(flurries):
            state.player.discard_pile.pop(i)
            state.player.hand.append(ct)

        return state

    def _add_mantra(self, state: SimCombatState, amount: int) -> SimCombatState:
        """Add mantra and potentially enter Divinity."""
        state.player.mantra += amount

        if state.player.mantra >= 10:
            state.player.mantra -= 10
            state = self._change_stance(state, StanceID.DIVINITY)

        return state

    def _apply_power_card(self, state: SimCombatState, card: Card) -> SimCombatState:
        """Apply a power card's effect."""
        power_mapping = {
            "MentalFortress": ("MentalFortress", card.magic_number if card.magic_number > 0 else 4),
            "Rushdown": ("Rushdown", card.magic_number if card.magic_number > 0 else 2),
            "Nirvana": ("Nirvana", card.magic_number if card.magic_number > 0 else 3),
            "LikeWater": ("LikeWater", card.magic_number if card.magic_number > 0 else 5),
            "DevaForm": ("DevaForm", 1),
            "Devotion": ("Devotion", card.magic_number if card.magic_number > 0 else 2),
            "Foresight": ("Foresight", card.magic_number if card.magic_number > 0 else 3),
            "Establishment": ("Establishment", 1),
            "BattleHymn": ("BattleHymn", 1),
            "Study": ("Study", 1),
        }

        if card.id in power_mapping:
            power_id, amount = power_mapping[card.id]
            current = state.player.powers.get(power_id, 0)
            state.player.powers[power_id] = current + amount

        return state

    def _end_player_turn(self, state: SimCombatState) -> SimCombatState:
        """End the player's turn."""
        state = state.copy()

        # Discard hand (unless Runic Pyramid)
        if not state.has_runic_pyramid:
            retained = []
            for card_tuple in state.player.hand:
                card = self._get_card_from_tuple(card_tuple)
                if card.retain:
                    retained.append(card_tuple)
                elif card.ethereal:
                    state.player.exhaust_pile.append(card_tuple)
                else:
                    state.player.discard_pile.append(card_tuple)
            state.player.hand = retained

        # Like Water
        if state.player.has_power("LikeWater") and state.player.stance == StanceID.CALM:
            state.player.block += state.player.get_power("LikeWater")

        # Divinity auto-exit
        if state.player.stance == StanceID.DIVINITY:
            state = self._change_stance(state, StanceID.NEUTRAL)

        # Process enemy turns
        state = self.simulate_enemy_turn(state)

        if state.combat_over:
            return state

        # Start next turn
        state = self.simulate_turn_start(state)

        return state

    def simulate_enemy_turn(self, state: SimCombatState) -> SimCombatState:
        """Execute all enemy actions."""
        state = state.copy()

        for enemy in state.enemies:
            if not enemy.is_alive():
                continue

            move = enemy.next_move
            if not move:
                continue

            # Apply strength to damage
            enemy_strength = enemy.strength + enemy.powers.get("strength", 0)

            # Execute attack moves
            if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
                base_damage = move.base_damage + enemy_strength
                hits = move.hits

                # Calculate damage with player's stance
                stance_effect = STANCES[state.player.stance]
                damage_mult = stance_effect.damage_receive_multiplier

                for _ in range(hits):
                    damage = int(base_damage * damage_mult)

                    # Apply block
                    blocked = min(state.player.block, damage)
                    hp_damage = damage - blocked
                    state.player.block -= blocked
                    state.player.current_hp -= hp_damage

                    state.total_damage_taken += hp_damage

                    if state.player.current_hp <= 0:
                        state.player.current_hp = 0
                        state.combat_over = True
                        state.player_won = False
                        return state

            # Enemy block
            if move.block > 0:
                enemy.block += move.block

            # Enemy buffs/debuffs
            if "strength" in move.effects:
                enemy.strength += move.effects["strength"]

            if "weak" in move.effects:
                current = state.player.powers.get("Weakened", 0)
                state.player.powers["Weakened"] = current + move.effects["weak"]

            if "vulnerable" in move.effects:
                current = state.player.powers.get("Vulnerable", 0)
                state.player.powers["Vulnerable"] = current + move.effects["vulnerable"]

            if "frail" in move.effects:
                current = state.player.powers.get("Frail", 0)
                state.player.powers["Frail"] = current + move.effects["frail"]

        # Decrement player debuffs
        for debuff in ["Weakened", "Vulnerable", "Frail"]:
            if state.player.has_power(debuff):
                state.player.powers[debuff] -= 1
                if state.player.powers[debuff] <= 0:
                    del state.player.powers[debuff]

        # Enemy block decay
        for enemy in state.enemies:
            enemy.block = 0

        # Roll next moves
        for enemy in state.enemies:
            if enemy.is_alive():
                self._roll_enemy_move(state, enemy)

        return state

    def _roll_enemy_move(self, state: SimCombatState, enemy: EnemyCombatState):
        """Roll next move for an enemy using deterministic logic."""
        # Simplified move rolling - uses state for determinism
        # In practice, would need full enemy AI logic
        roll = (state.ai_rng_state[0] + state.turn * 17 + hash(enemy.id)) % 100

        # Simple pattern: alternate between attack and other moves
        if len(enemy.move_history) == 0 or enemy.move_history[-1] != 1:
            # Attack move
            enemy.next_move = MoveInfo(
                move_id=1,
                name="Attack",
                intent=Intent.ATTACK,
                base_damage=6,
                hits=1,
            )
        else:
            # Other move
            enemy.next_move = MoveInfo(
                move_id=2,
                name="Buff",
                intent=Intent.BUFF,
                effects={"strength": 1},
            )

        enemy.move_history.append(enemy.next_move.move_id)

    def simulate_turn_end(self, state: SimCombatState) -> SimCombatState:
        """Process end of turn: discard hand, tick statuses, etc."""
        return self._end_player_turn(state)

    def simulate_turn_start(self, state: SimCombatState) -> SimCombatState:
        """Process start of turn: draw cards, reset energy, etc."""
        state = state.copy()

        state.turn += 1

        # Reset energy
        state.player.energy = state.player.max_energy

        # Lose block (unless Barricade)
        if not state.has_barricade and not state.player.has_power("Barricade"):
            state.player.block = 0

        # Reset turn counters
        state.player.cards_played_this_turn = 0
        state.player.attacks_played_this_turn = 0
        state.player.last_card_type = None

        # Draw cards
        draw_count = 5
        if state.player.has_power("NoDraw") or state.player.has_power("No Draw"):
            draw_count = 0

        state = self._draw_cards(state, draw_count)

        # Deva Form energy
        if state.player.has_power("DevaForm"):
            state.player.energy += state.player.get_power("DevaForm")
            state.player.powers["DevaForm"] = state.player.get_power("DevaForm") + 1

        # Devotion mantra
        if state.player.has_power("Devotion"):
            state = self._add_mantra(state, state.player.get_power("Devotion"))

        return state

    def _check_combat_end(self, state: SimCombatState) -> SimCombatState:
        """Check if combat should end."""
        if state.all_enemies_dead():
            state.combat_over = True
            state.player_won = True
        elif not state.player.is_alive():
            state.combat_over = True
            state.player_won = False

        return state

    def simulate_full_combat(
        self,
        state: SimCombatState,
        policy: Callable[[SimCombatState], Action],
        max_turns: int = 100,
    ) -> CombatResult:
        """
        Run combat to completion with given policy.

        Args:
            state: Initial combat state
            policy: Function that takes state and returns action
            max_turns: Maximum turns before timeout

        Returns:
            CombatResult with outcome and statistics
        """
        cards_played_sequence = []
        energy_spent = 0
        stance_changes = 0
        initial_hp = state.player.current_hp

        while not state.combat_over and state.turn <= max_turns:
            actions = self.get_legal_actions(state)
            if not actions:
                break

            action = policy(state)

            # Track card plays
            if action.action_type == ActionType.PLAY_CARD:
                card_tuple = state.player.hand[action.card_index]
                card = self._get_card_from_tuple(card_tuple)
                cards_played_sequence.append(card.id)
                energy_spent += card.current_cost

            old_stance = state.player.stance
            state = self.execute_action(state, action)

            if state.player.stance != old_stance:
                stance_changes += 1

        return CombatResult(
            victory=state.player_won,
            hp_remaining=state.player.current_hp,
            hp_lost=initial_hp - state.player.current_hp + state.total_damage_taken,
            turns=state.turn,
            cards_played=state.total_cards_played,
            damage_dealt=state.total_damage_dealt,
            damage_taken=state.total_damage_taken,
            cards_played_sequence=cards_played_sequence,
            stance_changes=stance_changes,
            energy_spent=energy_spent,
        )

    def random_policy(self, state: SimCombatState) -> Action:
        """Random legal action - baseline policy."""
        import random
        actions = self.get_legal_actions(state)
        return random.choice(actions) if actions else Action(ActionType.END_TURN)

    def greedy_policy(self, state: SimCombatState) -> Action:
        """
        Greedy policy: maximize immediate damage or block.

        Priority:
        1. Lethal damage if possible
        2. Play attacks when enemies have low HP
        3. Play block when enemy is attacking
        4. Play powers/setup cards
        5. End turn
        """
        actions = self.get_legal_actions(state)

        if len(actions) <= 1:
            return actions[0] if actions else Action(ActionType.END_TURN)

        # Score each action
        best_action = actions[-1]  # Default to end turn
        best_score = -float('inf')

        # Get incoming damage
        incoming_damage = self._estimate_incoming_damage(state)

        # Get total enemy HP
        total_enemy_hp = sum(e.current_hp for e in state.enemies if e.is_alive())

        for action in actions:
            if action.action_type == ActionType.END_TURN:
                # End turn has base score of 0
                score = 0
            else:
                card_tuple = state.player.hand[action.card_index]
                card = self._get_card_from_tuple(card_tuple)

                # Calculate damage output
                damage_state = self._build_damage_state(state, action.target_index)
                damage = 0
                if card.damage > 0:
                    hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
                    damage = calculate_card_damage(card.damage, damage_state, card.id) * hits

                # Calculate block
                block = 0
                if card.block > 0:
                    block = calculate_block(card.block, damage_state)

                # Scoring
                score = 0

                # Lethal bonus (huge priority)
                if damage >= total_enemy_hp:
                    score += 1000

                # Damage score (weighted by urgency)
                if total_enemy_hp > 0:
                    score += damage * (100 / total_enemy_hp)
                else:
                    score += damage

                # Block score (weighted by incoming damage)
                if incoming_damage > 0:
                    # Value block highly if we're about to take damage
                    useful_block = min(block, incoming_damage - state.player.block)
                    score += useful_block * 1.5

                # Stance entry bonuses
                if card.enter_stance == "Wrath":
                    # Wrath is good if we have attacks to follow up
                    attacks_in_hand = sum(
                        1 for ct in state.player.hand
                        if self._get_card_from_tuple(ct).card_type == CardType.ATTACK
                    )
                    if attacks_in_hand > 1:
                        score += 20
                    # But bad if enemy is attacking
                    if incoming_damage > 0:
                        score -= 10

                if card.enter_stance == "Calm":
                    # Calm is good for energy setup
                    score += 5

                # Power bonus (play powers early)
                if card.card_type == CardType.POWER:
                    score += 15

                # Energy efficiency
                if card.current_cost > 0:
                    score = score / card.current_cost
                else:
                    score *= 1.5  # 0-cost cards are great

            if score > best_score:
                best_score = score
                best_action = action

        return best_action

    def _estimate_incoming_damage(self, state: SimCombatState) -> int:
        """Estimate total incoming damage from enemies."""
        total = 0

        for enemy in state.enemies:
            if not enemy.is_alive():
                continue

            move = enemy.next_move
            if not move:
                continue

            if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
                damage = move.base_damage + enemy.strength

                # Apply stance multiplier
                stance_effect = STANCES[state.player.stance]
                damage = int(damage * stance_effect.damage_receive_multiplier)

                total += damage * move.hits

        return total


# =============================================================================
# TESTING
# =============================================================================

if __name__ == "__main__":
    print("=== Combat Simulator Test ===\n")

    # Setup
    sim = CombatSimulator()

    # Create a simple deck
    deck = [
        "Strike_P", "Strike_P", "Strike_P", "Strike_P",
        "Defend_P", "Defend_P", "Defend_P", "Defend_P",
        "Eruption", "Vigilance"
    ]

    # Create enemy
    from enemies import JawWorm
    ai_rng = Random(12345)
    hp_rng = Random(12346)
    enemies = [JawWorm(ai_rng, ascension=0, hp_rng=hp_rng)]

    # Setup combat
    state = sim.setup_combat(
        deck=deck,
        enemies=enemies,
        player_hp=80,
        player_max_hp=80,
        ascension=0,
    )

    print(f"Initial state:")
    print(f"  Player HP: {state.player.current_hp}/{state.player.max_hp}")
    print(f"  Energy: {state.player.energy}")
    print(f"  Hand: {[ct[0] for ct in state.player.hand]}")
    print(f"  Enemy: {state.enemies[0].name} - {state.enemies[0].current_hp} HP")
    print(f"  Enemy intent: {state.enemies[0].next_move.intent.value if state.enemies[0].next_move else 'None'}")

    # Run with greedy policy
    print("\n--- Running with greedy policy ---")
    result = sim.simulate_full_combat(state, sim.greedy_policy)

    print(f"\nResult:")
    print(f"  Victory: {result.victory}")
    print(f"  HP remaining: {result.hp_remaining}")
    print(f"  HP lost: {result.hp_lost}")
    print(f"  Turns: {result.turns}")
    print(f"  Cards played: {result.cards_played}")
    print(f"  Damage dealt: {result.damage_dealt}")

    # Run with random policy for comparison
    print("\n--- Running with random policy ---")
    state2 = sim.setup_combat(
        deck=deck,
        enemies=[JawWorm(Random(12345), ascension=0, hp_rng=Random(12346))],
        player_hp=80,
        player_max_hp=80,
        ascension=0,
    )
    result2 = sim.simulate_full_combat(state2, sim.random_policy)

    print(f"\nResult:")
    print(f"  Victory: {result2.victory}")
    print(f"  HP remaining: {result2.hp_remaining}")
    print(f"  Turns: {result2.turns}")

    print("\n=== Test Complete ===")
