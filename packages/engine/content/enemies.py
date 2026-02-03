"""
Enemy AI System - Exact replication from decompiled AbstractMonster and enemy classes.

This module contains the complete enemy AI system including:
- Data types (Intent, EnemyType, MoveInfo, EnemyState)
- AI logic for all enemy classes
- Factory function and registry

This module is self-contained: AI logic, data definitions, and helper functions.

AI Decision Flow (from source):
1. rollMove() is called at end of turn
2. rollMove() calls getMove(AbstractDungeon.aiRng.random(99))
3. getMove(num) uses the 0-99 roll to select next move
4. Move is stored via setMove() with intent and damage
5. takeTurn() executes the selected move

Key patterns:
- Most enemies use weighted random with anti-repeat rules
- lastMove(byte) checks if previous move matches
- lastTwoMoves(byte) checks if last TWO moves match
- Secondary RNG calls (aiRng.randomBoolean) for tie-breaking

Intent types (from AbstractMonster.Intent):
- ATTACK: Pure damage
- ATTACK_BUFF: Attack + buff self
- ATTACK_DEBUFF: Attack + debuff player
- ATTACK_DEFEND: Attack + gain block
- BUFF: Buff only
- DEBUFF: Debuff player
- STRONG_DEBUFF: Strong debuff (e.g., Slimed)
- DEFEND: Block only
- DEFEND_BUFF: Block + buff
- UNKNOWN: Hidden intent (e.g., before split)
"""

from dataclasses import dataclass, field
from typing import Any, List, Optional, Dict, Tuple, Callable
from enum import Enum
import math

from ..state.rng import Random



class Intent(Enum):
    """Enemy intent types matching AbstractMonster.Intent."""
    ATTACK = "ATTACK"
    ATTACK_BUFF = "ATTACK_BUFF"
    ATTACK_DEBUFF = "ATTACK_DEBUFF"
    ATTACK_DEFEND = "ATTACK_DEFEND"
    BUFF = "BUFF"
    DEBUFF = "DEBUFF"
    STRONG_DEBUFF = "STRONG_DEBUFF"
    DEFEND = "DEFEND"
    DEFEND_BUFF = "DEFEND_BUFF"
    DEFEND_DEBUFF = "DEFEND_DEBUFF"
    ESCAPE = "ESCAPE"
    MAGIC = "MAGIC"
    SLEEP = "SLEEP"
    STUN = "STUN"
    UNKNOWN = "UNKNOWN"
    NONE = "NONE"


class EnemyType(Enum):
    """Enemy types matching AbstractMonster.EnemyType."""
    NORMAL = "NORMAL"
    ELITE = "ELITE"
    BOSS = "BOSS"


@dataclass
class MoveInfo:
    """Information about an enemy move."""
    move_id: int
    name: str
    intent: Intent
    base_damage: int = -1
    hits: int = 1
    is_multi: bool = False
    block: int = 0
    effects: Dict = field(default_factory=dict)  # Additional effects


@dataclass
class EnemyState:
    """Current state of an enemy during combat."""
    id: str
    name: str
    enemy_type: EnemyType

    # HP
    current_hp: int
    max_hp: int

    # Combat state
    block: int = 0
    strength: int = 0
    powers: Dict[str, int] = field(default_factory=dict)

    # Move tracking
    move_history: List[int] = field(default_factory=list)
    next_move: Optional[MoveInfo] = None
    first_turn: bool = True

    # Context from CombatEngine (set before roll_move)
    player_hp: int = 0
    num_allies: int = 0

    def last_move(self, move_id: int) -> bool:
        """Check if last move was the given ID."""
        if not self.move_history:
            return False
        return self.move_history[-1] == move_id

    def last_two_moves(self, move_id: int) -> bool:
        """Check if last TWO moves were both the given ID."""
        if len(self.move_history) < 2:
            return False
        return (self.move_history[-1] == move_id and
                self.move_history[-2] == move_id)

    def last_move_before(self, move_id: int) -> bool:
        """Check if move before last was the given ID."""
        if len(self.move_history) < 2:
            return False
        return self.move_history[-2] == move_id


# ============ BASE ENEMY CLASS ============

class Enemy:
    """Base class for all enemies."""

    ID = "Unknown"
    NAME = "Unknown"
    TYPE = EnemyType.NORMAL

    # Move IDs (override in subclass)
    MOVES: Dict[int, str] = {}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        """
        Initialize enemy.

        Args:
            ai_rng: RNG for AI decisions
            ascension: Ascension level (affects stats/AI)
            hp_rng: RNG for HP roll (monsterHpRng)
        """
        self.ai_rng = ai_rng
        self.ascension = ascension
        self.hp_rng = hp_rng or ai_rng

        # Roll HP and create state
        min_hp, max_hp = self._get_hp_range()
        hp = self.hp_rng.random_range(min_hp, max_hp)

        self.state = EnemyState(
            id=self.ID,
            name=self.NAME,
            enemy_type=self.TYPE,
            current_hp=hp,
            max_hp=hp,
        )

    def _get_hp_range(self) -> Tuple[int, int]:
        """Get (min_hp, max_hp) based on ascension. Override in subclass."""
        return (1, 1)

    def _get_damage_values(self) -> Dict[str, int]:
        """Get damage values based on ascension. Override in subclass."""
        return {}

    def roll_move(self) -> MoveInfo:
        """Roll next move using AI RNG."""
        roll = self.ai_rng.random(99)  # 0-99 inclusive
        return self.get_move(roll)

    def get_move(self, roll: int) -> MoveInfo:
        """
        Determine next move based on RNG roll.

        Args:
            roll: Random 0-99 value from aiRng

        Returns:
            MoveInfo for the selected move
        """
        raise NotImplementedError("Subclass must implement get_move()")

    def set_move(self, move: MoveInfo):
        """Set the next move and update history."""
        if move.move_id != -1:
            self.state.move_history.append(move.move_id)
        self.state.next_move = move

    def take_turn(self) -> Dict:
        """
        Execute current move.

        Returns dict with:
            - damage: Total damage dealt
            - block: Block gained
            - effects: Any status effects applied
        """
        move = self.state.next_move
        if not move:
            return {"damage": 0, "block": 0, "effects": {}}

        result = {
            "move_id": move.move_id,
            "move_name": move.name,
            "intent": move.intent,
            "damage": move.base_damage if move.base_damage > 0 else 0,
            "hits": move.hits,
            "total_damage": (move.base_damage * move.hits) if move.base_damage > 0 else 0,
            "block": move.block,
            "effects": move.effects.copy(),
        }

        # Apply strength to damage
        if result["damage"] > 0:
            result["damage"] += self.state.strength
            result["total_damage"] = result["damage"] * result["hits"]

        return result


# ============ EXORDIUM ENEMIES ============

class JawWorm(Enemy):
    """
    Jaw Worm - Common enemy, first encounter in game.

    Moves:
    - CHOMP (1): Attack 11/12 damage
    - BELLOW (2): +3/4/5 Strength, gain 6/9 block
    - THRASH (3): Attack 7 damage + gain 5 block

    AI Pattern (from decompiled JawWorm.java):
    - First turn: Always CHOMP
    - Roll 0-24 (25%):
      - If last was CHOMP: 56.25% BELLOW, else THRASH
      - Else: CHOMP
    - Roll 25-54 (30%):
      - If last TWO were THRASH: 35.7% CHOMP, else BELLOW
      - Else: THRASH
    - Roll 55-99 (45%):
      - If last was BELLOW: 41.6% CHOMP, else THRASH
      - Else: BELLOW
    """

    ID = "JawWorm"
    NAME = "Jaw Worm"
    TYPE = EnemyType.NORMAL

    # Move IDs
    CHOMP = 1
    BELLOW = 2
    THRASH = 3

    MOVES = {1: "Chomp", 2: "Bellow", 3: "Thrash"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 hard_mode: bool = False):
        """
        Args:
            hard_mode: If True, starts with Strength/Block (Act 3 Endless)
        """
        self.hard_mode = hard_mode
        super().__init__(ai_rng, ascension, hp_rng)

        if hard_mode:
            self.state.first_turn = False

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (42, 46)
        return (40, 44)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 17:
            return {
                "chomp": 12,
                "thrash": 7,
                "thrash_block": 5,
                "bellow_str": 5,
                "bellow_block": 9,
            }
        elif self.ascension >= 2:
            return {
                "chomp": 12,
                "thrash": 7,
                "thrash_block": 5,
                "bellow_str": 4,
                "bellow_block": 6,
            }
        return {
            "chomp": 11,
            "thrash": 7,
            "thrash_block": 5,
            "bellow_str": 3,
            "bellow_block": 6,
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # First turn: always CHOMP
        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(
                move_id=self.CHOMP,
                name="Chomp",
                intent=Intent.ATTACK,
                base_damage=dmg["chomp"],
            )
            self.set_move(move)
            return move

        # Roll-based decision tree
        if roll < 25:
            # 25% chance branch
            if self.state.last_move(self.CHOMP):
                # Can't repeat CHOMP, choose between BELLOW and THRASH
                if self.ai_rng.random_boolean(0.5625):
                    move = MoveInfo(
                        move_id=self.BELLOW,
                        name="Bellow",
                        intent=Intent.DEFEND_BUFF,
                        block=dmg["bellow_block"],
                        effects={"strength": dmg["bellow_str"]},
                    )
                else:
                    move = MoveInfo(
                        move_id=self.THRASH,
                        name="Thrash",
                        intent=Intent.ATTACK_DEFEND,
                        base_damage=dmg["thrash"],
                        block=dmg["thrash_block"],
                    )
            else:
                move = MoveInfo(
                    move_id=self.CHOMP,
                    name="Chomp",
                    intent=Intent.ATTACK,
                    base_damage=dmg["chomp"],
                )

        elif roll < 55:
            # 30% chance branch
            if self.state.last_two_moves(self.THRASH):
                # Can't do THRASH 3x, choose between CHOMP and BELLOW
                if self.ai_rng.random_boolean(0.357):
                    move = MoveInfo(
                        move_id=self.CHOMP,
                        name="Chomp",
                        intent=Intent.ATTACK,
                        base_damage=dmg["chomp"],
                    )
                else:
                    move = MoveInfo(
                        move_id=self.BELLOW,
                        name="Bellow",
                        intent=Intent.DEFEND_BUFF,
                        block=dmg["bellow_block"],
                        effects={"strength": dmg["bellow_str"]},
                    )
            else:
                move = MoveInfo(
                    move_id=self.THRASH,
                    name="Thrash",
                    intent=Intent.ATTACK_DEFEND,
                    base_damage=dmg["thrash"],
                    block=dmg["thrash_block"],
                )

        else:
            # 45% chance branch
            if self.state.last_move(self.BELLOW):
                # Can't repeat BELLOW, choose between CHOMP and THRASH
                if self.ai_rng.random_boolean(0.416):
                    move = MoveInfo(
                        move_id=self.CHOMP,
                        name="Chomp",
                        intent=Intent.ATTACK,
                        base_damage=dmg["chomp"],
                    )
                else:
                    move = MoveInfo(
                        move_id=self.THRASH,
                        name="Thrash",
                        intent=Intent.ATTACK_DEFEND,
                        base_damage=dmg["thrash"],
                        block=dmg["thrash_block"],
                    )
            else:
                move = MoveInfo(
                    move_id=self.BELLOW,
                    name="Bellow",
                    intent=Intent.DEFEND_BUFF,
                    block=dmg["bellow_block"],
                    effects={"strength": dmg["bellow_str"]},
                )

        self.set_move(move)
        return move


class Cultist(Enemy):
    """
    Cultist - Simple enemy with ramping damage.

    Moves:
    - INCANTATION (1): Gain 3/4/5 Ritual (strength per turn)
    - DARK_STRIKE (2): Attack 6 damage

    AI Pattern:
    - First turn: Always INCANTATION
    - After: Always DARK_STRIKE
    """

    ID = "Cultist"
    NAME = "Cultist"
    TYPE = EnemyType.NORMAL

    DARK_STRIKE = 1
    INCANTATION = 3

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (50, 56)
        return (48, 54)

    def _get_damage_values(self) -> Dict[str, int]:
        ritual = 3
        if self.ascension >= 17:
            ritual = 5
        elif self.ascension >= 2:
            ritual = 4
        return {"dark_strike": 6, "ritual": ritual}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(
                move_id=self.INCANTATION,
                name="Incantation",
                intent=Intent.BUFF,
                effects={"ritual": dmg["ritual"]},
            )
        else:
            move = MoveInfo(
                move_id=self.DARK_STRIKE,
                name="Dark Strike",
                intent=Intent.ATTACK,
                base_damage=dmg["dark_strike"],
            )

        self.set_move(move)
        return move


class AcidSlimeM(Enemy):
    """
    Medium Acid Slime.

    Moves:
    - CORROSIVE_SPIT (1): 7/8 damage + 1 Slimed (W_TACKLE_DMG)
    - TACKLE (2): 10/12 damage (N_TACKLE_DMG)
    - LICK (4): Apply 1 Weak

    AI Pattern (A17+):
    - Roll 0-39 (40%): CORROSIVE_SPIT (no 3x repeat, else 50% TACKLE / 50% LICK)
    - Roll 40-79 (40%): TACKLE (no 3x repeat, else 50% SPIT / 50% LICK)
    - Roll 80-99 (20%): LICK (no repeat, else 40% SPIT / 60% TACKLE)

    AI Pattern (below A17):
    - Roll 0-29 (30%): CORROSIVE_SPIT (no 3x repeat, else 50% TACKLE / 50% LICK)
    - Roll 30-69 (40%): TACKLE (no repeat, else 40% SPIT / 60% LICK)
    - Roll 70-99 (30%): LICK (no 3x repeat, else 40% SPIT / 60% TACKLE)
    """

    ID = "AcidSlime_M"
    NAME = "Acid Slime (M)"
    TYPE = EnemyType.NORMAL

    CORROSIVE_SPIT = 1
    TACKLE = 2
    LICK = 4

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0, starting_hp: Optional[int] = None):
        """
        Args:
            poison_amount: Starting poison stacks (from split)
            starting_hp: Override HP (used when spawned from Large slime split)
        """
        self.poison_amount = poison_amount
        self.starting_hp = starting_hp
        super().__init__(ai_rng, ascension, hp_rng)
        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        # If spawned from split, use the starting HP directly
        if self.starting_hp is not None:
            return (self.starting_hp, self.starting_hp)
        if self.ascension >= 7:
            return (29, 34)
        return (28, 32)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"spit": 8, "tackle": 12}
        return {"spit": 7, "tackle": 10}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        weak_amount = 1  # Always 1 for medium slime

        if self.ascension >= 17:
            # A17+ pattern
            if roll < 40:
                if self.state.last_two_moves(self.CORROSIVE_SPIT):
                    if self.ai_rng.random_boolean(0.5):
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                   dmg["spit"], effects={"slimed": 1})
            elif roll < 80:
                if self.state.last_two_moves(self.TACKLE):
                    if self.ai_rng.random_boolean(0.5):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 1})
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                if self.state.last_move(self.LICK):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 1})
                    else:
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
        else:
            # Below A17 pattern
            if roll < 30:
                if self.state.last_two_moves(self.CORROSIVE_SPIT):
                    if self.ai_rng.random_boolean(0.5):
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                   dmg["spit"], effects={"slimed": 1})
            elif roll < 70:
                if self.state.last_move(self.TACKLE):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 1})
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                if self.state.last_two_moves(self.LICK):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 1})
                    else:
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})

        self.set_move(move)
        return move


class SpikeSlimeM(Enemy):
    """
    Medium Spike Slime.

    Moves:
    - FLAME_TACKLE (1): 8/10 damage + 1 Slimed
    - LICK (4): Apply 1/2 Frail

    AI Pattern (A17+):
    - Roll 0-29 (30%): FLAME_TACKLE (no 3x repeat, else LICK)
    - Roll 30-99 (70%): LICK (no repeat, else FLAME_TACKLE)

    AI Pattern (below A17):
    - Roll 0-29 (30%): FLAME_TACKLE (no 3x repeat, else LICK)
    - Roll 30-99 (70%): LICK (no 3x repeat, else FLAME_TACKLE)
    """

    ID = "SpikeSlime_M"
    NAME = "Spike Slime (M)"
    TYPE = EnemyType.NORMAL

    FLAME_TACKLE = 1
    LICK = 4

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0, starting_hp: Optional[int] = None):
        """
        Args:
            poison_amount: Starting poison stacks (from split)
            starting_hp: Override HP (used when spawned from Large slime split)
        """
        self.poison_amount = poison_amount
        self.starting_hp = starting_hp
        super().__init__(ai_rng, ascension, hp_rng)
        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        # If spawned from split, use the starting HP directly
        if self.starting_hp is not None:
            return (self.starting_hp, self.starting_hp)
        if self.ascension >= 7:
            return (29, 34)
        return (28, 32)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"tackle": 10}
        return {"tackle": 8}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        frail = 2 if self.ascension >= 17 else 1

        if self.ascension >= 17:
            if roll < 30:
                if self.state.last_two_moves(self.FLAME_TACKLE):
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail})
                else:
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 1})
            else:
                if self.state.last_move(self.LICK):
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 1})
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail})
        else:
            if roll < 30:
                if self.state.last_two_moves(self.FLAME_TACKLE):
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail})
                else:
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 1})
            else:
                if self.state.last_two_moves(self.LICK):
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 1})
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail})

        self.set_move(move)
        return move


# ============ LARGE AND SMALL SLIME VARIANTS ============

class AcidSlimeL(Enemy):
    """
    Large Acid Slime - Splits into 2 Medium Acid Slimes at 50% HP.

    Moves:
    - CORROSIVE_SPIT (1): 11/12 damage + 2 Slimed (W_TACKLE_DMG)
    - TACKLE (2): 16/18 damage (N_TACKLE_DMG)
    - SPLIT (3): Split into 2 AcidSlime_M at 50% HP
    - LICK (4): Apply 2 Weak

    AI Pattern (A17+):
    - Roll 0-39 (40%): CORROSIVE_SPIT (no 3x repeat, else 60% TACKLE / 40% LICK)
    - Roll 40-69 (30%): TACKLE (no 3x repeat, else 60% SPIT / 40% LICK)
    - Roll 70-99 (30%): LICK (no repeat, else 40% SPIT / 60% TACKLE)

    AI Pattern (below A17):
    - Roll 0-29 (30%): CORROSIVE_SPIT (no 3x repeat, else 50% TACKLE / 50% LICK)
    - Roll 30-69 (40%): TACKLE (no repeat, else 40% SPIT / 60% LICK)
    - Roll 70-99 (30%): LICK (no 3x repeat, else 40% SPIT / 60% TACKLE)

    Split Mechanics:
    - Has Split power (visible to player)
    - When HP <= 50% max HP, interrupts current action and sets next move to SPLIT
    - On SPLIT: Dies and spawns 2 AcidSlime_M with HP equal to current HP
    """

    ID = "AcidSlime_L"
    NAME = "Acid Slime (L)"
    TYPE = EnemyType.NORMAL

    CORROSIVE_SPIT = 1
    TACKLE = 2
    SPLIT = 3
    LICK = 4

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0, starting_hp: Optional[int] = None):
        """
        Args:
            poison_amount: Starting poison stacks
            starting_hp: Override HP (used in boss fight)
        """
        self.poison_amount = poison_amount
        self.starting_hp = starting_hp
        self.split_triggered = False
        super().__init__(ai_rng, ascension, hp_rng)

        # Apply Split power
        self.state.powers["split"] = 1

        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.starting_hp is not None:
            return (self.starting_hp, self.starting_hp)
        if self.ascension >= 7:
            return (68, 72)
        return (65, 69)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"spit": 12, "tackle": 18}
        return {"spit": 11, "tackle": 16}

    def check_split(self, current_hp: int) -> bool:
        """
        Check if the slime should split.
        Call this after taking damage.

        Returns True if split should be triggered.
        """
        if self.split_triggered:
            return False

        if current_hp <= self.state.max_hp // 2:
            self.split_triggered = True
            # Set next move to SPLIT
            move = MoveInfo(self.SPLIT, "Split", Intent.UNKNOWN)
            self.set_move(move)
            return True

        return False

    def get_split_spawn_info(self) -> Dict:
        """
        Get information needed to spawn the split slimes.

        Returns dict with:
            - enemy_class: "AcidSlime_M"
            - hp: Current HP of this slime
            - poison: Current poison amount
            - count: 2
        """
        return {
            "enemy_class": "AcidSlime_M",
            "hp": self.state.current_hp,
            "poison": self.state.powers.get("poison", 0),
            "count": 2
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        weak_amount = 2

        if self.ascension >= 17:
            # A17+ pattern
            if roll < 40:
                if self.state.last_two_moves(self.CORROSIVE_SPIT):
                    if self.ai_rng.random_boolean(0.6):
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                   dmg["spit"], effects={"slimed": 2})
            elif roll < 70:
                if self.state.last_two_moves(self.TACKLE):
                    if self.ai_rng.random_boolean(0.6):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 2})
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                if self.state.last_move(self.LICK):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 2})
                    else:
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
        else:
            # Below A17 pattern
            if roll < 30:
                if self.state.last_two_moves(self.CORROSIVE_SPIT):
                    if self.ai_rng.random_boolean(0.5):
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                   dmg["spit"], effects={"slimed": 2})
            elif roll < 70:
                if self.state.last_move(self.TACKLE):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 2})
                    else:
                        move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
                else:
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                if self.state.last_two_moves(self.LICK):
                    if self.ai_rng.random_boolean(0.4):
                        move = MoveInfo(self.CORROSIVE_SPIT, "Corrosive Spit", Intent.ATTACK_DEBUFF,
                                       dmg["spit"], effects={"slimed": 2})
                    else:
                        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})

        self.set_move(move)
        return move


class AcidSlimeS(Enemy):
    """
    Small Acid Slime - Smallest variant, does not split.

    Moves:
    - TACKLE (1): 3/4 damage
    - LICK (2): Apply 1 Weak

    AI Pattern (A17+):
    - Alternates: TACKLE -> LICK -> TACKLE -> ...
    - If last two were TACKLE: force TACKLE (bug? or intentional)

    AI Pattern (below A17):
    - Random 50/50 between TACKLE and LICK
    """

    ID = "AcidSlime_S"
    NAME = "Acid Slime (S)"
    TYPE = EnemyType.NORMAL

    TACKLE = 1
    LICK = 2

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0):
        """
        Args:
            poison_amount: Starting poison stacks (from spawn)
        """
        self.poison_amount = poison_amount
        super().__init__(ai_rng, ascension, hp_rng)
        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (9, 13)
        return (8, 12)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"tackle": 4}
        return {"tackle": 3}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        weak_amount = 1

        if self.ascension >= 17:
            # A17+: Alternating pattern with check
            if self.state.last_two_moves(self.TACKLE):
                # Force attack (this is from decompiled source)
                move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})
        else:
            # Below A17: Random 50/50
            if self.ai_rng.random_boolean(0.5):
                move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
            else:
                move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"weak": weak_amount})

        self.set_move(move)
        return move


class SpikeSlimeL(Enemy):
    """
    Large Spike Slime - Splits into 2 Medium Spike Slimes at 50% HP.

    Moves:
    - FLAME_TACKLE (1): 16/18 damage + 2 Slimed
    - SPLIT (3): Split into 2 SpikeSlime_M at 50% HP
    - LICK (4): Apply 2/3 Frail

    AI Pattern (A17+):
    - Roll 0-29 (30%): FLAME_TACKLE (no 3x repeat, else LICK)
    - Roll 30-99 (70%): LICK (no repeat, else FLAME_TACKLE)

    AI Pattern (below A17):
    - Roll 0-29 (30%): FLAME_TACKLE (no 3x repeat, else LICK)
    - Roll 30-99 (70%): LICK (no 3x repeat, else FLAME_TACKLE)

    Split Mechanics:
    - Has Split power (visible to player)
    - When HP <= 50% max HP, interrupts current action and sets next move to SPLIT
    - On SPLIT: Dies and spawns 2 SpikeSlime_M with HP equal to current HP
    """

    ID = "SpikeSlime_L"
    NAME = "Spike Slime (L)"
    TYPE = EnemyType.NORMAL

    FLAME_TACKLE = 1
    SPLIT = 3
    LICK = 4

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0, starting_hp: Optional[int] = None):
        """
        Args:
            poison_amount: Starting poison stacks
            starting_hp: Override HP (used in boss fight)
        """
        self.poison_amount = poison_amount
        self.starting_hp = starting_hp
        self.split_triggered = False
        super().__init__(ai_rng, ascension, hp_rng)

        # Apply Split power
        self.state.powers["split"] = 1

        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.starting_hp is not None:
            return (self.starting_hp, self.starting_hp)
        if self.ascension >= 7:
            return (67, 73)
        return (64, 70)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"tackle": 18}
        return {"tackle": 16}

    def check_split(self, current_hp: int) -> bool:
        """
        Check if the slime should split.
        Call this after taking damage.

        Returns True if split should be triggered.
        """
        if self.split_triggered:
            return False

        if current_hp <= self.state.max_hp // 2:
            self.split_triggered = True
            # Set next move to SPLIT
            move = MoveInfo(self.SPLIT, "Split", Intent.UNKNOWN)
            self.set_move(move)
            return True

        return False

    def get_split_spawn_info(self) -> Dict:
        """
        Get information needed to spawn the split slimes.

        Returns dict with:
            - enemy_class: "SpikeSlime_M"
            - hp: Current HP of this slime
            - poison: Current poison amount
            - count: 2
        """
        return {
            "enemy_class": "SpikeSlime_M",
            "hp": self.state.current_hp,
            "poison": self.state.powers.get("poison", 0),
            "count": 2
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        frail_amount = 3 if self.ascension >= 17 else 2

        if self.ascension >= 17:
            # A17+ pattern
            if roll < 30:
                if self.state.last_two_moves(self.FLAME_TACKLE):
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail_amount})
                else:
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 2})
            else:
                if self.state.last_move(self.LICK):
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 2})
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail_amount})
        else:
            # Below A17 pattern
            if roll < 30:
                if self.state.last_two_moves(self.FLAME_TACKLE):
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail_amount})
                else:
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 2})
            else:
                if self.state.last_two_moves(self.LICK):
                    move = MoveInfo(self.FLAME_TACKLE, "Flame Tackle", Intent.ATTACK_DEBUFF,
                                   dmg["tackle"], effects={"slimed": 2})
                else:
                    move = MoveInfo(self.LICK, "Lick", Intent.DEBUFF, effects={"frail": frail_amount})

        self.set_move(move)
        return move


class SpikeSlimeS(Enemy):
    """
    Small Spike Slime - Smallest variant, does not split.

    Moves:
    - TACKLE (1): 5/6 damage

    AI Pattern:
    - Always TACKLE (only has one move)
    """

    ID = "SpikeSlime_S"
    NAME = "Spike Slime (S)"
    TYPE = EnemyType.NORMAL

    TACKLE = 1

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 poison_amount: int = 0):
        """
        Args:
            poison_amount: Starting poison stacks (from spawn)
        """
        self.poison_amount = poison_amount
        super().__init__(ai_rng, ascension, hp_rng)
        if poison_amount > 0:
            self.state.powers["poison"] = poison_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (11, 15)
        return (10, 14)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"tackle": 6}
        return {"tackle": 5}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        # Only has one move
        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
        self.set_move(move)
        return move


class Louse(Enemy):
    """
    Red/Green Louse - Common enemy.

    Red Louse Moves:
    - BITE (1): 5-7 damage (rolled at combat start)
    - GROW (2): +3/4 Strength

    Green Louse has SPIT instead of GROW.
    """

    ID = "Louse"  # Will be overridden
    NAME = "Louse"
    TYPE = EnemyType.NORMAL

    BITE = 1
    GROW = 2  # or SPIT for green

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 is_red: bool = True):
        self.is_red = is_red
        self.bite_damage = 0  # Rolled in init
        super().__init__(ai_rng, ascension, hp_rng)

        # Roll bite damage
        if ascension >= 2:
            self.bite_damage = self.hp_rng.random_range(6, 8)
        else:
            self.bite_damage = self.hp_rng.random_range(5, 7)

        # Set curl up amount
        if ascension >= 17:
            self.curl_up = self.hp_rng.random_range(9, 12)
        elif ascension >= 7:
            self.curl_up = self.hp_rng.random_range(4, 8)
        else:
            self.curl_up = self.hp_rng.random_range(3, 7)

        self.state.powers["curl_up"] = self.curl_up

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (11, 16)  # Fixed: was (11, 17), Java has A_2_HP_MAX = 16
        return (10, 15)

    def get_move(self, roll: int) -> MoveInfo:
        str_gain = 4 if self.ascension >= 17 else 3

        if self.ascension >= 17:
            # A17+ pattern: GROW uses lastMove (single repeat check)
            if roll < 25:
                if self.state.last_move(self.GROW):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, self.bite_damage)
                else:
                    if self.is_red:
                        move = MoveInfo(self.GROW, "Grow", Intent.BUFF, effects={"strength": str_gain})
                    else:
                        move = MoveInfo(self.GROW, "Spit Web", Intent.DEBUFF, effects={"weak": 2})
            else:
                if self.state.last_two_moves(self.BITE):
                    if self.is_red:
                        move = MoveInfo(self.GROW, "Grow", Intent.BUFF, effects={"strength": str_gain})
                    else:
                        move = MoveInfo(self.GROW, "Spit Web", Intent.DEBUFF, effects={"weak": 2})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, self.bite_damage)
        else:
            # Below A17: GROW uses lastTwoMoves (can repeat once)
            if roll < 25:
                if self.state.last_two_moves(self.GROW):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, self.bite_damage)
                else:
                    if self.is_red:
                        move = MoveInfo(self.GROW, "Grow", Intent.BUFF, effects={"strength": str_gain})
                    else:
                        move = MoveInfo(self.GROW, "Spit Web", Intent.DEBUFF, effects={"weak": 2})
            else:
                if self.state.last_two_moves(self.BITE):
                    if self.is_red:
                        move = MoveInfo(self.GROW, "Grow", Intent.BUFF, effects={"strength": str_gain})
                    else:
                        move = MoveInfo(self.GROW, "Spit Web", Intent.DEBUFF, effects={"weak": 2})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, self.bite_damage)

        self.set_move(move)
        return move


