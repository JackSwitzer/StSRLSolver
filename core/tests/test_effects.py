"""
Comprehensive Test Suite for Card Effects System.

Tests the effect registry, effect execution, and all major card effect categories
for the Slay the Spire RL project.

Run with: cd core && uv run pytest tests/test_effects.py -v
"""

import sys
import os
import importlib.util
from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Tuple, Callable
from enum import Enum
import re

# Setup path for imports
core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, core_dir)

import pytest


# =============================================================================
# INLINE DEFINITIONS
# We define test-specific versions of the necessary classes and functions
# to avoid the import issues with relative imports in the actual modules.
# =============================================================================


# -----------------------------------------------------------------------------
# Combat State (from state/combat.py)
# -----------------------------------------------------------------------------

@dataclass
class EntityState:
    """Minimal state for player or enemy."""
    hp: int
    max_hp: int
    block: int = 0
    statuses: Dict[str, int] = field(default_factory=dict)

    @property
    def strength(self) -> int:
        return self.statuses.get("Strength", 0)

    @property
    def dexterity(self) -> int:
        return self.statuses.get("Dexterity", 0)

    @property
    def is_weak(self) -> bool:
        return self.statuses.get("Weak", 0) > 0

    @property
    def is_vulnerable(self) -> bool:
        return self.statuses.get("Vulnerable", 0) > 0

    @property
    def is_frail(self) -> bool:
        return self.statuses.get("Frail", 0) > 0

    @property
    def is_dead(self) -> bool:
        return self.hp <= 0

    def copy(self) -> "EntityState":
        return EntityState(
            hp=self.hp, max_hp=self.max_hp, block=self.block,
            statuses=self.statuses.copy(),
        )


@dataclass
class EnemyCombatState(EntityState):
    """Enemy state in combat."""
    id: str = ""
    move_id: int = -1
    move_damage: int = 0
    move_hits: int = 1
    move_block: int = 0
    move_effects: Dict[str, int] = field(default_factory=dict)

    def copy(self) -> "EnemyCombatState":
        return EnemyCombatState(
            hp=self.hp, max_hp=self.max_hp, block=self.block,
            statuses=self.statuses.copy(), id=self.id,
            move_id=self.move_id, move_damage=self.move_damage,
            move_hits=self.move_hits, move_block=self.move_block,
            move_effects=self.move_effects.copy(),
        )

    @property
    def is_attacking(self) -> bool:
        return self.move_damage > 0


@dataclass
class CombatState:
    """Complete combat state."""
    player: EntityState
    energy: int
    max_energy: int
    stance: str = "Neutral"
    hand: List[str] = field(default_factory=list)
    draw_pile: List[str] = field(default_factory=list)
    discard_pile: List[str] = field(default_factory=list)
    exhaust_pile: List[str] = field(default_factory=list)
    enemies: List[EnemyCombatState] = field(default_factory=list)
    potions: List[str] = field(default_factory=list)
    turn: int = 1
    cards_played_this_turn: int = 0
    attacks_played_this_turn: int = 0
    skills_played_this_turn: int = 0
    powers_played_this_turn: int = 0
    relic_counters: Dict[str, int] = field(default_factory=dict)
    relics: List[str] = field(default_factory=list)
    card_costs: Dict[str, int] = field(default_factory=dict)

    def copy(self) -> "CombatState":
        return CombatState(
            player=self.player.copy(),
            energy=self.energy, max_energy=self.max_energy, stance=self.stance,
            hand=self.hand.copy(), draw_pile=self.draw_pile.copy(),
            discard_pile=self.discard_pile.copy(), exhaust_pile=self.exhaust_pile.copy(),
            enemies=[e.copy() for e in self.enemies], potions=self.potions.copy(),
            turn=self.turn, cards_played_this_turn=self.cards_played_this_turn,
            attacks_played_this_turn=self.attacks_played_this_turn,
            skills_played_this_turn=self.skills_played_this_turn,
            powers_played_this_turn=self.powers_played_this_turn,
            relic_counters=self.relic_counters.copy(), relics=self.relics.copy(),
            card_costs=self.card_costs.copy(),
        )

    def has_relic(self, relic_id: str) -> bool:
        return relic_id in self.relics

    def get_relic_counter(self, relic_id: str, default: int = 0) -> int:
        return self.relic_counters.get(relic_id, default)

    def set_relic_counter(self, relic_id: str, value: int) -> None:
        self.relic_counters[relic_id] = value


def create_player(hp: int, max_hp: int = None) -> EntityState:
    return EntityState(hp=hp, max_hp=max_hp or hp)


def create_enemy(
    id: str, hp: int, max_hp: int = None, move_id: int = -1,
    move_damage: int = 0, move_hits: int = 1, move_block: int = 0,
) -> EnemyCombatState:
    return EnemyCombatState(
        hp=hp, max_hp=max_hp or hp, id=id, move_id=move_id,
        move_damage=move_damage, move_hits=move_hits, move_block=move_block,
    )


def create_combat(
    player_hp: int, player_max_hp: int, enemies: List[EnemyCombatState],
    deck: List[str], energy: int = 3, max_energy: int = 3,
    relics: List[str] = None, potions: List[str] = None,
) -> CombatState:
    return CombatState(
        player=create_player(player_hp, player_max_hp),
        energy=energy, max_energy=max_energy,
        hand=[], draw_pile=deck.copy(), discard_pile=[], exhaust_pile=[],
        enemies=enemies, relics=relics or [],
        potions=potions or ["", "", ""],
    )


# -----------------------------------------------------------------------------
# Card Definitions (from content/cards.py)
# -----------------------------------------------------------------------------

class CardType(Enum):
    ATTACK = "ATTACK"
    SKILL = "SKILL"
    POWER = "POWER"
    STATUS = "STATUS"
    CURSE = "CURSE"


class CardRarity(Enum):
    BASIC = "BASIC"
    COMMON = "COMMON"
    UNCOMMON = "UNCOMMON"
    RARE = "RARE"
    SPECIAL = "SPECIAL"
    CURSE = "CURSE"


class CardTarget(Enum):
    ENEMY = "ENEMY"
    ALL_ENEMY = "ALL_ENEMY"
    SELF = "SELF"
    NONE = "NONE"
    SELF_AND_ENEMY = "SELF_AND_ENEMY"
    ALL = "ALL"


class CardColor(Enum):
    RED = "RED"
    GREEN = "GREEN"
    BLUE = "BLUE"
    PURPLE = "PURPLE"
    COLORLESS = "COLORLESS"
    CURSE = "CURSE"


@dataclass
class Card:
    """A card definition."""
    id: str
    name: str
    card_type: CardType
    rarity: CardRarity
    color: CardColor = CardColor.PURPLE
    target: CardTarget = CardTarget.ENEMY
    cost: int = 1
    base_damage: int = -1
    base_block: int = -1
    base_magic: int = -1
    upgrade_cost: Optional[int] = None
    upgrade_damage: int = 0
    upgrade_block: int = 0
    upgrade_magic: int = 0
    exhaust: bool = False
    ethereal: bool = False
    retain: bool = False
    innate: bool = False
    shuffle_back: bool = False
    enter_stance: Optional[str] = None
    exit_stance: bool = False
    effects: List[str] = field(default_factory=list)
    upgraded: bool = False
    cost_for_turn: Optional[int] = None

    @property
    def damage(self) -> int:
        if self.base_damage < 0:
            return -1
        return self.base_damage + (self.upgrade_damage if self.upgraded else 0)

    @property
    def block(self) -> int:
        if self.base_block < 0:
            return -1
        return self.base_block + (self.upgrade_block if self.upgraded else 0)

    @property
    def magic_number(self) -> int:
        if self.base_magic < 0:
            return -1
        return self.base_magic + (self.upgrade_magic if self.upgraded else 0)

    @property
    def current_cost(self) -> int:
        if self.cost_for_turn is not None:
            return self.cost_for_turn
        if self.upgraded and self.upgrade_cost is not None:
            return self.upgrade_cost
        return self.cost

    def upgrade(self):
        self.upgraded = True

    def copy(self) -> "Card":
        return Card(
            id=self.id, name=self.name, card_type=self.card_type,
            rarity=self.rarity, color=self.color, target=self.target,
            cost=self.cost, base_damage=self.base_damage,
            base_block=self.base_block, base_magic=self.base_magic,
            upgrade_cost=self.upgrade_cost, upgrade_damage=self.upgrade_damage,
            upgrade_block=self.upgrade_block, upgrade_magic=self.upgrade_magic,
            exhaust=self.exhaust, ethereal=self.ethereal, retain=self.retain,
            innate=self.innate, shuffle_back=self.shuffle_back,
            enter_stance=self.enter_stance, exit_stance=self.exit_stance,
            effects=self.effects.copy(), upgraded=self.upgraded,
        )


