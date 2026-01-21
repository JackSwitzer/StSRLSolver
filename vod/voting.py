"""
Multi-pass voting engine for VOD extraction.

Aggregates decisions from multiple extraction passes and votes
on the most likely correct value for each decision point.
"""

from collections import Counter, defaultdict
from dataclasses import dataclass, field
from typing import Any, Optional
import hashlib
import json

from vod.state import DecisionLog, DecisionType


@dataclass
class VotedDecision:
    """A decision after voting across multiple passes."""
    decision_type: DecisionType
    floor: int
    timestamp: Optional[str]
    data: dict
    confidence: float
    votes: int
    total_passes: int
    disagreements: list[dict] = field(default_factory=list)

    @property
    def agreement_ratio(self) -> float:
        """What fraction of passes agreed on this decision."""
        return self.votes / self.total_passes if self.total_passes > 0 else 0.0

    def needs_review(self, threshold: float = 0.6) -> bool:
        """Check if this decision needs human review."""
        return self.confidence < threshold

    def to_decision_log(self) -> DecisionLog:
        """Convert back to DecisionLog format."""
        return DecisionLog(
            decision_type=self.decision_type,
            floor=self.floor,
            timestamp=self.timestamp,
            data=self.data,
            confidence=self.confidence,
            pass_number=0,  # Indicates voted result
        )


@dataclass
class VotingResult:
    """Complete result of voting across all decision points."""
    decisions: list[VotedDecision]
    overall_confidence: float
    flagged_for_review: list[VotedDecision]
    pass_count: int

    def get_decisions_by_type(self, dtype: DecisionType) -> list[VotedDecision]:
        return [d for d in self.decisions if d.decision_type == dtype]

    def to_dict(self) -> dict:
        return {
            "decisions": [
                {
                    "type": d.decision_type.value,
                    "floor": d.floor,
                    "timestamp": d.timestamp,
                    "data": d.data,
                    "confidence": d.confidence,
                    "votes": d.votes,
                    "total_passes": d.total_passes,
                }
                for d in self.decisions
            ],
            "overall_confidence": self.overall_confidence,
            "flagged_count": len(self.flagged_for_review),
            "pass_count": self.pass_count,
        }


