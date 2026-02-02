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
    PlayCard, UsePotion, EndTurn, Action,
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
        if self.run_state.has_relic("CoffeeDripper"):
            pass  # No energy bonus, but can't rest
        if self.run_state.has_relic("FusionHammer"):
            base_energy += 1
        if self.run_state.has_relic("Ectoplasm"):
            base_energy += 1
        if self.run_state.has_relic("Cursed Key"):
            base_energy += 1
        if self.run_state.has_relic("Busted Crown"):
            base_energy += 1
        if self.run_state.has_relic("Sozu"):
            base_energy += 1
        if self.run_state.has_relic("PhilosopherStone"):
            base_energy += 1
        if self.run_state.has_relic("MarkOfPain"):
            base_energy += 1
        if self.run_state.has_relic("Nuclear Battery"):
            base_energy += 1  # Defect only
        if self.run_state.has_relic("VelvetChoker"):
            base_energy += 1
        if self.run_state.has_relic("RunicDome"):
            base_energy += 1
        if self.run_state.has_relic("SneckoEye"):
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

        # Trigger start-of-combat relics
        self._trigger_start_of_combat_relics()

        # Draw initial hand
        draw_count = 5
        if self.state.has_relic("Ring of the Snake"):
            draw_count += 2
        if self.state.has_relic("SneckoEye"):
            # Snecko Eye randomizes card costs on draw
            pass

        self._draw_cards(draw_count)

        # Start first turn (without drawing since we already drew)
        self._start_player_turn(first_turn=True)

    def _trigger_start_of_combat_relics(self):
        """Trigger relics that activate at start of combat."""
        # Pure Water (Watcher) - Add Miracle to hand
        if self.state.has_relic("PureWater"):
            self.state.hand.append("Miracle")

        # Bag of Marbles - Apply 1 Vulnerable to all enemies
        if self.state.has_relic("Bag of Marbles"):
            for enemy in self.state.enemies:
                if not enemy.is_dead:
                    self._apply_status(enemy, "Vulnerable", 1)

        # Anchor - Gain 10 block
        if self.state.has_relic("Anchor"):
            self.state.player.block += 10
            self.total_block_gained += 10

        # Akabeko - Gain 8 Vigor
        if self.state.has_relic("Akabeko"):
            self._apply_status(self.state.player, "Vigor", 8)

        # Bronze Scales - Gain 3 Thorns
        if self.state.has_relic("BronzeScales"):
            self._apply_status(self.state.player, "Thorns", 3)

        # Preserved Insect - If room is elite, enemies start with 25% less HP
        # (This should be handled when creating enemies)

        # Thread and Needle - Gain 4 Plated Armor
        if self.state.has_relic("Thread and Needle"):
            self._apply_status(self.state.player, "Plated Armor", 4)

        # Pen Nib counter initialization
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

            if self.state.draw_pile:
                card = self.state.draw_pile.pop()
                self.state.hand.append(card)
                drawn.append(card)

                # Handle Void card (lose 1 energy when drawn)
                if card == "Void" or card == "Void+":
                    self.state.energy = max(0, self.state.energy - 1)

        return drawn

    def _start_player_turn(self, first_turn: bool = False):
        """
        Begin player turn.

        Args:
            first_turn: If True, skip drawing cards (already drawn in setup)
        """
        self.phase = CombatPhase.PLAYER_TURN_START

        # Increment turn counter
        if self.state.turn > 0:
            self.state.turn += 1
        else:
            self.state.turn = 1

        # Reset energy
        self.state.energy = self.state.max_energy

        # Remove block (unless Barricade/Blur) - skip on first turn
        if not first_turn:
            if not self.state.player.statuses.get("Barricade", 0) > 0:
                blur = self.state.player.statuses.get("Blur", 0)
                if blur > 0:
                    # Blur lets you keep some block
                    pass  # Keep block based on blur stacks
                else:
                    self.state.player.block = 0

            # Calipers - Lose only 15 block per turn
            if self.state.has_relic("Calipers"):
                self.state.player.block = max(0, self.state.player.block - 15)

        # Reset turn counters
        self.state.cards_played_this_turn = 0
        self.state.attacks_played_this_turn = 0
        self.state.skills_played_this_turn = 0
        self.state.powers_played_this_turn = 0

        # Draw cards (skip on first turn since setup already drew)
        if not first_turn and not self.state.player.statuses.get("NoDraw", 0) > 0:
            draw_count = 5
            # Relics that modify draw
            if self.state.has_relic("Snecko Skull"):
                pass  # Poison-related, not draw
            self._draw_cards(draw_count)

        # Trigger start-of-turn effects
        self._trigger_start_of_turn()

        self.phase = CombatPhase.PLAYER_TURN

    def _trigger_start_of_turn(self):
        """Trigger start-of-turn effects."""
        # Metallicize - Gain block at end of turn
        metallicize = self.state.player.statuses.get("Metallicize", 0)
        if metallicize > 0:
            self.state.player.block += metallicize
            self.total_block_gained += metallicize

        # Plated Armor - Gain block at end of turn
        plated = self.state.player.statuses.get("Plated Armor", 0)
        if plated > 0:
            self.state.player.block += plated
            self.total_block_gained += plated

        # Combust - Deal 5 damage to all enemies, take 1 damage
        combust = self.state.player.statuses.get("Combust", 0)
        if combust > 0:
            for enemy in self.state.enemies:
                if not enemy.is_dead:
                    self._deal_damage_to_enemy(enemy, 5)
            self.state.player.hp -= 1
            self.total_damage_taken += 1

        # Regeneration - Heal at end of turn
        regen = self.state.player.statuses.get("Regeneration", 0)
        if regen > 0:
            heal = min(regen, self.state.player.max_hp - self.state.player.hp)
            self.state.player.hp += heal
            self.state.player.statuses["Regeneration"] = regen - 1
            if self.state.player.statuses["Regeneration"] <= 0:
                del self.state.player.statuses["Regeneration"]

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
            action: Action to execute (PlayCard, UsePotion, EndTurn)

        Returns:
            Dict with action results
        """
        if isinstance(action, PlayCard):
            return self.play_card(action.card_idx, action.target_idx)
        elif isinstance(action, UsePotion):
            return self.use_potion(action.potion_idx, action.target_idx)
        elif isinstance(action, EndTurn):
            return {"action": "end_turn"}
        else:
            return {"error": "Unknown action type"}

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
            self.state.exhaust_pile.append(card_id)
        elif card.shuffle_back:
            # Insert at random position in draw pile
            pos = self.shuffle_rng.random(len(self.state.draw_pile)) if self.state.draw_pile else 0
            self.state.draw_pile.insert(pos, card_id)
        else:
            self.state.discard_pile.append(card_id)

        # Trigger on-play relics
        self._trigger_on_play_relics(card)

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

        # Exit Calm - gain energy
        if old_stance == "Calm":
            energy_gain = 2
            if self.state.has_relic("VioletLotus"):
                energy_gain = 3
            self.state.energy += energy_gain

        # Enter Wrath/Calm/Divinity
        self.state.stance = new_stance

        # Enter Divinity - gain energy
        if new_stance == "Divinity":
            self.state.energy += 3

        # Mental Fortress - gain block on stance change
        mental_fortress = self.state.player.statuses.get("MentalFortress", 0)
        if mental_fortress > 0:
            self.state.player.block += mental_fortress
            self.total_block_gained += mental_fortress

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

    def _trigger_on_play_relics(self, card: Card):
        """Trigger relics that activate on card play."""
        # Shuriken - +1 Strength per 3 attacks
        if card.card_type == CardType.ATTACK and self.state.has_relic("Shuriken"):
            counter = self.state.get_relic_counter("Shuriken", 0)
            counter += 1
            if counter >= 3:
                self._apply_status(self.state.player, "Strength", 1)
                counter = 0
            self.state.set_relic_counter("Shuriken", counter)

        # Kunai - +1 Dexterity per 3 attacks
        if card.card_type == CardType.ATTACK and self.state.has_relic("Kunai"):
            counter = self.state.get_relic_counter("Kunai", 0)
            counter += 1
            if counter >= 3:
                self._apply_status(self.state.player, "Dexterity", 1)
                counter = 0
            self.state.set_relic_counter("Kunai", counter)

        # Letter Opener - Deal 5 damage per 3 skills
        if card.card_type == CardType.SKILL and self.state.has_relic("LetterOpener"):
            counter = self.state.get_relic_counter("LetterOpener", 0)
            counter += 1
            if counter >= 3:
                for enemy in self.state.enemies:
                    if not enemy.is_dead:
                        self._deal_damage_to_enemy(enemy, 5)
                counter = 0
            self.state.set_relic_counter("LetterOpener", counter)

        # Ornamental Fan - Gain 4 block per 3 attacks
        if card.card_type == CardType.ATTACK and self.state.has_relic("OrnamentalFan"):
            counter = self.state.get_relic_counter("OrnamentalFan", 0)
            counter += 1
            if counter >= 3:
                self.state.player.block += 4
                self.total_block_gained += 4
                counter = 0
            self.state.set_relic_counter("OrnamentalFan", counter)

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

        result = {"success": True, "potion": potion_id, "effects": []}

        # Get target
        target = None
        if target_idx >= 0 and target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]

        # Apply potion effect
        self._apply_potion_effect(potion_id, target, result)

        # Remove potion
        self.state.potions[potion_idx] = ""
        self.potions_used.append(potion_id)

        # Check combat end
        self._check_combat_end()

        return result

    def _apply_potion_effect(self, potion_id: str, target: Optional[EnemyCombatState], result: dict):
        """Apply a potion's effect."""
        # Common potions
        if potion_id == "Block Potion":
            block = 12
            if self.state.has_relic("SacredBark"):
                block = 24
            self.state.player.block += block
            self.total_block_gained += block
            result["effects"].append({"type": "block", "amount": block})

        elif potion_id == "Fire Potion":
            damage = 20
            if self.state.has_relic("SacredBark"):
                damage = 40
            if target:
                self._deal_damage_to_enemy(target, damage)
                result["effects"].append({"type": "damage", "amount": damage})

        elif potion_id == "Strength Potion":
            amount = 2
            if self.state.has_relic("SacredBark"):
                amount = 4
            self._apply_status(self.state.player, "Strength", amount)
            result["effects"].append({"type": "strength", "amount": amount})

        elif potion_id == "Dexterity Potion":
            amount = 2
            if self.state.has_relic("SacredBark"):
                amount = 4
            self._apply_status(self.state.player, "Dexterity", amount)
            result["effects"].append({"type": "dexterity", "amount": amount})

        elif potion_id == "Weak Potion":
            amount = 3
            if self.state.has_relic("SacredBark"):
                amount = 6
            if target:
                self._apply_status(target, "Weak", amount)
                result["effects"].append({"type": "weak", "amount": amount})

        elif potion_id == "Fear Potion":
            amount = 3
            if self.state.has_relic("SacredBark"):
                amount = 6
            if target:
                self._apply_status(target, "Vulnerable", amount)
                result["effects"].append({"type": "vulnerable", "amount": amount})

        elif potion_id == "Energy Potion":
            amount = 2
            if self.state.has_relic("SacredBark"):
                amount = 4
            self.state.energy += amount
            result["effects"].append({"type": "energy", "amount": amount})

    def _end_player_turn(self):
        """End player turn and start enemy turn."""
        self.phase = CombatPhase.PLAYER_TURN_END

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
        if self.state.has_relic("RunicPyramid"):
            retained = self.state.hand.copy()
            self.state.discard_pile.clear()

        self.state.hand = retained

        # Trigger end-of-turn effects
        self._trigger_end_of_turn()

        # Enemy turns
        if not self.combat_over:
            self._do_enemy_turns()

    def _trigger_end_of_turn(self):
        """Trigger end-of-turn effects."""
        # Decrement debuffs
        for debuff in ["Weak", "Vulnerable", "Frail"]:
            if debuff in self.state.player.statuses:
                self.state.player.statuses[debuff] -= 1
                if self.state.player.statuses[debuff] <= 0:
                    del self.state.player.statuses[debuff]

        # Like Water - gain block if in Calm
        if self.state.stance == "Calm":
            like_water = self.state.player.statuses.get("LikeWater", 0)
            if like_water > 0:
                self.state.player.block += like_water
                self.total_block_gained += like_water

        # Divinity auto-exit
        if self.state.stance == "Divinity":
            self._change_stance("Neutral")

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

        # Decrement enemy debuffs
        for enemy_state in self.state.enemies:
            if enemy_state.is_dead:
                continue
            for debuff in ["Weak", "Vulnerable"]:
                if debuff in enemy_state.statuses:
                    enemy_state.statuses[debuff] -= 1
                    if enemy_state.statuses[debuff] <= 0:
                        del enemy_state.statuses[debuff]

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

                # Apply to player
                hp_loss, block_remaining = calculate_incoming_damage(
                    damage=final_damage,
                    block=self.state.player.block,
                    is_wrath=is_wrath,
                    vuln=vuln,
                )

                self.state.player.block = block_remaining
                self.state.player.hp -= hp_loss
                self.total_damage_taken += hp_loss

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

        # Player dead?
        if self.state.player.hp <= 0:
            self.combat_over = True
            self.victory = False
            self.phase = CombatPhase.COMBAT_END


