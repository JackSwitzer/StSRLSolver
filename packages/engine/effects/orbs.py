"""
Orb System for Defect Character.

Defect's core mechanic is orbs - channeled elemental abilities that provide
passive effects each turn and evoke effects when triggered.

Orb Types:
- Lightning: Passive deals damage, Evoke deals more damage
- Frost: Passive gains block, Evoke gains more block
- Dark: Passive gains damage each turn, Evoke deals accumulated damage
- Plasma: Passive gains 1 energy, Evoke gains 2 energy

Key Mechanics:
- Orb slots: default 3, can be increased (Capacitor, Inserter, Potion of Capacity)
- Focus: increases passive and evoke values (+1 Focus = +1 to each orb's values)
- Channeling: adds orb to rightmost empty slot, evokes leftmost if full
- Evoking: triggers evoke effect and removes orb
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, TYPE_CHECKING
from enum import Enum

if TYPE_CHECKING:
    from ..state.combat import CombatState, EnemyCombatState


class OrbType(Enum):
    """Types of orbs in the game."""
    LIGHTNING = "Lightning"
    FROST = "Frost"
    DARK = "Dark"
    PLASMA = "Plasma"


@dataclass
class Orb:
    """
    An orb instance in combat.

    Attributes:
        orb_type: The type of orb (Lightning, Frost, Dark, Plasma)
        passive_amount: Base passive effect value
        evoke_amount: Base evoke effect value (for Dark orbs, this accumulates)
    """
    orb_type: OrbType
    passive_amount: int
    evoke_amount: int

    # For Dark orbs, track accumulated damage
    accumulated_damage: int = 0

    def copy(self) -> Orb:
        """Create a copy of this orb."""
        orb = Orb(
            orb_type=self.orb_type,
            passive_amount=self.passive_amount,
            evoke_amount=self.evoke_amount,
            accumulated_damage=self.accumulated_damage
        )
        return orb


# Base orb values
ORB_BASE_VALUES = {
    OrbType.LIGHTNING: {"passive": 3, "evoke": 8},
    OrbType.FROST: {"passive": 2, "evoke": 5},
    OrbType.DARK: {"passive": 6, "evoke": 0},  # Dark starts at 0, accumulates
    OrbType.PLASMA: {"passive": 1, "evoke": 2},
}


def create_orb(orb_type: OrbType) -> Orb:
    """Create a new orb of the specified type."""
    base = ORB_BASE_VALUES[orb_type]
    if orb_type == OrbType.DARK:
        # Dark orbs start with 6 accumulated damage
        return Orb(
            orb_type=orb_type,
            passive_amount=base["passive"],
            evoke_amount=base["evoke"],
            accumulated_damage=6
        )
    return Orb(
        orb_type=orb_type,
        passive_amount=base["passive"],
        evoke_amount=base["evoke"]
    )


class OrbManager:
    """
    Manages orbs for a Defect combat.

    Handles channeling, evoking, passive triggers, and focus modifications.
    """

    def __init__(self, max_slots: int = 3):
        """Initialize orb manager with default 3 slots."""
        self.orbs: List[Orb] = []
        self.max_slots: int = max_slots
        self.focus: int = 0

        # Track orb stats for card effects
        self.lightning_channeled: int = 0
        self.frost_channeled: int = 0
        self.dark_channeled: int = 0
        self.plasma_channeled: int = 0

        # For Electrodynamics - lightning hits all enemies
        self.lightning_hits_all: bool = False

        # For Loop - extra passive triggers for rightmost orb
        self.loop_stacks: int = 0

    def copy(self) -> OrbManager:
        """Create a copy of this orb manager."""
        manager = OrbManager(self.max_slots)
        manager.orbs = [orb.copy() for orb in self.orbs]
        manager.focus = self.focus
        manager.lightning_channeled = self.lightning_channeled
        manager.frost_channeled = self.frost_channeled
        manager.dark_channeled = self.dark_channeled
        manager.plasma_channeled = self.plasma_channeled
        manager.lightning_hits_all = self.lightning_hits_all
        manager.loop_stacks = self.loop_stacks
        return manager

    def channel(self, orb_type: OrbType, state: CombatState) -> Dict[str, Any]:
        """
        Channel an orb.

        If slots are full, evokes the leftmost orb first.

        Returns:
            Dict with channel results including any evoke that occurred.
        """
        result = {"channeled": orb_type.value, "evoked": None, "evoke_result": None}

        # Track channeled orbs for card effects
        if orb_type == OrbType.LIGHTNING:
            self.lightning_channeled += 1
        elif orb_type == OrbType.FROST:
            self.frost_channeled += 1
        elif orb_type == OrbType.DARK:
            self.dark_channeled += 1
        elif orb_type == OrbType.PLASMA:
            self.plasma_channeled += 1

        # If full, evoke leftmost
        if len(self.orbs) >= self.max_slots:
            evoke_result = self.evoke(state)
            result["evoked"] = evoke_result.get("orb_type")
            result["evoke_result"] = evoke_result

        # Create and add new orb
        orb = create_orb(orb_type)
        self.orbs.append(orb)

        return result

    def evoke(self, state: CombatState, times: int = 1) -> Dict[str, Any]:
        """
        Evoke the leftmost orb (multiple times if specified).

        Returns:
            Dict with evoke results.
        """
        if not self.orbs:
            return {"evoked": False, "orb_type": None, "effect": None}

        orb = self.orbs[0]
        result = {
            "evoked": True,
            "orb_type": orb.orb_type.value,
            "effect": None,
            "times": times
        }

        # Execute evoke effect (potentially multiple times for Multi-Cast)
        for _ in range(times):
            effect_result = self._execute_evoke(orb, state)
            if result["effect"] is None:
                result["effect"] = effect_result
            else:
                # Aggregate results
                for key in ["damage", "block", "energy"]:
                    if key in effect_result:
                        result["effect"][key] = result["effect"].get(key, 0) + effect_result[key]

        # Remove the orb
        self.orbs.pop(0)

        return result

    def evoke_all(self, state: CombatState, gain_resources: bool = True) -> Dict[str, Any]:
        """
        Evoke all orbs (for Fission).

        Args:
            state: Combat state
            gain_resources: If True, gain energy and draw per orb (Fission+)

        Returns:
            Dict with results
        """
        result = {
            "orbs_evoked": len(self.orbs),
            "energy_gained": 0,
            "cards_drawn": 0,
            "total_damage": 0,
            "total_block": 0
        }

        while self.orbs:
            evoke_result = self.evoke(state)
            effect = evoke_result.get("effect", {})
            result["total_damage"] += effect.get("damage", 0)
            result["total_block"] += effect.get("block", 0)

            if gain_resources:
                result["energy_gained"] += 1
                result["cards_drawn"] += 1

        return result

    def _execute_evoke(self, orb: Orb, state: CombatState) -> Dict[str, Any]:
        """Execute the evoke effect of an orb."""
        result = {}
        focus = self.focus

        if orb.orb_type == OrbType.LIGHTNING:
            # Deal 8 + Focus damage to random enemy (or all if Electrodynamics)
            damage = orb.evoke_amount + focus
            result["damage"] = max(0, damage)

            if self.lightning_hits_all:
                # Deal to all enemies
                for enemy in state.get_living_enemies():
                    self._deal_damage_to_enemy(enemy, result["damage"], state)
            else:
                # Deal to random enemy
                living = state.get_living_enemies()
                if living:
                    target = _random_enemy_from_state(state, living)
                    self._deal_damage_to_enemy(target, result["damage"], state)

        elif orb.orb_type == OrbType.FROST:
            # Gain 5 + Focus block
            block = orb.evoke_amount + focus
            result["block"] = max(0, block)
            state.player.block += result["block"]

        elif orb.orb_type == OrbType.DARK:
            # Deal accumulated damage to lowest HP enemy
            damage = orb.accumulated_damage
            result["damage"] = damage

            living = state.get_living_enemies()
            if living:
                # Target enemy with lowest HP
                target = min(living, key=lambda e: e.hp)
                self._deal_damage_to_enemy(target, damage, state)

        elif orb.orb_type == OrbType.PLASMA:
            # Gain 2 energy
            energy = orb.evoke_amount
            result["energy"] = energy
            state.energy += energy

        return result

    def trigger_passives(self, state: CombatState) -> Dict[str, Any]:
        """
        Trigger all orb passive effects.

        Called at end of turn.

        Returns:
            Dict with results from all passive triggers.
        """
        result = {
            "total_damage": 0,
            "total_block": 0,
            "total_energy": 0,
            "dark_accumulated": 0
        }

        focus = self.focus

        for i, orb in enumerate(self.orbs):
            # Check for Loop (extra triggers on rightmost orb)
            extra_triggers = self.loop_stacks if i == len(self.orbs) - 1 else 0
            triggers = 1 + extra_triggers

            for _ in range(triggers):
                passive_result = self._execute_passive(orb, state, focus)
                result["total_damage"] += passive_result.get("damage", 0)
                result["total_block"] += passive_result.get("block", 0)
                result["total_energy"] += passive_result.get("energy", 0)
                result["dark_accumulated"] += passive_result.get("accumulated", 0)

        return result

    def _execute_passive(self, orb: Orb, state: CombatState, focus: int) -> Dict[str, Any]:
        """Execute the passive effect of an orb."""
        result = {}

        if orb.orb_type == OrbType.LIGHTNING:
            # Deal 3 + Focus damage to random enemy (or all if Electrodynamics)
            damage = orb.passive_amount + focus
            result["damage"] = max(0, damage)

            if self.lightning_hits_all:
                for enemy in state.get_living_enemies():
                    self._deal_damage_to_enemy(enemy, result["damage"], state)
            else:
                living = state.get_living_enemies()
                if living:
                    target = _random_enemy_from_state(state, living)
                    self._deal_damage_to_enemy(target, result["damage"], state)

        elif orb.orb_type == OrbType.FROST:
            # Gain 2 + Focus block
            block = orb.passive_amount + focus
            result["block"] = max(0, block)
            state.player.block += result["block"]

        elif orb.orb_type == OrbType.DARK:
            # Accumulate 6 + Focus damage (does NOT deal damage passively)
            accumulate = orb.passive_amount + focus
            orb.accumulated_damage += max(0, accumulate)
            result["accumulated"] = max(0, accumulate)

        elif orb.orb_type == OrbType.PLASMA:
            # Gain 1 energy
            energy = orb.passive_amount
            result["energy"] = energy
            state.energy += energy

        return result

    def _deal_damage_to_enemy(self, enemy: EnemyCombatState, amount: int, state: CombatState) -> int:
        """Deal damage to an enemy, accounting for Lock-On."""
        if amount <= 0:
            return 0

        # Apply Lock-On multiplier (50% more damage from orbs)
        lockon = enemy.statuses.get("Lock-On", 0)
        if lockon > 0:
            amount = int(amount * 1.5)

        # Block absorbs damage
        blocked = min(enemy.block, amount)
        enemy.block -= blocked
        hp_damage = amount - blocked

        # Apply HP damage
        enemy.hp -= hp_damage
        if enemy.hp < 0:
            enemy.hp = 0

        return hp_damage

    def get_unique_orb_types(self) -> int:
        """Get count of unique orb types currently channeled."""
        return len(set(orb.orb_type for orb in self.orbs))

    def get_orb_count(self) -> int:
        """Get total number of orbs channeled."""
        return len(self.orbs)

    def has_orbs(self) -> bool:
        """Check if any orbs are channeled."""
        return len(self.orbs) > 0

    def get_first_orb(self) -> Optional[Orb]:
        """Get the leftmost (first to be evoked) orb."""
        return self.orbs[0] if self.orbs else None

    def get_last_orb(self) -> Optional[Orb]:
        """Get the rightmost orb."""
        return self.orbs[-1] if self.orbs else None

    def add_orb_slot(self, amount: int = 1) -> None:
        """Increase max orb slots."""
        self.max_slots += amount

    def remove_orb_slot(self, amount: int = 1, state: Optional['CombatState'] = None) -> None:
        """Decrease max orb slots (minimum 0) and drop newest orbs past cap."""
        self.max_slots = max(0, self.max_slots - amount)
        while len(self.orbs) > self.max_slots:
            # Java decreaseMaxOrbSlots removes the newest orb slot contents.
            self.orbs.pop()

    def modify_focus(self, amount: int) -> None:
        """Modify focus amount."""
        self.focus += amount

    def has_empty_slot(self) -> bool:
        """Whether at least one orb slot is currently empty."""
        return len(self.orbs) < self.max_slots


def _get_orb_rng(state: CombatState):
    """RNG stream for orb randomization/target selection."""
    return (
        getattr(state, "card_random_rng", None)
        or getattr(state, "card_rng", None)
    )


def _random_enemy_from_state(state: CombatState, living: List['EnemyCombatState']) -> 'EnemyCombatState':
    """Deterministic random enemy choice using owned RNG streams."""
    if len(living) == 1:
        return living[0]
    rng = _get_orb_rng(state)
    if rng is None:
        return living[0]
    idx = rng.random(len(living) - 1)
    return living[idx]


def get_orb_manager(state: CombatState) -> OrbManager:
    """Get or create the orb manager for a combat state."""
    if not hasattr(state, 'orb_manager') or state.orb_manager is None:
        # Check for orb slot bonuses from relics/powers
        base_slots = 3
        orb_slot_bonus = state.player.statuses.get("OrbSlots", 0)
        state.orb_manager = OrbManager(base_slots + orb_slot_bonus)

        # Check for Focus from statuses
        focus = state.player.statuses.get("Focus", 0)
        state.orb_manager.focus = focus
    else:
        # Keep focus and slot count synchronized with combat statuses.
        state.orb_manager.focus = state.player.statuses.get("Focus", 0)
        desired_slots = max(0, 3 + state.player.statuses.get("OrbSlots", 0))
        if desired_slots > state.orb_manager.max_slots:
            state.orb_manager.add_orb_slot(desired_slots - state.orb_manager.max_slots)
        elif desired_slots < state.orb_manager.max_slots:
            state.orb_manager.remove_orb_slot(state.orb_manager.max_slots - desired_slots)

    return state.orb_manager


def channel_orb(state: CombatState, orb_type: str) -> Dict[str, Any]:
    """
    Channel an orb by type name.

    Args:
        state: Combat state
        orb_type: "Lightning", "Frost", "Dark", or "Plasma"

    Returns:
        Channel result dict
    """
    manager = get_orb_manager(state)
    orb_enum = OrbType(orb_type)
    return manager.channel(orb_enum, state)


def channel_random_orb(state: CombatState) -> Dict[str, Any]:
    """Channel a random orb type."""
    orb_types = list(OrbType)
    rng = _get_orb_rng(state)
    if rng is None:
        orb_type = orb_types[0]
    else:
        orb_type = orb_types[rng.random(len(orb_types) - 1)]
    manager = get_orb_manager(state)
    return manager.channel(orb_type, state)


def evoke_orb(state: CombatState, times: int = 1) -> Dict[str, Any]:
    """Evoke the leftmost orb."""
    manager = get_orb_manager(state)
    return manager.evoke(state, times)


def evoke_all_orbs(state: CombatState, gain_resources: bool = False) -> Dict[str, Any]:
    """Evoke all orbs (Fission)."""
    manager = get_orb_manager(state)
    return manager.evoke_all(state, gain_resources)


def trigger_orb_passives(state: CombatState) -> Dict[str, Any]:
    """Trigger all orb passives at end of turn."""
    manager = get_orb_manager(state)
    return manager.trigger_passives(state)


def trigger_first_orb_passive(state: CombatState) -> Dict[str, Any]:
    """Trigger the first orb's passive once (used by Gold-Plated Cables)."""
    manager = get_orb_manager(state)
    if not manager.orbs:
        return {}
    return manager._execute_passive(manager.orbs[0], state, manager.focus)


