"""Training-side utilities migrated from the legacy StSRLSolver wrapper."""

from .combat_calculator import CombatCalculator, CombatFeatures, Enemy, PlayerState, Stance
from .enemy_database import ENCOUNTERS, ENEMIES, EncounterInfo, EnemyInfo
from .kill_calculator import KillCalculator, can_kill_this_turn, get_kill_line
from .line_evaluator import (
    ActionType,
    LineOutcome,
    LineSimulator,
    SimulatedEnemy,
    SimulatedPlayer,
)
from .episode_logger import EpisodeLog, EpisodeStep, collect_episodes
from .gym_env import StsEnv, StsVecEnv
from .mcts import MCTS, MCTSNode, CombatMCTS
from .planner import StrategicPlanner, StSAgent
from .policy_net import PolicyValueNet
from .strategic_features import StrategicState, extract_strategic_features, strategic_state_to_vector
from .train import Trainer, RolloutBuffer

__all__ = [
    "ActionType",
    "CombatCalculator",
    "CombatFeatures",
    "CombatMCTS",
    "ENCOUNTERS",
    "ENEMIES",
    "EncounterInfo",
    "Enemy",
    "EnemyInfo",
    "EpisodeLog",
    "EpisodeStep",
    "KillCalculator",
    "LineOutcome",
    "LineSimulator",
    "MCTS",
    "MCTSNode",
    "PlayerState",
    "PolicyValueNet",
    "RolloutBuffer",
    "SimulatedEnemy",
    "SimulatedPlayer",
    "Stance",
    "StrategicPlanner",
    "StSAgent",
    "StsEnv",
    "StsVecEnv",
    "StrategicState",
    "Trainer",
    "can_kill_this_turn",
    "collect_episodes",
    "extract_strategic_features",
    "get_kill_line",
    "strategic_state_to_vector",
]
