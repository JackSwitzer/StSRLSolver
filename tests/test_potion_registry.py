"""
Tests for the potion registry integration.

Verifies that:
1. Potions use the registry system correctly
2. Sacred Bark properly doubles potency
3. Target handling works for damage potions
4. Various potion types apply their effects correctly
"""

import pytest
from packages.engine.registry import execute_potion_effect, POTION_REGISTRY
from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.handlers.combat import CombatRunner
from packages.engine.state.run import create_watcher_run
from packages.engine.content.enemies import JawWorm
from packages.engine.state.rng import Random


class TestPotionRegistry:
    """Test that potions are properly registered."""

    def test_all_potions_registered(self):
        """All 42 potions should have handlers registered."""
        from packages.engine.content.potions import ALL_POTIONS

        registered = set(POTION_REGISTRY.list_entities("onUsePotion"))
        all_ids = set(ALL_POTIONS.keys())

        assert registered == all_ids, f"Missing handlers: {all_ids - registered}"
        assert len(registered) == 42

    def test_potion_metadata_available(self):
        """Potion handlers should have metadata attached."""
        handler = POTION_REGISTRY.get_handler("onUsePotion", "Fire Potion")
        assert handler is not None
        assert hasattr(handler, "_potion_metadata")
        assert handler._potion_metadata["requires_target"] is True


class TestFirePotion:
    """Test Fire Potion (targeted damage potion)."""

    def _create_combat_state(self, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=["Fire Potion", "", ""],
        )

    def test_fire_potion_deals_20_damage(self):
        """Fire Potion should deal 20 damage to target."""
        state = self._create_combat_state()
        initial_hp = state.enemies[0].hp

        result = execute_potion_effect("Fire Potion", state, target_idx=0)

        assert result["success"] is True
        assert result["potency"] == 20
        assert state.enemies[0].hp == initial_hp - 20

    def test_fire_potion_with_sacred_bark(self):
        """Fire Potion with Sacred Bark should deal 40 damage."""
        state = self._create_combat_state(relics=["SacredBark"])
        initial_hp = state.enemies[0].hp

        result = execute_potion_effect("Fire Potion", state, target_idx=0)

        assert result["success"] is True
        assert result["potency"] == 40
        assert state.enemies[0].hp == initial_hp - 40

    def test_fire_potion_respects_block(self):
        """Fire Potion damage should be reduced by enemy block."""
        state = self._create_combat_state()
        state.enemies[0].block = 10
        initial_hp = state.enemies[0].hp

        execute_potion_effect("Fire Potion", state, target_idx=0)

        # 20 damage - 10 block = 10 HP damage
        assert state.enemies[0].hp == initial_hp - 10
        assert state.enemies[0].block == 0


