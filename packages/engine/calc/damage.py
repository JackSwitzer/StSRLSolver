"""
Damage Calculator - Single source of truth for all damage and block calculations.

Design principles:
1. Pure functions - no side effects, no state
2. Everything is a modifier (buffs, debuffs, relics, stances)
3. Clear calculation order matching game exactly
4. Optimized for millions of calls in simulations

Core calculation order (from decompiled AbstractCard.calculateCardDamage):
1. Base damage
2. Flat adds (Strength, Vigor)
3. Multipliers (Weak: 0.75, Pen Nib: 2.0)
4. Stance multiplier (Wrath: 2.0, Divinity: 3.0)
5. Target multipliers (Vulnerable: 1.5)
6. Final caps (Intangible: max 1)
7. Floor to int
"""

from typing import Tuple

__all__ = [
    "calculate_damage",
    "calculate_block",
    "calculate_incoming_damage",
    "apply_hp_loss",
    # Constants
    "WEAK_MULT",
    "WEAK_MULT_PAPER_CRANE",
    "VULN_MULT",
    "VULN_MULT_ODD_MUSHROOM",
    "VULN_MULT_PAPER_FROG",
    "FRAIL_MULT",
    "WRATH_MULT",
    "DIVINITY_MULT",
    "FLIGHT_MULT",
    "wrath_damage",
    "divinity_damage",
]


# =============================================================================
# CONSTANTS - Multipliers from decompiled source
# =============================================================================

# Weak - reduces NORMAL damage dealt by 25%
WEAK_MULT = 0.75
WEAK_MULT_PAPER_CRANE = 0.60  # Paper Crane: 40% reduction instead of 25%

# Vulnerable - increases NORMAL damage received by 50%
VULN_MULT = 1.50
VULN_MULT_ODD_MUSHROOM = 1.25  # Odd Mushroom: only 25% increase on player
VULN_MULT_PAPER_FROG = 1.75   # Paper Frog: 75% increase on enemies

# Frail - reduces block from cards by 25%
FRAIL_MULT = 0.75

# Stances
WRATH_MULT = 2.0      # 2x damage dealt AND received
DIVINITY_MULT = 3.0   # 3x damage dealt only

# Flight (enemy power)
FLIGHT_MULT = 0.50


# =============================================================================
# OUTGOING DAMAGE CALCULATION
# =============================================================================

def calculate_damage(
    base: int,
    strength: int = 0,
    vigor: int = 0,
    weak: bool = False,
    weak_paper_crane: bool = False,
    pen_nib: bool = False,
    double_damage: bool = False,
    stance_mult: float = 1.0,
    vuln: bool = False,
    vuln_paper_frog: bool = False,
    flight: bool = False,
    intangible: bool = False,
) -> int:
    """
    Calculate final damage for an attack.

    Follows exact order from AbstractCard.calculateCardDamage():
    1. Start with base damage
    2. Add flat bonuses (Strength, Vigor)
    3. Apply attacker multipliers (Weak, Pen Nib, Double Damage)
    4. Apply stance multiplier (Wrath, Divinity)
    5. Apply defender multipliers (Vulnerable, Flight)
    6. Apply caps (Intangible)
    7. Floor to int, minimum 0

    Args:
        base: Card's base damage value
        strength: Attacker's Strength (can be negative)
        vigor: Attacker's Vigor (one-time bonus, consumed after attack)
        weak: True if attacker is Weak
        weak_paper_crane: True if attacker has Paper Crane (enemy attacking player)
        pen_nib: True if Pen Nib is active (next attack 2x)
        double_damage: True if Double Damage power is active
        stance_mult: Stance multiplier (1.0 neutral, 2.0 Wrath, 3.0 Divinity)
        vuln: True if defender is Vulnerable
        vuln_paper_frog: True if attacker has Paper Frog (attacking vuln enemy)
        flight: True if defender has Flight (50% damage reduction)
        intangible: True if defender is Intangible (cap at 1)

    Returns:
        Final damage as int (minimum 0)
    """
    # 1. Base damage
    damage = float(base)

    # 2. Flat adds (Strength, Vigor)
    damage += strength
    damage += vigor

    # 3. Attacker multipliers (order matters for Weak)
    if pen_nib:
        damage *= 2.0

    if double_damage:
        damage *= 2.0

    if weak:
        if weak_paper_crane:
            damage *= WEAK_MULT_PAPER_CRANE
        else:
            damage *= WEAK_MULT

    # 4. Stance multiplier
    damage *= stance_mult

    # 5. Defender multipliers
    if vuln:
        if vuln_paper_frog:
            damage *= VULN_MULT_PAPER_FROG
        else:
            damage *= VULN_MULT

    if flight:
        damage *= FLIGHT_MULT

    # 6. Intangible cap (applied at atDamageFinalReceive)
    if intangible and damage > 1:
        damage = 1.0

    # 7. Floor to int, minimum 0
    return max(0, int(damage))


