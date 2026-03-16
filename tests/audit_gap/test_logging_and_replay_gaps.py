from __future__ import annotations

import json
from pathlib import Path

import pytest

from packages.parity.replay_runner import ReplayRunner
from packages.server.protocol import MessageType
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
