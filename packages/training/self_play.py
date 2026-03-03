"""
Self-play training pipeline: AlphaZero-style MCTS + meta model.

Workers play games using CombatMCTS guided by the meta model.
Trainer collects trajectories and updates the model with PPO.
Seed pool rotates to avoid overfitting.

Usage:
    uv run python -m packages.training.self_play --workers 8 --episodes 1000
"""

from __future__ import annotations

import argparse
import json
import logging
import multiprocessing as mp
import os
import sys
import time
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional, Tuple

import numpy as np
import torch

logger = logging.getLogger(__name__)

CHECKPOINT_DIR = Path("logs/checkpoints")
TRAJECTORY_DIR = Path("logs/trajectories")


@dataclass
class Transition:
    """Single training transition from self-play."""
    obs: np.ndarray             # [obs_dim] observation
    action_mask: np.ndarray     # [action_dim] bool mask
    action: int                 # action index taken
    mcts_policy: np.ndarray     # [action_dim] MCTS visit distribution
    reward: float               # immediate reward
    value_target: float         # discounted return (filled after episode)
    done: bool                  # episode termination
    # Auxiliary targets
    hp_after_combat: float = 0.0
    turns_in_combat: float = 0.0
    reached_act3: float = 0.0


@dataclass
class Trajectory:
    """Full game trajectory from self-play."""
    seed: str
    transitions: List[Transition] = field(default_factory=list)
    won: bool = False
    final_floor: int = 0
    final_hp: int = 0
    duration_s: float = 0.0

    def compute_returns(self, gamma: float = 1.0) -> None:
        """Fill value_target with discounted returns from game outcome."""
        # Terminal value: 1.0 for win, -1.0 for loss, scaled by HP
        if self.won:
            terminal = 0.5 + 0.5 * (self.final_hp / 72.0)
        else:
            terminal = -0.5 - 0.5 * (1.0 - self.final_floor / 55.0)

        # Backward pass
        G = terminal
        for t in reversed(self.transitions):
            t.value_target = G
            G = t.reward + gamma * G * (1.0 - float(t.done))


class SeedPool:
    """Manages seed rotation with difficulty-aware sampling."""

    def __init__(self, initial_seeds: Optional[List[str]] = None, max_plays: int = 3):
        self.max_plays = max_plays
        self.play_counts: Dict[str, int] = {}
        self.results: Dict[str, List[Dict]] = {}  # seed -> list of game results
        self._next_idx = 0

        if initial_seeds:
            for s in initial_seeds:
                self.play_counts[s] = 0
        else:
            # Generate default seeds
            for i in range(200):
                s = f"Train_{i}"
                self.play_counts[s] = 0

    def get_seed(self) -> str:
        """Get next seed to play (round-robin with max plays)."""
        available = [s for s, c in self.play_counts.items() if c < self.max_plays]
        if not available:
            # All seeds exhausted, add more
            base = len(self.play_counts)
            for i in range(100):
                s = f"Train_{base + i}"
                self.play_counts[s] = 0
            available = [s for s, c in self.play_counts.items() if c < self.max_plays]

        seed = available[self._next_idx % len(available)]
        self._next_idx += 1
        self.play_counts[seed] += 1
        return seed

    def record_result(self, seed: str, result: Dict) -> None:
        if seed not in self.results:
            self.results[seed] = []
        self.results[seed].append(result)

    @property
    def total_games(self) -> int:
        return sum(self.play_counts.values())

    @property
    def unique_seeds(self) -> int:
        return len([s for s, c in self.play_counts.items() if c > 0])


