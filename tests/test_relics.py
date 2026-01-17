"""
Comprehensive Relic Effect Tests for Slay the Spire RL.

Tests all relic trigger conditions and effects, organized by category:
1. Combat start triggers
2. Turn start triggers
3. Card play triggers
4. Damage modification
5. Energy modification
6. HP triggers
7. Gold triggers
8. Potion triggers
9. Rest site triggers
10. Boss relics with downsides
11. Watcher-specific relics
12. Relic synergies and interactions
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from core.content.relics import (
    Relic, RelicTier, PlayerClass, ALL_RELICS, get_relic, get_relics_by_tier,
    get_relics_for_class, get_starter_relic,
    # Starter relics
    BURNING_BLOOD, RING_OF_THE_SNAKE, CRACKED_CORE, PURE_WATER,
    # Common relics
    AKABEKO, ANCHOR, ANCIENT_TEA_SET, ART_OF_WAR, BAG_OF_MARBLES,
    BAG_OF_PREPARATION, BLOOD_VIAL, BRONZE_SCALES, CENTENNIAL_PUZZLE,
    CERAMIC_FISH, DAMARU, DATA_DISK, DREAM_CATCHER, HAPPY_FLOWER,
    JUZU_BRACELET, LANTERN, MAW_BANK, MEAL_TICKET, NUNCHAKU,
    ODDLY_SMOOTH_STONE, OMAMORI, ORICHALCUM, PEN_NIB, POTION_BELT,
    PRESERVED_INSECT, REGAL_PILLOW, SMILING_MASK, SNECKO_SKULL,
    STRAWBERRY, THE_BOOT, TINY_CHEST, TOY_ORNITHOPTER, VAJRA,
    WAR_PAINT, WHETSTONE, RED_SKULL,
    # Uncommon relics
    BLUE_CANDLE, BOTTLED_FLAME, BOTTLED_LIGHTNING, BOTTLED_TORNADO,
    DARKSTONE_PERIAPT, ETERNAL_FEATHER, FROZEN_EGG, GOLD_PLATED_CABLES,
    GREMLIN_HORN, HORN_CLEAT, INK_BOTTLE, KUNAI, LETTER_OPENER,
    MATRYOSHKA, MEAT_ON_THE_BONE, MERCURY_HOURGLASS, MOLTEN_EGG,
    MUMMIFIED_HAND, NINJA_SCROLL, ORNAMENTAL_FAN, PANTOGRAPH,
    PAPER_CRANE, PAPER_FROG, PEAR, QUESTION_CARD, SELF_FORMING_CLAY,
    SHURIKEN, SINGING_BOWL, STRIKE_DUMMY, SUNDIAL, SYMBIOTIC_VIRUS,
    TEARDROP_LOCKET, THE_COURIER, TOXIC_EGG, WHITE_BEAST_STATUE,
    DUALITY, DISCERNING_MONOCLE,
    # Rare relics
    BIRD_FACED_URN, CALIPERS, CAPTAINS_WHEEL, CHARONS_ASHES,
    CHAMPIONS_BELT, CLOAK_CLASP, DEAD_BRANCH, DU_VU_DOLL,
    EMOTION_CHIP, FOSSILIZED_HELIX, GAMBLING_CHIP, GINGER, GIRYA,
    GOLDEN_EYE, ICE_CREAM, INCENSE_BURNER, LIZARD_TAIL, MAGIC_FLOWER,
    MANGO, OLD_COIN, PEACE_PIPE, POCKETWATCH, PRAYER_WHEEL, SHOVEL,
    STONE_CALENDAR, THE_SPECIMEN, THREAD_AND_NEEDLE, TINGSHA, TORII,
    TOUGH_BANDAGES, TUNGSTEN_ROD, TURNIP, UNCEASING_TOP, WING_BOOTS,
    # Boss relics
    BLACK_BLOOD, RING_OF_THE_SERPENT, FROZEN_CORE, HOLY_WATER,
    ASTROLABE, BLACK_STAR, BUSTED_CROWN, CALLING_BELL, COFFEE_DRIPPER,
    CURSED_KEY, ECTOPLASM, EMPTY_CAGE, FUSION_HAMMER, HOVERING_KITE,
    INSERTER, MARK_OF_PAIN, NUCLEAR_BATTERY, PANDORAS_BOX,
    PHILOSOPHERS_STONE, RUNIC_CUBE, RUNIC_DOME, RUNIC_PYRAMID,
    SACRED_BARK, SLAVERS_COLLAR, SNECKO_EYE, SOZU, TINY_HOUSE,
    VELVET_CHOKER, VIOLET_LOTUS, WRIST_BLADE, RUNIC_CAPACITOR,
    # Shop relics
    THE_ABACUS, BRIMSTONE, CAULDRON, CHEMICAL_X, CLOCKWORK_SOUVENIR,
    DOLLYS_MIRROR, FROZEN_EYE, HAND_DRILL, LEES_WAFFLE, MEDICAL_KIT,
    MELANGE, MEMBERSHIP_CARD, ORANGE_PELLETS, ORRERY, PRISMATIC_SHARD,
    SLING, STRANGE_SPOON, TWISTED_FUNNEL,
    # Special relics
    BLOODY_IDOL, CIRCLET, CULTIST_MASK, ENCHIRIDION, FACE_OF_CLERIC,
    GOLDEN_IDOL, GREMLIN_MASK, MARK_OF_THE_BLOOM, MUTAGENIC_STRENGTH,
    NECRONOMICON, NEOWS_LAMENT, NILRYS_CODEX, NLOTHS_GIFT, NLOTHS_MASK,
    ODD_MUSHROOM, RED_CIRCLET, RED_MASK, SPIRIT_POOP, SSSERPENT_HEAD,
    WARPED_TONGS,
)
from core.state.combat import (
    CombatState, EntityState, EnemyCombatState, create_combat,
    create_player, create_enemy,
)


# =============================================================================
# TEST HELPERS
# =============================================================================

def create_test_combat(
    player_hp: int = 70,
    max_hp: int = 80,
    enemies: list = None,
    relics: list = None,
    energy: int = 3,
) -> CombatState:
    """Create a combat state for testing."""
    if enemies is None:
        enemies = [create_enemy("TestEnemy", hp=50, max_hp=50, move_damage=10)]
    if relics is None:
        relics = []

    return create_combat(
        player_hp=player_hp,
        player_max_hp=max_hp,
        enemies=enemies,
        deck=["Strike"] * 5 + ["Defend"] * 5,
        energy=energy,
        relics=relics,
    )


class MockRelicManager:
    """Mock relic manager for testing relic effects."""

    def __init__(self, relics: list = None):
        self.relics = {r.id: r.copy() for r in (relics or [])}
        self.counters = {}
        for relic in self.relics.values():
            if relic.counter_start != -1:
                self.counters[relic.id] = relic.counter_start

    def has_relic(self, relic_id: str) -> bool:
        return relic_id in self.relics

    def get_counter(self, relic_id: str) -> int:
        return self.counters.get(relic_id, -1)

    def set_counter(self, relic_id: str, value: int):
        self.counters[relic_id] = value

    def increment_counter(self, relic_id: str) -> int:
        if relic_id in self.counters:
            self.counters[relic_id] += 1
            return self.counters[relic_id]
        return -1

    def reset_counter(self, relic_id: str):
        if relic_id in self.relics:
            self.counters[relic_id] = self.relics[relic_id].counter_start


# =============================================================================
# 1. COMBAT START TRIGGERS
# =============================================================================

class TestCombatStartTriggers:
    """Test relics that trigger at combat start."""

    def test_bag_of_preparation_draws_two_cards(self):
        """Bag of Preparation: Draw 2 additional cards at combat start."""
        relic = BAG_OF_PREPARATION
        assert "atBattleStart: Draw 2 additional cards" in relic.effects
        assert relic.tier == RelicTier.COMMON

    def test_ring_of_snake_draws_two_cards(self):
        """Ring of Snake (Silent starter): Draw 2 additional cards."""
        relic = RING_OF_THE_SNAKE
        assert "atBattleStart: Draw 2 additional cards" in relic.effects
        assert relic.player_class == PlayerClass.SILENT
        assert relic.tier == RelicTier.STARTER

    def test_anchor_gives_block(self):
        """Anchor: Gain 10 Block at combat start."""
        relic = ANCHOR
        assert "atBattleStart: Gain 10 Block" in relic.effects
        assert relic.tier == RelicTier.COMMON

    def test_akabeko_gives_vigor(self):
        """Akabeko: Gain 8 Vigor at combat start."""
        relic = AKABEKO
        assert "atBattleStart: Gain 8 Vigor (first attack deals +8 damage)" in relic.effects

    def test_bag_of_marbles_applies_vulnerable(self):
        """Bag of Marbles: Apply 1 Vulnerable to all enemies."""
        relic = BAG_OF_MARBLES
        assert "atBattleStart: Apply 1 Vulnerable to ALL enemies" in relic.effects

    def test_blood_vial_heals(self):
        """Blood Vial: Heal 2 HP at combat start."""
        relic = BLOOD_VIAL
        assert "atBattleStart: Heal 2 HP" in relic.effects

    def test_bronze_scales_gives_thorns(self):
        """Bronze Scales: Gain 3 Thorns at combat start."""
        relic = BRONZE_SCALES
        assert "atBattleStart: Gain 3 Thorns" in relic.effects

    def test_vajra_gives_strength(self):
        """Vajra: Gain 1 Strength at combat start."""
        relic = VAJRA
        assert "atBattleStart: Gain 1 Strength" in relic.effects

    def test_oddly_smooth_stone_gives_dexterity(self):
        """Oddly Smooth Stone: Gain 1 Dexterity at combat start."""
        relic = ODDLY_SMOOTH_STONE
        assert "atBattleStart: Gain 1 Dexterity" in relic.effects

    def test_lantern_gives_energy_first_turn(self):
        """Lantern: Gain 1 Energy on first turn."""
        relic = LANTERN
        assert "atBattleStart (first turn): Gain 1 Energy" in relic.effects

    def test_cracked_core_channels_lightning(self):
        """Cracked Core (Defect starter): Channel 1 Lightning at combat start."""
        relic = CRACKED_CORE
        assert "atPreBattle: Channel 1 Lightning orb" in relic.effects
        assert relic.player_class == PlayerClass.DEFECT

    def test_pure_water_adds_miracle(self):
        """Pure Water (Watcher starter): Add Miracle to hand."""
        relic = PURE_WATER
        assert "atBattleStartPreDraw: Add 1 Miracle to hand" in relic.effects
        assert relic.player_class == PlayerClass.WATCHER

    def test_data_disk_gives_focus(self):
        """Data Disk: Gain 1 Focus at combat start."""
        relic = DATA_DISK
        assert "atBattleStart: Gain 1 Focus" in relic.effects

    def test_clockwork_souvenir_gives_artifact(self):
        """Clockwork Souvenir: Gain 1 Artifact at combat start."""
        relic = CLOCKWORK_SOUVENIR
        assert "atBattleStart: Gain 1 Artifact" in relic.effects

    def test_thread_and_needle_gives_plated_armor(self):
        """Thread and Needle: Gain 4 Plated Armor at combat start."""
        relic = THREAD_AND_NEEDLE
        assert "atBattleStart: Gain 4 Plated Armor" in relic.effects

    def test_ninja_scroll_adds_shivs(self):
        """Ninja Scroll: Add 3 Shivs to hand at combat start."""
        relic = NINJA_SCROLL
        assert "atBattleStartPreDraw: Add 3 Shivs to hand" in relic.effects

    def test_twisted_funnel_applies_poison(self):
        """Twisted Funnel: Apply 4 Poison to all enemies."""
        relic = TWISTED_FUNNEL
        assert "atBattleStart: Apply 4 Poison to ALL enemies" in relic.effects

    def test_red_mask_applies_weak(self):
        """Red Mask: Apply 1 Weak to all enemies."""
        relic = RED_MASK
        assert "atBattleStart: Apply 1 Weak to ALL enemies" in relic.effects

    def test_symbiotic_virus_channels_dark(self):
        """Symbiotic Virus: Channel 1 Dark orb at combat start."""
        relic = SYMBIOTIC_VIRUS
        assert "atPreBattle: Channel 1 Dark orb" in relic.effects

    def test_nuclear_battery_channels_plasma(self):
        """Nuclear Battery: Channel 1 Plasma orb at combat start."""
        relic = NUCLEAR_BATTERY
        assert "atPreBattle: Channel 1 Plasma orb" in relic.effects


# =============================================================================
# 2. TURN START TRIGGERS
# =============================================================================

class TestTurnStartTriggers:
    """Test relics that trigger at turn start."""

    def test_happy_flower_counter_mechanics(self):
        """Happy Flower: Counter increments each turn, gain energy at 3."""
        relic = HAPPY_FLOWER
        assert relic.counter_type == "combat"
        assert relic.counter_max == 3
        assert relic.counter_start == 0
        assert "atTurnStart: Counter +1. At 3, gain 1 Energy and reset" in relic.effects

    def test_nunchaku_counter_on_attacks(self):
        """Nunchaku: Counter increments on attack, gain energy at 10."""
        relic = NUNCHAKU
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 10
        assert relic.counter_start == 0
        assert "onUseCard (attack): Counter +1. At 10, gain 1 Energy and reset" in relic.effects

    def test_ink_bottle_counter_on_cards(self):
        """Ink Bottle: Counter increments on any card, draw at 10."""
        relic = INK_BOTTLE
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 10
        assert "onUseCard: Counter +1. At 10, draw 1 card and reset" in relic.effects

    def test_incense_burner_counter_for_intangible(self):
        """Incense Burner: Counter increments each turn, intangible at 6."""
        relic = INCENSE_BURNER
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 6
        assert "atTurnStart: Counter +1. At 6, gain 1 Intangible and reset" in relic.effects

    def test_horn_cleat_block_turn_two(self):
        """Horn Cleat: Gain 14 Block on turn 2."""
        relic = HORN_CLEAT
        assert "atTurnStart (turn 2): Gain 14 Block" in relic.effects

    def test_mercury_hourglass_damage_each_turn(self):
        """Mercury Hourglass: Deal 3 damage to all enemies each turn."""
        relic = MERCURY_HOURGLASS
        assert "atTurnStart: Deal 3 damage to ALL enemies" in relic.effects

    def test_brimstone_strength_with_downside(self):
        """Brimstone: Gain 2 Strength, enemies gain 1 Strength."""
        relic = BRIMSTONE
        assert "atTurnStart: Gain 2 Strength. ALL enemies gain 1 Strength" in relic.effects

    def test_warped_tongs_upgrades_card(self):
        """Warped Tongs: Upgrade a random card in hand."""
        relic = WARPED_TONGS
        assert "atTurnStart: Upgrade a random card in hand for this combat" in relic.effects

    def test_art_of_war_conditional_energy(self):
        """Art of War: Gain 1 Energy if no attacks played last turn."""
        relic = ART_OF_WAR
        assert "atTurnStart: If no attacks played last turn, gain 1 Energy" in relic.effects

    def test_ancient_tea_set_rest_room_trigger(self):
        """Ancient Tea Set: Gain 2 Energy first turn after rest room."""
        relic = ANCIENT_TEA_SET
        assert "onEnterRestRoom: Set counter to -2" in relic.effects
        assert "atTurnStart (first turn, if counter=-2): Gain 2 Energy" in relic.effects

    def test_inserter_orb_slots(self):
        """Inserter: Gain orb slot every 2 turns."""
        relic = INSERTER
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 2
        assert "atTurnStart: Counter +1. At 2, gain 1 Orb slot and reset" in relic.effects

    def test_stone_calendar_late_game_damage(self):
        """Stone Calendar: Deal 52 damage at end of turn 7+."""
        relic = STONE_CALENDAR
        assert relic.counter_type == "combat"
        assert "atTurnStart: Counter +1. At turn 7+, deal 52 damage at end of turn" in relic.effects


# =============================================================================
# 3. CARD PLAY TRIGGERS
# =============================================================================

class TestCardPlayTriggers:
    """Test relics that trigger when cards are played."""

    def test_shuriken_counter_on_attacks(self):
        """Shuriken: Gain 1 Strength after 3 attacks."""
        relic = SHURIKEN
        assert relic.counter_type == "combat"
        assert relic.counter_max == 3
        assert "onUseCard (attack): Counter +1. At 3, gain 1 Strength and reset" in relic.effects

    def test_kunai_counter_on_attacks(self):
        """Kunai: Gain 1 Dexterity after 3 attacks."""
        relic = KUNAI
        assert relic.counter_type == "combat"
        assert relic.counter_max == 3
        assert "onUseCard (attack): Counter +1. At 3, gain 1 Dexterity and reset" in relic.effects

    def test_ornamental_fan_counter_for_block(self):
        """Ornamental Fan: Gain 4 Block after 3 attacks."""
        relic = ORNAMENTAL_FAN
        assert relic.counter_type == "combat"
        assert relic.counter_max == 3
        assert "onUseCard (attack): Counter +1. At 3, gain 4 Block and reset" in relic.effects

    def test_letter_opener_counter_on_skills(self):
        """Letter Opener: Deal 5 damage to all after 3 skills."""
        relic = LETTER_OPENER
        assert relic.counter_type == "combat"
        assert relic.counter_max == 3
        assert "onUseCard (skill): Counter +1. At 3, deal 5 damage to ALL enemies and reset" in relic.effects

    def test_pen_nib_double_damage(self):
        """Pen Nib: Every 10th attack deals double damage."""
        relic = PEN_NIB
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 10
        assert "onUseCard (attack): Counter +1. At 9, next attack deals double damage" in relic.effects

    def test_duality_temporary_dexterity(self):
        """Duality (Watcher): Gain 1 Dexterity this turn when playing attack."""
        relic = DUALITY
        assert "onUseCard (attack): Gain 1 Dexterity this turn" in relic.effects

    def test_bird_faced_urn_heal_on_power(self):
        """Bird-Faced Urn: Heal 2 HP when playing a Power."""
        relic = BIRD_FACED_URN
        assert "onUseCard (power): Heal 2 HP" in relic.effects

    def test_mummified_hand_free_card_on_power(self):
        """Mummified Hand: Random card costs 0 when playing Power."""
        relic = MUMMIFIED_HAND
        assert "onUseCard (power): A random card in hand costs 0 this turn" in relic.effects

    def test_hovering_kite_energy_on_first_discard(self):
        """Hovering Kite: Gain 1 Energy on first manual discard each turn."""
        relic = HOVERING_KITE
        assert "onManualDiscard (first each turn): Gain 1 Energy" in relic.effects

    def test_tingsha_damage_on_discard(self):
        """Tingsha: Deal 3 damage per manually discarded card."""
        relic = TINGSHA
        assert "onManualDiscard: Deal 3 damage to random enemy per card" in relic.effects

    def test_tough_bandages_block_on_discard(self):
        """Tough Bandages: Gain 3 Block per manually discarded card."""
        relic = TOUGH_BANDAGES
        assert "onManualDiscard: Gain 3 Block per card" in relic.effects

    def test_sundial_energy_on_shuffle(self):
        """Sundial: Gain 2 Energy every 3 shuffles."""
        relic = SUNDIAL
        assert relic.counter_type == "permanent"
        assert relic.counter_max == 3
        assert "onShuffle: Counter +1. At 3, gain 2 Energy and reset" in relic.effects

    def test_the_abacus_block_on_shuffle(self):
        """The Abacus: Gain 6 Block when shuffling."""
        relic = THE_ABACUS
        assert "onShuffle: Gain 6 Block" in relic.effects

    def test_dead_branch_card_on_exhaust(self):
        """Dead Branch: Add random card to hand when exhausting."""
        relic = DEAD_BRANCH
        assert "onExhaust: Add a random card to hand" in relic.effects

    def test_charons_ashes_damage_on_exhaust(self):
        """Charon's Ashes: Deal 3 damage to all when exhausting."""
        relic = CHARONS_ASHES
        assert "onExhaust: Deal 3 damage to ALL enemies" in relic.effects


