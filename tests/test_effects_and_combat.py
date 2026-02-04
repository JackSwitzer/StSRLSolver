"""
Tests for effects/executor.py, effects/cards.py, effects/registry.py, and handlers/combat.py.

Covers:
- Effect registry: registration, lookup, pattern matching, execution
- Effect context: card manipulation, damage, block, stance, mantra, scry
- Effect executor: play_card pipeline, damage modifiers, start/end of turn
- Card effects: individual card effect implementations
- Combat handler: CombatRunner init, turn flow, victory/defeat
"""

import pytest
from tests.conftest import create_combat_state
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    PlayCard, UsePotion, EndTurn,
)
from packages.engine.effects.registry import (
    EffectContext, EffectTiming, execute_effect, get_effect_handler,
    list_registered_effects, _EFFECT_REGISTRY,
)
from packages.engine.effects.executor import EffectExecutor, EffectResult, create_executor
from packages.engine.effects.cards import (
    get_card_effects, get_card_cost_modifier, WATCHER_CARD_EFFECTS,
    trigger_on_stance_change, trigger_on_scry,
    apply_start_of_turn_effects, apply_end_of_turn_effects,
    perform_scry, can_play_card, gain_mantra_and_check_divinity,
    execute_card_effects,
)
from packages.engine.content.cards import Card, CardType, CardTarget, CardRarity, CardColor, get_card, ALL_CARDS


# =============================================================================
# Helpers
# =============================================================================

def make_enemy(hp=50, block=0, statuses=None, move_damage=10, is_attacking=True):
    return EnemyCombatState(
        hp=hp, max_hp=hp, block=block,
        statuses=statuses or {},
        id="TestEnemy",
        move_id=0,
        move_damage=move_damage if is_attacking else 0,
        move_hits=1,
        move_block=0,
        move_effects={},
    )


def make_state(hand=None, draw=None, discard=None, enemies=None, energy=3,
               stance="Neutral", relics=None, player_hp=80, player_statuses=None):
    player = EntityState(hp=player_hp, max_hp=80, block=0, statuses=player_statuses or {})
    return CombatState(
        player=player, energy=energy, max_energy=3, stance=stance,
        hand=hand or [], draw_pile=draw or [], discard_pile=discard or [],
        exhaust_pile=[], enemies=enemies or [make_enemy()],
        potions=[], relics=relics or [], turn=1,
        cards_played_this_turn=0, attacks_played_this_turn=0,
        skills_played_this_turn=0, powers_played_this_turn=0,
        relic_counters={}, card_costs={},
    )


def make_ctx(state=None, card=None, target=None, target_idx=-1, upgraded=False, magic=0):
    if state is None:
        state = make_state()
    if target is None and target_idx >= 0 and target_idx < len(state.enemies):
        target = state.enemies[target_idx]
    return EffectContext(
        state=state, card=card, target=target, target_idx=target_idx,
        is_upgraded=upgraded, magic_number=magic,
    )


# =============================================================================
# Registry Tests
# =============================================================================

