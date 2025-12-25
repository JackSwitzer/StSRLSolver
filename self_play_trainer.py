#!/usr/bin/env python3
"""
Self-play trainer for Slay the Spire.

This runs games autonomously, using line evaluation to make decisions
and learning from outcomes. Designed to run for days.

Key approach:
1. Use LineSimulator to evaluate all possible plays
2. Pick best line based on simulation score
3. Record (state, action, outcome) tuples
4. Periodically train model on collected data
5. Use model to bias action selection (exploration vs exploitation)

Run with: nohup uv run python3 self_play_trainer.py > logs/selfplay.log 2>&1 &
"""

import sys
import json
import logging
import time
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any, Tuple, Optional
from dataclasses import dataclass, asdict
import random

# Add paths
sys.path.insert(0, str(Path(__file__).parent / "spirecomm"))

from spirecomm.spire.game import Game
from spirecomm.spire.screen import ScreenType
from spirecomm.spire.character import PlayerClass
from spirecomm.communication.coordinator import Coordinator
from spirecomm.communication.action import *

from models.line_evaluator import (
    LineSimulator, SimulatedPlayer, SimulatedEnemy, LineOutcome, CARD_EFFECTS
)
from models.strategic_features import (
    StrategicState, extract_strategic_features, strategic_state_to_vector, STRATEGIC_FEATURE_DIM
)
from models.enemy_database import get_enemy_info, get_encounter_difficulty