# =============================================================================
# 4. DAMAGE MODIFICATION
# =============================================================================

class TestDamageModification:
    """Test relics that modify damage dealt or received."""

    def test_pen_nib_doubles_attack_damage(self):
        """Pen Nib: At counter 9, next attack deals double damage."""
        relic = PEN_NIB
        # Test counter mechanics
        manager = MockRelicManager([relic])
        for _ in range(9):
            manager.increment_counter(relic.id)
        assert manager.get_counter(relic.id) == 9

    def test_akabeko_first_attack_bonus(self):
        """Akabeko: First attack each combat deals +8 damage (via Vigor)."""
        relic = AKABEKO
        # Vigor adds to attack damage
        assert "Gain 8 Vigor" in relic.effects[0]

    def test_paper_crane_increased_weak_effect(self):
        """Paper Crane: Weakened enemies deal 40% less damage (not 25%)."""
        relic = PAPER_CRANE
        assert "Enemies with Weak deal 40% less damage instead of 25%" in relic.effects

    def test_paper_frog_increased_vulnerable_effect(self):
        """Paper Frog: Vulnerable enemies take 75% more damage (not 50%)."""
        relic = PAPER_FROG
        assert "Enemies with Vulnerable take 75% more damage instead of 50%" in relic.effects

    def test_torii_reduces_small_damage(self):
        """Torii: Reduce damage between 2-5 to 1."""
        relic = TORII
        assert "onAttacked: If damage 2-5, reduce to 1" in relic.effects

    def test_tungsten_rod_reduces_hp_loss(self):
        """Tungsten Rod: Reduce HP loss by 1."""
        relic = TUNGSTEN_ROD
        assert "onLoseHpLast: Reduce HP loss by 1" in relic.effects

    def test_wrist_blade_bonus_on_zero_cost(self):
        """Wrist Blade: 0-cost Attacks deal 4 additional damage."""
        relic = WRIST_BLADE
        assert "atDamageModify: 0-cost Attacks deal 4 additional damage" in relic.effects

    def test_the_boot_minimum_damage(self):
        """The Boot: Attacks that deal <5 damage deal 5 instead."""
        relic = THE_BOOT
        assert "onAttackToChangeDamage: If attack would deal <5 damage, deal 5 instead" in relic.effects

    def test_strike_dummy_bonus_damage(self):
        """Strike Dummy: Strike cards deal 3 additional damage."""
        relic = STRIKE_DUMMY
        assert "Cards containing 'Strike' deal 3 additional damage" in relic.effects

    def test_champions_belt_applies_weak(self):
        """Champion's Belt: Applying Vulnerable also applies 1 Weak."""
        relic = CHAMPIONS_BELT
        assert "Whenever you apply Vulnerable, also apply 1 Weak" in relic.effects

    def test_snecko_skull_extra_poison(self):
        """Snecko Skull: Apply 1 additional Poison."""
        relic = SNECKO_SKULL
        assert "Whenever you apply Poison, apply 1 additional Poison" in relic.effects

    def test_fossilized_helix_prevents_damage_once(self):
        """Fossilized Helix: Prevent all damage first time hit each combat."""
        relic = FOSSILIZED_HELIX
        assert "onAttacked (first time per combat): Prevent all damage" in relic.effects