class TestEffectRegistry:
    def test_effects_are_registered(self):
        effects = list_registered_effects()
        assert len(effects) > 20
        assert "draw_1" in effects
        assert "enter_wrath" in effects
        assert "scry_1" in effects

    def test_get_effect_handler_direct(self):
        result = get_effect_handler("draw_1")
        assert result is not None
        handler, params = result
        assert params == ()

    def test_get_effect_handler_pattern(self):
        result = get_effect_handler("draw_5")
        assert result is not None
        handler, params = result
        assert params == (5,)

    def test_get_effect_handler_unknown(self):
        result = get_effect_handler("nonexistent_effect_xyz")
        assert result is None

    def test_execute_effect_draw(self):
        state = make_state(hand=[], draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        assert execute_effect("draw_2", ctx) is True
        assert len(state.hand) == 2
        assert len(state.draw_pile) == 1

    def test_execute_effect_unknown(self):
        ctx = make_ctx()
        assert execute_effect("totally_fake_effect", ctx) is False

    def test_execute_effect_gain_block(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("gain_block_5", ctx)
        assert state.player.block == 5

    def test_execute_effect_gain_energy(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state)
        execute_effect("gain_energy_3", ctx)
        assert state.energy == 5


# =============================================================================
# EffectContext Tests
# =============================================================================

class TestEffectContext:
    def test_draw_cards_basic(self):
        state = make_state(hand=[], draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        drawn = ctx.draw_cards(2)
        assert len(drawn) == 2
        assert len(state.hand) == 2

    def test_draw_cards_shuffles_discard(self):
        state = make_state(hand=[], draw=[], discard=["X", "Y", "Z"])
        ctx = make_ctx(state=state)
        drawn = ctx.draw_cards(2)
        assert len(drawn) == 2
        assert len(state.discard_pile) == 0

    def test_draw_cards_empty(self):
        state = make_state(hand=[], draw=[], discard=[])
        ctx = make_ctx(state=state)
        drawn = ctx.draw_cards(3)
        assert drawn == []

    def test_discard_card(self):
        state = make_state(hand=["A", "B", "C"])
        ctx = make_ctx(state=state)
        assert ctx.discard_card("B") is True
        assert "B" not in state.hand
        assert "B" in state.discard_pile

    def test_exhaust_card(self):
        state = make_state(hand=["A", "B"])
        ctx = make_ctx(state=state)
        assert ctx.exhaust_card("A") is True
        assert "A" not in state.hand
        assert "A" in state.exhaust_pile

    def test_add_card_to_hand_limit(self):
        state = make_state(hand=["c"] * 10)
        ctx = make_ctx(state=state)
        assert ctx.add_card_to_hand("extra") is False
        assert len(state.hand) == 10

    def test_add_card_to_draw_pile_top(self):
        state = make_state(draw=["A", "B"])
        ctx = make_ctx(state=state)
        ctx.add_card_to_draw_pile("TOP", "top")
        assert state.draw_pile[-1] == "TOP"

    def test_deal_damage_to_enemy(self):
        enemy = make_enemy(hp=30, block=5)
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state)
        actual = ctx.deal_damage_to_enemy(enemy, 10)
        assert actual == 5  # 10 - 5 block
        assert enemy.hp == 25
        assert enemy.block == 0

    def test_deal_damage_to_enemy_overkill(self):
        enemy = make_enemy(hp=5)
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state)
        ctx.deal_damage_to_enemy(enemy, 100)
        assert enemy.hp == 0

    def test_gain_block(self):
        state = make_state()
        ctx = make_ctx(state=state)
        ctx.gain_block(8)
        assert state.player.block == 8
        assert ctx.block_gained == 8

    def test_deal_damage_to_player(self):
        state = make_state(player_hp=50)
        state.player.block = 10
        ctx = make_ctx(state=state)
        hp_loss = ctx.deal_damage_to_player(15)
        assert hp_loss == 5  # 15 - 10 block
        assert state.player.hp == 45

    def test_heal_player(self):
        state = make_state(player_hp=50)
        ctx = make_ctx(state=state)
        healed = ctx.heal_player(10)
        assert healed == 10
        assert state.player.hp == 60

    def test_heal_player_cap(self):
        state = make_state(player_hp=75)
        ctx = make_ctx(state=state)
        healed = ctx.heal_player(20)
        assert healed == 5
        assert state.player.hp == 80

    def test_apply_status_to_target(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        ctx.apply_status_to_target("Weak", 2)
        assert enemy.statuses["Weak"] == 2

    def test_apply_status_blocked_by_artifact(self):
        enemy = make_enemy(statuses={"Artifact": 1})
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        result = ctx.apply_status_to_target("Weak", 2)
        assert result is False
        assert "Weak" not in enemy.statuses

    def test_apply_status_to_player(self):
        state = make_state()
        ctx = make_ctx(state=state)
        ctx.apply_status_to_player("Strength", 3)
        assert state.player.statuses["Strength"] == 3

    def test_gain_energy(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state)
        ctx.gain_energy(2)
        assert state.energy == 4
        assert ctx.energy_gained == 2

    def test_spend_energy(self):
        state = make_state(energy=3)
        ctx = make_ctx(state=state)
        assert ctx.spend_energy(2) is True
        assert state.energy == 1
        assert ctx.spend_energy(5) is False

    def test_change_stance_calm_to_wrath(self):
        state = make_state(stance="Calm", energy=1)
        ctx = make_ctx(state=state)
        result = ctx.change_stance("Wrath")
        assert state.stance == "Wrath"
        assert result["energy_gained"] == 2  # Calm exit
        assert state.energy == 3

    def test_change_stance_to_divinity(self):
        state = make_state(stance="Neutral", energy=1)
        ctx = make_ctx(state=state)
        result = ctx.change_stance("Divinity")
        assert state.stance == "Divinity"
        assert result["energy_gained"] == 3

    def test_change_stance_same_no_effect(self):
        state = make_state(stance="Wrath")
        ctx = make_ctx(state=state)
        result = ctx.change_stance("Wrath")
        assert result["energy_gained"] == 0

    def test_change_stance_violet_lotus(self):
        state = make_state(stance="Calm", energy=0, relics=["VioletLotus"])
        ctx = make_ctx(state=state)
        result = ctx.change_stance("Neutral")
        assert result["energy_gained"] == 3
        assert state.energy == 3

    def test_change_stance_mental_fortress(self):
        state = make_state(stance="Neutral", player_statuses={"MentalFortress": 4})
        ctx = make_ctx(state=state)
        ctx.change_stance("Wrath")
        assert state.player.block == 4

    def test_change_stance_flurry_of_blows(self):
        state = make_state(stance="Neutral", hand=["A"], discard=["FlurryOfBlows"])
        ctx = make_ctx(state=state)
        ctx.change_stance("Wrath")
        assert "FlurryOfBlows" in state.hand
        assert "FlurryOfBlows" not in state.discard_pile

    def test_exit_stance(self):
        state = make_state(stance="Wrath")
        ctx = make_ctx(state=state)
        ctx.exit_stance()
        assert state.stance == "Neutral"

    def test_gain_mantra_below_10(self):
        state = make_state(player_statuses={"Mantra": 3})
        ctx = make_ctx(state=state)
        result = ctx.gain_mantra(4)
        assert result["divinity_triggered"] is False

    def test_gain_mantra_triggers_divinity(self):
        state = make_state(stance="Neutral", player_statuses={"Mantra": 7})
        ctx = make_ctx(state=state)
        result = ctx.gain_mantra(5)
        assert result["divinity_triggered"] is True
        assert state.stance == "Divinity"

    def test_scry(self):
        state = make_state(draw=["A", "B", "C"], hand=[])
        ctx = make_ctx(state=state)
        scried = ctx.scry(2)
        assert len(scried) == 2
        # Cards should be put back on top
        assert len(state.draw_pile) == 3

    def test_scry_triggers_weave(self):
        state = make_state(draw=["A", "B"], hand=[], discard=["Weave"])
        ctx = make_ctx(state=state)
        ctx.scry(1)
        assert "Weave" in state.hand

    def test_scry_triggers_nirvana(self):
        state = make_state(draw=["A", "B"], hand=[], player_statuses={"Nirvana": 3})
        ctx = make_ctx(state=state)
        ctx.scry(2)
        # Nirvana gives block once per scry action (matches Java)
        assert state.player.block == 3

    def test_end_turn_flag(self):
        ctx = make_ctx()
        assert ctx.should_end_turn() is False
        ctx.end_turn()
        assert ctx.should_end_turn() is True

    def test_last_card_type(self):
        ctx = make_ctx()
        assert ctx.get_last_card_type() is None
        ctx.set_last_card_type("ATTACK")
        assert ctx.get_last_card_type() == "ATTACK"

    def test_move_card_from_discard_to_hand(self):
        state = make_state(hand=[], discard=["A", "B"])
        ctx = make_ctx(state=state)
        assert ctx.move_card_from_discard_to_hand("A") is True
        assert "A" in state.hand
        assert "A" not in state.discard_pile

    def test_is_enemy_attacking(self):
        enemy = make_enemy(move_damage=10)
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        assert ctx.is_enemy_attacking() is True

        passive_enemy = make_enemy(move_damage=0, is_attacking=False)
        state2 = make_state(enemies=[passive_enemy])
        ctx2 = make_ctx(state=state2, target=passive_enemy, target_idx=0)
        assert ctx2.is_enemy_attacking() is False

    def test_deal_damage_to_all_enemies(self):
        e1 = make_enemy(hp=20)
        e2 = make_enemy(hp=30)
        state = make_state(enemies=[e1, e2])
        ctx = make_ctx(state=state)
        total = ctx.deal_damage_to_all_enemies(10)
        assert e1.hp == 10
        assert e2.hp == 20
        assert total == 20


# =============================================================================
# EffectExecutor Tests
# =============================================================================

class TestEffectExecutor:
    def test_play_strike(self):
        enemy = make_enemy(hp=50)
        state = make_state(enemies=[enemy], hand=["Strike_P"], energy=3)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        assert result.success is True
        assert result.energy_spent == 1
        assert state.energy == 2
        assert enemy.hp < 50

    def test_play_defend(self):
        state = make_state(hand=["Defend_P"], energy=3)
        card = get_card("Defend_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=-1)
        assert result.success is True
        assert state.player.block > 0

    def test_play_card_not_enough_energy(self):
        state = make_state(energy=0)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        assert result.success is False

    def test_play_card_free(self):
        state = make_state(energy=0)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0, free=True)
        assert result.success is True
        assert state.energy == 0

    def test_play_eruption_enters_wrath(self):
        enemy = make_enemy(hp=50)
        state = make_state(enemies=[enemy], energy=3, stance="Neutral")
        card = get_card("Eruption")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        assert result.success is True
        assert state.stance == "Wrath"
        assert result.stance_changed_to == "Wrath"

    def test_play_vigilance_enters_calm(self):
        state = make_state(energy=3, stance="Neutral")
        card = get_card("Vigilance")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=-1)
        assert result.success is True
        assert state.stance == "Calm"
        assert state.player.block > 0

    def test_wrath_doubles_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, stance="Wrath")
        card = get_card("Strike_P")  # 6 damage base
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 damage * 2 (wrath) = 12
        assert enemy.hp == 88

    def test_divinity_triples_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=5, stance="Divinity")
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 * 3 = 18
        assert enemy.hp == 82

    def test_strength_adds_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, player_statuses={"Strength": 3})
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 + 3 = 9
        assert enemy.hp == 91

    def test_weak_reduces_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, player_statuses={"Weak": 2})
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # floor(6 * 0.75) = 4
        assert enemy.hp == 96

    def test_vulnerable_increases_damage(self):
        enemy = make_enemy(hp=100, statuses={"Vulnerable": 2})
        state = make_state(enemies=[enemy], energy=3)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # floor(6 * 1.5) = 9
        assert enemy.hp == 91

    def test_dexterity_adds_block(self):
        state = make_state(energy=3, player_statuses={"Dexterity": 3})
        card = get_card("Defend_P")  # 5 block base
        executor = EffectExecutor(state)
        result = executor.play_card(card)
        # 5 + 3 = 8
        assert state.player.block == 8

    def test_frail_reduces_block(self):
        state = make_state(energy=3, player_statuses={"Frail": 2})
        card = get_card("Defend_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card)
        # floor(5 * 0.75) = 3
        assert state.player.block == 3

    def test_play_card_tracks_type(self):
        state = make_state(energy=3)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        executor.play_card(card, target_idx=0)
        assert state.cards_played_this_turn == 1
        assert state.attacks_played_this_turn == 1

    def test_play_skill_tracks_type(self):
        state = make_state(energy=3)
        card = get_card("Defend_P")
        executor = EffectExecutor(state)
        executor.play_card(card)
        assert state.skills_played_this_turn == 1

    def test_on_card_played_hook(self):
        hooks_called = []
        state = make_state(energy=3)
        executor = EffectExecutor(state)
        executor.register_on_card_played(lambda ctx, card: hooks_called.append(card.id))
        card = get_card("Strike_P")
        executor.play_card(card, target_idx=0)
        assert "Strike_P" in hooks_called

    def test_execute_effect_method(self):
        state = make_state(draw=["A", "B", "C"], hand=[])
        executor = EffectExecutor(state)
        result = executor.execute_effect("draw_2")
        assert result.success is True
        assert len(state.hand) == 2

    def test_start_of_turn_foresight(self):
        state = make_state(draw=["A", "B", "C"], player_statuses={"Foresight": 2})
        executor = EffectExecutor(state)
        result = executor.apply_start_of_turn_effects()
        assert any("foresight" in e for e in result.effects_executed)

    def test_start_of_turn_deva_form(self):
        state = make_state(energy=3, player_statuses={"DevaForm": 2})
        executor = EffectExecutor(state)
        result = executor.apply_start_of_turn_effects()
        assert state.energy == 5  # 3 + 2
        assert result.energy_gained >= 2

    def test_end_of_turn_like_water(self):
        state = make_state(stance="Calm", player_statuses={"LikeWater": 5})
        executor = EffectExecutor(state)
        result = executor.apply_end_of_turn_effects()
        assert result.block_gained == 5

    def test_end_of_turn_divinity_exits(self):
        state = make_state(stance="Divinity")
        executor = EffectExecutor(state)
        result = executor.apply_end_of_turn_effects()
        assert state.stance == "Neutral"
        assert result.stance_changed_to == "Neutral"

    def test_end_of_turn_study(self):
        state = make_state(draw=[], player_statuses={"Study": 1})
        executor = EffectExecutor(state)
        result = executor.apply_end_of_turn_effects()
        assert any("study" in e for e in result.effects_executed)
        assert len(state.draw_pile) == 1

    def test_end_of_turn_blasphemy(self):
        state = make_state(player_statuses={"Blasphemy": 1}, player_hp=50)
        executor = EffectExecutor(state)
        result = executor.apply_end_of_turn_effects()
        assert state.player.hp == 0
        assert "blasphemy_death" in result.effects_executed

    def test_create_executor_factory(self):
        state = make_state()
        executor = create_executor(state)
        assert isinstance(executor, EffectExecutor)
        assert executor.state is state

    def test_effect_result_should_end_turn(self):
        r = EffectResult(success=True)
        assert r.should_end_turn is False
        r.extra["end_turn"] = True
        assert r.should_end_turn is True

    def test_play_empty_fist_exits_stance(self):
        enemy = make_enemy(hp=50)
        state = make_state(enemies=[enemy], energy=3, stance="Wrath")
        card = get_card("EmptyFist")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        assert state.stance == "Neutral"

    def test_play_wheel_kick_draws(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, hand=[], draw=["A", "B", "C", "D"])
        card = get_card("WheelKick")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        assert len(state.hand) == 2

    def test_play_consecrate_aoe(self):
        e1 = make_enemy(hp=50)
        e2 = make_enemy(hp=50)
        state = make_state(enemies=[e1, e2], energy=3)
        card = get_card("Consecrate")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=-1)
        assert e1.hp < 50
        assert e2.hp < 50


