"""
Combat Simulation - Core combat loop for Slay the Spire.

This module implements the exact combat mechanics from the decompiled source:
1. Turn structure (player turn -> monster turn -> end of turn)
2. Card playing (energy check, effects, damage calculation)
3. Block/damage resolution
4. Power triggers at various hooks
5. Stance mechanics integration

Key hooks (from decompiled AbstractCreature, AbstractMonster, AbstractCard):
- atTurnStart: Called when turn starts
- atTurnEnd: Called when turn ends (before/after energy loss)
- atEndOfRound: Called after all turns complete
- onPlayCard: When any card is played
- onUseCard: When a specific card is used
- atDamageGive/Receive: Damage modification
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Tuple, Any
from enum import Enum
import math

from .state.rng import Random
from .damage import DamageType, calculate_card_damage, calculate_block, calculate_incoming_damage, CombatState, Power
from .content.stances import StanceManager, StanceID, STANCES
from .content.cards import Card, CardType, get_card
from .content.enemies import Enemy, Intent, MoveInfo


class CombatPhase(Enum):
    """Combat phases."""
    PLAYER_TURN = "PLAYER_TURN"
    MONSTER_TURN = "MONSTER_TURN"
    END_OF_ROUND = "END_OF_ROUND"
    COMBAT_END = "COMBAT_END"


@dataclass
class PlayerState:
    """Player state during combat."""
    # HP
    current_hp: int
    max_hp: int

    # Combat resources
    energy: int = 3
    max_energy: int = 3
    block: int = 0

    # Card piles
    draw_pile: List[Card] = field(default_factory=list)
    hand: List[Card] = field(default_factory=list)
    discard_pile: List[Card] = field(default_factory=list)
    exhaust_pile: List[Card] = field(default_factory=list)

    # Stances
    stance: StanceID = StanceID.NEUTRAL
    mantra: int = 0

    # Powers (id -> amount)
    powers: Dict[str, int] = field(default_factory=dict)

    # Combat tracking
    cards_played_this_turn: int = 0
    attacks_played_this_turn: int = 0
    last_card_type: Optional[CardType] = None

    def has_power(self, power_id: str) -> bool:
        return power_id in self.powers and self.powers[power_id] > 0

    def get_power(self, power_id: str) -> int:
        return self.powers.get(power_id, 0)

    def add_power(self, power_id: str, amount: int):
        self.powers[power_id] = self.powers.get(power_id, 0) + amount

    def remove_power(self, power_id: str):
        if power_id in self.powers:
            del self.powers[power_id]


@dataclass
class CombatLog:
    """Log of combat events for EV tracking."""
    events: List[Dict[str, Any]] = field(default_factory=list)

    def log(self, event_type: str, **kwargs):
        self.events.append({"type": event_type, **kwargs})


class Combat:
    """
    Combat simulation engine.

    Handles the full combat loop including:
    - Turn management
    - Card playing
    - Damage/block resolution
    - Enemy AI
    - Power/relic triggers
    """

    def __init__(
        self,
        player_hp: int,
        player_max_hp: int,
        deck: List[Card],
        enemies: List[Enemy],
        shuffle_rng: Random,
        card_rng: Random,
        has_violet_lotus: bool = False,
        base_energy: int = 3,
    ):
        """
        Initialize combat.

        Args:
            player_hp: Starting HP
            player_max_hp: Max HP
            deck: Starting deck (will be shuffled into draw pile)
            enemies: List of enemies
            shuffle_rng: RNG for deck shuffling
            card_rng: RNG for card random effects
            has_violet_lotus: Whether player has Violet Lotus relic
            base_energy: Base energy per turn
        """
        self.shuffle_rng = shuffle_rng
        self.card_rng = card_rng

        # Initialize player state
        self.player = PlayerState(
            current_hp=player_hp,
            max_hp=player_max_hp,
            max_energy=base_energy,
        )

        # Shuffle deck into draw pile
        self.player.draw_pile = [c.copy() for c in deck]
        self._shuffle_draw_pile()

        # Initialize enemies
        self.enemies = enemies

        # Stance manager
        self.stance_manager = StanceManager(has_violet_lotus)

        # Combat state
        self.turn = 0
        self.phase = CombatPhase.PLAYER_TURN
        self.combat_over = False
        self.player_won = False

        # Logging
        self.log = CombatLog()

    def _shuffle_draw_pile(self):
        """Shuffle the draw pile using shuffle RNG."""
        n = len(self.player.draw_pile)
        for i in range(n - 1, 0, -1):
            j = self.shuffle_rng.random(i)
            self.player.draw_pile[i], self.player.draw_pile[j] = \
                self.player.draw_pile[j], self.player.draw_pile[i]

    def _draw_cards(self, count: int) -> List[Card]:
        """Draw cards from draw pile to hand."""
        drawn = []
        for _ in range(count):
            if not self.player.draw_pile:
                # Shuffle discard into draw
                if not self.player.discard_pile:
                    break
                self.player.draw_pile = self.player.discard_pile.copy()
                self.player.discard_pile.clear()
                self._shuffle_draw_pile()

            if self.player.draw_pile:
                card = self.player.draw_pile.pop()
                self.player.hand.append(card)
                drawn.append(card)

        self.log.log("draw_cards", count=len(drawn), cards=[c.id for c in drawn])
        return drawn

    def _discard_hand(self):
        """Discard all cards in hand (except retained cards)."""
        retained = []
        for card in self.player.hand:
            if card.retain:
                retained.append(card)
            elif card.ethereal:
                self.player.exhaust_pile.append(card)
            else:
                self.player.discard_pile.append(card)

        self.player.hand = retained

    def start_combat(self):
        """Initialize combat - called once at start."""
        self.turn = 0

        # Roll initial moves for all enemies
        for enemy in self.enemies:
            enemy.roll_move()

        self.log.log("combat_start", enemies=[e.ID for e in self.enemies])

        # Start first turn
        self._start_player_turn()

    def _start_player_turn(self):
        """Begin player turn."""
        self.turn += 1
        self.phase = CombatPhase.PLAYER_TURN

        # Reset energy
        self.player.energy = self.player.max_energy

        # Lose block (unless player has Barricade)
        if not self.player.has_power("Barricade"):
            self.player.block = 0

        # Reset turn counters
        self.player.cards_played_this_turn = 0
        self.player.attacks_played_this_turn = 0
        self.player.last_card_type = None

        # Draw cards (base 5)
        draw_count = 5
        # Modify draw count based on powers
        if self.player.has_power("NoDraw"):
            draw_count = 0
        self._draw_cards(draw_count)

        # Trigger start of turn powers
        self._trigger_start_of_turn()

        self.log.log("turn_start", turn=self.turn, energy=self.player.energy,
                    hand_size=len(self.player.hand))

    def _trigger_start_of_turn(self):
        """Trigger all start-of-turn effects."""
        # Deva Form: gain stacking energy
        if self.player.has_power("DevaForm"):
            self.player.energy += self.player.get_power("DevaForm")
            self.player.add_power("DevaForm", 1)

        # Devotion: gain mantra
        if self.player.has_power("Devotion"):
            self._add_mantra(self.player.get_power("Devotion"))

        # Foresight: scry
        if self.player.has_power("Foresight"):
            self._scry(self.player.get_power("Foresight"))

    def _add_mantra(self, amount: int):
        """Add mantra and potentially enter Divinity."""
        self.player.mantra += amount
        self.log.log("mantra_gain", amount=amount, total=self.player.mantra)

        if self.player.mantra >= 10:
            self.player.mantra -= 10
            self._change_stance(StanceID.DIVINITY)

    def _scry(self, amount: int):
        """Scry - look at top cards and choose to discard."""
        # For simulation, we can see the cards but automatic discard logic needed
        top_cards = self.player.draw_pile[-amount:] if self.player.draw_pile else []
        self.log.log("scry", cards=[c.id for c in top_cards])

        # Trigger Nirvana
        if self.player.has_power("Nirvana"):
            self.player.block += self.player.get_power("Nirvana")

    def _change_stance(self, new_stance: StanceID) -> Dict:
        """Change stance and handle effects."""
        old_stance = self.stance_manager.current
        result = self.stance_manager.change_stance(new_stance)

        if result["is_stance_change"]:
            self.player.stance = new_stance

            # Energy from stance change
            self.player.energy += result["energy_gained"]

            # Mental Fortress
            if self.player.has_power("MentalFortress"):
                self.player.block += self.player.get_power("MentalFortress")

            # Rushdown (Wrath entry)
            if new_stance == StanceID.WRATH and self.player.has_power("Rushdown"):
                self._draw_cards(self.player.get_power("Rushdown"))

            # Flurry of Blows (from discard)
            self._trigger_flurry_of_blows()

            self.log.log("stance_change", from_stance=old_stance.value if old_stance else None,
                        to_stance=new_stance.value, energy_gained=result["energy_gained"])

        return result

    def _trigger_flurry_of_blows(self):
        """Move Flurry of Blows from discard to hand."""
        flurries = [c for c in self.player.discard_pile if c.id == "FlurryOfBlows"]
        for f in flurries:
            self.player.discard_pile.remove(f)
            self.player.hand.append(f)

    def can_play_card(self, card: Card) -> bool:
        """Check if a card can be played."""
        # Energy check
        if card.current_cost > self.player.energy:
            return False

        # Signature Move check
        if "only_attack_in_hand" in card.effects:
            attacks_in_hand = sum(1 for c in self.player.hand if c.card_type == CardType.ATTACK)
            if attacks_in_hand > 1:
                return False

        return True

    def get_playable_cards(self) -> List[Tuple[int, Card]]:
        """Get list of (index, card) for playable cards."""
        return [(i, c) for i, c in enumerate(self.player.hand) if self.can_play_card(c)]

    def play_card(self, hand_index: int, target_enemy_index: int = 0) -> Dict:
        """
        Play a card from hand.

        Args:
            hand_index: Index in hand
            target_enemy_index: Index of target enemy (for targeted cards)

        Returns:
            Dict with effects of playing the card
        """
        if hand_index >= len(self.player.hand):
            return {"success": False, "error": "Invalid hand index"}

        card = self.player.hand[hand_index]

        if not self.can_play_card(card):
            return {"success": False, "error": "Cannot play card"}

        # Pay energy
        self.player.energy -= card.current_cost

        # Remove from hand
        self.player.hand.pop(hand_index)

        # Track card type
        self.player.cards_played_this_turn += 1
        self.player.last_card_type = card.card_type
        if card.card_type == CardType.ATTACK:
            self.player.attacks_played_this_turn += 1

        result = {"success": True, "card": card.id, "effects": []}

        # Build combat state for damage calculation
        combat_state = self._build_combat_state(target_enemy_index)

        # Apply card effects
        target_enemy = self.enemies[target_enemy_index] if target_enemy_index < len(self.enemies) else None

        # Damage
        if card.damage > 0:
            hits = card.magic_number if card.magic_number > 0 and "damage_x_times" in card.effects else 1
            per_hit_damage = calculate_card_damage(card.damage, combat_state, card.id)

            for _ in range(hits):
                if target_enemy and target_enemy.state.current_hp > 0:
                    # Apply damage to enemy
                    blocked = min(target_enemy.state.block, per_hit_damage)
                    hp_damage = per_hit_damage - blocked
                    target_enemy.state.block -= blocked
                    target_enemy.state.current_hp -= hp_damage

                    result["effects"].append({
                        "type": "damage",
                        "target": target_enemy.ID,
                        "amount": per_hit_damage,
                        "blocked": blocked,
                        "hp_damage": hp_damage,
                    })

                    # Check for kill
                    if target_enemy.state.current_hp <= 0:
                        target_enemy.state.current_hp = 0
                        result["effects"].append({"type": "kill", "target": target_enemy.ID})

        # Block
        if card.block > 0:
            block_gained = calculate_block(card.block, combat_state)
            self.player.block += block_gained
            result["effects"].append({"type": "block", "amount": block_gained})

        # Stance changes
        if card.enter_stance:
            self._change_stance(StanceID(card.enter_stance.upper()))
            result["effects"].append({"type": "stance", "stance": card.enter_stance})

        if card.exit_stance:
            self._change_stance(StanceID.NEUTRAL)
            result["effects"].append({"type": "stance", "stance": "Neutral"})

        # Draw effects
        if "draw_1" in card.effects:
            self._draw_cards(1)
            result["effects"].append({"type": "draw", "amount": 1})
        if "draw_2" in card.effects:
            self._draw_cards(2)
            result["effects"].append({"type": "draw", "amount": 2})
        if "draw_cards" in card.effects and card.magic_number > 0:
            self._draw_cards(card.magic_number)
            result["effects"].append({"type": "draw", "amount": card.magic_number})

        # Scry
        if "scry_1" in card.effects or "scry_2" in card.effects or "scry" in card.effects:
            scry_amount = card.magic_number if card.magic_number > 0 else 2
            self._scry(scry_amount)

        # Mantra
        if "gain_mantra" in card.effects and card.magic_number > 0:
            self._add_mantra(card.magic_number)

        # Power application
        if card.card_type == CardType.POWER:
            # Add the power based on card ID
            self._apply_power_card(card)
            result["effects"].append({"type": "power", "card": card.id})

        # Card destination
        if card.exhaust:
            self.player.exhaust_pile.append(card)
        elif card.shuffle_back:
            self.player.draw_pile.insert(
                self.shuffle_rng.random(len(self.player.draw_pile)) if self.player.draw_pile else 0,
                card
            )
        else:
            self.player.discard_pile.append(card)

        # End turn effects
        if "end_turn" in card.effects:
            result["effects"].append({"type": "end_turn"})
            self.end_player_turn()

        self.log.log("play_card", **result)

        # Check combat end
        self._check_combat_end()

        return result

    def _apply_power_card(self, card: Card):
        """Apply a power card's effect."""
        power_mapping = {
            "MentalFortress": ("MentalFortress", card.magic_number),
            "Rushdown": ("Rushdown", card.magic_number),
            "Nirvana": ("Nirvana", card.magic_number),
            "LikeWater": ("LikeWater", card.magic_number),
            "DevaForm": ("DevaForm", 1),
            "Devotion": ("Devotion", card.magic_number),
            "Foresight": ("Foresight", card.magic_number),
            "Establishment": ("Establishment", 1),
            "BattleHymn": ("BattleHymn", 1),
            "Study": ("Study", 1),
        }

        if card.id in power_mapping:
            power_id, amount = power_mapping[card.id]
            self.player.add_power(power_id, amount)

    def _build_combat_state(self, target_index: int = 0) -> CombatState:
        """Build combat state for damage calculation."""
        player_powers = []

        # Strength
        str_amt = self.player.get_power("Strength")
        if str_amt != 0:
            player_powers.append(Power("Strength", str_amt))

        # Weak
        if self.player.has_power("Weak"):
            player_powers.append(Power("Weak", self.player.get_power("Weak")))

        # Vigor
        vigor = self.player.get_power("Vigor")
        if vigor > 0:
            player_powers.append(Power("Vigor", vigor))

        # Dexterity (for block)
        dex = self.player.get_power("Dexterity")
        if dex != 0:
            player_powers.append(Power("Dexterity", dex))

        # Frail
        if self.player.has_power("Frail"):
            player_powers.append(Power("Frail", 1))

        target_powers = []
        if target_index < len(self.enemies):
            enemy = self.enemies[target_index]
            if enemy.state.powers.get("Vulnerable", 0) > 0:
                target_powers.append(Power("Vulnerable", 1))

        # Stance multiplier
        stance_mult = self.stance_manager.effect.damage_give_multiplier
        stance_incoming_mult = self.stance_manager.effect.damage_receive_multiplier

        return CombatState(
            player_powers=player_powers,
            stance_damage_mult=stance_mult,
            stance_incoming_mult=stance_incoming_mult,
            target_powers=target_powers,
        )

    def end_player_turn(self):
        """End the player's turn."""
        if self.phase != CombatPhase.PLAYER_TURN:
            return

        # Discard hand
        self._discard_hand()

        # Like Water
        if self.player.has_power("LikeWater") and self.player.stance == StanceID.CALM:
            self.player.block += self.player.get_power("LikeWater")

        # Divinity auto-exit
        if self.player.stance == StanceID.DIVINITY:
            self._change_stance(StanceID.NEUTRAL)

        # Decrement debuffs
        for debuff in ["Weak", "Frail", "Vulnerable"]:
            if self.player.has_power(debuff):
                self.player.powers[debuff] -= 1
                if self.player.powers[debuff] <= 0:
                    del self.player.powers[debuff]

        self.log.log("player_turn_end", hp=self.player.current_hp, block=self.player.block)

        # Monster turns
        self._do_monster_turns()

    def _do_monster_turns(self):
        """Execute all monster turns."""
        self.phase = CombatPhase.MONSTER_TURN

        for enemy in self.enemies:
            if enemy.state.current_hp <= 0:
                continue

            move = enemy.state.next_move
            if not move:
                continue

            self.log.log("monster_move", enemy=enemy.ID, move=move.name, intent=move.intent.value)

            # Apply strength to damage
            enemy_strength = enemy.state.powers.get("strength", 0)

            # Execute move
            if move.intent in [Intent.ATTACK, Intent.ATTACK_BUFF, Intent.ATTACK_DEBUFF, Intent.ATTACK_DEFEND]:
                base_damage = move.base_damage + enemy_strength
                hits = move.hits

                # Calculate damage with player's stance
                damage_mult = self.stance_manager.effect.damage_receive_multiplier

                for _ in range(hits):
                    damage = int(base_damage * damage_mult)

                    # Apply block
                    blocked = min(self.player.block, damage)
                    hp_damage = damage - blocked
                    self.player.block -= blocked
                    self.player.current_hp -= hp_damage

                    self.log.log("player_damage", enemy=enemy.ID, damage=damage,
                                blocked=blocked, hp_damage=hp_damage)

                    if self.player.current_hp <= 0:
                        self.player.current_hp = 0
                        self._combat_end(player_won=False)
                        return

            # Enemy block
            if move.block > 0:
                enemy.state.block += move.block

            # Enemy buffs/debuffs
            if "strength" in move.effects:
                enemy.state.strength += move.effects["strength"]

            if "weak" in move.effects:
                self.player.add_power("Weak", move.effects["weak"])

            if "vulnerable" in move.effects:
                self.player.add_power("Vulnerable", move.effects["vulnerable"])

            if "frail" in move.effects:
                self.player.add_power("Frail", move.effects["frail"])

            # Roll next move
            enemy.roll_move()

        # End of round
        self._end_of_round()

    def _end_of_round(self):
        """End of round processing."""
        self.phase = CombatPhase.END_OF_ROUND

        # Enemy block decay
        for enemy in self.enemies:
            enemy.state.block = 0

        # Start next player turn
        if not self.combat_over:
            self._start_player_turn()

    def _check_combat_end(self):
        """Check if combat should end."""
        # All enemies dead?
        all_dead = all(e.state.current_hp <= 0 for e in self.enemies)
        if all_dead:
            self._combat_end(player_won=True)

    def _combat_end(self, player_won: bool):
        """End combat."""
        self.combat_over = True
        self.player_won = player_won
        self.phase = CombatPhase.COMBAT_END
        self.log.log("combat_end", player_won=player_won, turns=self.turn,
                    player_hp=self.player.current_hp)

    def get_state(self) -> Dict:
        """Get current combat state for observation."""
        return {
            "turn": self.turn,
            "phase": self.phase.value,
            "player": {
                "hp": self.player.current_hp,
                "max_hp": self.player.max_hp,
                "energy": self.player.energy,
                "block": self.player.block,
                "stance": self.player.stance.value,
                "mantra": self.player.mantra,
                "powers": self.player.powers.copy(),
                "hand_size": len(self.player.hand),
                "draw_size": len(self.player.draw_pile),
                "discard_size": len(self.player.discard_pile),
            },
            "enemies": [
                {
                    "id": e.ID,
                    "hp": e.state.current_hp,
                    "max_hp": e.state.max_hp,
                    "block": e.state.block,
                    "intent": e.state.next_move.intent.value if e.state.next_move else None,
                    "intent_damage": e.state.next_move.base_damage if e.state.next_move else None,
                    "powers": e.state.powers.copy(),
                }
                for e in self.enemies
            ],
            "hand": [c.id for c in self.player.hand],
            "playable": [i for i, _ in self.get_playable_cards()],
        }


