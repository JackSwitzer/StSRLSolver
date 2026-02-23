"""ORB-001 runtime integration and determinism checks."""

from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.state.rng import Random
from packages.engine.registry import execute_relic_triggers
from packages.engine.effects.orbs import (
    channel_orb,
    channel_random_orb,
    get_orb_manager,
    trigger_orb_passives,
)
from packages.engine.combat_engine import create_simple_combat


def _create_orb_state(*, relics=None):
    enemies = [
        create_enemy("E1", hp=50, max_hp=50, move_damage=6),
        create_enemy("E2", hp=50, max_hp=50, move_damage=6),
    ]
    return create_combat(
        player_hp=70,
        player_max_hp=70,
        enemies=enemies,
        deck=["Strike_B", "Defend_B", "Zap", "Dualcast"],
        relics=list(relics or []),
    )


def test_channel_random_orb_advances_owned_rng_counter():
    """Random orb channeling should consume the combat RNG stream."""
    state = _create_orb_state()
    state.card_random_rng = Random(12345)

    channel_random_orb(state)
    channel_random_orb(state)

    assert state.card_random_rng.counter == 2


def test_lightning_random_target_uses_owned_rng_counter():
    """Lightning random target selection should consume the combat RNG stream."""
    state = _create_orb_state()
    state.card_random_rng = Random(98765)
    manager = get_orb_manager(state)
    manager.lightning_hits_all = False
    channel_orb(state, "Lightning")

    before = state.card_random_rng.counter
    trigger_orb_passives(state)
    after = state.card_random_rng.counter

    assert after > before, "Lightning random target selection should use owned RNG"


def test_combat_engine_start_turn_triggers_orb_passives():
    """Orb passives should run in live combat turn startup."""
    engine = create_simple_combat("TestEnemy", enemy_hp=30, enemy_damage=5)
    engine.start_combat()
    channel_orb(engine.state, "Frost")

    # Force a fresh turn start path.
    engine.state.player.block = 0
    engine._start_player_turn()

    assert engine.state.player.block >= 2, "Frost passive should grant block on turn start"


def test_cables_canonical_id_triggers_extra_passive():
    """Canonical relic ID `Cables` should trigger extra orb passive."""
    state = _create_orb_state(relics=["Cables"])
    state.player.block = 0
    channel_orb(state, "Frost")
    before = state.player.block

    execute_relic_triggers("atTurnStart", state)

    assert state.player.block >= before + 2


def test_frozen_core_canonical_id_channels_with_empty_slot():
    """Canonical relic ID `FrozenCore` should use orb manager slot checks."""
    state = _create_orb_state(relics=["FrozenCore"])
    manager = get_orb_manager(state)
    channel_orb(state, "Lightning")
    before = manager.get_orb_count()

    execute_relic_triggers("onPlayerEndTurn", state)

    assert manager.get_orb_count() == before + 1
    assert manager.get_last_orb().orb_type.value == "Frost"
