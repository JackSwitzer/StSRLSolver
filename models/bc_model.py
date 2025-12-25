"""
Behavioral Cloning model for Slay the Spire.

A simple MLP that learns to predict expert decisions from game state.
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import List, Tuple
import numpy as np

class CardPickerBC(nn.Module):
    """
    Neural network for predicting which card to pick from rewards.

    Architecture: MLP with residual connections
    Input: Game state (deck, relics, HP, gold, floor, etc.)
    Output: Distribution over card pick actions
    """

    def __init__(
        self,
        state_dim: int,
        action_dim: int,
        hidden_dims: List[int] = [512, 256, 128],
        dropout: float = 0.2,
    ):
        super().__init__()

        self.state_dim = state_dim
        self.action_dim = action_dim

        # Input layer
        layers = []
        prev_dim = state_dim
        for hidden_dim in hidden_dims:
            layers.extend([
                nn.Linear(prev_dim, hidden_dim),
                nn.LayerNorm(hidden_dim),
                nn.GELU(),
                nn.Dropout(dropout),
            ])
            prev_dim = hidden_dim

        self.encoder = nn.Sequential(*layers)

        # Output head
        self.action_head = nn.Linear(prev_dim, action_dim)

    def forward(self, state: torch.Tensor) -> torch.Tensor:
        """
        Forward pass.

        Args:
            state: Batch of game states [batch_size, state_dim]

        Returns:
            Action logits [batch_size, action_dim]
        """
        features = self.encoder(state)
        logits = self.action_head(features)
        return logits

    def predict(self, state: torch.Tensor, available_actions: torch.Tensor = None) -> int:
        """
        Predict best action for a single state.

        Args:
            state: Single game state [state_dim]
            available_actions: Optional mask of available actions [action_dim]

        Returns:
            Index of best action
        """
        self.eval()
        with torch.no_grad():
            logits = self.forward(state.unsqueeze(0))[0]

            if available_actions is not None:
                # Mask unavailable actions with large negative value
                logits = logits.masked_fill(~available_actions.bool(), float('-inf'))

            return logits.argmax().item()

    def get_action_probs(self, state: torch.Tensor) -> torch.Tensor:
        """Get probability distribution over actions."""
        self.eval()
        with torch.no_grad():
            logits = self.forward(state.unsqueeze(0))[0]
            return F.softmax(logits, dim=-1)


class PathPickerBC(nn.Module):
    """
    Neural network for predicting which path to take on the map.
    """

    def __init__(
        self,
        state_dim: int,
        num_paths: int = 7,
        hidden_dims: List[int] = [256, 128],
        dropout: float = 0.2,
    ):
        super().__init__()

        layers = []
        prev_dim = state_dim
        for hidden_dim in hidden_dims:
            layers.extend([
                nn.Linear(prev_dim, hidden_dim),
                nn.LayerNorm(hidden_dim),
                nn.GELU(),
                nn.Dropout(dropout),
            ])
            prev_dim = hidden_dim

        self.encoder = nn.Sequential(*layers)
        self.path_head = nn.Linear(prev_dim, num_paths)

    def forward(self, state: torch.Tensor) -> torch.Tensor:
        features = self.encoder(state)
        return self.path_head(features)


class CombatPolicyBC(nn.Module):
    """
    Neural network for combat decisions.

    This is more complex as it needs to handle:
    - Which card to play
    - Which target to select (if applicable)
    - When to end turn
    """

    def __init__(
        self,
        state_dim: int,
        max_hand_size: int = 10,
        max_targets: int = 5,
        hidden_dims: List[int] = [512, 256],
        dropout: float = 0.2,
    ):
        super().__init__()

        self.max_hand_size = max_hand_size
        self.max_targets = max_targets

        # Shared encoder
        layers = []
        prev_dim = state_dim
        for hidden_dim in hidden_dims:
            layers.extend([
                nn.Linear(prev_dim, hidden_dim),
                nn.LayerNorm(hidden_dim),
                nn.GELU(),
                nn.Dropout(dropout),
            ])
            prev_dim = hidden_dim

        self.encoder = nn.Sequential(*layers)

        # Card selection head (which card in hand to play, or end turn)
        self.card_head = nn.Linear(prev_dim, max_hand_size + 1)  # +1 for end turn

        # Target selection head
        self.target_head = nn.Linear(prev_dim, max_targets)

    def forward(self, state: torch.Tensor) -> Tuple[torch.Tensor, torch.Tensor]:
        features = self.encoder(state)
        card_logits = self.card_head(features)
        target_logits = self.target_head(features)
        return card_logits, target_logits


# === TRAINING UTILITIES ===

class BCTrainer:
    """Training loop for behavioral cloning."""

    def __init__(
        self,
        model: nn.Module,
        lr: float = 1e-3,
        weight_decay: float = 1e-4,
        device: str = "cpu",
    ):
        self.model = model.to(device)
        self.device = device
        self.optimizer = torch.optim.AdamW(
            model.parameters(), lr=lr, weight_decay=weight_decay
        )
        self.scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
            self.optimizer, T_max=100, eta_min=1e-6
        )
        self.criterion = nn.CrossEntropyLoss()

    def train_epoch(
        self,
        train_loader: torch.utils.data.DataLoader,
    ) -> float:
        """Train for one epoch, return average loss."""
        self.model.train()
        total_loss = 0
        num_batches = 0

        for states, actions in train_loader:
            states = states.to(self.device)
            actions = actions.to(self.device)

            self.optimizer.zero_grad()
            logits = self.model(states)
            loss = self.criterion(logits, actions)
            loss.backward()

            # Gradient clipping
            torch.nn.utils.clip_grad_norm_(self.model.parameters(), 1.0)

            self.optimizer.step()

            total_loss += loss.item()
            num_batches += 1

        self.scheduler.step()
        return total_loss / max(num_batches, 1)

    def evaluate(
        self,
        val_loader: torch.utils.data.DataLoader,
    ) -> Tuple[float, float]:
        """Evaluate model, return (loss, accuracy)."""
        self.model.eval()
        total_loss = 0
        correct = 0
        total = 0

        with torch.no_grad():
            for states, actions in val_loader:
                states = states.to(self.device)
                actions = actions.to(self.device)

                logits = self.model(states)
                loss = self.criterion(logits, actions)

                total_loss += loss.item() * states.size(0)
                predictions = logits.argmax(dim=1)
                correct += (predictions == actions).sum().item()
                total += states.size(0)

        avg_loss = total_loss / max(total, 1)
        accuracy = correct / max(total, 1)
        return avg_loss, accuracy

    def save(self, path: str):
        """Save model checkpoint."""
        torch.save({
            'model_state_dict': self.model.state_dict(),
            'optimizer_state_dict': self.optimizer.state_dict(),
            'scheduler_state_dict': self.scheduler.state_dict(),
        }, path)

    def load(self, path: str):
        """Load model checkpoint."""
        checkpoint = torch.load(path, map_location=self.device)
        self.model.load_state_dict(checkpoint['model_state_dict'])
        self.optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        self.scheduler.load_state_dict(checkpoint['scheduler_state_dict'])


if __name__ == "__main__":
    # Test model creation
    from encoding import get_state_dim, get_card_pick_action_dim

    state_dim = get_state_dim()
    action_dim = get_card_pick_action_dim()

    print(f"Creating CardPickerBC model:")
    print(f"  State dim: {state_dim}")
    print(f"  Action dim: {action_dim}")

    model = CardPickerBC(state_dim, action_dim)
    print(f"  Parameters: {sum(p.numel() for p in model.parameters()):,}")

    # Test forward pass
    batch = torch.randn(32, state_dim)
    logits = model(batch)
    print(f"  Output shape: {logits.shape}")