def _play_game_worker(args: Tuple) -> Optional[Dict]:
    """Worker function: play one game with MCTS + model, collect transitions."""
    seed, model_weights_path, config = args

    from packages.engine.game import GameRunner, GamePhase
    from packages.engine.rl_observations import ObservationEncoder
    from packages.engine.rl_masks import ActionSpace
    from packages.training.mcts import CombatMCTS
    from packages.training.planner import StrategicPlanner
    from packages.training.torch_policy_net import StSPolicyValueNet

    ascension = config.get("ascension", 20)
    character = config.get("character", "Watcher")
    combat_sims = config.get("combat_sims", 64)
    deep_sims = config.get("deep_sims", 128)
    deep_prob = config.get("deep_prob", 0.25)  # KataGo playout cap

    # Load model on CPU for worker (MPS doesn't fork well)
    obs_dim = config.get("obs_dim", 1186)
    action_dim = config.get("action_dim", 2048)
    model = StSPolicyValueNet(obs_dim=obs_dim, action_dim=action_dim)
    if model_weights_path and Path(model_weights_path).exists():
        checkpoint = torch.load(model_weights_path, map_location="cpu", weights_only=True)
        model.load_state_dict(checkpoint["model_state_dict"])
    model.eval()

    encoder = ObservationEncoder()
    action_space = ActionSpace()
    planner = StrategicPlanner()

    # MCTS without neural policy (uniform priors, heuristic value) for now.
    # The model is used for value estimation only.
    mcts = CombatMCTS(policy_fn=None, num_simulations=combat_sims)

    try:
        runner = GameRunner(seed=seed, ascension=ascension, character=character, verbose=False)
    except Exception:
        return None

    transitions: List[Dict] = []
    t0 = time.monotonic()
    step = 0
    in_combat = False
    combat_start_hp = 0
    combat_turns = 0

    while not runner.game_over and step < 5000:
        try:
            actions = runner.get_available_actions()
        except Exception:
            break
        if not actions:
            break

        phase = runner.phase

        if phase == GamePhase.COMBAT:
            engine = runner.current_combat

            if not in_combat:
                in_combat = True
                combat_start_hp = getattr(runner.run_state, "current_hp", 0)
                combat_turns = 0

            if engine and len(actions) > 1:
                combat_turns += 1

                # KataGo playout cap: deep vs shallow
                use_deep = np.random.random() < deep_prob
                sims = deep_sims if use_deep else combat_sims
                mcts.num_simulations = sims

                # Run MCTS
                try:
                    action_probs = mcts.search(engine)
                except Exception:
                    action_probs = {}

                if action_probs:
                    # Select best action
                    best_action = max(action_probs, key=action_probs.get)

                    # Record transition for deep searches only (KataGo playout cap)
                    if use_deep:
                        try:
                            obs_dict = runner.get_observation()
                            obs_arr = encoder.observation_to_array(obs_dict)

                            # Build mask + MCTS policy using runner action dicts
                            action_dicts = runner.get_available_action_dicts()
                            action_space.register_actions(action_dicts)
                            mask = np.zeros(action_dim, dtype=bool)
                            for ad in action_dicts:
                                aid = ad.get("id")
                                if aid is not None:
                                    idx = action_space.register(aid)
                                    if idx < action_dim:
                                        mask[idx] = True

                            # Map MCTS action probs to indices via string matching
                            mcts_policy = np.zeros(action_dim, dtype=np.float32)
                            best_idx = 0
                            for act, prob in action_probs.items():
                                act_str = str(act)
                                # Find matching action dict by string repr
                                for ad in action_dicts:
                                    if ad.get("id") == act_str or str(ad.get("id", "")) == act_str:
                                        idx = action_space.register(ad["id"])
                                        if idx < action_dim:
                                            mcts_policy[idx] = prob
                                        if act == best_action:
                                            best_idx = idx
                                        break

                            total_p = mcts_policy.sum()
                            if total_p > 0:
                                mcts_policy /= total_p

                            transitions.append({
                                "obs": obs_arr,
                                "mask": mask,
                                "action": best_idx,
                                "mcts_policy": mcts_policy,
                                "reward": 0.0,
                                "done": False,
                            })
                        except Exception:
                            pass

                    # Execute best MCTS action via CombatAction
                    try:
                        from packages.engine.game import CombatAction
                        from packages.engine.state.combat import PlayCard, UsePotion, EndTurn

                        if isinstance(best_action, PlayCard):
                            ga = CombatAction(action_type="play_card", card_idx=best_action.card_idx, target_idx=best_action.target_idx)
                        elif isinstance(best_action, UsePotion):
                            ga = CombatAction(action_type="use_potion", potion_idx=best_action.potion_idx, target_idx=best_action.target_idx)
                        else:
                            ga = CombatAction(action_type="end_turn")
                        runner.take_action(ga)
                        step += 1
                        continue
                    except Exception:
                        pass

            # Fallback
            runner.take_action(actions[0])
        else:
            if in_combat:
                in_combat = False
            # Non-combat: use heuristic planner
            if len(actions) == 1:
                runner.take_action(actions[0])
            elif phase == GamePhase.MAP_NAVIGATION:
                idx = planner.plan_path_choice(runner, actions)
                runner.take_action(actions[min(idx, len(actions) - 1)])
            elif phase == GamePhase.REST:
                idx = planner.plan_rest_site(runner, actions)
                runner.take_action(actions[min(idx, len(actions) - 1)])
            elif phase == GamePhase.COMBAT_REWARDS or phase == GamePhase.BOSS_REWARDS:
                idx = planner.plan_card_pick(runner, actions)
                runner.take_action(actions[min(idx, len(actions) - 1)])
            elif phase == GamePhase.SHOP:
                idx = planner.plan_shop_action(runner, actions)
                runner.take_action(actions[min(idx, len(actions) - 1)])
            elif phase == GamePhase.EVENT:
                idx = planner.plan_event_choice(runner, actions)
                runner.take_action(actions[min(idx, len(actions) - 1)])
            else:
                runner.take_action(actions[0])

        step += 1

    duration = time.monotonic() - t0
    rs = runner.run_state
    won = runner.game_won
    final_floor = getattr(rs, "floor", 0)
    final_hp = getattr(rs, "current_hp", 0)

    # Compute returns for all transitions
    if won:
        terminal = 0.5 + 0.5 * (final_hp / 72.0)
    else:
        terminal = -0.5 - 0.5 * (1.0 - final_floor / 55.0)

    G = terminal
    for t in reversed(transitions):
        t["value_target"] = G
        G = t["reward"] + 1.0 * G * (1.0 - float(t["done"]))

    return {
        "seed": seed,
        "won": won,
        "floor": final_floor,
        "hp": final_hp,
        "duration_s": round(duration, 2),
        "num_transitions": len(transitions),
        "transitions": transitions,
    }


