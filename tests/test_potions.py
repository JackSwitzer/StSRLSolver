"""
Potion System Tests

Comprehensive tests for Slay the Spire potion mechanics.
Tests cover all potions, Sacred Bark doubling, slot mechanics,
drop rates, ascension effects, and relic interactions.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.content.potions import (
    # Base classes and enums
    Potion, PotionRarity, PotionTargetType, PlayerClass,
    # Helper functions
    get_potion_by_id, get_potion_pool, calculate_potion_slots,
    calculate_drop_chance,
    # Collections
    ALL_POTIONS, COMMON_POTIONS, UNCOMMON_POTIONS, RARE_POTIONS,
    UNIVERSAL_POTIONS, IRONCLAD_POTIONS, SILENT_POTIONS,
    DEFECT_POTIONS, WATCHER_POTIONS,
    # Constants
    POTION_COMMON_CHANCE, POTION_UNCOMMON_CHANCE, POTION_RARE_CHANCE,
    BASE_POTION_DROP_CHANCE, BLIZZARD_MOD_STEP,
    # Common Potions
    BLOCK_POTION, DEXTERITY_POTION, ENERGY_POTION, EXPLOSIVE_POTION,
    FIRE_POTION, STRENGTH_POTION, SWIFT_POTION, WEAK_POTION,
    FEAR_POTION, ATTACK_POTION, SKILL_POTION, POWER_POTION,
    COLORLESS_POTION, SPEED_POTION, STEROID_POTION,
    BLESSING_OF_THE_FORGE, BLOOD_POTION, POISON_POTION,
    FOCUS_POTION, BOTTLED_MIRACLE,
    # Uncommon Potions
    ANCIENT_POTION, REGEN_POTION, GAMBLERS_BREW, LIQUID_BRONZE,
    LIQUID_MEMORIES, ESSENCE_OF_STEEL, DUPLICATION_POTION,
    DISTILLED_CHAOS, ELIXIR, CUNNING_POTION, POTION_OF_CAPACITY,
    STANCE_POTION,
    # Rare Potions
    CULTIST_POTION, FRUIT_JUICE, SNECKO_OIL, FAIRY_POTION,
    SMOKE_BOMB, ENTROPIC_BREW, HEART_OF_IRON, GHOST_IN_A_JAR,
    ESSENCE_OF_DARKNESS, AMBROSIA,
)


# ============================================================================
# SECTION 1: POTION DATA STRUCTURE TESTS
# ============================================================================

class TestPotionDataStructure:
    """Test that potion data is properly structured."""

    def test_all_potions_have_required_fields(self):
        """Every potion has all required fields."""
        for potion_id, potion in ALL_POTIONS.items():
            assert potion.id is not None, f"{potion_id} missing id"
            assert potion.name is not None, f"{potion_id} missing name"
            assert potion.rarity is not None, f"{potion_id} missing rarity"
            assert isinstance(potion.potency, int), f"{potion_id} potency not int"
            assert potion.target_type is not None, f"{potion_id} missing target_type"
            assert potion.player_class is not None, f"{potion_id} missing player_class"

    def test_potion_count(self):
        """Verify total potion count matches expected."""
        # Game has 42 potions total (30 universal + 12 class-specific)
        assert len(ALL_POTIONS) == 42

    def test_rarity_distribution(self):
        """Verify potion rarity distribution."""
        common_count = len([p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.COMMON])
        uncommon_count = len([p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.UNCOMMON])
        rare_count = len([p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.RARE])

        assert common_count == 20  # 16 universal common + 4 class-specific common
        assert uncommon_count == 12  # 8 universal uncommon + 4 class-specific uncommon
        assert rare_count == 10   # 6 universal rare + 4 class-specific rare

    def test_get_potion_by_id_valid(self):
        """Get potion by valid ID."""
        potion = get_potion_by_id("Fire Potion")
        assert potion is not None
        assert potion.name == "Fire Potion"
        assert potion.potency == 20

    def test_get_potion_by_id_invalid(self):
        """Get potion by invalid ID returns None."""
        assert get_potion_by_id("Invalid Potion") is None
        assert get_potion_by_id("") is None


# ============================================================================
# SECTION 2: COMBAT POTIONS - DAMAGE
# ============================================================================

class TestDamagePotions:
    """Test damage-dealing potions."""

    def test_fire_potion_base_values(self):
        """Fire Potion deals 20 damage to single enemy."""
        assert FIRE_POTION.potency == 20
        assert FIRE_POTION.target_type == PotionTargetType.ENEMY
        assert FIRE_POTION.is_thrown == True

    def test_fire_potion_sacred_bark(self):
        """Fire Potion doubles with Sacred Bark."""
        assert FIRE_POTION.get_effective_potency(has_sacred_bark=False) == 20
        assert FIRE_POTION.get_effective_potency(has_sacred_bark=True) == 40

    def test_explosive_potion_base_values(self):
        """Explosive Potion deals 10 to all enemies."""
        assert EXPLOSIVE_POTION.potency == 10
        assert EXPLOSIVE_POTION.target_type == PotionTargetType.ALL_ENEMIES
        assert EXPLOSIVE_POTION.is_thrown == True

    def test_explosive_potion_sacred_bark(self):
        """Explosive Potion doubles with Sacred Bark."""
        assert EXPLOSIVE_POTION.get_effective_potency(has_sacred_bark=True) == 20

    def test_poison_potion_values(self):
        """Poison Potion applies 6 poison."""
        assert POISON_POTION.potency == 6
        assert POISON_POTION.target_type == PotionTargetType.ENEMY
        assert POISON_POTION.player_class == PlayerClass.SILENT
        assert POISON_POTION.get_effective_potency(has_sacred_bark=True) == 12


# ============================================================================
# SECTION 3: COMBAT POTIONS - BLOCK
# ============================================================================

class TestBlockPotions:
    """Test block-granting potions."""

    def test_block_potion_base_values(self):
        """Block Potion grants 12 block."""
        assert BLOCK_POTION.potency == 12
        assert BLOCK_POTION.target_type == PotionTargetType.SELF

    def test_block_potion_sacred_bark(self):
        """Block Potion doubles with Sacred Bark."""
        assert BLOCK_POTION.get_effective_potency(has_sacred_bark=True) == 24

    def test_block_potion_ignores_dexterity(self):
        """Block Potion ignores Dexterity (per special mechanics)."""
        assert "Block gained ignores Dexterity" in BLOCK_POTION.special_mechanics


# ============================================================================
# SECTION 4: BUFF APPLICATION POTIONS
# ============================================================================

class TestBuffPotions:
    """Test potions that apply buffs."""

    def test_strength_potion(self):
        """Strength Potion grants 2 Strength permanently."""
        assert STRENGTH_POTION.potency == 2
        assert STRENGTH_POTION.target_type == PotionTargetType.SELF
        assert "Permanent for combat duration" in STRENGTH_POTION.special_mechanics
        assert STRENGTH_POTION.get_effective_potency(has_sacred_bark=True) == 4

    def test_dexterity_potion(self):
        """Dexterity Potion grants 2 Dexterity permanently."""
        assert DEXTERITY_POTION.potency == 2
        assert "Permanent for combat duration" in DEXTERITY_POTION.special_mechanics
        assert DEXTERITY_POTION.get_effective_potency(has_sacred_bark=True) == 4

    def test_focus_potion(self):
        """Focus Potion grants 2 Focus (Defect-specific)."""
        assert FOCUS_POTION.potency == 2
        assert FOCUS_POTION.player_class == PlayerClass.DEFECT
        assert FOCUS_POTION.get_effective_potency(has_sacred_bark=True) == 4

    def test_ancient_potion_artifact(self):
        """Ancient Potion grants 1 Artifact."""
        assert ANCIENT_POTION.potency == 1
        assert ANCIENT_POTION.rarity == PotionRarity.UNCOMMON
        assert ANCIENT_POTION.get_effective_potency(has_sacred_bark=True) == 2


# ============================================================================
# SECTION 5: DEBUFF APPLICATION POTIONS
# ============================================================================

class TestDebuffPotions:
    """Test potions that apply debuffs to enemies."""

    def test_weak_potion(self):
        """Weak Potion applies 3 Weak."""
        assert WEAK_POTION.potency == 3
        assert WEAK_POTION.target_type == PotionTargetType.ENEMY
        assert WEAK_POTION.is_thrown == True
        assert WEAK_POTION.get_effective_potency(has_sacred_bark=True) == 6

    def test_fear_potion(self):
        """Fear Potion applies 3 Vulnerable."""
        assert FEAR_POTION.potency == 3
        assert FEAR_POTION.target_type == PotionTargetType.ENEMY
        assert FEAR_POTION.get_effective_potency(has_sacred_bark=True) == 6


# ============================================================================
# SECTION 6: INSTANT vs TURN-DURATION POTIONS
# ============================================================================

class TestPotionDuration:
    """Test instant effects vs temporary effects."""

    def test_permanent_buff_potions(self):
        """Strength/Dexterity/Focus potions are permanent for combat."""
        permanent_potions = [STRENGTH_POTION, DEXTERITY_POTION, FOCUS_POTION]
        for p in permanent_potions:
            assert "Permanent for combat duration" in p.special_mechanics

    def test_temporary_buff_potions(self):
        """Speed/Flex potions are temporary (lost at end of turn)."""
        # Speed Potion - temporary dexterity
        assert SPEED_POTION.potency == 5
        assert "Dexterity is temporary" in SPEED_POTION.special_mechanics[0]

        # Flex Potion (Steroid Potion) - temporary strength
        assert STEROID_POTION.potency == 5
        assert "Strength is temporary" in STEROID_POTION.special_mechanics[0]

    def test_speed_potion_vs_dexterity_potion(self):
        """Speed gives more dexterity but temporary vs permanent."""
        # Speed: 5 temp dex, Dexterity: 2 permanent
        assert SPEED_POTION.potency == 5
        assert DEXTERITY_POTION.potency == 2
        # Sacred Bark doubles both
        assert SPEED_POTION.get_effective_potency(has_sacred_bark=True) == 10
        assert DEXTERITY_POTION.get_effective_potency(has_sacred_bark=True) == 4

    def test_instant_draw_potions(self):
        """Swift Potion draws 3 cards (instant effect)."""
        assert SWIFT_POTION.potency == 3
        assert SWIFT_POTION.get_effective_potency(has_sacred_bark=True) == 6

    def test_energy_potion(self):
        """Energy Potion grants 2 energy (instant for turn)."""
        assert ENERGY_POTION.potency == 2
        assert ENERGY_POTION.target_type == PotionTargetType.SELF
        assert ENERGY_POTION.get_effective_potency(has_sacred_bark=True) == 4


# ============================================================================
# SECTION 7: SACRED BARK DOUBLING
# ============================================================================

class TestSacredBarkDoubling:
    """Test Sacred Bark relic interaction with potions."""

    def test_sacred_bark_scales_default_true(self):
        """Most potions scale with Sacred Bark by default."""
        scaling_potions = [
            FIRE_POTION, BLOCK_POTION, STRENGTH_POTION,
            ENERGY_POTION, SWIFT_POTION, WEAK_POTION
        ]
        for p in scaling_potions:
            assert p.sacred_bark_scales == True

    def test_potions_unaffected_by_sacred_bark(self):
        """Some potions are NOT affected by Sacred Bark."""
        non_scaling = [
            BLESSING_OF_THE_FORGE,  # Upgrades all cards - binary effect
            GAMBLERS_BREW,          # Discard/draw mechanic
            ELIXIR,                 # Exhaust mechanic
            STANCE_POTION,          # Enter stance - binary
            SMOKE_BOMB,             # Escape - binary
            ENTROPIC_BREW,          # Fill potion slots
            AMBROSIA,               # Enter Divinity - binary
        ]
        for p in non_scaling:
            assert p.sacred_bark_scales == False, f"{p.name} should not scale"

    def test_sacred_bark_on_non_scaling_potion(self):
        """Non-scaling potions return base potency even with Sacred Bark."""
        assert SMOKE_BOMB.get_effective_potency(has_sacred_bark=True) == 0
        assert GAMBLERS_BREW.get_effective_potency(has_sacred_bark=True) == 0

    def test_sacred_bark_doubles_correctly(self):
        """Sacred Bark exactly doubles potency values."""
        test_cases = [
            (FIRE_POTION, 20, 40),
            (BLOCK_POTION, 12, 24),
            (STRENGTH_POTION, 2, 4),
            (POISON_POTION, 6, 12),
            (REGEN_POTION, 5, 10),
            (LIQUID_BRONZE, 3, 6),  # Thorns
        ]
        for potion, base, doubled in test_cases:
            assert potion.get_effective_potency(False) == base
            assert potion.get_effective_potency(True) == doubled


# ============================================================================
# SECTION 8: FAIRY IN A BOTTLE RESURRECTION
# ============================================================================

class TestFairyPotion:
    """Test Fairy in a Bottle resurrection mechanics."""

    def test_fairy_potion_basic_properties(self):
        """Fairy in a Bottle has correct base values."""
        assert FAIRY_POTION.potency == 30  # 30% max HP
        assert FAIRY_POTION.rarity == PotionRarity.RARE
        assert FAIRY_POTION.target_type == PotionTargetType.NONE  # Auto-trigger

    def test_fairy_potion_cannot_be_used_manually(self):
        """Fairy triggers on death, not manual use."""
        assert "Cannot be manually used" in FAIRY_POTION.special_mechanics[1]
        assert "Triggers automatically on death" in FAIRY_POTION.special_mechanics[0]

    def test_fairy_potion_sacred_bark(self):
        """Sacred Bark doubles Fairy healing (30% -> 60%)."""
        assert FAIRY_POTION.sacred_bark_scales == True
        assert FAIRY_POTION.get_effective_potency(has_sacred_bark=False) == 30
        assert FAIRY_POTION.get_effective_potency(has_sacred_bark=True) == 60

    def test_fairy_potion_heal_percentage(self):
        """Fairy heals percentage of max HP."""
        # With 80 max HP, heals to 30% = 24 HP
        max_hp = 80
        heal_percent = FAIRY_POTION.potency
        expected_hp = int(max_hp * heal_percent / 100)
        assert expected_hp == 24

        # With Sacred Bark: 60% = 48 HP
        heal_percent_bark = FAIRY_POTION.get_effective_potency(has_sacred_bark=True)
        expected_hp_bark = int(max_hp * heal_percent_bark / 100)
        assert expected_hp_bark == 48


# ============================================================================
# SECTION 9: FRUIT JUICE PERMANENT HP
# ============================================================================

class TestFruitJuice:
    """Test Fruit Juice permanent max HP increase."""

    def test_fruit_juice_basic_properties(self):
        """Fruit Juice grants 5 max HP."""
        assert FRUIT_JUICE.potency == 5
        assert FRUIT_JUICE.rarity == PotionRarity.RARE

    def test_fruit_juice_is_permanent(self):
        """Fruit Juice max HP is permanent."""
        assert "Permanent Max HP increase" in FRUIT_JUICE.special_mechanics

    def test_fruit_juice_usable_outside_combat(self):
        """Fruit Juice can be used outside combat."""
        assert "Can be used outside combat" in FRUIT_JUICE.special_mechanics

    def test_fruit_juice_sacred_bark(self):
        """Sacred Bark doubles Fruit Juice (5 -> 10 max HP)."""
        assert FRUIT_JUICE.get_effective_potency(has_sacred_bark=True) == 10


# ============================================================================
# SECTION 10: SMOKE BOMB ESCAPE
# ============================================================================

class TestSmokeBomb:
    """Test Smoke Bomb escape mechanic."""

    def test_smoke_bomb_basic_properties(self):
        """Smoke Bomb has correct properties."""
        assert SMOKE_BOMB.potency == 0  # Binary effect
        assert SMOKE_BOMB.rarity == PotionRarity.RARE
        assert SMOKE_BOMB.is_thrown == True

    def test_smoke_bomb_boss_restriction(self):
        """Smoke Bomb cannot be used vs bosses."""
        assert "Cannot be used vs bosses" in SMOKE_BOMB.special_mechanics

    def test_smoke_bomb_back_attack_restriction(self):
        """Smoke Bomb cannot be used when enemy has BackAttack."""
        assert "Cannot be used if enemy has BackAttack power" in SMOKE_BOMB.special_mechanics

    def test_smoke_bomb_sacred_bark_no_effect(self):
        """Sacred Bark has no effect on Smoke Bomb."""
        assert SMOKE_BOMB.sacred_bark_scales == False
        assert SMOKE_BOMB.get_effective_potency(has_sacred_bark=True) == 0


# ============================================================================
# SECTION 11: ENTROPIC BREW RANDOM POTIONS
# ============================================================================

class TestEntropicBrew:
    """Test Entropic Brew random potion generation."""

    def test_entropic_brew_basic_properties(self):
        """Entropic Brew fills potion slots."""
        assert ENTROPIC_BREW.potency == 3  # Default slots
        assert ENTROPIC_BREW.rarity == PotionRarity.RARE

    def test_entropic_brew_uses_potion_slots(self):
        """Entropic Brew potency = potion slots."""
        assert "Potency = number of potion slots" in ENTROPIC_BREW.special_mechanics

    def test_entropic_brew_usable_outside_combat(self):
        """Entropic Brew can be used outside combat."""
        assert "Can be used outside combat" in ENTROPIC_BREW.special_mechanics

    def test_entropic_brew_sacred_bark_no_effect(self):
        """Sacred Bark doesn't affect number of potions generated."""
        assert ENTROPIC_BREW.sacred_bark_scales == False
        assert "Sacred Bark has no effect" in ENTROPIC_BREW.special_mechanics[-1]

    def test_entropic_brew_sozu_interaction(self):
        """Entropic Brew does nothing with Sozu equipped."""
        assert "Does nothing if Sozu is equipped" in ENTROPIC_BREW.special_mechanics[2]


