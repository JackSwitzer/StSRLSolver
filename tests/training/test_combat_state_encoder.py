"""Tests for combat state encoder and MCTS-InferenceServer integration."""

from types import SimpleNamespace
from unittest.mock import MagicMock

import numpy as np
import pytest

from packages.training.combat_state_encoder import (
    COMBAT_OBS_DIM,
    encode_combat_state,
    make_mcts_policy_fn,
    _heuristic_combat_value,
)


# ---------------------------------------------------------------------------
# Fixtures / helpers
# ---------------------------------------------------------------------------


def _make_enemy(hp=30, max_hp=40, block=0, move_damage=10, move_hits=1, is_escaping=False):
    return SimpleNamespace(
        hp=hp,
        max_hp=max_hp,
        block=block,
        move_damage=move_damage,
        move_hits=move_hits,
        is_escaping=is_escaping,
        statuses={},
    )


def _make_player(hp=70, max_hp=80, block=5):
    return SimpleNamespace(hp=hp, max_hp=max_hp, block=block, statuses={})


def _make_engine(
    player=None,
    enemies=None,
    energy=3,
    max_energy=3,
    stance="Neutral",
    hand=None,
    draw_pile=None,
    discard_pile=None,
    exhaust_pile=None,
    turn=1,
    mantra=0,
    combat_type="normal",
):
    if player is None:
        player = _make_player()
    if enemies is None:
        enemies = [_make_enemy()]
    if hand is None:
        hand = ["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption"]
    if draw_pile is None:
        draw_pile = ["Strike_P"] * 5
    if discard_pile is None:
        discard_pile = []
    if exhaust_pile is None:
        exhaust_pile = []

    state = SimpleNamespace(
        player=player,
        enemies=enemies,
        energy=energy,
        max_energy=max_energy,
        stance=stance,
        hand=hand,
        draw_pile=draw_pile,
        discard_pile=discard_pile,
        exhaust_pile=exhaust_pile,
        turn=turn,
        mantra=mantra,
        cards_played_this_turn=0,
        attacks_played_this_turn=0,
        skills_played_this_turn=0,
        powers_played_this_turn=0,
        combat_over=False,
        player_won=False,
        combat_type=combat_type,
        total_damage_dealt=0,
        total_damage_taken=0,
        total_cards_played=0,
        card_costs={},
        potions=[],
        relics=[],
        relic_counters={},
    )

    engine = SimpleNamespace(
        state=state,
        is_combat_over=lambda: state.combat_over,
        is_victory=lambda: state.player_won,
    )
    return engine


# ---------------------------------------------------------------------------
# Encoding tests
# ---------------------------------------------------------------------------


class TestEncodeCombatState:
    def test_output_shape(self):
        engine = _make_engine()
        obs = encode_combat_state(engine)
        assert obs.shape == (COMBAT_OBS_DIM,)
        assert obs.dtype == np.float32

    def test_custom_input_dim(self):
        engine = _make_engine()
        obs = encode_combat_state(engine, input_dim=128)
        assert obs.shape == (128,)

    def test_player_scalars_encoded(self):
        engine = _make_engine(player=_make_player(hp=40, max_hp=80, block=10))
        obs = encode_combat_state(engine)

        assert abs(obs[0] - 0.5) < 1e-5   # hp_ratio = 40/80
        assert abs(obs[1] - 0.8) < 1e-5   # max_hp / 100
        assert abs(obs[2] - 0.2) < 1e-5   # block / 50

    def test_stance_encoding(self):
        for stance_idx, stance_name in enumerate(["Neutral", "Wrath", "Calm", "Divinity"]):
            engine = _make_engine(stance=stance_name)
            obs = encode_combat_state(engine)
            for j in range(4):
                expected = 1.0 if j == stance_idx else 0.0
                assert obs[8 + j] == expected, f"stance={stance_name}, idx={j}"

    def test_enemies_encoded(self):
        e1 = _make_enemy(hp=20, max_hp=40, move_damage=15, move_hits=2)
        e2 = _make_enemy(hp=10, max_hp=30, move_damage=5, move_hits=1)
        engine = _make_engine(enemies=[e1, e2])
        obs = encode_combat_state(engine)

        # First enemy at offset 12
        assert abs(obs[12] - 0.5) < 1e-5    # hp_ratio = 20/40
        assert abs(obs[12 + 3] - 0.5) < 1e-5  # move_damage 15/30
        assert abs(obs[12 + 4] - 0.4) < 1e-5  # move_hits 2/5
        assert obs[12 + 5] == 1.0             # alive

        # Second enemy at offset 18
        assert abs(obs[18] - (10.0 / 30)) < 1e-5

    def test_dead_enemies_excluded(self):
        e_dead = _make_enemy(hp=0, max_hp=40)
        e_alive = _make_enemy(hp=20, max_hp=40)
        engine = _make_engine(enemies=[e_dead, e_alive])
        obs = encode_combat_state(engine)

        # Only the alive enemy should be in the first slot
        assert obs[12 + 5] == 1.0  # first encoded enemy is alive
        assert obs[18 + 5] == 0.0  # no second encoded enemy

    def test_escaping_enemies_excluded(self):
        e_escape = _make_enemy(hp=10, max_hp=40, is_escaping=True)
        e_alive = _make_enemy(hp=20, max_hp=40)
        engine = _make_engine(enemies=[e_escape, e_alive])
        obs = encode_combat_state(engine)

        # Only non-escaping enemy in first slot
        assert abs(obs[12] - 0.5) < 1e-5  # hp_ratio of alive enemy

    def test_hand_summary(self):
        hand = ["Strike_P", "Strike_P", "Defend_P", "Eruption", "MentalFortress"]
        engine = _make_engine(hand=hand, energy=2)
        obs = encode_combat_state(engine)

        # Offset 42: attacks, skills, powers, cost, ...
        # Strike_P x2 = 2 attacks, Defend_P = 1 skill, Eruption = 1 attack,
        # MentalFortress = 1 power
        assert obs[42] > 0  # attacks
        assert obs[43] > 0  # skills
        assert obs[44] > 0  # powers

    def test_power_features(self):
        player = _make_player()
        player.statuses = {"Strength": 3, "Dexterity": 1}
        engine = _make_engine(player=player)
        obs = encode_combat_state(engine)

        # Offset 54: player powers
        assert abs(obs[54] - 0.3) < 1e-5  # Strength 3 / 10
        assert abs(obs[55] - 0.1) < 1e-5  # Dexterity 1 / 10

    def test_zero_division_safe(self):
        """Encoding with zero hp enemies or player shouldn't crash."""
        player = _make_player(hp=0, max_hp=0)
        engine = _make_engine(
            player=player,
            enemies=[_make_enemy(hp=0, max_hp=0)],
            hand=[],
        )
        obs = encode_combat_state(engine)
        assert not np.any(np.isnan(obs))
        assert not np.any(np.isinf(obs))

    def test_all_values_finite(self):
        engine = _make_engine()
        obs = encode_combat_state(engine)
        assert np.all(np.isfinite(obs))


