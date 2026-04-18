"""
Potion Audit Tests

Verifies Python engine potion data and effects against decompiled Java values.
Tests cover: base potency, Sacred Bark doubling, rarity, targeting, ID consistency,
and effect implementation coverage.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.potions import (
    Potion, PotionRarity, PotionTargetType, PlayerClass,
    get_potion_by_id, get_potion_pool, calculate_potion_slots,
    calculate_drop_chance,
    ALL_POTIONS, COMMON_POTIONS, UNCOMMON_POTIONS, RARE_POTIONS,
    UNIVERSAL_POTIONS,
    BLOCK_POTION, DEXTERITY_POTION, ENERGY_POTION, EXPLOSIVE_POTION,
    FIRE_POTION, STRENGTH_POTION, SWIFT_POTION, WEAK_POTION,
    FEAR_POTION, ATTACK_POTION, SKILL_POTION, POWER_POTION,
    COLORLESS_POTION, SPEED_POTION, STEROID_POTION,
    BLESSING_OF_THE_FORGE, BLOOD_POTION, POISON_POTION,
    FOCUS_POTION, BOTTLED_MIRACLE,
    ANCIENT_POTION, REGEN_POTION, GAMBLERS_BREW, LIQUID_BRONZE,
    LIQUID_MEMORIES, ESSENCE_OF_STEEL, DUPLICATION_POTION,
    DISTILLED_CHAOS, ELIXIR, CUNNING_POTION, POTION_OF_CAPACITY,
    STANCE_POTION,
    CULTIST_POTION, FRUIT_JUICE, SNECKO_OIL, FAIRY_POTION,
    SMOKE_BOMB, ENTROPIC_BREW, HEART_OF_IRON, GHOST_IN_A_JAR,
    ESSENCE_OF_DARKNESS, AMBROSIA,
    BASE_POTION_DROP_CHANCE, BLIZZARD_MOD_STEP,
    POTION_COMMON_CHANCE, POTION_UNCOMMON_CHANCE, POTION_RARE_CHANCE,
)


# ============================================================================
# Java-verified base potency values
# ============================================================================

# (potion_obj, expected_potency, expected_rarity, expected_sacred_bark_scales)
JAVA_VERIFIED_VALUES = [
    # Common potions
    (BLOCK_POTION, 12, PotionRarity.COMMON, True),
    (DEXTERITY_POTION, 2, PotionRarity.COMMON, True),
    (ENERGY_POTION, 2, PotionRarity.COMMON, True),
    (EXPLOSIVE_POTION, 10, PotionRarity.COMMON, True),
    (FIRE_POTION, 20, PotionRarity.COMMON, True),
    (STRENGTH_POTION, 2, PotionRarity.COMMON, True),
    (SWIFT_POTION, 3, PotionRarity.COMMON, True),
    (WEAK_POTION, 3, PotionRarity.COMMON, True),
    (FEAR_POTION, 3, PotionRarity.COMMON, True),
    (ATTACK_POTION, 1, PotionRarity.COMMON, True),
    (SKILL_POTION, 1, PotionRarity.COMMON, True),
    (POWER_POTION, 1, PotionRarity.COMMON, True),
    (COLORLESS_POTION, 1, PotionRarity.COMMON, True),
    (SPEED_POTION, 5, PotionRarity.COMMON, True),
    (STEROID_POTION, 5, PotionRarity.COMMON, True),
    (BLESSING_OF_THE_FORGE, 0, PotionRarity.COMMON, False),
    (BLOOD_POTION, 20, PotionRarity.COMMON, True),
    (POISON_POTION, 6, PotionRarity.COMMON, True),
    (FOCUS_POTION, 2, PotionRarity.COMMON, True),
    (BOTTLED_MIRACLE, 2, PotionRarity.COMMON, True),
    # Uncommon potions
    (ANCIENT_POTION, 1, PotionRarity.UNCOMMON, True),
    (REGEN_POTION, 5, PotionRarity.UNCOMMON, True),
    (GAMBLERS_BREW, 0, PotionRarity.UNCOMMON, False),
    (LIQUID_BRONZE, 3, PotionRarity.UNCOMMON, True),
    (LIQUID_MEMORIES, 1, PotionRarity.UNCOMMON, True),
    (ESSENCE_OF_STEEL, 4, PotionRarity.UNCOMMON, True),
    (DUPLICATION_POTION, 1, PotionRarity.UNCOMMON, True),
    (DISTILLED_CHAOS, 3, PotionRarity.UNCOMMON, True),
    (ELIXIR, 0, PotionRarity.UNCOMMON, False),
    (CUNNING_POTION, 3, PotionRarity.UNCOMMON, True),
    (POTION_OF_CAPACITY, 2, PotionRarity.UNCOMMON, True),
    (STANCE_POTION, 0, PotionRarity.UNCOMMON, False),
    # Rare potions
    (CULTIST_POTION, 1, PotionRarity.RARE, True),
    (FRUIT_JUICE, 5, PotionRarity.RARE, True),
    (SNECKO_OIL, 5, PotionRarity.RARE, True),
    (FAIRY_POTION, 30, PotionRarity.RARE, True),
    (SMOKE_BOMB, 0, PotionRarity.RARE, False),
    (ENTROPIC_BREW, 3, PotionRarity.RARE, False),
    (HEART_OF_IRON, 6, PotionRarity.RARE, True),
    (GHOST_IN_A_JAR, 1, PotionRarity.RARE, True),
    (ESSENCE_OF_DARKNESS, 1, PotionRarity.RARE, True),
    (AMBROSIA, 2, PotionRarity.RARE, False),
]


class TestJavaParityPotency:
    """Verify all potion base potency values match Java decompiled source."""

    @pytest.mark.parametrize(
        "potion,expected_potency,expected_rarity,expected_bark",
        JAVA_VERIFIED_VALUES,
        ids=[p[0].name for p in JAVA_VERIFIED_VALUES],
    )
    def test_potency_matches_java(self, potion, expected_potency, expected_rarity, expected_bark):
        assert potion.potency == expected_potency, (
            f"{potion.name}: potency {potion.potency} != Java {expected_potency}"
        )

    @pytest.mark.parametrize(
        "potion,expected_potency,expected_rarity,expected_bark",
        JAVA_VERIFIED_VALUES,
        ids=[p[0].name for p in JAVA_VERIFIED_VALUES],
    )
    def test_rarity_matches_java(self, potion, expected_potency, expected_rarity, expected_bark):
        assert potion.rarity == expected_rarity, (
            f"{potion.name}: rarity {potion.rarity} != Java {expected_rarity}"
        )

    @pytest.mark.parametrize(
        "potion,expected_potency,expected_rarity,expected_bark",
        JAVA_VERIFIED_VALUES,
        ids=[p[0].name for p in JAVA_VERIFIED_VALUES],
    )
    def test_sacred_bark_flag_matches_java(self, potion, expected_potency, expected_rarity, expected_bark):
        assert potion.sacred_bark_scales == expected_bark, (
            f"{potion.name}: sacred_bark_scales {potion.sacred_bark_scales} != Java {expected_bark}"
        )


class TestSacredBarkDoubling:
    """Verify Sacred Bark doubles potency correctly for all scaling potions."""

    @pytest.mark.parametrize(
        "potion,expected_potency,expected_rarity,expected_bark",
        [v for v in JAVA_VERIFIED_VALUES if v[3]],  # Only scaling potions
        ids=[p[0].name for p in JAVA_VERIFIED_VALUES if p[3]],
    )
    def test_sacred_bark_doubles(self, potion, expected_potency, expected_rarity, expected_bark):
        doubled = potion.get_effective_potency(has_sacred_bark=True)
        assert doubled == expected_potency * 2, (
            f"{potion.name}: Sacred Bark potency {doubled} != {expected_potency * 2}"
        )

    @pytest.mark.parametrize(
        "potion,expected_potency,expected_rarity,expected_bark",
        [v for v in JAVA_VERIFIED_VALUES if not v[3]],  # Non-scaling potions
        ids=[p[0].name for p in JAVA_VERIFIED_VALUES if not p[3]],
    )
    def test_sacred_bark_no_effect(self, potion, expected_potency, expected_rarity, expected_bark):
        assert potion.get_effective_potency(has_sacred_bark=True) == expected_potency, (
            f"{potion.name}: should NOT scale with Sacred Bark"
        )


class TestPotionCounts:
    """Verify potion counts match Java PotionHelper."""

    def test_total_potion_count(self):
        """Java has 42 potions total."""
        assert len(ALL_POTIONS) == 42

    def test_common_count(self):
        """Java: 20 common potions (16 universal + 4 class)."""
        assert len(COMMON_POTIONS) == 20

    def test_uncommon_count(self):
        """Java: 12 uncommon potions (8 universal + 4 class)."""
        assert len(UNCOMMON_POTIONS) == 12

    def test_rare_count(self):
        """Java: 10 rare potions (6 universal + 4 class)."""
        assert len(RARE_POTIONS) == 10

    def test_universal_count(self):
        """Java: 30 universal potions."""
        assert len(UNIVERSAL_POTIONS) == 30

    def test_class_specific_count(self):
        """Each class has exactly 3 class-specific potions."""
        for cls in [PlayerClass.IRONCLAD, PlayerClass.SILENT, PlayerClass.DEFECT, PlayerClass.WATCHER]:
            class_potions = [p for p in ALL_POTIONS.values() if p.player_class == cls]
            assert len(class_potions) == 3, f"{cls.value} has {len(class_potions)} potions, expected 3"

    def test_pool_size_per_class(self):
        """Each class pool = 30 universal + 3 class = 33."""
        for cls in [PlayerClass.IRONCLAD, PlayerClass.SILENT, PlayerClass.DEFECT, PlayerClass.WATCHER]:
            pool = get_potion_pool(cls)
            assert len(pool) == 33, f"{cls.value} pool size {len(pool)} != 33"


class TestPotionTargeting:
    """Verify targeting types match Java implementations."""

    def test_fire_potion_targets_enemy(self):
        assert FIRE_POTION.target_type == PotionTargetType.ENEMY

    def test_explosive_targets_all(self):
        assert EXPLOSIVE_POTION.target_type == PotionTargetType.ALL_ENEMIES

    def test_fairy_no_target(self):
        """FairyPotion.canUse() returns false -- no manual targeting."""
        assert FAIRY_POTION.target_type == PotionTargetType.NONE

    def test_block_targets_self(self):
        assert BLOCK_POTION.target_type == PotionTargetType.SELF

    def test_weak_targets_enemy(self):
        assert WEAK_POTION.target_type == PotionTargetType.ENEMY

    def test_fear_targets_enemy(self):
        assert FEAR_POTION.target_type == PotionTargetType.ENEMY

    def test_poison_targets_enemy(self):
        assert POISON_POTION.target_type == PotionTargetType.ENEMY


class TestPotionIDs:
    """Verify potion IDs match Java internal IDs."""

    # Java ID -> Python ID mapping (verified from class constructors)
    JAVA_IDS = {
        "Block Potion": "Block Potion",
        "Dexterity Potion": "Dexterity Potion",
        "Energy Potion": "Energy Potion",
        "Explosive Potion": "Explosive Potion",
        "Fire Potion": "Fire Potion",
        "Strength Potion": "Strength Potion",
        "Swift Potion": "Swift Potion",
        "Weak Potion": "Weak Potion",
        "FearPotion": "FearPotion",
        "AttackPotion": "AttackPotion",
        "SkillPotion": "SkillPotion",
        "PowerPotion": "PowerPotion",
        "ColorlessPotion": "ColorlessPotion",
        "SpeedPotion": "SpeedPotion",
        "SteroidPotion": "SteroidPotion",
        "BlessingOfTheForge": "BlessingOfTheForge",
        "BloodPotion": "BloodPotion",
        "Poison Potion": "Poison Potion",
        "FocusPotion": "FocusPotion",
        "BottledMiracle": "BottledMiracle",
        "Ancient Potion": "Ancient Potion",
        "Regen Potion": "Regen Potion",
        "GamblersBrew": "GamblersBrew",
        "LiquidBronze": "LiquidBronze",
        "LiquidMemories": "LiquidMemories",
        "EssenceOfSteel": "EssenceOfSteel",
        "DuplicationPotion": "DuplicationPotion",
        "DistilledChaos": "DistilledChaos",
        "ElixirPotion": "ElixirPotion",
        "CunningPotion": "CunningPotion",
        "PotionOfCapacity": "PotionOfCapacity",
        "StancePotion": "StancePotion",
        "CultistPotion": "CultistPotion",
        "Fruit Juice": "Fruit Juice",
        "SneckoOil": "SneckoOil",
        "FairyPotion": "FairyPotion",
        "SmokeBomb": "SmokeBomb",
        "EntropicBrew": "EntropicBrew",
        "HeartOfIron": "HeartOfIron",
        "GhostInAJar": "GhostInAJar",
        "EssenceOfDarkness": "EssenceOfDarkness",
        "Ambrosia": "Ambrosia",
    }

    def test_all_java_ids_exist(self):
        """Every Java potion ID maps to a Python potion."""
        for java_id, python_id in self.JAVA_IDS.items():
            potion = get_potion_by_id(python_id)
            assert potion is not None, f"Java ID '{java_id}' -> Python ID '{python_id}' not found"

    def test_no_extra_python_potions(self):
        """Python has no potions that don't exist in Java."""
        java_ids = set(self.JAVA_IDS.values())
        for python_id in ALL_POTIONS:
            assert python_id in java_ids, f"Python potion '{python_id}' has no Java counterpart"