# ============================================================================
# SECTION 12: DUPLICATION POTION CARD COPYING
# ============================================================================

class TestDuplicationPotion:
    """Test Duplication Potion card copying."""

    def test_duplication_potion_basic_properties(self):
        """Duplication Potion plays next card twice."""
        assert DUPLICATION_POTION.potency == 1
        assert DUPLICATION_POTION.rarity == PotionRarity.UNCOMMON

    def test_duplication_potion_uses_power(self):
        """Duplication Potion applies DuplicationPower."""
        assert "Uses DuplicationPower" in DUPLICATION_POTION.special_mechanics

    def test_duplication_potion_sacred_bark(self):
        """Sacred Bark makes next 2 cards play twice."""
        assert DUPLICATION_POTION.sacred_bark_scales == True
        assert DUPLICATION_POTION.get_effective_potency(has_sacred_bark=True) == 2
        assert "Next 2 cards played twice" in DUPLICATION_POTION.special_mechanics[1]


# ============================================================================
# SECTION 13: POTION BELT SLOT LIMITS
# ============================================================================

class TestPotionSlots:
    """Test potion slot mechanics."""

    def test_base_potion_slots(self):
        """Base potion slots is 3."""
        assert calculate_potion_slots(ascension_level=0, has_potion_belt=False) == 3

    def test_ascension_11_reduces_slots(self):
        """Ascension 11+ reduces slots by 1."""
        assert calculate_potion_slots(ascension_level=10, has_potion_belt=False) == 3
        assert calculate_potion_slots(ascension_level=11, has_potion_belt=False) == 2
        assert calculate_potion_slots(ascension_level=15, has_potion_belt=False) == 2
        assert calculate_potion_slots(ascension_level=20, has_potion_belt=False) == 2

    def test_potion_belt_adds_slots(self):
        """Potion Belt adds 2 slots."""
        assert calculate_potion_slots(ascension_level=0, has_potion_belt=True) == 5  # 3 + 2
        assert calculate_potion_slots(ascension_level=11, has_potion_belt=True) == 4  # 2 + 2

    def test_combined_slot_modifiers(self):
        """Test combined ascension and Potion Belt."""
        # A0 + Belt: 3 + 2 = 5
        assert calculate_potion_slots(0, True) == 5
        # A11 + Belt: 3 - 1 + 2 = 4
        assert calculate_potion_slots(11, True) == 4
        # A20 + Belt: 3 - 1 + 2 = 4
        assert calculate_potion_slots(20, True) == 4


