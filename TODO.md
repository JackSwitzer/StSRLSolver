# Full-Game Java Parity + Repo Cleanup TODO

Last updated: 2026-02-22  
Canonical repo path: `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Current baseline
- [x] Confirm branch state is clean on `codex/parity-core-loop`.
- [x] Confirm baseline test run is green (`4610 passed, 5 skipped`).
- [ ] Move standard CI to `0 skipped, 0 failed` (artifact-only replay tests excluded from normal CI).

## Locked defaults
- [x] Scope = full game now (cards, relics, potions, events, powers, rewards, shops, rest, map, orbs).
- [x] Cleanup mode = safe archive (non-destructive while campaign is active).
- [x] Execution order per feature = `docs -> tests -> code -> commit -> todo update`.
- [x] First implementation region after docs/test scaffolding = relic selection surface.

## Acceptance gates
- [ ] Every Java parity gap has a traceability row (`java -> python -> tests -> feature id`).
- [ ] Every choice interaction is explicit in action dicts (no hidden UI assumptions).
- [ ] Full suite in normal CI is `0 skipped, 0 failed`.
- [ ] RL readiness checklist is complete and signed.

## Repo cleanup (safe archive)
- [ ] Add canonical workspace note in new audit README.
- [ ] Create new dated audit suite under `docs/audits/2026-02-22-full-game-parity/`.
- [ ] Mark `docs/audits/2026-02-21-parity/` as legacy reference and point to new suite.
- [ ] Consolidate open items into one authoritative gap manifest.
- [ ] Replace stale/approximation test notes with linked gap IDs.

## Comprehensive audit docs to create/update
- [ ] `docs/audits/2026-02-22-full-game-parity/README.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/CORE_TODO.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/EXECUTION_QUEUE.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/testing/test-baseline.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/rl/rl-readiness.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/traceability/java-inventory.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/traceability/python-coverage.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/traceability/gap-manifest.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/potions.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/relics.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/events.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/powers.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/cards.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/rewards-shops-rest-map.md`
- [ ] `docs/audits/2026-02-22-full-game-parity/domains/orbs.md`

## Traceability schema (required per gap row)
- [ ] `gap_id`
- [ ] `domain`
- [ ] `java_class`
- [ ] `java_method_or_path`
- [ ] `python_file`
- [ ] `python_symbol`
- [ ] `status` (`exact|approximate|missing|action-surface-missing`)
- [ ] `rng_streams` (`card_rng|card_random_rng|relic_rng|misc_rng|potion_rng|event_rng`)
- [ ] `decision_surface` (`explicit_action|implicit_ui|n/a`)
- [ ] `existing_tests`
- [ ] `required_tests`
- [ ] `priority`
- [ ] `feature_id`
- [ ] `planned_pr_region`
- [ ] `notes`

## Test program (docs-derived)
- [ ] Add `tests/parity_campaign/test_traceability_matrix.py`
- [ ] Add `tests/parity_campaign/test_action_surface_completeness.py`
- [ ] Add `tests/parity_campaign/test_rng_stream_contracts.py`
- [ ] Add `tests/parity_campaign/test_relic_selection_flows.py`
- [ ] Add `tests/parity_campaign/test_event_selection_flows.py`
- [ ] Add `tests/parity_campaign/test_reward_flow_parity.py`
- [ ] Add `tests/parity_campaign/test_cross_system_interactions.py`
- [ ] Add `tests/parity_campaign/test_zero_skip_policy.py`

### Required scenario classes
- [ ] Selection roundtrip (`missing params -> candidate actions -> resolved selection`) for potion/relic/event.
- [ ] Escape/reward suppression invariants.
- [ ] Chest/rest/relic ordering and counter transitions.
- [ ] Event multi-phase deterministic transitions.
- [ ] Reward phase correctness (emerald key, boss relic pick/skip, proceed gating).
- [ ] Power hook order/timing edge cases.
- [ ] Cross-system interactions where relics/powers modify events/potions/rewards.

## API/action-surface work
- [ ] Extend `event_choice` to support explicit selection follow-up.
- [ ] Reuse pending-selection model across potion/relic/event flows.
- [ ] Ensure deterministic action IDs for equivalent snapshots.
- [ ] Keep backward compatibility for existing action dict types.

## Region roadmap

### R0: docs + test scaffolding
- [ ] `DOC-001` Build new dated audit suite and canonical trackers.
- [ ] `DOC-002` Populate Java/Python inventory and gap manifest.
- [ ] `TST-001` Add traceability matrix tests.
- [ ] `TST-002` Add action-surface completeness tests.

### R1: relic selection surface (first code region)
- [ ] `REL-003` Orrery explicit selection actions.
- [ ] `REL-004` Bottled relic assignment actions.
- [ ] `REL-008` Dolly's Mirror explicit selection action path.
- [ ] `REL-005` Deterministic selection IDs + validation.
- [ ] `REL-006` Relic alias normalization + missing Java IDs (including `Toolbox`) across runtime and generation paths.
- [ ] `REL-007` Remaining boss/chest/reward ordering regressions.

### R2: event selection surface
- [ ] `EVT-001` Event card-selection follow-up actions.
- [ ] `EVT-002` Wire event selection params through `GameRunner -> EventHandler`.
- [ ] `EVT-003` Deterministic multi-phase event transitions with explicit action availability.
- [ ] `EVT-004` Event alias normalization and inventory coverage.

### R3: reward flow normalization
- [ ] `RWD-001` Route reward action emission through `RewardHandler` as primary path.
- [ ] `RWD-002` Route reward action execution through `RewardHandler` as primary path.
- [ ] `RWD-003` Enforce proceed gating correctness (mandatory vs optional rewards).
- [ ] `RWD-004` Cross-relic reward modifiers parity (Question Card, Prayer Wheel, Busted Crown, Sozu, White Beast Statue).

### R4: powers + orbs closure
- [ ] `POW-001` Map remaining Java power classes to Python gaps with concrete feature IDs.
- [ ] `POW-002` Close remaining hook/timing parity mismatches.
- [ ] `ORB-001` Implement required orb infrastructure for parity-critical relic/power behavior.
- [ ] `POW-003` Add power-orb-relic integration parity tests.

### R5: card long-tail closure
- [ ] `CRD-IC-*` Ironclad checklist closure.
- [ ] `CRD-SI-*` Silent checklist closure.
- [ ] `CRD-DE-*` Defect checklist closure.
- [ ] `CRD-WA-*` Watcher checklist closure.

### R6: final audit + RL gate
- [ ] `AUD-001` Re-run and clean Java-vs-Python diff manifests.
- [ ] `AUD-002` Verify zero-skip normal CI baseline.
- [ ] `AUD-003` RL readiness sign-off.

## Current known high-impact mismatches to close
- [ ] Event action path still forces `card_idx=None` on handler execution.
- [ ] Relic runtime still auto-picks for Orrery/bottled/Dolly paths.
- [ ] `Toolbox` present in generation inventory but missing from content registry.
- [ ] Power inventory has major class-level gaps beyond already-fixed targeted items.
- [ ] Placeholder orb-linked relic logic remains in registry.

## Subagent/skills orchestration (repo tracked)
- [ ] Create skill root: `docs/skills/parity-core-loop/`.
- [ ] Create `docs/skills/parity-core-loop/SKILL.md`.
- [ ] Add references:
- [ ] `docs/skills/parity-core-loop/references/core-loop.md`
- [ ] `docs/skills/parity-core-loop/references/doc-schema.md`
- [ ] `docs/skills/parity-core-loop/references/test-template.md`
- [ ] `docs/skills/parity-core-loop/references/pr-template.md`

### Subagent contract
- [ ] Each subagent takes one `gap_id`/feature ID only.
- [ ] Each subagent updates docs first, then tests, then code.
- [ ] Each subagent returns Java refs + RNG notes + test delta + skip delta.
- [ ] Integrator runs full suite before merge and updates core trackers.

## Commit and PR policy
- [ ] One feature ID per commit.
- [ ] One domain region per PR.
- [ ] Include in every PR: Java references, RNG stream notes, tests changed, skip delta, docs delta.
- [ ] Keep change size small (`~<=10 files`, `~<=400 LOC net`) where feasible.

## Steps until RL training
- [ ] Close all parity-critical decision/action-surface gaps.
- [ ] Close all parity-critical behavior gaps with deterministic tests.
- [ ] Achieve stable zero-skip normal CI baseline.
- [ ] Freeze action/observation contracts.
- [ ] Mark RL readiness complete in `docs/audits/2026-02-22-full-game-parity/rl/rl-readiness.md`.

## Working loop (must follow)
1. Pick next `gap_id` from manifest.
2. Update domain docs and expected behavior with Java references.
3. Add/adjust tests first (red/insufficient before code).
4. Implement minimal parity-correct code change.
5. Run targeted tests, then full suite.
6. Commit with one feature ID.
7. Update TODO + baseline docs + execution queue.