# =============================================================================
# 5. ENERGY MODIFICATION
# =============================================================================

class TestEnergyModification:
    """Test relics that modify energy."""

    def test_lantern_first_turn_energy(self):
        """Lantern: Gain 1 Energy on first turn of each combat."""
        relic = LANTERN
        assert "atBattleStart (first turn): Gain 1 Energy" in relic.effects

    def test_happy_flower_energy_every_3_turns(self):
        """Happy Flower: Gain 1 Energy every 3 turns."""
        relic = HAPPY_FLOWER
        assert relic.counter_max == 3
        assert "gain 1 Energy" in relic.effects[0]

    def test_ice_cream_conserves_energy(self):
        """Ice Cream: Energy is conserved between turns."""
        relic = ICE_CREAM
        assert "Energy is conserved between turns" in relic.effects

    def test_sundial_energy_on_shuffles(self):
        """Sundial: Gain 2 Energy every 3 shuffles."""
        relic = SUNDIAL
        assert "At 3, gain 2 Energy" in relic.effects[0]

    def test_boss_relics_energy_bonus(self):
        """Boss relics with +1 Energy."""
        energy_relics = [
            (BUSTED_CROWN, 1),
            (COFFEE_DRIPPER, 1),
            (CURSED_KEY, 1),
            (ECTOPLASM, 1),
            (FUSION_HAMMER, 1),
            (PHILOSOPHERS_STONE, 1),
            (RUNIC_DOME, 1),
            (SOZU, 1),
            (VELVET_CHOKER, 1),
        ]
        for relic, expected_bonus in energy_relics:
            assert relic.energy_bonus == expected_bonus, f"{relic.name} should give +{expected_bonus} energy"

    def test_gremlin_horn_energy_on_kill(self):
        """Gremlin Horn: Gain 1 Energy when enemy dies."""
        relic = GREMLIN_HORN
        assert "onMonsterDeath: Gain 1 Energy and draw 1 card" in relic.effects

    def test_slavers_collar_elite_boss_energy(self):
        """Slaver's Collar: +1 Energy in Elite and Boss combats."""
        relic = SLAVERS_COLLAR
        assert "In Elite and Boss combats: +1 Energy" in relic.effects


