from types import SimpleNamespace

import numpy as np

from packages.training.gumbel_mcts import GumbelMCTS


class FakeEngine:
    def __init__(self, last_action=None):
        self.last_action = last_action
        self.state = SimpleNamespace(
            player=SimpleNamespace(hp=70, max_hp=80),
            enemies=[SimpleNamespace(hp=30, max_hp=40)],
        )

    def copy(self):
        return FakeEngine(self.last_action)

    def get_legal_actions(self):
        return ["strike", "defend", "erupt"]

    def execute_action(self, action):
        self.last_action = action

    def is_combat_over(self):
        return False

    def is_victory(self):
        return False


def test_gumbel_mcts_exports_last_root_summary():
    np.random.seed(0)

    def policy_fn(engine):
        priors = {"strike": 0.6, "defend": 0.2, "erupt": 0.2}
        values = {None: 0.3, "strike": 0.8, "defend": 0.1, "erupt": 0.5}
        return priors, values.get(engine.last_action, 0.3)

    mcts = GumbelMCTS(policy_fn=policy_fn, num_simulations=8, max_candidates=3)
    probs = mcts.search(FakeEngine())
    best_action = max(probs, key=probs.get)

    payload = mcts.export_last_root_stats(selected_action=best_action)

    assert payload is not None
    assert payload["sims"] == 8
    assert abs(sum(action["pct"] for action in payload["actions"]) - 100.0) <= 0.2
    assert any(action["selected"] for action in payload["actions"])
    assert {action["id"] for action in payload["actions"]} == set(probs)