class FungiBeast(Enemy):
    """
    Fungi Beast - Exordium basic enemy.

    Moves:
    - BITE (1): 6 damage
    - GROW (2): Gain 3/4/5 Strength

    Special:
    - Has Spore Cloud power: On death, apply 2 Vulnerable to player

    AI Pattern:
    - Roll 0-59 (60%): BITE (no 3x repeat, else GROW)
    - Roll 60-99 (40%): GROW (no repeat, else BITE)
    """

    ID = "FungiBeast"
    NAME = "Fungi Beast"
    TYPE = EnemyType.NORMAL

    BITE = 1
    GROW = 2

    MOVES = {1: "Bite", 2: "Grow"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        # Spore Cloud: applies 2 Vulnerable to player on death
        self.state.powers["spore_cloud"] = 2

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (24, 28)
        return (22, 28)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 17:
            str_gain = 5
        elif self.ascension >= 2:
            str_gain = 4
        else:
            str_gain = 3
        return {"bite": 6, "strength": str_gain}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if roll < 60:
            if self.state.last_two_moves(self.BITE):
                move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                               effects={"strength": dmg["strength"]})
            else:
                move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
        else:
            if self.state.last_move(self.GROW):
                move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
            else:
                move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                               effects={"strength": dmg["strength"]})

        self.set_move(move)
        return move

    def on_death(self) -> Dict:
        """Called when this enemy dies. Returns effects to apply."""
        return {"player_effects": {"vulnerable": self.state.powers.get("spore_cloud", 2)}}


class LouseNormal(Enemy):
    """
    Red Louse (Normal variant) - Exordium basic enemy.

    Moves:
    - BITE (3): 5-7/6-8 damage (rolled at combat start)
    - GROW (4): Gain 3/4 Strength

    Special:
    - Has Curl Up power: gains block when attacked (3-7/4-8/9-12)
    """

    ID = "FuzzyLouseNormal"
    NAME = "Red Louse"
    TYPE = EnemyType.NORMAL

    BITE = 3
    GROW = 4

    MOVES = {3: "Bite", 4: "Grow"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.bite_damage = 0
        super().__init__(ai_rng, ascension, hp_rng)

        # Roll bite damage at combat start
        if ascension >= 2:
            self.bite_damage = self.hp_rng.random_range(6, 8)
        else:
            self.bite_damage = self.hp_rng.random_range(5, 7)

        # Curl Up power
        if ascension >= 17:
            curl_up = self.hp_rng.random_range(9, 12)
        elif ascension >= 7:
            curl_up = self.hp_rng.random_range(4, 8)
        else:
            curl_up = self.hp_rng.random_range(3, 7)
        self.state.powers["curl_up"] = curl_up

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (11, 16)
        return (10, 15)

    def _get_damage_values(self) -> Dict[str, int]:
        str_gain = 4 if self.ascension >= 17 else 3
        return {"bite": self.bite_damage, "strength": str_gain}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 17:
            if roll < 25:
                if self.state.last_move(self.GROW):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
                else:
                    move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                                   effects={"strength": dmg["strength"]})
            else:
                if self.state.last_two_moves(self.BITE):
                    move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                                   effects={"strength": dmg["strength"]})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
        else:
            if roll < 25:
                if self.state.last_two_moves(self.GROW):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
                else:
                    move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                                   effects={"strength": dmg["strength"]})
            else:
                if self.state.last_two_moves(self.BITE):
                    move = MoveInfo(self.GROW, "Grow", Intent.BUFF,
                                   effects={"strength": dmg["strength"]})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])

        self.set_move(move)
        return move


class LouseDefensive(Enemy):
    """
    Green Louse (Defensive variant) - Exordium basic enemy.

    Moves:
    - BITE (3): 5-7/6-8 damage (rolled at combat start)
    - SPIT_WEB (4): Apply 2 Weak to player

    Special:
    - Has Curl Up power: gains block when attacked (3-7/4-8/9-12)
    """

    ID = "FuzzyLouseDefensive"
    NAME = "Green Louse"
    TYPE = EnemyType.NORMAL

    BITE = 3
    SPIT_WEB = 4

    MOVES = {3: "Bite", 4: "Spit Web"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.bite_damage = 0
        super().__init__(ai_rng, ascension, hp_rng)

        # Roll bite damage at combat start
        if ascension >= 2:
            self.bite_damage = self.hp_rng.random_range(6, 8)
        else:
            self.bite_damage = self.hp_rng.random_range(5, 7)

        # Curl Up power
        if ascension >= 17:
            curl_up = self.hp_rng.random_range(9, 12)
        elif ascension >= 7:
            curl_up = self.hp_rng.random_range(4, 8)
        else:
            curl_up = self.hp_rng.random_range(3, 7)
        self.state.powers["curl_up"] = curl_up

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (12, 18)
        return (11, 17)

    def _get_damage_values(self) -> Dict[str, int]:
        return {"bite": self.bite_damage, "weak": 2}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 17:
            if roll < 25:
                if self.state.last_move(self.SPIT_WEB):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
                else:
                    move = MoveInfo(self.SPIT_WEB, "Spit Web", Intent.DEBUFF,
                                   effects={"weak": dmg["weak"]})
            else:
                if self.state.last_two_moves(self.BITE):
                    move = MoveInfo(self.SPIT_WEB, "Spit Web", Intent.DEBUFF,
                                   effects={"weak": dmg["weak"]})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
        else:
            if roll < 25:
                if self.state.last_two_moves(self.SPIT_WEB):
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])
                else:
                    move = MoveInfo(self.SPIT_WEB, "Spit Web", Intent.DEBUFF,
                                   effects={"weak": dmg["weak"]})
            else:
                if self.state.last_two_moves(self.BITE):
                    move = MoveInfo(self.SPIT_WEB, "Spit Web", Intent.DEBUFF,
                                   effects={"weak": dmg["weak"]})
                else:
                    move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])

        self.set_move(move)
        return move


# ============ EXORDIUM ELITES ============

class GremlinNob(Enemy):
    """
    Gremlin Nob - Elite enemy.

    Moves:
    - BELLOW (1): Gain Enrage (2/3 strength when player plays skill)
    - RUSH (2): 14/16 damage
    - SKULL_BASH (3): 6/8 damage + 2 Vulnerable

    AI Pattern:
    - Turn 1: Always BELLOW
    - Roll 0-32 (33%): SKULL_BASH (no repeat)
    - Roll 33-99 (67%): RUSH (no 3x repeat)
    """

    ID = "GremlinNob"
    NAME = "Gremlin Nob"
    TYPE = EnemyType.ELITE

    RUSH = 1       # Java: BULL_RUSH = 1
    SKULL_BASH = 2  # Java: SKULL_BASH = 2
    BELLOW = 3      # Java: BELLOW = 3

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (85, 90)
        return (82, 86)

    def _get_damage_values(self) -> Dict[str, int]:
        # Damage at A3+, but enrage only increases at A18+
        if self.ascension >= 3:
            enrage = 3 if self.ascension >= 18 else 2
            return {"rush": 16, "skull_bash": 8, "enrage": enrage}
        return {"rush": 14, "skull_bash": 6, "enrage": 2}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.BELLOW, "Bellow", Intent.BUFF,
                           effects={"enrage": dmg["enrage"]})
        # A18+ special logic: prioritize Skull Bash if not used in last 2 turns
        elif self.ascension >= 18:
            if not self.state.last_move(self.SKULL_BASH) and not self.state.last_move_before(self.SKULL_BASH):
                move = MoveInfo(self.SKULL_BASH, "Skull Bash", Intent.ATTACK_DEBUFF,
                               dmg["skull_bash"], effects={"vulnerable": 2})
            elif self.state.last_two_moves(self.RUSH):
                move = MoveInfo(self.SKULL_BASH, "Skull Bash", Intent.ATTACK_DEBUFF,
                               dmg["skull_bash"], effects={"vulnerable": 2})
            else:
                move = MoveInfo(self.RUSH, "Rush", Intent.ATTACK, dmg["rush"])
        elif roll < 33:
            # Java: unconditionally sets SKULL_BASH when num < 33 (no lastMove check)
            move = MoveInfo(self.SKULL_BASH, "Skull Bash", Intent.ATTACK_DEBUFF,
                           dmg["skull_bash"], effects={"vulnerable": 2})
        else:
            if self.state.last_two_moves(self.RUSH):
                move = MoveInfo(self.SKULL_BASH, "Skull Bash", Intent.ATTACK_DEBUFF,
                               dmg["skull_bash"], effects={"vulnerable": 2})
            else:
                move = MoveInfo(self.RUSH, "Rush", Intent.ATTACK, dmg["rush"])

        self.set_move(move)
        return move


class Lagavulin(Enemy):
    """
    Lagavulin - Elite enemy (sleeps initially).

    Moves:
    - ATTACK (1): 18/20 damage
    - SIPHON_SOUL (2): -1/-2 Strength and Dexterity to player
    - SLEEP (3): Sleeping (wakes on 3rd turn or if attacked)
    - STUN (4): Stunned (shown when woken by damage)

    AI Pattern:
    - Sleeps for 3 turns (unless attacked)
    - If attacked while asleep: Shows STUN intent, then starts normal pattern
    - After waking: ATTACK, ATTACK, SIPHON, repeat
    - Forces SIPHON if debuff_turn_count reaches 2 (prevents infinite attacks)
    """

    ID = "Lagavulin"
    NAME = "Lagavulin"
    TYPE = EnemyType.ELITE

    SIPHON_SOUL = 1  # Java: DEBUFF = 1
    ATTACK = 3        # Java: STRONG_ATK = 3
    SLEEP = 4         # Java: OPEN = 4
    STUN = 5          # Java: IDLE = 5

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        self.asleep = True
        self.sleep_turns = 0
        self.debuff_turn_count = 0  # Tracks turns without debuff (forces debuff at 2)
        self.is_out_triggered = False  # Tracks if wake-up has been triggered
        self.state.powers["metallicize"] = 8  # Gains 8 block per turn while asleep

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (112, 115)  # Fixed: was (112, 116), Java has A_2_HP_MAX = 115
        return (109, 111)

    def _get_damage_values(self) -> Dict[str, int]:
        # Damage at A3+, but debuff only increases at A18+
        if self.ascension >= 3:
            debuff = 2 if self.ascension >= 18 else 1
            return {"attack": 20, "debuff": debuff}
        return {"attack": 18, "debuff": 1}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.asleep:
            self.sleep_turns += 1
            if self.sleep_turns >= 3:
                self.asleep = False
                self.is_out_triggered = True
                # Wake up and attack
                move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK, dmg["attack"])
            else:
                move = MoveInfo(self.SLEEP, "Sleep", Intent.SLEEP)
        else:
            # Pattern: Attack, Attack, Siphon, repeat
            # But force Siphon if debuff_turn_count >= 2
            if self.debuff_turn_count >= 2:
                # Force debuff after 2 turns without it
                move = MoveInfo(self.SIPHON_SOUL, "Siphon Soul", Intent.STRONG_DEBUFF,
                               effects={"strength": -dmg["debuff"], "dexterity": -dmg["debuff"]})
            elif self.state.last_two_moves(self.ATTACK):
                # Normal pattern: debuff after 2 attacks
                move = MoveInfo(self.SIPHON_SOUL, "Siphon Soul", Intent.STRONG_DEBUFF,
                               effects={"strength": -dmg["debuff"], "dexterity": -dmg["debuff"]})
            else:
                # Attack
                move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK, dmg["attack"])

        self.set_move(move)
        return move

    def take_turn(self):
        """Execute the current move and update debuff_turn_count."""
        if self.state.move_id == self.SIPHON_SOUL:
            # Reset counter when debuff is used
            self.debuff_turn_count = 0
        elif self.state.move_id == self.ATTACK:
            # Increment counter when attack is used
            self.debuff_turn_count += 1

    def wake_up(self):
        """Called when attacked while sleeping."""
        if not self.is_out_triggered:
            # First time waking up - show STUN intent for this turn
            self.is_out_triggered = True
            self.asleep = False
            self.state.powers.pop("metallicize", None)
            # Set STUN move for this turn
            move = MoveInfo(self.STUN, "Stunned", Intent.STUN)
            self.set_move(move)


class Sentries(Enemy):
    """
    Sentries - Elite fight with 3 sentries.

    Each sentry alternates between:
    - BOLT (1): 9/10 damage
    - BEAM (2): 9/10 damage
    - Also apply Daze cards

    Pattern depends on position (left/middle/right start differently).
    """

    ID = "Sentry"
    NAME = "Sentry"
    TYPE = EnemyType.ELITE

    BOLT = 3  # Java: BOLT = 3
    BEAM = 4  # Java: BEAM = 4

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 position: int = 0):
        """
        Args:
            position: 0=left, 1=middle, 2=right (affects starting move)
        """
        self.position = position
        super().__init__(ai_rng, ascension, hp_rng)

        # Sentries start with different moves based on position
        # Left+Right start with BOLT, Middle starts with BEAM
        self._starting_move = self.BEAM if position == 1 else self.BOLT

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (39, 45)
        return (38, 42)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 3:
            return {"damage": 10}
        return {"damage": 9}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        daze_count = 3 if self.ascension >= 18 else 2

        if self.state.first_turn:
            self.state.first_turn = False
            if self._starting_move == self.BOLT:
                move = MoveInfo(self.BOLT, "Bolt", Intent.ATTACK, dmg["damage"])
            else:
                move = MoveInfo(self.BEAM, "Beam", Intent.ATTACK_DEBUFF,
                               dmg["damage"], effects={"daze": daze_count})
        else:
            # Alternate between BOLT and BEAM
            if self.state.last_move(self.BOLT):
                move = MoveInfo(self.BEAM, "Beam", Intent.ATTACK_DEBUFF,
                               dmg["damage"], effects={"daze": daze_count})
            else:
                move = MoveInfo(self.BOLT, "Bolt", Intent.ATTACK, dmg["damage"])

        self.set_move(move)
        return move


# ============ EXORDIUM BOSSES ============

class SlimeBoss(Enemy):
    """
    Slime Boss - Act 1 Boss.

    Moves:
    - SLAM (1): 35/38 damage
    - PREP_SLAM (2): Unknown intent (preparing)
    - SPLIT (3): Splits into 2 large slimes at 50% HP
    - STICKY (4): 3/5 Slimed cards to discard

    AI Pattern:
    - Turn 1: STICKY
    - Then: PREP_SLAM -> SLAM -> STICKY -> repeat
    - Splits when HP <= 50% (interrupts pattern)
    """

    ID = "SlimeBoss"
    NAME = "Slime Boss"
    TYPE = EnemyType.BOSS

    SLAM = 1
    PREP_SLAM = 2
    SPLIT = 3
    STICKY = 4

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (150, 150)
        return (140, 140)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {"slam": 38, "tackle": 10}
        return {"slam": 35, "tackle": 9}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        slimed_count = 5 if self.ascension >= 19 else 3

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.STICKY, "Goop Spray", Intent.STRONG_DEBUFF,
                           effects={"slimed": slimed_count})
        elif self.state.last_move(self.STICKY):
            move = MoveInfo(self.PREP_SLAM, "Preparing", Intent.UNKNOWN)
        elif self.state.last_move(self.PREP_SLAM):
            move = MoveInfo(self.SLAM, "Slam", Intent.ATTACK, dmg["slam"])
        else:
            move = MoveInfo(self.STICKY, "Goop Spray", Intent.STRONG_DEBUFF,
                           effects={"slimed": slimed_count})

        self.set_move(move)
        return move

    def should_split(self) -> bool:
        """Check if should split (HP <= 50%)."""
        return self.state.current_hp <= self.state.max_hp // 2


class TheGuardian(Enemy):
    """
    The Guardian - Act 1 Boss.

    Modes:
    - OFFENSIVE: Attacks
    - DEFENSIVE: Gains Sharp Hide, has Mode Shift damage threshold

    Moves:
    - CHARGING_UP (1): Gain 9 block
    - FIERCE_BASH (2): 32/36 damage
    - VENT_STEAM (3): 2 Weak + 2 Vulnerable
    - WHIRLWIND (4): 5x4 damage
    - ROLL_ATTACK (5): 9/10 damage
    - TWIN_SLAM (6): 8x2 damage

    AI Pattern varies by mode.
    """

    ID = "TheGuardian"
    NAME = "The Guardian"
    TYPE = EnemyType.BOSS

    CHARGING_UP = 6   # Java: CHARGE_UP = 6
    FIERCE_BASH = 2    # Java: FIERCE_BASH = 2
    ROLL_ATTACK = 3    # Java: ROLL_ATTACK = 3
    TWIN_SLAM = 4      # Java: TWIN_SLAM = 4
    WHIRLWIND = 5      # Java: WHIRLWIND = 5
    VENT_STEAM = 7     # Java: VENT_STEAM = 7

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        self.offensive_mode = True
        # Mode shift thresholds from Java: A19=40, A9+=35, below=30
        if ascension >= 19:
            self.mode_shift_damage = 40
        elif ascension >= 9:
            self.mode_shift_damage = 35
        else:
            self.mode_shift_damage = 30
        self.damage_taken_this_mode = 0

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (250, 250)
        return (240, 240)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {"fierce_bash": 36, "roll": 10, "whirlwind": 5, "twin_slam": 8}
        return {"fierce_bash": 32, "roll": 9, "whirlwind": 5, "twin_slam": 8}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.offensive_mode:
            # Offensive mode pattern
            if self.state.first_turn:
                self.state.first_turn = False
                move = MoveInfo(self.CHARGING_UP, "Charging Up", Intent.DEFEND, block=9)
            elif self.state.last_move(self.CHARGING_UP):
                move = MoveInfo(self.FIERCE_BASH, "Fierce Bash", Intent.ATTACK, dmg["fierce_bash"])
            elif self.state.last_move(self.FIERCE_BASH):
                move = MoveInfo(self.VENT_STEAM, "Vent Steam", Intent.STRONG_DEBUFF,
                               effects={"weak": 2, "vulnerable": 2})
            elif self.state.last_move(self.VENT_STEAM):
                move = MoveInfo(self.WHIRLWIND, "Whirlwind", Intent.ATTACK,
                               dmg["whirlwind"], hits=4, is_multi=True)
            else:
                move = MoveInfo(self.CHARGING_UP, "Charging Up", Intent.DEFEND, block=9)
        else:
            # Defensive mode pattern
            if self.state.last_move(self.ROLL_ATTACK):
                move = MoveInfo(self.TWIN_SLAM, "Twin Slam", Intent.ATTACK,
                               dmg["twin_slam"], hits=2, is_multi=True)
            else:
                move = MoveInfo(self.ROLL_ATTACK, "Roll Attack", Intent.ATTACK, dmg["roll"])

        self.set_move(move)
        return move

    def take_damage(self, amount: int):
        """Track damage for mode shift."""
        self.damage_taken_this_mode += amount
        if self.offensive_mode and self.damage_taken_this_mode >= self.mode_shift_damage:
            self.switch_to_defensive()

    def switch_to_defensive(self):
        """Switch to defensive mode."""
        self.offensive_mode = False
        self.damage_taken_this_mode = 0
        # Sharp hide from Java: A19+=4 (thornsDamage+1), below=3 (thornsDamage)
        self.state.powers["sharp_hide"] = 4 if self.ascension >= 19 else 3
        # Increment threshold by 10 (Java: dmgThreshold += dmgThresholdIncrease)
        self.mode_shift_damage += 10

    def switch_to_offensive(self):
        """Switch back to offensive mode."""
        self.offensive_mode = True
        self.damage_taken_this_mode = 0
        self.state.powers.pop("sharp_hide", None)