# =============================================================================
# 6. HP TRIGGERS
# =============================================================================

class TestHPTriggers:
    """Test relics that trigger on HP changes."""

    def test_meat_on_the_bone_heal_at_50_percent(self):
        """Meat on the Bone: Heal 12 HP on victory if HP <= 50%."""
        relic = MEAT_ON_THE_BONE
        assert "onVictory: If HP <= 50%, heal 12 HP" in relic.effects

    def test_self_forming_clay_block_on_damage(self):
        """Self-Forming Clay: Gain 3 Block next turn when losing HP."""
        relic = SELF_FORMING_CLAY
        assert "wasHPLost: Gain 3 Block next turn" in relic.effects

    def test_centennial_puzzle_draw_on_damage(self):
        """Centennial Puzzle: Draw 3 cards first time taking damage."""
        relic = CENTENNIAL_PUZZLE
        assert "wasHPLost (first time per combat): Draw 3 cards" in relic.effects

    def test_red_skull_strength_when_low(self):
        """Red Skull: Gain 3 Strength when HP <= 50%."""
        relic = RED_SKULL
        assert "onBloodied (HP <= 50%): Gain 3 Strength. Lose when healed above 50%" in relic.effects

    def test_runic_cube_draw_on_damage(self):
        """Runic Cube: Draw 1 card when losing HP."""
        relic = RUNIC_CUBE
        assert "wasHPLost: Draw 1 card" in relic.effects

    def test_burning_blood_heal_on_victory(self):
        """Burning Blood (Ironclad starter): Heal 6 HP on victory."""
        relic = BURNING_BLOOD
        assert "onVictory: Heal 6 HP" in relic.effects
        assert relic.player_class == PlayerClass.IRONCLAD

    def test_black_blood_improved_healing(self):
        """Black Blood: Heal 12 HP on victory (upgrade of Burning Blood)."""
        relic = BLACK_BLOOD
        assert "onVictory: Heal 12 HP (replaces Burning Blood)" in relic.effects
        assert relic.requires_relic == "Burning Blood"

    def test_magic_flower_healing_bonus(self):
        """Magic Flower: Healing is 50% more effective."""
        relic = MAGIC_FLOWER
        assert "Healing is 50% more effective" in relic.effects

    def test_mark_of_bloom_prevents_healing(self):
        """Mark of the Bloom: Cannot heal."""
        relic = MARK_OF_THE_BLOOM
        assert relic.prevents_healing == True

    def test_lizard_tail_revive(self):
        """Lizard Tail: Heal to 50% HP when dying (once per run)."""
        relic = LIZARD_TAIL
        assert "When you would die, heal to 50% HP (once per run)" in relic.effects


