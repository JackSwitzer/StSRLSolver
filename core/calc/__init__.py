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
    SimCombatState,
    CombatResult,
    Action,
    ActionType,
    PlayerCombatState,
    EnemyCombatState as SimEnemyState,
)

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
    "SimCombatState",
    "CombatResult",
    "Action",
    "ActionType",
    "PlayerCombatState",
    "SimEnemyState",
]
