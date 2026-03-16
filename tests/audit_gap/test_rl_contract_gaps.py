from __future__ import annotations

import inspect

import numpy as np
import pytest

from packages.engine.game import GameRunner, PendingSelectionContext
from packages.training.inference_server import InferenceClient
from packages.training.mlx_inference import MLXStrategicNet
from packages.training.strategic_net import StrategicNet


pytestmark = pytest.mark.audit_gap


def test_gap_rl_001_strategic_requests_preserve_candidate_semantics():
    """RL-001: strategic inference requests should carry semantic candidate action data."""
    client = InferenceClient(request_q=None, response_q=None, worker_slot=0)
    request = client._build_request(
        "strategic",
        obs=np.zeros(260, dtype=np.float32),
        n_actions=3,
        action_ids=["path_choice|node_index=0", "path_choice|node_index=1"],
        candidate_actions=[
            {"id": "path_choice|node_index=0", "type": "path_choice"},
            {"id": "path_choice|node_index=1", "type": "path_choice"},
        ],
    )

    assert "action_ids" in request or "candidate_actions" in request


def test_gap_rl_002_strategic_action_head_covers_engine_selection_surface():
    """RL-002: strategic action heads must represent large legal follow-up surfaces."""
    runner = GameRunner(seed="BIGSEL", ascension=20, skip_neow=True)
    runner.pending_selection = PendingSelectionContext(
        selection_type="card_select",
        source_action_type="event_choice",
        pile="deck",
        min_cards=2,
        max_cards=2,
        candidate_indices=list(range(25)),
        metadata={"event_choice_index": 0, "event_selection_type": "remove"},
        parent_action_id="event_choice|choice_index=0",
    )

    action_surface = runner.get_available_action_dicts()
    assert len(action_surface) > 256

    torch_action_dim = inspect.signature(StrategicNet.__init__).parameters["action_dim"].default
    mlx_action_dim = inspect.signature(MLXStrategicNet.__init__).parameters["action_dim"].default

    assert torch_action_dim >= len(action_surface)
    assert mlx_action_dim >= len(action_surface)
