"""Combat-first model scaffolding."""

from __future__ import annotations

from dataclasses import dataclass, field
from math import inf
from typing import Protocol

import numpy as np
import numpy.typing as npt


StateVector = tuple[float, ...]
CandidateVector = tuple[float, ...]


@dataclass(slots=True, frozen=True)
class CombatStateSummary:
    """Compact combat-state features for legal-candidate search."""

    combat_id: str
    turn: int
    hp: int
    block: int
    energy: int
    hand_size: int
    draw_pile_size: int
    discard_pile_size: int
    exhaust_pile_size: int
    stance: str = "Neutral"

    def to_vector(self) -> StateVector:
        stance_index = {
            "Neutral": 0.0,
            "Wrath": 1.0,
            "Calm": 2.0,
            "Divinity": 3.0,
        }.get(self.stance, 0.0)
        return (
            float(self.turn),
            float(self.hp),
            float(self.block),
            float(self.energy),
            float(self.hand_size),
            float(self.draw_pile_size),
            float(self.discard_pile_size),
            float(self.exhaust_pile_size),
            float(stance_index),
        )


@dataclass(slots=True, frozen=True)
class LegalCombatCandidate:
    """A legal action candidate for combat search."""

    action_id: str
    action_type: str
    target_idx: int = -1
    features: CandidateVector = ()
    legal: bool = True
    legality_reason: str = "legal"

    def padded_features(self, width: int) -> np.ndarray:
        vector = np.zeros(width, dtype=np.float32)
        if self.features:
            limit = min(width, len(self.features))
            vector[:limit] = np.asarray(self.features[:limit], dtype=np.float32)
        return vector


@dataclass(slots=True, frozen=True)
class CombatInferenceResult:
    """Result returned by the combat inference service."""

    request_id: str
    chosen_action_id: str | None
    chosen_score: float | None
    ranked_action_ids: tuple[str, ...]
    ranked_scores: tuple[float, ...]


class CombatScoringModel(Protocol):
    """Protocol for batch scoring legal combat candidates."""

    def score_batch(self, batch: "CombatSearchBatchLike") -> npt.NDArray[np.float32]:
        """Return a [batch, candidates] score matrix."""


class CombatSearchBatchLike(Protocol):
    """Structural type shared by the batcher and model."""

    request_ids: tuple[str, ...]
    state_matrix: npt.NDArray[np.float32]
    candidate_matrix: npt.NDArray[np.float32]
    legal_mask: npt.NDArray[np.bool_]
    candidate_counts: npt.NDArray[np.int32]
    candidate_ids: tuple[tuple[str, ...], ...]


@dataclass(slots=True)
class LinearCombatModel:
    """Small deterministic scoring stub for the rebuild."""

    state_scale: float = 0.01
    candidate_scale: float = 1.0
    legal_bias: float = 1.0
    illegal_bias: float = -inf
    _loaded_backend: str = field(default="numpy", init=False, repr=False)

    def score_batch(self, batch: CombatSearchBatchLike) -> npt.NDArray[np.float32]:
        state_score = batch.state_matrix.sum(axis=1, keepdims=True) * self.state_scale
        candidate_score = batch.candidate_matrix.sum(axis=2) * self.candidate_scale
        scores = candidate_score + state_score
        scores = scores.astype(np.float32, copy=False)
        scores = np.where(batch.legal_mask, scores + self.legal_bias, self.illegal_bias)
        return scores


@dataclass(slots=True)
class MLXCombatModel:
    """MLX-backed placeholder that can be wired to a real checkpoint later."""

    checkpoint_path: str | None = None
    state_scale: float = 0.01
    candidate_scale: float = 1.0
    legal_bias: float = 1.0

    def score_batch(self, batch: CombatSearchBatchLike) -> npt.NDArray[np.float32]:
        try:
            import mlx.core as mx  # type: ignore
        except Exception:
            return LinearCombatModel(
                state_scale=self.state_scale,
                candidate_scale=self.candidate_scale,
                legal_bias=self.legal_bias,
            ).score_batch(batch)

        state = mx.array(batch.state_matrix, dtype=mx.float32)
        candidate = mx.array(batch.candidate_matrix, dtype=mx.float32)
        state_score = mx.sum(state, axis=1, keepdims=True) * self.state_scale
        candidate_score = mx.sum(candidate, axis=2) * self.candidate_scale
        scores = candidate_score + state_score + self.legal_bias
        illegal_fill = mx.full(scores.shape, -1e9, dtype=mx.float32)
        scores = mx.where(mx.array(batch.legal_mask), scores, illegal_fill)
        return np.asarray(scores, dtype=np.float32)

