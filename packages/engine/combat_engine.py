"""
Combat Engine - Complete combat system for Slay the Spire.

This module provides a comprehensive combat engine that handles:
1. Complete turn flow (start -> player actions -> enemy turns -> end of round)
2. Full card execution with all effects
3. Damage calculation with all modifiers (strength, weak, vulnerable, stances)
4. Power/status effect system with proper triggering
5. Watcher stance mechanics (Neutral, Calm, Wrath, Divinity)
6. Enemy AI integration

Design principles:
- State is mutable for performance (use copy() for tree search)
- Full game mechanic accuracy from decompiled source
- Comprehensive logging for EV tracking
- Integration with GameRunner

Usage:
    from core.combat_engine import CombatEngine, create_combat_from_enemies

    engine = CombatEngine(combat_state)
    engine.start_combat()

    while not engine.is_combat_over():
        actions = engine.get_legal_actions()
        engine.execute_action(actions[0])

    result = engine.get_result()
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Union, Any, Callable
from enum import Enum
import copy

from .state.rng import Random, XorShift128
from .state.combat import (
    CombatState,
    EnemyCombatState,
    EntityState,
    PlayCard,
    UsePotion,
    EndTurn,
    Action,
    create_player,
    create_enemy,
    create_combat,
)
from .content.cards import Card, CardType, CardTarget, CardColor, get_card, ALL_CARDS
from .content.enemies import Enemy, Intent, MoveInfo, EnemyState, create_enemy as create_enemy_object
from .content.stances import StanceID, StanceEffect, STANCES, StanceManager
from .content.powers import PowerType, DamageType, create_power, POWER_DATA, resolve_power_id
from .calc.damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    WEAK_MULT,
    VULN_MULT,
    FRAIL_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
)
from .registry import execute_relic_triggers, execute_power_triggers, execute_potion_effect
from .effects.orbs import trigger_orb_start_of_turn


# =============================================================================
# COMBAT PHASE
# =============================================================================

class CombatPhase(Enum):
    """Current phase of combat."""
    NOT_STARTED = "NOT_STARTED"
    PLAYER_TURN = "PLAYER_TURN"
    ENEMY_TURN = "ENEMY_TURN"
    COMBAT_OVER = "COMBAT_OVER"


# =============================================================================
# COMBAT RESULT
# =============================================================================

@dataclass
class CombatResult:
    """Result of a completed combat."""
    victory: bool
    hp_remaining: int
    hp_lost: int
    turns: int
    cards_played: int
    damage_dealt: int
    damage_taken: int
    gold_earned: int = 0

    # Detailed tracking
    cards_played_sequence: List[str] = field(default_factory=list)
    stance_changes: int = 0
    energy_spent: int = 0


# =============================================================================
# COMBAT LOG
# =============================================================================

@dataclass
class CombatLogEntry:
    """A single combat log entry."""
    turn: int
    event_type: str
    data: Dict[str, Any]


@dataclass
class CombatLog:
    """Combat event log for EV tracking and replay."""
    entries: List[CombatLogEntry] = field(default_factory=list)

    def log(self, turn: int, event_type: str, **data):
        """Add a log entry."""
        self.entries.append(CombatLogEntry(turn=turn, event_type=event_type, data=data))

    def get_events(self, event_type: str) -> List[CombatLogEntry]:
        """Get all events of a specific type."""
        return [e for e in self.entries if e.event_type == event_type]


# =============================================================================
# COMBAT ENGINE
# =============================================================================

class CombatEngine:
    """
    Complete combat engine for Slay the Spire.

    Handles all combat mechanics including:
    - Turn flow
    - Card execution
    - Damage calculation
    - Powers and status effects
    - Enemy AI
    - Watcher stances
    """

    def __init__(
        self,
        state: CombatState,
        enemy_data: Optional[Dict[str, Enemy]] = None,
        shuffle_rng: Optional[Random] = None,
        card_rng: Optional[Random] = None,
        ai_rng: Optional[Random] = None,
    ):
        """
        Initialize combat engine.

        Args:
            state: Initial combat state
            enemy_data: Optional enemy definitions (for AI patterns)
            shuffle_rng: RNG for deck shuffling
            card_rng: RNG for card effects
            ai_rng: RNG for enemy AI
        """
        self.state = state
        # Registry handlers (e.g., Distilled Chaos) can reuse the live runtime engine.
        self.state._combat_engine_ref = self
        self.enemy_data = enemy_data or {}
        # Index-based enemy objects (parallel to state.enemies). Set by
        # create_combat_from_enemies or manually after construction.
        self.enemy_objects: List[Optional[Enemy]] = []

        # Initialize RNG from state or create new
        if shuffle_rng:
            self.shuffle_rng = shuffle_rng
        else:
            seed0, seed1 = state.shuffle_rng_state
            self.shuffle_rng = Random.__new__(Random)
            self.shuffle_rng._rng = XorShift128(seed0, seed1)
            self.shuffle_rng.counter = 0

        if card_rng:
            self.card_rng = card_rng
        else:
            seed0, seed1 = state.card_rng_state
            self.card_rng = Random.__new__(Random)
            self.card_rng._rng = XorShift128(seed0, seed1)
            self.card_rng.counter = 0

        if ai_rng:
            self.ai_rng = ai_rng
        else:
            seed0, seed1 = state.ai_rng_state
            self.ai_rng = Random.__new__(Random)
            self.ai_rng._rng = XorShift128(seed0, seed1)
            self.ai_rng.counter = 0

        # Combat phase
        self.phase = CombatPhase.NOT_STARTED

        # Combat log
        self.log = CombatLog()

        # Statistics
        self.initial_hp = state.player.hp
        self.cards_played_sequence: List[str] = []
        self.stance_changes = 0
        self.energy_spent = 0
        self._x_cost_amount = 0

    # =========================================================================
    # Core State Access
    # =========================================================================

    def copy(self) -> CombatEngine:
        """Create a copy of the combat engine with copied state."""
        new_engine = CombatEngine.__new__(CombatEngine)
        new_engine.state = self.state.copy()
        new_engine.state._combat_engine_ref = new_engine
        new_engine.enemy_data = self.enemy_data
        new_engine.enemy_objects = list(self.enemy_objects)
        new_engine.shuffle_rng = self.shuffle_rng.copy() if hasattr(self.shuffle_rng, 'copy') else self.shuffle_rng
        new_engine.card_rng = self.card_rng.copy() if hasattr(self.card_rng, 'copy') else self.card_rng
        new_engine.ai_rng = self.ai_rng.copy() if hasattr(self.ai_rng, 'copy') else self.ai_rng
        new_engine.phase = self.phase
        new_engine.log = CombatLog()  # Fresh log for copy
        new_engine.initial_hp = self.initial_hp
        new_engine.cards_played_sequence = self.cards_played_sequence.copy()
        new_engine.stance_changes = self.stance_changes
        new_engine.energy_spent = self.energy_spent
        new_engine._x_cost_amount = self._x_cost_amount
        return new_engine

    def get_living_enemies(self) -> List[EnemyCombatState]:
        """Get all living enemies."""
        return [e for e in self.state.enemies if e.hp > 0]

    def is_combat_over(self) -> bool:
        """Check if combat has ended."""
        return self.state.combat_over

    def is_victory(self) -> bool:
        """Check if player won."""
        return self.state.player_won

    def is_defeat(self) -> bool:
        """Check if player died."""
        return self.state.player.hp <= 0

    # =========================================================================
    # Combat Flow
    # =========================================================================

    def start_combat(self):
        """Initialize and start combat."""
        if self.phase != CombatPhase.NOT_STARTED:
            return

        self.log.log(0, "combat_start",
                    player_hp=self.state.player.hp,
                    enemies=[e.name for e in self.state.enemies])

        # Shuffle draw pile
        self._shuffle_draw_pile()

        # Move Innate cards to top of draw pile so they are drawn first
        innate_cards = []
        non_innate_cards = []
        for card_id in self.state.draw_pile:
            card = self._get_card(card_id)
            if card.innate:
                innate_cards.append(card_id)
            else:
                non_innate_cards.append(card_id)
        if innate_cards:
            self.state.draw_pile = non_innate_cards + innate_cards  # top = end of list

        # Roll initial moves for all enemies
        for enemy in self.state.enemies:
            self._roll_enemy_move(enemy)

        # Execute registry-based atBattleStart triggers (Vajra, Lantern, etc.)
        execute_relic_triggers("atBattleStart", self.state)

        # Execute atBattleStartPreDraw triggers (Pure Water adds Miracle)
        execute_relic_triggers("atBattleStartPreDraw", self.state)

        # Start first turn
        self._start_player_turn()

    def _start_player_turn(self):
        """Begin a player turn."""
        self.state.turn += 1
        self.phase = CombatPhase.PLAYER_TURN

        # Reset energy
        self.state.energy = self.state.max_energy

        # Lose block (unless Barricade/Blur/Calipers)
        if not self._has_barricade():
            blur = self.state.player.statuses.get("Blur", 0)
            if blur > 0:
                # Blur: retain all block, decrement Blur
                blur -= 1
                if blur <= 0:
                    del self.state.player.statuses["Blur"]
                else:
                    self.state.player.statuses["Blur"] = blur
            elif self._has_calipers():
                # Calipers: lose only 15 block instead of all
                self.state.player.block = max(0, self.state.player.block - 15)
            else:
                self.state.player.block = 0

        # Divinity auto-exit at start of turn (Java: DivinityStance.atStartOfTurn)
        # Must happen after block reset so Mental Fortress block from exit persists
        if self._get_stance() == StanceID.DIVINITY:
            self._change_stance(StanceID.NEUTRAL)

        # Reset turn counters
        self.state.cards_played_this_turn = 0
        self.state.attacks_played_this_turn = 0
        self.state.skills_played_this_turn = 0
        self.state.powers_played_this_turn = 0
        self.state.last_card_type = ""
        self.state.discarded_this_turn = 0

        # Execute registry-based atTurnStart relic triggers
        execute_relic_triggers("atTurnStart", self.state)

        # Trigger start of turn powers (inline and registry)
        self._trigger_start_of_turn()

        # Check for death from poison or other start-of-turn effects
        if self.state.player.hp <= 0:
            self.state.player.hp = 0
            if not self._check_fairy_in_bottle():
                self._end_combat(player_won=False)
                return

        # Execute onEnergyRecharge power triggers (DevaForm, Energized)
        execute_power_triggers("onEnergyRecharge", self.state, self.state.player)

        # Draw cards (default 5)
        draw_count = 5
        no_draw = (
            self.state.player.statuses.get("NoDraw", 0)
            or self.state.player.statuses.get("No Draw", 0)
        )
        if no_draw > 0:
            draw_count = 0
        self._draw_cards(draw_count)

        # Trigger post-draw hooks (Java: applyStartOfTurnPostDrawRelics/Powers).
        execute_relic_triggers("atTurnStartPostDraw", self.state)
        execute_power_triggers("atStartOfTurnPostDraw", self.state, self.state.player)

        self.log.log(self.state.turn, "turn_start",
                    energy=self.state.energy,
                    hand=[c for c in self.state.hand])

    def _trigger_start_of_turn(self):
        """Trigger all start-of-turn effects using registry and inline logic."""
        player = self.state.player

        # Execute atStartOfTurn power triggers for player (handles Poison via registry)
        execute_power_triggers("atStartOfTurn", self.state, player)

        # Note: Enemy atStartOfTurn (poison, etc.) is handled in _do_enemy_turns() at their turn start

        # Wrath Next Turn
        if player.statuses.get("WrathNextTurn", 0) > 0:
            self._change_stance(StanceID.WRATH)
            del player.statuses["WrathNextTurn"]

        # Defect orb passives trigger at start of turn; Cables bonus is handled
        # by its dedicated relic trigger.
        trigger_orb_start_of_turn(self.state, include_cables=False)

    def start_turn(self):
        """Public interface for starting a new player turn."""
        self._start_player_turn()

    def end_turn(self):
        """End the player's turn."""
        if self.phase != CombatPhase.PLAYER_TURN:
            return

        self.log.log(self.state.turn, "turn_end",
                    cards_played=self.state.cards_played_this_turn)

        # Execute registry-based onPlayerEndTurn triggers
        execute_relic_triggers("onPlayerEndTurn", self.state)

        # Discard hand (unless Runic Pyramid)
        self._discard_hand()

        # Execute atEndOfTurnPreEndTurnCards power triggers (Metallicize, Plated Armor, Like Water)
        execute_power_triggers("atEndOfTurnPreEndTurnCards", self.state, self.state.player)

        # Like Water (inline fallback)
        like_water = self.state.player.statuses.get("LikeWater", 0)
        if like_water > 0 and self._get_stance() == StanceID.CALM:
            self.state.player.block += like_water

        # Metallicize (inline fallback)
        metallicize = self.state.player.statuses.get("Metallicize", 0)
        if metallicize > 0:
            self.state.player.block += metallicize

        # Plated Armor (inline fallback)
        plated = self.state.player.statuses.get("Plated Armor", 0)
        if plated > 0:
            self.state.player.block += plated

        # Study: add Insight to draw pile
        study = self.state.player.statuses.get("Study", 0)
        if study > 0:
            for _ in range(study):
                self.state.draw_pile.append("Insight")

        # Regen: heal at end of turn (Java: no decrement, persists entire combat)
        regen = self.state.player.statuses.get("Regen", 0)
        if regen > 0:
            heal = min(regen, self.state.player.max_hp - self.state.player.hp)
            self.state.player.hp += heal

        # Execute atEndOfTurn power triggers (Constricted, Combust, Ritual, etc.)
        execute_power_triggers("atEndOfTurn", self.state, self.state.player)
        for enemy in self.state.enemies:
            if not enemy.is_dead:
                execute_power_triggers("atEndOfTurn", self.state, enemy)

        # Process enemy turns
        self._do_enemy_turns()

        # Execute atEndOfRound power triggers (decrement Weak, Vulnerable, Frail)
        execute_power_triggers("atEndOfRound", self.state, self.state.player)
        for enemy_state in self.state.enemies:
            if not enemy_state.is_dead:
                execute_power_triggers("atEndOfRound", self.state, enemy_state)

        # Check combat end
        if not self._check_combat_end():
            # Start next player turn
            self._start_player_turn()

    def _discard_hand(self):
        """Discard all cards in hand."""
        retain_all = (
            self._has_runic_pyramid()
            or self.state.player.statuses.get("RetainHand", 0) > 0
            or self.state.player.statuses.get("Equilibrium", 0) > 0
        )
        if retain_all:
            # Full-hand retain effects still exhaust Ethereal cards.
            kept = []
            for card_id in self.state.hand:
                card = self._get_card(card_id)
                if card.ethereal:
                    self.state.exhaust_pile.append(card_id)
                else:
                    kept.append(card_id)
            self.state.hand = kept
            return

        retained = []
        for card_id in self.state.hand:
            card = self._get_card(card_id)
            if card.retain:
                retained.append(card_id)
            elif card.ethereal:
                self.state.exhaust_pile.append(card_id)
            else:
                self.state.discard_pile.append(card_id)
        self.state.hand = retained

    def _do_enemy_turns(self):
        """Execute all enemy turns."""
        self.phase = CombatPhase.ENEMY_TURN

        for enemy in self.state.enemies:
            if enemy.hp <= 0:
                continue

            # Block decays at start of each enemy's turn (per-enemy)
            enemy.block = 0

            # Metallicize: enemy gains block at start of turn
            metallicize = enemy.statuses.get("Metallicize", 0)
            if metallicize > 0:
                enemy.block += metallicize

            # Enemy poison tick
            enemy_poison = enemy.statuses.get("Poison", 0)
            if enemy_poison > 0:
                enemy.hp -= enemy_poison
                self.state.total_damage_dealt += enemy_poison
                self.log.log(self.state.turn, "enemy_poison", enemy=enemy.id, damage=enemy_poison)
                enemy_poison -= 1
                if enemy_poison <= 0:
                    del enemy.statuses["Poison"]
                else:
                    enemy.statuses["Poison"] = enemy_poison
                if enemy.hp <= 0:
                    enemy.hp = 0
                    self._on_enemy_death(enemy)
                    continue

            # Apply Ritual strength gain at start of enemy turn
            ritual = enemy.statuses.get("Ritual", 0)
            if ritual > 0 and not enemy.first_turn:
                enemy.statuses["Strength"] = enemy.statuses.get("Strength", 0) + ritual

            # Execute enemy's move
            self._execute_enemy_move(enemy)

            # Check if player died
            if self.state.player.hp <= 0:
                # Check for Fairy in a Bottle before ending combat
                if not self._check_fairy_in_bottle():
                    self._end_combat(player_won=False)
                    return

        # Decrement enemy debuffs
        for enemy in self.state.enemies:
            if enemy.hp <= 0:
                continue
            for debuff in ["Vulnerable", "Weak", "Frail"]:
                if enemy.statuses.get(debuff, 0) > 0:
                    enemy.statuses[debuff] -= 1
                    if enemy.statuses[debuff] <= 0:
                        del enemy.statuses[debuff]

        # Roll next moves
        for enemy in self.state.enemies:
            if enemy.hp > 0:
                self._roll_enemy_move(enemy)

        # Decrement player debuffs
        for debuff in ["Weak", "Vulnerable", "Frail"]:
            if self.state.player.statuses.get(debuff, 0) > 0:
                self.state.player.statuses[debuff] -= 1
                if self.state.player.statuses[debuff] <= 0:
                    del self.state.player.statuses[debuff]

    def _execute_enemy_move(self, enemy: EnemyCombatState):
        """Execute a single enemy's move."""
        if enemy.move_id == -1:
            return

        self.log.log(self.state.turn, "enemy_move",
                    enemy=enemy.name,
                    move_id=enemy.move_id,
                    damage=enemy.move_damage,
                    hits=enemy.move_hits)

        # Get enemy strength
        enemy_strength = enemy.statuses.get("Strength", 0)

        # Execute attack
        if enemy.move_damage > 0:
            # Use float math throughout, floor only at end (Java parity)
            base_damage = float(enemy.move_damage + enemy_strength)
            if enemy.statuses.get("Weak", 0) > 0:
                weak_mult = WEAK_MULT
                if "Paper Crane" in self.state.relics:
                    weak_mult = 0.60  # Paper Crane: 40% reduction
                base_damage *= weak_mult
            hits = enemy.move_hits

            # Apply stance multiplier for incoming damage
            stance_mult = 1.0
            if self._get_stance() == StanceID.WRATH:
                stance_mult = 2.0

            for _ in range(hits):
                damage_f = base_damage * stance_mult

                # Apply vulnerable
                if self.state.player.statuses.get("Vulnerable", 0) > 0:
                    damage_f *= VULN_MULT

                # Floor to int only at the end (Java parity)
                damage = max(0, int(damage_f))

                # Torii: reduce damage 2-5 to 1 (BEFORE Intangible)
                if "Torii" in self.state.relics and 2 <= damage <= 5:
                    damage = 1

                # Intangible: cap damage to 1
                if self.state.player.statuses.get("Intangible", 0) > 0 and damage > 1:
                    damage = 1

                # Dispatch damage modification hooks for parity coverage.
                execute_power_triggers(
                    "atDamageGive",
                    self.state,
                    enemy,
                    {"value": float(damage), "damage_type": "NORMAL"},
                )
                execute_power_triggers(
                    "atDamageReceive",
                    self.state,
                    self.state.player,
                    {"value": float(damage), "damage_type": "NORMAL"},
                )
                execute_power_triggers(
                    "atDamageFinalReceive",
                    self.state,
                    self.state.player,
                    {"value": float(damage), "damage_type": "NORMAL"},
                )

                # Power damage replacement hook (Buffer, Invincible, etc.)
                replaced_damage = execute_power_triggers(
                    "onAttackedToChangeDamage",
                    self.state,
                    self.state.player,
                    {"value": damage, "damage_type": "NORMAL"},
                )
                if replaced_damage is not None:
                    damage = max(0, int(replaced_damage))

                # Apply block
                blocked = min(self.state.player.block, damage)
                hp_damage = damage - blocked
                self.state.player.block -= blocked

                # Tungsten Rod: reduce all HP loss by 1 (minimum 0)
                if hp_damage > 0 and "Tungsten Rod" in self.state.relics:
                    hp_damage = max(0, hp_damage - 1)

                self.state.player.hp -= hp_damage
                self.state.total_damage_taken += hp_damage

                # Trigger wasHPLost hooks if HP was lost
                if hp_damage > 0:
                    execute_relic_triggers("wasHPLost", self.state, {"hp_lost": hp_damage})
                    execute_power_triggers(
                        "wasHPLost",
                        self.state,
                        self.state.player,
                        {
                            "damage": hp_damage,
                            "unblocked": True,
                            "is_self_damage": False,
                            "damage_type": "NORMAL",
                        },
                    )

                execute_power_triggers(
                    "onAttack",
                    self.state,
                    enemy,
                    {
                        "target": self.state.player,
                        "damage": damage,
                        "unblocked_damage": hp_damage,
                        "damage_type": "NORMAL",
                    },
                )
                execute_power_triggers(
                    "onAttacked",
                    self.state,
                    self.state.player,
                    {
                        "attacker": enemy,
                        "damage": damage,
                        "unblocked_damage": hp_damage,
                        "damage_type": "NORMAL",
                    },
                )

                self.log.log(self.state.turn, "player_damaged",
                            enemy=enemy.name,
                            damage=damage,
                            blocked=blocked,
                            hp_damage=hp_damage)

                if self.state.player.hp <= 0:
                    self.state.player.hp = 0
                    return

        # Apply effects first (to check for block_all_monsters)
        effects = enemy.move_effects
        has_block_all = effects and "block_all_monsters" in effects

        # Apply block to self (unless block_all_monsters will handle it)
        if enemy.move_block > 0 and not has_block_all:
            enemy.block += enemy.move_block

        # Apply effects
        if effects:
            if "strength" in effects:
                enemy.statuses["Strength"] = enemy.statuses.get("Strength", 0) + effects["strength"]
            if "weak" in effects:
                self._apply_debuff_to_player("Weak", effects["weak"])
            if "vulnerable" in effects:
                self._apply_debuff_to_player("Vulnerable", effects["vulnerable"])
            if "frail" in effects:
                self._apply_debuff_to_player("Frail", effects["frail"])
            if "ritual" in effects:
                enemy.statuses["Ritual"] = enemy.statuses.get("Ritual", 0) + effects["ritual"]
            if "player_strength" in effects:
                self.state.player.statuses["Strength"] = self.state.player.statuses.get("Strength", 0) + effects["player_strength"]
            if "player_dexterity" in effects:
                self.state.player.statuses["Dexterity"] = self.state.player.statuses.get("Dexterity", 0) + effects["player_dexterity"]
            if "poison" in effects:
                self.state.player.statuses["Poison"] = self.state.player.statuses.get("Poison", 0) + effects["poison"]
            if "metallicize" in effects:
                enemy.statuses["Metallicize"] = enemy.statuses.get("Metallicize", 0) + effects["metallicize"]
            if "plated_armor" in effects:
                enemy.statuses["Plated Armor"] = enemy.statuses.get("Plated Armor", 0) + effects["plated_armor"]

            # Status cards added to discard pile
            if "slimed" in effects:
                for _ in range(effects["slimed"]):
                    self.state.discard_pile.append("Slimed")
            if "daze" in effects:
                for _ in range(effects["daze"]):
                    self.state.discard_pile.append("Daze")
            if "burn" in effects:
                for _ in range(effects["burn"]):
                    self.state.discard_pile.append("Burn")
            if "wound" in effects:
                for _ in range(effects["wound"]):
                    self.state.discard_pile.append("Wound")
            if "void" in effects:
                for _ in range(effects["void"]):
                    self.state.discard_pile.append("Void")

            # Status cards added to draw pile (e.g. CorruptHeart Debilitate)
            if "status_cards" in effects:
                for card_id in effects["status_cards"]:
                    self.state.draw_pile.append(card_id)

            # Spire Shield Fortify: block to ALL monsters
            if "block_all_monsters" in effects:
                block_amount = effects["block_all_monsters"]
                for e in self.state.enemies:
                    if e.hp > 0:
                        e.block += block_amount
                self.log.log(self.state.turn, "block_all_monsters",
                           enemy=enemy.id, amount=block_amount)

            # Strength to ALL monsters (Donu Circle of Protection, SpireSpear Piercer)
            if "strength_all_monsters" in effects:
                str_amount = effects["strength_all_monsters"]
                for e in self.state.enemies:
                    if e.hp > 0:
                        e.statuses["Strength"] = e.statuses.get("Strength", 0) + str_amount
                self.log.log(self.state.turn, "strength_all_monsters",
                           enemy=enemy.id, amount=str_amount)

            # Plated Armor to ALL monsters (Deca Square of Protection A19+)
            if "plated_armor_all_monsters" in effects:
                plated_amount = effects["plated_armor_all_monsters"]
                for e in self.state.enemies:
                    if e.hp > 0:
                        e.statuses["Plated Armor"] = e.statuses.get("Plated Armor", 0) + plated_amount

            # Spawning enemies (Reptomancer, Collector)
            if "spawn_daggers" in effects:
                self._handle_reptomancer_spawn(enemy, effects["spawn_daggers"])
            if "spawn_torchheads" in effects:
                self._handle_collector_spawn(enemy, effects["spawn_torchheads"])

        # Mark first turn as complete
        enemy.first_turn = False

    def _roll_enemy_move(self, enemy: EnemyCombatState):
        """Roll the next move for an enemy using pattern-based AI.

        If enemy_data contains a real Enemy object for this enemy, delegate
        to its roll_move() method for accurate AI. Otherwise fall back to
        the inline patterns below.
        """
        # Delegate to real Enemy AI if available (parallel array lookup)
        if self.enemy_objects:
            for idx, e in enumerate(self.state.enemies):
                if e is not enemy:
                    continue
                if idx < len(self.enemy_objects) and self.enemy_objects[idx] is not None:
                    real_enemy = self.enemy_objects[idx]
                    real_enemy.state.strength = enemy.statuses.get("Strength", 0)
                    real_enemy.state.block = enemy.block
                    real_enemy.state.current_hp = enemy.hp
                    real_enemy.state.move_history = list(enemy.move_history)
                    real_enemy.state.first_turn = enemy.first_turn
                    # Pass player HP for moves that scale with it (e.g. Hexaghost Divider)
                    real_enemy.state.player_hp = self.state.player.hp
                    # Pass number of living allies for AI that cares
                    real_enemy.state.num_allies = sum(
                        1 for e2 in self.state.enemies if e2.hp > 0 and e2 is not enemy
                    )
                    move = real_enemy.roll_move()
                    self._set_enemy_move(enemy, move)
                    return
                break

        roll = self.ai_rng.random(99) if hasattr(self.ai_rng, 'random') else (
            (self.state.turn * 17 + hash(enemy.id) + len(enemy.move_history)) % 100
        )

        enemy_id = enemy.id

        # =====================================================================
        # Jaw Worm: Turn 1 = Chomp. Then cycles Bellow -> Thrash -> repeat
        # =====================================================================
        if enemy_id == "JawWorm":
            if enemy.first_turn:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Chomp", intent=Intent.ATTACK, base_damage=11))
            else:
                last = enemy.move_history[-1] if enemy.move_history else 1
                if last == 1 or last == 3:
                    # Bellow: +3 str, +6 block
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=2, name="Bellow", intent=Intent.DEFEND_BUFF,
                        block=6, effects={"strength": 3}))
                elif last == 2:
                    # Thrash: 7 dmg + 5 block
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=3, name="Thrash", intent=Intent.ATTACK_DEFEND,
                        base_damage=7, block=5))
            return

        # =====================================================================
        # Cultist: Turn 1 = Incantation (Ritual). Then always Dark Strike
        # =====================================================================
        if enemy_id == "Cultist":
            if enemy.first_turn:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Incantation", intent=Intent.BUFF,
                    effects={"ritual": 3}))
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Dark Strike", intent=Intent.ATTACK,
                    base_damage=6))
            return

        # =====================================================================
        # Red Louse / Green Louse: Random Bite (5-7 dmg) or Grow/Spit
        # =====================================================================
        if enemy_id in ("LouseRed", "Louse (Red)", "RedLouse"):
            if roll < 50:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Bite", intent=Intent.ATTACK,
                    base_damage=5 + (roll % 3)))  # 5-7
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Grow", intent=Intent.BUFF,
                    effects={"strength": 3}))
            return

        if enemy_id in ("LouseGreen", "Louse (Green)", "GreenLouse"):
            if roll < 50:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Bite", intent=Intent.ATTACK,
                    base_damage=5 + (roll % 3)))  # 5-7
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Spit Web", intent=Intent.DEBUFF,
                    effects={"weak": 2}))
            return

        # =====================================================================
        # Gremlin Nob: Turn 1 = Rush. If player played Skill -> Skull Bash.
        #              Else Bellow (+2 str)
        # =====================================================================
        if enemy_id in ("GremlinNob", "Gremlin Nob"):
            if enemy.first_turn:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Rush", intent=Intent.ATTACK, base_damage=14))
            elif self.state.skills_played_this_turn > 0:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Skull Bash", intent=Intent.ATTACK_DEBUFF,
                    base_damage=6, effects={"vulnerable": 2}))
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=3, name="Bellow", intent=Intent.BUFF,
                    effects={"strength": 2}))
            return

        # =====================================================================
        # Lagavulin: Sleeps 3 turns (8 block each). Then cycles Attack / Siphon
        # =====================================================================
        if enemy_id in ("Lagavulin",):
            turn_count = len(enemy.move_history)
            if turn_count < 3:
                # Sleeping
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Sleep", intent=Intent.SLEEP, block=8))
            else:
                last = enemy.move_history[-1] if enemy.move_history else 1
                if last != 2:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=2, name="Attack", intent=Intent.ATTACK,
                        base_damage=18))
                else:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=3, name="Siphon Soul", intent=Intent.STRONG_DEBUFF,
                        effects={"player_strength": -1, "player_dexterity": -1}))
            return

        # =====================================================================
        # Sentry: Cycles Bolt (9 dmg) and Beam (9 dmg + Dazed to discard)
        # =====================================================================
        if enemy_id in ("Sentry",):
            last = enemy.move_history[-1] if enemy.move_history else 0
            if last != 1:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Bolt", intent=Intent.ATTACK, base_damage=9))
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Beam", intent=Intent.ATTACK,
                    base_damage=9, effects={"add_dazed": 1}))
            return

        # =====================================================================
        # Slime Boss: Slam (35 dmg) then Slime. On half HP, splits.
        # =====================================================================
        if enemy_id in ("SlimeBoss", "Slime Boss"):
            last = enemy.move_history[-1] if enemy.move_history else 0
            if last != 1:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Slam", intent=Intent.ATTACK, base_damage=35))
            else:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Slime", intent=Intent.STRONG_DEBUFF,
                    effects={"slimed": 3}))
            return

        # =====================================================================
        # Hexaghost: Turn 1 = Activate, Turn 2 = Divider (6x), then cycles
        # =====================================================================
        if enemy_id in ("Hexaghost",):
            turn_count = len(enemy.move_history)
            if turn_count == 0:
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=1, name="Activate", intent=Intent.BUFF))
            elif turn_count == 1:
                # Divider: 6 hits, damage based on player HP / 12 + 1
                div_damage = max(1, self.state.player.hp // 12 + 1)
                self._set_enemy_move(enemy, MoveInfo(
                    move_id=2, name="Divider", intent=Intent.ATTACK,
                    base_damage=div_damage, hits=6, is_multi=True))
            else:
                # Cycle through attack patterns
                cycle_pos = (turn_count - 2) % 4
                if cycle_pos == 0:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=3, name="Sear", intent=Intent.ATTACK,
                        base_damage=6, effects={"add_burn": 1}))
                elif cycle_pos == 1:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=4, name="Tackle", intent=Intent.ATTACK_DEBUFF,
                        base_damage=5, hits=2, is_multi=True))
                elif cycle_pos == 2:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=5, name="Sear", intent=Intent.ATTACK,
                        base_damage=6, effects={"add_burn": 1}))
                else:
                    self._set_enemy_move(enemy, MoveInfo(
                        move_id=6, name="Inferno", intent=Intent.ATTACK,
                        base_damage=2, hits=6, is_multi=True,
                        effects={"add_burn": 3}))
            return

        # =====================================================================
        # Default fallback: cycle through existing move data on the enemy
        # =====================================================================
        last_move = enemy.move_history[-1] if enemy.move_history else 0
        # Use whatever move data is already on the enemy, alternating
        if enemy.move_damage > 0 and last_move != 2:
            self._set_enemy_move(enemy, MoveInfo(
                move_id=2, name="Attack", intent=Intent.ATTACK,
                base_damage=enemy.move_damage))
        elif enemy.move_block > 0:
            self._set_enemy_move(enemy, MoveInfo(
                move_id=1, name="Defend", intent=Intent.DEFEND,
                block=enemy.move_block))
        else:
            # Absolute fallback: basic attack
            damage = enemy.move_damage if enemy.move_damage > 0 else 6
            self._set_enemy_move(enemy, MoveInfo(
                move_id=2, name="Attack", intent=Intent.ATTACK,
                base_damage=damage))

    def _set_enemy_move(self, enemy: EnemyCombatState, move: MoveInfo):
        """Set an enemy's next move."""
        enemy.move_id = move.move_id
        enemy.move_damage = move.base_damage
        enemy.move_hits = move.hits
        enemy.move_block = move.block
        enemy.move_effects = dict(move.effects) if move.effects else {}
        enemy.move_history.append(move.move_id)

    def _check_combat_end(self) -> bool:
        """Check if combat should end. Returns True if ended."""
        # All enemies dead?
        all_dead = all(e.hp <= 0 for e in self.state.enemies)
        if all_dead:
            self._end_combat(player_won=True)
            return True

        # Player dead?
        if self.state.player.hp <= 0:
            # Check for Fairy in a Bottle before ending combat
            if not self._check_fairy_in_bottle():
                self._end_combat(player_won=False)
                return True

        return False

    def _check_fairy_in_bottle(self) -> bool:
        """
        Check if player would die and has Fairy in a Bottle.
        Returns True if fairy triggered, False otherwise.
        """
        if self.state.player.hp > 0:
            return False

        # Look for Fairy in a Bottle in potion slots
        for i, potion_id in enumerate(self.state.potions):
            if potion_id == "FairyPotion":
                # Calculate heal amount (30% base, 60% with Sacred Bark)
                has_sacred_bark = self.state.has_relic("SacredBark")
                heal_percent = 60 if has_sacred_bark else 30
                heal_to = int(self.state.player.max_hp * heal_percent / 100)

                # Revive player
                self.state.player.hp = heal_to

                # Remove potion
                self.state.potions[i] = ""

                # Log the trigger
                self.log.log(self.state.turn, "fairy_trigger",
                           heal_to=heal_to,
                           sacred_bark=has_sacred_bark)

                return True

        return False

    def _end_combat(self, player_won: bool):
        """End the combat."""
        self.state.combat_over = True
        self.state.player_won = player_won
        self.phase = CombatPhase.COMBAT_OVER

        # Trigger onVictory relics (Burning Blood, Meat on the Bone, etc.)
        if player_won:
            execute_relic_triggers("onVictory", self.state)

        self.log.log(self.state.turn, "combat_end",
                    player_won=player_won,
                    hp_remaining=self.state.player.hp,
                    turns=self.state.turn)

    def get_result(self) -> CombatResult:
        """Get the combat result."""
        return CombatResult(
            victory=self.state.player_won,
            hp_remaining=self.state.player.hp,
            hp_lost=self.initial_hp - self.state.player.hp,
            turns=self.state.turn,
            cards_played=self.state.total_cards_played,
            damage_dealt=self.state.total_damage_dealt,
            damage_taken=self.state.total_damage_taken,
            cards_played_sequence=self.cards_played_sequence,
            stance_changes=self.stance_changes,
            energy_spent=self.energy_spent,
        )

    # =========================================================================
    # Card Execution
    # =========================================================================

    def get_legal_actions(self) -> List[Action]:
        """Get all legal actions from current state."""
        if self.phase != CombatPhase.PLAYER_TURN or self.state.combat_over:
            return []

        actions: List[Action] = []
        living_enemies = [i for i, e in enumerate(self.state.enemies) if e.hp > 0]

        # Card plays
        for hand_idx, card_id in enumerate(self.state.hand):
            card = self._get_card(card_id)

            if self._can_play_card(card, hand_idx):
                if card.target == CardTarget.ENEMY:
                    for enemy_idx in living_enemies:
                        actions.append(PlayCard(card_idx=hand_idx, target_idx=enemy_idx))
                else:
                    actions.append(PlayCard(card_idx=hand_idx, target_idx=-1))

        # Potion uses
        for pot_idx, potion_id in enumerate(self.state.potions):
            if potion_id:
                pot_target = self._get_potion_target(potion_id)
                if pot_target == "enemy":
                    for enemy_idx in living_enemies:
                        actions.append(UsePotion(potion_idx=pot_idx, target_idx=enemy_idx))
                else:
                    actions.append(UsePotion(potion_idx=pot_idx, target_idx=-1))

        # End turn is always legal
        actions.append(EndTurn())

        return actions

    def _can_play_card(self, card: Card, hand_index: int) -> bool:
        """Check if a card can be played."""
        # X-cost cards can always be played if energy >= 0
        if card.cost == -1:
            return self.state.energy >= 0

        # Energy check
        if card.current_cost > self.state.energy:
            return False

        # Unplayable check
        if card.cost == -2 or "unplayable" in card.effects:
            return False

        # Signature Move check
        if "only_attack_in_hand" in card.effects:
            attacks_in_hand = sum(
                1 for card_id in self.state.hand
                if self._get_card(card_id).card_type == CardType.ATTACK
            )
            if attacks_in_hand > 1:
                return False

        # Entangled check
        if self.state.player.statuses.get("Entangled", 0) > 0 and card.card_type == CardType.ATTACK:
            return False

        return True

    def execute_action(self, action: Action) -> Dict[str, Any]:
        """Execute an action and return result."""
        if isinstance(action, EndTurn):
            self.end_turn()
            return {"type": "end_turn"}
        elif isinstance(action, PlayCard):
            return self.play_card(action.card_idx, action.target_idx)
        elif isinstance(action, UsePotion):
            return self.use_potion(action.potion_idx, action.target_idx)
        return {"type": "unknown", "error": "Unknown action type"}

    def play_card(self, hand_index: int, target_index: int = -1) -> Dict[str, Any]:
        """Play a card from hand."""
        if hand_index >= len(self.state.hand):
            return {"success": False, "error": "Invalid hand index"}

        card_id = self.state.hand[hand_index]
        card = self._get_card(card_id)

        if not self._can_play_card(card, hand_index):
            return {"success": False, "error": "Cannot play card"}

        result = {"success": True, "card": card_id, "effects": []}

        # Pay energy - X-cost cards spend all remaining energy
        if card.cost == -1:
            cost = self.state.energy
            self._x_cost_amount = cost  # Store for effect scaling
            self.state.energy = 0
            self.energy_spent += cost
        else:
            cost = card.current_cost
            if cost > 0:
                self.state.energy -= cost
                self.energy_spent += cost

        # Remove from hand
        self.state.hand.pop(hand_index)

        # Track card play
        self.state.cards_played_this_turn += 1
        self.state.total_cards_played += 1
        self.state.last_card_type = card.card_type.value if hasattr(card.card_type, 'value') else str(card.card_type)
        self.cards_played_sequence.append(card_id)

        if card.card_type == CardType.ATTACK:
            self.state.attacks_played_this_turn += 1
        elif card.card_type == CardType.SKILL:
            self.state.skills_played_this_turn += 1
        elif card.card_type == CardType.POWER:
            self.state.powers_played_this_turn += 1

        # Get target enemy
        target_enemy = None
        if 0 <= target_index < len(self.state.enemies):
            target_enemy = self.state.enemies[target_index]
            if target_enemy.hp <= 0:
                target_enemy = None

        # Trigger registry-based onPlayCard relics (Shuriken, Kunai, etc.)
        execute_relic_triggers("onPlayCard", self.state, {"card": card})

        # Apply card effects
        self._apply_card_effects(card, target_index, result)

        # Card destination
        if card.exhaust:
            self.state.exhaust_pile.append(card_id)
            # Trigger registry-based onExhaust relics and powers
            execute_relic_triggers("onExhaust", self.state, {"card": card})
            execute_power_triggers("onExhaust", self.state, self.state.player, {"card": card})
            if card.id == "Sentinel":
                self.state.energy += 3 if card.upgraded else 2
        elif card.shuffle_back:
            # Random position in draw pile
            pos = self.state.turn % (len(self.state.draw_pile) + 1) if self.state.draw_pile else 0
            self.state.draw_pile.insert(pos, card_id)
        else:
            self.state.discard_pile.append(card_id)

        # Trigger onUseCard power triggers (After Image, Duplication, etc.)
        execute_power_triggers("onUseCard", self.state, self.state.player, {"card": card, "card_id": card.id})

        # Trigger onAfterUseCard power triggers (Beat of Death, Slow, Time Warp)
        force_end_turn = False
        after_use_data = {"card": card, "card_id": card.id}
        execute_power_triggers("onAfterUseCard", self.state, self.state.player, after_use_data)
        if after_use_data.get("force_end_turn"):
            force_end_turn = True
        for enemy in self.state.enemies:
            if enemy.hp <= 0:
                continue
            enemy_trigger = {"card": card, "card_id": card.id}
            execute_power_triggers("onAfterUseCard", self.state, enemy, enemy_trigger)
            if enemy_trigger.get("force_end_turn"):
                force_end_turn = True

        # Trigger onAfterCardPlayed power triggers (Thousand Cuts)
        after_play_data = {"card": card, "card_id": card.id}
        execute_power_triggers("onAfterCardPlayed", self.state, self.state.player, after_play_data)
        for enemy in self.state.enemies:
            if enemy.hp <= 0:
                continue
            execute_power_triggers("onAfterCardPlayed", self.state, enemy, after_play_data)

        # Log
        self.log.log(self.state.turn, "play_card",
                    card=card_id,
                    target=target_index,
                    effects=result["effects"])

        # Time Eater 12-card counter check (after card is played)
        self._check_time_eater_numen()

        # End turn effect
        if force_end_turn or "end_turn" in card.effects:
            self.end_turn()

        # Check combat end
        self._check_combat_end()

        return result

    def _apply_card_effects(self, card: Card, target_index: int, result: Dict):
        """Apply all effects of a card."""
        effects = result["effects"]

        # Damage
        if card.damage > 0:
            self._apply_card_damage(card, target_index, effects)

        # Block
        if card.block > 0:
            block_gained = self._calculate_block_gained(card.block)
            self.state.player.block += block_gained
            effects.append({"type": "block", "amount": block_gained})
            # Trigger onGainBlock power hooks (Juggernaut, Wave of the Hand).
            execute_power_triggers(
                "onGainBlock",
                self.state,
                self.state.player,
                {"block_amount": block_gained},
            )

        # Stance changes
        if card.enter_stance:
            new_stance = self._parse_stance(card.enter_stance)
            self._change_stance(new_stance)
            effects.append({"type": "stance", "stance": card.enter_stance})

        if card.exit_stance:
            self._change_stance(StanceID.NEUTRAL)
            effects.append({"type": "stance", "stance": "Neutral"})

        # Draw effects
        if "draw_1" in card.effects:
            self._draw_cards(1)
            effects.append({"type": "draw", "amount": 1})
        if "draw_2" in card.effects:
            self._draw_cards(2)
            effects.append({"type": "draw", "amount": 2})
        if "draw_cards" in card.effects and card.magic_number > 0:
            self._draw_cards(card.magic_number)
            effects.append({"type": "draw", "amount": card.magic_number})

        # Scry
        if "scry" in card.effects:
            scry_amount = card.magic_number if card.magic_number > 0 else 2
            self._scry(scry_amount)
            effects.append({"type": "scry", "amount": scry_amount})

        # Mantra
        if "gain_mantra" in card.effects and card.magic_number > 0:
            self._add_mantra(card.magic_number)
            effects.append({"type": "mantra", "amount": card.magic_number})

        # Energy gain
        if "gain_1_energy" in card.effects:
            self.state.energy += 1
            effects.append({"type": "energy", "amount": 1})
        if "gain_2_energy" in card.effects:
            self.state.energy += 2
            effects.append({"type": "energy", "amount": 2})

        # Equilibrium: retain hand via temporary turn-based power state.
        if "retain_hand" in card.effects:
            amount = card.magic_number if card.magic_number > 0 else 1
            self.state.player.statuses["Equilibrium"] = (
                self.state.player.statuses.get("Equilibrium", 0) + amount
            )
            self.state.player.statuses["RetainHand"] = 1
            effects.append({"type": "power", "power": "Equilibrium", "amount": amount})

        # Apply powers from power cards
        if card.card_type == CardType.POWER:
            self._apply_power_card(card, effects)

        # Apply debuffs to enemy
        if target_index >= 0 and target_index < len(self.state.enemies):
            enemy = self.state.enemies[target_index]
            if enemy.hp > 0:
                if "apply_weak" in card.effects or "if_last_card_attack_weak" in card.effects:
                    weak_amount = card.magic_number if card.magic_number > 0 else 1
                    self._apply_status(enemy, "Weak", weak_amount)
                    effects.append({"type": "debuff", "debuff": "Weak", "amount": weak_amount})
                if "apply_vulnerable" in card.effects or "if_last_card_skill_vulnerable" in card.effects:
                    vuln_amount = card.magic_number if card.magic_number > 0 else 1
                    self._apply_status(enemy, "Vulnerable", vuln_amount)
                    effects.append({"type": "debuff", "debuff": "Vulnerable", "amount": vuln_amount})

    def _apply_card_damage(self, card: Card, target_index: int, effects: List):
        """Apply damage from a card."""
        # Calculate number of hits
        hits = 1
        if "damage_x_times" in card.effects and card.magic_number > 0:
            hits = card.magic_number
        elif "damage_twice" in card.effects:
            hits = 2

        # Calculate damage per hit (includes vuln in single-chain calculation)
        base_damage = card.damage
        strength_mult = 1
        if "strength_multiplier" in card.effects:
            strength_mult = card.magic_number if card.magic_number > 0 else 3
        damage_per_hit = self._calculate_card_damage(base_damage, target_index, strength_mult)

        # For ALL_ENEMY cards, pre-compute per-enemy damage before vigor consumption
        enemy_damages = {}
        if card.target == CardTarget.ALL_ENEMY:
            for i, enemy in enumerate(self.state.enemies):
                if enemy.hp > 0:
                    enemy_damages[i] = self._calculate_card_damage(base_damage, i, strength_mult)

        # Consume Vigor after first attack card uses it
        if self.state.player.statuses.get("Vigor", 0) > 0:
            self.state.player.statuses["Vigor"] = 0

        # Apply damage
        if card.target == CardTarget.ALL_ENEMY:
            for i, enemy in enumerate(self.state.enemies):
                if enemy.hp > 0 and i in enemy_damages:
                    for _ in range(hits):
                        execute_power_triggers(
                            "atDamageGive",
                            self.state,
                            self.state.player,
                            {
                                "value": float(enemy_damages[i]),
                                "card": card,
                                "card_id": card.id,
                                "damage_type": "NORMAL",
                            },
                        )
                        execute_power_triggers(
                            "atDamageReceive",
                            self.state,
                            enemy,
                            {"value": float(enemy_damages[i]), "damage_type": "NORMAL"},
                        )
                        execute_power_triggers(
                            "atDamageFinalReceive",
                            self.state,
                            enemy,
                            {"value": float(enemy_damages[i]), "damage_type": "NORMAL"},
                        )
                        actual_damage = self._deal_damage_to_enemy(enemy, enemy_damages[i])
                        execute_power_triggers(
                            "onAttack",
                            self.state,
                            self.state.player,
                            {
                                "card": card,
                                "card_id": card.id,
                                "target": enemy,
                                "damage": enemy_damages[i],
                                "unblocked_damage": actual_damage,
                                "damage_type": "NORMAL",
                            },
                        )
                        execute_power_triggers(
                            "onAttacked",
                            self.state,
                            enemy,
                            {
                                "attacker": self.state.player,
                                "card": card,
                                "card_id": card.id,
                                "damage": enemy_damages[i],
                                "unblocked_damage": actual_damage,
                                "damage_type": "NORMAL",
                            },
                        )
                        effects.append({
                            "type": "damage",
                            "target": enemy.id,
                            "amount": actual_damage
                        })
        elif target_index >= 0 and target_index < len(self.state.enemies):
            enemy = self.state.enemies[target_index]
            if enemy.hp > 0:
                for _ in range(hits):
                    execute_power_triggers(
                        "atDamageGive",
                        self.state,
                        self.state.player,
                        {
                            "value": float(damage_per_hit),
                            "card": card,
                            "card_id": card.id,
                            "damage_type": "NORMAL",
                        },
                    )
                    execute_power_triggers(
                        "atDamageReceive",
                        self.state,
                        enemy,
                        {"value": float(damage_per_hit), "damage_type": "NORMAL"},
                    )
                    execute_power_triggers(
                        "atDamageFinalReceive",
                        self.state,
                        enemy,
                        {"value": float(damage_per_hit), "damage_type": "NORMAL"},
                    )
                    actual_damage = self._deal_damage_to_enemy(enemy, damage_per_hit)
                    execute_power_triggers(
                        "onAttack",
                        self.state,
                        self.state.player,
                        {
                            "card": card,
                            "card_id": card.id,
                            "target": enemy,
                            "damage": damage_per_hit,
                            "unblocked_damage": actual_damage,
                            "damage_type": "NORMAL",
                        },
                    )
                    execute_power_triggers(
                        "onAttacked",
                        self.state,
                        enemy,
                        {
                            "attacker": self.state.player,
                            "card": card,
                            "card_id": card.id,
                            "damage": damage_per_hit,
                            "unblocked_damage": actual_damage,
                            "damage_type": "NORMAL",
                        },
                    )
                    effects.append({
                        "type": "damage",
                        "target": enemy.id,
                        "amount": actual_damage
                    })
                    if enemy.hp <= 0:
                        effects.append({"type": "kill", "target": enemy.id})
                        break

    def _deal_damage_to_enemy(self, enemy: EnemyCombatState, damage: int, apply_vuln: bool = False) -> int:
        """Deal damage to an enemy, return actual HP damage dealt.

        For card damage, vuln is already factored in via _calculate_card_damage (single-chain).
        For non-card damage (potions, Juggernaut), pass apply_vuln=True.
        """
        if apply_vuln and enemy.statuses.get("Vulnerable", 0) > 0:
            damage = int(damage * VULN_MULT)

        # Apply block
        blocked = min(enemy.block, damage)
        hp_damage = damage - blocked
        enemy.block -= blocked
        enemy.hp -= hp_damage

        self.state.total_damage_dealt += hp_damage

        # Curl Up: gain block when first attacked (one-time trigger)
        curl_up = enemy.statuses.get("Curl Up", 0)
        if curl_up > 0 and hp_damage > 0:
            enemy.block += curl_up
            del enemy.statuses["Curl Up"]
            self.log.log(self.state.turn, "curl_up", enemy=enemy.id, block=curl_up)

        # Sharp Hide (Guardian): damage player per hit
        sharp_hide = enemy.statuses.get("Sharp Hide", 0)
        if sharp_hide > 0:
            sh_blocked = min(self.state.player.block, sharp_hide)
            sh_hp = sharp_hide - sh_blocked
            self.state.player.block -= sh_blocked
            self.state.player.hp -= sh_hp
            self.state.total_damage_taken += sh_hp

        # Clamp HP
        if enemy.hp < 0:
            enemy.hp = 0

        # Guardian mode shift (track damage taken)
        if hp_damage > 0:
            self._check_guardian_mode_shift(enemy, hp_damage)

        # Check split threshold (large slimes split at 50% HP)
        if hp_damage > 0 and enemy.hp > 0:
            self._check_split(enemy)

        # Awakened One rebirth check (phase 1 -> phase 2)
        if enemy.hp <= 0:
            if self._check_awakened_one_rebirth(enemy):
                # Rebirth successful, enemy is back to life
                return hp_damage

        # Death trigger
        if enemy.hp <= 0:
            self._on_enemy_death(enemy)

        return hp_damage

    def _check_split(self, enemy: EnemyCombatState):
        """Check if an enemy should split (large slimes at 50% HP)."""
        if enemy.hp > enemy.max_hp // 2:
            return

        # Find the real Enemy object
        real_enemy = self._get_real_enemy(enemy)
        if real_enemy is None:
            return

        if not hasattr(real_enemy, 'check_split'):
            return

        spawn_info = real_enemy.check_split(enemy.hp)
        if not spawn_info:
            return
        if isinstance(spawn_info, bool):
            if not hasattr(real_enemy, "get_split_spawn_info"):
                return
            spawn_info = real_enemy.get_split_spawn_info()
        if isinstance(spawn_info, dict) and "ascension" not in spawn_info:
            if hasattr(real_enemy, "ascension"):
                spawn_info["ascension"] = real_enemy.ascension

        # Kill the parent
        enemy.hp = 0
        self._spawn_enemies(spawn_info)

    def _spawn_enemies(self, spawn_info):
        """Spawn new enemies from split/summon."""
        if not spawn_info:
            return
        if isinstance(spawn_info, (EnemyCombatState, dict, tuple)):
            spawn_info = [spawn_info]
        for info in spawn_info:
            if isinstance(info, EnemyCombatState):
                self.state.enemies.append(info)
                self._roll_enemy_move(info)
            elif isinstance(info, tuple) and len(info) >= 3:
                enemy_id, hp, max_hp = info[0], info[1], info[2]
                new_enemy = EnemyCombatState(
                    hp=hp, max_hp=max_hp, block=0, statuses={},
                    id=enemy_id, name=enemy_id,
                    move_id=-1, move_damage=0, move_hits=1,
                    move_block=0, move_effects={},
                )
                self.state.enemies.append(new_enemy)
                # Roll initial move
                self._roll_enemy_move(new_enemy)
            elif isinstance(info, dict):
                enemy_id = info.get("enemy_class") or info.get("enemy_id") or info.get("id")
                if not enemy_id:
                    continue
                count = int(info.get("count", 1))
                starting_hp = info.get("hp")
                poison_amount = info.get("poison", 0)
                ascension = info.get("ascension", 0)
                for _ in range(count):
                    try:
                        kwargs = {}
                        if starting_hp is not None:
                            kwargs["starting_hp"] = starting_hp
                        if poison_amount:
                            kwargs["poison_amount"] = poison_amount
                        real_spawn = create_enemy_object(
                            enemy_id,
                            self.ai_rng,
                            ascension=ascension,
                            **kwargs,
                        )
                    except TypeError:
                        real_spawn = create_enemy_object(
                            enemy_id,
                            self.ai_rng,
                            ascension=ascension,
                        )
                    new_enemy = EnemyCombatState(
                        hp=real_spawn.state.current_hp,
                        max_hp=real_spawn.state.max_hp,
                        block=real_spawn.state.block,
                        statuses=dict(real_spawn.state.powers),
                        id=real_spawn.ID,
                        name=real_spawn.NAME,
                        enemy_type=str(real_spawn.TYPE.value) if hasattr(real_spawn.TYPE, "value") else str(real_spawn.TYPE),
                        move_history=list(real_spawn.state.move_history),
                        first_turn=real_spawn.state.first_turn,
                    )
                    if real_spawn.state.next_move:
                        move = real_spawn.state.next_move
                        new_enemy.move_id = move.move_id
                        new_enemy.move_damage = move.base_damage
                        new_enemy.move_hits = move.hits
                        new_enemy.move_block = move.block
                        new_enemy.move_effects = dict(move.effects) if move.effects else {}
                    self.state.enemies.append(new_enemy)
                    if self.enemy_objects and len(self.enemy_objects) == len(self.state.enemies) - 1:
                        self.enemy_objects.append(real_spawn)
                    self._roll_enemy_move(new_enemy)

    def _on_enemy_death(self, enemy: EnemyCombatState):
        """Handle enemy death triggers."""
        execute_power_triggers("onDeath", self.state, enemy, {"dying_enemy": enemy})

        # Spore Cloud (FungiBeast): apply Vulnerable to player
        spore_cloud = enemy.statuses.get("Spore Cloud", 0)
        if spore_cloud > 0:
            self.state.player.statuses["Vulnerable"] = (
                self.state.player.statuses.get("Vulnerable", 0) + spore_cloud
            )
            self.log.log(self.state.turn, "death_trigger",
                        enemy=enemy.id, effect="Spore Cloud", amount=spore_cloud)

        # Exploder: deal damage to player on death
        explode_damage = enemy.statuses.get("Explosive", 0)
        if explode_damage > 0:
            blocked = min(self.state.player.block, explode_damage)
            hp_dmg = explode_damage - blocked
            self.state.player.block -= blocked
            self.state.player.hp -= hp_dmg
            self.state.total_damage_taken += hp_dmg
            self.log.log(self.state.turn, "death_trigger",
                        enemy=enemy.id, effect="Explosive", damage=hp_dmg)

        # Delegate to real Enemy object if available
        real_enemy = self._get_real_enemy(enemy)
        if real_enemy and hasattr(real_enemy, 'on_death'):
            real_enemy.on_death()

    def _get_real_enemy(self, enemy: EnemyCombatState) -> Optional[Enemy]:
        """Get the real Enemy object for an EnemyCombatState, if available."""
        if not self.enemy_objects:
            return None
        for idx, e in enumerate(self.state.enemies):
            if e is enemy:
                if idx < len(self.enemy_objects):
                    return self.enemy_objects[idx]
                break
        return None

    def _apply_power_card(self, card: Card, effects: List):
        """Apply a power card's effect."""
        player = self.state.player

        power_mapping = {
            "MentalFortress": ("MentalFortress", card.magic_number if card.magic_number > 0 else 4),
            "Adaptation": ("Rushdown", card.magic_number if card.magic_number > 0 else 2),  # Internal ID
            "Rushdown": ("Rushdown", card.magic_number if card.magic_number > 0 else 2),
            "Nirvana": ("Nirvana", card.magic_number if card.magic_number > 0 else 3),
            "LikeWater": ("LikeWater", card.magic_number if card.magic_number > 0 else 5),
            "DevaForm": ("DevaForm", 1),
            "Devotion": ("Devotion", card.magic_number if card.magic_number > 0 else 2),
            "Wireheading": ("Foresight", card.magic_number if card.magic_number > 0 else 3),  # Internal ID
            "Foresight": ("Foresight", card.magic_number if card.magic_number > 0 else 3),
            "Establishment": ("Establishment", 1),
            "BattleHymn": ("BattleHymn", 1),
            "Study": ("Study", 1),
            "Inflame": ("Strength", card.magic_number if card.magic_number > 0 else 2),
            "Metallicize": ("Metallicize", card.magic_number if card.magic_number > 0 else 3),
            "Barricade": ("Barricade", 1),
            "Echo Form": ("Echo Form", card.magic_number if card.magic_number > 0 else 1),
        }

        if card.id in power_mapping:
            power_id, amount = power_mapping[card.id]
            player.statuses[power_id] = player.statuses.get(power_id, 0) + amount
            effects.append({"type": "power", "power": power_id, "amount": amount})

    def _rng_pick_index(self, rng: Any, upper_inclusive: int) -> int:
        """Pick an index with best-effort support for engine RNG and stdlib RNG."""
        if upper_inclusive <= 0:
            return 0
        if rng is None:
            return 0

        # Engine RNG: random(range) -> [0, range]
        try:
            return int(rng.random(upper_inclusive))
        except TypeError:
            pass

        # Python random.Random compatibility.
        if hasattr(rng, "randint"):
            return int(rng.randint(0, upper_inclusive))

        return 0

    def _resolve_random_enemy_target_index(self) -> int:
        """Choose a random living enemy index using cardRandomRng parity stream."""
        living_indices = [i for i, enemy in enumerate(self.state.enemies) if enemy.hp > 0]
        if not living_indices:
            return -1

        rng = (
            getattr(self.state, "card_random_rng", None)
            or getattr(self.state, "card_rng", None)
            or self.card_rng
        )
        pick = self._rng_pick_index(rng, len(living_indices) - 1)
        return living_indices[pick]

    def _draw_top_card_for_autoplay(self) -> Optional[str]:
        """Draw top card for autoplay effects (top = end of list)."""
        if not self.state.draw_pile:
            if not self.state.discard_pile:
                return None
            self.state.draw_pile = self.state.discard_pile.copy()
            self.state.discard_pile.clear()
            self._shuffle_draw_pile()
            execute_relic_triggers("onShuffle", self.state)

        if not self.state.draw_pile:
            return None
        return self.state.draw_pile.pop()

    def _autoplay_card(self, card_id: str, target_index: int = -1) -> Dict[str, Any]:
        """Play a card from non-hand sources (Distilled Chaos / PlayTopCardAction style)."""
        card = self._get_card(card_id)
        result = {"card": card_id, "target_index": target_index, "effects": [], "played": False}

        # Unplayable cards still move to destination piles but do not execute effects.
        unplayable = card.cost == -2 or "unplayable" in card.effects

        if not unplayable:
            self.state.cards_played_this_turn += 1
            self.state.total_cards_played += 1
            self.state.last_card_type = (
                card.card_type.value if hasattr(card.card_type, "value") else str(card.card_type)
            )
            self.cards_played_sequence.append(card_id)

            if card.card_type == CardType.ATTACK:
                self.state.attacks_played_this_turn += 1
            elif card.card_type == CardType.SKILL:
                self.state.skills_played_this_turn += 1
            elif card.card_type == CardType.POWER:
                self.state.powers_played_this_turn += 1

            execute_relic_triggers("onPlayCard", self.state, {"card": card})
            self._apply_card_effects(card, target_index, result)

            execute_power_triggers("onUseCard", self.state, self.state.player, {"card": card, "card_id": card.id})

            force_end_turn = False
            after_use_data = {"card": card, "card_id": card.id}
            execute_power_triggers("onAfterUseCard", self.state, self.state.player, after_use_data)
            if after_use_data.get("force_end_turn"):
                force_end_turn = True

            for enemy in self.state.enemies:
                if enemy.hp <= 0:
                    continue
                enemy_trigger = {"card": card, "card_id": card.id}
                execute_power_triggers("onAfterUseCard", self.state, enemy, enemy_trigger)
                if enemy_trigger.get("force_end_turn"):
                    force_end_turn = True

            after_play_data = {"card": card, "card_id": card.id}
            execute_power_triggers("onAfterCardPlayed", self.state, self.state.player, after_play_data)
            for enemy in self.state.enemies:
                if enemy.hp <= 0:
                    continue
                execute_power_triggers("onAfterCardPlayed", self.state, enemy, after_play_data)

            self._check_time_eater_numen()
            if force_end_turn or "end_turn" in card.effects:
                self.end_turn()

            result["played"] = True

        # Destination handling mirrors normal card flow.
        if card.exhaust:
            self.state.exhaust_pile.append(card_id)
            if not unplayable:
                execute_relic_triggers("onExhaust", self.state, {"card": card})
                execute_power_triggers("onExhaust", self.state, self.state.player, {"card": card})
                if card.id == "Sentinel":
                    self.state.energy += 3 if card.upgraded else 2
        elif card.shuffle_back:
            pos = self.state.turn % (len(self.state.draw_pile) + 1) if self.state.draw_pile else 0
            self.state.draw_pile.insert(pos, card_id)
        else:
            self.state.discard_pile.append(card_id)

        self._check_combat_end()
        return result

    def _play_top_cards_from_draw_pile(self, count: int) -> List[Dict[str, Any]]:
        """Play top cards with PlayTopCardAction semantics used by Distilled Chaos."""
        played: List[Dict[str, Any]] = []
        for _ in range(max(0, count)):
            card_id = self._draw_top_card_for_autoplay()
            if not card_id:
                break
            card = self._get_card(card_id)
            target_index = -1
            if card.target == CardTarget.ENEMY:
                target_index = self._resolve_random_enemy_target_index()
            played.append(self._autoplay_card(card_id, target_index))
            if self.state.combat_over:
                break
        return played

    def play_top_cards_from_draw_pile(self, count: int) -> List[Dict[str, Any]]:
        """Public wrapper for Distilled Chaos style top-deck autoplay."""
        return self._play_top_cards_from_draw_pile(count)

    def use_potion(self, potion_index: int, target_index: int = -1) -> Dict[str, Any]:
        """Use a potion."""
        if potion_index < 0 or potion_index >= len(self.state.potions):
            return {"success": False, "error": "Invalid potion index"}

        potion_id = self.state.potions[potion_index]
        if not potion_id:
            return {"success": False, "error": "Empty potion slot"}

        if potion_id == "FairyPotion":
            return {"success": False, "error": "Fairy in a Bottle triggers automatically on death"}

        from .content.potions import PotionTargetType, get_potion_by_id
        potion_data = get_potion_by_id(potion_id)
        if potion_data is None:
            return {"success": False, "error": f"Unknown potion: {potion_id}"}

        if potion_data.target_type == PotionTargetType.ENEMY:
            if target_index < 0 or target_index >= len(self.state.enemies):
                return {"success": False, "error": "Potion requires a living enemy target"}
            if self.state.enemies[target_index].hp <= 0:
                return {"success": False, "error": "Potion target is not alive"}

        has_sacred_bark = self.state.has_relic("SacredBark")
        potency = potion_data.get_effective_potency(has_sacred_bark)

        # Consume potion before applying effect (Java ordering).
        self.state.potions[potion_index] = ""
        result: Dict[str, Any] = {"success": True, "potion": potion_id, "effects": [], "potency": potency}
        registry_result = execute_potion_effect(potion_id, self.state, target_idx=target_index)
        if not registry_result.get("success"):
            # Keep behavior deterministic and non-destructive on unexpected failures.
            self.state.potions[potion_index] = potion_id
            return {"success": False, "error": registry_result.get("error", "Failed to use potion")}

        result["potency"] = registry_result.get("potency", potency)
        if "effects" in registry_result and isinstance(registry_result["effects"], list):
            result["effects"].extend(registry_result["effects"])
        if "played_cards" in registry_result:
            result["played_cards"] = registry_result["played_cards"]
        if potion_id == "SmokeBomb" and getattr(self.state, "escaped", False):
            result["effects"].append({"type": "escape"})

        self.log.log(self.state.turn, "use_potion", potion=potion_id, effects=result["effects"])
        self._check_combat_end()

        return result

    # =========================================================================
    # Damage Calculation
    # =========================================================================

    def _calculate_card_damage(
        self,
        base_damage: int,
        target_index: int = -1,
        strength_multiplier: int = 1,
    ) -> int:
        """Calculate damage for a card attack."""
        player = self.state.player

        # Get player modifiers
        strength = player.statuses.get("Strength", 0) * strength_multiplier
        vigor = player.statuses.get("Vigor", 0)
        weak = player.statuses.get("Weak", 0) > 0

        # Get stance multiplier
        stance = self._get_stance()
        stance_mult = 1.0
        if stance == StanceID.WRATH:
            stance_mult = WRATH_MULT
        elif stance == StanceID.DIVINITY:
            stance_mult = DIVINITY_MULT

        # Determine if target is vulnerable (must be included in single-chain calc)
        vuln = False
        if target_index >= 0 and target_index < len(self.state.enemies):
            vuln = self.state.enemies[target_index].statuses.get("Vulnerable", 0) > 0
        elif target_index == -1:
            # For ALL_ENEMY cards, vuln is checked per-enemy in _deal_damage_to_enemy
            # We pass vuln=False here; per-enemy vuln handled at deal time
            vuln = False

        return calculate_damage(
            base=base_damage,
            strength=strength,
            vigor=vigor,
            weak=weak,
            stance_mult=stance_mult,
            vuln=vuln,
        )

    def _calculate_block_gained(self, base_block: int) -> int:
        """Calculate block gained from a card."""
        modified = execute_power_triggers(
            "modifyBlock",
            self.state,
            self.state.player,
            {"value": float(base_block)},
        )
        if modified is None:
            modified = float(base_block)
        return max(0, int(modified))

    # =========================================================================
    # Stance System
    # =========================================================================

    def _get_stance(self) -> StanceID:
        """Get current stance as StanceID."""
        stance_str = self.state.stance
        if not stance_str or stance_str == "Neutral":
            return StanceID.NEUTRAL
        try:
            return StanceID(stance_str)
        except ValueError:
            return StanceID.NEUTRAL

    def _parse_stance(self, stance_str: str) -> StanceID:
        """Parse a stance string to StanceID."""
        if not stance_str:
            return StanceID.NEUTRAL
        stance_lower = stance_str.lower()
        if stance_lower == "wrath":
            return StanceID.WRATH
        elif stance_lower == "calm":
            return StanceID.CALM
        elif stance_lower == "divinity":
            return StanceID.DIVINITY
        return StanceID.NEUTRAL

    def _change_stance(self, new_stance: StanceID) -> Dict:
        """Change to a new stance."""
        old_stance = self._get_stance()

        if old_stance == new_stance:
            return {"changed": False}

        result = {"changed": True, "from": old_stance, "to": new_stance, "energy_gained": 0}

        # Exit effects
        if old_stance == StanceID.CALM:
            # Gain 2 energy base (Violet Lotus adds +1 via relic trigger)
            self.state.energy += 2
            result["energy_gained"] = 2

        # Enter effects
        if new_stance == StanceID.DIVINITY:
            self.state.energy += 3
            result["energy_gained"] += 3

        # Update stance
        self.state.stance = new_stance.value
        self.stance_changes += 1

        # Execute onChangeStance relic triggers (Violet Lotus)
        execute_relic_triggers("onChangeStance", self.state,
                              {"new_stance": new_stance.value, "old_stance": old_stance.value})

        # Execute onChangeStance power triggers (Mental Fortress, Rushdown)
        execute_power_triggers("onChangeStance", self.state, self.state.player,
                              {"new_stance": new_stance.value, "old_stance": old_stance.value})

        # Flurry of Blows trigger (card-based, not in power registry)
        flurries = [i for i, card_id in enumerate(self.state.discard_pile)
                   if "FlurryOfBlows" in card_id]
        for i in reversed(flurries):
            card_id = self.state.discard_pile.pop(i)
            self.state.hand.append(card_id)

        self.log.log(self.state.turn, "stance_change",
                    from_stance=old_stance.value,
                    to_stance=new_stance.value,
                    energy_gained=result["energy_gained"])

        return result

    def _add_mantra(self, amount: int):
        """Add mantra and potentially enter Divinity."""
        self.state.mantra += amount

        if self.state.mantra >= 10:
            self.state.mantra -= 10
            self._change_stance(StanceID.DIVINITY)

    def _scry(self, amount: int):
        """Scry - look at top cards of draw pile, discard chosen ones.

        For the RL engine, we auto-discard status/curse cards (Burns, Daze,
        Wound, Slimed, Void, Curse*) and keep the rest. This is a reasonable
        heuristic for simulation purposes.
        """
        actual_amount = min(amount, len(self.state.draw_pile))
        if actual_amount <= 0:
            # Still trigger onScry hooks even if no cards were revealed.
            execute_power_triggers(
                "onScry",
                self.state,
                self.state.player,
                {"cards_scried": 0},
            )
            return

        # Reveal top cards (top = end of list)
        revealed = self.state.draw_pile[-actual_amount:]
        self.state.draw_pile = self.state.draw_pile[:-actual_amount]

        # For the RL engine, discard all revealed cards.
        # A future decision-point implementation could let the agent choose.
        for card_id in revealed:
            self.state.discard_pile.append(card_id)

        # Trigger onScry hooks (Nirvana, etc.)
        execute_power_triggers(
            "onScry",
            self.state,
            self.state.player,
            {"cards_scried": actual_amount},
        )

        # Trigger Weave from discard
        weaves = [i for i, card_id in enumerate(self.state.discard_pile)
                 if "Weave" in card_id]
        for i in reversed(weaves):
            card_id = self.state.discard_pile.pop(i)
            self.state.hand.append(card_id)

    # =========================================================================
    # Deck Management
    # =========================================================================

    def _shuffle_draw_pile(self):
        """Shuffle the draw pile using Fisher-Yates with the shuffle RNG."""
        n = len(self.state.draw_pile)
        for i in range(n - 1, 0, -1):
            j = self.shuffle_rng.random(i) if hasattr(self.shuffle_rng, 'random') else (
                (self.state.shuffle_rng_state[0] + i * 7 + self.state.turn) % (i + 1)
            )
            self.state.draw_pile[i], self.state.draw_pile[j] = \
                self.state.draw_pile[j], self.state.draw_pile[i]

    def _draw_cards(self, count: int) -> List[str]:
        """Draw cards from draw pile to hand."""
        drawn = []

        for _ in range(count):
            if not self.state.draw_pile:
                # Shuffle discard into draw
                if not self.state.discard_pile:
                    break
                self.state.draw_pile = self.state.discard_pile.copy()
                self.state.discard_pile.clear()
                self._shuffle_draw_pile()
                # Trigger registry-based onShuffle relics (Sundial)
                execute_relic_triggers("onShuffle", self.state)

            if self.state.draw_pile:
                card_id = self.state.draw_pile.pop()
                self.state.hand.append(card_id)
                drawn.append(card_id)
                execute_power_triggers(
                    "onCardDraw",
                    self.state,
                    self.state.player,
                    {"card_id": card_id, "card": self._get_card(card_id)},
                )

        return drawn

    # =========================================================================
    # Utility Methods
    # =========================================================================

    def _get_card(self, card_id: str) -> Card:
        """Get a Card object from a card ID."""
        # Handle upgrade marker
        base_id = card_id.rstrip('+')
        upgraded = card_id.endswith('+')

        try:
            return get_card(base_id, upgraded)
        except ValueError:
            # Unknown card - create a dummy
            return Card(
                id=card_id,
                name=card_id,
                card_type=CardType.SKILL,
                rarity="COMMON",
                cost=1,
            )

    def _get_potion_target(self, potion_id: str) -> str:
        """Get targeting type for a potion."""
        from .content.potions import PotionTargetType, get_potion_by_id

        potion = get_potion_by_id(potion_id)
        if potion and potion.target_type == PotionTargetType.ENEMY:
            return "enemy"
        return "self"

    def _has_violet_lotus(self) -> bool:
        """Check if player has Violet Lotus relic."""
        return (self.state.relic_counters.get("_violet_lotus", 0) > 0 or
                self.state.has_relic("Violet Lotus") or
                self.state.has_relic("VioletLotus"))

    def _has_barricade(self) -> bool:
        """Check if player has Barricade (relic or power)."""
        return (self.state.relic_counters.get("_barricade", 0) > 0 or
                self.state.has_relic("Barricade") or
                self.state.player.statuses.get("Barricade", 0) > 0)

    def _has_calipers(self) -> bool:
        """Check if player has Calipers relic."""
        return self.state.has_relic("Calipers")

    def _apply_status(self, target: Union[EntityState, EnemyCombatState], status: str, amount: int) -> None:
        """Apply status to target using canonical IDs and onApplyPower hooks."""
        resolved_status = resolve_power_id(status)

        if resolved_status in ("Weak", "Vulnerable", "Frail", "Poison", "Constricted"):
            artifact = target.statuses.get("Artifact", 0)
            if artifact > 0:
                artifact -= 1
                if artifact <= 0:
                    del target.statuses["Artifact"]
                else:
                    target.statuses["Artifact"] = artifact
                return

        target.statuses[resolved_status] = target.statuses.get(resolved_status, 0) + amount

        trigger_data = {"power_id": resolved_status, "target": target}
        execute_power_triggers("onApplyPower", self.state, self.state.player, trigger_data)
        for enemy in self.state.enemies:
            if enemy.hp <= 0:
                continue
            execute_power_triggers("onApplyPower", self.state, enemy, trigger_data)

    def _apply_debuff_to_player(self, debuff: str, amount: int):
        """Apply a debuff to the player, checking Artifact first."""
        self._apply_status(self.state.player, debuff, amount)

    def _has_runic_pyramid(self) -> bool:
        """Check if player has Runic Pyramid relic."""
        return (self.state.relic_counters.get("_runic_pyramid", 0) > 0 or
                self.state.has_relic("Runic Pyramid"))

    # =========================================================================
    # Boss Mechanic Helpers
    # =========================================================================

    def _check_time_eater_numen(self):
        """Check if Time Eater should trigger Numen (12-card counter)."""
        # If Time Warp power is active, it handles the 12-card trigger
        for enemy in self.state.enemies:
            if enemy.hp > 0 and "Time Warp" in enemy.statuses:
                return

        # Find Time Eater enemy
        time_eater = None
        time_eater_idx = None
        for idx, enemy in enumerate(self.state.enemies):
            enemy_id = str(enemy.id)
            if enemy.hp > 0 and "TimeEater" in enemy_id:
                time_eater = enemy
                time_eater_idx = idx
                break

        if not time_eater:
            return

        # Check if 12 cards have been played this turn
        if self.state.cards_played_this_turn >= 12:
            # Trigger Numen: end turn, gain 2 strength, heal to 50%
            self.log.log(self.state.turn, "time_eater_numen", cards_played=self.state.cards_played_this_turn)

            # Gain 2 Strength
            time_eater.statuses["Strength"] = time_eater.statuses.get("Strength", 0) + 2

            # Heal to 50% max HP (only if below 50%)
            half_hp = time_eater.max_hp // 2
            if time_eater.hp < half_hp:
                heal_amount = half_hp - time_eater.hp
                time_eater.hp = half_hp
                self.log.log(self.state.turn, "time_eater_heal", amount=heal_amount)

            # At A19+, also trigger Beat of Death (1 damage per card played)
            if self.enemy_objects and time_eater_idx < len(self.enemy_objects):
                real_enemy = self.enemy_objects[time_eater_idx]
                if hasattr(real_enemy, 'ascension') and real_enemy.ascension >= 19:
                    # Beat of Death: 1 damage per card played
                    beat_damage = self.state.cards_played_this_turn
                    self.state.player.hp -= beat_damage
                    self.state.total_damage_taken += beat_damage
                    self.log.log(self.state.turn, "beat_of_death", damage=beat_damage)

            # Reset counter
            self.state.cards_played_this_turn = 0

            # End player turn immediately
            self.end_turn()

    def _check_awakened_one_rebirth(self, enemy: EnemyCombatState) -> bool:
        """Check if Awakened One should rebirth (phase 1 -> phase 2)."""
        enemy_id = str(enemy.id)
        if "AwakenedOne" not in enemy_id:
            return False

        # Find the real Enemy object
        real_enemy = self._get_real_enemy(enemy)
        if real_enemy is None:
            return False

        # Check if should rebirth
        if hasattr(real_enemy, 'should_rebirth') and real_enemy.should_rebirth():
            # Trigger rebirth
            real_enemy.trigger_rebirth()

            # Update combat state enemy with new stats
            enemy.hp = real_enemy.state.current_hp
            enemy.max_hp = real_enemy.state.max_hp
            enemy.statuses = dict(real_enemy.state.powers)

            # Clear debuffs
            for debuff in ["Weak", "Vulnerable", "Frail", "Poison"]:
                enemy.statuses.pop(debuff, None)

            # Gain phase 2 powers per ascension
            if hasattr(real_enemy, 'ascension'):
                if real_enemy.ascension >= 19:
                    enemy.statuses["Regen"] = 15
                    enemy.statuses["Curiosity"] = 2
                else:
                    enemy.statuses["Regen"] = 10
                    enemy.statuses["Curiosity"] = 1

            self.log.log(self.state.turn, "awakened_one_rebirth", new_hp=enemy.hp)
            return True

        return False

    def _check_guardian_mode_shift(self, enemy: EnemyCombatState, damage_taken: int):
        """Check if Guardian should shift modes and increment threshold."""
        enemy_id = str(enemy.id)
        if "Guardian" not in enemy_id:
            return

        # Find the real Enemy object
        real_enemy = self._get_real_enemy(enemy)
        if real_enemy is None or not hasattr(real_enemy, 'take_damage'):
            return

        # Get damage before shift
        old_threshold = getattr(real_enemy, 'mode_shift_damage', 30)
        was_offensive = getattr(real_enemy, 'offensive_mode', True)

        # Track damage and check for mode shift
        real_enemy.take_damage(damage_taken)

        # After shift to defensive, increment threshold by 10
        is_offensive_now = getattr(real_enemy, 'offensive_mode', True)
        if was_offensive and not is_offensive_now:
            # Just shifted to defensive, increment threshold by 10
            if hasattr(real_enemy, 'mode_shift_damage'):
                real_enemy.mode_shift_damage += 10
                self.log.log(self.state.turn, "guardian_threshold_increase",
                           old_threshold=old_threshold,
                           new_threshold=real_enemy.mode_shift_damage)

    def _handle_reptomancer_spawn(self, enemy: EnemyCombatState, num_daggers: int):
        """Handle Reptomancer dagger spawning."""
        # Find the real Enemy object
        real_enemy = self._get_real_enemy(enemy)
        if real_enemy is None or not hasattr(real_enemy, 'spawn_daggers'):
            return

        # Get dagger spawn info
        dagger_list = real_enemy.spawn_daggers()

        # Create dagger enemies
        for dagger_info in dagger_list:
            # Create a SnakeDagger enemy
            from .content.enemies import SnakeDagger, create_enemy
            dagger = EnemyCombatState(
                hp=25,  # SnakeDagger HP
                max_hp=25,
                block=0,
                statuses={},
                id="Dagger",
                name="Snake Dagger",
                enemy_type="NORMAL",
                move_id=-1,
                move_damage=0,
                move_hits=1,
                move_block=0,
                move_effects={},
            )
            self.state.enemies.append(dagger)
            # Roll initial move
            self._roll_enemy_move(dagger)

        self.log.log(self.state.turn, "reptomancer_spawn", daggers=len(dagger_list))

    def _handle_collector_spawn(self, enemy: EnemyCombatState, num_torchheads: int):
        """Handle Collector TorchHead spawning."""
        # Create TorchHead enemies
        for _ in range(num_torchheads):
            torchhead = EnemyCombatState(
                hp=40,  # TorchHead HP
                max_hp=40,
                block=0,
                statuses={},
                id="TorchHead",
                name="Torch Head",
                enemy_type="NORMAL",
                move_id=-1,
                move_damage=7,  # TorchHead Tackle damage
                move_hits=1,
                move_block=0,
                move_effects={},
            )
            self.state.enemies.append(torchhead)
            # Roll initial move
            self._roll_enemy_move(torchhead)

        self.log.log(self.state.turn, "collector_spawn", torchheads=num_torchheads)

    # =========================================================================
    # State Observation
    # =========================================================================

    def get_state_dict(self) -> Dict[str, Any]:
        """Get current combat state as a dictionary."""
        return {
            "turn": self.state.turn,
            "phase": self.phase.value,
            "player": {
                "hp": self.state.player.hp,
                "max_hp": self.state.player.max_hp,
                "energy": self.state.energy,
                "max_energy": self.state.max_energy,
                "block": self.state.player.block,
                "stance": self.state.stance,
                "mantra": self.state.mantra,
                "powers": dict(self.state.player.statuses),
            },
            "enemies": [
                {
                    "id": e.id,
                    "name": e.name,
                    "hp": e.hp,
                    "max_hp": e.max_hp,
                    "block": e.block,
                    "intent_damage": e.move_damage if e.move_damage > 0 else None,
                    "intent_hits": e.move_hits if e.move_damage > 0 else None,
                    "powers": dict(e.statuses),
                }
                for e in self.state.enemies
            ],
            "hand": list(self.state.hand),
            "draw_pile_size": len(self.state.draw_pile),
            "discard_pile_size": len(self.state.discard_pile),
            "exhaust_pile_size": len(self.state.exhaust_pile),
            "combat_over": self.state.combat_over,
            "player_won": self.state.player_won,
        }