# =============================================================================
# 7. GOLD TRIGGERS
# =============================================================================

class TestGoldTriggers:
    """Test relics that interact with gold."""

    def test_membership_card_discount(self):
        """Membership Card: 50% discount at shops."""
        relic = MEMBERSHIP_CARD
        assert "50% discount at shops" in relic.effects

    def test_the_courier_shop_bonus(self):
        """The Courier: 20% discount and card removal always available."""
        relic = THE_COURIER
        assert "Shop always has card removal. 20% discount on everything" in relic.effects

    def test_smiling_mask_face_trader(self):
        """Smiling Mask: Face Trader costs 50 Gold instead of HP."""
        relic = SMILING_MASK
        assert "Replaces Face Trader event's HP cost with fixed 50 Gold" in relic.effects

    def test_ceramic_fish_gold_on_card(self):
        """Ceramic Fish: Gain 9 Gold when adding cards."""
        relic = CERAMIC_FISH
        assert "onObtainCard: Gain 9 Gold" in relic.effects

    def test_maw_bank_passive_gold(self):
        """Maw Bank: Gain 12 Gold per non-shop room."""
        relic = MAW_BANK
        assert "onEnterRoom (not shop): Gain 12 Gold" in relic.effects
        assert "onSpendGold: Lose this relic's effect" in relic.effects

    def test_golden_idol_gold_bonus(self):
        """Golden Idol: Gain 25% more Gold."""
        relic = GOLDEN_IDOL
        assert "Gain 25% more Gold" in relic.effects

    def test_bloody_idol_heal_on_gold(self):
        """Bloody Idol: Heal 5 HP when gaining Gold."""
        relic = BLOODY_IDOL
        assert "onGainGold: Heal 5 HP" in relic.effects

    def test_ectoplasm_prevents_gold(self):
        """Ectoplasm: Cannot gain Gold."""
        relic = ECTOPLASM
        assert relic.prevents_gold_gain == True
        assert relic.act_restriction == 1  # Only appears in Act 1

    def test_old_coin_gold_on_equip(self):
        """Old Coin: Gain 300 Gold on pickup."""
        relic = OLD_COIN
        assert "onEquip: Gain 300 Gold" in relic.effects

    def test_ssserpent_head_mystery_gold(self):
        """Ssserpent Head: Gain 50 Gold on entering ? rooms."""
        relic = SSSERPENT_HEAD
        assert "Whenever you enter a ? room, gain 50 Gold" in relic.effects


# =============================================================================
# 8. POTION TRIGGERS
# =============================================================================

class TestPotionTriggers:
    """Test relics that interact with potions."""

    def test_white_beast_statue_guaranteed_potions(self):
        """White Beast Statue: Potions always drop from combat."""
        relic = WHITE_BEAST_STATUE
        assert "Potions always drop from combat rewards" in relic.effects

    def test_sacred_bark_doubles_potions(self):
        """Sacred Bark: Potion effects are doubled."""
        relic = SACRED_BARK
        assert "Potion effects are doubled" in relic.effects

    def test_toy_ornithopter_heal_on_potion(self):
        """Toy Ornithopter: Heal 5 HP when using potions."""
        relic = TOY_ORNITHOPTER
        assert "Whenever you use a potion, heal 5 HP" in relic.effects

    def test_potion_belt_extra_slots(self):
        """Potion Belt: Gain 2 potion slots."""
        relic = POTION_BELT
        assert relic.potion_slots == 2
        assert "onEquip: Gain 2 potion slots" in relic.effects

    def test_sozu_prevents_potions(self):
        """Sozu: Cannot obtain potions."""
        relic = SOZU
        assert relic.prevents_potions == True
        assert relic.energy_bonus == 1

    def test_cauldron_gives_potions(self):
        """Cauldron: Obtain 5 random potions."""
        relic = CAULDRON
        assert "onEquip: Obtain 5 random potions" in relic.effects


# =============================================================================
# 9. REST SITE TRIGGERS
# =============================================================================

