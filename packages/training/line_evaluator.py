"""
Line Evaluator - Simulates card play sequences to evaluate outcomes.

A "line" is a sequence of actions: [play card A -> target enemy 1 -> play card B -> end turn]

For each possible line, we compute:
- Final enemy HP states (who's dead, who's alive)
- Damage we'll take (after our block)
- Resources spent/gained
- Stance changes

This lets the model choose between lines based on OUTCOMES, not card names.
"""

from typing import List, Dict, Tuple, Optional, Any
from dataclasses import dataclass, field
from enum import Enum
from copy import deepcopy
import itertools


class ActionType(Enum):
    PLAY_CARD = "play"
    USE_POTION = "potion"
    END_TURN = "end"


@dataclass
class SimulatedEnemy:
    """Enemy state for simulation."""
    id: int
    hp: int
    max_hp: int
    block: int
    intent_damage: int
    intent_hits: int
    is_attacking: bool
    vulnerable: int = 0
    weak: int = 0

    def take_damage(self, amount: int) -> int:
        """Apply damage, return actual damage dealt."""
        if self.vulnerable > 0:
            amount = int(amount * 1.5)

        blocked = min(amount, self.block)
        self.block -= blocked
        remaining = amount - blocked

        actual = min(remaining, self.hp)
        self.hp -= actual
        return actual

    def is_dead(self) -> bool:
        return self.hp <= 0


@dataclass
class SimulatedPlayer:
    """Player state for simulation."""
    hp: int
    block: int
    energy: int
    stance: str = "Neutral"  # Neutral, Wrath, Calm, Divinity
    strength: int = 0
    dexterity: int = 0
    vulnerable: int = 0
    weak: int = 0

    def take_damage(self, amount: int) -> int:
        """Apply damage after block, return damage taken to HP."""
        if self.stance == "Wrath":
            amount *= 2
        if self.vulnerable > 0:
            amount = int(amount * 1.5)

        blocked = min(amount, self.block)
        self.block -= blocked
        remaining = amount - blocked

        actual = min(remaining, self.hp)
        self.hp -= actual
        return actual

    def deal_damage(self, base: int) -> int:
        """Calculate outgoing damage with modifiers."""
        damage = base + self.strength
        if self.stance == "Wrath":
            damage *= 2
        if self.weak > 0:
            damage = int(damage * 0.75)
        return max(0, damage)

    def gain_block(self, base: int) -> int:
        """Calculate block gained with dexterity."""
        block = base + self.dexterity
        return max(0, block)


@dataclass
class LineOutcome:
    """Result of simulating a line of play."""
    # Actions taken
    actions: List[str] = field(default_factory=list)

    # Final state
    player_hp: int = 0
    player_block: int = 0
    energy_remaining: int = 0
    final_stance: str = "Neutral"

    # Enemy states
    enemies_killed: int = 0
    enemies_remaining: List[Tuple[int, int]] = field(default_factory=list)  # [(id, hp), ...]
    total_enemy_hp_remaining: int = 0
    priority_target_dead: bool = False

    # Damage math
    damage_dealt: int = 0
    damage_taken: int = 0  # HP lost this turn (after enemy attacks)
    overkill: int = 0

    # Evaluation
    is_lethal: bool = False  # All enemies dead
    is_safe: bool = False    # Taking 0 damage
    we_die: bool = False     # We die this turn

    # Score (higher = better)
    score: float = 0.0


