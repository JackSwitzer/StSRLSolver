"""
Combat Planner: Two-level search over turn outcomes.

Level 1 (Turn Solver): Uses LineSimulator to find top K card sequences for this turn.
Level 2 (Multi-turn Lookahead): Light search over turn outcomes, looking 2-3 turns ahead.

This replaces per-card MCTS with per-TURN planning:
- Root = current combat state
- Actions = top K turn sequences from LineSimulator
- Children = state after executing turn sequence + enemy attacks
- Value = heuristic (HP preserved, enemy HP destroyed, turns to kill)

Branching: ~3-5 turn outcomes per level (vs 10-50 card plays in MCTS)
Speed target: <200ms per combat turn decision.
"""

from __future__ import annotations

import time
from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple

from .line_evaluator import (
    CARD_EFFECTS,
    LineOutcome,
    LineSimulator,
    SimulatedEnemy,
    SimulatedPlayer,
    simulate_from_engine,
)


@dataclass
class TurnPlan:
    """Complete plan for a single combat turn."""
    # Card sequence to execute: [(card_id, target_idx), ...]
    card_sequence: List[Tuple[str, Optional[int]]] = field(default_factory=list)
    # Expected outcome from line simulation
    expected_outcome: Optional[LineOutcome] = None
    # Multi-turn estimates
    turns_to_kill: int = 99
    expected_hp_loss: int = 0  # Total HP lost across all turns
    confidence: float = 0.0   # How reliable the estimate (0-1)
    # Planning metadata
    lines_considered: int = 0
    planning_ms: float = 0.0
    strategy: str = "balanced"


@dataclass
class CombatEvaluation:
    """Estimated outcome of an entire combat."""
    total_hp_loss: int = 0
    turns_to_kill: int = 99
    can_win: bool = True
    lethal_this_turn: bool = False
    best_line_score: float = 0.0