class TestClassAssignment:
    """Verify class-specific potion assignments match Java."""

    def test_ironclad_potions(self):
        expected = {"BloodPotion", "ElixirPotion", "HeartOfIron"}
        actual = {p.id for p in ALL_POTIONS.values() if p.player_class == PlayerClass.IRONCLAD}
        assert actual == expected

    def test_silent_potions(self):
        expected = {"Poison Potion", "CunningPotion", "GhostInAJar"}
        actual = {p.id for p in ALL_POTIONS.values() if p.player_class == PlayerClass.SILENT}
        assert actual == expected

    def test_defect_potions(self):
        expected = {"FocusPotion", "PotionOfCapacity", "EssenceOfDarkness"}
        actual = {p.id for p in ALL_POTIONS.values() if p.player_class == PlayerClass.DEFECT}
        assert actual == expected

    def test_watcher_potions(self):
        expected = {"BottledMiracle", "StancePotion", "Ambrosia"}
        actual = {p.id for p in ALL_POTIONS.values() if p.player_class == PlayerClass.WATCHER}
        assert actual == expected


class TestThrownPotions:
    """Verify is_thrown flag matches Java isThrown field."""

    JAVA_THROWN = {
        "Fire Potion", "Explosive Potion", "Weak Potion",
        "FearPotion", "Poison Potion", "SmokeBomb",
    }

    def test_thrown_potions_correct(self):
        actual_thrown = {p.id for p in ALL_POTIONS.values() if p.is_thrown}
        assert actual_thrown == self.JAVA_THROWN

    def test_non_thrown_potions(self):
        for p in ALL_POTIONS.values():
            if p.id not in self.JAVA_THROWN:
                assert not p.is_thrown, f"{p.name} should not be thrown"


