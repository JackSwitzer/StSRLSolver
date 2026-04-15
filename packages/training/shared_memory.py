"""Shared-memory shaped batching for combat search and reanalysis."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Mapping, Sequence

import numpy as np
import numpy.typing as npt

from .combat_model import CombatStateSummary, LegalCombatCandidate


@dataclass(slots=True, frozen=True)
class SharedMemoryConfig:
    """Batching config for lightweight search/reanalysis workers."""

    max_batch_size: int = 64
    max_candidates_per_request: int = 48
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


class CombatSharedMemoryBatcher:
    """Collects requests and packs them into dense numpy batches."""

    def __init__(
        self,
        max_batch_size: int = 64,
        *,
        max_candidates_per_request: int = 48,
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