# =============================================================================
# BLOCK CALCULATION
# =============================================================================

def calculate_block(
    base: int,
    dexterity: int = 0,
    frail: bool = False,
    no_block: bool = False,
) -> int:
    """
    Calculate final block for a card/action.

    Order from AbstractCard.applyPowersToBlock():
    1. Start with base block
    2. Add Dexterity (flat)
    3. Apply Frail (multiplicative)
    4. Apply No Block (sets to 0)
    5. Floor to int, minimum 0

    Args:
        base: Card's base block value
        dexterity: Player's Dexterity (can be negative)
        frail: True if player is Frail
        no_block: True if player has No Block power

    Returns:
        Final block as int (minimum 0)
    """
    # 1. Base block
    block = float(base)

    # 2. Add Dexterity
    block += dexterity

    # 3. Frail - 25% less (applied AFTER dex)
    if frail:
        block *= FRAIL_MULT

    # 4. No Block
    if no_block:
        block = 0.0

    # 5. Floor to int, minimum 0
    return max(0, int(block))


# =============================================================================
# INCOMING DAMAGE CALCULATION
# =============================================================================

def calculate_incoming_damage(
    damage: int,
    block: int,
    is_wrath: bool = False,
    vuln: bool = False,
    vuln_odd_mushroom: bool = False,
    intangible: bool = False,
    torii: bool = False,
    tungsten_rod: bool = False,
) -> Tuple[int, int]:
    """
    Calculate HP loss and remaining block when taking damage.

    This is for NORMAL damage (attacks). HP_LOSS bypasses block entirely.

    Order:
    1. Apply Wrath multiplier (2x incoming if in Wrath)
    2. Apply Vulnerable
    3. Apply Intangible (cap at 1)
    4. Apply Torii (damage 2-5 reduced to 1)
    5. Apply Tungsten Rod (-1 all damage)
    6. Subtract block
    7. Calculate HP loss

    IMPORTANT: Only Wrath increases incoming damage (2x).
    Divinity does NOT affect incoming damage - only outgoing.

    Args:
        damage: Incoming damage amount
        block: Current block
        is_wrath: True if receiver is in Wrath stance (doubles incoming damage)
        vuln: True if receiver is Vulnerable
        vuln_odd_mushroom: True if receiver (player) has Odd Mushroom
        intangible: True if receiver is Intangible
        torii: True if receiver has Torii relic
        tungsten_rod: True if receiver has Tungsten Rod relic

    Returns:
        Tuple of (hp_loss, block_remaining)
    """
    # 1. Start with incoming damage
    final_damage = float(damage)

    # 2. Wrath multiplier (2x incoming ONLY for Wrath, NOT Divinity)
    if is_wrath:
        final_damage *= WRATH_MULT

    # 3. Vulnerable
    if vuln:
        if vuln_odd_mushroom:
            final_damage *= VULN_MULT_ODD_MUSHROOM
        else:
            final_damage *= VULN_MULT

    # Floor before caps
    final_damage = int(final_damage)

    # 4. Intangible (cap at 1)
    if intangible and final_damage > 1:
        final_damage = 1

    # 5. Torii (damage 2-5 reduced to 1)
    # Note: Torii applies BEFORE block in the game
    if torii and 2 <= final_damage <= 5:
        final_damage = 1

    # 6. Tungsten Rod (-1 to all HP loss, applied at end)
    # Note: This is actually applied to HP loss, not blocked damage
    # But we track it here for the final HP loss calculation

    # 7. Block absorbs damage
    blocked = min(block, final_damage)
    hp_loss = final_damage - blocked
    block_remaining = block - blocked

    # Tungsten Rod reduces HP loss by 1 (minimum 0)
    if tungsten_rod and hp_loss > 0:
        hp_loss = max(0, hp_loss - 1)

    return hp_loss, block_remaining


