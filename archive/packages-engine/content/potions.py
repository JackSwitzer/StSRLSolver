"""
Slay the Spire - Potion System

Complete potion data extracted from decompiled game source.
All values reflect the actual game implementation.

Potion Drop Mechanics:
- Base chance: 40% from normal monsters, elites, and events
- Blizzard mod: +10% per miss, -10% per drop (accumulates)
- White Beast Statue relic: 100% drop chance
- Max 4 rewards per combat caps potion drops at 0%

Potion Rarity Distribution (PotionHelper):
- COMMON: 65%
- UNCOMMON: 25%
- RARE: 10%

Potion Slot Mechanics:
- Base slots: 3
- Ascension 11+: -1 slot (2 base)
- Potion Belt relic: +2 slots

Sacred Bark (Boss Relic):
- Doubles potency of ALL potions via getPotency() method
- Some potions have special Sacred Bark descriptions (card discovery potions)
- Potions with potency=0 or special mechanics are NOT doubled
"""

from enum import Enum, auto
from dataclasses import dataclass, field
from typing import Optional, List


class PotionRarity(Enum):
    """Potion rarity levels from AbstractPotion.PotionRarity"""
    PLACEHOLDER = auto()  # Used for empty potion slots
    COMMON = auto()
    UNCOMMON = auto()
    RARE = auto()


class PotionTargetType(Enum):
    """Target types for potions"""
    NONE = auto()          # Potions with no target (auto-use or passive)
    SELF = auto()          # Target is always the player
    ENEMY = auto()         # Target a single enemy (targetRequired=True)
    ALL_ENEMIES = auto()   # Hits all enemies


class PlayerClass(Enum):
    """Player classes for class-specific potions"""
    IRONCLAD = "IRONCLAD"
    SILENT = "THE_SILENT"
    DEFECT = "DEFECT"
    WATCHER = "WATCHER"
    ALL = "ALL"  # Universal potions


@dataclass
class Potion:
    """
    Represents a potion with all its properties.

    Attributes:
        id: Game internal ID (e.g., "Fire Potion", "BloodPotion")
        name: Display name
        rarity: COMMON, UNCOMMON, or RARE
        potency: Base potency value (doubled by Sacred Bark unless sacred_bark_scales=False)
        target_type: What this potion targets
        player_class: Which class can get this potion (ALL for universal)
        sacred_bark_scales: Whether Sacred Bark doubles the effect
        is_thrown: Whether the potion is thrown at target
        description: Effect description
        special_mechanics: Any special behaviors or notes
    """
    id: str
    name: str
    rarity: PotionRarity
    potency: int
    target_type: PotionTargetType
    player_class: PlayerClass = PlayerClass.ALL
    sacred_bark_scales: bool = True
    is_thrown: bool = False
    description: str = ""
    special_mechanics: List[str] = field(default_factory=list)

    def get_effective_potency(self, has_sacred_bark: bool = False) -> int:
        """Get the effective potency, accounting for Sacred Bark."""
        if has_sacred_bark and self.sacred_bark_scales:
            return self.potency * 2
        return self.potency


# ============================================================================
# COMMON POTIONS (65% drop rate)
# ============================================================================

BLOCK_POTION = Potion(
    id="Block Potion",
    name="Block Potion",
    rarity=PotionRarity.COMMON,
    potency=12,
    target_type=PotionTargetType.SELF,
    description="Gain 12 Block.",
    special_mechanics=["Block gained ignores Dexterity"]
)

DEXTERITY_POTION = Potion(
    id="Dexterity Potion",
    name="Dexterity Potion",
    rarity=PotionRarity.COMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    description="Gain 2 Dexterity.",
    special_mechanics=["Permanent for combat duration"]
)

ENERGY_POTION = Potion(
    id="Energy Potion",
    name="Energy Potion",
    rarity=PotionRarity.COMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    description="Gain 2 Energy.",
)

EXPLOSIVE_POTION = Potion(
    id="Explosive Potion",
    name="Explosive Potion",
    rarity=PotionRarity.COMMON,
    potency=10,
    target_type=PotionTargetType.ALL_ENEMIES,
    is_thrown=True,
    description="Deal 10 damage to ALL enemies.",
    special_mechanics=["Ignores player strength", "Uses THORNS damage type"]
)