# Card effects database
CARD_EFFECTS = {
    # Format: damage, block, energy_cost, special_effects
    "Strike_P": {"damage": 6, "block": 0, "cost": 1},
    "Defend_P": {"damage": 0, "block": 5, "cost": 1},

    # Watcher attacks
    "Eruption": {"damage": 9, "cost": 2, "enters": "Wrath"},
    "Eruption+": {"damage": 9, "cost": 1, "enters": "Wrath"},
    "Tantrum": {"damage": 3, "hits": 3, "cost": 1, "enters": "Wrath"},
    "Tantrum+": {"damage": 4, "hits": 3, "cost": 1, "enters": "Wrath"},
    "BowlingBash": {"damage": 7, "cost": 1, "aoe": True},
    "BowlingBash+": {"damage": 10, "cost": 1, "aoe": True},
    "FlyingSleeves": {"damage": 4, "hits": 2, "cost": 1},
    "FlyingSleeves+": {"damage": 6, "hits": 2, "cost": 1},
    "Ragnarok": {"damage": 5, "hits": 5, "cost": 3, "aoe": True},
    "Ragnarok+": {"damage": 6, "hits": 5, "cost": 3, "aoe": True},
    "Conclude": {"damage": 12, "cost": 1, "aoe": True, "ends_turn": True},
    "Conclude+": {"damage": 16, "cost": 1, "aoe": True, "ends_turn": True},
    "EmptyFist": {"damage": 9, "cost": 1, "exits_stance": True},
    "EmptyFist+": {"damage": 14, "cost": 1, "exits_stance": True},
    "Wallop": {"damage": 9, "cost": 2, "block_equal_damage": True},
    "Wallop+": {"damage": 12, "cost": 2, "block_equal_damage": True},
    "SashWhip": {"damage": 8, "cost": 1, "weak": 1},
    "SashWhip+": {"damage": 10, "cost": 1, "weak": 2},
    "FearNoEvil": {"damage": 8, "cost": 1, "enters": "Calm", "conditional": "enemy_attacking"},
    "FearNoEvil+": {"damage": 11, "cost": 1, "enters": "Calm", "conditional": "enemy_attacking"},
    "Smite": {"damage": 12, "cost": 1, "ethereal": False, "retain": True},
    "Smite+": {"damage": 16, "cost": 1, "ethereal": False, "retain": True},
    "FlurryOfBlows": {"damage": 4, "cost": 0, "on_stance_change": True},
    "FlurryOfBlows+": {"damage": 6, "cost": 0, "on_stance_change": True},
    "CutThroughFate": {"damage": 7, "cost": 1, "scry": 2, "draw": 1},
    "CutThroughFate+": {"damage": 9, "cost": 1, "scry": 3, "draw": 1},
    "TalkToTheHand": {"damage": 5, "cost": 1, "block_on_attack": 2},
    "TalkToTheHand+": {"damage": 7, "cost": 1, "block_on_attack": 3},
    "FollowUp": {"damage": 7, "cost": 1, "energy_if_last_attack": 1},
    "FollowUp+": {"damage": 11, "cost": 1, "energy_if_last_attack": 1},
    "SignatureMove": {"damage": 30, "cost": 2, "only_attack": True},
    "SignatureMove+": {"damage": 40, "cost": 2, "only_attack": True},
    "WheelKick": {"damage": 15, "cost": 2, "draw": 2},
    "WheelKick+": {"damage": 20, "cost": 2, "draw": 2},
    "Consecrate": {"damage": 5, "cost": 0, "aoe": True},
    "Consecrate+": {"damage": 8, "cost": 0, "aoe": True},
    "ReachHeaven": {"damage": 10, "cost": 2, "add_to_hand": "ThroughViolence"},
    "ThroughViolence": {"damage": 20, "cost": 0, "retain": True},
    "LessonLearned": {"damage": 6, "cost": 2, "upgrade_on_kill": True},
    "LessonLearned+": {"damage": 10, "cost": 2, "upgrade_on_kill": True},
    "Weave": {"damage": 4, "cost": 0, "on_scry": True},
    "Weave+": {"damage": 6, "cost": 0, "on_scry": True},
    "Windmill Strike": {"damage": 7, "cost": 2, "retain": True, "damage_up": 4},
    "WindmillStrike+": {"damage": 10, "cost": 2, "retain": True, "damage_up": 4},
    "Brilliance": {"damage": 12, "cost": 1, "per_mantra": True},
    "Brilliance+": {"damage": 16, "cost": 1, "per_mantra": True},
    "Sands of Time": {"damage": 20, "cost": 4, "retain": True, "cost_down": 1},
    "SandsOfTime+": {"damage": 26, "cost": 4, "retain": True, "cost_down": 1},
    "Judgment": {"damage": 0, "cost": 1, "execute": 30},
    "Judgment+": {"damage": 0, "cost": 1, "execute": 40},

    # Watcher skills
    "Vigilance": {"block": 8, "cost": 2, "enters": "Calm"},
    "Vigilance+": {"block": 12, "cost": 2, "enters": "Calm"},
    "EmptyBody": {"block": 7, "cost": 1, "exits_stance": True},
    "EmptyBody+": {"block": 10, "cost": 1, "exits_stance": True},
    "Halt": {"block": 3, "cost": 0, "wrath_bonus": 9},
    "Halt+": {"block": 4, "cost": 0, "wrath_bonus": 14},
    "Protect": {"block": 12, "cost": 2, "retain": True},
    "Protect+": {"block": 16, "cost": 2, "retain": True},
    "Crescendo": {"cost": 1, "enters": "Wrath", "retain": True},
    "Crescendo+": {"cost": 0, "enters": "Wrath", "retain": True},
    "Tranquility": {"cost": 1, "enters": "Calm", "retain": True},
    "Tranquility+": {"cost": 0, "enters": "Calm", "retain": True},
    "InnerPeace": {"cost": 1, "enters": "Calm", "draw": 3},
    "InnerPeace+": {"cost": 1, "enters": "Calm", "draw": 4},
    "EmptyMind": {"cost": 1, "exits_stance": True, "draw": 2},
    "EmptyMind+": {"cost": 1, "exits_stance": True, "draw": 3},
    "Meditate": {"cost": 1, "enters": "Calm", "ends_turn": True, "retrieve": 1},
    "Meditate+": {"cost": 1, "enters": "Calm", "ends_turn": True, "retrieve": 2},
    "Perseverance": {"block": 5, "cost": 1, "retain": True, "scales": True},
    "Perseverance+": {"block": 7, "cost": 1, "retain": True, "scales": True},
    "ThirdEye": {"block": 7, "cost": 1, "scry": 3},
    "ThirdEye+": {"block": 9, "cost": 1, "scry": 5},
    "Pray": {"cost": 1, "mantra": 3, "retain": True},
    "Pray+": {"cost": 1, "mantra": 4, "retain": True},
    "Worship": {"cost": 2, "mantra": 5},
    "Worship+": {"cost": 2, "mantra": 8},
    "Prostrate": {"cost": 0, "block": 4, "mantra": 2},
    "Prostrate+": {"cost": 0, "block": 6, "mantra": 3},
    "Evaluate": {"cost": 1, "block": 6, "add_insight": True},
    "Evaluate+": {"cost": 1, "block": 10, "add_insight": True},
    "SpiritShield": {"cost": 2, "block_per_card": 3},
    "SpiritShield+": {"cost": 2, "block_per_card": 4},
    "Sanctity": {"cost": 1, "block": 6, "draw_if_attack": 2},
    "Sanctity+": {"cost": 1, "block": 9, "draw_if_attack": 3},
    "WaveOfTheHand": {"cost": 1, "weak_all": 1},
    "WaveOfTheHand+": {"cost": 1, "weak_all": 2},
    "Indignation": {"cost": 1, "enters": "Wrath", "vulnerable_all": 3},
    "Indignation+": {"cost": 1, "enters": "Wrath", "vulnerable_all": 5},
    "Blasphemy": {"cost": 1, "enters": "Divinity", "die_next_turn": True},
    "Blasphemy+": {"cost": 0, "enters": "Divinity", "die_next_turn": True},
    "Swivel": {"cost": 2, "block": 8, "next_attack_retain": True},
    "Swivel+": {"cost": 2, "block": 11, "next_attack_retain": True},
    "ForeignInfluence": {"cost": 0, "add_random_attacks": 2},
    "ForeignInfluence+": {"cost": 0, "add_random_attacks": 3},
    "Scrawl": {"cost": 1, "draw_to_full": True},
    "Scrawl+": {"cost": 0, "draw_to_full": True},
    "Vault": {"cost": 3, "extra_turn": True},
    "Vault+": {"cost": 2, "extra_turn": True},
    "Wish": {"cost": 3, "choose_buff": True},
    "Wish+": {"cost": 3, "choose_buff": True, "upgraded": True},
}


