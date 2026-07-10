# AI RNG Wiring Audit — PR #138 Pre-Merge

Scope: `ai_rng` threading after Stage D (`9984bd86`). Findings only, no fixes.
`R:` = Rust, `J:` = Java. Severity: **P0** merge-blocker, **P1** register-
and-defer divergence, **P2** cosmetic/out-of-scope.

---

## 1. Summary

Stage D correctly establishes the `roll_next_move(enemy, &mut ai_rng) ->
roll_next_move_with_num(enemy, num)` seam and keeps the `ai_rng` stream
advancing one draw per enemy per turn across all acts, tested at
R:`src/tests/test_ai_rng_parity.rs:126-181`. Three **P0** issues break
replay/search determinism: `ai_rng` state dropped by `CombatSnapshotV1`,
`move_history` dropped by `EnemySnapshotV1`, and `combat_state_hash` ignores
both RNG streams. Several **P1** divergences (PRNG algorithm, missing
init-roll, Java recursive sub-rolls, Champ mid-turn re-roll, Writhing Mass
Reactive) are known Stage-D `DEFERRED` trade-offs to register. PR #138 can
merge the Stage-D contract, but the P0 snapshot/hash gaps silently break
determinism properties the engine already claims.

---

## 2. RNG source

- **Rust**: `CombatEngine.ai_rng: StsRandom` at R:`src/engine.rs:123`, seeded
  at R:`src/engine.rs:149` as `StsRandom::new(seed.wrapping_add(0xA1A1_A1A1))`,
  deep-cloned at R:`src/engine.rs:401`. Distinct stream from `engine.rng` so
  shuffle/card draws do not perturb intent.
- **PRNG**: xorshift128+ port of libGDX `RandomXS128` (R:`src/seed.rs:32-120`).
  **Java StS uses `java.util.Random`** (J:`AbstractDungeon.java:148,1737-1741`:
  `aiRng = new Random(Settings.seed + (long)floorNum)`). → **P1: PRNG
  algorithm mismatch**. Same seed yields different sequences; byte-level
  parity to Java is impossible without swapping the PRNG. Uniform-over-0..=99
  distribution parity preserved (JawWorm probability test still holds).
- **Combat-seed**: `combat_seed = self.seed + floor*1000` (R:`src/run.rs:1272`)
  mirrors Java per-floor reseed (J:`AbstractDungeon.java:1737-1741`). Extra
  `0xA1A1_A1A1` xor has no Java analogue — **P2**, moot under F5.

---

## 3. Call graph

One canonical consumer of `ai_rng`:

```
CombatEngine.ai_rng  (R:src/engine.rs:123)
  └─ enemies::roll_next_move(enemy, &mut ai_rng)  (R:src/enemies/mod.rs:822-825)
       ai_rng.random(99)  ← single draw, 0..=99 inclusive
       └─ roll_next_move_with_num(enemy, num)  (R:src/enemies/mod.rs:829-911)
             └─ per-enemy roll_* dispatcher (acts 1–4)
                  └─ set_move(move_id, dmg, hits, block)
```

Call sites of `roll_next_move` (the only functions that draw from `ai_rng`):

| Site | File:line | Trigger |
|---|---|---|
| Normal enemy-turn advance | R:`src/combat_hooks.rs:516` | Every enemy's end-of-turn |
| Champ mid-turn re-roll | R:`src/combat_hooks.rs:568-571` | HP crosses 50% |

There are **no other `ai_rng` call sites** outside these two (grep `ai_rng`
in `packages/engine-rs/src/` shows only field access inside `CombatEngine`
methods + these two consumers).

Mirrors Java `AbstractMonster.rollMove()` at J:`AbstractMonster.java:465-466`:
`getMove(aiRng.random(99))`, with `setMove` pushing to `moveHistory` at
J:`AbstractMonster.java:431-437`.

