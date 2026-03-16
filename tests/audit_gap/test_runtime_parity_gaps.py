from __future__ import annotations

import pytest

from packages.engine.game import EventAction, GamePhase, GameRunner
from packages.engine.generation.map import MapRoomNode, RoomType
from packages.engine.handlers.event_handler import EventPhase, EventState


pytestmark = pytest.mark.audit_gap


@pytest.mark.parametrize(
    ("event_id", "choice_index"),
    [
        ("MaskedBandits", 1),
        ("Mushrooms", 0),
        ("Colosseum", 0),
        ("MindBloom", 0),
        ("DeadAdventurer", 0),
    ],
)
def test_gap_runtime_001_event_fights_use_requested_encounter(event_id: str, choice_index: int):
    """RUNTIME-001: event fights should not degrade into the generic hallway encounter."""
    runner = GameRunner(seed="AUDIT", ascension=20, skip_neow=True)
    runner._monster_list = ["Jaw Worm"]
    runner._monster_index = 0
    runner.current_event_state = EventState(event_id=event_id, phase=EventPhase.INITIAL)
    runner.phase = GamePhase.EVENT

    ok, result = runner._handle_event_action(EventAction(choice_index))

    assert ok
    assert result.get("combat_triggered") is True
    assert runner.current_combat is not None
    actual_enemy_ids = [enemy.id for enemy in runner.current_combat.state.enemies]
    assert actual_enemy_ids != ["JawWorm"]


def test_gap_runtime_002_event_fight_victory_resolves_event_state():
    """RUNTIME-001: post-event combat should not leave the event stuck in COMBAT_PENDING."""
    runner = GameRunner(seed="AUDIT", ascension=20, skip_neow=True)
    runner._monster_list = ["Jaw Worm"]
    runner._monster_index = 0
    runner.current_event_state = EventState(event_id="Mushrooms", phase=EventPhase.INITIAL)
    runner.phase = GamePhase.EVENT

    runner._handle_event_action(EventAction(0))
    runner._end_combat(victory=True)

    assert runner.current_event_state is None or runner.current_event_state.phase != EventPhase.COMBAT_PENDING
    assert runner.phase != GamePhase.COMBAT_REWARDS


def test_gap_runtime_003_question_rooms_advance_tiny_chest_counter():
    """RUNTIME-002: entering runtime `?` rooms should advance Tiny Chest toward a treasure roll."""
    runner = GameRunner(seed="QROOM", ascension=20, skip_neow=True)
    runner.run_state.add_relic("Tiny Chest")

    node = MapRoomNode(x=0, y=0, room_type=RoomType.EVENT)
    runner._enter_room(node)

    tiny_chest = runner.run_state.get_relic("Tiny Chest")
    assert tiny_chest is not None
    assert tiny_chest.counter == 1


def test_gap_runtime_004_burning_elite_emits_emerald_key_reward():
    """RUNTIME-003: a map node marked as burning elite should surface the emerald-key reward."""
    runner = GameRunner(seed="BURN", ascension=20, skip_neow=True)
    node = MapRoomNode(x=0, y=0, room_type=RoomType.ELITE)
    node.has_emerald_key = True

    runner._enter_room(node)
    runner._end_combat(victory=True)

    assert runner.current_rewards is not None
    assert runner.current_rewards.emerald_key is not None


def test_gap_runtime_005_fusion_hammer_blocks_rest_upgrades():
    """RUNTIME-004: `GameRunner` should not emit smith actions when Fusion Hammer is equipped."""
    runner = GameRunner(seed="REST", ascension=20, skip_neow=True)
    runner.run_state.add_relic("Fusion Hammer")
    runner.phase = GamePhase.REST

    actions = runner.get_available_actions()

    assert not any(getattr(action, "action_type", "") == "upgrade" for action in actions)
