# Parity Execution Queue

Last updated: 2026-02-21

## Baseline
- Full suite: `4602 passed, 0 skipped, 0 failed`
- Branch: `consolidation/clean-base-2026-02-03`
- Policy: `1 PR region = 1 domain`, `1 feature = 1 code commit`

## Delivery Workflow (locked)
For each feature, execution order is:
1. `docs` (scope + Java refs + intended behavior)
2. `tests` (new/updated assertions for that feature)
3. `code` (implementation)
4. `commit` (feature code commit)
5. `todo update` (post-commit tracker update)

## PR Region R1: Events Parity Closure
Owner scope: `packages/engine/handlers/event_handler.py`, event tests, event docs.

Feature commits:
- `EVT-001` Register all existing `_get_*_choices` in `EVENT_CHOICE_GENERATORS`.
- `EVT-002` Add/finish handlers for `GremlinMatchGame`, `GremlinWheelGame`, `NoteForYourself`.
- `EVT-003` Dead Adventurer Java parity (`miscRng` reward/elite behavior).
- `EVT-004` Falling Java parity (preselection pools, disabling invalid options, exact-card removal).
- `EVT-005` Knowing Skull Java parity (escalating cost progression).
- `EVT-006` Pool consistency (`KnowingSkull`, `SecretPortal`) + alias normalization tests.
- `EVT-007` Deterministic multi-phase event action tests (choice availability + transitions).

## PR Region R2: Relic Agent-Selection Completeness
Owner scope: `packages/engine/game.py`, run/reward/room handlers, relic tests, relic docs.

Feature commits:
- `REL-001` Agent-facing selection actions for Astrolabe (`select_cards` on deck transforms).
- `REL-002` Agent-facing selection actions for Empty Cage (remove 2 cards).
- `REL-003` Agent-facing selection actions for Orrery (5-card reward selection path).
- `REL-004` Agent-facing selection actions for bottled relic assignment.
- `REL-005` Deterministic action IDs + validation for relic-selection contexts.
- `REL-006` Cross-handler relic ID alias normalization (`MawBank`/`Maw Bank`, etc.).
- `REL-007` Boss/chest/reward ordering regression tests for remaining edge interactions.

## PR Region R3: Powers Long-Tail Parity
Owner scope: `packages/engine/registry/powers.py`, combat trigger order, power tests, power docs.

Feature commits:
- `POW-001` Re-audit trigger ordering beyond existing `onAfter*` fixes (hook-order snapshot tests).
- `POW-002` System powers: Slow, Lock-On, Draw/No Draw, Draw Reduction, Entangled, NoBlockPower.
- `POW-003` Retention/cost powers: Establishment/Equilibrium/retain-selection paths.
- `POW-004` Enemy/boss timing powers: Time Warp, Beat of Death, Invincible, Growth/Ritual/Fading.
- `POW-005` Class-specific residuals (Corruption, Accuracy, Storm, Static Discharge, etc.).

## PR Region R4: Potion Cleanup + Determinism Hardening
Owner scope: `packages/engine/combat_engine.py`, `packages/engine/registry/potions.py`, potion tests/docs.

Feature commits:
- `POT-001` Remove remaining duplicate potion semantics (single authoritative execution path).
- `POT-002` Add explicit RNG-counter advancement tests (`card_rng`, `card_random_rng`, `potion_rng`).
- `POT-003` Expand selection roundtrip tests for all selection potions (missing-param -> candidate-actions -> apply).
- `POT-004` Fairy in a Bottle auto-trigger invariants (death hook, consumption, Sacred Bark %) with focused tests.

## PR Region R5: Audit + RL Readiness Gate
Owner scope: audit docs/scripts/tests.

Feature commits:
- `AUD-001` Re-run and snapshot Java-vs-Python parity diffs (events/potions/relics/powers).
- `AUD-002` Add parity snapshot tests and keep them green under CI.
- `AUD-003` Final RL readiness gate: full suite green + domain docs + core todo consistency.

## Known Open Issues (cross-region)
- Remaining explicit-agent selection surface for some relic acquisition flows.
- Event action-surface is still uneven for multi-step/choice-heavy rooms.
- Powers long-tail parity has many unchecked work-units outside already-fixed `onAfter*` path.
- Potion runtime/registry still has cleanup debt despite major parity fixes.