def apply_hp_loss(
    amount: int,
    tungsten_rod: bool = False,
    intangible: bool = False,
) -> int:
    """
    Calculate actual HP loss for HP_LOSS damage type.

    HP_LOSS (poison, self-damage, etc.) ignores block but is affected by:
    - Intangible (caps at 1)
    - Tungsten Rod (-1)

    Args:
        amount: HP loss amount
        tungsten_rod: True if player has Tungsten Rod
        intangible: True if player is Intangible

    Returns:
        Actual HP to lose
    """
    hp_loss = amount

    # Intangible caps at 1
    if intangible and hp_loss > 1:
        hp_loss = 1

    # Tungsten Rod reduces by 1
    if tungsten_rod and hp_loss > 0:
        hp_loss = max(0, hp_loss - 1)

    return hp_loss


# =============================================================================
# CONVENIENCE FUNCTIONS
# =============================================================================

def wrath_damage(base: int, strength: int = 0, vuln: bool = False) -> int:
    """Quick calc for Wrath stance damage."""
    return calculate_damage(base, strength=strength, stance_mult=WRATH_MULT, vuln=vuln)


def divinity_damage(base: int, strength: int = 0, vuln: bool = False) -> int:
    """Quick calc for Divinity stance damage."""
    return calculate_damage(base, strength=strength, stance_mult=DIVINITY_MULT, vuln=vuln)


# =============================================================================
# TESTS
# =============================================================================

