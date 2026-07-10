---
name: data-audit
description: Scan training data inventory, check quality, recommend curation strategy
user-invocable: true
---

# Data Audit

Comprehensive inventory and quality assessment of training data.

## Process

### Phase 1: Inventory (parallel scans)

**Agent 1 — Local project data:**
- Count files by type in logs/: .json trajectories, .npz combat, .pt checkpoints
- Total size per type
- Date range (oldest → newest)
- Check logs/archive/ for archived runs

**Agent 2 — Desktop scan:**
- Search ~/Desktop/ for .npz, .pt, .json trajectory files
- Check ~/Desktop/SlaythespireRL/ (alternate dir)
- Check ~/Desktop/sts-archive/ for historical data
- Report: location, size, type, date

**Agent 3 — Quality check:**
- Sample 100 trajectory files: check dimensions, field consistency
- Sample 100 combat .npz: verify shape matches CombatStateEncoder (298-dim)
- Identify stale-dimension data (older trajectories with different encoder output)
- Check for corrupted files (partial writes, zero-length)

### Phase 2: Quality Assessment

For each data source, assess:
- **Usable for BC pretrain?** (correct dimensions, reasonable floor reached)
- **Usable for combat training?** (298-dim, valid action masks)
- **Quality tier**: raw / filtered / curated / expert
- **Estimated useful transitions**: after filtering

### Phase 3: Recommendations

Report:
```
DATA INVENTORY
--------------
Trajectories: 96,234 files (620K transitions)
  - Usable for pretrain: 89,000 (574K transitions)
  - Stale dimensions: 7,234 (filtered)
Combat: 536,000 .npz files
  - Valid 298-dim: 520,000
  - Stale: 16,000
Checkpoints: 12 .pt files
  - Best: combat_net.pt (92% val acc)

Desktop (unconsolidated):
  ~/Desktop/SlaythespireRL/logs/: 45GB

RECOMMENDATIONS
1. Consolidate Desktop data → logs/archive/
2. Create tiered datasets: floor10+, floor16+, boss-wins
3. Remove stale-dim files (save 2GB)
```
