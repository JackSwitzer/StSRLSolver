"""Combat-first training package built on the Rust engine contract."""

from .benchmark import BenchmarkConfig, frontier_score
from .benchmarking import BenchmarkFrontierPoint, FrontierReport, FrontierWeights, build_frontier_report, pareto_frontier
from .bridge import (
    load_combat_training_state,
    load_training_schema_versions,
    parse_combat_training_state,
    parse_training_schema_versions,
)
from .combat_model import (
    CombatInferenceResult,
    CombatStateSummary,
    LegalCombatCandidate,
    LinearCombatModel,
    MLXCombatModel,
)
from .config import CombatModelConfig, SearchConfig, TrainingStackConfig, TrainingTopology
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
from .corpus import PreparedCorpusRequest, build_phase1_requests, default_watcher_a0_act1_corpus_plan
from .inference_service import (
    CombatInferenceService,
    CombatPreferenceExample,
    CombatSearchConfig,
    OvernightReanalysisLoop,
    ReanalysisEpochSummary,
    TrainingConfig,
)
from .selector import FrontierSelection, rank_frontier_lines, select_frontier, select_frontier_line
from .shared_memory import CombatSearchRequest, CombatSharedMemoryBatch, CombatSharedMemoryBatcher, SharedMemoryConfig

__all__ = [
    "BenchmarkConfig",
    "BenchmarkFrontierPoint",
    "CombatFrontierLine",
    "CombatFrontierSummary",
    "CombatInferenceResult",
    "CombatInferenceService",
    "CombatModelConfig",
    "CombatObservation",
    "CombatOutcomeVector",
    "CombatPreferenceExample",
    "CombatSearchConfig",
    "CombatSearchRequest",
    "CombatSharedMemoryBatch",
    "CombatSharedMemoryBatcher",
    "CombatStateSummary",
    "CombatTrainingState",
    "FrontierReport",
    "FrontierSelection",
    "FrontierWeights",
    "LegalActionCandidate",
    "LegalCombatCandidate",
    "LinearCombatModel",
    "MLXCombatModel",
    "OvernightReanalysisLoop",
    "PreparedCorpusRequest",
    "ReanalysisEpochSummary",
    "RestrictionBuiltin",
    "RestrictionPolicy",
    "SearchConfig",
    "SharedMemoryConfig",
    "TrainingConfig",
    "TrainingSchemaVersions",
    "TrainingStackConfig",
    "TrainingTopology",
    "build_frontier_report",
    "build_phase1_requests",
    "default_watcher_a0_act1_corpus_plan",
    "frontier_score",
    "load_combat_training_state",
    "load_training_schema_versions",
    "pareto_frontier",
    "parse_combat_training_state",
    "parse_training_schema_versions",
    "rank_frontier_lines",
    "select_frontier",
    "select_frontier_line",
]