class TestDropMechanics:
    """Verify drop mechanics constants match Java."""

    def test_base_drop_chance(self):
        """AbstractRoom: potionChance = 40."""
        assert BASE_POTION_DROP_CHANCE == 40

    def test_blizzard_step(self):
        """Java uses +/-10 for blizzard modifier."""
        assert BLIZZARD_MOD_STEP == 10

    def test_rarity_thresholds(self):
        """PotionHelper: 65 common, 25 uncommon, 10 rare."""
        assert POTION_COMMON_CHANCE == 65
        assert POTION_UNCOMMON_CHANCE == 25
        assert POTION_RARE_CHANCE == 10
        assert POTION_COMMON_CHANCE + POTION_UNCOMMON_CHANCE + POTION_RARE_CHANCE == 100

    def test_white_beast_statue_guarantees_drop(self):
        assert calculate_drop_chance("monster", has_white_beast_statue=True) == 100

    def test_four_rewards_cap(self):
        """Java: if rewards.size() >= 4, no potion added."""
        assert calculate_drop_chance("monster", current_rewards=4) == 0
        assert calculate_drop_chance("monster", current_rewards=3) == 40


class TestPotionSlots:
    """Verify slot mechanics match Java."""

    def test_base_slots(self):
        """Java: AbstractPlayer.potionSlots = 3."""
        assert calculate_potion_slots(0) == 3

    def test_a11_reduction(self):
        """Java: if ascensionLevel >= 11, potionSlots--."""
        assert calculate_potion_slots(11) == 2
        assert calculate_potion_slots(20) == 2

    def test_potion_belt(self):
        """Java: PotionBelt onEquip: potionSlots += 2."""
        assert calculate_potion_slots(0, has_potion_belt=True) == 5
        assert calculate_potion_slots(11, has_potion_belt=True) == 4