# ============ TESTING ============

if __name__ == "__main__":
    from .rng import Random as GameRandom
    from .cards import get_starting_deck
    from .enemies import JawWorm

    print("=== Combat Simulation Test ===\n")

    # Setup
    seed = 12345
    shuffle_rng = GameRandom(seed)
    card_rng = GameRandom(seed + 1)
    ai_rng = GameRandom(seed + 2)
    hp_rng = GameRandom(seed + 3)

    deck = get_starting_deck()
    enemies = [JawWorm(ai_rng, ascension=0, hp_rng=hp_rng)]

    combat = Combat(
        player_hp=80,
        player_max_hp=80,
        deck=deck,
        enemies=enemies,
        shuffle_rng=shuffle_rng,
        card_rng=card_rng,
    )

    # Start combat
    combat.start_combat()

    print("Initial state:")
    state = combat.get_state()
    print(f"  Player: {state['player']['hp']} HP, {state['player']['energy']} energy")
    print(f"  Hand: {state['hand']}")
    print(f"  Enemy: {enemies[0].ID} - {state['enemies'][0]['hp']} HP")
    print(f"  Enemy intent: {state['enemies'][0]['intent']}")

    # Play a few turns
    for turn in range(3):
        print(f"\n--- Turn {turn + 1} ---")

        playable = combat.get_playable_cards()
        print(f"Playable cards: {[(i, c.name) for i, c in playable]}")

        # Play first playable card
        if playable:
            idx, card = playable[0]
            result = combat.play_card(idx)
            print(f"Played {card.name}: {result['effects']}")

        # Check state
        state = combat.get_state()
        print(f"After play: {state['player']['energy']} energy, {state['player']['block']} block")

        # End turn
        if combat.phase == CombatPhase.PLAYER_TURN:
            combat.end_player_turn()

        if combat.combat_over:
            print(f"\nCombat ended! Player won: {combat.player_won}")
            break
