"""Combat-first inference service."""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import numpy.typing as npt

from .combat_model import CombatInferenceResult, CombatScoringModel
from .config import CombatSearchConfig
from .shared_memory import CombatSearchRequest, CombatSharedMemoryBatch, CombatSharedMemoryBatcher


@dataclass(slots=True)
class CombatInferenceService:
    """Legal-candidate combat inference with batched scoring."""

    model: CombatScoringModel
    config: CombatSearchConfig
    batcher: CombatSharedMemoryBatcher

    @classmethod
    def build(
        cls,
        model: CombatScoringModel,
        config: CombatSearchConfig | None = None,
        batcher: CombatSharedMemoryBatcher | None = None,
    ) -> "CombatInferenceService":
        config = config or CombatSearchConfig()
        batcher = batcher or CombatSharedMemoryBatcher(max_batch_size=config.top_k * 16)
        return cls(model=model, config=config, batcher=batcher)

    def submit(self, request: CombatSearchRequest) -> None:
        self.batcher.submit(request)

    def choose_action(self, request: CombatSearchRequest) -> CombatInferenceResult:
        packed = self.batcher.pack((request,))
        scores = self.model.score_batch(packed)
        return self._result_for_row(packed, scores, 0)

    def flush(self, limit: int | None = None) -> list[CombatInferenceResult]:
        requests = self.batcher.drain(limit)
        if not requests:
            return []
        packed = self.batcher.pack(requests)
        scores = self.model.score_batch(packed)
        return [self._result_for_row(packed, scores, row) for row in range(len(requests))]

    def _result_for_row(
        self,
        packed: CombatSharedMemoryBatch,
        scores: npt.NDArray[np.float32],
        row: int,
    ) -> CombatInferenceResult:
        legal_mask = packed.legal_mask[row]
        row_scores = scores[row]
        if not legal_mask.any():
            raise ValueError(f"request {packed.request_ids[row]} has no legal candidates")

        legal_indices = np.flatnonzero(legal_mask)
        legal_scores = row_scores[legal_indices]
        ranked_order = np.argsort(-legal_scores, kind="stable")
        ranked_indices = legal_indices[ranked_order]
        ranked_action_ids = tuple(packed.candidate_ids[row][idx] for idx in ranked_indices)
        ranked_scores = tuple(float(row_scores[idx]) for idx in ranked_indices)
        chosen_idx = int(ranked_indices[0])
        return CombatInferenceResult(
            request_id=packed.request_ids[row],
            chosen_action_id=packed.candidate_ids[row][chosen_idx],
            chosen_score=float(row_scores[chosen_idx]),
            ranked_action_ids=ranked_action_ids,
            ranked_scores=ranked_scores,
        )