class TestSpecialMechanics:
    """Verify special potion behaviors match Java."""

    def test_fairy_auto_trigger(self):
        """FairyPotion.canUse() returns false; triggers via onPlayerDeath."""
        assert FAIRY_POTION.target_type == PotionTargetType.NONE
        assert any("automatically on death" in m for m in FAIRY_POTION.special_mechanics)

    def test_smoke_bomb_boss_restriction(self):
        """SmokeBomb.canUse() checks AbstractRoom instanceof MonsterRoomBoss."""
        assert any("Cannot be used vs bosses" in m for m in SMOKE_BOMB.special_mechanics)

    def test_entropic_brew_sozu_check(self):
        """EntropicBrew.use() checks for Sozu relic."""
        assert any("Sozu" in m for m in ENTROPIC_BREW.special_mechanics)

    def test_blessing_forge_upgrades_all(self):
        """BlessingOfTheForge uses ArmamentsAction(true) for all-hand upgrade."""
        assert any("ArmamentsAction" in m for m in BLESSING_OF_THE_FORGE.special_mechanics)

    def test_speed_potion_temporary(self):
        """SpeedPotion applies Dex + LoseDexterityPower."""
        assert any("temporary" in m.lower() for m in SPEED_POTION.special_mechanics)

    def test_flex_potion_temporary(self):
        """SteroidPotion applies Str + LoseStrengthPower."""
        assert any("temporary" in m.lower() for m in STEROID_POTION.special_mechanics)

    def test_discovery_potions_choose_3(self):
        """Attack/Skill/Power/Colorless potions use Discovery (choose 1 of 3)."""
        for p in [ATTACK_POTION, SKILL_POTION, POWER_POTION, COLORLESS_POTION]:
            assert any("Discovery" in m for m in p.special_mechanics), f"{p.name} missing Discovery"

    def test_ambrosia_enters_divinity(self):
        """Ambrosia uses ChangeStanceAction('Divinity')."""
        assert "Enter Divinity" in AMBROSIA.description

    def test_stance_potion_offers_choice(self):
        """StancePotion shows Calm/Wrath option cards."""
        assert "Calm or Wrath" in STANCE_POTION.description

    def test_duplication_power_mechanic(self):
        """DuplicationPotion applies DuplicationPower."""
        assert any("DuplicationPower" in m for m in DUPLICATION_POTION.special_mechanics)

    def test_cultist_potion_ritual(self):
        """CultistPotion applies RitualPower."""
        assert "Ritual" in CULTIST_POTION.description

    def test_fruit_juice_outside_combat(self):
        """FruitJuice can be used outside combat (inherits from AbstractPotion)."""
        assert any("outside combat" in m for m in FRUIT_JUICE.special_mechanics)

    def test_blood_potion_percentage(self):
        """BloodPotion heals % of max HP, not flat."""
        assert any("Percentage" in m or "%" in m for m in BLOOD_POTION.special_mechanics)


