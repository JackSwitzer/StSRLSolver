"""
Test Sacred Bark and new potion implementations.
"""
import pytest

from packages.engine.combat_engine import CombatEngine
from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState


class TestSacredBarkDoubling:
    """Test that Sacred Bark correctly doubles potion potency."""

    def test_block_potion_doubled(self):
        """Block Potion should give 24 block with Sacred Bark (12 * 2)."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["Block Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.block == 24

    def test_strength_potion_doubled(self):
        """Strength Potion should give 4 strength with Sacred Bark (2 * 2)."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["Strength Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.statuses.get("Strength", 0) == 4

    def test_fire_potion_doubled(self):
        """Fire Potion should deal 40 damage with Sacred Bark (20 * 2)."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["Fire Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0, target_index=0)
        assert result["success"]
        assert state.enemies[0].hp == 10  # 50 - 40 = 10

    def test_energy_potion_doubled(self):
        """Energy Potion should give 4 energy with Sacred Bark (2 * 2)."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["Energy Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.energy == 7  # 3 + 4 = 7

    def test_swift_potion_doubled(self):
        """Swift Potion should draw 6 cards with Sacred Bark (3 * 2)."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            hand=[],
            draw_pile=["Strike"] * 10 + ["Defend"] * 10,  # Enough cards for initial draw + potion
            potions=["Swift Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        # start_combat draws 5 cards, so we should have 5 cards
        assert len(state.hand) == 5
        initial_hand = len(state.hand)

        result = engine.use_potion(0)
        assert result["success"]
        assert len(state.hand) == initial_hand + 6  # Should draw 6 more


class TestFairyInBottle:
    """Test Fairy in a Bottle auto-revive mechanics."""

    def test_fairy_revives_at_30_percent(self):
        """Fairy in a Bottle should revive at 30% max HP without Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=80, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["FairyPotion"],
            relics=[],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        state.player.hp = 0
        fairy_triggered = engine._check_fairy_in_bottle()

        assert fairy_triggered
        assert state.player.hp == 24  # 30% of 80 = 24
        assert state.potions[0] == ""  # Potion consumed

    def test_fairy_revives_at_60_percent_with_sacred_bark(self):
        """Fairy in a Bottle should revive at 60% max HP with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=80, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["FairyPotion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        state.player.hp = 0
        fairy_triggered = engine._check_fairy_in_bottle()

        assert fairy_triggered
        assert state.player.hp == 48  # 60% of 80 = 48
        assert state.potions[0] == ""  # Potion consumed

    def test_fairy_not_triggered_when_alive(self):
        """Fairy should not trigger when player is alive."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=0),
            energy=3,
            max_energy=3,
            potions=["FairyPotion"],
            relics=[],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        fairy_triggered = engine._check_fairy_in_bottle()
        assert not fairy_triggered
        assert state.potions[0] == "FairyPotion"  # Potion still there


class TestNewPotionImplementations:
    """Test newly implemented potions."""

    def test_regen_potion(self):
        """Regen Potion should apply 10 regen with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=50, block=0),
            energy=3,
            max_energy=3,
            potions=["Regen Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.statuses.get("Regeneration", 0) == 10

    def test_ancient_potion(self):
        """Ancient Potion should apply 2 artifact with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=50, block=0),
            energy=3,
            max_energy=3,
            potions=["Ancient Potion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.statuses.get("Artifact", 0) == 2

    def test_essence_of_steel(self):
        """Essence of Steel should apply 8 plated armor with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=50, block=0),
            energy=3,
            max_energy=3,
            potions=["EssenceOfSteel"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.statuses.get("Plated Armor", 0) == 8

    def test_fruit_juice(self):
        """Fruit Juice should increase max HP by 10 with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=50, block=0),
            energy=3,
            max_energy=3,
            potions=["Fruit Juice"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.max_hp == 60
        assert state.player.hp == 60  # Should heal too

    def test_duplication_potion(self):
        """Duplication Potion should apply 2 duplication stacks with Sacred Bark."""
        state = CombatState(
            player=EntityState(hp=50, max_hp=50, block=0),
            energy=3,
            max_energy=3,
            potions=["DuplicationPotion"],
            relics=["Sacred Bark"],
            enemies=[EnemyCombatState(hp=50, max_hp=50, id="Cultist", name="Cultist")]
        )
        engine = CombatEngine(state)
        engine.start_combat()

        result = engine.use_potion(0)
        assert result["success"]
        assert state.player.statuses.get("Duplication", 0) == 2
