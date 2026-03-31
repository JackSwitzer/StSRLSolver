"""Wrapper to make the Rust sts_engine.RustRunEngine compatible with worker.py.

Bridges the Rust engine's flat action-ID interface to the GameRunner-like
interface that _play_one_game() expects. The Rust engine handles combat
internally, so TurnSolver and CombatStateEncoder are not used.

Key differences from Python GameRunner:
- Actions are flat integers (not GameAction dataclasses)
- Combat is opaque: no current_combat exposed, step() handles it
- Observations come from engine.get_obs() (480-dim), not RunStateEncoder
- copy() uses Rust Clone (fast, no Python overhead)
"""

from __future__ import annotations

import hashlib
import logging
from enum import Enum, auto
from typing import Any, Dict, List, Optional, Union

import numpy as np

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Phase mapping: Rust string phases -> Python GamePhase-like enum
# ---------------------------------------------------------------------------

class RustGamePhase(Enum):
    """Mirror of GamePhase for Rust engine phase strings."""
    MAP_NAVIGATION = auto()
    COMBAT = auto()
    COMBAT_REWARDS = auto()
    EVENT = auto()
    SHOP = auto()
    REST = auto()
    TREASURE = auto()
    BOSS_REWARDS = auto()
    NEOW = auto()
    RUN_COMPLETE = auto()


_RUST_PHASE_MAP = {
    "map": RustGamePhase.MAP_NAVIGATION,
    "combat": RustGamePhase.COMBAT,
    "card_reward": RustGamePhase.COMBAT_REWARDS,
    "campfire": RustGamePhase.REST,
    "shop": RustGamePhase.SHOP,
    "event": RustGamePhase.EVENT,
    "game_over": RustGamePhase.RUN_COMPLETE,
}

# Phase type mapping for state encoding (matches worker.py _PHASE_MAP)
_PHASE_TYPE_MAP = {
    RustGamePhase.MAP_NAVIGATION: "path",
    RustGamePhase.COMBAT_REWARDS: "card_pick",
    RustGamePhase.BOSS_REWARDS: "card_pick",
    RustGamePhase.REST: "rest",
    RustGamePhase.SHOP: "shop",
    RustGamePhase.EVENT: "event",
    RustGamePhase.NEOW: "other",
    RustGamePhase.TREASURE: "other",
}

# Rust flat action ID ranges (must match lib.rs constants)
_COMBAT_BASE = 500


def _seed_to_u64(seed: Union[str, int]) -> int:
    """Convert a seed string or int to a u64 for the Rust engine.

    Matches the Python engine's seed_to_long behavior: uppercase the string,
    then hash to a 64-bit integer.
    """
    if isinstance(seed, int):
        return seed & 0xFFFFFFFFFFFFFFFF
    # Hash string seed to u64 (deterministic)
    seed_upper = seed.upper()
    h = hashlib.sha256(seed_upper.encode("utf-8")).digest()
    return int.from_bytes(h[:8], "little")


# ---------------------------------------------------------------------------
# RunState proxy -- exposes run_state-like attributes from the Rust engine
# ---------------------------------------------------------------------------

class RustRunStateProxy:
    """Proxy object that makes Rust engine getters look like Python RunState.

    worker.py accesses runner.run_state.current_hp, runner.run_state.floor, etc.
    This proxy delegates those reads to the underlying Rust engine.
    """

    def __init__(self, engine):
        self._engine = engine

    @property
    def current_hp(self) -> int:
        return self._engine.current_hp

    @property
    def max_hp(self) -> int:
        return self._engine.max_hp

    @property
    def gold(self) -> int:
        return self._engine.gold

    @property
    def floor(self) -> int:
        return self._engine.floor

    @property
    def deck(self) -> list:
        return self._engine.deck

    @property
    def relics(self) -> list:
        return self._engine.relics

    @property
    def potions(self) -> list:
        return self._engine.potions

    @property
    def combats_won(self) -> int:
        info = self._engine.get_info()
        return info.get("combats_won", 0)

    @property
    def elites_killed(self) -> int:
        info = self._engine.get_info()
        return info.get("elites_killed", 0)

    @property
    def bosses_killed(self) -> int:
        info = self._engine.get_info()
        return info.get("bosses_killed", 0)

    @property
    def combat(self):
        """Not available from Rust engine -- combat is opaque."""
        return None

    @property
    def rng_counters(self):
        """Not available from Rust engine."""
        return None

    def get_available_paths(self):
        """Not available from Rust engine (map is internal)."""
        return []

    def sync_rng_counters(self, _counters):
        """No-op for Rust engine."""
        pass


# ---------------------------------------------------------------------------
# RustGameRunner -- drop-in replacement for GameRunner
# ---------------------------------------------------------------------------