# =============================================================================
# Card Effect Implementation Tests (effects/cards.py)
# =============================================================================

class TestCardEffects:
    def test_draw_1(self):
        state = make_state(hand=[], draw=["A", "B"])
        ctx = make_ctx(state=state)
        execute_effect("draw_1", ctx)
        assert len(state.hand) == 1

    def test_draw_2(self):
        state = make_state(hand=[], draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        execute_effect("draw_2", ctx)
        assert len(state.hand) == 2

    def test_draw_3(self):
        state = make_state(hand=[], draw=["A", "B", "C", "D"])
        ctx = make_ctx(state=state)
        execute_effect("draw_3", ctx)
        assert len(state.hand) == 3

    def test_enter_wrath(self):
        state = make_state(stance="Neutral")
        ctx = make_ctx(state=state)
        execute_effect("enter_wrath", ctx)
        assert state.stance == "Wrath"

    def test_enter_calm(self):
        state = make_state(stance="Neutral")
        ctx = make_ctx(state=state)
        execute_effect("enter_calm", ctx)
        assert state.stance == "Calm"

    def test_enter_divinity(self):
        state = make_state(stance="Neutral", energy=0)
        ctx = make_ctx(state=state)
        execute_effect("enter_divinity", ctx)
        assert state.stance == "Divinity"
        assert state.energy == 3  # divinity grants 3 energy

    def test_exit_stance(self):
        state = make_state(stance="Wrath")
        ctx = make_ctx(state=state)
        execute_effect("exit_stance", ctx)
        assert state.stance == "Neutral"

    def test_gain_mantra(self):
        state = make_state(player_statuses={})
        ctx = make_ctx(state=state, magic=3)
        execute_effect("gain_mantra", ctx)
        assert ctx.mantra_gained == 3

    def test_gain_mantra_2(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("gain_mantra_2", ctx)
        assert ctx.mantra_gained == 2

    def test_gain_mantra_5(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("gain_mantra_5", ctx)
        assert ctx.mantra_gained == 5

    def test_scry_1(self):
        state = make_state(draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        execute_effect("scry_1", ctx)
        assert len(ctx.scried_cards) == 1

    def test_scry_2(self):
        state = make_state(draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        execute_effect("scry_2", ctx)
        assert len(ctx.scried_cards) == 2

    def test_scry_3(self):
        state = make_state(draw=["A", "B", "C", "D"])
        ctx = make_ctx(state=state)
        execute_effect("scry_3", ctx)
        assert len(ctx.scried_cards) == 3

    def test_apply_weak(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("apply_weak_2", ctx)
        assert enemy.statuses["Weak"] == 2

    def test_apply_vulnerable(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("apply_vulnerable_3", ctx)
        assert enemy.statuses["Vulnerable"] == 3

    def test_apply_strength(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("apply_strength_3", ctx)
        assert state.player.statuses["Strength"] == 3

    def test_apply_dexterity(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("apply_dexterity_2", ctx)
        assert state.player.statuses["Dexterity"] == 2

    def test_add_insight_to_draw(self):
        state = make_state(draw=[])
        ctx = make_ctx(state=state)
        execute_effect("add_insight_to_draw", ctx)
        assert state.draw_pile[-1] == "Insight"

    def test_add_smite_to_hand(self):
        state = make_state(hand=[])
        ctx = make_ctx(state=state)
        execute_effect("add_smite_to_hand", ctx)
        assert "Smite" in state.hand

    def test_add_safety_to_hand(self):
        state = make_state(hand=[])
        ctx = make_ctx(state=state)
        execute_effect("add_safety_to_hand", ctx)
        assert "Safety" in state.hand

    def test_add_through_violence_to_draw(self):
        state = make_state(draw=[])
        ctx = make_ctx(state=state)
        execute_effect("add_through_violence_to_draw", ctx)
        assert "ThroughViolence" in state.draw_pile

    def test_shuffle_beta_into_draw(self):
        state = make_state(draw=[])
        ctx = make_ctx(state=state)
        execute_effect("shuffle_beta_into_draw", ctx)
        assert "Beta" in state.draw_pile

    def test_shuffle_omega_into_draw(self):
        state = make_state(draw=[])
        ctx = make_ctx(state=state)
        execute_effect("shuffle_omega_into_draw", ctx)
        assert "Omega" in state.draw_pile

    def test_end_turn_effect(self):
        ctx = make_ctx()
        execute_effect("end_turn", ctx)
        assert ctx.should_end_turn() is True

    def test_take_extra_turn(self):
        ctx = make_ctx()
        execute_effect("take_extra_turn", ctx)
        assert ctx.extra_data.get("extra_turn") is True

    def test_die_next_turn(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("die_next_turn", ctx)
        assert state.player.statuses.get("Blasphemy", 0) > 0

    def test_draw_until_hand_full(self):
        state = make_state(hand=["A", "B"], draw=list("CDEFGHIJKL"))
        ctx = make_ctx(state=state)
        execute_effect("draw_until_hand_full", ctx)
        assert len(state.hand) == 10

    def test_gain_1_energy(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state)
        execute_effect("gain_1_energy", ctx)
        assert state.energy == 3

    def test_gain_1_energy_upgraded(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state, upgraded=True)
        execute_effect("gain_1_energy", ctx)
        assert state.energy == 4

    def test_if_last_card_attack_gain_energy(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state)
        ctx.set_last_card_type("ATTACK")
        execute_effect("if_last_card_attack_gain_energy", ctx)
        assert state.energy == 3

    def test_if_last_card_attack_gain_energy_no_attack(self):
        state = make_state(energy=2)
        ctx = make_ctx(state=state)
        ctx.set_last_card_type("SKILL")
        execute_effect("if_last_card_attack_gain_energy", ctx)
        assert state.energy == 2

    def test_if_calm_draw_3_else_calm_in_calm(self):
        state = make_state(stance="Calm", hand=[], draw=list("ABCDE"))
        ctx = make_ctx(state=state)
        execute_effect("if_calm_draw_3_else_calm", ctx)
        assert len(state.hand) == 3

    def test_if_calm_draw_3_else_calm_not_calm(self):
        state = make_state(stance="Neutral")
        ctx = make_ctx(state=state)
        execute_effect("if_calm_draw_3_else_calm", ctx)
        assert state.stance == "Calm"

    def test_if_wrath_gain_mantra_else_wrath_in_wrath(self):
        state = make_state(stance="Wrath")
        ctx = make_ctx(state=state)
        execute_effect("if_wrath_gain_mantra_else_wrath", ctx)
        assert ctx.mantra_gained == 3

    def test_if_wrath_gain_mantra_else_wrath_not_wrath(self):
        state = make_state(stance="Neutral")
        ctx = make_ctx(state=state)
        execute_effect("if_wrath_gain_mantra_else_wrath", ctx)
        assert state.stance == "Wrath"

    def test_if_in_wrath_extra_block(self):
        state = make_state(stance="Wrath")
        ctx = make_ctx(state=state)
        execute_effect("if_in_wrath_extra_block_6", ctx)
        assert state.player.block == 6

    def test_if_in_wrath_extra_block_not_wrath(self):
        state = make_state(stance="Neutral")
        ctx = make_ctx(state=state)
        execute_effect("if_in_wrath_extra_block_6", ctx)
        assert state.player.block == 0

    def test_if_enemy_attacking_enter_calm(self):
        enemy = make_enemy(move_damage=10)
        state = make_state(enemies=[enemy], stance="Neutral")
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("if_enemy_attacking_enter_calm", ctx)
        assert state.stance == "Calm"

    def test_if_enemy_attacking_enter_calm_not_attacking(self):
        enemy = make_enemy(move_damage=0, is_attacking=False)
        state = make_state(enemies=[enemy], stance="Neutral")
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("if_enemy_attacking_enter_calm", ctx)
        assert state.stance == "Neutral"

    def test_if_enemy_hp_below_kill(self):
        enemy = make_enemy(hp=25)
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("if_enemy_hp_below_kill", ctx)
        assert enemy.hp == 0

    def test_if_enemy_hp_above_threshold(self):
        enemy = make_enemy(hp=50)
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0)
        execute_effect("if_enemy_hp_below_kill", ctx)
        assert enemy.hp == 50

    def test_gain_block_per_card_in_hand(self):
        state = make_state(hand=["A", "B", "C"])
        ctx = make_ctx(state=state, magic=4)
        execute_effect("gain_block_per_card_in_hand", ctx)
        assert state.player.block == 12  # 4 * 3

    def test_apply_mark(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0, magic=8)
        execute_effect("apply_mark", ctx)
        assert enemy.statuses["Mark"] == 8

    def test_trigger_all_marks(self):
        e1 = make_enemy(hp=50, statuses={"Mark": 10})
        e2 = make_enemy(hp=50, statuses={"Mark": 5})
        state = make_state(enemies=[e1, e2])
        ctx = make_ctx(state=state)
        execute_effect("trigger_all_marks", ctx)
        assert e1.hp == 40
        assert e2.hp == 45

    def test_apply_block_return(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        ctx = make_ctx(state=state, target=enemy, target_idx=0, magic=3)
        execute_effect("apply_block_return", ctx)
        assert enemy.statuses["BlockReturn"] == 3

    def test_on_stance_change_gain_block(self):
        state = make_state()
        ctx = make_ctx(state=state, magic=6)
        execute_effect("on_stance_change_gain_block", ctx)
        assert state.player.statuses["MentalFortress"] == 6

    def test_on_scry_gain_block(self):
        state = make_state()
        ctx = make_ctx(state=state, magic=4)
        execute_effect("on_scry_gain_block", ctx)
        assert state.player.statuses["Nirvana"] == 4

    def test_on_wrath_draw(self):
        state = make_state()
        ctx = make_ctx(state=state, magic=2)
        execute_effect("on_wrath_draw", ctx)
        assert state.player.statuses["Rushdown"] == 2

    def test_gain_mantra_each_turn(self):
        state = make_state()
        ctx = make_ctx(state=state, magic=3)
        execute_effect("gain_mantra_each_turn", ctx)
        assert state.player.statuses["Devotion"] == 3

    def test_gain_mantra_add_insight(self):
        state = make_state(draw=[], player_statuses={})
        ctx = make_ctx(state=state, magic=3)
        execute_effect("gain_mantra_add_insight", ctx)
        assert ctx.mantra_gained == 3
        assert "Insight" in state.draw_pile

    def test_heal_magic_number(self):
        state = make_state(player_hp=50)
        ctx = make_ctx(state=state, magic=7)
        execute_effect("heal_magic_number", ctx)
        assert state.player.hp == 57

    def test_put_cards_from_discard_to_hand(self):
        state = make_state(hand=[], discard=["A", "B", "C"])
        ctx = make_ctx(state=state, magic=2)
        execute_effect("put_cards_from_discard_to_hand", ctx)
        assert len(state.hand) == 2

    def test_gain_strength_and_dex(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("gain_strength_and_dex_lose_focus", ctx)
        assert state.player.statuses["Strength"] == 3
        assert state.player.statuses["Dexterity"] == 3

    def test_gain_artifact(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("gain_artifact", ctx)
        assert state.player.statuses["Artifact"] == 1

    def test_omega_power(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("deal_50_damage_end_turn", ctx)
        assert state.player.statuses["Omega"] == 50

    def test_wish_effect_default_strength(self):
        state = make_state()
        ctx = make_ctx(state=state)
        execute_effect("choose_plated_armor_or_strength_or_gold", ctx)
        assert state.player.statuses.get("Strength", 0) == 3

    def test_cost_0_in_wrath_marker(self):
        # Just verify it executes without error
        ctx = make_ctx()
        execute_effect("cost_0_in_wrath", ctx)

    def test_unplayable(self):
        ctx = make_ctx()
        assert execute_effect("unplayable", ctx) is True


# =============================================================================
# Card Effects Utility Functions
# =============================================================================

class TestCardEffectsUtils:
    def test_get_card_effects(self):
        effects = get_card_effects("Eruption")
        assert "enter_wrath" in effects

    def test_get_card_effects_upgraded(self):
        effects = get_card_effects("Eruption+")
        assert "enter_wrath" in effects

    def test_get_card_effects_unknown(self):
        assert get_card_effects("NonexistentCard") == []

    def test_get_card_cost_modifier_scrawl_wrath(self):
        assert get_card_cost_modifier("Scrawl", "Wrath") == 0

    def test_get_card_cost_modifier_scrawl_neutral(self):
        assert get_card_cost_modifier("Scrawl", "Neutral") is None

    def test_get_card_cost_modifier_other(self):
        assert get_card_cost_modifier("Strike_P", "Wrath") is None

    def test_trigger_on_stance_change_mental_fortress(self):
        state = make_state(player_statuses={"MentalFortress": 5})
        ctx = make_ctx(state=state)
        trigger_on_stance_change(ctx, "Neutral", "Wrath")
        assert state.player.block == 5

    def test_trigger_on_stance_change_rushdown(self):
        state = make_state(player_statuses={"Rushdown": 2}, draw=["A", "B", "C"], hand=[])
        ctx = make_ctx(state=state)
        trigger_on_stance_change(ctx, "Neutral", "Wrath")
        assert len(state.hand) == 2

    def test_trigger_on_stance_change_flurry(self):
        state = make_state(hand=[], discard=["FlurryOfBlows"])
        ctx = make_ctx(state=state)
        trigger_on_stance_change(ctx, "Neutral", "Wrath")
        assert "FlurryOfBlows" in state.hand

    def test_trigger_on_scry_nirvana(self):
        state = make_state(player_statuses={"Nirvana": 3})
        ctx = make_ctx(state=state)
        trigger_on_scry(ctx, ["A", "B"])
        assert state.player.block == 3  # flat per scry action

    def test_trigger_on_scry_weave(self):
        state = make_state(hand=[], discard=["Weave"])
        ctx = make_ctx(state=state)
        trigger_on_scry(ctx, ["A"])
        assert "Weave" in state.hand

    def test_perform_scry_basic(self):
        state = make_state(draw=["A", "B", "C", "D"], hand=[])
        ctx = make_ctx(state=state)
        scried = perform_scry(ctx, 3)
        assert len(scried) == 3

    def test_perform_scry_with_discard(self):
        state = make_state(draw=["A", "B", "C"], hand=[])
        ctx = make_ctx(state=state)
        scried = perform_scry(ctx, 3, discard_indices=[0, 2])
        # 2 cards discarded, 1 remains on draw
        assert len(state.discard_pile) == 2

    def test_can_play_card_normal(self):
        state = make_state(hand=["Strike_P", "Defend_P"])
        ctx = make_ctx(state=state)
        can, reason = can_play_card(ctx, "Strike_P")
        assert can is True

    def test_can_play_signature_move_only_attack(self):
        state = make_state(hand=["SignatureMove"])
        ctx = make_ctx(state=state)
        can, reason = can_play_card(ctx, "SignatureMove")
        assert can is True

    def test_can_play_signature_move_blocked(self):
        state = make_state(hand=["SignatureMove", "Strike_P"])
        ctx = make_ctx(state=state)
        can, reason = can_play_card(ctx, "SignatureMove")
        assert can is False

    def test_gain_mantra_and_check_divinity(self):
        state = make_state(stance="Neutral", player_statuses={"Mantra": 8})
        ctx = make_ctx(state=state)
        result = gain_mantra_and_check_divinity(ctx, 5)
        assert result["divinity_triggered"] is True

    def test_gain_mantra_and_check_divinity_no_trigger(self):
        state = make_state(player_statuses={"Mantra": 2})
        ctx = make_ctx(state=state)
        result = gain_mantra_and_check_divinity(ctx, 3)
        assert result["divinity_triggered"] is False
        assert result["total_mantra"] == 5

    def test_execute_card_effects(self):
        state = make_state(hand=[], draw=["A", "B", "C"])
        ctx = make_ctx(state=state)
        execute_card_effects(ctx, ["draw_2"])
        assert len(state.hand) == 2

    def test_apply_start_of_turn_effects_devotion(self):
        state = make_state(player_statuses={"Devotion": 3, "Mantra": 0})
        ctx = make_ctx(state=state)
        result = apply_start_of_turn_effects(ctx)
        assert result["mantra_gained"] == 3

    def test_apply_start_of_turn_effects_simmering_fury(self):
        state = make_state(
            stance="Neutral",
            player_statuses={"SimmeringFury": 2},
            draw=["A", "B", "C"], hand=[],
        )
        ctx = make_ctx(state=state)
        result = apply_start_of_turn_effects(ctx)
        assert state.stance == "Wrath"
        assert result["cards_drawn"] == 2

    def test_apply_end_of_turn_effects_like_water(self):
        state = make_state(stance="Calm", player_statuses={"LikeWater": 7})
        ctx = make_ctx(state=state)
        result = apply_end_of_turn_effects(ctx)
        assert result["block_gained"] == 7

    def test_apply_end_of_turn_effects_omega(self):
        e1 = make_enemy(hp=100)
        state = make_state(enemies=[e1], player_statuses={"Omega": 50})
        ctx = make_ctx(state=state)
        result = apply_end_of_turn_effects(ctx)
        assert e1.hp == 50
        assert result["damage_dealt"] == 50

    def test_apply_end_of_turn_effects_divinity_exit(self):
        state = make_state(stance="Divinity")
        ctx = make_ctx(state=state)
        result = apply_end_of_turn_effects(ctx)
        assert state.stance == "Neutral"

    def test_apply_end_of_turn_blasphemy(self):
        state = make_state(player_hp=50, player_statuses={"Blasphemy": 1})
        ctx = make_ctx(state=state)
        result = apply_end_of_turn_effects(ctx)
        # Blasphemy decrements: 1 -> 0 means death
        assert result["player_died"] is True
        assert state.player.hp == 0

    def test_apply_end_of_turn_study(self):
        state = make_state(draw=[], player_statuses={"Study": 1})
        ctx = make_ctx(state=state)
        result = apply_end_of_turn_effects(ctx)
        assert "Insight" in state.draw_pile

    def test_watcher_card_effects_completeness(self):
        """Verify key cards have effects registered."""
        assert "Eruption" in WATCHER_CARD_EFFECTS
        assert "Vigilance" in WATCHER_CARD_EFFECTS
        assert "WheelKick" in WATCHER_CARD_EFFECTS
        assert "Ragnarok" in WATCHER_CARD_EFFECTS
        assert "Blasphemy" in WATCHER_CARD_EFFECTS
        assert "MentalFortress" in WATCHER_CARD_EFFECTS


# =============================================================================
# Combat Handler (handlers/combat.py) Tests
# =============================================================================

class TestCombatRunner:
    """Tests for CombatRunner from handlers/combat.py."""

    def _make_runner(self, enemies=None, deck=None, relics=None, ascension=0, hp=80):
        from packages.engine.handlers.combat import CombatRunner, CombatPhase
        from packages.engine.state.run import create_watcher_run
        from packages.engine.state.rng import Random
        from packages.engine.content.enemies import JawWorm

        run = create_watcher_run("TEST", ascension=ascension)
        run._current_hp = hp
        run._max_hp = hp

        if deck:
            run._deck = deck

        if relics:
            for r in relics:
                run.add_relic(r)

        rng = Random(12345)
        ai_rng = Random(12346)
        hp_rng = Random(12347)

        if enemies is None:
            enemies = [JawWorm(ai_rng=ai_rng, ascension=ascension, hp_rng=hp_rng)]

        return CombatRunner(
            run_state=run,
            enemies=enemies,
            shuffle_rng=rng,
        )

    def test_combat_runner_init(self):
        runner = self._make_runner()
        assert runner.state is not None
        assert runner.state.player.hp > 0
        assert len(runner.state.enemies) == 1
        assert runner.phase.value == "PLAYER_TURN"
        assert runner.combat_over is False

    def test_combat_runner_has_hand(self):
        runner = self._make_runner()
        assert len(runner.state.hand) >= 5

    def test_combat_runner_get_legal_actions(self):
        runner = self._make_runner()
        actions = runner.get_legal_actions()
        assert len(actions) > 0
        # Should have at least EndTurn
        assert any(isinstance(a, EndTurn) for a in actions)

    def test_combat_runner_play_card(self):
        runner = self._make_runner()
        initial_hand = len(runner.state.hand)
        initial_hp = runner.state.enemies[0].hp
        # Play first card
        result = runner.play_card(0, target_idx=0)
        assert result["success"] is True
        # Hand should shrink
        assert len(runner.state.hand) < initial_hand

    def test_combat_runner_play_card_invalid(self):
        runner = self._make_runner()
        result = runner.play_card(99, target_idx=0)
        assert result["success"] is False

    def test_combat_runner_end_turn(self):
        from packages.engine.handlers.combat import CombatPhase
        runner = self._make_runner()
        runner.execute_action(EndTurn())
        # After end turn + enemy turn, should be back to player turn or combat over
        assert runner.phase in (CombatPhase.PLAYER_TURN, CombatPhase.COMBAT_END)

    def test_combat_runner_run_to_completion(self):
        runner = self._make_runner()
        result = runner.run()
        assert result.turns_taken >= 1
        assert result.victory is True or result.player_hp_remaining == 0

    def test_combat_result_properties(self):
        from packages.engine.handlers.combat import CombatResult
        r = CombatResult(victory=True, player_hp_remaining=60, player_max_hp=80, turns_taken=5)
        assert r.hp_lost == 20
        assert r.hp_percent_remaining == 0.75

    def test_combat_result_defeat(self):
        from packages.engine.handlers.combat import CombatResult
        r = CombatResult(victory=False, player_hp_remaining=0, player_max_hp=80, turns_taken=3)
        assert r.hp_lost == 80
        assert r.hp_percent_remaining == 0.0

    def test_combat_runner_use_potion(self):
        from packages.engine.handlers.combat import CombatRunner
        from packages.engine.state.run import create_watcher_run
        from packages.engine.state.rng import Random
        from packages.engine.content.enemies import JawWorm

        run = create_watcher_run("TEST", ascension=0)
        # Add a potion
        run.add_potion("Fire Potion")
        rng = Random(12345)
        ai_rng = Random(12346)
        hp_rng = Random(12347)
        enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]

        runner = CombatRunner(run_state=run, enemies=enemies, shuffle_rng=rng)
        initial_hp = runner.state.enemies[0].hp
        result = runner.use_potion(0, target_idx=0)
        assert result["success"] is True
        # Fire potion should deal damage
        assert runner.state.enemies[0].hp < initial_hp

    def test_combat_runner_stance_change(self):
        runner = self._make_runner()
        # Find Eruption in hand
        eruption_idx = None
        for i, card_id in enumerate(runner.state.hand):
            if card_id.startswith("Eruption"):
                eruption_idx = i
                break
        if eruption_idx is not None:
            runner.play_card(eruption_idx, target_idx=0)
            assert runner.state.stance == "Wrath"

    def test_combat_runner_victory(self):
        """Kill a weak enemy to verify victory detection."""
        from packages.engine.content.enemies import JawWorm
        from packages.engine.state.rng import Random

        runner = self._make_runner()
        # Manually kill enemy
        runner.state.enemies[0].hp = 0
        runner._check_combat_end()
        assert runner.combat_over is True
        assert runner.victory is True

    def test_combat_runner_defeat(self):
        runner = self._make_runner()
        runner.state.player.hp = 0
        runner._check_combat_end()
        assert runner.combat_over is True
        assert runner.victory is False

    def test_build_card_registry(self):
        from packages.engine.handlers.combat import build_card_registry, CARD_REGISTRY
        assert len(CARD_REGISTRY) > 20
        assert "Strike_P" in CARD_REGISTRY
        assert "Strike_P+" in CARD_REGISTRY
        assert CARD_REGISTRY["Strike_P"]["type"] == "ATTACK"

    def test_encounter_table(self):
        from packages.engine.handlers.combat import ENCOUNTER_TABLE, create_enemies_from_encounter
        from packages.engine.state.rng import Random
        assert "Jaw Worm" in ENCOUNTER_TABLE
        assert "Cultist" in ENCOUNTER_TABLE

        rng = Random(42)
        enemies = create_enemies_from_encounter("Jaw Worm", rng, ascension=0)
        assert len(enemies) == 1

    def test_encounter_unknown_raises(self):
        from packages.engine.handlers.combat import create_enemies_from_encounter
        from packages.engine.state.rng import Random
        with pytest.raises(ValueError, match="Unknown encounter"):
            create_enemies_from_encounter("Fake Encounter", Random(1), 0)

    def test_combat_phase_enum(self):
        from packages.engine.handlers.combat import CombatPhase
        assert CombatPhase.PLAYER_TURN.value == "PLAYER_TURN"
        assert CombatPhase.COMBAT_END.value == "COMBAT_END"


# =============================================================================
# Special Executor Effects (more complex card interactions)
# =============================================================================

class TestExecutorSpecialEffects:
    """Test the special effects handled in executor._handle_special_effect."""

    def test_if_last_card_attack_weak(self):
        enemy = make_enemy()
        state = make_state(enemies=[enemy])
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card, target=enemy, target_idx=0)
        ctx.set_last_card_type("ATTACK")
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("if_last_card_attack_weak_1", ctx, card, result)
        assert handled is True
        assert enemy.statuses.get("Weak", 0) > 0

    def test_gain_block_equal_unblocked_damage(self):
        enemy = make_enemy(hp=50)
        state = make_state(enemies=[enemy])
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card, target=enemy, target_idx=0)
        ctx.damage_dealt = 10
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("gain_block_equal_unblocked_damage", ctx, card, result)
        assert handled is True
        assert state.player.block == 10

    def test_end_turn_special(self):
        state = make_state()
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card)
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("end_turn", ctx, card, result)
        assert handled is True
        assert ctx.should_end_turn() is True

    def test_enter_divinity_special(self):
        state = make_state(stance="Neutral", energy=0)
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card)
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("enter_divinity", ctx, card, result)
        assert handled is True
        assert state.stance == "Divinity"

    def test_enter_calm_special(self):
        state = make_state(stance="Neutral")
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card)
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("enter_calm", ctx, card, result)
        assert handled is True
        assert state.stance == "Calm"

    def test_scry_pattern(self):
        state = make_state(draw=["A", "B", "C"])
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card)
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("scry_2", ctx, card, result)
        assert handled is True

    def test_unknown_special_effect_returns_false(self):
        state = make_state()
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        ctx = make_ctx(state=state, card=card)
        result = EffectResult(success=True)
        handled = executor._handle_special_effect("totally_unknown_xyz", ctx, card, result)
        assert handled is False

    def test_pen_nib_double_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, relics=["Pen Nib"])
        state.relic_counters["Pen Nib"] = 9  # 10th attack triggers
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 * 2 = 12
        assert enemy.hp == 88

    def test_vigor_adds_damage(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, player_statuses={"Vigor": 8})
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 + 8 = 14
        assert enemy.hp == 86
        # Vigor consumed
        assert state.player.statuses["Vigor"] == 0

    def test_wreath_of_flame(self):
        enemy = make_enemy(hp=100)
        state = make_state(enemies=[enemy], energy=3, player_statuses={"WreathOfFlame": 5})
        card = get_card("Strike_P")
        executor = EffectExecutor(state)
        result = executor.play_card(card, target_idx=0)
        # 6 + 5 = 11
        assert enemy.hp == 89
        assert state.player.statuses["WreathOfFlame"] == 0
