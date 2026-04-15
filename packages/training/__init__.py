"""Fresh combat-first training package built on the Rust engine contract."""

from .benchmark import BenchmarkConfig, BenchmarkReport, frontier_score
from .benchmarking import BenchmarkFrontierPoint, FrontierWeights, build_frontier_report, pareto_frontier
from .bridge import parse_combat_training_state, parse_training_schema_versions
from .combat_stack import (
    CombatInferenceService,
    CombatSearchConfig,
    CombatSearchRequest,
    CombatSharedMemoryBatcher,
    CombatStateSummary,
    LegalCombatCandidate,
    LinearCombatModel,
    TrainingConfig,
)
from .config import CombatModelConfig, SearchConfig, TrainingTopology
from .contracts import (
    CombatFrontierLine,
    CombatFrontierSummary,
    CombatObservation,
    CombatOutcomeVector,
    CombatTrainingState,
    LegalActionCandidate,
    RestrictionBuiltin,
    RestrictionPolicy,
    TrainingSchemaVersions,
)

__all__ = [
    "BenchmarkConfig",
    "BenchmarkFrontierPoint",
    "BenchmarkReport",
    "CombatInferenceService",
    "CombatFrontierLine",
    "CombatFrontierSummary",
    "CombatModelConfig",
    "CombatObservation",
    "CombatOutcomeVector",
    "CombatSearchConfig",
    "CombatSearchRequest",
    "CombatSharedMemoryBatcher",
    "CombatStateSummary",
    "CombatTrainingState",
    "LegalActionCandidate",
    "LegalCombatCandidate",
    "LinearCombatModel",
    "FrontierWeights",
    "RestrictionBuiltin",
    "RestrictionPolicy",
    "SearchConfig",
    "TrainingConfig",
    "TrainingSchemaVersions",
    "TrainingTopology",
    "build_frontier_report",
    "frontier_score",
    "parse_combat_training_state",
    "parse_training_schema_versions",
    "pareto_frontier",
]
