# Archived: legacy Python engine + parity tools

Date: 2026-07-06
Reason: superseded by `packages/engine-rs`. Archived per `docs/goal/UNITS.md` U01.
Policy: never delete — see repo data policy (`AGENTS.md` / `docs/goal/GOAL.md` Invariants).

## Provenance

The legacy Python combat engine (`packages/engine/`) and its Java-parity comparison
tools (`packages/parity/`) were the pre-Rust reference implementation. Both were
already removed from `main`'s git history before this unit ran:

- `8821afa3` (`chore(archive): move legacy Python engine + parity tools + 91 tests
  to archive/`, 2026-04-18) first moved them to `archive/packages-engine/`,
  `archive/packages-parity/`, and `archive/tests-legacy/`.
- `116010cf` (`chore: delete archive/ -- legacy Python engine fully removed`) then
  deleted that archive outright, and PR #136 (merge commit `2d5df652`) landed that
  deletion on `main`. That deletion predates this unit and predates the
  "archive, never delete" invariant as currently written; it is immutable merged
  history and is not reverted here, but it means there is no tracked source left
  to `git mv` today.

As of this unit (U01, 2026-07-06):
- `packages/engine` and `packages/parity` are **absent from `main`'s git tree**
  (verified via `git ls-tree -r HEAD --name-only`).
- No file under `packages/training/`, `tests/training/`, `scripts/`, or
  `packages/app/` imports `packages.engine` or `packages.parity` (verified by
  grep; the only plain-text substring matches were `packages/engine-rs`, a
  different, live package).

This directory exists so the historical note lives with the rest of the archive
tree and future agents don't go looking for `packages/engine` expecting it to be
live code.
