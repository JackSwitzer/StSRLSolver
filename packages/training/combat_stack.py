"""Small combat-first training skeleton used by the initial scaffold tests."""

from __future__ import annotations

from dataclasses import dataclass, field

import numpy as np


@dataclass(frozen=True)
class CombatStateSummary:
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

    def as_vector(self) -> np.ndarray:
        stance_bucket = {
            "Neutral": 0.0,
            "Wrath": 1.0,
            "Calm": 2.0,
            "Divinity": 3.0,
        }.get(self.stance, -1.0)
        return np.asarray(
            [
                float(self.turn),
                float(self.hp),
                float(self.block),
                float(self.energy),
                float(self.hand_size),
                float(self.draw_pile_size),
                float(self.discard_pile_size),
                float(self.exhaust_pile_size),
                stance_bucket,
            ],
            dtype=np.float32,
        )


@dataclass(frozen=True)
class LegalCombatCandidate:
    action_id: str
    action_kind: str
    target_idx: int = -1
    features: tuple[float, ...] = ()
    legal: bool = True


@dataclass(frozen=True)
class CombatSearchRequest:
    request_id: str
    state: CombatStateSummary
    candidates: tuple[LegalCombatCandidate, ...]


@dataclass(frozen=True)
class CombatSearchConfig:
    top_k: int = 4
    require_legal_candidates: bool = True


@dataclass(frozen=True)
class SharedMemoryConfig:
    max_batch_size: int = 64
    max_candidates: int = 32


@dataclass(frozen=True)
class TrainingConfig:
    model_backend: str = "mlx"
    shared_memory: SharedMemoryConfig = field(default_factory=SharedMemoryConfig)
    combat_search: CombatSearchConfig = field(default_factory=CombatSearchConfig)


@dataclass(frozen=True)
class PackedCombatBatch:
    request_ids: tuple[str, ...]
    state_matrix: np.ndarray
    candidate_matrix: np.ndarray
    legal_mask: np.ndarray
    candidate_ids: tuple[tuple[str, ...], ...]


class CombatSharedMemoryBatcher:
    def __init__(self, *, max_batch_size: int, max_candidates: int = 32):
        self.max_batch_size = max_batch_size
        self.max_candidates = max_candidates

    def pack(self, requests: tuple[CombatSearchRequest, ...]) -> PackedCombatBatch:
        if len(requests) > self.max_batch_size:
            raise ValueError("request batch exceeds configured max_batch_size")
        request_ids = tuple(request.request_id for request in requests)
        state_matrix = np.stack([request.state.as_vector() for request in requests], axis=0)
        max_candidates = max((len(request.candidates) for request in requests), default=0)
        max_features = max(
            (len(candidate.features) for request in requests for candidate in request.candidates),
            default=0,
        )
        candidate_matrix = np.zeros(
            (len(requests), max_candidates, max_features),
            dtype=np.float32,
        )
        legal_mask = np.zeros((len(requests), max_candidates), dtype=bool)
        candidate_ids: list[tuple[str, ...]] = []
        for request_idx, request in enumerate(requests):
            ids: list[str] = []
            for candidate_idx, candidate in enumerate(request.candidates):
                ids.append(candidate.action_id)
                legal_mask[request_idx, candidate_idx] = candidate.legal
                candidate_matrix[request_idx, candidate_idx, : len(candidate.features)] = candidate.features
            candidate_ids.append(tuple(ids))
        return PackedCombatBatch(
            request_ids=request_ids,
            state_matrix=state_matrix,
            candidate_matrix=candidate_matrix,
            legal_mask=legal_mask,
            candidate_ids=tuple(candidate_ids),
        )


@dataclass(frozen=True)
class CombatSearchResult:
    request_id: str
    chosen_action_id: str
    ranked_action_ids: tuple[str, ...]
    ranked_scores: tuple[float, ...]


@dataclass(frozen=True)
class LinearCombatModel:
    state_scale: float = 1.0
    candidate_scale: float = 1.0
    legal_bias: float = 0.0

    def score(self, state: CombatStateSummary, candidate: LegalCombatCandidate) -> float:
        return (
            state.as_vector().sum() * self.state_scale
            + sum(candidate.features) * self.candidate_scale
            + (self.legal_bias if candidate.legal else 0.0)
        )


class CombatInferenceService:
    def __init__(self, model: LinearCombatModel, config: CombatSearchConfig):
        self.model = model
        self.config = config

    @classmethod
    def build(cls, model: LinearCombatModel, config: CombatSearchConfig) -> "CombatInferenceService":
        return cls(model, config)

    def choose_action(self, request: CombatSearchRequest) -> CombatSearchResult:
        candidates = [
            candidate
            for candidate in request.candidates
            if candidate.legal or not self.config.require_legal_candidates
        ]
        ranked = sorted(
            ((candidate.action_id, self.model.score(request.state, candidate)) for candidate in candidates),
            key=lambda item: item[1],
            reverse=True,
        )
        if not ranked:
            raise ValueError("no candidates available to score")
        top_ranked = ranked[: self.config.top_k]
        return CombatSearchResult(
            request_id=request.request_id,
            chosen_action_id=top_ranked[0][0],
            ranked_action_ids=tuple(action_id for action_id, _ in top_ranked),
            ranked_scores=tuple(score for _, score in top_ranked),
        )