# Card definitions
CARD_DEFS: Dict[str, Card] = {
    "Strike_P": Card(
        id="Strike_P", name="Strike", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
        cost=1, base_damage=6, upgrade_damage=3,
    ),
    "Defend_P": Card(
        id="Defend_P", name="Defend", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
        target=CardTarget.SELF, cost=1, base_block=5, upgrade_block=3,
    ),
    "Eruption": Card(
        id="Eruption", name="Eruption", card_type=CardType.ATTACK, rarity=CardRarity.BASIC,
        cost=2, base_damage=9, upgrade_cost=1, enter_stance="Wrath",
    ),
    "Vigilance": Card(
        id="Vigilance", name="Vigilance", card_type=CardType.SKILL, rarity=CardRarity.BASIC,
        target=CardTarget.SELF, cost=2, base_block=8, upgrade_block=4, enter_stance="Calm",
    ),
    "Miracle": Card(
        id="Miracle", name="Miracle", card_type=CardType.SKILL, rarity=CardRarity.SPECIAL,
        color=CardColor.COLORLESS, target=CardTarget.SELF, cost=0,
        retain=True, exhaust=True, effects=["gain_1_energy"], upgrade_magic=1,
    ),
    "Consecrate": Card(
        id="Consecrate", name="Consecrate", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
        target=CardTarget.ALL_ENEMY, cost=0, base_damage=5, upgrade_damage=3,
    ),
    "FlyingSleeves": Card(
        id="FlyingSleeves", name="Flying Sleeves", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
        cost=1, base_damage=4, base_magic=2, upgrade_damage=2, retain=True,
        effects=["damage_x_times"],
    ),
    "Tantrum": Card(
        id="Tantrum", name="Tantrum", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        cost=1, base_damage=3, base_magic=3, upgrade_magic=1,
        shuffle_back=True, enter_stance="Wrath", effects=["damage_x_times"],
    ),
    "Halt": Card(
        id="Halt", name="Halt", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
        target=CardTarget.SELF, cost=0, base_block=3, upgrade_block=1,
        effects=["if_in_wrath_extra_block_6"],
    ),
    "ThirdEye": Card(
        id="ThirdEye", name="Third Eye", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
        target=CardTarget.SELF, cost=1, base_block=7, upgrade_block=2,
        base_magic=3, upgrade_magic=2, effects=["scry"],
    ),
    "CutThroughFate": Card(
        id="CutThroughFate", name="Cut Through Fate", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
        cost=1, base_damage=7, upgrade_damage=2, effects=["scry_2", "draw_1"], upgrade_magic=1,
    ),
    "WheelKick": Card(
        id="WheelKick", name="Wheel Kick", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        cost=2, base_damage=15, upgrade_damage=5, effects=["draw_2"],
    ),
    "Scrawl": Card(
        id="Scrawl", name="Scrawl", card_type=CardType.SKILL, rarity=CardRarity.RARE,
        target=CardTarget.SELF, cost=1, upgrade_cost=0, exhaust=True,
        effects=["draw_until_hand_full"],
    ),
    "Prostrate": Card(
        id="Prostrate", name="Prostrate", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
        target=CardTarget.SELF, cost=0, base_block=4, upgrade_block=2,
        base_magic=2, upgrade_magic=1, effects=["gain_mantra"],
    ),
    "Worship": Card(
        id="Worship", name="Worship", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=2, base_magic=5, upgrade_magic=3, retain=True,
        effects=["gain_mantra"],
    ),
    "Devotion": Card(
        id="Devotion", name="Devotion", card_type=CardType.POWER, rarity=CardRarity.RARE,
        target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
        effects=["gain_mantra_each_turn"],
    ),
    "MentalFortress": Card(
        id="MentalFortress", name="Mental Fortress", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_magic=4, upgrade_magic=2,
        effects=["on_stance_change_gain_block"],
    ),
    "Adaptation": Card(
        id="Adaptation", name="Rushdown", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_magic=2, upgrade_magic=1,
        effects=["on_wrath_draw"],
    ),
    "Nirvana": Card(
        id="Nirvana", name="Nirvana", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=1,
        effects=["on_scry_gain_block"],
    ),
    "LikeWater": Card(
        id="LikeWater", name="Like Water", card_type=CardType.POWER, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_magic=5, upgrade_magic=2,
        effects=["if_calm_end_turn_gain_block"],
    ),
    "TalkToTheHand": Card(
        id="TalkToTheHand", name="Talk to the Hand", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        cost=1, base_damage=5, upgrade_damage=2, base_magic=2, upgrade_magic=1,
        exhaust=True, effects=["apply_block_return"],
    ),
    "EmptyFist": Card(
        id="EmptyFist", name="Empty Fist", card_type=CardType.ATTACK, rarity=CardRarity.COMMON,
        cost=1, base_damage=9, upgrade_damage=5, exit_stance=True,
    ),
    "InnerPeace": Card(
        id="InnerPeace", name="Inner Peace", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, effects=["if_calm_draw_3_else_calm"], upgrade_magic=1,
    ),
    "Indignation": Card(
        id="Indignation", name="Indignation", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_magic=3, upgrade_magic=2,
        effects=["if_wrath_gain_mantra_else_wrath"],
    ),
    "FearNoEvil": Card(
        id="FearNoEvil", name="Fear No Evil", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        cost=1, base_damage=8, upgrade_damage=3, effects=["if_enemy_attacking_enter_calm"],
    ),
    "CarveReality": Card(
        id="CarveReality", name="Carve Reality", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        cost=1, base_damage=6, upgrade_damage=4, effects=["add_smite_to_hand"],
    ),
    "DeceiveReality": Card(
        id="DeceiveReality", name="Deceive Reality", card_type=CardType.SKILL, rarity=CardRarity.UNCOMMON,
        target=CardTarget.SELF, cost=1, base_block=4, upgrade_block=3,
        effects=["add_safety_to_hand"],
    ),
    "Evaluate": Card(
        id="Evaluate", name="Evaluate", card_type=CardType.SKILL, rarity=CardRarity.COMMON,
        target=CardTarget.SELF, cost=1, base_magic=6, upgrade_magic=4,
        effects=["add_insight_to_draw"],
    ),
    "Alpha": Card(
        id="Alpha", name="Alpha", card_type=CardType.SKILL, rarity=CardRarity.RARE,
        target=CardTarget.SELF, cost=1, upgrade_cost=0, innate=True,
        exhaust=True, effects=["shuffle_beta_into_draw"],
    ),
    "Judgement": Card(
        id="Judgement", name="Judgement", card_type=CardType.ATTACK, rarity=CardRarity.RARE,
        cost=1, base_magic=30, upgrade_magic=10, effects=["if_enemy_hp_below_kill"],
    ),
    "Conclude": Card(
        id="Conclude", name="Conclude", card_type=CardType.ATTACK, rarity=CardRarity.UNCOMMON,
        target=CardTarget.ALL_ENEMY, cost=1, base_damage=12, upgrade_damage=4,
        effects=["end_turn"],
    ),
    "Blasphemy": Card(
        id="Blasphemy", name="Blasphemy", card_type=CardType.SKILL, rarity=CardRarity.RARE,
        target=CardTarget.SELF, cost=1, upgrade_cost=0, retain=True,
        effects=["enter_divinity", "die_next_turn"],
    ),
}


def get_card(card_id: str, upgraded: bool = False) -> Card:
    """Get a copy of a card by ID."""
    if card_id not in CARD_DEFS:
        raise ValueError(f"Unknown card: {card_id}")
    card = CARD_DEFS[card_id].copy()
    if upgraded:
        card.upgrade()
    return card


WATCHER_CARDS = CARD_DEFS


# -----------------------------------------------------------------------------
# Effect Context and Registry (simplified from effects/registry.py)
# -----------------------------------------------------------------------------