**P1 — missing init draw**: Java `AbstractMonster.init()` calls `rollMove()`
at J:`AbstractMonster.java:712-715`, consuming one `aiRng` value per enemy
before turn 1. Rust `create_enemy` hard-codes initial intents without drawing
from `ai_rng` (R:`src/enemies/mod.rs:397-810`, see each `create_*` helper
setting `move_id`/intent inline). Stream offset vs Java = `N_enemies`.
`start_combat` at R:`src/engine.rs:237` does not re-roll intents. Out-of-
scope for PR #138 (Stage D was narrowed to per-turn rolls), but required for
full multi-enemy sequence parity.

---

## 4. `num` consumption per enemy

Dispatcher at R:`src/enemies/mod.rs:833-909`: ~28 enemies consume `num`
(JawWorm/Chosen/Byrd/ShelledParasite/SnakePlant/Centurion/Mystic/BookOfStabbing/
Snecko/BronzeOrb/Champ/Collector/Looter/GremlinNob/Darkling/OrbWalker/Spiker/
Repulsor/WrithingMass/SpireGrowth/Maw/GiantHead/Nemesis/Reptomancer/AwakenedOne/
TimeEater/CorruptHeart/Louse-2v); ~13 ignore via `_num` (Cultist/FungiBeast/
SpikeSlime-S/M/L/Mugger/GremlinLeader/Taskmaster/Bear/BanditLeader/
BronzeAutomaton/Exploder/Transient/SnakeDagger/Donu/Deca); ~9 have no `num`
param (Guardian/Hexaghost/SlimeBoss/Lagavulin/Sentry/GremlinWizard/
SphericGuardian/SpireShield/SpireSpear).

**Critical invariant**: every path through `roll_next_move` consumes exactly
one `ai_rng.random(99)` draw — the draw happens at R:`src/enemies/mod.rs:823`
before dispatch, so Act 4 (R:`src/enemies/act4.rs` header) and Cultist
(R:`src/tests/test_ai_rng_parity.rs:189-209`) still advance the stream.

**P1 — `_num` enemies that Java branches on**: Mugger/GremlinLeader/
Taskmaster/Bear/BanditLeader/BronzeAutomaton/Exploder/SnakeDagger all have
`aiRng` branches in Java collapsed to deterministic fallback (Stage-D
`DEFERRED`). Behavioral divergence, not a threading bug — outer draw
preserves stream alignment.

---

## 5. `lastMove` recursion safety

Java `getMove` recurses with a fresh `aiRng.random(...)` when the anti-repeat
guard fires. Known sites in the decompile (additional `aiRng` draws beyond
the initial `rollMove` draw):

- J:`JawWorm.java:153-158` — `aiRng.randomBoolean(0.5625f)` after
  `lastMove(CHOMP)`.
- J:`TimeEater.java:183,202` — `getMove(aiRng.random(...))` recursion.
- J:`Darkling.java:164,179` — recursion.
- J:`WrithingMass.java:160,168,174,176,184,189` — multiple recursions.
- J:`Reptomancer.java:176,191`.
- J:`GremlinLeader.java:160,171`.
- J:`ShelledParasite.java:189`.

Rust implements all of these as deterministic fallbacks inside the single-num
contract (search `DEFERRED` in `act2.rs`/`act3.rs`; JawWorm sub-roll collapse
documented in R:`src/tests/test_ai_rng_parity.rs:77-93`). Stream alignment is
preserved (only one outer draw per `rollMove`) but **behavioral parity to
Java is deliberately lost on the recursive branches**. **P1 (known, Stage-D
DEFERRED)** — must be tracked for Stage E+ if Java parity on anti-repeat
becomes a requirement.

Helpers at R:`src/enemies/mod.rs:917-925` (`last_move`, `last_two_moves`)
read `move_history` only; no additional `ai_rng` draws. Safe.

---

## 6. `move_history` lifecycle

- **Initialized** in `EnemyCombatState::new` (R:`src/state.rs:141-172`) as
  `Vec::new()`, `move_id: -1`.
