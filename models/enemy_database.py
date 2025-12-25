"""
Enemy encounter database for Slay the Spire.

Pre-computed difficulty ratings and expected damage for each encounter.
This lets the model know "Gremlin Nob is dangerous early" without learning it.
"""

from dataclasses import dataclass
from typing import Dict, List, Tuple

@dataclass
class EnemyInfo:
    """Info about a single enemy type."""
    name: str
    base_hp: Tuple[int, int]  # (min, max) at A0
    a20_hp: Tuple[int, int]   # (min, max) at A20
    typical_damage: int        # Average damage per turn
    threat_level: float        # 0-1, how dangerous
    attack_pattern: str        # "consistent", "scaling", "burst", "weak"
    notes: str = ""


@dataclass
class EncounterInfo:
    """Info about an encounter (group of enemies)."""
    enemies: List[str]
    act: int
    is_elite: bool
    is_boss: bool
    difficulty: float          # 0-1 overall difficulty rating
    expected_damage_taken: int # Average HP loss for decent deck
    turns_to_kill: float       # Expected turns to clear
    danger_spikes: List[int]   # Turns where danger increases
    notes: str = ""


# === ENEMY DATABASE ===
ENEMIES: Dict[str, EnemyInfo] = {
    # Act 1 normals
    "Cultist": EnemyInfo(
        name="Cultist", base_hp=(48, 54), a20_hp=(50, 56),
        typical_damage=6, threat_level=0.2, attack_pattern="scaling",
        notes="Gains strength each turn. Kill fast."
    ),
    "JawWorm": EnemyInfo(
        name="Jaw Worm", base_hp=(40, 44), a20_hp=(42, 46),
        typical_damage=11, threat_level=0.3, attack_pattern="consistent",
        notes="Can block. Straightforward fight."
    ),
    "Louse": EnemyInfo(
        name="Louse", base_hp=(10, 15), a20_hp=(11, 17),
        typical_damage=6, threat_level=0.1, attack_pattern="consistent",
        notes="Curls up to gain block. Usually paired."
    ),
    "SpikeSlime_M": EnemyInfo(
        name="Spike Slime (M)", base_hp=(28, 32), a20_hp=(29, 34),
        typical_damage=8, threat_level=0.2, attack_pattern="consistent",
        notes="Adds Slimed to deck."
    ),
    "AcidSlime_M": EnemyInfo(
        name="Acid Slime (M)", base_hp=(28, 32), a20_hp=(29, 34),
        typical_damage=10, threat_level=0.2, attack_pattern="consistent",
        notes="Applies weak."
    ),
    "FungiBeast": EnemyInfo(
        name="Fungi Beast", base_hp=(22, 28), a20_hp=(24, 30),
        typical_damage=6, threat_level=0.2, attack_pattern="scaling",
        notes="Grows spore cloud on death."
    ),
    "GremlinNob": EnemyInfo(
        name="Gremlin Nob", base_hp=(82, 86), a20_hp=(85, 90),
        typical_damage=14, threat_level=0.8, attack_pattern="burst",
        notes="ELITE. Enrages on skills. Front-load damage."
    ),
    "Lagavulin": EnemyInfo(
        name="Lagavulin", base_hp=(109, 111), a20_hp=(112, 115),
        typical_damage=18, threat_level=0.7, attack_pattern="scaling",
        notes="ELITE. Sleeps 3 turns. Debuffs you."
    ),
    "Sentries": EnemyInfo(
        name="Sentry", base_hp=(38, 42), a20_hp=(39, 45),
        typical_damage=9, threat_level=0.6, attack_pattern="consistent",
        notes="ELITE. 3 of them. Add Dazed to deck."
    ),
    "Hexaghost": EnemyInfo(
        name="Hexaghost", base_hp=(250, 250), a20_hp=(264, 264),
        typical_damage=0, threat_level=0.7, attack_pattern="burst",
        notes="BOSS. Big attacks every 6 turns. Adds Burns."
    ),
    "SlimeBoss": EnemyInfo(
        name="Slime Boss", base_hp=(140, 140), a20_hp=(150, 150),
        typical_damage=35, threat_level=0.6, attack_pattern="burst",
        notes="BOSS. Splits at 50% HP. Kill in one turn after."
    ),
    "TheGuardian": EnemyInfo(
        name="The Guardian", base_hp=(240, 240), a20_hp=(250, 250),
        typical_damage=32, threat_level=0.8, attack_pattern="burst",
        notes="BOSS. Mode shifts. Don't over-hit in defense mode."
    ),

    # Act 2
    "Chosen": EnemyInfo(
        name="Chosen", base_hp=(95, 99), a20_hp=(98, 103),
        typical_damage=12, threat_level=0.4, attack_pattern="consistent",
        notes="Hexes you (weak, vulnerable). Has thorns."
    ),
    "ShelledParasite": EnemyInfo(
        name="Shelled Parasite", base_hp=(68, 72), a20_hp=(71, 76),
        typical_damage=18, threat_level=0.4, attack_pattern="burst",
        notes="High block. Big attacks."
    ),
    "SnakePlant": EnemyInfo(
        name="Snake Plant", base_hp=(75, 79), a20_hp=(78, 83),
        typical_damage=7, threat_level=0.3, attack_pattern="consistent",
        notes="Malleable - gains block when hit."
    ),
    "BookOfStabbing": EnemyInfo(
        name="Book of Stabbing", base_hp=(160, 162), a20_hp=(168, 172),
        typical_damage=21, threat_level=0.7, attack_pattern="scaling",
        notes="ELITE. Multi-hit attack scales each turn."
    ),
    "GremlinLeader": EnemyInfo(
        name="Gremlin Leader", base_hp=(140, 148), a20_hp=(145, 155),
        typical_damage=0, threat_level=0.6, attack_pattern="scaling",
        notes="ELITE. Summons gremlins. Kill adds fast."
    ),
    "Taskmaster": EnemyInfo(
        name="Taskmaster", base_hp=(54, 60), a20_hp=(57, 64),
        typical_damage=7, threat_level=0.5, attack_pattern="consistent",
        notes="ELITE. 2 of them. Add wounds to deck."
    ),
    "BronzeAutomaton": EnemyInfo(
        name="Bronze Automaton", base_hp=(300, 300), a20_hp=(320, 320),
        typical_damage=45, threat_level=0.8, attack_pattern="burst",
        notes="BOSS. Hyper beam is huge. Summons orbs."
    ),
    "TheChamp": EnemyInfo(
        name="The Champ", base_hp=(420, 420), a20_hp=(440, 440),
        typical_damage=18, threat_level=0.7, attack_pattern="burst",
        notes="BOSS. Executes under 50%. Big turns."
    ),
    "Collector": EnemyInfo(
        name="The Collector", base_hp=(282, 282), a20_hp=(296, 296),
        typical_damage=15, threat_level=0.7, attack_pattern="scaling",
        notes="BOSS. Spawns torches. Gets angry."
    ),

    # Act 3
    "Transient": EnemyInfo(
        name="Transient", base_hp=(999, 999), a20_hp=(999, 999),
        typical_damage=30, threat_level=0.5, attack_pattern="scaling",
        notes="Runs away. Just survive 5 turns."
    ),
    "GiantHead": EnemyInfo(
        name="Giant Head", base_hp=(500, 500), a20_hp=(520, 520),
        typical_damage=0, threat_level=0.7, attack_pattern="scaling",
        notes="ELITE. Slow = debuff on playing 12+ cards."
    ),
    "Nemesis": EnemyInfo(
        name="Nemesis", base_hp=(185, 185), a20_hp=(200, 200),
        typical_damage=40, threat_level=0.8, attack_pattern="burst",
        notes="ELITE. Intangible. Burns. Nasty."
    ),
    "Reptomancer": EnemyInfo(
        name="Reptomancer", base_hp=(180, 190), a20_hp=(190, 200),
        typical_damage=16, threat_level=0.7, attack_pattern="consistent",
        notes="ELITE. Summons daggers that attack."
    ),
    "AwakenedOne": EnemyInfo(
        name="Awakened One", base_hp=(300, 300), a20_hp=(320, 320),
        typical_damage=20, threat_level=0.9, attack_pattern="scaling",
        notes="BOSS. Gains strength on power plays. Two phases."
    ),
    "TimeEater": EnemyInfo(
        name="Time Eater", base_hp=(456, 456), a20_hp=(480, 480),
        typical_damage=30, threat_level=0.9, attack_pattern="scaling",
        notes="BOSS. Ends turn at 12 cards. Heals."
    ),
    "Donu": EnemyInfo(
        name="Donu", base_hp=(250, 250), a20_hp=(265, 265),
        typical_damage=0, threat_level=0.4, attack_pattern="scaling",
        notes="BOSS (paired). Buffs. Kill Deca first usually."
    ),
    "Deca": EnemyInfo(
        name="Deca", base_hp=(250, 250), a20_hp=(265, 265),
        typical_damage=12, threat_level=0.5, attack_pattern="consistent",
        notes="BOSS (paired). Adds Dazed. Kill first."
    ),

    # Act 4
    "SpireShield": EnemyInfo(
        name="Spire Shield", base_hp=(110, 110), a20_hp=(120, 120),
        typical_damage=0, threat_level=0.3, attack_pattern="consistent",
        notes="High block. Buffs spear."
    ),
    "SpireSpear": EnemyInfo(
        name="Spire Spear", base_hp=(160, 160), a20_hp=(180, 180),
        typical_damage=25, threat_level=0.5, attack_pattern="scaling",
        notes="Burns. Multi-hits. Kill first."
    ),
    "CorruptHeart": EnemyInfo(
        name="Corrupt Heart", base_hp=(800, 800), a20_hp=(800, 800),
        typical_damage=1, threat_level=1.0, attack_pattern="scaling",
        notes="HEART. Beat of Death. Invincible first turn."
    ),
}


