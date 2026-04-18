"""Integration tests verifying that trigger hooks are correctly wired.

Tests cover:
1. onRestOption (Girya lift) - rest site -> combat Strength flow
2. onAttack (Envenom, Thorns, Thievery) - power triggers fire during combat
3. onLoseHpLast (Tungsten Rod) - hardcoded HP reduction in combat engine
"""

import pytest

from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState, create_combat,
)
from packages.engine.state.run import create_watcher_run
from packages.engine.registry import (
    execute_power_triggers, execute_relic_triggers,
    POWER_REGISTRY, RELIC_REGISTRY,
)
from packages.engine.handlers.rooms import RestHandler
from packages.engine.handlers.combat import CombatRunner
from packages.engine.combat_engine import CombatEngine
from packages.engine.state.rng import Random
from packages.engine.content.enemies import JawWorm


# =============================================================================
# HOOK 1: onRestOption / Girya lift -> combat Strength
# =============================================================================

class TestGiryaRestToCombat:
    """Girya: lift at rest site grants Strength at next combat start."""

    def test_girya_lift_increments_counter(self):
        """RestHandler.lift() increments Girya counter each use."""
        run = create_watcher_run("GIRYA_LIFT", ascension=0)
        run.add_relic("Girya")

        result = RestHandler.lift(run)
        assert result.strength_gained == 1
        assert run.get_relic("Girya").counter == 1

    def test_girya_lift_caps_at_3(self):
        """Girya can only be used 3 times total."""
        run = create_watcher_run("GIRYA_CAP", ascension=0)
        run.add_relic("Girya")

        for _ in range(3):
            result = RestHandler.lift(run)
            assert result.strength_gained == 1

        # 4th lift fails
        result = RestHandler.lift(run)
        assert result.strength_gained == 0
        assert run.get_relic("Girya").counter == 3

    def test_girya_lift_option_disappears_after_3_uses(self):
        """Lift option no longer shows after 3 uses."""
        run = create_watcher_run("GIRYA_OPT", ascension=0)
        run.add_relic("Girya")

        for _ in range(3):
            RestHandler.lift(run)

        options = RestHandler.get_options(run)
        assert "lift" not in options

    def test_girya_combat_strength_matches_lift_count(self):
        """atBattleStart grants Strength equal to lift counter."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"], relics=["Girya"],
        )
        state.set_relic_counter("Girya", 2)

        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 2

    def test_girya_zero_lifts_no_strength(self):
        """Girya with counter=0 grants no Strength."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="test")],
            deck=["Strike"], relics=["Girya"],
        )
        state.set_relic_counter("Girya", 0)

        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Strength", 0) == 0

    def test_girya_full_flow_lift_then_combat(self):
        """End-to-end: lift 2x at rest, then verify Strength in next combat."""
        run = create_watcher_run("GIRYA_FULL", ascension=0)
        run.add_relic("Girya")

        # Two lifts at rest sites
        RestHandler.lift(run)
        RestHandler.lift(run)
        assert run.get_relic("Girya").counter == 2

        # Enter combat using proper Enemy objects
        rng = Random(12345)
        ai_rng = Random(12346)
        hp_rng = Random(12347)
        enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]
        runner = CombatRunner(run_state=run, enemies=enemies, shuffle_rng=rng)

        # Girya should have granted 2 Strength at combat start
        assert runner.state.player.statuses.get("Strength", 0) == 2

    def test_onRestOption_handler_registered(self):
        """The onRestOption handler is registered for Girya."""
        assert RELIC_REGISTRY.has_handler("onRestOption", "Girya")

    def test_atBattleStart_handler_registered(self):
        """The atBattleStart handler is registered for Girya."""
        assert RELIC_REGISTRY.has_handler("atBattleStart", "Girya")


# =============================================================================
# HOOK 2: onAttack power triggers (Envenom, Thorns, Thievery)
# =============================================================================

