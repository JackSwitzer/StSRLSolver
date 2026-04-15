"""Tests for packages/training/data_utils.py — shared data loading utilities."""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest


@pytest.fixture
def tmp_traj_dir(tmp_path: Path) -> Path:
    """Create a temp directory with sample trajectory .npz files."""
    d = tmp_path / "trajectories"
    d.mkdir()

    # Valid 480-dim trajectory at floor 14
    T = 10
    np.savez_compressed(
        d / "traj_F14_seed123.npz",
        obs=np.random.randn(T, 480).astype(np.float32),
        masks=np.ones((T, 512), dtype=np.bool_),
        actions=np.zeros(T, dtype=np.int32),
        rewards=np.random.randn(T).astype(np.float32),
        dones=np.zeros(T, dtype=np.bool_),
        values=np.random.randn(T).astype(np.float32),
        log_probs=np.random.randn(T).astype(np.float32),
        final_floors=np.full(T, 14.0 / 55.0, dtype=np.float32),
        cleared_act1=np.ones(T, dtype=np.float32),
        floor=np.array([14]),
    )

    # Valid 480-dim trajectory at floor 8
    T2 = 5
    np.savez_compressed(
        d / "traj_F08_seed456.npz",
        obs=np.random.randn(T2, 480).astype(np.float32),
        masks=np.ones((T2, 512), dtype=np.bool_),
        actions=np.zeros(T2, dtype=np.int32),
        rewards=np.random.randn(T2).astype(np.float32),
        dones=np.zeros(T2, dtype=np.bool_),
        values=np.random.randn(T2).astype(np.float32),
        log_probs=np.random.randn(T2).astype(np.float32),
        final_floors=np.full(T2, 8.0 / 55.0, dtype=np.float32),
        cleared_act1=np.zeros(T2, dtype=np.float32),
        floor=np.array([8]),
    )

    # Old 540-dim trajectory (should be filtered)
    T3 = 3
    np.savez_compressed(
        d / "traj_F10_old_seed.npz",
        obs=np.random.randn(T3, 540).astype(np.float32),
        masks=np.ones((T3, 512), dtype=np.bool_),
        actions=np.zeros(T3, dtype=np.int32),
        rewards=np.random.randn(T3).astype(np.float32),
        dones=np.zeros(T3, dtype=np.bool_),
    )

    return d


@pytest.fixture
def tmp_combat_dir(tmp_path: Path) -> Path:
    """Create a temp directory with sample combat .npz files."""
    combat = tmp_path / "run1" / "combat_data"
    combat.mkdir(parents=True)

    np.savez_compressed(
        combat / "combat_000001.npz",
        combat_obs=np.random.randn(298).astype(np.float32),
        won=np.array(True),
    )
    np.savez_compressed(
        combat / "combat_000002.npz",
        combat_obs=np.random.randn(298).astype(np.float32),
        won=np.array(False),
    )

    return tmp_path


class TestParseFloor:
    def test_format1_standard(self):
        from packages.training.data_utils import parse_floor_from_filename
        assert parse_floor_from_filename(Path("traj_F14_Train_210.npz")) == 14

    def test_format1_two_digit(self):
        from packages.training.data_utils import parse_floor_from_filename
        assert parse_floor_from_filename(Path("traj_F08_seed.npz")) == 8

    def test_format2_numbered(self):
        from packages.training.data_utils import parse_floor_from_filename
        assert parse_floor_from_filename(Path("traj_000001_F06.npz")) == 6

    def test_unparseable(self):
        from packages.training.data_utils import parse_floor_from_filename
        assert parse_floor_from_filename(Path("unknown_file.npz")) == 0


class TestFindTrajectoryFiles:
    def test_finds_files(self, tmp_traj_dir):
        from packages.training.data_utils import find_trajectory_files
        files = find_trajectory_files([tmp_traj_dir])
        assert len(files) == 3

    def test_min_floor_filter(self, tmp_traj_dir):
        from packages.training.data_utils import find_trajectory_files
        files = find_trajectory_files([tmp_traj_dir], min_floor=10)
        assert len(files) == 2  # F14 and F10, not F08

    def test_sort_floor_desc(self, tmp_traj_dir):
        from packages.training.data_utils import find_trajectory_files
        files = find_trajectory_files([tmp_traj_dir], sort_by="floor_desc")
        floors = [f.stem for f in files]
        assert floors == sorted(floors, reverse=True)

    def test_nonexistent_dir_ignored(self, tmp_traj_dir):
        from packages.training.data_utils import find_trajectory_files
        files = find_trajectory_files([tmp_traj_dir, Path("/nonexistent")])
        assert len(files) == 3


