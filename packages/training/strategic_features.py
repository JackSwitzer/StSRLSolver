"""
Strategic feature extraction for STS decision making.

This extracts the high-level features that top players actually think about:
1. Lethal detection (can I kill? can I die?)
2. Resource efficiency (damage per energy, block per energy)
3. Deck cycling (when will I see my good cards again?)
4. Potion timing (is this the right moment?)
5. Risk assessment (what's the expected damage I take?)
6. Win condition progress (am I building toward infinite? scaling? front-loaded?)

These features REPLACE raw card/relic encodings - the model doesn't need to learn
that "Tantrum + Wrath = good damage", we just tell it the damage numbers.
"""

from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, field
import math


@dataclass
class StrategicState:
    """High-level strategic features for decision making."""

    # === LETHAL ANALYSIS ===
    can_lethal_all: bool = False           # Kill all enemies this turn
    can_lethal_priority: bool = False      # Kill the most dangerous enemy
    lethal_requires_wrath: bool = False    # Need wrath to kill
    lethal_requires_potion: bool = False   # Need potion to kill
    turns_to_kill: float = 1.0             # Expected turns to clear fight

    enemy_can_lethal: bool = False         # Enemy kills us if we don't block
    enemy_lethal_next_turn: bool = False   # Could die next turn
    guaranteed_safe: bool = False          # Can block all damage

    # === RESOURCE STATE ===
    energy: int = 3
    energy_next_turn: int = 3              # Including calm exit
    effective_energy: float = 3.0          # Energy + cards that give energy

    cards_in_hand: int = 5
    cards_playable: int = 5                # Have energy for
    hand_quality: float = 0.5              # 0-1, how good is this hand

    # === DAMAGE MATH ===
    max_damage_this_turn: int = 0
    max_damage_with_potions: int = 0
    expected_damage_dealt: float = 0       # Accounting for variance
    damage_efficiency: float = 0           # Damage per energy

    total_enemy_hp: int = 0
    priority_enemy_hp: int = 0             # Highest threat enemy

    # === DEFENSE MATH ===
    incoming_damage: int = 0
    max_block_this_turn: int = 0
    expected_damage_taken: float = 0       # After blocking
    block_efficiency: float = 0            # Block per energy

    hp_percentage: float = 1.0
    effective_hp: int = 80                 # HP + block

    # === STANCE STATE (Watcher) ===
    current_stance: str = "Neutral"
    can_enter_wrath: bool = False
    can_enter_calm: bool = False
    can_exit_stance: bool = False
    wrath_damage_bonus: int = 0            # Extra damage if we go wrath
    calm_energy_pending: int = 0           # Energy when we leave calm

    # === DECK CYCLE ===
    deck_size: int = 20
    draw_pile_size: int = 15
    discard_size: int = 5
    turns_until_reshuffle: float = 3.0
    key_cards_in_draw: int = 0             # Important cards coming up
    key_cards_in_discard: int = 0          # Important cards we've used

    # === SCALING/SETUP ===
    has_infinite_potential: bool = False   # Can go infinite this fight
    scaling_per_turn: float = 0            # Strength gain, etc.
    enemy_scaling_per_turn: float = 0      # Enemy getting stronger
    race_condition: bool = False           # Need to kill before enemy scales

    # === POTION STATE ===
    num_potions: int = 0
    has_damage_potion: bool = False
    has_block_potion: bool = False
    has_utility_potion: bool = False
    potion_value_now: float = 0            # How much would potion help
    should_potion: bool = False            # Is this the right moment

    # === FIGHT CONTEXT ===
    is_elite: bool = False
    is_boss: bool = False
    fight_turn: int = 1
    estimated_fight_length: int = 3
    floor_number: int = 1

    # === WIN CONDITION ===
    archetype: str = "unknown"             # "aggro", "block", "infinite", "scaling"
    on_track: bool = True                  # Executing our plan well


