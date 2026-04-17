"""Top-level configs for the combat-first training rebuild."""

from __future__ import annotations

from dataclasses import dataclass, field

from .shared_memory import SharedMemoryConfig


@dataclass(frozen=True)
class TrainingTopology:
    actor_workers: int = 12
    inference_workers: int = 2
    trainer_workers: int = 1
    target_memory_gb: float = 19.0
    require_no_sustained_swap: bool = True


@dataclass(frozen=True)
class SearchConfig:
    root_simulations: int = 384
    max_depth: int = 32
    cpuct: float = 1.5
    dirichlet_alpha: float = 0.25
    dirichlet_epsilon: float = 0.2
    frontier_capacity: int = 12


@dataclass(frozen=True)
class CombatModelConfig:
    d_model: int = 256
    trunk_layers: int = 8
    attention_heads: int = 8
    mlp_ratio: float = 4.0
    dropout: float = 0.0
    candidate_head_dim: int = 128
    outcome_head_dim: int = 128
    use_set_pooling: bool = True

    @property
    def token_dim(self) -> int:
        """Compatibility alias for the older name used by smoke tooling."""

        return self.d_model


@dataclass(frozen=True)
class TrainingStackConfig:
    topology: TrainingTopology = field(default_factory=TrainingTopology)
    shared_memory: SharedMemoryConfig = field(default_factory=SharedMemoryConfig)
    search: SearchConfig = field(default_factory=SearchConfig)
    model: CombatModelConfig = field(default_factory=CombatModelConfig)