if __name__ == "__main__":
    print("=== Damage Calculator Tests ===\n")

    # Basic damage
    assert calculate_damage(6) == 6
    print("Strike (6 base): 6")

    # With Strength
    assert calculate_damage(6, strength=3) == 9
    print("Strike + 3 Str: 9")

    # Strength + Vigor
    assert calculate_damage(6, strength=3, vigor=5) == 14
    print("Strike + 3 Str + 5 Vigor: 14")

    # With Weak
    assert calculate_damage(10, weak=True) == 7  # 10 * 0.75 = 7.5 -> 7
    print("10 base + Weak: 7")

    # With Wrath
    assert calculate_damage(6, stance_mult=WRATH_MULT) == 12
    print("Strike in Wrath: 12")

    # Wrath + Vulnerable
    assert calculate_damage(6, stance_mult=WRATH_MULT, vuln=True) == 18
    print("Strike in Wrath vs Vuln: 18 (6*2*1.5)")

    # Full combo: Str + Wrath + Vuln
    assert calculate_damage(6, strength=3, stance_mult=WRATH_MULT, vuln=True) == 27
    print("Strike + 3 Str + Wrath + Vuln: 27 ((6+3)*2*1.5)")

    # Divinity
    assert calculate_damage(6, stance_mult=DIVINITY_MULT) == 18
    print("Strike in Divinity: 18")

    # Pen Nib
    assert calculate_damage(6, pen_nib=True) == 12
    print("Strike + Pen Nib: 12")

    # Intangible
    assert calculate_damage(100, intangible=True) == 1
    print("100 damage vs Intangible: 1")

    # Flight
    assert calculate_damage(10, flight=True) == 5
    print("10 damage vs Flight: 5")

    # Paper Frog + Vuln
    assert calculate_damage(10, vuln=True, vuln_paper_frog=True) == 17  # 10 * 1.75
    print("10 damage vs Vuln with Paper Frog: 17")

    print("\n=== Block Tests ===\n")

    # Basic block
    assert calculate_block(5) == 5
    print("Defend (5 base): 5")

    # With Dexterity
    assert calculate_block(5, dexterity=2) == 7
    print("Defend + 2 Dex: 7")

    # With Frail
    assert calculate_block(8, frail=True) == 6  # 8 * 0.75 = 6
    print("8 block + Frail: 6")

    # Dex + Frail
    assert calculate_block(5, dexterity=2, frail=True) == 5  # (5+2) * 0.75 = 5.25 -> 5
    print("5 block + 2 Dex + Frail: 5")

    # Negative Dex
    assert calculate_block(5, dexterity=-2) == 3
    print("5 block - 2 Dex: 3")

    # Negative Dex causing 0
    assert calculate_block(5, dexterity=-10) == 0
    print("5 block - 10 Dex: 0 (minimum)")

    print("\n=== Incoming Damage Tests ===\n")

    # Basic incoming
    hp, blk = calculate_incoming_damage(10, 5)
    assert hp == 5 and blk == 0
    print("10 damage, 5 block: 5 HP lost, 0 block remaining")

    # Fully blocked
    hp, blk = calculate_incoming_damage(5, 10)
    assert hp == 0 and blk == 5
    print("5 damage, 10 block: 0 HP lost, 5 block remaining")

    # Wrath incoming
    hp, blk = calculate_incoming_damage(10, 5, is_wrath=True)
    assert hp == 15 and blk == 0
    print("10 damage in Wrath, 5 block: 15 HP lost (10*2-5)")

    # Vulnerable incoming
    hp, blk = calculate_incoming_damage(10, 0, vuln=True)
    assert hp == 15
    print("10 damage vs Vuln: 15 HP lost")

    # Torii
    hp, blk = calculate_incoming_damage(4, 0, torii=True)
    assert hp == 1
    print("4 damage with Torii: 1 HP lost")

    # Torii (below threshold)
    hp, blk = calculate_incoming_damage(1, 0, torii=True)
    assert hp == 1
    print("1 damage with Torii: 1 HP lost (below threshold)")

    # Torii (above threshold)
    hp, blk = calculate_incoming_damage(10, 0, torii=True)
    assert hp == 10
    print("10 damage with Torii: 10 HP lost (above threshold)")

    # Tungsten Rod
    hp, blk = calculate_incoming_damage(10, 5, tungsten_rod=True)
    assert hp == 4
    print("10 damage, 5 block, Tungsten Rod: 4 HP lost (5-1)")

    # Intangible
    hp, blk = calculate_incoming_damage(100, 0, intangible=True)
    assert hp == 1
    print("100 damage vs Intangible: 1 HP lost")

    print("\n=== HP Loss Tests ===\n")

    # Basic HP loss
    assert apply_hp_loss(5) == 5
    print("5 HP loss: 5")

    # Intangible
    assert apply_hp_loss(10, intangible=True) == 1
    print("10 HP loss vs Intangible: 1")

    # Tungsten Rod
    assert apply_hp_loss(5, tungsten_rod=True) == 4
    print("5 HP loss with Tungsten Rod: 4")

    # Both
    assert apply_hp_loss(10, intangible=True, tungsten_rod=True) == 0
    print("10 HP loss vs Intangible + Tungsten Rod: 0 (1-1)")

    print("\n=== All tests passed! ===")
