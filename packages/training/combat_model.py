"""Combat-first model and lightweight reanalysis training primitives."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from math import inf
from pathlib import Path
from typing import Any, Mapping, Protocol

import numpy as np
import numpy.typing as npt


StateVector = tuple[float, ...]
CandidateVector = tuple[float, ...]
JsonDict = dict[str, Any]


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

    def to_dict(self) -> JsonDict:
        return asdict(self)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "CombatStateSummary":
        return cls(
            combat_id=str(payload["combat_id"]),
            turn=int(payload["turn"]),
            hp=int(payload["hp"]),
            block=int(payload["block"]),
            energy=int(payload["energy"]),
            hand_size=int(payload["hand_size"]),
            draw_pile_size=int(payload["draw_pile_size"]),
            discard_pile_size=int(payload["discard_pile_size"]),
            exhaust_pile_size=int(payload["exhaust_pile_size"]),
            stance=str(payload.get("stance", "Neutral")),
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

    def to_dict(self) -> JsonDict:
        return {
            "action_id": self.action_id,
            "action_type": self.action_type,
            "target_idx": self.target_idx,
            "features": list(self.features),
            "legal": self.legal,
            "legality_reason": self.legality_reason,
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "LegalCombatCandidate":
        return cls(
            action_id=str(payload["action_id"]),
            action_type=str(payload["action_type"]),
            target_idx=int(payload.get("target_idx", -1)),
            features=tuple(float(value) for value in payload.get("features", ())),
            legal=bool(payload.get("legal", True)),
            legality_reason=str(payload.get("legality_reason", "legal")),
        )


@dataclass(slots=True, frozen=True)
class CombatInferenceResult:
    """Result returned by the combat inference service."""

    request_id: str
    chosen_action_id: str | None
    chosen_score: float | None
    ranked_action_ids: tuple[str, ...]
    ranked_scores: tuple[float, ...]
    frontier_action_ids: tuple[str, ...] = ()
    frontier_scores: tuple[float, ...] = ()

    def to_dict(self) -> JsonDict:
        return {
            "request_id": self.request_id,
            "chosen_action_id": self.chosen_action_id,
            "chosen_score": self.chosen_score,
            "ranked_action_ids": list(self.ranked_action_ids),
            "ranked_scores": list(self.ranked_scores),
            "frontier_action_ids": list(self.frontier_action_ids),
            "frontier_scores": list(self.frontier_scores),
        }


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


def _pad_weights(values: tuple[float, ...], width: int, fill: float) -> tuple[float, ...]:
    if len(values) >= width:
        return values
    return values + tuple(fill for _ in range(width - len(values)))


@dataclass(slots=True)
class LinearCombatModel:
    """Small deterministic model that also supports lightweight weight updates."""

    state_scale: float = 0.01
    candidate_scale: float = 1.0
    legal_bias: float = 1.0
    illegal_bias: float = -inf
    bias: float = 0.0
    default_learning_rate: float = 0.05
    state_weights: tuple[float, ...] = ()
    candidate_weights: tuple[float, ...] = ()
    _loaded_backend: str = field(default="numpy", init=False, repr=False)

    def _ensure_dimensions(self, state_dim: int, candidate_dim: int) -> None:
        self.state_weights = _pad_weights(self.state_weights, state_dim, self.state_scale)
        self.candidate_weights = _pad_weights(self.candidate_weights, candidate_dim, self.candidate_scale)

    def score_batch(self, batch: CombatSearchBatchLike) -> npt.NDArray[np.float32]:
        state_dim = int(batch.state_matrix.shape[1]) if batch.state_matrix.ndim == 2 else 0
        candidate_dim = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
        self._ensure_dimensions(state_dim, candidate_dim)

        if state_dim:
            state_weights = np.asarray(self.state_weights[:state_dim], dtype=np.float32)
            state_score = np.sum(batch.state_matrix * state_weights[None, :], axis=1, keepdims=True)
        else:
            state_score = np.zeros((len(batch.request_ids), 1), dtype=np.float32)

        if candidate_dim:
            candidate_weights = np.asarray(self.candidate_weights[:candidate_dim], dtype=np.float32)
            candidate_score = np.sum(
                batch.candidate_matrix * candidate_weights[None, None, :],
                axis=2,
            )
        else:
            candidate_score = np.zeros(batch.legal_mask.shape, dtype=np.float32)

        scores = candidate_score + state_score + np.float32(self.bias)
        scores = scores.astype(np.float32, copy=False)
        scores = np.where(batch.legal_mask, scores + np.float32(self.legal_bias), np.float32(self.illegal_bias))
        return scores

    def update_preference(
        self,
        batch: CombatSearchBatchLike,
        scores: npt.NDArray[np.float32],
        row: int,
        preferred_action_id: str,
        *,
        learning_rate: float | None = None,
    ) -> dict[str, Any]:
        """Apply a simple pairwise preference update for one request row."""

        ids = batch.candidate_ids[row]
        if not ids:
            return {"updated": False, "reason": "empty_request"}

        try:
            preferred_idx = ids.index(preferred_action_id)
        except ValueError:
            return {"updated": False, "reason": "preferred_missing"}

        legal_indices = np.flatnonzero(batch.legal_mask[row])
        if preferred_idx not in legal_indices:
            return {"updated": False, "reason": "preferred_not_legal"}

        rival_indices = [int(idx) for idx in legal_indices if int(idx) != preferred_idx]
        if not rival_indices:
            return {"updated": False, "reason": "single_legal_candidate"}

        rival_idx = max(rival_indices, key=lambda idx: float(scores[row, idx]))
        preferred_score = float(scores[row, preferred_idx])
        rival_score = float(scores[row, rival_idx])
        margin_before = preferred_score - rival_score
        if margin_before >= 0.0:
            return {
                "updated": False,
                "reason": "preferred_already_top",
                "preferred_rank": 1,
                "margin_before": margin_before,
            }

        candidate_dim = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
        self._ensure_dimensions(0, candidate_dim)
        lr = float(self.default_learning_rate if learning_rate is None else learning_rate)
        weights = np.asarray(self.candidate_weights[:candidate_dim], dtype=np.float32)
        preferred_features = batch.candidate_matrix[row, preferred_idx, :candidate_dim]
        rival_features = batch.candidate_matrix[row, rival_idx, :candidate_dim]
        weights += np.float32(lr) * (preferred_features - rival_features)
        self.candidate_weights = tuple(float(value) for value in weights)

        return {
            "updated": True,
            "reason": "pairwise_preference",
            "preferred_action_id": preferred_action_id,
            "rival_action_id": ids[rival_idx],
            "margin_before": margin_before,
            "preferred_rank": None,
        }

    def to_snapshot(self) -> JsonDict:
        return {
            "kind": "linear_combat_model/v1",
            "state_scale": self.state_scale,
            "candidate_scale": self.candidate_scale,
            "legal_bias": self.legal_bias,
            "illegal_bias": None if self.illegal_bias == -inf else self.illegal_bias,
            "bias": self.bias,
            "default_learning_rate": self.default_learning_rate,
            "state_weights": list(self.state_weights),
            "candidate_weights": list(self.candidate_weights),
        }

    @classmethod
    def from_snapshot(cls, payload: Mapping[str, Any]) -> "LinearCombatModel":
        illegal_bias = payload.get("illegal_bias")
        return cls(
            state_scale=float(payload.get("state_scale", 0.01)),
            candidate_scale=float(payload.get("candidate_scale", 1.0)),
            legal_bias=float(payload.get("legal_bias", 1.0)),
            illegal_bias=-inf if illegal_bias is None else float(illegal_bias),
            bias=float(payload.get("bias", 0.0)),
            default_learning_rate=float(payload.get("default_learning_rate", 0.05)),
            state_weights=tuple(float(value) for value in payload.get("state_weights", ())),
            candidate_weights=tuple(float(value) for value in payload.get("candidate_weights", ())),
        )

    def save_checkpoint(self, path: str | Path) -> Path:
        destination = Path(path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps(self.to_snapshot(), indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return destination

    @classmethod
    def load_checkpoint(cls, path: str | Path) -> "LinearCombatModel":
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
        return cls.from_snapshot(payload)


@dataclass(slots=True)
class MLXCombatModel:
    """MLX-backed linear scorer that shares checkpoints with LinearCombatModel."""

    checkpoint_path: str | None = None
    state_scale: float = 0.01
    candidate_scale: float = 1.0
    legal_bias: float = 1.0
    bias: float = 0.0
    _loaded_backend: str = field(default="pending", init=False, repr=False)
    _cached_model: LinearCombatModel | None = field(default=None, init=False, repr=False)
    _cached_checkpoint_mtime_ns: int | None = field(default=None, init=False, repr=False)

    def _checkpoint_model(self) -> LinearCombatModel:
        if self.checkpoint_path is None:
            self._loaded_backend = "numpy-fallback"
            return LinearCombatModel(
                state_scale=self.state_scale,
                candidate_scale=self.candidate_scale,
                legal_bias=self.legal_bias,
                bias=self.bias,
            )

        checkpoint = Path(self.checkpoint_path)
        if not checkpoint.exists():
            self._loaded_backend = "numpy-fallback"
            return LinearCombatModel(
                state_scale=self.state_scale,
                candidate_scale=self.candidate_scale,
                legal_bias=self.legal_bias,
                bias=self.bias,
            )

        mtime_ns = checkpoint.stat().st_mtime_ns
        if self._cached_model is None or self._cached_checkpoint_mtime_ns != mtime_ns:
            self._cached_model = LinearCombatModel.load_checkpoint(checkpoint)
            self._cached_checkpoint_mtime_ns = mtime_ns
        return self._cached_model

    def score_batch(self, batch: CombatSearchBatchLike) -> npt.NDArray[np.float32]:
        linear_model = self._checkpoint_model()
        state_dim = int(batch.state_matrix.shape[1]) if batch.state_matrix.ndim == 2 else 0
        candidate_dim = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
        linear_model._ensure_dimensions(state_dim, candidate_dim)

        try:
            import mlx.core as mx  # type: ignore
        except Exception:
            self._loaded_backend = "numpy-fallback"
            return linear_model.score_batch(batch)

        self._loaded_backend = "mlx"
        if state_dim:
            state_weights = mx.array(np.asarray(linear_model.state_weights[:state_dim], dtype=np.float32), dtype=mx.float32)
            state = mx.array(batch.state_matrix, dtype=mx.float32)
            state_score = mx.sum(state * state_weights[None, :], axis=1, keepdims=True)
        else:
            state_score = mx.zeros((len(batch.request_ids), 1), dtype=mx.float32)

        if candidate_dim:
            candidate_weights = mx.array(
                np.asarray(linear_model.candidate_weights[:candidate_dim], dtype=np.float32),
                dtype=mx.float32,
            )
            candidate = mx.array(batch.candidate_matrix, dtype=mx.float32)
            candidate_score = mx.sum(candidate * candidate_weights[None, None, :], axis=2)
        else:
            candidate_score = mx.zeros(batch.legal_mask.shape, dtype=mx.float32)

        raw_scores = candidate_score + state_score + np.float32(linear_model.bias)
        illegal_fill = mx.full(raw_scores.shape, -1e9, dtype=mx.float32)
        scored = mx.where(mx.array(batch.legal_mask), raw_scores + np.float32(linear_model.legal_bias), illegal_fill)
        return np.asarray(scored, dtype=np.float32)