class TestRestSiteTriggers:
    """Test relics that interact with rest sites."""

    def test_regal_pillow_bonus_healing(self):
        """Regal Pillow: Heal 15 additional HP when resting."""
        relic = REGAL_PILLOW
        assert "onRest: Heal 15 additional HP" in relic.effects

    def test_dream_catcher_card_reward(self):
        """Dream Catcher: Add a card to deck when resting."""
        relic = DREAM_CATCHER
        assert "onRest: Add a card to your deck" in relic.effects

    def test_shovel_dig_option(self):
        """Shovel: Can dig at rest sites for a relic."""
        relic = SHOVEL
        assert "Can Dig at rest sites for a relic" in relic.effects

    def test_peace_pipe_toke_option(self):
        """Peace Pipe: Can remove a card at rest sites."""
        relic = PEACE_PIPE
        assert "Can Toke at rest sites to remove a card" in relic.effects

    def test_girya_lift_option(self):
        """Girya: Can lift at rest sites for Strength (max 3)."""
        relic = GIRYA
        assert relic.counter_type == "uses"
        assert "Can Lift at rest sites (max 3 times). Each Lift gives 1 permanent Strength" in relic.effects

    def test_eternal_feather_passive_healing(self):
        """Eternal Feather: Heal 3 HP per 5 cards when entering rest site."""
        relic = ETERNAL_FEATHER
        assert "Whenever you enter a Rest Site, heal 3 HP per 5 cards in deck" in relic.effects

    def test_coffee_dripper_prevents_rest(self):
        """Coffee Dripper: Cannot rest at rest sites."""
        relic = COFFEE_DRIPPER
        assert relic.prevents_resting == True
        assert relic.energy_bonus == 1

    def test_fusion_hammer_prevents_smith(self):
        """Fusion Hammer: Cannot smith at rest sites."""
        relic = FUSION_HAMMER
        assert relic.prevents_smithing == True
        assert relic.energy_bonus == 1


# =============================================================================
# 10. BOSS RELICS WITH DOWNSIDES
# =============================================================================

class TestBossRelicsWithDownsides:
    """Test boss relics that have significant downsides."""

    def test_runic_dome_hides_intents(self):
        """Runic Dome: Cannot see enemy intents."""
        relic = RUNIC_DOME
        assert relic.hides_intent == True
        assert relic.energy_bonus == 1

    def test_snecko_eye_confusion(self):
        """Snecko Eye: +2 draw but randomizes card costs."""
        relic = SNECKO_EYE
        assert relic.hand_size_bonus == 2
        assert "atPreBattle: Apply Confused (randomize costs)" in relic.effects[0]

    def test_velvet_choker_card_limit(self):
        """Velvet Choker: Can only play 6 cards per turn."""
        relic = VELVET_CHOKER
        assert relic.energy_bonus == 1
        assert relic.counter_type == "combat"
        assert relic.counter_max == 6
        assert "Can only play 6 cards per turn" in relic.effects[0]

    def test_busted_crown_fewer_cards(self):
        """Busted Crown: +1 Energy but 2 fewer card choices."""
        relic = BUSTED_CROWN
        assert relic.energy_bonus == 1
        assert "Card rewards contain 2 fewer choices" in relic.effects[0]

    def test_philosophers_stone_enemy_buff(self):
        """Philosopher's Stone: +1 Energy but enemies gain 1 Strength."""
        relic = PHILOSOPHERS_STONE
        assert relic.energy_bonus == 1
        assert "atBattleStart: ALL enemies gain 1 Strength" in relic.effects[0]

    def test_mark_of_pain_adds_wounds(self):
        """Mark of Pain: +1 Energy but start with 2 Wounds."""
        relic = MARK_OF_PAIN
        assert relic.energy_bonus == 1
        assert "atBattleStart: Shuffle 2 Wounds into draw pile" in relic.effects[0]

    def test_cursed_key_gives_curses(self):
        """Cursed Key: +1 Energy but gain Curse from chests."""
        relic = CURSED_KEY
        assert relic.energy_bonus == 1
        assert "onChestOpen: Obtain a random Curse" in relic.effects[0]

    def test_calling_bell_gives_relics_and_curse(self):
        """Calling Bell: Get relics but also a Curse."""
        relic = CALLING_BELL
        assert "onEquip: Obtain 1 Curse, 1 Common, 1 Uncommon, 1 Rare relic" in relic.effects


# =============================================================================
# 11. WATCHER-SPECIFIC RELICS
# =============================================================================

class TestWatcherRelics:
    """Test Watcher-specific relics."""

    def test_violet_lotus_calm_energy(self):
        """Violet Lotus: Gain 1 additional Energy when exiting Calm."""
        relic = VIOLET_LOTUS
        assert "onChangeStance (exit Calm): Gain 1 additional Energy" in relic.effects

    def test_damaru_mantra_each_turn(self):
        """Damaru: Gain 1 Mantra at start of each turn."""
        relic = DAMARU
        assert "atTurnStart: Gain 1 Mantra" in relic.effects

    def test_teardrop_locket_start_in_calm(self):
        """Teardrop Locket: Start combat in Calm stance."""
        relic = TEARDROP_LOCKET
        assert "atBattleStart: Enter Calm" in relic.effects

    def test_pure_water_miracle_watcher_starter(self):
        """Pure Water: Watcher starter adds Miracle to hand."""
        relic = PURE_WATER
        assert relic.player_class == PlayerClass.WATCHER
        assert relic.tier == RelicTier.STARTER
        assert "Add 1 Miracle to hand" in relic.effects[0]

    def test_holy_water_upgraded_pure_water(self):
        """Holy Water: Adds 3 Miracles (upgrades Pure Water)."""
        relic = HOLY_WATER
        assert relic.requires_relic == "PureWater"
        assert "Add 3 Miracles to hand" in relic.effects[0]

    def test_golden_eye_scry_bonus(self):
        """Golden Eye: Scry 2 additional cards."""
        relic = GOLDEN_EYE
        assert "onScry: Scry 2 additional cards" in relic.effects

    def test_duality_attack_dexterity(self):
        """Duality: Gain 1 Dexterity this turn when playing Attack."""
        relic = DUALITY
        assert "onUseCard (attack): Gain 1 Dexterity this turn" in relic.effects


# =============================================================================
# 12. RELIC SYNERGIES AND INTERACTIONS
# =============================================================================

