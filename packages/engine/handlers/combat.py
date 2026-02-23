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
from ..registry import execute_relic_triggers, execute_power_triggers, RelicContext
from ..effects.orbs import trigger_orb_start_of_turn

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
# Combat Runner
# =============================================================================

class CombatRunner:
    """
    Executes full combats with proper game mechanics.

    Designed to:
    1. Run complete combats from start to finish
    2. Support external action providers (for RL agents)
    3. Integrate with CombatState for tree search
    4. Track stats for EV calculation
    """

    def __init__(
        self,
        run_state: RunState,
        enemies: List[Enemy],
        shuffle_rng: Random,
        card_rng: Optional[Random] = None,
        ai_rng: Optional[Random] = None,
    ):
        """
        Initialize combat from run state.

        Args:
            run_state: Current RunState with deck, HP, relics, potions
            enemies: List of Enemy instances to fight
            shuffle_rng: RNG for deck shuffling
            card_rng: RNG for card random effects (defaults to copy of shuffle_rng)
            ai_rng: RNG for enemy AI (defaults to copy of shuffle_rng)
        """
        self.run_state = run_state
        self.enemies = enemies
        self.shuffle_rng = shuffle_rng
        self.card_rng = card_rng or shuffle_rng.copy()
        self.ai_rng = ai_rng or shuffle_rng.copy()

        # Build deck as list of card IDs
        self.deck_cards = run_state.get_deck_card_ids()

        # Initialize CombatState
        self.state = self._create_combat_state()

        # Combat tracking
        self.phase = CombatPhase.PLAYER_TURN_START
        self.combat_over = False
        self.victory = False

        # Stats tracking
        self.cards_played: List[str] = []
        self.total_damage_dealt = 0
        self.total_damage_taken = 0
        self.total_block_gained = 0
        self.potions_used: List[str] = []
        self.enemies_killed = 0

        # Initialize combat
        self._setup_combat()

    def _create_combat_state(self) -> CombatState:
        """Create initial CombatState from RunState."""
        # Calculate base energy
        base_energy = 3
        if self.run_state.has_relic("Coffee Dripper"):
            pass  # No energy bonus, but can't rest
        if self.run_state.has_relic("Fusion Hammer"):
            base_energy += 1
        if self.run_state.has_relic("Ectoplasm"):
            base_energy += 1
        if self.run_state.has_relic("Cursed Key"):
            base_energy += 1
        if self.run_state.has_relic("Busted Crown"):
            base_energy += 1
        if self.run_state.has_relic("Sozu"):
            base_energy += 1
        if self.run_state.has_relic("Philosopher's Stone"):
            base_energy += 1
        if self.run_state.has_relic("Mark of Pain"):
            base_energy += 1
        if self.run_state.has_relic("Nuclear Battery"):
            base_energy += 1  # Defect only
        if self.run_state.has_relic("Velvet Choker"):
            base_energy += 1
        if self.run_state.has_relic("Runic Dome"):
            base_energy += 1
        if self.run_state.has_relic("Snecko Eye"):
            base_energy += 1
        if self.run_state.has_relic("Runic Pyramid"):
            pass  # No energy bonus, but retain hand

        # Convert enemies to EnemyCombatState
        enemy_states = []
        for enemy in self.enemies:
            enemy_state = EnemyCombatState(
                hp=enemy.state.current_hp,
                max_hp=enemy.state.max_hp,
                block=enemy.state.block,
                statuses=enemy.state.powers.copy(),
                id=enemy.ID,
                move_id=-1,
                move_damage=0,
                move_hits=1,
                move_block=0,
            )
            enemy_states.append(enemy_state)

        # Get potions (as list of IDs, empty string for empty slots)
        potions = [s.potion_id or "" for s in self.run_state.potion_slots]

        # Shuffle deck
        shuffled_deck = self._shuffle_deck(self.deck_cards.copy())

        return create_combat(
            player_hp=self.run_state.current_hp,
            player_max_hp=self.run_state.max_hp,
            enemies=enemy_states,
            deck=shuffled_deck,
            energy=base_energy,
            max_energy=base_energy,
            relics=self.run_state.get_relic_ids(),
            potions=potions,
            bottled_cards=self.run_state.get_bottled_cards(),
        )

    def _shuffle_deck(self, deck: List[str]) -> List[str]:
        """Shuffle deck using Fisher-Yates."""
        n = len(deck)
        for i in range(n - 1, 0, -1):
            j = self.shuffle_rng.random(i)
            deck[i], deck[j] = deck[j], deck[i]
        return deck

    def _setup_combat(self):
        """Set up combat: roll initial enemy moves, trigger start-of-combat effects."""
        # Roll initial moves for all enemies
        for i, enemy in enumerate(self.enemies):
            move = enemy.roll_move()
            self._update_enemy_move(i, move)

        # Execute registry-based atBattleStart triggers
        execute_relic_triggers("atBattleStart", self.state)

        # Trigger any remaining start-of-combat relics not in registry
        self._trigger_start_of_combat_relics()

        # Execute atBattleStartPreDraw triggers (e.g., Pure Water adds Miracle)
        execute_relic_triggers("atBattleStartPreDraw", self.state)

        # Draw initial hand
        draw_count = 5
        if self.state.has_relic("Ring of the Snake"):
            draw_count += 2
        if self.state.has_relic("Snecko Eye"):
            # Snecko Eye randomizes card costs on draw
            pass
        if self.state.has_relic("Bag of Preparation"):
            draw_count += 2

        self._draw_cards(draw_count)

        # Gambling Chip - discard and redraw at combat start
        if self.state.has_relic("Gambling Chip"):
            # Discard entire hand and redraw same number
            hand_size = len(self.state.hand)
            self.state.discard_pile.extend(self.state.hand)
            self.state.hand.clear()
            self._draw_cards(hand_size)

        # Start first turn (without drawing since we already drew)
        self._start_player_turn(first_turn=True)

    def _trigger_start_of_combat_relics(self):
        """Trigger remaining relics not yet in registry that activate at combat start.

        Note: Most atBattleStart relics are now handled via execute_relic_triggers().
        This method only handles relics requiring CombatRunner-specific logic.
        """
        # Pen Nib counter initialization - needs run_state access
        if self.state.has_relic("Pen Nib"):
            counter = self.run_state.get_relic_counter("Pen Nib")
            if counter >= 0:
                self.state.set_relic_counter("Pen Nib", counter)

    def _update_enemy_move(self, enemy_idx: int, move: MoveInfo):
        """Update CombatState enemy with move info."""
        if enemy_idx >= len(self.state.enemies):
            return

        enemy = self.state.enemies[enemy_idx]
        enemy.move_id = move.move_id
        enemy.move_damage = move.base_damage if move.base_damage > 0 else 0
        enemy.move_hits = move.hits
        enemy.move_block = move.block
        enemy.move_effects = move.effects.copy()

    def _draw_cards(self, count: int) -> List[str]:
        """Draw cards from draw pile to hand."""
        drawn = []
        for _ in range(count):
            if not self.state.draw_pile:
                # Shuffle discard into draw
                if not self.state.discard_pile:
                    break
                self.state.draw_pile = self._shuffle_deck(self.state.discard_pile.copy())
                self.state.discard_pile.clear()

                # Trigger registry-based onShuffle relics (Sundial)
                execute_relic_triggers("onShuffle", self.state)

            if self.state.draw_pile:
                card = self.state.draw_pile.pop()

                # Snecko Eye - randomize card cost to 0-3
                if self.state.has_relic("Snecko Eye"):
                    cost = self.card_rng.random(3)  # 0-3 inclusive
                    self.state.card_costs[card] = cost

                self.state.hand.append(card)
                drawn.append(card)

                # Handle Void card (lose 1 energy when drawn)
                if card == "Void" or card == "Void+":
                    self.state.energy = max(0, self.state.energy - 1)


        # Unceasing Top - after drawing, if hand empty and draw pile has cards, draw 1
        if self.state.has_relic("Unceasing Top"):
            if not self.state.hand and self.state.draw_pile:
                card = self.state.draw_pile.pop()
                if self.state.has_relic("Snecko Eye"):
                    cost = self.card_rng.random(3)
                    self.state.card_costs[card] = cost
                self.state.hand.append(card)
                drawn.append(card)

        return drawn

    def _start_player_turn(self, first_turn: bool = False):
        """
        Begin player turn.

        Args:
            first_turn: If True, skip drawing cards (already drawn in setup)
        """
        self.phase = CombatPhase.PLAYER_TURN_START

        # Increment turn counter (starts at 0, so first turn becomes 1)
        self.state.turn += 1

        # Reset energy
        if self.state.has_relic("Ice Cream"):
            # Keep leftover energy
            pass  # Energy persists
        else:
            self.state.energy = self.state.max_energy

        # Remove block (unless Barricade/Blur/Calipers) - skip on first turn
        if not first_turn:
            if not self.state.player.statuses.get("Barricade", 0) > 0:
                blur = self.state.player.statuses.get("Blur", 0)
                if blur > 0:
                    # Blur: retain block, decrement Blur by 1
                    self.state.player.statuses["Blur"] = blur - 1
                    if self.state.player.statuses["Blur"] <= 0:
                        del self.state.player.statuses["Blur"]
                elif self.state.has_relic("Calipers"):
                    # Calipers - Lose only 15 block per turn (only when no Barricade/Blur)
                    self.state.player.block = max(0, self.state.player.block - 15)
                else:
                    self.state.player.block = 0

        # Reset turn counters
        self.state.cards_played_this_turn = 0
        self.state.attacks_played_this_turn = 0
        self.state.skills_played_this_turn = 0
        self.state.discarded_this_turn = 0
        self.state.powers_played_this_turn = 0

        # Execute registry-based atTurnStart triggers (handles counter resets and energy effects)
        execute_relic_triggers("atTurnStart", self.state)

        # Draw cards (skip on first turn since setup already drew)
        if not first_turn and not self.state.player.statuses.get("NoDraw", 0) > 0:
            draw_count = 5
            # Relics that modify draw
            if self.state.has_relic("Snake Skull"):
                pass  # Poison-related, not draw
            self._draw_cards(draw_count)

        # Trigger start-of-turn effects
        self._trigger_start_of_turn()

        self.phase = CombatPhase.PLAYER_TURN

    def _trigger_start_of_turn(self):
        """Trigger start-of-turn power effects using registry system."""
        # Execute atStartOfTurn power triggers for player
        execute_power_triggers("atStartOfTurn", self.state, self.state.player)

        # Execute atStartOfTurn for enemy powers (e.g., Poison deals damage then decrements)
        for enemy in self.state.enemies:
            if not enemy.is_dead:
                execute_power_triggers("atStartOfTurn", self.state, enemy)

        # Execute onEnergyRecharge power triggers (DevaForm, Energized)
        execute_power_triggers("onEnergyRecharge", self.state, self.state.player)

        # NextTurnBlock power (from Self-Forming Clay) - handled in registry
        next_turn_block = self.state.player.statuses.get("NextTurnBlock", 0)
        if next_turn_block > 0:
            self.state.player.block += next_turn_block
            self.total_block_gained += next_turn_block
            del self.state.player.statuses["NextTurnBlock"]

        # Defect orb passives trigger each turn start; Cables bonus is handled
        # by the explicit relic trigger.
        trigger_orb_start_of_turn(self.state, include_cables=False)

    def _trigger_was_hp_lost(self, hp_loss: int):
        """Trigger relics that activate when HP is lost."""
        if hp_loss <= 0:
            return

        # Execute registry-based wasHPLost triggers
        execute_relic_triggers("wasHPLost", self.state, {"hp_lost": hp_loss})

    def _apply_status(self, target: Union[EntityState, EnemyCombatState], status: str, amount: int):
        """Apply a status effect to target."""
        # Check Artifact
        debuffs = {"Weak", "Vulnerable", "Frail", "Poison", "Constricted"}
        if status in debuffs:
            artifact = target.statuses.get("Artifact", 0)
            if artifact > 0:
                target.statuses["Artifact"] = artifact - 1
                if target.statuses["Artifact"] <= 0:
                    del target.statuses["Artifact"]
                return  # Status blocked

        current = target.statuses.get(status, 0)
        target.statuses[status] = current + amount

        # Champion's Belt - when applying Vulnerable, also apply 1 Weak
        if status == "Vulnerable" and self.state.has_relic("Champion Belt"):
            if isinstance(target, EnemyCombatState):
                weak_current = target.statuses.get("Weak", 0)
                target.statuses["Weak"] = weak_current + 1

    def get_legal_actions(self) -> List[Action]:
        """Get all legal actions from current state."""
        return self.state.get_legal_actions(CARD_REGISTRY)

    def run(self, action_provider=None) -> CombatResult:
        """
        Run complete combat.

        Args:
            action_provider: Optional callable(CombatRunner) -> Action
                            If None, uses a simple heuristic.

        Returns:
            CombatResult with combat statistics.
        """
        while not self.combat_over:
            if self.phase == CombatPhase.PLAYER_TURN:
                # Get action
                if action_provider:
                    action = action_provider(self)
                else:
                    action = self._default_action()

                # Execute action
                self.execute_action(action)

                # Check if turn ended
                if isinstance(action, EndTurn):
                    self._end_player_turn()

        return CombatResult(
            victory=self.victory,
            player_hp_remaining=self.state.player.hp if self.victory else 0,
            player_max_hp=self.state.player.max_hp,
            turns_taken=self.state.turn,
            cards_played=self.cards_played.copy(),
            damage_dealt=self.total_damage_dealt,
            damage_taken=self.total_damage_taken,
            block_gained=self.total_block_gained,
            enemies_killed=self.enemies_killed,
            potions_used=self.potions_used.copy(),
        )

    def _default_action(self) -> Action:
        """Simple heuristic for default action selection."""
        actions = self.get_legal_actions()
        if not actions:
            return EndTurn()

        # Prefer playing cards over ending turn
        card_actions = [a for a in actions if isinstance(a, PlayCard)]
        if card_actions:
            # Simple priority: play first available card
            return card_actions[0]

        return EndTurn()

    def execute_action(self, action: Action) -> Dict[str, Any]:
        """
        Execute a single action.

        Args:
            action: Action to execute (PlayCard, UsePotion, EndTurn, SelectScryDiscard)

        Returns:
            Dict with action results
        """
        if isinstance(action, PlayCard):
            return self.play_card(action.card_idx, action.target_idx)
        elif isinstance(action, UsePotion):
            return self.use_potion(action.potion_idx, action.target_idx)
        elif isinstance(action, SelectScryDiscard):
            return self.execute_scry_selection(action.discard_indices)
        elif isinstance(action, EndTurn):
            return {"action": "end_turn"}
        else:
            return {"error": "Unknown action type"}

    def execute_scry_selection(self, discard_indices: tuple) -> Dict[str, Any]:
        """
        Execute a scry discard selection.

        Args:
            discard_indices: Tuple of indices into pending_scry_cards to discard

        Returns:
            Dict with action results
        """
        if not self.state.pending_scry_selection:
            return {"success": False, "error": "No pending scry selection"}

        cards = self.state.pending_scry_cards
        kept = []
        discarded = []

        for i, card in enumerate(cards):
            if i in discard_indices:
                self.state.discard_pile.append(card)
                discarded.append(card)
            else:
                kept.append(card)

        # Put kept cards back on top of draw pile (in reverse order so first is on top)
        for card in reversed(kept):
            self.state.draw_pile.append(card)

        # Clear pending state
        self.state.pending_scry_cards = []
        self.state.pending_scry_selection = False

        return {
            "success": True,
            "action": "scry_selection",
            "kept": kept,
            "discarded": discarded,
        }

    def play_card(self, card_idx: int, target_idx: int = -1) -> Dict[str, Any]:
        """
        Play a card from hand.

        Args:
            card_idx: Index in hand
            target_idx: Target enemy index (-1 for self/no target)

        Returns:
            Dict with card effects
        """
        if card_idx >= len(self.state.hand):
            return {"success": False, "error": "Invalid card index"}

        card_id = self.state.hand[card_idx]
        card = self._get_card(card_id)

        if not card:
            return {"success": False, "error": f"Unknown card: {card_id}"}

        # Check energy cost
        cost = self._get_card_cost(card_id)
        if cost > self.state.energy:
            return {"success": False, "error": "Not enough energy"}

        # Pay energy
        self.state.energy -= cost

        # Remove from hand
        self.state.hand.pop(card_idx)

        # Track
        self.cards_played.append(card_id)
        self.state.cards_played_this_turn += 1
        if card.card_type == CardType.ATTACK:
            self.state.attacks_played_this_turn += 1
        elif card.card_type == CardType.SKILL:
            self.state.skills_played_this_turn += 1
        elif card.card_type == CardType.POWER:
            self.state.powers_played_this_turn += 1

        result = {"success": True, "card": card_id, "effects": []}

        # Get target
        target = None
        if target_idx >= 0 and target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]

        # Apply card effects
        self._apply_card_effects(card, card_id, target, result)

        # Card destination
        if card.exhaust:
            # Strange Spoon - 50% chance card goes to discard instead
            if self.state.has_relic("Strange Spoon") and self.card_rng.random_boolean():
                self.state.discard_pile.append(card_id)
            else:
                self.state.exhaust_pile.append(card_id)
                # Trigger registry-based onExhaust relics (Dead Branch, Charon's Ashes, etc.)
                execute_relic_triggers("onExhaust", self.state, {"card": card})
                # Trigger onExhaust power triggers (Dark Embrace, Feel No Pain)
                execute_power_triggers("onExhaust", self.state, self.state.player, {"card": card})
                if card.id == "Sentinel":
                    self.state.energy += 3 if card.upgraded else 2
        elif card.shuffle_back:
            # Insert at random position in draw pile
            pos = self.shuffle_rng.random(len(self.state.draw_pile)) if self.state.draw_pile else 0
            self.state.draw_pile.insert(pos, card_id)
        else:
            self.state.discard_pile.append(card_id)

        # Trigger registry-based onPlayCard relics
        execute_relic_triggers("onPlayCard", self.state, {"card": card})

        # Trigger onUseCard power triggers (After Image, Choked, Duplication)
        execute_power_triggers("onUseCard", self.state, self.state.player, {"card": card, "card_id": card.id})

        # Trigger onAfterUseCard power triggers (Beat of Death, Slow, Time Warp)
        force_end_turn = False
        after_use_data = {"card": card, "card_id": card.id}
        execute_power_triggers("onAfterUseCard", self.state, self.state.player, after_use_data)
        if after_use_data.get("force_end_turn"):
            force_end_turn = True
        for enemy in self.state.enemies:
            if enemy.is_dead:
                continue
            enemy_trigger = {"card": card, "card_id": card.id}
            execute_power_triggers("onAfterUseCard", self.state, enemy, enemy_trigger)
            if enemy_trigger.get("force_end_turn"):
                force_end_turn = True

        # Trigger onAfterCardPlayed power triggers (Thousand Cuts)
        after_play_data = {"card": card, "card_id": card.id}
        execute_power_triggers("onAfterCardPlayed", self.state, self.state.player, after_play_data)
        for enemy in self.state.enemies:
            if enemy.is_dead:
                continue
            execute_power_triggers("onAfterCardPlayed", self.state, enemy, after_play_data)

        # Unceasing Top - if hand is empty after playing card, draw 1
        if self.state.has_relic("Unceasing Top"):
            if not self.state.hand and self.state.draw_pile:
                self._draw_cards(1)


        # Time Warp can force end of turn after the card resolves
        if force_end_turn:
            self.end_turn()

        # Check combat end
        self._check_combat_end()

        return result

    def _get_card(self, card_id: str) -> Optional[Card]:
        """Get Card object from ID (handles upgraded cards)."""
        base_id = card_id.rstrip("+")
        is_upgraded = card_id.endswith("+")

        if base_id in ALL_CARDS:
            card = ALL_CARDS[base_id].copy()
            if is_upgraded:
                card.upgrade()
            return card
        return None

    def _get_card_cost(self, card_id: str) -> int:
        """Get effective cost for a card."""
        # Check modified cost cache
        if card_id in self.state.card_costs:
            return self.state.card_costs[card_id]

        card = self._get_card(card_id)
        if card:
            return card.current_cost
        return 1

    def _apply_card_effects(self, card: Card, card_id: str, target: Optional[EnemyCombatState], result: dict):
        """Apply all effects of a card."""
        # Calculate damage
        if card.damage > 0:
            hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
            per_hit_damage = self._calculate_player_damage(card.damage, target)

            # Apply stance multiplier
            if self.state.stance == "Wrath":
                per_hit_damage = int(per_hit_damage * WRATH_MULT)
            elif self.state.stance == "Divinity":
                per_hit_damage = int(per_hit_damage * DIVINITY_MULT)

            if card.target == CardTarget.ALL_ENEMY:
                # Hit all enemies
                for enemy in self.state.enemies:
                    if not enemy.is_dead:
                        for _ in range(hits):
                            self._deal_damage_to_enemy(enemy, per_hit_damage)
                            result["effects"].append({
                                "type": "damage", "target": enemy.id, "amount": per_hit_damage
                            })
            elif target:
                # Single target
                for _ in range(hits):
                    if not target.is_dead:
                        self._deal_damage_to_enemy(target, per_hit_damage)
                        result["effects"].append({
                            "type": "damage", "target": target.id, "amount": per_hit_damage
                        })

        # Calculate block
        if card.block > 0:
            block_amount = self._calculate_block(card.block)
            self.state.player.block += block_amount
            self.total_block_gained += block_amount
            result["effects"].append({"type": "block", "amount": block_amount})
            # Trigger onGainBlock power triggers (Juggernaut, Wave of the Hand)
            execute_power_triggers("onGainBlock", self.state, self.state.player, {"block_amount": block_amount})

        # Handle stance changes
        if card.enter_stance:
            self._change_stance(card.enter_stance)
            result["effects"].append({"type": "stance", "stance": card.enter_stance})

        if card.exit_stance:
            self._change_stance("Neutral")
            result["effects"].append({"type": "stance", "stance": "Neutral"})

        # Handle draw effects
        if "draw_1" in card.effects:
            self._draw_cards(1)
            result["effects"].append({"type": "draw", "amount": 1})
        if "draw_2" in card.effects:
            self._draw_cards(2)
            result["effects"].append({"type": "draw", "amount": 2})
        if "draw_cards" in card.effects and card.magic_number > 0:
            self._draw_cards(card.magic_number)
            result["effects"].append({"type": "draw", "amount": card.magic_number})

        # Handle energy gain
        if "gain_1_energy" in card.effects:
            self.state.energy += 1
            result["effects"].append({"type": "energy", "amount": 1})

        # Handle status applications
        for effect in card.effects:
            if effect.startswith("apply_"):
                status = effect.replace("apply_", "").replace("_", " ").title()
                if target:
                    amount = card.magic_number if card.magic_number > 0 else 1
                    self._apply_status(target, status, amount)

        # Handle power cards
        if card.card_type == CardType.POWER:
            self._apply_power_card(card)

    def _calculate_player_damage(self, base: int, target: Optional[EnemyCombatState]) -> int:
        """Calculate player damage output."""
        strength = self.state.player.strength
        vigor = self.state.player.statuses.get("Vigor", 0)
        weak = self.state.player.is_weak
        vuln = target.is_vulnerable if target else False

        # Pen Nib check
        pen_nib = False
        if self.state.has_relic("Pen Nib"):
            counter = self.state.get_relic_counter("Pen Nib", 0)
            if counter >= 9:  # 10th attack triggers
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

        # The Boot - minimum 5 damage on attacks
        if self.state.has_relic("Boot") and 0 < damage < 5:
            damage = 5

        # Consume Vigor after first attack of the turn
        if vigor > 0 and self.state.attacks_played_this_turn == 1:
            self.state.player.statuses["Vigor"] = 0

        return damage

    def _calculate_block(self, base: int) -> int:
        """Calculate player block gain."""
        dexterity = self.state.player.dexterity
        frail = self.state.player.is_frail

        return calculate_block(
            base=base,
            dexterity=dexterity,
            frail=frail,
        )

    def _deal_damage_to_enemy(self, enemy: EnemyCombatState, amount: int):
        """Deal damage to an enemy."""
        # Apply block first
        blocked = min(enemy.block, amount)
        hp_damage = amount - blocked
        enemy.block -= blocked

        # Deal HP damage
        enemy.hp -= hp_damage
        self.total_damage_dealt += hp_damage

        # Check death
        if enemy.hp <= 0:
            enemy.hp = 0
            self.enemies_killed += 1

    def _change_stance(self, new_stance: str):
        """Change player stance."""
        old_stance = self.state.stance

        if old_stance == new_stance:
            return  # No change

        # Exit Calm - gain 2 energy (Violet Lotus adds +1 via relic trigger)
        if old_stance == "Calm":
            self.state.energy += 2

        # Enter Wrath/Calm/Divinity
        self.state.stance = new_stance

        # Enter Divinity - gain energy
        if new_stance == "Divinity":
            self.state.energy += 3

        # Execute onChangeStance relic triggers (Violet Lotus)
        execute_relic_triggers("onChangeStance", self.state, {"new_stance": new_stance, "old_stance": old_stance})

        # Execute onChangeStance power triggers (Mental Fortress, Rushdown)
        execute_power_triggers("onChangeStance", self.state, self.state.player, {"new_stance": new_stance, "old_stance": old_stance})

        # Trigger Flurry of Blows
        self._trigger_flurry_of_blows()

    def _trigger_flurry_of_blows(self):
        """Move Flurry of Blows from discard to hand on stance change."""
        flurries = [c for c in self.state.discard_pile if c.startswith("FlurryOfBlows")]
        for f in flurries:
            self.state.discard_pile.remove(f)
            self.state.hand.append(f)

    def _apply_power_card(self, card: Card):
        """Apply a power card's effects."""
        power_id = card.id
        amount = card.magic_number if card.magic_number > 0 else 1

        # Map power cards to statuses
        power_map = {
            "MentalFortress": ("MentalFortress", amount),
            "Rushdown": ("Rushdown", amount),
            "Nirvana": ("Nirvana", amount),
            "LikeWater": ("LikeWater", amount),
            "Devotion": ("Devotion", amount),
            "Establishment": ("Establishment", 1),
        }

        if power_id in power_map:
            status, value = power_map[power_id]
            self._apply_status(self.state.player, status, value)


    def use_potion(self, potion_idx: int, target_idx: int = -1) -> Dict[str, Any]:
        """
        Use a potion.

        Args:
            potion_idx: Index in potion slots
            target_idx: Target enemy index (-1 for self)

        Returns:
            Dict with potion effects
        """
        if potion_idx >= len(self.state.potions):
            return {"success": False, "error": "Invalid potion index"}

        potion_id = self.state.potions[potion_idx]
        if not potion_id:
            return {"success": False, "error": "Empty potion slot"}
        if potion_id == "FairyPotion":
            return {"success": False, "error": "Fairy Potion triggers automatically"}

        result = {"success": True, "potion": potion_id, "effects": []}

        # Get target
        target = None
        if target_idx >= 0 and target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]

        # Apply potion effect
        applied = self._apply_potion_effect(potion_id, target, result)
        if not applied:
            return result

        # Remove potion
        self.state.potions[potion_idx] = ""
        self.potions_used.append(potion_id)

        # Check combat end
        self._check_combat_end()

        return result

    def _apply_potion_effect(self, potion_id: str, target: Optional[EnemyCombatState], result: dict) -> bool:
        """Apply a potion's effect using the registry system."""
        from ..registry import execute_potion_effect

        target_idx = -1
        if target is not None:
            for i, enemy in enumerate(self.state.enemies):
                if enemy is target:
                    target_idx = i
                    break

        registry_result = execute_potion_effect(potion_id, self.state, target_idx)

        if registry_result.get("success"):
            potency = registry_result.get("potency", 0)
            result["effects"].append({"type": "potion_used", "potion": potion_id, "potency": potency})
            return True
        else:
            result["success"] = False
            result["error"] = registry_result.get("error", "Unknown error")
            result["effects"].append({"type": "error", "error": result["error"]})
            return False

    def _end_player_turn(self):
        """End player turn and start enemy turn."""
        self.phase = CombatPhase.PLAYER_TURN_END

        # Trigger card-in-hand end-of-turn effects before discard/exhaust.
        self._trigger_end_of_turn_hand_cards()
        if self.state.player.hp <= 0:
            self.combat_over = True
            self.victory = False
            self.phase = CombatPhase.COMBAT_END
            return

        # Discard hand (except retained cards)
        retained = []
        for card_id in self.state.hand:
            card = self._get_card(card_id)
            if card and card.retain:
                retained.append(card_id)
            elif card and card.ethereal:
                self.state.exhaust_pile.append(card_id)
            else:
                self.state.discard_pile.append(card_id)

        # Runic Pyramid - retain entire hand
        if self.state.has_relic("Runic Pyramid"):
            retained = self.state.hand.copy()
            self.state.discard_pile.clear()

        self.state.hand = retained

        # Trigger end-of-turn effects
        self._trigger_end_of_turn()
        if self.state.player.hp <= 0:
            self.combat_over = True
            self.victory = False
            self.phase = CombatPhase.COMBAT_END
            return

        # Enemy turns
        if not self.combat_over:
            self._do_enemy_turns()

    def _trigger_end_of_turn_hand_cards(self):
        """Trigger end-of-turn effects from cards currently in hand."""
        from ..effects.registry import EffectContext, execute_effect

        for card_id in self.state.hand.copy():
            card = self._get_card(card_id)
            if card is None:
                continue

            end_turn_effects = [
                effect_name
                for effect_name in card.effects
                if effect_name.startswith("end_of_turn_")
            ]
            if not end_turn_effects:
                continue

            ctx = EffectContext(
                state=self.state,
                card=card,
                is_upgraded=card.upgraded,
                magic_number=card.magic_number,
            )
            for effect_name in end_turn_effects:
                execute_effect(effect_name, ctx)

            if self.state.player.hp <= 0:
                return

    def _trigger_end_of_turn(self):
        """Trigger end-of-turn effects using registry system."""
        # Execute registry-based onPlayerEndTurn triggers
        execute_relic_triggers("onPlayerEndTurn", self.state)

        # Execute atEndOfTurnPreEndTurnCards power triggers (Metallicize, Plated Armor, Like Water)
        execute_power_triggers("atEndOfTurnPreEndTurnCards", self.state, self.state.player)

        # Execute atEndOfTurn power triggers (Constricted damage, Combust, Ritual, etc.)
        execute_power_triggers("atEndOfTurn", self.state, self.state.player)
        for enemy in self.state.enemies:
            if not enemy.is_dead:
                execute_power_triggers("atEndOfTurn", self.state, enemy)

        # Divinity auto-exit
        if self.state.stance == "Divinity":
            self._change_stance("Neutral")

        # LoseDexterity - lose dexterity at end of turn (from Duality) - handled in registry
        # LoseStrength (Flex) - handled in registry

    def _do_enemy_turns(self):
        """Execute all enemy turns."""
        self.phase = CombatPhase.ENEMY_TURN

        for i, (enemy_state, enemy) in enumerate(zip(self.state.enemies, self.enemies)):
            if enemy_state.is_dead:
                continue

            # Reset enemy block
            enemy_state.block = 0

            # Execute move
            move = enemy.state.next_move
            if move:
                self._execute_enemy_move(i, enemy_state, enemy, move)

            # Roll next move
            if not enemy_state.is_dead:
                new_move = enemy.roll_move()
                self._update_enemy_move(i, new_move)

        # Check player death
        if self.state.player.hp <= 0:
            self.combat_over = True
            self.victory = False
            self.phase = CombatPhase.COMBAT_END
            return

        # Execute atEndOfRound power triggers (decrement Weak, Vulnerable, Frail)
        execute_power_triggers("atEndOfRound", self.state, self.state.player)
        for enemy_state in self.state.enemies:
            if not enemy_state.is_dead:
                execute_power_triggers("atEndOfRound", self.state, enemy_state)

        # Start next player turn
        self._start_player_turn()

    def _execute_enemy_move(self, enemy_idx: int, enemy_state: EnemyCombatState, enemy: Enemy, move: MoveInfo):
        """Execute an enemy's move."""
        # Apply strength to damage
        enemy_strength = enemy_state.statuses.get("Strength", 0)

        # Attack
        if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
            base_damage = move.base_damage + enemy_strength
            hits = move.hits

            # Calculate damage with player modifiers
            is_wrath = self.state.stance == "Wrath"
            vuln = self.state.player.is_vulnerable

            for _ in range(hits):
                # Calculate incoming damage
                final_damage = calculate_damage(
                    base=base_damage,
                    vuln=vuln,
                    stance_mult=WRATH_MULT if is_wrath else 1.0,
                )

                # Torii - reduce damage 2-5 to 1
                if self.state.has_relic("Torii"):
                    if 2 <= final_damage <= 5:
                        final_damage = 1

                # Apply to player
                hp_loss, block_remaining = calculate_incoming_damage(
                    damage=final_damage,
                    block=self.state.player.block,
                    is_wrath=is_wrath,
                    vuln=vuln,
                )

                # Buffer - prevent all damage from first unblocked hit
                buffer = self.state.player.statuses.get("Buffer", 0)
                if buffer > 0 and hp_loss > 0:
                    hp_loss = 0
                    self.state.player.statuses["Buffer"] = buffer - 1
                    if self.state.player.statuses["Buffer"] <= 0:
                        del self.state.player.statuses["Buffer"]

                # Tungsten Rod - reduce HP loss by 1
                if self.state.has_relic("TungstenRod"):
                    hp_loss = max(0, hp_loss - 1)

                # Centennial Puzzle - draw 3 cards first time taking damage
                if hp_loss > 0 and self.state.has_relic("Centennial Puzzle"):
                    counter = self.state.get_relic_counter("Centennial Puzzle", 0)
                    if counter == 0:
                        self._draw_cards(3)
                        self.state.set_relic_counter("Centennial Puzzle", 1)

                self.state.player.block = block_remaining
                self.state.player.hp -= hp_loss
                self.total_damage_taken += hp_loss

                # Trigger wasHPLost relics
                self._trigger_was_hp_lost(hp_loss)

                # Check death
                if self.state.player.hp <= 0:
                    self.state.player.hp = 0
                    self.combat_over = True
                    self.victory = False
                    self.phase = CombatPhase.COMBAT_END
                    return

        # Block
        if move.block > 0:
            enemy_state.block += move.block

        # Effects (buffs/debuffs)
        for effect_name, effect_value in move.effects.items():
            if effect_name == "strength":
                enemy_state.statuses["Strength"] = enemy_state.statuses.get("Strength", 0) + effect_value
            elif effect_name == "weak":
                self._apply_status(self.state.player, "Weak", effect_value)
            elif effect_name == "vulnerable":
                self._apply_status(self.state.player, "Vulnerable", effect_value)
            elif effect_name == "frail":
                self._apply_status(self.state.player, "Frail", effect_value)
            elif effect_name == "ritual":
                # Ritual: gain strength each turn
                enemy_state.statuses["Ritual"] = enemy_state.statuses.get("Ritual", 0) + effect_value

        # Apply Ritual if present (after move)
        ritual = enemy_state.statuses.get("Ritual", 0)
        if ritual > 0:
            enemy_state.statuses["Strength"] = enemy_state.statuses.get("Strength", 0) + ritual

    def _check_combat_end(self):
        """Check if combat should end."""
        # All enemies dead?
        if all(e.is_dead for e in self.state.enemies):
            self.combat_over = True
            self.victory = True
            self.phase = CombatPhase.COMBAT_END
            # Trigger onVictory relics (Burning Blood, Meat on the Bone, etc.)
            execute_relic_triggers("onVictory", self.state)

        # Player dead?
        if self.state.player.hp <= 0:
            if "FairyPotion" in self.state.potions:
                idx = self.state.potions.index("FairyPotion")
                self.state.potions[idx] = ""
                heal_percent = 0.6 if self.state.has_relic("SacredBark") else 0.3
                revived_hp = max(1, int(self.state.player.max_hp * heal_percent))
                self.state.player.hp = revived_hp
                self.potions_used.append("FairyPotion")
                return
            self.combat_over = True
            self.victory = False
            self.phase = CombatPhase.COMBAT_END


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