class TestEnvenomOnAttack:
    """Envenom: apply Poison to target on unblocked attack damage."""

    def test_envenom_handler_registered(self):
        """onAttack handler exists for Envenom."""
        assert POWER_REGISTRY.has_handler("onAttack", "Envenom")

    def test_envenom_applies_poison_on_unblocked_hit(self):
        """Envenom applies Poison when player deals unblocked damage."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="target")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        # Give player Envenom (1 stack = apply 1 Poison per hit)
        state.player.statuses["Envenom"] = 1

        execute_power_triggers(
            "onAttack",
            state,
            state.player,
            {
                "target": enemy,
                "damage": 6,
                "unblocked_damage": 6,
                "damage_type": "NORMAL",
            },
        )

        assert enemy.statuses.get("Poison", 0) == 1

    def test_envenom_no_poison_if_fully_blocked(self):
        """Envenom does NOT apply Poison when damage is fully blocked."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="target")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        state.player.statuses["Envenom"] = 1

        execute_power_triggers(
            "onAttack",
            state,
            state.player,
            {
                "target": enemy,
                "damage": 6,
                "unblocked_damage": 0,  # fully blocked
                "damage_type": "NORMAL",
            },
        )

        assert enemy.statuses.get("Poison", 0) == 0

    def test_envenom_stacks_apply_proportionally(self):
        """Envenom 3 applies 3 Poison per unblocked hit."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="target")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        state.player.statuses["Envenom"] = 3

        execute_power_triggers(
            "onAttack",
            state,
            state.player,
            {
                "target": enemy,
                "damage": 6,
                "unblocked_damage": 3,
                "damage_type": "NORMAL",
            },
        )

        assert enemy.statuses.get("Poison", 0) == 3


class TestThieveryOnAttack:
    """Thievery: steal gold from player on unblocked attack."""

    def test_thievery_handler_registered(self):
        """onAttack handler exists for Thievery."""
        assert POWER_REGISTRY.has_handler("onAttack", "Thievery")

    def test_thievery_steals_gold(self):
        """Enemy with Thievery steals gold on unblocked hit."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="thief")],
            deck=["Strike"],
        )
        state.gold = 100
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15  # steal 15 gold per hit

        execute_power_triggers(
            "onAttack",
            state,
            enemy,
            {
                "target": state.player,
                "damage": 10,
                "unblocked_damage": 5,
                "damage_type": "NORMAL",
            },
        )

        assert state.gold == 85  # 100 - 15

    def test_thievery_no_steal_if_blocked(self):
        """Thievery does NOT steal gold when damage is fully blocked."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="thief")],
            deck=["Strike"],
        )
        state.gold = 100
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15

        execute_power_triggers(
            "onAttack",
            state,
            enemy,
            {
                "target": state.player,
                "damage": 10,
                "unblocked_damage": 0,
                "damage_type": "NORMAL",
            },
        )

        assert state.gold == 100  # unchanged

    def test_thievery_cannot_steal_more_than_player_has(self):
        """Thievery steals min(amount, gold)."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="thief")],
            deck=["Strike"],
        )
        state.gold = 5
        enemy = state.enemies[0]
        enemy.statuses["Thievery"] = 15

        execute_power_triggers(
            "onAttack",
            state,
            enemy,
            {
                "target": state.player,
                "damage": 10,
                "unblocked_damage": 5,
                "damage_type": "NORMAL",
            },
        )

        assert state.gold == 0  # only had 5


