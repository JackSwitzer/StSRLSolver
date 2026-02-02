"""
Calculation utilities for Slay the Spire combat simulation.

Contains:
- Damage calculation (pure functions, no side effects)
- Combat simulation (fight simulator with policies)
"""

from .damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    wrath_damage,
    divinity_damage,
    # Constants
    WEAK_MULT,
    WEAK_MULT_PAPER_CRANE,
    VULN_MULT,
    VULN_MULT_ODD_MUSHROOM,
    VULN_MULT_PAPER_FROG,
    FRAIL_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
    FLIGHT_MULT,
)

from .combat_sim import (
    CombatSimulator,
    CombatResult,
    Action,
    ActionType,
    encode_card_id,
    decode_card_id,
    set_enemy_move,
    get_enemy_move,
)

# Re-export CombatState from canonical location for backwards compatibility
from ..state.combat import CombatState, EnemyCombatState, EntityState

# Backwards compatibility aliases
SimCombatState = CombatState  # Alias for legacy code
SimEnemyState = EnemyCombatState  # Alias for legacy code

__all__ = [
    # Damage calculation
    "calculate_damage",
    "calculate_block",
    "calculate_incoming_damage",
    "apply_hp_loss",
    "wrath_damage",
    "divinity_damage",
    # Constants
    "WEAK_MULT",
    "WEAK_MULT_PAPER_CRANE",
    "VULN_MULT",
    "VULN_MULT_ODD_MUSHROOM",
    "VULN_MULT_PAPER_FROG",
    "FRAIL_MULT",
    "WRATH_MULT",
    "DIVINITY_MULT",
    "FLIGHT_MULT",
    # Combat simulation
    "CombatSimulator",
    "CombatState",
    "SimCombatState",  # Backwards compatibility alias
    "CombatResult",
    "Action",
    "ActionType",
    "EnemyCombatState",
    "EntityState",
    "SimEnemyState",  # Backwards compatibility alias
    # Card ID helpers
    "encode_card_id",
    "decode_card_id",
    "set_enemy_move",
    "get_enemy_move",
]