class TestFairyPotionHealCalc:
    """Verify Fairy in a Bottle heal calculation matches Java."""

    def test_fairy_30_percent_base(self):
        """Java: player.currentHealth = floor(maxHP * potency / 100)."""
        max_hp = 80
        heal_to = int(max_hp * FAIRY_POTION.potency / 100)
        assert heal_to == 24  # floor(80 * 30 / 100) = 24

    def test_fairy_60_percent_sacred_bark(self):
        """With Sacred Bark: floor(maxHP * 60 / 100)."""
        max_hp = 80
        potency = FAIRY_POTION.get_effective_potency(has_sacred_bark=True)
        heal_to = int(max_hp * potency / 100)
        assert heal_to == 48

    def test_fairy_rounding(self):
        """Java uses floor for the calculation."""
        max_hp = 75
        heal_to = int(max_hp * 30 / 100)
        assert heal_to == 22  # floor(22.5) = 22


class TestBloodPotionHealCalc:
    """Verify Blood Potion heal calculation matches Java."""

    def test_blood_20_percent_base(self):
        max_hp = 80
        heal = int(max_hp * BLOOD_POTION.potency / 100)
        assert heal == 16

    def test_blood_40_percent_sacred_bark(self):
        max_hp = 80
        potency = BLOOD_POTION.get_effective_potency(has_sacred_bark=True)
        heal = int(max_hp * potency / 100)
        assert heal == 32
