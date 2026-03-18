# RL Training Analysis: Breaking the Floor 5.5 Plateau

**Date:** 2026-03-17
**Context:** Watcher bot, A20, PPO with 3M param residual MLP, PBRS + event rewards, TurnSolver for combat, 12 workers on M4 Mac Mini. Current avg floor 5.5, 0% win rate, loss oscillating near zero.

---

## Table of Contents

1. [Diagnosis: Why We Are Stuck](#1-diagnosis-why-we-are-stuck)
2. [PPO vs Alternatives](#2-ppo-vs-alternatives)
3. [Reward Shaping Analysis](#3-reward-shaping-analysis)
4. [Exploration Strategies](#4-exploration-strategies)
5. [Value Function Issues](#5-value-function-issues)
6. [Similar Game AI Approaches](#6-similar-game-ai-approaches)
7. [Architecture Considerations](#7-architecture-considerations)
8. [Ranked Recommendations](#8-ranked-recommendations)

---

## 1. Diagnosis: Why We Are Stuck

Before recommending changes, we need to identify the actual bottleneck. At floor 5.5 avg, the bot is dying in early Act 1 monster fights. This means one or more of:

**Hypothesis A: The combat solver is weak.** The TurnSolver handles card play, so if it plays poorly, the strategic model cannot compensate. However, TurnSolver uses lexicographic scoring with DFS/beam search (10ms for monsters, 100ms for elites, 200ms for bosses), and it is unlikely to play *so* badly that the bot dies on floor 5 monsters consistently. The solver does not learn -- it is a fixed heuristic search.

**Hypothesis B: Strategic decisions are actively harmful.** The model is choosing bad paths (walking into elites at 30 HP), bad card picks (bloating the deck), bad rest decisions (upgrading when at 10 HP), or bad event choices. At 0% win rate with avg floor 5.5, the model is making decisions roughly equivalent to random.

**Hypothesis C: The model has not learned anything meaningful yet.** With the reward structure and training dynamics, the policy gradient signal may be too noisy or contradictory to produce useful updates. The model collapses to a near-uniform or near-deterministic local optimum early and never escapes.

**Most likely root cause: A combination of B and C.** The evidence:
- Loss oscillating near zero suggests the value function and policy are in equilibrium, but at a bad equilibrium.
- 0% win rate means NO positive terminal signals (win_reward = 10.0) have ever been observed. The model is learning entirely from PBRS and intermediate event rewards.
- With ~15-25 strategic decisions per game (at floor 5.5), the model sees very few transitions per game, and all of them end in death.

### Key Diagnostic Questions

To confirm which hypothesis dominates, log and inspect:

1. **Value predictions over a game**: Plot V(s) at each decision. If V(s) is flat or near-zero throughout, the value function has not learned the shape of the game. If V(s) is highly variable but uncorrelated with actual returns, the value function is fitting noise.

2. **Policy entropy per decision type**: If entropy is very low at path selection but high at card picks (or vice versa), the model has collapsed on some decision types but not others.

3. **Advantage distribution**: If advantages are tightly clustered near zero after normalization, there is no gradient signal. If they are bimodal (large positive and negative), the signal is present but contradictory.

4. **Combat outcomes**: Track HP lost per fight for the TurnSolver alone. If the solver is hemorrhaging HP in Jaw Worm / Cultist fights, the problem is combat, not strategy.

5. **Clip fraction**: If clip fraction is consistently > 0.3, the policy is changing too fast per update. If it is near 0, the updates are doing nothing.

---

## 2. PPO vs Alternatives

### 2.1 Is PPO Right for This Domain?

PPO is a reasonable choice but has specific failure modes in this setting:

**Long episodes with sparse terminal reward.** Each game has 15-25 strategic decisions (at floor 5.5; a winning game would have ~80-120). The terminal reward (win = +10, loss = -0.5 to -1.0) is discounted over many steps. With gamma=0.99, a reward 100 steps away is attenuated by 0.99^100 = 0.366x. Combined with 0% win rate, the terminal win reward is literally never observed.

**PPO's specific weakness:** PPO is an on-policy algorithm. Every batch of data is used once (or for a few PPO epochs, currently 4), then discarded. At 100 games per collect phase, with 15-25 transitions each, we get ~1500-2500 transitions. With batch_size=512 and 4 PPO epochs, we do approximately (2000/512) * 4 = ~16 gradient updates per collect phase. This is very sample-inefficient.

**PPO's entropy collapse problem:** Research shows that PPO agents in sparse-reward environments often "discover a safe but low-reward behavior pattern early in training, leading to rapid collapse in policy entropy." Once entropy drops, the policy assigns negligible probability to alternative actions and cannot recover. Even when high-reward trajectories are occasionally discovered, on-policy updates fail to reinforce them because the policy has already moved on (Optimistic Policy Regularization, arxiv 2603.06793).

### 2.2 GRPO (Group Relative Policy Optimization)

GRPO, popularized by DeepSeek-R1, eliminates the critic network entirely. Instead of a learned value baseline, it generates a *group* of candidates for each state and normalizes advantages within the group.

**Relevance to our problem:**
- Eliminates value function inaccuracy as a failure mode (significant, given our concerns).
- Group normalization provides lower-variance gradient estimates.
- Designed for settings where a verifiable reward (correct/incorrect) exists at the episode level.

**Limitations:**
- GRPO needs multiple rollouts from the same state, which is straightforward in LLM settings but awkward for game states. We would need to save/restore game states and branch. The engine does support `copy()`, so this is feasible but adds complexity.
- In sparse-reward settings, GRPO still faces "advantage collapse" -- if all group members fail equally, normalized advantages are all near zero.
- GRPO is off-policy with respect to group-relative baselines, which introduces distributional bias.

**Verdict:** Worth investigating, but not a clear win over PPO for this domain. The bigger problem is that we never observe wins, and GRPO does not fix that.

### 2.3 TD(lambda) vs Monte Carlo Returns

Currently using GAE with lambda=0.95 and gamma=0.99. This blends TD (bootstrapping from value estimates) and Monte Carlo (using actual returns).

- **lambda=0.95** is close to Monte Carlo, meaning we rely heavily on actual trajectory returns and less on the (potentially inaccurate) value function. This is appropriate when the value function is suspect.
- For long episodes, high lambda increases variance but reduces bias. Since our episodes are short (dying at floor 5.5 = ~15-25 decisions), variance is manageable.
- **Recommendation:** Keep lambda=0.95. If the value function improves, consider reducing to 0.90 for more stable learning.

### 2.4 PPO-Specific Tuning

| Parameter | Current | Issue | Recommendation |
|-----------|---------|-------|----------------|
| clip_epsilon | 0.2 | Standard, fine | Keep |
| ppo_epochs | 4 | Low sample reuse | Increase to 8-10 (watch clip fraction) |
| batch_size | 512 | Good | Keep |
| gamma | 0.99 | Fine for this episode length | Keep |
| gae_lambda | 0.95 | Good given value function quality | Keep |
| entropy_coeff | 0.05 -> 0.01 (decayed) | May be decaying too fast | See section 4 |
| value_coeff | 0.5 | Standard | Consider reducing to 0.25 if value loss dominates |

---

## 3. Reward Shaping Analysis

### 3.1 PBRS Assessment

The current potential function:

```python
Phi(s) = 0.45 * floor_pct + 0.30 * hp_pct + 0.15 * deck_quality + 0.10 * relic_bonus
```

PBRS reward: `R_shaped = gamma * Phi(s') - Phi(s)`

**Mathematical guarantees (Ng, Harada, Russell 1999):** PBRS preserves the set of optimal policies if and only if the shaping function is of the form `F(s, s') = gamma * Phi(s') - Phi(s)` for some potential function Phi. Our implementation follows this form correctly.

**However:** The guarantee assumes the agent can reach the optimal policy. In practice, with 0% win rate, the agent never experiences the states where the optimal policy diverges from the shaped reward signal. The guarantee is about the *fixed point* of learning, not the *path* to it.

**Issues with the current potential function:**

1. **floor_pct dominance (45%):** Floor progress is the strongest signal. This is reasonable but creates a greedy incentive: the model is rewarded for advancing floors regardless of preparation. Walking into floor 6 at 20 HP gives a positive PBRS signal, even though it leads to death.

2. **hp_pct is backwards for Act 1:** Early in the game, spending HP to kill elites and gain relics is correct strategy. The HP component punishes the model for taking necessary damage, biasing it toward safe paths (all monster rooms, skip elites).

3. **deck_quality heuristic is crude:** A 12-card deck with Strikes is scored the same as a 12-card deck with Rushdown + Tantrum + MentalFortress. The heuristic penalizes deck bloat (correct) but cannot distinguish good cards from bad ones.

4. **Scale:** The potential function ranges from [0, ~0.60]. PBRS rewards are the *difference* between consecutive states, typically in [-0.05, +0.05]. This is small compared to event rewards (combat_win=0.05, elite_win=0.30, boss_win=0.80, floor milestones up to 5.0).

### 3.2 Reward Complexity: Too Dense?

The current reward has 7+ components per transition:
1. PBRS (gamma * Phi(s') - Phi(s))
2. Combat event rewards (0.05-0.80 scaled by HP efficiency)
3. Damage taken penalty (-0.005/HP)
4. Potion use/waste/hoard rewards
5. Card removal reward (+0.40)
6. Upgrade rewards
7. Floor milestone rewards (0.10 to 5.00)
8. Terminal win/loss (10.0 / -1.0*(1-progress))

**Risks of this complexity:**

- **Reward hacking:** The model could learn to maximize card removals (shop_remove=0.40) by always buying removals when possible, even when the gold would be better spent on a key card. It could learn to hoard potions to avoid the waste penalty, or avoid elites to minimize damage_per_hp.

- **Contradictory signals:** Taking an elite fight incurs damage_per_hp penalty but gives elite_win=0.30. The net signal depends on how much damage is taken, creating a noisy reward for "should I fight elites?" decisions.

- **Masking the terminal signal:** With floor milestones summing to 15+ points over a winning game, and terminal win at 10.0, the milestones are actually the dominant signal. The model may learn to reach milestones rather than to win.

- **PBRS + dense events = redundant:** PBRS already gives floor-progress reward through the floor_pct component. Floor milestones also give floor-progress reward. These double-count the same thing.

### 3.3 Should We Switch to Pure Outcome-Based Rewards?

**Arguments for:**
- Eliminates all reward hacking pathways.
- Forces the model to learn the *actual* value of each decision, not a proxy.
- Worked for AlphaZero (pure win/loss signal).

**Arguments against:**
- With 0% win rate, there is literally no positive terminal signal to learn from. Monte Carlo returns would be uniformly negative.
- The model would need to discover winning first. This is a chicken-and-egg problem.

**Verdict:** Pure outcome-based rewards are correct *eventually* but cannot work until the bot can occasionally win. The milestones are a necessary curriculum. However, the current reward is too dense and has too many components. See recommendations in Section 8.

### 3.4 Reward Hacking Risk Assessment

| Reward Component | Hack Risk | Description |
|-----------------|-----------|-------------|
| shop_remove=0.40 | MEDIUM | Model always buys removal even when wrong |
| damage_per_hp=-0.005 | HIGH | Model avoids elites/bosses to minimize damage |
| upgrade_rewards | LOW | Small values, only triggers at rest |
| potion_hoard_penalty | MEDIUM | Model might use potions wastefully to avoid penalty |
| floor_milestones | LOW | Aligned with winning, hard to hack |
| combat_win HP scaling | MEDIUM | Model prefers easy fights over necessary hard ones |

---

## 4. Exploration Strategies

### 4.1 Current Exploration: Temperature + Entropy

- **Temperature:** 0.9 (exploit), mixed with 25% explore at 1.35.
- **Entropy coefficient:** 0.05, decaying to 0.01.

**Assessment:** Temperature-based exploration is standard and fine for PPO. However:

- Entropy decay from 0.05 to 0.01 may be premature. If the model has not learned anything useful (floor 5.5), reducing entropy means it is committing to a bad policy.
- The 75/25 exploit/explore split creates a bimodal distribution but does not guarantee coverage of critical decision branches (e.g., "always take the elite path in Act 1").

### 4.2 Intrinsic Motivation (Curiosity / RND / Count-Based)

**Random Network Distillation (RND):** Uses a fixed random network and a predictor network. States the agent has seen frequently are well-predicted (low intrinsic reward); novel states get high intrinsic reward.

**Applicability:**
- RND could push the agent to explore deeper floors (novel = unseen states), which is exactly what we want.
- However, adding intrinsic motivation to an already-complex reward function risks further obscuring the true signal.
- RND adds computational cost (second network forward pass per transition) and implementation complexity.
- Recent work (Springer 2025) shows that intrinsic rewards can conflict with extrinsic rewards, with the optimal blend being task-dependent.

**Verdict:** Not recommended as a first intervention. Fix the fundamentals (value function, reward structure) before adding more reward signal.

### 4.3 Hindsight Experience Replay (HER)

HER replays failed episodes with modified goals. Originally designed for goal-conditioned robotics tasks.

**Applicability to StS:**
- Our task is not goal-conditioned in the HER sense. There is no "alternative goal" for a failed run. We cannot say "pretend you wanted to die on floor 7."
- However, a related concept is useful: **Hindsight Relabeling.** When a run dies on floor 12 (good for current performance), we could relabel it as a "success" for the floor_pred and act_completion auxiliary heads. This is essentially what the aux targets already do.

**Verdict:** Not directly applicable. The replay buffer (keeping top trajectories) serves a similar purpose.

### 4.4 Population-Based Training (PBT)

PBT runs multiple agents in parallel with different hyperparameters, periodically replacing poor agents with copies of better ones (with mutated hyperparameters).

**Applicability:**
- We have 12 workers but they all share one model. True PBT would require multiple models training simultaneously.
- On M4 Mac Mini with 24GB RAM, running even 2-3 separate models (3M params each) plus their optimizers is feasible.
- Generalized PBT (GPBT, 2024) shows 10-50% improvement over single-config training for RL.
- Multiple-Frequencies PBT (MF-PBT, 2025) addresses PBT's greediness by using sub-populations evolving at different rates.

**Verdict:** High potential but high implementation effort. Consider after fixing the core training loop.

---

## 5. Value Function Issues

### 5.1 Diagnosing Value Function Quality

The value head predicts expected return from a state. With the current reward structure, returns vary from about -1.5 (early death) to potentially 25+ (winning game with all milestones). This 15x range is a problem.

**Key diagnostic metrics to log:**

1. **Explained variance:** `1 - Var(returns - predictions) / Var(returns)`. Target: > 0.5. If <= 0, the value function is worse than predicting the mean.

2. **Value prediction range:** If the value head always predicts ~0 (range < 0.5), it has collapsed to the mean and is not providing useful baselines.

3. **Value loss trend:** Currently "oscillating near zero" -- this could mean (a) the value function is perfect (unlikely at floor 5.5) or (b) the returns are near-zero and easy to predict.

**The near-zero-loss trap:** If most games end at floor 5-6 with similar rewards (small PBRS + small event rewards + moderate death penalty), the returns across the buffer are very similar. The value function can predict "about -0.3" for everything and achieve low MSE. This is useless for policy improvement because advantages are all near zero.

### 5.2 Separate Value Network vs Shared Backbone

Currently: shared backbone (4 residual blocks) with separate policy and value heads.

**Arguments for separate networks:**
- Policy and value have different learning dynamics. The policy needs to be responsive to advantages; the value needs to be stable.
- Shared backbones create interference: value gradients can distort policy features and vice versa.
- The "37 Implementation Details of PPO" (ICLR Blog Track 2022) notes that separate value networks can improve performance in some settings.
- GRPO eliminates the critic entirely, suggesting it may be more burden than help.

**Arguments for shared backbone:**
- More parameter-efficient (important at 3M params).
- Shared features can help both tasks learn faster.
- Standard in most PPO implementations.

**Verdict:** At 3M params, separating the networks would cut effective capacity roughly in half. Keep shared backbone but consider increasing total params (see Section 7).

### 5.3 PopArt Value Normalization

PopArt (Preserving Outputs Precisely while Adaptively Rescaling Targets) normalizes value targets to zero mean and unit variance, while preserving the output by adjusting the final linear layer weights.

**Relevance:**
- Our reward scale varies from -1.5 to 25+ depending on how far the run goes. Early in training, all returns are ~-0.5 to -1.5. Later (if the bot improves), returns shift dramatically.
- PopArt would prevent the value function from being overwhelmed by the scale change.
- DeepMind showed PopArt is critical for multi-task Atari learning with variable reward scales.

**Verdict:** Worth implementing once the bot starts progressing past floor 10 and the reward distribution shifts. Low priority for the immediate plateau problem.

---

## 6. Similar Game AI Approaches

### 6.1 AlphaZero / MuZero

**AlphaZero:** Self-play + MCTS with a learned policy/value network. No reward shaping -- just win/loss.

**Key differences from our setup:**
- AlphaZero's MCTS evaluates many future states per move (800 simulations). Our TurnSolver searches ~500-10,000 nodes but only within a single turn of combat.
- AlphaZero plays against itself, generating a natural curriculum. StS is single-player against a fixed environment.
- AlphaZero uses full game simulation for MCTS rollouts. We separate combat (solver) from strategy (NN).
- AlphaZero trained on 5,000 TPUs. We have one M4 Mac Mini.

**Applicable lessons:**
- *Simple reward signal.* AlphaZero used only win/loss. Complex reward shaping was unnecessary because MCTS provided the planning backbone.
- *Value target accuracy.* AlphaZero's value targets came from actual game outcomes, not bootstrapped estimates. Consider using Monte Carlo returns (lambda=1.0) instead of GAE.
- *Sufficient compute.* AlphaZero needed massive compute. Our setup may simply need more training time.

### 6.2 MuZero and Stochastic MuZero

MuZero learns a *model* of the environment and uses it for MCTS planning. Stochastic MuZero extends this to stochastic environments (like card draws in StS).

**Relevance:**
- A learned model could enable planning at the strategic level (not just combat), allowing the agent to simulate "if I take this path, what happens?"
- However, building a learned model of StS is extremely complex. The engine already IS the model -- we have a perfect simulator.
- **The real insight from MuZero:** Use the simulator for strategic MCTS. Instead of a neural network for strategy, run Monte Carlo simulations of full games from the current state.

### 6.3 bottled_ai (52% WR at A0)

bottled_ai uses zero machine learning:
- **Combat:** Full graph traversal of all possible card play orderings, evaluating each with a heuristic score function. Picks the best outcome. This is similar to our TurnSolver.
- **Strategic decisions:** Hand-crafted priority lists for card picks, boss relics, upgrades. Condition-gated heuristics (e.g., "take Shining Light damage only if HP > X").
- **Pathing:** Graph algorithm for route optimization.

**Key takeaway:** A hand-crafted heuristic agent achieves 52% at A0. Our RL agent at A0 achieves floor 5.5. The RL agent is performing *far worse* than a hand-crafted baseline.

**Implication:** The strategic model is making decisions worse than a simple priority list. This suggests the problem is not fundamentally about RL algorithm choice but about the model learning nothing useful from the current training signal.

### 6.4 LLM-Based Approaches

Bateni & Whitehead (FDG 2024) and Banjo Obayomi's Amazon demo used LLMs to play StS by prompting with game state and available actions. These are not RL -- they use pretrained language understanding.

**Not applicable** to our setting, but interesting as a comparison: LLMs can leverage prior knowledge about card games, deck building, and strategy without training on StS specifically.

### 6.5 Published StS RL Papers

Limited published work on RL specifically for StS. The AI Playtesting project explores deep RL for PvE card games but has not published win rates. The KTH thesis (diva2:1565751) focused on ML for map path prediction, not full game play.

**The lack of published StS RL successes is itself informative.** StS is genuinely hard for RL due to: enormous state space, long episodes, imperfect information (draw pile), and the need for coherent long-term planning (deck building requires foresight over 20+ floors).

---

## 7. Architecture Considerations

### 7.1 Model Capacity (3M Params)

The StrategicNet has:
- Input: 260 dims -> 768 hidden (projection)
- 4 residual blocks at 768 dims
- Policy head: 768->256->512
- Value head: 768->64->1
- Total: ~3M params

**Assessment:**
- For the strategic task alone (~6 decision types, ~15-25 decisions per game), 3M params is adequate. This is not a vision task requiring convolutional feature extraction.
- The state encoding is 260 dims -- a relatively compact input. 3M params gives more than enough capacity to learn a function from 260 inputs to 512 outputs.
- Research on parameter scaling (Multi-Task RL, 2025) shows diminishing returns in single-task settings. More params help when the task distribution is diverse.
- **Verdict:** 3M params is fine. The bottleneck is not model capacity.

### 7.2 Transformer vs Residual MLP

Decision Transformer (Chen et al., 2021) showed that transformers can model RL as sequence prediction, conditioning on desired return.

**For our setting:**
- Transformers excel when temporal dependencies matter (a card picked on floor 3 affects combat on floor 40). However, our state encoding already summarizes the deck, so temporal dependencies are captured in the state representation.
- Transformers add significant overhead (attention is O(n^2) in sequence length).
- The stabilizing innovations for RL transformers (GTrXL, IMR) add complexity.
- **For 260-dim input to 512-dim output with no sequential structure in the input, a residual MLP is appropriate.** A transformer would be useful if we encoded the *history* of decisions as a sequence.

**Verdict:** Keep the residual MLP. A transformer would be beneficial if we moved to a trajectory-conditioned architecture (like Decision Transformer for offline RL), but that is a larger architectural change.

### 7.3 Observation Encoding

The 260-dim RunStateEncoder covers:
- HP, gold, floor, act, ascension (6 dims)
- Keys (3 dims)
- Deck functional aggregate (16 dims -- avg effects + composition)
- Relic binary flags (181 dims)
- Potion functional summary (20 dims)
- Map lookahead (21 dims -- 3 rows x 7 room types)
- Progress features (4 dims)
- HP deficit + room type flags (3 dims)
- Decision phase type (6 dims)

**Missing information that could matter:**
1. **Available actions encoding.** The model receives an action mask but no information about WHAT the actions are. For card picks, it does not know which cards are offered. For path selection, it does not know which nodes are options. The model must learn a mapping from action_index to action_meaning purely from experience.
2. **Card pick context.** When choosing between 3 cards, the model gets deck_aggregate + action_mask[0:3]. It does not see the 18-dim effect vectors of the offered cards. This is a critical missing input.
3. **Event details.** In events, the model does not know which event it is or what the options mean.
4. **Enemy preview for path decisions.** The map lookahead gives room types but not specific enemies.

**This is likely a major contributor to the plateau.** The model cannot learn card pick preferences because it never sees what the cards are. It can only learn "in this deck state, pick action 0/1/2/3" without knowing what action 0 corresponds to.

### 7.4 Action Space (512 Dims)

The action space is 512 with action masking. Most decisions have 2-5 valid actions (path: 1-4, card pick: 3-4 + skip, rest: 2-3, shop: varies, event: 2-4).

**Issues:**
- 512 is large but with masking, only 2-5 actions are ever valid. The effective action space is small.
- The policy head outputs 512 logits, but 507+ are masked to -1e8 every time. This means 99%+ of the policy head's capacity is wasted on actions that are never valid.
- **More importantly:** Action indices are not stable across decisions. "Action 0" at a path decision is a different thing than "Action 0" at a card pick. The model must learn separate policies for each phase type within a single output head.

**Recommendation:** Consider a factored action space or a smaller output head with action embeddings. However, this is lower priority than fixing the observation encoding.

---

## 8. Ranked Recommendations

### Tier 1: Highest Impact, Do Immediately

#### 1. Encode the available actions in the observation (CRITICAL)

**Problem:** The model receives an action mask but has no information about what the actions mean. For card picks, it cannot see the offered cards. For path selection, it cannot see the room types of available nodes. The model is essentially choosing blind.

**Fix:** Extend the observation to include action descriptions. For each valid action (up to the first 10), append a fixed-size descriptor:
- Card pick: 18-dim card effect vector for each offered card
- Path: room type one-hot for each available node
- Rest: action type one-hot (rest, upgrade, dig, lift)
- Shop: item type + cost for available items
- Event: event choice description

This adds 10 * 20 = 200 dims to the observation (input_dim goes from 260 to ~460). The model can then learn "card X is good for this deck" rather than "action index 2 is good when my deck looks like this."

**Expected impact:** This is almost certainly the single most impactful change. The model literally cannot learn what it is choosing between. No RL algorithm can overcome missing input features.

**Effort:** Medium (2-4 hours). Requires changes to state_encoders.py and worker.py.

#### 2. Simplify the reward to PBRS + milestones + terminal only

**Problem:** The reward function has 8+ components, several of which contradict each other, create reward hacking opportunities, or duplicate signal.

**Fix:** Strip to three components:
1. **PBRS** (potential-based, preserves optimal policy): Keep, but simplify Phi to `floor_pct` only. Remove HP/deck/relic components since they create perverse incentives.
2. **Floor milestones** (curriculum signal): Keep the existing milestones. They provide a clear curriculum of "go further."
3. **Terminal win/loss:** Keep.

Remove: damage_per_hp, combat event rewards, potion rewards, shop_remove, upgrade_rewards, card_pick_rewards. Let the model learn these preferences from whether they lead to surviving longer.

**Rationale:** PBRS theory guarantees policy invariance only for the potential-based component. All the other rewards are standard reward shaping (not potential-based) and CAN change the optimal policy. Every additional reward component is a human prior that may be wrong.

**Expected impact:** High. Removes contradictory signals and reward hacking opportunities. The model will learn slower initially but converge to a better policy.

**Effort:** Low (1 hour). Changes to reward_config.py and worker.py.

#### 3. Log and inspect diagnostic metrics

**Problem:** We are flying blind. We do not know if the value function is working, what the advantage distribution looks like, or which decision types are problematic.

**Fix:** Add logging for every train_batch call:
- Explained variance: `1 - Var(returns - values) / Var(returns)`
- Mean and std of value predictions
- Mean and std of advantages (before normalization)
- Mean and std of returns
- Entropy per decision phase type (if feasible)
- Clip fraction (already logged)
- KL divergence between old and new policy

**Expected impact:** Does not directly improve performance but is essential for diagnosing whether other changes help or hurt. Without this, we are guessing.

**Effort:** Low (30 minutes). Changes to strategic_trainer.py.

### Tier 2: High Impact, Moderate Effort

#### 4. Increase entropy floor and delay decay

**Problem:** Entropy decays from 0.05 to 0.01. At floor 5.5, the model has not learned useful behavior yet. Premature entropy decay locks in a bad policy.

**Fix:**
- Increase initial entropy_coeff to 0.10.
- Set min_coeff to 0.03 (not 0.01).
- Do not decay entropy until avg_floor > 10 (the model has demonstrated some learning).
- Alternatively, use a fixed entropy_coeff=0.05 and never decay it until win rate > 0.

**Expected impact:** Medium. Prevents premature policy collapse. The model will explore more diverse strategies.

**Effort:** Low (15 minutes). Change in strategic_trainer.py and overnight.py.

#### 5. Implement Optimistic Policy Regularization (OPR)

**Problem:** Rare high-performing trajectories are observed but then forgotten as the policy moves on. The replay buffer addresses this partially, but replaying transitions with stale log_probs creates off-policy issues.

**Fix:** Implement OPR (arxiv 2603.06793):
- Maintain a buffer of the top-K best episodes (already have this with best_trajectories).
- Add a behavioral cloning auxiliary loss: `L_BC = -E[log pi(a|s)]` over the top-K buffer.
- Weight the BC loss at 0.1-0.3 of the policy loss.
- This prevents the policy from forgetting what worked.

**Expected impact:** Medium-high. OPR specifically addresses the "early local optimum" problem in PPO. The paper shows it helps PPO escape plateaus in Atari with 5x fewer samples.

**Effort:** Medium (2-3 hours). New loss component in strategic_trainer.py, integration with replay buffer.

#### 6. Heuristic baseline comparison

**Problem:** We do not know if the model is better or worse than random. "Avg floor 5.5" could be what random achieves.

**Fix:** Run 100 games with a simple heuristic agent (first legal action / random action) and 100 games with a hand-crafted priority list (like bottled_ai style). Compare avg floor.

**Expected impact:** Diagnostic. If random achieves floor 5.0 and our model achieves 5.5, the model has barely learned anything. If random achieves 3.0, the model has learned a bit.

**Effort:** Low (1 hour).

### Tier 3: Worth Investigating, Higher Effort

#### 7. Monte Carlo Tree Search for strategic decisions

**Problem:** The neural network is trying to learn a mapping from state to action. This is hard because the state is high-dimensional and the number of training samples is low.

**Fix:** Use the engine as a forward model for MCTS at strategic decision points. For each decision (path, card pick, rest, shop, event), simulate N full games from each choice point (using the current policy for remaining decisions) and pick the action with the best average outcome.

**Expected impact:** Potentially very high. This is how AlphaZero works -- MCTS provides strong policy improvement even with a weak neural network. The NN then learns from MCTS-improved decisions.

**Challenges:**
- Full game simulation is slow (100-500ms per game). Running 10 rollouts x 5 actions = 50 simulations per decision point = 5-25 seconds per decision.
- With 12 workers, this would dramatically reduce throughput.
- Could use a budget per decision type: 10 rollouts for path/card pick (important), 2 for rest/event (less important).

**Effort:** High (1-2 days).

#### 8. Curriculum learning via reduced game complexity

**Problem:** A20 is the hardest difficulty. The bot needs to learn basic strategy before tackling A20.

**Fix:** Start training at A0 (no ascension modifiers). Progress to higher ascension only after achieving consistent floor 17+ performance.

**Expected impact:** Medium. A0 is significantly easier (no extra damage, no extra elites, no boss damage modifier, etc.). The model could learn basic strategy faster.

**Effort:** Low (config change). But note: current ASCENSION_BREAKPOINTS already define this, just starting at A0 not A20. **Verify this is actually being used.**

#### 9. Separate policy heads per decision type

**Problem:** One policy head (768->256->512) handles all decision types. Path decisions, card picks, rest decisions, and shop decisions all share the same output space.

**Fix:** Use separate smaller heads for each decision type:
- Path head: 768->128->8 (max 8 path options)
- Card pick head: 768->128->10 (3-4 cards + skip + potions)
- Rest head: 768->64->4 (rest, upgrade, dig, lift)
- Shop head: 768->128->20 (variable items)
- Event head: 768->64->8 (2-4 options)

Select head based on the phase_type encoding.

**Expected impact:** Medium. Reduces interference between decision types. Each head can specialize.

**Effort:** Medium (3-4 hours). Changes to strategic_net.py, strategic_trainer.py, worker.py.

#### 10. Decision Transformer for offline bootstrapping

**Problem:** We accumulate trajectories but only use them via on-policy PPO (with replays as a hack). The best trajectories contain valuable information that is underutilized.

**Fix:** Train a Decision Transformer on accumulated trajectories offline:
- Input: (desired_return, state_1, action_1, state_2, action_2, ...)
- Condition on high desired_return to generate good actions.
- Use DT policy as initialization for PPO fine-tuning.

Decision Transformer is "substantially better than both CQL and BC in sparse-reward and low-quality data settings" (Chen et al., 2021).

**Expected impact:** Medium-high if we accumulate enough high-quality trajectories. Currently we have 200 trajectory files up to floor 16.

**Effort:** High (2-3 days). New model architecture + offline training pipeline.

### Tier 4: Long-term / Low Priority

| Idea | Expected Impact | Effort | Notes |
|------|----------------|--------|-------|
| GRPO instead of PPO | Medium | High | Eliminates value function but has its own issues |
| RND intrinsic motivation | Low-Medium | Medium | Fix fundamentals first |
| PopArt value normalization | Low (now) | Medium | Useful once reward scale varies more |
| Transformer backbone | Low | High | MLP is fine for non-sequential 260-dim input |
| Larger model (10M+) | Low | Low (config) | Capacity is not the bottleneck |
| Population-based training | Medium | Very High | Multiple models + mutation logic |

---

## Summary: Top 3 Changes

1. **Encode available actions in the observation.** The model is choosing blind. This is the most likely single cause of the plateau. No algorithm can compensate for missing input features.

2. **Simplify rewards to PBRS(floor_only) + milestones + terminal.** Remove the 6+ auxiliary reward components that create contradictory signals and reward hacking opportunities.

3. **Log diagnostic metrics (explained variance, advantage distribution, value predictions) and run a heuristic baseline comparison.** We need to know what is actually failing before making further changes.

After implementing these three, re-evaluate. If the model starts progressing past floor 10, the next priorities become OPR (preventing policy forgetting), strategic MCTS (for stronger decisions at key branching points), and separate policy heads per decision type.

---

## Sources

- [Ng, Harada, Russell 1999 - PBRS Theory](https://people.eecs.berkeley.edu/~pabbeel/cs287-fa09/readings/NgHaradaRussell-shaping-ICML1999.pdf) - PBRS optimal policy invariance proof
- [Optimistic Policy Regularization (2026)](https://arxiv.org/abs/2603.06793) - OPR for escaping PPO plateaus
- [GRPO - Group Relative Policy Optimization](https://cameronrwolfe.substack.com/p/grpo) - Eliminating the critic network
- [Hindsight-Anchored Policy Optimization (2026)](https://arxiv.org/abs/2603.11321) - Turning failure into feedback in sparse reward settings
- [Language-Driven Play: LLMs as Game-Playing Agents in StS (FDG 2024)](https://dl.acm.org/doi/10.1145/3649921.3650013) - LLM-based StS play
- [bottled_ai - Watcher Bot (52% A0)](https://github.com/xaved88/bottled_ai/) - Heuristic baseline for comparison
- [37 Implementation Details of PPO (ICLR 2022)](https://iclr-blog-track.github.io/2022/03/25/ppo-implementation-details/) - PPO diagnostics and best practices
- [Decision Transformer (NeurIPS 2021)](https://arxiv.org/abs/2106.01345) - RL via sequence modeling
- [PopArt - Adaptive Value Normalization (DeepMind)](https://deepmind.google/discover/blog/preserving-outputs-precisely-while-adaptively-rescaling-targets/) - Variable-scale value normalization
- [Generalized Population-Based Training (2024)](https://arxiv.org/abs/2404.08233) - GPBT for RL hyperparameter optimization
- [Improving PBRS Effectiveness (2025)](https://arxiv.org/abs/2502.01307) - Recent PBRS improvements
- [Comprehensive Reward Engineering Survey (2024)](https://arxiv.org/abs/2408.10215) - Reward shaping overview and risks
- [Reward Hacking in RL (Lilian Weng, 2024)](https://lilianweng.github.io/posts/2024-11-28-reward-hacking/) - Reward hacking taxonomy
- [Dealing with Sparse Rewards in RL (2019)](https://arxiv.org/abs/1910.09281) - Sheffield survey on sparse reward techniques
- [Stochastic MuZero](https://openreview.net/forum?id=X6D9bAHhBQ1) - MuZero for stochastic environments
- [LightZero - MCTS Benchmark (NeurIPS 2023)](https://github.com/opendilab/LightZero) - Unified MCTS benchmark
- [Policy-Based RL in Imperfect Information Card Games (2025)](https://www.mdpi.com/2076-3417/15/4/2121) - PPO for card games
- [Factored Action Spaces in Deep RL](https://openreview.net/forum?id=naSAkn2Xo46) - Handling large action spaces
- [Hindsight Experience Replay (NeurIPS 2017)](https://arxiv.org/abs/1707.01495) - Learning from failure
- [Survey on Transformers in RL (2023)](https://arxiv.org/abs/2301.03044) - When transformers help in RL
- [Multi-Task RL Parameter Scaling (2025)](https://arxiv.org/abs/2503.05126) - When more parameters help
- [Intrinsic Motivation Impact Study (Springer 2025)](https://link.springer.com/article/10.1007/s00521-025-11340-0) - When curiosity helps vs hurts
