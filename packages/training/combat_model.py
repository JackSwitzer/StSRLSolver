"""Combat-first policy/value model primitives."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from math import inf
from pathlib import Path
from typing import Any, Mapping, Protocol

import numpy as np
import numpy.typing as npt

from .value_targets import CombatValueTarget, PHASE1_VALUE_HEAD_NAMES


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
    card_id: str | None = None
    potion_id: str | None = None
    label: str | None = None

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
            "card_id": self.card_id,
            "potion_id": self.potion_id,
            "label": self.label,
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
            card_id=payload.get("card_id"),
            potion_id=payload.get("potion_id"),
            label=payload.get("label"),
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
    predicted_value: CombatValueTarget | None = None

    def to_dict(self) -> JsonDict:
        return {
            "request_id": self.request_id,
            "chosen_action_id": self.chosen_action_id,
            "chosen_score": self.chosen_score,
            "ranked_action_ids": list(self.ranked_action_ids),
            "ranked_scores": list(self.ranked_scores),
            "frontier_action_ids": list(self.frontier_action_ids),
            "frontier_scores": list(self.frontier_scores),
            "predicted_value": None if self.predicted_value is None else self.predicted_value.to_dict(),
        }


@dataclass(slots=True, frozen=True)
class CombatBatchPredictions:
    """Policy scores plus multi-head value predictions for one packed batch."""

    policy_scores: npt.NDArray[np.float32]
    value_matrix: npt.NDArray[np.float32]
    value_head_names: tuple[str, ...] = PHASE1_VALUE_HEAD_NAMES


class CombatPolicyValueModel(Protocol):
    """Protocol shared by the simple linear and MLX policy/value models."""

    def predict_batch(self, batch: "CombatSearchBatchLike") -> CombatBatchPredictions:
        """Return policy scores and value-head predictions."""

    def train_puct_batch(
        self,
        batch: "CombatPuctTargetBatchLike",
        *,
        learning_rate: float | None = None,
    ) -> dict[str, Any]:
        """Apply one policy/value update step."""


class CombatSearchBatchLike(Protocol):
    request_ids: tuple[str, ...]
    state_matrix: npt.NDArray[np.float32]
    candidate_matrix: npt.NDArray[np.float32]
    legal_mask: npt.NDArray[np.bool_]
    candidate_counts: npt.NDArray[np.int32]
    candidate_ids: tuple[tuple[str, ...], ...]
    candidate_types: tuple[tuple[str, ...], ...]


class CombatPuctTargetBatchLike(CombatSearchBatchLike, Protocol):
    policy_target_matrix: npt.NDArray[np.float32]
    policy_target_mask: npt.NDArray[np.bool_]
    chosen_action_indices: npt.NDArray[np.int32]
    value_target_names: tuple[str, ...]
    value_target_matrix: npt.NDArray[np.float32]
    sample_weights: npt.NDArray[np.float32]


def _pad_weights(values: tuple[float, ...], width: int, fill: float) -> tuple[float, ...]:
    if len(values) >= width:
        return values
    return values + tuple(fill for _ in range(width - len(values)))


def _softmax_masked(scores: npt.NDArray[np.float32], mask: npt.NDArray[np.bool_]) -> npt.NDArray[np.float32]:
    masked = np.where(mask, scores, np.float32(-1e9))
    shifted = masked - np.max(masked, axis=1, keepdims=True)
    weights = np.exp(shifted, dtype=np.float32)
    weights = np.where(mask, weights, np.float32(0.0))
    totals = np.sum(weights, axis=1, keepdims=True)
    totals = np.where(totals > 0.0, totals, np.float32(1.0))
    return np.asarray(weights / totals, dtype=np.float32)


def _value_feature_matrix(batch: CombatSearchBatchLike) -> npt.NDArray[np.float32]:
    request_count = len(batch.request_ids)
    state_width = int(batch.state_matrix.shape[1]) if batch.state_matrix.ndim == 2 else 0
    candidate_width = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
    value_width = state_width + candidate_width * 2
    if request_count == 0 or value_width == 0:
        return np.zeros((request_count, value_width), dtype=np.float32)

    mean_candidate = np.zeros((request_count, candidate_width), dtype=np.float32)
    max_candidate = np.zeros((request_count, candidate_width), dtype=np.float32)
    for row in range(request_count):
        legal_indices = np.flatnonzero(batch.legal_mask[row])
        if legal_indices.size == 0:
            continue
        legal_features = batch.candidate_matrix[row, legal_indices]
        mean_candidate[row] = np.mean(legal_features, axis=0, dtype=np.float32)
        max_candidate[row] = np.max(legal_features, axis=0)
    return np.ascontiguousarray(
        np.concatenate((batch.state_matrix, mean_candidate, max_candidate), axis=1),
        dtype=np.float32,
    )


@dataclass(slots=True)
class LinearCombatModel:
    """Small deterministic policy/value model with direct PUCT target updates."""

    state_scale: float = 0.0
    candidate_scale: float = 1.0
    legal_bias: float = 1.0
    illegal_bias: float = -inf
    bias: float = 0.0
    default_learning_rate: float = 0.01
    candidate_weights: tuple[float, ...] = ()
    value_head_names: tuple[str, ...] = PHASE1_VALUE_HEAD_NAMES
    value_feature_weights: Mapping[str, tuple[float, ...]] = field(default_factory=dict)
    value_head_biases: Mapping[str, float] = field(default_factory=dict)
    _loaded_backend: str = field(default="numpy", init=False, repr=False)

    def _ensure_dimensions(self, candidate_dim: int, value_feature_dim: int) -> None:
        self.candidate_weights = _pad_weights(self.candidate_weights, candidate_dim, self.candidate_scale)
        weight_map = dict(self.value_feature_weights)
        bias_map = dict(self.value_head_biases)
        for head_name in self.value_head_names:
            weight_map[head_name] = _pad_weights(weight_map.get(head_name, ()), value_feature_dim, 0.0)
            bias_map.setdefault(head_name, 0.0)
        self.value_feature_weights = weight_map
        self.value_head_biases = bias_map

    def predict_batch(self, batch: CombatSearchBatchLike) -> CombatBatchPredictions:
        candidate_dim = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
        value_features = _value_feature_matrix(batch)
        value_feature_dim = int(value_features.shape[1]) if value_features.ndim == 2 else 0
        self._ensure_dimensions(candidate_dim, value_feature_dim)

        if candidate_dim:
            candidate_weights = np.asarray(self.candidate_weights[:candidate_dim], dtype=np.float32)
            policy_scores = np.sum(
                batch.candidate_matrix * candidate_weights[None, None, :],
                axis=2,
            )
        else:
            policy_scores = np.zeros(batch.legal_mask.shape, dtype=np.float32)
        if self.state_scale:
            state_term = np.sum(batch.state_matrix, axis=1, dtype=np.float32) * np.float32(self.state_scale)
            policy_scores = policy_scores + state_term[:, None]
        policy_scores = policy_scores + np.float32(self.bias)
        policy_scores = np.where(
            batch.legal_mask,
            policy_scores + np.float32(self.legal_bias),
            np.float32(self.illegal_bias),
        )

        value_matrix = np.zeros((len(batch.request_ids), len(self.value_head_names)), dtype=np.float32)
        for column, head_name in enumerate(self.value_head_names):
            head_weights = np.asarray(self.value_feature_weights.get(head_name, ()), dtype=np.float32)
            bias = np.float32(self.value_head_biases.get(head_name, 0.0))
            if head_weights.size:
                value_matrix[:, column] = np.sum(value_features * head_weights[None, :], axis=1) + bias
            else:
                value_matrix[:, column] = bias

        return CombatBatchPredictions(
            policy_scores=np.asarray(policy_scores, dtype=np.float32),
            value_matrix=np.asarray(value_matrix, dtype=np.float32),
            value_head_names=self.value_head_names,
        )

    def train_puct_batch(
        self,
        batch: CombatPuctTargetBatchLike,
        *,
        learning_rate: float | None = None,
    ) -> dict[str, Any]:
        predictions = self.predict_batch(batch)
        policy_probs = _softmax_masked(predictions.policy_scores, batch.legal_mask)
        sample_weights = batch.sample_weights[:, None]
        lr = float(self.default_learning_rate if learning_rate is None else learning_rate)

        target_policy = np.where(batch.policy_target_mask, batch.policy_target_matrix, np.float32(0.0))
        policy_grad = (policy_probs - target_policy) * sample_weights
        candidate_grad = np.sum(
            policy_grad[:, :, None] * batch.candidate_matrix,
            axis=(0, 1),
            dtype=np.float32,
        ) / np.float32(max(len(batch.request_ids), 1))
        candidate_weights = np.asarray(self.candidate_weights, dtype=np.float32)
        if candidate_grad.size:
            candidate_weights -= np.float32(lr) * candidate_grad
            self.candidate_weights = tuple(float(value) for value in candidate_weights)

        value_features = _value_feature_matrix(batch)
        if batch.value_target_names != self.value_head_names:
            raise ValueError("batch value head names must match model head names")
        prediction_error = (predictions.value_matrix - batch.value_target_matrix) * sample_weights
        feature_count = np.float32(max(len(batch.request_ids), 1))
        weight_map = dict(self.value_feature_weights)
        bias_map = dict(self.value_head_biases)
        for column, head_name in enumerate(self.value_head_names):
            head_error = prediction_error[:, column]
            grad = np.sum(head_error[:, None] * value_features, axis=0, dtype=np.float32) / feature_count
            bias_grad = float(np.sum(head_error, dtype=np.float32) / feature_count)
            head_weights = np.asarray(weight_map.get(head_name, ()), dtype=np.float32)
            if grad.size:
                head_weights -= np.float32(lr) * grad
                weight_map[head_name] = tuple(float(value) for value in head_weights)
            bias_map[head_name] = float(bias_map.get(head_name, 0.0) - lr * bias_grad)
        self.value_feature_weights = weight_map
        self.value_head_biases = bias_map

        chosen_mass = []
        for row, chosen_idx in enumerate(batch.chosen_action_indices.tolist()):
            if chosen_idx >= 0:
                chosen_mass.append(float(policy_probs[row, chosen_idx]))
        policy_loss = float(
            -np.sum(target_policy * np.log(np.clip(policy_probs, 1e-8, 1.0)), dtype=np.float32)
            / np.float32(max(len(batch.request_ids), 1))
        )
        value_loss = float(
            np.mean((predictions.value_matrix - batch.value_target_matrix) ** 2, dtype=np.float32)
        )
        return {
            "updated": True,
            "policy_loss": policy_loss,
            "value_loss": value_loss,
            "mean_chosen_action_mass": float(np.mean(chosen_mass, dtype=np.float32)) if chosen_mass else 0.0,
            "value_head_count": len(self.value_head_names),
        }

    def to_snapshot(self) -> JsonDict:
        return {
            "kind": "linear_policy_value_model/v2",
            "state_scale": self.state_scale,
            "candidate_scale": self.candidate_scale,
            "legal_bias": self.legal_bias,
            "illegal_bias": None if self.illegal_bias == -inf else self.illegal_bias,
            "bias": self.bias,
            "default_learning_rate": self.default_learning_rate,
            "candidate_weights": list(self.candidate_weights),
            "value_head_names": list(self.value_head_names),
            "value_feature_weights": {
                key: list(values) for key, values in sorted(self.value_feature_weights.items())
            },
            "value_head_biases": dict(sorted(self.value_head_biases.items())),
        }

    @classmethod
    def from_snapshot(cls, payload: Mapping[str, Any]) -> "LinearCombatModel":
        kind = str(payload.get("kind", "linear_policy_value_model/v2"))
        illegal_bias = payload.get("illegal_bias")
        if kind == "linear_combat_model/v1":
            return cls(
                candidate_scale=float(payload.get("candidate_scale", 1.0)),
                state_scale=float(payload.get("state_scale", 0.0)),
                legal_bias=float(payload.get("legal_bias", 1.0)),
                illegal_bias=-inf if illegal_bias is None else float(illegal_bias),
                bias=float(payload.get("bias", 0.0)),
                default_learning_rate=float(payload.get("default_learning_rate", 0.05)),
                candidate_weights=tuple(float(value) for value in payload.get("candidate_weights", ())),
            )
        return cls(
            candidate_scale=float(payload.get("candidate_scale", 1.0)),
            state_scale=float(payload.get("state_scale", 0.0)),
            legal_bias=float(payload.get("legal_bias", 1.0)),
            illegal_bias=-inf if illegal_bias is None else float(illegal_bias),
            bias=float(payload.get("bias", 0.0)),
            default_learning_rate=float(payload.get("default_learning_rate", 0.01)),
            candidate_weights=tuple(float(value) for value in payload.get("candidate_weights", ())),
            value_head_names=tuple(str(value) for value in payload.get("value_head_names", PHASE1_VALUE_HEAD_NAMES)),
            value_feature_weights={
                str(key): tuple(float(value) for value in values)
                for key, values in dict(payload.get("value_feature_weights", {})).items()
            },
            value_head_biases={
                str(key): float(value) for key, value in dict(payload.get("value_head_biases", {})).items()
            },
        )

    def save_checkpoint(self, path: str | Path) -> Path:
        destination = Path(path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(self.to_snapshot(), indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        return destination

    @classmethod
    def load_checkpoint(cls, path: str | Path) -> "LinearCombatModel":
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
        return cls.from_snapshot(payload)


@dataclass(slots=True)
class MLXCombatModel:
    """MLX-backed policy/value scorer that shares checkpoints with LinearCombatModel."""

    checkpoint_path: str | None = None
    candidate_scale: float = 1.0
    legal_bias: float = 1.0
    bias: float = 0.0
    default_learning_rate: float = 0.01
    _loaded_backend: str = field(default="pending", init=False, repr=False)
    _cached_model: LinearCombatModel | None = field(default=None, init=False, repr=False)
    _cached_checkpoint_mtime_ns: int | None = field(default=None, init=False, repr=False)

    def _checkpoint_model(self) -> LinearCombatModel:
        if self.checkpoint_path is None:
            if self._cached_model is None:
                self._cached_model = LinearCombatModel(
                    candidate_scale=self.candidate_scale,
                    state_scale=self.state_scale,
                    legal_bias=self.legal_bias,
                    bias=self.bias,
                    default_learning_rate=self.default_learning_rate,
                )
            self._loaded_backend = "numpy-fallback"
            return self._cached_model

        checkpoint = Path(self.checkpoint_path)
        if not checkpoint.exists():
            if self._cached_model is None:
                self._cached_model = LinearCombatModel(
                    candidate_scale=self.candidate_scale,
                    state_scale=self.state_scale,
                    legal_bias=self.legal_bias,
                    bias=self.bias,
                    default_learning_rate=self.default_learning_rate,
                )
            self._loaded_backend = "numpy-fallback"
            return self._cached_model

        mtime_ns = checkpoint.stat().st_mtime_ns
        if self._cached_model is None or self._cached_checkpoint_mtime_ns != mtime_ns:
            self._cached_model = LinearCombatModel.load_checkpoint(checkpoint)
            self._cached_checkpoint_mtime_ns = mtime_ns
        return self._cached_model

    def predict_batch(self, batch: CombatSearchBatchLike) -> CombatBatchPredictions:
        linear_model = self._checkpoint_model()
        try:
            import mlx.core as mx  # type: ignore
        except Exception:
            self._loaded_backend = "numpy-fallback"
            return linear_model.predict_batch(batch)

        self._loaded_backend = "mlx"
        candidate_dim = int(batch.candidate_matrix.shape[2]) if batch.candidate_matrix.ndim == 3 else 0
        value_features = _value_feature_matrix(batch)
        value_feature_dim = int(value_features.shape[1]) if value_features.ndim == 2 else 0
        linear_model._ensure_dimensions(candidate_dim, value_feature_dim)

        if candidate_dim:
            candidate_weights = mx.array(
                np.asarray(linear_model.candidate_weights[:candidate_dim], dtype=np.float32),
                dtype=mx.float32,
            )
            candidate = mx.array(batch.candidate_matrix, dtype=mx.float32)
            policy_scores = mx.sum(candidate * candidate_weights[None, None, :], axis=2)
        else:
            policy_scores = mx.zeros(batch.legal_mask.shape, dtype=mx.float32)

        raw_scores = policy_scores + np.float32(linear_model.bias)
        illegal_fill = mx.full(raw_scores.shape, -1e9, dtype=mx.float32)
        policy_scores = mx.where(
            mx.array(batch.legal_mask),
            raw_scores + np.float32(linear_model.legal_bias),
            illegal_fill,
        )

        value_features_mx = mx.array(value_features, dtype=mx.float32)
        value_columns = []
        for head_name in linear_model.value_head_names:
            head_weights = mx.array(
                np.asarray(linear_model.value_feature_weights.get(head_name, ()), dtype=np.float32),
                dtype=mx.float32,
            )
            bias = np.float32(linear_model.value_head_biases.get(head_name, 0.0))
            if head_weights.size == 0:
                value_columns.append(mx.full((len(batch.request_ids),), bias, dtype=mx.float32))
            else:
                value_columns.append(mx.sum(value_features_mx * head_weights[None, :], axis=1) + bias)

        value_matrix = mx.stack(value_columns, axis=1) if value_columns else mx.zeros((len(batch.request_ids), 0), dtype=mx.float32)
        return CombatBatchPredictions(
            policy_scores=np.asarray(policy_scores, dtype=np.float32),
            value_matrix=np.asarray(value_matrix, dtype=np.float32),
            value_head_names=linear_model.value_head_names,
        )

    def train_puct_batch(
        self,
        batch: CombatPuctTargetBatchLike,
        *,
        learning_rate: float | None = None,
    ) -> dict[str, Any]:
        linear_model = self._checkpoint_model()
        result = linear_model.train_puct_batch(batch, learning_rate=learning_rate)
        if self.checkpoint_path is not None:
            linear_model.save_checkpoint(self.checkpoint_path)
            self._cached_checkpoint_mtime_ns = Path(self.checkpoint_path).stat().st_mtime_ns
        self._cached_model = linear_model
        return result
