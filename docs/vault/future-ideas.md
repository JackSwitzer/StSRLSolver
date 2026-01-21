# Future Ideas - Slay the Spire RL

## Seed Solving & Convergence

### Local Seed Finetuning
- Run many attempts on single seed to find optimal path
- Extract patterns that work across similar seeds
- Extrapolate successful strategies to base model

### Convergence Analysis
- Find win percentage groups (seeds with 60%, 80%, 95%+ win rates)
- Identify "ideas that just work" - universal patterns:
  - Always take Rushdown when offered early
  - Never skip shops before Act 2 boss
  - Rest at X HP threshold vs upgrade
- Document "every way to solve a seed" - multiple valid paths

### Seed Clustering
- Group seeds by characteristics:
  - Early relic quality
  - Card reward quality
  - Map topology (rest sites, elite spacing)
- Train specialized models per cluster

## Compute-Heavy Search Strategy

### Per-Hand Optimization
- Spend significant compute finding roughly best option each combat turn
- Track: most efficient way to kill while not dying
- Resource maintenance weighting:
  - HP preservation (highest priority)
  - Scaling (powers, relics triggered)
  - Potions (save for elites/bosses)
  - Gold (for key shop purchases)

### EV Weighting System (Learnable)
- Each resource type has learned importance weight
- Weights vary by:
  - Floor in act (early vs late)
  - Current HP percentage
  - Deck archetype (block-heavy vs damage-heavy)
  - Upcoming encounters

### Search Orchestration
- Parallel MCTS across process pool
- Root parallelization for different branches
- Leaf parallelization for rollouts
- Smart pruning: skip obviously bad moves early

## Parallel Simulation Architecture

### Watch Mode
- Run N games simultaneously (most headless)
- Optional: attach renderer to any game to watch
- Live EV tracking overlay during watched games

### Seed Testing Pipeline
1. Generate batch of seeds
2. Run initial evaluation (quick heuristic)
3. Deep search on promising seeds
4. Extract winning strategies
5. Validate on holdout seeds

## Model Architecture Ideas

### Distillation Pipeline
- Train "oracle" model with full seed knowledge
- Distill to "realistic" model with partial info
- Bridge: what can we infer from visible info?

### Hierarchical Decision Making
- High-level: path selection (strategic)
- Mid-level: room decisions (tactical)
- Low-level: card plays (mechanical)

### State Representation
- Encode deck as bag-of-cards embedding
- Encode relics as binary presence + counter states
- Encode enemies as (type, HP%, intent, powers)
- Position encoding for floor/act progress