def extract_strategic_features(
    game_state: Any,  # Game object from spirecomm
    calculator: Any = None,  # CombatCalculator
) -> StrategicState:
    """Extract strategic features from game state."""

    state = StrategicState()

    if not game_state:
        return state

    # Basic stats
    state.hp_percentage = game_state.current_hp / max(game_state.max_hp, 1)
    state.effective_hp = game_state.current_hp + getattr(game_state, 'block', 0)
    state.floor_number = getattr(game_state, 'floor', 1)

    # Combat state
    if hasattr(game_state, 'combat_state') and game_state.combat_state:
        cs = game_state.combat_state

        # Energy
        state.energy = getattr(cs, 'player', {}).get('energy', 3) if hasattr(cs, 'player') else 3

        # Hand analysis
        hand = getattr(game_state, 'hand', [])
        state.cards_in_hand = len(hand)
        state.cards_playable = sum(1 for c in hand if getattr(c, 'cost', 99) <= state.energy)

        # Enemy analysis
        monsters = getattr(game_state, 'monsters', [])
        state.total_enemy_hp = sum(
            max(0, m.current_hp) for m in monsters
            if not getattr(m, 'is_gone', False) and not getattr(m, 'half_dead', False)
        )

        # Incoming damage
        state.incoming_damage = sum(
            getattr(m, 'move_adjusted_damage', 0) * max(getattr(m, 'move_hits', 1), 1)
            for m in monsters
            if getattr(m, 'intent', None) and 'ATTACK' in str(getattr(m, 'intent', ''))
        )

        # Lethal checks
        current_block = getattr(game_state, 'block', 0)
        damage_through = max(0, state.incoming_damage - current_block)
        state.enemy_can_lethal = damage_through >= game_state.current_hp
        state.guaranteed_safe = state.incoming_damage <= current_block

        # Fight context
        state.is_elite = any(
            'elite' in str(type(m).__name__).lower()
            for m in monsters
        )
        state.is_boss = getattr(game_state, 'floor', 0) in [16, 33, 50, 56]

    # Deck state
    state.deck_size = len(getattr(game_state, 'deck', []))
    state.draw_pile_size = getattr(game_state, 'draw_pile_count', 0) if hasattr(game_state, 'draw_pile_count') else 15
    state.discard_size = getattr(game_state, 'discard_pile_count', 0) if hasattr(game_state, 'discard_pile_count') else 0

    # Potions
    potions = getattr(game_state, 'potions', [])
    state.num_potions = len([p for p in potions if p and getattr(p, 'potion_id', '') != 'Potion Slot'])
    state.has_damage_potion = any(
        'fire' in getattr(p, 'potion_id', '').lower() or
        'attack' in getattr(p, 'potion_id', '').lower() or
        'explosive' in getattr(p, 'potion_id', '').lower()
        for p in potions if p
    )
    state.has_block_potion = any(
        'block' in getattr(p, 'potion_id', '').lower() or
        'ghost' in getattr(p, 'potion_id', '').lower()
        for p in potions if p
    )

    # Should use potion?
    state.should_potion = (
        state.enemy_can_lethal and state.has_block_potion
    ) or (
        state.is_boss and state.has_damage_potion
    )

    # Estimated turns to kill (very rough)
    if state.total_enemy_hp > 0:
        avg_damage_per_turn = 15  # rough estimate
        state.turns_to_kill = max(1, state.total_enemy_hp / avg_damage_per_turn)
        state.estimated_fight_length = int(state.turns_to_kill) + 1

    return state


def strategic_state_to_vector(state: StrategicState) -> List[float]:
    """Convert strategic state to neural network input vector."""
    return [
        # Lethal (6)
        float(state.can_lethal_all),
        float(state.can_lethal_priority),
        float(state.lethal_requires_wrath),
        float(state.lethal_requires_potion),
        min(state.turns_to_kill / 5, 1.0),
        float(state.enemy_can_lethal),

        # Resources (6)
        state.energy / 5,
        state.energy_next_turn / 5,
        state.cards_in_hand / 10,
        state.cards_playable / 10,
        state.hand_quality,
        state.effective_energy / 5,

        # Damage (6)
        state.max_damage_this_turn / 50,
        state.expected_damage_dealt / 50,
        state.damage_efficiency / 10,
        state.total_enemy_hp / 100,
        state.priority_enemy_hp / 50,
        min(state.turns_to_kill / 5, 1.0),

        # Defense (6)
        state.incoming_damage / 30,
        state.max_block_this_turn / 30,
        state.expected_damage_taken / 30,
        state.block_efficiency / 10,
        state.hp_percentage,
        state.effective_hp / 100,

        # Stance (6)
        float(state.current_stance == "Wrath"),
        float(state.current_stance == "Calm"),
        float(state.can_enter_wrath),
        float(state.can_enter_calm),
        state.wrath_damage_bonus / 30,
        state.calm_energy_pending / 3,

        # Deck (5)
        state.deck_size / 30,
        state.draw_pile_size / 30,
        state.discard_size / 30,
        min(state.turns_until_reshuffle / 5, 1.0),
        state.key_cards_in_draw / 5,

        # Potions (4)
        state.num_potions / 3,
        float(state.has_damage_potion),
        float(state.has_block_potion),
        float(state.should_potion),

        # Context (5)
        float(state.is_elite),
        float(state.is_boss),
        state.fight_turn / 10,
        state.floor_number / 57,
        float(state.race_condition),
    ]


# Feature dimension
STRATEGIC_FEATURE_DIM = 44


if __name__ == "__main__":
    # Test
    state = StrategicState(
        can_lethal_all=False,
        enemy_can_lethal=True,
        energy=2,
        cards_in_hand=4,
        total_enemy_hp=35,
        incoming_damage=18,
        hp_percentage=0.6,
        current_stance="Calm",
        can_enter_wrath=True,
        has_block_potion=True,
        should_potion=True,
        is_elite=True,
    )

    vec = strategic_state_to_vector(state)
    print(f"Strategic feature vector: {len(vec)} dimensions")
    print(f"Sample values: {vec[:10]}")