class TestBlockPotion:
    """Test Block Potion (self-target block gain)."""

    def _create_combat_state(self, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=["Block Potion", "", ""],
        )

    def test_block_potion_gains_12_block(self):
        """Block Potion should gain 12 block."""
        state = self._create_combat_state()
        assert state.player.block == 0

        result = execute_potion_effect("Block Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 12
        assert state.player.block == 12

    def test_block_potion_with_sacred_bark(self):
        """Block Potion with Sacred Bark should gain 24 block."""
        state = self._create_combat_state(relics=["SacredBark"])

        result = execute_potion_effect("Block Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 24
        assert state.player.block == 24


class TestStrengthPotion:
    """Test Strength Potion (permanent strength)."""

    def _create_combat_state(self, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=["Strength Potion", "", ""],
        )

    def test_strength_potion_gains_2_strength(self):
        """Strength Potion should gain 2 Strength."""
        state = self._create_combat_state()
        assert state.player.statuses.get("Strength", 0) == 0

        result = execute_potion_effect("Strength Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 2
        assert state.player.statuses["Strength"] == 2

    def test_strength_potion_with_sacred_bark(self):
        """Strength Potion with Sacred Bark should gain 4 Strength."""
        state = self._create_combat_state(relics=["SacredBark"])

        result = execute_potion_effect("Strength Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 4
        assert state.player.statuses["Strength"] == 4

    def test_strength_stacks(self):
        """Multiple Strength Potions should stack."""
        state = self._create_combat_state()
        state.player.statuses["Strength"] = 3

        execute_potion_effect("Strength Potion", state, target_idx=-1)

        assert state.player.statuses["Strength"] == 5


class TestWeakPotion:
    """Test Weak Potion (targeted debuff)."""

    def _create_combat_state(self, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=["Weak Potion", "", ""],
        )

    def test_weak_potion_applies_3_weak(self):
        """Weak Potion should apply 3 Weak to target."""
        state = self._create_combat_state()

        result = execute_potion_effect("Weak Potion", state, target_idx=0)

        assert result["success"] is True
        assert result["potency"] == 3
        # The registry uses "Weakened" instead of "Weak"
        assert state.enemies[0].statuses.get("Weakened", 0) == 3

    def test_weak_potion_with_sacred_bark(self):
        """Weak Potion with Sacred Bark should apply 6 Weak."""
        state = self._create_combat_state(relics=["SacredBark"])

        result = execute_potion_effect("Weak Potion", state, target_idx=0)

        assert result["success"] is True
        assert result["potency"] == 6
        assert state.enemies[0].statuses.get("Weakened", 0) == 6


class TestEnergyPotion:
    """Test Energy Potion (energy gain)."""

    def _create_combat_state(self, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=["Energy Potion", "", ""],
        )

    def test_energy_potion_gains_2_energy(self):
        """Energy Potion should gain 2 energy."""
        state = self._create_combat_state()
        assert state.energy == 3

        result = execute_potion_effect("Energy Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 2
        assert state.energy == 5

    def test_energy_potion_with_sacred_bark(self):
        """Energy Potion with Sacred Bark should gain 4 energy."""
        state = self._create_combat_state(relics=["SacredBark"])
        assert state.energy == 3

        result = execute_potion_effect("Energy Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 4
        assert state.energy == 7


class TestCombatRunnerPotionIntegration:
    """Test that CombatRunner properly uses the registry for potions."""

    def test_combat_runner_uses_registry(self):
        """CombatRunner.use_potion should use the registry system."""
        run = create_watcher_run("TEST123", ascension=0)
        run.potion_slots[0].potion_id = "Strength Potion"

        rng = Random(12345)
        enemies = [JawWorm(ai_rng=rng, ascension=0, hp_rng=rng)]

        runner = CombatRunner(
            run_state=run,
            enemies=enemies,
            shuffle_rng=rng,
        )

        initial_strength = runner.state.player.statuses.get("Strength", 0)
        result = runner.use_potion(potion_idx=0, target_idx=-1)

        assert result["success"] is True
        assert runner.state.player.statuses.get("Strength", 0) == initial_strength + 2
        assert runner.state.potions[0] == ""  # Potion consumed

    def test_combat_runner_sacred_bark_doubles_potency(self):
        """CombatRunner with Sacred Bark should double potion potency."""
        run = create_watcher_run("TEST123", ascension=0)
        run.potion_slots[0].potion_id = "Strength Potion"
        run.relics.append(type("Relic", (), {"id": "SacredBark"})())

        rng = Random(12345)
        enemies = [JawWorm(ai_rng=rng, ascension=0, hp_rng=rng)]

        runner = CombatRunner(
            run_state=run,
            enemies=enemies,
            shuffle_rng=rng,
        )

        initial_strength = runner.state.player.statuses.get("Strength", 0)
        result = runner.use_potion(potion_idx=0, target_idx=-1)

        assert result["success"] is True
        assert runner.state.player.statuses.get("Strength", 0) == initial_strength + 4

    def test_combat_runner_damage_potion_to_enemy(self):
        """CombatRunner should correctly apply damage potions to enemies."""
        run = create_watcher_run("TEST123", ascension=0)
        run.potion_slots[0].potion_id = "Fire Potion"

        rng = Random(12345)
        enemies = [JawWorm(ai_rng=rng, ascension=0, hp_rng=rng)]

        runner = CombatRunner(
            run_state=run,
            enemies=enemies,
            shuffle_rng=rng,
        )

        initial_hp = runner.state.enemies[0].hp
        result = runner.use_potion(potion_idx=0, target_idx=0)

        assert result["success"] is True
        assert runner.state.enemies[0].hp == initial_hp - 20


class TestSpecialPotions:
    """Test potions with special mechanics."""

    def _create_combat_state(self, potions, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=potions + [""] * (3 - len(potions)),
        )

    def test_explosive_potion_hits_all_enemies(self):
        """Explosive Potion should deal damage to all enemies."""
        state = create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[
                create_enemy(30, 30, "Enemy1"),
                create_enemy(40, 40, "Enemy2"),
            ],
            deck=["Strike"],
            energy=3,
            max_energy=3,
            relics=[],
            potions=["Explosive Potion", "", ""],
        )

        result = execute_potion_effect("Explosive Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 10
        assert state.enemies[0].hp == 20  # 30 - 10
        assert state.enemies[1].hp == 30  # 40 - 10

    def test_swift_potion_draws_cards(self):
        """Swift Potion should draw 3 cards."""
        state = self._create_combat_state(["Swift Potion"])
        state.draw_pile = ["Card1", "Card2", "Card3", "Card4"]
        state.hand = []

        result = execute_potion_effect("Swift Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 3
        assert len(state.hand) == 3

    def test_dexterity_potion(self):
        """Dexterity Potion should grant Dexterity."""
        state = self._create_combat_state(["Dexterity Potion"])

        result = execute_potion_effect("Dexterity Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 2
        assert state.player.statuses.get("Dexterity", 0) == 2

    def test_ancient_potion_grants_artifact(self):
        """Ancient Potion should grant 1 Artifact."""
        state = self._create_combat_state(["Ancient Potion"])

        result = execute_potion_effect("Ancient Potion", state, target_idx=-1)

        assert result["success"] is True
        assert result["potency"] == 1
        assert state.player.statuses.get("Artifact", 0) == 1

    def test_ambrosia_enters_divinity(self):
        """Ambrosia should enter Divinity stance."""
        state = self._create_combat_state(["Ambrosia"])
        state.stance = "Neutral"

        result = execute_potion_effect("Ambrosia", state, target_idx=-1)

        assert result["success"] is True
        assert state.stance == "Divinity"

    def test_ambrosia_from_calm_gains_energy(self):
        """Ambrosia from Calm should gain energy from exiting Calm."""
        state = self._create_combat_state(["Ambrosia"])
        state.stance = "Calm"
        state.energy = 3

        execute_potion_effect("Ambrosia", state, target_idx=-1)

        # Calm exit: +2, Divinity enter: +3
        assert state.energy == 8
        assert state.stance == "Divinity"


class TestPotionsSacredBarkExempt:
    """Test potions that should NOT be doubled by Sacred Bark."""

    def _create_combat_state(self, potions, relics=None):
        return create_combat(
            player_hp=50,
            player_max_hp=80,
            enemies=[create_enemy(50, 50, "TestEnemy")],
            deck=["Strike", "Defend"],
            energy=3,
            max_energy=3,
            relics=relics or [],
            potions=potions + [""] * (3 - len(potions)),
        )

    def test_blessing_of_forge_not_doubled(self):
        """Blessing of the Forge should upgrade hand, not doubled by Sacred Bark."""
        state = self._create_combat_state(["BlessingOfTheForge"], relics=["SacredBark"])
        state.hand = ["Strike", "Defend"]

        result = execute_potion_effect("BlessingOfTheForge", state, target_idx=-1)

        assert result["success"] is True
        # Cards should be upgraded
        assert "Strike+" in state.hand
        assert "Defend+" in state.hand

    def test_gamblers_brew_not_doubled(self):
        """Gambler's Brew should reshuffle hand, not doubled by Sacred Bark."""
        state = self._create_combat_state(["GamblersBrew"], relics=["SacredBark"])
        state.hand = ["Card1", "Card2"]
        state.draw_pile = ["Card3", "Card4"]

        result = execute_potion_effect("GamblersBrew", state, target_idx=-1)

        assert result["success"] is True
        # Hand should be refreshed
        assert len(state.hand) == 2
