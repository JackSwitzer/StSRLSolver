# Parity Deviations Register

Last updated: 2026-04-18 (initial scaffold).
Branch: `claude/sharp-solomon-a1c9ec`

This is the standing register for every known Java↔Rust gameplay deviation. The companion of [parity-status.md](./parity-status.md) (which tracks the headline state) and [comprehensive-audit-2026-04-17.md](./comprehensive-audit-2026-04-17.md) (which lists the open work).

The audit phrase to remember: **"some deviations are OK, many are not."** This doc is what enforces that distinction explicitly rather than relying on tribal knowledge.

## Categories

Every entry is exactly one of:

- **`intentional`** — RL-driven design choice, documented and accepted. Will not be reverted unless we explicitly decide to. Cite [DESIGN_DECISIONS.md](../../packages/engine-rs/DESIGN_DECISIONS.md).
- **`bug`** — engine differs from Java in a way that is wrong and should be fixed. Has a target fix (audit doc reference + estimated scope).
- **`deferred`** — known divergence we have explicitly chosen not to address yet, with a tracked condition for when to revisit (e.g. "after MLX policy is trained").
- **`unverified`** — suspected divergence; no proof yet. Rotates to one of the three above after investigation.

## Workflow

1. **Add a row** whenever you discover a divergence, even before you understand it. Better to register it as `unverified` than to forget.
2. **Move rows up the kanban** as you investigate: `unverified → bug` (with fix in flight) or `unverified → intentional` (with DESIGN_DECISIONS update) or `unverified → deferred` (with revisit condition).
3. **Close rows** by linking the commit that fixes them; the row stays in the doc with a `closed:` tag so we can see the history. Do not delete closed rows for at least one release cycle — the historical record is the audit trail.
4. **Per-PR check**: any new gameplay-affecting code must either match Java exactly or land here as `intentional` with rationale. Reviewers should reject PRs that introduce silent divergence.

## Register