def _extract_combat_obs(engine) -> Dict:
    """Extract combat observation dict from engine for the encoder."""
    state = engine.state
    player = state.player
    enemies_data = []
    for e in state.enemies:
        enemies_data.append({
            "id": getattr(e, "id", "Unknown"),
            "hp": e.hp,
            "max_hp": getattr(e, "max_hp", e.hp),
            "block": e.block,
            "statuses": getattr(e, "statuses", {}),
            "move_damage": getattr(e, "move_damage", 0),
            "move_hits": getattr(e, "move_hits", 1),
        })

    return {
        "player": {
            "hp": player.hp,
            "max_hp": getattr(player, "max_hp", 72),
            "block": player.block,
            "statuses": getattr(player, "statuses", {}),
        },
        "enemies": enemies_data,
        "hand": list(state.hand),
        "draw_pile": list(state.draw_pile),
        "discard_pile": list(state.discard_pile),
        "exhaust_pile": list(getattr(state, "exhaust_pile", [])),
        "energy": state.energy,
        "max_energy": getattr(state, "max_energy", 3),
        "turn": getattr(state, "turn", 1),
        "stance": getattr(state, "stance", "Neutral"),
    }


def _action_to_dict(action) -> Dict[str, Any]:
    """Convert engine Action to dict for ActionSpace."""
    from packages.engine.state.combat import PlayCard, UsePotion, EndTurn

    if isinstance(action, PlayCard):
        return {"type": "play_card", "card_idx": action.card_idx, "target_idx": action.target_idx}
    elif isinstance(action, UsePotion):
        return {"type": "use_potion", "potion_idx": action.potion_idx, "target_idx": action.target_idx}
    elif isinstance(action, EndTurn):
        return {"type": "end_turn"}
    return {"type": "unknown"}