class Hexaghost(Enemy):
    """
    Hexaghost - Act 1 Boss.

    Moves:
    - ACTIVATE (1): Set up flames
    - DIVIDER (2): 6+ damage x6 (scales with player max HP)
    - SEAR (3): 6 damage + 1 Burn
    - TACKLE (4): 5x2 damage
    - INFLAME (5): Gain 2/3 Strength + 12/15 block
    - INFERNO (6): 2x6 damage + 3 Burns

    AI Pattern:
    - Turn 1: ACTIVATE
    - Turn 2: DIVIDER (based on player's max HP)
    - Then cycle through SEAR variations and INFERNO
    """

    ID = "Hexaghost"
    NAME = "Hexaghost"
    TYPE = EnemyType.BOSS

    DIVIDER = 1    # Java: DIVIDER = 1
    TACKLE = 2     # Java: TACKLE = 2
    INFLAME = 3    # Java: INFLAME = 3
    SEAR = 4       # Java: SEAR = 4
    ACTIVATE = 5   # Java: ACTIVATE = 5
    INFERNO = 6    # Java: INFERNO = 6

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 player_max_hp: int = 80):
        super().__init__(ai_rng, ascension, hp_rng)
        self.player_max_hp = player_max_hp
        self.turn_count = 0

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (264, 264)
        return (250, 250)

    def _get_damage_values(self) -> Dict[str, int]:
        # Divider uses player's CURRENT HP, not max HP (Java: currentHealth / 12 + 1)
        player_hp = self.state.player_hp if self.state.player_hp > 0 else self.player_max_hp
        if self.ascension >= 19:
            return {
                "divider_base": (player_hp // 12) + 1,
                "sear": 6,
                "tackle": 6,
                "inferno": 3,
                "inflame_str": 3,
                "inflame_block": 12,
                "burn_count": 2,
            }
        elif self.ascension >= 4:
            return {
                "divider_base": (player_hp // 12) + 1,
                "sear": 6,
                "tackle": 6,
                "inferno": 3,
                "inflame_str": 2,
                "inflame_block": 12,
                "burn_count": 1,
            }
        return {
            "divider_base": (player_hp // 12) + 1,
            "sear": 6,
            "tackle": 5,
            "inferno": 2,
            "inflame_str": 2,
            "inflame_block": 12,
            "burn_count": 1,
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        self.turn_count += 1

        burn_count = 2 if self.ascension >= 19 else 1

        if self.turn_count == 1:
            move = MoveInfo(self.ACTIVATE, "Activate", Intent.UNKNOWN)
        elif self.turn_count == 2:
            move = MoveInfo(self.DIVIDER, "Divider", Intent.ATTACK,
                           dmg["divider_base"], hits=6, is_multi=True)
        else:
            # Cycle through pattern
            pattern_turn = (self.turn_count - 3) % 7
            if pattern_turn == 0:
                move = MoveInfo(self.SEAR, "Sear", Intent.ATTACK_DEBUFF,
                               dmg["sear"], effects={"burn": burn_count})
            elif pattern_turn == 1:
                move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK,
                               dmg["tackle"], hits=2, is_multi=True)
            elif pattern_turn == 2:
                move = MoveInfo(self.SEAR, "Sear", Intent.ATTACK_DEBUFF,
                               dmg["sear"], effects={"burn": burn_count})
            elif pattern_turn == 3:
                move = MoveInfo(self.INFLAME, "Inflame", Intent.DEFEND_BUFF,
                               block=dmg["inflame_block"],
                               effects={"strength": dmg["inflame_str"]})
            elif pattern_turn == 4:
                move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK,
                               dmg["tackle"], hits=2, is_multi=True)
            elif pattern_turn == 5:
                move = MoveInfo(self.SEAR, "Sear", Intent.ATTACK_DEBUFF,
                               dmg["sear"], effects={"burn": burn_count})
            else:  # pattern_turn == 6
                move = MoveInfo(self.INFERNO, "Inferno", Intent.ATTACK_DEBUFF,
                               dmg["inferno"], hits=6, is_multi=True,
                               effects={"burn": 3})

        self.set_move(move)
        return move


# ============ ACT 2 (CITY) BASIC ENEMIES ============

class Chosen(Enemy):
    """
    Chosen - Act 2 basic enemy.

    Moves:
    - POKE (5): 5/6 damage x2 multi-attack
    - ZAP (1): 18/21 damage
    - DRAIN (2): Apply 3 Weak to player, gain 3 Strength
    - DEBILITATE (3): 10/12 damage + 2 Vulnerable
    - HEX (4): Apply 1 Hex (on draw non-attack, add Daze to discard)

    AI Pattern:
    - A17+: First turn always HEX, then 50% DEBILITATE/DRAIN, if used -> 40% ZAP else POKE
    - Below A17: First turn POKE, second turn HEX, then same as above
    """

    ID = "Chosen"
    NAME = "Chosen"
    TYPE = EnemyType.NORMAL

    POKE = 5
    ZAP = 1
    DRAIN = 2
    DEBILITATE = 3
    HEX = 4

    MOVES = {1: "Zap", 2: "Drain", 3: "Debilitate", 4: "Hex", 5: "Poke"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.used_hex = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (98, 103)
        return (95, 99)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"zap": 21, "debilitate": 12, "poke": 6}
        return {"zap": 18, "debilitate": 10, "poke": 5}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 17:
            # A17+: Always start with Hex
            if not self.used_hex:
                self.used_hex = True
                move = MoveInfo(self.HEX, "Hex", Intent.STRONG_DEBUFF, effects={"hex": 1})
                self.set_move(move)
                return move

            # If didn't just use Debilitate or Drain
            if not self.state.last_move(self.DEBILITATE) and not self.state.last_move(self.DRAIN):
                if roll < 50:
                    move = MoveInfo(self.DEBILITATE, "Debilitate", Intent.ATTACK_DEBUFF,
                                   dmg["debilitate"], effects={"vulnerable": 2})
                else:
                    move = MoveInfo(self.DRAIN, "Drain", Intent.DEBUFF,
                                   effects={"weak_player": 3, "strength_self": 3})
            else:
                if roll < 40:
                    move = MoveInfo(self.ZAP, "Zap", Intent.ATTACK, dmg["zap"])
                else:
                    move = MoveInfo(self.POKE, "Poke", Intent.ATTACK, dmg["poke"], hits=2, is_multi=True)
        else:
            # Below A17: First turn Poke
            if self.state.first_turn:
                self.state.first_turn = False
                move = MoveInfo(self.POKE, "Poke", Intent.ATTACK, dmg["poke"], hits=2, is_multi=True)
                self.set_move(move)
                return move

            # Second turn (or after): Hex if not used
            if not self.used_hex:
                self.used_hex = True
                move = MoveInfo(self.HEX, "Hex", Intent.STRONG_DEBUFF, effects={"hex": 1})
                self.set_move(move)
                return move

            # After Hex: same pattern as A17+
            if not self.state.last_move(self.DEBILITATE) and not self.state.last_move(self.DRAIN):
                if roll < 50:
                    move = MoveInfo(self.DEBILITATE, "Debilitate", Intent.ATTACK_DEBUFF,
                                   dmg["debilitate"], effects={"vulnerable": 2})
                else:
                    move = MoveInfo(self.DRAIN, "Drain", Intent.DEBUFF,
                                   effects={"weak_player": 3, "strength_self": 3})
            else:
                if roll < 40:
                    move = MoveInfo(self.ZAP, "Zap", Intent.ATTACK, dmg["zap"])
                else:
                    move = MoveInfo(self.POKE, "Poke", Intent.ATTACK, dmg["poke"], hits=2, is_multi=True)

        self.set_move(move)
        return move


class Byrd(Enemy):
    """
    Byrd - Act 2 flying enemy.

    Moves (Flying):
    - PECK (1): 1 damage x5/x6 multi-attack
    - CAW (6): Gain 1 Strength
    - SWOOP (3): 12/14 damage

    Moves (Grounded - after Flight broken):
    - HEADBUTT (5): 3 damage, then GO_AIRBORNE
    - GO_AIRBORNE (2): Regain Flight
    - STUNNED (4): Stun (when grounded by losing Flight)

    Special: Starts with Flight 3/4 (A17+)

    AI Pattern (Flying):
    - First turn: 37.5% CAW, else PECK
    - Roll 0-49: PECK (no 3x repeat, else 40% SWOOP, 60% CAW)
    - Roll 50-69: SWOOP (no repeat, else 37.5% CAW, 62.5% PECK)
    - Roll 70-99: CAW (no repeat, else 28.57% SWOOP, 71.43% PECK)
    """

    ID = "Byrd"
    NAME = "Byrd"
    TYPE = EnemyType.NORMAL

    PECK = 1
    GO_AIRBORNE = 2
    SWOOP = 3
    STUNNED = 4
    HEADBUTT = 5
    CAW = 6

    MOVES = {1: "Peck", 2: "Fly", 3: "Swoop", 4: "Stunned", 5: "Headbutt", 6: "Caw"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.is_flying = True
        super().__init__(ai_rng, ascension, hp_rng)
        self.flight_amount = 4 if ascension >= 17 else 3
        self.state.powers["flight"] = self.flight_amount

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (26, 33)
        return (25, 31)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"peck": 1, "peck_count": 6, "swoop": 14, "headbutt": 3}
        return {"peck": 1, "peck_count": 5, "swoop": 12, "headbutt": 3}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if not self.is_flying:
            # Grounded: Headbutt then fly
            move = MoveInfo(self.HEADBUTT, "Headbutt", Intent.ATTACK, dmg["headbutt"])
            self.set_move(move)
            return move

        # Flying behavior
        if self.state.first_turn:
            self.state.first_turn = False
            if self.ai_rng.random_boolean(0.375):
                move = MoveInfo(self.CAW, "Caw", Intent.BUFF, effects={"strength": 1})
            else:
                move = MoveInfo(self.PECK, "Peck", Intent.ATTACK, dmg["peck"],
                               hits=dmg["peck_count"], is_multi=True)
            self.set_move(move)
            return move

        if roll < 50:
            if self.state.last_two_moves(self.PECK):
                if self.ai_rng.random_boolean(0.4):
                    move = MoveInfo(self.SWOOP, "Swoop", Intent.ATTACK, dmg["swoop"])
                else:
                    move = MoveInfo(self.CAW, "Caw", Intent.BUFF, effects={"strength": 1})
            else:
                move = MoveInfo(self.PECK, "Peck", Intent.ATTACK, dmg["peck"],
                               hits=dmg["peck_count"], is_multi=True)
        elif roll < 70:
            if self.state.last_move(self.SWOOP):
                if self.ai_rng.random_boolean(0.375):
                    move = MoveInfo(self.CAW, "Caw", Intent.BUFF, effects={"strength": 1})
                else:
                    move = MoveInfo(self.PECK, "Peck", Intent.ATTACK, dmg["peck"],
                                   hits=dmg["peck_count"], is_multi=True)
            else:
                move = MoveInfo(self.SWOOP, "Swoop", Intent.ATTACK, dmg["swoop"])
        else:
            if self.state.last_move(self.CAW):
                if self.ai_rng.random_boolean(0.2857):
                    move = MoveInfo(self.SWOOP, "Swoop", Intent.ATTACK, dmg["swoop"])
                else:
                    move = MoveInfo(self.PECK, "Peck", Intent.ATTACK, dmg["peck"],
                                   hits=dmg["peck_count"], is_multi=True)
            else:
                move = MoveInfo(self.CAW, "Caw", Intent.BUFF, effects={"strength": 1})

        self.set_move(move)
        return move

    def ground(self):
        """Called when Flight is broken."""
        self.is_flying = False
        self.state.powers.pop("flight", None)

    def take_off(self):
        """Called when Byrd flies again."""
        self.is_flying = True
        self.state.powers["flight"] = self.flight_amount


class Centurion(Enemy):
    """
    Centurion - Act 2 tank enemy (pairs with Mystic/Healer).

    Moves:
    - SLASH (1): 12/14 damage
    - PROTECT (2): Give 15/20 block to random ally (or self if alone)
    - FURY (3): 6/7 damage x3 multi-attack (used when alone)

    AI Pattern:
    - Roll 65-99 (35%) and didn't do PROTECT or FURY last 2 turns and allies alive: PROTECT
    - If alone: FURY instead of PROTECT
    - Otherwise: SLASH (no 3x repeat)
    """

    ID = "Centurion"
    NAME = "Centurion"
    TYPE = EnemyType.NORMAL

    SLASH = 1
    PROTECT = 2
    FURY = 3

    MOVES = {1: "Slash", 2: "Defend", 3: "Fury"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (78, 83)
        return (76, 80)

    def _get_damage_values(self) -> Dict[str, int]:
        block = 20 if self.ascension >= 17 else 15
        if self.ascension >= 2:
            return {"slash": 14, "fury": 7, "fury_hits": 3, "block": block}
        return {"slash": 12, "fury": 6, "fury_hits": 3, "block": block}

    def get_move(self, roll: int, allies_alive: int = 1) -> MoveInfo:
        dmg = self._get_damage_values()

        if roll >= 65 and not self.state.last_two_moves(self.PROTECT) and not self.state.last_two_moves(self.FURY):
            if allies_alive > 1:
                move = MoveInfo(self.PROTECT, "Defend", Intent.DEFEND, block=dmg["block"])
            else:
                move = MoveInfo(self.FURY, "Fury", Intent.ATTACK, dmg["fury"],
                               hits=dmg["fury_hits"], is_multi=True)
        elif not self.state.last_two_moves(self.SLASH):
            move = MoveInfo(self.SLASH, "Slash", Intent.ATTACK, dmg["slash"])
        else:
            if allies_alive > 1:
                move = MoveInfo(self.PROTECT, "Defend", Intent.DEFEND, block=dmg["block"])
            else:
                move = MoveInfo(self.FURY, "Fury", Intent.ATTACK, dmg["fury"],
                               hits=dmg["fury_hits"], is_multi=True)

        self.set_move(move)
        return move


class Healer(Enemy):
    """
    Healer/Mystic - Act 2 support enemy (pairs with Centurion).

    Moves:
    - ATTACK (1): 8/9 damage + 2 Frail
    - HEAL (2): Heal all allies 16/20 HP
    - BUFF (3): Give all allies 2/3/4 Strength

    AI Pattern:
    - If total missing HP > 15/20: HEAL (no 3x repeat)
    - Roll 40-99 (60%): ATTACK (no repeat at A17+, no 3x repeat below)
    - Else: BUFF (no 3x repeat)
    """

    ID = "Healer"
    NAME = "Mystic"
    TYPE = EnemyType.NORMAL

    ATTACK = 1
    HEAL = 2
    BUFF = 3

    MOVES = {1: "Attack", 2: "Heal", 3: "Buff"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (50, 58)
        return (48, 56)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 17:
            return {"attack": 9, "strength": 4, "heal": 20}
        elif self.ascension >= 2:
            return {"attack": 9, "strength": 3, "heal": 16}
        return {"attack": 8, "strength": 2, "heal": 16}

    def get_move(self, roll: int, total_missing_hp: int = 0) -> MoveInfo:
        dmg = self._get_damage_values()
        heal_threshold = 20 if self.ascension >= 17 else 15

        # Priority: Heal if needed
        if total_missing_hp > heal_threshold and not self.state.last_two_moves(self.HEAL):
            move = MoveInfo(self.HEAL, "Heal", Intent.BUFF, effects={"heal_all": dmg["heal"]})
            self.set_move(move)
            return move

        # Attack check
        if self.ascension >= 17:
            if roll >= 40 and not self.state.last_move(self.ATTACK):
                move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK_DEBUFF,
                               dmg["attack"], effects={"frail": 2})
                self.set_move(move)
                return move
        else:
            if roll >= 40 and not self.state.last_two_moves(self.ATTACK):
                move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK_DEBUFF,
                               dmg["attack"], effects={"frail": 2})
                self.set_move(move)
                return move

        # Default: Buff
        if not self.state.last_two_moves(self.BUFF):
            move = MoveInfo(self.BUFF, "Buff", Intent.BUFF,
                           effects={"strength_all": dmg["strength"]})
        else:
            move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK_DEBUFF,
                           dmg["attack"], effects={"frail": 2})

        self.set_move(move)
        return move


class Snecko(Enemy):
    """
    Snecko - Act 2 basic enemy.

    Moves:
    - GLARE (1): Apply Confused (randomize card costs)
    - BITE (2): 15/18 damage
    - TAIL (3): 8/10 damage + 2 Vulnerable (+ 2 Weak at A17+)

    AI Pattern:
    - First turn: Always GLARE
    - Roll 0-39 (40%): TAIL
    - Roll 40-99 (60%): BITE (no 3x repeat, else TAIL)
    """

    ID = "Snecko"
    NAME = "Snecko"
    TYPE = EnemyType.NORMAL

    GLARE = 1
    BITE = 2
    TAIL = 3

    MOVES = {1: "Glare", 2: "Bite", 3: "Tail Whip"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (120, 125)
        return (114, 120)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"bite": 18, "tail": 10}
        return {"bite": 15, "tail": 8}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.GLARE, "Glare", Intent.STRONG_DEBUFF, effects={"confused": 1})
            self.set_move(move)
            return move

        tail_effects = {"vulnerable": 2}
        if self.ascension >= 17:
            tail_effects["weak"] = 2

        if roll < 40:
            move = MoveInfo(self.TAIL, "Tail Whip", Intent.ATTACK_DEBUFF,
                           dmg["tail"], effects=tail_effects)
        elif self.state.last_two_moves(self.BITE):
            move = MoveInfo(self.TAIL, "Tail Whip", Intent.ATTACK_DEBUFF,
                           dmg["tail"], effects=tail_effects)
        else:
            move = MoveInfo(self.BITE, "Bite", Intent.ATTACK, dmg["bite"])

        self.set_move(move)
        return move


class SnakePlant(Enemy):
    """
    Snake Plant - Act 2 basic enemy.

    Moves:
    - CHOMP (1): 7/8 damage x3 multi-attack
    - SPORES (2): Apply 2 Frail + 2 Weak

    Special: Starts with Malleable (gains block when attacked multiple times per turn)

    AI Pattern:
    - Roll 0-64 (65%): CHOMP (no 3x repeat, else SPORES)
    - Roll 65-99 (35%): SPORES (no repeat, or at A17+ no repeat and not used move before last, else CHOMP)
    """

    ID = "SnakePlant"
    NAME = "Snake Plant"
    TYPE = EnemyType.NORMAL

    CHOMP = 1
    SPORES = 2

    MOVES = {1: "Chomp", 2: "Spores"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        self.state.powers["malleable"] = 1

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (78, 82)
        return (75, 79)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"chomp": 8, "chomp_hits": 3}
        return {"chomp": 7, "chomp_hits": 3}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 17:
            if roll < 65:
                if self.state.last_two_moves(self.CHOMP):
                    move = MoveInfo(self.SPORES, "Spores", Intent.STRONG_DEBUFF,
                                   effects={"frail": 2, "weak": 2})
                else:
                    move = MoveInfo(self.CHOMP, "Chomp", Intent.ATTACK, dmg["chomp"],
                                   hits=dmg["chomp_hits"], is_multi=True)
            else:
                if self.state.last_move(self.SPORES) or self.state.last_move_before(self.SPORES):
                    move = MoveInfo(self.CHOMP, "Chomp", Intent.ATTACK, dmg["chomp"],
                                   hits=dmg["chomp_hits"], is_multi=True)
                else:
                    move = MoveInfo(self.SPORES, "Spores", Intent.STRONG_DEBUFF,
                                   effects={"frail": 2, "weak": 2})
        else:
            if roll < 65:
                if self.state.last_two_moves(self.CHOMP):
                    move = MoveInfo(self.SPORES, "Spores", Intent.STRONG_DEBUFF,
                                   effects={"frail": 2, "weak": 2})
                else:
                    move = MoveInfo(self.CHOMP, "Chomp", Intent.ATTACK, dmg["chomp"],
                                   hits=dmg["chomp_hits"], is_multi=True)
            else:
                if self.state.last_move(self.SPORES):
                    move = MoveInfo(self.CHOMP, "Chomp", Intent.ATTACK, dmg["chomp"],
                                   hits=dmg["chomp_hits"], is_multi=True)
                else:
                    move = MoveInfo(self.SPORES, "Spores", Intent.STRONG_DEBUFF,
                                   effects={"frail": 2, "weak": 2})

        self.set_move(move)
        return move


class Mugger(Enemy):
    """
    Mugger - Act 2 thief enemy (upgraded Looter).

    Moves:
    - MUG (1): 10/11 damage + steal 15/20 gold
    - BIGSWIPE (4): 16/18 damage + steal gold
    - SMOKE_BOMB (2): Gain 11/17 block
    - ESCAPE (3): Flee combat

    Special: Has Thievery power (steals gold on attack)

    AI Pattern:
    - Always starts with MUG
    - After first MUG: MUG again
    - After second MUG: 50% SMOKE_BOMB, 50% BIGSWIPE
    - After BIGSWIPE: SMOKE_BOMB
    - After SMOKE_BOMB: ESCAPE
    """

    ID = "Mugger"
    NAME = "Mugger"
    TYPE = EnemyType.NORMAL

    MUG = 1
    SMOKE_BOMB = 2
    ESCAPE = 3
    BIGSWIPE = 4

    MOVES = {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.slash_count = 0
        self.stolen_gold = 0
        super().__init__(ai_rng, ascension, hp_rng)
        gold_amt = 20 if ascension >= 17 else 15
        self.state.powers["thievery"] = gold_amt

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (50, 54)
        return (48, 52)

    def _get_damage_values(self) -> Dict[str, int]:
        block = 17 if self.ascension >= 17 else 11
        if self.ascension >= 2:
            return {"swipe": 11, "bigswipe": 18, "block": block}
        return {"swipe": 10, "bigswipe": 16, "block": block}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # Always start with MUG
        move = MoveInfo(self.MUG, "Mug", Intent.ATTACK, dmg["swipe"])
        self.set_move(move)
        return move


class Looter(Enemy):
    """
    Looter - Act 1 thief enemy.

    Moves:
    - MUG (1): 10/11 damage + steal 15/20 gold
    - LUNGE (4): 12/14 damage + steal gold
    - SMOKE_BOMB (2): Gain 6 block
    - ESCAPE (3): Flee combat

    Special: Has Thievery power (steals gold on attack)

    AI Pattern:
    - Always starts with MUG
    - After first MUG: MUG again
    - After second MUG: 50% SMOKE_BOMB, 50% LUNGE
    - After LUNGE: SMOKE_BOMB
    - After SMOKE_BOMB: ESCAPE
    """

    ID = "Looter"
    NAME = "Looter"
    TYPE = EnemyType.NORMAL

    MUG = 1
    SMOKE_BOMB = 2
    ESCAPE = 3
    LUNGE = 4

    MOVES = {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.slash_count = 0
        self.stolen_gold = 0
        super().__init__(ai_rng, ascension, hp_rng)
        gold_amt = 20 if ascension >= 17 else 15
        self.state.powers["thievery"] = gold_amt

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (46, 50)
        return (44, 48)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"swipe": 11, "lunge": 14, "block": 6}
        return {"swipe": 10, "lunge": 12, "block": 6}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        move = MoveInfo(self.MUG, "Mug", Intent.ATTACK, dmg["swipe"],
                       effects={"steal_gold": self.state.powers.get("thievery", 15)})
        self.set_move(move)
        return move


class SlaverBlue(Enemy):
    """
    Blue Slaver - Act 1 enemy that applies Weak.

    Moves:
    - STAB (1): 12/13 damage
    - RAKE (4): 7/8 damage + 1/2 Weak

    AI Pattern:
    - Roll 40-99 (60%): STAB (no 3x repeat)
    - Roll 0-39 (40%): RAKE (A17+: no repeat, else no 3x repeat)
    """

    ID = "SlaverBlue"
    NAME = "Blue Slaver"
    TYPE = EnemyType.NORMAL

    STAB = 1
    RAKE = 4

    MOVES = {1: "Stab", 4: "Rake"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (48, 52)
        return (46, 50)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"stab": 13, "rake": 8}
        return {"stab": 12, "rake": 7}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        weak_amt = 2 if self.ascension >= 17 else 1

        if roll >= 40 and not self.state.last_two_moves(self.STAB):
            move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])
        else:
            if self.ascension >= 17:
                if not self.state.last_move(self.RAKE):
                    move = MoveInfo(self.RAKE, "Rake", Intent.ATTACK_DEBUFF,
                                   dmg["rake"], effects={"weak": weak_amt})
                else:
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])
            else:
                if not self.state.last_two_moves(self.RAKE):
                    move = MoveInfo(self.RAKE, "Rake", Intent.ATTACK_DEBUFF,
                                   dmg["rake"], effects={"weak": weak_amt})
                else:
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])

        self.set_move(move)
        return move


class SlaverRed(Enemy):
    """
    Red Slaver - Act 1 enemy that applies Vulnerable and Entangle.

    Moves:
    - STAB (1): 13/14 damage
    - ENTANGLE (2): Apply Entangle (player can't play attacks this turn)
    - SCRAPE (3): 8/9 damage + 1/2 Vulnerable

    AI Pattern:
    - First turn: Always STAB
    - Roll 75-99 (25%): ENTANGLE if not used
    - Roll 55-99 (45%): STAB if Entangle used (no 3x repeat)
    - A17+: SCRAPE (no repeat), else SCRAPE (no 3x repeat)
    """

    ID = "SlaverRed"
    NAME = "Red Slaver"
    TYPE = EnemyType.NORMAL

    STAB = 1
    ENTANGLE = 2
    SCRAPE = 3

    MOVES = {1: "Stab", 2: "Entangle", 3: "Scrape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.used_entangle = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (48, 52)
        return (46, 50)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"stab": 14, "scrape": 9}
        return {"stab": 13, "scrape": 8}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        vuln_amt = 2 if self.ascension >= 17 else 1

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])
            self.set_move(move)
            return move

        if roll >= 75 and not self.used_entangle:
            self.used_entangle = True
            move = MoveInfo(self.ENTANGLE, "Entangle", Intent.STRONG_DEBUFF,
                           effects={"entangle": 1})
            self.set_move(move)
            return move

        if roll >= 55 and self.used_entangle and not self.state.last_two_moves(self.STAB):
            move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])
            self.set_move(move)
            return move

        if self.ascension >= 17:
            if not self.state.last_move(self.SCRAPE):
                move = MoveInfo(self.SCRAPE, "Scrape", Intent.ATTACK_DEBUFF,
                               dmg["scrape"], effects={"vulnerable": vuln_amt})
            else:
                move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])
        else:
            if not self.state.last_two_moves(self.SCRAPE):
                move = MoveInfo(self.SCRAPE, "Scrape", Intent.ATTACK_DEBUFF,
                               dmg["scrape"], effects={"vulnerable": vuln_amt})
            else:
                move = MoveInfo(self.STAB, "Stab", Intent.ATTACK, dmg["stab"])

        self.set_move(move)
        return move