class TestRelicSynergies:
    """Test relic synergies and interactions."""

    def test_orichalcum_no_block_fallback(self):
        """Orichalcum: Gain 6 Block if ending turn with no Block."""
        relic = ORICHALCUM
        assert "onPlayerEndTurn: If no Block, gain 6 Block" in relic.effects

    def test_cloak_clasp_hand_size_block(self):
        """Cloak Clasp: Gain 1 Block per card in hand at end of turn."""
        relic = CLOAK_CLASP
        assert "onPlayerEndTurn: Gain 1 Block per card in hand" in relic.effects

    def test_pocketwatch_card_limit_draw(self):
        """Pocketwatch: Draw 3 if played 3 or fewer cards."""
        relic = POCKETWATCH
        assert "onPlayerEndTurn: If played 3 or fewer cards, draw 3 next turn" in relic.effects

    def test_calipers_block_retention(self):
        """Calipers: Lose only 15 Block at turn start."""
        relic = CALIPERS
        assert relic.block_loss_reduction == 15
        assert "At turn start, lose 15 Block instead of all Block" in relic.effects

    def test_runic_pyramid_hand_retention(self):
        """Runic Pyramid: Hand not discarded at end of turn."""
        relic = RUNIC_PYRAMID
        assert "Hand is not discarded at end of turn" in relic.effects

    def test_du_vu_doll_curse_synergy(self):
        """Du-Vu Doll: Gain 1 Strength per Curse in deck."""
        relic = DU_VU_DOLL
        assert "atBattleStart: Gain 1 Strength per Curse in deck" in relic.effects

    def test_darkstone_periapt_curse_synergy(self):
        """Darkstone Periapt: Gain 6 Max HP when obtaining Curse."""
        relic = DARKSTONE_PERIAPT
        assert "Whenever you obtain a Curse, gain 6 Max HP" in relic.effects

    def test_chemical_x_synergy(self):
        """Chemical X: X-cost cards receive +2 to X."""
        relic = CHEMICAL_X
        assert "X-cost cards receive +2 to X" in relic.effects

    def test_orange_pellets_debuff_removal(self):
        """Orange Pellets: Remove debuffs when playing Attack+Skill+Power."""
        relic = ORANGE_PELLETS
        assert "If Attack, Skill, Power played in same turn: Remove all debuffs" in relic.effects

    def test_omamori_curse_negation(self):
        """Omamori: Negate next 2 Curses."""
        relic = OMAMORI
        assert relic.counter_type == "uses"
        assert relic.counter_start == 2
        assert "Negates next 2 Curses added to deck" in relic.effects

    def test_blue_candle_curse_playable(self):
        """Blue Candle: Curse cards can be played (exhaust, lose 1 HP)."""
        relic = BLUE_CANDLE
        assert "Curse cards can be played. Playing a Curse exhausts it and deals 1 HP loss" in relic.effects

    def test_medical_kit_status_playable(self):
        """Medical Kit: Status cards can be played (exhaust)."""
        relic = MEDICAL_KIT
        assert "Status cards can be played. Playing a Status exhausts it" in relic.effects

    def test_unceasing_top_empty_hand_draw(self):
        """Unceasing Top: Draw 1 card when hand is empty."""
        relic = UNCEASING_TOP
        assert "When hand is empty, draw 1 card" in relic.effects

    def test_captains_wheel_turn_three_block(self):
        """Captain's Wheel: Gain 18 Block on turn 3."""
        relic = CAPTAINS_WHEEL
        assert relic.counter_type == "combat"
        assert "atTurnStart (turn 3): Gain 18 Block (once per combat)" in relic.effects


# =============================================================================
# RELIC REGISTRY AND UTILITY TESTS
# =============================================================================

class TestRelicRegistry:
    """Test relic registry functions."""

    def test_all_relics_registered(self):
        """All relics should be in the registry."""
        assert len(ALL_RELICS) >= 150  # Should have ~160+ relics

    def test_get_relic_returns_copy(self):
        """get_relic should return a copy, not the original."""
        relic1 = get_relic("Akabeko")
        relic2 = get_relic("Akabeko")
        assert relic1 is not relic2
        assert relic1.id == relic2.id

    def test_get_relic_unknown_raises(self):
        """get_relic should raise for unknown relics."""
        with pytest.raises(ValueError):
            get_relic("NonexistentRelic")

    def test_get_relics_by_tier(self):
        """get_relics_by_tier should return correct relics."""
        common = get_relics_by_tier(RelicTier.COMMON)
        assert len(common) >= 25
        assert all(r.tier == RelicTier.COMMON for r in common)

    def test_get_starter_relic_for_each_class(self):
        """Each class should have a starter relic."""
        ironclad = get_starter_relic(PlayerClass.IRONCLAD)
        assert ironclad.id == "Burning Blood"

        silent = get_starter_relic(PlayerClass.SILENT)
        assert silent.id == "Ring of the Snake"

        defect = get_starter_relic(PlayerClass.DEFECT)
        assert defect.id == "Cracked Core"

        watcher = get_starter_relic(PlayerClass.WATCHER)
        assert watcher.id == "PureWater"

    def test_get_relics_for_class_includes_all(self):
        """get_relics_for_class should include class-specific and all relics."""
        watcher_relics = get_relics_for_class(PlayerClass.WATCHER)
        # Should include Watcher starter
        assert "PureWater" in watcher_relics
        # Should include ALL relics
        assert "Akabeko" in watcher_relics


class TestRelicDataIntegrity:
    """Test relic data integrity."""

    def test_all_relics_have_id(self):
        """All relics should have an id."""
        for relic_id, relic in ALL_RELICS.items():
            assert relic.id, f"Relic {relic_id} missing id"

    def test_all_relics_have_name(self):
        """All relics should have a name."""
        for relic_id, relic in ALL_RELICS.items():
            assert relic.name, f"Relic {relic_id} missing name"

    def test_all_relics_have_tier(self):
        """All relics should have a tier."""
        for relic_id, relic in ALL_RELICS.items():
            assert relic.tier is not None, f"Relic {relic_id} missing tier"
            assert isinstance(relic.tier, RelicTier)

    def test_starter_relics_have_class(self):
        """Starter relics should have a player class."""
        for relic_id, relic in ALL_RELICS.items():
            if relic.tier == RelicTier.STARTER:
                assert relic.player_class != PlayerClass.ALL, f"Starter {relic_id} missing class"

    def test_counter_relics_have_valid_config(self):
        """Relics with counters should have valid counter config."""
        for relic_id, relic in ALL_RELICS.items():
            if relic.counter_type is not None:
                assert relic.counter_start is not None, f"Relic {relic_id} missing counter_start"

    def test_boss_upgrade_relics_have_requires(self):
        """Boss relics that upgrade starters should have requires_relic."""
        upgrade_relics = {
            "Black Blood": "Burning Blood",
            "Ring of the Serpent": "Ring of the Snake",
            "FrozenCore": "Cracked Core",
            "HolyWater": "PureWater",
        }
        for relic_id, required in upgrade_relics.items():
            relic = ALL_RELICS[relic_id]
            assert relic.requires_relic == required, f"{relic_id} should require {required}"


