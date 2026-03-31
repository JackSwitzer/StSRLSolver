"""Tests for packages/training/data_tiers.py — data tiering, quality, and replay."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pytest


@pytest.fixture
def traj_dir(tmp_path: Path) -> Path:
    d = tmp_path / "trajectories"
    d.mkdir()

    def _make(name, floor, obs_dim=480, mask_dim=512, T=10, nan_reward=False):
        rewards = np.random.randn(T).astype(np.float32)
        if nan_reward:
            rewards[0] = np.nan
        np.savez_compressed(
            d / name,
            obs=np.random.randn(T, obs_dim).astype(np.float32),
            masks=np.ones((T, mask_dim), dtype=np.bool_),
            actions=np.zeros(T, dtype=np.int32),
            rewards=rewards,
            dones=np.zeros(T, dtype=np.bool_),
            values=np.random.randn(T).astype(np.float32),
            log_probs=np.random.randn(T).astype(np.float32),
            final_floors=np.full(T, floor / 55.0, dtype=np.float32),
            cleared_act1=np.ones(T, dtype=np.float32) if floor >= 17 else np.zeros(T, dtype=np.float32),
            floor=np.array([floor]),
        )

    _make("traj_F18_expert.npz", floor=18, T=50)
    _make("traj_F12_curated.npz", floor=12, T=30)
    _make("traj_F05_filtered.npz", floor=5, T=10)
    _make("traj_F10_olddim.npz", floor=10, obs_dim=540, T=8)
    _make("traj_F08_nan.npz", floor=8, nan_reward=True, T=5)
    return d


@pytest.fixture
def episodes_file(tmp_path: Path) -> Path:
    ep_path = tmp_path / "episodes.jsonl"
    episodes = [
        {"seed": "seed_001", "floor": 18, "won": False, "hp": 25, "max_hp": 80,
         "decisions": 120, "duration_s": 45.5, "total_reward": 15.3,
         "deck_final": ["Eruption", "Tantrum", "InnerPeace", "Crescendo",
                        "FlurryOfBlows", "Protect", "Strike", "Defend"],
         "relics_final": ["BurningBlood", "Vajra"],
         "combats": [{"floor": 1, "enemies": "JawWorm", "won": True, "hp_lost": 5}],
         "path_choices": ["M", "M", "E", "R"],
         "death_enemy": "Lagavulin", "death_room": "elite"},
        {"seed": "seed_002", "floor": 5, "won": False, "hp": 0, "max_hp": 80,
         "decisions": 30, "duration_s": 12.0, "total_reward": 2.1,
         "deck_final": ["Strike", "Strike", "Strike", "Defend", "Defend"],
         "relics_final": [], "combats": [], "path_choices": ["M"],
         "death_enemy": "Cultist", "death_room": "monster"},
        {"seed": "seed_003", "floor": 16, "won": False, "hp": 10, "max_hp": 80,
         "decisions": 100, "duration_s": 38.0, "total_reward": 12.0,
         "deck_final": ["Eruption", "Tantrum", "Ragnarok", "FlyingSleeves",
                        "Conclude", "SashWhip", "Strike"],
         "relics_final": ["BurningBlood"], "combats": [], "path_choices": [],
         "death_enemy": "", "death_room": ""},
    ]
    with open(ep_path, "w") as f:
        for ep in episodes:
            f.write(json.dumps(ep) + "\n")
    return ep_path


@pytest.fixture
def checkpoint_dir(tmp_path: Path) -> Path:
    d = tmp_path / "run_test"
    d.mkdir()
    for i in range(8):
        (d / f"checkpoint_{i:04d}.pt").write_bytes(b"fake checkpoint data " * 100)
    (d / "shutdown_checkpoint.pt").write_bytes(b"shutdown data")
    (d / "best_checkpoint.pt").write_bytes(b"best data")
    return d


class TestTierPipeline:
    def test_raw_accepts_all(self, traj_dir):
        from packages.training.data_tiers import DataTier, run_tier_pipeline
        assert len(run_tier_pipeline([traj_dir])[DataTier.RAW].accepted) == 5

    def test_filtered_rejects_wrong_dim_and_nan(self, traj_dir):
        from packages.training.data_tiers import DataTier, run_tier_pipeline
        f = run_tier_pipeline([traj_dir])[DataTier.FILTERED]
        assert len(f.accepted) == 3
        assert f.rejected == 2

    def test_curated_requires_floor_10(self, traj_dir):
        from packages.training.data_tiers import DataTier, run_tier_pipeline
        assert len(run_tier_pipeline([traj_dir])[DataTier.CURATED].accepted) == 2

    def test_expert_requires_floor_17(self, traj_dir):
        from packages.training.data_tiers import DataTier, run_tier_pipeline
        expert = run_tier_pipeline([traj_dir])[DataTier.EXPERT]
        assert len(expert.accepted) == 1
        assert "expert" in expert.accepted[0].name

    def test_tier_result_has_transitions(self, traj_dir):
        from packages.training.data_tiers import DataTier, run_tier_pipeline
        r = run_tier_pipeline([traj_dir])
        assert r[DataTier.EXPERT].total_transitions == 50
        assert r[DataTier.RAW].total_transitions > 0


class TestQualityScoring:
    def test_higher_floor_scores_higher(self, traj_dir):
        from packages.training.data_tiers import score_trajectories
        scores = score_trajectories([traj_dir], top_n=10)
        assert scores[0].floor >= scores[-1].floor

    def test_score_trajectory_components(self, traj_dir):
        from packages.training.data_utils import load_trajectory_file
        from packages.training.data_tiers import score_trajectory
        traj = load_trajectory_file(traj_dir / "traj_F18_expert.npz")
        q = score_trajectory(traj)
        assert q.floor == 18
        assert 0 < q.composite_score <= 1.0
        assert q.floor_score == pytest.approx(18 / 55.0, rel=0.01)
        assert q.decisions_count == 50

    def test_top_n_limits(self, traj_dir):
        from packages.training.data_tiers import score_trajectories
        assert len(score_trajectories([traj_dir], top_n=2)) <= 2


class TestDeckAnalysis:
    def test_classify_stance_cycling(self):
        from packages.training.data_tiers import classify_deck
        deck = ["Eruption", "Tantrum", "InnerPeace", "Crescendo", "FlurryOfBlows",
                "EmptyBody", "Protect", "Strike"]
        assert classify_deck(deck).name == "stance-cycling"

    def test_classify_attack_heavy(self):
        from packages.training.data_tiers import classify_deck
        deck = ["Eruption", "Tantrum", "Ragnarok", "FlyingSleeves", "Conclude",
                "SashWhip", "CrushJoints", "Strike", "Defend"]
        assert classify_deck(deck).name == "attack-heavy"

    def test_classify_empty(self):
        from packages.training.data_tiers import classify_deck
        assert classify_deck([]).name == "empty"

    def test_analyze_decks(self, episodes_file):
        from packages.training.data_tiers import analyze_decks
        stats = analyze_decks(episodes_file)
        assert len(stats) > 0
        for s in stats.values():
            assert s.count > 0

    def test_write_deck_analysis(self, episodes_file, tmp_path):
        from packages.training.data_tiers import write_deck_analysis
        output = tmp_path / "deck_analysis.json"
        result = write_deck_analysis(episodes_file, output)
        assert output.exists()
        assert "archetypes" in result


class TestCheckpointPruning:
    def test_keeps_latest_n(self, checkpoint_dir):
        from packages.training.data_tiers import prune_checkpoints
        deleted = prune_checkpoints(checkpoint_dir, keep_latest=5)
        assert len(list(checkpoint_dir.glob("checkpoint_*.pt"))) == 5
        assert len(deleted) == 3

    def test_keeps_special_checkpoints(self, checkpoint_dir):
        from packages.training.data_tiers import prune_checkpoints
        prune_checkpoints(checkpoint_dir, keep_latest=2)
        assert (checkpoint_dir / "shutdown_checkpoint.pt").exists()
        assert (checkpoint_dir / "best_checkpoint.pt").exists()

    def test_dry_run(self, checkpoint_dir):
        from packages.training.data_tiers import prune_checkpoints
        deleted = prune_checkpoints(checkpoint_dir, keep_latest=2, dry_run=True)
        assert len(deleted) == 6
        assert len(list(checkpoint_dir.glob("checkpoint_*.pt"))) == 8

    def test_empty_dir(self, tmp_path):
        from packages.training.data_tiers import prune_checkpoints
        d = tmp_path / "empty"
        d.mkdir()
        assert prune_checkpoints(d) == []


class TestCompression:
    def test_compress_old_jsonl(self, tmp_path):
        import os
        from packages.training.data_tiers import compress_old_jsonl
        run = tmp_path / "runs" / "run_old"
        run.mkdir(parents=True)
        jsonl = run / "episodes.jsonl"
        jsonl.write_text('{"seed": "test"}\n')
        old_time = jsonl.stat().st_mtime - 100000
        os.utime(jsonl, (old_time, old_time))
        compressed = compress_old_jsonl(tmp_path / "runs", max_age_hours=1)
        assert len(compressed) == 1
        assert not jsonl.exists()
        assert compressed[0].exists()

    def test_skip_recent(self, tmp_path):
        from packages.training.data_tiers import compress_old_jsonl
        run = tmp_path / "runs" / "run_new"
        run.mkdir(parents=True)
        (run / "episodes.jsonl").write_text('{"seed": "test"}\n')
        assert compress_old_jsonl(tmp_path / "runs", max_age_hours=24) == []


class TestReplay:
    def test_replay_by_seed(self, episodes_file):
        from packages.training.data_tiers import replay_episode
        assert replay_episode(episodes_file, episode_id="seed_001")["floor"] == 18

    def test_replay_by_index(self, episodes_file):
        from packages.training.data_tiers import replay_episode
        assert replay_episode(episodes_file, index=0)["seed"] == "seed_001"

    def test_replay_last(self, episodes_file):
        from packages.training.data_tiers import replay_episode
        assert replay_episode(episodes_file, index=-1)["seed"] == "seed_003"

    def test_replay_not_found(self, episodes_file):
        from packages.training.data_tiers import replay_episode
        assert replay_episode(episodes_file, episode_id="nonexistent") is None

    def test_format_replay(self, episodes_file):
        from packages.training.data_tiers import format_replay, replay_episode
        text = format_replay(replay_episode(episodes_file, episode_id="seed_001"))
        assert "Seed seed_001" in text
        assert "Floor: 18" in text
        assert "Lagavulin" in text


class TestExport:
    def test_local_exporter(self, traj_dir, tmp_path):
        import tarfile
        from packages.training.data_tiers import DataTier, LocalExporter
        files = list(traj_dir.glob("*.npz"))[:2]
        LocalExporter().export(DataTier.FILTERED, files, tmp_path / "export")
        archive = tmp_path / "export" / "filtered_data.tar.gz"
        assert archive.exists()
        with tarfile.open(archive, "r:gz") as tar:
            assert len(tar.getnames()) == 2

    def test_gdrive_exporter_raises(self, tmp_path):
        from packages.training.data_tiers import DataTier, GDriveExporter
        with pytest.raises(NotImplementedError):
            GDriveExporter().export(DataTier.RAW, [], tmp_path)
