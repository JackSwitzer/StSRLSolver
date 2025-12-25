#!/usr/bin/env python3
"""
Train Behavioral Cloning model on Slay the Spire expert data.

IMPORTANT: Only trains on WINS (victories) to learn winning strategies.

Usage:
    uv run python3 train_bc.py --data data/watcher_training/watcher_a20_wins.json
"""

import argparse
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import List, Tuple, Dict, Any

import numpy as np
import torch
from torch.utils.data import Dataset, DataLoader, random_split

from models.encoding import (
    encode_game_state,
    encode_card_choice,
    encode_deck,
    encode_relics,
    get_state_dim,
    get_card_pick_action_dim,
    normalize_card_name,
    CARD_TO_IDX,
)
from models.bc_model import CardPickerBC, BCTrainer

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class CardPickDataset(Dataset):
    """Dataset for card pick decisions from winning runs only."""

    def __init__(self, data_path: Path, character_filter: str = "WATCHER"):
        self.examples: List[Tuple[np.ndarray, int]] = []
        self.stats = {
            "total_runs": 0,
            "winning_runs": 0,
            "watcher_wins": 0,
            "total_choices": 0,
            "valid_choices": 0,
            "skipped_choices": 0,
        }

        self._load_data(data_path, character_filter)

    def _load_data(self, data_path: Path, character_filter: str):
        """Load and process run data, keeping only wins."""
        logger.info(f"Loading data from {data_path}")

        with open(data_path) as f:
            runs = json.load(f)

        # Handle both list and single-run formats
        if not isinstance(runs, list):
            runs = [runs]

        self.stats["total_runs"] = len(runs)

        for run in runs:
            # === CRITICAL: Only include WINS ===
            if not run.get("victory", False):
                continue

            self.stats["winning_runs"] += 1

            # Filter by character (skip if data is pre-filtered)
            character = run.get("character_chosen", "")
            if character_filter and character:
                if character.upper() != character_filter.upper():
                    continue

            self.stats["watcher_wins"] += 1

            # Skip invalid runs
            if not self._is_valid_run(run):
                continue

            # Process card choices
            self._process_card_choices(run)

        logger.info(f"Data loading complete:")
        logger.info(f"  Total runs: {self.stats['total_runs']}")
        logger.info(f"  Winning runs: {self.stats['winning_runs']}")
        logger.info(f"  Watcher wins: {self.stats['watcher_wins']}")
        logger.info(f"  Total card choices: {self.stats['total_choices']}")
        logger.info(f"  Valid examples: {self.stats['valid_choices']}")
        logger.info(f"  Skipped (SKIP action): {self.stats['skipped_choices']}")

    def _is_valid_run(self, run: Dict[str, Any]) -> bool:
        """Check if run data is valid and complete."""
        # Must have card choices
        if not run.get("card_choices"):
            return False

        # Must have reached at least floor 17 (beat first boss)
        if run.get("floor_reached", 0) < 17:
            return False

        return True

    def _process_card_choices(self, run: Dict[str, Any]):
        """Extract training examples from card choices."""
        card_choices = run.get("card_choices", [])

        for choice in card_choices:
            self.stats["total_choices"] += 1

            picked = choice.get("picked", "")
            floor = choice.get("floor", 0)

            # Skip if SKIP action (optional: could include these)
            if not picked or picked.upper() == "SKIP":
                self.stats["skipped_choices"] += 1
                continue

            # Encode the action (card picked)
            action = encode_card_choice(choice)

            # Skip unknown cards
            if action == 0:  # 0 is SKIP/unknown
                continue

            # Build approximate state at this floor
            state = self._build_state_at_floor(run, int(floor))

            self.examples.append((state, action))
            self.stats["valid_choices"] += 1

    def _build_state_at_floor(self, run: Dict[str, Any], floor: int) -> np.ndarray:
        """Build game state vector at a given floor."""
        # For now, use simplified state (full deck at end)
        # TODO: Reconstruct deck evolution for more accurate states
        return encode_game_state(run, floor)

    def __len__(self):
        return len(self.examples)

    def __getitem__(self, idx):
        state, action = self.examples[idx]
        return torch.FloatTensor(state), torch.LongTensor([action])[0]