FIRE_POTION = Potion(
    id="Fire Potion",
    name="Fire Potion",
    rarity=PotionRarity.COMMON,
    potency=20,
    target_type=PotionTargetType.ENEMY,
    is_thrown=True,
    description="Deal 20 damage to target enemy.",
    special_mechanics=["Uses THORNS damage type", "Affected by enemy Vulnerable/Weak"]
)

STRENGTH_POTION = Potion(
    id="Strength Potion",
    name="Strength Potion",
    rarity=PotionRarity.COMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    description="Gain 2 Strength.",
    special_mechanics=["Permanent for combat duration"]
)

SWIFT_POTION = Potion(
    id="Swift Potion",
    name="Swift Potion",
    rarity=PotionRarity.COMMON,
    potency=3,
    target_type=PotionTargetType.SELF,
    description="Draw 3 cards.",
)

WEAK_POTION = Potion(
    id="Weak Potion",
    name="Weak Potion",
    rarity=PotionRarity.COMMON,
    potency=3,
    target_type=PotionTargetType.ENEMY,
    is_thrown=True,
    description="Apply 3 Weak.",
)

FEAR_POTION = Potion(
    id="FearPotion",
    name="Fear Potion",
    rarity=PotionRarity.COMMON,
    potency=3,
    target_type=PotionTargetType.ENEMY,
    is_thrown=True,
    description="Apply 3 Vulnerable.",
)

ATTACK_POTION = Potion(
    id="AttackPotion",
    name="Attack Potion",
    rarity=PotionRarity.COMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Add a random Attack card to your hand. It costs 0 this turn.",
    special_mechanics=[
        "Uses Discovery action (choose from 3 options)",
        "With Sacred Bark: Choose 1 of 3 cards, add 2 copies to hand"
    ]
)

SKILL_POTION = Potion(
    id="SkillPotion",
    name="Skill Potion",
    rarity=PotionRarity.COMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Add a random Skill card to your hand. It costs 0 this turn.",
    special_mechanics=[
        "Uses Discovery action (choose from 3 options)",
        "With Sacred Bark: Choose 1 of 3 cards, add 2 copies to hand"
    ]
)

POWER_POTION = Potion(
    id="PowerPotion",
    name="Power Potion",
    rarity=PotionRarity.COMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Add a random Power card to your hand. It costs 0 this turn.",
    special_mechanics=[
        "Uses Discovery action (choose from 3 options)",
        "With Sacred Bark: Choose 1 of 3 cards, add 2 copies to hand"
    ]
)

COLORLESS_POTION = Potion(
    id="ColorlessPotion",
    name="Colorless Potion",
    rarity=PotionRarity.COMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Add a random Colorless card to your hand. It costs 0 this turn.",
    special_mechanics=[
        "Uses Discovery action (choose from 3 options)",
        "With Sacred Bark: Choose 1 of 3 cards, add 2 copies to hand"
    ]
)

SPEED_POTION = Potion(
    id="SpeedPotion",
    name="Speed Potion",
    rarity=PotionRarity.COMMON,
    potency=5,
    target_type=PotionTargetType.SELF,
    description="Gain 5 Dexterity. At the end of turn, lose 5 Dexterity.",
    special_mechanics=["Dexterity is temporary (LoseDexterityPower)"]
)

STEROID_POTION = Potion(
    id="SteroidPotion",
    name="Flex Potion",
    rarity=PotionRarity.COMMON,
    potency=5,
    target_type=PotionTargetType.SELF,
    description="Gain 5 Strength. At the end of turn, lose 5 Strength.",
    special_mechanics=["Strength is temporary (LoseStrengthPower)"]
)

BLESSING_OF_THE_FORGE = Potion(
    id="BlessingOfTheForge",
    name="Blessing of the Forge",
    rarity=PotionRarity.COMMON,
    potency=0,
    target_type=PotionTargetType.SELF,
    sacred_bark_scales=False,
    description="Upgrade ALL cards in your hand for the rest of combat.",
    special_mechanics=["Uses ArmamentsAction(true)", "Sacred Bark has no effect"]
)

# Class-specific COMMON potions

BLOOD_POTION = Potion(
    id="BloodPotion",
    name="Blood Potion",
    rarity=PotionRarity.COMMON,
    potency=20,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.IRONCLAD,
    description="Heal for 20% of your Max HP.",
    special_mechanics=[
        "Can be used outside combat",
        "Percentage-based healing",
        "With Sacred Bark: heals 40%"
    ]
)

