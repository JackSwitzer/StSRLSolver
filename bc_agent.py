#!/usr/bin/env python3
"""
Watcher agent powered by trained BC model.
Uses neural network for card pick decisions, falls back to heuristics for combat.
"""

import sys
import logging
from pathlib import Path
from datetime import datetime

import torch

# Add paths
sys.path.insert(0, str(Path(__file__).parent / "spirecomm"))

from spirecomm.spire.game import Game
from spirecomm.spire.screen import ScreenType
from spirecomm.spire.character import PlayerClass
from spirecomm.communication.coordinator import Coordinator
from spirecomm.communication.action import *

from models.encoding import encode_game_state, CARD_PICK_ACTIONS, get_state_dim, get_card_pick_action_dim
from models.bc_model import CardPickerBC
from watcher_agent import WatcherAgent  # Use existing agent for combat

# Setup logging
log_dir = Path(__file__).parent / "logs"
log_dir.mkdir(exist_ok=True)
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(log_dir / f"bc_agent_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


class BCWatcherAgent:
    """Watcher agent using BC model for card picks."""

    def __init__(self, model_path: Path = None):
        # Load BC model for card picks
        self.model = None
        self.device = "mps" if torch.backends.mps.is_available() else "cpu"

        if model_path is None:
            # Find best model
            checkpoints = Path(__file__).parent / "checkpoints"
            model_paths = list(checkpoints.glob("*/best_model.pt"))
            if model_paths:
                model_path = model_paths[0]
                logger.info(f"Using model: {model_path}")

        if model_path and model_path.exists():
            self._load_model(model_path)
        else:
            logger.warning("No BC model found, using heuristics only")

        # Use base WatcherAgent for combat decisions
        self.base_agent = WatcherAgent()
        self.game = None

    def _load_model(self, path: Path):
        """Load trained BC model."""
        try:
            state_dim = get_state_dim()
            action_dim = get_card_pick_action_dim()
            self.model = CardPickerBC(state_dim, action_dim)

            checkpoint = torch.load(path, map_location=self.device)
            self.model.load_state_dict(checkpoint['model_state_dict'])
            self.model.to(self.device)
            self.model.eval()
            logger.info(f"Loaded BC model from {path}")
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            self.model = None

    def get_next_action_in_game(self, game_state: Game):
        """Main decision function."""
        self.game = game_state
        self.base_agent.game = game_state

        screen_type = game_state.screen_type

        # Use BC model for card rewards
        if screen_type == ScreenType.CARD_REWARD:
            return self._handle_card_reward(game_state)

        # Use base agent for everything else (combat, events, etc.)
        return self.base_agent.get_next_action_in_game(game_state)

    def _handle_card_reward(self, game_state: Game):
        """Use BC model for card selection."""
        if not self.model:
            return self.base_agent.get_next_action_in_game(game_state)

        try:
            # Build state
            run_data = self._game_to_run_data(game_state)
            state = encode_game_state(run_data, game_state.floor)
            state_tensor = torch.FloatTensor(state).to(self.device)

            # Get model prediction
            with torch.no_grad():
                logits = self.model(state_tensor.unsqueeze(0))[0]
                probs = torch.softmax(logits, dim=-1)

            # Get available cards
            reward_cards = game_state.screen.cards if hasattr(game_state.screen, 'cards') else []

            # Find best card from available options
            best_action = None
            best_prob = -1

            for card in reward_cards:
                card_name = card.card_id.replace("+", "").replace("1", "")
                for i, action_name in enumerate(CARD_PICK_ACTIONS):
                    if action_name.lower() == card_name.lower() or card_name.lower() in action_name.lower():
                        if probs[i].item() > best_prob:
                            best_prob = probs[i].item()
                            best_action = card
                            logger.info(f"BC model prefers {card.name} (prob: {best_prob:.3f})")

            # Check if SKIP is preferred
            skip_prob = probs[0].item()  # Index 0 is SKIP
            if skip_prob > best_prob:
                logger.info(f"BC model prefers SKIP (prob: {skip_prob:.3f})")
                return CancelAction()

            if best_action:
                return ChooseAction(card_index=reward_cards.index(best_action))

            # Fallback to base agent
            logger.info("BC model uncertain, using heuristics")
            return self.base_agent.get_next_action_in_game(game_state)

        except Exception as e:
            logger.error(f"BC model error: {e}")
            return self.base_agent.get_next_action_in_game(game_state)

    def _game_to_run_data(self, game_state: Game) -> dict:
        """Convert game state to run data format for encoding."""
        return {
            "master_deck": [c.card_id for c in game_state.deck] if game_state.deck else [],
            "relics": [r.relic_id for r in game_state.relics] if game_state.relics else [],
            "current_hp_per_floor": [game_state.current_hp],
            "max_hp_per_floor": [game_state.max_hp],
            "gold": game_state.gold,
            "ascension_level": game_state.ascension_level if hasattr(game_state, 'ascension_level') else 0,
            "floor_reached": game_state.floor,
        }

    def get_next_action_out_of_game(self, game_state: Game):
        """Handle out-of-game actions."""
        return self.base_agent.get_next_action_out_of_game(game_state)

    def handle_error(self, error):
        """Handle errors."""
        logger.error(f"Error: {error}")


def main():
    logger.info("=== BC Watcher Agent Starting ===")

    # Find best model
    model_path = None
    checkpoints = Path(__file__).parent / "checkpoints"
    for run_dir in sorted(checkpoints.glob("run*")):
        best = run_dir / "best_model.pt"
        if best.exists():
            model_path = best
            break

    if model_path:
        logger.info(f"Using model: {model_path}")
    else:
        logger.warning("No model found, using pure heuristics")

    agent = BCWatcherAgent(model_path)
    coordinator = Coordinator()

    coordinator.signal_ready()
    coordinator.register_command_error_callback(agent.handle_error)
    coordinator.register_state_change_callback(agent.get_next_action_in_game)
    coordinator.register_out_of_game_callback(agent.get_next_action_out_of_game)

    logger.info("Starting game...")
    result = coordinator.play_one_game(PlayerClass.WATCHER, ascension_level=0)

    victory = "WIN" if result.victory else "LOSS"
    logger.info(f"Game finished: {victory} at floor {result.floor}")


if __name__ == "__main__":
    main()
