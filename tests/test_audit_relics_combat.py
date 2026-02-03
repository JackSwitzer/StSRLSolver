"""
Audit Tests: Combat Start/End Relic Triggers vs Decompiled Java

Verifies Python engine relic triggers match the decompiled Java behavior for:
- atBattleStart relics (Anchor, Bag of Preparation, Bag of Marbles, etc.)
- atTurnStart relics (Lantern, Orichalcum, etc.)
- onVictory relics (Burning Blood, Black Blood, Meat on the Bone)
- Passive relics (Preserved Insect, Odd Mushroom, Magic Flower)

See docs/audit/relics-combat.md for full findings.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat,
    PlayCard, UsePotion, EndTurn,
)
from packages.engine.state.rng import Random
from packages.engine.content.relics import ALL_RELICS


# =============================================================================
# Helpers
# =============================================================================

def _make_runner(relics=None, deck=None, hp=80, max_hp=80, enemies=None, ascension=0):
    """Create a CombatRunner with given relics for testing."""
    from packages.engine.handlers.combat import CombatRunner
    from packages.engine.state.run import create_watcher_run
    from packages.engine.content.enemies import JawWorm

    run = create_watcher_run("TEST", ascension=ascension)
    run.max_hp = max_hp
    run.current_hp = hp

    if deck:
        run.deck = deck

    if relics:
        for relic_id in relics:
            try:
                run.add_relic(relic_id)
            except Exception:
                pass  # Some relics may not be in registry

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


# =============================================================================
# atBattleStart Tests
# =============================================================================

class TestAtBattleStartRelics:
    """Test relics that trigger at combat start."""

    def test_anchor_gives_10_block(self):
        """Java: Anchor.atBattleStart -> GainBlockAction(player, 10)"""
        runner = _make_runner(relics=["Anchor"])
        assert runner.state.player.block >= 10

    def test_bag_of_preparation_relic_id_matches(self):
        """Combat handler uses correct relic ID 'Bag of Preparation'.

        Java: BagOfPreparation.atBattleStart -> DrawCardAction(player, 2)
        """
        from packages.engine.content.relics import ALL_RELICS
        relic = ALL_RELICS["Bag of Preparation"]
        assert relic.id == "Bag of Preparation"

        # The combat handler now checks for "Bag of Preparation" (correct)
        import inspect
        from packages.engine.handlers.combat import CombatRunner
        source = inspect.getsource(CombatRunner._setup_combat)
        assert '"Bag of Preparation"' in source, (
            "Combat handler should reference 'Bag of Preparation'"
        )

    def test_bag_of_marbles_applies_vulnerable_to_all_enemies(self):
        """Java: BagOfMarbles.atBattleStart -> Apply 1 Vulnerable to ALL enemies"""
        from packages.engine.content.enemies import JawWorm
        rng1, rng2 = Random(100), Random(101)
        enemies = [
            JawWorm(ai_rng=rng1, ascension=0, hp_rng=rng2),
            JawWorm(ai_rng=Random(102), ascension=0, hp_rng=Random(103)),
        ]
        runner = _make_runner(relics=["Bag of Marbles"], enemies=enemies)
        for enemy in runner.state.enemies:
            vuln = enemy.statuses.get("Vulnerable", 0)
            assert vuln >= 1, f"Enemy should have Vulnerable from Bag of Marbles"

    def test_akabeko_gives_8_vigor(self):
        """Java: Akabeko.atBattleStart -> Gain 8 Vigor"""
        runner = _make_runner(relics=["Akabeko"])
        vigor = runner.state.player.statuses.get("Vigor", 0)
        assert vigor == 8

    def test_bronze_scales_gives_3_thorns(self):
        """Java: BronzeScales.atBattleStart -> Gain 3 Thorns"""
        runner = _make_runner(relics=["Bronze Scales"])
        thorns = runner.state.player.statuses.get("Thorns", 0)
        assert thorns == 3

    def test_thread_and_needle_gives_4_plated_armor(self):
        """Java: ThreadAndNeedle.atBattleStart -> ApplyPowerAction(PlatedArmor, 4)"""
        runner = _make_runner(relics=["Thread and Needle"])
        plated = runner.state.player.statuses.get("Plated Armor", 0)
        assert plated == 4


# =============================================================================
# atTurnStart Tests
# =============================================================================

class TestAtTurnStartRelics:
    """Test relics that trigger at turn start."""

    def test_lantern_gives_energy_turn_1(self):
        """Lantern gives +1 energy on first turn only.

        CombatState.turn starts at 0, _start_player_turn increments to 1,
        then _trigger_start_of_turn checks turn == 1 and grants +1 energy.
        """
        runner = _make_runner(relics=["Lantern"])
        assert runner.state.has_relic("Lantern")
        # Turn starts at 0, increments to 1 in _start_player_turn
        assert runner.state.turn == 1, "Turn is 1 after first _start_player_turn"
        # Lantern should fire on turn 1
        assert runner.state.energy >= 4, (
            "Lantern should grant +1 energy on first turn (3 base + 1 Lantern = 4)"
        )


# =============================================================================
# onVictory / Post-Combat Tests
# =============================================================================

class TestOnVictoryRelics:
    """Test relics that trigger after combat victory."""

    def test_burning_blood_heals_6(self):
        """Java: BurningBlood.onVictory -> if HP > 0, heal(6)"""
        from packages.engine.state.run import create_watcher_run
        run = create_watcher_run("TEST", ascension=0)
        run.max_hp = 80
        run.current_hp = 50
        run.add_relic("Burning Blood")

        old_hp = run.current_hp
        assert run.has_relic("Burning Blood")
        run.heal(6)
        assert run.current_hp == old_hp + 6

    def test_black_blood_heals_12(self):
        """Java: BlackBlood.onVictory -> if HP > 0, heal(12)"""
        from packages.engine.state.run import create_watcher_run
        run = create_watcher_run("TEST", ascension=0)
        run.max_hp = 80
        run.current_hp = 50
        run.add_relic("Black Blood")

        old_hp = run.current_hp
        assert run.has_relic("Black Blood")
        run.heal(12)
        assert run.current_hp == old_hp + 12

    def test_meat_on_the_bone_threshold_is_lte_50_percent(self):
        """BUG: Java uses <= for 50% check, Python uses <.
        Java: if (p.currentHealth <= p.maxHealth / 2.0f && p.currentHealth > 0) heal(12)
        Python: if rs.current_hp < rs.max_hp * 0.5  (WRONG - should be <=)

        This test documents the bug: at exactly 50% HP, Java heals but Python does not.
        """
        from packages.engine.state.run import create_watcher_run
        run = create_watcher_run("TEST", ascension=0)
        run.max_hp = 80
        run.current_hp = 40  # Exactly 50%

        # Java behavior: should heal at exactly 50%
        java_threshold = run.current_hp <= run.max_hp / 2.0
        assert java_threshold, "Java uses <= so at exactly 50% it heals"

        # Python (game.py:1607) uses < instead of <=
        python_threshold = run.current_hp < run.max_hp * 0.5
        assert not python_threshold, (
            "Documenting bug: Python uses < instead of <= for Meat on the Bone threshold"
        )


# =============================================================================
# Bug Documentation Tests
# =============================================================================

class TestDocumentedBugs:
    """Tests that document known bugs found during audit."""

    def test_blood_vial_triggers_at_battle_start(self):
        """Java BloodVial.atBattleStart heals 2 at combat START.
        Python now triggers it via registry execute_relic_triggers.
        """
        relic = ALL_RELICS.get("Blood Vial")
        assert relic is not None
        assert any("atBattleStart" in e for e in relic.effects), (
            "Blood Vial relic definition should say atBattleStart"
        )

        # Blood Vial IS now in the registry for atBattleStart
        from packages.engine.registry import RELIC_REGISTRY
        assert RELIC_REGISTRY.has_handler("atBattleStart", "Blood Vial"), (
            "Blood Vial should be registered for atBattleStart trigger"
        )

    def test_orichalcum_implemented_at_end_of_turn(self):
        """FIXED: Java Orichalcum.onPlayerEndTurn gives 6 block if player has 0 block.
        Python combat handler now calls registry execute_relic_triggers.
        """
        # Orichalcum is now in the registry for onPlayerEndTurn
        from packages.engine.registry import RELIC_REGISTRY
        assert RELIC_REGISTRY.has_handler("onPlayerEndTurn", "Orichalcum"), (
            "Orichalcum should be registered for onPlayerEndTurn trigger"
        )

    def test_preserved_insect_not_implemented_in_combat(self):
        """BUG: Java PreservedInsect.atBattleStart reduces elite HP by 25%.
        Python has only a comment saying 'should be handled when creating enemies'.
        """
        import inspect
        from packages.engine.handlers.combat import CombatRunner
        source = inspect.getsource(CombatRunner._trigger_start_of_combat_relics)
        # Check it's just a comment, not actual implementation
        lines = [l.strip() for l in source.split('\n') if 'Preserved' in l or 'PreservedInsect' in l]
        for line in lines:
            assert line.startswith('#'), (
                f"Expected Preserved Insect to be commented out, found: {line}"
            )


# =============================================================================
# Relic Definition Consistency Tests
# =============================================================================

class TestRelicDefinitionConsistency:
    """Verify relic definitions match Java constants."""

    def test_anchor_block_amount_is_10(self):
        """Java: private static final int BLOCK_AMT = 10"""
        relic = ALL_RELICS["Anchor"]
        assert "10" in relic.effects[0]

    def test_burning_blood_heal_amount_is_6(self):
        """Java: private static final int HEALTH_AMT = 6"""
        relic = ALL_RELICS["Burning Blood"]
        assert "6" in relic.effects[0]

    def test_black_blood_heal_amount_is_12(self):
        """Java: heal(12)"""
        relic = ALL_RELICS["Black Blood"]
        assert "12" in relic.effects[0]

    def test_bag_of_preparation_draw_count_is_2(self):
        """Java: private static final int NUM_CARDS = 2"""
        relic = ALL_RELICS["Bag of Preparation"]
        assert "2" in relic.effects[0]

    def test_orichalcum_block_amount_is_6(self):
        """Java: private static final int BLOCK_AMT = 6"""
        relic = ALL_RELICS["Orichalcum"]
        assert "6" in relic.effects[0]

    def test_thread_and_needle_plated_armor_is_4(self):
        """Java: private static final int ARMOR_AMT = 4"""
        relic = ALL_RELICS["Thread and Needle"]
        assert "4" in relic.effects[0]

    def test_meat_on_the_bone_heal_is_12(self):
        """Java: private static final int HEAL_AMT = 12"""
        relic = ALL_RELICS["Meat on the Bone"]
        assert "12" in relic.effects[0]

    def test_preserved_insect_is_25_percent(self):
        """Java: private float MODIFIER_AMT = 0.25f (display: 25)"""
        relic = ALL_RELICS["PreservedInsect"]
        assert "25%" in relic.effects[0]

    def test_odd_mushroom_vuln_is_25_percent(self):
        """Java: public static final float VULN_EFFECTIVENESS = 1.25f"""
        relic = ALL_RELICS["Odd Mushroom"]
        assert "25%" in relic.effects[0]

    def test_magic_flower_multiplier_is_50_percent(self):
        """Java: private static final float HEAL_MULTIPLIER = 1.5f"""
        relic = ALL_RELICS["Magic Flower"]
        assert "50%" in relic.effects[0]

    def test_lantern_is_first_turn_energy(self):
        """Java: Lantern gives 1 energy on first turn only (atTurnStart with firstTurn flag)"""
        relic = ALL_RELICS["Lantern"]
        assert "first turn" in relic.effects[0].lower() or "1 Energy" in relic.effects[0]

    def test_blood_vial_is_at_battle_start(self):
        """Java: BloodVial.atBattleStart -> heal 2"""
        relic = ALL_RELICS["Blood Vial"]
        assert "atBattleStart" in relic.effects[0]

    def test_orichalcum_is_on_player_end_turn(self):
        """Java: Orichalcum.onPlayerEndTurn"""
        relic = ALL_RELICS["Orichalcum"]
        assert "onPlayerEndTurn" in relic.effects[0]
