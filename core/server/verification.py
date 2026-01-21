"""
Verification system for detecting and logging simulation mismatches.

Compares predicted Python simulation results with actual Java game state
to identify bugs and improve simulation accuracy.
"""

from __future__ import annotations

import json
import logging
import os
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple

from ..state.combat import CombatState
from .state_converter import json_to_combat_state, compare_states


logger = logging.getLogger("Verification")


# =============================================================================
# Mismatch Types
# =============================================================================


@dataclass
class FieldMismatch:
    """A single field mismatch between predicted and actual state."""

    field: str
    predicted: Any
    actual: Any
    diff: Optional[float] = None
    diagnosis: str = ""
    probable_cause: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return {
            "field": self.field,
            "predicted": self.predicted,
            "actual": self.actual,
            "diff": self.diff,
            "diagnosis": self.diagnosis,
            "probable_cause": self.probable_cause,
        }


@dataclass
class VerificationResult:
    """Result of a state verification."""

    matches: bool
    mismatches: List[FieldMismatch] = field(default_factory=list)
    action_context: Optional[Dict[str, Any]] = None
    timestamp: float = 0.0

    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "matches": self.matches,
            "mismatches": [m.to_dict() for m in self.mismatches],
            "action_context": self.action_context,
            "timestamp": self.timestamp,
        }


# =============================================================================
# Verification Engine
# =============================================================================


