"""Tests for POT-001 runtime/registry potion de-duplication."""

from packages.engine.combat_engine import CombatEngine
from packages.engine.registry import execute_potion_effect
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState


def test_distilled_chaos_registry_reuses_attached_combat_engine():
    """Distilled Chaos registry path should call active engine autoplay path when available."""
    state = CombatState(
        player=EntityState(hp=50, max_hp=80, block=0),
        energy=3,
        max_energy=3,
        hand=[],
        draw_pile=["Defend_P", "Vigilance", "Strike_P"],
        discard_pile=[],
        potions=["DistilledChaos", "", ""],
        enemies=[
            EnemyCombatState(hp=40, max_hp=40, id="EnemyA", name="Enemy A"),
            EnemyCombatState(hp=40, max_hp=40, id="EnemyB", name="Enemy B"),
        ],
    )
    engine = CombatEngine(state)

    calls = []
    original = engine.play_top_cards_from_draw_pile

    def _spy(count: int):
        calls.append(count)
        return original(count)

    engine.play_top_cards_from_draw_pile = _spy  # type: ignore[method-assign]

    result = execute_potion_effect("DistilledChaos", state, target_idx=-1)

    assert result["success"] is True
    assert calls == [3]


def test_smoke_bomb_blocked_registry_path_returns_failure():
    """Blocked Smoke Bomb use should fail consistently in registry execution path."""
    state = CombatState(
        player=EntityState(hp=50, max_hp=80, block=0),
        energy=3,
        max_energy=3,
        potions=["SmokeBomb", "", ""],
        enemies=[
            EnemyCombatState(
                hp=50,
                max_hp=50,
                id="SpireSpear",
                name="Spire Spear",
                statuses={"BackAttack": 1},
            )
        ],
    )
    state.is_boss_combat = True

    result = execute_potion_effect("SmokeBomb", state, target_idx=-1)

    assert result["success"] is False
    assert "cannot" in result.get("error", "").lower()
    assert getattr(state, "escaped", False) is False