# ============================================================================
# SECTION 14: POTION DROP GENERATION AND RARITY
# ============================================================================

class TestPotionDrops:
    """Test potion drop mechanics."""

    def test_rarity_chances_sum_to_100(self):
        """Rarity chances must sum to 100%."""
        total = POTION_COMMON_CHANCE + POTION_UNCOMMON_CHANCE + POTION_RARE_CHANCE
        assert total == 100

    def test_rarity_distribution_values(self):
        """Verify exact rarity percentages."""
        assert POTION_COMMON_CHANCE == 65
        assert POTION_UNCOMMON_CHANCE == 25
        assert POTION_RARE_CHANCE == 10

    def test_base_drop_chance(self):
        """Base drop chance is 40%."""
        assert BASE_POTION_DROP_CHANCE == 40

    def test_blizzard_modifier_step(self):
        """Blizzard mod changes by 10% per miss/drop."""
        assert BLIZZARD_MOD_STEP == 10

    def test_drop_chance_calculation_base(self):
        """Base drop chance from monster/elite/event."""
        for room_type in ["monster", "elite", "event"]:
            assert calculate_drop_chance(room_type) == 40

    def test_drop_chance_blizzard_positive(self):
        """Drop chance increases with positive blizzard mod."""
        # After 2 misses: +20%
        assert calculate_drop_chance("monster", blizzard_mod=20) == 60

    def test_drop_chance_blizzard_negative(self):
        """Drop chance decreases with negative blizzard mod."""
        # After 2 drops: -20%
        assert calculate_drop_chance("monster", blizzard_mod=-20) == 20
        # Can't go below 0
        assert calculate_drop_chance("monster", blizzard_mod=-50) == 0

    def test_drop_chance_white_beast_statue(self):
        """White Beast Statue guarantees potion drop."""
        assert calculate_drop_chance("monster", has_white_beast_statue=True) == 100
        # Even with negative blizzard
        assert calculate_drop_chance("monster", blizzard_mod=-50, has_white_beast_statue=True) == 100

    def test_drop_chance_max_rewards_cap(self):
        """Drop chance is 0 if 4+ rewards already."""
        assert calculate_drop_chance("monster", current_rewards=4) == 0
        assert calculate_drop_chance("monster", current_rewards=5) == 0
        # But 3 rewards is still fine
        assert calculate_drop_chance("monster", current_rewards=3) == 40


