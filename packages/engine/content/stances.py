"""
Watcher Stance System - Exact replication from decompiled source.

Stances:
- Neutral: No effects (default)
- Calm: +2 energy on exit (Violet Lotus: +3)
- Wrath: 2x damage dealt AND received
- Divinity: 3x damage dealt, +3 energy on enter, exits at end of turn

From decompiled WrathStance.java:
    public float atDamageGive(float damage, DamageType type) {
        if (type == DamageType.NORMAL) return damage * 2.0f;
        return damage;
    }
    public float atDamageReceive(float damage, DamageType type) {
        if (type == DamageType.NORMAL) return damage * 2.0f;
        return damage;
    }
"""

from enum import Enum
from dataclasses import dataclass
from typing import Optional


class StanceID(Enum):
    """Stance identifiers matching game's STANCE_ID constants."""
    NEUTRAL = "Neutral"
    CALM = "Calm"
    WRATH = "Wrath"
    DIVINITY = "Divinity"


@dataclass
class StanceEffect:
    """Effects applied by a stance."""
    damage_give_multiplier: float = 1.0  # Multiplier for damage dealt
    damage_receive_multiplier: float = 1.0  # Multiplier for damage received
    energy_on_enter: int = 0  # Energy gained when entering
    energy_on_exit: int = 0  # Energy gained when exiting
    exits_at_turn_end: bool = False  # Auto-exit at end of turn


# Stance definitions matching decompiled source
STANCES = {
    StanceID.NEUTRAL: StanceEffect(
        damage_give_multiplier=1.0,
        damage_receive_multiplier=1.0,
    ),
    StanceID.CALM: StanceEffect(
        damage_give_multiplier=1.0,
        damage_receive_multiplier=1.0,
        energy_on_exit=2,  # Violet Lotus makes this 3
    ),
    StanceID.WRATH: StanceEffect(
        damage_give_multiplier=2.0,
        damage_receive_multiplier=2.0,
    ),
    StanceID.DIVINITY: StanceEffect(
        damage_give_multiplier=3.0,
        damage_receive_multiplier=1.0,  # No damage increase received
        energy_on_enter=3,
        exits_at_turn_end=True,
    ),
}


class StanceManager:
    """
    Manages stance state during combat.

    Tracks current stance, handles transitions, triggers effects.
    """

    def __init__(self, has_violet_lotus: bool = False):
        """
        Initialize stance manager.

        Args:
            has_violet_lotus: If True, Calm exit gives +3 energy instead of +2
        """
        self.current = StanceID.NEUTRAL
        self.has_violet_lotus = has_violet_lotus
        self.mantra = 0  # Mantra counter for Divinity

    @property
    def effect(self) -> StanceEffect:
        """Get current stance's effects."""
        return STANCES[self.current]

    def at_damage_give(self, damage: float, damage_type: str = "NORMAL") -> float:
        """
        Modify outgoing damage based on stance.

        Only applies to NORMAL damage type (not HP_LOSS, THORNS).
        """
        if damage_type == "NORMAL":
            return damage * self.effect.damage_give_multiplier
        return damage

    def at_damage_receive(self, damage: float, damage_type: str = "NORMAL") -> float:
        """
        Modify incoming damage based on stance.

        Only applies to NORMAL damage type.
        """
        if damage_type == "NORMAL":
            return damage * self.effect.damage_receive_multiplier
        return damage

    def change_stance(self, new_stance: StanceID) -> dict:
        """
        Change to a new stance.

        Returns dict with effects triggered:
        - energy_gained: Energy from exit/enter
        - exited: Previous stance (if changed)
        - entered: New stance (if changed)
        - triggered_rushdown: If Calm->Wrath with Rushdown
        """
        result = {
            "energy_gained": 0,
            "exited": None,
            "entered": None,
            "is_stance_change": False,
        }

        # No change if same stance
        if new_stance == self.current:
            return result

        result["is_stance_change"] = True
        old_stance = self.current

        # Exit current stance
        if old_stance != StanceID.NEUTRAL:
            result["exited"] = old_stance
            exit_effect = STANCES[old_stance]

            # Calm exit energy
            if old_stance == StanceID.CALM:
                energy = 3 if self.has_violet_lotus else exit_effect.energy_on_exit
                result["energy_gained"] += energy

        # Enter new stance
        if new_stance != StanceID.NEUTRAL:
            result["entered"] = new_stance
            enter_effect = STANCES[new_stance]
            result["energy_gained"] += enter_effect.energy_on_enter

        self.current = new_stance
        return result

    def exit_stance(self) -> dict:
        """Exit current stance (go to Neutral)."""
        return self.change_stance(StanceID.NEUTRAL)

    def add_mantra(self, amount: int) -> dict:
        """
        Add mantra and potentially enter Divinity.

        Returns dict with effects if Divinity triggered.
        """
        self.mantra += amount
        result = {"mantra_added": amount, "divinity_triggered": False}

        if self.mantra >= 10:
            self.mantra -= 10  # Excess carries over
            result["divinity_triggered"] = True
            result.update(self.change_stance(StanceID.DIVINITY))

        return result

    def on_turn_end(self) -> dict:
        """
        Called at end of turn.

        Handles Divinity auto-exit.
        """
        result = {}
        if self.current == StanceID.DIVINITY:
            result = self.change_stance(StanceID.NEUTRAL)
            result["divinity_ended"] = True
        return result

    def is_in_calm(self) -> bool:
        return self.current == StanceID.CALM

    def is_in_wrath(self) -> bool:
        return self.current == StanceID.WRATH

    def is_in_divinity(self) -> bool:
        return self.current == StanceID.DIVINITY