POISON_POTION = Potion(
    id="Poison Potion",
    name="Poison Potion",
    rarity=PotionRarity.COMMON,
    potency=6,
    target_type=PotionTargetType.ENEMY,
    player_class=PlayerClass.SILENT,
    is_thrown=True,
    description="Apply 6 Poison to target enemy.",
)

FOCUS_POTION = Potion(
    id="FocusPotion",
    name="Focus Potion",
    rarity=PotionRarity.COMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.DEFECT,
    description="Gain 2 Focus.",
    special_mechanics=["Permanent for combat duration"]
)

BOTTLED_MIRACLE = Potion(
    id="BottledMiracle",
    name="Bottled Miracle",
    rarity=PotionRarity.COMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.WATCHER,
    description="Add 2 Miracle cards to your hand.",
    special_mechanics=["Miracles give 1 energy and Retain"]
)


# ============================================================================
# UNCOMMON POTIONS (25% drop rate)
# ============================================================================

ANCIENT_POTION = Potion(
    id="Ancient Potion",
    name="Ancient Potion",
    rarity=PotionRarity.UNCOMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Gain 1 Artifact.",
    special_mechanics=["Negates next debuff"]
)

REGEN_POTION = Potion(
    id="Regen Potion",
    name="Regeneration Potion",
    rarity=PotionRarity.UNCOMMON,
    potency=5,
    target_type=PotionTargetType.SELF,
    description="Gain 5 Regeneration.",
    special_mechanics=["Heals 5 HP at end of each turn", "Decrements each turn"]
)

GAMBLERS_BREW = Potion(
    id="GamblersBrew",
    name="Gambler's Brew",
    rarity=PotionRarity.UNCOMMON,
    potency=0,
    target_type=PotionTargetType.SELF,
    sacred_bark_scales=False,
    description="Discard any number of cards, then draw that many.",
    special_mechanics=["Uses GamblingChipAction", "Sacred Bark has no effect"]
)

LIQUID_BRONZE = Potion(
    id="LiquidBronze",
    name="Liquid Bronze",
    rarity=PotionRarity.UNCOMMON,
    potency=3,
    target_type=PotionTargetType.SELF,
    description="Gain 3 Thorns.",
    special_mechanics=["Permanent for combat duration"]
)

LIQUID_MEMORIES = Potion(
    id="LiquidMemories",
    name="Liquid Memories",
    rarity=PotionRarity.UNCOMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Choose a card in your discard pile and return it to your hand. It costs 0 this turn.",
    special_mechanics=["With Sacred Bark: return 2 cards"]
)

ESSENCE_OF_STEEL = Potion(
    id="EssenceOfSteel",
    name="Essence of Steel",
    rarity=PotionRarity.UNCOMMON,
    potency=4,
    target_type=PotionTargetType.SELF,
    description="Gain 4 Plated Armor.",
    special_mechanics=["Permanent for combat until attacked unblocked"]
)

DUPLICATION_POTION = Potion(
    id="DuplicationPotion",
    name="Duplication Potion",
    rarity=PotionRarity.UNCOMMON,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="This turn, your next card is played twice.",
    special_mechanics=[
        "Uses DuplicationPower",
        "With Sacred Bark: Next 2 cards played twice"
    ]
)

DISTILLED_CHAOS = Potion(
    id="DistilledChaos",
    name="Distilled Chaos",
    rarity=PotionRarity.UNCOMMON,
    potency=3,
    target_type=PotionTargetType.SELF,
    description="Play the top 3 cards of your draw pile.",
    special_mechanics=[
        "Targets random enemy for each card",
        "Rainbow visual effect"
    ]
)

# Class-specific UNCOMMON potions

ELIXIR = Potion(
    id="ElixirPotion",
    name="Elixir",
    rarity=PotionRarity.UNCOMMON,
    potency=0,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.IRONCLAD,
    sacred_bark_scales=False,
    description="Exhaust any number of cards in your hand.",
    special_mechanics=["Sacred Bark has no effect"]
)

CUNNING_POTION = Potion(
    id="CunningPotion",
    name="Cunning Potion",
    rarity=PotionRarity.UNCOMMON,
    potency=3,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.SILENT,
    description="Add 3 Upgraded Shivs to your hand.",
    special_mechanics=["Shivs are pre-upgraded (deal 6 damage)"]
)

