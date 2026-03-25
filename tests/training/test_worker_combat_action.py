from types import SimpleNamespace

from packages.training.worker import _pick_combat_action


class _DummySolver:
    def __init__(self, action):
        self._action = action

    def pick_action(self, actions, runner, room_type):
        return self._action


def _mk_runner():
    return SimpleNamespace(
        current_combat=SimpleNamespace(state=SimpleNamespace()),
        current_room_type="monster",
    )


def test_pick_combat_action_overrides_solver_end_turn_when_card_play_exists():
    end_turn = SimpleNamespace(action_type="end_turn")
    play_card = SimpleNamespace(action_type="play_card")
    actions = [end_turn, play_card]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(end_turn))

    assert chosen is play_card


def test_pick_combat_action_keeps_solver_end_turn_when_no_card_play_exists():
    end_turn = SimpleNamespace(action_type="end_turn")
    use_potion = SimpleNamespace(action_type="use_potion")
    actions = [end_turn, use_potion]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(end_turn))

    assert chosen is end_turn
