# Data Curation — Progress + Formats + Logging Spec

## Current Inventory (2026-03-31)
- **Trajectories**: 2,253 files (40,784 transitions) across `logs/pretrain_data/`, `logs/v3_collect/`, `logs/runs/`, `logs/archive/`
- **Combat positions**: 11,892 `.npz` files
- **Checkpoints**: 12 `.pt` files in `logs/strategic_checkpoints/` (819 MB)
- **Usable**: 93.5% of trajectory data matches current 480-dim architecture

Run `bash scripts/training.sh data inventory` for live inventory.
Run `bash scripts/training.sh data quality` for quality report.

## Data Formats

### Trajectory Files (`traj_F{NN}_{seed}.npz`)
Per-episode strategic decision records, saved as numpy compressed archives:
- `obs`: `[T, 480]` float32 — RunStateEncoder output
- `masks`: `[T, 512]` bool — valid action mask
- `actions`: `[T]` int32 — action index taken
- `rewards`: `[T]` float32 — immediate shaped reward
- `dones`: `[T]` bool — episode termination flag
- `values`: `[T]` float32 — value estimate at decision time
- `log_probs`: `[T]` float32 — log probability of chosen action
- `final_floors`: `[T]` float32 — normalized final floor (floor/55.0)
- `cleared_act1`: `[T]` float32 — whether act 1 was cleared
- `floor`: `[1]` int — max floor reached

Two filename formats exist:
- `traj_F{NN}_{seed}.npz` — standard (e.g., `traj_F14_Train_210.npz`)
- `traj_{seq}_F{NN}.npz` — v3_collect format (e.g., `traj_000001_F06.npz`)

### Combat Positions (`combat_{NNNNNN}.npz`)
Per-combat outcome snapshots:
- `combat_obs`: `[298]` float32 — CombatStateEncoder output
- `won`: `()` bool — combat result

### Checkpoints (`.pt`)
PyTorch state dicts:
- `model_state_dict`: network weights
- `optimizer_state_dict`: optimizer state
- `epoch`, `metrics`: training metadata

## Storage Strategy
- **Active data**: `logs/runs/<run_dir>/best_trajectories/`
- **Consolidated**: `logs/pretrain_data/` (for BC pretrain)
- **Organized**: `logs/data/{raw,filtered,curated}/` (tiered)
- **Archive**: `logs/archive/YYYY-MM-DD/` for old runs
- **Never**: `rm -rf` training data. Always archive with timestamps.

## Tiered Directory Structure (`logs/data/`)
- `raw/` — symlinks to all trajectory files regardless of quality
- `filtered/` — copies of 480-dim, valid-mask, no-NaN-reward files
- `curated/` — high-quality games only (floor 20+, correct masking)

Run `bash scripts/training.sh data organize` to populate.

---

## Logging Spec

Defines the canonical fields for future data collection. Matches Java `Metrics.java`
and `MetricData.java` field names where possible for ecosystem consistency.

### Per-Game Record (`episodes.jsonl`)

Currently captured by `episode_log.py`. Fields marked with * are new additions.

**Identity & Config:**
| Field | Type | Java Equivalent | Notes |
|---|---|---|---|
| `seed` | str | `seed_played` | Game seed |
| `config_name` | str | — | Sweep config that produced this game |
| `timestamp` | str | `local_time` | ISO 8601 |
| `build_version`* | str | `build_version` | Engine git hash for reproducibility |
| `ascension_level`* | int | `ascension_level` | Always 20 for us |
| `character_chosen`* | str | `character_chosen` | Always "WATCHER" for us |

**Outcome:**
| Field | Type | Java Equivalent | Notes |
|---|---|---|---|
| `floor` | int | `floor_reached` | Final floor number |
| `won` | bool | `victory` | Whether the run was a win |
| `hp` | int | — | HP at death/win |
| `max_hp` | int | — | Max HP at death/win |
| `score`* | int | `score` | Java-compatible score calculation |
| `killed_by`* | str | `killed_by` | Monster key that killed the player |
| `death_enemy` | str | — | Same as killed_by (legacy name) |
| `death_room` | str | — | Room type where death occurred |
| `duration_s` | float | `playtime` | Wall-clock seconds |
| `decisions` | int | — | Number of strategic decision points |

**Deck & Relics (end-of-run):**
| Field | Type | Java Equivalent | Notes |
|---|---|---|---|
| `deck_final` | list[str] | `master_deck` | Card IDs in final deck |
| `relics_final` | list[str] | `relics` | Relic IDs collected |
| `potions_final`* | list[str] | — | Potion IDs held at end |
| `gold`* | int | `gold` | Gold at end of run |