POTION_OF_CAPACITY = Potion(
    id="PotionOfCapacity",
    name="Potion of Capacity",
    rarity=PotionRarity.UNCOMMON,
    potency=2,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.DEFECT,
    description="Gain 2 Orb slots.",
    special_mechanics=["Permanent for combat duration"]
)

STANCE_POTION = Potion(
    id="StancePotion",
    name="Stance Potion",
    rarity=PotionRarity.UNCOMMON,
    potency=0,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.WATCHER,
    sacred_bark_scales=False,
    description="Enter Calm or Wrath.",
    special_mechanics=["Choose between Calm or Wrath", "Sacred Bark has no effect"]
)


# ============================================================================
# RARE POTIONS (10% drop rate)
# ============================================================================

CULTIST_POTION = Potion(
    id="CultistPotion",
    name="Cultist Potion",
    rarity=PotionRarity.RARE,
    potency=1,
    target_type=PotionTargetType.SELF,
    description="Gain 1 Ritual. At the end of each turn, gain 1 Strength.",
    special_mechanics=[
        "Plays 'CAW!' sound effect",
        "Permanent ritual stacking"
    ]
)

FRUIT_JUICE = Potion(
    id="Fruit Juice",
    name="Fruit Juice",
    rarity=PotionRarity.RARE,
    potency=5,
    target_type=PotionTargetType.SELF,
    description="Gain 5 Max HP.",
    special_mechanics=[
        "Can be used outside combat",
        "Permanent Max HP increase",
        "Excluded from Alchemize's 'limited' pool"
    ]
)

SNECKO_OIL = Potion(
    id="SneckoOil",
    name="Snecko Oil",
    rarity=PotionRarity.RARE,
    potency=5,
    target_type=PotionTargetType.SELF,
    description="Draw 5 cards. Randomize the costs of all cards in your hand.",
    special_mechanics=["RandomizeHandCostAction sets costs 0-3"]
)

FAIRY_POTION = Potion(
    id="FairyPotion",
    name="Fairy in a Bottle",
    rarity=PotionRarity.RARE,
    potency=30,
    target_type=PotionTargetType.NONE,  # Triggers on death
    sacred_bark_scales=True,
    description="When you would die, heal to 30% of your Max HP instead and discard this potion.",
    special_mechanics=[
        "Triggers automatically on death via onPlayerDeath()",
        "Cannot be manually used (canUse returns false)",
        "With Sacred Bark: heals to 60%"
    ]
)

SMOKE_BOMB = Potion(
    id="SmokeBomb",
    name="Smoke Bomb",
    rarity=PotionRarity.RARE,
    potency=0,
    target_type=PotionTargetType.SELF,
    is_thrown=True,
    sacred_bark_scales=False,
    description="Escape from a non-boss combat.",
    special_mechanics=[
        "Cannot be used vs bosses",
        "Cannot be used if enemy has BackAttack power",
        "Sets room.smoked = true",
        "Sacred Bark has no effect"
    ]
)

ENTROPIC_BREW = Potion(
    id="EntropicBrew",
    name="Entropic Brew",
    rarity=PotionRarity.RARE,
    potency=3,  # Default to 3, but actually uses player.potionSlots
    target_type=PotionTargetType.SELF,
    sacred_bark_scales=False,
    description="Fill all your empty potion slots with random potions.",
    special_mechanics=[
        "Potency = number of potion slots",
        "Can be used outside combat",
        "Does nothing if Sozu is equipped (relics flash)",
        "Sacred Bark has no effect on number of potions"
    ]
)

# Class-specific RARE potions

HEART_OF_IRON = Potion(
    id="HeartOfIron",
    name="Heart of Iron",
    rarity=PotionRarity.RARE,
    potency=6,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.IRONCLAD,
    description="Gain 6 Metallicize.",
    special_mechanics=["Permanent for combat duration"]
)

GHOST_IN_A_JAR = Potion(
    id="GhostInAJar",
    name="Ghost In A Jar",
    rarity=PotionRarity.RARE,
    potency=1,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.SILENT,
    description="Gain 1 Intangible.",
    special_mechanics=["Reduces ALL damage to 1 for 1 turn"]
)

