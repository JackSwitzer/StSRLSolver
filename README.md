# StSRLSolver

> **Status: Work In Progress** 🚧

A reinforcement learning agent targeting **>96% win rate** on Slay the Spire's Watcher class at Ascension 20 (highest difficulty).

## Current Progress

### Completed
- [x] Game communication layer via `spirecomm`
- [x] Behavioral cloning model trained on 77M official game runs
- [x] 44 high-level strategic features engineered
- [x] Self-play data collection pipeline
- [x] Training infrastructure with overnight runs

### In Development
- [ ] Validate behavioral cloning baseline performance
- [ ] Self-play training loop optimization
- [ ] Feature importance analysis
- [ ] Win rate benchmarking system

## Architecture

```
StSRLSolver/
├── agent.py              # Base agent interface
├── bc_agent.py           # Behavioral cloning agent
├── watcher_agent.py      # Watcher-specific logic
├── watcher_priorities.py # Card/relic priority system
├── train_bc.py           # BC training script
├── self_play_trainer.py  # RL self-play training
├── collect_self_play_data.py
├── models/               # Saved model checkpoints
└── spirecomm/            # Game communication library
```

## Feature Engineering

44 strategic features including:
- **Combat**: Kill probability, damage predictions, block efficiency
- **Stance**: Wrath/Calm cycling optimization
- **Deck**: Card synergy scores, energy curve analysis
- **Pathing**: Elite/rest site value calculations
- **Meta**: Act-specific strategy adjustments

## Roadmap

### Phase 1: Baseline Validation
- [ ] Benchmark BC model on 1000 seeded runs
- [ ] Establish win rate baseline (target: 60%+)
- [ ] Identify failure modes and patterns

### Phase 2: Self-Play Improvement
- [ ] Implement reward shaping for intermediate goals
- [ ] Add exploration bonuses for novel strategies
- [ ] Train on diverse starting conditions

### Phase 3: Advanced Features
- [ ] Monte Carlo tree search for combat
- [ ] Card pick probability distributions
- [ ] Dynamic priority adjustments

### Phase 4: Target Achievement
- [ ] Reach 80% win rate milestone
- [ ] Fine-tune for 90%+ consistency
- [ ] Push for 96% target

## Running the Agent

### Prerequisites
- Python 3.11+
- Slay the Spire with ModTheSpire
- Communication Mod installed

### Quick Start
```bash
# Install dependencies
uv sync

# Launch game with mod
./launch_game.sh

# Run agent
./run_agent.sh
```

### Training
```bash
# Collect self-play data
./run_collector.sh

# Train behavioral cloning
python train_bc.py

# Run self-play training
./run_selfplay.sh
```

## Data

Trained on 77 million official game runs from the Slay the Spire community dataset, filtered for:
- Watcher class only
- Ascension 20 difficulty
- Winning runs (for initial BC)

## Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Win Rate (A20) | TBD | >96% |
| Avg Score | TBD | >1000 |
| BC Loss | - | <0.1 |

## Acknowledgments

- [spirecomm](https://github.com/ForgottenArbiter/spirecomm) - Game communication
- Slay the Spire community for game data

## License

MIT