class VotingEngine:
    """
    Aggregates decisions from multiple extraction passes and votes
    on the correct value for each decision point.

    Usage:
        engine = VotingEngine()
        engine.add_pass(decisions_from_pass_1)
        engine.add_pass(decisions_from_pass_2)
        engine.add_pass(decisions_from_pass_3)
        result = engine.vote()
    """

    def __init__(self, confidence_threshold: float = 0.5):
        self.passes: list[list[DecisionLog]] = []
        self.confidence_threshold = confidence_threshold

    def add_pass(self, decisions: list[DecisionLog]) -> None:
        """Add decisions from one extraction pass."""
        self.passes.append(decisions)

    def vote(self) -> VotingResult:
        """
        Vote on all decision points across passes.

        Returns aggregated decisions with confidence scores.
        """
        if not self.passes:
            return VotingResult(
                decisions=[],
                overall_confidence=0.0,
                flagged_for_review=[],
                pass_count=0,
            )

        # Group decisions by unique key
        decision_groups = self._group_decisions()

        # Vote on each group
        voted_decisions = []
        for key, group in decision_groups.items():
            voted = self._vote_on_group(key, group)
            voted_decisions.append(voted)

        # Sort by floor, then by decision order within floor
        voted_decisions.sort(key=lambda d: (d.floor, self._decision_order(d.decision_type)))

        # Calculate overall confidence
        if voted_decisions:
            overall_confidence = sum(d.confidence for d in voted_decisions) / len(voted_decisions)
        else:
            overall_confidence = 0.0

        # Find decisions that need review
        flagged = [d for d in voted_decisions if d.needs_review(self.confidence_threshold)]

        return VotingResult(
            decisions=voted_decisions,
            overall_confidence=overall_confidence,
            flagged_for_review=flagged,
            pass_count=len(self.passes),
        )

    def _group_decisions(self) -> dict[str, list[DecisionLog]]:
        """Group decisions by their unique key across all passes."""
        groups: dict[str, list[DecisionLog]] = defaultdict(list)

        for pass_decisions in self.passes:
            for decision in pass_decisions:
                key = self._decision_key(decision)
                groups[key].append(decision)

        return dict(groups)

    def _decision_key(self, decision: DecisionLog) -> str:
        """
        Generate a unique key for grouping equivalent decisions.

        Decisions are grouped by:
        - Floor number
        - Decision type
        - For decisions with options, a hash of sorted options
        """
        parts = [str(decision.floor), decision.decision_type.value]

        # For certain types, include options in the key to distinguish
        # different card rewards at the same floor
        if decision.decision_type in [DecisionType.CARD_REWARD, DecisionType.BOSS_RELIC]:
            options = decision.data.get("options", [])
            if options:
                opts_hash = hashlib.md5(
                    json.dumps(sorted(options), sort_keys=True).encode()
                ).hexdigest()[:8]
                parts.append(opts_hash)

        # For combat turns, include turn number
        if decision.decision_type == DecisionType.COMBAT_TURN:
            parts.append(str(decision.data.get("turn", 0)))

        return ":".join(parts)

    def _vote_on_group(self, key: str, decisions: list[DecisionLog]) -> VotedDecision:
        """Vote on a group of equivalent decisions to get consensus."""
        if len(decisions) == 1:
            # Only one pass had this decision
            d = decisions[0]
            return VotedDecision(
                decision_type=d.decision_type,
                floor=d.floor,
                timestamp=d.timestamp,
                data=d.data,
                confidence=d.confidence * 0.8,  # Lower confidence for single-pass
                votes=1,
                total_passes=len(self.passes),
            )

        decision_type = decisions[0].decision_type

        # Vote based on decision type
        if decision_type == DecisionType.CARD_REWARD:
            return self._vote_card_reward(decisions)
        elif decision_type == DecisionType.COMBAT_END:
            return self._vote_combat_end(decisions)
        elif decision_type == DecisionType.COMBAT_TURN:
            return self._vote_combat_turn(decisions)
        else:
            return self._vote_generic(decisions)

    def _vote_card_reward(self, decisions: list[DecisionLog]) -> VotedDecision:
        """Vote on card reward - majority wins on 'chosen' field."""
        chosen_votes: Counter[str] = Counter()
        for d in decisions:
            chosen = d.data.get("chosen", "SKIP")
            chosen_votes[chosen] += 1

        winner, votes = chosen_votes.most_common(1)[0]
        confidence = votes / len(decisions)

        # Use the data from the winning decision
        winning_decision = next(d for d in decisions if d.data.get("chosen") == winner)

        # Track disagreements
        disagreements = [
            {"chosen": d.data.get("chosen"), "pass": d.pass_number}
            for d in decisions
            if d.data.get("chosen") != winner
        ]

        return VotedDecision(
            decision_type=DecisionType.CARD_REWARD,
            floor=winning_decision.floor,
            timestamp=winning_decision.timestamp,
            data=winning_decision.data,
            confidence=confidence,
            votes=votes,
            total_passes=len(self.passes),
            disagreements=disagreements,
        )

    def _vote_combat_end(self, decisions: list[DecisionLog]) -> VotedDecision:
        """Vote on combat end - take median HP."""
        hp_values = [d.data.get("hp", 0) for d in decisions]
        hp_values.sort()

        # Take median
        mid = len(hp_values) // 2
        if len(hp_values) % 2 == 0:
            median_hp = (hp_values[mid - 1] + hp_values[mid]) // 2
        else:
            median_hp = hp_values[mid]

        # Find closest decision to median
        closest = min(decisions, key=lambda d: abs(d.data.get("hp", 0) - median_hp))

        # Calculate spread-based confidence
        if len(hp_values) > 1:
            spread = max(hp_values) - min(hp_values)
            # Lower confidence for higher spread
            confidence = max(0.5, 1.0 - (spread / 100))
        else:
            confidence = 0.8

        return VotedDecision(
            decision_type=DecisionType.COMBAT_END,
            floor=closest.floor,
            timestamp=closest.timestamp,
            data={**closest.data, "hp": median_hp},
            confidence=confidence,
            votes=len(decisions),
            total_passes=len(self.passes),
        )

    def _vote_combat_turn(self, decisions: list[DecisionLog]) -> VotedDecision:
        """Vote on combat turn - majority on card sequence."""
        # Serialize card sequences for comparison
        sequence_votes: Counter[str] = Counter()
        for d in decisions:
            cards = d.data.get("cards", [])
            seq_key = json.dumps(cards, sort_keys=True)
            sequence_votes[seq_key] += 1

        winner_key, votes = sequence_votes.most_common(1)[0]
        winner_cards = json.loads(winner_key)
        confidence = votes / len(decisions)

        # Use data from a winning decision
        winning_decision = next(
            d for d in decisions
            if json.dumps(d.data.get("cards", []), sort_keys=True) == winner_key
        )

        return VotedDecision(
            decision_type=DecisionType.COMBAT_TURN,
            floor=winning_decision.floor,
            timestamp=winning_decision.timestamp,
            data={**winning_decision.data, "cards": winner_cards},
            confidence=confidence,
            votes=votes,
            total_passes=len(self.passes),
        )

    def _vote_generic(self, decisions: list[DecisionLog]) -> VotedDecision:
        """Generic voting - serialize and compare full data."""
        data_votes: Counter[str] = Counter()
        for d in decisions:
            # Create comparable key from data (excluding timestamp)
            data_key = json.dumps(d.data, sort_keys=True)
            data_votes[data_key] += 1

        winner_key, votes = data_votes.most_common(1)[0]
        confidence = votes / len(decisions)

        # Find a decision with the winning data
        winning_decision = next(
            d for d in decisions
            if json.dumps(d.data, sort_keys=True) == winner_key
        )

        return VotedDecision(
            decision_type=winning_decision.decision_type,
            floor=winning_decision.floor,
            timestamp=winning_decision.timestamp,
            data=winning_decision.data,
            confidence=confidence,
            votes=votes,
            total_passes=len(self.passes),
        )

    def _decision_order(self, dtype: DecisionType) -> int:
        """Get ordering priority for decision types within a floor."""
        order = {
            DecisionType.SEED: 0,
            DecisionType.NEOW: 1,
            DecisionType.PATH: 2,
            DecisionType.COMBAT_START: 3,
            DecisionType.COMBAT_TURN: 4,
            DecisionType.COMBAT_END: 5,
            DecisionType.CARD_REWARD: 6,
            DecisionType.RELIC_REWARD: 7,
            DecisionType.POTION_REWARD: 8,
            DecisionType.SHOP: 9,
            DecisionType.REST: 10,
            DecisionType.EVENT: 11,
            DecisionType.BOSS_RELIC: 12,
            DecisionType.RESULT: 99,
        }
        return order.get(dtype, 50)


def vote_on_decisions(
    passes: list[list[DecisionLog]],
    threshold: float = 0.5,
) -> VotingResult:
    """
    Convenience function to vote on multiple extraction passes.

    Args:
        passes: List of decision lists, one per extraction pass
        threshold: Confidence threshold below which decisions are flagged

    Returns:
        VotingResult with aggregated decisions and confidence scores
    """
    engine = VotingEngine(confidence_threshold=threshold)
    for pass_decisions in passes:
        engine.add_pass(pass_decisions)
    return engine.vote()
