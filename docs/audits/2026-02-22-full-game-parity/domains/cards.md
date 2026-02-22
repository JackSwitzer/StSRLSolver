# Cards Domain Audit

## Status
- Largest remaining surface area.
- Phase-0 deterministic hardening landed in shared card effects: random selection paths in `packages/engine/effects/cards.py` now route through effect-context RNG helpers.
- Work tracked by class-specific granular checklists.

## Manifest (docs-first Phase R4)
- Non-Defect manifest: [`cards-manifest-non-defect.md`](./cards-manifest-non-defect.md)
- Snapshot from manifest:
  - Java rows in non-Defect scope: `285`
  - Python mapped rows: `284`
  - Missing Java IDs in non-Defect scope: `1` (`Discipline`)
  - Rows with unresolved effect-handler keys: `38` (primarily colorless/curse/status generated behaviors)

## Closure tracks
- [ ] Ironclad (`CRD-IC-*`)
- [ ] Silent (`CRD-SI-*`)
- [ ] Defect (`CRD-DE-*`)
- [ ] Watcher (`CRD-WA-*`)

## Rules
- One card-mechanic feature per commit when possible.
- Ensure decision cards emit explicit action choices.
- Add deterministic tests tied to Java behavior references.