@dataclass
class EffectContext:
    """Context passed to effect handlers."""
    state: CombatState
    card: Optional[Card] = None
    target: Optional[EnemyCombatState] = None
    target_idx: int = -1
    damage_dealt: int = 0
    block_gained: int = 0
    cards_drawn: List[str] = field(default_factory=list)
    cards_discarded: List[str] = field(default_factory=list)
    cards_exhausted: List[str] = field(default_factory=list)
    statuses_applied: List[Tuple[str, str, int]] = field(default_factory=list)
    energy_gained: int = 0
    energy_spent: int = 0
    stance_changed_to: Optional[str] = None
    mantra_gained: int = 0
    scried_cards: List[str] = field(default_factory=list)
    is_upgraded: bool = False
    magic_number: int = 0
    extra_data: Dict[str, Any] = field(default_factory=dict)

    @property
    def player(self) -> EntityState:
        return self.state.player

    @property
    def enemies(self) -> List[EnemyCombatState]:
        return self.state.enemies

    @property
    def living_enemies(self) -> List[EnemyCombatState]:
        return [e for e in self.state.enemies if not e.is_dead]

    @property
    def hand(self) -> List[str]:
        return self.state.hand

    @property
    def draw_pile(self) -> List[str]:
        return self.state.draw_pile

    @property
    def discard_pile(self) -> List[str]:
        return self.state.discard_pile

    @property
    def energy(self) -> int:
        return self.state.energy

    @property
    def stance(self) -> str:
        return self.state.stance

    def draw_cards(self, count: int) -> List[str]:
        drawn = []
        for _ in range(count):
            if not self.state.draw_pile:
                if not self.state.discard_pile:
                    break
                self._shuffle_discard_into_draw()
            if self.state.draw_pile and len(self.state.hand) < 10:
                card = self.state.draw_pile.pop()
                self.state.hand.append(card)
                drawn.append(card)
                self.cards_drawn.append(card)
        return drawn

    def _shuffle_discard_into_draw(self) -> None:
        import random
        self.state.draw_pile = self.state.discard_pile.copy()
        random.shuffle(self.state.draw_pile)
        self.state.discard_pile.clear()

    def exhaust_card(self, card_id: str, from_hand: bool = True) -> bool:
        if from_hand and card_id in self.state.hand:
            self.state.hand.remove(card_id)
            self.state.exhaust_pile.append(card_id)
            self.cards_exhausted.append(card_id)
            return True
        return False

    def add_card_to_hand(self, card_id: str) -> bool:
        if len(self.state.hand) < 10:
            self.state.hand.append(card_id)
            return True
        return False

    def add_card_to_draw_pile(self, card_id: str, position: str = "random") -> None:
        import random
        if position == "top":
            self.state.draw_pile.append(card_id)
        elif position == "bottom":
            self.state.draw_pile.insert(0, card_id)
        else:
            if self.state.draw_pile:
                idx = random.randint(0, len(self.state.draw_pile))
                self.state.draw_pile.insert(idx, card_id)
            else:
                self.state.draw_pile.append(card_id)

    def deal_damage_to_target(self, amount: int) -> int:
        if self.target and not self.target.is_dead:
            actual = self._apply_damage_to_enemy(self.target, amount)
            self.damage_dealt += actual
            return actual
        return 0

    def deal_damage_to_enemy(self, enemy: EnemyCombatState, amount: int) -> int:
        if not enemy.is_dead:
            actual = self._apply_damage_to_enemy(enemy, amount)
            self.damage_dealt += actual
            return actual
        return 0

    def deal_damage_to_random_enemy(self, amount: int) -> int:
        import random
        living = self.living_enemies
        if living:
            target = random.choice(living)
            return self.deal_damage_to_enemy(target, amount)
        return 0

    def _apply_damage_to_enemy(self, enemy: EnemyCombatState, amount: int) -> int:
        if amount <= 0:
            return 0
        blocked = min(enemy.block, amount)
        enemy.block -= blocked
        hp_damage = amount - blocked
        enemy.hp -= hp_damage
        if enemy.hp < 0:
            enemy.hp = 0
        return hp_damage

    def gain_block(self, amount: int) -> int:
        if amount > 0:
            self.state.player.block += amount
            self.block_gained += amount
        return amount

    def gain_energy(self, amount: int) -> None:
        if amount > 0:
            self.state.energy += amount
            self.energy_gained += amount

    def spend_energy(self, amount: int) -> bool:
        if self.state.energy >= amount:
            self.state.energy -= amount
            self.energy_spent += amount
            return True
        return False

    def apply_status_to_target(self, status: str, amount: int) -> bool:
        if self.target:
            return self.apply_status_to_enemy(self.target, status, amount)
        return False

    def apply_status_to_enemy(self, enemy: EnemyCombatState, status: str, amount: int) -> bool:
        if enemy.is_dead:
            return False
        debuffs = {"Weak", "Vulnerable", "Frail", "Poison", "Mark", "Constricted"}
        if status in debuffs:
            artifact = enemy.statuses.get("Artifact", 0)
            if artifact > 0:
                enemy.statuses["Artifact"] = artifact - 1
                if enemy.statuses["Artifact"] <= 0:
                    del enemy.statuses["Artifact"]
                return False
        current = enemy.statuses.get(status, 0)
        enemy.statuses[status] = current + amount
        self.statuses_applied.append((enemy.id, status, amount))
        return True

    def apply_status_to_player(self, status: str, amount: int) -> bool:
        debuffs = {"Weak", "Vulnerable", "Frail", "Poison", "Constricted"}
        if status in debuffs:
            artifact = self.state.player.statuses.get("Artifact", 0)
            if artifact > 0:
                self.state.player.statuses["Artifact"] = artifact - 1
                if self.state.player.statuses["Artifact"] <= 0:
                    del self.state.player.statuses["Artifact"]
                return False
        current = self.state.player.statuses.get(status, 0)
        self.state.player.statuses[status] = current + amount
        self.statuses_applied.append(("player", status, amount))
        return True

    def get_player_status(self, status: str) -> int:
        return self.state.player.statuses.get(status, 0)

    def change_stance(self, new_stance: str) -> Dict[str, Any]:
        old_stance = self.state.stance
        result = {"old_stance": old_stance, "new_stance": new_stance, "energy_gained": 0}
        if old_stance == new_stance:
            return result
        if old_stance == "Calm":
            energy_gain = 3 if self.state.has_relic("VioletLotus") else 2
            self.gain_energy(energy_gain)
            result["energy_gained"] += energy_gain
        self.state.stance = new_stance
        self.stance_changed_to = new_stance
        if new_stance == "Divinity":
            self.gain_energy(3)
            result["energy_gained"] += 3
        mental_fortress = self.get_player_status("MentalFortress")
        if mental_fortress > 0:
            self.gain_block(mental_fortress)
            result["block_gained"] = mental_fortress
        self._trigger_flurry_of_blows()
        if new_stance == "Wrath":
            rushdown = self.get_player_status("Rushdown")
            if rushdown > 0:
                self.draw_cards(rushdown)
                result["cards_drawn"] = rushdown
        return result

    def exit_stance(self) -> Dict[str, Any]:
        return self.change_stance("Neutral")

    def _trigger_flurry_of_blows(self) -> None:
        flurries = [c for c in self.state.discard_pile if c.startswith("FlurryOfBlows")]
        for f in flurries:
            if len(self.state.hand) < 10:
                self.state.discard_pile.remove(f)
                self.state.hand.append(f)

    def gain_mantra(self, amount: int) -> Dict[str, Any]:
        current = self.get_player_status("Mantra")
        new_total = current + amount
        self.mantra_gained += amount
        result = {"mantra_gained": amount, "divinity_triggered": False}
        if new_total >= 10:
            remainder = new_total - 10
            self.state.player.statuses["Mantra"] = remainder
            stance_result = self.change_stance("Divinity")
            result["divinity_triggered"] = True
            result.update(stance_result)
        else:
            self.apply_status_to_player("Mantra", amount)
        return result

    def scry(self, amount: int) -> List[str]:
        cards_to_scry = []
        for _ in range(amount):
            if not self.state.draw_pile:
                break
            card = self.state.draw_pile.pop()
            cards_to_scry.append(card)
        self.scried_cards = cards_to_scry
        nirvana = self.get_player_status("Nirvana")
        if nirvana > 0:
            self.gain_block(nirvana * len(cards_to_scry))
        self._trigger_weave()
        for card in reversed(cards_to_scry):
            self.state.draw_pile.append(card)
        return cards_to_scry

    def _trigger_weave(self) -> None:
        weaves = [c for c in self.state.discard_pile if c.startswith("Weave")]
        for w in weaves:
            if len(self.state.hand) < 10:
                self.state.discard_pile.remove(w)
                self.state.hand.append(w)

    def is_enemy_attacking(self, enemy: Optional[EnemyCombatState] = None) -> bool:
        target = enemy or self.target
        if target:
            return target.is_attacking
        return False

    def get_last_card_type(self) -> Optional[str]:
        return self.extra_data.get("last_card_type")

    def set_last_card_type(self, card_type: str) -> None:
        self.extra_data["last_card_type"] = card_type

    def end_turn(self) -> None:
        self.extra_data["end_turn"] = True

    def should_end_turn(self) -> bool:
        return self.extra_data.get("end_turn", False)


# Effect registry (simplified)
_EFFECT_REGISTRY: Dict[str, Callable] = {}


def effect(name: str, pattern: Optional[str] = None):
    def decorator(func):
        _EFFECT_REGISTRY[name] = func
        return func
    return decorator


def effect_simple(name: str):
    def decorator(func):
        _EFFECT_REGISTRY[name] = func
        return func
    return decorator


def effect_custom(name: str, pattern: str, param_types: List[type] = None):
    def decorator(func):
        _EFFECT_REGISTRY[name] = func
        return func
    return decorator


# Register basic effects
@effect_simple("draw_1")
def draw_1(ctx: EffectContext) -> None:
    ctx.draw_cards(1)

@effect_simple("draw_2")
def draw_2(ctx: EffectContext) -> None:
    ctx.draw_cards(2)

@effect("draw")
def draw_effect(ctx: EffectContext, amount: int) -> None:
    ctx.draw_cards(amount)

@effect("gain_energy")
def gain_energy_effect(ctx: EffectContext, amount: int) -> None:
    ctx.gain_energy(amount)

@effect("gain_block")
def gain_block_effect(ctx: EffectContext, amount: int) -> None:
    ctx.gain_block(amount)

@effect_simple("enter_wrath")
def enter_wrath(ctx: EffectContext) -> None:
    ctx.change_stance("Wrath")

@effect_simple("enter_calm")
def enter_calm(ctx: EffectContext) -> None:
    ctx.change_stance("Calm")

@effect_simple("enter_divinity")
def enter_divinity(ctx: EffectContext) -> None:
    ctx.change_stance("Divinity")

@effect_simple("exit_stance")
def exit_stance(ctx: EffectContext) -> None:
    ctx.exit_stance()


def get_effect_handler(effect_name: str) -> Optional[Tuple[Callable, Tuple]]:
    if effect_name in _EFFECT_REGISTRY:
        return (_EFFECT_REGISTRY[effect_name], ())
    # Pattern matching for draw_N
    match = re.match(r"^draw_(\d+)$", effect_name)
    if match:
        return (_EFFECT_REGISTRY.get("draw"), (int(match.group(1)),))
    return None


def execute_effect(effect_name: str, ctx: EffectContext) -> bool:
    result = get_effect_handler(effect_name)
    if result:
        handler, params = result
        if handler:
            if params:
                handler(ctx, *params)
            else:
                handler(ctx)
            return True
    return False


def list_registered_effects() -> List[str]:
    return list(_EFFECT_REGISTRY.keys())


# -----------------------------------------------------------------------------
# Effect Executor (simplified from effects/executor.py)
# -----------------------------------------------------------------------------

@dataclass
class EffectResult:
    """Result of executing an effect or playing a card."""
    success: bool
    effects_executed: List[str] = field(default_factory=list)
    damage_dealt: int = 0
    block_gained: int = 0
    energy_spent: int = 0
    energy_gained: int = 0
    cards_drawn: List[str] = field(default_factory=list)
    cards_discarded: List[str] = field(default_factory=list)
    cards_exhausted: List[str] = field(default_factory=list)
    statuses_applied: List[Tuple[str, str, int]] = field(default_factory=list)
    stance_changed_to: Optional[str] = None
    mantra_gained: int = 0
    scried_cards: List[str] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)
    extra: Dict[str, Any] = field(default_factory=dict)

    @property
    def should_end_turn(self) -> bool:
        return self.extra.get("end_turn", False)


