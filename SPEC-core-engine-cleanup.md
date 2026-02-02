# SPEC: Core Engine Consolidation & Parity Audit

## One-liner
Strip, consolidate, and rigorously test the STS Python game engine into a clean monorepo with verified Java parity across all characters.

## Success Criteria
- [ ] Clean `packages/engine/` with zero dead code, no duplicate implementations
- [ ] Clean `packages/parity/` with consolidated verification tooling
- [ ] All non-engine code archived via git tag (recoverable, not cluttering main)
- [ ] Exhaustive test suite: RNG, combat, cards, enemies, events, relics, shops, rewards, potions, powers, map gen
- [ ] Parity verified against 20+ seeds catalog AND mod-logged game sessions
- [ ] All 4 characters' content present and tested (Watcher primary, others complete)
- [ ] `uv run pytest` passes clean with >90% coverage on engine package
- [ ] Engine is importable as a standalone package for future RL integration

## Phased Plan

### Phase 0: Archive & Branch Setup
1. Tag current state as `archive/pre-cleanup` (preserves everything)
2. Create branches for archivable code before removal:
   - `archive/sts-oracle` — web dashboard (web/, server code)
   - `archive/vod-extraction` — VOD/training pipeline (training/, vod/, vod_data/)
   - `archive/comparison-tools` — verification/debugging scripts (comparison/)
3. Document each archive branch in a short README at its root
4. Start clean work on `main` (or `core-cleanup` branch)

### Phase 1: Restructure to Monorepo
Target structure:
```
SlayTheSpireRL/
├── packages/
│   ├── engine/          # Pure game engine
│   │   ├── __init__.py
│   │   ├── game.py      # GameRunner - orchestrates full runs
│   │   ├── combat.py    # Combat engine (consolidated from combat_engine.py)
│   │   ├── content/     # Cards, enemies, events, relics, potions, powers, stances
│   │   ├── state/       # RNG, combat state, run state
│   │   ├── generation/  # Map, encounters, rewards, shops
│   │   ├── handlers/    # Room/phase dispatch (combat, event, shop, rest, reward)
│   │   ├── effects/     # Card effect system
│   │   └── calc/        # Damage calculation
│   └── parity/          # Java parity verification
│       ├── __init__.py
│       ├── seed_catalog/ # 20+ verified seeds with expected outcomes
│       ├── mod_logger/   # Tools to capture & replay mod-logged sessions
│       └── verifier.py   # Automated parity checking
├── tests/
│   ├── engine/          # Engine unit + integration tests
│   │   ├── test_rng.py
│   │   ├── test_combat.py
│   │   ├── test_cards.py
│   │   ├── test_enemies.py
│   │   ├── test_events.py
│   │   ├── test_relics.py
│   │   ├── test_shops.py
│   │   ├── test_rewards.py
│   │   ├── test_potions.py
│   │   ├── test_powers.py
│   │   ├── test_map_gen.py
│   │   ├── test_handlers.py
│   │   └── test_full_run.py  # Integration: seed → full run
│   └── parity/          # Parity regression tests
│       ├── test_seed_catalog.py
│       └── test_mod_replay.py
├── mod/                 # Java mod (EVTracker) - stays as-is
├── decompiled/          # Java source reference - stays
├── docs/vault/          # Game mechanics ground truth - stays
├── scripts/             # Dev scripts (trimmed)
├── pyproject.toml
├── CLAUDE.md
└── README.md
```

### Phase 2: Code Cleanup & Consolidation
**Delete (after archiving):**
- `core/combat.py` (754 lines, deprecated — replaced by combat_engine.py)
- `core/game.py.backup`
- `core/data/enemies.py` (698 lines, duplicates content/)
- `core/data/events.py` (853 lines, duplicates content/)
- All training/ code (archived)
- All web/ server code (archived)
- All comparison/ tools except what's consolidated into packages/parity/
- vod/, vod_data/, annotations/, agents/ directories
- Unused top-level scripts

**Consolidate:**
- 3 combat simulators → 1 in packages/engine/combat.py
- 10 verification tools → 1 consolidated verifier in packages/parity/
- 3 training extractors → archived (not needed for engine)

**Optimize (audit each module):**
- Review each content file for correctness vs Java decompiled source
- Simplify over-engineered abstractions
- Remove defensive code / dead branches from AI iteration artifacts
- Ensure consistent coding patterns across modules

### Phase 3: Exhaustive Test Suite
**RNG Tests:**
- All 13 streams independently verified
- Act transition snapping
- Per-floor reseeding
- Neow option consumption
- 20+ seed catalog with full floor-by-floor predictions

**Combat Tests:**
- Card play mechanics (all characters)
- Damage calculation with all modifier combos
- Stance transitions (Watcher)
- Power/status interactions
- Enemy AI patterns (all enemies)
- Multi-turn combat sequences

**Content Tests:**
- Every card effect (at minimum smoke-test per card)
- Every relic trigger condition
- Every event branch outcome
- Every potion effect
- Power stacking / interaction matrix

**Generation Tests:**
- Map generation determinism
- Encounter pools per act/ascension
- Card reward rarity weighting
- Shop inventory generation
- Potion drop rates

**Integration Tests:**
- Full run simulation (seed → victory/defeat)
- Specific seed replay matching known outcomes
- Handler dispatch for all room types

### Phase 4: Parity Verification
- Build seed catalog (20+ seeds, all acts, varied paths)
- For each seed: verify encounters, card rewards, shop contents, event outcomes
- Set up mod logging to capture live game data
- Automated replay: mod log → Python engine → assert match
- Document any known divergences with root cause

## Technical Decisions
- **Package structure**: Monorepo with packages/ — clean separation, independently testable
- **Character scope**: All 4 characters maintained (general-purpose engine)
- **Archive method**: Git tags for point-in-time, branches for archived modules with READMEs
- **Parity source**: Dual — seed catalog for CI + mod logging for regression
- **RL API**: Deferred — engine stays API-agnostic for now
- **Test framework**: pytest with uv
- **No new dependencies**: Engine should be pure Python (numpy ok for perf-critical paths)

## Edge Cases
- Act 4 / Heart fight mechanics (different from Acts 1-3)
- Ascension 20 modifiers on all systems (HP, damage, encounters, elites)
- Rare card interactions (Dead Branch + exhaust, Corruption + skills)
- Multi-relic proc chains (order-dependent triggering)
- Snecko Eye randomization on card costs
- Infinite loop detection in combat (Watcher infinite combos)

## Out of Scope
- RL training pipeline (future, after engine is solid)
- RL API design (exploring this afternoon separately)
- Web dashboard / STS Oracle (archived)
- VOD extraction system (archived)
- Java mod changes (EVTracker stays as-is)
- Performance benchmarking (future, after correctness)

## Open Questions
- How many seeds do we need for confident parity? Starting with 20, expand as needed.
- Should packages/engine/ be pip-installable or just importable? Start with importable, formalize later.
- Do we need a CLI for the engine (run simulations from command line)? Defer.

## Implementation Plan
1. **Phase 0**: Archive & tag (30 min) — git ops, branch creation, READMEs
2. **Phase 1**: Restructure (move files into packages/ layout, fix imports)
3. **Phase 2**: Audit & clean each module with subagents (biggest phase — module by module)
4. **Phase 3**: Build test suite (parallel with Phase 2 — test what you clean)
5. **Phase 4**: Parity verification (seed catalog + mod integration)
