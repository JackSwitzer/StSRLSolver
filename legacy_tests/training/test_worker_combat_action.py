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


def test_pick_combat_action_trusts_solver_end_turn():
    """EndTurn is a valid choice -- solver decision is trusted (stance cycling)."""
    end_turn = SimpleNamespace(action_type="end_turn")
    play_card = SimpleNamespace(action_type="play_card")
    actions = [end_turn, play_card]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(end_turn))

    assert chosen is end_turn


def test_pick_combat_action_keeps_solver_end_turn_when_no_card_play_exists():
    end_turn = SimpleNamespace(action_type="end_turn")
    use_potion = SimpleNamespace(action_type="use_potion")
    actions = [end_turn, use_potion]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(end_turn))

    assert chosen is end_turn


def test_pick_combat_action_fallback_prefers_card_play():
    """When solver returns None, fallback prefers card play over end turn."""
    end_turn = SimpleNamespace(action_type="end_turn")
    play_card = SimpleNamespace(action_type="play_card")
    actions = [end_turn, play_card]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(None))

    assert chosen is play_card


def test_pick_combat_action_returns_solver_card():
    play_card1 = SimpleNamespace(action_type="play_card")
    play_card2 = SimpleNamespace(action_type="play_card")
    actions = [play_card1, play_card2]

    chosen = _pick_combat_action(actions, _mk_runner(), _DummySolver(play_card2))

    assert chosen is play_card2