def train(
    data_path: Path,
    output_dir: Path,
    epochs: int = 100,
    batch_size: int = 64,
    lr: float = 1e-3,
    device: str = "cpu",
):
    """Main training loop."""
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load data
    dataset = CardPickDataset(data_path, character_filter="WATCHER")

    if len(dataset) == 0:
        logger.error("No valid training examples found!")
        return

    logger.info(f"Total training examples: {len(dataset)}")

    # Split into train/val
    train_size = int(0.9 * len(dataset))
    val_size = len(dataset) - train_size
    train_dataset, val_dataset = random_split(dataset, [train_size, val_size])

    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        shuffle=True,
        num_workers=0,
    )
    val_loader = DataLoader(
        val_dataset,
        batch_size=batch_size,
        shuffle=False,
        num_workers=0,
    )

    logger.info(f"Train size: {len(train_dataset)}, Val size: {len(val_dataset)}")

    # Create model
    state_dim = get_state_dim()
    action_dim = get_card_pick_action_dim()

    model = CardPickerBC(state_dim, action_dim)
    trainer = BCTrainer(model, lr=lr, device=device)

    logger.info(f"Model parameters: {sum(p.numel() for p in model.parameters()):,}")

    # Training loop
    best_val_acc = 0
    for epoch in range(epochs):
        train_loss = trainer.train_epoch(train_loader)
        val_loss, val_acc = trainer.evaluate(val_loader)

        logger.info(
            f"Epoch {epoch+1}/{epochs} | "
            f"Train Loss: {train_loss:.4f} | "
            f"Val Loss: {val_loss:.4f} | "
            f"Val Acc: {val_acc:.4f}"
        )

        # Save best model
        if val_acc > best_val_acc:
            best_val_acc = val_acc
            trainer.save(output_dir / "best_model.pt")
            logger.info(f"  New best model saved (acc: {val_acc:.4f})")

        # Save checkpoint every 10 epochs
        if (epoch + 1) % 10 == 0:
            trainer.save(output_dir / f"checkpoint_{epoch+1}.pt")

    # Save final model
    trainer.save(output_dir / "final_model.pt")
    logger.info(f"Training complete! Best validation accuracy: {best_val_acc:.4f}")

    # Save training config
    config = {
        "data_path": str(data_path),
        "epochs": epochs,
        "batch_size": batch_size,
        "lr": lr,
        "train_size": len(train_dataset),
        "val_size": len(val_dataset),
        "best_val_acc": best_val_acc,
        "state_dim": state_dim,
        "action_dim": action_dim,
    }
    with open(output_dir / "config.json", "w") as f:
        json.dump(config, f, indent=2)


def main():
    parser = argparse.ArgumentParser(description="Train BC model on STS data")
    parser.add_argument(
        "--data",
        type=Path,
        default=Path("data/watcher_training/watcher_a20_wins.json"),
        help="Path to training data JSON",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("checkpoints") / datetime.now().strftime("%Y%m%d_%H%M%S"),
        help="Output directory for checkpoints",
    )
    parser.add_argument("--epochs", type=int, default=100)
    parser.add_argument("--batch-size", type=int, default=64)
    parser.add_argument("--lr", type=float, default=1e-3)
    parser.add_argument(
        "--device",
        type=str,
        default="mps" if torch.backends.mps.is_available() else "cpu",
        help="Device to train on (cpu, cuda, mps)",
    )

    args = parser.parse_args()

    logger.info("=== STS Behavioral Cloning Training ===")
    logger.info(f"Data: {args.data}")
    logger.info(f"Output: {args.output}")
    logger.info(f"Device: {args.device}")
    logger.info(f"Epochs: {args.epochs}")

    if not args.data.exists():
        logger.error(f"Data file not found: {args.data}")
        logger.error("Please download and process training data first:")
        logger.error("  1. Download 2020 data from Google Drive")
        logger.error("  2. Run: uv run python3 data/process_watcher_data.py")
        return

    train(
        data_path=args.data,
        output_dir=args.output,
        epochs=args.epochs,
        batch_size=args.batch_size,
        lr=args.lr,
        device=args.device,
    )


if __name__ == "__main__":
    main()
