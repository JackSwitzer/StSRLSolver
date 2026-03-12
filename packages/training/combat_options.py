"""
Pareto Frontier Search over combat strategies.

Explores multiple strategy weight configurations via LineSimulator and
returns the non-dominated set of combat options.  Each option represents
a different trade-off (aggressive, defensive, burst, etc.) so the
strategic planner can choose based on run-level context.
"""

from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple

from .combat_planner import CombatPlanner
from .line_evaluator import (
    LineOutcome,
    LineSimulator,
    SimulatedEnemy,
    SimulatedPlayer,
    simulate_from_engine,
)


@dataclass
class CombatOption:
    """A single combat strategy outcome."""
    hp_lost: int = 0
    turns: int = 99
    potions_used: List[str] = field(default_factory=list)
    cards_exhausted: List[str] = field(default_factory=list)
    relic_procs: Dict[str, Any] = field(default_factory=dict)
    final_stance: str = "Neutral"
    powers_in_play: List[str] = field(default_factory=list)
    infinite_detected: bool = False
    card_sequence: List[Tuple[str, Optional[int]]] = field(default_factory=list)
    score: float = 0.0
    strategy_name: str = "balanced"


# Strategy weight configurations for LineSimulator scoring
STRATEGY_VARIANTS: Dict[str, Dict[str, float]] = {
    "aggressive": {
        "damage_weight": 3.0,
        "block_weight": 0.5,
        "kill_bonus": 75.0,
    },
    "defensive": {
        "damage_weight": 0.5,
        "block_weight": 3.0,
        "kill_bonus": 30.0,
    },
    "balanced": {
        "damage_weight": 1.5,
        "block_weight": 1.5,
        "kill_bonus": 50.0,
    },
    "burst": {
        "damage_weight": 4.0,
        "block_weight": 0.2,
        "kill_bonus": 100.0,
    },
    "stance_preserve": {
        "damage_weight": 1.0,
        "block_weight": 2.0,
        "kill_bonus": 40.0,
    },
}


def _dominates(a: CombatOption, b: CombatOption) -> bool:
    """Return True if option *a* dominates option *b*.

    a dominates b iff a is <= b on hp_lost AND <= b on turns AND
    >= b on relic_procs count, with at least one strict inequality.
    """
    a_relics = len(a.relic_procs)
    b_relics = len(b.relic_procs)

    leq_hp = a.hp_lost <= b.hp_lost
    leq_turns = a.turns <= b.turns
    geq_relics = a_relics >= b_relics

    if not (leq_hp and leq_turns and geq_relics):
        return False

    # At least one strict inequality
    return (a.hp_lost < b.hp_lost or a.turns < b.turns or a_relics > b_relics)


def _build_option(
    outcome: LineOutcome,
    actions: List[Tuple[str, Optional[int]]],
    est_turns: int,
    strategy_name: str,
    player: SimulatedPlayer,
    enemies: List[SimulatedEnemy],
) -> CombatOption:
    """Build a CombatOption from a LineOutcome."""
    # Detect relic-relevant cards
    relic_procs: Dict[str, Any] = {}
    for card_id, _ in actions:
        base = card_id.rstrip("+")
        if base == "LessonLearned":
            relic_procs["LessonLearned"] = True
        elif base == "RitualDagger":
            relic_procs["RitualDagger"] = outcome.damage_dealt

    # Detect exhausted cards (simplified: cards that self-exhaust)
    _EXHAUST_CARDS = frozenset({
        "Blasphemy", "Omniscience", "Vault", "Wish",
        "LessonLearned", "RitualDagger", "Apparition",
        "TrueGrit", "Offering", "FeedPower",
    })
    cards_exhausted = [
        card_id for card_id, _ in actions
        if card_id.rstrip("+") in _EXHAUST_CARDS
    ]

    # Detect powers played
    from .line_evaluator import CARD_EFFECTS
    powers_in_play = [
        card_id for card_id, _ in actions
        if CARD_EFFECTS.get(card_id, {}).get("type") == "power"
    ]

    return CombatOption(
        hp_lost=outcome.damage_taken,
        turns=est_turns,
        cards_exhausted=cards_exhausted,
        relic_procs=relic_procs,
        final_stance=outcome.final_stance,
        powers_in_play=powers_in_play,
        card_sequence=list(actions),
        score=outcome.score,
        strategy_name=strategy_name,
    )


def find_pareto_options(
    engine: Any,
    combat_planner: Optional[CombatPlanner] = None,
) -> List[CombatOption]:
    """Explore multiple strategy weights and return non-dominated options.

    Args:
        engine: CombatEngine instance (not mutated).
        combat_planner: Optional planner for multi-turn evaluation.
            If None, a default one is created.

    Returns:
        List of non-dominated CombatOptions sorted by score descending.
    """
    if combat_planner is None:
        combat_planner = CombatPlanner(top_k=5, lookahead_turns=2)

    player, enemies, hand = simulate_from_engine(engine)

    if not enemies:
        return []

    sim = LineSimulator()
    all_options: List[CombatOption] = []

    for strategy_name, weights in STRATEGY_VARIANTS.items():
        top_lines = sim.find_top_k_lines(
            player, enemies, hand,
            k=3,
            strategy_weights=weights,
        )

        for outcome, actions in top_lines:
            if outcome.we_die:
                continue

            # Estimate turns to kill
            if outcome.is_lethal:
                est_turns = 1
            else:
                total_enemy_hp = sum(e.hp for e in enemies)
                avg_dmg = max(outcome.damage_dealt, 5)
                est_turns = max(1, (total_enemy_hp + avg_dmg - 1) // avg_dmg)

            option = _build_option(
                outcome, actions, est_turns, strategy_name, player, enemies,
            )
            all_options.append(option)

    if not all_options:
        return []

    # Pareto filter: remove dominated options
    pareto: List[CombatOption] = []
    for candidate in all_options:
        dominated = False
        for other in all_options:
            if other is candidate:
                continue
            if _dominates(other, candidate):
                dominated = True
                break
        if not dominated:
            pareto.append(candidate)

    # Deduplicate by card sequence (same sequence from different strategies)
    seen_sequences: set = set()
    unique: List[CombatOption] = []
    for opt in pareto:
        key = tuple(opt.card_sequence)
        if key not in seen_sequences:
            seen_sequences.add(key)
            unique.append(opt)

    # Sort by score descending
    unique.sort(key=lambda o: -o.score)
    return unique
