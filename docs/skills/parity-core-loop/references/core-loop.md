# Core Loop (Agent/Swarm)

## Lane roles
- Audit lane (extra-high): inventory, Java refs, gap manifest updates.
- Edit lane (medium): narrow implementation for one feature ID.
- Test lane (medium): deterministic and regression tests.
- Integrator lane (extra-high): merge gate and tracker synchronization.

## Single feature protocol
1. Select one `feature_id` from `gap-manifest.md`.
2. Update domain doc and manifest row.
3. Update tests first.
4. Implement code change.
5. Run tests.
6. Commit.
7. Update `CORE_TODO.md` + test baseline.

## Hard rules
- One feature ID per commit.
- No unrelated refactors.
- No hidden action choices for model-facing flows.
