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
from ..state.combat import (
    CombatState,
    EnemyCombatState as CoreEnemyCombatState,
    EntityState,
    create_player,
    create_enemy,
    create_combat,
)
from ..content.cards import Card, CardType, CardTarget, get_card, ALL_CARDS
from ..content.enemies import Enemy, Intent, MoveInfo, EnemyState, EnemyType
from ..content.stances import StanceID, StanceManager, STANCES
from .damage import calculate_damage, calculate_block


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
# HELPER: Card ID encoding
# =============================================================================

def encode_card_id(card_id: str, upgraded: bool) -> str:
    """Encode card ID with upgrade status (e.g., 'Strike_P+')."""
    if upgraded and not card_id.endswith('+'):
        return f"{card_id}+"
    return card_id


def decode_card_id(card_id: str) -> Tuple[str, bool]:
    """Decode card ID to (base_id, upgraded)."""
    if card_id.endswith('+'):
        return card_id[:-1], True
    return card_id, False


# =============================================================================
# HELPER: MoveInfo storage on EnemyCombatState
# =============================================================================

def set_enemy_move(enemy: CoreEnemyCombatState, move: Optional[MoveInfo]) -> None:
    """Set enemy move info using the enemy's fields."""
    if move is None:
        enemy.move_id = -1
        enemy.move_damage = 0
        enemy.move_hits = 1
        enemy.move_block = 0
        enemy.move_effects = {}
    else:
        enemy.move_id = move.move_id
        enemy.move_damage = move.base_damage
        enemy.move_hits = move.hits
        enemy.move_block = move.block
        enemy.move_effects = dict(move.effects) if move.effects else {}