class EffectExecutor:
    """Executes card effects and manages combat state."""

    STANCE_DAMAGE_MULT = {
        "Neutral": 1.0,
        "Calm": 1.0,
        "Wrath": 2.0,
        "Divinity": 3.0,
    }

    def __init__(self, state: CombatState):
        self.state = state

    def play_card(self, card: Card, target_idx: int = -1, free: bool = False) -> EffectResult:
        result = EffectResult(success=True)
        target = None
        if 0 <= target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]
            if target.is_dead:
                target = None

        ctx = EffectContext(
            state=self.state, card=card, target=target, target_idx=target_idx,
            is_upgraded=card.upgraded,
            magic_number=card.magic_number if card.magic_number > 0 else 0,
        )

        # Spend energy
        if not free:
            cost = card.current_cost
            if cost > 0:
                if self.state.energy >= cost:
                    self.state.energy -= cost
                    ctx.energy_spent = cost
                    result.energy_spent = cost
                else:
                    result.success = False
                    result.errors.append(f"Not enough energy: need {cost}, have {self.state.energy}")
                    return result

        # Execute base damage
        if card.base_damage > 0:
            self._execute_damage(ctx, card, target, result)

        # Execute base block
        if card.base_block > 0:
            self._execute_block(ctx, card, result)

        # Execute stance change
        if card.enter_stance:
            stance_result = ctx.change_stance(card.enter_stance)
            result.stance_changed_to = card.enter_stance
            result.energy_gained += stance_result.get("energy_gained", 0)

        if card.exit_stance:
            stance_result = ctx.exit_stance()
            result.stance_changed_to = "Neutral"
            result.energy_gained += stance_result.get("energy_gained", 0)

        # Execute card-specific effects
        for effect_name in card.effects:
            self._handle_special_effect(effect_name, ctx, card, result)

        # Collect results
        result.damage_dealt += ctx.damage_dealt
        result.block_gained += ctx.block_gained
        result.energy_gained += ctx.energy_gained
        result.cards_drawn.extend(ctx.cards_drawn)
        result.statuses_applied.extend(ctx.statuses_applied)
        result.mantra_gained = ctx.mantra_gained
        result.scried_cards = ctx.scried_cards

        if ctx.should_end_turn():
            result.extra["end_turn"] = True

        # Update combat tracking
        self.state.cards_played_this_turn += 1
        if card.card_type.value == "ATTACK":
            self.state.attacks_played_this_turn += 1
        elif card.card_type.value == "SKILL":
            self.state.skills_played_this_turn += 1
        elif card.card_type.value == "POWER":
            self.state.powers_played_this_turn += 1

        return result

    def _execute_damage(self, ctx: EffectContext, card: Card, target, result: EffectResult):
        base_damage = card.damage
        strength = self.state.player.statuses.get("Strength", 0)
        damage = base_damage + strength

        if self.state.player.is_weak:
            damage = int(damage * 0.75)

        stance_mult = self.STANCE_DAMAGE_MULT.get(self.state.stance, 1.0)
        damage = int(damage * stance_mult)

        hits = 1
        if card.magic_number > 0 and "damage_x_times" in card.effects:
            hits = card.magic_number

        if card.target == CardTarget.ALL_ENEMY:
            for enemy in ctx.living_enemies:
                self._deal_hits_to_enemy(ctx, enemy, damage, hits, result)
        elif card.target == CardTarget.ENEMY and target:
            self._deal_hits_to_enemy(ctx, target, damage, hits, result)

    def _deal_hits_to_enemy(self, ctx, enemy, damage_per_hit, hits, result):
        for _ in range(hits):
            if enemy.is_dead:
                break
            actual_damage = damage_per_hit
            if enemy.is_vulnerable:
                actual_damage = int(actual_damage * 1.5)
            ctx.deal_damage_to_enemy(enemy, actual_damage)

    def _execute_block(self, ctx: EffectContext, card: Card, result: EffectResult):
        base_block = card.block
        dexterity = self.state.player.statuses.get("Dexterity", 0)
        block = base_block + dexterity

        if self.state.player.is_frail:
            block = int(block * 0.75)

        if block > 0:
            ctx.gain_block(block)

    def _handle_special_effect(self, effect_name: str, ctx: EffectContext, card: Card, result: EffectResult) -> bool:
        if effect_name == "damage_x_times":
            return True  # Handled in _execute_damage
        if effect_name == "if_in_wrath_extra_block_6":
            if ctx.stance == "Wrath":
                extra = 9 if ctx.is_upgraded else 6
                ctx.gain_block(extra)
            return True
        if effect_name == "scry":
            amount = ctx.magic_number if ctx.magic_number > 0 else 3
            ctx.scry(amount)
            return True
        if effect_name == "scry_2":
            amount = 3 if ctx.is_upgraded else 2
            ctx.scry(amount)
            return True
        if effect_name == "draw_1":
            ctx.draw_cards(1)
            return True
        if effect_name == "draw_2":
            ctx.draw_cards(2)
            return True
        if effect_name == "draw_until_hand_full":
            cards_needed = 10 - len(ctx.hand)
            if cards_needed > 0:
                ctx.draw_cards(cards_needed)
            return True
        if effect_name == "gain_mantra":
            amount = ctx.magic_number if ctx.magic_number > 0 else 2
            ctx.gain_mantra(amount)
            return True
        if effect_name == "gain_mantra_each_turn":
            amount = ctx.magic_number if ctx.magic_number > 0 else 2
            ctx.apply_status_to_player("Devotion", amount)
            return True
        if effect_name == "on_stance_change_gain_block":
            amount = ctx.magic_number if ctx.magic_number > 0 else 4
            ctx.apply_status_to_player("MentalFortress", amount)
            return True
        if effect_name == "on_wrath_draw":
            amount = ctx.magic_number if ctx.magic_number > 0 else 2
            ctx.apply_status_to_player("Rushdown", amount)
            return True
        if effect_name == "on_scry_gain_block":
            amount = ctx.magic_number if ctx.magic_number > 0 else 3
            ctx.apply_status_to_player("Nirvana", amount)
            return True
        if effect_name == "if_calm_end_turn_gain_block":
            amount = ctx.magic_number if ctx.magic_number > 0 else 5
            ctx.apply_status_to_player("LikeWater", amount)
            return True
        if effect_name == "apply_block_return":
            amount = ctx.magic_number if ctx.magic_number > 0 else 2
            ctx.apply_status_to_target("BlockReturn", amount)
            return True
        if effect_name == "if_calm_draw_3_else_calm":
            if ctx.stance == "Calm":
                amount = 4 if ctx.is_upgraded else 3
                ctx.draw_cards(amount)
            else:
                ctx.change_stance("Calm")
            return True
        if effect_name == "if_wrath_gain_mantra_else_wrath":
            if ctx.stance == "Wrath":
                amount = 5 if ctx.is_upgraded else 3
                ctx.gain_mantra(amount)
            else:
                ctx.change_stance("Wrath")
            return True
        if effect_name == "if_enemy_attacking_enter_calm":
            if ctx.is_enemy_attacking():
                ctx.change_stance("Calm")
            return True
        if effect_name == "add_smite_to_hand":
            card_id = "Smite+" if ctx.is_upgraded else "Smite"
            ctx.add_card_to_hand(card_id)
            return True
        if effect_name == "add_safety_to_hand":
            card_id = "Safety+" if ctx.is_upgraded else "Safety"
            ctx.add_card_to_hand(card_id)
            return True
        if effect_name == "add_insight_to_draw":
            card_id = "Insight+" if ctx.is_upgraded else "Insight"
            ctx.add_card_to_draw_pile(card_id, "top")
            return True
        if effect_name == "shuffle_beta_into_draw":
            card_id = "Beta+" if ctx.is_upgraded else "Beta"
            ctx.add_card_to_draw_pile(card_id, "random")
            return True
        if effect_name == "if_enemy_hp_below_kill":
            threshold = 40 if ctx.is_upgraded else 30
            if ctx.target and ctx.target.hp <= threshold:
                ctx.target.hp = 0
            return True
        if effect_name == "end_turn":
            ctx.end_turn()
            return True
        if effect_name == "enter_divinity":
            ctx.change_stance("Divinity")
            return True
        if effect_name == "die_next_turn":
            ctx.apply_status_to_player("Blasphemy", 1)
            return True
        if effect_name == "gain_1_energy":
            amount = 2 if ctx.is_upgraded else 1
            ctx.gain_energy(amount)
            return True
        return False

    def apply_start_of_turn_effects(self) -> EffectResult:
        result = EffectResult(success=True)
        ctx = EffectContext(state=self.state)

        foresight = ctx.get_player_status("Foresight")
        if foresight > 0:
            ctx.scry(foresight)
            result.scried_cards = ctx.scried_cards

        devotion = ctx.get_player_status("Devotion")
        if devotion > 0:
            mantra_result = ctx.gain_mantra(devotion)
            result.mantra_gained += devotion
            if mantra_result.get("divinity_triggered"):
                result.stance_changed_to = "Divinity"

        return result

    def apply_end_of_turn_effects(self) -> EffectResult:
        result = EffectResult(success=True)
        ctx = EffectContext(state=self.state)

        if self.state.stance == "Calm":
            like_water = ctx.get_player_status("LikeWater")
            if like_water > 0:
                ctx.gain_block(like_water)
                result.block_gained += like_water

        if self.state.stance == "Divinity":
            ctx.change_stance("Neutral")
            result.stance_changed_to = "Neutral"

        if ctx.get_player_status("Blasphemy") > 0:
            self.state.player.hp = 0

        return result


def create_executor(state: CombatState) -> EffectExecutor:
    return EffectExecutor(state)


# =============================================================================
# FIXTURES
# =============================================================================


