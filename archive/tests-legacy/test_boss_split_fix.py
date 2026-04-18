"""
Tests for the Slime Boss split fix.

Regression tests ensuring:
1. Split triggers even when a single hit drops Slime Boss past 50% to 0 HP
2. After children die, combat ends with player_won=True
3. Safety net in get_legal_actions catches stuck combats
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from packages.engine.state.rng import Random
from packages.engine.content.enemies import SlimeBoss, Reptomancer, SnakeDagger, TheCollector, TorchHead, Darkling
from packages.engine.combat_engine import (
    CombatEngine,
    CombatPhase,
    create_combat_from_enemies,
    PlayCard,
    EndTurn,
)


class TestSlimeBossSplitFix:
    """Verify split fires on lethal one-shot damage."""

    def _make_slime_boss_combat(self, boss_hp=140, player_hp=80):
        """Create a combat against Slime Boss with configurable HP."""
        ai_rng = Random(42)
        hp_rng = Random(42)
        boss = SlimeBoss(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)
        engine = create_combat_from_enemies(
            enemies=[boss],
            player_hp=player_hp,
            player_max_hp=player_hp,
            deck=["Strike_P"] * 5 + ["Defend_P"] * 5,
            energy=99,  # unlimited energy for testing
        )
        engine.start_combat()
        # Override boss HP for precise testing
        engine.state.enemies[0].hp = boss_hp
        engine.state.enemies[0].max_hp = 140
        return engine

    def test_split_on_normal_damage(self):
        """Split triggers when damage brings HP below 50% but above 0."""
        engine = self._make_slime_boss_combat(boss_hp=75)
        boss = engine.state.enemies[0]
        # Deal 6 damage: 75 -> 69, below 70 threshold
        engine._deal_damage_to_enemy(boss, 6)
        # Boss should be "dead" (hp=0) and children spawned
        assert boss.hp == 0
        assert len(engine.state.enemies) > 1, "Children should have been spawned"
        assert not engine.state.combat_over, "Combat should continue with children alive"

    def test_split_on_lethal_oneshot(self):
        """Split triggers when a single hit drops HP from above 50% to 0."""
        engine = self._make_slime_boss_combat(boss_hp=80)
        boss = engine.state.enemies[0]
        # Deal 200 damage: 80 -> 0 (clamped), should still trigger split
        engine._deal_damage_to_enemy(boss, 200)
        assert boss.hp == 0
        assert len(engine.state.enemies) > 1, (
            "Children should have been spawned even on lethal one-shot"
        )
        # Children should be alive
        children = [e for e in engine.state.enemies if e is not boss]
        assert any(e.hp > 0 for e in children), "At least one child should be alive"
        assert not engine.state.combat_over

    def test_split_on_exact_lethal(self):
        """Split triggers when damage brings HP to exactly 0 from above threshold."""
        engine = self._make_slime_boss_combat(boss_hp=80)
        boss = engine.state.enemies[0]
        # Deal exactly 80 damage: 80 -> 0
        engine._deal_damage_to_enemy(boss, 80)
        assert boss.hp == 0
        assert len(engine.state.enemies) > 1, "Children should spawn on exact lethal"

    def test_combat_ends_after_children_die(self):
        """After split children die, combat ends with player_won=True.

        Note: Large slimes also split into mediums, so we kill iteratively
        until no living enemies remain. _deal_damage_to_enemy is a low-level
        method that doesn't call _check_combat_end, so we use get_legal_actions
        (which has the safety net) to trigger combat end detection.
        """
        engine = self._make_slime_boss_combat(boss_hp=75)
        boss = engine.state.enemies[0]
        # Trigger split
        engine._deal_damage_to_enemy(boss, 6)
        assert len(engine.state.enemies) > 1

        # Kill everything iteratively (children may split into more children)
        for _ in range(20):  # safety limit
            living = [e for e in engine.state.enemies if e.hp > 0]
            if not living:
                break
            for enemy in living:
                engine._deal_damage_to_enemy(enemy, 9999)

        # All enemies should be dead
        assert all(e.hp <= 0 for e in engine.state.enemies)
        # get_legal_actions safety net should detect and end combat
        actions = engine.get_legal_actions()
        assert actions == []
        assert engine.state.combat_over
        assert engine.state.player_won

    def test_safety_net_in_get_legal_actions(self):
        """get_legal_actions forces combat end when all enemies are dead."""
        engine = self._make_slime_boss_combat(boss_hp=10)
        # Manually set all enemies to 0 HP without going through normal death path
        for enemy in engine.state.enemies:
            enemy.hp = 0

        # get_legal_actions should detect the stuck state and end combat
        actions = engine.get_legal_actions()
        assert actions == []
        assert engine.state.combat_over
        assert engine.state.player_won

    def test_no_double_split(self):
        """Split should only trigger once per Slime Boss."""
        engine = self._make_slime_boss_combat(boss_hp=75)
        boss = engine.state.enemies[0]
        # First hit triggers split
        engine._deal_damage_to_enemy(boss, 6)
        n_enemies_after_first = len(engine.state.enemies)
        # Second hit on the (now dead) boss should not trigger another split
        engine._deal_damage_to_enemy(boss, 1)
        assert len(engine.state.enemies) == n_enemies_after_first

    def test_poison_triggers_split(self):
        """Poison damage must trigger split on Slime Boss (BUG 2 regression).

        Previously, poison ticks bypassed _check_split entirely — poison
        directly decremented HP and called _on_enemy_death without checking
        if the enemy should split first. A poisoned Slime Boss would die
        without spawning children.
        """
        engine = self._make_slime_boss_combat(boss_hp=65, player_hp=80)
        boss = engine.state.enemies[0]
        # Apply 70 poison — enough to kill in one tick
        boss.statuses["Poison"] = 70
        # End turn to trigger poison tick during enemy turns
        engine.execute_action(EndTurn())
        # Boss should have split, not just died
        assert boss.hp == 0, "Boss should be dead from poison"
        assert len(engine.state.enemies) > 1, (
            "Poison kill must trigger split — children should have been spawned"
        )
        children = [e for e in engine.state.enemies if e is not boss]
        assert any(e.hp > 0 for e in children), "At least one child should be alive"


class TestSummonerDeathKillsMinions:
    """Verify that when a summoner dies, all its minions die too (Java parity)."""

    def _make_reptomancer_combat(self, repto_hp=180, dagger_hp=20):
        """Create a combat: Reptomancer flanked by two SnakeDaggers."""
        ai_rng = Random(42)
        hp_rng = Random(42)
        dagger1 = SnakeDagger(ai_rng=Random(43), ascension=0, hp_rng=Random(43))
        repto = Reptomancer(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)
        dagger2 = SnakeDagger(ai_rng=Random(44), ascension=0, hp_rng=Random(44))
        engine = create_combat_from_enemies(
            enemies=[dagger1, repto, dagger2],
            player_hp=80,
            player_max_hp=80,
            deck=["Strike_P"] * 5 + ["Defend_P"] * 5,
            energy=99,
        )
        engine.start_combat()
        # Override HPs for precise testing
        engine.state.enemies[0].hp = dagger_hp  # dagger1
        engine.state.enemies[1].hp = repto_hp   # reptomancer
        engine.state.enemies[2].hp = dagger_hp  # dagger2
        return engine

    def test_reptomancer_death_kills_daggers(self):
        """When Reptomancer dies, all SnakeDaggers should die too."""
        engine = self._make_reptomancer_combat(repto_hp=10, dagger_hp=20)
        repto = engine.state.enemies[1]
        dagger1 = engine.state.enemies[0]
        dagger2 = engine.state.enemies[2]

        # Daggers alive before
        assert dagger1.hp > 0
        assert dagger2.hp > 0

        # Kill Reptomancer
        engine._deal_damage_to_enemy(repto, 9999)

        # All daggers should be dead
        assert repto.hp <= 0, "Reptomancer should be dead"
        assert dagger1.hp <= 0, "Dagger 1 should die when Reptomancer dies"
        assert dagger2.hp <= 0, "Dagger 2 should die when Reptomancer dies"

    def test_dagger_death_does_not_kill_reptomancer(self):
        """Killing a dagger should NOT kill Reptomancer (only summoner->minion)."""
        engine = self._make_reptomancer_combat(repto_hp=180, dagger_hp=10)
        repto = engine.state.enemies[1]
        dagger1 = engine.state.enemies[0]

        engine._deal_damage_to_enemy(dagger1, 9999)

        assert dagger1.hp <= 0, "Dagger should be dead"
        assert repto.hp > 0, "Reptomancer should still be alive"

    def test_reptomancer_death_already_dead_daggers(self):
        """If daggers are already dead, Reptomancer death should not error."""
        engine = self._make_reptomancer_combat(repto_hp=10, dagger_hp=1)
        dagger1 = engine.state.enemies[0]
        dagger2 = engine.state.enemies[2]
        repto = engine.state.enemies[1]

        # Kill daggers first
        engine._deal_damage_to_enemy(dagger1, 9999)
        engine._deal_damage_to_enemy(dagger2, 9999)
        assert dagger1.hp <= 0
        assert dagger2.hp <= 0

        # Now kill Reptomancer — should not crash
        engine._deal_damage_to_enemy(repto, 9999)
        assert repto.hp <= 0

    def test_reptomancer_death_ends_combat(self):
        """Combat should end after Reptomancer + all daggers die."""
        engine = self._make_reptomancer_combat(repto_hp=10, dagger_hp=20)

        repto = engine.state.enemies[1]
        engine._deal_damage_to_enemy(repto, 9999)

        # All enemies should be dead
        assert all(e.hp <= 0 for e in engine.state.enemies)
        # Safety net should detect and end combat
        actions = engine.get_legal_actions()
        assert actions == []
        assert engine.state.combat_over
        assert engine.state.player_won

    def _make_collector_combat(self, collector_hp=282, torch_hp=38):
        """Create a combat: TheCollector flanked by two TorchHeads."""
        ai_rng = Random(42)
        hp_rng = Random(42)
        torch1 = TorchHead(ai_rng=Random(43), ascension=0, hp_rng=Random(43))
        collector = TheCollector(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)
        torch2 = TorchHead(ai_rng=Random(44), ascension=0, hp_rng=Random(44))
        engine = create_combat_from_enemies(
            enemies=[torch1, collector, torch2],
            player_hp=80,
            player_max_hp=80,
            deck=["Strike_P"] * 5 + ["Defend_P"] * 5,
            energy=99,
        )
        engine.start_combat()
        engine.state.enemies[0].hp = torch_hp   # torch1
        engine.state.enemies[1].hp = collector_hp  # collector
        engine.state.enemies[2].hp = torch_hp   # torch2
        return engine

    def test_collector_death_kills_torchheads(self):
        """When TheCollector dies, all TorchHeads should die too."""
        engine = self._make_collector_combat(collector_hp=10, torch_hp=38)
        collector = engine.state.enemies[1]
        torch1 = engine.state.enemies[0]
        torch2 = engine.state.enemies[2]

        assert torch1.hp > 0
        assert torch2.hp > 0

        engine._deal_damage_to_enemy(collector, 9999)

        assert collector.hp <= 0, "Collector should be dead"
        assert torch1.hp <= 0, "TorchHead 1 should die when Collector dies"
        assert torch2.hp <= 0, "TorchHead 2 should die when Collector dies"

    def test_collector_death_ends_combat(self):
        """Combat should end after Collector + all TorchHeads die."""
        engine = self._make_collector_combat(collector_hp=10, torch_hp=38)

        collector = engine.state.enemies[1]
        engine._deal_damage_to_enemy(collector, 9999)

        assert all(e.hp <= 0 for e in engine.state.enemies)
        actions = engine.get_legal_actions()
        assert actions == []
        assert engine.state.combat_over
        assert engine.state.player_won


class TestDarklingRespawn:
    """Verify Darkling half_dead / Regrow mechanic."""

    def _make_darkling_combat(self, hp=50, player_hp=80):
        """Create a combat with 3 Darklings."""
        darklings = []
        for i in range(3):
            ai_rng = Random(42 + i)
            hp_rng = Random(100 + i)
            d = Darkling(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng, position=i)
            darklings.append(d)
        engine = create_combat_from_enemies(
            enemies=darklings,
            player_hp=player_hp,
            player_max_hp=player_hp,
            deck=["Strike_P"] * 5 + ["Defend_P"] * 5,
            energy=99,
        )
        engine.start_combat()
        # Override HP for precise testing
        for e in engine.state.enemies:
            e.hp = hp
            e.max_hp = hp
        return engine

    def test_single_darkling_death_enters_half_dead(self):
        """When one Darkling dies but others live, it enters half_dead."""
        engine = self._make_darkling_combat(hp=50)
        d0 = engine.state.enemies[0]
        d1 = engine.state.enemies[1]
        d2 = engine.state.enemies[2]

        # Kill only the first Darkling
        engine._deal_damage_to_enemy(d0, 9999)

        assert d0.hp <= 0, "Darkling 0 should have 0 HP"
        assert d0.half_dead, "Darkling 0 should be half_dead"
        assert d1.hp > 0, "Darkling 1 should still be alive"
        assert d2.hp > 0, "Darkling 2 should still be alive"
        assert not engine.state.combat_over, "Combat should NOT end with half_dead Darklings"

    def test_half_dead_darkling_not_targetable(self):
        """Half_dead Darklings should not appear in legal action targets."""
        engine = self._make_darkling_combat(hp=50)
        d0 = engine.state.enemies[0]

        engine._deal_damage_to_enemy(d0, 9999)
        assert d0.half_dead

        actions = engine.get_legal_actions()
        # No action should target enemy index 0 (the half_dead one)
        for action in actions:
            if hasattr(action, 'target_index'):
                assert action.target_index != 0, "Half_dead Darkling should not be targetable"

    def test_half_dead_darkling_revives_on_enemy_turn(self):
        """Half_dead Darkling revives with 50% HP when enemy turns execute."""
        engine = self._make_darkling_combat(hp=50)
        d0 = engine.state.enemies[0]

        engine._deal_damage_to_enemy(d0, 9999)
        assert d0.half_dead
        assert d0.hp <= 0

        # End turn triggers enemy turns, which should revive the half_dead darkling
        engine.execute_action(EndTurn())

        assert d0.hp == 25, f"Darkling should revive with 50% of {d0.max_hp} HP, got {d0.hp}"
        assert not d0.half_dead, "Darkling should no longer be half_dead after reviving"

    def test_half_dead_darkling_debuffs_cleared_on_revive(self):
        """Debuffs should be cleared when a Darkling revives."""
        engine = self._make_darkling_combat(hp=50)
        d0 = engine.state.enemies[0]

        # Apply debuffs before killing
        d0.statuses["Weakened"] = 2
        d0.statuses["Vulnerable"] = 2
        d0.statuses["Poison"] = 5

        engine._deal_damage_to_enemy(d0, 9999)
        assert d0.half_dead

        # End turn to trigger revive
        engine.execute_action(EndTurn())

        assert d0.hp > 0, "Darkling should have revived"
        assert d0.statuses.get("Weakened", 0) == 0, "Weakened should be cleared on revive"
        assert d0.statuses.get("Vulnerable", 0) == 0, "Vulnerable should be cleared on revive"
        assert d0.statuses.get("Poison", 0) == 0, "Poison should be cleared on revive"

    def test_all_darklings_die_simultaneously_true_death(self):
        """When ALL Darklings die at once, they all truly die (no half_dead)."""
        engine = self._make_darkling_combat(hp=10)
        d0 = engine.state.enemies[0]
        d1 = engine.state.enemies[1]
        d2 = engine.state.enemies[2]

        # Kill all three
        engine._deal_damage_to_enemy(d0, 9999)
        assert d0.half_dead, "First kill should be half_dead (others still alive)"

        engine._deal_damage_to_enemy(d1, 9999)
        assert d1.half_dead, "Second kill should be half_dead (one still alive)"

        engine._deal_damage_to_enemy(d2, 9999)
        # When the last one dies, ALL should truly die
        assert not d0.half_dead, "All Darklings dead — d0 should no longer be half_dead"
        assert not d1.half_dead, "All Darklings dead — d1 should no longer be half_dead"
        assert not d2.half_dead, "All Darklings dead — d2 should no longer be half_dead"

    def test_all_darklings_die_combat_ends(self):
        """Combat ends when all Darklings truly die."""
        engine = self._make_darkling_combat(hp=10)

        for e in engine.state.enemies:
            engine._deal_damage_to_enemy(e, 9999)

        # Safety net should detect all truly dead and end combat
        actions = engine.get_legal_actions()
        assert actions == []
        assert engine.state.combat_over
        assert engine.state.player_won

    def test_combat_does_not_end_with_half_dead(self):
        """Combat should NOT end when some Darklings are half_dead but not all dead."""
        engine = self._make_darkling_combat(hp=50)

        # Kill two, leave one alive
        engine._deal_damage_to_enemy(engine.state.enemies[0], 9999)
        engine._deal_damage_to_enemy(engine.state.enemies[1], 9999)

        assert engine.state.enemies[0].half_dead
        assert engine.state.enemies[1].half_dead
        assert engine.state.enemies[2].hp > 0

        assert not engine.state.combat_over, "Combat should continue"
        actions = engine.get_legal_actions()
        assert len(actions) > 0, "Should still have actions (EndTurn at minimum)"

    def test_revived_darkling_takes_turn_after_revive(self):
        """After reviving, Darkling should take normal turns on subsequent rounds."""
        engine = self._make_darkling_combat(hp=50)
        d0 = engine.state.enemies[0]

        engine._deal_damage_to_enemy(d0, 9999)
        assert d0.half_dead

        # First end turn: Darkling revives
        engine.execute_action(EndTurn())
        assert d0.hp > 0
        assert not d0.half_dead

        # Second end turn: Darkling should execute a move (attacking or blocking)
        player_hp_before = engine.state.player.hp
        engine.execute_action(EndTurn())
        # Darkling should still be alive and participating
        assert d0.hp > 0, "Revived Darkling should still be alive"
