"""
Kill Calculator - Determines if current hand can kill enemies and finds optimal sequences.

Core questions answered:
- can_kill_all(): Can we kill all enemies this turn?
- can_kill_priority(): Can we kill the most dangerous enemy?
- get_kill_sequence(): What's the optimal card sequence?
- get_kill_stats(): Full analysis with overkill, energy remaining, HP cost

This is DETERMINISTIC math, not behavioral cloning. We model the game, not humans.
"""

from typing import List, Dict, Tuple, Optional, Set
from dataclasses import dataclass, field
from copy import deepcopy
import itertools

from .line_evaluator import (
    SimulatedPlayer, SimulatedEnemy, LineOutcome, LineSimulator, CARD_EFFECTS
)
from .enemy_database import get_enemy_info, ENEMIES


@dataclass
class Action:
    """Single action in a kill sequence."""
    card_id: str
    target_id: Optional[int]
    damage: int = 0
    block: int = 0
    energy_cost: int = 0
    stance_change: Optional[str] = None


@dataclass
class EnemyKillInfo:
    """Kill analysis for a single enemy."""
    enemy_id: int
    enemy_name: str
    current_hp: int
    current_block: int

    can_kill: bool
    damage_needed: int           # HP + block to kill
    damage_available: int        # Max we can deal to this target

    threat_level: float          # From enemy_database (0-1)
    incoming_damage: int         # What this enemy will deal
    is_priority: bool            # Should we kill this first?
    is_scaling: bool            # Enemy gets stronger over time


@dataclass
class KillStats:
    """Complete kill analysis for current game state."""

    # Core answers
    can_kill_all: bool = False
    can_kill_priority: bool = False
    guaranteed_kill: bool = False    # No RNG involved

    # Kill sequence
    optimal_sequence: List[Action] = field(default_factory=list)
    sequence_damage: int = 0
    sequence_block: int = 0

    # Resource accounting
    overkill_damage: int = 0
    energy_remaining: int = 0
    hp_cost: int = 0                 # Damage we take after our play
    final_stance: str = "Neutral"

    # Multi-target breakdown
    enemy_kill_info: Dict[int, EnemyKillInfo] = field(default_factory=dict)
    priority_target_id: Optional[int] = None
    enemies_killed: int = 0

    # Alternative sequences
    safest_kill_sequence: List[Action] = field(default_factory=list)  # Minimize HP loss
    fastest_kill_sequence: List[Action] = field(default_factory=list)  # Minimize cards

    # For wrath optimization
    requires_wrath: bool = False
    safe_wrath_available: bool = False  # Can enter AND exit same turn


