"""
Combat calculator for Slay the Spire.

Pre-computes strategic features instead of making the model learn arithmetic:
- Kill probability this turn / next turn
- Expected damage dealt / taken
- Potion impact analysis
- Lethal detection
- Block efficiency

These become input features for the neural network.
"""

import math
from typing import List, Dict, Tuple, Optional, Any
from dataclasses import dataclass
from enum import Enum


class Stance(Enum):
    NEUTRAL = "Neutral"
    WRATH = "Wrath"
    CALM = "Calm"
    DIVINITY = "Divinity"


@dataclass
class Enemy:
    hp: int
    block: int
    intent_damage: int  # 0 if not attacking
    intent_hits: int    # number of hits
    is_attacking: bool
    debuffs: Dict[str, int]  # e.g., {"Vulnerable": 2, "Weak": 1}


@dataclass
class PlayerState:
    hp: int
    max_hp: int
    block: int
    energy: int
    stance: Stance
    hand: List[Dict]  # cards in hand with stats
    draw_pile_size: int
    discard_size: int
    powers: Dict[str, int]  # e.g., {"Strength": 2, "Dexterity": 1}
    orbs: List[Dict]  # for Defect


@dataclass
class CombatFeatures:
    """Pre-computed features for neural network input."""

    # Survival features
    hp_ratio: float                    # current_hp / max_hp
    effective_hp: float                # hp + block
    incoming_damage: int               # total damage enemies will deal
    damage_after_block: int            # max(0, incoming - block)
    lethal_threat: bool                # will die if don't block
    overkill_amount: int               # how much we'd die by

    # Offensive features
    can_kill_all: bool                 # can kill all enemies this turn
    kill_probability: float            # 0-1 chance to kill (with draw variance)
    total_enemy_hp: int
    total_damage_in_hand: int          # damage we can deal this turn
    damage_per_energy: float           # efficiency metric

    # Resource features
    energy: int
    cards_in_hand: int
    cards_playable: int                # cards we have energy for
    draw_pile_size: int

    # Watcher-specific
    stance: str
    can_enter_wrath: bool
    can_exit_wrath: bool
    wrath_damage_multiplier: float     # 2x in wrath
    calm_energy_on_exit: int           # 2 energy when leaving calm

    # Potion features
    has_attack_potion: bool
    has_block_potion: bool
    has_energy_potion: bool
    potion_saves_lethal: bool          # would a potion prevent death

    # Turn planning
    expected_draw_damage: float        # avg damage from drawing more cards
    needs_to_block: bool               # taking significant damage otherwise
    block_in_hand: int                 # total block available
    block_deficit: int                 # incoming - available block