# ============================================================================
# SECTION 15: CLASS-SPECIFIC POTIONS
# ============================================================================

class TestClassSpecificPotions:
    """Test class-specific potion availability."""

    def test_ironclad_potions(self):
        """Ironclad has Blood Potion, Elixir, Heart of Iron."""
        ironclad = [p.id for p in IRONCLAD_POTIONS]
        assert "BloodPotion" in ironclad
        assert "ElixirPotion" in ironclad
        assert "HeartOfIron" in ironclad
        assert len(IRONCLAD_POTIONS) == 3

    def test_silent_potions(self):
        """Silent has Poison, Cunning, Ghost in a Jar."""
        silent = [p.id for p in SILENT_POTIONS]
        assert "Poison Potion" in silent
        assert "CunningPotion" in silent
        assert "GhostInAJar" in silent
        assert len(SILENT_POTIONS) == 3

    def test_defect_potions(self):
        """Defect has Focus, Capacity, Essence of Darkness."""
        defect = [p.id for p in DEFECT_POTIONS]
        assert "FocusPotion" in defect
        assert "PotionOfCapacity" in defect
        assert "EssenceOfDarkness" in defect
        assert len(DEFECT_POTIONS) == 3

    def test_watcher_potions(self):
        """Watcher has Bottled Miracle, Stance Potion, Ambrosia."""
        watcher = [p.id for p in WATCHER_POTIONS]
        assert "BottledMiracle" in watcher
        assert "StancePotion" in watcher
        assert "Ambrosia" in watcher
        assert len(WATCHER_POTIONS) == 3

    def test_universal_potions_count(self):
        """Universal potions available to all classes."""
        # 42 total - 12 class-specific = 30 universal
        assert len(UNIVERSAL_POTIONS) == 30