class KillCalculator:
    """
    Main kill calculation interface.

    Uses LineSimulator for damage math, adds:
    - Exhaustive search for kill sequences
    - Priority targeting based on threat
    - Watcher stance optimization
    - Resource accounting
    """

    def __init__(self, simulator: Optional[LineSimulator] = None):
        self.simulator = simulator or LineSimulator()

        # Cards that enter wrath
        self.wrath_entry = {
            "Eruption", "Eruption+",
            "Tantrum", "Tantrum+",
            "Crescendo", "Crescendo+",
            "Indignation", "Indignation+",
        }

        # Cards that exit to Calm or Neutral
        self.stance_exit = {
            "EmptyFist", "EmptyFist+",
            "EmptyBody", "EmptyBody+",
            "EmptyMind", "EmptyMind+",
            "Tranquility", "Tranquility+",
            "Vigilance", "Vigilance+",
            "FearNoEvil", "FearNoEvil+",
            "InnerPeace", "InnerPeace+",
            "Meditate", "Meditate+",
        }

    def can_kill_all(
        self,
        hand: List[Dict],
        enemies: List[SimulatedEnemy],
        player: SimulatedPlayer,
    ) -> bool:
        """Quick check: can we kill all enemies this turn?"""
        stats = self.get_kill_stats(hand, enemies, player)
        return stats.can_kill_all

    def can_kill_priority(
        self,
        hand: List[Dict],
        enemies: List[SimulatedEnemy],
        player: SimulatedPlayer,
    ) -> bool:
        """Quick check: can we kill the most dangerous enemy?"""
        stats = self.get_kill_stats(hand, enemies, player)
        return stats.can_kill_priority

    def get_kill_sequence(
        self,
        hand: List[Dict],
        enemies: List[SimulatedEnemy],
        player: SimulatedPlayer,
        objective: str = "kill_all",  # "kill_all", "kill_priority", "minimize_damage"
    ) -> List[Action]:
        """Find optimal card sequence for kill objective."""
        stats = self.get_kill_stats(hand, enemies, player)

        if objective == "kill_all" and stats.can_kill_all:
            return stats.optimal_sequence
        elif objective == "kill_priority" and stats.can_kill_priority:
            return stats.optimal_sequence
        elif objective == "minimize_damage":
            return stats.safest_kill_sequence

        return []

    def get_kill_stats(
        self,
        hand: List[Dict],
        enemies: List[SimulatedEnemy],
        player: SimulatedPlayer,
    ) -> KillStats:
        """Complete kill analysis with all details."""
        stats = KillStats()

        # Skip if no enemies
        alive_enemies = [e for e in enemies if not e.is_dead()]
        if not alive_enemies:
            stats.can_kill_all = True
            stats.can_kill_priority = True
            return stats

        # Analyze each enemy
        total_hp_to_kill = 0
        for enemy in alive_enemies:
            info = self._analyze_enemy(enemy, player)
            stats.enemy_kill_info[enemy.id] = info
            total_hp_to_kill += info.damage_needed

        # Identify priority target
        stats.priority_target_id = self._identify_priority_target(alive_enemies)
        if stats.priority_target_id is not None:
            stats.enemy_kill_info[stats.priority_target_id].is_priority = True

        # Check wrath optimization potential
        hand_ids = {c.get("id", "") for c in hand}
        can_enter_wrath = bool(hand_ids & self.wrath_entry)
        can_exit_wrath = bool(hand_ids & self.stance_exit)
        stats.safe_wrath_available = can_enter_wrath and can_exit_wrath

        # Calculate maximum possible damage
        max_damage = self._calculate_max_damage(hand, player, alive_enemies)

        # Quick check: can we even theoretically kill?
        if max_damage < total_hp_to_kill:
            # Can't kill all, but maybe can kill priority
            if stats.priority_target_id is not None:
                priority_hp = stats.enemy_kill_info[stats.priority_target_id].damage_needed
                if max_damage >= priority_hp:
                    # Search for priority kill
                    self._search_kill_sequences(
                        hand, alive_enemies, player, stats,
                        target_mode="priority"
                    )
            return stats

        # Search for kill sequences
        self._search_kill_sequences(hand, alive_enemies, player, stats, target_mode="all")

        return stats

    def _analyze_enemy(
        self,
        enemy: SimulatedEnemy,
        player: SimulatedPlayer,
    ) -> EnemyKillInfo:
        """Analyze a single enemy for kill potential."""
        # Get threat info from database
        threat_level = 0.3
        is_scaling = False
        enemy_name = f"Enemy_{enemy.id}"

        # Try to look up in database
        for name, info in ENEMIES.items():
            if name.lower().replace(" ", "").replace("_", "") == \
               str(enemy.id).lower().replace(" ", "").replace("_", ""):
                threat_level = info.threat_level
                is_scaling = info.attack_pattern == "scaling"
                enemy_name = name
                break

        # Calculate incoming damage from this enemy
        incoming = 0
        if enemy.is_attacking:
            base_dmg = enemy.intent_damage * enemy.intent_hits
            if enemy.weak > 0:
                base_dmg = int(base_dmg * 0.75)
            # Account for wrath if player is in wrath
            if player.stance == "Wrath":
                base_dmg *= 2
            incoming = base_dmg

        return EnemyKillInfo(
            enemy_id=enemy.id,
            enemy_name=enemy_name,
            current_hp=enemy.hp,
            current_block=enemy.block,
            can_kill=False,  # Will be updated by search
            damage_needed=enemy.hp + enemy.block,
            damage_available=0,
            threat_level=threat_level,
            incoming_damage=incoming,
            is_priority=False,
            is_scaling=is_scaling,
        )

    def _identify_priority_target(
        self,
        enemies: List[SimulatedEnemy],
    ) -> Optional[int]:
        """Determine which enemy should die first."""
        if not enemies:
            return None

        priority_scores = {}

        for enemy in enemies:
            if enemy.is_dead():
                continue

            score = 0.0

            # 1. Immediate threat (incoming damage)
            if enemy.is_attacking:
                score += enemy.intent_damage * enemy.intent_hits * 10

            # 2. Threat level from database
            info = get_enemy_info(str(enemy.id))
            if info:
                score += info.threat_level * 100
                # Scaling enemies are priority
                if info.attack_pattern == "scaling":
                    score += 50

            # 3. Low HP = easy kill, bonus for quick removal
            kill_threshold = enemy.hp + enemy.block
            if kill_threshold < 20:
                score += 30

            # 4. Vulnerable enemies are efficient kills
            if enemy.vulnerable > 0:
                score += 25

            priority_scores[enemy.id] = score

        if not priority_scores:
            return None

        return max(priority_scores, key=priority_scores.get)

    def _calculate_max_damage(
        self,
        hand: List[Dict],
        player: SimulatedPlayer,
        enemies: List[SimulatedEnemy],
    ) -> int:
        """Calculate theoretical maximum damage possible this turn."""
        # Get potential energy (including calm exit)
        max_energy = player.energy
        if player.stance == "Calm":
            max_energy += 2

        # Calculate damage from all attack cards
        total_damage = 0

        # Check if we can use wrath
        hand_ids = {c.get("id", "") for c in hand}
        will_have_wrath = player.stance == "Wrath" or bool(hand_ids & self.wrath_entry)

        for card in hand:
            card_id = card.get("id", "")
            effect = CARD_EFFECTS.get(card_id, {})

            cost = effect.get("cost", 99)
            if cost > max_energy:
                continue

            base_dmg = effect.get("damage", 0)
            hits = effect.get("hits", 1)

            if base_dmg > 0:
                # Apply strength
                dmg = base_dmg + player.strength

                # Apply wrath (2x) if we'll be in wrath
                if will_have_wrath:
                    dmg *= 2

                # Weak reduces by 25%
                if player.weak > 0:
                    dmg = int(dmg * 0.75)

                # Apply vulnerable on enemy (assume at least one vulnerable)
                has_vulnerable = any(e.vulnerable > 0 for e in enemies)
                if has_vulnerable:
                    dmg = int(dmg * 1.5)

                # Multiply by hits
                dmg *= hits

                # AOE multiplier
                if effect.get("aoe"):
                    dmg *= len([e for e in enemies if not e.is_dead()])

                total_damage += dmg

        return total_damage

    def _search_kill_sequences(
        self,
        hand: List[Dict],
        enemies: List[SimulatedEnemy],
        player: SimulatedPlayer,
        stats: KillStats,
        target_mode: str = "all",
        max_cards: int = 6,
    ) -> None:
        """Search for kill sequences using iterative deepening."""

        # Get playable cards
        playable = self._get_playable_cards(hand, player)

        if not playable:
            return

        best_outcome = None
        best_actions = []
        safest_outcome = None
        safest_actions = []
        fastest_outcome = None
        fastest_actions = []

        # Determine priority target for targeting
        priority_id = stats.priority_target_id if target_mode == "priority" else None

        # Iterative deepening - try 1 card, then 2, etc.
        for depth in range(1, min(max_cards + 1, len(playable) + 1)):
            found_kill_at_depth = False

            for combo in itertools.permutations(playable, depth):
                # Build action sequence (target priority enemy if in priority mode)
                actions = self._cards_to_actions(combo, enemies, priority_id)

                # Simulate
                try:
                    outcome = self.simulator.simulate_line(
                        player, enemies, hand, actions
                    )
                except Exception:
                    continue

                # Check if this achieves our objective
                kills_target = False
                if target_mode == "all":
                    kills_target = outcome.is_lethal
                elif target_mode == "priority":
                    # Check if priority target is dead
                    if stats.priority_target_id is not None:
                        target_hp = None
                        for eid, hp in outcome.enemies_remaining:
                            if eid == stats.priority_target_id:
                                target_hp = hp
                                break
                        kills_target = target_hp is None  # Not in remaining = dead

                if kills_target:
                    found_kill_at_depth = True

                    # Update best if better score
                    if best_outcome is None or outcome.score > best_outcome.score:
                        best_outcome = outcome
                        best_actions = actions

                    # Track safest (least damage taken)
                    if safest_outcome is None or outcome.damage_taken < safest_outcome.damage_taken:
                        safest_outcome = outcome
                        safest_actions = actions

                    # Track fastest (fewest cards) - first kill at this depth
                    if fastest_outcome is None:
                        fastest_outcome = outcome
                        fastest_actions = actions

            # Early termination: found kills at this depth, no need to search deeper
            if found_kill_at_depth and fastest_outcome is not None:
                break

        # Update stats with results
        if best_outcome is not None:
            if target_mode == "all":
                stats.can_kill_all = True
            stats.can_kill_priority = True
            stats.guaranteed_kill = True  # Deterministic sim

            stats.optimal_sequence = self._actions_to_action_list(best_actions)
            stats.sequence_damage = best_outcome.damage_dealt
            stats.overkill_damage = -best_outcome.total_enemy_hp_remaining
            stats.energy_remaining = best_outcome.energy_remaining
            stats.hp_cost = best_outcome.damage_taken
            stats.final_stance = best_outcome.final_stance
            stats.enemies_killed = best_outcome.enemies_killed

            # Check if wrath was needed
            for action in best_actions:
                card_id = action[0]
                if card_id in self.wrath_entry:
                    stats.requires_wrath = True
                    break

        if safest_outcome is not None:
            stats.safest_kill_sequence = self._actions_to_action_list(safest_actions)

        if fastest_outcome is not None:
            stats.fastest_kill_sequence = self._actions_to_action_list(fastest_actions)

    def _get_playable_cards(
        self,
        hand: List[Dict],
        player: SimulatedPlayer,
    ) -> List[Dict]:
        """Get cards that can potentially be played (accounting for calm energy)."""
        max_energy = player.energy
        if player.stance == "Calm":
            max_energy += 2  # Will get +2 on first stance change

        playable = []
        for card in hand:
            card_id = card.get("id", "")
            effect = CARD_EFFECTS.get(card_id, {})
            cost = effect.get("cost", 99)

            if cost <= max_energy:
                playable.append(card)

        return playable

    def _cards_to_actions(
        self,
        cards: Tuple[Dict, ...],
        enemies: List[SimulatedEnemy],
        priority_target: Optional[int] = None,
    ) -> List[Tuple[str, Optional[int]]]:
        """Convert card dicts to (card_id, target) tuples."""
        actions = []
        alive = [e for e in enemies if not e.is_dead()]

        for card in cards:
            card_id = card.get("id", "")
            effect = CARD_EFFECTS.get(card_id, {})

            # Determine target
            if effect.get("aoe") or effect.get("damage", 0) == 0:
                target = None
            else:
                # Target priority if specified, else first enemy
                if priority_target is not None:
                    target = priority_target
                else:
                    target = alive[0].id if alive else None

            actions.append((card_id, target))

        return actions

    def _actions_to_action_list(
        self,
        raw_actions: List[Tuple[str, Optional[int]]],
    ) -> List[Action]:
        """Convert raw action tuples to Action dataclass list."""
        actions = []
        for card_id, target_id in raw_actions:
            effect = CARD_EFFECTS.get(card_id, {})

            stance_change = None
            if effect.get("enters"):
                stance_change = effect["enters"]
            elif effect.get("exits_stance"):
                stance_change = "Neutral"

            actions.append(Action(
                card_id=card_id,
                target_id=target_id,
                damage=effect.get("damage", 0) * effect.get("hits", 1),
                block=effect.get("block", 0),
                energy_cost=effect.get("cost", 0),
                stance_change=stance_change,
            ))

        return actions