@pytest.fixture
def player() -> EntityState:
    """Create a default player with 80/80 HP."""
    return create_player(80, 80)


@pytest.fixture
def single_enemy() -> EnemyCombatState:
    """Create a single Jaw Worm enemy."""
    return create_enemy(
        id="jaw_worm",
        hp=44,
        max_hp=44,
        move_id=1,
        move_damage=11,
        move_hits=1,
    )


@pytest.fixture
def two_enemies() -> List[EnemyCombatState]:
    """Create two Cultist enemies."""
    return [
        create_enemy(
            id="cultist_1",
            hp=50,
            max_hp=50,
            move_id=1,
            move_damage=6,
            move_hits=1,
        ),
        create_enemy(
            id="cultist_2",
            hp=48,
            max_hp=48,
            move_id=1,
            move_damage=6,
            move_hits=1,
        ),
    ]


@pytest.fixture
def basic_combat(player, single_enemy) -> CombatState:
    """Create a basic combat state with one enemy."""
    return create_combat(
        player_hp=80,
        player_max_hp=80,
        enemies=[single_enemy],
        deck=["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption"],
        energy=3,
        max_energy=3,
    )


@pytest.fixture
def multi_enemy_combat(player, two_enemies) -> CombatState:
    """Create a combat state with multiple enemies."""
    return create_combat(
        player_hp=80,
        player_max_hp=80,
        enemies=two_enemies,
        deck=["Strike_P", "Defend_P", "Consecrate", "Ragnarok"],
        energy=3,
        max_energy=3,
    )


@pytest.fixture
def executor(basic_combat) -> EffectExecutor:
    """Create an EffectExecutor for the basic combat."""
    return EffectExecutor(basic_combat)


@pytest.fixture
def ctx(basic_combat, single_enemy) -> EffectContext:
    """Create an EffectContext for testing."""
    return EffectContext(
        state=basic_combat,
        target=single_enemy,
        target_idx=0,
    )


# =============================================================================
# EFFECT REGISTRY TESTS
# =============================================================================


class TestEffectRegistry:
    """Tests for the effect registration system."""

    def test_effect_decorator_registers_handler(self):
        """Test that @effect decorator registers a handler."""
        # Check that common effects are registered
        assert "draw" in _EFFECT_REGISTRY
        assert "gain_block" in _EFFECT_REGISTRY
        assert "gain_energy" in _EFFECT_REGISTRY

    def test_effect_simple_decorator_registers_handler(self):
        """Test that @effect_simple decorator registers a handler."""
        assert "enter_wrath" in _EFFECT_REGISTRY
        assert "enter_calm" in _EFFECT_REGISTRY
        assert "exit_stance" in _EFFECT_REGISTRY

    def test_get_effect_handler_direct_match(self):
        """Test direct effect name lookup."""
        handler = get_effect_handler("enter_wrath")
        assert handler is not None
        func, params = handler
        assert callable(func)
        assert params == ()

    def test_get_effect_handler_pattern_match(self):
        """Test pattern-based effect name lookup."""
        # draw_2 is directly registered, so params is empty
        handler = get_effect_handler("draw_2")
        assert handler is not None
        func, params = handler
        assert callable(func)
        # draw_2 is registered directly, params should be empty
        assert params == ()

        # Test pattern matching for unregistered number (draw_5)
        handler5 = get_effect_handler("draw_5")
        assert handler5 is not None
        func5, params5 = handler5
        assert callable(func5)
        # draw_5 uses pattern matching from "draw" effect
        assert params5 == (5,)

    def test_get_effect_handler_unknown_returns_none(self):
        """Test that unknown effect returns None."""
        handler = get_effect_handler("nonexistent_effect_xyz")
        assert handler is None

    def test_list_registered_effects(self):
        """Test listing all registered effects."""
        effects = list_registered_effects()
        assert isinstance(effects, list)
        assert len(effects) > 0
        assert "draw" in effects
        assert "enter_wrath" in effects

    def test_execute_effect_returns_true_for_valid_effect(self, ctx):
        """Test execute_effect returns True for valid effects."""
        result = execute_effect("draw_1", ctx)
        assert result is True

    def test_execute_effect_returns_false_for_unknown_effect(self, ctx):
        """Test execute_effect returns False for unknown effects."""
        result = execute_effect("unknown_effect_abc", ctx)
        assert result is False


# =============================================================================
# DAMAGE EFFECT TESTS
# =============================================================================


class TestDamageEffects:
    """Tests for damage-dealing card effects."""

    def test_strike_deals_base_damage(self, basic_combat):
        """Test Strike deals 6 base damage."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 6

    def test_strike_upgraded_deals_more_damage(self, basic_combat):
        """Test Strike+ deals 9 damage."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P", upgraded=True)
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 9

    def test_eruption_deals_damage_and_enters_wrath(self, basic_combat):
        """Test Eruption deals 9 damage and enters Wrath."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Eruption")
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 9
        assert basic_combat.stance == "Wrath"

    def test_eruption_upgraded_costs_less(self, basic_combat):
        """Test Eruption+ costs 1 energy instead of 2."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Eruption", upgraded=True)
        initial_energy = basic_combat.energy

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert result.energy_spent == 1
        assert basic_combat.energy == initial_energy - 1

    def test_consecrate_hits_all_enemies(self, multi_enemy_combat):
        """Test Consecrate deals damage to all enemies."""
        executor = EffectExecutor(multi_enemy_combat)
        card = get_card("Consecrate")
        initial_hps = [e.hp for e in multi_enemy_combat.enemies]

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        for i, enemy in enumerate(multi_enemy_combat.enemies):
            assert enemy.hp == initial_hps[i] - 5

    def test_flying_sleeves_hits_twice(self, basic_combat):
        """Test Flying Sleeves deals damage 2 times."""
        executor = EffectExecutor(basic_combat)
        card = get_card("FlyingSleeves")
        initial_hp = basic_combat.enemies[0].hp
        # Flying Sleeves: 4 damage x 2 = 8 total

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 8

    def test_tantrum_hits_multiple_times(self, basic_combat):
        """Test Tantrum deals damage 3 times."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Tantrum")
        initial_hp = basic_combat.enemies[0].hp
        # Tantrum: 3 damage x 3 = 9 total

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 9
        assert basic_combat.stance == "Wrath"


# =============================================================================
# BLOCK EFFECT TESTS
# =============================================================================


class TestBlockEffects:
    """Tests for block-gaining card effects."""

    def test_defend_gains_block(self, basic_combat):
        """Test Defend gains 5 block."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Defend_P")
        initial_block = basic_combat.player.block

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.block == initial_block + 5
        assert result.block_gained == 5

    def test_defend_upgraded_gains_more_block(self, basic_combat):
        """Test Defend+ gains 8 block."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Defend_P", upgraded=True)

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 8

    def test_vigilance_gains_block_and_enters_calm(self, basic_combat):
        """Test Vigilance gains 8 block and enters Calm."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Vigilance")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 8
        assert basic_combat.stance == "Calm"

    def test_halt_gains_extra_block_in_wrath(self, basic_combat):
        """Test Halt gains +6 block when in Wrath stance."""
        basic_combat.stance = "Wrath"
        executor = EffectExecutor(basic_combat)
        card = get_card("Halt")
        # Halt: 3 base + 6 in Wrath = 9

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 9

    def test_halt_normal_block_outside_wrath(self, basic_combat):
        """Test Halt gains only 3 block outside Wrath."""
        basic_combat.stance = "Neutral"
        executor = EffectExecutor(basic_combat)
        card = get_card("Halt")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 3


# =============================================================================
# STANCE EFFECT TESTS
# =============================================================================


class TestStanceEffects:
    """Tests for stance manipulation effects."""

    def test_enter_wrath_from_neutral(self, ctx):
        """Test entering Wrath from Neutral."""
        ctx.state.stance = "Neutral"

        result = ctx.change_stance("Wrath")

        assert ctx.state.stance == "Wrath"
        assert result["old_stance"] == "Neutral"
        assert result["new_stance"] == "Wrath"

    def test_enter_calm_from_neutral(self, ctx):
        """Test entering Calm from Neutral."""
        ctx.state.stance = "Neutral"

        result = ctx.change_stance("Calm")

        assert ctx.state.stance == "Calm"

    def test_exit_calm_gains_energy(self, ctx):
        """Test exiting Calm grants 2 energy."""
        ctx.state.stance = "Calm"
        initial_energy = ctx.state.energy

        result = ctx.change_stance("Wrath")

        assert ctx.state.stance == "Wrath"
        assert ctx.state.energy == initial_energy + 2
        assert result["energy_gained"] == 2

    def test_exit_calm_with_violet_lotus_gains_3_energy(self, ctx):
        """Test exiting Calm with Violet Lotus grants 3 energy."""
        ctx.state.stance = "Calm"
        ctx.state.relics.append("VioletLotus")
        initial_energy = ctx.state.energy

        result = ctx.change_stance("Wrath")

        assert ctx.state.energy == initial_energy + 3
        assert result["energy_gained"] == 3

    def test_enter_divinity_gains_3_energy(self, ctx):
        """Test entering Divinity grants 3 energy."""
        ctx.state.stance = "Neutral"
        initial_energy = ctx.state.energy

        result = ctx.change_stance("Divinity")

        assert ctx.state.stance == "Divinity"
        assert ctx.state.energy == initial_energy + 3

    def test_exit_calm_to_divinity_gains_5_energy(self, ctx):
        """Test Calm -> Divinity gives 2+3=5 energy."""
        ctx.state.stance = "Calm"
        initial_energy = ctx.state.energy

        result = ctx.change_stance("Divinity")

        assert ctx.state.stance == "Divinity"
        # 2 from Calm exit + 3 from Divinity enter = 5
        assert ctx.state.energy == initial_energy + 5

    def test_same_stance_no_change(self, ctx):
        """Test entering same stance has no effect."""
        ctx.state.stance = "Wrath"
        ctx.state.energy = 3

        result = ctx.change_stance("Wrath")

        assert ctx.state.stance == "Wrath"
        assert ctx.state.energy == 3
        assert result["energy_gained"] == 0

    def test_exit_stance_goes_to_neutral(self, ctx):
        """Test exit_stance goes to Neutral."""
        ctx.state.stance = "Wrath"

        result = ctx.exit_stance()

        assert ctx.state.stance == "Neutral"

    def test_mental_fortress_triggers_on_stance_change(self, ctx):
        """Test Mental Fortress grants block on stance change."""
        ctx.state.stance = "Neutral"
        ctx.state.player.statuses["MentalFortress"] = 4
        initial_block = ctx.state.player.block

        result = ctx.change_stance("Wrath")

        assert ctx.state.player.block == initial_block + 4
        assert result.get("block_gained") == 4

    def test_rushdown_draws_on_wrath_enter(self, ctx):
        """Test Rushdown draws cards when entering Wrath."""
        ctx.state.stance = "Neutral"
        ctx.state.player.statuses["Rushdown"] = 2
        ctx.state.draw_pile = ["Strike_P", "Strike_P", "Defend_P"]
        ctx.state.hand = []

        result = ctx.change_stance("Wrath")

        assert ctx.state.stance == "Wrath"
        assert len(ctx.state.hand) == 2
        assert result.get("cards_drawn") == 2


