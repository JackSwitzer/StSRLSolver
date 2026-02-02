"""
Card Library Order for Java Parity

This module defines the exact insertion order of cards into Java's CardLibrary,
then uses JavaHashMap simulation to compute the correct iteration order.

The card pool order in the game depends on:
1. The order cards are added to CardLibrary.cards (a HashMap)
2. Java HashMap's internal iteration order (bucket-based, not insertion order)

This module provides the correct pool orders for seed-deterministic reward generation.
"""

from typing import List, Dict, Set
import os
import sys
import importlib.util

# Handle imports flexibly - works as module, direct execution, and via _load_module
_this_dir = os.path.dirname(os.path.abspath(__file__))

def _load_java_hashmap():
    """Load java_hashmap module from same directory."""
    spec = importlib.util.spec_from_file_location(
        "java_hashmap",
        os.path.join(_this_dir, "java_hashmap.py")
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

_java_hashmap = _load_java_hashmap()
JavaHashMap = _java_hashmap.JavaHashMap
get_java_iteration_order = _java_hashmap.get_java_iteration_order

# =============================================================================
# CARD IDs BY COLOR IN INSERTION ORDER
# Extracted from CardLibrary.java add*Cards() methods
# =============================================================================

# Ironclad (Red) cards - from addRedCards()
RED_CARD_IDS = [
    "Anger", "Armaments", "Barricade", "Bash", "BattleTrance", "Berserk",
    "BloodForBlood", "Bloodletting", "Bludgeon", "BodySlam", "Brutality",
    "BurningPact", "Carnage", "Clash", "Cleave", "Clothesline", "Combust",
    "Corruption", "DarkEmbrace", "Defend_R", "DemonForm", "Disarm", "DoubleTap",
    "Dropkick", "DualWield", "Entrench", "Evolve", "Exhume", "Feed", "FeelNoPain",
    "FiendFire", "FireBreathing", "FlameBarrier", "Flex", "GhostlyArmor", "Havoc",
    "Headbutt", "HeavyBlade", "Hemokinesis", "Immolate", "Impervious", "InfernalBlade",
    "Inflame", "Intimidate", "IronWave", "Juggernaut", "LimitBreak", "Metallicize",
    "Offering", "PerfectedStrike", "PommelStrike", "PowerThrough", "Pummel", "Rage",
    "Rampage", "Reaper", "RecklessCharge", "Rupture", "SearingBlow", "SecondWind",
    "SeeingRed", "Sentinel", "SeverSoul", "Shockwave", "ShrugItOff", "SpotWeakness",
    "Strike_R", "SwordBoomerang", "ThunderClap", "TrueGrit", "TwinStrike", "Uppercut",
    "Warcry", "Whirlwind", "WildStrike",
]

# Silent (Green) cards - from addGreenCards()
GREEN_CARD_IDS = [
    "Accuracy", "Acrobatics", "Adrenaline", "AfterImage", "Alchemize", "AllOutAttack",
    "AThousandCuts", "Backflip", "Backstab", "Bane", "BladeDance", "Blur",
    "BouncingFlask", "BulletTime", "Burst", "CalculatedGamble", "Caltrops", "Catalyst",
    "Choke", "CloakAndDagger", "Concentrate", "CorpseExplosion", "CripplingPoison",
    "DaggerSpray", "DaggerThrow", "Dash", "DeadlyPoison", "Defend_G", "Deflect",
    "DieDieDie", "Distraction", "DodgeAndRoll", "Doppelganger", "EndlessAgony", "Envenom",
    "EscapePlan", "Eviscerate", "Expertise", "Finisher", "Flechettes", "FlyingKnee",
    "Footwork", "GlassKnife", "GrandFinale", "HeelHook", "InfiniteBlades", "LegSweep",
    "Malaise", "MasterfulStab", "Neutralize", "Nightmare", "NoxiousFumes", "Outmaneuver",
    "PhantasmalKiller", "PiercingWail", "PoisonedStab", "Predator", "Prepared", "QuickSlash",
    "Reflex", "RiddleWithHoles", "Setup", "Skewer", "Slice", "StormOfSteel", "Strike_G",
    "SuckerPunch", "Survivor", "Tactician", "Terror", "ToolsOfTheTrade", "SneakyStrike",
    "Unload", "WellLaidPlans", "WraithForm",
]

# Defect (Blue) cards - from addBlueCards()
BLUE_CARD_IDS = [
    "Aggregate", "AllForOne", "Amplify", "AutoShields", "BallLightning", "Barrage",
    "BeamCell", "BiasedCognition", "Blizzard", "BootSequence", "Buffer", "Capacitor",
    "Chaos", "Chill", "Claw", "ColdSnap", "CompileDriver", "ConserveBattery", "Consume",
    "Coolheaded", "CoreSurge", "CreativeAI", "Darkness", "Defend_B", "Defragment",
    "DoomAndGloom", "DoubleEnergy", "Dualcast", "EchoForm", "Electrodynamics", "Fission",
    "ForceField", "FTL", "Fusion", "GeneticAlgorithm", "Glacier", "GoForTheEyes",
    "Heatsinks", "HelloWorld", "Hologram", "Hyperbeam", "Leap", "LockOn", "Loop",
    "MachineLearning", "Melter", "MeteorStrike", "MultiCast", "Overclock", "Rainbow",
    "Reboot", "Rebound", "Recursion", "Recycle", "ReinforcedBody", "Reprogram",
    "RipAndTear", "Scrape", "Seek", "SelfRepair", "Skim", "Stack", "StaticDischarge",
    "SteamBarrier", "Storm", "Streamline", "Strike_B", "Sunder", "SweepingBeam", "Tempest",
    "ThunderStrike", "Turbo", "Equilibrium", "WhiteNoise", "Zap",
]

# Watcher (Purple) cards - from addPurpleCards() in CardLibrary.java
# IN EXACT INSERTION ORDER from the Java source (lines 685-761)
# IMPORTANT: Uses actual Java cardID values, NOT class names!
# Key ID mappings (where class name differs from cardID):
#   - SimmeringFury.java -> ID "Vengeance"
#   - Defend_Watcher.java -> ID "Defend_P"
#   - Strike_Purple.java -> ID "Strike_P"
#   - Tranquility.java -> ID "ClearTheMind"
#   - Foresight.java -> ID "Wireheading"
#   - PressurePoints.java -> ID "PathToVictory"
#   - Rushdown.java -> ID "Adaptation"
#   - Fasting.java -> ID "Fasting2"
# Note: Discipline.java and Unraveling.java exist but are NOT in CardLibrary
PURPLE_CARD_IDS = [
    "Alpha",              # Alpha.java
    "BattleHymn",         # BattleHymn.java
    "Blasphemy",          # Blasphemy.java
    "BowlingBash",        # BowlingBash.java
    "Brilliance",         # Brilliance.java
    "CarveReality",       # CarveReality.java
    "Collect",            # Collect.java
    "Conclude",           # Conclude.java
    "ConjureBlade",       # ConjureBlade.java
    "Consecrate",         # Consecrate.java
    "Crescendo",          # Crescendo.java
    "CrushJoints",        # CrushJoints.java
    "CutThroughFate",     # CutThroughFate.java
    "DeceiveReality",     # DeceiveReality.java
    "Defend_P",           # Defend_Watcher.java -> ID "Defend_P"
    "DeusExMachina",      # DeusExMachina.java
    "DevaForm",           # DevaForm.java
    "Devotion",           # Devotion.java
    "EmptyBody",          # EmptyBody.java
    "EmptyFist",          # EmptyFist.java
    "EmptyMind",          # EmptyMind.java
    "Eruption",           # Eruption.java
    "Establishment",      # Establishment.java
    "Evaluate",           # Evaluate.java
    "Fasting2",           # Fasting.java -> ID "Fasting2"
    "FearNoEvil",         # FearNoEvil.java
    "FlurryOfBlows",      # FlurryOfBlows.java
    "FlyingSleeves",      # FlyingSleeves.java
    "FollowUp",           # FollowUp.java
    "ForeignInfluence",   # ForeignInfluence.java
    "Wireheading",        # Foresight.java -> ID "Wireheading"
    "Halt",               # Halt.java
    "Indignation",        # Indignation.java
    "InnerPeace",         # InnerPeace.java
    "Judgement",          # Judgement.java
    "JustLucky",          # JustLucky.java
    "LessonLearned",      # LessonLearned.java
    "LikeWater",          # LikeWater.java
    "MasterReality",      # MasterReality.java
    "Meditate",           # Meditate.java
    "MentalFortress",     # MentalFortress.java
    "Nirvana",            # Nirvana.java
    "Omniscience",        # Omniscience.java
    "Perseverance",       # Perseverance.java
    "Pray",               # Pray.java
    "PathToVictory",      # PressurePoints.java -> ID "PathToVictory"
    "Prostrate",          # Prostrate.java
    "Protect",            # Protect.java
    "Ragnarok",           # Ragnarok.java
    "ReachHeaven",        # ReachHeaven.java
    "Adaptation",         # Rushdown.java -> ID "Adaptation"
    "Sanctity",           # Sanctity.java
    "SandsOfTime",        # SandsOfTime.java
    "SashWhip",           # SashWhip.java
    "Scrawl",             # Scrawl.java
    "SignatureMove",      # SignatureMove.java
    "Vengeance",          # SimmeringFury.java -> ID "Vengeance"
    "SpiritShield",       # SpiritShield.java
    "Strike_P",           # Strike_Purple.java -> ID "Strike_P"
    "Study",              # Study.java
    "Swivel",             # Swivel.java
    "TalkToTheHand",      # TalkToTheHand.java
    "Tantrum",            # Tantrum.java
    "ThirdEye",           # ThirdEye.java
    "ClearTheMind",       # Tranquility.java -> ID "ClearTheMind"
    "Vault",              # Vault.java
    "Vigilance",          # Vigilance.java
    "Wallop",             # Wallop.java
    "WaveOfTheHand",      # WaveOfTheHand.java
    "Weave",              # Weave.java
    "WheelKick",          # WheelKick.java
    "WindmillStrike",     # WindmillStrike.java
    "Wish",               # Wish.java
    "Worship",            # Worship.java
    "WreathOfFlame",      # WreathOfFlame.java
]

# Colorless cards - from addColorlessCards()
COLORLESS_CARD_IDS = [
    "Apotheosis", "BandageUp", "Blind", "Chrysalis", "DarkShackles", "DeepBreath",
    "Discovery", "DramaticEntrance", "Enlightenment", "Finesse", "FlashOfSteel",
    "Forethought", "GoodInstincts", "HandOfGreed", "Impatience", "JackOfAllTrades",
    "Madness", "Magnetism", "MasterOfStrategy", "Mayhem", "Metamorphosis", "MindBlast",
    "Panacea", "Panache", "PanicButton", "Purity", "SadisticNature", "SecretTechnique",
    "SecretWeapon", "SwiftStrike", "TheBomb", "ThinkingAhead", "Transmutation", "Trip",
    "Violence",
    # Status cards (also in colorless for CardLibrary purposes)
    "Burn", "Dazed", "Slimed", "Void", "Wound",
    # Special cards
    "Apparition", "Beta", "Bite", "JAX", "Miracle", "Omega", "Ritual", "Safety",
    "Shiv", "Smite", "ThroughViolence",
]

# Curse cards - from addCurseCards()
CURSE_CARD_IDS = [
    "AscendersBane", "CurseOfTheBell", "Clumsy", "Decay", "Doubt", "Injury",
    "Necronomicurse", "Normality", "Pain", "Parasite", "Pride", "Regret", "Shame", "Writhe",
]

# All card IDs in CardLibrary insertion order
ALL_CARD_IDS = (
    RED_CARD_IDS +
    GREEN_CARD_IDS +
    BLUE_CARD_IDS +
    PURPLE_CARD_IDS +
    COLORLESS_CARD_IDS +
    CURSE_CARD_IDS
)


# =============================================================================
# CARD POOL ORDER COMPUTATION
# =============================================================================

def _compute_card_library_iteration_order() -> List[str]:
    """
    Compute the iteration order of CardLibrary.cards HashMap.

    This simulates inserting all cards into a Java HashMap in the same
    order as CardLibrary.initialize(), then getting the iteration order.
    """
    return get_java_iteration_order(ALL_CARD_IDS)


def get_watcher_pool_order(card_ids_set: Set[str] = None) -> List[str]:
    """
    Get Watcher card IDs in Java HashMap iteration order.

    Args:
        card_ids_set: Optional set of card IDs to filter by.
                     If None, returns all Watcher cards.

    Returns:
        List of Watcher card IDs in HashMap iteration order.
    """
    if card_ids_set is None:
        card_ids_set = set(PURPLE_CARD_IDS)

    # Get full iteration order
    iteration_order = _compute_card_library_iteration_order()

    # Filter to just Watcher cards
    return [cid for cid in iteration_order if cid in card_ids_set]


# Cache the computed orders for performance
_WATCHER_POOL_ORDER_CACHE: List[str] = None


def get_cached_watcher_pool_order() -> List[str]:
    """Get cached Watcher pool order (computed once)."""
    global _WATCHER_POOL_ORDER_CACHE
    if _WATCHER_POOL_ORDER_CACHE is None:
        _WATCHER_POOL_ORDER_CACHE = get_watcher_pool_order()
    return _WATCHER_POOL_ORDER_CACHE


def get_card_pool_index(card_id: str) -> int:
    """
    Get the index of a card in the Watcher pool iteration order.

    Returns -1 if card not found.
    """
    order = get_cached_watcher_pool_order()
    try:
        return order.index(card_id)
    except ValueError:
        return -1


# =============================================================================
# RARITY-SPECIFIC POOL ORDERING
# =============================================================================

# Card ID to rarity mapping for Watcher cards
# Uses ACTUAL Java card IDs (not class names)
# Extracted from decompiled Java source
WATCHER_CARD_RARITIES = {
    # BASIC cards (not in reward pools)
    "Defend_P": "BASIC",
    "Strike_P": "BASIC",
    "Eruption": "BASIC",
    "Vigilance": "BASIC",

    # COMMON cards (19 total)
    "BowlingBash": "COMMON",
    "ClearTheMind": "COMMON",  # Tranquility.java
    "Consecrate": "COMMON",
    "Crescendo": "COMMON",
    "CrushJoints": "COMMON",
    "CutThroughFate": "COMMON",
    "EmptyBody": "COMMON",
    "EmptyFist": "COMMON",
    "Evaluate": "COMMON",
    "FlurryOfBlows": "COMMON",
    "FlyingSleeves": "COMMON",
    "FollowUp": "COMMON",
    "Halt": "COMMON",
    "JustLucky": "COMMON",
    "PathToVictory": "COMMON",  # PressurePoints.java
    "Prostrate": "COMMON",
    "Protect": "COMMON",
    "SashWhip": "COMMON",
    "ThirdEye": "COMMON",

    # UNCOMMON cards (36 total)
    "Adaptation": "UNCOMMON",  # Rushdown.java
    "BattleHymn": "UNCOMMON",
    "CarveReality": "UNCOMMON",
    "Collect": "UNCOMMON",
    "Conclude": "UNCOMMON",
    "DeceiveReality": "UNCOMMON",
    "EmptyMind": "UNCOMMON",
    "Fasting2": "UNCOMMON",  # Fasting.java
    "FearNoEvil": "UNCOMMON",
    "ForeignInfluence": "UNCOMMON",
    "Wireheading": "UNCOMMON",  # Foresight.java
    "Indignation": "UNCOMMON",
    "InnerPeace": "UNCOMMON",
    "LikeWater": "UNCOMMON",
    "Meditate": "UNCOMMON",
    "MentalFortress": "UNCOMMON",
    "Nirvana": "UNCOMMON",
    "Perseverance": "UNCOMMON",
    "Pray": "UNCOMMON",
    "ReachHeaven": "UNCOMMON",
    "Sanctity": "UNCOMMON",
    "SandsOfTime": "UNCOMMON",
    "SignatureMove": "UNCOMMON",
    "Study": "UNCOMMON",
    "Swivel": "UNCOMMON",
    "TalkToTheHand": "UNCOMMON",
    "Tantrum": "UNCOMMON",
    "Vengeance": "UNCOMMON",  # SimmeringFury.java
    "Wallop": "UNCOMMON",
    "WaveOfTheHand": "UNCOMMON",
    "Weave": "UNCOMMON",
    "WheelKick": "UNCOMMON",
    "WindmillStrike": "UNCOMMON",
    "Worship": "UNCOMMON",
    "WreathOfFlame": "UNCOMMON",

    # RARE cards (17 total) - Discipline and Unraveling exist as files but NOT in CardLibrary
    "Alpha": "RARE",
    "Blasphemy": "RARE",
    "Brilliance": "RARE",
    "ConjureBlade": "RARE",
    "DeusExMachina": "RARE",
    "DevaForm": "RARE",
    "Devotion": "RARE",
    "Establishment": "RARE",
    "Judgement": "RARE",
    "LessonLearned": "RARE",
    "MasterReality": "RARE",
    "Omniscience": "RARE",
    "Ragnarok": "RARE",
    "Scrawl": "RARE",
    "SpiritShield": "RARE",
    "Vault": "RARE",
    "Wish": "RARE",
}


def get_watcher_pool_by_rarity(rarity: str) -> List[str]:
    """
    Get Watcher card IDs for a specific rarity in HashMap iteration order.

    Args:
        rarity: "COMMON", "UNCOMMON", or "RARE"

    Returns:
        List of card IDs in HashMap iteration order, filtered to the rarity.
    """
    # Get full Watcher pool in HashMap order
    watcher_order = get_cached_watcher_pool_order()

    # Filter by rarity
    return [cid for cid in watcher_order if WATCHER_CARD_RARITIES.get(cid) == rarity]


# =============================================================================
# IRONCLAD CARD RARITIES
# =============================================================================

# Card ID to rarity mapping for Ironclad cards
# Uses actual Java card IDs
# Extracted from decompiled Java source
IRONCLAD_CARD_RARITIES = {
    # BASIC cards (not in reward pools)
    "Defend_R": "BASIC",
    "Strike_R": "BASIC",
    "Bash": "BASIC",

    # COMMON cards (22 total)
    # Common Attacks
    "Anger": "COMMON",
    "BodySlam": "COMMON",
    "Clash": "COMMON",
    "Cleave": "COMMON",
    "Clothesline": "COMMON",
    "Headbutt": "COMMON",
    "HeavyBlade": "COMMON",
    "IronWave": "COMMON",
    "PerfectedStrike": "COMMON",
    "PommelStrike": "COMMON",
    "SwordBoomerang": "COMMON",
    "ThunderClap": "COMMON",
    "TwinStrike": "COMMON",
    "WildStrike": "COMMON",
    # Common Skills
    "Armaments": "COMMON",
    "Flex": "COMMON",
    "Havoc": "COMMON",
    "ShrugItOff": "COMMON",
    "TrueGrit": "COMMON",
    "Warcry": "COMMON",

    # UNCOMMON cards (37 total)
    # Uncommon Attacks
    "BloodForBlood": "UNCOMMON",
    "Carnage": "UNCOMMON",
    "Dropkick": "UNCOMMON",
    "Hemokinesis": "UNCOMMON",
    "Pummel": "UNCOMMON",
    "Rampage": "UNCOMMON",
    "RecklessCharge": "UNCOMMON",
    "SearingBlow": "UNCOMMON",
    "SeverSoul": "UNCOMMON",
    "Uppercut": "UNCOMMON",
    "Whirlwind": "UNCOMMON",
    # Uncommon Skills
    "BattleTrance": "UNCOMMON",
    "Bloodletting": "UNCOMMON",
    "BurningPact": "UNCOMMON",
    "Disarm": "UNCOMMON",
    "DualWield": "UNCOMMON",
    "Entrench": "UNCOMMON",
    "FlameBarrier": "UNCOMMON",
    "GhostlyArmor": "UNCOMMON",
    "InfernalBlade": "UNCOMMON",
    "Intimidate": "UNCOMMON",
    "PowerThrough": "UNCOMMON",
    "Rage": "UNCOMMON",
    "SecondWind": "UNCOMMON",
    "SeeingRed": "UNCOMMON",
    "Sentinel": "UNCOMMON",
    "Shockwave": "UNCOMMON",
    "SpotWeakness": "UNCOMMON",
    # Uncommon Powers
    "Combust": "UNCOMMON",
    "DarkEmbrace": "UNCOMMON",
    "Evolve": "UNCOMMON",
    "FeelNoPain": "UNCOMMON",
    "FireBreathing": "UNCOMMON",
    "Inflame": "UNCOMMON",
    "Metallicize": "UNCOMMON",
    "Rupture": "UNCOMMON",

    # RARE cards (16 total)
    # Rare Attacks
    "Bludgeon": "RARE",
    "Feed": "RARE",
    "FiendFire": "RARE",
    "Immolate": "RARE",
    "Reaper": "RARE",
    # Rare Skills
    "DoubleTap": "RARE",
    "Exhume": "RARE",
    "Impervious": "RARE",
    "LimitBreak": "RARE",
    "Offering": "RARE",
    # Rare Powers
    "Barricade": "RARE",
    "Berserk": "RARE",
    "Brutality": "RARE",
    "Corruption": "RARE",
    "DemonForm": "RARE",
    "Juggernaut": "RARE",
}


def get_ironclad_pool_order(card_ids_set: Set[str] = None) -> List[str]:
    """
    Get Ironclad card IDs in Java HashMap iteration order.

    Args:
        card_ids_set: Optional set of card IDs to filter by.
                     If None, returns all Ironclad cards.

    Returns:
        List of Ironclad card IDs in HashMap iteration order.
    """
    if card_ids_set is None:
        card_ids_set = set(RED_CARD_IDS)

    # Get full iteration order
    iteration_order = _compute_card_library_iteration_order()

    # Filter to just Ironclad cards
    return [cid for cid in iteration_order if cid in card_ids_set]


# Cache the computed orders for performance
_IRONCLAD_POOL_ORDER_CACHE: List[str] = None


def get_cached_ironclad_pool_order() -> List[str]:
    """Get cached Ironclad pool order (computed once)."""
    global _IRONCLAD_POOL_ORDER_CACHE
    if _IRONCLAD_POOL_ORDER_CACHE is None:
        _IRONCLAD_POOL_ORDER_CACHE = get_ironclad_pool_order()
    return _IRONCLAD_POOL_ORDER_CACHE


def get_ironclad_pool_by_rarity(rarity: str) -> List[str]:
    """
    Get Ironclad card IDs for a specific rarity in HashMap iteration order.

    Args:
        rarity: "COMMON", "UNCOMMON", or "RARE"

    Returns:
        List of card IDs in HashMap iteration order, filtered to the rarity.
    """
    # Get full Ironclad pool in HashMap order
    ironclad_order = get_cached_ironclad_pool_order()

    # Filter by rarity
    return [cid for cid in ironclad_order if IRONCLAD_CARD_RARITIES.get(cid) == rarity]


# =============================================================================
# SILENT CARD RARITIES
# =============================================================================

# Card ID to rarity mapping for Silent cards
# Uses actual Java card IDs from decompiled source
# Key ID mappings (where class name differs from cardID):
#   - SneakyStrike.java -> ID "Underhanded Strike" (but listed as SneakyStrike in GREEN_CARD_IDS)
#   - Nightmare.java -> ID "Night Terror"
#   - Alchemize.java -> ID "Venomology"
#   - WraithForm.java -> ID "Wraith Form v2"
SILENT_CARD_RARITIES = {
    # BASIC cards (not in reward pools)
    "Defend_G": "BASIC",
    "Strike_G": "BASIC",
    "Neutralize": "BASIC",
    "Survivor": "BASIC",

    # COMMON cards (22 total)
    # Common Attacks (9)
    "Bane": "COMMON",
    "DaggerSpray": "COMMON",
    "DaggerThrow": "COMMON",
    "FlyingKnee": "COMMON",
    "PoisonedStab": "COMMON",
    "QuickSlash": "COMMON",
    "Slice": "COMMON",
    "SneakyStrike": "COMMON",  # "Underhanded Strike" in game
    "SuckerPunch": "COMMON",
    # Common Skills (10)
    "Acrobatics": "COMMON",
    "Backflip": "COMMON",
    "BladeDance": "COMMON",
    "CloakAndDagger": "COMMON",
    "DeadlyPoison": "COMMON",
    "Deflect": "COMMON",
    "DodgeAndRoll": "COMMON",
    "Outmaneuver": "COMMON",
    "PiercingWail": "COMMON",
    "Prepared": "COMMON",

    # UNCOMMON cards (35 total)
    # Uncommon Attacks (13)
    "AllOutAttack": "UNCOMMON",
    "Backstab": "UNCOMMON",
    "Choke": "UNCOMMON",
    "Dash": "UNCOMMON",
    "EndlessAgony": "UNCOMMON",
    "Eviscerate": "UNCOMMON",
    "Finisher": "UNCOMMON",
    "Flechettes": "UNCOMMON",
    "HeelHook": "UNCOMMON",
    "MasterfulStab": "UNCOMMON",
    "Predator": "UNCOMMON",
    "RiddleWithHoles": "UNCOMMON",
    "Skewer": "UNCOMMON",
    # Uncommon Skills (14)
    "Blur": "UNCOMMON",
    "BouncingFlask": "UNCOMMON",
    "CalculatedGamble": "UNCOMMON",
    "Catalyst": "UNCOMMON",
    "Concentrate": "UNCOMMON",
    "CripplingPoison": "UNCOMMON",
    "Distraction": "UNCOMMON",
    "EscapePlan": "UNCOMMON",
    "Expertise": "UNCOMMON",
    "LegSweep": "UNCOMMON",
    "Reflex": "UNCOMMON",
    "Setup": "UNCOMMON",
    "Tactician": "UNCOMMON",
    "Terror": "UNCOMMON",
    # Uncommon Powers (6)
    "Accuracy": "UNCOMMON",
    "Caltrops": "UNCOMMON",
    "Footwork": "UNCOMMON",
    "InfiniteBlades": "UNCOMMON",
    "NoxiousFumes": "UNCOMMON",
    "WellLaidPlans": "UNCOMMON",

    # RARE cards (18 total)
    # Rare Attacks (4)
    "DieDieDie": "RARE",
    "GlassKnife": "RARE",
    "GrandFinale": "RARE",
    "Unload": "RARE",
    # Rare Skills (10)
    "Adrenaline": "RARE",
    "Alchemize": "RARE",  # "Venomology" in game
    "BulletTime": "RARE",
    "Burst": "RARE",
    "CorpseExplosion": "RARE",
    "Doppelganger": "RARE",
    "Malaise": "RARE",
    "Nightmare": "RARE",  # "Night Terror" in game
    "PhantasmalKiller": "RARE",
    "StormOfSteel": "RARE",
    # Rare Powers (5)
    "AfterImage": "RARE",
    "AThousandCuts": "RARE",
    "Envenom": "RARE",
    "ToolsOfTheTrade": "RARE",
    "WraithForm": "RARE",  # "Wraith Form v2" in game
}


def get_silent_pool_order(card_ids_set: Set[str] = None) -> List[str]:
    """
    Get Silent card IDs in Java HashMap iteration order.

    Args:
        card_ids_set: Optional set of card IDs to filter by.
                     If None, returns all Silent cards.

    Returns:
        List of Silent card IDs in HashMap iteration order.
    """
    if card_ids_set is None:
        card_ids_set = set(GREEN_CARD_IDS)

    # Get full iteration order
    iteration_order = _compute_card_library_iteration_order()

    # Filter to just Silent cards
    return [cid for cid in iteration_order if cid in card_ids_set]


# Cache the computed orders for performance
_SILENT_POOL_ORDER_CACHE: List[str] = None


def get_cached_silent_pool_order() -> List[str]:
    """Get cached Silent pool order (computed once)."""
    global _SILENT_POOL_ORDER_CACHE
    if _SILENT_POOL_ORDER_CACHE is None:
        _SILENT_POOL_ORDER_CACHE = get_silent_pool_order()
    return _SILENT_POOL_ORDER_CACHE


def get_silent_pool_by_rarity(rarity: str) -> List[str]:
    """
    Get Silent card IDs for a specific rarity in HashMap iteration order.

    Args:
        rarity: "COMMON", "UNCOMMON", or "RARE"

    Returns:
        List of card IDs in HashMap iteration order, filtered to the rarity.
    """
    # Get full Silent pool in HashMap order
    silent_order = get_cached_silent_pool_order()

    # Filter by rarity
    return [cid for cid in silent_order if SILENT_CARD_RARITIES.get(cid) == rarity]


# =============================================================================
# DEFECT CARD RARITIES
# =============================================================================

# Card ID to rarity mapping for Defect cards
# Uses actual Java card IDs (class ID, not display name)
# Key ID mappings (where class name differs from cardID):
#   - Overclock.java -> ID "Steam Power"
#   - Recursion.java -> ID "Redo"
#   - SteamBarrier.java -> ID "Steam"
#   - Equilibrium.java -> ID "Undo"
#   - LockOn.java -> ID "Lockon"
#   - MultiCast.java -> ID "Multi-Cast"
DEFECT_CARD_RARITIES = {
    # BASIC cards (not in reward pools)
    "Defend_B": "BASIC",
    "Strike_B": "BASIC",
    "Zap": "BASIC",
    "Dualcast": "BASIC",

    # COMMON cards (18 total)
    # Common Attacks (10)
    "BallLightning": "COMMON",
    "Barrage": "COMMON",
    "BeamCell": "COMMON",
    "Claw": "COMMON",
    "ColdSnap": "COMMON",
    "CompileDriver": "COMMON",
    "GoForTheEyes": "COMMON",
    "Rebound": "COMMON",
    "Streamline": "COMMON",
    "SweepingBeam": "COMMON",
    # Common Skills (8)
    "ConserveBattery": "COMMON",  # Charge Battery
    "Coolheaded": "COMMON",
    "Hologram": "COMMON",
    "Leap": "COMMON",
    "Recursion": "COMMON",  # ID is "Redo" in game
    "Stack": "COMMON",
    "SteamBarrier": "COMMON",  # ID is "Steam" in game
    "Turbo": "COMMON",

    # UNCOMMON cards (28 total)
    # Uncommon Attacks (8)
    "Blizzard": "UNCOMMON",
    "DoomAndGloom": "UNCOMMON",
    "FTL": "UNCOMMON",
    "LockOn": "UNCOMMON",  # ID is "Lockon" in game
    "Melter": "UNCOMMON",
    "RipAndTear": "UNCOMMON",
    "Scrape": "UNCOMMON",
    "Sunder": "UNCOMMON",
    # Uncommon Skills (12)
    "Aggregate": "UNCOMMON",
    "AutoShields": "UNCOMMON",
    "BootSequence": "UNCOMMON",
    "Chaos": "UNCOMMON",
    "Chill": "UNCOMMON",
    "Consume": "UNCOMMON",
    "Darkness": "UNCOMMON",
    "DoubleEnergy": "UNCOMMON",
    "Equilibrium": "UNCOMMON",  # ID is "Undo" in game
    "ForceField": "UNCOMMON",
    "Fusion": "UNCOMMON",
    "GeneticAlgorithm": "UNCOMMON",
    "Glacier": "UNCOMMON",
    "Overclock": "UNCOMMON",  # ID is "Steam Power" in game
    "Recycle": "UNCOMMON",
    "ReinforcedBody": "UNCOMMON",
    "Reprogram": "UNCOMMON",
    "Skim": "UNCOMMON",
    "Tempest": "UNCOMMON",
    "WhiteNoise": "UNCOMMON",
    # Uncommon Powers (8)
    "Capacitor": "UNCOMMON",
    "Defragment": "UNCOMMON",
    "Heatsinks": "UNCOMMON",
    "HelloWorld": "UNCOMMON",
    "Loop": "UNCOMMON",
    "SelfRepair": "UNCOMMON",
    "StaticDischarge": "UNCOMMON",
    "Storm": "UNCOMMON",

    # RARE cards (18 total)
    # Rare Attacks (5)
    "AllForOne": "RARE",
    "CoreSurge": "RARE",
    "Hyperbeam": "RARE",
    "MeteorStrike": "RARE",
    "ThunderStrike": "RARE",
    # Rare Skills (6)
    "Amplify": "RARE",
    "Fission": "RARE",
    "MultiCast": "RARE",  # ID is "Multi-Cast" in game
    "Rainbow": "RARE",
    "Reboot": "RARE",
    "Seek": "RARE",
    # Rare Powers (6)
    "BiasedCognition": "RARE",
    "Buffer": "RARE",
    "CreativeAI": "RARE",
    "EchoForm": "RARE",
    "Electrodynamics": "RARE",
    "MachineLearning": "RARE",
}


def get_defect_pool_order(card_ids_set: Set[str] = None) -> List[str]:
    """
    Get Defect card IDs in Java HashMap iteration order.

    Args:
        card_ids_set: Optional set of card IDs to filter by.
                     If None, returns all Defect cards.

    Returns:
        List of Defect card IDs in HashMap iteration order.
    """
    if card_ids_set is None:
        card_ids_set = set(BLUE_CARD_IDS)

    # Get full iteration order
    iteration_order = _compute_card_library_iteration_order()

    # Filter to just Defect cards
    return [cid for cid in iteration_order if cid in card_ids_set]


# Cache the computed orders for performance
_DEFECT_POOL_ORDER_CACHE: List[str] = None


def get_cached_defect_pool_order() -> List[str]:
    """Get cached Defect pool order (computed once)."""
    global _DEFECT_POOL_ORDER_CACHE
    if _DEFECT_POOL_ORDER_CACHE is None:
        _DEFECT_POOL_ORDER_CACHE = get_defect_pool_order()
    return _DEFECT_POOL_ORDER_CACHE


def get_defect_pool_by_rarity(rarity: str) -> List[str]:
    """
    Get Defect card IDs for a specific rarity in HashMap iteration order.

    Args:
        rarity: "COMMON", "UNCOMMON", or "RARE"

    Returns:
        List of card IDs in HashMap iteration order, filtered to the rarity.
    """
    # Get full Defect pool in HashMap order
    defect_order = get_cached_defect_pool_order()

    # Filter by rarity
    return [cid for cid in defect_order if DEFECT_CARD_RARITIES.get(cid) == rarity]


# =============================================================================
# TESTING
# =============================================================================

if __name__ == "__main__":
    print("=== Card Library Order Tests ===\n")

    print(f"Total cards in CardLibrary: {len(ALL_CARD_IDS)}")
    print(f"  Red (Ironclad): {len(RED_CARD_IDS)}")
    print(f"  Green (Silent): {len(GREEN_CARD_IDS)}")
    print(f"  Blue (Defect): {len(BLUE_CARD_IDS)}")
    print(f"  Purple (Watcher): {len(PURPLE_CARD_IDS)}")
    print(f"  Colorless: {len(COLORLESS_CARD_IDS)}")
    print(f"  Curse: {len(CURSE_CARD_IDS)}")

    print("\n--- Watcher Cards in HashMap Iteration Order ---")
    watcher_order = get_cached_watcher_pool_order()
    print(f"Total Watcher cards: {len(watcher_order)}")
    for i, card_id in enumerate(watcher_order):
        rarity = WATCHER_CARD_RARITIES.get(card_id, "???")
        print(f"  {i:2d}: {card_id} ({rarity})")

    # Show rarity-specific pools
    print("\n--- COMMON Pool (HashMap Order) ---")
    common_pool = get_watcher_pool_by_rarity("COMMON")
    print(f"Total: {len(common_pool)}")
    for i, card_id in enumerate(common_pool):
        print(f"  {i:2d}: {card_id}")

    print("\n--- UNCOMMON Pool (HashMap Order) ---")
    uncommon_pool = get_watcher_pool_by_rarity("UNCOMMON")
    print(f"Total: {len(uncommon_pool)}")
    for i, card_id in enumerate(uncommon_pool):
        print(f"  {i:2d}: {card_id}")

    print("\n--- RARE Pool (HashMap Order) ---")
    rare_pool = get_watcher_pool_by_rarity("RARE")
    print(f"Total: {len(rare_pool)}")
    for i, card_id in enumerate(rare_pool):
        print(f"  {i:2d}: {card_id}")

    # Find specific cards in their rarity pools
    print("\n--- Test Card Positions in Rarity Pools ---")
    test_cards = [
        ("LikeWater", "UNCOMMON"),
        ("BowlingBash", "COMMON"),
        ("DeceiveReality", "UNCOMMON"),
    ]
    for card_id, rarity in test_cards:
        pool = get_watcher_pool_by_rarity(rarity)
        try:
            idx = pool.index(card_id)
            print(f"  {card_id} ({rarity}): index {idx}")
        except ValueError:
            print(f"  {card_id} ({rarity}): NOT FOUND")
