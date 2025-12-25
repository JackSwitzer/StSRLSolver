"""
Watcher-specific Agent for Slay the Spire

Handles Watcher-specific mechanics:
- Stance management (Neutral, Wrath, Calm, Divinity)
- Mantra tracking
- Scry optimization
"""
import sys
import os
import logging

sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'spirecomm'))

from spirecomm.spire.game import Game
from spirecomm.spire.character import Intent, PlayerClass
import spirecomm.spire.card as card_module
from spirecomm.spire.screen import RestOption, ScreenType, RewardType
from spirecomm.communication.action import *
from watcher_priorities import WatcherPriority


logger = logging.getLogger(__name__)


class WatcherAgent:
    """AI Agent optimized for Watcher gameplay."""

    def __init__(self):
        self.game = Game()
        self.errors = 0
        self.choose_good_card = False
        self.skipped_cards = False
        self.visited_shop = False
        self.map_route = []
        self.priorities = WatcherPriority()

    def handle_error(self, error):
        """Handle errors from the game."""
        self.errors += 1
        logger.error(f"Game error #{self.errors}: {error}")
        if self.errors > 10:
            raise Exception(f"Too many errors: {error}")

    def get_next_action_in_game(self, game_state):
        """Main decision function called each game tick."""
        self.game = game_state
        logger.debug(f"Floor {self.game.floor}, HP: {self.game.current_hp}/{self.game.max_hp}")

        if self.game.choice_available:
            return self.handle_screen()
        if self.game.proceed_available:
            return ProceedAction()
        if self.game.play_available:
            return self.get_play_card_action()
        if self.game.end_available:
            return EndTurnAction()
        if self.game.cancel_available:
            return CancelAction()

        # Fallback
        return StateAction()

    def get_next_action_out_of_game(self):
        """Called when not in a game (main menu)."""
        logger.info("Starting new Watcher game at Ascension 0")
        return StartGameAction(PlayerClass.WATCHER, ascension_level=0)

    # === Combat Logic ===

    def get_current_stance(self):
        """Get current stance from player powers."""
        if not hasattr(self.game, 'player') or self.game.player is None:
            return "Neutral"

        for power in self.game.player.powers:
            if power.power_id == "Wrath":
                return "Wrath"
            elif power.power_id == "Calm":
                return "Calm"
            elif power.power_id == "Divinity":
                return "Divinity"
        return "Neutral"

    def is_monster_attacking(self):
        """Check if any monster is attacking."""
        for monster in self.game.monsters:
            if monster.intent.is_attack() or monster.intent == Intent.NONE:
                return True
        return False

    def get_incoming_damage(self):
        """Calculate incoming damage this turn."""
        incoming = 0
        stance = self.get_current_stance()
        multiplier = 2.0 if stance == "Wrath" else 1.0

        for monster in self.game.monsters:
            if not monster.is_gone and not monster.half_dead:
                if monster.move_adjusted_damage is not None:
                    incoming += monster.move_adjusted_damage * monster.move_hits
                elif monster.intent == Intent.NONE:
                    incoming += 5 * self.game.act

        return int(incoming * multiplier)

    def get_low_hp_target(self):
        """Find monster with lowest HP."""
        available = [m for m in self.game.monsters
                     if m.current_hp > 0 and not m.half_dead and not m.is_gone]
        if not available:
            return None
        return min(available, key=lambda x: x.current_hp)

    def get_high_hp_target(self):
        """Find monster with highest HP."""
        available = [m for m in self.game.monsters
                     if m.current_hp > 0 and not m.half_dead and not m.is_gone]
        if not available:
            return None
        return max(available, key=lambda x: x.current_hp)

    def many_monsters_alive(self):
        """Check if multiple monsters are alive."""
        available = [m for m in self.game.monsters
                     if m.current_hp > 0 and not m.half_dead and not m.is_gone]
        return len(available) > 1

    def get_play_card_action(self):
        """Decide which card to play."""
        playable_cards = [c for c in self.game.hand if c.is_playable]

        if not playable_cards:
            return EndTurnAction()

        # Get current stance for decision making
        stance = self.get_current_stance()
        incoming_damage = self.get_incoming_damage()
        current_block = self.game.player.block if self.game.player else 0

        # Categorize cards
        zero_cost = [c for c in playable_cards if c.cost == 0]
        nonzero_cost = [c for c in playable_cards if c.cost != 0]
        attacks = [c for c in playable_cards if c.type == card_module.CardType.ATTACK]
        skills = [c for c in playable_cards if c.type == card_module.CardType.SKILL]
        powers = [c for c in playable_cards if c.type == card_module.CardType.POWER]
        aoe_cards = [c for c in playable_cards if self.priorities.is_card_aoe(c)]
        defensive_cards = [c for c in playable_cards if self.priorities.is_card_defensive(c)]

        # Watcher-specific logic

        # 1. If in Wrath and monsters attacking, try to exit
        if stance == "Wrath" and self.is_monster_attacking():
            exit_wrath = [c for c in playable_cards if c.card_id in
                          ["EmptyFist", "EmptyBody", "EmptyMind", "Tranquility",
                           "InnerPeace", "Vigilance", "FearNoEvil", "Meditate"]]
            if exit_wrath:
                card = self.priorities.get_best_card_to_play(exit_wrath)
                return self._play_card(card)

        # 2. Play powers first (setup)
        if powers:
            card = self.priorities.get_best_card_to_play(powers)
            return self._play_card(card)

        # 3. If we have enough block, focus on offense
        if current_block >= incoming_damage + 5:
            offensive = [c for c in nonzero_cost if not self.priorities.is_card_defensive(c)]
            if offensive:
                nonzero_cost = offensive

        # 4. Zero-cost non-attacks first (usually skills/setup)
        zero_cost_non_attacks = [c for c in zero_cost if c.type != card_module.CardType.ATTACK]
        if zero_cost_non_attacks:
            card = self.priorities.get_best_card_to_play(zero_cost_non_attacks)
            return self._play_card(card)

        # 5. Non-zero cost cards
        if nonzero_cost:
            card = self.priorities.get_best_card_to_play(nonzero_cost)
            # If multiple enemies, prefer AOE
            if self.many_monsters_alive() and aoe_cards and card.type == card_module.CardType.ATTACK:
                card = self.priorities.get_best_card_to_play(aoe_cards)
            return self._play_card(card)

        # 6. Zero-cost attacks last
        zero_cost_attacks = [c for c in zero_cost if c.type == card_module.CardType.ATTACK]
        if zero_cost_attacks:
            card = self.priorities.get_best_card_to_play(zero_cost_attacks)
            return self._play_card(card)

        # Nothing playable
        return EndTurnAction()

    def _play_card(self, card):
        """Create PlayCardAction for a card."""
        if card.has_target:
            target = self.get_low_hp_target() if card.type == card_module.CardType.ATTACK else self.get_high_hp_target()
            if target is None:
                return EndTurnAction()
            return PlayCardAction(card=card, target_monster=target)
        return PlayCardAction(card=card)

    # === Screen Handling ===

    def handle_screen(self):
        """Handle various game screens."""
        screen_type = self.game.screen_type

        if screen_type == ScreenType.EVENT:
            return self.handle_event()
        elif screen_type == ScreenType.CHEST:
            return OpenChestAction()
        elif screen_type == ScreenType.SHOP_ROOM:
            return self.handle_shop_room()
        elif screen_type == ScreenType.REST:
            return self.handle_rest()
        elif screen_type == ScreenType.CARD_REWARD:
            return self.handle_card_reward()
        elif screen_type == ScreenType.COMBAT_REWARD:
            return self.handle_combat_reward()
        elif screen_type == ScreenType.MAP:
            return self.handle_map()
        elif screen_type == ScreenType.BOSS_REWARD:
            return self.handle_boss_reward()
        elif screen_type == ScreenType.SHOP_SCREEN:
            return self.handle_shop()
        elif screen_type == ScreenType.GRID:
            return self.handle_grid()
        elif screen_type == ScreenType.HAND_SELECT:
            return self.handle_hand_select()
        else:
            return ProceedAction()

    def handle_event(self):
        """Handle event choices."""
        # Dangerous events - take last option (usually leave)
        dangerous = ["Vampires", "Masked Bandits", "Knowing Skull", "Ghosts",
                     "Liars Game", "Golden Idol", "Drug Dealer", "The Library"]
        if self.game.screen.event_id in dangerous:
            return ChooseAction(len(self.game.screen.options) - 1)
        return ChooseAction(0)

    def handle_shop_room(self):
        """Handle entering a shop."""
        if not self.visited_shop:
            self.visited_shop = True
            return ChooseShopkeeperAction()
        self.visited_shop = False
        return ProceedAction()

    def handle_rest(self):
        """Handle rest site choices."""
        options = self.game.screen.rest_options

        if not options or self.game.screen.has_rested:
            return ProceedAction()

        hp_ratio = self.game.current_hp / self.game.max_hp

        # Rest if below 50% HP
        if RestOption.REST in options and hp_ratio < 0.5:
            return RestAction(RestOption.REST)

        # Rest before boss if below 90%
        if RestOption.REST in options and self.game.floor % 17 == 15 and hp_ratio < 0.9:
            return RestAction(RestOption.REST)

        # Otherwise upgrade
        if RestOption.SMITH in options:
            return RestAction(RestOption.SMITH)
        if RestOption.LIFT in options:
            return RestAction(RestOption.LIFT)
        if RestOption.DIG in options:
            return RestAction(RestOption.DIG)
        if RestOption.REST in options:
            return RestAction(RestOption.REST)

        return ChooseAction(0)

    def handle_card_reward(self):
        """Handle card reward screen."""
        reward_cards = self.game.screen.cards
        can_skip = self.game.screen.can_skip and not self.game.in_combat

        if can_skip:
            # Only pick cards we want more copies of
            pickable = [c for c in reward_cards
                        if self.priorities.needs_more_copies(c, self.count_copies(c))]
        else:
            pickable = reward_cards

        if pickable:
            best = self.priorities.get_best_card(pickable)
            return CardRewardAction(best)

        if self.game.screen.can_bowl:
            return CardRewardAction(bowl=True)

        self.skipped_cards = True
        return CancelAction()

    def count_copies(self, card):
        """Count copies of a card in deck."""
        return sum(1 for c in self.game.deck if c.card_id == card.card_id)

    def handle_combat_reward(self):
        """Handle post-combat rewards."""
        for reward in self.game.screen.rewards:
            if reward.reward_type == RewardType.POTION and self.game.are_potions_full():
                continue
            if reward.reward_type == RewardType.CARD and self.skipped_cards:
                continue
            return CombatRewardAction(reward)

        self.skipped_cards = False
        return ProceedAction()

    def handle_boss_reward(self):
        """Handle boss relic selection."""
        relics = self.game.screen.relics
        best = self.priorities.get_best_boss_relic(relics)
        return BossRewardAction(best)

    def handle_shop(self):
        """Handle shop purchases."""
        # Prioritize card removal
        if self.game.screen.purge_available and self.game.gold >= self.game.screen.purge_cost:
            return ChooseAction(name="purge")

        # Buy good cards
        for card in self.game.screen.cards:
            if self.game.gold >= card.price and not self.priorities.should_skip(card):
                return BuyCardAction(card)

        # Buy relics
        for relic in self.game.screen.relics:
            if self.game.gold >= relic.price:
                return BuyRelicAction(relic)

        return CancelAction()

    def handle_grid(self):
        """Handle grid card selection (upgrade, etc.)."""
        if not self.game.choice_available:
            return ProceedAction()

        if self.game.screen.for_upgrade or self.choose_good_card:
            available = self.priorities.get_sorted_cards(self.game.screen.cards)
        else:
            available = self.priorities.get_sorted_cards(self.game.screen.cards, reverse=True)

        num = self.game.screen.num_cards
        return CardSelectAction(available[:num])

    def handle_hand_select(self):
        """Handle hand card selection."""
        if not self.game.choice_available:
            return ProceedAction()

        num = min(self.game.screen.num_cards, 3)
        cards = self.priorities.get_cards_for_action(
            self.game.current_action, self.game.screen.cards, num
        )
        return CardSelectAction(cards)

    def handle_map(self):
        """Handle map navigation."""
        if self.game.screen.boss_available:
            return ChooseMapBossAction()

        next_nodes = self.game.screen.next_nodes
        if not next_nodes:
            return ProceedAction()

        # Generate route on floor 0
        if next_nodes[0].y == 0:
            self.generate_map_route()

        # Follow planned route
        current_y = self.game.screen.current_node.y if self.game.screen.current_node else -1
        if current_y + 1 < len(self.map_route):
            target_x = self.map_route[current_y + 1]
            for node in next_nodes:
                if node.x == target_x:
                    return ChooseMapNodeAction(node)

        # Fallback to first option
        return ChooseAction(0)

    def generate_map_route(self):
        """Use dynamic programming to find best map path."""
        node_rewards = self.priorities.MAP_NODE_PRIORITIES.get(self.game.act, self.priorities.MAP_NODE_PRIORITIES_1)

        # Initialize
        best_rewards = {0: {node.x: node_rewards.get(node.symbol, 1)
                            for node in self.game.map.nodes.get(0, {}).values()}}
        best_parents = {0: {node.x: 0 for node in self.game.map.nodes.get(0, {}).values()}}

        min_reward = min(node_rewards.values())
        map_height = max(self.game.map.nodes.keys()) if self.game.map.nodes else 0

        for y in range(0, map_height):
            if y + 1 not in self.game.map.nodes:
                continue

            best_rewards[y + 1] = {node.x: min_reward * 20
                                    for node in self.game.map.nodes[y + 1].values()}
            best_parents[y + 1] = {node.x: -1
                                    for node in self.game.map.nodes[y + 1].values()}

            if y not in best_rewards:
                continue

            for x in best_rewards[y]:
                node = self.game.map.get_node(x, y)
                if node is None:
                    continue

                best_node_reward = best_rewards[y][x]
                for child in node.children:
                    child_reward = best_node_reward + node_rewards.get(child.symbol, 1)
                    if child_reward > best_rewards[y + 1].get(child.x, float('-inf')):
                        best_rewards[y + 1][child.x] = child_reward
                        best_parents[y + 1][child.x] = node.x

        # Backtrack to find path
        best_path = [0] * (map_height + 1)
        if map_height in best_rewards and best_rewards[map_height]:
            best_path[map_height] = max(best_rewards[map_height].keys(),
                                         key=lambda x: best_rewards[map_height][x])

            for y in range(map_height, 0, -1):
                if y in best_parents and best_path[y] in best_parents[y]:
                    best_path[y - 1] = best_parents[y][best_path[y]]

        self.map_route = best_path
        logger.debug(f"Generated map route: {self.map_route}")
