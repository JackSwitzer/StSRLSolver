"""
Combat Execution System - Runs full combats for Slay the Spire RL.

This module provides the CombatRunner class that executes complete combats
with proper turn structure, damage calculation, and enemy AI. It integrates
with the existing CombatState for tree search compatibility.

Combat Flow:
1. Initialize combat from RunState (deck, HP, relics, potions)
2. Set up enemies with HP and initial moves
3. Shuffle deck and draw initial hand
4. Combat loop:
   - Player turn: reset energy, start-of-turn effects, player actions
   - End player turn: discard hand, end-of-turn effects
   - Enemy turn: execute moves, roll next moves
   - Check victory/defeat
5. Return CombatResult with stats
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple, Union, Any, TYPE_CHECKING
from enum import Enum

from ..state.combat import (
    CombatState, EntityState, EnemyCombatState,
    PlayCard, UsePotion, EndTurn, SelectScryDiscard, Action,
    create_combat, create_enemy,
)
from ..state.rng import Random, GameRNG
from ..state.run import RunState
from ..calc.damage import (
    calculate_damage, calculate_block, calculate_incoming_damage,
    WRATH_MULT, DIVINITY_MULT,
)
from ..content.cards import Card, CardType, CardTarget, get_card, ALL_CARDS
from ..content.enemies import Enemy, Intent, MoveInfo, EnemyType
from ..content.powers import resolve_power_id
from ..registry import execute_relic_triggers, execute_power_triggers, RelicContext
from ..effects.orbs import trigger_orb_start_of_turn
from ..combat_engine import CombatEngine, CombatPhase as EngineCombatPhase, create_combat_from_enemies


if TYPE_CHECKING:
    from ..content.enemies import Enemy as EnemyClass


# =============================================================================
# Combat Result
# =============================================================================

@dataclass
class CombatResult:
    """Result of a completed combat."""
    victory: bool
    player_hp_remaining: int
    player_max_hp: int
    turns_taken: int
    cards_played: List[str] = field(default_factory=list)
    damage_dealt: int = 0
    damage_taken: int = 0
    block_gained: int = 0
    enemies_killed: int = 0
    potions_used: List[str] = field(default_factory=list)

    @property
    def hp_lost(self) -> int:
        """Total HP lost during combat."""
        return self.player_max_hp - self.player_hp_remaining if self.victory else self.player_max_hp

    @property
    def hp_percent_remaining(self) -> float:
        """HP remaining as a percentage of max."""
        return self.player_hp_remaining / self.player_max_hp if self.player_max_hp > 0 else 0.0


class CombatPhase(Enum):
    """Combat phases."""
    PLAYER_TURN_START = "PLAYER_TURN_START"
    PLAYER_TURN = "PLAYER_TURN"
    PLAYER_TURN_END = "PLAYER_TURN_END"
    ENEMY_TURN = "ENEMY_TURN"
    COMBAT_END = "COMBAT_END"


# =============================================================================
# Card Registry Wrapper
# =============================================================================

def build_card_registry() -> Dict[str, dict]:
    """Build a card registry dict for CombatState.get_legal_actions()."""
    registry = {}
    for card_id, card in ALL_CARDS.items():
        target_type = "self"
        if card.target == CardTarget.ENEMY:
            target_type = "enemy"
        elif card.target == CardTarget.ALL_ENEMY:
            target_type = "all_enemies"

        registry[card_id] = {
            "cost": card.cost,
            "target": target_type,
            "type": card.card_type.value,
        }
        # Also add upgraded version
        registry[card_id + "+"] = {
            "cost": card.upgrade_cost if card.upgrade_cost is not None else card.cost,
            "target": target_type,
            "type": card.card_type.value,
        }
    return registry


CARD_REGISTRY = build_card_registry()


# =============================================================================
# Combat Runner Compatibility Shim (CONS-002A/CONS-002B)
# =============================================================================



class CombatRunner:
    """Compatibility facade that routes combat execution through CombatEngine."""

    ENERGY_RELICS = (
        "Fusion Hammer",
        "Ectoplasm",
        "Cursed Key",
        "Busted Crown",
        "Sozu",
        "Philosopher's Stone",
        "Mark of Pain",
        "Nuclear Battery",
        "Velvet Choker",
        "Runic Dome",
        "Snecko Eye",
    )

    PHASE_MAP = {
        EngineCombatPhase.NOT_STARTED: CombatPhase.PLAYER_TURN_START,
        EngineCombatPhase.PLAYER_TURN: CombatPhase.PLAYER_TURN,
        EngineCombatPhase.ENEMY_TURN: CombatPhase.ENEMY_TURN,
        EngineCombatPhase.COMBAT_OVER: CombatPhase.COMBAT_END,
    }

    def __init__(
        self,
        run_state: RunState,
        enemies: List[Enemy],
        shuffle_rng: Random,
        card_rng: Optional[Random] = None,
        ai_rng: Optional[Random] = None,
    ):
        self.run_state = run_state
        self.enemies = enemies
        self.shuffle_rng = shuffle_rng
        self.card_rng = card_rng or shuffle_rng.copy()
        self.ai_rng = ai_rng or shuffle_rng.copy()

        energy = self._compute_base_energy(run_state)
        potions = [slot.potion_id or "" for slot in run_state.potion_slots]

        self.engine = create_combat_from_enemies(
            enemies=enemies,
            player_hp=run_state.current_hp,
            player_max_hp=run_state.max_hp,
            deck=run_state.get_deck_card_ids(),
            energy=energy,
            relics=run_state.get_relic_ids(),
            potions=potions,
            ascension=run_state.ascension,
            bottled_cards=run_state.get_bottled_cards(),
        )

        # Preserve explicit RNG ownership for deterministic parity.
        self.engine.shuffle_rng = self.shuffle_rng
        self.engine.card_rng = self.card_rng
        self.engine.ai_rng = self.ai_rng

        self.state = self.engine.state

        # Compatibility attributes expected by legacy tests.
        self.phase = CombatPhase.PLAYER_TURN_START
        self.combat_over = False
        self.victory = False
        self.cards_played: List[str] = []
        self.total_damage_dealt = 0
        self.total_damage_taken = 0
        self.total_block_gained = 0
        self.potions_used: List[str] = []
        self.enemies_killed = 0

        self._setup_combat()

    @classmethod
    def _compute_base_energy(cls, run_state: RunState) -> int:
        base_energy = 3
        for relic_id in cls.ENERGY_RELICS:
            if run_state.has_relic(relic_id):
                base_energy += 1
        return base_energy

    def _sync_runtime_state(self) -> None:
        self.state = self.engine.state
        self.phase = self.PHASE_MAP.get(self.engine.phase, CombatPhase.PLAYER_TURN_START)
        self.combat_over = self.state.combat_over
        self.victory = self.state.player_won
        self.cards_played = list(self.engine.cards_played_sequence)
        self.total_damage_dealt = self.state.total_damage_dealt
        self.total_damage_taken = self.state.total_damage_taken
        self.total_block_gained = getattr(self.state, "total_block_gained", 0)
        self.enemies_killed = sum(1 for enemy in self.state.enemies if enemy.hp <= 0)

    def _setup_combat(self):
        """Set up combat and start the first turn.

        Bag of Preparation and other atBattleStart draw effects are executed by
        CombatEngine.start_combat() through the relic registry.
        """
        self.engine.start_combat()

        # Legacy CombatRunner kept first-turn Anchor block; preserve compatibility.
        if self.state.has_relic("Anchor") and self.state.turn == 1:
            self.state.player.block = max(self.state.player.block, 10)

        self._sync_runtime_state()

    def get_legal_actions(self) -> List[Action]:
        return self.engine.get_legal_actions()

    def _default_action(self) -> Action:
        actions = self.get_legal_actions()
        if not actions:
            return EndTurn()
        card_actions = [action for action in actions if isinstance(action, PlayCard)]
        if card_actions:
            return card_actions[0]
        return EndTurn()

    def run(self, action_provider=None) -> CombatResult:
        while not self.combat_over:
            if self.phase == CombatPhase.PLAYER_TURN:
                action = action_provider(self) if action_provider else self._default_action()
                self.execute_action(action)
            else:
                # Engine owns phase transitions.
                self._sync_runtime_state()
                if self.phase == CombatPhase.COMBAT_END:
                    break

        player_hp_remaining = self.state.player.hp if self.victory else 0
        return CombatResult(
            victory=self.victory,
            player_hp_remaining=player_hp_remaining,
            player_max_hp=self.state.player.max_hp,
            turns_taken=self.state.turn,
            cards_played=self.cards_played.copy(),
            damage_dealt=self.total_damage_dealt,
            damage_taken=self.total_damage_taken,
            block_gained=self.total_block_gained,
            enemies_killed=self.enemies_killed,
            potions_used=self.potions_used.copy(),
        )

    def execute_action(self, action: Action) -> Dict[str, Any]:
        if isinstance(action, EndTurn):
            self._end_player_turn()
            return {"action": "end_turn"}
        if isinstance(action, PlayCard):
            return self.play_card(action.card_idx, action.target_idx)
        if isinstance(action, UsePotion):
            return self.use_potion(action.potion_idx, action.target_idx)
        if isinstance(action, SelectScryDiscard):
            return self.execute_scry_selection(action.discard_indices)
        return {"success": False, "error": "Unknown action type"}

    def execute_scry_selection(self, discard_indices: tuple) -> Dict[str, Any]:
        if not self.state.pending_scry_selection:
            return {"success": False, "error": "No pending scry selection"}

        cards = self.state.pending_scry_cards
        kept: List[str] = []
        discarded: List[str] = []

        for idx, card_id in enumerate(cards):
            if idx in discard_indices:
                self.state.discard_pile.append(card_id)
                discarded.append(card_id)
            else:
                kept.append(card_id)

        for card_id in reversed(kept):
            self.state.draw_pile.append(card_id)

        self.state.pending_scry_cards = []
        self.state.pending_scry_selection = False

        execute_power_triggers(
            "onScry",
            self.state,
            self.state.player,
            {"cards_scried": len(cards)},
        )
        self._sync_runtime_state()
        return {"success": True, "action": "scry_selection", "kept": kept, "discarded": discarded}

    def play_card(self, card_idx: int, target_idx: int = -1, **kwargs) -> Dict[str, Any]:
        if "target_index" in kwargs and target_idx == -1:
            target_idx = kwargs["target_index"]
        result = self.engine.play_card(hand_index=card_idx, target_index=target_idx)
        self._sync_runtime_state()
        return result

    def use_potion(
        self,
        potion_idx: Optional[int] = None,
        target_idx: int = -1,
        **kwargs,
    ) -> Dict[str, Any]:
        if potion_idx is None:
            potion_idx = kwargs.get("potion_index")
        if potion_idx is None:
            return {"success": False, "error": "Missing potion index"}
        if "target_index" in kwargs and target_idx == -1:
            target_idx = kwargs["target_index"]

        result = self.engine.use_potion(potion_index=potion_idx, target_index=target_idx)
        if result.get("success") and result.get("potion"):
            self.potions_used.append(result["potion"])
        self._sync_runtime_state()
        return result

    def _draw_cards(self, count: int) -> List[str]:
        drawn = self.engine._draw_cards(count)
        void_draws = sum(1 for card_id in drawn if card_id in {"Void", "Void+"})
        if void_draws > 0:
            self.state.energy = max(0, self.state.energy - void_draws)
        self._sync_runtime_state()
        return drawn


    def _apply_status(self, target: Union[EntityState, EnemyCombatState], status: str, amount: int):
        resolved_status = resolve_power_id(status)
        vulnerable_before = target.statuses.get("Vulnerable", 0)
        self.engine._apply_status(target, status, amount)

        if (
            resolved_status == "Vulnerable"
            and isinstance(target, EnemyCombatState)
            and self.state.has_relic("Champion Belt")
            and target.statuses.get("Vulnerable", 0) > vulnerable_before
        ):
            target.statuses["Weak"] = target.statuses.get("Weak", 0) + 1

        self._sync_runtime_state()


    def _trigger_was_hp_lost(self, hp_loss: int):
        if hp_loss <= 0:
            return
        execute_relic_triggers("wasHPLost", self.state, {"hp_lost": hp_loss})
        execute_power_triggers(
            "wasHPLost",
            self.state,
            self.state.player,
            {
                "damage": hp_loss,
                "unblocked": True,
                "is_self_damage": False,
                "damage_type": "NORMAL",
            },
        )
        self._sync_runtime_state()

    def _calculate_player_damage(self, base: int, target: Optional[EnemyCombatState]) -> int:
        strength = self.state.player.statuses.get("Strength", 0)
        vigor = self.state.player.statuses.get("Vigor", 0)
        weak = self.state.player.statuses.get("Weak", 0) > 0
        vuln = bool(target and target.statuses.get("Vulnerable", 0) > 0)

        # Pen Nib compatibility behavior: every 10th attack doubles damage.
        pen_nib = False
        if self.state.has_relic("Pen Nib"):
            counter = self.state.get_relic_counter("Pen Nib", 0)
            if counter >= 9:
                pen_nib = True
                self.state.set_relic_counter("Pen Nib", 0)
            else:
                self.state.set_relic_counter("Pen Nib", counter + 1)

        damage = calculate_damage(
            base=base,
            strength=strength,
            vigor=vigor,
            weak=weak,
            pen_nib=pen_nib,
            vuln=vuln,
        )

        if self.state.has_relic("Boot") and 0 < damage < 5:
            damage = 5

        if vigor > 0 and self.state.attacks_played_this_turn == 1:
            self.state.player.statuses["Vigor"] = 0

        return damage

    def _trigger_end_of_turn_hand_cards(self):
        from ..effects.registry import EffectContext, execute_effect

        for card_id in self.state.hand.copy():
            card = self.engine._get_card(card_id)
            if card is None:
                continue

            end_turn_effects = [
                effect_name
                for effect_name in card.effects
                if effect_name.startswith("end_of_turn_")
            ]
            if not end_turn_effects:
                continue

            context = EffectContext(
                state=self.state,
                card=card,
                is_upgraded=card.upgraded,
                magic_number=card.magic_number,
            )
            for effect_name in end_turn_effects:
                execute_effect(effect_name, context)
            if self.state.player.hp <= 0:
                return

    def _trigger_end_of_turn(self):
        execute_relic_triggers("onPlayerEndTurn", self.state)
        execute_power_triggers("atEndOfTurnPreEndTurnCards", self.state, self.state.player)
        execute_power_triggers("atEndOfTurn", self.state, self.state.player)
        for enemy in self.state.enemies:
            if enemy.hp > 0:
                execute_power_triggers("atEndOfTurn", self.state, enemy)

        if self.state.stance == "Divinity":
            self.engine._change_stance(self.engine._parse_stance("Neutral"))
        self._sync_runtime_state()

    def _do_enemy_turns(self):
        self.engine._do_enemy_turns()
        self._sync_runtime_state()

    def _end_player_turn(self):
        # Default path delegates full turn transition to CombatEngine.
        method_func = getattr(self._do_enemy_turns, "__func__", None)
        if method_func is CombatRunner._do_enemy_turns:
            self.engine.end_turn()
            self._sync_runtime_state()
            return

        # Compatibility path used when tests monkeypatch _do_enemy_turns.
        self.phase = CombatPhase.PLAYER_TURN_END

        self._trigger_end_of_turn_hand_cards()
        if self.state.player.hp <= 0:
            self.state.player.hp = 0
            self.engine._end_combat(player_won=False)
            self._sync_runtime_state()
            return

        self.engine._discard_hand()
        self._sync_runtime_state()

        self._trigger_end_of_turn()
        if self.state.player.hp <= 0:
            self.state.player.hp = 0
            self.engine._end_combat(player_won=False)
            self._sync_runtime_state()
            return

        if not self.combat_over:
            self._do_enemy_turns()

    def _check_combat_end(self):
        self.engine._check_combat_end()
        self._sync_runtime_state()

    def _trigger_post_combat_relics(self):
        execute_relic_triggers("onVictory", self.state)
        self._sync_runtime_state()

# =============================================================================
# Encounter Creation
# =============================================================================

from ..content.enemies import (
    Enemy as EnemyObj, ENEMY_CLASSES as _ALL_ENEMIES, create_enemy as _create_enemy,
    # Act 1
    JawWorm, Cultist, Louse, LouseNormal, LouseDefensive, FungiBeast,
    AcidSlimeM, SpikeSlimeM, AcidSlimeL, SpikeSlimeL, AcidSlimeS, SpikeSlimeS,
    Looter, SlaverBlue, SlaverRed,
    GremlinNob, Lagavulin, Sentries,
    SlimeBoss, TheGuardian, Hexaghost,
    # Act 1 minions
    GremlinFat, GremlinThief, GremlinTsundere, GremlinWarrior, GremlinWizard,
    # Act 2
    Chosen, Byrd, Centurion, Healer, Snecko, SnakePlant, Mugger,
    ShelledParasite, SphericGuardian, BanditBear, BanditLeader, BanditPointy,
    GremlinLeader, BookOfStabbing, Taskmaster,
    Champ, TheCollector, BronzeAutomaton,
    # Act 3
    Maw, Darkling, OrbWalker, Spiker, Repulsor, WrithingMass, Transient,
    Exploder, SpireGrowth, SnakeDagger,
    GiantHead, Nemesis, Reptomancer,
    AwakenedOne, TimeEater, Donu, Deca,
    # Act 4
    SpireShield, SpireSpear, CorruptHeart,
    # Minions
    TorchHead, BronzeOrb,
)

ENEMY_CLASSES: Dict[str, type] = _ALL_ENEMIES


def _make(cls, ai_rng, asc, hp_rng, **kw):
    """Create a single Enemy instance."""
    return cls(ai_rng=ai_rng, ascension=asc, hp_rng=hp_rng, **kw)


def _make_louse(ai_rng, asc, hp_rng):
    """Create a random red/green louse using ai_rng to pick color (matches Java)."""
    return _make(Louse, ai_rng, asc, hp_rng, is_red=ai_rng.random_boolean())


def _make_gremlin(ai_rng, asc, hp_rng):
    """Create a random gremlin type (matches Java GremlinGang logic)."""
    gremlin_types = [GremlinFat, GremlinThief, GremlinTsundere, GremlinWarrior, GremlinWizard]
    return _make(gremlin_types[ai_rng.random(len(gremlin_types) - 1)], ai_rng, asc, hp_rng)


def _make_random_shape(ai_rng, asc, hp_rng):
    """Create a random shape (Exploder, Repulsor, or Spiker)."""
    shape_types = [Exploder, Repulsor, Spiker]
    return _make(shape_types[ai_rng.random(2)], ai_rng, asc, hp_rng)


# ---- Encounter factories for encounters requiring RNG-based composition ----

def _enc_2_louse(ai, a, hp):
    return [_make_louse(ai, a, hp), _make_louse(ai, a, hp)]

def _enc_gremlin_gang(ai, a, hp):
    return [_make_gremlin(ai, a, hp) for _ in range(4)]

def _enc_large_slime(ai, a, hp):
    cls = AcidSlimeL if ai.random_boolean() else SpikeSlimeL
    return [_make(cls, ai, a, hp)]

def _enc_lots_of_slimes(ai, a, hp):
    return [_make(AcidSlimeS if ai.random_boolean() else SpikeSlimeS, ai, a, hp) for _ in range(5)]

def _enc_exordium_wildlife(ai, a, hp):
    return [_make(FungiBeast, ai, a, hp), _make_louse(ai, a, hp)]

def _enc_3_louse(ai, a, hp):
    return [_make_louse(ai, a, hp) for _ in range(3)]

def _enc_3_sentries(ai, a, hp):
    return [_make(Sentries, ai, a, hp, position=i) for i in range(3)]

def _enc_gremlin_leader(ai, a, hp):
    return [_make(GremlinLeader, ai, a, hp), _make_gremlin(ai, a, hp), _make_gremlin(ai, a, hp)]

def _enc_3_shapes(ai, a, hp):
    return [_make_random_shape(ai, a, hp) for _ in range(3)]

def _enc_4_shapes(ai, a, hp):
    return [_make_random_shape(ai, a, hp) for _ in range(4)]

def _enc_sphere_and_2_shapes(ai, a, hp):
    return [_make(SphericGuardian, ai, a, hp), _make_random_shape(ai, a, hp), _make_random_shape(ai, a, hp)]


# Master encounter table: maps encounter name strings to factory functions.
# Simple encounters (fixed enemy composition) use lambda; complex ones use named functions above.
ENCOUNTER_TABLE: Dict[str, Any] = {
    # Act 1 Weak
    "Jaw Worm":         lambda ai, a, hp: [_make(JawWorm, ai, a, hp)],
    "Cultist":          lambda ai, a, hp: [_make(Cultist, ai, a, hp)],
    "2 Louse":          _enc_2_louse,
    "Small Slimes":     lambda ai, a, hp: [_make(SpikeSlimeS, ai, a, hp), _make(AcidSlimeS, ai, a, hp)],
    # Act 1 Strong
    "Blue Slaver":      lambda ai, a, hp: [_make(SlaverBlue, ai, a, hp)],
    "Red Slaver":       lambda ai, a, hp: [_make(SlaverRed, ai, a, hp)],
    "Gremlin Gang":     _enc_gremlin_gang,
    "Looter":           lambda ai, a, hp: [_make(Looter, ai, a, hp)],
    "Large Slime":      _enc_large_slime,
    "Lots of Slimes":   _enc_lots_of_slimes,
    "Exordium Thugs":   lambda ai, a, hp: [_make(SlaverBlue, ai, a, hp), _make(SlaverRed, ai, a, hp)],
    "Exordium Wildlife": _enc_exordium_wildlife,
    "3 Louse":          _enc_3_louse,
    "2 Fungi Beasts":   lambda ai, a, hp: [_make(FungiBeast, ai, a, hp), _make(FungiBeast, ai, a, hp)],
    # Act 1 Elites
    "Gremlin Nob":      lambda ai, a, hp: [_make(GremlinNob, ai, a, hp)],
    "Lagavulin":        lambda ai, a, hp: [_make(Lagavulin, ai, a, hp)],
    "3 Sentries":       _enc_3_sentries,
    # Act 1 Bosses
    "Slime Boss":       lambda ai, a, hp: [_make(SlimeBoss, ai, a, hp)],
    "The Guardian":     lambda ai, a, hp: [_make(TheGuardian, ai, a, hp)],
    "Hexaghost":        lambda ai, a, hp: [_make(Hexaghost, ai, a, hp)],
    # Act 2 Weak
    "Spheric Guardian": lambda ai, a, hp: [_make(SphericGuardian, ai, a, hp)],
    "Chosen":           lambda ai, a, hp: [_make(Chosen, ai, a, hp)],
    "Shell Parasite":   lambda ai, a, hp: [_make(ShelledParasite, ai, a, hp)],
    "3 Byrds":          lambda ai, a, hp: [_make(Byrd, ai, a, hp) for _ in range(3)],
    "2 Thieves":        lambda ai, a, hp: [_make(Mugger, ai, a, hp), _make(Looter, ai, a, hp)],
    # Act 2 Strong
    "Chosen and Byrds": lambda ai, a, hp: [_make(Chosen, ai, a, hp), _make(Byrd, ai, a, hp), _make(Byrd, ai, a, hp)],
    "Sentry and Sphere": lambda ai, a, hp: [_make(Sentries, ai, a, hp, position=0), _make(SphericGuardian, ai, a, hp)],
    "Snake Plant":      lambda ai, a, hp: [_make(SnakePlant, ai, a, hp)],
    "Snecko":           lambda ai, a, hp: [_make(Snecko, ai, a, hp)],
    "Centurion and Healer": lambda ai, a, hp: [_make(Centurion, ai, a, hp), _make(Healer, ai, a, hp)],
    "Cultist and Chosen": lambda ai, a, hp: [_make(Cultist, ai, a, hp), _make(Chosen, ai, a, hp)],
    "3 Cultists":       lambda ai, a, hp: [_make(Cultist, ai, a, hp) for _ in range(3)],
    "Shelled Parasite and Fungi": lambda ai, a, hp: [_make(ShelledParasite, ai, a, hp), _make(FungiBeast, ai, a, hp)],
    # Act 2 Elites
    "Gremlin Leader":   _enc_gremlin_leader,
    "Slavers":          lambda ai, a, hp: [_make(SlaverBlue, ai, a, hp), _make(SlaverRed, ai, a, hp), _make(Taskmaster, ai, a, hp)],
    "Book of Stabbing": lambda ai, a, hp: [_make(BookOfStabbing, ai, a, hp)],
    # Act 2 Bosses
    "Automaton":        lambda ai, a, hp: [_make(BronzeAutomaton, ai, a, hp)],
    "Collector":        lambda ai, a, hp: [_make(TheCollector, ai, a, hp)],
    "Champ":            lambda ai, a, hp: [_make(Champ, ai, a, hp)],
    # Act 3 Weak
    "3 Darklings":      lambda ai, a, hp: [_make(Darkling, ai, a, hp) for _ in range(3)],
    "Orb Walker":       lambda ai, a, hp: [_make(OrbWalker, ai, a, hp)],
    "3 Shapes":         _enc_3_shapes,
    # Act 3 Strong
    "Spire Growth":     lambda ai, a, hp: [_make(SpireGrowth, ai, a, hp)],
    "Transient":        lambda ai, a, hp: [_make(Transient, ai, a, hp)],
    "4 Shapes":         _enc_4_shapes,
    "Maw":              lambda ai, a, hp: [_make(Maw, ai, a, hp)],
    "Sphere and 2 Shapes": _enc_sphere_and_2_shapes,
    "Jaw Worm Horde":   lambda ai, a, hp: [_make(JawWorm, ai, a, hp) for _ in range(3)],
    "Writhing Mass":    lambda ai, a, hp: [_make(WrithingMass, ai, a, hp)],
    # Act 3 Elites
    "Giant Head":       lambda ai, a, hp: [_make(GiantHead, ai, a, hp)],
    "Nemesis":          lambda ai, a, hp: [_make(Nemesis, ai, a, hp)],
    "Reptomancer":      lambda ai, a, hp: [_make(Reptomancer, ai, a, hp), _make(SnakeDagger, ai, a, hp), _make(SnakeDagger, ai, a, hp)],
    # Act 3 Bosses
    "Awakened One":     lambda ai, a, hp: [_make(AwakenedOne, ai, a, hp)],
    "Time Eater":       lambda ai, a, hp: [_make(TimeEater, ai, a, hp)],
    "Donu and Deca":    lambda ai, a, hp: [_make(Donu, ai, a, hp), _make(Deca, ai, a, hp)],
    # Act 4
    "Spire Shield and Spire Spear": lambda ai, a, hp: [_make(SpireShield, ai, a, hp), _make(SpireSpear, ai, a, hp)],
    "Corrupt Heart":    lambda ai, a, hp: [_make(CorruptHeart, ai, a, hp)],
}


def create_enemies_from_encounter(
    encounter_name: str,
    ai_rng: Random,
    ascension: int = 0,
    hp_rng: Optional[Random] = None,
) -> List[EnemyObj]:
    """
    Create Enemy instances for a named encounter.

    Args:
        encounter_name: Name from generation/encounters.py (e.g. "2 Louse", "Gremlin Gang")
        ai_rng: RNG for enemy AI decisions
        ascension: Ascension level
        hp_rng: RNG for enemy HP rolls (defaults to ai_rng if None)

    Returns:
        List of Enemy instances for this encounter

    Raises:
        ValueError: If encounter name not found in ENCOUNTER_TABLE
    """
    if hp_rng is None:
        hp_rng = ai_rng
    factory = ENCOUNTER_TABLE.get(encounter_name)
    if factory is None:
        raise ValueError(f"Unknown encounter: {encounter_name!r}. "
                         f"Available: {sorted(ENCOUNTER_TABLE.keys())}")
    return factory(ai_rng, ascension, hp_rng)


# =============================================================================
# Testing
# =============================================================================

if __name__ == "__main__":
    from ..state.run import create_watcher_run
    from ..content.enemies import JawWorm

    print("=== Combat Runner Test ===\n")

    # Create a test run
    run = create_watcher_run("TEST123", ascension=0)
    print(f"Created run: {run}")

    # Create RNGs
    seed = 12345
    shuffle_rng = Random(seed)
    ai_rng = Random(seed + 1)
    hp_rng = Random(seed + 2)

    # Create enemies
    enemies = [JawWorm(ai_rng=ai_rng, ascension=0, hp_rng=hp_rng)]
    print(f"Enemy: {enemies[0].ID} with {enemies[0].state.current_hp} HP")

    # Create combat runner
    runner = CombatRunner(
        run_state=run,
        enemies=enemies,
        shuffle_rng=shuffle_rng,
    )

    print(f"\nInitial state:")
    print(f"  Player HP: {runner.state.player.hp}/{runner.state.player.max_hp}")
    print(f"  Energy: {runner.state.energy}")
    print(f"  Hand: {runner.state.hand}")
    print(f"  Enemy move: {runner.state.enemies[0].move_damage}x{runner.state.enemies[0].move_hits}")

    # Run combat with default heuristic
    result = runner.run()

    print(f"\nCombat result:")
    print(f"  Victory: {result.victory}")
    print(f"  HP remaining: {result.player_hp_remaining}/{result.player_max_hp}")
    print(f"  Turns: {result.turns_taken}")
    print(f"  Cards played: {len(result.cards_played)}")
    print(f"  Damage dealt: {result.damage_dealt}")
    print(f"  Damage taken: {result.damage_taken}")