- **Pushed** only in `roll_next_move_with_num` at R:`src/enemies/mod.rs:830`
  before dispatch. Pushes the *previous* `move_id` (just-executed), matching
  Java's `setMove` contract at J:`AbstractMonster.java:431-437`.
- **Cleared** in three places:
  - `guardian_check_mode_shift` / `guardian_switch_to_offensive`
    (R:`src/enemies/act1.rs:423,435`) — Guardian mode flip.
  - `awakened_one_rebirth` (R:`src/enemies/act3.rs:502`) — phase 2 transition.
- **Read** in `last_move` / `last_two_moves` helpers only.

**P0 — `move_history` dropped by snapshot**: `EnemySnapshotV1` at
R:`src/training_contract.rs:506-521` does **not** include `move_history`.
`combat_engine_from_snapshot` at R:`src/training_contract.rs:737-753`
reconstructs enemies via `EnemyCombatState::new` (empty history) and
`set_move` (R:`src/state.rs:236` — does not push history). After roundtrip,
`last_move(CHOMP)` always returns false, so JawWorm's sub-roll branch (and
every other anti-repeat guard) silently flips behavior. Breaks training
corpus replay / benchmark determinism.

**P1 — Champ mid-turn re-roll double-push**: `on_enemy_damaged`
(R:`src/combat_hooks.rs:568-571`) calls `roll_next_move` after HP crosses
50%. Each call pushes `move_id` to history at R:`src/enemies/mod.rs:830`,
so a single turn can push twice (the `enemy_turn_advance` at line 516 will
*also* run at end-of-turn). History has one extra entry per Champ phase-2
transition. Java triggers phase 2 via `ChangeStateAction` inside `takeTurn`,
not by re-rolling intent, so this is a Rust-only semantic: accepted
divergence but anti-repeat guards behave differently afterward.

**P1 — Writhing Mass Reactive re-roll skips the stream**:
`writhing_mass_reactive_reroll` at R:`src/enemies/act3.rs:186-215` mutates
the intent when Reactive is hit but does **not** consume `ai_rng` and does
**not** push to `move_history` (the function is called outside
`roll_next_move_with_num`). Java handles this through the same
`getMove(aiRng.random(...))` path and consumes a draw. Stream desync for
encounters containing Writhing Mass.

---

## 7. Snapshot-replay determinism

**P0 — `ai_rng` state not serialized.** `CombatSnapshotV1` at
R:`src/training_contract.rs:523-555` contains `rng_seed0`, `rng_seed1`,
`rng_counter` but has no `ai_rng_*` fields. `combat_snapshot_from_combat`
at R:`src/training_contract.rs:658-728` calls `engine.rng.state_tuple()`
only (line 659); `engine.ai_rng.state_tuple()` is never read.
`combat_engine_from_snapshot` at R:`src/training_contract.rs:735-803`
constructs `CombatEngine::new(state, 0)` (line 791) — which re-seeds
`ai_rng` from `0.wrapping_add(0xA1A1_A1A1)` (R:`src/engine.rs:149`) — then
restores `engine.rng` at line 792 but leaves `ai_rng` at the fresh default.
**Result**: snapshot roundtrip silently resets the AI RNG stream.

**P0 — snapshot roundtrip test does not cover RNG.** The existing roundtrip
test `combat_snapshot_roundtrip_preserves_training_surface` at
R:`src/training_contract.rs:1522-1545` asserts player/hand/potion/relic
surface but never asserts RNG state or intent parity across a post-
roundtrip `roll_next_move` call. The bug is not caught by any automated
check.

**Deep-clone path is fine.** `CombatEngine::clone_state` at
R:`src/engine.rs:395-413` copies `ai_rng` (line 401). Search's
`combat_search_is_deterministic_and_replayable`
(R:`src/tests/test_search_harness.rs:8-26`) uses `clone_state`, not
snapshot — which is why existing determinism tests pass while the snapshot
path is broken.