# ============================================================================
# SECTION 16: POTION POOL BY CLASS
# ============================================================================

class TestPotionPool:
    """Test potion pool generation by class."""

    def test_ironclad_pool_includes_universal(self):
        """Ironclad pool has universal + Ironclad potions."""
        pool = get_potion_pool(PlayerClass.IRONCLAD)
        pool_ids = [p.id for p in pool]
        # Has universal
        assert "Fire Potion" in pool_ids
        assert "Block Potion" in pool_ids
        # Has Ironclad-specific
        assert "BloodPotion" in pool_ids
        # Doesn't have other class
        assert "Poison Potion" not in pool_ids

    def test_silent_pool(self):
        """Silent pool has universal + Silent potions."""
        pool = get_potion_pool(PlayerClass.SILENT)
        pool_ids = [p.id for p in pool]
        assert "Poison Potion" in pool_ids
        assert "BloodPotion" not in pool_ids

    def test_defect_pool(self):
        """Defect pool has universal + Defect potions."""
        pool = get_potion_pool(PlayerClass.DEFECT)
        pool_ids = [p.id for p in pool]
        assert "FocusPotion" in pool_ids
        assert "BloodPotion" not in pool_ids

    def test_watcher_pool(self):
        """Watcher pool has universal + Watcher potions."""
        pool = get_potion_pool(PlayerClass.WATCHER)
        pool_ids = [p.id for p in pool]
        assert "BottledMiracle" in pool_ids
        assert "Ambrosia" in pool_ids
        assert "BloodPotion" not in pool_ids

    def test_pool_size_by_class(self):
        """Each class has 30 universal + 3 class-specific = 33 potions."""
        for player_class in [PlayerClass.IRONCLAD, PlayerClass.SILENT,
                            PlayerClass.DEFECT, PlayerClass.WATCHER]:
            pool = get_potion_pool(player_class)
            assert len(pool) == 33


