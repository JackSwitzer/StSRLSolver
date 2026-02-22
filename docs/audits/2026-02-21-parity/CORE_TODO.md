# Core TODO: RL-Blocking Parity

Last updated: 2026-02-22

Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)

## Current baseline
- Full tests: `4608 passed, 5 skipped, 0 failed`
- Command: `uv run pytest tests/ -ra`
- Merged parity integration PR: [`#9`](https://github.com/JackSwitzer/StSRLSolver/pull/9)
- Skip reason: `tests/test_parity.py` skips when `consolidated_seed_run.jsonl` is absent.

## Full-game open issue inventory (known from docs)
- Cards
  - `granular-cards-defect.md`: `68` unchecked
  - `granular-cards-ironclad.md`: `62` unchecked
  - `granular-cards-silent.md`: `61` unchecked
  - `granular-cards-watcher.md`: `6` unchecked
- Events: `granular-events.md` has `49` unchecked
- Powers: `granular-powers.md` has `47` unchecked
- Rewards/shops flow: `granular-rewards.md` has `29` unchecked
- Relics: `granular-relics.md` has `15` unchecked
- Orbs/system support: `granular-orbs.md` has `14` unchecked
- Potions: `granular-potions.md` has `0` unchecked

## Campaign order (locked)
1. Potions
2. Relics
3. Events
4. Powers
5. Cards
6. Rewards/shops/rest/map flow
7. Final audit and RL gate

## Active region: Potions (`R1`) - COMPLETE
- [x] `POT-001` Unify runtime potion semantics where registry and combat paths diverge.
- [x] `POT-002` Add deterministic RNG-counter assertions for Discovery/Snecko/Entropic/Distilled paths.
- [x] `POT-003` Ensure all selection potions complete action roundtrip coverage (`missing params => candidate actions`).
- [x] `POT-004` Close Fairy in a Bottle invariants (trigger, consumption, Sacred Bark % and defeat prevention).

## Next active region: Relics (`R2`)
- [x] `REL-001` Astrolabe explicit selection actions.
- [ ] `REL-002` Empty Cage explicit selection actions.
- [ ] `REL-003` Orrery explicit selection actions.
- [ ] `REL-004` Bottled relic assignment actions.
- [ ] `REL-005` Deterministic action IDs/validation in relic selection contexts.
- [ ] `REL-006` Relic alias normalization.
- [ ] `REL-007` Remaining boss/chest/reward edge-order tests.

## Quality gates (must stay true)
- [ ] Feature loop strictly followed per feature: `docs -> tests -> code -> commit -> todo update`.
- [ ] Every feature commit includes Java reference notes and RNG notes in domain docs.
- [ ] Full suite remains green after each merged feature (`uv run pytest tests/ -q`).
- [ ] No parity-relevant skip reintroduction.