# ---------------------------------------------------------------------------
# Heuristic value tests
# ---------------------------------------------------------------------------


class TestHeuristicCombatValue:
    def test_victory_value(self):
        engine = _make_engine()
        engine.state.combat_over = True
        engine.state.player_won = True
        engine.is_combat_over = lambda: True
        engine.is_victory = lambda: True

        val = _heuristic_combat_value(engine)
        assert 0.7 <= val <= 1.0

    def test_defeat_value(self):
        engine = _make_engine()
        engine.state.combat_over = True
        engine.state.player_won = False
        engine.is_combat_over = lambda: True
        engine.is_victory = lambda: False

        val = _heuristic_combat_value(engine)
        assert val == 0.0

    def test_ongoing_combat_value(self):
        engine = _make_engine()
        val = _heuristic_combat_value(engine)
        assert 0.0 <= val <= 1.0


# ---------------------------------------------------------------------------
# make_mcts_policy_fn tests
# ---------------------------------------------------------------------------


class TestMakeMctsPolicyFn:
    def test_returns_priors_and_value(self):
        mock_client = MagicMock()
        mock_client.infer_combat.return_value = {
            "ok": True,
            "value": 0.4,  # tanh output in [-1, 1]
        }

        policy_fn = make_mcts_policy_fn(mock_client)

        # Create a fake engine that has legal actions
        engine = _make_engine()
        engine.get_legal_actions = lambda: ["strike", "defend", "end_turn"]

        priors, value = policy_fn(engine)

        # Priors should be uniform
        assert len(priors) == 3
        for p in priors.values():
            assert abs(p - 1 / 3) < 1e-5

        # Value should be mapped from [-1,1] to [0,1]: (0.4 + 1) / 2 = 0.7
        assert abs(value - 0.7) < 1e-5

    def test_fallback_on_error(self):
        mock_client = MagicMock()
        mock_client.infer_combat.return_value = None  # Timeout/error

        policy_fn = make_mcts_policy_fn(mock_client)
        engine = _make_engine()
        engine.get_legal_actions = lambda: ["strike", "defend"]

        priors, value = policy_fn(engine)

        # Should fall back to heuristic
        assert len(priors) == 2
        assert 0.0 <= value <= 1.0

    def test_no_legal_actions(self):
        mock_client = MagicMock()
        policy_fn = make_mcts_policy_fn(mock_client)

        engine = _make_engine()
        engine.get_legal_actions = lambda: []

        priors, value = policy_fn(engine)
        assert priors == {}
        assert value == 0.0

    def test_obs_sent_to_client(self):
        mock_client = MagicMock()
        mock_client.infer_combat.return_value = {"ok": True, "value": 0.0}

        policy_fn = make_mcts_policy_fn(mock_client, input_dim=260)
        engine = _make_engine()
        engine.get_legal_actions = lambda: ["strike"]

        policy_fn(engine)

        # Verify infer_combat was called with a proper obs array
        call_args = mock_client.infer_combat.call_args
        obs_arg = call_args[0][0]
        assert isinstance(obs_arg, np.ndarray)
        assert obs_arg.shape == (260,)
        assert obs_arg.dtype == np.float32


