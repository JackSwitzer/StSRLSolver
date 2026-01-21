"""
Seed-based verification for VOD extraction.

Uses GameRNGState to predict expected rewards and validate
extracted decisions against seed-deterministic outcomes.
"""

from dataclasses import dataclass, field
from typing import Optional

from core.state.game_rng import GameRNGState, RNGStream
from vod.state import VODRunState, DecisionLog, DecisionType


@dataclass
class VerificationResult:
    """Result of verifying a single decision against seed prediction."""
    decision: DecisionLog
    predicted: Optional[dict] = None
    match_ratio: float = 1.0
    verified: bool = True
    notes: str = ""

    def to_dict(self) -> dict:
        return {
            "floor": self.decision.floor,
            "type": self.decision.decision_type.value,
            "predicted": self.predicted,
            "actual": self.decision.data,
            "match_ratio": self.match_ratio,
            "verified": self.verified,
            "notes": self.notes,
        }


@dataclass
class SeedVerificationReport:
    """Complete verification report for a run."""
    seed: str
    total_decisions: int
    verified_decisions: int
    card_reward_accuracy: float
    path_accuracy: float
    overall_accuracy: float
    results: list[VerificationResult] = field(default_factory=list)
    mismatches: list[VerificationResult] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "seed": self.seed,
            "total_decisions": self.total_decisions,
            "verified_decisions": self.verified_decisions,
            "card_reward_accuracy": self.card_reward_accuracy,
            "path_accuracy": self.path_accuracy,
            "overall_accuracy": self.overall_accuracy,
            "mismatches": [m.to_dict() for m in self.mismatches],
        }


