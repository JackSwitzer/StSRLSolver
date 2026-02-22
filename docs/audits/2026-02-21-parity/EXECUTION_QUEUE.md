# Parity Execution Queue

Last updated: 2026-02-22

## Baseline
- Full suite: `4606 passed, 5 skipped, 0 failed`
- Core branch: `consolidation/clean-base-2026-02-03`
- Active work branch: `codex/parity-core-loop`
- Policy: `1 PR region = 1 domain`, `1 feature = 1 code commit`

## Locked Core Loop (required per feature)
1. `docs` - update scope, Java refs, RNG stream notes, and expected API behavior.
2. `tests` - add/update tests that fail before implementation.
3. `code` - implement minimal parity-correct fix.
4. `commit` - commit feature change (single feature ID).
5. `todo update` - post-commit update of `CORE_TODO.md` + domain audit + test baseline.

## PR Region Order (user-locked)
1. Potions
2. Relics
3. Events
4. Powers
5. Cards
6. Rewards/shops/rest/map flow
7. Final audit + RL gate

## PR Region R1: Potions Determinism + API Completion (COMPLETE)
Owner scope: `packages/engine/game.py`, `packages/engine/combat_engine.py`, `packages/engine/registry/potions.py`, potion tests/docs.

Feature commits:
- `POT-001` Remove duplicate potion semantics where runtime path and registry diverge; keep one authoritative combat execution behavior. (`done` in commit `78d3f93`)
- `POT-002` Add explicit RNG-counter advancement tests for `card_rng`, `card_random_rng`, and `potion_rng` on RNG-sensitive potions. (`done` in commit `26375e7`)
- `POT-003` Expand action roundtrip tests for all selection potions (missing params -> candidate actions -> apply selection). (`done` in commit `26f34ec`)
- `POT-004` Close Fairy in a Bottle invariants (death hook, consumption, Sacred Bark % heal, combat-loss suppression). (`done` in commit `c25d2d3`)

## PR Region R2: Relic Agent-Selection Completeness
Owner scope: run/reward/room handlers, `packages/engine/state/run.py`, `packages/engine/game.py`, relic tests/docs.

Feature commits:
- `REL-001` Agent-facing selection actions for Astrolabe transforms.
- `REL-002` Agent-facing selection actions for Empty Cage removals.
- `REL-003` Agent-facing selection actions for Orrery picks.
- `REL-004` Agent-facing selection actions for bottled relic assignment.
- `REL-005` Deterministic action IDs + validation in relic-selection contexts.
- `REL-006` Relic ID alias normalization (`MawBank` / `Maw Bank`, etc.) across handlers.
- `REL-007` Boss/chest/reward ordering regression coverage for remaining edge interactions.

## PR Region R3: Event Java-Exact Closure
Owner scope: `packages/engine/handlers/event_handler.py`, event tests/docs.

Feature commits:
- `EVT-001` Register all existing `_get_*_choices` in `EVENT_CHOICE_GENERATORS` and assert full registration.
- `EVT-002` Finish/verify handlers for `GremlinMatchGame`, `GremlinWheelGame`, `NoteForYourself`.
- `EVT-003` Dead Adventurer parity (`miscRng` behavior and elite/reward flow).
- `EVT-004` Falling parity (valid card pools, disabled options, exact card removals).
- `EVT-005` Knowing Skull parity (cost escalation and choice transitions).
- `EVT-006` Event pool consistency (`KnowingSkull`, `SecretPortal`) + alias normalization tests.
- `EVT-007` Deterministic multi-phase event action tests (choice availability + transition integrity).

## PR Region R4: Powers Long-Tail Parity
Owner scope: `packages/engine/registry/powers.py`, combat hook ordering, power tests/docs.

Feature commits:
- `POW-001` Re-audit and assert hook ordering beyond existing `onAfter*` fixes.
- `POW-002` System powers: Slow, Lock-On, Draw/No Draw, Draw Reduction, Entangled, NoBlockPower.
- `POW-003` Retention/cost powers: Establishment, Equilibrium, retain-selection paths.
- `POW-004` Enemy/boss timing powers: Time Warp, Beat of Death, Invincible, Growth/Ritual/Fading.
- `POW-005` Class residuals: Corruption, Accuracy, Storm, Static Discharge, etc.

## PR Region R5: Card Interaction Completion
Owner scope: card execution + action surface + class card tests/docs.

Feature commits:
- `CRD-001` Core card action-surface guarantees (all choice/target cards emit explicit actions).
- `CRD-002` Ironclad checklist closure (`granular-cards-ironclad.md`).
- `CRD-003` Silent checklist closure (`granular-cards-silent.md`).
- `CRD-004` Defect checklist closure (`granular-cards-defect.md`).
- `CRD-005` Watcher checklist closure (`granular-cards-watcher.md`).

## PR Region R6: Rewards, Shops, Rest, Map Flow Parity
Owner scope: reward/shop/rest/map handlers and domain docs/tests.

Feature commits:
- `RSM-001` Reward generation and reward-choice parity (`granular-rewards.md`).
- `RSM-002` Rest options/blockers + cross-relic interactions parity.
- `RSM-003` Shop pricing/inventory/relic interactions parity.
- `RSM-004` Map flow and path/room transition invariants.

## PR Region R7: Final Audit + RL Readiness Gate
Owner scope: audit scripts/docs/tests.

Feature commits:
- `AUD-001` Re-run Java-vs-Python parity diffs (events/potions/relics/powers/cards/rewards).
- `AUD-002` Snapshot tests for parity deltas and domain regressions.
- `AUD-003` RL readiness sign-off (`full suite green`, docs synced, TODO closed).

## Known Open Issue Inventory (docs scan)
Unchecked work-unit items currently:
- `68` in `docs/work_units/granular-cards-defect.md`
- `62` in `docs/work_units/granular-cards-ironclad.md`
- `61` in `docs/work_units/granular-cards-silent.md`
- `49` in `docs/work_units/granular-events.md`
- `47` in `docs/work_units/granular-powers.md`
- `29` in `docs/work_units/granular-rewards.md`
- `15` in `docs/work_units/granular-relics.md`
- `14` in `docs/work_units/granular-orbs.md`
- `6` in `docs/work_units/granular-cards-watcher.md`
- `0` in `docs/work_units/granular-potions.md`