class Taskmaster(Enemy):
    """
    Taskmaster (Slaver Boss) - Act 2 Elite.

    Moves:
    - SCOURING_WHIP (2): 7 damage + shuffle 1/2/3 Wounds into discard (+ Strength at A18)

    AI Pattern:
    - Always uses SCOURING_WHIP
    """

    ID = "SlaverBoss"
    NAME = "Taskmaster"
    TYPE = EnemyType.ELITE

    SCOURING_WHIP = 2

    MOVES = {2: "Scouring Whip"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (57, 64)
        return (54, 60)

    def _get_damage_values(self) -> Dict[str, int]:
        return {"whip": 7}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 18:
            wound_count = 3
            effects = {"wound": wound_count, "strength": 1}
        elif self.ascension >= 3:
            wound_count = 2
            effects = {"wound": wound_count}
        else:
            wound_count = 1
            effects = {"wound": wound_count}

        move = MoveInfo(self.SCOURING_WHIP, "Scouring Whip", Intent.ATTACK_DEBUFF,
                       dmg["whip"], effects=effects)
        self.set_move(move)
        return move


class ShelledParasite(Enemy):
    """
    Shelled Parasite - Act 2 basic enemy.

    Moves:
    - FELL (1): 18/21 damage + 2 Frail
    - DOUBLE_STRIKE (2): 6/7 damage x2 multi-attack
    - LIFE_SUCK (3): 10/12 damage, heal for damage dealt
    - STUNNED (4): Stun (when Plated Armor broken)

    Special:
    - Starts with 14 Plated Armor + 14 Block
    - When Plated Armor reaches 0, becomes STUNNED for a turn

    AI Pattern:
    - First turn (A17+): FELL
    - First turn (below A17): 50% DOUBLE_STRIKE, 50% LIFE_SUCK
    - Roll 0-19 (20%): FELL (no repeat)
    - Roll 20-59 (40%): DOUBLE_STRIKE (no 3x repeat, else LIFE_SUCK)
    - Roll 60-99 (40%): LIFE_SUCK (no 3x repeat, else DOUBLE_STRIKE)
    """

    ID = "Shelled Parasite"
    NAME = "Shelled Parasite"
    TYPE = EnemyType.NORMAL

    FELL = 1
    DOUBLE_STRIKE = 2
    LIFE_SUCK = 3
    STUNNED = 4

    MOVES = {1: "Fell", 2: "Double Strike", 3: "Suck", 4: "Stunned"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        self.state.powers["plated_armor"] = 14
        self.state.block = 14

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (70, 75)
        return (68, 72)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"fell": 21, "double_strike": 7, "suck": 12}
        return {"fell": 18, "double_strike": 6, "suck": 10}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            if self.ascension >= 17:
                move = MoveInfo(self.FELL, "Fell", Intent.ATTACK_DEBUFF,
                               dmg["fell"], effects={"frail": 2})
            elif self.ai_rng.random_boolean(0.5):
                move = MoveInfo(self.DOUBLE_STRIKE, "Double Strike", Intent.ATTACK,
                               dmg["double_strike"], hits=2, is_multi=True)
            else:
                move = MoveInfo(self.LIFE_SUCK, "Suck", Intent.ATTACK_BUFF, dmg["suck"])
            self.set_move(move)
            return move

        if roll < 20:
            if not self.state.last_move(self.FELL):
                move = MoveInfo(self.FELL, "Fell", Intent.ATTACK_DEBUFF,
                               dmg["fell"], effects={"frail": 2})
            else:
                # Re-roll in 20-99 range
                new_roll = self.ai_rng.random_range(20, 99)
                return self.get_move(new_roll)
        elif roll < 60:
            if not self.state.last_two_moves(self.DOUBLE_STRIKE):
                move = MoveInfo(self.DOUBLE_STRIKE, "Double Strike", Intent.ATTACK,
                               dmg["double_strike"], hits=2, is_multi=True)
            else:
                move = MoveInfo(self.LIFE_SUCK, "Suck", Intent.ATTACK_BUFF, dmg["suck"])
        else:
            if not self.state.last_two_moves(self.LIFE_SUCK):
                move = MoveInfo(self.LIFE_SUCK, "Suck", Intent.ATTACK_BUFF, dmg["suck"])
            else:
                move = MoveInfo(self.DOUBLE_STRIKE, "Double Strike", Intent.ATTACK,
                               dmg["double_strike"], hits=2, is_multi=True)

        self.set_move(move)
        return move


class SphericGuardian(Enemy):
    """
    Spheric Guardian - Act 2 basic enemy.

    Moves:
    - BIG_ATTACK (1): 10/11 damage x2 multi-attack
    - INITIAL_BLOCK (2): Gain 25/35 block (Activate)
    - BLOCK_ATTACK (3): Gain 15 block + 10/11 damage
    - FRAIL_ATTACK (4): 10/11 damage + 5 Frail

    Special:
    - Starts with Barricade (block doesn't decay)
    - Starts with 3 Artifact
    - Starts with 40 Block

    AI Pattern:
    - Turn 1: INITIAL_BLOCK (Activate)
    - Turn 2: FRAIL_ATTACK
    - After: Alternates BIG_ATTACK <-> BLOCK_ATTACK
    """

    ID = "SphericGuardian"
    NAME = "Spheric Guardian"
    TYPE = EnemyType.NORMAL

    BIG_ATTACK = 1
    INITIAL_BLOCK = 2
    BLOCK_ATTACK = 3
    FRAIL_ATTACK = 4

    MOVES = {1: "Slam", 2: "Activate", 3: "Harden", 4: "Attack"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.second_move = True
        super().__init__(ai_rng, ascension, hp_rng)
        self.state.powers["barricade"] = 1
        self.state.powers["artifact"] = 3
        self.state.block = 40

    def _get_hp_range(self) -> Tuple[int, int]:
        # Spheric Guardian has fixed HP (low HP but high block)
        return (20, 20)

    def _get_damage_values(self) -> Dict[str, int]:
        activate_block = 35 if self.ascension >= 17 else 25
        if self.ascension >= 2:
            return {"attack": 11, "activate_block": activate_block, "harden_block": 15}
        return {"attack": 10, "activate_block": activate_block, "harden_block": 15}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.INITIAL_BLOCK, "Activate", Intent.DEFEND,
                           block=dmg["activate_block"])
            self.set_move(move)
            return move

        if self.second_move:
            self.second_move = False
            move = MoveInfo(self.FRAIL_ATTACK, "Attack", Intent.ATTACK_DEBUFF,
                           dmg["attack"], effects={"frail": 5})
            self.set_move(move)
            return move

        # Alternate pattern
        if self.state.last_move(self.BIG_ATTACK):
            move = MoveInfo(self.BLOCK_ATTACK, "Harden", Intent.ATTACK_DEFEND,
                           dmg["attack"], block=dmg["harden_block"])
        else:
            move = MoveInfo(self.BIG_ATTACK, "Slam", Intent.ATTACK,
                           dmg["attack"], hits=2, is_multi=True)

        self.set_move(move)
        return move


class BanditBear(Enemy):
    """
    Bandit Bear - City basic enemy (part of Bandit trio).

    Moves:
    - BEAR_HUG (2): Apply -2/-4 Dexterity to player
    - MAUL (1): 18/20 damage
    - LUNGE (3): 9/10 damage + gain 9 block

    Special:
    - Always starts with BEAR_HUG
    - Fixed rotation: BEAR_HUG -> LUNGE -> MAUL -> LUNGE -> MAUL...
    - When dies, triggers deathReact on other Bandits
    """

    ID = "BanditBear"
    NAME = "Bear"
    TYPE = EnemyType.NORMAL

    MAUL = 1
    BEAR_HUG = 2
    LUNGE = 3

    MOVES = {1: "Maul", 2: "Bear Hug", 3: "Lunge"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.move_sequence = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (40, 44)
        return (38, 42)

    def _get_damage_values(self) -> Dict[str, int]:
        dex_reduction = -4 if self.ascension >= 17 else -2
        if self.ascension >= 2:
            return {"maul": 20, "lunge": 10, "lunge_block": 9, "dex_reduction": dex_reduction}
        return {"maul": 18, "lunge": 9, "lunge_block": 9, "dex_reduction": dex_reduction}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # First move is always BEAR_HUG
        if self.move_sequence == 0:
            self.move_sequence = 1
            move = MoveInfo(self.BEAR_HUG, "Bear Hug", Intent.STRONG_DEBUFF,
                           effects={"dexterity": dmg["dex_reduction"]})
        elif self.move_sequence % 2 == 1:
            # Odd sequence: LUNGE
            self.move_sequence += 1
            move = MoveInfo(self.LUNGE, "Lunge", Intent.ATTACK_DEFEND,
                           dmg["lunge"], block=dmg["lunge_block"])
        else:
            # Even sequence: MAUL
            self.move_sequence += 1
            move = MoveInfo(self.MAUL, "Maul", Intent.ATTACK, dmg["maul"])

        self.set_move(move)
        return move


class BanditLeader(Enemy):
    """
    Bandit Leader (Romeo) - City basic enemy (part of Bandit trio).

    Moves:
    - MOCK (2): Unknown intent (does nothing visible, triggers dialogue)
    - AGONIZING_SLASH (3): 10/12 damage + 2/3 Weak
    - CROSS_SLASH (1): 15/17 damage

    Special:
    - Always starts with MOCK
    - Fixed rotation: MOCK -> AGONIZING_SLASH -> CROSS_SLASH -> (A17+: can repeat CROSS_SLASH)
    - Has deathReact dialogue when Bear dies
    """

    ID = "BanditLeader"
    NAME = "Romeo"
    TYPE = EnemyType.NORMAL

    CROSS_SLASH = 1
    MOCK = 2
    AGONIZING_SLASH = 3

    MOVES = {1: "Cross Slash", 2: "Mock", 3: "Agonizing Slash"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.move_sequence = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (37, 41)
        return (35, 39)

    def _get_damage_values(self) -> Dict[str, int]:
        weak_amt = 3 if self.ascension >= 17 else 2
        if self.ascension >= 2:
            return {"cross_slash": 17, "agonizing_slash": 12, "weak": weak_amt}
        return {"cross_slash": 15, "agonizing_slash": 10, "weak": weak_amt}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.move_sequence == 0:
            # First move: MOCK
            self.move_sequence = 1
            move = MoveInfo(self.MOCK, "Mock", Intent.UNKNOWN)
        elif self.move_sequence == 1:
            # Second move: AGONIZING_SLASH
            self.move_sequence = 2
            move = MoveInfo(self.AGONIZING_SLASH, "Agonizing Slash", Intent.ATTACK_DEBUFF,
                           dmg["agonizing_slash"], effects={"weak": dmg["weak"]})
        else:
            # A17+: can use CROSS_SLASH twice before AGONIZING_SLASH
            if self.ascension >= 17 and not self.state.last_two_moves(self.CROSS_SLASH):
                move = MoveInfo(self.CROSS_SLASH, "Cross Slash", Intent.ATTACK, dmg["cross_slash"])
            elif self.state.last_move(self.CROSS_SLASH):
                move = MoveInfo(self.AGONIZING_SLASH, "Agonizing Slash", Intent.ATTACK_DEBUFF,
                               dmg["agonizing_slash"], effects={"weak": dmg["weak"]})
            else:
                move = MoveInfo(self.CROSS_SLASH, "Cross Slash", Intent.ATTACK, dmg["cross_slash"])
            self.move_sequence += 1

        self.set_move(move)
        return move


class BanditPointy(Enemy):
    """
    Bandit Pointy - City basic enemy (part of Bandit trio).

    Moves:
    - ATTACK (1): 5/6 damage x2 multi-attack

    Special:
    - Has deathReact dialogue when Bear dies
    - Always uses same attack pattern
    """

    ID = "BanditChild"
    NAME = "Pointy"
    TYPE = EnemyType.NORMAL

    ATTACK = 1

    MOVES = {1: "Stab"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (34, 34)
        return (30, 30)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"attack": 6}
        return {"attack": 5}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        move = MoveInfo(self.ATTACK, "Stab", Intent.ATTACK,
                       dmg["attack"], hits=2, is_multi=True)

        self.set_move(move)
        return move


# ============ ACT 2 (CITY) ELITES ============

class GremlinLeader(Enemy):
    """
    Gremlin Leader - Act 2 Elite enemy.

    Summons gremlins and buffs allies. Fight starts with 2 random gremlins.

    Moves:
    - RALLY (2): Summons 2 gremlins (if slots available)
    - ENCOURAGE (3): Gives all allies +3/4/5 Strength, gives minions 6/10 Block
    - STAB (4): 6 damage x3 multi-attack

    AI Pattern:
    - If 0 gremlins alive:
      - Roll 0-74 (75%): RALLY if didn't just RALLY, else STAB
      - Roll 75-99 (25%): STAB if didn't just STAB, else RALLY
    - If 1 gremlin alive:
      - Roll 0-49 (50%): RALLY if didn't just RALLY, else roll 50-99
      - Roll 50-79 (30%): ENCOURAGE if didn't just ENCOURAGE, else STAB
      - Roll 80-99 (20%): STAB if didn't just STAB, else roll 0-80
    - If 2+ gremlins alive:
      - Roll 0-65 (66%): ENCOURAGE if didn't just ENCOURAGE, else STAB
      - Roll 66-99 (34%): STAB if didn't just STAB, else ENCOURAGE

    Note: Gremlins are minions (die when leader dies).
    """

    ID = "GremlinLeader"
    NAME = "Gremlin Leader"
    TYPE = EnemyType.ELITE

    RALLY = 2
    ENCOURAGE = 3
    STAB = 4

    MOVES = {2: "Rally!", 3: "Encourage", 4: "Stab"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 num_gremlins_alive: int = 2):
        """
        Args:
            num_gremlins_alive: Number of alive gremlin minions (for AI decisions)
        """
        super().__init__(ai_rng, ascension, hp_rng)
        self.num_gremlins_alive = num_gremlins_alive

    def _get_hp_range(self) -> Tuple[int, int]:
        # A8+ (ascension 8): 145-155
        # Below: 140-148
        if self.ascension >= 8:
            return (145, 155)
        return (140, 148)

    def _get_damage_values(self) -> Dict[str, int]:
        # Damage values from decompiled source
        # A18+: 5 STR, 10 Block
        # A3+: 4 STR, 6 Block
        # Base: 3 STR, 6 Block
        if self.ascension >= 18:
            return {"stab": 6, "stab_count": 3, "str_amt": 5, "block_amt": 10}
        elif self.ascension >= 3:
            return {"stab": 6, "stab_count": 3, "str_amt": 4, "block_amt": 6}
        return {"stab": 6, "stab_count": 3, "str_amt": 3, "block_amt": 6}

    def update_gremlin_count(self, count: int):
        """Update the number of alive gremlins (called externally)."""
        self.num_gremlins_alive = count

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        gremlins = self.num_gremlins_alive

        if gremlins == 0:
            # No gremlins alive
            if roll < 75:
                if not self.state.last_move(self.RALLY):
                    move = MoveInfo(self.RALLY, "Rally!", Intent.UNKNOWN,
                                   effects={"summon_gremlins": 2})
                else:
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
            else:
                if not self.state.last_move(self.STAB):
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
                else:
                    move = MoveInfo(self.RALLY, "Rally!", Intent.UNKNOWN,
                                   effects={"summon_gremlins": 2})

        elif gremlins == 1:
            # 1 gremlin alive
            if roll < 50:
                if not self.state.last_move(self.RALLY):
                    move = MoveInfo(self.RALLY, "Rally!", Intent.UNKNOWN,
                                   effects={"summon_gremlins": 2})
                else:
                    # Recursive call with roll 50-99
                    return self.get_move(self.ai_rng.random_range(50, 99))
            elif roll < 80:
                if not self.state.last_move(self.ENCOURAGE):
                    move = MoveInfo(self.ENCOURAGE, "Encourage", Intent.DEFEND_BUFF,
                                   effects={
                                       "strength_all": dmg["str_amt"],
                                       "block_minions": dmg["block_amt"]
                                   })
                else:
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
            else:
                if not self.state.last_move(self.STAB):
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
                else:
                    # Recursive call with roll 0-80
                    return self.get_move(self.ai_rng.random_range(0, 80))

        else:
            # 2+ gremlins alive
            if roll < 66:
                if not self.state.last_move(self.ENCOURAGE):
                    move = MoveInfo(self.ENCOURAGE, "Encourage", Intent.DEFEND_BUFF,
                                   effects={
                                       "strength_all": dmg["str_amt"],
                                       "block_minions": dmg["block_amt"]
                                   })
                else:
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
            else:
                if not self.state.last_move(self.STAB):
                    move = MoveInfo(self.STAB, "Stab", Intent.ATTACK,
                                   dmg["stab"], hits=dmg["stab_count"], is_multi=True)
                else:
                    move = MoveInfo(self.ENCOURAGE, "Encourage", Intent.DEFEND_BUFF,
                                   effects={
                                       "strength_all": dmg["str_amt"],
                                       "block_minions": dmg["block_amt"]
                                   })

        self.set_move(move)
        return move


class BookOfStabbing(Enemy):
    """
    Book of Stabbing - Act 2 Elite enemy.

    Multi-stab attack that increases each turn.

    Moves:
    - MULTI_STAB (1): 6/7 damage x(stabCount) - stabCount starts at 1 and increases
    - SINGLE_STAB (2): 21/24 single heavy hit

    Pre-battle: Applies Painful Stabs power (when you take unblocked attack damage,
                add a Wound to discard pile)

    AI Pattern:
    - Roll 0-14 (15%):
      - If last move was SINGLE_STAB: increment stabCount, do MULTI_STAB
      - Else: SINGLE_STAB, and if A18+ increment stabCount anyway
    - Roll 15-99 (85%):
      - If last TWO moves were MULTI_STAB: SINGLE_STAB (A18+ increments stabCount)
      - Else: increment stabCount, MULTI_STAB

    stabCount always increases, making MULTI_STAB deadlier each turn.
    """

    ID = "BookOfStabbing"
    NAME = "Book of Stabbing"
    TYPE = EnemyType.ELITE

    MULTI_STAB = 1
    SINGLE_STAB = 2

    MOVES = {1: "Multi-Stab", 2: "Single Stab"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        self.stab_count = 1  # Starts at 1, increases each turn

    def _get_hp_range(self) -> Tuple[int, int]:
        # A8+: 168-172
        # Below: 160-164
        if self.ascension >= 8:
            return (168, 172)
        return (160, 164)

    def _get_damage_values(self) -> Dict[str, int]:
        # A3+: 7 multi-stab, 24 single stab
        # Below: 6 multi-stab, 21 single stab
        if self.ascension >= 3:
            return {"multi_stab": 7, "single_stab": 24}
        return {"multi_stab": 6, "single_stab": 21}

    def use_pre_battle_action(self) -> Dict:
        """Apply Painful Stabs power at start of combat."""
        return {"apply_power": {"painful_stabs": True}}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if roll < 15:
            # 15% chance branch
            if self.state.last_move(self.SINGLE_STAB):
                # After single stab, do multi-stab
                self.stab_count += 1
                move = MoveInfo(self.MULTI_STAB, "Multi-Stab", Intent.ATTACK,
                               dmg["multi_stab"], hits=self.stab_count, is_multi=True)
            else:
                # Single stab
                move = MoveInfo(self.SINGLE_STAB, "Single Stab", Intent.ATTACK,
                               dmg["single_stab"])
                # A18+: stabCount still increases on single stab
                if self.ascension >= 18:
                    self.stab_count += 1
        else:
            # 85% chance branch
            if self.state.last_two_moves(self.MULTI_STAB):
                # Can't do multi-stab 3x in a row
                move = MoveInfo(self.SINGLE_STAB, "Single Stab", Intent.ATTACK,
                               dmg["single_stab"])
                # A18+: stabCount still increases
                if self.ascension >= 18:
                    self.stab_count += 1
            else:
                # Multi-stab (most common)
                self.stab_count += 1
                move = MoveInfo(self.MULTI_STAB, "Multi-Stab", Intent.ATTACK,
                               dmg["multi_stab"], hits=self.stab_count, is_multi=True)

        self.set_move(move)
        return move

# ============ ACT 3 (BEYOND) BASIC ENEMIES ============

class Maw(Enemy):
    """
    The Maw - Act 3 basic enemy (appears in both Act 2 and Act 3).

    Moves:
    - ROAR (2): Apply 3/5 Weak + 3/5 Frail
    - SLAM (3): 25/30 damage
    - DROOL (4): Gain 3/5 Strength
    - NOMNOMNOM (5): 5 damage x(turnCount/2) - scales with turns

    AI Pattern:
    - First turn: Always ROAR
    - Roll 0-49 (50%): NOMNOMNOM (no repeat, hits scale with turn count)
    - After SLAM or NOMNOMNOM: DROOL
    - Otherwise: SLAM
    """

    ID = "Maw"
    NAME = "The Maw"
    TYPE = EnemyType.NORMAL

    ROAR = 2
    SLAM = 3
    DROOL = 4
    NOMNOMNOM = 5

    MOVES = {2: "Roar", 3: "Slam", 4: "Drool", 5: "Nom"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.roared = False
        self.turn_count = 1
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        return (300, 300)  # Fixed HP

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 17:
            return {"slam": 30 if self.ascension >= 2 else 25, "nom": 5,
                    "strength": 5, "debuff_dur": 5}
        return {"slam": 30 if self.ascension >= 2 else 25, "nom": 5,
                "strength": 3, "debuff_dur": 3}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        self.turn_count += 1

        if not self.roared:
            self.roared = True
            move = MoveInfo(self.ROAR, "Roar", Intent.STRONG_DEBUFF,
                           effects={"weak": dmg["debuff_dur"], "frail": dmg["debuff_dur"]})
            self.set_move(move)
            return move

        nom_hits = self.turn_count // 2
        if roll < 50 and not self.state.last_move(self.NOMNOMNOM):
            if nom_hits <= 1:
                move = MoveInfo(self.NOMNOMNOM, "Nom", Intent.ATTACK, dmg["nom"])
            else:
                move = MoveInfo(self.NOMNOMNOM, "Nom", Intent.ATTACK, dmg["nom"],
                               hits=nom_hits, is_multi=True)
        elif self.state.last_move(self.SLAM) or self.state.last_move(self.NOMNOMNOM):
            move = MoveInfo(self.DROOL, "Drool", Intent.BUFF,
                           effects={"strength": dmg["strength"]})
        else:
            move = MoveInfo(self.SLAM, "Slam", Intent.ATTACK, dmg["slam"])

        self.set_move(move)
        return move


class Darkling(Enemy):
    """
    Darkling - Act 3 basic enemy (comes in groups of 3).

    Moves:
    - CHOMP (1): 8/9 damage x2 multi-attack
    - HARDEN (2): Gain 12 block (+ 2 Strength at A17+)
    - NIP (3): 7-11/9-13 damage (rolled at start)
    - COUNT (4): Unknown (waiting while dead)
    - REINCARNATE (5): Revive with 50% HP

    Special:
    - Has Regrow power: revives unless all Darklings die simultaneously
    - When killed, enters "half dead" state
    - Only truly dies when ALL Darklings are killed

    AI Pattern:
    - First turn: 50% HARDEN, 50% NIP
    - Roll 0-39 (40%): CHOMP (no repeat, only for even-indexed Darkling)
    - Roll 40-69 (30%): HARDEN (no repeat, else NIP)
    - Roll 70-99 (30%): NIP (no 3x repeat, else re-roll)
    """

    ID = "Darkling"
    NAME = "Darkling"
    TYPE = EnemyType.NORMAL

    CHOMP = 1
    HARDEN = 2
    NIP = 3
    COUNT = 4
    REINCARNATE = 5

    MOVES = {1: "Chomp", 2: "Harden", 3: "Nip", 4: "Count", 5: "Reincarnate"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 position: int = 0):
        self.position = position
        self.half_dead = False
        super().__init__(ai_rng, ascension, hp_rng)
        # Roll nip damage at initialization
        if ascension >= 2:
            self.nip_damage = self.hp_rng.random_range(9, 13)
        else:
            self.nip_damage = self.hp_rng.random_range(7, 11)
        self.state.powers["regrow"] = 1

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (50, 59)
        return (48, 56)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"chomp": 9, "block": 12}
        return {"chomp": 8, "block": 12}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # If half dead, will reincarnate
        if self.half_dead:
            move = MoveInfo(self.REINCARNATE, "Reincarnate", Intent.BUFF,
                           effects={"heal_percent": 50})
            self.set_move(move)
            return move

        harden_intent = Intent.DEFEND_BUFF if self.ascension >= 17 else Intent.DEFEND
        harden_effects = {"strength": 2} if self.ascension >= 17 else {}

        if self.state.first_turn:
            self.state.first_turn = False
            if roll < 50:
                move = MoveInfo(self.HARDEN, "Harden", harden_intent,
                               block=dmg["block"], effects=harden_effects)
            else:
                move = MoveInfo(self.NIP, "Nip", Intent.ATTACK, self.nip_damage)
            self.set_move(move)
            return move

        if roll < 40:
            # CHOMP only for even-indexed Darklings
            if not self.state.last_move(self.CHOMP) and self.position % 2 == 0:
                move = MoveInfo(self.CHOMP, "Chomp", Intent.ATTACK,
                               dmg["chomp"], hits=2, is_multi=True)
            else:
                new_roll = self.ai_rng.random_range(40, 99)
                return self.get_move(new_roll)
        elif roll < 70:
            if not self.state.last_move(self.HARDEN):
                move = MoveInfo(self.HARDEN, "Harden", harden_intent,
                               block=dmg["block"], effects=harden_effects)
            else:
                move = MoveInfo(self.NIP, "Nip", Intent.ATTACK, self.nip_damage)
        else:
            if not self.state.last_two_moves(self.NIP):
                move = MoveInfo(self.NIP, "Nip", Intent.ATTACK, self.nip_damage)
            else:
                new_roll = self.ai_rng.random_range(0, 99)
                return self.get_move(new_roll)

        self.set_move(move)
        return move


class OrbWalker(Enemy):
    """
    Orb Walker - Act 3 basic enemy.

    Moves:
    - LASER (1): 10/11 damage + shuffle 1 Burn into discard AND draw pile
    - CLAW (2): 15/16 damage

    Special: Has strength-up power (gains 3/5 Strength per turn at A17+)

    AI Pattern:
    - Roll 0-39 (40%): CLAW (no 3x repeat, else LASER)
    - Roll 40-99 (60%): LASER (no 3x repeat, else CLAW)
    """

    ID = "Orb Walker"
    NAME = "Orb Walker"
    TYPE = EnemyType.NORMAL

    LASER = 1
    CLAW = 2

    MOVES = {1: "Laser", 2: "Claw"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        str_up = 5 if ascension >= 17 else 3
        self.state.powers["strength_up"] = str_up

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (92, 102)
        return (90, 96)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"laser": 11, "claw": 16}
        return {"laser": 10, "claw": 15}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if roll < 40:
            if not self.state.last_two_moves(self.CLAW):
                move = MoveInfo(self.CLAW, "Claw", Intent.ATTACK, dmg["claw"])
            else:
                move = MoveInfo(self.LASER, "Laser", Intent.ATTACK_DEBUFF,
                               dmg["laser"], effects={"burn": 2})
        else:
            if not self.state.last_two_moves(self.LASER):
                move = MoveInfo(self.LASER, "Laser", Intent.ATTACK_DEBUFF,
                               dmg["laser"], effects={"burn": 2})
            else:
                move = MoveInfo(self.CLAW, "Claw", Intent.ATTACK, dmg["claw"])

        self.set_move(move)
        return move


class Spiker(Enemy):
    """
    Spiker - Act 3 basic enemy.

    Moves:
    - ATTACK (1): 7/9 damage
    - BUFF_THORNS (2): Gain 2 Thorns

    Special: Starts with 3/4 Thorns (A17+: 6/7 Thorns)

    AI Pattern:
    - If used BUFF_THORNS more than 5 times: always ATTACK
    - Roll 0-49 (50%): ATTACK (no repeat)
    - Otherwise: BUFF_THORNS
    """

    ID = "Spiker"
    NAME = "Spiker"
    TYPE = EnemyType.NORMAL

    ATTACK = 1
    BUFF_THORNS = 2

    MOVES = {1: "Cut", 2: "Spike"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.thorns_count = 0
        super().__init__(ai_rng, ascension, hp_rng)
        if ascension >= 17:
            thorns = 7 if ascension >= 2 else 6
        else:
            thorns = 4 if ascension >= 2 else 3
        self.state.powers["thorns"] = thorns

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (44, 60)
        return (42, 56)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"attack": 9}
        return {"attack": 7}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.thorns_count > 5:
            move = MoveInfo(self.ATTACK, "Cut", Intent.ATTACK, dmg["attack"])
            self.set_move(move)
            return move

        if roll < 50 and not self.state.last_move(self.ATTACK):
            move = MoveInfo(self.ATTACK, "Cut", Intent.ATTACK, dmg["attack"])
        else:
            self.thorns_count += 1
            move = MoveInfo(self.BUFF_THORNS, "Spike", Intent.BUFF, effects={"thorns": 2})

        self.set_move(move)
        return move


class Repulsor(Enemy):
    """
    Repulsor - Act 3 basic enemy.

    Moves:
    - DAZE (1): Shuffle 2 Daze into draw pile
    - ATTACK (2): 11/13 damage

    AI Pattern:
    - Roll 0-19 (20%): ATTACK (no repeat)
    - Otherwise: DAZE
    """

    ID = "Repulsor"
    NAME = "Repulsor"
    TYPE = EnemyType.NORMAL

    DAZE = 1
    ATTACK = 2

    MOVES = {1: "Bash", 2: "Repulse"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (31, 38)
        return (29, 35)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"attack": 13}
        return {"attack": 11}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if roll < 20 and not self.state.last_move(self.ATTACK):
            move = MoveInfo(self.ATTACK, "Bash", Intent.ATTACK, dmg["attack"])
        else:
            move = MoveInfo(self.DAZE, "Repulse", Intent.DEBUFF, effects={"daze": 2})

        self.set_move(move)
        return move


class WrithingMass(Enemy):
    """
    Writhing Mass - Act 3 basic enemy.

    Moves:
    - BIG_HIT (0): 32/38 damage
    - MULTI_HIT (1): 7/9 damage x3 multi-attack
    - ATTACK_BLOCK (2): 15/16 damage + gain 15/16 block
    - ATTACK_DEBUFF (3): 10/12 damage + 2 Weak + 2 Vulnerable
    - MEGA_DEBUFF (4): Add Parasite to deck (used once)

    Special:
    - Has Reactive power (changes intent when damaged)
    - Has Malleable power (gains block when hit multiple times)

    AI Pattern:
    - First turn: 33% MULTI_HIT, 33% ATTACK_BLOCK, 33% ATTACK_DEBUFF
    - Roll 0-9 (10%): BIG_HIT (no repeat)
    - Roll 10-19 (10%): MEGA_DEBUFF (only if not used, no repeat)
    - Roll 20-39 (20%): ATTACK_DEBUFF (no repeat)
    - Roll 40-69 (30%): MULTI_HIT (no repeat)
    - Roll 70-99 (30%): ATTACK_BLOCK (no repeat)
    """

    ID = "WrithingMass"
    NAME = "Writhing Mass"
    TYPE = EnemyType.NORMAL

    BIG_HIT = 0
    MULTI_HIT = 1
    ATTACK_BLOCK = 2
    ATTACK_DEBUFF = 3
    MEGA_DEBUFF = 4

    MOVES = {0: "Crush", 1: "Flail", 2: "Wither", 3: "Implant", 4: "Parasite"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.used_mega_debuff = False
        super().__init__(ai_rng, ascension, hp_rng)
        self.state.powers["reactive"] = 1
        self.state.powers["malleable"] = 1

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (175, 175)
        return (160, 160)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"big_hit": 38, "multi": 9, "block_atk": 16, "debuff_atk": 12}
        return {"big_hit": 32, "multi": 7, "block_atk": 15, "debuff_atk": 10}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            if roll < 33:
                move = MoveInfo(self.MULTI_HIT, "Flail", Intent.ATTACK,
                               dmg["multi"], hits=3, is_multi=True)
            elif roll < 66:
                move = MoveInfo(self.ATTACK_BLOCK, "Wither", Intent.ATTACK_DEFEND,
                               dmg["block_atk"], block=dmg["block_atk"])
            else:
                move = MoveInfo(self.ATTACK_DEBUFF, "Implant", Intent.ATTACK_DEBUFF,
                               dmg["debuff_atk"], effects={"weak": 2, "vulnerable": 2})
            self.set_move(move)
            return move

        if roll < 10:
            if not self.state.last_move(self.BIG_HIT):
                move = MoveInfo(self.BIG_HIT, "Crush", Intent.ATTACK, dmg["big_hit"])
            else:
                new_roll = self.ai_rng.random_range(10, 99)
                return self.get_move(new_roll)
        elif roll < 20:
            if not self.used_mega_debuff and not self.state.last_move(self.MEGA_DEBUFF):
                self.used_mega_debuff = True
                move = MoveInfo(self.MEGA_DEBUFF, "Parasite", Intent.STRONG_DEBUFF,
                               effects={"parasite": 1})
            elif self.ai_rng.random_boolean(0.1):
                move = MoveInfo(self.BIG_HIT, "Crush", Intent.ATTACK, dmg["big_hit"])
            else:
                new_roll = self.ai_rng.random_range(20, 99)
                return self.get_move(new_roll)
        elif roll < 40:
            if not self.state.last_move(self.ATTACK_DEBUFF):
                move = MoveInfo(self.ATTACK_DEBUFF, "Implant", Intent.ATTACK_DEBUFF,
                               dmg["debuff_atk"], effects={"weak": 2, "vulnerable": 2})
            elif self.ai_rng.random_boolean(0.4):
                new_roll = self.ai_rng.random_range(0, 19)
                return self.get_move(new_roll)
            else:
                new_roll = self.ai_rng.random_range(40, 99)
                return self.get_move(new_roll)
        elif roll < 70:
            if not self.state.last_move(self.MULTI_HIT):
                move = MoveInfo(self.MULTI_HIT, "Flail", Intent.ATTACK,
                               dmg["multi"], hits=3, is_multi=True)
            elif self.ai_rng.random_boolean(0.3):
                move = MoveInfo(self.ATTACK_BLOCK, "Wither", Intent.ATTACK_DEFEND,
                               dmg["block_atk"], block=dmg["block_atk"])
            else:
                new_roll = self.ai_rng.random_range(0, 39)
                return self.get_move(new_roll)
        else:
            if not self.state.last_move(self.ATTACK_BLOCK):
                move = MoveInfo(self.ATTACK_BLOCK, "Wither", Intent.ATTACK_DEFEND,
                               dmg["block_atk"], block=dmg["block_atk"])
            else:
                new_roll = self.ai_rng.random_range(0, 69)
                return self.get_move(new_roll)

        self.set_move(move)
        return move


class Transient(Enemy):
    """
    Transient - Act 3 special enemy.

    Moves:
    - ATTACK (1): 30/40 damage, increases by 10 each turn

    Special:
    - Has 999 HP
    - Has Fading 5/6 (dies after 5/6 turns)
    - Has Shifting (can't be damaged for more than remaining HP)
    - Damage increases by 10 each turn: 30/40 -> 40/50 -> 50/60...

    AI Pattern:
    - Always attacks with escalating damage
    """

    ID = "Transient"
    NAME = "Transient"
    TYPE = EnemyType.NORMAL

    ATTACK = 1

    MOVES = {1: "Attack"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.turn_count = 0
        super().__init__(ai_rng, ascension, hp_rng)
        fading = 6 if ascension >= 17 else 5
        self.state.powers["fading"] = fading
        self.state.powers["shifting"] = 1

    def _get_hp_range(self) -> Tuple[int, int]:
        return (999, 999)

    def _get_damage_values(self) -> Dict[str, int]:
        base = 40 if self.ascension >= 2 else 30
        return {"base": base, "increment": 10}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        current_damage = dmg["base"] + (self.turn_count * dmg["increment"])
        self.turn_count += 1

        move = MoveInfo(self.ATTACK, "Attack", Intent.ATTACK, current_damage)
        self.set_move(move)
        return move


class Exploder(Enemy):
    """
    Exploder - Act 3 basic enemy.

    Moves:
    - ATTACK (1): 9/11 damage
    - UNKNOWN (2): Prepares to explode

    Special:
    - Has Explosive power: Explodes after 3 turns dealing 30 damage

    AI Pattern:
    - Turn 1-2: ATTACK
    - Turn 3+: UNKNOWN (preparing to explode)
    """

    ID = "Exploder"
    NAME = "Exploder"
    TYPE = EnemyType.NORMAL

    ATTACK = 1
    EXPLODE = 2

    MOVES = {1: "Slam", 2: "Explode"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.turn_count = 0
        super().__init__(ai_rng, ascension, hp_rng)
        self.state.powers["explosive"] = 3  # Turns until explosion

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (30, 35)
        return (30, 30)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"attack": 11}
        return {"attack": 9}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.turn_count < 2:
            move = MoveInfo(self.ATTACK, "Slam", Intent.ATTACK, dmg["attack"])
        else:
            move = MoveInfo(self.EXPLODE, "Explode", Intent.UNKNOWN)

        self.turn_count += 1
        self.set_move(move)
        return move


class SpireGrowth(Enemy):
    """
    Spire Growth - Act 3 basic enemy.

    Moves:
    - QUICK_TACKLE (1): 16/18 damage
    - CONSTRICT (2): Apply 10/12 Constricted
    - SMASH (3): 22/25 damage

    AI Pattern (A17+):
    - If player not Constricted and last wasn't CONSTRICT: CONSTRICT
    - Roll 0-49 (50%): QUICK_TACKLE (no 3x repeat)
    - If player not Constricted: CONSTRICT
    - Otherwise: SMASH (no 3x repeat, else QUICK_TACKLE)
    """

    ID = "Serpent"
    NAME = "Spire Growth"
    TYPE = EnemyType.NORMAL

    QUICK_TACKLE = 1
    CONSTRICT = 2
    SMASH = 3

    MOVES = {1: "Quick Tackle", 2: "Constrict", 3: "Smash"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (190, 190)
        return (170, 170)

    def _get_damage_values(self) -> Dict[str, int]:
        constrict = 12 if self.ascension >= 17 else 10
        if self.ascension >= 2:
            return {"tackle": 18, "smash": 25, "constrict": constrict}
        return {"tackle": 16, "smash": 22, "constrict": constrict}

    def get_move(self, roll: int, player_constricted: bool = False) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.ascension >= 17:
            if not player_constricted and not self.state.last_move(self.CONSTRICT):
                move = MoveInfo(self.CONSTRICT, "Constrict", Intent.STRONG_DEBUFF,
                               effects={"constricted": dmg["constrict"]})
                self.set_move(move)
                return move

        if roll < 50 and not self.state.last_two_moves(self.QUICK_TACKLE):
            move = MoveInfo(self.QUICK_TACKLE, "Quick Tackle", Intent.ATTACK, dmg["tackle"])
        elif not player_constricted and not self.state.last_move(self.CONSTRICT):
            move = MoveInfo(self.CONSTRICT, "Constrict", Intent.STRONG_DEBUFF,
                           effects={"constricted": dmg["constrict"]})
        elif not self.state.last_two_moves(self.SMASH):
            move = MoveInfo(self.SMASH, "Smash", Intent.ATTACK, dmg["smash"])
        else:
            move = MoveInfo(self.QUICK_TACKLE, "Quick Tackle", Intent.ATTACK, dmg["tackle"])

        self.set_move(move)
        return move


class SnakeDagger(Enemy):
    """
    Snake Dagger - Act 3 minion enemy (summoned by Reptomancer).

    Moves:
    - WOUND (1): 9 damage + shuffle 1 Wound into discard
    - EXPLODE (2): 25 damage + dies

    AI Pattern:
    - First turn: WOUND
    - After: EXPLODE (suicides)
    """

    ID = "Dagger"
    NAME = "Dagger"
    TYPE = EnemyType.NORMAL

    WOUND = 1
    EXPLODE = 2

    MOVES = {1: "Stab", 2: "Explode"}

    def _get_hp_range(self) -> Tuple[int, int]:
        return (20, 25)

    def _get_damage_values(self) -> Dict[str, int]:
        return {"stab": 9, "explode": 25}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.WOUND, "Stab", Intent.ATTACK_DEBUFF,
                           dmg["stab"], effects={"wound": 1})
        else:
            move = MoveInfo(self.EXPLODE, "Explode", Intent.ATTACK, dmg["explode"])

        self.set_move(move)
        return move


# ============ ACT 2 (CITY) BOSSES ============

class Champ(Enemy):
    """
    The Champ - Act 2 Boss.

    Two-phase boss that removes debuffs and gains massive strength at <50% HP.

    Moves:
    Phase 1:
    - HEAVY_SLASH (1): 16/18 damage
    - DEFENSIVE_STANCE (2): Gain 15/18/20 block + 5/6/7 Metallicize
    - FACE_SLAP (4): 12/14 damage + 2 Frail + 2 Vulnerable
    - GLOAT (5): Gain 2/3/4 Strength
    - TAUNT (6): Apply 2 Weak + 2 Vulnerable to player

    Phase 2 (triggered at <50% HP):
    - ANGER (7): Remove all debuffs, gain 3x strength amount (6/9/12)
    - EXECUTE (3): 10 damage x2 (used repeatedly after ANGER)

    AI Pattern (Phase 1):
    - Every 4th turn: TAUNT
    - A19+: 30% (else 15%) DEFENSIVE_STANCE if used < 2 times (no repeat)
    - 30% GLOAT if last wasn't GLOAT and last wasn't DEFENSIVE_STANCE
    - 55% FACE_SLAP (no repeat)
    - Otherwise: HEAVY_SLASH (no repeat, else FACE_SLAP)

    AI Pattern (Phase 2):
    - First move: ANGER (removes debuffs, gains strength)
    - After: EXECUTE (cannot be used 3x in a row, falls back to Phase 1 moves)
    """

    ID = "Champ"
    NAME = "The Champ"
    TYPE = EnemyType.BOSS

    # Move IDs from decompiled source
    HEAVY_SLASH = 1
    DEFENSIVE_STANCE = 2
    EXECUTE = 3
    FACE_SLAP = 4
    GLOAT = 5
    TAUNT = 6
    ANGER = 7

    MOVES = {
        1: "Heavy Slash",
        2: "Defensive Stance",
        3: "Execute",
        4: "Face Slap",
        5: "Gloat",
        6: "Taunt",
        7: "Anger"
    }

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.num_turns = 0
        self.forge_times = 0  # How many times DEFENSIVE_STANCE used
        self.forge_threshold = 2  # Max times to use DEFENSIVE_STANCE
        self.threshold_reached = False  # Phase 2 triggered
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (440, 440)
        return (420, 420)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 19:
            return {
                "slash": 18,
                "execute": 10,
                "slap": 14,
                "strength": 4,
                "forge": 7,
                "block": 20
            }
        elif self.ascension >= 9:
            return {
                "slash": 18,
                "execute": 10,
                "slap": 14,
                "strength": 3,
                "forge": 6,
                "block": 18
            }
        elif self.ascension >= 4:
            return {
                "slash": 18,
                "execute": 10,
                "slap": 14,
                "strength": 3,
                "forge": 5,
                "block": 15
            }
        return {
            "slash": 16,
            "execute": 10,
            "slap": 12,
            "strength": 2,
            "forge": 5,
            "block": 15
        }

    def check_phase_transition(self) -> bool:
        """Check if should transition to Phase 2 (<50% HP)."""
        if self.state.current_hp < self.state.max_hp // 2 and not self.threshold_reached:
            return True
        return False

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        self.num_turns += 1

        # Check for phase transition
        if self.check_phase_transition():
            self.threshold_reached = True
            move = MoveInfo(
                self.ANGER, "Anger", Intent.BUFF,
                effects={
                    "remove_debuffs": True,
                    "remove_shackled": True,
                    "strength": dmg["strength"] * 3
                }
            )
            self.set_move(move)
            return move

        # Phase 2: Execute spam (but not 3x in a row)
        if self.threshold_reached:
            # Only use Execute if NOT used in last 2 turns
            if not self.state.last_move(self.EXECUTE) and not self.state.last_move_before(self.EXECUTE):
                move = MoveInfo(
                    self.EXECUTE, "Execute", Intent.ATTACK,
                    dmg["execute"], hits=2, is_multi=True
                )
                self.set_move(move)
                return move
            # Fall through to Phase 1 logic if Execute was used twice in a row

        # Phase 1: Normal pattern

        # Every 4th turn in Phase 1: TAUNT
        if self.num_turns == 4 and not self.threshold_reached:
            self.num_turns = 0
            move = MoveInfo(
                self.TAUNT, "Taunt", Intent.DEBUFF,
                effects={"weak": 2, "vulnerable": 2}
            )
            self.set_move(move)
            return move

        # DEFENSIVE_STANCE check
        stance_threshold = 30 if self.ascension >= 19 else 15
        if (not self.state.last_move(self.DEFENSIVE_STANCE) and
                self.forge_times < self.forge_threshold and
                roll <= stance_threshold):
            self.forge_times += 1
            move = MoveInfo(
                self.DEFENSIVE_STANCE, "Defensive Stance", Intent.DEFEND_BUFF,
                block=dmg["block"],
                effects={"metallicize": dmg["forge"]}
            )
            self.set_move(move)
            return move

        # GLOAT check
        if (not self.state.last_move(self.GLOAT) and
                not self.state.last_move(self.DEFENSIVE_STANCE) and
                roll <= 30):
            move = MoveInfo(
                self.GLOAT, "Gloat", Intent.BUFF,
                effects={"strength": dmg["strength"]}
            )
            self.set_move(move)
            return move

        # FACE_SLAP check
        if not self.state.last_move(self.FACE_SLAP) and roll <= 55:
            move = MoveInfo(
                self.FACE_SLAP, "Face Slap", Intent.ATTACK_DEBUFF,
                dmg["slap"],
                effects={"frail": 2, "vulnerable": 2}
            )
            self.set_move(move)
            return move

        # HEAVY_SLASH (default) or FACE_SLAP if can't repeat
        if not self.state.last_move(self.HEAVY_SLASH):
            move = MoveInfo(
                self.HEAVY_SLASH, "Heavy Slash", Intent.ATTACK,
                dmg["slash"]
            )
        else:
            move = MoveInfo(
                self.FACE_SLAP, "Face Slap", Intent.ATTACK_DEBUFF,
                dmg["slap"],
                effects={"frail": 2, "vulnerable": 2}
            )

        self.set_move(move)
        return move


class TheCollector(Enemy):
    """
    The Collector - Act 2 Boss.

    Spawns TorchHead minions and applies Mega Debuff.

    Moves:
    - SPAWN (1): Summon 2 TorchHead minions (first turn)
    - FIREBALL (2): 18/21 damage
    - BUFF (3): Gain 15/18 block (A19: +5 more), give all monsters 3/4/5 Strength
    - MEGA_DEBUFF (4): Apply 3/5 Weak + Vulnerable + Frail to player (used once after turn 3)
    - REVIVE (5): Resummon dead TorchHeads (25% chance when minion dead)

    Minions spawned: TorchHead (2x at start)

    AI Pattern:
    - First turn: SPAWN (summon 2 TorchHeads)
    - Turn 4+: MEGA_DEBUFF if not used
    - 25% REVIVE if minion dead (no repeat)
    - 70% FIREBALL (no 2x repeat)
    - Otherwise: BUFF (no repeat, else FIREBALL)
    """

    ID = "TheCollector"
    NAME = "The Collector"
    TYPE = EnemyType.BOSS

    # Move IDs from decompiled source
    SPAWN = 1
    FIREBALL = 2
    BUFF = 3
    MEGA_DEBUFF = 4
    REVIVE = 5

    MOVES = {
        1: "Spawn",
        2: "Fireball",
        3: "Buff",
        4: "Mega Debuff",
        5: "Revive"
    }

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.turns_taken = 0
        self.ult_used = False  # Mega Debuff used
        self.initial_spawn = True
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (300, 300)
        return (282, 282)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 19:
            return {
                "fireball": 21,
                "strength": 5,
                "block": 18,  # Base block, +5 added at A19
                "mega_debuff": 5
            }
        elif self.ascension >= 9:
            return {
                "fireball": 21,
                "strength": 4,
                "block": 18,
                "mega_debuff": 3
            }
        elif self.ascension >= 4:
            return {
                "fireball": 21,
                "strength": 4,
                "block": 15,
                "mega_debuff": 3
            }
        return {
            "fireball": 18,
            "strength": 3,
            "block": 15,
            "mega_debuff": 3
        }

    def get_move(self, roll: int, minion_dead: bool = False) -> MoveInfo:
        dmg = self._get_damage_values()

        # First turn: Spawn minions
        if self.initial_spawn:
            self.initial_spawn = False
            move = MoveInfo(
                self.SPAWN, "Spawn", Intent.UNKNOWN,
                effects={"spawn_torchheads": 2}
            )
            self.turns_taken += 1
            self.set_move(move)
            return move

        # Mega Debuff after turn 3 if not used
        if self.turns_taken >= 3 and not self.ult_used:
            self.ult_used = True
            move = MoveInfo(
                self.MEGA_DEBUFF, "Mega Debuff", Intent.STRONG_DEBUFF,
                effects={
                    "weak": dmg["mega_debuff"],
                    "vulnerable": dmg["mega_debuff"],
                    "frail": dmg["mega_debuff"]
                }
            )
            self.turns_taken += 1
            self.set_move(move)
            return move

        # Revive check (25% if minion dead)
        if roll <= 25 and minion_dead and not self.state.last_move(self.REVIVE):
            move = MoveInfo(
                self.REVIVE, "Revive", Intent.UNKNOWN,
                effects={"revive_torchheads": True}
            )
            self.turns_taken += 1
            self.set_move(move)
            return move

        # Fireball (70% chance, no 2x repeat)
        if roll <= 70 and not self.state.last_two_moves(self.FIREBALL):
            move = MoveInfo(
                self.FIREBALL, "Fireball", Intent.ATTACK,
                dmg["fireball"]
            )
            self.turns_taken += 1
            self.set_move(move)
            return move

        # Buff (no repeat, else Fireball)
        if not self.state.last_move(self.BUFF):
            block_amt = dmg["block"]
            if self.ascension >= 19:
                block_amt += 5
            move = MoveInfo(
                self.BUFF, "Buff", Intent.DEFEND_BUFF,
                block=block_amt,
                effects={"strength_all_monsters": dmg["strength"]}
            )
        else:
            move = MoveInfo(
                self.FIREBALL, "Fireball", Intent.ATTACK,
                dmg["fireball"]
            )

        self.turns_taken += 1
        self.set_move(move)
        return move


class BronzeAutomaton(Enemy):
    """
    Bronze Automaton - Act 2 Boss.

    Charges up Hyper Beam and spawns BronzeOrb minions.

    Moves:
    - SPAWN_ORBS (4): First turn - Summon 2 BronzeOrbs
    - FLAIL (1): 7/8 damage x2
    - BOOST (5): Gain 9/12 block + 3/4 Strength
    - HYPER_BEAM (2): 45/50 damage (used every 5th turn cycle)
    - STUNNED (3): Stun after Hyper Beam (below A19)

    Minions spawned: BronzeOrb (2x at start)

    Pre-battle: Gains 3 Artifact

    AI Pattern:
    - First turn: SPAWN_ORBS
    - Every 5th turn (numTurns == 4): HYPER_BEAM
    - After HYPER_BEAM:
      - A19+: BOOST (no stun)
      - Below A19: STUNNED
    - After STUNNED/BOOST/SPAWN: FLAIL
    - Otherwise: BOOST
    """

    ID = "BronzeAutomaton"
    NAME = "Bronze Automaton"
    TYPE = EnemyType.BOSS

    # Move IDs from decompiled source
    FLAIL = 1
    HYPER_BEAM = 2
    STUNNED = 3
    SPAWN_ORBS = 4
    BOOST = 5

    MOVES = {
        1: "Flail",
        2: "Hyper Beam",
        3: "Stunned",
        4: "Spawn Orbs",
        5: "Boost"
    }

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.num_turns = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (320, 320)
        return (300, 300)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 9:
            block = 12
        else:
            block = 9

        if self.ascension >= 4:
            return {
                "flail": 8,
                "beam": 50,
                "strength": 4,
                "block": block
            }
        return {
            "flail": 7,
            "beam": 45,
            "strength": 3,
            "block": block
        }

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        return {
            "self_effects": {"artifact": 3}
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # First turn: Spawn Orbs
        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(
                self.SPAWN_ORBS, "Spawn Orbs", Intent.UNKNOWN,
                effects={"spawn_bronze_orbs": 2}
            )
            self.set_move(move)
            return move

        # Hyper Beam every 5th turn (numTurns reaches 4)
        if self.num_turns == 4:
            self.num_turns = 0
            move = MoveInfo(
                self.HYPER_BEAM, "Hyper Beam", Intent.ATTACK,
                dmg["beam"]
            )
            self.set_move(move)
            return move

        # After Hyper Beam
        if self.state.last_move(self.HYPER_BEAM):
            if self.ascension >= 19:
                # A19+: Boost instead of stun
                move = MoveInfo(
                    self.BOOST, "Boost", Intent.DEFEND_BUFF,
                    block=dmg["block"],
                    effects={"strength": dmg["strength"]}
                )
            else:
                # Below A19: Stunned
                move = MoveInfo(
                    self.STUNNED, "Stunned", Intent.STUN
                )
            self.set_move(move)
            return move

        # After Stunned, Boost, or Spawn: Flail
        if (self.state.last_move(self.STUNNED) or
                self.state.last_move(self.BOOST) or
                self.state.last_move(self.SPAWN_ORBS)):
            move = MoveInfo(
                self.FLAIL, "Flail", Intent.ATTACK,
                dmg["flail"], hits=2, is_multi=True
            )
        else:
            # Default: Boost
            move = MoveInfo(
                self.BOOST, "Boost", Intent.DEFEND_BUFF,
                block=dmg["block"],
                effects={"strength": dmg["strength"]}
            )

        self.num_turns += 1
        self.set_move(move)
        return move


# ============ ACT 3 (BEYOND) BOSSES ============

class AwakenedOne(Enemy):
    """
    Awakened One - Act 3 Boss (Two-phase fight).

    Phase 1 Moves:
    - SLASH (1): 20 damage
    - SOUL_STRIKE (2): 6 damage x4

    Phase 2 Moves (after rebirth):
    - DARK_ECHO (5): 40 damage
    - SLUDGE (6): 18 damage + 1 Void card to draw pile
    - TACKLE (8): 10 damage x3

    Pre-battle:
    - Regenerate: 10 HP/turn (A19: 15)
    - Curiosity: Gains 1 Strength when player plays Power (A19: 2)
    - Unawakened: Can't die until Phase 2
    - A4+: +2 Strength

    AI Pattern:
    Phase 1:
    - First turn: Always SLASH
    - Roll 0-24: SOUL_STRIKE (no repeat) -> fallback SLASH
    - Roll 25-99: SLASH (no 2x repeat) -> fallback SOUL_STRIKE

    Phase 2:
    - First turn after rebirth: Always DARK_ECHO
    - Roll 0-49: SLUDGE (no 2x repeat) -> fallback TACKLE
    - Roll 50-99: TACKLE (no 2x repeat) -> fallback SLUDGE
    """

    ID = "AwakenedOne"
    NAME = "Awakened One"
    TYPE = EnemyType.BOSS

    # Phase 1 moves
    SLASH = 1
    SOUL_STRIKE = 2
    REBIRTH = 3

    # Phase 2 moves
    DARK_ECHO = 5
    SLUDGE = 6
    TACKLE = 8

    MOVES = {
        1: "Slash", 2: "Soul Strike", 3: "Rebirth",
        5: "Dark Echo", 6: "Sludge", 8: "Tackle"
    }

    # Damage constants from source
    SLASH_DMG = 20
    SS_DMG = 6
    SS_AMT = 4
    ECHO_DMG = 40
    SLUDGE_DMG = 18
    TACKLE_DMG = 10
    TACKLE_AMT = 3

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.phase = 1  # 1 = form1, 2 = form2
        self.phase_first_turn = True  # Reset after rebirth
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        # Same HP for both phases
        if self.ascension >= 9:
            return (320, 320)
        return (300, 300)

    def _get_damage_values(self) -> Dict[str, int]:
        # Damage values are fixed, not ascension-dependent
        return {
            "slash": self.SLASH_DMG,
            "soul_strike": self.SS_DMG,
            "soul_strike_hits": self.SS_AMT,
            "dark_echo": self.ECHO_DMG,
            "sludge": self.SLUDGE_DMG,
            "tackle": self.TACKLE_DMG,
            "tackle_hits": self.TACKLE_AMT,
        }

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        if self.ascension >= 19:
            regen = 15
            curiosity = 2
        else:
            regen = 10
            curiosity = 1

        effects = {
            "self_effects": {
                "regenerate": regen,
                "curiosity": curiosity,
                "unawakened": True,  # Can't die in phase 1
            }
        }

        # A4+: +2 Strength at start
        if self.ascension >= 4:
            effects["self_effects"]["strength"] = 2

        return effects

    def get_phase2_hp(self) -> int:
        """Get max HP for phase 2 (used when rebirth)."""
        if self.ascension >= 9:
            return 320
        return 300

    def trigger_rebirth(self):
        """Called when Awakened One dies in Phase 1 - transitions to Phase 2."""
        self.phase = 2
        self.phase_first_turn = True
        # HP is restored to full in phase 2
        phase2_hp = self.get_phase2_hp()
        self.state.max_hp = phase2_hp
        self.state.current_hp = phase2_hp
        # Clear phase 1 powers (Curiosity, Unawakened, debuffs)
        self.state.powers = {}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.phase == 1:
            # Phase 1 logic
            if self.phase_first_turn:
                self.phase_first_turn = False
                move = MoveInfo(self.SLASH, "Slash", Intent.ATTACK, dmg["slash"])
            elif roll < 25:
                # 25% branch - prefer Soul Strike
                if not self.state.last_move(self.SOUL_STRIKE):
                    move = MoveInfo(self.SOUL_STRIKE, "Soul Strike", Intent.ATTACK,
                                   dmg["soul_strike"], hits=dmg["soul_strike_hits"], is_multi=True)
                else:
                    move = MoveInfo(self.SLASH, "Slash", Intent.ATTACK, dmg["slash"])
            else:
                # 75% branch - prefer Slash
                if not self.state.last_two_moves(self.SLASH):
                    move = MoveInfo(self.SLASH, "Slash", Intent.ATTACK, dmg["slash"])
                else:
                    move = MoveInfo(self.SOUL_STRIKE, "Soul Strike", Intent.ATTACK,
                                   dmg["soul_strike"], hits=dmg["soul_strike_hits"], is_multi=True)
        else:
            # Phase 2 logic
            if self.phase_first_turn:
                self.phase_first_turn = False
                move = MoveInfo(self.DARK_ECHO, "Dark Echo", Intent.ATTACK, dmg["dark_echo"])
            elif roll < 50:
                # 50% branch - prefer Sludge
                if not self.state.last_two_moves(self.SLUDGE):
                    move = MoveInfo(self.SLUDGE, "Sludge", Intent.ATTACK_DEBUFF,
                                   dmg["sludge"], effects={"void": 1})
                else:
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK,
                                   dmg["tackle"], hits=dmg["tackle_hits"], is_multi=True)
            else:
                # 50% branch - prefer Tackle
                if not self.state.last_two_moves(self.TACKLE):
                    move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK,
                                   dmg["tackle"], hits=dmg["tackle_hits"], is_multi=True)
                else:
                    move = MoveInfo(self.SLUDGE, "Sludge", Intent.ATTACK_DEBUFF,
                                   dmg["sludge"], effects={"void": 1})

        self.set_move(move)
        return move

    def should_rebirth(self) -> bool:
        """Check if should trigger rebirth (HP <= 0 in phase 1)."""
        return self.phase == 1 and self.state.current_hp <= 0


class TimeEater(Enemy):
    """
    Time Eater - Act 3 Boss.

    Moves:
    - REVERBERATE (2): 7/8 damage x3
    - RIPPLE (3): Gain 20 block + 1 Weak, 1 Vulnerable (A19: +1 Frail)
    - HEAD_SLAM (4): 26/32 damage + 1 Draw Reduction (A19: +2 Slimed)
    - HASTE (5): Remove debuffs, heal to 50% HP (A19: +32 block)

    Pre-battle:
    - Time Warp: After player plays 12 cards, enemy heals and gains Strength

    AI Pattern:
    - Haste triggers automatically when HP < 50% (once per fight)
    - Roll 0-44 (45%): REVERBERATE (no 2x repeat) -> recurse with 50-99
    - Roll 45-79 (35%): HEAD_SLAM (no repeat) -> 66% REVERBERATE else RIPPLE
    - Roll 80-99 (20%): RIPPLE (no repeat) -> recurse with 0-74
    """

    ID = "TimeEater"
    NAME = "Time Eater"
    TYPE = EnemyType.BOSS

    REVERBERATE = 2
    RIPPLE = 3
    HEAD_SLAM = 4
    HASTE = 5

    MOVES = {2: "Reverberate", 3: "Ripple", 4: "Head Slam", 5: "Haste"}

    # Constants from source
    REVERB_DMG = 7
    REVERB_AMT = 3
    A_2_REVERB_DMG = 8
    RIPPLE_BLOCK = 20
    HEAD_SLAM_DMG = 26
    A_2_HEAD_SLAM_DMG = 32
    HEAD_SLAM_DRAW_REDUCTION = 1
    RIPPLE_DEBUFF_TURNS = 1

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.used_haste = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (480, 480)
        return (456, 456)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {
                "reverberate": self.A_2_REVERB_DMG,
                "head_slam": self.A_2_HEAD_SLAM_DMG,
            }
        return {
            "reverberate": self.REVERB_DMG,
            "head_slam": self.HEAD_SLAM_DMG,
        }

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        return {
            "self_effects": {
                "time_warp": 12,  # After 12 cards, triggers heal + strength
            }
        }

    def should_use_haste(self) -> bool:
        """Check if should use Haste (HP < 50% and not used yet)."""
        return (not self.used_haste and
                self.state.current_hp < self.state.max_hp // 2)

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # Haste triggers when HP < 50% (priority over normal pattern)
        if self.should_use_haste():
            self.used_haste = True
            haste_effects = {"remove_debuffs": True, "heal_to_half": True}
            if self.ascension >= 19:
                haste_effects["block"] = dmg["head_slam"]  # 32 block at A19
            move = MoveInfo(self.HASTE, "Haste", Intent.BUFF, effects=haste_effects)
            self.set_move(move)
            return move

        # Normal AI pattern
        if roll < 45:
            # 45% branch - prefer Reverberate
            if not self.state.last_two_moves(self.REVERBERATE):
                move = MoveInfo(self.REVERBERATE, "Reverberate", Intent.ATTACK,
                               dmg["reverberate"], hits=self.REVERB_AMT, is_multi=True)
            else:
                # Recurse with roll 50-99
                return self.get_move(self.ai_rng.random_range(50, 99))
        elif roll < 80:
            # 35% branch - prefer Head Slam
            if not self.state.last_move(self.HEAD_SLAM):
                slam_effects = {"draw_reduction": self.HEAD_SLAM_DRAW_REDUCTION}
                if self.ascension >= 19:
                    slam_effects["slimed"] = 2
                move = MoveInfo(self.HEAD_SLAM, "Head Slam", Intent.ATTACK_DEBUFF,
                               dmg["head_slam"], effects=slam_effects)
            else:
                # 66% Reverberate, else Ripple
                if self.ai_rng.random_boolean(0.66):
                    move = MoveInfo(self.REVERBERATE, "Reverberate", Intent.ATTACK,
                                   dmg["reverberate"], hits=self.REVERB_AMT, is_multi=True)
                else:
                    ripple_effects = {
                        "vulnerable": self.RIPPLE_DEBUFF_TURNS,
                        "weak": self.RIPPLE_DEBUFF_TURNS,
                    }
                    if self.ascension >= 19:
                        ripple_effects["frail"] = self.RIPPLE_DEBUFF_TURNS
                    move = MoveInfo(self.RIPPLE, "Ripple", Intent.DEFEND_DEBUFF,
                                   block=self.RIPPLE_BLOCK, effects=ripple_effects)
        else:
            # 20% branch - prefer Ripple
            if not self.state.last_move(self.RIPPLE):
                ripple_effects = {
                    "vulnerable": self.RIPPLE_DEBUFF_TURNS,
                    "weak": self.RIPPLE_DEBUFF_TURNS,
                }
                if self.ascension >= 19:
                    ripple_effects["frail"] = self.RIPPLE_DEBUFF_TURNS
                move = MoveInfo(self.RIPPLE, "Ripple", Intent.DEFEND_DEBUFF,
                               block=self.RIPPLE_BLOCK, effects=ripple_effects)
            else:
                # Recurse with roll 0-74
                return self.get_move(self.ai_rng.random_range(0, 74))

        self.set_move(move)
        return move


class Donu(Enemy):
    """
    Donu - Act 3 Boss (fights alongside Deca).

    Moves:
    - BEAM (0): 10/12 damage x2
    - CIRCLE_OF_PROTECTION (2): +3 Strength to all monsters

    Pre-battle:
    - Artifact: 2 (A19: 3)

    AI Pattern:
    - Alternates: CIRCLE -> BEAM -> CIRCLE -> BEAM...
    - Starts with CIRCLE (isAttacking=false)
    """

    ID = "Donu"
    NAME = "Donu"
    TYPE = EnemyType.BOSS

    BEAM = 0
    CIRCLE_OF_PROTECTION = 2

    MOVES = {0: "Beam", 2: "Circle of Protection"}

    # Constants from source
    BEAM_DMG = 10
    BEAM_AMT = 2
    A_2_BEAM_DMG = 12
    CIRCLE_STR_AMT = 3
    ARTIFACT_AMT = 2

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.is_attacking = False  # Starts with buff
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (265, 265)
        return (250, 250)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {"beam": self.A_2_BEAM_DMG}
        return {"beam": self.BEAM_DMG}

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        artifact = 3 if self.ascension >= 19 else self.ARTIFACT_AMT
        return {
            "self_effects": {
                "artifact": artifact,
            }
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.is_attacking:
            # Attack turn
            move = MoveInfo(self.BEAM, "Beam", Intent.ATTACK,
                           dmg["beam"], hits=self.BEAM_AMT, is_multi=True)
            self.is_attacking = False
        else:
            # Buff turn - gives Strength to ALL monsters (both Donu and Deca)
            move = MoveInfo(self.CIRCLE_OF_PROTECTION, "Circle of Protection", Intent.BUFF,
                           effects={"strength_all_monsters": self.CIRCLE_STR_AMT})
            self.is_attacking = True

        self.set_move(move)
        return move


class Deca(Enemy):
    """
    Deca - Act 3 Boss (fights alongside Donu).

    Moves:
    - BEAM (0): 10/12 damage x2 + 2 Dazed cards to discard
    - SQUARE_OF_PROTECTION (2): 16 block to all monsters (A19: +3 Plated Armor)

    Pre-battle:
    - Artifact: 2 (A19: 3)
    - Plays music for Donu & Deca fight

    AI Pattern:
    - Alternates: BEAM -> SQUARE -> BEAM -> SQUARE...
    - Starts with BEAM (isAttacking=true)
    """

    ID = "Deca"
    NAME = "Deca"
    TYPE = EnemyType.BOSS

    BEAM = 0
    SQUARE_OF_PROTECTION = 2

    MOVES = {0: "Beam", 2: "Square of Protection"}

    # Constants from source
    BEAM_DMG = 10
    BEAM_AMT = 2
    A_2_BEAM_DMG = 12
    BEAM_DAZE_AMT = 2
    PROTECT_BLOCK = 16
    ARTIFACT_AMT = 2

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.is_attacking = True  # Starts with attack
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (265, 265)
        return (250, 250)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {"beam": self.A_2_BEAM_DMG}
        return {"beam": self.BEAM_DMG}

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        artifact = 3 if self.ascension >= 19 else self.ARTIFACT_AMT
        return {
            "self_effects": {
                "artifact": artifact,
            }
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.is_attacking:
            # Attack turn - deals damage + adds Dazed cards
            move = MoveInfo(self.BEAM, "Beam", Intent.ATTACK_DEBUFF,
                           dmg["beam"], hits=self.BEAM_AMT, is_multi=True,
                           effects={"dazed": self.BEAM_DAZE_AMT})
            self.is_attacking = False
        else:
            # Defend turn - gives block to ALL monsters (both Donu and Deca)
            protect_effects = {"block_all_monsters": self.PROTECT_BLOCK}
            if self.ascension >= 19:
                protect_effects["plated_armor_all_monsters"] = 3

            if self.ascension >= 19:
                intent = Intent.DEFEND_BUFF
            else:
                intent = Intent.DEFEND

            move = MoveInfo(self.SQUARE_OF_PROTECTION, "Square of Protection", intent,
                           effects=protect_effects)
            self.is_attacking = True

        self.set_move(move)
        return move


# ============ ACT 4 ENEMIES ============

class SpireShield(Enemy):
    """
    Spire Shield - Act 4 Elite (fights alongside Spire Spear).

    Moves:
    - BASH (1): 12/14 damage + -1 Strength OR -1 Focus (50% if orbs)
    - FORTIFY (2): 30 block to ALL monsters
    - SMASH (3): 34/38 damage + gain block (A18: 99 block)

    AI Pattern (3-turn cycle):
    - Turn 0: 50% FORTIFY, 50% BASH
    - Turn 1: If last wasn't BASH -> BASH, else FORTIFY
    - Turn 2: SMASH (attack + defend)

    Pre-battle: Applies Surrounded to player, gains Artifact 1 (A18+: 2)
    """

    ID = "SpireShield"
    NAME = "Spire Shield"
    TYPE = EnemyType.ELITE

    BASH = 1
    FORTIFY = 2
    SMASH = 3

    MOVES = {1: "Bash", 2: "Fortify", 3: "Smash"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.move_count = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (125, 125)
        return (110, 110)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 3:
            return {"bash": 14, "smash": 38, "fortify_block": 30}
        return {"bash": 12, "smash": 34, "fortify_block": 30}

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        artifact = 2 if self.ascension >= 18 else 1
        return {
            "player_effects": {"surrounded": True},
            "self_effects": {"artifact": artifact}
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        cycle_pos = self.move_count % 3

        if cycle_pos == 0:
            # 50% chance each
            if self.ai_rng.random_boolean():
                move = MoveInfo(self.FORTIFY, "Fortify", Intent.DEFEND,
                               block=dmg["fortify_block"],
                               effects={"block_all_monsters": dmg["fortify_block"]})
            else:
                move = MoveInfo(self.BASH, "Bash", Intent.ATTACK_DEBUFF,
                               dmg["bash"],
                               effects={"debuff": "strength_or_focus", "amount": -1})
        elif cycle_pos == 1:
            if not self.state.last_move(self.BASH):
                move = MoveInfo(self.BASH, "Bash", Intent.ATTACK_DEBUFF,
                               dmg["bash"],
                               effects={"debuff": "strength_or_focus", "amount": -1})
            else:
                move = MoveInfo(self.FORTIFY, "Fortify", Intent.DEFEND,
                               block=dmg["fortify_block"],
                               effects={"block_all_monsters": dmg["fortify_block"]})
        else:  # cycle_pos == 2
            smash_block = 99 if self.ascension >= 18 else dmg["smash"]
            move = MoveInfo(self.SMASH, "Smash", Intent.ATTACK_DEFEND,
                           dmg["smash"], block=smash_block)

        self.move_count += 1
        self.set_move(move)
        return move


class SpireSpear(Enemy):
    """
    Spire Spear - Act 4 Elite (fights alongside Spire Shield).

    Moves:
    - BURN_STRIKE (1): 5/6 damage x2 + 2 Burns to discard (A18: to draw pile)
    - PIERCER (2): +2 Strength to ALL monsters
    - SKEWER (3): 10 damage x3/x4

    AI Pattern (3-turn cycle):
    - Turn 0: If last wasn't BURN_STRIKE -> BURN_STRIKE, else PIERCER
    - Turn 1: SKEWER
    - Turn 2: 50% PIERCER, 50% BURN_STRIKE

    Pre-battle: Gains Artifact 1 (A18+: 2)
    """

    ID = "SpireSpear"
    NAME = "Spire Spear"
    TYPE = EnemyType.ELITE

    BURN_STRIKE = 1
    PIERCER = 2
    SKEWER = 3

    MOVES = {1: "Burn Strike", 2: "Piercer", 3: "Skewer"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.move_count = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (180, 180)
        return (160, 160)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 3:
            return {"burn_strike": 6, "skewer": 10, "skewer_count": 4}
        return {"burn_strike": 5, "skewer": 10, "skewer_count": 3}

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        artifact = 2 if self.ascension >= 18 else 1
        return {"self_effects": {"artifact": artifact}}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        cycle_pos = self.move_count % 3
        burns_to_draw = self.ascension >= 18

        if cycle_pos == 0:
            if not self.state.last_move(self.BURN_STRIKE):
                move = MoveInfo(self.BURN_STRIKE, "Burn Strike", Intent.ATTACK_DEBUFF,
                               dmg["burn_strike"], hits=2, is_multi=True,
                               effects={"burn": 2, "to_draw_pile": burns_to_draw})
            else:
                move = MoveInfo(self.PIERCER, "Piercer", Intent.BUFF,
                               effects={"strength_all_monsters": 2})
        elif cycle_pos == 1:
            move = MoveInfo(self.SKEWER, "Skewer", Intent.ATTACK,
                           dmg["skewer"], hits=dmg["skewer_count"], is_multi=True)
        else:  # cycle_pos == 2
            if self.ai_rng.random_boolean():
                move = MoveInfo(self.PIERCER, "Piercer", Intent.BUFF,
                               effects={"strength_all_monsters": 2})
            else:
                move = MoveInfo(self.BURN_STRIKE, "Burn Strike", Intent.ATTACK_DEBUFF,
                               dmg["burn_strike"], hits=2, is_multi=True,
                               effects={"burn": 2, "to_draw_pile": burns_to_draw})

        self.move_count += 1
        self.set_move(move)
        return move


class CorruptHeart(Enemy):
    """
    Corrupt Heart - Act 4 Final Boss.

    Moves:
    - DEBILITATE (3): First turn - 2 Vuln, 2 Weak, 2 Frail + 5 status cards
    - BLOOD_SHOTS (1): 2 damage x12/x15
    - ECHO (2): 40/45 damage
    - BUFF (4): +2 Str (+ clears negative), plus cycling buffs

    Buff cycle:
    - Buff 0: +2 Artifact
    - Buff 1: +1 Beat of Death
    - Buff 2: Painful Stabs
    - Buff 3: +10 Strength
    - Buff 4+: +50 Strength

    AI Pattern:
    - Turn 1: Always DEBILITATE
    - Then 3-turn cycle:
      - Turn 0: 50% BLOOD_SHOTS, 50% ECHO
      - Turn 1: If last wasn't ECHO -> ECHO, else BLOOD_SHOTS
      - Turn 2: BUFF

    Pre-battle: Invincible 300 (A19: 200), Beat of Death 1 (A19: 2)
    """

    ID = "CorruptHeart"
    NAME = "Corrupt Heart"
    TYPE = EnemyType.BOSS

    BLOOD_SHOTS = 1
    ECHO = 2
    DEBILITATE = 3
    BUFF = 4

    MOVES = {1: "Blood Shots", 2: "Echo", 3: "Debilitate", 4: "Buff"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.move_count = 0
        self.buff_count = 0
        self.is_first_move = True
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (800, 800)
        return (750, 750)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 4:
            return {"echo": 45, "blood": 2, "blood_count": 15}
        return {"echo": 40, "blood": 2, "blood_count": 12}

    def get_pre_battle_effects(self) -> Dict:
        """Returns effects applied at start of combat."""
        invincible = 200 if self.ascension >= 19 else 300
        beat_of_death = 2 if self.ascension >= 19 else 1
        return {
            "self_effects": {
                "invincible": invincible,
                "beat_of_death": beat_of_death
            }
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # First turn: Always DEBILITATE
        if self.is_first_move:
            self.is_first_move = False
            move = MoveInfo(self.DEBILITATE, "Debilitate", Intent.STRONG_DEBUFF,
                           effects={
                               "vulnerable": 2,
                               "weak": 2,
                               "frail": 2,
                               "status_cards": ["Dazed", "Slimed", "Wound", "Burn", "Void"]
                           })
            self.set_move(move)
            return move

        cycle_pos = self.move_count % 3

        if cycle_pos == 0:
            # 50% chance each attack
            if self.ai_rng.random_boolean():
                move = MoveInfo(self.BLOOD_SHOTS, "Blood Shots", Intent.ATTACK,
                               dmg["blood"], hits=dmg["blood_count"], is_multi=True)
            else:
                move = MoveInfo(self.ECHO, "Echo", Intent.ATTACK, dmg["echo"])
        elif cycle_pos == 1:
            if not self.state.last_move(self.ECHO):
                move = MoveInfo(self.ECHO, "Echo", Intent.ATTACK, dmg["echo"])
            else:
                move = MoveInfo(self.BLOOD_SHOTS, "Blood Shots", Intent.ATTACK,
                               dmg["blood"], hits=dmg["blood_count"], is_multi=True)
        else:  # cycle_pos == 2
            # Buff move - cycling buffs
            buff_effects = {"strength": 2, "clear_negative_strength": True}

            if self.buff_count == 0:
                buff_effects["artifact"] = 2
            elif self.buff_count == 1:
                buff_effects["beat_of_death"] = 1
            elif self.buff_count == 2:
                buff_effects["painful_stabs"] = True
            elif self.buff_count == 3:
                buff_effects["strength"] = 12  # 2 base + 10
            else:  # buff_count >= 4
                buff_effects["strength"] = 52  # 2 base + 50

            self.buff_count += 1
            move = MoveInfo(self.BUFF, "Buff", Intent.BUFF, effects=buff_effects)

        self.move_count += 1
        self.set_move(move)
        return move


# ============ MINION/SUMMONED ENEMIES ============

class TorchHead(Enemy):
    """
    Torch Head - Collector's minion (City boss).

    Moves:
    - TACKLE (1): 7 damage

    AI Pattern:
    - Always uses TACKLE
    - Very simple enemy with no move variation
    """

    ID = "TorchHead"
    NAME = "Torch Head"
    TYPE = EnemyType.NORMAL

    TACKLE = 1

    MOVES = {1: "Tackle"}

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (40, 45)
        return (38, 40)

    def _get_damage_values(self) -> Dict[str, int]:
        return {"tackle": 7}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()
        move = MoveInfo(self.TACKLE, "Tackle", Intent.ATTACK, dmg["tackle"])
        self.set_move(move)
        return move


class BronzeOrb(Enemy):
    """
    Bronze Orb - Automaton's minion (City boss).

    Moves:
    - BEAM (1): 8 damage (laser attack)
    - SUPPORT_BEAM (2): Give 12 block to BronzeAutomaton
    - STASIS (3): Strong debuff - puts a card in stasis

    AI Pattern:
    - 25% chance to use STASIS once (if not used and roll >= 25)
    - 70%+ roll: SUPPORT_BEAM (no 2x repeat, else BEAM)
    - Default: BEAM (no 2x repeat, else SUPPORT_BEAM)
    """

    ID = "BronzeOrb"
    NAME = "Bronze Orb"
    TYPE = EnemyType.NORMAL

    BEAM = 1
    SUPPORT_BEAM = 2
    STASIS = 3

    MOVES = {1: "Beam", 2: "Support Beam", 3: "Stasis"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None,
                 count: int = 0):
        self.count = count  # Orb index (affects animation)
        self.used_stasis = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 9:
            return (54, 60)
        return (52, 58)

    def _get_damage_values(self) -> Dict[str, int]:
        return {"beam": 8, "block": 12}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # One-time Stasis move
        if not self.used_stasis and roll >= 25:
            self.used_stasis = True
            move = MoveInfo(self.STASIS, "Stasis", Intent.STRONG_DEBUFF,
                           effects={"stasis": True})
            self.set_move(move)
            return move

        # Support Beam (give block to Automaton)
        if roll >= 70 and not self.state.last_two_moves(self.SUPPORT_BEAM):
            move = MoveInfo(self.SUPPORT_BEAM, "Support Beam", Intent.DEFEND,
                           block=dmg["block"], effects={"target": "BronzeAutomaton"})
            self.set_move(move)
            return move

        # Beam attack
        if not self.state.last_two_moves(self.BEAM):
            move = MoveInfo(self.BEAM, "Beam", Intent.ATTACK, dmg["beam"])
        else:
            move = MoveInfo(self.SUPPORT_BEAM, "Support Beam", Intent.DEFEND,
                           block=dmg["block"], effects={"target": "BronzeAutomaton"})

        self.set_move(move)
        return move


class GremlinFat(Enemy):
    """
    Fat Gremlin - Gremlin gang minion.

    Moves:
    - BLUNT (2): 4/5 damage + apply 1 Weak (+ 1 Frail at A17+)
    - ESCAPE (99): Flee from combat

    Special: Has escape behavior when other Gremlins die.

    AI Pattern:
    - Always uses BLUNT (Smash attack)
    - Will switch to ESCAPE when triggered by deathReact
    """

    ID = "GremlinFat"
    NAME = "Fat Gremlin"
    TYPE = EnemyType.NORMAL

    BLUNT = 2
    ESCAPE_MOVE = 99

    MOVES = {2: "Smash", 99: "Escape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.escape_next = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (14, 18)
        return (13, 17)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"blunt": 5}
        return {"blunt": 4}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.escape_next:
            move = MoveInfo(self.ESCAPE_MOVE, "Escape", Intent.ESCAPE)
            self.set_move(move)
            return move

        effects = {"weak": 1}
        if self.ascension >= 17:
            effects["frail"] = 1

        move = MoveInfo(self.BLUNT, "Smash", Intent.ATTACK_DEBUFF,
                       dmg["blunt"], effects=effects)
        self.set_move(move)
        return move

    def trigger_escape(self):
        """Called when another Gremlin dies - triggers escape behavior."""
        self.escape_next = True


class GremlinThief(Enemy):
    """
    Sneaky Gremlin - Gremlin gang minion (thief).

    Moves:
    - PUNCTURE (1): 9/10 damage
    - ESCAPE (99): Flee from combat

    Special: Has escape behavior when other Gremlins die.
    Note: Despite name, doesn't actually steal gold in the game.

    AI Pattern:
    - Always uses PUNCTURE
    - Will switch to ESCAPE when triggered by deathReact
    """

    ID = "GremlinThief"
    NAME = "Sneaky Gremlin"
    TYPE = EnemyType.NORMAL

    PUNCTURE = 1
    ESCAPE_MOVE = 99

    MOVES = {1: "Puncture", 99: "Escape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.escape_next = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (11, 15)
        return (10, 14)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"puncture": 10}
        return {"puncture": 9}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.escape_next:
            move = MoveInfo(self.ESCAPE_MOVE, "Escape", Intent.ESCAPE)
            self.set_move(move)
            return move

        move = MoveInfo(self.PUNCTURE, "Puncture", Intent.ATTACK, dmg["puncture"])
        self.set_move(move)
        return move

    def trigger_escape(self):
        """Called when another Gremlin dies - triggers escape behavior."""
        self.escape_next = True


class GremlinTsundere(Enemy):
    """
    Shield Gremlin - Gremlin gang minion (protector).

    Moves:
    - PROTECT (1): Give 7/8/11 block to a random ally
    - BASH (2): 6/8 damage (when alone)
    - ESCAPE (99): Flee from combat

    Special: Has escape behavior when other Gremlins die.

    AI Pattern:
    - Uses PROTECT when allies are alive
    - Uses BASH when alone
    - Will switch to ESCAPE when triggered by deathReact
    """

    ID = "GremlinTsundere"
    NAME = "Shield Gremlin"
    TYPE = EnemyType.NORMAL

    PROTECT = 1
    BASH = 2
    ESCAPE_MOVE = 99

    MOVES = {1: "Protect", 2: "Bash", 99: "Escape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.escape_next = False
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (13, 17)
        return (12, 15)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 17:
            block_amt = 11
        elif self.ascension >= 7:
            block_amt = 8
        else:
            block_amt = 7

        if self.ascension >= 2:
            return {"bash": 8, "block": block_amt}
        return {"bash": 6, "block": block_amt}

    def get_move(self, roll: int, allies_alive: int = 1) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.escape_next:
            move = MoveInfo(self.ESCAPE_MOVE, "Escape", Intent.ESCAPE)
            self.set_move(move)
            return move

        if allies_alive > 1:
            move = MoveInfo(self.PROTECT, "Protect", Intent.DEFEND,
                           block=dmg["block"], effects={"target": "random_ally"})
        else:
            move = MoveInfo(self.BASH, "Bash", Intent.ATTACK, dmg["bash"])

        self.set_move(move)
        return move

    def trigger_escape(self):
        """Called when another Gremlin dies - triggers escape behavior."""
        self.escape_next = True


class GremlinWarrior(Enemy):
    """
    Mad Gremlin - Gremlin gang minion (warrior).

    Moves:
    - SCRATCH (1): 4/5 damage
    - ESCAPE (99): Flee from combat

    Special:
    - Starts with Angry power (gains 1/2 Strength when hit)
    - Has escape behavior when other Gremlins die.

    AI Pattern:
    - Always uses SCRATCH
    - Will switch to ESCAPE when triggered by deathReact
    """

    ID = "GremlinWarrior"
    NAME = "Mad Gremlin"
    TYPE = EnemyType.NORMAL

    SCRATCH = 1
    ESCAPE_MOVE = 99

    MOVES = {1: "Scratch", 99: "Escape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.escape_next = False
        super().__init__(ai_rng, ascension, hp_rng)
        # Angry power
        angry_amt = 2 if ascension >= 17 else 1
        self.state.powers["angry"] = angry_amt

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (21, 25)
        return (20, 24)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"scratch": 5}
        return {"scratch": 4}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.escape_next:
            move = MoveInfo(self.ESCAPE_MOVE, "Escape", Intent.ESCAPE)
            self.set_move(move)
            return move

        move = MoveInfo(self.SCRATCH, "Scratch", Intent.ATTACK, dmg["scratch"])
        self.set_move(move)
        return move

    def trigger_escape(self):
        """Called when another Gremlin dies - triggers escape behavior."""
        self.escape_next = True


class GremlinWizard(Enemy):
    """
    Gremlin Wizard - Gremlin gang minion (mage).

    Moves:
    - CHARGE (2): Charging... (Unknown intent)
    - DOPE_MAGIC (1): 25/30 damage (Ultimate attack after charging)
    - ESCAPE (99): Flee from combat

    Special:
    - Charges for 2 turns (3 total including initial), then fires Ultimate
    - At A17+, fires Ultimate every turn after first Ultimate
    - Has escape behavior when other Gremlins die.

    AI Pattern:
    - Starts with CHARGE
    - At 3 charges: DOPE_MAGIC (Ultimate)
    - After Ultimate (A17+): keeps using DOPE_MAGIC
    - After Ultimate (below A17): resets to CHARGE
    """

    ID = "GremlinWizard"
    NAME = "Gremlin Wizard"
    TYPE = EnemyType.NORMAL

    CHARGE = 2
    DOPE_MAGIC = 1
    ESCAPE_MOVE = 99

    MOVES = {1: "Ultimate", 2: "Charging", 99: "Escape"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.escape_next = False
        self.current_charge = 1  # Starts at 1
        self.charge_limit = 3
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 7:
            return (22, 26)
        return (21, 25)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 2:
            return {"ultimate": 30}
        return {"ultimate": 25}

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        if self.escape_next:
            move = MoveInfo(self.ESCAPE_MOVE, "Escape", Intent.ESCAPE)
            self.set_move(move)
            return move

        # Check if should fire Ultimate
        if self.current_charge >= self.charge_limit:
            move = MoveInfo(self.DOPE_MAGIC, "Ultimate", Intent.ATTACK, dmg["ultimate"])
            # At A17+, keeps firing Ultimate; below A17, resets
            if self.ascension < 17:
                self.current_charge = 0
            self.set_move(move)
            return move

        # Charging
        self.current_charge += 1
        move = MoveInfo(self.CHARGE, "Charging", Intent.UNKNOWN)
        self.set_move(move)
        return move

    def trigger_escape(self):
        """Called when another Gremlin dies - triggers escape behavior."""
        self.escape_next = True


# ============ ACT 3 (BEYOND) ELITES ============

class GiantHead(Enemy):
    """
    Giant Head - Act 3 Elite.

    Moves:
    - GLARE (1): Apply 1 Weak
    - IT_IS_TIME (2): Massive damage (scales with countdown)
    - COUNT (3): 13 damage

    Special:
    - Has Slow power (applies at battle start)
    - Counts down from 5 (4 at A18+), then uses IT_IS_TIME
    - IT_IS_TIME damage scales: base 30/40, +5 each turn after countdown

    AI Pattern:
    - Pre-battle: Applies Slow power
    - While count > 1: Roll 0-49 -> GLARE (no 2x repeat), Roll 50-99 -> COUNT (no 2x repeat)
    - When count <= 1: IT_IS_TIME with escalating damage

    From GiantHead.java:
    - HP: 500 (A8+: 520)
    - COUNT_DMG: 13
    - DEATH_DMG: 30 (A3+: 40), +5 each turn
    - Weak amount: 1
    - Starting count: 5 (A18+: 4)
    """

    ID = "GiantHead"
    NAME = "Giant Head"
    TYPE = EnemyType.ELITE

    # Move IDs
    GLARE = 1
    IT_IS_TIME = 2
    COUNT = 3

    MOVES = {1: "Glare", 2: "It Is Time", 3: "Count"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        super().__init__(ai_rng, ascension, hp_rng)
        # Starting count: 5 (A18+: 4)
        self.count = 4 if ascension >= 18 else 5
        # Base death damage
        self.starting_death_dmg = 40 if ascension >= 3 else 30
        # Slow power applied in pre-battle
        self.state.powers["slow"] = 0

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (520, 520)
        return (500, 500)

    def _get_damage_values(self) -> Dict[str, int]:
        return {
            "count": 13,
            "base_death": self.starting_death_dmg,
            "increment": 5,
            "weak": 1,
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # Check if countdown has reached IT_IS_TIME threshold
        if self.count <= 1:
            # Death damage scales: base + (1 - count) * 5
            # When count = 1, first IT_IS_TIME = base
            # When count = 0, next IT_IS_TIME = base + 5
            # etc.
            if self.count > -6:
                self.count -= 1
            death_dmg = self.starting_death_dmg + (1 - self.count - 1) * dmg["increment"]
            # Cap the damage calculation index at 7 (like in source)
            if death_dmg > self.starting_death_dmg + 35:
                death_dmg = self.starting_death_dmg + 35
            move = MoveInfo(self.IT_IS_TIME, "It Is Time", Intent.ATTACK, death_dmg)
            self.set_move(move)
            return move

        # Decrement count each turn
        self.count -= 1

        # Roll-based move selection
        if roll < 50:
            if not self.state.last_two_moves(self.GLARE):
                move = MoveInfo(self.GLARE, "Glare", Intent.DEBUFF,
                               effects={"weak": dmg["weak"]})
            else:
                move = MoveInfo(self.COUNT, "Count", Intent.ATTACK, dmg["count"])
        else:
            if not self.state.last_two_moves(self.COUNT):
                move = MoveInfo(self.COUNT, "Count", Intent.ATTACK, dmg["count"])
            else:
                move = MoveInfo(self.GLARE, "Glare", Intent.DEBUFF,
                               effects={"weak": dmg["weak"]})

        self.set_move(move)
        return move

    def use_pre_battle_action(self) -> Dict:
        """Returns pre-battle effects to apply."""
        return {"apply_slow": True}


class Nemesis(Enemy):
    """
    Nemesis - Act 3 Elite.

    Moves:
    - TRI_ATTACK (2): 6/7 damage x3 multi-attack
    - SCYTHE (3): 45 damage (2-turn cooldown)
    - TRI_BURN (4): Add 3/5 Burns to discard pile

    Special:
    - Gains Intangible at end of each turn (if doesn't already have it)
    - Scythe has 2-turn cooldown between uses
    - Burns: 3 (A18+: 5)

    AI Pattern:
    - First turn: 50% TRI_ATTACK, 50% TRI_BURN
    - Roll 0-29 (30%): SCYTHE if cooldown <= 0 and not last move, else fallback
    - Roll 30-64 (35%): TRI_ATTACK (no 2x repeat)
    - Roll 65-99 (35%): TRI_BURN (no repeat)
    - At end of turn: Gain Intangible 1 if not already intangible

    From Nemesis.java:
    - HP: 185 (A8+: 200)
    - SCYTHE_DMG: 45
    - FIRE_DMG: 6 (A3+: 7), x3 hits
    - BURN_AMT: 3 (A18+: 5)
    - SCYTHE_COOLDOWN: 2 turns
    """

    ID = "Nemesis"
    NAME = "Nemesis"
    TYPE = EnemyType.ELITE

    # Move IDs
    TRI_ATTACK = 2
    SCYTHE = 3
    TRI_BURN = 4

    MOVES = {2: "Tri Attack", 3: "Scythe", 4: "Burn"}

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.scythe_cooldown = 0
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (200, 200)
        return (185, 185)

    def _get_damage_values(self) -> Dict[str, int]:
        fire_dmg = 7 if self.ascension >= 3 else 6
        burn_amt = 5 if self.ascension >= 18 else 3
        return {
            "scythe": 45,
            "fire": fire_dmg,
            "fire_hits": 3,
            "burn": burn_amt,
        }

    def get_move(self, roll: int) -> MoveInfo:
        dmg = self._get_damage_values()

        # Decrement scythe cooldown
        self.scythe_cooldown -= 1

        # First turn: 50% TRI_ATTACK, 50% TRI_BURN
        if self.state.first_turn:
            self.state.first_turn = False
            if roll < 50:
                move = MoveInfo(self.TRI_ATTACK, "Tri Attack", Intent.ATTACK,
                               dmg["fire"], hits=dmg["fire_hits"], is_multi=True)
            else:
                move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                               effects={"burn": dmg["burn"]})
            self.set_move(move)
            return move

        # Roll-based move selection
        if roll < 30:
            # Try Scythe
            if not self.state.last_move(self.SCYTHE) and self.scythe_cooldown <= 0:
                move = MoveInfo(self.SCYTHE, "Scythe", Intent.ATTACK, dmg["scythe"])
                self.scythe_cooldown = 2
            elif self.ai_rng.random_boolean():
                # Fallback
                if not self.state.last_two_moves(self.TRI_ATTACK):
                    move = MoveInfo(self.TRI_ATTACK, "Tri Attack", Intent.ATTACK,
                                   dmg["fire"], hits=dmg["fire_hits"], is_multi=True)
                else:
                    move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                                   effects={"burn": dmg["burn"]})
            else:
                if not self.state.last_move(self.TRI_BURN):
                    move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                                   effects={"burn": dmg["burn"]})
                else:
                    move = MoveInfo(self.TRI_ATTACK, "Tri Attack", Intent.ATTACK,
                                   dmg["fire"], hits=dmg["fire_hits"], is_multi=True)
        elif roll < 65:
            # TRI_ATTACK
            if not self.state.last_two_moves(self.TRI_ATTACK):
                move = MoveInfo(self.TRI_ATTACK, "Tri Attack", Intent.ATTACK,
                               dmg["fire"], hits=dmg["fire_hits"], is_multi=True)
            elif self.ai_rng.random_boolean():
                if self.scythe_cooldown > 0:
                    move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                                   effects={"burn": dmg["burn"]})
                else:
                    move = MoveInfo(self.SCYTHE, "Scythe", Intent.ATTACK, dmg["scythe"])
                    self.scythe_cooldown = 2
            else:
                move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                               effects={"burn": dmg["burn"]})
        else:
            # TRI_BURN
            if not self.state.last_move(self.TRI_BURN):
                move = MoveInfo(self.TRI_BURN, "Burn", Intent.DEBUFF,
                               effects={"burn": dmg["burn"]})
            elif self.ai_rng.random_boolean() and self.scythe_cooldown <= 0:
                move = MoveInfo(self.SCYTHE, "Scythe", Intent.ATTACK, dmg["scythe"])
                self.scythe_cooldown = 2
            else:
                move = MoveInfo(self.TRI_ATTACK, "Tri Attack", Intent.ATTACK,
                               dmg["fire"], hits=dmg["fire_hits"], is_multi=True)

        self.set_move(move)
        return move

    def end_of_turn_effect(self) -> Dict:
        """
        Returns effects to apply at end of turn.
        Nemesis gains Intangible 1 at end of every turn if not already intangible.
        """
        if self.state.powers.get("intangible", 0) <= 0:
            self.state.powers["intangible"] = 1
            return {"gain_intangible": 1}
        return {}


class Reptomancer(Enemy):
    """
    Reptomancer - Act 3 Elite.

    Moves:
    - SNAKE_STRIKE (1): 13/16 damage x2 + 1 Weak
    - SPAWN_DAGGER (2): Summon 1/2 SnakeDagger minions (UNKNOWN intent)
    - BIG_BITE (3): 30/34 damage

    Special:
    - Starts combat with 2 SnakeDagger minions (one on each side)
    - Can summon up to 4 daggers total (positions tracked)
    - First turn: Always SPAWN_DAGGER
    - Daggers per spawn: 1 (A18+: 2)
    - When Reptomancer dies, all daggers die

    AI Pattern:
    - First turn: Always SPAWN_DAGGER
    - Roll 0-32 (33%): SNAKE_STRIKE (no repeat)
    - Roll 33-65 (33%): SPAWN_DAGGER if can spawn and no 2x repeat
    - Roll 66-99 (34%): BIG_BITE (no repeat)

    From Reptomancer.java:
    - HP: 180-190 (A8+: 190-200)
    - BITE_DMG: 30 (A3+: 34)
    - SNAKE_STRIKE_DMG: 13 (A3+: 16), x2 hits
    - DAGGERS_PER_SPAWN: 1 (A18+: 2)
    - Dagger positions: [210, 75], [-220, 115], [180, 345], [-250, 335]
    """

    ID = "Reptomancer"
    NAME = "Reptomancer"
    TYPE = EnemyType.ELITE

    # Move IDs
    SNAKE_STRIKE = 1
    SPAWN_DAGGER = 2
    BIG_BITE = 3

    MOVES = {1: "Snake Strike", 2: "Summon", 3: "Big Bite"}

    # Dagger positions (from source)
    DAGGER_POSITIONS = [
        (210.0, 75.0),
        (-220.0, 115.0),
        (180.0, 345.0),
        (-250.0, 335.0),
    ]

    def __init__(self, ai_rng: Random, ascension: int = 0, hp_rng: Optional[Random] = None):
        self.daggers_per_spawn = 2 if ascension >= 18 else 1
        # Track which dagger slots are filled (None = empty, True = alive, False = dead)
        self.dagger_slots = [None, None, None, None]
        super().__init__(ai_rng, ascension, hp_rng)

    def _get_hp_range(self) -> Tuple[int, int]:
        if self.ascension >= 8:
            return (190, 200)
        return (180, 190)

    def _get_damage_values(self) -> Dict[str, int]:
        if self.ascension >= 3:
            return {
                "snake_strike": 16,
                "snake_strike_hits": 2,
                "big_bite": 34,
            }
        return {
            "snake_strike": 13,
            "snake_strike_hits": 2,
            "big_bite": 30,
        }

    def use_pre_battle_action(self) -> Dict:
        """
        Returns pre-battle effects.
        Reptomancer spawns with 2 daggers at positions 0 and 1.
        """
        # Mark initial daggers as alive
        self.dagger_slots[0] = True
        self.dagger_slots[1] = True
        return {
            "spawn_daggers": [
                {"position": 0, "x": self.DAGGER_POSITIONS[0][0], "y": self.DAGGER_POSITIONS[0][1]},
                {"position": 1, "x": self.DAGGER_POSITIONS[1][0], "y": self.DAGGER_POSITIONS[1][1]},
            ],
            "apply_minion_power_to_daggers": True,
        }

    def can_spawn(self, alive_monster_count: int) -> bool:
        """Check if Reptomancer can spawn more daggers (max 4 monsters total including self)."""
        return alive_monster_count <= 3

    def get_empty_dagger_slots(self) -> List[int]:
        """Get list of empty or dead dagger slot indices."""
        return [i for i, slot in enumerate(self.dagger_slots) if slot is None or slot is False]

    def get_move(self, roll: int, alive_monster_count: int = 1) -> MoveInfo:
        dmg = self._get_damage_values()

        # First turn: Always SPAWN_DAGGER
        if self.state.first_turn:
            self.state.first_turn = False
            move = MoveInfo(self.SPAWN_DAGGER, "Summon", Intent.UNKNOWN,
                           effects={"spawn_daggers": self.daggers_per_spawn})
            self.set_move(move)
            return move

        # Roll-based move selection
        if roll < 33:
            # SNAKE_STRIKE
            if not self.state.last_move(self.SNAKE_STRIKE):
                move = MoveInfo(self.SNAKE_STRIKE, "Snake Strike", Intent.ATTACK_DEBUFF,
                               dmg["snake_strike"], hits=dmg["snake_strike_hits"], is_multi=True,
                               effects={"weak": 1})
            else:
                # Fallback: re-roll in range 33-99
                new_roll = self.ai_rng.random_range(33, 99)
                return self.get_move(new_roll, alive_monster_count)
        elif roll < 66:
            # SPAWN_DAGGER
            if not self.state.last_two_moves(self.SPAWN_DAGGER):
                if self.can_spawn(alive_monster_count):
                    move = MoveInfo(self.SPAWN_DAGGER, "Summon", Intent.UNKNOWN,
                                   effects={"spawn_daggers": self.daggers_per_spawn})
                else:
                    # Can't spawn, use SNAKE_STRIKE instead
                    move = MoveInfo(self.SNAKE_STRIKE, "Snake Strike", Intent.ATTACK_DEBUFF,
                                   dmg["snake_strike"], hits=dmg["snake_strike_hits"], is_multi=True,
                                   effects={"weak": 1})
            else:
                # Used spawn twice, use SNAKE_STRIKE
                move = MoveInfo(self.SNAKE_STRIKE, "Snake Strike", Intent.ATTACK_DEBUFF,
                               dmg["snake_strike"], hits=dmg["snake_strike_hits"], is_multi=True,
                               effects={"weak": 1})
        else:
            # BIG_BITE
            if not self.state.last_move(self.BIG_BITE):
                move = MoveInfo(self.BIG_BITE, "Big Bite", Intent.ATTACK, dmg["big_bite"])
            else:
                # Fallback: re-roll in range 0-65
                new_roll = self.ai_rng.random_range(0, 65)
                return self.get_move(new_roll, alive_monster_count)

        self.set_move(move)
        return move

    def spawn_daggers(self) -> List[Dict]:
        """
        Execute dagger spawning. Returns list of dagger spawn info.
        Called when SPAWN_DAGGER move is executed.
        """
        spawned = []
        empty_slots = self.get_empty_dagger_slots()
        daggers_to_spawn = min(self.daggers_per_spawn, len(empty_slots))

        for i in range(daggers_to_spawn):
            if i < len(empty_slots):
                slot = empty_slots[i]
                self.dagger_slots[slot] = True
                spawned.append({
                    "position": slot,
                    "x": self.DAGGER_POSITIONS[slot][0],
                    "y": self.DAGGER_POSITIONS[slot][1],
                })

        return spawned

    def on_dagger_death(self, slot: int):
        """Called when a dagger at the given slot dies."""
        if 0 <= slot < len(self.dagger_slots):
            self.dagger_slots[slot] = False

    def on_death(self) -> Dict:
        """When Reptomancer dies, all daggers die."""
        return {"kill_all_daggers": True}


# ============ ENEMY REGISTRY ============

ENEMY_CLASSES = {
    # Exordium Basic
    "JawWorm": JawWorm,
    "Cultist": Cultist,
    "AcidSlime_L": AcidSlimeL,
    "AcidSlime_M": AcidSlimeM,
    "AcidSlime_S": AcidSlimeS,
    "SpikeSlime_L": SpikeSlimeL,
    "SpikeSlime_M": SpikeSlimeM,
    "SpikeSlime_S": SpikeSlimeS,
    "Louse": Louse,
    "FuzzyLouseNormal": LouseNormal,
    "FuzzyLouseDefensive": LouseDefensive,
    "FungiBeast": FungiBeast,
    "Looter": Looter,
    "SlaverBlue": SlaverBlue,
    "SlaverRed": SlaverRed,
    # Exordium Elites
    "GremlinNob": GremlinNob,
    "Lagavulin": Lagavulin,
    "Sentry": Sentries,
    # Exordium Bosses
    "SlimeBoss": SlimeBoss,
    "TheGuardian": TheGuardian,
    "Hexaghost": Hexaghost,
    # Act 2 (City) Basic
    "Chosen": Chosen,
    "Byrd": Byrd,
    "Centurion": Centurion,
    "Healer": Healer,
    "Snecko": Snecko,
    "SnakePlant": SnakePlant,
    "Mugger": Mugger,
    "Shelled Parasite": ShelledParasite,
    "SphericGuardian": SphericGuardian,
    "BanditBear": BanditBear,
    "BanditLeader": BanditLeader,
    "BanditChild": BanditPointy,
    # Act 2 (City) Elites
    "GremlinLeader": GremlinLeader,
    "BookOfStabbing": BookOfStabbing,
    "SlaverBoss": Taskmaster,
    "Taskmaster": Taskmaster,
    # Act 2 (City) Bosses
    "Champ": Champ,
    "TheCollector": TheCollector,
    "BronzeAutomaton": BronzeAutomaton,
    # Act 3 (Beyond) Basic
    "Maw": Maw,
    "Darkling": Darkling,
    "Orb Walker": OrbWalker,
    "Spiker": Spiker,
    "Repulsor": Repulsor,
    "WrithingMass": WrithingMass,
    "Transient": Transient,
    "Exploder": Exploder,
    "Serpent": SpireGrowth,
    "Dagger": SnakeDagger,
    # Act 3 (Beyond) Elites
    "GiantHead": GiantHead,
    "Nemesis": Nemesis,
    "Reptomancer": Reptomancer,
    # Act 3 (Beyond) Bosses
    "AwakenedOne": AwakenedOne,
    "TimeEater": TimeEater,
    "Donu": Donu,
    "Deca": Deca,
    # Act 4 (Ending)
    "SpireShield": SpireShield,
    "SpireSpear": SpireSpear,
    "CorruptHeart": CorruptHeart,
    # Minions/Summoned
    "TorchHead": TorchHead,
    "BronzeOrb": BronzeOrb,
    "GremlinFat": GremlinFat,
    "GremlinThief": GremlinThief,
    "GremlinTsundere": GremlinTsundere,
    "GremlinWarrior": GremlinWarrior,
    "GremlinWizard": GremlinWizard,
}


def create_enemy(enemy_id: str, ai_rng: Random, ascension: int = 0,
                 hp_rng: Optional[Random] = None, **kwargs) -> Enemy:
    """Factory function to create enemies."""
    if enemy_id not in ENEMY_CLASSES:
        raise ValueError(f"Unknown enemy: {enemy_id}")
    return ENEMY_CLASSES[enemy_id](ai_rng, ascension, hp_rng, **kwargs)


# ============ TESTING ============

if __name__ == "__main__":
    from ..state.rng import Random as GameRandom

    print("=== Enemy AI Tests ===\n")

    # Test Jaw Worm
    seed = 12345
    ai_rng = GameRandom(seed)
    hp_rng = GameRandom(seed)

    jaw = JawWorm(ai_rng, ascension=20, hp_rng=hp_rng)
    print(f"Jaw Worm HP: {jaw.state.current_hp}/{jaw.state.max_hp}")

    print("\nJaw Worm move sequence:")
    for i in range(10):
        move = jaw.roll_move()
        print(f"  Turn {i+1}: {move.name} ({move.intent.value})", end="")
        if move.base_damage > 0:
            print(f" - {move.base_damage} dmg", end="")
        if move.block > 0:
            print(f" - {move.block} block", end="")
        print()

    # Test Slime Boss
    print("\n" + "="*40)
    ai_rng2 = GameRandom(seed)
    boss = SlimeBoss(ai_rng2, ascension=20)
    print(f"Slime Boss HP: {boss.state.current_hp}/{boss.state.max_hp}")

    print("\nSlime Boss move sequence:")
    for i in range(6):
        move = boss.roll_move()
        print(f"  Turn {i+1}: {move.name} ({move.intent.value})")
# =============================================================================
# ENEMY DATA DEFINITIONS
# =============================================================================
# Format: {
#     "id": str,           # Game ID
#     "name": str,         # Display name
#     "type": EnemyType,   # NORMAL, ELITE, or BOSS
#     "hp": dict,          # {"base": (min, max), "a7": (min, max), ...}
#     "damage": dict,      # {"move_name": {"base": int, "a2": int, ...}, ...}
#     "moves": dict,       # {move_id: "MoveName", ...}
#     "passives": list,    # List of passive powers at combat start
# }
# =============================================================================


# =============================================================================
# EXORDIUM (ACT 1) - BASIC ENEMIES
# =============================================================================

JAW_WORM_DATA = {
    "id": "JawWorm",
    "name": "Jaw Worm",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (40, 44),
        "a7": (42, 46),
    },
    "damage": {
        "chomp": {"base": 11, "a2": 12},
        "thrash": {"base": 7},
        "thrash_block": {"base": 5},
        "bellow_str": {"base": 3, "a2": 4, "a17": 5},
        "bellow_block": {"base": 6, "a17": 9},
    },
    "moves": {1: "Chomp", 2: "Bellow", 3: "Thrash"},
    "move_ids": {"CHOMP": 1, "BELLOW": 2, "THRASH": 3},
    "passives": [],
}

CULTIST_DATA = {
    "id": "Cultist",
    "name": "Cultist",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 54),
        "a7": (50, 56),
    },
    "damage": {
        "dark_strike": {"base": 6},
        "ritual": {"base": 3, "a2": 4, "a17": 5},
    },
    "moves": {1: "Incantation", 2: "Dark Strike"},
    "move_ids": {"DARK_STRIKE": 1, "INCANTATION": 3},
    "passives": [],
}

ACID_SLIME_M_DATA = {
    "id": "AcidSlime_M",
    "name": "Acid Slime (M)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (28, 32),
        "a7": (29, 34),
    },
    "damage": {
        "spit": {"base": 7, "a2": 8},
        "tackle": {"base": 10, "a2": 12},
    },
    "moves": {1: "Corrosive Spit", 2: "Tackle", 4: "Lick"},
    "move_ids": {"CORROSIVE_SPIT": 1, "TACKLE": 2, "LICK": 4},
    "passives": [],
}

ACID_SLIME_L_DATA = {
    "id": "AcidSlime_L",
    "name": "Acid Slime (L)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (65, 69),
        "a7": (68, 72),
    },
    "damage": {
        "spit": {"base": 11, "a2": 12},
        "tackle": {"base": 16, "a2": 18},
    },
    "moves": {1: "Corrosive Spit", 2: "Tackle", 3: "Split", 4: "Lick"},
    "move_ids": {"CORROSIVE_SPIT": 1, "TACKLE": 2, "SPLIT": 3, "LICK": 4},
    "passives": ["split"],
}

ACID_SLIME_S_DATA = {
    "id": "AcidSlime_S",
    "name": "Acid Slime (S)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (8, 12),
        "a7": (9, 13),
    },
    "damage": {
        "tackle": {"base": 3, "a2": 4},
    },
    "moves": {1: "Tackle", 2: "Lick"},
    "move_ids": {"TACKLE": 1, "LICK": 2},
    "passives": [],
}

SPIKE_SLIME_M_DATA = {
    "id": "SpikeSlime_M",
    "name": "Spike Slime (M)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (28, 32),
        "a7": (29, 34),
    },
    "damage": {
        "tackle": {"base": 8, "a2": 10},
        "frail": {"base": 1, "a17": 2},  # Debuff amount, not damage
    },
    "moves": {1: "Flame Tackle", 4: "Lick"},
    "move_ids": {"FLAME_TACKLE": 1, "LICK": 4},
    "passives": [],
}

SPIKE_SLIME_L_DATA = {
    "id": "SpikeSlime_L",
    "name": "Spike Slime (L)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (64, 70),
        "a7": (67, 73),
    },
    "damage": {
        "tackle": {"base": 16, "a2": 18},
        "frail": {"base": 2, "a17": 3},  # Debuff amount
    },
    "moves": {1: "Flame Tackle", 3: "Split", 4: "Lick"},
    "move_ids": {"FLAME_TACKLE": 1, "SPLIT": 3, "LICK": 4},
    "passives": ["split"],
}

SPIKE_SLIME_S_DATA = {
    "id": "SpikeSlime_S",
    "name": "Spike Slime (S)",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 14),
        "a7": (11, 15),
    },
    "damage": {
        "tackle": {"base": 5, "a2": 6},
    },
    "moves": {1: "Tackle"},
    "move_ids": {"TACKLE": 1},
    "passives": [],
}

LOUSE_DATA = {
    "id": "Louse",
    "name": "Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 15),
        "a7": (11, 17),
    },
    "damage": {
        # Damage is rolled at init: 5-7 base, 6-8 at A2
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "grow_str": {"base": 3, "a17": 4},
    },
    "moves": {3: "Bite", 4: "Grow"},
    "move_ids": {"BITE": 3, "GROW": 4},
    "passives": ["curl_up"],  # 3-7 block (a7: 4-8, a17: 9-12)
}

