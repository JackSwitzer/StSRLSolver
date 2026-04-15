"""Top-level configs for the combat-first training rebuild."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class TrainingTopology:
    actor_workers: int = 6
    inference_workers: int = 1
    trainer_workers: int = 1
    target_memory_gb: float = 20.0
    require_no_sustained_swap: bool = True


@dataclass(frozen=True)
class SearchConfig:
    root_simulations: int = 256
    max_depth: int = 24
    cpuct: float = 1.35
    dirichlet_alpha: float = 0.30
    dirichlet_epsilon: float = 0.25
    frontier_capacity: int = 8


@dataclass(frozen=True)
class CombatModelConfig:
    token_dim: int = 192
    trunk_layers: int = 8
    attention_heads: int = 6
    mlp_ratio: float = 4.0
    dropout: float = 0.0
    candidate_head_dim: int = 128
    outcome_head_dim: int = 128
    use_set_pooling: bool = True


@dataclass(frozen=True)
class TrainingStackConfig:
    topology: TrainingTopology = field(default_factory=TrainingTopology)
    search: SearchConfig = field(default_factory=SearchConfig)
    model: CombatModelConfig = field(default_factory=CombatModelConfig)