ESSENCE_OF_DARKNESS = Potion(
    id="EssenceOfDarkness",
    name="Essence of Darkness",
    rarity=PotionRarity.RARE,
    potency=1,
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.DEFECT,
    description="Channel 1 Dark for each Orb slot.",
    special_mechanics=[
        "Channels (potency * orb_slots) Dark orbs",
        "With Sacred Bark: channels 2 Dark per slot"
    ]
)

AMBROSIA = Potion(
    id="Ambrosia",
    name="Ambrosia",
    rarity=PotionRarity.RARE,
    potency=2,  # Unused in effect, but present in code
    target_type=PotionTargetType.SELF,
    player_class=PlayerClass.WATCHER,
    sacred_bark_scales=False,  # Effect doesn't use potency
    description="Enter Divinity.",
    special_mechanics=[
        "Immediately enter Divinity stance",
        "Triple damage, +3 energy",
        "Returns to Neutral at end of turn",
        "Sacred Bark has no effect"
    ]
)


# ============================================================================
# POTION COLLECTIONS
# ============================================================================

# All potions by ID
ALL_POTIONS = {
    # Common
    BLOCK_POTION.id: BLOCK_POTION,
    DEXTERITY_POTION.id: DEXTERITY_POTION,
    ENERGY_POTION.id: ENERGY_POTION,
    EXPLOSIVE_POTION.id: EXPLOSIVE_POTION,
    FIRE_POTION.id: FIRE_POTION,
    STRENGTH_POTION.id: STRENGTH_POTION,
    SWIFT_POTION.id: SWIFT_POTION,
    WEAK_POTION.id: WEAK_POTION,
    FEAR_POTION.id: FEAR_POTION,
    ATTACK_POTION.id: ATTACK_POTION,
    SKILL_POTION.id: SKILL_POTION,
    POWER_POTION.id: POWER_POTION,
    COLORLESS_POTION.id: COLORLESS_POTION,
    SPEED_POTION.id: SPEED_POTION,
    STEROID_POTION.id: STEROID_POTION,
    BLESSING_OF_THE_FORGE.id: BLESSING_OF_THE_FORGE,
    BLOOD_POTION.id: BLOOD_POTION,
    POISON_POTION.id: POISON_POTION,
    FOCUS_POTION.id: FOCUS_POTION,
    BOTTLED_MIRACLE.id: BOTTLED_MIRACLE,
    # Uncommon
    ANCIENT_POTION.id: ANCIENT_POTION,
    REGEN_POTION.id: REGEN_POTION,
    GAMBLERS_BREW.id: GAMBLERS_BREW,
    LIQUID_BRONZE.id: LIQUID_BRONZE,
    LIQUID_MEMORIES.id: LIQUID_MEMORIES,
    ESSENCE_OF_STEEL.id: ESSENCE_OF_STEEL,
    DUPLICATION_POTION.id: DUPLICATION_POTION,
    DISTILLED_CHAOS.id: DISTILLED_CHAOS,
    ELIXIR.id: ELIXIR,
    CUNNING_POTION.id: CUNNING_POTION,
    POTION_OF_CAPACITY.id: POTION_OF_CAPACITY,
    STANCE_POTION.id: STANCE_POTION,
    # Rare
    CULTIST_POTION.id: CULTIST_POTION,
    FRUIT_JUICE.id: FRUIT_JUICE,
    SNECKO_OIL.id: SNECKO_OIL,
    FAIRY_POTION.id: FAIRY_POTION,
    SMOKE_BOMB.id: SMOKE_BOMB,
    ENTROPIC_BREW.id: ENTROPIC_BREW,
    HEART_OF_IRON.id: HEART_OF_IRON,
    GHOST_IN_A_JAR.id: GHOST_IN_A_JAR,
    ESSENCE_OF_DARKNESS.id: ESSENCE_OF_DARKNESS,
    AMBROSIA.id: AMBROSIA,
}

# Potions by rarity
COMMON_POTIONS = [p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.COMMON]
UNCOMMON_POTIONS = [p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.UNCOMMON]
RARE_POTIONS = [p for p in ALL_POTIONS.values() if p.rarity == PotionRarity.RARE]

# Universal potions (available to all classes)
UNIVERSAL_POTIONS = [p for p in ALL_POTIONS.values() if p.player_class == PlayerClass.ALL]