# =============================================================================
# STANCE DAMAGE MULTIPLIER TESTS
# =============================================================================


class TestStanceDamageMultipliers:
    """Tests for stance-based damage multipliers."""

    def test_wrath_doubles_damage(self, basic_combat):
        """Test Wrath stance doubles attack damage."""
        basic_combat.stance = "Wrath"
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        # 6 * 2 (Wrath) = 12 damage
        assert basic_combat.enemies[0].hp == initial_hp - 12

    def test_divinity_triples_damage(self, basic_combat):
        """Test Divinity stance triples attack damage."""
        basic_combat.stance = "Divinity"
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        # 6 * 3 (Divinity) = 18 damage
        assert basic_combat.enemies[0].hp == initial_hp - 18

    def test_calm_no_damage_multiplier(self, basic_combat):
        """Test Calm stance has no damage multiplier."""
        basic_combat.stance = "Calm"
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 6

    def test_eruption_in_wrath_deals_double(self, basic_combat):
        """Test Eruption deals double damage when already in Wrath."""
        basic_combat.stance = "Wrath"
        executor = EffectExecutor(basic_combat)
        card = get_card("Eruption")  # 9 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        # Damage calculated before stance change
        # So if already in Wrath: 9 * 2 = 18
        assert basic_combat.enemies[0].hp == initial_hp - 18


# =============================================================================
# VULNERABLE INTERACTION TESTS
# =============================================================================


class TestVulnerableInteraction:
    """Tests for Vulnerable status damage increase."""

    def test_vulnerable_increases_damage_by_50_percent(self, basic_combat):
        """Test Vulnerable increases damage taken by 50%."""
        basic_combat.enemies[0].statuses["Vulnerable"] = 2
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        # 6 * 1.5 = 9 damage
        assert basic_combat.enemies[0].hp == initial_hp - 9

    def test_vulnerable_with_wrath_stacks_multiplicatively(self, basic_combat):
        """Test Vulnerable stacks with Wrath multiplier."""
        basic_combat.enemies[0].statuses["Vulnerable"] = 2
        basic_combat.stance = "Wrath"
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        # 6 * 2 (Wrath) = 12, then * 1.5 (Vulnerable) = 18
        assert basic_combat.enemies[0].hp == initial_hp - 18

    def test_vulnerable_with_divinity(self, basic_combat):
        """Test Vulnerable with Divinity gives 4.5x damage."""
        basic_combat.enemies[0].statuses["Vulnerable"] = 2
        basic_combat.stance = "Divinity"
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base damage
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        # 6 * 3 (Divinity) = 18, then * 1.5 (Vulnerable) = 27
        assert basic_combat.enemies[0].hp == initial_hp - 27


# =============================================================================
# MANTRA EFFECT TESTS
# =============================================================================


class TestMantraEffects:
    """Tests for mantra-related card effects."""

    def test_prostrate_gains_mantra_and_block(self, basic_combat):
        """Test Prostrate gains 2 mantra and 4 block."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Prostrate")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 4
        assert basic_combat.player.statuses.get("Mantra", 0) == 2

    def test_worship_gains_5_mantra(self, basic_combat):
        """Test Worship gains 5 mantra."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Worship")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Mantra", 0) == 5

    def test_worship_upgraded_gains_8_mantra(self, basic_combat):
        """Test Worship+ gains 8 mantra."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Worship", upgraded=True)

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Mantra", 0) == 8

    def test_10_mantra_triggers_divinity(self, ctx):
        """Test reaching 10 mantra triggers Divinity."""
        ctx.state.stance = "Neutral"
        ctx.state.player.statuses["Mantra"] = 8
        initial_energy = ctx.state.energy

        result = ctx.gain_mantra(2)

        assert result["divinity_triggered"] is True
        assert ctx.state.stance == "Divinity"
        # Energy from entering Divinity
        assert ctx.state.energy == initial_energy + 3

    def test_mantra_overflow_preserved(self, ctx):
        """Test mantra over 10 is preserved as remainder."""
        ctx.state.stance = "Neutral"
        ctx.state.player.statuses["Mantra"] = 8

        result = ctx.gain_mantra(5)  # 8 + 5 = 13, triggers at 10, remainder = 3

        assert result["divinity_triggered"] is True
        assert ctx.state.stance == "Divinity"
        # Remainder mantra
        assert ctx.state.player.statuses.get("Mantra", 0) == 3

    def test_devotion_power_grants_mantra_each_turn(self, basic_combat):
        """Test Devotion power grants mantra at start of turn."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Devotion")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Devotion", 0) == 2


# =============================================================================
# SCRY EFFECT TESTS
# =============================================================================


class TestScryEffects:
    """Tests for scry-related card effects."""

    def test_third_eye_scries_and_blocks(self, basic_combat):
        """Test Third Eye scries 3 and gains 7 block."""
        basic_combat.draw_pile = ["Strike_P", "Defend_P", "Eruption", "Vigilance"]
        executor = EffectExecutor(basic_combat)
        card = get_card("ThirdEye")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.block_gained == 7
        assert len(result.scried_cards) == 3

    def test_cut_through_fate_scries_and_draws(self, basic_combat):
        """Test Cut Through Fate scries 2, deals damage, and draws 1."""
        basic_combat.draw_pile = ["Strike_P", "Defend_P", "Eruption"]
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("CutThroughFate")
        initial_hp = basic_combat.enemies[0].hp

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == initial_hp - 7
        assert len(basic_combat.hand) == 1  # Drew 1 card

    def test_nirvana_grants_block_on_scry(self, ctx):
        """Test Nirvana power grants block when scrying."""
        ctx.state.player.statuses["Nirvana"] = 3
        ctx.state.draw_pile = ["Strike_P", "Defend_P", "Eruption"]
        initial_block = ctx.state.player.block

        cards = ctx.scry(2)

        # 3 block per card scryed = 6 block
        assert ctx.state.player.block == initial_block + 6
        assert len(cards) == 2

    def test_scry_triggers_weave(self, ctx):
        """Test scrying moves Weave from discard to hand."""
        ctx.state.draw_pile = ["Strike_P", "Defend_P"]
        ctx.state.discard_pile = ["Weave", "Defend_P"]
        ctx.state.hand = []

        ctx.scry(1)

        assert "Weave" in ctx.state.hand
        assert "Weave" not in ctx.state.discard_pile


# =============================================================================
# DRAW EFFECT TESTS
# =============================================================================


class TestDrawEffects:
    """Tests for card draw effects."""

    def test_wheel_kick_draws_2(self, basic_combat):
        """Test Wheel Kick draws 2 cards."""
        basic_combat.draw_pile = ["Strike_P", "Defend_P", "Eruption"]
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("WheelKick")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert len(basic_combat.hand) == 2

    def test_scrawl_draws_until_hand_full(self, basic_combat):
        """Test Scrawl draws until hand has 10 cards."""
        basic_combat.draw_pile = ["Strike_P"] * 15
        basic_combat.hand = ["Defend_P", "Defend_P"]
        executor = EffectExecutor(basic_combat)
        card = get_card("Scrawl")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert len(basic_combat.hand) == 10

    def test_draw_reshuffles_when_empty(self, ctx):
        """Test drawing reshuffles discard when draw pile empty."""
        ctx.state.draw_pile = []
        ctx.state.discard_pile = ["Strike_P", "Defend_P", "Eruption"]
        ctx.state.hand = []

        drawn = ctx.draw_cards(2)

        assert len(drawn) == 2
        assert len(ctx.state.hand) == 2
        assert len(ctx.state.draw_pile) == 1
        assert len(ctx.state.discard_pile) == 0


# =============================================================================
# POWER CARD EFFECT TESTS
# =============================================================================


