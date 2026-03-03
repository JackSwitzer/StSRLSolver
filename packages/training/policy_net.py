"""Simple MLP policy/value network for StS RL.

Numpy-only implementation with Xavier initialization and ReLU activations.
Outputs action logits (for policy) and a scalar value estimate (for baseline).

The network is intentionally simple -- a 3-layer MLP is sufficient for
initial training.  A torch backend can be swapped in later without changing
the predict_action / save / load interface.

Usage:
    from packages.training.policy_net import PolicyValueNet

    net = PolicyValueNet(obs_dim=1186)
    logits, value = net.forward(obs)
    action = net.predict_action(obs, action_mask)
    net.save("checkpoints/policy_0.npz")
    net.load("checkpoints/policy_0.npz")
"""

from __future__ import annotations

from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np


def _softmax(x: np.ndarray) -> np.ndarray:
    """Numerically stable softmax."""
    e = np.exp(x - np.max(x))
    return e / e.sum()


def _xavier_init(fan_in: int, fan_out: int, rng: np.random.Generator) -> np.ndarray:
    """Xavier/Glorot uniform initialization."""
    limit = np.sqrt(6.0 / (fan_in + fan_out))
    return rng.uniform(-limit, limit, size=(fan_in, fan_out)).astype(np.float32)


