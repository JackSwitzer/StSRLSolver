# Code Consolidation Review

Last updated: 2026-02-22

## Findings (ordered by severity)

1. **[P0] RNG determinism drift from direct Python `random` usage**
   - `packages/engine/registry/potions.py:347`
   - `packages/engine/registry/potions.py:369`
   - `packages/engine/registry/relics.py:149`
   - `packages/engine/registry/relics.py:197`
   - `packages/engine/registry/powers.py:121`
   - `packages/engine/effects/cards.py:1052`
   - `packages/engine/effects/defect_cards.py:460`
   - `packages/engine/effects/orbs.py:248`
   - Static scan count: `66` direct callsites under `packages/engine/`.
   - Impact: breaks strict seed determinism and Java-stream parity assumptions.

2. **[P0] Orb-linked relic behavior still contains placeholder/no-op logic**
   - `packages/engine/registry/relics.py:164`
   - `packages/engine/registry/relics.py:293`
   - `packages/engine/registry/relics.py:611`
   - `packages/engine/registry/relics.py:635`
   - `packages/engine/registry/relics.py:767`
   - Impact: Defect and orb-dependent relic/power parity cannot be considered closed.

3. **[P1] Reward generation still hardcodes enemy kill count**
   - `packages/engine/game.py:3285`
   - Current code uses `enemies_killed = 1` placeholder.
   - Impact: reward-modifier edge behavior (e.g., kill-dependent reward tuning) can diverge.

4. **[P1] Action-surface selection orchestration is concentrated in one large runner file**
   - `packages/engine/game.py` (`~3.3k` lines)
   - Selection interception logic spans multiple blocks:
     - event: `packages/engine/game.py:1555`
     - potion: `packages/engine/game.py:1588`
     - boss relic: `packages/engine/game.py:1641`
     - relic acquire: `packages/engine/game.py:1674`
   - Impact: higher regression risk when adding new selection mechanics.

5. **[P2] Test suite still has contingency skips in core API tests**
   - `tests/test_agent_api.py:112`
   - `tests/test_agent_api.py:133`
   - `tests/test_agent_api.py:184`
   - `tests/test_agent_api.py:271`
   - `tests/test_agent_api.py:1339`
   - Impact: less strict guarantees for deterministic map-room reachability scenarios.

6. **[P2] Effect registry string-escape cleanup**
   - `packages/engine/effects/registry.py:818`
   - Impact: avoidable warning risk and documentation/example hygiene debt.

## Consolidation project queue

### `CONS-001` RNG normalization pass
- Replace direct `random` usage in engine logic with owned RNG streams.
- Add stream-advancement tests for each migrated behavior.
- Acceptance: deterministic replay for audited scenarios and no gameplay `random.*` callsites.

### `CONS-002` Orb runtime parity core
- Implement orb slot/channel/evoke/passive infrastructure with correct timing.
- Remove orb placeholder branches in relic/power hooks.
- Acceptance: `ORB-001` closed in manifest, integration tests passing.

### `CONS-003` Runner selection-router extraction
- Extract selection interception and pending-selection replay from `GameRunner` into a dedicated module.
- Keep external API stable.
- Acceptance: no behavior change; reduced surface in `game.py` with equivalent tests.

### `CONS-004` Test strictness hardening
- Convert contingency skips in agent API tests to deterministic fixtures.
- Split replay-artifact tests from normal CI target.
- Acceptance: normal suite `0 skipped, 0 failed` with parity replay in dedicated profile.

### `CONS-005` Reward/stat cleanup
- Replace placeholder enemy-kill count with combat-derived value.
- Add tests for kill-dependent reward behavior.

## Recommended execution order
1. `CONS-001`
2. `CONS-002`
3. `POW-001` / `POW-002` / `POW-003`
4. `CONS-004`
5. `CONS-003` and residual cleanup/refactors