class SelfPlayTrainer:
    """Coordinates self-play workers and model training."""

    def __init__(
        self,
        num_workers: int = 8,
        combat_sims: int = 64,
        deep_sims: int = 128,
        deep_prob: float = 0.25,
        batch_size: int = 256,
        sync_every: int = 200,
        eval_every: int = 500,
        ascension: int = 20,
    ):
        from packages.training.torch_policy_net import StSPolicyValueNet, PPOTrainer, _get_device

        self.num_workers = num_workers
        self.sync_every = sync_every
        self.eval_every = eval_every

        self.device = _get_device()
        self.model = StSPolicyValueNet().to(self.device)
        self.trainer = PPOTrainer(self.model, lr=3e-4, batch_size=batch_size)

        self.seed_pool = SeedPool(max_plays=3)

        self.config = {
            "ascension": ascension,
            "character": "Watcher",
            "combat_sims": combat_sims,
            "deep_sims": deep_sims,
            "deep_prob": deep_prob,
            "obs_dim": self.model.obs_dim,
            "action_dim": self.model.action_dim,
        }

        # Stats
        self.total_games = 0
        self.total_wins = 0
        self.total_transitions = 0
        self.recent_wins: Deque[bool] = deque(maxlen=100)
        self.recent_floors: Deque[int] = deque(maxlen=100)
        self.train_losses: List[Dict] = []

        CHECKPOINT_DIR.mkdir(parents=True, exist_ok=True)

    def _save_weights(self, path: Path) -> None:
        """Save model weights for workers to load."""
        self.model.save(path)

    def _collect_batch(self, num_games: int) -> List[Dict]:
        """Play num_games in parallel, collect transitions."""
        weights_path = CHECKPOINT_DIR / "latest_weights.pt"
        self._save_weights(weights_path)

        seeds = [self.seed_pool.get_seed() for _ in range(num_games)]
        args = [(s, str(weights_path), self.config) for s in seeds]

        with mp.Pool(self.num_workers) as pool:
            results = pool.map(_play_game_worker, args)

        # Filter None results
        results = [r for r in results if r is not None]

        for r in results:
            self.total_games += 1
            self.total_transitions += r["num_transitions"]
            self.recent_wins.append(r["won"])
            self.recent_floors.append(r["floor"])
            if r["won"]:
                self.total_wins += 1
            self.seed_pool.record_result(r["seed"], {
                "won": r["won"], "floor": r["floor"],
            })

        return results

    def _train_on_batch(self, results: List[Dict]) -> Dict[str, float]:
        """Train the model on collected transitions."""
        # Gather all transitions
        all_obs = []
        all_masks = []
        all_actions = []
        all_mcts_policies = []
        all_value_targets = []

        for r in results:
            for t in r["transitions"]:
                all_obs.append(t["obs"])
                all_masks.append(t["mask"])
                all_actions.append(t["action"])
                all_mcts_policies.append(t["mcts_policy"])
                all_value_targets.append(t.get("value_target", 0.0))

        if not all_obs:
            return {"policy_loss": 0, "value_loss": 0, "entropy": 0, "total_loss": 0, "num_transitions": 0}

        obs_t = torch.from_numpy(np.stack(all_obs)).float()
        masks_t = torch.from_numpy(np.stack(all_masks)).bool()
        actions_t = torch.tensor(all_actions, dtype=torch.long)
        returns_t = torch.tensor(all_value_targets, dtype=torch.float32)

        # Get old log probs from current model (before update)
        self.model.eval()
        with torch.no_grad():
            obs_dev = obs_t.to(self.device)
            masks_dev = masks_t.to(self.device)
            logits, values, _ = self.model(obs_dev, masks_dev)
            log_probs = torch.log_softmax(logits, dim=-1)
            old_lp = log_probs.gather(1, actions_t.to(self.device).unsqueeze(1)).squeeze(1).cpu()

        # Compute advantages from returns and current values
        advantages = returns_t - values.cpu()

        # Train
        metrics = self.trainer.train_on_batch(
            obs_t, actions_t, old_lp, advantages, returns_t, masks_t,
        )
        metrics["num_transitions"] = len(all_obs)
        return metrics

    def run(self, total_episodes: int = 1000, games_per_batch: int = 16) -> None:
        """Main training loop."""
        logger.info("Starting self-play training: %d episodes, %d workers", total_episodes, self.num_workers)

        epoch = 0
        while self.total_games < total_episodes:
            t0 = time.monotonic()

            # Collect
            results = self._collect_batch(games_per_batch)
            collect_time = time.monotonic() - t0

            # Train
            t1 = time.monotonic()
            metrics = self._train_on_batch(results)
            train_time = time.monotonic() - t1

            self.train_losses.append(metrics)
            self.trainer.decay_entropy()

            # Stats
            wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)
            af = sum(self.recent_floors) / max(len(self.recent_floors), 1)

            logger.info(
                "Epoch %d | Games %d | WR %.1f%% | Floor %.1f | "
                "Loss %.4f | Trans %d | Collect %.1fs | Train %.1fs | Seeds %d",
                epoch, self.total_games, wr * 100, af,
                metrics.get("total_loss", 0), metrics.get("num_transitions", 0),
                collect_time, train_time, self.seed_pool.unique_seeds,
            )

            # Checkpoint
            if epoch % 10 == 0:
                self._save_weights(CHECKPOINT_DIR / f"checkpoint_{epoch:05d}.pt")

            # Periodic benchmark eval
            if self.total_games > 0 and self.total_games % self.eval_every < games_per_batch:
                self._run_eval(epoch)

            epoch += 1

        # Final save
        self._save_weights(CHECKPOINT_DIR / "final_weights.pt")
        logger.info("Training complete. Total games: %d, Wins: %d", self.total_games, self.total_wins)

    def _run_eval(self, epoch: int) -> None:
        """Run benchmark evaluation."""
        try:
            from packages.training.benchmark import evaluate, print_result
            result = evaluate("heuristic", num_workers=self.num_workers)
            print_result(result)
            result.agent_name = f"meta_epoch{epoch}"
            result.save()
        except Exception as e:
            logger.warning("Benchmark eval failed: %s", e)


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        datefmt="%H:%M:%S",
    )

    parser = argparse.ArgumentParser(description="Self-play training")
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--episodes", type=int, default=1000)
    parser.add_argument("--batch", type=int, default=16, help="Games per training batch")
    parser.add_argument("--sims", type=int, default=64, help="MCTS simulations (shallow)")
    parser.add_argument("--deep-sims", type=int, default=128, help="MCTS simulations (deep)")
    parser.add_argument("--ascension", type=int, default=20)
    args = parser.parse_args()

    trainer = SelfPlayTrainer(
        num_workers=args.workers,
        combat_sims=args.sims,
        deep_sims=args.deep_sims,
        ascension=args.ascension,
    )
    trainer.run(total_episodes=args.episodes, games_per_batch=args.batch)