class LineSimulator:
    """Simulates lines of play to evaluate outcomes."""

    def __init__(self):
        pass

    def simulate_line(
        self,
        player: SimulatedPlayer,
        enemies: List[SimulatedEnemy],
        hand: List[Dict],
        actions: List[Tuple[str, Optional[int]]],  # [(card_id, target_idx), ...]
    ) -> LineOutcome:
        """
        Simulate a sequence of card plays.

        Args:
            player: Starting player state
            enemies: Starting enemy states
            hand: Cards in hand
            actions: List of (card_id, target_index) tuples

        Returns:
            LineOutcome with final state and evaluation
        """
        # Deep copy to not modify originals
        p = deepcopy(player)
        es = deepcopy(enemies)
        h = list(hand)

        outcome = LineOutcome()
        outcome.actions = [a[0] for a in actions]

        damage_dealt = 0
        turn_ended = False

        for card_id, target_idx in actions:
            if turn_ended:
                break

            effect = CARD_EFFECTS.get(card_id, {})
            cost = effect.get("cost", 1)

            # Check if we can play
            if cost > p.energy:
                continue

            p.energy -= cost

            # Handle stance changes FIRST (affects damage calc)
            if effect.get("enters"):
                old_stance = p.stance
                new_stance = effect["enters"]

                # Calm -> anything gives +2 energy
                if old_stance == "Calm" and new_stance != "Calm":
                    p.energy += 2
                    outcome.energy_remaining = p.energy

                # Divinity gives +3 energy
                if new_stance == "Divinity" and old_stance != "Divinity":
                    p.energy += 3

                p.stance = new_stance

            if effect.get("exits_stance"):
                if p.stance == "Calm":
                    p.energy += 2
                p.stance = "Neutral"

            # Deal damage
            base_damage = effect.get("damage", 0)
            if base_damage > 0:
                hits = effect.get("hits", 1)
                is_aoe = effect.get("aoe", False)

                dmg_per_hit = p.deal_damage(base_damage)

                if is_aoe:
                    for e in es:
                        if not e.is_dead():
                            for _ in range(hits):
                                damage_dealt += e.take_damage(dmg_per_hit)
                else:
                    # Single target
                    if target_idx is not None and target_idx < len(es):
                        target = es[target_idx]
                        if not target.is_dead():
                            for _ in range(hits):
                                damage_dealt += target.take_damage(dmg_per_hit)

            # Execute effect (Judgment)
            execute_threshold = effect.get("execute", 0)
            if execute_threshold > 0 and target_idx is not None:
                target = es[target_idx]
                if target.hp <= execute_threshold:
                    damage_dealt += target.hp
                    target.hp = 0

            # Gain block
            base_block = effect.get("block", 0)
            if base_block > 0:
                # Wrath bonus (Halt)
                if p.stance == "Wrath" and effect.get("wrath_bonus"):
                    base_block += effect["wrath_bonus"]

                p.block += p.gain_block(base_block)

            # Block equal to damage (Wallop)
            if effect.get("block_equal_damage") and base_damage > 0:
                p.block += p.deal_damage(base_damage)

            # Ends turn
            if effect.get("ends_turn"):
                turn_ended = True

        # Enemy attacks (end of turn)
        total_incoming = 0
        for e in es:
            if not e.is_dead() and e.is_attacking:
                dmg = e.intent_damage * e.intent_hits
                if e.weak > 0:
                    dmg = int(dmg * 0.75)
                total_incoming += dmg

        damage_taken = p.take_damage(total_incoming)

        # Build outcome
        outcome.player_hp = p.hp
        outcome.player_block = p.block
        outcome.energy_remaining = p.energy
        outcome.final_stance = p.stance

        outcome.damage_dealt = damage_dealt
        outcome.damage_taken = damage_taken

        alive_enemies = [e for e in es if not e.is_dead()]
        outcome.enemies_killed = len(es) - len(alive_enemies)
        outcome.enemies_remaining = [(e.id, e.hp) for e in alive_enemies]
        outcome.total_enemy_hp_remaining = sum(e.hp for e in alive_enemies)

        outcome.is_lethal = len(alive_enemies) == 0
        outcome.is_safe = damage_taken == 0
        outcome.we_die = p.hp <= 0

        # Calculate score
        outcome.score = self._score_outcome(outcome, player.hp)

        return outcome

    def _score_outcome(self, outcome: LineOutcome, starting_hp: int) -> float:
        """Score a line outcome. Higher = better."""
        score = 0.0

        # Lethal is best
        if outcome.is_lethal:
            score += 1000
            score -= outcome.damage_taken * 2  # Prefer taking less damage

        # Don't die
        if outcome.we_die:
            score -= 10000

        # Damage dealt
        score += outcome.damage_dealt * 2

        # Damage taken (bad)
        score -= outcome.damage_taken * 5

        # Being safe is good
        if outcome.is_safe:
            score += 100

        # Kills are good
        score += outcome.enemies_killed * 50

        # Ending in good stance
        if outcome.final_stance == "Calm":
            score += 20  # Energy next turn
        elif outcome.final_stance == "Wrath" and not outcome.is_lethal:
            score -= 50  # Dangerous if fight continues

        return score

    def find_best_line(
        self,
        player: SimulatedPlayer,
        enemies: List[SimulatedEnemy],
        hand: List[Dict],
        max_actions: int = 5,
    ) -> Tuple[LineOutcome, List[Tuple[str, Optional[int]]]]:
        """
        Find the best line of play through simulation.

        This is a simplified search - for real MCTS we'd be smarter.
        """
        playable = [
            c for c in hand
            if CARD_EFFECTS.get(c.get("id", ""), {}).get("cost", 99) <= player.energy
        ]

        best_outcome = None
        best_actions = []

        # Try single card plays
        for card in playable:
            card_id = card.get("id", "")
            effect = CARD_EFFECTS.get(card_id, {})

            # Determine targets
            if effect.get("aoe") or effect.get("block", 0) > 0:
                targets = [None]  # No target needed
            else:
                targets = [i for i, e in enumerate(enemies) if not e.is_dead()]
                if not targets:
                    targets = [None]

            for target in targets:
                actions = [(card_id, target)]
                outcome = self.simulate_line(player, enemies, hand, actions)

                if best_outcome is None or outcome.score > best_outcome.score:
                    best_outcome = outcome
                    best_actions = actions

        # Try 2-card combos (most common case)
        if len(playable) >= 2:
            for c1, c2 in itertools.permutations(playable, 2):
                c1_id = c1.get("id", "")
                c2_id = c2.get("id", "")
                e1 = CARD_EFFECTS.get(c1_id, {})
                e2 = CARD_EFFECTS.get(c2_id, {})

                # Quick energy check
                if e1.get("cost", 1) + e2.get("cost", 1) > player.energy + 2:  # +2 for calm
                    continue

                t1 = 0 if not e1.get("aoe") and e1.get("damage", 0) > 0 else None
                t2 = 0 if not e2.get("aoe") and e2.get("damage", 0) > 0 else None

                actions = [(c1_id, t1), (c2_id, t2)]
                outcome = self.simulate_line(player, enemies, hand, actions)

                if outcome.score > best_outcome.score:
                    best_outcome = outcome
                    best_actions = actions

        return best_outcome, best_actions