# =============================================================================
# FACTORY FUNCTIONS
# =============================================================================

def create_combat_from_enemies(
    enemies: List[Enemy],
    player_hp: int,
    player_max_hp: int,
    deck: List[str],
    energy: int = 3,
    relics: List[str] = None,
    potions: List[str] = None,
    ascension: int = 0,
    bottled_cards: Dict[str, str] = None,
) -> CombatEngine:
    """
    Create a combat engine from Enemy objects.

    Args:
        enemies: List of Enemy objects
        player_hp: Current player HP
        player_max_hp: Maximum player HP
        deck: List of card IDs
        energy: Base energy per turn
        relics: List of relic IDs
        potions: List of potion IDs
        ascension: Ascension level

    Returns:
        Initialized CombatEngine
    """
    relics = relics or []
    potions = potions or ["", "", ""]

    # Convert Enemy objects to EnemyCombatState
    enemy_states = []
    for enemy in enemies:
        enemy_combat = EnemyCombatState(
            hp=enemy.state.current_hp,
            max_hp=enemy.state.max_hp,
            block=enemy.state.block,
            statuses=dict(enemy.state.powers),
            id=enemy.ID,
            name=enemy.NAME,
            enemy_type=str(enemy.TYPE.value) if hasattr(enemy.TYPE, 'value') else str(enemy.TYPE),
            move_history=list(enemy.state.move_history),
            first_turn=enemy.state.first_turn,
        )

        # Copy move info if available
        if enemy.state.next_move:
            move = enemy.state.next_move
            enemy_combat.move_id = move.move_id
            enemy_combat.move_damage = move.base_damage
            enemy_combat.move_hits = move.hits
            enemy_combat.move_block = move.block
            enemy_combat.move_effects = dict(move.effects) if move.effects else {}

        if enemy.state.strength != 0:
            enemy_combat.statuses["Strength"] = enemy.state.strength

        enemy_states.append(enemy_combat)

    # Create combat state
    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_max_hp,
        enemies=enemy_states,
        deck=deck,
        energy=energy,
        max_energy=energy,
        relics=relics,
        potions=potions,
        bottled_cards=bottled_cards or {},
    )

    # Check relic flags
    if "VioletLotus" in relics or "Violet Lotus" in relics:
        state.relic_counters["_violet_lotus"] = 1
    if "Barricade" in relics or any("Barricade" in r for r in relics):
        state.relic_counters["_barricade"] = 1
    if "Runic Pyramid" in relics:
        state.relic_counters["_runic_pyramid"] = 1

    engine = CombatEngine(state)
    engine.enemy_objects = list(enemies)
    return engine


