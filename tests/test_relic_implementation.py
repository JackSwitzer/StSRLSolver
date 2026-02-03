"""Integration tests for newly implemented damage-triggered relics."""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.handlers.combat import CombatRunner
from packages.engine.state.run import create_watcher_run
from packages.engine.state.rng import Random
from packages.engine.content.enemies import JawWorm


def _make_runner(relics=None, hp=80, max_hp=80):
    """Create a CombatRunner with given relics for testing."""
    run = create_watcher_run("TEST", ascension=0)
    run.max_hp = max_hp
    run.current_hp = hp

    if relics:
        for relic_id in relics:
            run.add_relic(relic_id)

    rng = Random(12345)
    ai_rng = Random(12346)
    hp_rng = Random(12347)
    enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]

    return CombatRunner(
        run_state=run,
        enemies=enemies,
        shuffle_rng=rng,
    )


def test_self_forming_clay_grants_block_next_turn():
    """Test Self-Forming Clay grants 3 block next turn after taking damage."""
    runner = _make_runner(relics=["Self Forming Clay"], hp=50, max_hp=80)

    # Initial state - no NextTurnBlock
    assert runner.state.player.statuses.get("NextTurnBlock", 0) == 0

    # Simulate taking damage
    runner._trigger_was_hp_lost(10)

    # Should now have NextTurnBlock power
    assert runner.state.player.statuses.get("NextTurnBlock", 0) == 3


def test_runic_cube_draws_on_hp_loss():
    """Test Runic Cube draws a card when HP is lost."""
    runner = _make_runner(relics=["Runic Cube"], hp=50, max_hp=80)

    initial_hand_size = len(runner.state.hand)

    # Trigger HP loss
    runner._trigger_was_hp_lost(10)

    # Should have drawn 1 card
    assert len(runner.state.hand) == initial_hand_size + 1


def test_boot_min_damage_5():
    """Test The Boot raises attacks dealing 1-4 damage to 5."""
    runner = _make_runner(relics=["Boot"])

    # Test damage < 5 gets boosted
    damage = runner._calculate_player_damage(3, runner.state.enemies[0])
    assert damage == 5

    # Test damage >= 5 is unchanged
    damage = runner._calculate_player_damage(10, runner.state.enemies[0])
    assert damage == 10


def test_red_skull_at_50_percent():
    """Test Red Skull grants 3 Strength at 50% HP threshold via registry."""
    from packages.engine.registry import execute_relic_triggers

    runner = _make_runner(relics=["Red Skull"], hp=40, max_hp=80)

    # Start at 50% - should gain Strength when HP is lost
    execute_relic_triggers("wasHPLost", runner.state, {"hp_lost": 5})
    assert runner.state.player.statuses.get("Strength", 0) == 3
    assert runner.state.get_relic_counter("Red Skull", 0) == 1

    # Note: Red Skull strength removal on healing above 50% is not part of wasHPLost
    # The registry correctly handles the trigger when HP drops below threshold


def test_champions_belt_applies_weak_with_vulnerable():
    """Test Champion's Belt applies 1 Weak when applying Vulnerable."""
    runner = _make_runner(relics=["Champion Belt"])

    enemy = runner.state.enemies[0]

    # Apply Vulnerable
    runner._apply_status(enemy, "Vulnerable", 2)

    # Should also have Weak
    assert enemy.statuses.get("Vulnerable", 0) == 2
    assert enemy.statuses.get("Weak", 0) == 1


def test_fossilized_helix_prevents_first_damage():
    """Test Fossilized Helix grants Buffer at combat start."""
    runner = _make_runner(relics=["FossilizedHelix"])

    # Should have Buffer at start
    assert runner.state.player.statuses.get("Buffer", 0) == 1


def test_orichalcum_grants_block_at_end_of_turn():
    """Test Orichalcum grants 6 block if player has 0 block at end of turn."""
    runner = _make_runner(relics=["Orichalcum"])

    # Start with 0 block
    runner.state.player.block = 0

    # Trigger end of turn
    runner._trigger_end_of_turn()

    # Should now have 6 block
    assert runner.state.player.block == 6


def test_orichalcum_does_not_grant_if_has_block():
    """Test Orichalcum does not grant block if player already has block."""
    runner = _make_runner(relics=["Orichalcum"])

    # Start with some block
    runner.state.player.block = 5

    # Trigger end of turn
    runner._trigger_end_of_turn()

    # Should still have 5 block (no Orichalcum trigger)
    assert runner.state.player.block == 5


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