class CombatPlanner:
    """
    Multi-turn lookahead planner using LineSimulator for turn-level search.

    Instead of searching individual card plays (MCTS), searches TURN OUTCOMES:
    1. For current turn, find top K lines via LineSimulator
    2. For top lines, simulate forward 1-2 more turns
    3. Score by total expected HP loss and time to kill
    """

    def __init__(
        self,
        top_k: int = 5,
        lookahead_turns: int = 2,
        strategy_weights: Optional[Dict[str, float]] = None,
    ):
        """
        Args:
            top_k: Number of top lines to consider per turn
            lookahead_turns: How many turns to look ahead (1-3)
            strategy_weights: Optional scoring weights from meta-learner
        """
        self.top_k = top_k
        self.lookahead_turns = min(lookahead_turns, 3)
        self.strategy_weights = strategy_weights
        self.sim = LineSimulator()

    def plan_turn(self, engine: Any) -> TurnPlan:
        """
        Find best action sequence for current turn with multi-turn lookahead.

        Args:
            engine: CombatEngine instance (not mutated).

        Returns:
            TurnPlan with card sequence and expected outcomes.
        """
        t0 = time.monotonic()

        player, enemies, hand = simulate_from_engine(engine)

        if not enemies:
            return TurnPlan(planning_ms=(time.monotonic() - t0) * 1000)

        # Level 1: Get top K lines for this turn
        top_lines = self.sim.find_top_k_lines(
            player, enemies, hand,
            k=self.top_k,
            strategy_weights=self.strategy_weights,
        )

        if not top_lines:
            return TurnPlan(planning_ms=(time.monotonic() - t0) * 1000)

        # Check for immediate lethal
        for outcome, actions in top_lines:
            if outcome.is_lethal:
                elapsed = (time.monotonic() - t0) * 1000
                return TurnPlan(
                    card_sequence=actions,
                    expected_outcome=outcome,
                    turns_to_kill=1,
                    expected_hp_loss=outcome.damage_taken,
                    confidence=0.95,
                    lines_considered=len(top_lines),
                    planning_ms=elapsed,
                    strategy="lethal",
                )

        # Level 2: Multi-turn lookahead for top candidates
        best_plan = None
        best_multi_score = float("-inf")

        for outcome, actions in top_lines:
            if outcome.we_die:
                continue

            # Score this turn + estimated future turns
            multi_score, est_turns, est_hp_loss = self._evaluate_multi_turn(
                player, enemies, hand, outcome, actions,
            )

            if multi_score > best_multi_score:
                best_multi_score = multi_score
                confidence = self._compute_confidence(outcome, est_turns, len(top_lines))
                best_plan = TurnPlan(
                    card_sequence=actions,
                    expected_outcome=outcome,
                    turns_to_kill=est_turns,
                    expected_hp_loss=est_hp_loss,
                    confidence=confidence,
                    lines_considered=len(top_lines),
                    strategy=self._classify_strategy(outcome),
                )

        elapsed = (time.monotonic() - t0) * 1000

        if best_plan is None:
            # Fallback: use highest-scored single-turn line
            outcome, actions = top_lines[0]
            best_plan = TurnPlan(
                card_sequence=actions,
                expected_outcome=outcome,
                turns_to_kill=99,
                expected_hp_loss=outcome.damage_taken,
                confidence=0.3,
                lines_considered=len(top_lines),
            )

        best_plan.planning_ms = elapsed
        return best_plan

    def evaluate_combat(self, engine: Any) -> CombatEvaluation:
        """
        Estimate total HP loss and turns to kill for this combat.

        Useful for strategic planner to score path choices.
        """
        player, enemies, hand = simulate_from_engine(engine)

        if not enemies:
            return CombatEvaluation(total_hp_loss=0, turns_to_kill=0, can_win=True)

        top_lines = self.sim.find_top_k_lines(player, enemies, hand, k=3)

        if not top_lines:
            return CombatEvaluation(can_win=False)

        best_outcome, best_actions = top_lines[0]

        if best_outcome.is_lethal:
            return CombatEvaluation(
                total_hp_loss=best_outcome.damage_taken,
                turns_to_kill=1,
                can_win=True,
                lethal_this_turn=True,
                best_line_score=best_outcome.score,
            )

        # Estimate turns to kill from damage rate
        total_enemy_hp = sum(e.hp for e in enemies)
        avg_damage = best_outcome.damage_dealt if best_outcome.damage_dealt > 0 else 5
        est_turns = max(1, (total_enemy_hp + avg_damage - 1) // avg_damage)

        # Estimate HP loss per turn
        avg_incoming = sum(
            e.intent_damage * e.intent_hits for e in enemies if e.is_attacking
        )
        avg_block_eff = 0.5  # rough estimate
        hp_per_turn = int(avg_incoming * (1 - avg_block_eff))
        est_hp_loss = hp_per_turn * est_turns

        can_win = est_hp_loss < player.hp

        return CombatEvaluation(
            total_hp_loss=est_hp_loss,
            turns_to_kill=est_turns,
            can_win=can_win,
            lethal_this_turn=False,
            best_line_score=best_outcome.score,
        )

    def _evaluate_multi_turn(
        self,
        player: SimulatedPlayer,
        enemies: List[SimulatedEnemy],
        hand: List[Dict],
        turn1_outcome: LineOutcome,
        turn1_actions: List[Tuple[str, Optional[int]]],
    ) -> Tuple[float, int, int]:
        """
        Evaluate a line choice by simulating forward turns.

        Returns: (multi_turn_score, estimated_turns_to_kill, estimated_total_hp_loss)
        """
        # Start with turn 1 score
        total_score = turn1_outcome.score
        total_hp_loss = turn1_outcome.damage_taken
        total_damage = turn1_outcome.damage_dealt

        # Estimate remaining enemy HP after turn 1
        remaining_hp = turn1_outcome.total_enemy_hp_remaining
        if remaining_hp <= 0:
            return total_score + 500, 1, total_hp_loss

        # Simulate future turns with simplified model
        sim_player = SimulatedPlayer(
            hp=turn1_outcome.player_hp,
            block=0,  # Block resets each turn
            energy=3,  # Assume base energy
            stance=turn1_outcome.final_stance,
            strength=player.strength,
            dexterity=player.dexterity,
        )

        # Calm exit gives +2 energy
        if sim_player.stance == "Calm":
            sim_player.energy += 2

        # Build approximate enemies for future turns
        future_enemies = []
        for eid, ehp in turn1_outcome.enemies_remaining:
            # Use original enemy data for intent estimation
            orig = next((e for e in enemies if e.id == eid), None)
            if orig:
                future_enemies.append(SimulatedEnemy(
                    id=eid, hp=ehp, max_hp=orig.max_hp, block=0,
                    intent_damage=orig.intent_damage, intent_hits=orig.intent_hits,
                    is_attacking=orig.is_attacking,
                    vulnerable=max(0, orig.vulnerable - 1),
                    weak=max(0, orig.weak - 1),
                ))

        if not future_enemies:
            return total_score + 500, 1, total_hp_loss

        # For future turns, we don't know exact hand - use average damage estimate
        avg_damage_per_turn = max(turn1_outcome.damage_dealt, 5)
        avg_incoming_per_turn = sum(
            e.intent_damage * e.intent_hits for e in future_enemies if e.is_attacking
        )

        turns = 1
        for _ in range(self.lookahead_turns):
            if remaining_hp <= 0:
                break

            turns += 1
            remaining_hp -= avg_damage_per_turn
            total_damage += avg_damage_per_turn

            # Estimate damage taken (rough: 50% blocked)
            unblocked = max(0, avg_incoming_per_turn - sim_player.dexterity * 2 - 5)
            hp_lost = max(0, unblocked)
            total_hp_loss += hp_lost
            sim_player.hp -= hp_lost

            if sim_player.hp <= 0:
                total_score -= 5000
                break

            # Remove dead enemies from damage pool
            for e in future_enemies:
                e.hp -= avg_damage_per_turn // max(len(future_enemies), 1)
            future_enemies = [e for e in future_enemies if e.hp > 0]
            avg_incoming_per_turn = sum(
                e.intent_damage * e.intent_hits for e in future_enemies if e.is_attacking
            )

        # Multi-turn scoring
        total_score += total_damage * 1.5
        total_score -= total_hp_loss * 4.0
        if remaining_hp <= 0:
            total_score += 300  # Kill bonus
            total_score += max(0, sim_player.hp) * 2  # HP preservation bonus

        # Prefer shorter fights
        total_score -= turns * 10

        return total_score, turns, total_hp_loss

    def _compute_confidence(
        self, outcome: LineOutcome, est_turns: int, lines_considered: int,
    ) -> float:
        """Estimate confidence in the plan (0-1)."""
        conf = 0.5

        # More lines considered = more explored
        if lines_considered >= 5:
            conf += 0.1

        # Short fights are more predictable
        if est_turns <= 2:
            conf += 0.2
        elif est_turns <= 4:
            conf += 0.1

        # Safe turns are confident
        if outcome.is_safe:
            conf += 0.15

        # Lethal is very confident
        if outcome.is_lethal:
            conf = 0.95

        return min(conf, 1.0)

    def _classify_strategy(self, outcome: LineOutcome) -> str:
        """Classify the chosen line's strategy."""
        if outcome.is_lethal:
            return "lethal"
        if outcome.damage_dealt > outcome.damage_taken * 3:
            return "aggressive"
        if outcome.is_safe and outcome.damage_dealt < 10:
            return "defensive"
        return "balanced"
