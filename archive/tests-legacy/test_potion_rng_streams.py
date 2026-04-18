"""Potion RNG stream advancement and runtime invariants."""

from packages.engine import GameRunner, GamePhase
from packages.engine.combat_engine import CombatEngine
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState, create_combat, create_enemy
from packages.engine.state.rng import Random


def _make_runner_with_combat(state):
    runner = GameRunner(seed="POTION_RNG", ascension=20, verbose=False)
    runner.current_combat = CombatEngine(state)
    runner.phase = GamePhase.COMBAT
    return runner


def test_discovery_offer_consumes_three_card_rng_calls():
    """Discovery-family potion offer generation should consume exactly 3 card RNG calls."""
    state = create_combat(
        player_hp=60,
        player_max_hp=80,
        enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
        deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
        relics=[],
        potions=["AttackPotion", "", ""],
    )
    runner = _make_runner_with_combat(state)

    before = runner.card_rng.counter
    result = runner.take_action_dict({
        "type": "use_potion",
        "params": {"potion_slot": 0},
    })
    after = runner.card_rng.counter

    assert result.get("requires_selection") is True
    assert after - before == 3


def test_snecko_oil_uses_card_random_rng_per_eligible_hand_card():
    """Snecko Oil should advance card_random_rng once per eligible hand card randomized."""
    state = CombatState(
        player=EntityState(hp=50, max_hp=80, block=0),
        energy=3,
        max_energy=3,
        hand=["Strike_P", "Defend_P", "Vigilance"],
        draw_pile=[],
        discard_pile=[],
        potions=["SneckoOil", "", ""],
        enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")],
    )
    state.card_random_rng = Random(12345)

    engine = CombatEngine(state)

    before = state.card_random_rng.counter
    result = engine.use_potion(0)
    after = state.card_random_rng.counter

    assert result["success"] is True
    assert after - before == 3


def test_entropic_brew_advances_potion_rng_per_filled_slot():
    """Entropic Brew should consume potion_rng once per empty slot filled."""
    state = CombatState(
        player=EntityState(hp=50, max_hp=80, block=0),
        energy=3,
        max_energy=3,
        potions=["EntropicBrew", "", ""],
        enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")],
    )
    state.potion_rng = Random(98765)

    engine = CombatEngine(state)

    before = state.potion_rng.counter
    result = engine.use_potion(0)
    after = state.potion_rng.counter

    assert result["success"] is True
    assert after - before == 3
    assert all(slot for slot in state.potions)


def test_distilled_chaos_enemy_targeting_uses_card_random_rng():
    """Distilled Chaos should use card_random_rng to choose random enemy targets."""
    state = CombatState(
        player=EntityState(hp=50, max_hp=80, block=0),
        energy=3,
        max_energy=3,
        hand=[],
        # top of draw pile is end of list, so only Strike_P needs random target selection
        draw_pile=["Defend_P", "Vigilance", "Strike_P"],
        discard_pile=[],
        potions=["DistilledChaos", "", ""],
        enemies=[
            EnemyCombatState(hp=40, max_hp=40, id="EnemyA", name="Enemy A"),
            EnemyCombatState(hp=40, max_hp=40, id="EnemyB", name="Enemy B"),
        ],
    )
    state.card_random_rng = Random(24680)

    engine = CombatEngine(state)

    before = state.card_random_rng.counter
    result = engine.use_potion(0)
    after = state.card_random_rng.counter

    assert result["success"] is True
    assert after - before == 1
