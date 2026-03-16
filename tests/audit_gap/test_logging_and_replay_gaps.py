from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

import pytest

from packages.parity.replay_runner import ReplayRunner
from packages.server.protocol import MessageType
from packages.training.overnight import OvernightRunner
from packages.training.episode_logger import EpisodeLog


pytestmark = pytest.mark.audit_gap


FIXTURE_DIR = Path(__file__).resolve().parents[1] / "fixtures" / "audit_gap"


def test_gap_log_001_replay_runner_advances_past_floor_zero():
    """LOG-001: replay must compare at least one real floor transition."""
    runner = ReplayRunner(str(FIXTURE_DIR / "replay_minimal.jsonl"))
    result = runner.run(max_floors=1)

    compared_floors = [expected.floor for expected, _actual in result.floor_states]
    assert result.floors_replayed == 1
    assert 1 in compared_floors


def test_gap_log_002_episode_artifacts_expose_required_traces():
    """LOG-002: compact episode metadata is not enough for replay or audit workflows."""
    episode = EpisodeLog(
        seed="TEST123",
        ascension=20,
        character="Watcher",
        won=False,
        total_reward=0.0,
        floors_reached=3,
    )

    data = episode.to_dict()
    required_keys = {"run_id", "phase_trace", "decision_trace", "combat_trace"}
    assert required_keys.issubset(data.keys())


def test_gap_log_003_training_status_script_uses_supported_protocol_message():
    """LOG-003: training-status tooling should only send protocol-supported message types."""
    script = (Path(__file__).resolve().parents[2] / "scripts" / "training-status.sh").read_text()
    assert "get_status" in {message_type.value for message_type in MessageType}
    assert "get_status" in script


def test_gap_log_004_overnight_episode_entries_preserve_run_identity_and_traces(tmp_path: Path):
    """LOG-002: persisted episode rows should keep run identity and replay-grade traces."""
    runner = OvernightRunner({"run_dir": str(tmp_path)})
    rich_result = {
        "seed": "AUDIT-RUN",
        "floor": 12,
        "won": False,
        "hp": 37,
        "duration_s": 9.5,
        "transitions": [{"reward": 1.0, "pbrs": 0.4, "event_reward": 0.6}],
        "decisions": 3,
        "deck_final": ["Strike_P", "Eruption+"],
        "death_enemy": "Taskmaster",
        "room_type": "event",
        "combats": [{"floor": 12, "enemy": "Taskmaster"}],
        "construction_failure": False,
        "run_id": "run_20260316_010203",
        "config_snapshot": {"games": 500000, "workers": 12},
        "git_sha": "abc1234",
        "resume_lineage": ["checkpoint_0001.pt", "checkpoint_0002.pt"],
        "phase_trace": [{"floor": 12, "phase": "EVENT"}],
        "decision_trace": [
            {
                "floor": 12,
                "phase": "event",
                "event_id": "MaskedBandits",
                "action_id": "event_choice|choice_index=1",
            }
        ],
        "combat_trace": [{"floor": 12, "encounter": "Masked Bandits"}],
    }

    runner._log_episode(rich_result)

    episode_entry = json.loads((tmp_path / "episodes.jsonl").read_text().splitlines()[0])
    required_keys = {
        "run_id",
        "config_snapshot",
        "git_sha",
        "resume_lineage",
        "phase_trace",
        "decision_trace",
        "combat_trace",
    }

    assert required_keys.issubset(episode_entry.keys())
    assert episode_entry["decision_trace"][0]["action_id"] == "event_choice|choice_index=1"


def test_gap_log_005_recent_episode_snapshots_preserve_decision_reward_and_potion_detail(tmp_path: Path):
    """LOG-002: dashboard-visible recent episodes should keep semantic decision context."""
    runner = OvernightRunner({"run_dir": str(tmp_path)})
    rich_result = {
        "seed": "AUDIT-RECENT",
        "won": False,
        "floor": 7,
        "hp": 41,
        "duration_s": 4.2,
        "decisions": 3,
        "combats": [{"floor": 7, "enemy": "Cultist"}],
        "deck_changes": ["+Rushdown"],
        "decision_trace": [
            {
                "floor": 6,
                "phase": "event",
                "event_id": "Mushrooms",
                "action_id": "event_choice|choice_index=0",
            },
            {
                "floor": 7,
                "phase": "path",
                "action_id": "path_choice|node_index=2",
                "options": ["monster", "elite", "rest"],
                "chosen_node": {"x": 2, "y": 7, "room_type": "ELITE"},
            },
            {
                "floor": 7,
                "phase": "rest",
                "action_id": "rest|upgrade|card=Eruption",
                "choice": "smith",
                "upgraded_card": "Eruption",
            },
        ],
        "reward_trace": [{"floor": 6, "source": "event", "gold": 30}],
        "potion_trace": [
            {
                "floor": 7,
                "potion_id": "Fire Potion",
                "slot": 0,
                "target": "Cultist",
                "outcome": "lethal",
            }
        ],
        "path_trace": [{"floor": 7, "options": ["monster", "elite", "rest"], "chosen": "elite"}],
        "rest_trace": [{"floor": 7, "choice": "smith", "card": "Eruption"}],
    }

    for idx in range(10):
        runner._record_game({**rich_result, "seed": f"AUDIT-RECENT-{idx}"})

    recent_entries = json.loads((tmp_path / "recent_episodes.json").read_text())
    first_entry = recent_entries[0]

    assert "decision_trace" in first_entry
    assert first_entry["decision_trace"][0]["event_id"] == "Mushrooms"
    assert "reward_trace" in first_entry
    assert first_entry["reward_trace"][0]["source"] == "event"
    assert "potion_trace" in first_entry
    assert first_entry["potion_trace"][0]["potion_id"] == "Fire Potion"
    assert "path_trace" in first_entry
    assert "rest_trace" in first_entry


def test_gap_log_006_archive_fresh_runs_do_not_leave_episode_history_behind(tmp_path: Path):
    """LOG-002: archiving for a fresh run should clear stale episode history from weekend-run."""
    repo_root = Path(__file__).resolve().parents[2]
    script_src = repo_root / "scripts" / "training.sh"
    script_dir = tmp_path / "scripts"
    script_dir.mkdir()
    script_dst = script_dir / "training.sh"
    shutil.copy2(script_src, script_dst)

    run_dir = tmp_path / "logs" / "weekend-run"
    run_dir.mkdir(parents=True)
    (run_dir / "status.json").write_text("{}")
    (run_dir / "recent_episodes.json").write_text("[]")
    (run_dir / "summary.json").write_text("{}")
    (run_dir / "episodes.jsonl").write_text('{"seed":"old-run"}\n')
    (run_dir / "nohup.log").write_text("old log\n")

    completed = subprocess.run(
        ["bash", str(script_dst), "archive", "audit-gap"],
        cwd=tmp_path,
        check=False,
        capture_output=True,
        text=True,
    )

    assert completed.returncode == 0, completed.stderr or completed.stdout

    archived_runs = list((tmp_path / "logs" / "runs").glob("run_*_audit-gap"))
    assert len(archived_runs) == 1
    assert (archived_runs[0] / "episodes.jsonl").exists()
    assert not (run_dir / "episodes.jsonl").exists()