LOUSE_NORMAL_DATA = {
    "id": "FuzzyLouseNormal",
    "name": "Red Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 15),
        "a7": (11, 16),
    },
    "damage": {
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "grow_str": {"base": 3, "a17": 4},
    },
    "moves": {3: "Bite", 4: "Grow"},
    "move_ids": {"BITE": 3, "GROW": 4},
    "passives": ["curl_up"],
    "curl_up_range": {
        "base": (3, 7),
        "a7": (4, 8),
        "a17": (9, 12),
    },
}

LOUSE_DEFENSIVE_DATA = {
    "id": "FuzzyLouseDefensive",
    "name": "Green Louse",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (11, 17),
        "a7": (12, 18),
    },
    "damage": {
        "bite_min": {"base": 5, "a2": 6},
        "bite_max": {"base": 7, "a2": 8},
        "weak": {"base": 2},  # Spit Web weak amount
    },
    "moves": {3: "Bite", 4: "Spit Web"},
    "move_ids": {"BITE": 3, "SPIT_WEB": 4},
    "passives": ["curl_up"],
    "curl_up_range": {
        "base": (3, 7),
        "a7": (4, 8),
        "a17": (9, 12),
    },
}

FUNGI_BEAST_DATA = {
    "id": "FungiBeast",
    "name": "Fungi Beast",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (22, 28),
        "a7": (24, 28),
    },
    "damage": {
        "bite": {"base": 6},
        "grow_str": {"base": 3, "a2": 4, "a17": 5},
    },
    "moves": {1: "Bite", 2: "Grow"},
    "move_ids": {"BITE": 1, "GROW": 2},
    "passives": ["spore_cloud"],  # 2 vulnerable on death
}