# === ENCOUNTER DATABASE ===
ENCOUNTERS: Dict[str, EncounterInfo] = {
    # Act 1
    "Cultist": EncounterInfo(
        enemies=["Cultist"], act=1, is_elite=False, is_boss=False,
        difficulty=0.2, expected_damage_taken=8, turns_to_kill=3,
        danger_spikes=[4, 5], notes="Kill before ritual stacks"
    ),
    "JawWorm": EncounterInfo(
        enemies=["JawWorm"], act=1, is_elite=False, is_boss=False,
        difficulty=0.3, expected_damage_taken=12, turns_to_kill=3,
        danger_spikes=[], notes="Straightforward"
    ),
    "2Louse": EncounterInfo(
        enemies=["Louse", "Louse"], act=1, is_elite=False, is_boss=False,
        difficulty=0.2, expected_damage_taken=6, turns_to_kill=2,
        danger_spikes=[], notes="Fast fight"
    ),
    "SmallSlimes": EncounterInfo(
        enemies=["SpikeSlime_M", "AcidSlime_M"], act=1, is_elite=False, is_boss=False,
        difficulty=0.3, expected_damage_taken=10, turns_to_kill=3,
        danger_spikes=[], notes="Kill spike first usually"
    ),
    "GremlinNob": EncounterInfo(
        enemies=["GremlinNob"], act=1, is_elite=True, is_boss=False,
        difficulty=0.8, expected_damage_taken=30, turns_to_kill=4,
        danger_spikes=[2, 3, 4], notes="Front-load. Avoid skills."
    ),
    "Lagavulin": EncounterInfo(
        enemies=["Lagavulin"], act=1, is_elite=True, is_boss=False,
        difficulty=0.7, expected_damage_taken=25, turns_to_kill=6,
        danger_spikes=[4, 5, 6], notes="Setup during sleep. Block debuffs."
    ),
    "3Sentries": EncounterInfo(
        enemies=["Sentry", "Sentry", "Sentry"], act=1, is_elite=True, is_boss=False,
        difficulty=0.6, expected_damage_taken=20, turns_to_kill=5,
        danger_spikes=[2, 4], notes="Kill outside ones first"
    ),
    "Hexaghost": EncounterInfo(
        enemies=["Hexaghost"], act=1, is_elite=False, is_boss=True,
        difficulty=0.7, expected_damage_taken=40, turns_to_kill=8,
        danger_spikes=[6], notes="Block the inferno"
    ),
    "SlimeBoss": EncounterInfo(
        enemies=["SlimeBoss"], act=1, is_elite=False, is_boss=True,
        difficulty=0.6, expected_damage_taken=35, turns_to_kill=6,
        danger_spikes=[3], notes="Get to 50% then burst"
    ),
    "TheGuardian": EncounterInfo(
        enemies=["TheGuardian"], act=1, is_elite=False, is_boss=True,
        difficulty=0.8, expected_damage_taken=50, turns_to_kill=10,
        danger_spikes=[3, 6], notes="Respect mode shift"
    ),
}