class CombatCalculator:
    """Calculates combat features for model input."""

    # Card database (damage, block, energy, type)
    WATCHER_CARDS = {
        # Attacks
        "Strike_P": {"damage": 6, "block": 0, "energy": 1, "type": "attack"},
        "Eruption": {"damage": 9, "block": 0, "energy": 2, "type": "attack", "enters_wrath": True},
        "Eruption+": {"damage": 9, "block": 0, "energy": 1, "type": "attack", "enters_wrath": True},
        "Tantrum": {"damage": 3, "block": 0, "energy": 1, "type": "attack", "hits": 3, "enters_wrath": True},
        "Tantrum+": {"damage": 4, "block": 0, "energy": 1, "type": "attack", "hits": 3, "enters_wrath": True},
        "BowlingBash": {"damage": 7, "block": 0, "energy": 1, "type": "attack", "per_enemy": True},
        "FlyingSleeves": {"damage": 4, "block": 0, "energy": 1, "type": "attack", "hits": 2},
        "FlyingSleeves+": {"damage": 6, "block": 0, "energy": 1, "type": "attack", "hits": 2},
        "Ragnarok": {"damage": 5, "block": 0, "energy": 3, "type": "attack", "hits": 5},
        "Ragnarok+": {"damage": 6, "block": 0, "energy": 3, "type": "attack", "hits": 5},
        "Conclude": {"damage": 12, "block": 0, "energy": 1, "type": "attack", "aoe": True, "ends_turn": True},
        "Conclude+": {"damage": 16, "block": 0, "energy": 1, "type": "attack", "aoe": True, "ends_turn": True},
        "SashWhip": {"damage": 8, "block": 0, "energy": 1, "type": "attack"},
        "SashWhip+": {"damage": 10, "block": 0, "energy": 1, "type": "attack"},
        "Wallop": {"damage": 9, "block": 0, "energy": 2, "type": "attack", "block_equal": True},
        "Wallop+": {"damage": 12, "block": 0, "energy": 2, "type": "attack", "block_equal": True},
        "CutThroughFate": {"damage": 7, "block": 0, "energy": 1, "type": "attack", "scry": 2},
        "EmptyFist": {"damage": 9, "block": 0, "energy": 1, "type": "attack", "exit_stance": True},
        "EmptyFist+": {"damage": 14, "block": 0, "energy": 1, "type": "attack", "exit_stance": True},
        "FlurryOfBlows": {"damage": 4, "block": 0, "energy": 0, "type": "attack"},
        "FlurryOfBlows+": {"damage": 6, "block": 0, "energy": 0, "type": "attack"},
        "ReachHeaven": {"damage": 10, "block": 0, "energy": 2, "type": "attack"},
        "TalkToTheHand": {"damage": 5, "block": 0, "energy": 1, "type": "attack", "block_per_hit": 2},
        "TalkToTheHand+": {"damage": 7, "block": 0, "energy": 1, "type": "attack", "block_per_hit": 3},
        "Smite": {"damage": 12, "block": 0, "energy": 1, "type": "attack", "retain": True},
        "Smite+": {"damage": 16, "block": 0, "energy": 1, "type": "attack", "retain": True},

        # Skills - Defense
        "Defend_P": {"damage": 0, "block": 5, "energy": 1, "type": "skill"},
        "Vigilance": {"damage": 0, "block": 8, "energy": 2, "type": "skill", "enters_calm": True},
        "Vigilance+": {"damage": 0, "block": 12, "energy": 2, "type": "skill", "enters_calm": True},
        "EmptyBody": {"damage": 0, "block": 7, "energy": 1, "type": "skill", "exit_stance": True},
        "EmptyBody+": {"damage": 0, "block": 10, "energy": 1, "type": "skill", "exit_stance": True},
        "Halt": {"damage": 0, "block": 3, "energy": 0, "type": "skill", "wrath_bonus": 9},
        "Halt+": {"damage": 0, "block": 4, "energy": 0, "type": "skill", "wrath_bonus": 14},
        "Protect": {"damage": 0, "block": 12, "energy": 2, "type": "skill", "retain": True},
        "Protect+": {"damage": 0, "block": 16, "energy": 2, "type": "skill", "retain": True},
        "SpiritShield": {"damage": 0, "block": 3, "energy": 2, "type": "skill", "per_card": True},
        "Perseverance": {"damage": 0, "block": 5, "energy": 1, "type": "skill", "retain": True, "scales": True},
        "Perseverance+": {"damage": 0, "block": 7, "energy": 1, "type": "skill", "retain": True, "scales": True},

        # Stance changers
        "Crescendo": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "enters_wrath": True, "retain": True},
        "Crescendo+": {"damage": 0, "block": 0, "energy": 0, "type": "skill", "enters_wrath": True, "retain": True},
        "Tranquility": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "enters_calm": True, "retain": True},
        "Tranquility+": {"damage": 0, "block": 0, "energy": 0, "type": "skill", "enters_calm": True, "retain": True},
        "InnerPeace": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "enters_calm": True, "draw": 3},
        "InnerPeace+": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "enters_calm": True, "draw": 4},
        "EmptyMind": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "exit_stance": True, "draw": 2},
        "EmptyMind+": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "exit_stance": True, "draw": 3},
        "FearNoEvil": {"damage": 8, "block": 0, "energy": 1, "type": "attack", "enters_calm": True},
        "FearNoEvil+": {"damage": 11, "block": 0, "energy": 1, "type": "attack", "enters_calm": True},

        # Draw/energy
        "Rushdown": {"damage": 0, "block": 0, "energy": 1, "type": "power", "draw_on_wrath": 2},
        "Scrawl": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "fill_hand": True},
        "Meditate": {"damage": 0, "block": 0, "energy": 1, "type": "skill", "enters_calm": True, "ends_turn": True},
    }

    def __init__(self):
        pass

    def calculate_features(
        self,
        player: PlayerState,
        enemies: List[Enemy],
        potions: List[Dict] = None,
    ) -> CombatFeatures:
        """Calculate all combat features for model input."""

        potions = potions or []

        # Basic stats
        hp_ratio = player.hp / max(player.max_hp, 1)

        # Incoming damage calculation
        incoming_damage = self._calc_incoming_damage(enemies, player)
        damage_after_block = max(0, incoming_damage - player.block)
        lethal_threat = damage_after_block >= player.hp
        overkill = max(0, damage_after_block - player.hp)

        # Offensive calculations
        total_enemy_hp = sum(max(0, e.hp - e.block) for e in enemies if e.hp > 0)
        damage_in_hand, playable_damage = self._calc_hand_damage(player, enemies)
        can_kill_all = playable_damage >= total_enemy_hp
        kill_prob = min(1.0, playable_damage / max(total_enemy_hp, 1))

        # Resource stats
        playable_count = sum(1 for c in player.hand if c.get("cost", 99) <= player.energy)

        # Block calculations
        block_in_hand = self._calc_hand_block(player)
        block_deficit = max(0, incoming_damage - player.block - block_in_hand)
        needs_block = incoming_damage > player.block + 5  # some buffer

        # Watcher stance
        stance_str = player.stance.value if player.stance else "Neutral"
        wrath_mult = 2.0 if player.stance == Stance.WRATH else 1.0
        calm_energy = 2 if player.stance == Stance.CALM else 0

        can_enter_wrath = any(
            self.WATCHER_CARDS.get(c.get("id", ""), {}).get("enters_wrath", False)
            for c in player.hand
        )
        can_exit_wrath = any(
            self.WATCHER_CARDS.get(c.get("id", ""), {}).get("exit_stance", False) or
            self.WATCHER_CARDS.get(c.get("id", ""), {}).get("enters_calm", False)
            for c in player.hand
        )

        # Potion analysis
        has_attack_pot = any("attack" in p.get("id", "").lower() or "fire" in p.get("id", "").lower() for p in potions)
        has_block_pot = any("block" in p.get("id", "").lower() or "ghost" in p.get("id", "").lower() for p in potions)
        has_energy_pot = any("energy" in p.get("id", "").lower() for p in potions)
        potion_saves = lethal_threat and (has_block_pot or has_attack_pot)

        # Expected draw value (simplified)
        avg_damage_per_card = damage_in_hand / max(len(player.hand), 1)
        expected_draw = avg_damage_per_card * 2  # rough estimate

        return CombatFeatures(
            hp_ratio=hp_ratio,
            effective_hp=player.hp + player.block,
            incoming_damage=incoming_damage,
            damage_after_block=damage_after_block,
            lethal_threat=lethal_threat,
            overkill_amount=overkill,
            can_kill_all=can_kill_all,
            kill_probability=kill_prob,
            total_enemy_hp=total_enemy_hp,
            total_damage_in_hand=playable_damage,
            damage_per_energy=playable_damage / max(player.energy, 1),
            energy=player.energy,
            cards_in_hand=len(player.hand),
            cards_playable=playable_count,
            draw_pile_size=player.draw_pile_size,
            stance=stance_str,
            can_enter_wrath=can_enter_wrath,
            can_exit_wrath=can_exit_wrath,
            wrath_damage_multiplier=wrath_mult,
            calm_energy_on_exit=calm_energy,
            has_attack_potion=has_attack_pot,
            has_block_potion=has_block_pot,
            has_energy_potion=has_energy_pot,
            potion_saves_lethal=potion_saves,
            expected_draw_damage=expected_draw,
            needs_to_block=needs_block,
            block_in_hand=block_in_hand,
            block_deficit=block_deficit,
        )

    def _calc_incoming_damage(self, enemies: List[Enemy], player: PlayerState) -> int:
        """Calculate total incoming damage from enemies."""
        total = 0
        for enemy in enemies:
            if enemy.is_attacking and enemy.hp > 0:
                dmg = enemy.intent_damage * enemy.intent_hits

                # Adjust for vulnerable on player
                if player.powers.get("Vulnerable", 0) > 0:
                    dmg = int(dmg * 1.5)

                # Adjust for weak on enemy
                if enemy.debuffs.get("Weak", 0) > 0:
                    dmg = int(dmg * 0.75)

                total += dmg

        # Double damage if in Wrath
        if player.stance == Stance.WRATH:
            total *= 2

        return total

    def _calc_hand_damage(self, player: PlayerState, enemies: List[Enemy]) -> Tuple[int, int]:
        """Calculate total and playable damage in hand."""
        total_damage = 0
        playable_damage = 0
        energy_left = player.energy

        # Sort by damage efficiency
        attacks = []
        for card in player.hand:
            card_id = card.get("id", "")
            card_data = self.WATCHER_CARDS.get(card_id, {})
            if card_data.get("type") == "attack":
                dmg = card_data.get("damage", 0)
                hits = card_data.get("hits", 1)
                cost = card_data.get("energy", card.get("cost", 1))
                attacks.append((dmg * hits, cost, card_id))

        # Wrath multiplier
        mult = 2 if player.stance == Stance.WRATH else 1

        # Calculate vulnerable bonus on enemies
        vuln_mult = 1.5 if any(e.debuffs.get("Vulnerable", 0) > 0 for e in enemies) else 1.0

        for dmg, cost, _ in sorted(attacks, key=lambda x: -x[0]/max(x[1], 0.5)):
            effective_dmg = int(dmg * mult * vuln_mult)
            total_damage += effective_dmg
            if cost <= energy_left:
                playable_damage += effective_dmg
                energy_left -= cost

        return total_damage, playable_damage

    def _calc_hand_block(self, player: PlayerState) -> int:
        """Calculate total block available in hand."""
        total = 0
        energy_left = player.energy

        for card in player.hand:
            card_id = card.get("id", "")
            card_data = self.WATCHER_CARDS.get(card_id, {})
            block = card_data.get("block", 0)
            cost = card_data.get("energy", card.get("cost", 1))

            if block > 0 and cost <= energy_left:
                # Wrath bonus for Halt
                if player.stance == Stance.WRATH and card_data.get("wrath_bonus"):
                    block += card_data["wrath_bonus"]

                total += block
                energy_left -= cost

        # Add dexterity
        total += player.powers.get("Dexterity", 0) * sum(
            1 for c in player.hand
            if self.WATCHER_CARDS.get(c.get("id", ""), {}).get("block", 0) > 0
        )

        return total

    def features_to_vector(self, features: CombatFeatures) -> List[float]:
        """Convert features to flat vector for neural network."""
        return [
            features.hp_ratio,
            features.effective_hp / 100,  # normalize
            features.incoming_damage / 50,
            features.damage_after_block / 50,
            float(features.lethal_threat),
            features.overkill_amount / 50,
            float(features.can_kill_all),
            features.kill_probability,
            features.total_enemy_hp / 100,
            features.total_damage_in_hand / 50,
            features.damage_per_energy / 10,
            features.energy / 5,
            features.cards_in_hand / 10,
            features.cards_playable / 10,
            features.draw_pile_size / 30,
            float(features.stance == "Wrath"),
            float(features.stance == "Calm"),
            float(features.stance == "Divinity"),
            float(features.can_enter_wrath),
            float(features.can_exit_wrath),
            features.wrath_damage_multiplier - 1,  # 0 or 1
            features.calm_energy_on_exit / 2,
            float(features.has_attack_potion),
            float(features.has_block_potion),
            float(features.has_energy_potion),
            float(features.potion_saves_lethal),
            features.expected_draw_damage / 20,
            float(features.needs_to_block),
            features.block_in_hand / 30,
            features.block_deficit / 30,
        ]