class TestThornsOnAttack:
    """Thorns: deal damage back when attacked."""

    def test_thorns_handler_registered(self):
        """onAttack and onAttacked handlers exist for Thorns."""
        assert POWER_REGISTRY.has_handler("onAttack", "Thorns")
        assert POWER_REGISTRY.has_handler("onAttacked", "Thorns")

    def test_thorns_damages_attacker(self):
        """Thorns deals damage back to attacker through onAttacked."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="attacker")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        state.player.statuses["Thorns"] = 3

        initial_enemy_hp = enemy.hp

        execute_power_triggers(
            "onAttacked",
            state,
            state.player,
            {
                "attacker": enemy,
                "damage": 8,
                "unblocked_damage": 8,
                "damage_type": "NORMAL",
            },
        )

        # Thorns should have dealt 3 damage back to the enemy
        assert enemy.hp == initial_enemy_hp - 3


# =============================================================================
# HOOK 3: onLoseHpLast / Tungsten Rod (hardcoded in combat engine)
# =============================================================================

class TestTungstenRodCombat:
    """Tungsten Rod: reduce HP loss by 1 (hardcoded in combat_engine.py).

    The Tungsten Rod check is hardcoded inline in the combat engine's
    enemy attack resolution (combat_engine.py ~line 718) rather than
    dispatched via execute_relic_triggers("onLoseHpLast", ...).

    This is correct behavior -- the hardcoded check matches Java's
    TungstenRod.onLoseHp which fires inline during damage resolution.
    The registry handler exists for documentation/parity tracking.
    """

    def test_onLoseHpLast_handler_registered(self):
        """The registry handler exists for completeness."""
        assert RELIC_REGISTRY.has_handler("onLoseHpLast", "TungstenRod")

    def test_tungsten_rod_reduces_damage_via_combat_engine(self):
        """Combat engine reduces HP loss by 1 when Tungsten Rod present.

        Uses CombatEngine directly with an enemy that has a pre-set move.
        """
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="slime", name="Slime")],
            deck=["Strike"],
            relics=["Tungsten Rod"],
        )
        enemy = state.enemies[0]
        enemy.move_damage = 10
        enemy.move_hits = 1
        enemy.move_block = 0
        enemy.move_id = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))

        initial_hp = state.player.hp
        engine._execute_enemy_move(enemy)

        # 10 damage - 1 Tungsten Rod = 9 HP loss
        assert state.player.hp == initial_hp - 9

    def test_tungsten_rod_reduces_1_to_0(self):
        """1 HP damage with Tungsten Rod is reduced to 0."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="slime", name="Slime")],
            deck=["Strike"],
            relics=["Tungsten Rod"],
        )
        enemy = state.enemies[0]
        enemy.move_damage = 1
        enemy.move_hits = 1
        enemy.move_block = 0
        enemy.move_id = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))

        initial_hp = state.player.hp
        engine._execute_enemy_move(enemy)

        assert state.player.hp == initial_hp  # 1 - 1 = 0 damage

    def test_no_tungsten_rod_full_damage(self):
        """Without Tungsten Rod, full damage is taken."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="slime", name="Slime")],
            deck=["Strike"],
        )
        enemy = state.enemies[0]
        enemy.move_damage = 10
        enemy.move_hits = 1
        enemy.move_block = 0
        enemy.move_id = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))

        initial_hp = state.player.hp
        engine._execute_enemy_move(enemy)

        assert state.player.hp == initial_hp - 10

    def test_tungsten_rod_with_block(self):
        """Tungsten Rod only reduces UNBLOCKED damage."""
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="slime", name="Slime")],
            deck=["Strike"],
            relics=["Tungsten Rod"],
        )
        enemy = state.enemies[0]
        enemy.move_damage = 10
        enemy.move_hits = 1
        enemy.move_block = 0
        enemy.move_id = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))
        state.player.block = 7

        initial_hp = state.player.hp
        engine._execute_enemy_move(enemy)

        # 10 - 7 block = 3 HP loss, Tungsten Rod -1 = 2
        assert state.player.hp == initial_hp - 2

    def test_tungsten_rod_multi_hit(self):
        """Tungsten Rod applies to EACH hit, not just total."""
        state = create_combat(
            player_hp=80, player_max_hp=80,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="slime", name="Slime")],
            deck=["Strike"],
            relics=["Tungsten Rod"],
        )
        enemy = state.enemies[0]
        enemy.move_damage = 3
        enemy.move_hits = 4
        enemy.move_block = 0
        enemy.move_id = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))

        initial_hp = state.player.hp
        engine._execute_enemy_move(enemy)

        # 4 hits of 3 damage: each hit = 3-1 = 2 HP loss. Total = 8
        assert state.player.hp == initial_hp - 8


# =============================================================================
# HOOK REGISTRATION COVERAGE
# =============================================================================

class TestHookRegistration:
    """Verify all three hook types are properly registered in the registry."""

    def test_onRestOption_enum_exists(self):
        """ON_REST_OPTION is a valid hook enum."""
        from packages.engine.registry import TriggerHook
        assert hasattr(TriggerHook, "ON_REST_OPTION")

    def test_onAttack_handlers_count(self):
        """Multiple onAttack handlers are registered (Envenom, Thorns, Thievery)."""
        handlers = POWER_REGISTRY.get_handlers("onAttack")
        power_ids = [pid for pid, _ in handlers]
        assert "Envenom" in power_ids
        assert "Thorns" in power_ids
        assert "Thievery" in power_ids

    def test_onLoseHpLast_handler_exists(self):
        """onLoseHpLast handler is registered for TungstenRod."""
        assert RELIC_REGISTRY.has_handler("onLoseHpLast", "TungstenRod")

    def test_onAttack_wired_in_combat_engine(self):
        """Verify onAttack fires during player card attack via combat engine.

        Create combat, give player Envenom, play Strike, check Poison applied.
        """
        state = create_combat(
            player_hp=50, player_max_hp=50,
            enemies=[EnemyCombatState(hp=30, max_hp=30, id="target", name="Target")],
            deck=["Strike_P"],
        )
        state.player.statuses["Envenom"] = 1

        engine = CombatEngine(state, shuffle_rng=Random(12345))
        # Draw cards
        engine._draw_cards(5)

        # Find Strike in hand
        strike_idx = None
        for i, card_id in enumerate(state.hand):
            if "Strike" in card_id:
                strike_idx = i
                break

        if strike_idx is not None:
            from packages.engine.state.combat import PlayCard
            action = PlayCard(card_idx=strike_idx, target_idx=0)
            engine.execute_action(action)

            # After playing Strike with Envenom, enemy should have Poison
            enemy = state.enemies[0]
            assert enemy.statuses.get("Poison", 0) >= 1