LOOTER_DATA = {
    "id": "Looter",
    "name": "Looter",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (44, 48),
        "a7": (46, 50),
    },
    "damage": {
        "swipe": {"base": 10, "a2": 11},
        "lunge": {"base": 12, "a2": 14},
        "block": {"base": 6},
        "gold_steal": {"base": 15, "a17": 20},
    },
    "moves": {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"},
    "move_ids": {"MUG": 1, "SMOKE_BOMB": 2, "ESCAPE": 3, "LUNGE": 4},
    "passives": ["thievery"],
}

MUGGER_DATA = {
    "id": "Mugger",
    "name": "Mugger",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 52),
        "a7": (50, 54),
    },
    "damage": {
        "swipe": {"base": 10, "a2": 11},
        "bigswipe": {"base": 16, "a2": 18},
        "block": {"base": 11, "a17": 17},
        "gold_steal": {"base": 15, "a17": 20},
    },
    "moves": {1: "Mug", 2: "Smoke Bomb", 3: "Escape", 4: "Lunge"},
    "move_ids": {"MUG": 1, "SMOKE_BOMB": 2, "ESCAPE": 3, "BIGSWIPE": 4},
    "passives": ["thievery"],
}

SLAVER_BLUE_DATA = {
    "id": "SlaverBlue",
    "name": "Blue Slaver",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (46, 50),
        "a7": (48, 52),
    },
    "damage": {
        "stab": {"base": 12, "a2": 13},
        "rake": {"base": 7, "a2": 8},
        "weak": {"base": 1, "a17": 2},
    },
    "moves": {1: "Stab", 4: "Rake"},
    "move_ids": {"STAB": 1, "RAKE": 4},
    "passives": [],
}