**Per-Floor Tracking (arrays indexed by floor):**
| Field | Type | Java Equivalent | Notes |
|---|---|---|---|
| `current_hp_per_floor`* | list[int] | `current_hp_per_floor` | HP entering each floor |
| `max_hp_per_floor`* | list[int] | `max_hp_per_floor` | Max HP at each floor |
| `gold_per_floor`* | list[int] | `gold_per_floor` | Gold at each floor |
| `path_per_floor`* | list[str] | `path_per_floor` | Room type at each floor |
| `path_taken`* | list[str] | `path_taken` | Path choices made |

**Decision History (arrays of events):**
| Field | Type | Java Equivalent | Notes |
|---|---|---|---|
| `card_choices`* | list[dict] | `card_choices` | `{floor, picked, not_picked}` |
| `event_choices`* | list[dict] | `event_choices` | `{floor, event_name, choice}` |
| `campfire_choices`* | list[dict] | `campfire_choices` | `{floor, key, data}` |
| `boss_relics`* | list[dict] | `boss_relics` | `{floor, picked, not_picked}` |
| `damage_taken`* | list[dict] | `damage_taken` | `{floor, enemies, damage, turns}` |
| `relics_obtained`* | list[dict] | `relics_obtained` | `{key, floor}` |
| `potions_obtained`* | list[dict] | `potions_obtained` | `{key, floor}` |
| `items_purchased`* | list[str] | `items_purchased` | Shop purchase keys |
| `items_purged`* | list[str] | `items_purged` | Card removal keys |
| `neow_bonus`* | str | `neow_bonus` | Neow reward chosen |
| `neow_cost`* | str | `neow_cost` | Neow cost paid |

**Training Signals:**
| Field | Type | Notes |
|---|---|---|
| `num_transitions` | int | Strategic decision count |
| `total_reward` | float | Sum of shaped rewards |
| `pbrs_reward` | float | Potential-based reward shaping component |
| `event_reward` | float | Event-based reward component |
| `combats` | list | Per-combat summaries |
| `events` | list | Event encounters |
| `path_choices` | list | Path decisions |
| `construction_failure` | bool | Whether game construction failed |

### Per-Decision Record (Trajectory `.npz`)

The per-transition data stored in trajectory files (see format above). Future
additions should include:

| Field | Status | Notes |
|---|---|---|
| `obs` | Captured | 480-dim state encoding |
| `masks` | Captured | Valid action mask |
| `actions` | Captured | Chosen action index |
| `rewards` | Captured | Shaped reward |
| `action_probs`* | TODO | Full policy distribution (for KL tracking) |
| `alternatives`* | TODO | Top-5 alternative actions + their values |
| `decision_type`* | TODO | card_pick / path / rest / shop / event |
| `floor`* | TODO | Floor number at this decision |
| `search_stats`* | TODO | MCTS visit counts, depth, time spent |

### Per-Combat-Turn Record (future)

Not currently captured at per-turn granularity. Spec for future implementation:

| Field | Type | Notes |
|---|---|---|
| `floor` | int | Floor number |
| `turn` | int | Turn within combat |
| `cards_played` | list[str] | Card IDs played this turn |
| `energy_used` | int | Energy spent |
| `energy_available` | int | Energy at start of turn |
| `damage_dealt` | int | Total damage to enemies |
| `damage_taken` | int | Damage received |
| `hp_before` | int | Player HP before turn |
| `hp_after` | int | Player HP after turn |
| `stance` | str | Watcher stance (None/Calm/Wrath/Divinity) |
| `block_gained` | int | Block generated |
| `enemies_killed` | int | Enemies killed this turn |
| `combat_obs` | ndarray | 298-dim combat state encoding |
| `action_mask` | ndarray | Valid combat actions |
| `solver_action` | int | Action chosen by solver |
| `solver_score` | float | Solver evaluation score |

---

## Completed
- [x] Scan Desktop for scattered data (none found — only scipy test .npz)
- [x] Catalog all data: 2,253 traj files, 11,892 combat files
- [x] Dimension consistency check: 93.5% usable (480-dim)
- [x] Quality assessment: 0 NaN, 0 extreme rewards, 0 invalid actions
- [x] Extract shared data loading to `data_utils.py`
- [x] Create tiered directory structure (`logs/data/raw|filtered|curated`)
- [x] CLI: `training.sh data [inventory|quality|organize]`

## TODO: Logging Improvements
- [ ] Add starred (*) fields to `episode_log.py` (per-game Java-aligned fields)
- [ ] Add `action_probs` to trajectory .npz saves
- [ ] Add `decision_type` and `floor` to per-transition records
- [ ] Implement per-combat-turn logging
- [ ] Dashboard integration: all logs consumable by SwiftUI app

## TODO: Pretrain Dataset Curation
- [ ] Scale BC pretrain to full trajectory dataset (currently 38k valid transitions)
- [ ] Create tiered datasets: "all data", "floor 10+", "floor 16+", "boss wins"
- [ ] Combat-specific dataset: positions where solver made good/bad decisions
- [ ] Expert seed dataset: Merl A20 seed trajectories (known-good runs)