# ============================================================================
# SECTION 17: SPECIAL POTION MECHANICS
# ============================================================================

class TestSpecialPotions:
    """Test potions with unique mechanics."""

    def test_blood_potion_percentage_heal(self):
        """Blood Potion heals percentage of max HP."""
        assert BLOOD_POTION.potency == 20  # 20%
        assert "Percentage-based healing" in BLOOD_POTION.special_mechanics
        # Sacred Bark: 40%
        assert BLOOD_POTION.get_effective_potency(has_sacred_bark=True) == 40

    def test_regen_potion_over_time(self):
        """Regeneration Potion heals over multiple turns."""
        assert REGEN_POTION.potency == 5
        assert "Heals 5 HP at end of each turn" in REGEN_POTION.special_mechanics[0]

    def test_snecko_oil_card_draw_and_randomize(self):
        """Snecko Oil draws 5 and randomizes costs."""
        assert SNECKO_OIL.potency == 5
        assert SNECKO_OIL.rarity == PotionRarity.RARE

    def test_liquid_memories_discard_retrieval(self):
        """Liquid Memories retrieves from discard."""
        assert LIQUID_MEMORIES.potency == 1
        assert "return it to your hand" in LIQUID_MEMORIES.description
        # Sacred Bark: return 2 cards
        assert LIQUID_MEMORIES.get_effective_potency(has_sacred_bark=True) == 2

    def test_distilled_chaos_plays_cards(self):
        """Distilled Chaos plays top 3 cards from draw pile."""
        assert DISTILLED_CHAOS.potency == 3
        assert "Play the top 3 cards" in DISTILLED_CHAOS.description
        # Sacred Bark doubles to 6
        assert DISTILLED_CHAOS.get_effective_potency(has_sacred_bark=True) == 6

    def test_cunning_potion_upgraded_shivs(self):
        """Cunning Potion adds upgraded Shivs."""
        assert CUNNING_POTION.potency == 3
        assert "Upgraded Shivs" in CUNNING_POTION.description
        # Sacred Bark: 6 shivs
        assert CUNNING_POTION.get_effective_potency(has_sacred_bark=True) == 6

    def test_ghost_in_a_jar_intangible(self):
        """Ghost In A Jar grants Intangible."""
        assert GHOST_IN_A_JAR.potency == 1
        assert GHOST_IN_A_JAR.player_class == PlayerClass.SILENT
        # Sacred Bark: 2 turns of Intangible
        assert GHOST_IN_A_JAR.get_effective_potency(has_sacred_bark=True) == 2