# ---------------------------------------------------------------------------
# InferenceServer combat route tests
# ---------------------------------------------------------------------------


class TestInferenceServerCombatRoute:
    def test_combat_route_returns_value(self):
        """Test that the combat route returns value from strategic backend."""
        from packages.training.inference_server import InferenceServer

        server = InferenceServer(n_workers=1, max_batch_size=4, use_mlx=False)
        responses = []

        class DummyNet:
            input_dim = 8

        class RecordingBackend:
            def __init__(self):
                self.action_dim = 6
                self.version = 5
                self._net = DummyNet()

            def forward_batch(self, obs_batch, mask_batch):
                # All masks should be True for combat (value-only)
                assert np.all(mask_batch), "Combat route should set all masks True"
                logits = np.zeros((len(obs_batch), self.action_dim), dtype=np.float32)
                values = np.array([0.42] * len(obs_batch), dtype=np.float32)
                return logits, values

        backend = RecordingBackend()
        server._strategic_backend = backend
        server._send_response = lambda slot, resp: responses.append(resp)

        server._forward_combat([
            {
                "req_id": 1,
                "worker_slot": 0,
                "route": "combat",
                "obs": np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], dtype=np.float32),
                "legal_indices": np.array([0, 1, 2]),
            }
        ])

        assert len(responses) == 1
        resp = responses[0]
        assert resp["ok"] is True
        assert resp["route"] == "combat"
        assert abs(resp["value"] - 0.42) < 1e-5
        assert resp["logits"] is None  # Combat route doesn't return logits

    def test_combat_route_no_backend(self):
        """Test error when backend not initialized."""
        from packages.training.inference_server import InferenceServer

        server = InferenceServer(n_workers=1, max_batch_size=4, use_mlx=False)
        errors = []

        def capture_error(req, msg):
            errors.append(msg)

        server._send_error = capture_error

        server._forward_combat([
            {"req_id": 1, "worker_slot": 0, "route": "combat", "obs": None}
        ])

        assert len(errors) == 1
        assert "not available" in errors[0]

    def test_combat_batch_multiple_requests(self):
        """Test batched combat inference with multiple requests."""
        from packages.training.inference_server import InferenceServer

        server = InferenceServer(n_workers=2, max_batch_size=4, use_mlx=False)
        responses = []

        class DummyNet:
            input_dim = 4

        class ValueBackend:
            def __init__(self):
                self.action_dim = 4
                self.version = 3
                self._net = DummyNet()

            def forward_batch(self, obs_batch, mask_batch):
                n = len(obs_batch)
                logits = np.zeros((n, self.action_dim), dtype=np.float32)
                # Return different values for each request
                values = np.arange(n, dtype=np.float32) * 0.1
                return logits, values

        server._strategic_backend = ValueBackend()
        server._send_response = lambda slot, resp: responses.append((slot, resp))

        server._forward_combat([
            {"req_id": 10, "worker_slot": 0, "route": "combat",
             "obs": np.ones(4, dtype=np.float32)},
            {"req_id": 11, "worker_slot": 1, "route": "combat",
             "obs": np.ones(4, dtype=np.float32) * 2},
        ])

        assert len(responses) == 2
        # First request: value=0.0, Second: value=0.1
        assert abs(responses[0][1]["value"] - 0.0) < 1e-5
        assert abs(responses[1][1]["value"] - 0.1) < 1e-5


# ---------------------------------------------------------------------------
# GumbelMCTS with mock policy integration test
# ---------------------------------------------------------------------------


class TestGumbelMCTSWithPolicyFn:
    def test_mcts_with_mock_inference(self):
        """End-to-end: GumbelMCTS uses make_mcts_policy_fn with mock client."""
        from packages.training.gumbel_mcts import GumbelMCTS

        np.random.seed(42)

        mock_client = MagicMock()
        # Return moderate value for all positions
        mock_client.infer_combat.return_value = {"ok": True, "value": 0.3}

        policy_fn = make_mcts_policy_fn(mock_client)

        engine = _make_engine()
        actions = ["strike", "defend", "end_turn"]
        engine.get_legal_actions = lambda: list(actions)
        engine.copy = lambda: _make_engine_with_actions(actions)

        def execute_noop(action):
            pass
        engine.execute_action = execute_noop

        mcts = GumbelMCTS(policy_fn=policy_fn, num_simulations=8, max_candidates=3)
        probs = mcts.search(engine)

        assert len(probs) > 0
        assert all(v >= 0 for v in probs.values())
        total = sum(probs.values())
        assert abs(total - 1.0) < 1e-5

        # infer_combat should have been called multiple times (once per simulation)
        assert mock_client.infer_combat.call_count > 0


def _make_engine_with_actions(actions):
    """Create a copyable engine mock for MCTS."""
    engine = _make_engine()
    engine.get_legal_actions = lambda: list(actions)
    engine.copy = lambda: _make_engine_with_actions(actions)
    engine.execute_action = lambda a: None
    return engine