def get_enemy_info(enemy_name: str) -> EnemyInfo:
    """Get info about an enemy type."""
    # Normalize name
    normalized = enemy_name.replace(" ", "").replace("_", "")
    for key, info in ENEMIES.items():
        if key.lower() == normalized.lower() or info.name.lower().replace(" ", "") == normalized.lower():
            return info
    return None


def get_encounter_difficulty(enemies: List[str], floor: int, is_elite: bool, is_boss: bool) -> float:
    """Estimate difficulty of an encounter."""
    base_difficulty = 0.3

    # Sum threat levels
    for enemy in enemies:
        info = get_enemy_info(enemy)
        if info:
            base_difficulty += info.threat_level * 0.3

    # Adjust for fight type
    if is_boss:
        base_difficulty += 0.2
    if is_elite:
        base_difficulty += 0.1

    # Adjust for floor (later = harder context)
    base_difficulty += (floor / 57) * 0.1

    return min(1.0, base_difficulty)


def estimate_damage_vs_encounter(
    deck_power: float,  # 0-1, how strong is our deck
    encounter: str,
) -> Tuple[int, int]:
    """
    Estimate (min_damage, max_damage) we'll take against an encounter.

    Returns (optimistic, pessimistic) damage estimates.
    """
    enc = ENCOUNTERS.get(encounter)
    if not enc:
        return (10, 30)  # Default guess

    base = enc.expected_damage_taken

    # Adjust for deck power
    # Strong deck = less damage
    multiplier = 1.5 - deck_power  # 0.5 to 1.5

    optimistic = int(base * multiplier * 0.5)
    pessimistic = int(base * multiplier * 1.5)

    return (optimistic, pessimistic)


if __name__ == "__main__":
    print("=== Enemy Database ===\n")

    print("Act 1 Elites:")
    for name in ["GremlinNob", "Lagavulin", "Sentries"]:
        info = get_enemy_info(name)
        if info:
            print(f"  {info.name}: threat={info.threat_level}, typical_dmg={info.typical_damage}")

    print("\nEncounter Difficulty:")
    for enc_name, enc in ENCOUNTERS.items():
        print(f"  {enc_name}: difficulty={enc.difficulty:.1f}, expected_dmg={enc.expected_damage_taken}")

    print("\nDamage Estimates (weak deck, power=0.3):")
    for enc_name in ["Cultist", "GremlinNob", "Hexaghost"]:
        opt, pess = estimate_damage_vs_encounter(0.3, enc_name)
        print(f"  {enc_name}: {opt}-{pess} damage")