def get_enemy_move(enemy: CoreEnemyCombatState) -> Optional[MoveInfo]:
    """Get MoveInfo from enemy's fields."""
    if enemy.move_id == -1:
        return None
    return MoveInfo(
        move_id=enemy.move_id,
        name="",  # Name not stored
        intent=Intent.ATTACK if enemy.move_damage > 0 else Intent.UNKNOWN,
        base_damage=enemy.move_damage,
        hits=enemy.move_hits,
        block=enemy.move_block,
        effects=dict(enemy.move_effects) if enemy.move_effects else {},
    )


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

    Uses CombatState from core.state.combat as the canonical state representation.
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
    ) -> CombatState:
        """
        Initialize combat state.

        Args:
            deck: List of card IDs (e.g., ["Strike_P", "Defend_P", ...])
                  Upgraded cards can be marked with '+' suffix
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
            Initial CombatState
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

        # Build draw pile from deck (keep upgrade markers in card IDs)
        draw_pile = list(deck)

        # Shuffle draw pile
        draw_pile = self._shuffle_pile(draw_pile, shuffle_rng)

        # Check relic flags
        has_violet_lotus = "VioletLotus" in relics or "Violet Lotus" in relics
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

        # Convert enemies to combat state
        enemy_states = []
        for enemy in enemies:
            # Roll initial move
            enemy.roll_move()

            enemy_combat = CoreEnemyCombatState(
                hp=enemy.state.current_hp,
                max_hp=enemy.state.max_hp,
                block=enemy.state.block,
                statuses=dict(enemy.state.powers),
                id=enemy.ID,
                name=enemy.NAME,
                enemy_type=str(enemy.TYPE.value) if hasattr(enemy.TYPE, 'value') else str(enemy.TYPE),
                move_history=list(enemy.state.move_history),
                first_turn=enemy.state.first_turn,
            )
            # Set the move info
            set_enemy_move(enemy_combat, enemy.state.next_move)
            # Set strength in statuses
            if enemy.state.strength != 0:
                enemy_combat.statuses["Strength"] = enemy.state.strength
            enemy_states.append(enemy_combat)

        # Create initial state using CombatState
        state = CombatState(
            player=EntityState(hp=player_hp, max_hp=player_max_hp),
            energy=base_energy,
            max_energy=base_energy,
            stance="Neutral",
            draw_pile=draw_pile,
            hand=[],
            discard_pile=[],
            exhaust_pile=[],
            enemies=enemy_states,
            potions=potions if potions else ["", "", ""],
            relics=relics,
            shuffle_rng_state=(shuffle_rng._rng.seed0, shuffle_rng._rng.seed1),
            card_rng_state=(card_rng._rng.seed0, card_rng._rng.seed1),
            ai_rng_state=(ai_rng._rng.seed0, ai_rng._rng.seed1),
        )

        # Store relic flags in relic_counters for quick access
        if has_violet_lotus:
            state.relic_counters["_violet_lotus"] = 1
        if has_barricade:
            state.relic_counters["_barricade"] = 1
        if has_runic_pyramid:
            state.relic_counters["_runic_pyramid"] = 1

        # Draw starting hand
        state = self._draw_cards(state, 5)

        return state

    def _has_violet_lotus(self, state: CombatState) -> bool:
        """Check if player has Violet Lotus relic."""
        return state.relic_counters.get("_violet_lotus", 0) > 0 or state.has_relic("Violet Lotus") or state.has_relic("VioletLotus")

    def _has_barricade(self, state: CombatState) -> bool:
        """Check if player has Barricade (relic or power)."""
        return (state.relic_counters.get("_barricade", 0) > 0 or
                state.has_relic("Barricade") or
                state.player.statuses.get("Barricade", 0) > 0)

    def _has_runic_pyramid(self, state: CombatState) -> bool:
        """Check if player has Runic Pyramid relic."""
        return state.relic_counters.get("_runic_pyramid", 0) > 0 or state.has_relic("Runic Pyramid")

    def _get_stance_id(self, stance_str: str) -> StanceID:
        """Convert stance string to StanceID enum."""
        if not stance_str:
            return StanceID.NEUTRAL
        # Handle both formats: "Neutral", "neutral", "NEUTRAL", etc.
        stance_lower = stance_str.lower()
        if stance_lower == "neutral":
            return StanceID.NEUTRAL
        elif stance_lower == "calm":
            return StanceID.CALM
        elif stance_lower == "wrath":
            return StanceID.WRATH
        elif stance_lower == "divinity":
            return StanceID.DIVINITY
        else:
            return StanceID.NEUTRAL

    def _shuffle_pile(
        self,
        pile: List[str],
        rng: Random,
    ) -> List[str]:
        """Shuffle a card pile using Fisher-Yates."""
        result = list(pile)
        n = len(result)
        for i in range(n - 1, 0, -1):
            j = rng.random(i)
            result[i], result[j] = result[j], result[i]
        return result

    def _draw_cards(self, state: CombatState, count: int) -> CombatState:
        """Draw cards from draw pile to hand."""
        state = state.copy()

        for _ in range(count):
            if not state.draw_pile:
                # Shuffle discard into draw
                if not state.discard_pile:
                    break

                # Copy discard to draw and shuffle
                state.draw_pile = list(state.discard_pile)
                state.discard_pile = []

                # Simple deterministic shuffle based on state
                n = len(state.draw_pile)
                for i in range(n - 1, 0, -1):
                    # Use a deterministic index based on position and turn
                    j = (state.shuffle_rng_state[0] + i * 7 + state.turn) % (i + 1)
                    state.draw_pile[i], state.draw_pile[j] = \
                        state.draw_pile[j], state.draw_pile[i]

            if state.draw_pile:
                card = state.draw_pile.pop()
                state.hand.append(card)

        return state

    def _get_card(self, card_id: str) -> Card:
        """Get a Card object from a card ID (with optional '+' suffix for upgraded)."""
        base_id, upgraded = decode_card_id(card_id)
        return get_card(base_id, upgraded)

    def get_legal_actions(self, state: CombatState) -> List[Action]:
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
        for i, card_id in enumerate(state.hand):
            card = self._get_card(card_id)

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

    def _can_play_card(self, state: CombatState, card: Card, hand_index: int) -> bool:
        """Check if a card can be played."""
        # Energy check
        if card.current_cost > state.energy:
            return False

        # Unplayable check (curses, statuses)
        if card.cost == -2 or "unplayable" in card.effects:
            return False

        # Signature Move check
        if "only_attack_in_hand" in card.effects:
            attacks_in_hand = sum(
                1 for card_id in state.hand
                if self._get_card(card_id).card_type == CardType.ATTACK
            )
            if attacks_in_hand > 1:
                return False

        # Entangled check
        if state.player.statuses.get("Entangled", 0) > 0 and card.card_type == CardType.ATTACK:
            return False

        return True

    def execute_action(self, state: CombatState, action: Action) -> CombatState:
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
        state: CombatState,
        hand_index: int,
        target_index: int,
    ) -> CombatState:
        """Play a card from hand."""
        state = state.copy()

        if hand_index >= len(state.hand):
            return state

        card_id = state.hand[hand_index]
        card = self._get_card(card_id)

        if not self._can_play_card(state, card, hand_index):
            return state

        # Pay energy
        state.energy -= card.current_cost

        # Remove from hand
        state.hand.pop(hand_index)

        # Track card play
        state.cards_played_this_turn += 1
        state.last_card_type = card.card_type.value if hasattr(card.card_type, 'value') else str(card.card_type)
        state.total_cards_played += 1

        if card.card_type == CardType.ATTACK:
            state.attacks_played_this_turn += 1

        # Get target enemy
        target_enemy = None
        if target_index < len(state.enemies) and state.enemies[target_index].is_alive():
            target_enemy = state.enemies[target_index]

        # Apply card effects
        state = self._apply_card_effects(state, card, target_index)

        # Card destination
        if card.exhaust:
            state.exhaust_pile.append(card_id)
        elif card.shuffle_back:
            # Insert at random position in draw pile
            pos = (state.shuffle_rng_state[0] + state.turn) % (len(state.draw_pile) + 1)
            state.draw_pile.insert(pos, card_id)
        else:
            state.discard_pile.append(card_id)

        # Check for end turn effect
        if "end_turn" in card.effects:
            state = self._end_player_turn(state)

        # Check combat end
        state = self._check_combat_end(state)

        return state

    def _apply_card_effects(
        self,
        state: CombatState,
        card: Card,
        target_index: int,
    ) -> CombatState:
        """Apply a card's effects."""
        # Damage
        if card.damage > 0:
            hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
            per_hit_damage = self._calculate_card_damage(state, card.damage, target_index)

            for _ in range(hits):
                if target_index < len(state.enemies) and state.enemies[target_index].is_alive():
                    enemy = state.enemies[target_index]

                    # Apply damage to enemy
                    blocked = min(enemy.block, per_hit_damage)
                    hp_damage = per_hit_damage - blocked
                    enemy.block -= blocked
                    enemy.hp -= hp_damage

                    state.total_damage_dealt += hp_damage

                    if enemy.hp <= 0:
                        enemy.hp = 0

        # Block
        if card.block > 0:
            block_gained = self._calculate_block_gained(state, card.block)
            state.player.block += block_gained

        # Stance changes
        if card.enter_stance:
            state = self._change_stance(state, self._get_stance_id(card.enter_stance))

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

    def _calculate_card_damage(
        self,
        state: CombatState,
        base_damage: int,
        target_index: int,
    ) -> int:
        """Calculate damage for a card attack."""
        # Get player modifiers
        strength = state.player.statuses.get("Strength", 0)
        vigor = state.player.statuses.get("Vigor", 0)
        weak = state.player.statuses.get("Weak", 0) > 0 or state.player.statuses.get("Weakened", 0) > 0

        # Get stance multiplier
        stance_id = self._get_stance_id(state.stance)
        stance_effect = STANCES[stance_id]
        stance_mult = stance_effect.damage_give_multiplier

        # Get target modifiers
        vuln = False
        if target_index < len(state.enemies):
            enemy = state.enemies[target_index]
            vuln = enemy.statuses.get("Vulnerable", 0) > 0

        return calculate_damage(
            base=base_damage,
            strength=strength,
            vigor=vigor,
            weak=weak,
            stance_mult=stance_mult,
            vuln=vuln,
        )

    def _calculate_block_gained(
        self,
        state: CombatState,
        base_block: int,
    ) -> int:
        """Calculate block gained from a card."""
        dexterity = state.player.statuses.get("Dexterity", 0)
        frail = state.player.statuses.get("Frail", 0) > 0

        return calculate_block(
            base=base_block,
            dexterity=dexterity,
            frail=frail,
        )

    def _change_stance(
        self,
        state: CombatState,
        new_stance: StanceID,
    ) -> CombatState:
        """Change stance and handle effects."""
        old_stance_str = state.stance
        old_stance = self._get_stance_id(old_stance_str)

        if old_stance == new_stance:
            return state

        # Exit effects
        if old_stance == StanceID.CALM:
            # Gain 2 energy base, +1 if Violet Lotus
            energy_gain = 2
            if self._has_violet_lotus(state):
                energy_gain += 1
            state.energy += energy_gain

        # Enter effects
        if new_stance == StanceID.DIVINITY:
            state.energy += 3

        state.stance = new_stance.value if hasattr(new_stance, 'value') else str(new_stance)

        # Mental Fortress trigger
        mental_fortress = state.player.statuses.get("MentalFortress", 0)
        if mental_fortress > 0:
            state.player.block += mental_fortress

        # Rushdown trigger
        rushdown = state.player.statuses.get("Rushdown", 0)
        if new_stance == StanceID.WRATH and rushdown > 0:
            state = self._draw_cards(state, rushdown)

        # Flurry of Blows trigger
        flurries = [(i, card_id) for i, card_id in enumerate(state.discard_pile)
                    if "FlurryOfBlows" in card_id]
        for i, card_id in reversed(flurries):
            state.discard_pile.pop(i)
            state.hand.append(card_id)

        return state

    def _add_mantra(self, state: CombatState, amount: int) -> CombatState:
        """Add mantra and potentially enter Divinity."""
        state.mantra += amount

        if state.mantra >= 10:
            state.mantra -= 10
            state = self._change_stance(state, StanceID.DIVINITY)

        return state

    def _apply_power_card(self, state: CombatState, card: Card) -> CombatState:
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
            current = state.player.statuses.get(power_id, 0)
            state.player.statuses[power_id] = current + amount

        return state

    def _end_player_turn(self, state: CombatState) -> CombatState:
        """End the player's turn."""
        state = state.copy()

        # Discard hand (unless Runic Pyramid)
        if not self._has_runic_pyramid(state):
            retained = []
            for card_id in state.hand:
                card = self._get_card(card_id)
                if card.retain:
                    retained.append(card_id)
                elif card.ethereal:
                    state.exhaust_pile.append(card_id)
                else:
                    state.discard_pile.append(card_id)
            state.hand = retained

        # Like Water
        like_water = state.player.statuses.get("LikeWater", 0)
        if like_water > 0 and state.stance == "Calm":
            state.player.block += like_water

        # Divinity auto-exit
        if state.stance == "Divinity" or state.stance == StanceID.DIVINITY.value:
            state = self._change_stance(state, StanceID.NEUTRAL)

        # Process enemy turns
        state = self.simulate_enemy_turn(state)

        if state.combat_over:
            return state

        # Start next turn
        state = self.simulate_turn_start(state)

        return state

    def simulate_enemy_turn(self, state: CombatState) -> CombatState:
        """Execute all enemy actions."""
        state = state.copy()

        for enemy in state.enemies:
            if not enemy.is_alive():
                continue

            move = get_enemy_move(enemy)
            if not move:
                continue

            # Apply strength to damage
            enemy_strength = enemy.strength

            # Execute attack moves
            if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
                base_damage = move.base_damage + enemy_strength
                hits = move.hits

                # Calculate damage with player's stance
                stance_id = self._get_stance_id(state.stance)
                stance_effect = STANCES[stance_id]
                damage_mult = stance_effect.damage_receive_multiplier

                for _ in range(hits):
                    damage = int(base_damage * damage_mult)

                    # Apply block
                    blocked = min(state.player.block, damage)
                    hp_damage = damage - blocked
                    state.player.block -= blocked
                    state.player.hp -= hp_damage

                    state.total_damage_taken += hp_damage

                    if state.player.hp <= 0:
                        state.player.hp = 0
                        state.combat_over = True
                        state.player_won = False
                        return state

            # Enemy block
            if move.block > 0:
                enemy.block += move.block

            # Enemy buffs/debuffs
            if move.effects:
                if "strength" in move.effects:
                    enemy.statuses["Strength"] = enemy.statuses.get("Strength", 0) + move.effects["strength"]

                if "weak" in move.effects:
                    current = state.player.statuses.get("Weakened", 0)
                    state.player.statuses["Weakened"] = current + move.effects["weak"]

                if "vulnerable" in move.effects:
                    current = state.player.statuses.get("Vulnerable", 0)
                    state.player.statuses["Vulnerable"] = current + move.effects["vulnerable"]

                if "frail" in move.effects:
                    current = state.player.statuses.get("Frail", 0)
                    state.player.statuses["Frail"] = current + move.effects["frail"]

        # Decrement player debuffs
        for debuff in ["Weakened", "Vulnerable", "Frail"]:
            if state.player.statuses.get(debuff, 0) > 0:
                state.player.statuses[debuff] -= 1
                if state.player.statuses[debuff] <= 0:
                    del state.player.statuses[debuff]

        # Enemy block decay
        for enemy in state.enemies:
            enemy.block = 0

        # Roll next moves
        for enemy in state.enemies:
            if enemy.is_alive():
                self._roll_enemy_move(state, enemy)

        return state

    def _roll_enemy_move(self, state: CombatState, enemy: CoreEnemyCombatState):
        """Roll next move for an enemy using deterministic logic."""
        # Simplified move rolling - uses state for determinism
        # In practice, would need full enemy AI logic
        roll = (state.ai_rng_state[0] + state.turn * 17 + hash(enemy.id)) % 100

        # Simple pattern: alternate between attack and other moves
        if len(enemy.move_history) == 0 or enemy.move_history[-1] != 1:
            # Attack move
            move = MoveInfo(
                move_id=1,
                name="Attack",
                intent=Intent.ATTACK,
                base_damage=6,
                hits=1,
            )
        else:
            # Other move
            move = MoveInfo(
                move_id=2,
                name="Buff",
                intent=Intent.BUFF,
                effects={"strength": 1},
            )

        set_enemy_move(enemy, move)
        enemy.move_history.append(move.move_id)

    def simulate_turn_end(self, state: CombatState) -> CombatState:
        """Process end of turn: discard hand, tick statuses, etc."""
        return self._end_player_turn(state)

    def simulate_turn_start(self, state: CombatState) -> CombatState:
        """Process start of turn: draw cards, reset energy, etc."""
        state = state.copy()

        state.turn += 1

        # Reset energy
        state.energy = state.max_energy

        # Lose block (unless Barricade)
        if not self._has_barricade(state):
            state.player.block = 0

        # Reset turn counters
        state.cards_played_this_turn = 0
        state.attacks_played_this_turn = 0
        state.last_card_type = ""

        # Draw cards
        draw_count = 5
        no_draw = state.player.statuses.get("NoDraw", 0) > 0 or state.player.statuses.get("No Draw", 0) > 0
        if no_draw:
            draw_count = 0

        state = self._draw_cards(state, draw_count)

        # Deva Form energy
        deva_form = state.player.statuses.get("DevaForm", 0)
        if deva_form > 0:
            state.energy += deva_form
            state.player.statuses["DevaForm"] = deva_form + 1

        # Devotion mantra
        devotion = state.player.statuses.get("Devotion", 0)
        if devotion > 0:
            state = self._add_mantra(state, devotion)

        return state

    def _check_combat_end(self, state: CombatState) -> CombatState:
        """Check if combat should end."""
        if state.all_enemies_dead():
            state.combat_over = True
            state.player_won = True
        elif state.player.is_dead:
            state.combat_over = True
            state.player_won = False

        return state

    def simulate_full_combat(
        self,
        state: CombatState,
        policy: Callable[[CombatState], Action],
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
        initial_hp = state.player.hp

        while not state.combat_over and state.turn <= max_turns:
            actions = self.get_legal_actions(state)
            if not actions:
                break

            action = policy(state)

            # Track card plays
            if action.action_type == ActionType.PLAY_CARD:
                card_id = state.hand[action.card_index]
                card = self._get_card(card_id)
                cards_played_sequence.append(card.id)
                energy_spent += card.current_cost

            old_stance = state.stance
            state = self.execute_action(state, action)

            if state.stance != old_stance:
                stance_changes += 1

        return CombatResult(
            victory=state.player_won,
            hp_remaining=state.player.hp,
            hp_lost=initial_hp - state.player.hp + state.total_damage_taken,
            turns=state.turn,
            cards_played=state.total_cards_played,
            damage_dealt=state.total_damage_dealt,
            damage_taken=state.total_damage_taken,
            cards_played_sequence=cards_played_sequence,
            stance_changes=stance_changes,
            energy_spent=energy_spent,
        )

    def random_policy(self, state: CombatState) -> Action:
        """Random legal action - baseline policy."""
        import random
        actions = self.get_legal_actions(state)
        return random.choice(actions) if actions else Action(ActionType.END_TURN)

    def greedy_policy(self, state: CombatState) -> Action:
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
        total_enemy_hp = sum(e.hp for e in state.enemies if e.is_alive())

        for action in actions:
            if action.action_type == ActionType.END_TURN:
                # End turn has base score of 0
                score = 0
            else:
                card_id = state.hand[action.card_index]
                card = self._get_card(card_id)

                # Calculate damage output
                damage = 0
                if card.damage > 0:
                    hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
                    damage = self._calculate_card_damage(state, card.damage, action.target_index) * hits

                # Calculate block
                block = 0
                if card.block > 0:
                    block = self._calculate_block_gained(state, card.block)

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
                        1 for card_id in state.hand
                        if self._get_card(card_id).card_type == CardType.ATTACK
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

    def _estimate_incoming_damage(self, state: CombatState) -> int:
        """Estimate total incoming damage from enemies."""
        total = 0

        for enemy in state.enemies:
            if not enemy.is_alive():
                continue

            move = get_enemy_move(enemy)
            if not move:
                continue

            if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
                damage = move.base_damage + enemy.strength

                # Apply stance multiplier
                stance_id = self._get_stance_id(state.stance)
                stance_effect = STANCES[stance_id]
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
    from ..content.enemies import JawWorm
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
    print(f"  Player HP: {state.player.hp}/{state.player.max_hp}")
    print(f"  Energy: {state.energy}")
    print(f"  Hand: {state.hand}")
    print(f"  Enemy: {state.enemies[0].name} - {state.enemies[0].hp} HP")
    move = get_enemy_move(state.enemies[0])
    print(f"  Enemy intent: {move.intent.value if move else 'None'}")

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