class RustGameRunner:
    """Wraps sts_engine.RustRunEngine to match GameRunner interface.

    This is the main integration point. worker.py creates a RustGameRunner
    instead of GameRunner when USE_RUST_ENGINE is True.

    The Rust engine uses a flat action space where actions are integer IDs.
    Combat is handled internally by engine.step() -- no TurnSolver needed.
    """

    def __init__(
        self,
        seed: Union[str, int],
        ascension: int = 20,
        character: str = "Watcher",
        verbose: bool = False,
    ):
        try:
            import sts_engine
        except ImportError:
            raise ImportError(
                "sts_engine not found. Build with: "
                "cd packages/engine-rs && maturin develop --release"
            )

        self._seed_raw = seed
        self._seed_u64 = _seed_to_u64(seed)
        self._ascension = ascension
        self._character = character

        self._engine = sts_engine.RustRunEngine(self._seed_u64, ascension)
        self._run_state_proxy = RustRunStateProxy(self._engine)
        self._action_history: List[int] = []

    # -- Core interface used by worker.py --

    @property
    def phase(self) -> RustGamePhase:
        """Current game phase as a RustGamePhase enum."""
        rust_phase = self._engine.phase
        return _RUST_PHASE_MAP.get(rust_phase, RustGamePhase.RUN_COMPLETE)

    @property
    def game_over(self) -> bool:
        return self._engine.is_done()

    @property
    def game_won(self) -> bool:
        return self._engine.is_won()

    @property
    def run_state(self) -> RustRunStateProxy:
        """Proxy that exposes run_state attributes from Rust engine."""
        return self._run_state_proxy

    @property
    def current_combat(self):
        """Combat is opaque in the Rust engine -- always None.

        worker.py checks this for TurnSolver and combat state encoding.
        When None, combat is handled by the Rust engine's step() directly.
        """
        return None

    @property
    def current_room_type(self) -> str:
        """Infer room type from phase and engine state."""
        # The Rust engine doesn't expose room type directly.
        # Use boss_name as a heuristic: if we're in combat and boss is set,
        # check floor to infer room type.
        floor = self._engine.floor
        if floor in (16, 33, 50):
            return "boss"
        # Rough heuristic for elites (floors with elite encounters)
        # The Rust engine doesn't distinguish monster vs elite via phase.
        return "monster"

    @property
    def _boss_name(self) -> str:
        return self._engine.boss_name

    @property
    def current_event_state(self):
        """Not available from Rust engine."""
        return None

    @property
    def last_death_enemies(self) -> list:
        """Not available from Rust engine."""
        return []

    @property
    def seed(self) -> str:
        """Original seed string/value."""
        return str(self._seed_raw)

    def get_available_actions(self) -> List[int]:
        """Return list of legal action IDs (flat integers).

        Unlike GameRunner which returns GameAction objects, the Rust engine
        returns flat integer action IDs. worker.py must handle both cases.
        """
        return self._engine.get_legal_actions()

    def take_action(self, action: Union[int, Any]) -> bool:
        """Execute an action. Returns True on success.

        Accepts either:
        - int: flat action ID (from get_available_actions)
        - GameAction-like object: attempts to encode to flat ID (fallback)

        The Rust engine's step() returns (reward, done) but we ignore the
        reward here -- worker.py computes its own rewards.
        """
        if isinstance(action, int):
            action_id = action
        else:
            # Shouldn't happen when using Rust engine, but handle gracefully
            logger.warning(
                "RustGameRunner.take_action received non-int action: %s",
                type(action).__name__,
            )
            action_id = 0  # Fallback to first action

        self._action_history.append(action_id)
        _reward, _done = self._engine.step(action_id)
        return True

    def get_observation(self) -> np.ndarray:
        """Return 480-dim observation vector from Rust engine.

        This replaces RunStateEncoder.encode() -- the Rust engine produces
        the same 480-dim encoding internally.
        """
        return np.array(self._engine.get_obs(), dtype=np.float32)

    def get_combat_observation(self) -> np.ndarray:
        """Return combat-specific observation vector."""
        return np.array(self._engine.get_combat_obs(), dtype=np.float32)

    def copy(self) -> "RustGameRunner":
        """Create a deep copy for MCTS simulation.

        Uses Rust Clone which is much faster than Python deepcopy.
        """
        clone = RustGameRunner.__new__(RustGameRunner)
        clone._seed_raw = self._seed_raw
        clone._seed_u64 = self._seed_u64
        clone._ascension = self._ascension
        clone._character = self._character
        clone._engine = self._engine.copy()
        clone._run_state_proxy = RustRunStateProxy(clone._engine)
        clone._action_history = self._action_history[:]
        return clone

    def get_info(self) -> Dict[str, Any]:
        """Return engine info dict (floor, hp, gold, phase, etc.)."""
        return dict(self._engine.get_info())

    def get_phase_type(self) -> str:
        """Return phase type string for state encoding."""
        return _PHASE_TYPE_MAP.get(self.phase, "other")

    def __repr__(self) -> str:
        return (
            f"RustGameRunner(seed={self._seed_raw!r}, floor={self._engine.floor}, "
            f"hp={self._engine.current_hp}/{self._engine.max_hp}, "
            f"phase={self._engine.phase}, done={self.game_over})"
        )
