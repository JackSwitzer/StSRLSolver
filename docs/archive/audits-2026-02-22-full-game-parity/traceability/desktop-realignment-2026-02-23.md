# Desktop Realignment Verification (CONS-DESKTOP-001)

Date: 2026-02-23

## Objective
Realign to a single active Desktop repository and archive all other STS repo folders with recovery snapshots.

## Canonical active repo
- `/Users/jackswitzer/Desktop/SlayTheSpireRL`

## Archive root
- `/Users/jackswitzer/Archives/sts-repo-consolidation/20260223_183127`

## Safety artifacts captured
- `SlayTheSpireRL.bundle`
- `StSRLSolver.bundle`
- dirty/untracked snapshots under:
  - `SlayTheSpireRL/`
  - `StSRLSolver/`
- inventory/topology snapshots:
  - `desktop-inventory.txt`
  - `slaythespirerl-git-topology.txt`
  - `slaythespirerl-worktrees-after-remove.txt`
  - `slaythespirerl-realign-before.txt`
  - `slaythespirerl-realign-after.txt`
  - `post-consolidation-verification.txt`
- integrity file:
  - `checksums.sha256.txt`

## Worktree collapse + realignment
- Removed linked worktrees and pruned stale entries.
- Verified active worktree list resolves to:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL  571740d [main]`
- Switched primary repo to `main` and verified parity with `origin/main`.

## Desktop folder consolidation result
Moved to archive root:
- `/Users/jackswitzer/Desktop/StSRLSolver`
- `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees`
- `/Users/jackswitzer/Desktop/SlayTheSpire_backup_20260113`
- `/Users/jackswitzer/Desktop/StSRLSolver.zip`

Remaining active STS repo on Desktop:
- `/Users/jackswitzer/Desktop/SlayTheSpireRL`

## Follow-on
- Runtime unification proceeds under `CONS-002A` and `CONS-002B`.