def create_simple_combat(
    enemy_id: str,
    enemy_hp: int,
    enemy_damage: int = 6,
    player_hp: int = 80,
    deck: List[str] = None,
) -> CombatEngine:
    """
    Create a simple combat for testing.

    Args:
        enemy_id: Enemy identifier
        enemy_hp: Enemy HP
        enemy_damage: Enemy base damage
        player_hp: Player HP
        deck: Card list (uses starter deck if None)

    Returns:
        Initialized CombatEngine
    """
    if deck is None:
        deck = [
            "Strike_P", "Strike_P", "Strike_P", "Strike_P",
            "Defend_P", "Defend_P", "Defend_P", "Defend_P",
            "Eruption", "Vigilance"
        ]

    enemy = EnemyCombatState(
        hp=enemy_hp,
        max_hp=enemy_hp,
        id=enemy_id,
        name=enemy_id,
        enemy_type="NORMAL",
        move_damage=enemy_damage,
        move_hits=1,
        first_turn=True,
    )

    state = create_combat(
        player_hp=player_hp,
        player_max_hp=player_hp,
        enemies=[enemy],
        deck=deck,
        energy=3,
        max_energy=3,
    )

    return CombatEngine(state)


# =============================================================================
# TESTING
# =============================================================================

if __name__ == "__main__":
    print("=== Combat Engine Test ===\n")

    # Create a simple combat
    engine = create_simple_combat(
        enemy_id="JawWorm",
        enemy_hp=42,
        enemy_damage=11,
        player_hp=80,
    )

    # Start combat
    engine.start_combat()

    print("Initial state:")
    state = engine.get_state_dict()
    print(f"  Player: {state['player']['hp']} HP, {state['player']['energy']} energy")
    print(f"  Hand: {state['hand']}")
    print(f"  Enemy: {state['enemies'][0]['name']} - {state['enemies'][0]['hp']} HP")

    # Play a few turns
    max_turns = 10
    turn = 0

    while not engine.is_combat_over() and turn < max_turns:
        turn += 1
        print(f"\n--- Turn {engine.state.turn} ---")

        actions = engine.get_legal_actions()
        print(f"Available actions: {len(actions)}")

        # Play cards until we can't
        cards_played = 0
        for action in actions:
            if isinstance(action, PlayCard):
                result = engine.execute_action(action)
                if result.get("success"):
                    print(f"Played: {result['card']}")
                    cards_played += 1
                    if engine.is_combat_over():
                        break

        # End turn if not over
        if not engine.is_combat_over():
            engine.execute_action(EndTurn())

    # Get result
    if engine.is_combat_over():
        result = engine.get_result()
        print(f"\n=== Combat Result ===")
        print(f"Victory: {result.victory}")
        print(f"HP remaining: {result.hp_remaining}")
        print(f"HP lost: {result.hp_lost}")
        print(f"Turns: {result.turns}")
        print(f"Cards played: {result.cards_played}")
        print(f"Damage dealt: {result.damage_dealt}")
    else:
        print("\nCombat did not finish within max turns")
