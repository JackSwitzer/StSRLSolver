"""Shared-memory shaped batching for combat search and policy/value learning."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Mapping, Sequence

import numpy as np
import numpy.typing as npt

from .combat_model import CombatInferenceResult, CombatStateSummary, LegalCombatCandidate
from .value_targets import CombatValueTarget, PHASE1_VALUE_HEAD_NAMES


@dataclass(slots=True, frozen=True)
class SharedMemoryConfig:
    """Batching config for snapshot-backed search and policy/value workers."""

    max_batch_size: int = 128
    max_candidates_per_request: int = 64
    state_feature_dim: int = 16
    candidate_feature_dim: int = 16

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True, frozen=True)
class CombatSearchRequest:
    """A legal-candidate combat search request."""

    request_id: str
    state: CombatStateSummary
    candidates: tuple[LegalCombatCandidate, ...]
    metadata: Mapping[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "request_id": self.request_id,
            "state": self.state.to_dict(),
            "candidates": [candidate.to_dict() for candidate in self.candidates],
            "metadata": dict(self.metadata),
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "CombatSearchRequest":
        return cls(
            request_id=str(payload["request_id"]),
            state=CombatStateSummary.from_dict(payload["state"]),
            candidates=tuple(
                LegalCombatCandidate.from_dict(candidate)
                for candidate in payload.get("candidates", ())
            ),
            metadata=dict(payload.get("metadata", {})),
        )


def _normalized_policy_distribution(values: Sequence[float], temperature: float) -> tuple[float, ...]:
    if not values:
        return ()

    if len(values) == 1:
        return (1.0,)

    if temperature <= 0.0:
        best_idx = int(np.argmax(np.asarray(values, dtype=np.float32)))
        return tuple(1.0 if idx == best_idx else 0.0 for idx in range(len(values)))

    scores = np.asarray(values, dtype=np.float32) / np.float32(temperature)
    scores = scores - np.max(scores)
    weights = np.exp(scores)
    total = float(np.sum(weights))
    if not np.isfinite(total) or total <= 0.0:
        return tuple(1.0 / len(values) for _ in values)
    return tuple(float(value / total) for value in weights)


@dataclass(slots=True, frozen=True)
class CombatPuctTargetExample:
    """Canonical PUCT target record built from one combat root state."""

    request: CombatSearchRequest
    policy_action_ids: tuple[str, ...]
    policy_scores: tuple[float, ...]
    value_target: CombatValueTarget
    chosen_action_id: str | None = None
    visit_counts: tuple[int, ...] = ()
    temperature: float = 1.0
    sample_weight: float = 1.0
    metadata: Mapping[str, Any] = field(default_factory=dict)

    def policy_distribution(self) -> tuple[float, ...]:
        if self.visit_counts and len(self.visit_counts) == len(self.policy_action_ids):
            total_visits = sum(self.visit_counts)
            if total_visits > 0:
                return tuple(float(count / total_visits) for count in self.visit_counts)
        return _normalized_policy_distribution(self.policy_scores, self.temperature)

    @classmethod
    def from_result(
        cls,
        request: CombatSearchRequest,
        result: CombatInferenceResult,
        *,
        value_target: CombatValueTarget | None = None,
        chosen_action_id: str | None = None,
        visit_counts: Sequence[int] = (),
        temperature: float = 1.0,
        sample_weight: float = 1.0,
        metadata: Mapping[str, Any] | None = None,
    ) -> "CombatPuctTargetExample":
        return cls(
            request=request,
            policy_action_ids=result.frontier_action_ids,
            policy_scores=result.frontier_scores,
            value_target=value_target or CombatValueTarget(
                solve_probability=0.0,
                expected_hp_loss=0.0,
                expected_turns=0.0,
                potion_spend_count=0.0,
                setup_delta=0.0,
                persistent_scaling_delta=0.0,
            ),
            chosen_action_id=chosen_action_id if chosen_action_id is not None else result.chosen_action_id,
            visit_counts=tuple(int(value) for value in visit_counts),
            temperature=temperature,
            sample_weight=sample_weight,
            metadata=dict(metadata or {}),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "request": self.request.to_dict(),
            "policy_action_ids": list(self.policy_action_ids),
            "policy_scores": list(self.policy_scores),
            "value_target": self.value_target.to_dict(),
            "chosen_action_id": self.chosen_action_id,
            "visit_counts": list(self.visit_counts),
            "temperature": self.temperature,
            "sample_weight": self.sample_weight,
            "metadata": dict(self.metadata),
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "CombatPuctTargetExample":
        return cls(
            request=CombatSearchRequest.from_dict(payload["request"]),
            policy_action_ids=tuple(str(value) for value in payload.get("policy_action_ids", ())),
            policy_scores=tuple(float(value) for value in payload.get("policy_scores", ())),
            value_target=CombatValueTarget.from_dict(payload.get("value_target", {})),
            chosen_action_id=payload.get("chosen_action_id"),
            visit_counts=tuple(int(value) for value in payload.get("visit_counts", ())),
            temperature=float(payload.get("temperature", 1.0)),
            sample_weight=float(payload.get("sample_weight", 1.0)),
            metadata=dict(payload.get("metadata", {})),
        )


@dataclass(slots=True, frozen=True)
class CombatSharedMemoryBatch:
    """Packed batch ready for inference."""

    request_ids: tuple[str, ...]
    state_matrix: npt.NDArray[np.float32]
    candidate_matrix: npt.NDArray[np.float32]
    legal_mask: npt.NDArray[np.bool_]
    candidate_counts: npt.NDArray[np.int32]
    candidate_ids: tuple[tuple[str, ...], ...]
    candidate_types: tuple[tuple[str, ...], ...]

    @property
    def request_count(self) -> int:
        return len(self.request_ids)

    @property
    def state_width(self) -> int:
        return int(self.state_matrix.shape[1]) if self.state_matrix.ndim == 2 else 0

    @property
    def candidate_width(self) -> int:
        return int(self.candidate_matrix.shape[2]) if self.candidate_matrix.ndim == 3 else 0

    def legal_indices(self, row: int) -> npt.NDArray[np.int64]:
        return np.flatnonzero(self.legal_mask[row])

    def frontier_action_ids(self, row: int) -> tuple[str, ...]:
        legal_indices = self.legal_indices(row)
        return tuple(self.candidate_ids[row][int(index)] for index in legal_indices)


@dataclass(slots=True, frozen=True)
class CombatPuctTargetBatch:
    """Packed PUCT-style training batch with policy and value targets."""

    request_ids: tuple[str, ...]
    state_matrix: npt.NDArray[np.float32]
    candidate_matrix: npt.NDArray[np.float32]
    legal_mask: npt.NDArray[np.bool_]
    candidate_counts: npt.NDArray[np.int32]
    candidate_ids: tuple[tuple[str, ...], ...]
    target_action_ids: tuple[tuple[str, ...], ...]
    policy_target_matrix: npt.NDArray[np.float32]
    policy_target_mask: npt.NDArray[np.bool_]
    chosen_action_indices: npt.NDArray[np.int32]
    value_target_names: tuple[str, ...]
    value_target_matrix: npt.NDArray[np.float32]
    sample_weights: npt.NDArray[np.float32]

    @property
    def request_count(self) -> int:
        return len(self.request_ids)

    @property
    def candidate_width(self) -> int:
        return int(self.candidate_matrix.shape[2]) if self.candidate_matrix.ndim == 3 else 0

    @property
    def policy_width(self) -> int:
        return int(self.policy_target_matrix.shape[1]) if self.policy_target_matrix.ndim == 2 else 0


class CombatSharedMemoryBatcher:
    """Collects requests and packs them into dense numpy batches."""

    def __init__(
        self,
        max_batch_size: int = 128,
        *,
        max_candidates_per_request: int = 64,
    ) -> None:
        if max_batch_size <= 0:
            raise ValueError("max_batch_size must be positive")
        if max_candidates_per_request <= 0:
            raise ValueError("max_candidates_per_request must be positive")
        self.max_batch_size = max_batch_size
        self.max_candidates_per_request = max_candidates_per_request
        self._pending: list[CombatSearchRequest] = []

    @classmethod
    def from_config(cls, config: SharedMemoryConfig) -> "CombatSharedMemoryBatcher":
        return cls(
            max_batch_size=config.max_batch_size,
            max_candidates_per_request=config.max_candidates_per_request,
        )

    @property
    def pending_count(self) -> int:
        return len(self._pending)

    def submit(self, request: CombatSearchRequest) -> None:
        self._pending.append(request)

    def can_drain(self) -> bool:
        return bool(self._pending)

    def drain(self, limit: int | None = None) -> list[CombatSearchRequest]:
        if not self._pending:
            return []
        limit = self.max_batch_size if limit is None else max(1, limit)
        count = min(len(self._pending), limit)
        drained = self._pending[:count]
        del self._pending[:count]
        return drained

    def pack(self, requests: Sequence[CombatSearchRequest]) -> CombatSharedMemoryBatch:
        if len(requests) > self.max_batch_size:
            raise ValueError("request batch exceeds configured max_batch_size")
        if any(len(request.candidates) > self.max_candidates_per_request for request in requests):
            raise ValueError("request exceeds configured max_candidates_per_request")

        if not requests:
            empty_f32 = np.zeros((0, 0), dtype=np.float32)
            empty_bool = np.zeros((0, 0), dtype=bool)
            empty_i32 = np.zeros((0,), dtype=np.int32)
            return CombatSharedMemoryBatch(
                request_ids=(),
                state_matrix=empty_f32,
                candidate_matrix=np.zeros((0, 0, 0), dtype=np.float32),
                legal_mask=empty_bool,
                candidate_counts=empty_i32,
                candidate_ids=(),
                candidate_types=(),
            )

        state_width = max(len(request.state.to_vector()) for request in requests)
        candidate_width = max(
            [len(candidate.features) for request in requests for candidate in request.candidates] or [0]
        )
        candidate_width = max(candidate_width, 1)
        max_candidates = max(len(request.candidates) for request in requests)

        state_matrix = np.zeros((len(requests), state_width), dtype=np.float32)
        candidate_matrix = np.zeros((len(requests), max_candidates, candidate_width), dtype=np.float32)
        legal_mask = np.zeros((len(requests), max_candidates), dtype=bool)
        candidate_counts = np.zeros((len(requests),), dtype=np.int32)
        candidate_ids: list[tuple[str, ...]] = []
        candidate_types: list[tuple[str, ...]] = []

        for row, request in enumerate(requests):
            state_vec = np.asarray(request.state.to_vector(), dtype=np.float32)
            state_matrix[row, : len(state_vec)] = state_vec
            candidate_counts[row] = len(request.candidates)
            candidate_ids.append(tuple(candidate.action_id for candidate in request.candidates))
            candidate_types.append(tuple(candidate.action_type for candidate in request.candidates))

            for col, candidate in enumerate(request.candidates):
                candidate_matrix[row, col] = candidate.padded_features(candidate_width)
                legal_mask[row, col] = candidate.legal

        return CombatSharedMemoryBatch(
            request_ids=tuple(request.request_id for request in requests),
            state_matrix=np.ascontiguousarray(state_matrix),
            candidate_matrix=np.ascontiguousarray(candidate_matrix),
            legal_mask=np.ascontiguousarray(legal_mask),
            candidate_counts=np.ascontiguousarray(candidate_counts),
            candidate_ids=tuple(candidate_ids),
            candidate_types=tuple(candidate_types),
        )

    def pack_puct_targets(self, examples: Sequence[CombatPuctTargetExample]) -> CombatPuctTargetBatch:
        if len(examples) > self.max_batch_size:
            raise ValueError("request batch exceeds configured max_batch_size")
        if not examples:
            empty_f32 = np.zeros((0, 0), dtype=np.float32)
            empty_bool = np.zeros((0, 0), dtype=bool)
            empty_i32 = np.zeros((0,), dtype=np.int32)
            return CombatPuctTargetBatch(
                request_ids=(),
                state_matrix=empty_f32,
                candidate_matrix=np.zeros((0, 0, 0), dtype=np.float32),
                legal_mask=empty_bool,
                candidate_counts=empty_i32,
                candidate_ids=(),
                target_action_ids=(),
                policy_target_matrix=empty_f32,
                policy_target_mask=empty_bool,
                chosen_action_indices=empty_i32,
                value_target_names=PHASE1_VALUE_HEAD_NAMES,
                value_target_matrix=np.zeros((0, len(PHASE1_VALUE_HEAD_NAMES)), dtype=np.float32),
                sample_weights=np.zeros((0,), dtype=np.float32),
            )

        base_batch = self.pack(tuple(example.request for example in examples))
        policy_width = base_batch.legal_mask.shape[1] if base_batch.legal_mask.ndim == 2 else 0
        policy_matrix = np.zeros((len(examples), policy_width), dtype=np.float32)
        policy_mask = np.zeros((len(examples), policy_width), dtype=bool)
        chosen_action_indices = np.full((len(examples),), -1, dtype=np.int32)
        value_target_names = PHASE1_VALUE_HEAD_NAMES
        value_target_matrix = np.zeros((len(examples), len(value_target_names)), dtype=np.float32)
        sample_weights = np.zeros((len(examples),), dtype=np.float32)
        target_action_ids: list[tuple[str, ...]] = []

        for row, example in enumerate(examples):
            candidate_positions = {action_id: idx for idx, action_id in enumerate(base_batch.candidate_ids[row])}
            target_action_ids.append(tuple(example.policy_action_ids))
            distribution = example.policy_distribution()
            if len(distribution) != len(example.policy_action_ids):
                raise ValueError("policy distribution width does not match policy support width")
            for action_id, weight in zip(example.policy_action_ids, distribution):
                if action_id not in candidate_positions:
                    raise ValueError(f"policy action id {action_id!r} is not present in the request candidates")
                col = candidate_positions[action_id]
                policy_matrix[row, col] = np.float32(weight)
                policy_mask[row, col] = True

            if example.chosen_action_id is not None:
                chosen_action_indices[row] = candidate_positions.get(example.chosen_action_id, -1)
            value_target_matrix[row] = np.asarray(example.value_target.to_vector(value_target_names), dtype=np.float32)
            sample_weights[row] = np.float32(example.sample_weight)

        return CombatPuctTargetBatch(
            request_ids=base_batch.request_ids,
            state_matrix=base_batch.state_matrix,
            candidate_matrix=base_batch.candidate_matrix,
            legal_mask=base_batch.legal_mask,
            candidate_counts=base_batch.candidate_counts,
            candidate_ids=base_batch.candidate_ids,
            target_action_ids=tuple(target_action_ids),
            policy_target_matrix=np.ascontiguousarray(policy_matrix),
            policy_target_mask=np.ascontiguousarray(policy_mask),
            chosen_action_indices=np.ascontiguousarray(chosen_action_indices),
            value_target_names=value_target_names,
            value_target_matrix=np.ascontiguousarray(value_target_matrix),
            sample_weights=np.ascontiguousarray(sample_weights),
        )