class PolicyValueNet:
    """MLP that outputs action logits + state value.

    Architecture:
        obs -> [Linear + ReLU] * num_layers -> shared features
        shared features -> Linear -> action logits  (unbounded)
        shared features -> Linear -> tanh -> value  (in [-1, 1])

    Uses numpy only.  No torch dependency for core training loop.
    """

    def __init__(
        self,
        obs_dim: int = 1186,
        action_dim: int = 2048,
        hidden_dim: int = 256,
        num_layers: int = 3,
        seed: int = 42,
    ) -> None:
        self.obs_dim = obs_dim
        self.action_dim = action_dim
        self.hidden_dim = hidden_dim
        self.num_layers = num_layers
        self.rng = np.random.default_rng(seed)

        # Build weights: list of (W, b) tuples for hidden layers
        self.layers: List[Tuple[np.ndarray, np.ndarray]] = []
        in_dim = obs_dim
        for _ in range(num_layers):
            W = _xavier_init(in_dim, hidden_dim, self.rng)
            b = np.zeros(hidden_dim, dtype=np.float32)
            self.layers.append((W, b))
            in_dim = hidden_dim

        # Policy head: hidden -> action logits
        self.policy_W = _xavier_init(hidden_dim, action_dim, self.rng)
        self.policy_b = np.zeros(action_dim, dtype=np.float32)

        # Value head: hidden -> 1
        self.value_W = _xavier_init(hidden_dim, 1, self.rng)
        self.value_b = np.zeros(1, dtype=np.float32)

    def forward(self, obs: np.ndarray) -> Tuple[np.ndarray, float]:
        """Forward pass through the network.

        Args:
            obs: Observation array of shape (obs_dim,).

        Returns:
            (action_logits, value_estimate) where action_logits has shape
            (action_dim,) and value_estimate is a scalar float in [-1, 1].
        """
        x = obs.astype(np.float32)

        # Hidden layers with ReLU
        for W, b in self.layers:
            x = x @ W + b
            x = np.maximum(x, 0.0)  # ReLU

        # Policy head
        logits = x @ self.policy_W + self.policy_b

        # Value head (tanh squash to [-1, 1])
        value = float(np.tanh((x @ self.value_W + self.value_b)[0]))

        return logits, value

    def forward_batch(self, obs_batch: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
        """Batched forward pass.

        Args:
            obs_batch: Array of shape (batch, obs_dim).

        Returns:
            (logits_batch, values_batch) of shapes (batch, action_dim) and (batch,).
        """
        x = obs_batch.astype(np.float32)

        for W, b in self.layers:
            x = x @ W + b
            x = np.maximum(x, 0.0)

        logits = x @ self.policy_W + self.policy_b
        values = np.tanh(x @ self.value_W + self.value_b).flatten()

        return logits, values

    def predict_action(
        self,
        obs: np.ndarray,
        action_mask: np.ndarray,
        temperature: float = 1.0,
    ) -> int:
        """Sample an action from the masked policy distribution.

        Args:
            obs: Observation array of shape (obs_dim,).
            action_mask: Boolean mask of shape (action_dim,). True = legal.
            temperature: Sampling temperature. 0 = greedy, 1 = proportional.

        Returns:
            Sampled action index (int).
        """
        logits, _ = self.forward(obs)

        # Mask invalid actions
        masked = np.where(action_mask[:len(logits)], logits[:len(action_mask)], -1e9)

        # Pad or trim to match mask length
        if len(masked) < len(action_mask):
            padded = np.full(len(action_mask), -1e9, dtype=np.float32)
            padded[:len(masked)] = masked
            masked = padded

        if temperature == 0.0:
            return int(np.argmax(masked))

        # Apply temperature
        scaled = masked / max(temperature, 1e-8)
        probs = _softmax(scaled)

        # Safety: ensure probabilities are valid
        probs = np.clip(probs, 0.0, None)
        total = probs.sum()
        if total < 1e-10:
            # All actions masked -- fall back to uniform over legal actions
            probs = action_mask.astype(np.float32)
            total = probs.sum()
            if total < 1e-10:
                return 0
            probs /= total

        probs /= probs.sum()  # re-normalize
        return int(self.rng.choice(len(probs), p=probs))

    def get_log_probs_and_values(
        self,
        obs_batch: np.ndarray,
        actions: np.ndarray,
        action_masks: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """Compute log probabilities, values, and entropy for a batch.

        Args:
            obs_batch: (batch, obs_dim)
            actions: (batch,) integer action indices
            action_masks: (batch, action_dim) boolean masks

        Returns:
            (log_probs, values, entropy) each of shape (batch,).
        """
        logits_batch, values = self.forward_batch(obs_batch)
        batch_size = logits_batch.shape[0]

        log_probs = np.zeros(batch_size, dtype=np.float32)
        entropy = np.zeros(batch_size, dtype=np.float32)

        for i in range(batch_size):
            logits = logits_batch[i]
            mask = action_masks[i]

            # Mask invalid
            masked = np.where(mask[:len(logits)], logits[:len(mask)], -1e9)

            # Softmax
            probs = _softmax(masked)
            probs = np.clip(probs, 1e-10, 1.0)

            log_probs[i] = np.log(probs[actions[i]])
            entropy[i] = -np.sum(probs * np.log(probs))

        return log_probs, values, entropy

    # ------------------------------------------------------------------
    # SGD update (simple numpy-based)
    # ------------------------------------------------------------------

    def apply_gradients(
        self,
        obs_batch: np.ndarray,
        actions: np.ndarray,
        advantages: np.ndarray,
        returns: np.ndarray,
        action_masks: np.ndarray,
        learning_rate: float = 3e-4,
        entropy_coeff: float = 0.01,
        value_coeff: float = 0.5,
    ) -> Dict[str, float]:
        """Apply a REINFORCE-style gradient update using finite differences.

        This is a simple numerical gradient approach.  For production use,
        swap in a torch backend.  The interface stays the same.

        Args:
            obs_batch: (batch, obs_dim)
            actions: (batch,) integer action indices
            advantages: (batch,) advantage estimates
            returns: (batch,) discounted returns for value loss
            action_masks: (batch, action_dim) boolean masks
            learning_rate: SGD step size
            entropy_coeff: Entropy bonus coefficient
            value_coeff: Value loss coefficient

        Returns:
            Dict with 'policy_loss', 'value_loss', 'entropy' scalars.
        """
        log_probs, values, ent = self.get_log_probs_and_values(
            obs_batch, actions, action_masks
        )

        # REINFORCE loss: -E[log_pi * advantage]
        policy_loss = -np.mean(log_probs * advantages)

        # Value loss: MSE
        value_loss = np.mean((values - returns) ** 2)

        # Entropy bonus (we want to maximize entropy)
        mean_entropy = np.mean(ent)

        # Total loss (for logging; actual update is param-by-param below)
        total_loss = policy_loss + value_coeff * value_loss - entropy_coeff * mean_entropy

        # Numerical gradient update for all parameters
        eps = 1e-3
        all_params = self._get_all_params()
        grads = self._estimate_gradients(
            obs_batch, actions, advantages, returns, action_masks,
            value_coeff, entropy_coeff, eps,
        )

        # Apply SGD
        idx = 0
        for layer_i, (W, b) in enumerate(self.layers):
            W_flat_size = W.size
            b_flat_size = b.size
            W -= learning_rate * grads[idx:idx + W_flat_size].reshape(W.shape)
            b -= learning_rate * grads[idx + W_flat_size:idx + W_flat_size + b_flat_size]
            idx += W_flat_size + b_flat_size

        pW_size = self.policy_W.size
        pb_size = self.policy_b.size
        self.policy_W -= learning_rate * grads[idx:idx + pW_size].reshape(self.policy_W.shape)
        self.policy_b -= learning_rate * grads[idx + pW_size:idx + pW_size + pb_size]
        idx += pW_size + pb_size

        vW_size = self.value_W.size
        vb_size = self.value_b.size
        self.value_W -= learning_rate * grads[idx:idx + vW_size].reshape(self.value_W.shape)
        self.value_b -= learning_rate * grads[idx + vW_size:idx + vW_size + vb_size]

        return {
            "policy_loss": float(policy_loss),
            "value_loss": float(value_loss),
            "entropy": float(mean_entropy),
            "total_loss": float(total_loss),
        }

    def _get_all_params(self) -> np.ndarray:
        """Flatten all parameters into a single vector."""
        parts = []
        for W, b in self.layers:
            parts.append(W.flatten())
            parts.append(b.flatten())
        parts.append(self.policy_W.flatten())
        parts.append(self.policy_b.flatten())
        parts.append(self.value_W.flatten())
        parts.append(self.value_b.flatten())
        return np.concatenate(parts)

    def _loss_fn(
        self,
        obs_batch: np.ndarray,
        actions: np.ndarray,
        advantages: np.ndarray,
        returns: np.ndarray,
        action_masks: np.ndarray,
        value_coeff: float,
        entropy_coeff: float,
    ) -> float:
        """Compute scalar loss for current parameters."""
        log_probs, values, ent = self.get_log_probs_and_values(
            obs_batch, actions, action_masks
        )
        policy_loss = -np.mean(log_probs * advantages)
        value_loss = np.mean((values - returns) ** 2)
        mean_entropy = np.mean(ent)
        return float(policy_loss + value_coeff * value_loss - entropy_coeff * mean_entropy)

    def _estimate_gradients(
        self,
        obs_batch: np.ndarray,
        actions: np.ndarray,
        advantages: np.ndarray,
        returns: np.ndarray,
        action_masks: np.ndarray,
        value_coeff: float,
        entropy_coeff: float,
        eps: float,
    ) -> np.ndarray:
        """Estimate gradients via forward finite differences.

        This is O(n_params) forward passes -- extremely slow for large networks.
        For real training, use a torch backend.  This exists so that the
        training loop can run end-to-end with zero dependencies beyond numpy.
        """
        # For large param counts, use a random subset (SPSA-style)
        all_params = self._get_all_params()
        n_params = len(all_params)

        # SPSA: random perturbation gradient estimate (much faster)
        grads = np.zeros(n_params, dtype=np.float32)
        n_samples = min(n_params, 64)  # Limit for speed

        base_loss = self._loss_fn(
            obs_batch, actions, advantages, returns, action_masks,
            value_coeff, entropy_coeff,
        )

        for _ in range(n_samples):
            # Random perturbation direction
            delta = self.rng.choice([-1.0, 1.0], size=n_params).astype(np.float32)

            # Perturb parameters
            self._set_all_params(all_params + eps * delta)
            loss_plus = self._loss_fn(
                obs_batch, actions, advantages, returns, action_masks,
                value_coeff, entropy_coeff,
            )

            # Approximate gradient
            grads += (loss_plus - base_loss) / eps * delta

        grads /= n_samples

        # Restore original parameters
        self._set_all_params(all_params)

        return grads

    def _set_all_params(self, flat_params: np.ndarray) -> None:
        """Set all parameters from a flat vector."""
        idx = 0
        for layer_i in range(len(self.layers)):
            W, b = self.layers[layer_i]
            W_size = W.size
            b_size = b.size
            new_W = flat_params[idx:idx + W_size].reshape(W.shape).astype(np.float32)
            new_b = flat_params[idx + W_size:idx + W_size + b_size].astype(np.float32)
            self.layers[layer_i] = (new_W, new_b)
            idx += W_size + b_size

        pW_size = self.policy_W.size
        pb_size = self.policy_b.size
        self.policy_W = flat_params[idx:idx + pW_size].reshape(self.policy_W.shape).astype(np.float32)
        self.policy_b = flat_params[idx + pW_size:idx + pW_size + pb_size].astype(np.float32)
        idx += pW_size + pb_size

        vW_size = self.value_W.size
        vb_size = self.value_b.size
        self.value_W = flat_params[idx:idx + vW_size].reshape(self.value_W.shape).astype(np.float32)
        self.value_b = flat_params[idx + vW_size:idx + vW_size + vb_size].astype(np.float32)

    # ------------------------------------------------------------------
    # Persistence
    # ------------------------------------------------------------------

    def save(self, path: str) -> None:
        """Save all weights to a .npz file.

        Args:
            path: File path (should end with .npz).
        """
        save_dict = {
            "obs_dim": np.array([self.obs_dim]),
            "action_dim": np.array([self.action_dim]),
            "hidden_dim": np.array([self.hidden_dim]),
            "num_layers": np.array([self.num_layers]),
            "policy_W": self.policy_W,
            "policy_b": self.policy_b,
            "value_W": self.value_W,
            "value_b": self.value_b,
        }
        for i, (W, b) in enumerate(self.layers):
            save_dict[f"layer_{i}_W"] = W
            save_dict[f"layer_{i}_b"] = b

        Path(path).parent.mkdir(parents=True, exist_ok=True)
        np.savez(path, **save_dict)

    def load(self, path: str) -> None:
        """Load weights from a .npz file.

        Args:
            path: File path. A .npz extension is appended if missing.
        """
        if not path.endswith(".npz"):
            path = path + ".npz"
        data = np.load(path)

        self.obs_dim = int(data["obs_dim"][0])
        self.action_dim = int(data["action_dim"][0])
        self.hidden_dim = int(data["hidden_dim"][0])
        self.num_layers = int(data["num_layers"][0])

        self.layers = []
        for i in range(self.num_layers):
            W = data[f"layer_{i}_W"].astype(np.float32)
            b = data[f"layer_{i}_b"].astype(np.float32)
            self.layers.append((W, b))

        self.policy_W = data["policy_W"].astype(np.float32)
        self.policy_b = data["policy_b"].astype(np.float32)
        self.value_W = data["value_W"].astype(np.float32)
        self.value_b = data["value_b"].astype(np.float32)

    @property
    def param_count(self) -> int:
        """Total number of trainable parameters."""
        total = 0
        for W, b in self.layers:
            total += W.size + b.size
        total += self.policy_W.size + self.policy_b.size
        total += self.value_W.size + self.value_b.size
        return total
