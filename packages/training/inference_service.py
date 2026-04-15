"""Phase-1 combat inference and lightweight reanalysis loop."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from time import perf_counter
from typing import Any, Mapping, Sequence

import numpy as np
import numpy.typing as npt

from .combat_model import CombatInferenceResult, CombatScoringModel
from .shared_memory import (
    CombatSearchRequest,
    CombatSharedMemoryBatch,
    CombatSharedMemoryBatcher,
    SharedMemoryConfig,
)


@dataclass(slots=True, frozen=True)
class CombatSearchConfig:
    """Policy-side config for local combat search scoring."""

    top_k: int = 4
    require_legal_candidates: bool = True
    batch_timeout_ms: int = 10
    max_candidates_per_request: int = 48

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True, frozen=True)
class TrainingConfig:
    """Minimal phase-1 training bring-up config."""

    model_backend: str = "mlx"
    shared_memory: SharedMemoryConfig = field(default_factory=SharedMemoryConfig)
    combat_search: CombatSearchConfig = field(default_factory=CombatSearchConfig)
    overnight_epochs: int = 4
    learning_rate: float = 0.05

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True, frozen=True)
class CombatPreferenceExample:
    """One search request plus the preferred frontier action."""

    request: CombatSearchRequest
    preferred_action_id: str
    weight: float = 1.0
    metadata: Mapping[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "request": self.request.to_dict(),
            "preferred_action_id": self.preferred_action_id,
            "weight": self.weight,
            "metadata": dict(self.metadata),
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "CombatPreferenceExample":
        return cls(
            request=CombatSearchRequest.from_dict(payload["request"]),
            preferred_action_id=str(payload["preferred_action_id"]),
            weight=float(payload.get("weight", 1.0)),
            metadata=dict(payload.get("metadata", {})),
        )


@dataclass(slots=True, frozen=True)
class ReanalysisEpochSummary:
    """Epoch-level metrics for the phase-1 reanalysis loop."""

    epoch_index: int
    example_count: int
    chosen_preferred_count: int
    accuracy: float
    mean_preferred_rank: float | None
    mean_frontier_size: float
    mean_preferred_margin: float | None
    updated_examples: int
    throughput_examples_per_sec: float

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


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
        batcher = batcher or CombatSharedMemoryBatcher(
            max_batch_size=max(config.top_k * 16, 1),
            max_candidates_per_request=config.max_candidates_per_request,
        )
        return cls(model=model, config=config, batcher=batcher)

    def submit(self, request: CombatSearchRequest) -> None:
        self.batcher.submit(request)

    def score_requests(
        self,
        requests: Sequence[CombatSearchRequest],
    ) -> tuple[CombatSharedMemoryBatch, npt.NDArray[np.float32]]:
        packed = self.batcher.pack(tuple(requests))
        scores = self.model.score_batch(packed)
        return packed, scores

    def results_from_scores(
        self,
        packed: CombatSharedMemoryBatch,
        scores: npt.NDArray[np.float32],
    ) -> list[CombatInferenceResult]:
        return [self._result_for_row(packed, scores, row) for row in range(packed.request_count)]

    def choose_action(self, request: CombatSearchRequest) -> CombatInferenceResult:
        packed, scores = self.score_requests((request,))
        return self._result_for_row(packed, scores, 0)

    def reanalyze_requests(self, requests: Sequence[CombatSearchRequest]) -> list[CombatInferenceResult]:
        if not requests:
            return []
        packed, scores = self.score_requests(requests)
        return self.results_from_scores(packed, scores)

    def flush(self, limit: int | None = None) -> list[CombatInferenceResult]:
        requests = self.batcher.drain(limit)
        if not requests:
            return []
        return self.reanalyze_requests(requests)

    def _result_for_row(
        self,
        packed: CombatSharedMemoryBatch,
        scores: npt.NDArray[np.float32],
        row: int,
    ) -> CombatInferenceResult:
        eligible_mask = (
            packed.legal_mask[row]
            if self.config.require_legal_candidates
            else np.ones_like(packed.legal_mask[row], dtype=bool)
        )
        row_scores = scores[row]
        if not eligible_mask.any():
            raise ValueError(f"request {packed.request_ids[row]} has no eligible candidates")

        frontier_indices = np.flatnonzero(eligible_mask)
        frontier_scores = row_scores[frontier_indices]
        ranked_order = np.argsort(-frontier_scores, kind="stable")
        ranked_indices = frontier_indices[ranked_order]
        frontier_action_ids = tuple(packed.candidate_ids[row][int(idx)] for idx in ranked_indices)
        frontier_score_values = tuple(float(row_scores[int(idx)]) for idx in ranked_indices)
        top_k = max(1, self.config.top_k)
        ranked_action_ids = frontier_action_ids[:top_k]
        ranked_scores = frontier_score_values[:top_k]
        chosen_idx = int(ranked_indices[0])
        return CombatInferenceResult(
            request_id=packed.request_ids[row],
            chosen_action_id=packed.candidate_ids[row][chosen_idx],
            chosen_score=float(row_scores[chosen_idx]),
            ranked_action_ids=ranked_action_ids,
            ranked_scores=ranked_scores,
            frontier_action_ids=frontier_action_ids,
            frontier_scores=frontier_score_values,
        )


@dataclass(slots=True)
class OvernightReanalysisLoop:
    """Simple epoch loop for search reanalysis and lightweight weight updates."""

    service: CombatInferenceService
    learning_rate: float = 0.05
    batch_size: int | None = None

    def run_epoch(
        self,
        examples: Sequence[CombatPreferenceExample],
        *,
        epoch_index: int = 0,
        update: bool = True,
    ) -> tuple[list[CombatInferenceResult], ReanalysisEpochSummary]:
        if not examples:
            return [], ReanalysisEpochSummary(
                epoch_index=epoch_index,
                example_count=0,
                chosen_preferred_count=0,
                accuracy=0.0,
                mean_preferred_rank=None,
                mean_frontier_size=0.0,
                mean_preferred_margin=None,
                updated_examples=0,
                throughput_examples_per_sec=0.0,
            )

        chunk_size = self.batch_size or self.service.batcher.max_batch_size
        start = perf_counter()
        results: list[CombatInferenceResult] = []
        chosen_preferred = 0
        preferred_ranks: list[float] = []
        frontier_sizes: list[float] = []
        preferred_margins: list[float] = []
        updated_examples = 0

        for offset in range(0, len(examples), chunk_size):
            chunk = tuple(examples[offset : offset + chunk_size])
            packed, scores = self.service.score_requests(tuple(example.request for example in chunk))
            batch_results = self.service.results_from_scores(packed, scores)

            for row, (example, result) in enumerate(zip(chunk, batch_results)):
                results.append(result)
                frontier_sizes.append(float(len(result.frontier_action_ids)))
                if result.chosen_action_id == example.preferred_action_id:
                    chosen_preferred += 1

                try:
                    preferred_rank = result.frontier_action_ids.index(example.preferred_action_id) + 1
                except ValueError:
                    preferred_rank = None

                if preferred_rank is not None:
                    preferred_ranks.append(float(preferred_rank))
                    preferred_score = result.frontier_scores[preferred_rank - 1]
                    chosen_score = result.chosen_score if result.chosen_score is not None else preferred_score
                    preferred_margins.append(float(preferred_score - chosen_score))

                if update and hasattr(self.service.model, "update_preference"):
                    update_result = self.service.model.update_preference(
                        packed,
                        scores,
                        row,
                        example.preferred_action_id,
                        learning_rate=self.learning_rate * example.weight,
                    )
                    updated_examples += int(bool(update_result.get("updated")))

        elapsed = max(perf_counter() - start, 1e-9)
        summary = ReanalysisEpochSummary(
            epoch_index=epoch_index,
            example_count=len(examples),
            chosen_preferred_count=chosen_preferred,
            accuracy=chosen_preferred / len(examples),
            mean_preferred_rank=(sum(preferred_ranks) / len(preferred_ranks)) if preferred_ranks else None,
            mean_frontier_size=sum(frontier_sizes) / len(frontier_sizes),
            mean_preferred_margin=(sum(preferred_margins) / len(preferred_margins)) if preferred_margins else None,
            updated_examples=updated_examples,
            throughput_examples_per_sec=len(examples) / elapsed,
        )
        return results, summary

    def run(
        self,
        examples: Sequence[CombatPreferenceExample],
        *,
        epochs: int,
        update: bool = True,
    ) -> list[ReanalysisEpochSummary]:
        summaries: list[ReanalysisEpochSummary] = []
        for epoch_index in range(epochs):
            _, summary = self.run_epoch(
                examples,
                epoch_index=epoch_index,
                update=update,
            )
            summaries.append(summary)
        return summaries
