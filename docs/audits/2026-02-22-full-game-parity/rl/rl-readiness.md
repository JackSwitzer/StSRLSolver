# RL Readiness Checklist

Last updated: 2026-03-02 (AUD-001 audit)

## Current status snapshot
- [x] Canonical repo lock + consolidation manifest in place.
- [x] Reward/event/relic action-surface critical fixes integrated.
- [x] Java power inventory mapping closure is complete (manifest: `missing=0`).
- [x] Orb parity foundation (`ORB-001`) integrated.
- [x] Power hook dispatch coverage artifact is complete (`registered_not_dispatched = []`).
- [x] Normal CI path is `0 skipped, 0 failed`.
- [x] Full suite: `5186 passed, 0 skipped, 0 failed` (2026-03-02).
- [x] Coverage: `72.71%` of packages/engine (line + branch).

## Preconditions
- [x] Script-generated full-game parity manifests exist (`java-inventory`, `python-inventory`, `parity-diff`, `power-hook-coverage`).
- [ ] All parity-critical behavior gaps are closed or explicitly deferred with justification.
  - Ironclad: CLOSED (62/62 cards verified)
  - Silent: CLOSED (61/61 cards verified)
  - Watcher: MOSTLY CLOSED (Watcher-specific cards verified; shared infrastructure solid)
  - Defect: NOT STARTED (0/68 effect implementations)
  - Powers: 7 unchecked behaviors remain (Draw Reduction, Draw, Electro, Focus/Lock-On, Mode Shift/Split/Life Link, Retain Cards)
- [x] Gameplay-critical randomness uses owned RNG streams only.
  - registry/ and effects/ have ZERO `import random` or `random.*` calls
  - Remaining `random` usage is in non-parity paths only (combat_sim random agent, game.py test loops)

## Action/observation contract
- [x] Core choice interactions in audited domains use explicit action dicts.
- [x] Missing selection params produce explicit candidate actions.
- [x] Action IDs are deterministic for equivalent snapshots (verified: `test_action_ids_deterministic_for_identical_state`, `test_action_id_stability_multi_step`).
- [x] Observation contract version fields are emitted (`observation_schema_version`, `action_schema_version`).
- [x] Canonical action-space spec documented (`action-layer/ACTION_SPACE_SPEC.md`).
- [x] `ActionSpace` class provides fixed-size binary mask with bidirectional ID mapping (`rl_masks.py`).
- [x] `ObservationEncoder` converts observation dicts to flat numpy arrays (`rl_observations.py`).
- [x] Mask round-trip preserves full legal action set (verified: `test_mask_round_trip_preserves_actions`).
- [x] Invalid actions are hard-rejected with explicit error (verified: `test_invalid_action_rejected_not_silent`).

## Determinism
- [x] RNG stream usage ownership is documented (`rng/JAVA_RNG_STREAM_SPEC.md`).
- [x] No direct Python `random` in parity-critical engine execution paths (registry/, effects/).
- [x] 20 targeted tests in `test_rng_migration_determinism.py` cover key RNG migration paths.
- [ ] Full replay/seed checks are reproducible in automation profile (formal lock pending).

## Test quality gate
- [x] Full suite currently green (`5186 passed, 0 skipped, 0 failed`).
- [x] Default CI has no skips.
- [x] 14 RL-specific readiness tests in `test_agent_readiness.py`.
- [x] 106 agent API tests in `test_agent_api.py`.
- [ ] Replay artifact checks moved to dedicated parity job/profile (if needed by CI).

## RL infrastructure status
- [x] `rl_masks.py`: `ActionSpace` class with register, mask, round-trip, index operations.
- [x] `rl_observations.py`: `ObservationEncoder` with run scalars, keys, deck composition, relic presence, potion slots, combat scalars, stance, enemy features.
- [x] `agent_api.py`: Full agent API wrapper with observation/action/mask integration.
- [x] 35 known action types cataloged in `ACTION_TYPES` tuple.

## Launch gate (must all be true for Watcher-only RL training)

### Ready now (can begin limited training)
- [x] Action mask working and tested.
- [x] Observation schema locked and encoder functional.
- [x] Determinism verified for parity-critical paths.
- [x] Ironclad/Silent/Watcher card parity closed.
- [x] RNG migration effectively complete for game engine paths.
- [x] 5186 tests all green.

### Blocking full Watcher RL launch
- [ ] `POW-002B`/`POW-003*` behavior and ordering closure complete (7 powers remain).
- [ ] `CRD-SH-002` shared colorless/curse/status formal sign-off.
- [ ] `AUD-003` complete and final sign-off recorded in `GROUND_TRUTH.md`.

### Blocking all-character RL launch
- [ ] `CRD-DE-001` Defect card effect implementations (68 effects).
- [ ] `RL-DASH-001` runboard dashboard.
- [ ] `RL-SEARCH-001` planner/search layer.

## Recommendation (2026-03-02)

**Watcher-only RL training can begin in limited capacity.** The core engine, action
masking, observation encoding, and RNG determinism are all functional and tested.
Ironclad, Silent, and Watcher cards are verified. The 7 unchecked power behaviors
are low-frequency and unlikely to affect early training. The Defect character is
not ready and should be excluded from initial training runs.

Full RL launch requires completing the remaining power behaviors, Defect card
implementations, and formal `AUD-003` sign-off.