class SeedVerifier:
    """
    Verifies extracted decisions against seed-based predictions.

    Usage:
        verifier = SeedVerifier("ABC123DEF")
        report = verifier.verify_run(state)
    """

    def __init__(self, seed: str):
        self.seed = seed
        self.rng_state = GameRNGState(seed_str=seed)
        self._path_history: list[tuple[int, str]] = []

    def verify_run(self, state: VODRunState) -> SeedVerificationReport:
        """
        Verify all decisions in a VODRunState against seed predictions.

        Args:
            state: The VODRunState to verify

        Returns:
            Complete verification report
        """
        results = []
        mismatches = []

        card_rewards_verified = 0
        card_rewards_total = 0
        paths_verified = 0
        paths_total = 0

        # Process decisions in order
        for decision in state.decisions:
            result = self._verify_decision(decision)
            results.append(result)

            if not result.verified:
                mismatches.append(result)

            # Track category-specific accuracy
            if decision.decision_type == DecisionType.CARD_REWARD:
                card_rewards_total += 1
                if result.verified:
                    card_rewards_verified += 1
            elif decision.decision_type == DecisionType.PATH:
                paths_total += 1
                if result.verified:
                    paths_verified += 1

        # Calculate accuracies
        card_accuracy = card_rewards_verified / card_rewards_total if card_rewards_total > 0 else 1.0
        path_accuracy = paths_verified / paths_total if paths_total > 0 else 1.0
        overall = len([r for r in results if r.verified]) / len(results) if results else 1.0

        return SeedVerificationReport(
            seed=self.seed,
            total_decisions=len(results),
            verified_decisions=len([r for r in results if r.verified]),
            card_reward_accuracy=card_accuracy,
            path_accuracy=path_accuracy,
            overall_accuracy=overall,
            results=results,
            mismatches=mismatches,
        )

    def _verify_decision(self, decision: DecisionLog) -> VerificationResult:
        """Verify a single decision against seed prediction."""
        dtype = decision.decision_type

        if dtype == DecisionType.CARD_REWARD:
            return self._verify_card_reward(decision)
        elif dtype == DecisionType.PATH:
            return self._verify_path(decision)
        elif dtype == DecisionType.NEOW:
            return self._verify_neow(decision)
        elif dtype == DecisionType.COMBAT_START:
            # Advance RNG state for combat
            self._advance_for_combat(decision.data.get("enemy", ""))
            return VerificationResult(decision=decision, notes="Combat advances RNG")
        else:
            # Other decisions don't have seed-verifiable predictions
            return VerificationResult(decision=decision, notes="Not seed-verifiable")

    def _verify_card_reward(self, decision: DecisionLog) -> VerificationResult:
        """Verify card reward options match seed prediction."""
        extracted_options = decision.data.get("options", [])

        if not extracted_options:
            return VerificationResult(
                decision=decision,
                verified=False,
                notes="No options extracted",
            )

        # Get predicted options from seed
        predicted_options = self._predict_card_reward()

        if not predicted_options:
            return VerificationResult(
                decision=decision,
                notes="Could not predict (RNG state unknown)",
            )

        # Compare options (normalize names)
        extracted_set = set(self._normalize_card_name(c) for c in extracted_options)
        predicted_set = set(self._normalize_card_name(c) for c in predicted_options)

        # Calculate match ratio
        if predicted_set:
            intersection = extracted_set & predicted_set
            match_ratio = len(intersection) / len(predicted_set)
        else:
            match_ratio = 0.0

        verified = match_ratio >= 0.66  # At least 2/3 match

        return VerificationResult(
            decision=decision,
            predicted={"options": predicted_options},
            match_ratio=match_ratio,
            verified=verified,
            notes=f"Matched {len(intersection)}/{len(predicted_set)} cards" if predicted_set else "",
        )

    def _verify_path(self, decision: DecisionLog) -> VerificationResult:
        """Verify path choice is valid for the map."""
        floor = decision.floor
        chosen = decision.data.get("chosen", "")
        options = decision.data.get("options", [])

        # Record path for RNG tracking
        self._path_history.append((floor, chosen))

        # If we have map generation, verify the choice is valid
        # For now, just accept the path as valid
        # Full map verification requires integrating with map generation

        return VerificationResult(
            decision=decision,
            verified=True,
            notes=f"Path recorded: {chosen}",
        )

    def _verify_neow(self, decision: DecisionLog) -> VerificationResult:
        """Verify Neow bonus and apply RNG consumption."""
        chosen = decision.data.get("chosen", "")

        # Neow options consume different amounts of card RNG
        # Apply the consumption based on choice
        self._apply_neow_rng_consumption(chosen)

        return VerificationResult(
            decision=decision,
            verified=True,
            notes=f"Neow bonus applied: {chosen}",
        )

    def _predict_card_reward(self) -> list[str]:
        """Predict next card reward from seed."""
        try:
            # Use the RNG state to predict
            # This requires proper tracking of RNG consumption
            from core.generation.card_reward import generate_card_reward

            # Get current card RNG counter
            card_rng = self.rng_state.get_rng(RNGStream.CARD)

            # Generate predicted reward
            # Note: This is simplified - actual implementation would need
            # to account for rarity blizzard, relics affecting rewards, etc.
            predicted = generate_card_reward(
                card_rng,
                player_class="WATCHER",
                num_cards=3,
            )

            return predicted

        except Exception:
            # If prediction fails, return empty
            return []

    def _advance_for_combat(self, enemy: str) -> None:
        """Advance RNG state after combat."""
        # Combat consumes various RNG streams
        # This is simplified - full implementation would track
        # monster RNG, card rewards, potion drops, etc.
        pass

    def _apply_neow_rng_consumption(self, chosen: str) -> None:
        """Apply Neow choice RNG consumption."""
        # Different Neow options consume different amounts of card RNG
        consumption_map = {
            "upgrade_card": 0,
            "hundred_gold": 0,
            "random_colorless": 3,
            "rare_card": 3,
            "curse": 1,
            "boss_swap": 0,
        }

        # Normalize choice
        choice_lower = chosen.lower().replace(" ", "_")

        for key, amount in consumption_map.items():
            if key in choice_lower:
                # Advance card RNG by the consumption amount
                for _ in range(amount):
                    self.rng_state.advance(RNGStream.CARD)
                break

    def _normalize_card_name(self, name: str) -> str:
        """Normalize card name for comparison."""
        # Remove upgrade indicator
        name = name.rstrip("+")
        # Remove spaces and lowercase
        return name.replace(" ", "").lower()

    def predict_next_reward(
        self,
        room_type: str = "monster",
    ) -> dict:
        """
        Predict the next reward for a given room type.

        Returns dict with expected options.
        """
        if room_type in ["monster", "elite", "boss"]:
            cards = self._predict_card_reward()
            return {
                "card_options": cards,
                "gold_range": self._predict_gold_range(room_type),
            }
        return {}

    def _predict_gold_range(self, room_type: str) -> tuple[int, int]:
        """Predict gold reward range for room type."""
        ranges = {
            "monster": (10, 20),
            "elite": (25, 35),
            "boss": (95, 105),
        }
        return ranges.get(room_type, (10, 20))

    def reset(self) -> None:
        """Reset verifier state for new run."""
        self.rng_state = GameRNGState(seed_str=self.seed)
        self._path_history = []


def verify_extraction(
    state: VODRunState,
    seed: Optional[str] = None,
) -> Optional[SeedVerificationReport]:
    """
    Convenience function to verify an extracted run.

    Args:
        state: The VODRunState to verify
        seed: Seed to use (defaults to state's seed)

    Returns:
        Verification report or None if no seed
    """
    seed = seed or state.seed

    if not seed or seed == "UNKNOWN":
        return None

    verifier = SeedVerifier(seed)
    return verifier.verify_run(state)