| ID | Title | Category | Rust ref | Java ref | Notes / fix scope | Status |
|---:|-------|----------|----------|----------|-------------------|--------|
| D1 | Enemy AI consumes no RNG | bug | `enemies/mod.rs:822` | `AbstractMonster.java:466` | Threaded `&mut StsRandom` through `roll_next_move` in commit `f0a0bfd4`; JawWorm body matches Java exactly. Phase 2 (Chosen, Centurion, Mystic, Snecko, Bear, Champ, Darkling, ~20 others) still uses old deterministic bodies but consumes from stream. | **partial-fix** (commits `f0a0bfd4` + `d91c7039`); Phase 2 pending |
| D2 | `expected_hp_loss` reported as actual HP | bug | `run_replay.py:283` | n/a | Solver reports a probabilistic search statistic as if it were an integer played-out outcome. Audit §1.2 scopes per-turn replanning loop to fix. | **open** |
| D3 | `solver.run_combat_puct` stops at TimeCap not Converged | bug | `search.rs:351` (hard_visit_cap, time_cap) | n/a | Audit §1.3: iterative-deepening to target so search always runs until it matches/beats human or hits a hard wall. | **open** |
| D4 | Transient does not auto-fade | not-a-bug | `enemies/act3.rs::roll_transient` + `combat_hooks.rs:53-62` | `Transient.java::getMove` | Investigated 2026-04-18: Fading IS decremented every enemy turn-start at `combat_hooks.rs:53-62` and Transient self-destructs at turn 5 (4 attacks dealing 40+50+60+70 = 220 dmg total). The `failed` status on F38 of the golden run is a downstream artifact of D10 (Pandora's Box deck transforms not simulated) — solver fights with near-starter deck and can't block 220 dmg over 4 turns, while the recorded human used a developed deck (Wreath of Flame burst, established blockers). Closing as not-a-Transient-bug; the same combat will resolve cleanly once D10 lands or once the solver has a trained policy. | **closed (not-a-bug)** |
| D5 | Encounter catalog (Python) is incomplete | bug | `packages/training/encounters.py` | n/a | Was Act-1-only (15 entries), drove 18/23 unsupported on the golden run. Extended to Acts 2-4 in commit `88234d5a` (now 32 entries; covers all 23 combats in the WATCHER A0 winning seed). | **closed by `88234d5a`** |
| D6 | Card hook surface: 71/352 cards on `complex_hook` legacy | bug | `complex_hook` files in `cards/` | various | Pre-existing audit baseline finding; not tonight's work. Lesson Learned is the only one that materially hits Watcher decks (audit §1.6). | **deferred** (revisit when training data shows we draw any of the 5) |
| D7 | Neow always offers 4 choices | intentional | `engine.rs::neow_options` | `NeowEvent.java` | Java gates options by deck/relic state; we expose all 4 for the RL surface. | **closed (intentional)** ([DESIGN_DECISIONS.md:8-28](../../packages/engine-rs/DESIGN_DECISIONS.md)) |
| D8 | NoteForYourself uses runtime-internal storage | intentional | `events/exordium.rs::note_for_yourself` | `NoteForYourselfEvent.java` | Java persists to the player profile JSON; we keep it in the run state since the RL pipeline doesn't have a player profile. | **closed (intentional)** ([DESIGN_DECISIONS.md:30-57](../../packages/engine-rs/DESIGN_DECISIONS.md)) |
| D9 | Match and Keep! is an indexed minigame | intentional | `events/exordium.rs::match_and_keep` | `MatchAndKeepEvent.java` | Java has a 12-card grid with click-to-flip UI; we expose it as 6 indexed pairs. | **closed (intentional)** ([DESIGN_DECISIONS.md:59-79](../../packages/engine-rs/DESIGN_DECISIONS.md)) |
| D10 | Pandora's Box random transforms not simulated in `run_parser` | bug | `packages/training/run_parser.py::reconstruct_combat_cases` | `PandorasBox.java::onEquip` | Boss relic transforms all Strikes/Defends into random commons; we cannot predict the random commons from the seed alone in Python. Causes deck mismatch warning on golden run after F16. Workaround: ignore the diff and let solver play with approximate deck. | **deferred** (revisit when we add seed-aware random reconstruction) |
| D11 | Adaptation upgrade source unaccounted for in golden run | unverified | `packages/training/run_parser.py` | various | Adaptation appears as `Adaptation+1` in master_deck but no `SMITH:Adaptation` campfire choice exists. Likely a relic-driven upgrade (Frozen Eye? Apotheosis?) we are not simulating. Single-card discrepancy. | **open** |
| D12 | Multi-enemy intent stream order vs Java | unverified | `enemies/mod.rs::roll_next_move` (called per enemy) | `AbstractRoom.endTurn` | We now correctly advance ai_rng once per `roll_next_move` (commit `f0a0bfd4`). Need to verify the ORDER in which we call roll_next_move per enemy matches Java's `monsters.monsters.forEach`. Multi-enemy tests in `test_ai_rng_parity.rs` cover stream-advancement but not absolute order. | **open** |
| D13 | Energy hardcoded to 3 in run_replay | bug | `run_replay.py:217` | n/a | `RustCombatEngine(..., 3, ...)` — should pull from .run / character defaults so Defect (3 base) and Watcher with energy relics (Cursed Key etc) get correct value. Currently OK for Watcher base but blocks future characters and Defect. | **open** (audit §1.4) |
| D14 | Neow `TEN_PERCENT_HP_LOSS` not applied to entry HP | bug | `run_parser.py:283` (TODO marker) | `NeowReward.java::TEN_PERCENT_HP_LOSS` | Floor 1 entry HP stays at full max instead of `0.9 * max_hp`. One-floor off-by-one. | **open** (audit §1.5, comment landed) |
| D15 | Neow `ONE_RANDOM_RARE_CARD`, `BOSS_RELIC` not handled | bug | `run_parser.py::_apply_neow` | `NeowReward.java` | Reconstruction skips with a warning. Affects different seeds than the WATCHER A0 winning run. | **open** (audit §1.7) |

## Adding a new row — quick template

```markdown
| Dn | Title | unverified | `path/file.rs:line` | `decompiled/.../File.java:line` | Symptom + speculative fix + scope | open |
```

After investigation, change `unverified` to `bug`/`intentional`/`deferred` and add a `Status` link to either the fixing commit or a DESIGN_DECISIONS update.

## Future automation (audit §6 stretch)

A `scripts/audit-deviations.sh` would:
- Diff Rust enemy IDs (extracted from `enemies/mod.rs::roll_next_move` match arms) vs Java monster classes in `decompiled/java-src/com/megacrit/cardcrawl/monsters/`. Any diff that's not in this register is a new `unverified` row.
- Diff Rust card IDs vs Java card pools.
- Diff Rust event IDs vs Java event registries.
- Output a delta to `docs/work_units/deviations-delta.md` and fail CI if anything new appeared without a register entry.

That's a follow-up work unit; for now the manual register is the source of truth and the discipline is "every PR adds rows for any new divergence it touches."
