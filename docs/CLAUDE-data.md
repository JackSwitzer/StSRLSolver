# Data Curation — Progress + TODO

## Current Inventory (2026-03-25)
- **Trajectories**: ~96k files (620k transitions) in `logs/`
- **Combat positions**: ~536k `.npz` files
- **Checkpoints**: `logs/active/combat_net.pt` (92% val acc), various strategic_net checkpoints
- **Scattered data**: Unknown quantity across Desktop (needs consolidation)

## Data Formats

### Trajectory Files (`.json`)
Per-episode strategic decision records:
- `state`: RunStateEncoder output (480-dim float array)
- `action`: action index taken
- `reward`: immediate reward
- `value`: value estimate at decision time
- `floor`, `hp`, `gold`, `deck_size`: scalar context

### Combat Positions (`.npz`)
Per-turn combat snapshots:
- `state`: CombatStateEncoder output (298-dim)
- `action_mask`: valid actions boolean array
- `outcome`: combat result (win/loss/hp_remaining)

### Checkpoints (`.pt`)
PyTorch state dicts:
- `model_state_dict`: network weights
- `optimizer_state_dict`: optimizer state
- `epoch`, `metrics`: training metadata

## Storage Strategy
- **Local**: `logs/` for active training data, `logs/archive/YYYY-MM-DD/` for old runs
- **GitHub Releases**: Curated checkpoints (e.g., `v3-pretrain-checkpoints` release)
- **Never**: `rm -rf` training data. Always archive with timestamps.

---

## TODO: Data Consolidation
- [ ] Scan Desktop for all `.npz`, `.pt`, `.json` trajectory files
- [ ] Catalog: location, size, type, date, estimated quality
- [ ] Move/symlink into organized structure under `logs/`
- [ ] Assess: what's usable for BC pretrain, what's noise

## TODO: Quality Assessment
- [ ] Define "good" pretrain data (min floor reached? win? specific decisions?)
- [ ] Filter trajectories by quality tier: raw → filtered → curated → expert
- [ ] Measure: how many transitions per tier, dimension consistency
- [ ] Remove/archive trajectories with stale dimensions (~1k/18k filtered currently)

## TODO: Logging Improvements
- [ ] Per-episode: full decision trace (state, action, alternatives, reward, model confidence)
- [ ] Per-combat-turn: cards played, energy used, damage dealt, HP change
- [ ] Per-decision: what was offered vs what was chosen (card picks, paths, events)
- [ ] Dashboard integration: all logs consumable by SwiftUI app

## TODO: Pretrain Dataset Curation
- [ ] Scale BC pretrain to full 96k trajectory dataset (currently 12,942 transitions)
- [ ] Create tiered datasets: "all data", "floor 10+", "floor 16+", "boss wins"
- [ ] Combat-specific dataset: positions where solver made good/bad decisions
- [ ] Expert seed dataset: Merl A20 seed trajectories (known-good runs)