**P0 — `combat_state_hash` does not hash RNG state.**
R:`src/search.rs:1087-1138` hashes phase, player, piles, statuses,
enemies.move_history, but never `engine.rng` or `engine.ai_rng`. Two
engines identical in gameplay surface but divergent in `ai_rng.counter`
hash identically → search transposition table can merge states that
produce different enemy intent sequences. Subtle but corrupts MCTS
backups in any branch after an enemy turn.

---

## 8. Multi-RNG hygiene

- Engine separates `rng` vs `ai_rng` (R:`src/engine.rs:118,123`), mirroring
  Java's split of `cardRandomRng`/`shuffleRng` vs `aiRng`. Good.
- `roll_next_move_with_num` (test path) never touches any RNG — verified at
  R:`src/tests/test_ai_rng_parity.rs:213-225`.
- `RunEngine` has a **single** `rng` field (R:`src/run.rs:486`) vs Java's
  10+ top-level streams (`AbstractDungeon.java:390-401`: `monsterRng`,
  `cardRng`, `mapRng`, `treasureRng`, `eventRng`, `relicRng`, `merchantRng`,
  `miscRng`, `shuffleRng`, `aiRng`). Run-level RNG parity is **out of scope
  for PR #138** (Stage D touches only combat AI), but worth flagging:
  training determinism at the run level currently depends on a single
  stream, and any future work that introduces new draws into `RunEngine.rng`
  will shift every downstream draw. **P2, out-of-scope for this PR**.

---

## 9. Severity-banded recap

| ID | Sev | File:line | What |
|---|---|---|---|
| F1 | **P0** | `training_contract.rs:523-555,658-728,735-803` | `CombatSnapshotV1` drops `ai_rng` state; roundtrip re-seeds from 0 |
| F2 | **P0** | `training_contract.rs:506-521,737-753` | `EnemySnapshotV1` drops `move_history`; anti-repeat corrupted post-roundtrip |
| F3 | **P0** | `search.rs:1087-1138` | `combat_state_hash` ignores `rng`/`ai_rng`; transposition merges diverge |
| F4 | **P0** (cov) | `training_contract.rs:1522-1545` | roundtrip test asserts gameplay surface only, not RNG/history |
| F5 | P1 | `seed.rs:32-120` vs `java/util/Random.java` | PRNG algorithm mismatch (xorshift128+ vs java.util.Random) |
| F6 | P1 | `enemies/mod.rs:397-810` vs `AbstractMonster.java:712-715` | `create_enemy` skips Java init `rollMove()` draw (stream offset by N) |
| F7 | P1 | `act{2,3}.rs` `DEFERRED` + `JawWorm.java:153`, `TimeEater.java:183`, `WrithingMass.java:160`, `Reptomancer.java:176`, `GremlinLeader.java:160`, `Darkling.java:164`, `ShelledParasite.java:189` | Java recursive sub-rolls collapsed to deterministic fallback |
| F8 | P1 | `combat_hooks.rs:568-571` + `enemies/mod.rs:830` | Champ mid-turn re-roll double-pushes `move_history` |
| F9 | P1 | `enemies/act3.rs:186-215` | Writhing Mass Reactive re-roll skips `ai_rng` + `move_history` |
| F10 | P2 | `enemies/mod.rs:836-902` | 13 `_num` enemies collapse deterministic branches (stream stays aligned) |
| F11 | P2 | `run.rs:486` vs `AbstractDungeon.java:390-401` | RunEngine single `rng` vs Java 10+ streams (out-of-scope) |

**Merge guidance**: F1-F4 block merge if PR #138 claims replay/search
determinism; otherwise register-and-defer with the Stage-D seam (well-tested
at R:`src/tests/test_ai_rng_parity.rs`) merging as-is. F1-F3 can be caught
by a single test: snapshot → `combat_engine_from_snapshot` → two
`roll_next_move` calls → assert `move_id` equality (fails today).
