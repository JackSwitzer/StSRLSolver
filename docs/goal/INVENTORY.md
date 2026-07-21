# INVENTORY — Existing Work the Goal Builds On

Everything below already exists. Read before building anything new; most "new" needs are resurrections or thin glue. Status legend: **USE** (build on it) / **MINE** (extract knowledge) / **ARCHIVE** (legacy, clean-room unit U01) / **RESCUE** (at risk).

## Rust engine — USE (the product)

- `packages/engine-rs/` — 3,016/3,016 lib tests green, zero ignored. Watcher A0 implementation is approximately 95% complete against known work but only about 9% real-game-corpus certified.
  - `src/run.rs` — `RunEngine::new(seed, ascension)` + `step_game(&GameAction)`: canonical full-run surface (Neow/path/combat/rewards/shop/event/campfire/Act 4).
  - `src/seed.rs` — native libGDX `RandomXS128`, counted StS `Random`, Java-util LCG/shuffle, and typed persistent/floor/combat stream ownership.
  - `src/engine.rs`, `src/state.rs`, `src/combat_hooks.rs`, `src/card_effects.rs` — combat core; state structs already serde-derive.
  - Content ledger: 370 cards, 68 monsters, 186 relics, and 43 potions source-verified; card-owned secondary behavior uses typed metadata, with 12 intentionally imperative card hooks remaining.
  - Obsolete obs/search/training-contract/gameplay-session/PyO3 consumers are archived under `archive/2026-07-engine-consumers`; they are not core dependencies.
  - `src/tests/` — 214 Rust files, including canonical run, RNG, checkpoint, trace, and source-derived content coverage.
- `scripts/test_engine_rs.sh` — canonical cargo test/check/build runner for the engine crate.

## Audits & registers — MINE (they ARE the work queue)

- `docs/work_units/comprehensive-audit-2026-04-17.md` — the gap list with Java citations: §1.1 enemy AI RNG (critical), §1.4 replay energy hardcode, §1.5 Neow TEN_PERCENT_HP_LOSS, §3 fix ordering. Units U08/U09 are lifted from it.
- `docs/research/engine-rs-audits/` — `COMPLEX_HOOK_AUDIT.md` (the 71 fallback cards; 5 truly complex: Wish, Nightmare, Omniscience, Lesson Learned, Time Warp), `DECOMPILE_PARITY_ENDGAME.md`, `AUDIT_PARITY_STATUS.md`, `INCONSISTENCY_REPORT.md`.
- `docs/work_units/parity-deviations-register.md` — existing intentional deviations (e.g. Neow exposes 4 choices). Becomes the `DEV-NNN` source of truth for masks.
- `docs/canonical-status-keys.md`, `docs/DESIGN_DECISIONS.md` — ID ground truth + prior rulings.

## Ground truth — USE

- `decompiled/java-src/com/megacrit/cardcrawl/` — 2008 CFR-0.152 .java files; `decompiled/manifest.json` (jar SHA256); tool at `decompiled/.tools/cfr-0.152.jar`.
- Game install `$GAME = ~/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources`:
  - `ModTheSpire.jar` v3.30.3 (built from source, CLI flags work: `--mods --skip-launcher --skip-intro --close-when-finished`), bundled JRE 1.8.0_252 at `$GAME/jre/bin/java`.
  - `$GAME/mods/`: `BaseMod.jar` 5.56.0, `StSLib.jar`, **`EVTracker.jar`** (ours — per-turn JSON state dumps incl. RNG counters), **`PracticeLab.jar`** (Codex-authored — seeded Watcher launcher, rewind).
  - `$GAME/runs/WATCHER/` — **311 real .run files** incl. golden run **`1776347657.run`** (23 combats, Heart kill). Seed pool + script source for the corpus.
  - CommunicationMod jar in Steam workshop dir `646570/2131373661` (fallback action-feed; protocol in vault).
- **Mod sources live only in git history** — resurrect, don't rewrite: `git show d71be8af -- mod/` (EVTracker: `TurnStateCapture.java`, `EVLogger.java`, `DecisionTrackingPatches.java`), `git show ec608e30 -- mod/` (PracticeLab + `pom.xml` with system-scoped deps against `$GAME`).

## Vault (`docs/vault/`) — MINE (hard-won mechanics knowledge)

Key files: `rng-system-analysis.md` (all 13 RNG streams, per-floor reseed, cardRng act snapping), `headless-launch.md` (proven MTS launch procedure + JDK notes), `verified-seeds.md` + `test-seeds.md` + 3× `seed-*-full-prediction.md` (per-seed expected values — Phase-0 validation targets), `communication-mod-api.md`, `modding-infrastructure.md`, `basemod-hooks.md`, `enemy-ai-patterns.md`, `rng-parity-audit.md`, `save-system.md`, `map-generation.md`, `shop-mechanics.md`, `card-rewards.md`, `damage-mechanics.md`, `relic-effects.md`, `event-mechanics.md`, `ascension-modifiers.md`, `watcher-cards-complete.md`, `watcher-stances.md`.

## Viz — RESCUE then USE

- **Untracked** in worktree `.claude/worktrees/wonderful-tharp` (branch `claude/wonderful-tharp`): `packages/viz/src/` (React: `sprites/index.tsx` with Watcher/enemy/boss/heart/potion SVG sprites + stance glow, views incl. `CombatAccordion.tsx`, api/hooks), `scripts/viz.sh`, `.claude/launch.json`. Main has only `packages/viz/dist` + `macos`. Preserved by safety commit on that branch (U00 ships it properly).
- Sprites were the "SVGs we made that one time" — they become ParityView's rendering layer (U13).

## Legacy Python — ARCHIVE (U01)

- `packages/engine/` — superseded Python engine (100% Watcher parity in its day; reference when Java is ambiguous, then archive).
- `packages/parity/` — comparison tooling importing the Python engine.
- `tests/` root — already purged to `tests/training/` (live, keep) + `__pycache__`.
- `docs/work_units/python-to-rust-migration.md` + granular-*.md — mark superseded where stale; `CLAUDE.md` "Active Branch Shape" section is stale (references PR #132/#133 stack).

## Training stack — PROTECTED (do not touch)

- `packages/training/` (MLX policy/value, PUCT collection, manifests), `packages/app/SpireMonitor/`, `scripts/training.sh`, `tests/training/`. Consumers of the sim; parity work never edits them.

## Agent infra — USE

- Codex CLI `/Users/jackswitzer/bin/codex` v0.142.5; `~/.codex/config.toml` already `model = "gpt-5.5"`, `model_reasoning_effort = "xhigh"`; `codex exec` for non-interactive. `~/.codex/prompts/goal.md` = the `/goal` loop prompt (created alongside this spec).
- Repo entry point for any agent: `AGENTS.md` (root) → this directory.
- Toolchain: Maven 3.9.12 (`/opt/homebrew/bin/mvn`), brew `openjdk@11` (`--release 8`) or Adoptium JDK8 per `headless-launch.md`; `bun`, `uv`, `jq`, `gh`.
