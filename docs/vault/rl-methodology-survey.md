# Slay the Spire RL/AI Methodology Survey

## Key Challenges

### 1. State Space Complexity
- 200+ cards/character, 50+ relics, 50+ enemies
- Deck size 10-40+, procedural maps
- Partial observability (intents, deck order)

### 2. Long-Horizon Credit Assignment
- Act 1 choices affect Act 3 outcomes
- 15+ encounters with sparse terminal rewards
- Per-timestep penalties -> "die quickly" strategies

### 3. Decision Type Separation
- **Combat**: Tactical, short-horizon (kill, minimize HP loss)
- **Drafting**: Strategic, long-horizon (synergies for future)
- **Pathing**: Risk/reward (elites vs rest vs shops)

### 4. Action Space Issues
- Card index != card semantics (index 4 changes meaning)
- Solutions: Autoregressive actions, direct semantics, action masking

## Existing Projects

| Project | Approach | Best Result |
|---------|----------|-------------|
| bottled_ai | Graph traversal simulation | 52% Watcher A0 |
| Slay-I | ML fight prediction (325K fights) | +/-7 HP accuracy |
| MiniStS | LLM agents + backtracking | CoT excels at planning |
| conquer-the-spire | C++ sim + DQN | Framework only |
| decapitate-the-spire | Python headless clone | WIP |
| simulatethespire | Python simulation | Infrastructure |
| StSRLSolver (yours) | BC + Self-play + MCTS | In development |

### bottled_ai Deep Dive

**Architecture**:
- Graph traversal evaluating all card play sequences
- Considers damage dealt, incoming damage, enemy kills, relic states
- Two-factor optimization: survivability + reward potential
- Damage-focused card priority (Perfected Strike prioritized)

**Key Insight**: No cheating - only uses player-visible information.

**Limitations**: Heuristic-bound, doesn't improve over time.

### Kai Brewer-Krebs PPO/A2C Case Study

**What Failed**:
1. Generic RL with large action spaces (card_index × target_index)
2. Sparse terminal-only rewards
3. Full dungeon episodes (15+ turn credit assignment)

**What Worked**:
- Autoregressive actions: "decide to play" → "which card" → "which target"
- Frequent intermediate rewards (HP changes)
- Combat-only agents (simpler problem)

**Breakthrough Insight**: Action space design critical - index-based prevents learning card semantics.

### LLM Agent Research (FDG 2024)

**Paper**: "Language-Driven Play: Large Language Models as Game-Playing Agents"

**Key Findings**:
- Without Chain-of-Thought: LLM = random agent
- With CoT: Outperforms graph traversal on multi-turn planning
- LLMs independently discover card synergies
- No training required, but slow (API calls) and expensive

## Algorithms Tried

| Algorithm | Result |
|-----------|--------|
| PPO/A2C | Limited success on toy problems |
| DQN | Breakthrough with elaborate reward shaping |
| A* Search | Strong combat (exploits RNG) |
| Backtracking | Effective for immediate synergies |
| GPT-4 CoT | Superior long-term planning |
| Random Forest | 72.86% run outcome prediction |

## What Works

### State Representation
- HP, block, energy, current hand (by ID not index)
- Enemy HP, block, intent, status effects
- Deck composition as counts, relic list
- Derived features: "single target scaling", "AoE damage", "card manipulation"

### Reward Shaping
- HP-based intermediate rewards (HP is fungible currency)
- Per-card rewards (domain knowledge injection)
- Hierarchical sub-goals (survive Act 1, beat elite, etc.)
- Potential-based shaping

### What Fails
- Win/lose only (too sparse)
- Time penalties (die quickly)
- Neutral end-turn (local optima of always ending)

## Human Benchmarks (A20)

| Character | Expert | Estimated Optimal |
|-----------|--------|-------------------|
| Ironclad | 52-75% | 75%+ |
| Silent | 40-60% | 60%+ |
| Defect | 46-65% | 65%+ |
| Watcher | 72-94% | 96%+ |

Lifecoach achieved 94% Watcher A20. Target: >96%.

## Recommendations

1. **Hierarchical RL**: Separate combat, drafting, pathing agents
2. **Curriculum**: Start Act 1 only, add complexity
3. **Attention architectures**: Handle variable-size inputs naturally
4. **Imitation learning**: Build replay collection mod
5. **Model-based RL**: Learn dynamics for planning

## Data Resources

| Resource | Size |
|----------|------|
| Official data dump | 380 GB, 75M+ runs |
| SpireLogs | 18M runs, 90 GB |
| Slay-I training | 325K fights |

## Baseline Targets

1. Combat-only (starter deck): >90%
2. Act 1 (A0): >80%
3. Full game (A0): >50%
4. A20 Heart: No ML has demonstrated consistent performance yet