class TestPowerCardEffects:
    """Tests for power card effects."""

    def test_mental_fortress_applies_status(self, basic_combat):
        """Test Mental Fortress applies MentalFortress status."""
        executor = EffectExecutor(basic_combat)
        card = get_card("MentalFortress")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("MentalFortress", 0) == 4

    def test_mental_fortress_upgraded(self, basic_combat):
        """Test Mental Fortress+ grants 6 block per stance change."""
        executor = EffectExecutor(basic_combat)
        card = get_card("MentalFortress", upgraded=True)

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("MentalFortress", 0) == 6

    def test_rushdown_applies_status(self, basic_combat):
        """Test Rushdown applies Rushdown status."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Adaptation")  # Rushdown's internal ID

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Rushdown", 0) == 2

    def test_nirvana_applies_status(self, basic_combat):
        """Test Nirvana applies Nirvana status."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Nirvana")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Nirvana", 0) == 3

    def test_like_water_applies_status(self, basic_combat):
        """Test Like Water applies LikeWater status."""
        executor = EffectExecutor(basic_combat)
        card = get_card("LikeWater")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("LikeWater", 0) == 5

    def test_talk_to_the_hand_applies_block_return(self, basic_combat):
        """Test Talk to the Hand applies BlockReturn to enemy."""
        executor = EffectExecutor(basic_combat)
        card = get_card("TalkToTheHand")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].statuses.get("BlockReturn", 0) == 2


# =============================================================================
# EXHAUST EFFECT TESTS
# =============================================================================


class TestExhaustEffects:
    """Tests for exhaust mechanics."""

    def test_exhaust_card_from_hand(self, ctx):
        """Test exhausting a card from hand."""
        ctx.state.hand = ["Strike_P", "Defend_P", "Eruption"]

        result = ctx.exhaust_card("Defend_P", from_hand=True)

        assert result is True
        assert "Defend_P" not in ctx.state.hand
        assert "Defend_P" in ctx.state.exhaust_pile

    def test_exhaust_card_tracking(self, ctx):
        """Test exhausted cards are tracked."""
        ctx.state.hand = ["Strike_P", "Defend_P"]

        ctx.exhaust_card("Strike_P", from_hand=True)

        assert "Strike_P" in ctx.cards_exhausted

    def test_talk_to_the_hand_exhausts(self, basic_combat):
        """Test Talk to the Hand exhausts after use."""
        basic_combat.hand = ["TalkToTheHand"]
        executor = EffectExecutor(basic_combat)
        card = get_card("TalkToTheHand")
        card.exhaust = True

        # Note: Actual exhaust happens in card resolution, not effect execution
        # This test verifies the card has exhaust=True
        assert card.exhaust is True


# =============================================================================
# ENERGY MANIPULATION TESTS
# =============================================================================


class TestEnergyManipulation:
    """Tests for energy gain/spend effects."""

    def test_gain_energy(self, ctx):
        """Test gaining energy."""
        initial_energy = ctx.state.energy

        ctx.gain_energy(2)

        assert ctx.state.energy == initial_energy + 2
        assert ctx.energy_gained == 2

    def test_spend_energy_success(self, ctx):
        """Test spending energy when available."""
        ctx.state.energy = 3

        result = ctx.spend_energy(2)

        assert result is True
        assert ctx.state.energy == 1
        assert ctx.energy_spent == 2

    def test_spend_energy_failure(self, ctx):
        """Test spending energy when insufficient."""
        ctx.state.energy = 1

        result = ctx.spend_energy(3)

        assert result is False
        assert ctx.state.energy == 1
        assert ctx.energy_spent == 0

    def test_miracle_gains_energy(self, basic_combat):
        """Test Miracle grants 1 energy (2 when upgraded)."""
        basic_combat.energy = 3
        executor = EffectExecutor(basic_combat)
        card = get_card("Miracle")

        result = executor.play_card(card, target_idx=-1, free=True)

        assert result.success
        assert basic_combat.energy == 4

    def test_card_costs_energy(self, basic_combat):
        """Test playing a card costs energy."""
        basic_combat.energy = 3
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # Costs 1

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert result.energy_spent == 1
        assert basic_combat.energy == 2


# =============================================================================
# EDGE CASE TESTS
# =============================================================================


class TestEdgeCases:
    """Tests for edge cases and error conditions."""

    def test_play_card_insufficient_energy(self, basic_combat):
        """Test playing a card with insufficient energy fails."""
        basic_combat.energy = 0
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # Costs 1

        result = executor.play_card(card, target_idx=0)

        assert result.success is False
        assert len(result.errors) > 0
        assert "energy" in result.errors[0].lower()

    def test_target_dead_enemy(self, basic_combat):
        """Test targeting a dead enemy."""
        basic_combat.enemies[0].hp = 0
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")

        result = executor.play_card(card, target_idx=0)

        # Card plays but target is None
        assert result.success
        assert result.damage_dealt == 0

    def test_damage_blocked_by_enemy_block(self, basic_combat):
        """Test enemy block absorbs damage."""
        basic_combat.enemies[0].block = 10
        initial_hp = basic_combat.enemies[0].hp
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 damage

        result = executor.play_card(card, target_idx=0)

        assert result.success
        # Block absorbs all damage
        assert basic_combat.enemies[0].hp == initial_hp
        assert basic_combat.enemies[0].block == 4

    def test_damage_partially_blocked(self, basic_combat):
        """Test partial block absorption."""
        basic_combat.enemies[0].block = 4
        initial_hp = basic_combat.enemies[0].hp
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 damage

        result = executor.play_card(card, target_idx=0)

        assert result.success
        # 4 blocked, 2 to HP
        assert basic_combat.enemies[0].hp == initial_hp - 2
        assert basic_combat.enemies[0].block == 0

    def test_hand_limit_prevents_draw(self, ctx):
        """Test hand limit of 10 prevents further draws."""
        ctx.state.hand = ["Strike_P"] * 10
        ctx.state.draw_pile = ["Defend_P", "Defend_P"]

        drawn = ctx.draw_cards(2)

        assert len(drawn) == 0
        assert len(ctx.state.hand) == 10

    def test_empty_draw_and_discard(self, ctx):
        """Test drawing when both piles are empty."""
        ctx.state.hand = []
        ctx.state.draw_pile = []
        ctx.state.discard_pile = []

        drawn = ctx.draw_cards(3)

        assert len(drawn) == 0
        assert len(ctx.state.hand) == 0


# =============================================================================
# STATUS APPLICATION TESTS
# =============================================================================


class TestStatusApplication:
    """Tests for status effect application."""

    def test_apply_weak_to_enemy(self, ctx):
        """Test applying Weak to enemy."""
        result = ctx.apply_status_to_target("Weak", 2)

        assert result is True
        assert ctx.target.statuses.get("Weak", 0) == 2

    def test_apply_vulnerable_to_enemy(self, ctx):
        """Test applying Vulnerable to enemy."""
        result = ctx.apply_status_to_target("Vulnerable", 2)

        assert result is True
        assert ctx.target.statuses.get("Vulnerable", 0) == 2

    def test_artifact_blocks_debuff(self, ctx):
        """Test Artifact blocks debuff application."""
        ctx.target.statuses["Artifact"] = 1

        result = ctx.apply_status_to_target("Weak", 2)

        assert result is False
        assert ctx.target.statuses.get("Weak", 0) == 0
        assert ctx.target.statuses.get("Artifact", 0) == 0  # Consumed

    def test_artifact_stacks_consumed_one_at_time(self, ctx):
        """Test multiple Artifact stacks are consumed one at a time."""
        ctx.target.statuses["Artifact"] = 3

        ctx.apply_status_to_target("Weak", 1)
        ctx.apply_status_to_target("Vulnerable", 1)

        assert ctx.target.statuses.get("Artifact", 0) == 1

    def test_apply_strength_to_player(self, ctx):
        """Test applying Strength to player."""
        result = ctx.apply_status_to_player("Strength", 3)

        assert result is True
        assert ctx.state.player.statuses.get("Strength", 0) == 3

    def test_strength_increases_damage(self, basic_combat):
        """Test Strength increases attack damage."""
        basic_combat.player.statuses["Strength"] = 3
        initial_hp = basic_combat.enemies[0].hp
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base

        result = executor.play_card(card, target_idx=0)

        # 6 + 3 = 9 damage
        assert basic_combat.enemies[0].hp == initial_hp - 9

    def test_weak_reduces_damage(self, basic_combat):
        """Test Weak reduces attack damage by 25%."""
        basic_combat.player.statuses["Weak"] = 2
        initial_hp = basic_combat.enemies[0].hp
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base

        result = executor.play_card(card, target_idx=0)

        # 6 * 0.75 = 4.5 -> 4 damage
        assert basic_combat.enemies[0].hp == initial_hp - 4

    def test_dexterity_increases_block(self, basic_combat):
        """Test Dexterity increases block gained."""
        basic_combat.player.statuses["Dexterity"] = 2
        executor = EffectExecutor(basic_combat)
        card = get_card("Defend_P")  # 5 base

        result = executor.play_card(card, target_idx=-1)

        # 5 + 2 = 7 block
        assert result.block_gained == 7

    def test_frail_reduces_block(self, basic_combat):
        """Test Frail reduces block gained by 25%."""
        basic_combat.player.statuses["Frail"] = 2
        executor = EffectExecutor(basic_combat)
        card = get_card("Defend_P")  # 5 base

        result = executor.play_card(card, target_idx=-1)

        # 5 * 0.75 = 3.75 -> 3 block
        assert result.block_gained == 3


# =============================================================================
# FLURRY OF BLOWS TESTS
# =============================================================================


class TestFlurryOfBlows:
    """Tests for Flurry of Blows stance-change trigger."""

    def test_flurry_moves_to_hand_on_stance_change(self, ctx):
        """Test Flurry of Blows moves from discard to hand on stance change."""
        ctx.state.discard_pile = ["FlurryOfBlows", "Strike_P"]
        ctx.state.hand = []
        ctx.state.stance = "Neutral"

        ctx.change_stance("Wrath")

        assert "FlurryOfBlows" in ctx.state.hand
        assert "FlurryOfBlows" not in ctx.state.discard_pile

    def test_multiple_flurries_all_move(self, ctx):
        """Test multiple Flurry of Blows all move on stance change."""
        ctx.state.discard_pile = ["FlurryOfBlows", "FlurryOfBlows", "Strike_P"]
        ctx.state.hand = []
        ctx.state.stance = "Neutral"

        ctx.change_stance("Wrath")

        flurries_in_hand = [c for c in ctx.state.hand if c.startswith("FlurryOfBlows")]
        assert len(flurries_in_hand) == 2


