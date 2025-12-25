#!/usr/bin/env python3
"""
Slay the Spire Watcher AI Agent

This script is launched by CommunicationMod and communicates via stdin/stdout.
It plays Ascension 0 Watcher to test the setup.
"""
import sys
import os
import json
import logging
from datetime import datetime

# Add spirecomm to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'spirecomm'))

from spirecomm.communication.coordinator import Coordinator
from spirecomm.spire.character import PlayerClass
from watcher_agent import WatcherAgent


def setup_logging():
    """Set up logging to file for debugging."""
    log_dir = os.path.join(os.path.dirname(__file__), 'logs')
    os.makedirs(log_dir, exist_ok=True)

    log_file = os.path.join(log_dir, f'agent_{datetime.now().strftime("%Y%m%d_%H%M%S")}.log')

    logging.basicConfig(
        level=logging.DEBUG,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_file),
        ]
    )
    return logging.getLogger(__name__)


def main():
    """Main entry point for the agent."""
    logger = setup_logging()
    logger.info("=== Slay the Spire Watcher Agent Starting ===")

    try:
        # Create our Watcher agent
        agent = WatcherAgent()
        logger.info("WatcherAgent created")

        # Create coordinator for communication
        coordinator = Coordinator()
        logger.info("Coordinator created")

        # Signal we're ready
        coordinator.signal_ready()
        logger.info("Signaled ready to CommunicationMod")

        # Register callbacks
        coordinator.register_command_error_callback(agent.handle_error)
        coordinator.register_state_change_callback(agent.get_next_action_in_game)
        coordinator.register_out_of_game_callback(agent.get_next_action_out_of_game)
        logger.info("Callbacks registered")

        # Play games
        games_played = 0
        max_games = 10  # Limit for testing

        while games_played < max_games:
            logger.info(f"Starting game {games_played + 1}")
            try:
                result = coordinator.play_one_game(PlayerClass.WATCHER, ascension_level=0)
                games_played += 1

                if result:
                    victory = "VICTORY" if result.victory else "DEFEAT"
                    logger.info(f"Game {games_played} finished: {victory}")
                    logger.info(f"  Floor reached: {result.floor}")
                    logger.info(f"  Final HP: {result.current_hp}/{result.max_hp}")
                    logger.info(f"  Gold: {result.gold}")
                else:
                    logger.warning(f"Game {games_played} returned no result")

            except Exception as e:
                logger.error(f"Error during game {games_played + 1}: {e}", exc_info=True)
                break

        logger.info(f"=== Agent finished after {games_played} games ===")

    except Exception as e:
        logger.error(f"Fatal error: {e}", exc_info=True)
        raise


if __name__ == "__main__":
    main()