def evaluate_all_lines(
    player: SimulatedPlayer,
    enemies: List[SimulatedEnemy],
    hand: List[Dict],
) -> List[LineOutcome]:
    """Evaluate all reasonable lines and return sorted by score."""
    sim = LineSimulator()
    outcomes = []

    playable = [
        c for c in hand
        if CARD_EFFECTS.get(c.get("id", ""), {}).get("cost", 99) <= player.energy + 2  # +2 for calm
    ]

    # Generate lines up to 3 cards
    for r in range(1, min(4, len(playable) + 1)):
        for combo in itertools.permutations(playable, r):
            actions = []
            for card in combo:
                card_id = card.get("id", "")
                effect = CARD_EFFECTS.get(card_id, {})
                target = 0 if effect.get("damage", 0) > 0 and not effect.get("aoe") else None
                actions.append((card_id, target))

            try:
                outcome = sim.simulate_line(player, enemies, hand, actions)
                outcomes.append(outcome)
            except:
                pass

    return sorted(outcomes, key=lambda o: -o.score)


if __name__ == "__main__":
    # Test simulation
    player = SimulatedPlayer(
        hp=50, block=0, energy=3, stance="Calm", strength=0
    )

    enemies = [
        SimulatedEnemy(id=0, hp=40, max_hp=40, block=0,
                       intent_damage=12, intent_hits=1, is_attacking=True),
    ]

    hand = [
        {"id": "Eruption+"},  # 9 damage, enters wrath, 1 energy
        {"id": "Tantrum"},    # 3x3 damage, enters wrath, 1 energy
        {"id": "EmptyFist+"}, # 14 damage, exits stance, 1 energy
        {"id": "Defend_P"},   # 5 block
    ]

    sim = LineSimulator()

    print("=== Line Evaluation Test ===\n")

    # Test: Eruption (wrath) -> Tantrum (stay wrath) -> EmptyFist (exit)
    # Calm->Wrath gives +2 energy = 5 total
    # Eruption: 9 * 2 = 18 damage (in wrath)
    # Tantrum: 3 * 3 * 2 = 18 damage (in wrath)
    # EmptyFist: 14 damage (exits to neutral, so no double)
    # Total: 18 + 18 + 14 = 50 damage

    actions = [("Eruption+", 0), ("Tantrum", 0), ("EmptyFist+", 0)]
    outcome = sim.simulate_line(player, enemies, hand, actions)

    print(f"Line: {outcome.actions}")
    print(f"Damage dealt: {outcome.damage_dealt}")
    print(f"Enemy HP remaining: {outcome.total_enemy_hp_remaining}")
    print(f"Lethal: {outcome.is_lethal}")
    print(f"Damage taken: {outcome.damage_taken}")
    print(f"Final stance: {outcome.final_stance}")
    print(f"Score: {outcome.score}")

    print("\n--- Finding best line ---")
    best, best_actions = sim.find_best_line(player, enemies, hand)
    print(f"Best line: {best_actions}")
    print(f"Score: {best.score}")
    print(f"Lethal: {best.is_lethal}")
