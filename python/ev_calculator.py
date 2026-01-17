"""
EV Calculator for Slay the Spire

Accurate damage/EV calculations matching game mechanics.
Uses NumPy for vectorized operations when dealing with multiple scenarios.

Formulas from decompiled game source (docs/vault/damage-mechanics.md):
- Damage = floor((base + strength) * weak * vulnerable * stance)
- Block = floor((base + dexterity) * frail)
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple
from enum import Enum

# ============ CONSTANTS ============

class Stance(Enum):
    NEUTRAL = "Neutral"
    CALM = "Calm"
    WRATH = "Wrath"
    DIVINITY = "Divinity"

# Relic modifier constants (from decompiled code)
VULN_NORMAL = 1.5
VULN_ODD_MUSHROOM = 1.25  # Player takes less from vulnerable
VULN_PAPER_FROG = 1.75    # Enemies take more from vulnerable

WEAK_NORMAL = 0.75
WEAK_PAPER_CRANE = 0.6    # Enemies deal even less when weak

STANCE_DAMAGE_MULT = {
    Stance.NEUTRAL: 1.0,
    Stance.CALM: 1.0,
    Stance.WRATH: 2.0,
    Stance.DIVINITY: 3.0,
}

STANCE_DAMAGE_RECEIVED_MULT = {
    Stance.NEUTRAL: 1.0,
    Stance.CALM: 1.0,
    Stance.WRATH: 2.0,   # Also doubles incoming!
    Stance.DIVINITY: 1.0,
}

# ============ DATA CLASSES ============

@dataclass
class PowerState:
    """Represents a power/buff/debuff on a creature."""
    id: str
    amount: int = 0

@dataclass
class CreatureState:
    """Base state for player or monster."""
    hp: int
    max_hp: int
    block: int = 0
    strength: int = 0
    dexterity: int = 0
    vulnerable: int = 0
    weak: int = 0
    intangible: int = 0
    powers: List[PowerState] = field(default_factory=list)

@dataclass
class PlayerState(CreatureState):
    """Player-specific state."""
    stance: Stance = Stance.NEUTRAL
    energy: int = 0
    has_odd_mushroom: bool = False
    has_paper_frog: bool = False
    has_paper_crane: bool = False
    has_pen_nib: bool = False
    pen_nib_counter: int = 0

@dataclass
class MonsterState(CreatureState):
    """Monster state including intent."""
    id: str = ""
    name: str = ""
    intent_damage: int = 0
    intent_multi: int = 1
    intent_type: str = "UNKNOWN"

@dataclass
class CardState:
    """Card state for damage calculation."""
    id: str
    name: str
    card_type: str  # ATTACK, SKILL, POWER
    base_damage: int = 0
    base_block: int = 0
    cost: int = 0
    exhausts: bool = False
    target_type: str = "ENEMY"  # ENEMY, ALL_ENEMY, SELF, etc.

@dataclass
class CombatState:
    """Full combat state for EV calculation."""
    player: PlayerState
    monsters: List[MonsterState]
    hand: List[CardState]
    draw_pile_size: int = 0
    discard_pile_size: int = 0
    turn: int = 1

# ============ DAMAGE CALCULATIONS ============

def calculate_player_attack_damage(
    base_damage: int,
    player: PlayerState,
    target: MonsterState,
    is_first_attack: bool = False
) -> int:
    """
    Calculate damage player deals to a monster.

    Order of operations (from DamageInfo.applyPowers):
    1. Add Strength (additive)
    2. Apply Weak (if player weak)
    3. Apply stance multiplier
    4. Apply Vulnerable on target
    5. Apply relic multipliers (Pen Nib)
    6. Floor and cap at 0
    """
    damage = float(base_damage)

    # 1. Add Strength
    damage += player.strength

    # 2. Apply Weak (player is weak = deals less damage)
    if player.weak > 0:
        damage = np.floor(damage * WEAK_NORMAL)

    # 3. Apply stance multiplier
    stance_mult = STANCE_DAMAGE_MULT.get(player.stance, 1.0)
    damage = np.floor(damage * stance_mult)

    # 4. Apply Vulnerable on target
    if target.vulnerable > 0:
        vuln_mult = VULN_PAPER_FROG if player.has_paper_frog else VULN_NORMAL
        damage = np.floor(damage * vuln_mult)

    # 5. Relic multipliers
    if player.has_pen_nib and player.pen_nib_counter == 9 and is_first_attack:
        damage *= 2.0

    return max(0, int(damage))


def calculate_incoming_damage(
    monster: MonsterState,
    player: PlayerState
) -> int:
    """
    Calculate damage player would take from a monster's attack.

    Order:
    1. Apply Weak on monster (reduces their damage)
    2. Apply Vulnerable on player (increases damage taken)
    3. Apply Wrath stance (doubles incoming)
    4. Multiply by multi-hit count
    """
    if monster.intent_damage <= 0:
        return 0

    damage = float(monster.intent_damage)

    # 1. Monster is weak
    if monster.weak > 0:
        weak_mult = WEAK_PAPER_CRANE if player.has_paper_crane else WEAK_NORMAL
        damage = np.floor(damage * weak_mult)

    # 2. Player is vulnerable
    if player.vulnerable > 0:
        vuln_mult = VULN_ODD_MUSHROOM if player.has_odd_mushroom else VULN_NORMAL
        damage = np.floor(damage * vuln_mult)

    # 3. Wrath stance doubles incoming
    incoming_mult = STANCE_DAMAGE_RECEIVED_MULT.get(player.stance, 1.0)
    damage = np.floor(damage * incoming_mult)

    # 4. Multi-hit
    total_damage = int(damage) * max(1, monster.intent_multi)

    return max(0, total_damage)


def calculate_total_incoming_damage(combat: CombatState) -> int:
    """Calculate total incoming damage from all monsters."""
    total = 0
    for monster in combat.monsters:
        if monster.hp > 0:
            total += calculate_incoming_damage(monster, combat.player)
    return total


def calculate_net_damage(combat: CombatState) -> int:
    """Calculate damage after block is applied."""
    incoming = calculate_total_incoming_damage(combat)
    return max(0, incoming - combat.player.block)


def calculate_block_gained(base_block: int, player: PlayerState) -> int:
    """
    Calculate actual block gained from a card.

    Block = floor((base + dexterity) * frail_mult)
    """
    block = float(base_block + player.dexterity)

    # Check for Frail power
    frail = next((p.amount for p in player.powers if p.id == "Frail"), 0)
    if frail > 0:
        block = np.floor(block * 0.75)

    return max(0, int(block))

# ============ EV CALCULATIONS ============

def calculate_hand_potential_damage(combat: CombatState) -> int:
    """Calculate total potential damage from playing all attacks in hand."""
    total_damage = 0
    energy = combat.player.energy
    is_first = True

    # Simple greedy: play all affordable attacks
    for card in sorted(combat.hand, key=lambda c: -c.base_damage):  # Highest damage first
        if card.card_type == "ATTACK" and card.cost <= energy:
            # Calculate against first alive monster
            target = next((m for m in combat.monsters if m.hp > 0), None)
            if target:
                damage = calculate_player_attack_damage(
                    card.base_damage, combat.player, target, is_first
                )

                # AoE multiplier
                if card.target_type in ("ALL_ENEMY", "ALL"):
                    alive_count = sum(1 for m in combat.monsters if m.hp > 0)
                    damage *= alive_count

                total_damage += damage
                energy -= card.cost
                is_first = False

    return total_damage


def calculate_turns_to_kill(combat: CombatState) -> Tuple[float, int]:
    """
    Calculate estimated turns to kill all monsters.

    Returns: (turns_to_kill, expected_damage_taken)
    """
    total_enemy_hp = sum(m.hp + m.block for m in combat.monsters if m.hp > 0)
    damage_per_turn = calculate_hand_potential_damage(combat)

    if damage_per_turn <= 0:
        return float('inf'), float('inf')

    turns_to_kill = total_enemy_hp / damage_per_turn
    damage_per_turn_incoming = calculate_total_incoming_damage(combat)
    expected_damage = int(np.ceil(turns_to_kill) * damage_per_turn_incoming)

    return turns_to_kill, expected_damage


def calculate_turn_ev(combat: CombatState) -> float:
    """
    Calculate Expected Value for the current turn.

    EV = -expected_hp_loss

    A negative EV means we expect to lose HP this turn.
    """
    net_damage = calculate_net_damage(combat)
    return -float(net_damage)


def calculate_combat_ev(combat: CombatState) -> float:
    """
    Calculate Expected Value for the entire remaining combat.

    EV = -expected_total_hp_loss
    """
    turns, expected_damage = calculate_turns_to_kill(combat)

    if turns == float('inf'):
        return float('-inf')

    # Account for block we'll generate
    avg_block_per_turn = estimate_avg_block_per_turn(combat)
    adjusted_damage = max(0, expected_damage - int(np.ceil(turns) * avg_block_per_turn))

    return -float(adjusted_damage)


def estimate_avg_block_per_turn(combat: CombatState) -> float:
    """Estimate average block generated per turn from hand."""
    total_block = 0
    energy = combat.player.energy

    for card in combat.hand:
        if card.card_type == "SKILL" and card.base_block > 0 and card.cost <= energy:
            total_block += calculate_block_gained(card.base_block, combat.player)
            energy -= card.cost

    return float(total_block)

# ============ LETHAL DETECTION ============

def can_lethal_this_turn(combat: CombatState) -> bool:
    """Check if we can kill all enemies this turn."""
    total_enemy_hp = sum(m.hp + m.block for m in combat.monsters if m.hp > 0)
    potential_damage = calculate_hand_potential_damage(combat)
    return potential_damage >= total_enemy_hp


def should_enter_wrath(combat: CombatState) -> bool:
    """
    Determine if entering Wrath is safe.

    Safe to enter Wrath if:
    1. Can lethal this turn, OR
    2. No enemies attacking, OR
    3. Have enough block to survive doubled damage
    """
    if combat.player.stance == Stance.WRATH:
        return True  # Already in Wrath

    # Check lethal
    if can_lethal_this_turn(combat):
        return True

    # Check if enemies attacking
    total_incoming = calculate_total_incoming_damage(combat)
    if total_incoming == 0:
        return True  # No attacks coming

    # Check if block covers doubled damage
    # (When we enter Wrath, incoming will double)
    doubled_incoming = total_incoming * 2
    if combat.player.block >= doubled_incoming:
        return True

    return False


def calculate_wrath_ev_delta(combat: CombatState) -> float:
    """
    Calculate EV change from entering Wrath.

    Wrath doubles damage dealt AND received.

    EV_delta = 2x_damage_dealt_value - 2x_damage_taken_value
    """
    potential_damage = calculate_hand_potential_damage(combat)
    incoming_damage = calculate_total_incoming_damage(combat)
    block = combat.player.block

    # Without Wrath
    net_without = max(0, incoming_damage - block)

    # With Wrath (doubles both)
    potential_with = potential_damage * 2
    incoming_with = incoming_damage * 2
    net_with = max(0, incoming_with - block)

    # EV delta: positive means Wrath is better
    # Value of damage dealt vs cost of damage taken
    # Simplified: extra damage dealt - extra damage taken
    damage_gain = potential_with - potential_damage
    damage_cost = net_with - net_without

    return float(damage_gain - damage_cost)

# ============ VECTORIZED OPERATIONS ============

def batch_calculate_incoming_damage(
    intent_damages: np.ndarray,
    intent_multis: np.ndarray,
    monster_weak: np.ndarray,
    player_vulnerable: int,
    player_stance: Stance,
    has_paper_crane: bool = False,
    has_odd_mushroom: bool = False
) -> np.ndarray:
    """
    Vectorized incoming damage calculation for multiple monsters.

    Args:
        intent_damages: Array of base intent damages
        intent_multis: Array of multi-hit counts
        monster_weak: Array of weak stacks on each monster
        player_vulnerable: Player's vulnerable stacks
        player_stance: Player's current stance
        has_paper_crane: Player has Paper Crane relic
        has_odd_mushroom: Player has Odd Mushroom relic

    Returns:
        Array of calculated incoming damages
    """
    damages = intent_damages.astype(float)

    # Apply weak on monsters
    weak_mult = np.where(
        monster_weak > 0,
        WEAK_PAPER_CRANE if has_paper_crane else WEAK_NORMAL,
        1.0
    )
    damages = np.floor(damages * weak_mult)

    # Apply player vulnerable
    if player_vulnerable > 0:
        vuln_mult = VULN_ODD_MUSHROOM if has_odd_mushroom else VULN_NORMAL
        damages = np.floor(damages * vuln_mult)

    # Apply Wrath stance
    if player_stance == Stance.WRATH:
        damages = np.floor(damages * 2.0)

    # Apply multi-hit
    total_damages = damages * np.maximum(1, intent_multis)

    return np.maximum(0, total_damages).astype(int)


def batch_calculate_card_damage(
    base_damages: np.ndarray,
    player_strength: int,
    player_weak: int,
    player_stance: Stance,
    target_vulnerable: np.ndarray,
    has_paper_frog: bool = False
) -> np.ndarray:
    """
    Vectorized card damage calculation.

    Useful for evaluating all cards in hand against all monsters.
    """
    damages = base_damages.astype(float) + player_strength

    # Apply player weak
    if player_weak > 0:
        damages = np.floor(damages * WEAK_NORMAL)

    # Apply stance
    stance_mult = STANCE_DAMAGE_MULT.get(player_stance, 1.0)
    damages = np.floor(damages * stance_mult)

    # Apply vulnerable on targets
    vuln_mult = np.where(
        target_vulnerable > 0,
        VULN_PAPER_FROG if has_paper_frog else VULN_NORMAL,
        1.0
    )
    damages = np.floor(damages * vuln_mult)

    return np.maximum(0, damages).astype(int)

# ============ UTILITY FUNCTIONS ============

def parse_combat_state(state_dict: Dict) -> CombatState:
    """Parse combat state from JSON dictionary (from EVTracker mod)."""

    player_state = state_dict.get("player_state", {})
    damage_mods = state_dict.get("damage_modifiers", {})

    player = PlayerState(
        hp=player_state.get("hp", 0),
        max_hp=player_state.get("max_hp", 0),
        block=player_state.get("block", 0),
        strength=player_state.get("strength", 0),
        dexterity=player_state.get("dexterity", 0),
        vulnerable=player_state.get("vulnerable", 0),
        weak=player_state.get("weak", 0),
        intangible=player_state.get("intangible", 0),
        stance=Stance(player_state.get("stance", "Neutral")),
        energy=state_dict.get("energy", 0),
        has_odd_mushroom=damage_mods.get("has_odd_mushroom", False),
        has_paper_frog=damage_mods.get("has_paper_frog", False),
        has_paper_crane=damage_mods.get("has_paper_crane", False),
        has_pen_nib=damage_mods.get("has_pen_nib", False),
        pen_nib_counter=damage_mods.get("pen_nib_counter", 0),
    )

    monsters = []
    for m in state_dict.get("monsters", []):
        monsters.append(MonsterState(
            id=m.get("id", ""),
            name=m.get("name", ""),
            hp=m.get("hp", 0),
            max_hp=m.get("max_hp", 0),
            block=m.get("block", 0),
            strength=m.get("strength", 0),
            vulnerable=m.get("vulnerable", 0),
            weak=m.get("weak", 0),
            intent_damage=m.get("intent_base_damage", -1),
            intent_multi=m.get("intent_multi", 1),
            intent_type=m.get("intent", "UNKNOWN"),
        ))

    hand = []
    for c in state_dict.get("hand_post_draw", state_dict.get("hand", [])):
        hand.append(CardState(
            id=c.get("id", ""),
            name=c.get("name", ""),
            card_type=c.get("type", "SKILL"),
            base_damage=c.get("base_damage", 0),
            base_block=c.get("base_block", 0),
            cost=c.get("cost", 0),
            exhausts=c.get("exhausts", False),
            target_type=c.get("target_type", "ENEMY"),
        ))

    return CombatState(
        player=player,
        monsters=monsters,
        hand=hand,
        draw_pile_size=state_dict.get("draw_pile_size", 0),
        discard_pile_size=state_dict.get("discard_pile_size", 0),
        turn=state_dict.get("turn", 1),
    )


def calculate_full_ev_report(combat: CombatState) -> Dict:
    """
    Generate a full EV report for the current combat state.
    """
    turns_to_kill, expected_damage = calculate_turns_to_kill(combat)

    return {
        "turn_ev": calculate_turn_ev(combat),
        "combat_ev": calculate_combat_ev(combat),
        "incoming_damage": calculate_total_incoming_damage(combat),
        "net_damage": calculate_net_damage(combat),
        "potential_damage": calculate_hand_potential_damage(combat),
        "turns_to_kill": turns_to_kill,
        "expected_damage_taken": expected_damage,
        "can_lethal": can_lethal_this_turn(combat),
        "wrath_safe": should_enter_wrath(combat),
        "wrath_ev_delta": calculate_wrath_ev_delta(combat),
        "player_stance": combat.player.stance.value,
        "player_strength": combat.player.strength,
        "player_vulnerable": combat.player.vulnerable,
        "player_weak": combat.player.weak,
    }


# ============ TESTING ============

if __name__ == "__main__":
    # Test with sample state
    player = PlayerState(
        hp=50, max_hp=70, block=5,
        strength=2, dexterity=0,
        vulnerable=0, weak=0, intangible=0,
        stance=Stance.NEUTRAL,
        energy=3,
        has_paper_frog=True,
    )

    monsters = [
        MonsterState(
            id="Cultist", name="Cultist",
            hp=30, max_hp=50, block=0,
            strength=0, vulnerable=2, weak=0,
            intent_damage=6, intent_multi=1, intent_type="ATTACK"
        ),
    ]

    hand = [
        CardState(id="Strike_P", name="Strike", card_type="ATTACK", base_damage=6, cost=1),
        CardState(id="Strike_P", name="Strike", card_type="ATTACK", base_damage=6, cost=1),
        CardState(id="Defend_P", name="Defend", card_type="SKILL", base_block=5, cost=1),
        CardState(id="Eruption", name="Eruption", card_type="ATTACK", base_damage=9, cost=2),
    ]

    combat = CombatState(
        player=player,
        monsters=monsters,
        hand=hand,
        turn=1,
    )

    report = calculate_full_ev_report(combat)

    print("=== EV Report ===")
    for k, v in report.items():
        print(f"  {k}: {v}")

    # Test damage calculation
    print("\n=== Damage Calculations ===")
    for card in hand:
        if card.card_type == "ATTACK":
            damage = calculate_player_attack_damage(
                card.base_damage, player, monsters[0]
            )
            print(f"  {card.name}: {card.base_damage} base -> {damage} calculated")
            print(f"    (Str +{player.strength}, Target Vuln x{VULN_PAPER_FROG})")