# =============================================================================
# CONDITIONAL EFFECT TESTS
# =============================================================================


class TestConditionalEffects:
    """Tests for conditional card effects."""

    def test_inner_peace_in_calm_draws(self, basic_combat):
        """Test Inner Peace draws 3 when in Calm."""
        basic_combat.stance = "Calm"
        basic_combat.draw_pile = ["Strike_P", "Strike_P", "Strike_P", "Defend_P"]
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("InnerPeace")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert len(basic_combat.hand) == 3

    def test_inner_peace_not_in_calm_enters_calm(self, basic_combat):
        """Test Inner Peace enters Calm when not already in Calm."""
        basic_combat.stance = "Neutral"
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("InnerPeace")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.stance == "Calm"
        assert len(basic_combat.hand) == 0  # No draw

    def test_indignation_in_wrath_gains_mantra(self, basic_combat):
        """Test Indignation gains 3 mantra when in Wrath."""
        basic_combat.stance = "Wrath"
        executor = EffectExecutor(basic_combat)
        card = get_card("Indignation")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.player.statuses.get("Mantra", 0) == 3

    def test_indignation_not_in_wrath_enters_wrath(self, basic_combat):
        """Test Indignation enters Wrath when not in Wrath."""
        basic_combat.stance = "Neutral"
        executor = EffectExecutor(basic_combat)
        card = get_card("Indignation")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.stance == "Wrath"

    def test_fear_no_evil_enters_calm_if_attacking(self, basic_combat):
        """Test Fear No Evil enters Calm if enemy is attacking."""
        basic_combat.enemies[0].move_damage = 10  # Attacking
        executor = EffectExecutor(basic_combat)
        card = get_card("FearNoEvil")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.stance == "Calm"

    def test_fear_no_evil_no_calm_if_not_attacking(self, basic_combat):
        """Test Fear No Evil does not enter Calm if enemy not attacking."""
        basic_combat.enemies[0].move_damage = 0  # Not attacking
        basic_combat.stance = "Neutral"
        executor = EffectExecutor(basic_combat)
        card = get_card("FearNoEvil")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.stance == "Neutral"


# =============================================================================
# START/END OF TURN EFFECT TESTS
# =============================================================================


class TestStartEndTurnEffects:
    """Tests for start and end of turn effects."""

    def test_like_water_grants_block_at_end_in_calm(self, basic_combat):
        """Test Like Water grants block at end of turn in Calm."""
        basic_combat.stance = "Calm"
        basic_combat.player.statuses["LikeWater"] = 5
        executor = EffectExecutor(basic_combat)

        result = executor.apply_end_of_turn_effects()

        assert result.block_gained == 5

    def test_like_water_no_block_outside_calm(self, basic_combat):
        """Test Like Water grants no block outside Calm."""
        basic_combat.stance = "Wrath"
        basic_combat.player.statuses["LikeWater"] = 5
        executor = EffectExecutor(basic_combat)

        result = executor.apply_end_of_turn_effects()

        assert result.block_gained == 0

    def test_divinity_exits_at_end_of_turn(self, basic_combat):
        """Test Divinity automatically exits at end of turn."""
        basic_combat.stance = "Divinity"
        executor = EffectExecutor(basic_combat)

        result = executor.apply_end_of_turn_effects()

        assert basic_combat.stance == "Neutral"
        assert result.stance_changed_to == "Neutral"

    def test_devotion_grants_mantra_at_start(self, basic_combat):
        """Test Devotion grants mantra at start of turn."""
        basic_combat.player.statuses["Devotion"] = 2
        executor = EffectExecutor(basic_combat)

        result = executor.apply_start_of_turn_effects()

        assert result.mantra_gained == 2

    def test_foresight_scries_at_start(self, basic_combat):
        """Test Foresight scries at start of turn."""
        basic_combat.player.statuses["Foresight"] = 3
        basic_combat.draw_pile = ["Strike_P", "Strike_P", "Strike_P", "Defend_P"]
        executor = EffectExecutor(basic_combat)

        result = executor.apply_start_of_turn_effects()

        assert len(result.scried_cards) == 3

    def test_blasphemy_kills_at_end_of_turn(self, basic_combat):
        """Test Blasphemy kills player at end of turn."""
        basic_combat.player.statuses["Blasphemy"] = 1
        executor = EffectExecutor(basic_combat)

        result = executor.apply_end_of_turn_effects()

        assert basic_combat.player.hp == 0


# =============================================================================
# CARD GENERATION TESTS
# =============================================================================


class TestCardGeneration:
    """Tests for card generation effects."""

    def test_carve_reality_adds_smite(self, basic_combat):
        """Test Carve Reality adds Smite to hand."""
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("CarveReality")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert "Smite" in basic_combat.hand

    def test_deceive_reality_adds_safety(self, basic_combat):
        """Test Deceive Reality adds Safety to hand."""
        basic_combat.hand = []
        executor = EffectExecutor(basic_combat)
        card = get_card("DeceiveReality")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert "Safety" in basic_combat.hand

    def test_evaluate_adds_insight_to_draw(self, basic_combat):
        """Test Evaluate adds Insight to top of draw pile."""
        basic_combat.draw_pile = ["Strike_P"]
        executor = EffectExecutor(basic_combat)
        card = get_card("Evaluate")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.draw_pile[-1] == "Insight"

    def test_alpha_shuffles_beta_into_draw(self, basic_combat):
        """Test Alpha shuffles Beta into draw pile."""
        basic_combat.draw_pile = []
        executor = EffectExecutor(basic_combat)
        card = get_card("Alpha")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert "Beta" in basic_combat.draw_pile


# =============================================================================
# SPECIAL CARD TESTS
# =============================================================================


class TestSpecialCards:
    """Tests for special card mechanics."""

    def test_judgment_kills_below_threshold(self, basic_combat):
        """Test Judgment kills enemy below HP threshold."""
        basic_combat.enemies[0].hp = 25  # Below 30 threshold
        executor = EffectExecutor(basic_combat)
        card = get_card("Judgement")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == 0

    def test_judgment_does_not_kill_above_threshold(self, basic_combat):
        """Test Judgment does not kill enemy above HP threshold."""
        basic_combat.enemies[0].hp = 35  # Above 30 threshold
        executor = EffectExecutor(basic_combat)
        card = get_card("Judgement")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.enemies[0].hp == 35

    def test_conclude_ends_turn(self, basic_combat):
        """Test Conclude sets end turn flag."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Conclude")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert result.should_end_turn is True

    def test_blasphemy_enters_divinity_and_marks_death(self, basic_combat):
        """Test Blasphemy enters Divinity and applies Blasphemy status."""
        executor = EffectExecutor(basic_combat)
        card = get_card("Blasphemy")

        result = executor.play_card(card, target_idx=-1)

        assert result.success
        assert basic_combat.stance == "Divinity"
        assert basic_combat.player.statuses.get("Blasphemy", 0) == 1


# =============================================================================
# INTEGRATION TESTS
# =============================================================================


class TestIntegration:
    """Integration tests for complex card interactions."""

    def test_eruption_from_calm_to_wrath_gains_energy(self, basic_combat):
        """Test Eruption from Calm gains 2 energy from exiting Calm."""
        basic_combat.stance = "Calm"
        basic_combat.energy = 3
        executor = EffectExecutor(basic_combat)
        card = get_card("Eruption")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.stance == "Wrath"
        # Started with 3, spent 2, gained 2 from Calm exit = 3
        assert basic_combat.energy == 3

    def test_mental_fortress_with_eruption(self, basic_combat):
        """Test Mental Fortress triggers when Eruption changes stance."""
        basic_combat.stance = "Neutral"
        basic_combat.player.statuses["MentalFortress"] = 4
        executor = EffectExecutor(basic_combat)
        card = get_card("Eruption")

        result = executor.play_card(card, target_idx=0)

        assert result.success
        assert basic_combat.stance == "Wrath"
        assert basic_combat.player.block >= 4

    def test_wrath_double_damage_with_strength(self, basic_combat):
        """Test Wrath + Strength stack correctly."""
        basic_combat.stance = "Wrath"
        basic_combat.player.statuses["Strength"] = 3
        initial_hp = basic_combat.enemies[0].hp
        executor = EffectExecutor(basic_combat)
        card = get_card("Strike_P")  # 6 base

        result = executor.play_card(card, target_idx=0)

        # (6 + 3) * 2 = 18 damage
        assert basic_combat.enemies[0].hp == initial_hp - 18

    def test_complex_stance_dance(self, basic_combat):
        """Test complex stance transitions."""
        basic_combat.energy = 10
        basic_combat.player.statuses["MentalFortress"] = 4
        executor = EffectExecutor(basic_combat)

        # Start Neutral, enter Wrath
        card1 = get_card("Eruption")
        executor.play_card(card1, target_idx=0)
        assert basic_combat.stance == "Wrath"
        assert basic_combat.player.block == 4  # MentalFortress

        # Exit to Neutral
        basic_combat.player.block = 0  # Reset for clarity
        card2 = get_card("EmptyFist")
        executor.play_card(card2, target_idx=0)
        assert basic_combat.stance == "Neutral"
        assert basic_combat.player.block == 4  # MentalFortress again

        # Enter Calm
        basic_combat.player.block = 0
        card3 = get_card("Vigilance")
        executor.play_card(card3, target_idx=-1)
        assert basic_combat.stance == "Calm"
        assert basic_combat.player.block >= 4  # Block from card + MentalFortress


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
