# Canonical Repository Policy

This repository is the canonical implementation base for Slay the Spire parity and RL readiness.

## Canonical path
- `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Source-of-truth hierarchy
1. Java behavior reference:
   - `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl`
2. Python runtime implementation:
   - `packages/engine/`
3. Canonical parity tracker/docs:
   - `TODO.md`
   - `docs/audits/2026-02-22-full-game-parity/CORE_TODO.md`
   - `docs/audits/2026-02-22-full-game-parity/traceability/`

## Branch and merge policy
- `main` remains the only protected integration branch.
- Feature branches use `codex/*` prefix.
- Per feature loop is mandatory:
  - docs -> tests -> code -> tracker update -> commit.
- Each parity PR must include:
  - Java class/method references,
  - RNG stream notes,
  - test delta and skip delta,
  - tracker updates in both `TODO.md` and `CORE_TODO.md`.

## Runtime ownership policy
- Combat runtime target is `CombatEngine` in `packages/engine/combat_engine.py`.
- `CombatRunner` in `packages/engine/handlers/combat.py` is compatibility-only and must not receive new feature logic.
- New combat behavior and power/relic integration changes land in `CombatEngine` paths first.

## Consolidation policy
- Legacy wrapper repositories are archival-only once approved files are migrated.
- Canonical engine and training interfaces are owned in this repo.
- Any remaining wrapper-side logic must be either:
  - migrated under `packages/training/`, or
  - explicitly archived in the consolidation manifest.

## RL contract freeze policy
- Preserve bot-facing API compatibility:
  - `GameRunner.get_available_action_dicts()`
  - `GameRunner.take_action_dict()`
  - `GameRunner.get_observation()`
- Schema versions are tracked explicitly in observations/action docs.
