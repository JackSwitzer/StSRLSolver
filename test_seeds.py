#!/usr/bin/env python3
"""
Test the AI agent against specific seeds.

This allows us to:
1. Test against known "lost" seeds to see if we can beat them
2. Reproducibly test agent improvements
3. Compare different agent versions

Usage:
    # Test a specific seed
    uv run python3 test_seeds.py --seed 2887516270255843204

    # Test multiple seeds from a file
    uv run python3 test_seeds.py --seeds-file data/lost_seeds.txt

    # Run the agent on a random seed N times
    uv run python3 test_seeds.py --random --num-games 10
"""

import argparse
import json
import logging
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Optional

# Add spirecomm to path
sys.path.insert(0, str(Path(__file__).parent / "spirecomm"))

from spirecomm.spire.character import PlayerClass
from spirecomm.communication.coordinator import Coordinator

from watcher_agent import WatcherAgent

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# === KNOWN LOST SEEDS ===
# These are seeds where expert players lost - our goal is to beat them
LOST_SEEDS = [
    # From example run in our data (Defect A20 loss)
    "2887516270255843204",

    # Add more lost seeds here as we collect them
    # Format: "SEED_STRING"  # Source: player/run_id
]

class SeedTester:
    """Test agent against specific seeds."""

    def __init__(self, agent_class=WatcherAgent, model_path: Optional[Path] = None):
        self.agent_class = agent_class
        self.model_path = model_path
        self.results = []

    def test_seed(
        self,
        seed: str,
        character: PlayerClass = PlayerClass.WATCHER,
        ascension: int = 20,
    ) -> dict:
        """
        Test the agent on a specific seed.

        Returns:
            Dict with test results
        """
        logger.info(f"Testing seed: {seed}")
        logger.info(f"Character: {character.name}, Ascension: {ascension}")

        # Create agent
        agent = self.agent_class()

        # TODO: Load trained model if specified
        # if self.model_path:
        #     agent.load_model(self.model_path)

        # Setup coordinator
        coordinator = Coordinator()
        coordinator.signal_ready()
        coordinator.register_command_error_callback(agent.handle_error)
        coordinator.register_state_change_callback(agent.get_next_action_in_game)
        coordinator.register_out_of_game_callback(agent.get_next_action_out_of_game)

        # Play the game with the specific seed
        try:
            result = coordinator.play_one_game(
                character,
                ascension_level=ascension,
                seed=seed,  # This requires game support for seeded runs
            )

            test_result = {
                "seed": seed,
                "character": character.name,
                "ascension": ascension,
                "victory": result.victory if hasattr(result, 'victory') else False,
                "floor_reached": result.floor if hasattr(result, 'floor') else 0,
                "timestamp": datetime.now().isoformat(),
            }

            victory_str = "WIN" if test_result["victory"] else "LOSS"
            logger.info(f"Result: {victory_str} at floor {test_result['floor_reached']}")

        except Exception as e:
            logger.error(f"Error testing seed {seed}: {e}")
            test_result = {
                "seed": seed,
                "character": character.name,
                "ascension": ascension,
                "victory": False,
                "floor_reached": 0,
                "error": str(e),
                "timestamp": datetime.now().isoformat(),
            }

        self.results.append(test_result)
        return test_result

    def test_seeds(
        self,
        seeds: List[str],
        character: PlayerClass = PlayerClass.WATCHER,
        ascension: int = 20,
    ) -> dict:
        """Test multiple seeds and return summary."""
        logger.info(f"Testing {len(seeds)} seeds...")

        for seed in seeds:
            self.test_seed(seed, character, ascension)

        # Compute summary
        wins = sum(1 for r in self.results if r.get("victory", False))
        losses = len(self.results) - wins

        summary = {
            "total_games": len(self.results),
            "wins": wins,
            "losses": losses,
            "win_rate": wins / len(self.results) if self.results else 0,
            "avg_floor": sum(r.get("floor_reached", 0) for r in self.results) / len(self.results) if self.results else 0,
            "results": self.results,
        }

        logger.info(f"\n=== Summary ===")
        logger.info(f"Win rate: {wins}/{len(self.results)} = {summary['win_rate']:.1%}")
        logger.info(f"Average floor: {summary['avg_floor']:.1f}")

        return summary

    def save_results(self, output_path: Path):
        """Save results to JSON file."""
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2)
        logger.info(f"Results saved to {output_path}")


def load_seeds_from_file(filepath: Path) -> List[str]:
    """Load seeds from a text file (one per line)."""
    seeds = []
    with open(filepath) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#'):
                seeds.append(line)
    return seeds


def main():
    parser = argparse.ArgumentParser(description="Test AI agent against specific seeds")
    parser.add_argument("--seed", type=str, help="Single seed to test")
    parser.add_argument("--seeds-file", type=Path, help="File with seeds to test")
    parser.add_argument("--lost-seeds", action="store_true", help="Test against known lost seeds")
    parser.add_argument("--random", action="store_true", help="Test with random seeds")
    parser.add_argument("--num-games", type=int, default=1, help="Number of games to run")
    parser.add_argument("--ascension", type=int, default=0, help="Ascension level")
    parser.add_argument("--character", type=str, default="WATCHER", help="Character to play")
    parser.add_argument("--model", type=Path, help="Path to trained model checkpoint")
    parser.add_argument("--output", type=Path, help="Output file for results")

    args = parser.parse_args()

    # Determine which seeds to test
    seeds = []
    if args.seed:
        seeds = [args.seed]
    elif args.seeds_file:
        seeds = load_seeds_from_file(args.seeds_file)
    elif args.lost_seeds:
        seeds = LOST_SEEDS
    elif args.random:
        seeds = [None] * args.num_games  # None = random seed

    if not seeds:
        logger.error("No seeds specified. Use --seed, --seeds-file, --lost-seeds, or --random")
        return

    # Get character class
    try:
        character = PlayerClass[args.character.upper()]
    except KeyError:
        logger.error(f"Invalid character: {args.character}")
        logger.error(f"Valid characters: {[c.name for c in PlayerClass]}")
        return

    # Run tests
    tester = SeedTester(model_path=args.model)

    if args.random:
        # For random seeds, we need to handle None specially
        logger.info(f"Running {args.num_games} games with random seeds...")
        for i in range(args.num_games):
            logger.info(f"\nGame {i+1}/{args.num_games}")
            tester.test_seed(
                seed=None,  # Random
                character=character,
                ascension=args.ascension,
            )
    else:
        tester.test_seeds(seeds, character, args.ascension)

    # Save results
    if args.output:
        tester.save_results(args.output)
    else:
        # Default output path
        output_path = Path("results") / f"seed_test_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        output_path.parent.mkdir(exist_ok=True)
        tester.save_results(output_path)


if __name__ == "__main__":
    main()
