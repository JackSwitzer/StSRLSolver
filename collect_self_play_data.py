#!/usr/bin/env python3
"""
Self-play data collection agent for Slay the Spire.

Runs the game in fast mode and collects state-action pairs.
Works with CommunicationMod + LudicrousSpeed for fast data generation.

Usage:
    1. Ensure mods are loaded: basemod, stslib, CommunicationMod, LudicrousSpeed
    2. Configure CommunicationMod to run this script
    3. Let it run overnight to collect data
"""

import json
import os
import sys
import logging
from datetime import datetime
from pathlib import Path

# Add spirecomm to path
sys.path.insert(0, str(Path(__file__).parent / "spirecomm"))

from spirecomm.spire.game import Game
from spirecomm.spire.character import PlayerClass
from spirecomm.communication.coordinator import Coordinator

# Import our Watcher agent
from watcher_agent import WatcherAgent

# Configuration
DATA_DIR = Path(__file__).parent / "data" / "self_play"
MAX_GAMES = 1000  # Number of games to play
ASCENSION = 0  # Start with A0, increase as agent improves

class DataCollector:
    """Wraps WatcherAgent to collect training data."""

    def __init__(self, output_dir: Path):
        self.agent = WatcherAgent()
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.current_game_data = []
        self.game_count = 0

    def record_state_action(self, game_state: Game, action: str):
        """Record a state-action pair."""
        self.current_game_data.append({
            "turn": game_state.turn if hasattr(game_state, 'turn') else 0,
            "floor": game_state.floor,
            "screen_type": game_state.screen_type.name if game_state.screen_type else "UNKNOWN",
            "current_hp": game_state.current_hp,
            "max_hp": game_state.max_hp,
            "gold": game_state.gold,
            "action": action,
            # Deck state
            "deck_size": len(game_state.deck) if game_state.deck else 0,
            "hand": [c.card_id for c in game_state.hand] if game_state.hand else [],
            "relics": [r.relic_id for r in game_state.relics] if game_state.relics else [],
        })

    def get_next_action_in_game(self, game_state: Game):
        """Wrapper that records state before getting action."""
        action = self.agent.get_next_action_in_game(game_state)

        # Record the state-action pair
        action_str = str(action) if action else "None"
        self.record_state_action(game_state, action_str)

        return action

    def get_next_action_out_of_game(self, game_state: Game):
        """Handle out-of-game states."""
        return self.agent.get_next_action_out_of_game(game_state)

    def handle_error(self, error):
        """Handle errors."""
        self.agent.handle_error(error)

    def save_game_data(self, result):
        """Save collected game data."""
        self.game_count += 1

        game_record = {
            "game_id": self.game_count,
            "timestamp": datetime.now().isoformat(),
            "victory": result.victory if hasattr(result, 'victory') else False,
            "floor_reached": result.floor if hasattr(result, 'floor') else 0,
            "ascension": ASCENSION,
            "state_actions": self.current_game_data,
        }

        # Save to individual file
        filename = f"game_{self.game_count:05d}.json"
        with open(self.output_dir / filename, 'w') as f:
            json.dump(game_record, f)

        victory_str = "WIN" if game_record["victory"] else "LOSS"
        print(f"Game {self.game_count}: {victory_str} at floor {game_record['floor_reached']}")

        # Reset for next game
        self.current_game_data = []

def main():
    # Setup logging
    log_dir = Path(__file__).parent / "logs"
    log_dir.mkdir(exist_ok=True)
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_dir / f"collector_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"),
            logging.StreamHandler()
        ]
    )

    logging.info("=== Self-Play Data Collection Started ===")
    logging.info(f"Target games: {MAX_GAMES}")
    logging.info(f"Ascension: {ASCENSION}")
    logging.info(f"Output dir: {DATA_DIR}")

    collector = DataCollector(DATA_DIR)
    coordinator = Coordinator()

    coordinator.signal_ready()
    coordinator.register_command_error_callback(collector.handle_error)
    coordinator.register_state_change_callback(collector.get_next_action_in_game)
    coordinator.register_out_of_game_callback(collector.get_next_action_out_of_game)

    wins = 0
    for game_num in range(MAX_GAMES):
        logging.info(f"Starting game {game_num + 1}/{MAX_GAMES}")

        try:
            result = coordinator.play_one_game(PlayerClass.WATCHER, ascension_level=ASCENSION)
            collector.save_game_data(result)

            if hasattr(result, 'victory') and result.victory:
                wins += 1

            logging.info(f"Win rate so far: {wins}/{game_num+1} = {100*wins/(game_num+1):.1f}%")

        except Exception as e:
            logging.error(f"Game {game_num + 1} failed: {e}")
            collector.current_game_data = []  # Reset on error

    logging.info(f"=== Collection Complete ===")
    logging.info(f"Total games: {MAX_GAMES}")
    logging.info(f"Wins: {wins} ({100*wins/MAX_GAMES:.1f}%)")

if __name__ == "__main__":
    main()
