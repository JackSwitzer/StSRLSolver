"""Combat-first inference and policy/value training helpers."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from time import perf_counter
from typing import Any, Sequence

import numpy as np
import numpy.typing as npt

from .combat_model import CombatBatchPredictions, CombatInferenceResult, CombatPolicyValueModel
from .shared_memory import (
    CombatPuctTargetBatch,
    CombatPuctTargetExample,
    CombatSearchRequest,
    CombatSharedMemoryBatch,
    CombatSharedMemoryBatcher,
    SharedMemoryConfig,
)
from .value_targets import CombatValueTarget


@dataclass(slots=True, frozen=True)
class CombatSearchConfig:
    """Policy-side config for local combat search scoring."""

    top_k: int = 4
    require_legal_candidates: bool = True
    puct_target_temperature: float = 1.0
    batch_timeout_ms: int = 15
    max_candidates_per_request: int = 64

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True, frozen=True)
class TrainingConfig:
    """Minimal phase-1 training config."""

    model_backend: str = "mlx"
    shared_memory: SharedMemoryConfig = field(default_factory=SharedMemoryConfig)
    combat_search: CombatSearchConfig = field(default_factory=CombatSearchConfig)
    overnight_epochs: int = 4
    learning_rate: float = 0.01

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True, frozen=True)
class PolicyValueEpochSummary:
    epoch_index: int
    example_count: int
    policy_loss: float
    value_loss: float
    mean_frontier_size: float
    mean_chosen_action_mass: float
    updated_batches: int
    throughput_examples_per_sec: float

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


class CombatInferenceService:
    """Legal-candidate combat inference with batched policy/value predictions."""

    def __init__(
        self,
        model: CombatPolicyValueModel,
        config: CombatSearchConfig,
        batcher: CombatSharedMemoryBatcher,
    ) -> None:
        self.model = model
        self.config = config
        self.batcher = batcher

    @classmethod
    def build(
        cls,
        model: CombatPolicyValueModel,
        config: CombatSearchConfig | None = None,
        batcher: CombatSharedMemoryBatcher | None = None,
    ) -> "CombatInferenceService":
        config = config or CombatSearchConfig()
        batcher = batcher or CombatSharedMemoryBatcher(
            max_batch_size=max(config.top_k * 32, 128),
            max_candidates_per_request=config.max_candidates_per_request,
        )
        return cls(model=model, config=config, batcher=batcher)

    def submit(self, request: CombatSearchRequest) -> None:
        self.batcher.submit(request)

    def predict_requests(
        self,
        requests: Sequence[CombatSearchRequest],
    ) -> tuple[CombatSharedMemoryBatch, CombatBatchPredictions]:
        packed = self.batcher.pack(tuple(requests))
        predictions = self.model.predict_batch(packed)
        return packed, predictions

    def results_from_predictions(
        self,
        packed: CombatSharedMemoryBatch,
        predictions: CombatBatchPredictions,
    ) -> list[CombatInferenceResult]:
        return [self._result_for_row(packed, predictions, row) for row in range(packed.request_count)]

    def choose_action(self, request: CombatSearchRequest) -> CombatInferenceResult:
        packed, predictions = self.predict_requests((request,))
        return self._result_for_row(packed, predictions, 0)

    def build_puct_target_batch(
        self,
        examples: Sequence[CombatPuctTargetExample],
    ) -> CombatPuctTargetBatch:
        return self.batcher.pack_puct_targets(examples)

    def flush(self, limit: int | None = None) -> list[CombatInferenceResult]:
        requests = self.batcher.drain(limit)
        if not requests:
            return []
        packed, predictions = self.predict_requests(requests)
        return self.results_from_predictions(packed, predictions)

    def _result_for_row(
        self,
        packed: CombatSharedMemoryBatch,
        predictions: CombatBatchPredictions,
        row: int,
    ) -> CombatInferenceResult:
        eligible_mask = (
            packed.legal_mask[row]
            if self.config.require_legal_candidates
            else np.ones_like(packed.legal_mask[row], dtype=bool)
        )
        row_scores = predictions.policy_scores[row]
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
        predicted_value = CombatValueTarget.from_vector(
            predictions.value_head_names,
            predictions.value_matrix[row].tolist(),
        )
        return CombatInferenceResult(
            request_id=packed.request_ids[row],
            chosen_action_id=packed.candidate_ids[row][chosen_idx],
            chosen_score=float(row_scores[chosen_idx]),
            ranked_action_ids=ranked_action_ids,
            ranked_scores=ranked_scores,
            frontier_action_ids=frontier_action_ids,
            frontier_scores=frontier_score_values,
            predicted_value=predicted_value,
        )


@dataclass(slots=True)
class CombatPolicyValueTrainer:
    """Simple epoch loop for direct policy/value updates from PUCT targets."""

    service: CombatInferenceService
    learning_rate: float = 0.01
    batch_size: int | None = None

    def run_epoch(
        self,
        examples: Sequence[CombatPuctTargetExample],
        *,
        epoch_index: int = 0,
        update: bool = True,
    ) -> PolicyValueEpochSummary:
        if not examples:
            return PolicyValueEpochSummary(
                epoch_index=epoch_index,
                example_count=0,
                policy_loss=0.0,
                value_loss=0.0,
                mean_frontier_size=0.0,
                mean_chosen_action_mass=0.0,
                updated_batches=0,
                throughput_examples_per_sec=0.0,
            )

        chunk_size = self.batch_size or self.service.batcher.max_batch_size
        start = perf_counter()
        policy_losses: list[float] = []
        value_losses: list[float] = []
        frontier_sizes: list[float] = []
        chosen_masses: list[float] = []
        updated_batches = 0

        for offset in range(0, len(examples), chunk_size):
            chunk = tuple(examples[offset : offset + chunk_size])
            batch = self.service.build_puct_target_batch(chunk)
            predictions = self.service.model.predict_batch(batch)
            frontier_sizes.extend(float(np.count_nonzero(row)) for row in batch.policy_target_mask)
            policy_probs = self._softmax_masked(predictions.policy_scores, batch.legal_mask)
            target_policy = np.where(batch.policy_target_mask, batch.policy_target_matrix, np.float32(0.0))
            policy_losses.append(
                float(
                    -np.sum(target_policy * np.log(np.clip(policy_probs, 1e-8, 1.0)), dtype=np.float32)
                    / np.float32(max(batch.request_count, 1))
                )
            )
            value_losses.append(
                float(
                    np.mean((predictions.value_matrix - batch.value_target_matrix) ** 2, dtype=np.float32)
                )
            )
            for row, chosen_idx in enumerate(batch.chosen_action_indices.tolist()):
                if chosen_idx >= 0:
                    chosen_masses.append(float(policy_probs[row, chosen_idx]))

            if update:
                update_result = self.service.model.train_puct_batch(
                    batch,
                    learning_rate=self.learning_rate,
                )
                updated_batches += int(bool(update_result.get("updated", True)))

        elapsed = max(perf_counter() - start, 1e-9)
        return PolicyValueEpochSummary(
            epoch_index=epoch_index,
            example_count=len(examples),
            policy_loss=float(np.mean(policy_losses, dtype=np.float32)) if policy_losses else 0.0,
            value_loss=float(np.mean(value_losses, dtype=np.float32)) if value_losses else 0.0,
            mean_frontier_size=float(np.mean(frontier_sizes, dtype=np.float32)) if frontier_sizes else 0.0,
            mean_chosen_action_mass=float(np.mean(chosen_masses, dtype=np.float32)) if chosen_masses else 0.0,
            updated_batches=updated_batches,
            throughput_examples_per_sec=len(examples) / elapsed,
        )

    def run(
        self,
        examples: Sequence[CombatPuctTargetExample],
        *,
        epochs: int,
        update: bool = True,
    ) -> list[PolicyValueEpochSummary]:
        summaries: list[PolicyValueEpochSummary] = []
        for epoch_index in range(epochs):
            summaries.append(
                self.run_epoch(examples, epoch_index=epoch_index, update=update)
            )
        return summaries

    @staticmethod
    def _softmax_masked(
        scores: npt.NDArray[np.float32],
        mask: npt.NDArray[np.bool_],
    ) -> npt.NDArray[np.float32]:
        masked = np.where(mask, scores, np.float32(-1e9))
        shifted = masked - np.max(masked, axis=1, keepdims=True)
        weights = np.exp(shifted, dtype=np.float32)
        weights = np.where(mask, weights, np.float32(0.0))
        totals = np.sum(weights, axis=1, keepdims=True)
        totals = np.where(totals > 0.0, totals, np.float32(1.0))
        return np.asarray(weights / totals, dtype=np.float32)
