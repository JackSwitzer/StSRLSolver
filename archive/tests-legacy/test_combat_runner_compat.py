"""Compatibility locks for the CombatRunner facade over CombatEngine."""

from packages.engine.handlers.combat import CombatRunner
from packages.engine.combat_engine import CombatEngine
from packages.engine.state.combat import EndTurn
from packages.engine.state.run import create_watcher_run
from packages.engine.state.rng import Random
from packages.engine.content.enemies import JawWorm


def _make_runner(seed: int = 12345) -> CombatRunner:
    run = create_watcher_run("TEST_COMPAT", ascension=0)
    rng = Random(seed)
    enemies = [JawWorm(ai_rng=Random(seed + 1), ascension=0, hp_rng=Random(seed + 2))]
    return CombatRunner(run_state=run, enemies=enemies, shuffle_rng=rng)


def test_combat_runner_wraps_combat_engine():
    """CombatRunner should expose the canonical CombatEngine runtime instance."""
    runner = _make_runner()
    assert isinstance(runner.engine, CombatEngine)
    assert runner.state is runner.engine.state


def test_combat_runner_play_card_uses_engine(monkeypatch):
    """Compatibility surface should route card play through CombatEngine."""
    runner = _make_runner()

    called = {"count": 0}
    original = runner.engine.play_card

    def _spy(hand_index, target_index=-1):
        called["count"] += 1
        return original(hand_index, target_index)

    monkeypatch.setattr(runner.engine, "play_card", _spy)

    if runner.state.hand:
        runner.play_card(0, target_idx=0)

    assert called["count"] == 1