# Quick test
if __name__ == "__main__":
    calc = CombatCalculator()

    player = PlayerState(
        hp=50, max_hp=80, block=5, energy=3,
        stance=Stance.CALM,
        hand=[
            {"id": "Strike_P", "cost": 1},
            {"id": "Eruption", "cost": 2},
            {"id": "Defend_P", "cost": 1},
            {"id": "Tantrum", "cost": 1},
        ],
        draw_pile_size=20,
        discard_size=5,
        powers={"Strength": 1},
        orbs=[],
    )

    enemies = [
        Enemy(hp=40, block=0, intent_damage=12, intent_hits=1, is_attacking=True, debuffs={}),
    ]

    features = calc.calculate_features(player, enemies)
    print("Combat Features:")
    print(f"  HP ratio: {features.hp_ratio:.2f}")
    print(f"  Incoming damage: {features.incoming_damage}")
    print(f"  Can kill all: {features.can_kill_all}")
    print(f"  Kill probability: {features.kill_probability:.2f}")
    print(f"  Damage in hand: {features.total_damage_in_hand}")
    print(f"  Lethal threat: {features.lethal_threat}")
    print(f"  Block in hand: {features.block_in_hand}")
    print(f"  Block deficit: {features.block_deficit}")
    print(f"  Stance: {features.stance}")

    vec = calc.features_to_vector(features)
    print(f"\nFeature vector length: {len(vec)}")