# Setup
LOG_DIR = Path(__file__).parent / "logs"
DATA_DIR = Path(__file__).parent / "data" / "self_play"
LOG_DIR.mkdir(exist_ok=True)
DATA_DIR.mkdir(parents=True, exist_ok=True)

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(LOG_DIR / f"selfplay_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


@dataclass
class Experience:
    """Single experience tuple for training."""
    state_features: List[float]  # Strategic features
    action_taken: str            # What we did
    line_score: float            # Simulated score of the line
    actual_outcome: str          # "win", "loss", "continue"
    damage_taken: int
    damage_dealt: int
    floor: int
    turn: int


class SelfPlayAgent:
    """Agent that learns through self-play with line evaluation."""

    def __init__(self, exploration_rate: float = 0.1):
        self.simulator = LineSimulator()
        self.exploration_rate = exploration_rate

        # Experience buffer
        self.experiences: List[Experience] = []
        self.current_game_experiences: List[Experience] = []

        # Stats
        self.games_played = 0
        self.wins = 0
        self.total_floors = 0

        # Game state tracking
        self.game = None
        self.last_hp = 80
        self.last_floor = 0
        self.turn_in_combat = 0

    def get_next_action_in_game(self, game_state: Game):
        """Main decision function using line evaluation."""
        self.game = game_state

        # Combat is detected via in_combat flag, not screen_type
        if game_state.in_combat:
            return self._handle_combat(game_state)

        screen_type = game_state.screen_type

        if screen_type == ScreenType.CARD_REWARD:
            return self._handle_card_reward(game_state)
        elif screen_type == ScreenType.MAP:
            return self._handle_map(game_state)
        elif screen_type == ScreenType.REST:
            return self._handle_rest(game_state)
        elif screen_type == ScreenType.SHOP_SCREEN:
            return self._handle_shop(game_state)
        elif screen_type == ScreenType.BOSS_REWARD:
            return self._handle_boss_reward(game_state)
        elif screen_type == ScreenType.GRID:
            return self._handle_grid(game_state)
        elif screen_type == ScreenType.HAND_SELECT:
            return self._handle_hand_select(game_state)
        elif screen_type == ScreenType.EVENT:
            return self._handle_event(game_state)
        elif screen_type == ScreenType.CHEST:
            return self._handle_chest(game_state)
        elif screen_type == ScreenType.COMBAT_REWARD:
            return self._handle_combat_reward(game_state)
        else:
            return self._handle_other(game_state)

    def _handle_combat(self, game_state: Game):
        """Use line simulation to pick best combat action."""
        self.turn_in_combat += 1

        # Build simulation state
        player = self._make_sim_player(game_state)
        enemies = self._make_sim_enemies(game_state)
        hand = self._make_hand_list(game_state)

        if not hand or not enemies:
            return EndTurnAction()

        # Find best line
        best_outcome, best_actions = self.simulator.find_best_line(
            player, enemies, hand, max_actions=3
        )

        # Record experience
        features = strategic_state_to_vector(extract_strategic_features(game_state))
        action_str = str(best_actions[0]) if best_actions else "end_turn"

        exp = Experience(
            state_features=features,
            action_taken=action_str,
            line_score=best_outcome.score if best_outcome else 0,
            actual_outcome="continue",
            damage_taken=0,
            damage_dealt=0,
            floor=game_state.floor,
            turn=self.turn_in_combat,
        )
        self.current_game_experiences.append(exp)

        # Exploration: sometimes pick random playable action
        if random.random() < self.exploration_rate:
            playable_cards = [c for c in hand if c.get("playable", True)]
            if playable_cards:
                random_card = random.choice(playable_cards)
                card_idx = hand.index(random_card)
                logger.debug(f"Exploring: playing random card {random_card.get('id')}")
                return PlayCardAction(card_index=card_idx)

        # Execute best line
        if best_outcome and best_outcome.is_lethal:
            logger.info(f"Found lethal line: {best_actions}, score={best_outcome.score:.0f}")

        if best_actions:
            card_id, target = best_actions[0]
            # Find card index in hand
            for i, card in enumerate(hand):
                if card.get("id") == card_id:
                    if target is not None:
                        return PlayCardAction(card_index=i, target_index=target)
                    return PlayCardAction(card_index=i)

        # Fallback: play first playable card or end turn
        for i, card in enumerate(game_state.hand):
            if card.is_playable:
                return PlayCardAction(card_index=i)

        return EndTurnAction()

    def _make_sim_player(self, game_state: Game) -> SimulatedPlayer:
        """Convert game state to simulation player."""
        # Get stance
        stance = "Neutral"
        if hasattr(game_state, 'player') and game_state.player:
            for power in getattr(game_state.player, 'powers', []):
                if 'wrath' in power.power_id.lower():
                    stance = "Wrath"
                elif 'calm' in power.power_id.lower():
                    stance = "Calm"
                elif 'divinity' in power.power_id.lower():
                    stance = "Divinity"

        # Get strength/dex
        strength = 0
        dexterity = 0
        if hasattr(game_state, 'player') and game_state.player:
            for power in getattr(game_state.player, 'powers', []):
                if power.power_id == "Strength":
                    strength = power.amount
                elif power.power_id == "Dexterity":
                    dexterity = power.amount

        return SimulatedPlayer(
            hp=game_state.current_hp,
            block=getattr(game_state.player, 'block', 0) if game_state.player else 0,
            energy=game_state.player.energy if game_state.player else 3,
            stance=stance,
            strength=strength,
            dexterity=dexterity,
        )

    def _make_sim_enemies(self, game_state: Game) -> List[SimulatedEnemy]:
        """Convert monsters to simulation enemies."""
        enemies = []
        for i, monster in enumerate(game_state.monsters):
            if monster.is_gone or monster.current_hp <= 0:
                continue

            # Check intent
            intent_str = str(monster.intent) if monster.intent else ""
            is_attacking = "ATTACK" in intent_str

            # Get debuffs
            vulnerable = 0
            weak = 0
            for power in monster.powers:
                if power.power_id == "Vulnerable":
                    vulnerable = power.amount
                elif power.power_id == "Weakened":
                    weak = power.amount

            enemies.append(SimulatedEnemy(
                id=i,
                hp=monster.current_hp,
                max_hp=monster.max_hp,
                block=monster.block,
                intent_damage=monster.move_adjusted_damage or 0,
                intent_hits=max(monster.move_hits or 1, 1),
                is_attacking=is_attacking,
                vulnerable=vulnerable,
                weak=weak,
            ))

        return enemies

    def _make_hand_list(self, game_state: Game) -> List[Dict]:
        """Convert hand to list of card dicts."""
        hand = []
        for card in game_state.hand:
            # Normalize card ID
            card_id = card.card_id
            if card.upgrades > 0 and not card_id.endswith("+"):
                card_id = card_id + "+"

            hand.append({
                "id": card_id,
                "cost": card.cost,
                "playable": card.is_playable,
            })
        return hand

    def _handle_card_reward(self, game_state: Game):
        """Pick card from reward (simple heuristic for now)."""
        # Reset combat turn counter
        self.turn_in_combat = 0

        cards = game_state.screen.cards
        if not cards:
            return CancelAction()

        # Priority list (will be learned eventually)
        priorities = {
            "Rushdown": 100, "Tantrum": 90, "MentalFortress": 85,
            "TalkToTheHand": 80, "Ragnarok": 75, "Blasphemy": 70,
            "Conclude": 65, "InnerPeace": 60, "EmptyFist": 55,
            "FearNoEvil": 50, "Wallop": 45, "Scrawl": 40,
        }

        best_card = None
        best_priority = -1

        for card in cards:
            base_id = card.card_id.replace("+", "").split("+")[0]
            priority = priorities.get(base_id, 10)

            # Bonus for upgrades
            if card.upgrades > 0:
                priority += 5

            if priority > best_priority:
                best_priority = priority
                best_card = card

        if best_priority > 20:
            return ChooseAction(card_index=cards.index(best_card))

        return CancelAction()  # Skip low-priority cards

    def _handle_map(self, game_state: Game):
        """Choose map path (simple heuristic)."""
        choices = game_state.screen.next_nodes

        # Prefer: elite (if strong) > campfire > ? > shop > monster
        for node in choices:
            if node.symbol == "R":  # Rest
                return ChooseMapNodeAction(node)

        for node in choices:
            if node.symbol == "?":  # Unknown
                return ChooseMapNodeAction(node)

        for node in choices:
            if node.symbol == "$":  # Shop
                return ChooseMapNodeAction(node)

        # Default: first option
        if choices:
            return ChooseMapNodeAction(choices[0])

        return ProceedAction()

    def _handle_rest(self, game_state: Game):
        """Rest or smith at campfire."""
        hp_ratio = game_state.current_hp / game_state.max_hp

        if hp_ratio < 0.5:
            return RestAction()

        # Upgrade best card
        return SmithAction()

    def _handle_shop(self, game_state: Game):
        """Handle shop (skip for now)."""
        return CancelAction()

    def _handle_boss_reward(self, game_state: Game):
        """Pick boss relic."""
        relics = game_state.screen.relics
        if relics:
            return BossRewardAction(relic_index=0)
        return ProceedAction()

    def _handle_grid(self, game_state: Game):
        """Handle grid selection (card transform, etc)."""
        cards = game_state.screen.cards
        if cards:
            return ChooseAction(card_index=0)
        return CancelAction()

    def _handle_hand_select(self, game_state: Game):
        """Handle hand selection."""
        cards = game_state.screen.cards
        if cards:
            return ChooseAction(card_index=0)
        return CancelAction()

    def _handle_event(self, game_state: Game):
        """Handle event screens - pick first available option."""
        self.turn_in_combat = 0  # Reset combat turn counter
        options = game_state.screen.options if hasattr(game_state.screen, 'options') else []
        if options:
            # Simple heuristic: avoid options with "lose" or "damage" in name if possible
            for i, opt in enumerate(options):
                opt_text = str(opt).lower()
                if 'leave' in opt_text or 'ignore' in opt_text:
                    continue  # Skip leave options for now
                return ChooseAction(choice_index=i)
            return ChooseAction(choice_index=0)
        return ProceedAction()

    def _handle_chest(self, game_state: Game):
        """Handle chest screens - open the chest."""
        self.turn_in_combat = 0
        return ProceedAction()

    def _handle_combat_reward(self, game_state: Game):
        """Handle combat rewards - claim gold, potion, relic."""
        self.turn_in_combat = 0  # Reset combat turn counter
        rewards = game_state.screen.rewards if hasattr(game_state.screen, 'rewards') else []
        for i, reward in enumerate(rewards):
            reward_type = getattr(reward, 'reward_type', None)
            # Claim gold, relics, and potions (if we have space)
            if reward_type and str(reward_type) in ['GOLD', 'RELIC', 'STOLEN_GOLD']:
                return CombatRewardAction(reward_index=i)
            # Skip cards here - let card_reward screen handle them
        return ProceedAction()

    def _handle_other(self, game_state: Game):
        """Handle other screens."""
        if game_state.screen_type == ScreenType.GAME_OVER:
            return None

        # Try to proceed
        if hasattr(game_state.screen, 'proceed_available') and game_state.screen.proceed_available:
            return ProceedAction()

        if hasattr(game_state.screen, 'confirm_available') and game_state.screen.confirm_available:
            return ConfirmAction()

        return ProceedAction()

    def get_next_action_out_of_game(self, game_state: Game):
        """Handle main menu / game over."""
        if game_state.in_game:
            return None

        # Start new game
        return StartGameAction(PlayerClass.WATCHER, ascension_level=0)

    def handle_error(self, error):
        """Handle errors."""
        logger.error(f"Error: {error}")

    def on_game_end(self, victory: bool, floor: int):
        """Called when game ends."""
        self.games_played += 1
        self.total_floors += floor

        if victory:
            self.wins += 1

        # Mark experiences with outcome
        for exp in self.current_game_experiences:
            exp.actual_outcome = "win" if victory else "loss"

        # Save experiences
        self.experiences.extend(self.current_game_experiences)
        self.current_game_experiences = []

        # Log stats
        win_rate = self.wins / max(self.games_played, 1) * 100
        avg_floor = self.total_floors / max(self.games_played, 1)
        logger.info(f"Game {self.games_played}: {'WIN' if victory else 'LOSS'} at floor {floor}")
        logger.info(f"Stats: {self.wins}/{self.games_played} wins ({win_rate:.1f}%), avg floor {avg_floor:.1f}")

        # Save data periodically
        if self.games_played % 10 == 0:
            self._save_experiences()

    def _save_experiences(self):
        """Save collected experiences to disk."""
        filename = DATA_DIR / f"experiences_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        data = [asdict(exp) for exp in self.experiences]
        with open(filename, 'w') as f:
            json.dump(data, f)
        logger.info(f"Saved {len(self.experiences)} experiences to {filename}")
        self.experiences = []


def run_self_play(num_games: int = 1000, exploration_rate: float = 0.1):
    """Main self-play loop."""
    logger.info("=== Self-Play Training Started ===")
    logger.info(f"Target games: {num_games}")
    logger.info(f"Exploration rate: {exploration_rate}")

    # Progress file for monitoring
    progress_file = LOG_DIR / "selfplay_progress.txt"
    start_time = time.time()

    agent = SelfPlayAgent(exploration_rate=exploration_rate)
    coordinator = Coordinator()

    coordinator.signal_ready()
    coordinator.register_command_error_callback(agent.handle_error)
    coordinator.register_state_change_callback(agent.get_next_action_in_game)
    coordinator.register_out_of_game_callback(agent.get_next_action_out_of_game)

    for game_num in range(num_games):
        logger.info(f"\n=== Starting game {game_num + 1}/{num_games} ===")

        try:
            victory = coordinator.play_one_game(PlayerClass.WATCHER, ascension_level=0)
            # Get floor from last game state
            floor = 0
            if coordinator.last_game_state:
                floor = getattr(coordinator.last_game_state, 'floor', 0)
            agent.on_game_end(victory, floor)

            # Update progress file
            elapsed = time.time() - start_time
            games_per_hour = agent.games_played / max(elapsed / 3600, 0.001)
            eta_hours = (num_games - agent.games_played) / max(games_per_hour, 0.1)
            win_rate = agent.wins / max(agent.games_played, 1) * 100
            avg_floor = agent.total_floors / max(agent.games_played, 1)

            progress_str = (
                f"Games: {agent.games_played}/{num_games} ({100*agent.games_played/num_games:.1f}%)\n"
                f"Wins: {agent.wins} ({win_rate:.1f}%)\n"
                f"Avg Floor: {avg_floor:.1f}\n"
                f"Speed: {games_per_hour:.1f} games/hr\n"
                f"ETA: {eta_hours:.1f} hours\n"
                f"Experiences: {len(agent.experiences)}\n"
                f"Updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n"
            )
            with open(progress_file, 'w') as f:
                f.write(progress_str)

        except Exception as e:
            logger.error(f"Game error: {e}")
            import traceback
            logger.error(traceback.format_exc())
            agent.current_game_experiences = []

        # Small delay between games
        time.sleep(1)

    # Final save
    agent._save_experiences()
    logger.info("=== Self-Play Complete ===")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--games", type=int, default=100)
    parser.add_argument("--exploration", type=float, default=0.1)
    args = parser.parse_args()

    run_self_play(num_games=args.games, exploration_rate=args.exploration)