# =============================================================================
# Encounter Creation
# =============================================================================

# Enemy class registry
ENEMY_CLASSES: Dict[str, type] = {}


def register_enemy_class(enemy_class: type):
    """Register an enemy class for encounter creation."""
    ENEMY_CLASSES[enemy_class.ID] = enemy_class


# Import and register enemies
try:
    from ..content.enemies import (
        JawWorm, Cultist, AcidSlimeM, SpikeSlimeM,
        AcidSlimeL, SpikeSlimeL, AcidSlimeS, SpikeSlimeS,
    )
    register_enemy_class(JawWorm)
    register_enemy_class(Cultist)
    register_enemy_class(AcidSlimeM)
    register_enemy_class(SpikeSlimeM)
    register_enemy_class(AcidSlimeL)
    register_enemy_class(SpikeSlimeL)
    register_enemy_class(AcidSlimeS)
    register_enemy_class(SpikeSlimeS)
except ImportError:
    pass  # Enemies not fully available


# Act 1 encounter pools (from decompiled MonsterHelper.java)
ACT1_WEAK_ENCOUNTERS = [
    ["JawWorm"],
    ["Cultist"],
    ["Cultist", "Cultist"],
    ["AcidSlime_S", "AcidSlime_S"],
    ["SpikeSlime_S", "SpikeSlime_S"],
]

