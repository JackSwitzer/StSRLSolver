# Ground Truth: Java Parity + Agent Contract

Last updated: 2026-02-22  
Main branch baseline commit: `0f0c8f415d51676f7d1a42021c0eacc5d61ba3ff`

## Current baseline
- Command: `uv run pytest tests/ -q`
- Result: `4663 passed, 5 skipped, 0 failed`
- Active merged parity chain: PRs [#14](https://github.com/JackSwitzer/StSRLSolver/pull/14) to [#25](https://github.com/JackSwitzer/StSRLSolver/pull/25)

## Source-of-truth references
- Java source root used in this campaign:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`
- Python engine root:
  - `packages/engine/`
- PR ledger:
  - [`PR_HISTORY.md`](./PR_HISTORY.md)
- Gap tracker:
  - [`traceability/gap-manifest.md`](./traceability/gap-manifest.md)

## Java vs Python behavior matrix

| System | Java reference | Python reference | Status | Ground-truth note |
|---|---|---|---|---|
| Run phases + transitions | `dungeons/AbstractDungeon`, room classes, reward screens | `packages/engine/game.py` | strong | Full phase loop exists with explicit action surface (`map/combat/reward/event/shop/rest/treasure/neow/complete`) |
| Combat core | `cards`, `actions`, `monsters`, `powers` packages | `packages/engine/combat_engine.py`, `packages/engine/handlers/combat.py` | partial | Stable for audited flows; residual long-tail parity remains in powers/cards/orb-linked interactions |
| Rewards | `rewards/RewardItem`, `screens/CombatRewardScreen` | `packages/engine/handlers/reward_handler.py`, `packages/engine/game.py` | strong | `RWD-001..004` complete including indexed Black Star secondary relic claims |
| Events | `events/**/*.java` | `packages/engine/handlers/event_handler.py`, `packages/engine/game.py` | strong | Definitions/handlers/choice generators at `51/51/51`; event card-selection action surface is explicit |
| Relics | `relics/*.java` | `packages/engine/content/relics.py`, `packages/engine/registry/relics.py`, `packages/engine/state/run.py` | partial | Inventory count parity reached; REL-003/004/005/006/007 closed; orb-linked behavior still open (`ORB-001`) |
| Potions | `potions/*.java` (local class snapshot missing) | `packages/engine/content/potions.py`, `packages/engine/registry/potions.py`, `packages/engine/game.py` | strong | Audited high-priority potion parity slice implemented (selection + RNG paths); Java class inventory restore still needed |
| Powers | `powers/*.java`, `powers/watcher/*.java` | `packages/engine/content/powers.py`, `packages/engine/registry/powers.py` | partial | Hook-order fixes landed; large inventory gap remains (`149 Java` vs `94 Python`) |
| Cards | `cards/**/*.java` | `packages/engine/effects/cards.py`, `packages/engine/effects/defect_cards.py` | partial | Broad coverage exists, but long-tail closure is still tracked under `CRD-*` |
| Orbs | `orbs/*.java` + orb-linked relic/power references | `packages/engine/effects/orbs.py`, `packages/engine/registry/relics.py` | open | Core parity blocker for final Defect/power/relic closure (`ORB-001`) |
| RNG/determinism | Java RNG streams (`card`, `relic`, `potion`, etc.) | `packages/engine/state/game_rng.py`, `packages/engine/game.py` | partial | Stream architecture exists. Phase-0 hardening normalized card/power/effect-context random selection paths; residual callsites remain in relic/potion/orb flows. |

## Agent interface ground truth (current)

### Public API
- `GameRunner.get_available_action_dicts() -> List[ActionDict]`
- `GameRunner.take_action_dict(action: ActionDict) -> ActionResult`
- `GameRunner.get_observation() -> ObservationDict`
- Legacy dataclass API remains supported for compatibility.

### Action dict contract
- Core fields: `id`, `type`, `label`, `params`, `phase`
- Deterministic ID rule: `id` is generated from action `type + sorted params`.
- Missing required selection params does not hard-fail:
  - returns `{success: false, requires_selection: true, candidate_actions: [...]}`.

### Explicit follow-up selection model
- Internal state: `PendingSelectionContext`
  - fields: `selection_type`, `source_action_type`, `pile`, `min_cards`, `max_cards`, `candidate_indices`/`candidate_values`, `metadata`, `parent_action_id`
- Follow-up actions:
  - `select_cards` for card-index picks (event/relic/potion/boss-relic flows)
  - `select_stance` for stance selection (`StancePotion`)

### Currently explicit decision surfaces
- Reward relic selection flows: Orrery, Bottled relics, Dolly's Mirror.
- Event card-required flows: explicit follow-up actions and selected index passthrough.
- Reward indexing: `claim_relic{relic_reward_index}` for primary + Black Star secondary relic rewards.

## Design decisions that are now locked
- Canonical domain handler ownership:
  - rewards are sourced/executed through `RewardHandler` and adapted in `GameRunner`.
- Backward compatibility preserved:
  - existing action dataclass API retained while JSON action dict surface is primary for agents.
- Explicitness over implicit UI behavior:
  - selection-required flows expose candidate actions rather than hidden internal prompts.
- Java naming mismatch tolerance:
  - alias canonicalization (events/relics) accepted as compatibility layer to keep API stable.

## Test audit (structure and quality)

### Structural inventory
- Test files: `70`
- Test function definitions: `4086` (static count by `def test_` scan)
- Baseline pass result: `4663 passed, 5 skipped, 0 failed`

### Skip inventory
- Executed skips in normal run: `5`
  - all from `tests/test_parity.py` when `consolidated_seed_run.jsonl` is missing.
- Additional latent `pytest.skip(...)` callsites exist for fallback scenarios in:
  - `tests/test_agent_api.py`
  - `tests/test_integration.py`
  - `tests/test_coverage_boost.py`

### Current quality issues
- Some test modules still include contingency skips where deterministic fixtures are preferable for strict parity CI.

## Code review and consolidation findings

High priority findings are tracked in [`CODE_CONSOLIDATION_REVIEW.md`](./CODE_CONSOLIDATION_REVIEW.md).  
Most critical blockers before final RL parity gate:
- Orb-linked placeholder behavior in relic hooks.
- RNG normalization away from direct Python `random` callsites in engine logic.
- Power inventory closure and per-class parity assertions.

## Remaining conversion backlog (authoritative)

1. `POW-001`: power inventory closure (`149 Java` vs `94 Python`, `69` normalized unmatched candidates).
2. `POW-002`: residual power hook/timing closure.
3. `ORB-001`: orb infrastructure parity and orb-linked relic/power behavior.
4. `POW-003`: integration tests for powers + orbs + relic interactions.
5. `CRD-IC-*`, `CRD-SI-*`, `CRD-DE-*`, `CRD-WA-*`: card long-tail closure.
6. `AUD-001`: final Java-vs-Python manifest diff with no unresolved parity rows.
7. `AUD-002`: normal CI path at `0 skipped, 0 failed`.
8. `AUD-003`: RL readiness sign-off.

## RL training readiness path (execution order)

1. Close `POW-*` and `ORB-*` behavior/test debt.
2. Normalize remaining `random`-based callsites to owned RNG streams and add stream-advancement tests.
3. Burn down normal-run skips by moving artifact replay tests into dedicated parity profile/job.
4. Freeze action/observation contract snapshot and version it.
5. Run final full diff audit + full suite on `main`.
6. Mark [`rl/rl-readiness.md`](./rl/rl-readiness.md) complete and publish launch baseline.

## Superseded PR handling
- [#8](https://github.com/JackSwitzer/StSRLSolver/pull/8) (`consolidation/clean-base-2026-02-03`) is explicitly treated as archival history and was closed stale on 2026-02-22.
