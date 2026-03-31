# World Model Specification

## Overview

A world model predicts the next state given (observation, action), enabling model-based planning via learned rollouts. This document evaluates feasibility and architecture options for the Slay the Spire RL project.

## Architecture Options

### Option A: MLP (Recommended for Phase 1)
- **Input**: concat(obs_t [480], action_onehot [512]) = 992 dims
- **Architecture**: Linear(992, 512) -> ReLU -> Linear(512, 512) -> ReLU -> Linear(512, 480+1+1)
- **Output**: predicted obs_{t+1} [480], reward_{t+1} [1], done_{t+1} [1]
- **Parameters**: ~760K
- **Pros**: Fast training and inference, simple to implement, enough capacity for strategic decisions
- **Cons**: No temporal context, treats each transition independently

### Option B: Transformer (Deferred)
- **Input**: sequence of (obs, action) pairs over last K steps
- **Architecture**: 4-layer transformer with 256-dim embeddings, 4 heads
- **Parameters**: ~2M
- **Pros**: Captures temporal patterns (e.g., building toward a combo, accumulating relics)
- **Cons**: Slower inference (problematic for MCTS integration), needs sequence data format, more complex training

## Data Format

Each training sample is a transition tuple:

```
(obs_t, action_t) -> (obs_{t+1}, reward_{t+1}, done_{t+1})
```

Source: existing trajectory .npz files contain sequential observations, actions, and rewards. Consecutive pairs within an episode form transition tuples naturally.

## Loss Function

```
L_world = MSE(obs_pred, obs_true) + MSE(reward_pred, reward_true) + BCE(done_pred, done_true)
```

Observation MSE dominates the loss since it's 480-dimensional. Consider:
- Weighting reward and done losses higher (10x) for planning quality
- Feature-group normalization on observation MSE (HP features vs card features vs relic features)

## MCTS Integration

The world model enables model-based rollouts in strategic MCTS:

1. At each MCTS node, use the world model to predict next state instead of requiring a real game step
2. Value estimate comes from the value head of StrategicNet on the predicted state
3. Rollout depth limited to 3-5 steps (prediction error compounds)

Key benefit: MCTS can explore hypothetical futures without running the full game engine, dramatically increasing search breadth at strategic decision points (card picks, path selection).

## Feasibility Analysis

### Data availability
- Current: ~18.5K trajectories x ~20 strategic decisions each = ~370K transition pairs
- This is sufficient for an MLP world model but borderline for a transformer
- More data accumulates with each training run

### Compute budget
- MLP training: ~5 minutes on M4 Mac Mini (370K samples, 10 epochs)
- MLP inference: <0.1ms per prediction (fast enough for MCTS)
- Transformer would be ~10x slower on both counts

### Prediction quality concerns
- Card pick decisions change the observation space discontinuously (new card added)
- Events/shops have complex, rule-based effects hard to learn from limited data
- Combat outcomes depend on the combat solver, not directly learnable from strategic observations

## Recommendation

**Defer world model implementation.** Rationale:

1. **Auxiliary heads provide similar benefits with less complexity.** The win_loss and boss_ready heads already give the value function richer gradient signal about long-term outcomes. The deck_quality head provides implicit state quality assessment that overlaps with what a world model would provide for planning.

2. **Data volume is borderline.** 370K transitions is enough to train an MLP, but not enough to get reliable multi-step rollouts. Prediction error compounds: even 5% per-step error becomes ~25% after 5 steps.

3. **MCTS already works without it.** The current MCTS uses the real game engine for rollouts at strategic decisions. A world model would only help if engine simulation is too slow -- but strategic decisions (unlike combat) are fast to simulate.

4. **Better ROI from other improvements.** The current bottleneck is the Act 1 boss wall (peak F16, 0 wins). Improving the combat solver and reward shaping will have more impact than a world model at this stage.

**Revisit when**: win rate > 10% and data volume > 1M transitions. At that point, the model has enough strategic diversity to learn meaningful state transitions, and the remaining performance gap likely requires deeper planning.
