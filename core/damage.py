"""
Damage Calculation - Exact replication from decompiled AbstractCard.calculateCardDamage()

Damage calculation order (from source):
1. Base damage
2. Relic modifiers (atDamageModify) - e.g., Strike Dummy
3. Player power atDamageGive - Strength, Pen Nib, Vigor, etc.
4. Stance modifier (atDamageGive) - Wrath 2x, Divinity 3x
5. Monster power atDamageReceive - Vulnerable 1.5x
6. Player power atDamageFinalGive - rare final modifiers
7. Floor to int

Block calculation order:
1. Base block
2. Player power modifyBlock - Dexterity, etc.
3. Player power modifyBlockLast - rare final modifiers
4. Floor to int

Damage types (from DamageInfo.DamageType):
- NORMAL: Standard attack damage (affected by Str, Weak, Vulnerable, stances)
- THORNS: Retaliation damage (not affected by most modifiers)
- HP_LOSS: Direct HP loss (ignores block, not affected by modifiers)
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict
from enum import Enum
import math


class DamageType(Enum):
    """Damage types matching DamageInfo.DamageType."""
    NORMAL = "NORMAL"
    THORNS = "THORNS"
    HP_LOSS = "HP_LOSS"


@dataclass
class Power:
    """A combat buff/debuff."""
    id: str
    amount: int = 0

    def at_damage_give(self, damage: float, damage_type: DamageType) -> float:
        """Modify outgoing damage."""
        if damage_type != DamageType.NORMAL:
            return damage

        # Strength: +amount per attack
        if self.id == "Strength":
            return damage + self.amount

        # Vigor: +amount, consumed on attack
        if self.id == "Vigor":
            return damage + self.amount

        # Pen Nib: 2x damage on 10th attack
        if self.id == "Pen Nib":
            return damage * 2.0

        # Weak: 25% less damage
        if self.id == "Weak":
            return damage * 0.75

        # Double Damage: 2x
        if self.id == "DoubleDamage":
            return damage * 2.0

        return damage

    def at_damage_receive(self, damage: float, damage_type: DamageType) -> float:
        """Modify incoming damage."""
        if damage_type != DamageType.NORMAL:
            return damage

        # Vulnerable: 50% more damage
        if self.id == "Vulnerable":
            return damage * 1.5

        # Intangible: reduce to 1
        if self.id == "Intangible":
            return 1.0

        # Flight: 50% less damage
        if self.id == "Flight":
            return damage * 0.5

        return damage

    def modify_block(self, block: float) -> float:
        """Modify block gained."""
        # Dexterity: +amount per block
        if self.id == "Dexterity":
            return block + self.amount

        # Frail: 25% less block
        if self.id == "Frail":
            return block * 0.75

        return block


@dataclass
class Relic:
    """A relic that can modify damage."""
    id: str
    counter: int = 0

    def at_damage_modify(self, damage: float, card_id: str) -> float:
        """Modify damage based on relic effect."""
        # Strike Dummy: +3 damage on Strike cards
        if self.id == "StrikeDummy" and "Strike" in card_id:
            return damage + 3.0

        # Vajra: +1 Strength (handled as power, but mentioned)
        # Pen Nib: handled as power

        return damage


@dataclass
class CombatState:
    """State needed for damage calculation."""
    # Player
    player_powers: List[Power] = field(default_factory=list)
    player_relics: List[Relic] = field(default_factory=list)
    stance_damage_mult: float = 1.0  # Outgoing damage: Wrath=2, Divinity=3
    stance_incoming_mult: float = 1.0  # Incoming damage: Wrath=2, Divinity=1

    # Target
    target_powers: List[Power] = field(default_factory=list)

    def get_power(self, powers: List[Power], power_id: str) -> Optional[Power]:
        """Get a power by ID."""
        for p in powers:
            if p.id == power_id:
                return p
        return None


def calculate_card_damage(
    base_damage: int,
    state: CombatState,
    card_id: str = "",
    damage_type: DamageType = DamageType.NORMAL,
    is_multi_hit: bool = False,
    hits: int = 1,
) -> int:
    """
    Calculate final damage for a card.

    Follows exact order from AbstractCard.calculateCardDamage():
    1. Relic modifiers
    2. Player power atDamageGive (Strength, Weak, etc.)
    3. Stance modifier
    4. Target power atDamageReceive (Vulnerable, etc.)
    5. Player power atDamageFinalGive
    6. Floor to int, minimum 0

    Args:
        base_damage: Card's base damage value
        state: Current combat state
        card_id: Card identifier (for Strike Dummy, etc.)
        damage_type: Type of damage
        is_multi_hit: If True, returns per-hit damage
        hits: Number of hits (for display)

    Returns:
        Final damage value (per hit if multi-hit)
    """
    damage = float(base_damage)

    # 1. Relic modifiers
    for relic in state.player_relics:
        damage = relic.at_damage_modify(damage, card_id)

    # 2. Player power atDamageGive
    for power in state.player_powers:
        damage = power.at_damage_give(damage, damage_type)

    # 3. Stance modifier
    if damage_type == DamageType.NORMAL:
        damage *= state.stance_damage_mult

    # 4. Target power atDamageReceive
    for power in state.target_powers:
        damage = power.at_damage_receive(damage, damage_type)

    # 5. Player power atDamageFinalGive (rare, usually nothing)
    # (no common powers use this)

    # 6. Floor to int, minimum 0
    final_damage = max(0, math.floor(damage))

    return final_damage


def calculate_block(
    base_block: int,
    state: CombatState,
) -> int:
    """
    Calculate final block for a card/action.

    Order from AbstractCard.applyPowersToBlock():
    1. Player power modifyBlock (Dexterity, Frail)
    2. Player power modifyBlockLast
    3. Floor to int, minimum 0
    """
    block = float(base_block)

    # 1. Player power modifyBlock
    for power in state.player_powers:
        block = power.modify_block(block)

    # 2. modifyBlockLast (rare)
    # (no common powers use this)

    # 3. Floor to int, minimum 0
    return max(0, math.floor(block))


def calculate_incoming_damage(
    damage: int,
    block: int,
    state: CombatState,
    damage_type: DamageType = DamageType.NORMAL,
) -> Dict[str, int]:
    """
    Calculate damage taken after block.

    Returns:
        {
            "damage": final damage dealt,
            "hp_loss": actual HP lost after block,
            "block_used": block consumed,
            "block_remaining": block left after
        }
    """
    # Apply incoming damage modifiers
    modified_damage = float(damage)
    if damage_type == DamageType.NORMAL:
        # Stance receive multiplier (Wrath 2x, Divinity 1x)
        modified_damage *= state.stance_incoming_mult

        # Player powers that reduce incoming
        for power in state.player_powers:
            modified_damage = power.at_damage_receive(modified_damage, damage_type)

    modified_damage = max(0, math.floor(modified_damage))

    # Apply block (only for NORMAL damage)
    if damage_type == DamageType.HP_LOSS:
        # HP_LOSS ignores block
        return {
            "damage": modified_damage,
            "hp_loss": modified_damage,
            "block_used": 0,
            "block_remaining": block,
        }

    blocked = min(block, modified_damage)
    hp_loss = modified_damage - blocked

    return {
        "damage": modified_damage,
        "hp_loss": hp_loss,
        "block_used": blocked,
        "block_remaining": block - blocked,
    }


# ============ WATCHER-SPECIFIC HELPERS ============

def wrath_damage(base: int, strength: int = 0, vulnerable: bool = False) -> int:
    """Quick calc for Wrath stance damage."""
    state = CombatState(
        player_powers=[Power("Strength", strength)] if strength else [],
        stance_damage_mult=2.0,
        stance_incoming_mult=2.0,  # Wrath also doubles incoming
        target_powers=[Power("Vulnerable", 1)] if vulnerable else [],
    )
    return calculate_card_damage(base, state)


def divinity_damage(base: int, strength: int = 0, vulnerable: bool = False) -> int:
    """Quick calc for Divinity stance damage."""
    state = CombatState(
        player_powers=[Power("Strength", strength)] if strength else [],
        stance_damage_mult=3.0,
        stance_incoming_mult=1.0,  # Divinity does NOT affect incoming
        target_powers=[Power("Vulnerable", 1)] if vulnerable else [],
    )
    return calculate_card_damage(base, state)


# ============ TESTING ============

if __name__ == "__main__":
    print("=== Damage Calculation Tests ===\n")

    # Basic damage
    state = CombatState()
    dmg = calculate_card_damage(6, state)
    print(f"Strike (6 base): {dmg}")

    # With Strength
    state = CombatState(player_powers=[Power("Strength", 3)])
    dmg = calculate_card_damage(6, state)
    print(f"Strike + 3 Str: {dmg}")

    # With Wrath
    state = CombatState(stance_damage_mult=2.0, stance_incoming_mult=2.0)
    dmg = calculate_card_damage(6, state)
    print(f"Strike in Wrath: {dmg}")

    # With Str + Wrath + Vulnerable
    state = CombatState(
        player_powers=[Power("Strength", 3)],
        stance_damage_mult=2.0,
        stance_incoming_mult=2.0,
        target_powers=[Power("Vulnerable", 1)],
    )
    dmg = calculate_card_damage(6, state)
    print(f"Strike + 3 Str + Wrath + Vuln: {dmg}")
    print(f"  Math: (6+3)*2*1.5 = {(6+3)*2*1.5}")

    # Divinity
    print(f"\nDivinity 6 base: {divinity_damage(6)}")
    print(f"Divinity 6 base + Vuln: {divinity_damage(6, vulnerable=True)}")

    # Block
    print("\n=== Block Tests ===")
    state = CombatState()
    blk = calculate_block(5, state)
    print(f"Defend (5 base): {blk}")

    state = CombatState(player_powers=[Power("Dexterity", 2)])
    blk = calculate_block(5, state)
    print(f"Defend + 2 Dex: {blk}")

    state = CombatState(player_powers=[Power("Frail", 1)])
    blk = calculate_block(5, state)
    print(f"Defend while Frail: {blk}")

    # Incoming damage
    print("\n=== Incoming Damage Tests ===")
    state = CombatState()
    result = calculate_incoming_damage(10, 5, state)
    print(f"10 dmg, 5 block: {result}")

    state = CombatState(stance_damage_mult=2.0, stance_incoming_mult=2.0)
    result = calculate_incoming_damage(10, 5, state)
    print(f"10 dmg, 5 block, Wrath: {result}")

    state = CombatState(stance_damage_mult=3.0, stance_incoming_mult=1.0)
    result = calculate_incoming_damage(10, 5, state)
    print(f"10 dmg, 5 block, Divinity: {result} (should be same as neutral, NOT 30)")