# Class-specific potions
IRONCLAD_POTIONS = [p for p in ALL_POTIONS.values() if p.player_class == PlayerClass.IRONCLAD]
SILENT_POTIONS = [p for p in ALL_POTIONS.values() if p.player_class == PlayerClass.SILENT]
DEFECT_POTIONS = [p for p in ALL_POTIONS.values() if p.player_class == PlayerClass.DEFECT]
WATCHER_POTIONS = [p for p in ALL_POTIONS.values() if p.player_class == PlayerClass.WATCHER]


# ============================================================================
# POTION DROP MECHANICS
# ============================================================================

# Rarity drop chances (from PotionHelper)
POTION_COMMON_CHANCE = 65  # 65%
POTION_UNCOMMON_CHANCE = 25  # 25%
POTION_RARE_CHANCE = 10  # 10% (100 - 65 - 25)

# Base potion drop chance from combat
BASE_POTION_DROP_CHANCE = 40  # 40% base

# Blizzard potion modifier (accumulates)
# +10% per miss, -10% per drop
BLIZZARD_MOD_STEP = 10


def get_potion_pool(player_class: PlayerClass) -> List[Potion]:
    """
    Get the potion pool for a given player class.
    Includes universal potions plus class-specific potions.

    Args:
        player_class: The player's class

    Returns:
        List of all potions available to that class
    """
    pool = list(UNIVERSAL_POTIONS)

    if player_class == PlayerClass.IRONCLAD:
        pool.extend(IRONCLAD_POTIONS)
    elif player_class == PlayerClass.SILENT:
        pool.extend(SILENT_POTIONS)
    elif player_class == PlayerClass.DEFECT:
        pool.extend(DEFECT_POTIONS)
    elif player_class == PlayerClass.WATCHER:
        pool.extend(WATCHER_POTIONS)

    return pool


def get_potion_by_id(potion_id: str) -> Optional[Potion]:
    """Get a potion by its game ID."""
    return ALL_POTIONS.get(potion_id)


def calculate_potion_slots(ascension_level: int = 0, has_potion_belt: bool = False) -> int:
    """
    Calculate the number of potion slots.

    Args:
        ascension_level: Current ascension level (0-20)
        has_potion_belt: Whether player has Potion Belt relic

    Returns:
        Number of potion slots
    """
    slots = 3
    if ascension_level >= 11:
        slots -= 1  # A11+ reduces by 1
    if has_potion_belt:
        slots += 2  # Potion Belt adds 2
    return slots


def calculate_drop_chance(
    room_type: str,
    blizzard_mod: int = 0,
    has_white_beast_statue: bool = False,
    current_rewards: int = 0
) -> int:
    """
    Calculate the chance for a potion to drop.

    Args:
        room_type: "monster", "elite", or "event"
        blizzard_mod: Current blizzard modifier (-10 per drop, +10 per miss)
        has_white_beast_statue: Whether player has White Beast Statue
        current_rewards: Number of rewards already in the list

    Returns:
        Drop chance as a percentage (0-100)
    """
    if has_white_beast_statue:
        return 100

    if current_rewards >= 4:
        return 0

    if room_type in ("monster", "elite", "event"):
        return max(0, BASE_POTION_DROP_CHANCE + blizzard_mod)

    return 0


# ============================================================================
# SUMMARY STATISTICS
# ============================================================================

def print_potion_summary():
    """Print a summary of all potions."""
    print(f"Total Potions: {len(ALL_POTIONS)}")
    print(f"  Common: {len(COMMON_POTIONS)}")
    print(f"  Uncommon: {len(UNCOMMON_POTIONS)}")
    print(f"  Rare: {len(RARE_POTIONS)}")
    print(f"\nUniversal: {len(UNIVERSAL_POTIONS)}")
    print(f"Ironclad-specific: {len(IRONCLAD_POTIONS)}")
    print(f"Silent-specific: {len(SILENT_POTIONS)}")
    print(f"Defect-specific: {len(DEFECT_POTIONS)}")
    print(f"Watcher-specific: {len(WATCHER_POTIONS)}")

    # Potions where Sacred Bark has no effect
    no_bark = [p for p in ALL_POTIONS.values() if not p.sacred_bark_scales]
    print(f"\nPotions unaffected by Sacred Bark: {len(no_bark)}")
    for p in no_bark:
        print(f"  - {p.name}")


if __name__ == "__main__":
    print_potion_summary()