def _trigger_single_orb_passive_cycle(
    manager: OrbManager,
    state: CombatState,
    *,
    include_cables: bool = False,
) -> Dict[str, Any]:
    """Trigger one onStart/onEnd-equivalent passive cycle across all current orbs."""
    result = {
        "total_damage": 0,
        "total_block": 0,
        "total_energy": 0,
        "dark_accumulated": 0,
    }
    for orb in list(manager.orbs):
        passive_result = manager._execute_passive(orb, state, manager.focus)
        result["total_damage"] += passive_result.get("damage", 0)
        result["total_block"] += passive_result.get("block", 0)
        result["total_energy"] += passive_result.get("energy", 0)
        result["dark_accumulated"] += passive_result.get("accumulated", 0)

    if include_cables and state.has_relic("Cables") and manager.orbs:
        passive_result = manager._execute_passive(manager.orbs[0], state, manager.focus)
        result["total_damage"] += passive_result.get("damage", 0)
        result["total_block"] += passive_result.get("block", 0)
        result["total_energy"] += passive_result.get("energy", 0)
        result["dark_accumulated"] += passive_result.get("accumulated", 0)

    return result


def trigger_orb_start_of_turn(state: CombatState, *, include_cables: bool = False) -> Dict[str, Any]:
    """Trigger one start-of-turn orb passive cycle."""
    manager = get_orb_manager(state)
    return _trigger_single_orb_passive_cycle(manager, state, include_cables=include_cables)


def trigger_orb_start_end(state: CombatState, *, include_cables: bool = False) -> Dict[str, Any]:
    """Trigger combined onStartOfTurn+onEndOfTurn orb behavior once."""
    manager = get_orb_manager(state)
    first = _trigger_single_orb_passive_cycle(manager, state, include_cables=include_cables)
    second = _trigger_single_orb_passive_cycle(manager, state, include_cables=include_cables)
    return {
        "total_damage": first["total_damage"] + second["total_damage"],
        "total_block": first["total_block"] + second["total_block"],
        "total_energy": first["total_energy"] + second["total_energy"],
        "dark_accumulated": first["dark_accumulated"] + second["dark_accumulated"],
    }
