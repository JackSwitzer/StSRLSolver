"""Shared data loading utilities for trajectory and combat .npz files.

Consolidates duplicated loading logic from strategic_trainer, training_runner,
pretrain_bc, pretrain_combat, and offline_data into reusable functions.

All dimension/format constants come from training_config.py.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterator, List, Optional, Sequence, Tuple

import numpy as np

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# File discovery
# ---------------------------------------------------------------------------

def find_trajectory_files(
    dirs: Sequence[Path],
    min_floor: int = 0,
    sort_by: str = "floor_desc",
) -> List[Path]:
    """Find trajectory .npz files across multiple directories.

    Args:
        dirs: Directories to search (non-existent dirs are silently skipped).
        min_floor: Minimum floor to include (parsed from filename).
        sort_by: "floor_desc" (highest floor first), "floor_asc", or "mtime_desc".

    Returns:
        Sorted list of Path objects.
    """
    files: List[Path] = []
    for d in dirs:
        d = Path(d)
        if not d.exists():
            continue
        files.extend(d.glob("traj_F*.npz"))

    if min_floor > 0:
        files = [f for f in files if parse_floor_from_filename(f) >= min_floor]

    if sort_by == "floor_desc":
        files.sort(key=lambda p: p.stem, reverse=True)
    elif sort_by == "floor_asc":
        files.sort(key=lambda p: p.stem)
    elif sort_by == "mtime_desc":
        files.sort(key=lambda p: p.stat().st_mtime, reverse=True)

    return files


def find_combat_files(dirs: Sequence[Path]) -> List[Path]:
    """Find combat .npz files across multiple directories (recursive).

    Searches for combat_data/ subdirectories and collects combat_*.npz files.

    Args:
        dirs: Root directories to search recursively.

    Returns:
        List of combat .npz file paths.
    """
    files: List[Path] = []
    for d in dirs:
        d = Path(d)
        if not d.exists():
            continue
        for combat_dir in d.rglob("combat_data"):
            files.extend(combat_dir.glob("combat_*.npz"))
    return files


def parse_floor_from_filename(path: Path) -> int:
    """Extract floor number from trajectory filename.

    Handles both formats:
      - traj_F{NN}_{seed}.npz  (e.g. traj_F16_Train_1021.npz)
      - traj_{NNNNNN}_F{NN}.npz (e.g. traj_000001_F06.npz)

    Returns 0 if parsing fails.
    """
    stem = path.stem
    parts = stem.split("_")
    # Format 1: traj_F{NN}_...
    if len(parts) >= 2 and parts[1].startswith("F"):
        try:
            return int(parts[1][1:])
        except ValueError:
            pass
    # Format 2: traj_{num}_F{NN}
    for part in parts:
        if part.startswith("F") and len(part) > 1:
            try:
                return int(part[1:])
            except ValueError:
                pass
    return 0


# ---------------------------------------------------------------------------
# Trajectory loading
# ---------------------------------------------------------------------------

@dataclass
class TrajectoryData:
    """Loaded trajectory arrays with validated dimensions."""
    obs: np.ndarray          # [T, obs_dim] float32
    masks: np.ndarray        # [T, action_dim] bool
    actions: np.ndarray      # [T] int32
    rewards: np.ndarray      # [T] float32
    dones: np.ndarray        # [T] bool
    values: np.ndarray       # [T] float32
    log_probs: np.ndarray    # [T] float32
    final_floors: np.ndarray # [T] float32
    cleared_act1: np.ndarray # [T] float32
    floor: int               # max floor reached
    source_path: Path        # original file path


def load_trajectory_file(
    path: Path,
    expected_obs_dim: Optional[int] = None,
    expected_action_dim: Optional[int] = None,
) -> Optional[TrajectoryData]:
    """Load a single trajectory .npz file with dimension validation.

    Validates obs_dim matches expected (filters old encoder versions).
    Pads action masks if smaller than expected_action_dim.

    Args:
        path: Path to the .npz file.
        expected_obs_dim: Required observation dimension. If set, files with
            mismatched obs_dim are rejected (returns None).
        expected_action_dim: Required action dimension. Masks are padded if
            smaller, rejected if larger.

    Returns:
        TrajectoryData if valid, None if rejected or corrupt.
    """
    try:
        data = np.load(path)
    except Exception as e:
        logger.warning("Failed to load %s: %s", path.name, e)
        return None

    required_keys = {"obs", "masks", "actions", "rewards", "dones"}
    missing = required_keys - set(data.keys())
    if missing:
        logger.warning("Trajectory %s missing keys: %s", path.name, missing)
        return None

    obs = data["obs"]
    if obs.ndim != 2 or len(obs) < 1:
        logger.warning("Trajectory %s has invalid obs shape: %s", path.name, obs.shape)
        return None

    # Dimension validation
    if expected_obs_dim is not None and obs.shape[1] != expected_obs_dim:
        logger.debug(
            "Skipping %s: obs_dim=%d, expected=%d",
            path.name, obs.shape[1], expected_obs_dim,
        )
        return None

    T = len(obs)
    masks = data["masks"]

    # Pad masks if needed
    if expected_action_dim is not None:
        if masks.shape[1] < expected_action_dim:
            pad_width = expected_action_dim - masks.shape[1]
            masks = np.pad(masks, ((0, 0), (0, pad_width)), constant_values=False)
        elif masks.shape[1] > expected_action_dim:
            logger.warning(
                "Trajectory %s has mask_dim=%d > expected=%d, skipping",
                path.name, masks.shape[1], expected_action_dim,
            )
            return None

    floor = parse_floor_from_filename(path)
    if "floor" in data and len(data["floor"]) > 0:
        floor = int(data["floor"][0])

    return TrajectoryData(
        obs=obs.astype(np.float32),
        masks=masks.astype(np.bool_),
        actions=data["actions"][:T].astype(np.int32),
        rewards=data["rewards"][:T].astype(np.float32),
        dones=data["dones"][:T].astype(np.bool_),
        values=data.get("values", np.zeros(T, dtype=np.float32))[:T].astype(np.float32),
        log_probs=data.get("log_probs", np.zeros(T, dtype=np.float32))[:T].astype(np.float32),
        final_floors=data.get("final_floors", np.full(T, floor / 55.0, dtype=np.float32))[:T].astype(np.float32),
        cleared_act1=data.get("cleared_act1", np.zeros(T, dtype=np.float32))[:T].astype(np.float32),
        floor=floor,
        source_path=path,
    )


def load_trajectories(
    dirs: Sequence[Path],
    max_transitions: int = 48000,
    min_floor: int = 0,
    expected_obs_dim: Optional[int] = None,
    expected_action_dim: Optional[int] = None,
) -> Tuple[List[TrajectoryData], int]:
    """Load trajectory files from multiple directories.

    Args:
        dirs: Directories to search for trajectory .npz files.
        max_transitions: Maximum total transitions to load.
        min_floor: Minimum floor filter.
        expected_obs_dim: Filter by observation dimension.
        expected_action_dim: Expected action dimension (pads masks).

    Returns:
        Tuple of (list of TrajectoryData, total transitions loaded).
    """
    files = find_trajectory_files(dirs, min_floor=min_floor)
    trajectories: List[TrajectoryData] = []
    loaded = 0
    files_failed = 0

    for f in files:
        if loaded >= max_transitions:
            break

        traj = load_trajectory_file(
            f,
            expected_obs_dim=expected_obs_dim,
            expected_action_dim=expected_action_dim,
        )
        if traj is None:
            files_failed += 1
            continue

        remaining = max_transitions - loaded
        if len(traj.obs) > remaining:
            # Truncate to fit budget
            traj = TrajectoryData(
                obs=traj.obs[:remaining],
                masks=traj.masks[:remaining],
                actions=traj.actions[:remaining],
                rewards=traj.rewards[:remaining],
                dones=traj.dones[:remaining],
                values=traj.values[:remaining],
                log_probs=traj.log_probs[:remaining],
                final_floors=traj.final_floors[:remaining],
                cleared_act1=traj.cleared_act1[:remaining],
                floor=traj.floor,
                source_path=traj.source_path,
            )

        trajectories.append(traj)
        loaded += len(traj.obs)

    logger.info(
        "load_trajectories: %d transitions from %d files (%d failed, %d skipped dim mismatch)",
        loaded, len(trajectories), files_failed,
        len(files) - len(trajectories) - files_failed,
    )
    return trajectories, loaded


# ---------------------------------------------------------------------------
# Combat data loading
# ---------------------------------------------------------------------------

@dataclass
class CombatSample:
    """Single combat observation with outcome."""
    combat_obs: np.ndarray  # [obs_dim] float32
    won: bool


def load_combat_data(dirs: Sequence[Path]) -> List[Dict[str, Any]]:
    """Load combat .npz files into dicts compatible with train_combat_net().

    Args:
        dirs: Root directories to search recursively for combat_data/ subdirs.

    Returns:
        List of {"combat_obs": ndarray, "won": bool} dicts.
    """
    files = find_combat_files(dirs)
    results: List[Dict[str, Any]] = []
    failed = 0

    for f in files:
        try:
            data = np.load(f)
            results.append({
                "combat_obs": data["combat_obs"],
                "won": bool(data["won"]),
            })
        except Exception as e:
            logger.warning("Failed to load combat file %s: %s", f.name, e)
            failed += 1

    logger.info("load_combat_data: %d samples from %d files (%d failed)", len(results), len(files), failed)
    return results


# ---------------------------------------------------------------------------
# Batch iterator
# ---------------------------------------------------------------------------

def batch_iterator(
    trajectories: List[TrajectoryData],
    batch_size: int,
    shuffle: bool = True,
) -> Iterator[Dict[str, np.ndarray]]:
    """Yield batches of transitions from loaded trajectories.

    Concatenates all trajectories, optionally shuffles, and yields
    dicts with keys: obs, masks, actions, rewards, dones, values,
    log_probs, final_floors, cleared_act1.

    Args:
        trajectories: List of loaded TrajectoryData.
        batch_size: Number of transitions per batch.
        shuffle: Whether to shuffle transitions.

    Yields:
        Dict of numpy arrays, each with shape [batch_size, ...].
    """
    if not trajectories:
        return

    obs = np.concatenate([t.obs for t in trajectories], axis=0)
    masks = np.concatenate([t.masks for t in trajectories], axis=0)
    actions = np.concatenate([t.actions for t in trajectories], axis=0)
    rewards = np.concatenate([t.rewards for t in trajectories], axis=0)
    dones = np.concatenate([t.dones for t in trajectories], axis=0)
    values = np.concatenate([t.values for t in trajectories], axis=0)
    log_probs = np.concatenate([t.log_probs for t in trajectories], axis=0)
    final_floors = np.concatenate([t.final_floors for t in trajectories], axis=0)
    cleared_act1 = np.concatenate([t.cleared_act1 for t in trajectories], axis=0)

    n = len(obs)
    indices = np.random.permutation(n) if shuffle else np.arange(n)

    for start in range(0, n, batch_size):
        end = min(start + batch_size, n)
        idx = indices[start:end]
        yield {
            "obs": obs[idx],
            "masks": masks[idx],
            "actions": actions[idx],
            "rewards": rewards[idx],
            "dones": dones[idx],
            "values": values[idx],
            "log_probs": log_probs[idx],
            "final_floors": final_floors[idx],
            "cleared_act1": cleared_act1[idx],
        }


# ---------------------------------------------------------------------------
# Train/validation split
# ---------------------------------------------------------------------------

def train_val_split(
    trajectories: List[TrajectoryData],
    val_ratio: float = 0.1,
) -> Tuple[List[TrajectoryData], List[TrajectoryData]]:
    """Split trajectories into train and validation sets.

    Splits by trajectory (not by transition) to prevent data leakage
    from the same game appearing in both sets.

    Args:
        trajectories: List of loaded TrajectoryData.
        val_ratio: Fraction of trajectories for validation.

    Returns:
        (train_trajectories, val_trajectories)
    """
    n = len(trajectories)
    n_val = max(1, int(n * val_ratio)) if n > 1 else 0

    indices = np.random.permutation(n)
    val_idx = set(indices[:n_val].tolist())

    train = [t for i, t in enumerate(trajectories) if i not in val_idx]
    val = [t for i, t in enumerate(trajectories) if i in val_idx]
    return train, val


# ---------------------------------------------------------------------------
# Quality checks
# ---------------------------------------------------------------------------

@dataclass
class QualityReport:
    """Summary of data quality checks."""
    total_files: int
    total_transitions: int
    valid_files: int
    valid_transitions: int
    dim_mismatch_files: int
    nan_reward_files: int
    extreme_reward_files: int
    invalid_action_transitions: int
    obs_dim_distribution: Dict[int, int]
    floor_distribution: Dict[int, int]

    @property
    def usable_pct(self) -> float:
        if self.total_files == 0:
            return 0.0
        return self.valid_files / self.total_files * 100


def check_trajectory_quality(
    dirs: Sequence[Path],
    expected_obs_dim: int = 480,
    expected_action_dim: int = 512,
    extreme_reward_threshold: float = 100.0,
) -> QualityReport:
    """Run quality checks on trajectory files.

    Checks: dimension consistency, NaN rewards, extreme rewards,
    action mask validity (invalid actions chosen).

    Args:
        dirs: Directories to scan.
        expected_obs_dim: Current model obs dimension.
        expected_action_dim: Current model action dimension.
        extreme_reward_threshold: Flag rewards with abs > this.

    Returns:
        QualityReport with detailed stats.
    """
    files = find_trajectory_files(dirs)

    obs_dims: Dict[int, int] = {}
    floor_dist: Dict[int, int] = {}
    total_transitions = 0
    valid_files = 0
    valid_transitions = 0
    dim_mismatch = 0
    nan_rewards = 0
    extreme_rewards = 0
    invalid_actions = 0

    for f in files:
        try:
            data = np.load(f)
            obs = data["obs"]
            T = len(obs)
            total_transitions += T
            obs_dim = obs.shape[1]
            obs_dims[obs_dim] = obs_dims.get(obs_dim, 0) + 1

            floor = parse_floor_from_filename(f)
            floor_dist[floor] = floor_dist.get(floor, 0) + 1

            if obs_dim != expected_obs_dim:
                dim_mismatch += 1
                continue

            rewards = data["rewards"]
            if np.any(np.isnan(rewards)):
                nan_rewards += 1
                continue

            if np.any(np.abs(rewards) > extreme_reward_threshold):
                extreme_rewards += 1

            # Check action mask validity
            if "masks" in data and "actions" in data:
                masks = data["masks"]
                actions = data["actions"]
                for i in range(min(T, len(masks), len(actions))):
                    if actions[i] < len(masks[i]) and not masks[i][actions[i]]:
                        invalid_actions += 1

            valid_files += 1
            valid_transitions += T

        except Exception as e:
            logger.warning("Quality check failed for %s: %s", f.name, e)

    return QualityReport(
        total_files=len(files),
        total_transitions=total_transitions,
        valid_files=valid_files,
        valid_transitions=valid_transitions,
        dim_mismatch_files=dim_mismatch,
        nan_reward_files=nan_rewards,
        extreme_reward_files=extreme_rewards,
        invalid_action_transitions=invalid_actions,
        obs_dim_distribution=obs_dims,
        floor_distribution=floor_dist,
    )