SLAVER_RED_DATA = {
    "id": "SlaverRed",
    "name": "Red Slaver",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (46, 50),
        "a7": (48, 52),
    },
    "damage": {
        "stab": {"base": 13, "a2": 14},
        "scrape": {"base": 8, "a2": 9},
        "vulnerable": {"base": 1, "a17": 2},
    },
    "moves": {1: "Stab", 2: "Entangle", 4: "Scrape"},
    "move_ids": {"STAB": 1, "ENTANGLE": 2, "SCRAPE": 4},
    "passives": [],
}


# =============================================================================
# EXORDIUM (ACT 1) - ELITES
# =============================================================================

GREMLIN_NOB_DATA = {
    "id": "GremlinNob",
    "name": "Gremlin Nob",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (82, 86),
        "a8": (85, 90),
    },
    "damage": {
        "bellow_enrage": {"base": 2, "a18": 3},
        "rush": {"base": 14, "a3": 16},
        "skull_bash": {"base": 6, "a3": 8},
        "skull_bash_vuln": {"base": 2},
    },
    "moves": {1: "Bellow", 2: "Rush", 3: "Skull Bash"},
    "move_ids": {"BELLOW": 1, "RUSH": 2, "SKULL_BASH": 3},
    "passives": [],
}

LAGAVULIN_DATA = {
    "id": "Lagavulin",
    "name": "Lagavulin",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (109, 111),
        "a8": (112, 115),
    },
    "damage": {
        "attack": {"base": 18, "a3": 20},
        "siphon_str_dex": {"base": 1, "a18": 2},
    },
    "moves": {1: "Sleep", 2: "Attack", 3: "Siphon Soul"},
    "move_ids": {"SLEEP": 1, "ATTACK": 2, "SIPHON_SOUL": 3},
    "passives": ["metallicize"],  # 8 metallicize while asleep
    "metallicize_amount": 8,
}

SENTRY_DATA = {
    "id": "Sentry",
    "name": "Sentry",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (38, 42),
        "a8": (39, 45),
    },
    "damage": {
        "beam": {"base": 9, "a3": 10},
        "dazed": {"base": 2, "a18": 3},
    },
    "moves": {3: "Bolt", 4: "Beam"},
    "move_ids": {"BOLT": 3, "BEAM": 4},
    "passives": ["artifact"],
    "artifact_amount": 1,
}


# =============================================================================
# EXORDIUM (ACT 1) - BOSSES
# =============================================================================

SLIME_BOSS_DATA = {
    "id": "SlimeBoss",
    "name": "Slime Boss",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (140, 140),
        "a9": (150, 150),
    },
    "damage": {
        "slam": {"base": 35, "a4": 38},
        "slimed": {"base": 3, "a19": 5},
    },
    "moves": {1: "Goop Spray", 2: "Preparing", 3: "Slam", 4: "Split"},
    "move_ids": {"GOOP_SPRAY": 1, "PREPARING": 2, "SLAM": 3, "SPLIT": 4},
    "passives": ["split"],
}

THE_GUARDIAN_DATA = {
    "id": "TheGuardian",
    "name": "The Guardian",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (240, 240),
        "a9": (250, 250),
    },
    "damage": {
        "fierce_bash": {"base": 32, "a4": 36},
        "whirlwind": {"base": 5},
        "whirlwind_hits": {"base": 4},
        "roll": {"base": 9, "a4": 10},
        "twin_slam": {"base": 8},
        "twin_slam_hits": {"base": 2},
        "charge_block": {"base": 9},
    },
    "moves": {
        1: "Charge Up", 2: "Fierce Bash", 3: "Vent Steam",
        4: "Whirlwind", 5: "Roll Attack", 6: "Twin Slam"
    },
    "move_ids": {
        "CHARGE_UP": 1, "FIERCE_BASH": 2, "VENT_STEAM": 3,
        "WHIRLWIND": 4, "ROLL_ATTACK": 5, "TWIN_SLAM": 6
    },
    "passives": ["mode_shift"],
    "mode_shift_threshold": {"base": 30, "a9": 35, "a19": 40},
    "sharp_hide": {"base": 3, "a19": 4},
}

HEXAGHOST_DATA = {
    "id": "Hexaghost",
    "name": "Hexaghost",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (264, 264),
    },
    "damage": {
        "sear": {"base": 6},
        "sear_burn": {"base": 1, "a19": 2},
        "tackle": {"base": 5, "a4": 6},
        "tackle_hits": {"base": 2},
        "inflame_str": {"base": 2, "a19": 3},
        "inflame_block": {"base": 12},
        "inferno": {"base": 2, "a4": 3},
        "inferno_hits": {"base": 6},
    },
    "moves": {
        1: "Activate", 2: "Divider", 3: "Sear",
        4: "Tackle", 5: "Inflame", 6: "Inferno"
    },
    "move_ids": {
        "ACTIVATE": 1, "DIVIDER": 2, "SEAR": 3,
        "TACKLE": 4, "INFLAME": 5, "INFERNO": 6
    },
    "passives": [],
}


# =============================================================================
# CITY (ACT 2) - BASIC ENEMIES
# =============================================================================

CHOSEN_DATA = {
    "id": "Chosen",
    "name": "Chosen",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (95, 99),
        "a7": (98, 103),
    },
    "damage": {
        "poke": {"base": 5, "a2": 6},
        "poke_hits": {"base": 2},
        "zap": {"base": 18, "a2": 21},
        "debilitate": {"base": 10, "a2": 12},
    },
    "moves": {1: "Poke", 2: "Zap", 3: "Debilitate", 4: "Drain", 5: "Hex"},
    "move_ids": {"POKE": 1, "ZAP": 2, "DEBILITATE": 3, "DRAIN": 4, "HEX": 5},
    "passives": [],
}

BYRD_DATA = {
    "id": "Byrd",
    "name": "Byrd",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (25, 31),
        "a7": (26, 33),
    },
    "damage": {
        "peck": {"base": 1},
        "peck_hits": {"base": 5, "a2": 6},
        "swoop": {"base": 12, "a2": 14},
        "headbutt": {"base": 3},
    },
    "moves": {1: "Peck", 2: "Fly", 3: "Swoop", 4: "Stunned", 5: "Headbutt", 6: "Caw"},
    "move_ids": {
        "PECK": 1, "GO_AIRBORNE": 2, "SWOOP": 3,
        "STUNNED": 4, "HEADBUTT": 5, "CAW": 6
    },
    "passives": ["flight"],
    "flight_amount": {"base": 3, "a17": 4},
}

CENTURION_DATA = {
    "id": "Centurion",
    "name": "Centurion",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (76, 80),
        "a7": (78, 83),
    },
    "damage": {
        "slash": {"base": 12, "a2": 14},
        "fury": {"base": 6, "a2": 7},
        "fury_hits": {"base": 3},
        "block": {"base": 15, "a17": 20},
    },
    "moves": {1: "Slash", 2: "Defend", 3: "Fury"},
    "move_ids": {"SLASH": 1, "PROTECT": 2, "FURY": 3},
    "passives": [],
}

HEALER_DATA = {
    "id": "Healer",
    "name": "Mystic",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 56),
        "a7": (50, 58),
    },
    "damage": {
        "attack": {"base": 8, "a2": 9},
        "strength": {"base": 2, "a2": 3, "a17": 4},
        "heal": {"base": 16, "a17": 20},
    },
    "moves": {1: "Attack", 2: "Heal", 3: "Buff"},
    "move_ids": {"ATTACK": 1, "HEAL": 2, "BUFF": 3},
    "passives": [],
}

SNECKO_DATA = {
    "id": "Snecko",
    "name": "Snecko",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (114, 120),
        "a7": (120, 125),
    },
    "damage": {
        "bite": {"base": 15, "a2": 18},
        "tail": {"base": 8, "a2": 10},
    },
    "moves": {1: "Glare", 2: "Bite", 3: "Tail Whip"},
    "move_ids": {"GLARE": 1, "BITE": 2, "TAIL": 3},
    "passives": [],
}

SNAKE_PLANT_DATA = {
    "id": "SnakePlant",
    "name": "Snake Plant",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (75, 79),
        "a7": (78, 82),
    },
    "damage": {
        "chomp": {"base": 7, "a2": 8},
        "chomp_hits": {"base": 3},
    },
    "moves": {1: "Chomp", 2: "Spores"},
    "move_ids": {"CHOMP": 1, "SPORES": 2},
    "passives": ["malleable"],
}

