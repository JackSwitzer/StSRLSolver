"""REL-007 regression tests for relic ordering/context edge cases."""

from packages.engine.combat_engine import CombatResult
from packages.engine.game import GameRunner
from packages.engine.registry import execute_relic_triggers
from packages.engine.state.combat import create_combat, create_enemy


def _base_combat_result(hp: int, max_hp: int) -> CombatResult:
    return CombatResult(
        victory=True,
        hp_remaining=hp,
        hp_lost=max_hp - hp,
        turns=1,
        cards_played=0,
        damage_dealt=0,
        damage_taken=0,
    )


def test_end_combat_with_combat_result_does_not_reapply_on_victory_relics():
    runner = GameRunner(seed="REL007_DUP_VICTORY", ascension=0, verbose=False)
    runner.run_state.max_hp = 80
    runner.run_state.current_hp = 50
    runner.run_state.add_relic("Burning Blood")
    runner.current_room_type = "monster"

    runner._end_combat(
        victory=True,
        combat_result=_base_combat_result(
            hp=runner.run_state.current_hp,
            max_hp=runner.run_state.max_hp,
        ),
    )

    assert runner.run_state.current_hp == 50


def test_post_combat_fallback_meat_on_the_bone_heals_at_exactly_50_percent():
    runner = GameRunner(seed="REL007_MEAT_THRESHOLD", ascension=0, verbose=False)
    runner.run_state.max_hp = 80
    runner.run_state.current_hp = 40  # exactly 50%
    runner.run_state.add_relic("Meat on the Bone")
    runner.current_room_type = "monster"

    runner._end_combat(victory=True)

    assert runner.run_state.current_hp == 52


def test_post_combat_fallback_does_not_apply_blood_vial_heal():
    runner = GameRunner(seed="REL007_BLOOD_VIAL", ascension=0, verbose=False)
    runner.run_state.max_hp = 80
    runner.run_state.current_hp = 60
    runner.run_state.add_relic("Blood Vial")
    runner.current_room_type = "monster"

    runner._end_combat(victory=True)

    assert runner.run_state.current_hp == 60


def test_preserved_insect_reduces_elite_hp_by_25_percent():
    state = create_combat(
        player_hp=70,
        player_max_hp=70,
        enemies=[create_enemy("EliteDummy", hp=100, max_hp=100, enemy_type="ELITE")],
        deck=["Strike_P"],
        relics=["Preserved Insect"],
    )
    state.combat_type = "elite"

    execute_relic_triggers("atBattleStart", state)

    enemy = state.enemies[0]
    assert enemy.max_hp == 75
    assert enemy.hp == 75


def test_at_battle_start_relic_hooks_have_default_combat_type_context():
    state = create_combat(
        player_hp=70,
        player_max_hp=70,
        enemies=[create_enemy("NormalDummy", hp=30, max_hp=30)],
        deck=["Strike_P"],
        relics=["Sling"],
    )

    execute_relic_triggers("atBattleStart", state)

    assert state.player.statuses.get("Strength", 0) == 0