# ============================================================================
# SECTION 18: WATCHER-SPECIFIC POTIONS
# ============================================================================

class TestWatcherPotions:
    """Test Watcher-specific potion mechanics."""

    def test_bottled_miracle_adds_miracles(self):
        """Bottled Miracle adds Miracle cards."""
        assert BOTTLED_MIRACLE.potency == 2
        assert "Miracle cards" in BOTTLED_MIRACLE.description
        # Sacred Bark: 4 miracles
        assert BOTTLED_MIRACLE.get_effective_potency(has_sacred_bark=True) == 4

    def test_stance_potion_choice(self):
        """Stance Potion lets you choose stance."""
        assert "Enter Calm or Wrath" in STANCE_POTION.description
        assert STANCE_POTION.sacred_bark_scales == False

    def test_ambrosia_enters_divinity(self):
        """Ambrosia enters Divinity stance."""
        assert "Enter Divinity" in AMBROSIA.description
        assert AMBROSIA.sacred_bark_scales == False


# ============================================================================
# SECTION 19: DEFECT-SPECIFIC POTIONS
# ============================================================================

class TestDefectPotions:
    """Test Defect-specific potion mechanics."""

    def test_potion_of_capacity_orb_slots(self):
        """Potion of Capacity grants orb slots."""
        assert POTION_OF_CAPACITY.potency == 2
        assert "Orb slots" in POTION_OF_CAPACITY.description
        # Sacred Bark: 4 slots
        assert POTION_OF_CAPACITY.get_effective_potency(has_sacred_bark=True) == 4

    def test_essence_of_darkness_channels_dark(self):
        """Essence of Darkness channels Dark orbs."""
        assert ESSENCE_OF_DARKNESS.potency == 1  # Per orb slot
        assert "Dark" in ESSENCE_OF_DARKNESS.description
        # Sacred Bark: 2 dark per slot
        assert ESSENCE_OF_DARKNESS.get_effective_potency(has_sacred_bark=True) == 2


# ============================================================================
# SECTION 20: CARD DISCOVERY POTIONS
# ============================================================================

class TestCardDiscoveryPotions:
    """Test potions that discover cards."""

    def test_attack_potion_discovery(self):
        """Attack Potion uses Discovery mechanic."""
        assert ATTACK_POTION.potency == 1
        assert "Discovery" in ATTACK_POTION.special_mechanics[0]
        assert "choose from 3" in ATTACK_POTION.special_mechanics[0].lower()

    def test_skill_potion_discovery(self):
        """Skill Potion uses Discovery mechanic."""
        assert SKILL_POTION.potency == 1
        assert "Discovery" in SKILL_POTION.special_mechanics[0]

    def test_power_potion_discovery(self):
        """Power Potion uses Discovery mechanic."""
        assert POWER_POTION.potency == 1
        assert "Discovery" in POWER_POTION.special_mechanics[0]

    def test_colorless_potion_discovery(self):
        """Colorless Potion uses Discovery mechanic."""
        assert COLORLESS_POTION.potency == 1
        assert "Discovery" in COLORLESS_POTION.special_mechanics[0]

    def test_discovery_potions_sacred_bark(self):
        """Sacred Bark: discover potions add 2 copies."""
        for p in [ATTACK_POTION, SKILL_POTION, POWER_POTION, COLORLESS_POTION]:
            assert "Sacred Bark" in p.special_mechanics[1]
            assert "2 copies" in p.special_mechanics[1]


