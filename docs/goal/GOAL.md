# GOAL — Watcher Full-Run Parity

**This is the end-state spec.** Any agent (Codex, Claude, human) should be able to read this file cold, plus [INVENTORY.md](INVENTORY.md) / [TOOLING.md](TOOLING.md) / [UNITS.md](UNITS.md), and make correct progress with no other context.

## Mission

`packages/engine-rs` becomes a **trace-exact, clean, portable Rust simulator** of Slay the Spire for everything a **Watcher run** touches (Neow → Corrupt Heart), verified per-action against the real game. Training (MLX/PUCT/obs) and viz are consumers of the sim, never entangled with it. The decompiled Java at `decompiled/java-src/com/megacrit/cardcrawl/` is the reference; the running game is the oracle.

## Definition of Done

1. **Corpus exact** — every trace script in `data/traces/scripts/` replays through the Rust engine with a `match` verdict against its frozen Java golden in `data/traces/java/`: identical per-action player/enemy state, ordered piles, intents, gold, relic counters, and **all 13 RNG stream counters** (`docs/vault/rng-system-analysis.md`). Corpus = ~10 seeded Watcher A0 full runs (coverage-maximizing seeds) + the golden run `1776347657` (23 combats incl. Heart).
2. **Coverage proven** — the generated ledger (`docs/goal/ledger.json`) shows every Watcher-reachable card, relic, potion, enemy, event, and boss either `green` (exercised by the corpus and exact) or `quarantined` (see Edge-Case Policy). No `red`, no `unknown`.
3. **Existing tests green** — `./scripts/test_engine_rs.sh test --lib` (2219+ tests) and `uv run pytest tests/training -q` pass throughout; count only goes up.
4. **Sim-first boundaries hold** — core sim modules have no dependency on obs/search/training-contract/PyO3; python bindings behind a cargo feature; `scripts/goal.sh check-arch` passes (see TOOLING.md T6).
5. **Quarantine triaged** — every quarantined item has an entry in `docs/work_units/parity-deviations-register.md` and a mask in `docs/goal/masks.json`; the quarantine list is short enough to review in one sitting (guideline: <15 items).
6. **Divergences inspectable** — the viz ParityView renders any divergence report side-by-side (Java vs Rust at the diverging action) using the SVG sprites.

## Ground Truth & Oracle

- **Reference**: `decompiled/java-src/` (CFR 0.152 from `desktop-1.0.jar`, manifest at `decompiled/manifest.json`). Every Rust behavior change cites the Java file it ports, using the existing header-comment convention.
- **Oracle**: frozen golden traces minted once from the real, modded game (TraceLab mod, TOOLING.md T2) and committed. All routine verification is **offline** — `./scripts/trace_diff.sh <script>` needs no game launch. New goldens are minted only by a human-attended session (game launches are not agent-loop work).
- **Precedence**: running game > decompiled source > vault notes > existing Rust comments. If they disagree, trust the trace.

## Architecture Target

- `packages/engine-rs` core = pure sim: state, actions, RNG, content, trace emission. Deterministic given `(seed, ascension, action sequence)`.
- Layers on top (may depend on core; core never depends on them): `obs.rs`, `search.rs`, `training_contract.rs`, PyO3 bindings (feature `python`), trace differ bin, viz artifacts.
- Content stays **character-modular**: Watcher is the proven path; Ironclad/Silent/Defect content compiles and keeps its tests but is out of parity scope. Adding a character later = new trace scripts + ledger rows, not a rewrite.
- Portability: no Java/game-install/absolute paths in the crate; game paths live only in `scripts/`.

## Edge-Case Policy (the 1% rule)

Nasty items (weird RNG ordering, Wish/Nightmare-class cards, timing quirks) must not stall the loop:

1. **Effort cap**: 2 serious attempts per item (an attempt = focused session with a hypothesis, ending in a divergence report you can explain or can't).
2. Cap exceeded → **quarantine**: set ledger status `quarantined`, add a `DEV-NNN` entry to `docs/work_units/parity-deviations-register.md` (what diverges, smallest repro script, suspected cause), add a matching mask in `docs/goal/masks.json` scoped as narrowly as possible (exact path + item, never blanket).
3. Move on. Quarantined items never block green ones. Humans triage the quarantine list periodically; fixing one = removing its mask and turning the row green.
4. Never mask to make a diff pass without a register entry. A mask without a `DEV-` reference is a spec violation.

## Invariants (hard rules for every agent)

- **Protected paths** — never modify: `data/traces/java/` (goldens), `decompiled/`, `packages/training/` (live training stack), `logs/`, `runs/`. Masks/register are append-or-edit-with-justification only.
- **Archive, never delete** — superseded code/data moves to `archive/` with a dated note; `rm -rf` is forbidden.
- **Tests only go green** — no skipping/deleting failing tests to pass an oracle; a legitimately wrong old test is fixed with a Java citation.
- **Bash-first infra** — new tooling extends `scripts/` (bash + jq + cargo bins); no new Python infra files. JS via `bun`, Python via `uv`.
- **Scoped diffs** — a unit touches only files in its stated scope plus its tests. `packages/training/` diffs in a parity unit = automatic bounce.
- **Cite Java** — every ported behavior names its `decompiled/java-src/...` source in a comment.
- **Branches** — work lands as `codex/uNN-<slug>` (or `claude/`) branches, stacked in dependency order, PR per unit; never commit to `main`.

## Non-Goals (do not spend effort here)

Other characters' parity; A20 corpus (stretch, U14); strategic/pathing training changes; performance optimization beyond keeping the suite fast; mod support; a playable human game client; refactors not required by a unit's oracle.

## How to Work the Goal

```
scripts/goal.sh status      # where things stand; next actionable unit
# pick next ready unit from docs/goal/UNITS.md (or red ledger row within an open unit)
# implement smallest complete increment, citing Java
./scripts/test_engine_rs.sh test --lib          # stays green
./scripts/trace_diff.sh data/traces/scripts/<relevant>.json   # oracle
# update UNITS.md status / ledger, commit "uNN: <what>", stack a PR
# stuck past effort cap -> quarantine protocol above
# repeat until scripts/goal.sh status reports Definition of Done
```
