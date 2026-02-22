---
name: parity-core-loop
description: Execute the Slay the Spire Java parity campaign in strict feature loops. Use when working on parity gaps, action-surface completeness, audit/doc synchronization, deterministic RNG behavior, and region-based feature delivery with docs->tests->code->commit->todo-update sequencing.
---

# Parity Core Loop Skill

Use this skill for any parity-region feature work.

## Required loop
1. Update audit docs and gap manifest rows first.
2. Add or tighten tests for the target feature ID.
3. Implement minimal parity-correct code.
4. Run targeted tests, then full suite.
5. Update trackers and baseline docs.

## Scope discipline
- Work one feature ID at a time.
- Keep commits small and reviewable.
- Do not merge without Java references and RNG notes in docs.

## References
- Core loop runbook: `references/core-loop.md`
- Doc schema and traceability fields: `references/doc-schema.md`
- Test writing template: `references/test-template.md`
- PR template: `references/pr-template.md`