# ============ CARD STANCE INTERACTIONS ============

# Cards that enter Wrath
WRATH_ENTRY_CARDS = [
    "Eruption",      # 2 cost, 9 dmg, enter Wrath
    "Tantrum",       # 1 cost, 3x3 dmg, enter Wrath, shuffle into draw
    "Crescendo",     # 0 cost, enter Wrath, exhaust, retain
    "Indignation",   # 1 cost, enter Wrath OR 3 Mantra if in Wrath
    "Ragnarok",      # 3 cost, 5x5 dmg, enter Wrath (if not exhausted version)
]

# Cards that enter Calm
CALM_ENTRY_CARDS = [
    "Vigilance",     # 2 cost, 8 block, enter Calm
    "Tranquility",   # 0 cost, enter Calm, exhaust, retain
    "Fear No Evil",  # 1 cost, 8 dmg, enter Calm if enemy attacking
    "Inner Peace",   # 1 cost, enter Calm OR draw 3 if in Calm
    "Like Water",    # Power, at end of turn if in Calm gain 5 block
]

# Cards that exit stance
STANCE_EXIT_CARDS = [
    "Empty Body",    # 1 cost, 7 block, exit stance
    "Empty Fist",    # 1 cost, 9 dmg, exit stance
    "Empty Mind",    # 1 cost, draw 2, exit stance
    "Flurry of Blows", # 0 cost, auto-plays when changing stance
]


# ============ TESTING ============

if __name__ == "__main__":
    # Test basic stance changes
    sm = StanceManager()
    print(f"Starting stance: {sm.current.value}")

    # Enter Wrath
    result = sm.change_stance(StanceID.WRATH)
    print(f"\nEntered Wrath: {result}")
    print(f"Damage multiplier: {sm.at_damage_give(10)} (10 base)")

    # Enter Calm from Wrath
    result = sm.change_stance(StanceID.CALM)
    print(f"\nEntered Calm from Wrath: {result}")

    # Exit Calm (get energy)
    result = sm.exit_stance()
    print(f"\nExited Calm: {result}")
    print(f"Energy gained: {result['energy_gained']}")

    # Test with Violet Lotus
    sm2 = StanceManager(has_violet_lotus=True)
    sm2.change_stance(StanceID.CALM)
    result = sm2.exit_stance()
    print(f"\nWith Violet Lotus, Calm exit energy: {result['energy_gained']}")

    # Test Mantra -> Divinity
    sm3 = StanceManager()
    for i in range(3):
        result = sm3.add_mantra(3)
        print(f"\nAdded 3 Mantra ({sm3.mantra} total): Divinity? {result['divinity_triggered']}")
    result = sm3.add_mantra(1)
    print(f"Added 1 Mantra ({sm3.mantra} total): Divinity? {result['divinity_triggered']}")
    print(f"Energy gained from Divinity: {result.get('energy_gained', 0)}")