# Convenience functions
def can_kill_this_turn(
    hand: List[Dict],
    enemies: List[SimulatedEnemy],
    player: SimulatedPlayer,
) -> bool:
    """Quick check if lethal is possible."""
    calc = KillCalculator()
    return calc.can_kill_all(hand, enemies, player)


def get_kill_line(
    hand: List[Dict],
    enemies: List[SimulatedEnemy],
    player: SimulatedPlayer,
) -> Optional[List[Action]]:
    """Get the optimal kill sequence, or None if no kill possible."""
    calc = KillCalculator()
    stats = calc.get_kill_stats(hand, enemies, player)
    if stats.can_kill_all:
        return stats.optimal_sequence
    return None


if __name__ == "__main__":
    # Test the kill calculator
    print("=== Kill Calculator Tests ===\n")

    # Test 1: Simple kill
    print("Test 1: Simple kill with Eruption + Tantrum")
    player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Calm")
    enemies = [SimulatedEnemy(
        id=0, hp=30, max_hp=30, block=0,
        intent_damage=10, intent_hits=1, is_attacking=True
    )]
    hand = [
        {"id": "Eruption+"},  # 9 dmg, enters wrath, 1 cost
        {"id": "Tantrum"},    # 3x3 dmg, enters wrath, 1 cost
    ]

    calc = KillCalculator()
    stats = calc.get_kill_stats(hand, enemies, player)

    print(f"  Can kill all: {stats.can_kill_all}")
    print(f"  Sequence: {[a.card_id for a in stats.optimal_sequence]}")
    print(f"  Damage dealt: {stats.sequence_damage}")
    print(f"  HP cost: {stats.hp_cost}")
    print(f"  Requires wrath: {stats.requires_wrath}")

    # Test 2: Multi-enemy
    print("\nTest 2: Multi-enemy with AOE")
    enemies = [
        SimulatedEnemy(id=0, hp=20, max_hp=20, block=0,
                       intent_damage=8, intent_hits=1, is_attacking=True),
        SimulatedEnemy(id=1, hp=15, max_hp=15, block=0,
                       intent_damage=12, intent_hits=1, is_attacking=True),
    ]
    hand = [
        {"id": "Ragnarok"},   # 5x5 AOE, 3 cost
        {"id": "Eruption+"},  # 9 dmg, 1 cost
    ]
    player = SimulatedPlayer(hp=50, block=0, energy=4, stance="Neutral")

    stats = calc.get_kill_stats(hand, enemies, player)
    print(f"  Can kill all: {stats.can_kill_all}")
    print(f"  Sequence: {[a.card_id for a in stats.optimal_sequence]}")
    print(f"  Priority target: {stats.priority_target_id}")

    # Test 3: Priority kill
    print("\nTest 3: Can only kill priority (not all)")
    enemies = [
        SimulatedEnemy(id=0, hp=100, max_hp=100, block=0,
                       intent_damage=5, intent_hits=1, is_attacking=True),
        SimulatedEnemy(id=1, hp=20, max_hp=20, block=0,
                       intent_damage=20, intent_hits=1, is_attacking=True),  # High threat
    ]
    hand = [
        {"id": "Eruption+"},
        {"id": "Tantrum+"},
    ]
    player = SimulatedPlayer(hp=50, block=0, energy=3, stance="Calm")

    stats = calc.get_kill_stats(hand, enemies, player)
    print(f"  Can kill all: {stats.can_kill_all}")
    print(f"  Can kill priority: {stats.can_kill_priority}")
    print(f"  Priority target ID: {stats.priority_target_id}")

    print("\n=== Tests Complete ===")