class TestMaxHPRelics:
    """Test relics that modify max HP."""

    def test_strawberry_max_hp(self):
        """Strawberry: Gain 7 Max HP."""
        relic = STRAWBERRY
        assert relic.max_hp_bonus == 7

    def test_pear_max_hp(self):
        """Pear: Gain 10 Max HP."""
        relic = PEAR
        assert relic.max_hp_bonus == 10

    def test_mango_max_hp(self):
        """Mango: Gain 14 Max HP."""
        relic = MANGO
        assert relic.max_hp_bonus == 14

    def test_lees_waffle_max_hp(self):
        """Lee's Waffle: Gain 7 Max HP and heal to full."""
        relic = LEES_WAFFLE
        assert relic.max_hp_bonus == 7
        assert "heal to full" in relic.effects[0]


class TestUpgradeRelics:
    """Test relics that upgrade cards."""

    def test_frozen_egg_upgrades_powers(self):
        """Frozen Egg: Powers added to deck are upgraded."""
        relic = FROZEN_EGG
        assert "Whenever you add a Power to your deck, it is Upgraded" in relic.effects

    def test_molten_egg_upgrades_attacks(self):
        """Molten Egg: Attacks added to deck are upgraded."""
        relic = MOLTEN_EGG
        assert "Whenever you add an Attack to your deck, it is Upgraded" in relic.effects

    def test_toxic_egg_upgrades_skills(self):
        """Toxic Egg: Skills added to deck are upgraded."""
        relic = TOXIC_EGG
        assert "Whenever you add a Skill to your deck, it is Upgraded" in relic.effects

    def test_war_paint_upgrades_skills_on_equip(self):
        """War Paint: Upgrade 2 random Skills on pickup."""
        relic = WAR_PAINT
        assert "onEquip: Upgrade 2 random Skills" in relic.effects

    def test_whetstone_upgrades_attacks_on_equip(self):
        """Whetstone: Upgrade 2 random Attacks on pickup."""
        relic = WHETSTONE
        assert "onEquip: Upgrade 2 random Attacks" in relic.effects


class TestSpecialRelics:
    """Test special/event relics."""

    def test_neows_lament_first_combats(self):
        """Neow's Lament: First 3 combats have enemies at 1 HP."""
        relic = NEOWS_LAMENT
        assert relic.counter_type == "uses"
        assert relic.counter_start == 3
        assert "First 3 combats: Enemies have 1 HP" in relic.effects

    def test_cultist_mask_ritual(self):
        """Cultist Headpiece: Gain 1 Ritual at combat start."""
        relic = CULTIST_MASK
        assert "atBattleStart: Gain 1 Ritual (gain 1 Strength each turn)" in relic.effects

    def test_enchiridion_free_power(self):
        """Enchiridion: Add free random Power at combat start."""
        relic = ENCHIRIDION
        assert "atBattleStart: Add random Power to hand, it costs 0 this turn" in relic.effects

    def test_face_of_cleric_max_hp(self):
        """Face of Cleric: Gain 1 Max HP after every combat."""
        relic = FACE_OF_CLERIC
        assert relic.max_hp_bonus == 1
        assert "onVictory (any combat): Gain 1 Max HP" in relic.effects

    def test_gremlin_visage_weak_start(self):
        """Gremlin Visage: Start combat with 1 Weak."""
        relic = GREMLIN_MASK
        assert "atBattleStart: Gain 1 Weak" in relic.effects

    def test_mutagenic_strength_temporary(self):
        """Mutagenic Strength: Gain 3 Strength, lose it at end of turn."""
        relic = MUTAGENIC_STRENGTH
        assert "atBattleStart: Gain 3 Strength. At end of turn, lose 3 Strength" in relic.effects

    def test_necronomicon_double_attack(self):
        """Necronomicon: First 2+ cost Attack each turn plays twice."""
        relic = NECRONOMICON
        assert "First 2+ cost Attack each turn plays twice" in relic.effects[0]


# =============================================================================
# EDGE CASES AND SPECIAL MECHANICS
# =============================================================================

class TestEdgeCases:
    """Test edge cases and special mechanics."""

    def test_emotion_chip_orb_trigger(self):
        """Emotion Chip: Trigger all orb passives when losing HP."""
        relic = EMOTION_CHIP
        assert "wasHPLost (once per combat): Trigger passive of all Orbs" in relic.effects

    def test_gambling_chip_mulligan(self):
        """Gambling Chip: Discard and redraw at combat start."""
        relic = GAMBLING_CHIP
        assert "atBattleStart: Discard any cards, draw that many" in relic.effects

    def test_hand_drill_vulnerable_on_break(self):
        """Hand Drill: Apply Vulnerable when breaking Block."""
        relic = HAND_DRILL
        assert "onBlockBroken: Apply 2 Vulnerable" in relic.effects

    def test_strange_spoon_exhaust_chance(self):
        """Strange Spoon: 50% chance exhausted cards go to discard."""
        relic = STRANGE_SPOON
        assert "50% chance exhausted cards go to discard instead" in relic.effects

    def test_nilrys_codex_end_turn_choice(self):
        """Nilry's Codex: Choose card to add to hand next turn."""
        relic = NILRYS_CODEX
        assert "onPlayerEndTurn: Choose 1 of 3 cards to add to hand next turn" in relic.effects

    def test_specimen_poison_transfer(self):
        """The Specimen: Transfer Poison to random enemy on kill."""
        relic = THE_SPECIMEN
        assert "onMonsterDeath: Transfer enemy's Poison to random enemy" in relic.effects

    def test_ginger_weak_immunity(self):
        """Ginger: Cannot become Weakened."""
        relic = GINGER
        assert "You can no longer become Weakened" in relic.effects

    def test_turnip_frail_immunity(self):
        """Turnip: Cannot become Frail."""
        relic = TURNIP
        assert "You can no longer become Frail" in relic.effects

    def test_preserved_insect_elite_hp(self):
        """Preserved Insect: Elites have 25% less HP."""
        relic = PRESERVED_INSECT
        assert "Elites have 25% less HP" in relic.effects

    def test_pantograph_boss_heal(self):
        """Pantograph: Heal 25 HP at boss combat start."""
        relic = PANTOGRAPH
        assert "atBattleStart (boss): Heal 25 HP" in relic.effects

    def test_sling_elite_strength(self):
        """Sling of Courage: Gain 2 Strength in Elite combats."""
        relic = SLING
        assert "atBattleStart (Elite): Gain 2 Strength" in relic.effects

    def test_odd_mushroom_vulnerable_reduction(self):
        """Odd Mushroom: Vulnerable only increases damage by 25%."""
        relic = ODD_MUSHROOM
        assert "Vulnerable only increases damage by 25% instead of 50%" in relic.effects


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