class VerificationEngine:
    """
    Engine for verifying simulation accuracy.

    Tracks mismatches over time and provides diagnostic information
    for improving the Python simulation.
    """

    def __init__(self, log_dir: Optional[str] = None):
        """
        Initialize verification engine.

        Args:
            log_dir: Directory for mismatch logs (default: ~/Desktop/SlayTheSpireRL/logs/verification)
        """
        self.log_dir = log_dir or os.path.expanduser(
            "~/Desktop/SlayTheSpireRL/logs/verification"
        )
        os.makedirs(self.log_dir, exist_ok=True)

        # Statistics
        self.total_verifications = 0
        self.total_mismatches = 0
        self.mismatch_by_field: Dict[str, int] = {}
        self.mismatch_history: List[VerificationResult] = []

    def verify(
        self,
        predicted: CombatState,
        actual: Dict[str, Any],
        action_context: Optional[Dict[str, Any]] = None,
    ) -> VerificationResult:
        """
        Verify predicted state matches actual state.

        Args:
            predicted: Python-predicted CombatState
            actual: Java-observed state as JSON dict
            action_context: Optional context about what action was taken

        Returns:
            VerificationResult with match status and any mismatches
        """
        self.total_verifications += 1

        # Convert actual to CombatState for comparison
        actual_state = json_to_combat_state(actual)

        mismatches = []

        # Compare player state
        mismatches.extend(self._compare_player(predicted.player, actual_state.player))

        # Compare energy
        if predicted.energy != actual_state.energy:
            mismatches.append(FieldMismatch(
                field="energy",
                predicted=predicted.energy,
                actual=actual_state.energy,
                diff=actual_state.energy - predicted.energy,
                diagnosis="Energy mismatch",
                probable_cause=self._diagnose_energy_mismatch(
                    predicted, actual_state, action_context
                ),
            ))

        # Compare stance
        if predicted.stance != actual_state.stance:
            mismatches.append(FieldMismatch(
                field="stance",
                predicted=predicted.stance,
                actual=actual_state.stance,
                diagnosis="Stance mismatch",
                probable_cause=self._diagnose_stance_mismatch(
                    predicted, actual_state, action_context
                ),
            ))

        # Compare enemies
        for i, (pred_enemy, actual_enemy) in enumerate(
            zip(predicted.enemies, actual_state.enemies)
        ):
            mismatches.extend(
                self._compare_enemy(pred_enemy, actual_enemy, i)
            )

        # Compare hand size (cards are trickier due to draw randomness)
        if len(predicted.hand) != len(actual_state.hand):
            mismatches.append(FieldMismatch(
                field="hand.length",
                predicted=len(predicted.hand),
                actual=len(actual_state.hand),
                diff=len(actual_state.hand) - len(predicted.hand),
                diagnosis="Hand size mismatch",
                probable_cause="Draw/discard effect not correctly modeled",
            ))

        # Track statistics
        if mismatches:
            self.total_mismatches += 1
            for m in mismatches:
                self.mismatch_by_field[m.field] = (
                    self.mismatch_by_field.get(m.field, 0) + 1
                )

        result = VerificationResult(
            matches=len(mismatches) == 0,
            mismatches=mismatches,
            action_context=action_context,
        )

        # Log if mismatch
        if not result.matches:
            self._log_mismatch(result, predicted, actual)
            self.mismatch_history.append(result)

        return result

    def _compare_player(
        self,
        predicted,
        actual,
    ) -> List[FieldMismatch]:
        """Compare player states."""
        mismatches = []

        if predicted.hp != actual.hp:
            diff = actual.hp - predicted.hp
            mismatches.append(FieldMismatch(
                field="player.hp",
                predicted=predicted.hp,
                actual=actual.hp,
                diff=diff,
                diagnosis=f"Player HP off by {abs(diff)}",
                probable_cause=self._diagnose_hp_mismatch(diff),
            ))

        if predicted.block != actual.block:
            diff = actual.block - predicted.block
            mismatches.append(FieldMismatch(
                field="player.block",
                predicted=predicted.block,
                actual=actual.block,
                diff=diff,
                diagnosis=f"Block off by {abs(diff)}",
                probable_cause=self._diagnose_block_mismatch(diff),
            ))

        # Compare key statuses
        for status in ["Strength", "Dexterity", "Weak", "Vulnerable", "Frail"]:
            pred_val = predicted.statuses.get(status, 0)
            actual_val = actual.statuses.get(status, 0)
            if pred_val != actual_val:
                mismatches.append(FieldMismatch(
                    field=f"player.{status}",
                    predicted=pred_val,
                    actual=actual_val,
                    diff=actual_val - pred_val,
                    diagnosis=f"{status} mismatch",
                    probable_cause=f"{status} application not correctly modeled",
                ))

        return mismatches

    def _compare_enemy(
        self,
        predicted,
        actual,
        index: int,
    ) -> List[FieldMismatch]:
        """Compare enemy states."""
        mismatches = []

        if predicted.hp != actual.hp:
            diff = actual.hp - predicted.hp
            mismatches.append(FieldMismatch(
                field=f"enemies[{index}].hp",
                predicted=predicted.hp,
                actual=actual.hp,
                diff=diff,
                diagnosis=f"Enemy {predicted.id} HP off by {abs(diff)}",
                probable_cause=self._diagnose_enemy_hp_mismatch(diff, predicted),
            ))

        if predicted.block != actual.block:
            diff = actual.block - predicted.block
            mismatches.append(FieldMismatch(
                field=f"enemies[{index}].block",
                predicted=predicted.block,
                actual=actual.block,
                diff=diff,
                diagnosis=f"Enemy {predicted.id} block off by {abs(diff)}",
                probable_cause="Enemy block not correctly calculated",
            ))

        return mismatches

    def _diagnose_hp_mismatch(self, diff: int) -> str:
        """Diagnose player HP mismatch."""
        if diff > 0:
            # Player has more HP than predicted (took less damage)
            return (
                "Player took less damage than predicted. Check: "
                "Block calculation, enemy Weak, Intangible, damage mitigation relics"
            )
        else:
            # Player has less HP (took more damage)
            return (
                "Player took more damage than predicted. Check: "
                "Vulnerable, Wrath stance (2x damage taken), enemy Strength"
            )

    def _diagnose_block_mismatch(self, diff: int) -> str:
        """Diagnose block mismatch."""
        if diff > 0:
            return (
                "Player has more block than predicted. Check: "
                "Dexterity application, block-gaining relics, end-of-turn block retention"
            )
        else:
            return (
                "Player has less block than predicted. Check: "
                "Frail (25% reduction), block-losing effects, multi-hit attacks"
            )

    def _diagnose_energy_mismatch(
        self,
        predicted: CombatState,
        actual: CombatState,
        action_context: Optional[Dict],
    ) -> str:
        """Diagnose energy mismatch."""
        diff = actual.energy - predicted.energy

        if predicted.stance == "Calm" and actual.stance != "Calm":
            return "Calm exit grants +2 energy (Violet Lotus: +3)"

        if actual.stance == "Divinity":
            return "Divinity grants +3 energy on enter"

        if diff > 0:
            return (
                "More energy than predicted. Check: "
                "Calm exit, Divinity, energy-gaining cards/relics"
            )
        else:
            return (
                "Less energy than predicted. Check: "
                "Card cost calculations, X-cost cards, cost modifiers"
            )

    def _diagnose_stance_mismatch(
        self,
        predicted: CombatState,
        actual: CombatState,
        action_context: Optional[Dict],
    ) -> str:
        """Diagnose stance mismatch."""
        return (
            f"Stance changed from {predicted.stance} to {actual.stance}. "
            "Check: Stance change card effects, Flurry of Blows triggers"
        )

    def _diagnose_enemy_hp_mismatch(self, diff: int, enemy) -> str:
        """Diagnose enemy HP mismatch."""
        if diff > 0:
            # Enemy has more HP (took less damage)
            return (
                f"Enemy {enemy.id} took less damage. Check: "
                "Player Strength application, Weak (25% reduction), "
                "multi-hit damage calculation"
            )
        else:
            # Enemy has less HP (took more damage)
            return (
                f"Enemy {enemy.id} took more damage. Check: "
                "Vulnerable (50% more), Wrath stance (2x), "
                "Pen Nib, damage-amplifying relics"
            )

    def _log_mismatch(
        self,
        result: VerificationResult,
        predicted: CombatState,
        actual: Dict[str, Any],
    ):
        """Log mismatch to file."""
        timestamp = int(result.timestamp)
        filename = f"mismatch_{self.total_verifications:04d}_{timestamp}.json"
        filepath = os.path.join(self.log_dir, filename)

        # Serialize predicted state
        predicted_dict = {
            "player": {
                "hp": predicted.player.hp,
                "max_hp": predicted.player.max_hp,
                "block": predicted.player.block,
                "statuses": predicted.player.statuses,
            },
            "energy": predicted.energy,
            "stance": predicted.stance,
            "hand": predicted.hand,
            "enemies": [
                {
                    "id": e.id,
                    "hp": e.hp,
                    "block": e.block,
                    "statuses": e.statuses,
                }
                for e in predicted.enemies
            ],
        }

        log_data = {
            "verification_id": self.total_verifications,
            "timestamp": timestamp,
            "result": result.to_dict(),
            "predicted_state": predicted_dict,
            "actual_state": actual,
        }

        try:
            with open(filepath, "w") as f:
                json.dump(log_data, f, indent=2)
            logger.info(f"Mismatch logged: {filepath}")
        except Exception as e:
            logger.error(f"Failed to log mismatch: {e}")

    def get_stats(self) -> Dict[str, Any]:
        """Get verification statistics."""
        accuracy = (
            (self.total_verifications - self.total_mismatches) / self.total_verifications
            if self.total_verifications > 0 else 1.0
        )

        return {
            "total_verifications": self.total_verifications,
            "total_mismatches": self.total_mismatches,
            "accuracy": accuracy,
            "mismatch_by_field": self.mismatch_by_field,
            "most_common_mismatch": (
                max(self.mismatch_by_field.items(), key=lambda x: x[1])[0]
                if self.mismatch_by_field else None
            ),
        }

    def get_diagnosis_report(self) -> str:
        """Generate a diagnosis report for common mismatches."""
        stats = self.get_stats()

        lines = [
            "=== Verification Diagnosis Report ===",
            f"Total verifications: {stats['total_verifications']}",
            f"Accuracy: {stats['accuracy']:.1%}",
            "",
            "Mismatch frequency by field:",
        ]

        if self.mismatch_by_field:
            sorted_fields = sorted(
                self.mismatch_by_field.items(),
                key=lambda x: x[1],
                reverse=True,
            )
            for field, count in sorted_fields:
                lines.append(f"  {field}: {count}")
        else:
            lines.append("  (none)")

        lines.append("")
        lines.append("Recent mismatches:")

        for result in self.mismatch_history[-5:]:
            for m in result.mismatches[:2]:
                lines.append(f"  - {m.field}: {m.diagnosis}")
                lines.append(f"    Cause: {m.probable_cause}")

        return "\n".join(lines)


# =============================================================================
# Singleton Instance
# =============================================================================


_verification_engine: Optional[VerificationEngine] = None


def get_verification_engine() -> VerificationEngine:
    """Get the global verification engine instance."""
    global _verification_engine
    if _verification_engine is None:
        _verification_engine = VerificationEngine()
    return _verification_engine