ACT1_STRONG_ENCOUNTERS = [
    ["AcidSlime_L"],
    ["SpikeSlime_L"],
    ["JawWorm", "JawWorm"],
    ["Cultist", "AcidSlime_M"],
]


def create_encounter(
    run_state: RunState,
    room_type: str,
    floor: int,
    monster_rng: Random,
    ai_rng: Random,
    hp_rng: Random,
) -> List[Enemy]:
    """
    Create an enemy encounter for a room.

    Args:
        run_state: Current RunState
        room_type: "monster", "elite", "boss"
        floor: Current floor number
        monster_rng: RNG for encounter selection
        ai_rng: RNG for enemy AI
        hp_rng: RNG for enemy HP rolls

    Returns:
        List of Enemy instances
    """
    act = run_state.act
    ascension = run_state.ascension

    # Select encounter based on act and room type
    if room_type == "monster":
        if act == 1:
            if floor <= 2:
                # First few floors are easier
                pool = ACT1_WEAK_ENCOUNTERS
            else:
                pool = ACT1_WEAK_ENCOUNTERS + ACT1_STRONG_ENCOUNTERS
        else:
            pool = ACT1_WEAK_ENCOUNTERS  # Fallback

        # Select from pool
        idx = monster_rng.random(len(pool) - 1)
        encounter_ids = pool[idx]
    else:
        # Elite/boss encounters need more specific handling
        encounter_ids = ["JawWorm"]  # Fallback

    # Create enemy instances
    enemies = []
    for enemy_id in encounter_ids:
        if enemy_id in ENEMY_CLASSES:
            enemy_class = ENEMY_CLASSES[enemy_id]
            enemy = enemy_class(ai_rng=ai_rng.copy(), ascension=ascension, hp_rng=hp_rng.copy())
            enemies.append(enemy)

    return enemies


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