class TestLoadTrajectoryFile:
    def test_loads_valid(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectory_file
        path = tmp_traj_dir / "traj_F14_seed123.npz"
        traj = load_trajectory_file(path)
        assert traj is not None
        assert traj.obs.shape == (10, 480)
        assert traj.masks.shape == (10, 512)
        assert traj.floor == 14

    def test_filters_wrong_dim(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectory_file
        path = tmp_traj_dir / "traj_F10_old_seed.npz"
        traj = load_trajectory_file(path, expected_obs_dim=480)
        assert traj is None

    def test_accepts_matching_dim(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectory_file
        path = tmp_traj_dir / "traj_F14_seed123.npz"
        traj = load_trajectory_file(path, expected_obs_dim=480)
        assert traj is not None

    def test_pads_small_masks(self, tmp_traj_dir):
        # Create file with smaller mask dim
        T = 5
        np.savez_compressed(
            tmp_traj_dir / "traj_F05_small.npz",
            obs=np.random.randn(T, 480).astype(np.float32),
            masks=np.ones((T, 256), dtype=np.bool_),
            actions=np.zeros(T, dtype=np.int32),
            rewards=np.zeros(T, dtype=np.float32),
            dones=np.zeros(T, dtype=np.bool_),
        )
        from packages.training.data_utils import load_trajectory_file
        traj = load_trajectory_file(
            tmp_traj_dir / "traj_F05_small.npz",
            expected_action_dim=512,
        )
        assert traj is not None
        assert traj.masks.shape == (5, 512)
        # Padded portion should be False
        assert not traj.masks[0, 256]

    def test_corrupt_file_returns_none(self, tmp_path):
        from packages.training.data_utils import load_trajectory_file
        bad = tmp_path / "traj_F01_bad.npz"
        bad.write_bytes(b"not a numpy file")
        assert load_trajectory_file(bad) is None


class TestLoadTrajectories:
    def test_loads_with_budget(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectories
        trajs, total = load_trajectories([tmp_traj_dir], max_transitions=8)
        assert total <= 8

    def test_filters_by_obs_dim(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectories
        trajs, total = load_trajectories(
            [tmp_traj_dir], expected_obs_dim=480, max_transitions=100,
        )
        # Should skip the 540-dim file
        for t in trajs:
            assert t.obs.shape[1] == 480

    def test_min_floor(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectories
        trajs, _ = load_trajectories(
            [tmp_traj_dir], min_floor=10, max_transitions=100,
        )
        for t in trajs:
            assert t.floor >= 10


class TestLoadCombatData:
    def test_loads_combat(self, tmp_combat_dir):
        from packages.training.data_utils import load_combat_data
        samples = load_combat_data([tmp_combat_dir])
        assert len(samples) == 2
        assert "combat_obs" in samples[0]
        assert "won" in samples[0]


class TestBatchIterator:
    def test_yields_batches(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectory_file, batch_iterator
        traj = load_trajectory_file(tmp_traj_dir / "traj_F14_seed123.npz")
        batches = list(batch_iterator([traj], batch_size=4))
        assert len(batches) == 3  # 10 transitions / 4 = 3 batches (4+4+2)
        assert batches[0]["obs"].shape[0] == 4
        assert batches[2]["obs"].shape[0] == 2

    def test_empty_input(self):
        from packages.training.data_utils import batch_iterator
        batches = list(batch_iterator([], batch_size=4))
        assert batches == []


class TestTrainValSplit:
    def test_split(self, tmp_traj_dir):
        from packages.training.data_utils import load_trajectories, train_val_split
        trajs, _ = load_trajectories([tmp_traj_dir], max_transitions=100)
        train, val = train_val_split(trajs, val_ratio=0.5)
        assert len(train) + len(val) == len(trajs)
        assert len(val) >= 1


class TestQualityReport:
    def test_report(self, tmp_traj_dir):
        from packages.training.data_utils import check_trajectory_quality
        report = check_trajectory_quality([tmp_traj_dir], expected_obs_dim=480)
        assert report.total_files == 3
        assert report.valid_files == 2  # 540-dim file is excluded
        assert report.dim_mismatch_files == 1
        assert report.nan_reward_files == 0
        assert report.invalid_action_transitions == 0
        assert 480 in report.obs_dim_distribution
        assert report.usable_pct == pytest.approx(66.67, rel=0.01)