# ============================================================================
# SECTION 21: EDGE CASES AND RARITY COLLECTIONS
# ============================================================================

class TestPotionCollections:
    """Test potion collection groupings."""

    def test_common_potions_all_common(self):
        """All common potions have COMMON rarity."""
        for p in COMMON_POTIONS:
            assert p.rarity == PotionRarity.COMMON

    def test_uncommon_potions_all_uncommon(self):
        """All uncommon potions have UNCOMMON rarity."""
        for p in UNCOMMON_POTIONS:
            assert p.rarity == PotionRarity.UNCOMMON

    def test_rare_potions_all_rare(self):
        """All rare potions have RARE rarity."""
        for p in RARE_POTIONS:
            assert p.rarity == PotionRarity.RARE

    def test_no_placeholder_rarity_in_real_potions(self):
        """No real potion has PLACEHOLDER rarity."""
        for p in ALL_POTIONS.values():
            assert p.rarity != PotionRarity.PLACEHOLDER


# ============================================================================
# SECTION 22: IRONCLAD-SPECIFIC POTIONS
# ============================================================================

class TestIroncladPotions:
    """Test Ironclad-specific potions."""

    def test_blood_potion_class_restriction(self):
        """Blood Potion is Ironclad-only."""
        assert BLOOD_POTION.player_class == PlayerClass.IRONCLAD

    def test_elixir_exhaust_mechanic(self):
        """Elixir exhausts cards from hand."""
        assert ELIXIR.player_class == PlayerClass.IRONCLAD
        assert "Exhaust" in ELIXIR.description
        assert ELIXIR.sacred_bark_scales == False

    def test_heart_of_iron_metallicize(self):
        """Heart of Iron grants Metallicize."""
        assert HEART_OF_IRON.potency == 6
        assert HEART_OF_IRON.player_class == PlayerClass.IRONCLAD
        # Sacred Bark: 12 Metallicize
        assert HEART_OF_IRON.get_effective_potency(has_sacred_bark=True) == 12


# ============================================================================
# SECTION 23: POTION TARGET TYPES
# ============================================================================

class TestPotionTargeting:
    """Test potion targeting mechanics."""

    def test_self_targeting_potions(self):
        """Potions that target self."""
        self_target = [p for p in ALL_POTIONS.values() if p.target_type == PotionTargetType.SELF]
        # Most potions are self-targeting
        assert len(self_target) > 20

    def test_enemy_targeting_potions(self):
        """Potions that target single enemy."""
        enemy_target = [p for p in ALL_POTIONS.values() if p.target_type == PotionTargetType.ENEMY]
        enemy_ids = [p.id for p in enemy_target]
        assert "Fire Potion" in enemy_ids
        assert "Weak Potion" in enemy_ids
        assert "FearPotion" in enemy_ids
        assert "Poison Potion" in enemy_ids

    def test_all_enemies_targeting_potions(self):
        """Potions that target all enemies."""
        all_target = [p for p in ALL_POTIONS.values() if p.target_type == PotionTargetType.ALL_ENEMIES]
        assert len(all_target) == 1  # Only Explosive Potion
        assert all_target[0].id == "Explosive Potion"

    def test_none_targeting_potions(self):
        """Potions with no targeting (auto-trigger)."""
        none_target = [p for p in ALL_POTIONS.values() if p.target_type == PotionTargetType.NONE]
        none_ids = [p.id for p in none_target]
        assert "FairyPotion" in none_ids  # Auto-triggers on death


# ============================================================================
# SECTION 24: THROWN POTIONS
# ============================================================================

class TestThrownPotions:
    """Test thrown potion flag."""

    def test_thrown_potions_list(self):
        """Verify which potions are thrown."""
        thrown = [p for p in ALL_POTIONS.values() if p.is_thrown]
        thrown_ids = [p.id for p in thrown]
        assert "Fire Potion" in thrown_ids
        assert "Explosive Potion" in thrown_ids
        assert "Weak Potion" in thrown_ids
        assert "FearPotion" in thrown_ids
        assert "Poison Potion" in thrown_ids
        assert "SmokeBomb" in thrown_ids

    def test_non_thrown_potions(self):
        """Block/Strength/etc are not thrown."""
        assert BLOCK_POTION.is_thrown == False
        assert STRENGTH_POTION.is_thrown == False
        assert ENERGY_POTION.is_thrown == False


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