SHELLED_PARASITE_DATA = {
    "id": "Shelled Parasite",
    "name": "Shelled Parasite",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (68, 72),
        "a7": (70, 75),
    },
    "damage": {
        "double_strike": {"base": 6, "a2": 7},
        "double_strike_hits": {"base": 2},
        "fell": {"base": 18, "a2": 21},
        "suck": {"base": 10, "a2": 12},
    },
    "moves": {1: "Double Strike", 2: "Fell", 3: "Stunned", 4: "Suck"},
    "move_ids": {"DOUBLE_STRIKE": 1, "FELL": 2, "STUNNED": 3, "SUCK": 4},
    "passives": ["plated_armor"],
    "plated_armor_amount": {"base": 14, "a7": 14, "a17": 19},
}

SPHERIC_GUARDIAN_DATA = {
    "id": "SphericGuardian",
    "name": "Spheric Guardian",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 20),
        "a7": (20, 20),
    },
    "damage": {
        "slam": {"base": 10, "a2": 11},
        "activate": {"base": 10, "a17": 15},  # Block amount
    },
    "moves": {1: "Slam", 2: "Activate", 3: "Attack & Debuff", 4: "Harden"},
    "move_ids": {"SLAM": 1, "ACTIVATE": 2, "ATTACK_DEBUFF": 3, "HARDEN": 4},
    "passives": ["barricade", "artifact"],
    "artifact_amount": 3,
}

BANDIT_BEAR_DATA = {
    "id": "BanditBear",
    "name": "Bear",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (38, 42),
        "a7": (40, 44),
    },
    "damage": {
        "maul": {"base": 18, "a2": 20},
        "bear_hug": {"base": 8, "a2": 10},
        "lunge": {"base": 9, "a2": 10},
    },
    "moves": {1: "Maul", 2: "Bear Hug", 3: "Lunge"},
    "move_ids": {"MAUL": 1, "BEAR_HUG": 2, "LUNGE": 3},
    "passives": [],
}

BANDIT_LEADER_DATA = {
    "id": "BanditLeader",
    "name": "Romeo",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (62, 66),
        "a7": (64, 68),
    },
    "damage": {
        "mock": {"base": 0},  # Debuff only
        "agonizing_slash": {"base": 10, "a2": 12},
    },
    "moves": {1: "Mock", 2: "Agonizing Slash", 3: "Shiv"},
    "move_ids": {"MOCK": 1, "AGONIZING_SLASH": 2, "SHIV": 3},
    "passives": [],
}

BANDIT_POINTY_DATA = {
    "id": "BanditChild",
    "name": "Pointy",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (30, 34),
        "a7": (32, 36),
    },
    "damage": {
        "attack": {"base": 5, "a2": 6},
    },
    "moves": {1: "Attack"},
    "move_ids": {"ATTACK": 1},
    "passives": [],
}


# =============================================================================
# CITY (ACT 2) - ELITES
# =============================================================================

GREMLIN_LEADER_DATA = {
    "id": "GremlinLeader",
    "name": "Gremlin Leader",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (140, 148),
        "a8": (145, 155),
    },
    "damage": {
        "encourage_str": {"base": 3, "a3": 4, "a18": 5},
        "encourage_block": {"base": 6, "a18": 10},
        "stab": {"base": 6},
        "stab_hits": {"base": 3},
    },
    "moves": {1: "Encourage", 2: "Rally", 3: "Stab"},
    "move_ids": {"ENCOURAGE": 1, "RALLY": 2, "STAB": 3},
    "passives": [],
}

BOOK_OF_STABBING_DATA = {
    "id": "BookOfStabbing",
    "name": "Book of Stabbing",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (160, 164),
        "a8": (168, 172),
    },
    "damage": {
        "multi_stab": {"base": 6, "a3": 7},
        "single_stab": {"base": 21, "a3": 24},
    },
    "moves": {1: "Multi-Stab", 2: "Single Stab"},
    "move_ids": {"MULTI_STAB": 1, "SINGLE_STAB": 2},
    "passives": ["painful_stabs"],  # Add wound on unblocked damage
}

TASKMASTER_DATA = {
    "id": "SlaverBoss",
    "name": "Taskmaster",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (54, 60),
        "a8": (57, 64),
    },
    "damage": {
        "whip": {"base": 7},
        "wounds": {"base": 1, "a3": 2, "a18": 3},
    },
    "moves": {1: "Scouring Whip"},
    "move_ids": {"SCOURING_WHIP": 1},
    "passives": [],
    "gains_strength": {"a18": 1},  # A18+: gains 1 str per turn
}


# =============================================================================
# CITY (ACT 2) - BOSSES
# =============================================================================

CHAMP_DATA = {
    "id": "Champ",
    "name": "The Champ",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (420, 420),
        "a9": (440, 440),
    },
    "damage": {
        "slash": {"base": 16, "a4": 18},
        "execute": {"base": 10},
        "execute_hits": {"base": 2},
        "slap": {"base": 12, "a4": 14},
        "strength": {"base": 2, "a4": 3, "a19": 4},
        "anger_str": {"base": 6, "a4": 9, "a19": 12},
        "forge": {"base": 5, "a9": 6, "a19": 7},
        "block": {"base": 15, "a9": 18, "a19": 20},
    },
    "moves": {
        1: "Heavy Slash", 2: "Defensive Stance", 3: "Gloat",
        4: "Taunt", 5: "Execute", 6: "Face Slap", 7: "Anger"
    },
    "move_ids": {
        "HEAVY_SLASH": 1, "DEFENSIVE_STANCE": 2, "GLOAT": 3,
        "TAUNT": 4, "EXECUTE": 5, "FACE_SLAP": 6, "ANGER": 7
    },
    "passives": [],
}

THE_COLLECTOR_DATA = {
    "id": "TheCollector",
    "name": "The Collector",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (282, 282),
        "a9": (300, 300),
    },
    "damage": {
        "fireball": {"base": 18, "a4": 21},
        "strength": {"base": 3, "a4": 4, "a19": 5},
        "block": {"base": 15, "a9": 18},
        "mega_debuff": {"base": 3, "a19": 5},
    },
    "moves": {
        1: "Spawn", 2: "Fireball", 3: "Buff", 4: "Mega Debuff", 5: "Revive"
    },
    "move_ids": {
        "SPAWN": 1, "FIREBALL": 2, "BUFF": 3, "MEGA_DEBUFF": 4, "REVIVE": 5
    },
    "passives": [],
}

BRONZE_AUTOMATON_DATA = {
    "id": "BronzeAutomaton",
    "name": "Bronze Automaton",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (300, 300),
        "a9": (320, 320),
    },
    "damage": {
        "flail": {"base": 7, "a4": 8},
        "flail_hits": {"base": 2},
        "beam": {"base": 45, "a4": 50},
        "strength": {"base": 3, "a4": 4},
        "block": {"base": 9, "a9": 12},
    },
    "moves": {
        1: "Flail", 2: "Hyper Beam", 3: "Stunned", 4: "Spawn Orbs", 5: "Boost"
    },
    "move_ids": {
        "FLAIL": 1, "HYPER_BEAM": 2, "STUNNED": 3, "SPAWN_ORBS": 4, "BOOST": 5
    },
    "passives": ["artifact"],
    "artifact_amount": 3,
}


# =============================================================================
# BEYOND (ACT 3) - BASIC ENEMIES
# =============================================================================

MAW_DATA = {
    "id": "Maw",
    "name": "Maw",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (300, 300),
        "a7": (300, 300),
    },
    "damage": {
        "roar": {"base": 0},  # Debuff only
        "drool": {"base": 0},  # Status cards
        "slam": {"base": 25, "a2": 30},
        "nom": {"base": 5, "a2": 5},  # NOM_DMG (healing attack)
    },
    "moves": {1: "Roar", 2: "Drool", 3: "Slam", 4: "Nom"},
    "move_ids": {"ROAR": 1, "DROOL": 2, "SLAM": 3, "NOM": 4},
    "passives": [],
}

DARKLING_DATA = {
    "id": "Darkling",
    "name": "Darkling",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (48, 56),
        "a7": (50, 59),
    },
    "damage": {
        "chomp": {"base": 8, "a2": 9},
        "chomp_hits": {"base": 2},
        "nip_min": {"base": 7, "a2": 9},
        "nip_max": {"base": 11, "a2": 13},
        "harden_block": {"base": 12},
        "harden_str": {"a17": 2},  # Only at A17
    },
    "moves": {1: "Chomp", 2: "Harden", 3: "Nip", 4: "Reincarnate"},
    "move_ids": {"CHOMP": 1, "HARDEN": 2, "NIP": 3, "REINCARNATE": 4},
    "passives": ["regrow"],
}

ORB_WALKER_DATA = {
    "id": "Orb Walker",
    "name": "Orb Walker",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (90, 96),
        "a7": (92, 98),
    },
    "damage": {
        "claw": {"base": 15, "a2": 16},
        "laser": {"base": 10, "a2": 11},
        "burn": {"base": 1, "a17": 2},  # Number of burns
    },
    "moves": {1: "Claw", 2: "Laser"},
    "move_ids": {"CLAW": 1, "LASER": 2},
    "passives": [],
}

SPIKER_DATA = {
    "id": "Spiker",
    "name": "Spiker",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (42, 56),
        "a7": (44, 58),
    },
    "damage": {
        "cut": {"base": 7, "a2": 9},
        "spike": {"base": 2, "a17": 3},  # Thorns amount
    },
    "moves": {1: "Cut", 2: "Spike"},
    "move_ids": {"CUT": 1, "SPIKE": 2},
    "passives": ["thorns"],
    "thorns_amount": {"base": 3, "a17": 4},
}

REPULSOR_DATA = {
    "id": "Repulsor",
    "name": "Repulsor",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (29, 35),
        "a7": (31, 37),
    },
    "damage": {
        "bash": {"base": 11, "a2": 13},
        "dazed": {"base": 2, "a17": 3},
    },
    "moves": {1: "Bash", 2: "Repulse"},
    "move_ids": {"BASH": 1, "REPULSE": 2},
    "passives": [],
}

WRITHING_MASS_DATA = {
    "id": "WrithingMass",
    "name": "Writhing Mass",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (160, 160),
        "a7": (175, 175),
    },
    "damage": {
        "strong_hit": {"base": 32, "a2": 38},
        "multi_strike": {"base": 7, "a2": 9},
        "multi_strike_hits": {"base": 3},
        "flail": {"base": 15, "a2": 16},
        "wither": {"base": 10, "a2": 12},
    },
    "moves": {1: "Strong Hit", 2: "Multi-Strike", 3: "Flail", 4: "Wither", 5: "Implant"},
    "move_ids": {
        "STRONG_HIT": 1, "MULTI_STRIKE": 2, "FLAIL": 3, "WITHER": 4, "IMPLANT": 5
    },
    "passives": ["reactive", "malleable"],
}

TRANSIENT_DATA = {
    "id": "Transient",
    "name": "Transient",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (999, 999),
    },
    "damage": {
        "attack": {"base": 30, "a2": 40},
        "attack_increment": {"base": 10},  # +10 each turn
    },
    "moves": {1: "Attack"},
    "move_ids": {"ATTACK": 1},
    "passives": ["fading"],  # Dies after 5 turns, escapes if low HP
    "fading_turns": 5,
}

EXPLODER_DATA = {
    "id": "Exploder",
    "name": "Exploder",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (30, 35),
        "a7": (32, 38),
    },
    "damage": {
        "slam": {"base": 9, "a2": 11},
        "explode": {"base": 30},
    },
    "moves": {1: "Slam", 2: "Explode"},
    "move_ids": {"SLAM": 1, "EXPLODE": 2},
    "passives": [],
}

SPIRE_GROWTH_DATA = {
    "id": "Serpent",
    "name": "Spire Growth",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (170, 190),
        "a7": (178, 198),
    },
    "damage": {
        "quick_tackle": {"base": 16, "a2": 18},
        "smash": {"base": 22, "a2": 25},
        "constrict": {"base": 10, "a2": 12},
    },
    "moves": {1: "Quick Tackle", 2: "Smash", 3: "Constrict"},
    "move_ids": {"QUICK_TACKLE": 1, "SMASH": 2, "CONSTRICT": 3},
    "passives": [],
}

SNAKE_DAGGER_DATA = {
    "id": "Dagger",
    "name": "Snake Dagger",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 25),
        "a8": (22, 27),
    },
    "damage": {
        "stab": {"base": 9, "a3": 10},
        "explode": {"base": 25},
    },
    "moves": {1: "Stab", 2: "Explode"},
    "move_ids": {"STAB": 1, "EXPLODE": 2},
    "passives": ["minion"],
}


# =============================================================================
# BEYOND (ACT 3) - ELITES
# =============================================================================

GIANT_HEAD_DATA = {
    "id": "GiantHead",
    "name": "Giant Head",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (500, 500),
        "a8": (520, 520),
    },
    "damage": {
        "count": {"base": 13},
        "it_is_time": {"base": 30, "a3": 40},
        "it_is_time_increment": {"base": 5},
    },
    "moves": {1: "Count", 2: "Glare", 3: "It Is Time"},
    "move_ids": {"COUNT": 1, "GLARE": 2, "IT_IS_TIME": 3},
    "passives": ["slow"],  # First card played each turn costs +1
    "countdown": {"base": 5, "a18": 4},
}

NEMESIS_DATA = {
    "id": "Nemesis",
    "name": "Nemesis",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (185, 185),
        "a8": (200, 200),
    },
    "damage": {
        "scythe": {"base": 45},
        "fire": {"base": 6, "a3": 7},
        "fire_hits": {"base": 3},
        "burn": {"base": 3, "a18": 5},  # Number of burns
    },
    "moves": {1: "Scythe", 2: "Tri Attack", 3: "Burn"},
    "move_ids": {"SCYTHE": 1, "TRI_ATTACK": 2, "TRI_BURN": 3},
    "passives": ["intangible"],  # Gains intangible at end of turn
}

REPTOMANCER_DATA = {
    "id": "Reptomancer",
    "name": "Reptomancer",
    "type": EnemyType.ELITE,
    "hp": {
        "base": (180, 190),
        "a8": (190, 200),
    },
    "damage": {
        "snake_strike": {"base": 13, "a3": 16},
        "snake_strike_hits": {"base": 2},
        "big_bite": {"base": 30, "a3": 34},
    },
    "moves": {1: "Snake Strike", 2: "Summon", 3: "Big Bite"},
    "move_ids": {"SNAKE_STRIKE": 1, "SPAWN_DAGGER": 2, "BIG_BITE": 3},
    "passives": [],
    "daggers_per_spawn": {"base": 1, "a18": 2},
}


# =============================================================================
# BEYOND (ACT 3) - BOSSES
# =============================================================================

AWAKENED_ONE_DATA = {
    "id": "AwakenedOne",
    "name": "Awakened One",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (300, 300),
        "a9": (320, 320),
    },
    "damage": {
        # Phase 1
        "slash": {"base": 20},
        "soul_strike": {"base": 6},
        "soul_strike_hits": {"base": 4},
        # Phase 2
        "dark_echo": {"base": 40},
        "sludge": {"base": 18},
        "tackle": {"base": 10},
        "tackle_hits": {"base": 3},
    },
    "moves": {
        1: "Slash", 2: "Soul Strike", 3: "Rebirth",
        5: "Dark Echo", 6: "Sludge", 8: "Tackle"
    },
    "move_ids": {
        "SLASH": 1, "SOUL_STRIKE": 2, "REBIRTH": 3,
        "DARK_ECHO": 5, "SLUDGE": 6, "TACKLE": 8
    },
    "passives": ["curiosity", "regenerate"],
    "curiosity_str": {"base": 1, "a19": 2},
    "regenerate_hp": {"base": 10, "a19": 15},
    "starting_strength": {"a4": 2},
}

TIME_EATER_DATA = {
    "id": "TimeEater",
    "name": "Time Eater",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (456, 456),
        "a9": (480, 480),
    },
    "damage": {
        "reverberate": {"base": 7, "a4": 8},
        "reverberate_hits": {"base": 3},
        "head_slam": {"base": 26, "a4": 32},
        "ripple_block": {"base": 20},
    },
    "moves": {1: "Reverberate", 2: "Head Slam", 3: "Ripple", 4: "Haste"},
    "move_ids": {"REVERBERATE": 1, "HEAD_SLAM": 2, "RIPPLE": 3, "HASTE": 4},
    "passives": ["time_warp"],  # Ends turn after 12 cards, gains 2 str
    "cards_until_warp": 12,
}

DONU_DATA = {
    "id": "Donu",
    "name": "Donu",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (265, 265),
    },
    "damage": {
        "beam": {"base": 10, "a4": 12},
        "beam_hits": {"base": 2},
        "circle_str": {"base": 3},
    },
    "moves": {1: "Circle of Power", 2: "Beam"},
    "move_ids": {"CIRCLE": 1, "BEAM": 2},
    "passives": ["artifact"],
    "artifact_amount": {"base": 2, "a19": 3},
}

DECA_DATA = {
    "id": "Deca",
    "name": "Deca",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (250, 250),
        "a9": (265, 265),
    },
    "damage": {
        "beam": {"base": 10, "a4": 12},
        "beam_hits": {"base": 2},
        "square_block": {"base": 16},
        "plated_armor": {"a19": 3},  # Only at A19
    },
    "moves": {1: "Square of Protection", 2: "Beam"},
    "move_ids": {"SQUARE": 1, "BEAM": 2},
    "passives": ["artifact"],
    "artifact_amount": {"base": 2, "a19": 3},
}


# =============================================================================
# ACT 4 - ENDING
# =============================================================================

SPIRE_SHIELD_DATA = {
    "id": "SpireShield",
    "name": "Spire Shield",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (110, 110),
        "a9": (120, 120),
    },
    "damage": {
        "bash": {"base": 12, "a4": 14},
        "fortify_block": {"base": 30},
        "smash": {"base": 34, "a4": 38},
    },
    "moves": {1: "Bash", 2: "Fortify", 3: "Smash"},
    "move_ids": {"BASH": 1, "FORTIFY": 2, "SMASH": 3},
    "passives": ["artifact"],
    "artifact_amount": 1,
}

SPIRE_SPEAR_DATA = {
    "id": "SpireSpear",
    "name": "Spire Spear",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (160, 160),
        "a9": (180, 180),
    },
    "damage": {
        "burn_strike": {"base": 5, "a4": 6},
        "burn_strike_hits": {"base": 2},
        "skewer": {"base": 10, "a4": 11},
        "skewer_hits": {"base": 3},
    },
    "moves": {1: "Burn Strike", 2: "Piercer", 3: "Skewer"},
    "move_ids": {"BURN_STRIKE": 1, "PIERCER": 2, "SKEWER": 3},
    "passives": ["artifact"],
    "artifact_amount": 1,
}

CORRUPT_HEART_DATA = {
    "id": "CorruptHeart",
    "name": "Corrupt Heart",
    "type": EnemyType.BOSS,
    "hp": {
        "base": (750, 750),
        "a9": (800, 800),
    },
    "damage": {
        "blood_shots": {"base": 2},
        "blood_shots_hits": {"base": 12, "a4": 15},
        "echo": {"base": 40, "a4": 45},
        "buff_str": {"base": 2},
    },
    "moves": {1: "Debilitate", 2: "Blood Shots", 3: "Echo", 4: "Buff"},
    "move_ids": {"DEBILITATE": 1, "BLOOD_SHOTS": 2, "ECHO": 3, "BUFF": 4},
    "passives": ["invincible", "beat_of_death"],
    "invincible_threshold": {"base": 300, "a19": 200},
    "beat_of_death": {"base": 1, "a19": 2},
}


# =============================================================================
# MINIONS / SUMMONS
# =============================================================================

TORCH_HEAD_DATA = {
    "id": "TorchHead",
    "name": "Torch Head",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (38, 40),
        "a9": (40, 44),
    },
    "damage": {
        "tackle": {"base": 7},
    },
    "moves": {1: "Tackle"},
    "move_ids": {"TACKLE": 1},
    "passives": ["minion"],
}

BRONZE_ORB_DATA = {
    "id": "BronzeOrb",
    "name": "Bronze Orb",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (52, 58),
        "a9": (54, 60),
    },
    "damage": {
        "beam": {"base": 8},
    },
    "moves": {1: "Stasis", 2: "Beam", 3: "Support Beam"},
    "move_ids": {"STASIS": 1, "BEAM": 2, "SUPPORT_BEAM": 3},
    "passives": ["minion"],
}

GREMLIN_FAT_DATA = {
    "id": "GremlinFat",
    "name": "Fat Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (13, 17),
        "a8": (14, 18),
    },
    "damage": {
        "smash": {"base": 4, "a3": 5},
        "weak": {"base": 1, "a18": 2},
    },
    "moves": {1: "Smash"},
    "move_ids": {"SMASH": 1},
    "passives": [],
}

GREMLIN_THIEF_DATA = {
    "id": "GremlinThief",
    "name": "Sneaky Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (10, 14),
        "a8": (11, 15),
    },
    "damage": {
        "puncture": {"base": 9, "a3": 10},
    },
    "moves": {1: "Puncture"},
    "move_ids": {"PUNCTURE": 1},
    "passives": [],
}

GREMLIN_TSUNDERE_DATA = {
    "id": "GremlinTsundere",
    "name": "Shield Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (12, 15),
        "a8": (13, 16),
    },
    "damage": {
        "protect": {"base": 7, "a18": 11},  # Block amount
        "shield_bash": {"base": 6, "a3": 8},
    },
    "moves": {1: "Protect", 2: "Shield Bash"},
    "move_ids": {"PROTECT": 1, "SHIELD_BASH": 2},
    "passives": [],
}

GREMLIN_WARRIOR_DATA = {
    "id": "GremlinWarrior",
    "name": "Mad Gremlin",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (20, 24),
        "a8": (21, 25),
    },
    "damage": {
        "scratch": {"base": 4, "a3": 5},
    },
    "moves": {1: "Scratch"},
    "move_ids": {"SCRATCH": 1},
    "passives": ["angry"],  # Gains 1 str when damaged
    "angry_str": {"base": 1, "a18": 2},
}

GREMLIN_WIZARD_DATA = {
    "id": "GremlinWizard",
    "name": "Gremlin Wizard",
    "type": EnemyType.NORMAL,
    "hp": {
        "base": (22, 26),
        "a8": (23, 27),
    },
    "damage": {
        "ultimate_blast": {"base": 25},
    },
    "moves": {1: "Charging", 2: "Ultimate Blast"},
    "move_ids": {"CHARGING": 1, "ULTIMATE_BLAST": 2},
    "passives": [],
    "charge_turns": 3,
}


# =============================================================================
# ENEMY DATA REGISTRY
# =============================================================================

ENEMY_DATA: Dict[str, Dict[str, Any]] = {
    # Exordium Basic
    "JawWorm": JAW_WORM_DATA,
    "Cultist": CULTIST_DATA,
    "AcidSlime_M": ACID_SLIME_M_DATA,
    "AcidSlime_L": ACID_SLIME_L_DATA,
    "AcidSlime_S": ACID_SLIME_S_DATA,
    "SpikeSlime_M": SPIKE_SLIME_M_DATA,
    "SpikeSlime_L": SPIKE_SLIME_L_DATA,
    "SpikeSlime_S": SPIKE_SLIME_S_DATA,
    "Louse": LOUSE_DATA,
    "FuzzyLouseNormal": LOUSE_NORMAL_DATA,
    "FuzzyLouseDefensive": LOUSE_DEFENSIVE_DATA,
    "FungiBeast": FUNGI_BEAST_DATA,
    "Looter": LOOTER_DATA,
    "Mugger": MUGGER_DATA,
    "SlaverBlue": SLAVER_BLUE_DATA,
    "SlaverRed": SLAVER_RED_DATA,
    # Exordium Elites
    "GremlinNob": GREMLIN_NOB_DATA,
    "Lagavulin": LAGAVULIN_DATA,
    "Sentry": SENTRY_DATA,
    # Exordium Bosses
    "SlimeBoss": SLIME_BOSS_DATA,
    "TheGuardian": THE_GUARDIAN_DATA,
    "Hexaghost": HEXAGHOST_DATA,
    # City Basic
    "Chosen": CHOSEN_DATA,
    "Byrd": BYRD_DATA,
    "Centurion": CENTURION_DATA,
    "Healer": HEALER_DATA,
    "Snecko": SNECKO_DATA,
    "SnakePlant": SNAKE_PLANT_DATA,
    "Shelled Parasite": SHELLED_PARASITE_DATA,
    "SphericGuardian": SPHERIC_GUARDIAN_DATA,
    "BanditBear": BANDIT_BEAR_DATA,
    "BanditLeader": BANDIT_LEADER_DATA,
    "BanditChild": BANDIT_POINTY_DATA,
    # City Elites
    "GremlinLeader": GREMLIN_LEADER_DATA,
    "BookOfStabbing": BOOK_OF_STABBING_DATA,
    "SlaverBoss": TASKMASTER_DATA,
    "Taskmaster": TASKMASTER_DATA,
    # City Bosses
    "Champ": CHAMP_DATA,
    "TheCollector": THE_COLLECTOR_DATA,
    "BronzeAutomaton": BRONZE_AUTOMATON_DATA,
    # Beyond Basic
    "Maw": MAW_DATA,
    "Darkling": DARKLING_DATA,
    "Orb Walker": ORB_WALKER_DATA,
    "Spiker": SPIKER_DATA,
    "Repulsor": REPULSOR_DATA,
    "WrithingMass": WRITHING_MASS_DATA,
    "Transient": TRANSIENT_DATA,
    "Exploder": EXPLODER_DATA,
    "Serpent": SPIRE_GROWTH_DATA,
    "Dagger": SNAKE_DAGGER_DATA,
    # Beyond Elites
    "GiantHead": GIANT_HEAD_DATA,
    "Nemesis": NEMESIS_DATA,
    "Reptomancer": REPTOMANCER_DATA,
    # Beyond Bosses
    "AwakenedOne": AWAKENED_ONE_DATA,
    "TimeEater": TIME_EATER_DATA,
    "Donu": DONU_DATA,
    "Deca": DECA_DATA,
    # Act 4
    "SpireShield": SPIRE_SHIELD_DATA,
    "SpireSpear": SPIRE_SPEAR_DATA,
    "CorruptHeart": CORRUPT_HEART_DATA,
    # Minions
    "TorchHead": TORCH_HEAD_DATA,
    "BronzeOrb": BRONZE_ORB_DATA,
    "GremlinFat": GREMLIN_FAT_DATA,
    "GremlinThief": GREMLIN_THIEF_DATA,
    "GremlinTsundere": GREMLIN_TSUNDERE_DATA,
    "GremlinWarrior": GREMLIN_WARRIOR_DATA,
    "GremlinWizard": GREMLIN_WIZARD_DATA,
}


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================

def get_hp_range(enemy_id: str, ascension: int = 0) -> Tuple[int, int]:
    """Get HP range for an enemy at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return (1, 1)

    hp_data = data.get("hp", {})
    result = hp_data.get("base", (1, 1))

    # Check ascension thresholds (sorted numerically, not alphabetically)
    asc_keys = [k for k in hp_data.keys() if k.startswith("a")]
    for key in sorted(asc_keys, key=lambda x: int(x[1:])):
        threshold = int(key[1:])
        if ascension >= threshold:
            result = hp_data[key]

    return result


def get_damage_value(enemy_id: str, move_key: str, ascension: int = 0) -> int:
    """Get damage value for an enemy move at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return 0

    damage_data = data.get("damage", {}).get(move_key, {})
    if isinstance(damage_data, int):
        return damage_data

    result = damage_data.get("base", 0)

    # Check ascension thresholds (sorted numerically, not alphabetically)
    asc_keys = [k for k in damage_data.keys() if k.startswith("a")]
    for key in sorted(asc_keys, key=lambda x: int(x[1:])):
        threshold = int(key[1:])
        if ascension >= threshold:
            result = damage_data[key]

    return result


def get_damage_values(enemy_id: str, ascension: int = 0) -> Dict[str, int]:
    """Get all damage values for an enemy at given ascension level."""
    data = ENEMY_DATA.get(enemy_id)
    if not data:
        return {}

    result = {}
    for move_key in data.get("damage", {}).keys():
        result[move_key] = get_damage_value(enemy_id, move_key, ascension)

    return result


def get_enemy_type(enemy_id: str) -> EnemyType:
    """Get the type of an enemy."""
    data = ENEMY_DATA.get(enemy_id)
    if data:
        return data.get("type", EnemyType.NORMAL)
    return EnemyType.NORMAL


def get_move_name(enemy_id: str, move_id: int) -> str:
    """Get the name of a move by ID."""
    data = ENEMY_DATA.get(enemy_id)
    if data:
        return data.get("moves", {}).get(move_id, "Unknown")
    return "Unknown"
