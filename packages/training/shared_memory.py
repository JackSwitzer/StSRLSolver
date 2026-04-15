"""Shared-memory shaped batching for combat search."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Mapping, Sequence

import numpy as np
import numpy.typing as npt

from .combat_model import CombatStateSummary, LegalCombatCandidate


@dataclass(slots=True, frozen=True)
class CombatSearchRequest:
    """A legal-candidate combat search request."""

    request_id: str
    state: CombatStateSummary
    candidates: tuple[LegalCombatCandidate, ...]
    metadata: Mapping[str, str] = field(default_factory=dict)


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


class CombatSharedMemoryBatcher:
    """Collects requests and packs them into dense numpy batches."""

    def __init__(self, max_batch_size: int = 64) -> None:
        if max_batch_size <= 0:
            raise ValueError("max_batch_size must be positive")
        self.max_batch_size = max_batch_size
        self._pending: list[CombatSearchRequest] = []

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
            state_matrix=state_matrix,
            candidate_matrix=candidate_matrix,
            legal_mask=legal_mask,
            candidate_counts=candidate_counts,
            candidate_ids=tuple(candidate_ids),
            candidate_types=tuple(candidate_types),
        )

