# Simulator Completion Map

**Snapshot:** `codex/engine-deep-audit`, 2026-07-15

**End state:** [`docs/goal/GOAL.md`](../goal/GOAL.md) Definition of Done

**Behavioral authority:** decompiled Java first; frozen real-game traces are the integration oracle

## 2026-07-18 stack update

Layers 1-3 and the pure-core portion of Layer 6 have landed in the RNG through
pure-sim stack. Current proof is 3,011 passing tests, zero ignored, native Java
RNG/stream ownership, canonical full-run actions, causal checkpoints, and no
active Python/observation/search/training dependency in `engine-rs`.

The detailed tables below preserve the 2026-07-15 derivation and should not be
read as current source descriptions. The live remaining gate is Layer 5:
complete the trace differ's power/orb/move-history comparisons, remint the
human-attended Watcher corpus, and burn down first divergences. See
[`pure-sim-freeze.md`](audit-reports/pure-sim-freeze.md).

This is the execution map from the completed 667-row source-verification sweep to a finished Watcher simulator. It is deliberately not a claim that “667 verified” means parity is complete. The ledger currently proves source-derived coverage for four item kinds; the corpus, run RNG topology, missing content domains, and consumer boundaries are still open. Known oracle findings F1-F8 remain owned by [`docs/goal/FINDINGS.md`](../goal/FINDINGS.md) and are referenced here rather than re-derived.

Effort estimates include implementation and engine-path tests, but not queue time for a human-attended game session:

- **S:** up to two focused engineer-days
- **M:** three to seven engineer-days
- **L:** two to four engineer-weeks; split into the worker slices in the two-month plan

## What is proven today

| Surface | Proven state | Evidence | What this does **not** prove |
|---|---|---|---|
| Source-verification ledger | All 667 current rows are `verified`: 370 cards, 68 monsters, 43 potions, 186 relics. | `docs/goal/ledger.json`; `scripts/ledger.sh status` reports no next unverified row. | The ledger has no event, Neow, map-generation, or power rows and no corpus-reachability stamp. |
| Unit suite | The audit began at 2,989 green tests. Accounted dead-helper cleanup now leaves 2,883 passing tests plus 11 ignored audit reproducers. | `AUDIT_PROMPT.md` branch snapshot; cleanup commits retired exactly 106 subject-only tests; `packages/engine-rs/src/tests/` contains 213 `test*.rs` files (215 Rust files total) and 60,616 lines. | Tests alone do not prove full-run ordering or RNG counters; some use private state/setup paths. |
| Combat content registry | Card and gameplay registries are process-wide immutable values returned by `&'static` reference. | `packages/engine-rs/src/cards/mod.rs:1253-1258`; `packages/engine-rs/src/gameplay/registry.rs:102-105`; `CombatEngine` stores the card registry by static reference at `engine.rs:191-204`. | All state is not yet state-local: Note for Yourself uses a process-global mutex, and string IDs still allocate inside snapshots. |
| Trace pipeline | Java capture, Rust replay/diff, schema types, and an offline smoke golden exist. B0 is achieved. | `docs/goal/BENCHMARKS.md` B0; `data/traces/scripts/smoke-neow-floor1.json`; `data/traces/java/smoke-neow-floor1.jsonl`. | The target corpus is about ten A0 runs plus seed `1776347657`; only one script/golden exists and the differ omits fields identified by F6. |
| Performance baseline | Current engine operations are already small enough to measure meaningfully: simple full-turn about 4.14-4.28 us, legal actions about 96.6-104.9 ns, real-world clone cases about 568-776 ns, real-world turn windows about 7.87-11.13 us on the audit M4. | `packages/engine-rs/benches/combat_bench.rs`; `packages/engine-rs/benches/real_world_bench.rs`; 2026-07-15 audit Criterion run. | There is no batch API, allocation budget, thread-scaling benchmark, or pure-core build; PyO3 is unconditional. |

Finding IDs below resolve to the ranked register at
[`audit-reports/engine-deep-audit.md`](audit-reports/engine-deep-audit.md).

## Layer 1 — Combat simulator

### Current shape

`CombatEngine::execute_action` rebuilds the effect runtime and routes player actions through the active engine path (`packages/engine-rs/src/engine.rs:525-543`). `EffectRuntime` owns event dispatch, hidden instance state, and persisted cross-combat state. `CombatState` contains the card piles, enemies, counters, relic IDs/counters, and RNG-bearing engine state (`state.rs:284-357`). The current registries are shareable, but a cloned engine still deep-clones causal and diagnostic vectors (`engine.rs:546-565`).

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **C1. Close confirmed combat P0s through engine-path tests.** | The audit repro file covers victory truncation, numeric overflow, per-floor RNG, nested dispatch, Devotion, copy/use semantics, enemy-power timing, orb routing, and terminal-death gating; shuffle and owner-turn timing have explicit queued test sketches. | Each deterministic repro is fixed, loses `#[ignore]`, cites Java, and passes through `CombatEngine`; family sweeps add sibling boundary cases rather than only the original scalar example. | None. Do this before architectural movement so behavior changes and refactors remain separable. | L | EDA-001, EDA-002, EDA-003, EDA-004, EDA-006, EDA-007, EDA-008, EDA-009, EDA-010, EDA-011, EDA-012, EDA-013. |
| **C2. Replace lossy gameplay integers with Java-compatible widths.** | `EntityState::set_status` and `add_status` cast `i32` to `i16` (`state.rs:120-134`); card `misc` is also compact and can wrap under repeated modification. | Gameplay-significant amounts use `i32` or a proven bounded type; serialization, observation, trace, effect op, and snapshot paths preserve the same value; tests cover positive and negative boundaries plus stacking actions. | Boundary/schema plan API4, because stored widths cross snapshots and tokens. | L | EDA-002, EDA-003. |
| **C3. Make dispatch completion and reentrancy explicit.** | Combat victory is represented in state before victory events finish, and the audit found handler iteration can be cut short. Effect dispatch is rebuilt once per action to accommodate externally mutated setup (`engine.rs:525-530`). | A documented event contract specifies handler ordering, install/remove visibility, nested emits, owner death, power removal, victory, and queue resume. A source-derived matrix drives the production path for every trigger family. | C1 first; API1/API4 should preserve this contract rather than expose internals. | M | EDA-001; EDA-006. |
| **C4. Retire `complex_hook` as an unbounded escape hatch.** | The card definitions still contain 24 `complex_hook: Some(...)` entries; powers, potions, and relics have additional hooks. | Each hook is classified as inherently imperative or expressible in typed ops. Declarative cases migrate with engine-path parity tests; the remaining hook interface is owner-aware, deterministic, and intentionally small. | C3 event contract; C1 P0 closure. | M | EDA-028. |
| **C5. Separate causal state from diagnostics and worker scratch.** | `clone_state` copies `event_log`, effect runtime vectors, choice/program queues, and card/enemy vectors (`engine.rs:546-565`). | A combat snapshot contains only future-behavior state; trace/event sinks and reusable temporary buffers live outside it. Clone benchmarks and deterministic replay tests prove the split. | API1 pure-core boundary, API2 typed IDs. | M | EDA-014, EDA-015, EDA-018. |

**Combat exit tests**

1. All audit P0 repros run unignored inside `./scripts/test_engine_rs.sh test --lib`.
2. Trigger-family tests cover combat start, turn start/end, card play, discard/exhaust/shuffle, damage/debuff, potion use, enemy death, and victory through `CombatEngine`.
3. A snapshot clone followed by the same action sequence produces byte-equivalent causal state, legal actions, RNG counters, and canonical trace records.
4. No gameplay-significant value silently narrows; boundary tests exercise values above `i16::MAX` and below `i16::MIN` where Java permits them.

## Layer 2 — Run layer

### Current shape

The run engine implements Neow, map choice, combat, reward, campfire, shop, event, and game-over phases, but one `RunEngine` owns all phase sidecars and pending relic/event state (`packages/engine-rs/src/run.rs:758-811`). `RunState` keeps both `deck: Vec<String>` and aligned `deck_card_states`, plus both `relics: Vec<String>` and a skipped `relic_flags` cache (`run.rs:338-390`). `step_with_result` validates an action, steps, rebuilds decisions, and eagerly produces legal actions, decision objects, events, context, and a copied observation (`run.rs:1471-1521`).

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **R1. Normalize `RunEngine` into explicit phase state.** | One cloneable struct carries reward, event, shop, combat, encounter, Neow, and many `pending_*` fields (`run.rs:758-811`). | `RunSnapshot` owns stable run state and an enum owns exactly one phase payload. Relic/event continuations are typed decision frames, not unrelated booleans. Transitions are exhaustively matched and source-tested. | API1 public core API design; preserve trace action identity. | L | EDA-027. |
| **R2. Establish one canonical deck and relic representation.** | Deck strings are reconciled to aligned `CardInstance`s by base-name search (`run.rs:616-638`); relic flags are omitted by serde (`run.rs:384-390`). | Deck entries carry stable instance IDs and persistent card state in one vector. Relics are canonical typed instances with flags/counters derived or rebuilt on load. Round-trip serialization cannot attach `misc` to the wrong duplicate or lose relic capability state. | API2 typed content IDs; API4 versioned snapshots. | L | EDA-020, EDA-026. |
| **R3. Remove cross-state global gameplay data.** | Note for Yourself is stored in `OnceLock<Mutex<String>>` (`run.rs:27-43`). Parallel simulations can therefore affect one another. | Profile/save-derived inputs are an immutable `ProfileSnapshot` supplied to a root run; profile updates are explicit outputs outside rollout state. No gameplay outcome reads mutable process-global data. | API1 design. | S | EDA-019. |
| **R4. Finish run transition semantics and trace action coverage.** | The trace mapper documents only play, end turn, potion, Neow, and path mappings and rejects other actions (`trace.rs:706-712`); F4 records a known Neow mapping mismatch. | Every run decision used by a Watcher A0 full run has a stable typed action and Java/Rust mapping: rewards, events, campfires, shops, keys, chests, boss transitions, and Act 4. Illegal actions are deterministic and do not consume RNG. | R1 decision model, G1 RNG model, T1 trace schema. | L | FINDINGS F4/F7; EDA-023. |
| **R5. Verify phase transitions as behaviors, not incidental side effects.** | The four-kind item ledger cannot express floor transition, room generation, reward ordering, or save/profile behavior. | Source-derived scenario tests cover Neow-to-map, every room phase, boss/act changes, keys/Act 4, event combat resume, shop refill, reward sequencing, death, and run victory. Each scenario has a ledger row or generator invariant. | Coverage layer K1 and G1 RNG model. | L | FINDINGS F2/F4; EDA-032. |
| **R6. Port canonical seeded encounter queues.** | `run.rs:216-300` has incomplete simplified encounter arrays and `run.rs:1656-1685` rotates them by ordinal, independent of seed. The EDA-005 seed-4 repro selects Cultist where Java's weighted queue selects Jaw Worm. | Every act uses Java's names, compositions, weights, first-strong exclusions, and repeat rejection with a dedicated persistent `monsterRng`. Queue contents/counters match source-derived seed fixtures and route through canonical `MonsterHelper` encounter expansion. | G1 typed run RNG ownership; K1 generator-invariant rows. | L | EDA-005. |

**Run-layer exit tests**

1. Serialize/deserialize at each run phase, then compare all future legal actions, next transition, and RNG counters.
2. Two identical roots stepped independently never communicate through globals or caches.
3. A script can express and replay every decision in a complete Watcher A0 run without adapter-specific guesses.
4. Run phase transitions are covered by source-derived integration tests and then by at least one Java golden.

## Layer 3 — RNG streams and determinism

### Current shape

Java parity requires 13 named streams. Outside combat, Rust exposes one catch-all run RNG as `card` (`packages/engine-rs/src/run.rs:7646-7663`, FINDINGS F2). Combat exposes five counters (`engine.rs:228-243`) and constructs its AI stream from a Rust-specific seed offset (`engine.rs:186-204`). The trace schema knows all 13 canonical names (`trace.rs:670-693`), but knowledge of names is not equivalent to owning the streams.

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **G1. Model all persistent and per-floor Java RNG streams.** | `RunEngine` owns one `StsRandom` (`run.rs:758-765`) for map/shop/event/relic/etc.; only one counter is available out of combat. | A typed `RunRngs` owns all canonical streams with Java seed initialization, signed/unsigned semantics, and exact call-site routing. It clones/serializes with the run and exposes counters without relabeling. | API1 state ownership; authoritative initialization audit from decompiled `AbstractDungeon`. | L | FINDINGS F2; EDA-004. |
| **G2. Reproduce floor-transition reseeding exactly.** | Combat RNG construction currently seeds `ai` with an implementation-specific offset and aliases potion/misc/card-random seeds (`engine.rs:186-204`). | Floor transition tests assert seed and counter zero-points for monster HP, AI, shuffle, card-random, misc, and every persistent stream that must *not* reset. Combat receives handles/state derived from `RunRngs`, never invents seeds. | G1. | M | EDA-004. |
| **G3. Prove draw-count parity at every stochastic boundary.** | The one smoke trace cannot exercise the family; F1 demonstrates how a green hand-written AI test can omit an extra draw. | Table-driven source tests cover each conditional second draw and rejection loop. Corpus records compare all counters after every action; a counter divergence is always reported before downstream state. | G1/G2; T2/T3. | L | FINDINGS F1/F3; FINDINGS F6. |
| **G4. Prove thread-local determinism.** | Registries are immutable, but Note for Yourself is global and there is no `step_many` test. | Identical indexed states produce identical outputs under 1, 2, and N worker schedules. No RNG or gameplay-significant data is shared through a lock/global; result ordering follows input indices. | R3, API3 batch API. | M | EDA-019, EDA-017. |

**RNG exit tests**

1. Every trace record contains the full canonical stream set and agrees with the Java golden counter-for-counter.
2. A same-seed/same-action run repeated serially and in parallel produces byte-identical snapshots and canonical traces.
3. A per-floor fixture proves reset/non-reset behavior independently of the full corpus.
4. No stochastic call site accesses an undifferentiated catch-all RNG.

## Layer 4 — Content and reachability coverage

### Current shape

The current ledger is complete only for extracted cards, monsters, potions, and relics. It has no rows for events, Neow rewards, map generation, or powers, despite those systems affecting every full run. It also has no `reachable` or `covered_by` field, so source verification cannot be joined to actual corpus exercise. The 667 verified rows are valuable and remain the base layer; extending the schema must preserve their evidence.

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **K1. Extend extraction and the ledger to missing domains.** | `ledger.json` contains only `card`, `monster`, `potion`, and `relic`. | `scripts/extract.sh` deterministically emits event, Neow, map/generator, and power rows with Java/method references. Existing statuses survive regeneration. Generator behaviors that do not map 1:1 to a class are named invariant rows with explicit source ranges. | Agree row identity and reachability rules; no engine refactor required. | M | EDA-032. |
| **K2. Compute Watcher reachability.** | “Watcher-reachable” is a prose condition in GOAL, not data in each row. | A deterministic reachability pass records character/act/ascension gates and explains exclusions. Coverage status commands distinguish `verified`, `quarantined`, `unverified`, `unreachable`, and `not exercised`. | K1; content catalog IDs from API2 help but are not required. | M | EDA-032. |
| **K3. Verify the new domains from Java.** | Events and powers have implementations/tests, but no ledger evidence contract equivalent to the completed four families. | Every reachable row has a Java citation, source-derived engine-path test, and `verified`/`quarantined` status. Powers are tested on the correct owner and event family. Event rows cover option legality, mutations, continuations, RNG draws, and event combat resume. | K1; C3 for power dispatch; G1 for stochastic content; R1/R5 for events. | L | EDA-032. |
| **K4. Join source verification to corpus coverage.** | U07/`scripts/goal.sh coverage` is still future in canonical docs; the one golden cannot cover the catalog. | Each trace action stamps observed content/invariant rows via stable IDs. The dashboard lists reachable-but-unexercised rows and drives targeted scripts; a row is never called corpus-covered by a test-only reference. | T4 corpus and K1 row identity. | M | `docs/goal/UNITS.md` U07; FINDINGS F6. |

**Coverage exit tests**

1. Ledger regeneration is stable and preserves evidence/status for all current rows.
2. Zero `unverified` rows remain on the Watcher-reachable set across all eight domains.
3. Every reachable row is either exercised by at least one corpus trace or listed explicitly as a targeted-corpus gap/quarantine.
4. Every quarantine has a narrow `DEV-NNN` register entry and mask; total remains reviewable under GOAL's guideline.

## Layer 5 — Trace oracle and corpus

### Current shape

The schema is hard-versioned at `v:1` (`packages/engine-rs/src/trace.rs:45-60`). The differ compares RNG, player scalars, enemy scalars/intents, piles, relic IDs, and potions (`trace.rs:561-634`), while F6 lists missing power/history/counter fields. Rust hardcodes relic counters to `-1` in snapshots (`trace.rs:770-800`). Only `smoke-neow-floor1` has both a script and golden; GOAL requires roughly ten A0 coverage seeds plus seed `1776347657`.

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **T1. Define an additive-tolerant trace v2.** | `check_version` rejects every value except exactly 1 (`trace.rs:49-60`); action contracts contain unresolved type nits in F7. | An envelope declares schema name, major/minor, capabilities, and producer. Unknown additive fields survive or are ignored by projection; incompatible majors fail clearly. Canonical field ordering is specified for deterministic JSONL. | API4 shared versioning policy; action vocabulary from R4. | M | FINDINGS F7; EDA-022. |
| **T2. Complete Rust and Java action vocabulary.** | The Rust mapper rejects reward/event/campfire/shop actions beyond its documented subset (`trace.rs:706-712`). | All actions needed by the corpus have one canonical typed representation and equivalent Java/Rust execution. Schema fixtures cover every variant and invalid payload. | R4; T1. | M | FINDINGS F7; EDA-023. |
| **T3. Make emitted and compared state complete.** | Relic counters are `-1`; F6 identifies powers, orbs, move history, and counters omitted from comparison. | Rust emits real counters and normalized IDs. The differ compares every GOAL field, with RNG first, and contains negative fixtures proving a single-field regression cannot report `match`. | R2 canonical relics; C3 power ownership; G1 counters. | M | FINDINGS F6; EDA-020. |
| **T4. Build the full A0 corpus.** | One smoke script/golden exists versus the target of about eleven full runs. | Coverage-oriented scripts exist for about ten A0 seeds plus `1776347657`; each has a human-minted A/B deterministic golden and an offline oracle test. Targeted short scripts cover otherwise unreachable edge cases. | T1/T2 for usable scripts. Minting itself is human-attended; engine matching can proceed incrementally after G1. | L plus human sessions | `docs/goal/UNITS.md` U06; FINDINGS F4/F6; EDA-033. |
| **T5. Burn down divergences to B3.** | B0 is achieved; B1+ are not established by a complete corpus. | Every committed script returns `match` or a narrow reviewed `DEV` mask; all 13 counters and complete post-state agree after every action. First-divergence reports remain reproducible offline. | G1-G3, K3, T3/T4. | L | All correctness EDA entries plus FINDINGS F1-F7. |

**Oracle exit tests**

```bash
for script in data/traces/scripts/*.json; do
  scripts/trace_diff.sh "$script" || exit 1
done
./scripts/test_engine_rs.sh test --lib
```

The loop passes only when every script has a committed protected golden, schema fixture tests cover compatibility behavior, and intentionally perturbed fixtures prove every required state family is compared.

## Layer 6 — Core, batch, observation, and consumer boundaries

### Current shape

The future Python training and SwiftUI monitor consumers are being rebuilt, so current wire compatibility is not a constraint. Today, however, PyO3 is unconditional in `packages/engine-rs/Cargo.toml:12-24` and appears directly in core `actions.rs`, `state.rs`, `engine.rs`, and `lib.rs`. `obs.rs` exposes a version-4 fixed vector through functions still named v2 (`obs.rs:584-611`). All training contract constants are exact version 1 (`training_contract.rs:21-26`), and adding `power_cards_played_this_combat` already demonstrated that an additive Rust field can break a strict Python constructor. `RunEngine::step_with_result` also allocates a consumer-heavy result on every step (`run.rs:1471-1521`).

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **API1. Extract a zero-Python simulation core.** | PyO3 types/attributes are imported by core state and engine modules; the dependency is unconditional, and the organically public surface has no growth boundary. | `sim-core` (crate or dependency-enforced module boundary) contains content, state, actions, RNG, step, legal enumeration, snapshots, and trace events with no Python/obs/search imports. Python and app adapters are opt-in features/crates; `check-arch`, no-default-feature builds, and a public-API allowlist enforce direction. | Decide public API before R1/R2 migration. | M | EDA-021, EDA-031. |
| **API2. Replace hot string identity with typed stable IDs.** | Enemy and relic/potion state owns `String`s (`state.rs:141-155`, `state.rs:304-340`); run state also uses strings (`run.rs:338-355`). | Content strings exist in the immutable registry/serialization adapter; causal state uses compact `CardId`, `EnemyId`, `RelicId`, `PotionId`, and instance IDs. Lookups are O(1) and snapshots do not clone names. | Stable catalog from API1; serialization migration API4. | L | EDA-024, EDA-020, EDA-026. |
| **API3. Add deterministic in-place and batch step APIs.** | Only scalar `step`/`step_with_result` exist; the latter enumerates legal actions before and after, builds observations, and clones events. | Core provides `step_in_place(state, action, scratch, event_sink)` and `legal_actions_into`; a thin indexed parallel executor applies one immutable `ContentDb` to N independent states. Output ordering is deterministic and policies can batch observations separately. | API1/API2, G4, C5. | M | EDA-015, EDA-016, EDA-017. |
| **API4. Design versioned snapshots/tokens for rebuilt consumers.** | Fixed exact versions and strict consumer construction made an additive field breaking. | Every boundary uses `{schema:{name,major,minor}, capabilities, payload}` or an equivalent documented envelope. Consumers project known fields, defaults cover missing additive fields, unknown minor fields do not fail, and breaking changes increment major. Core state is not the wire model. | API1 and R2 canonical state. | M | EDA-022. |
| **API5. Make observations a consumer-owned projection.** | `obs.rs` reads engine internals such as hidden effect values and mixes current engine state with a fixed legacy shape. | Core exposes typed public snapshot/events sufficient to derive legal observations. The rebuilt trainer owns vector/token encoding and versioning; the rebuilt monitor consumes trace/snapshot DTOs. No RL-specific field forces a core dependency. | API1/API4; C3 identifies canonical counters. | M | EDA-025. |
| **API6. Add throughput and allocation gates.** | Existing benches measure scalar turns, legal actions, and cloning, but not allocation counts or multithread scaling. | Criterion records scalar and batch throughput at representative combats, snapshot clone, legal enumeration, and allocations/step. A 1-vs-N determinism benchmark runs in CI or a stable performance job. Changes cannot silently regress the 2026-07-15 baseline. | API3 implementation. | S | EDA-034. |

**Boundary and batch exit tests**

1. `cargo check --manifest-path packages/engine-rs/Cargo.toml --all-targets --no-default-features` builds without Python or framework environment variables.
2. `scripts/goal.sh check-arch` rejects any core import of Python, observation, search, or training adapters.
3. Compatibility fixtures cover old-minor, current-minor, future-additive-minor, and rejected-major payloads.
4. Batch execution with one shared registry is byte-equivalent to scalar execution at each input index and has no gameplay locks/globals.
5. Criterion publishes scalar/batch throughput, clone cost, legal action cost, and allocation counts against the audit baseline before setting tighter regression thresholds.

## Layer 7 — Test suite health

### Current shape

The large test corpus is a valuable source-verification asset, but its physical organization and setup style make refactoring harder than necessary. The 215 Rust files under `src/tests` total 60,616 lines; more than 700 assignments directly mutate `engine.state.*`, and 19 files reference hidden effect accessors. Direct setup is not automatically wrong, but tests that skip installation, event dispatch, action queues, or phase transitions cannot by themselves prove production behavior. The single real golden is the only current cross-layer oracle.

| Gap | Current evidence | Done looks like | Blocking dependencies | Effort | Feeds from |
|---|---|---|---|---|---|
| **Q1. Turn every confirmed P0 into a permanent engine-path regression.** | Audit repros are intentionally ignored until their fixes land. | No fixed issue remains ignored. Each test names the Java method or golden, enters through the narrowest production action, and fails under the pre-fix behavior. | C1/G2/R6 fixes. | S per wave | EDA-001–013. |
| **Q2. Classify and migrate private-coupled tests.** | Hidden effect access and direct state mutation are common in test setup. | A scripted audit classifies each use as valid fixture setup, oracle inspection, or pipeline-skipping coverage. Pipeline-skipping assertions gain an engine-path companion before internals are hidden. | C3/API1 public snapshot and fixture builders. | L | EDA-030. |
| **Q3. De-duplicate wave files by behavior matrix.** | Hundreds of wave-oriented files and several very large modules obscure which Java behavior is uniquely covered. | Tests are grouped by production subsystem/event family with source references and a coverage index. Exact duplicates are consolidated only after proving the retained test fails under the same mutation. Test count drops, if any, are explicitly accounted for by stronger replacement coverage. | Q2 classification; no runtime blocker. | M | EDA-030. |
| **Q4. Add cross-layer and mutation-strength gates.** | A test can pass while the trace differ ignores the regressed field (F6), and helper-only tests can validate the implementation against itself (F1 pattern). | For each critical family, one unit test, one engine-path scenario, and one trace/differ negative fixture exist. Targeted mutation checks prove the scenario fails when RNG draw, queue order, amount, or owner is changed. | T3 and corpus. | M | FINDINGS F1/F6; EDA-029. |
| **Q5. Keep performance tests representative.** | Current benches cover six real-world encounters but not run decisions, effect-heavy fights, batched stepping, or allocation counts. | Benchmark fixtures add effect-heavy Watcher turns, multi-enemy death/victory, run phase transitions, clone-heavy PUCT expansion, and 1/N worker batches. | API3/API6. | S | EDA-034. |
| **Q6. Approve the post-sweep agent contract.** | Repository facts are current, but the external `/goal` prompt still prescribes the completed 667-row loop and protected canonical inventories retain stale links. | A human approves the proposed successor contract in the audit register, then the external prompt and protected canonical references are updated together. | Human approval; no engine dependency. | S | EDA-035. |

**Test-health exit tests**

1. Every verified content or generator row links to at least one source-derived test; every open P0 links to an explicit queued test task.
2. Helper/internal tests are never the sole evidence for a gameplay behavior.
3. The library suite, all offline goldens, schema compatibility fixtures, architecture check, and deterministic batch tests run in CI.
4. Any removed duplicate test is documented with the retained replacement and a mutation demonstrating equivalent fault detection.

## Finished-sim gates

The simulator is finished only when these gates hold together:

| Gate | Command/evidence | Required result |
|---|---|---|
| **Parity corpus (B3)** | `scripts/trace_diff.sh` over every committed script | About eleven full A0 runs plus targeted scripts match through Heart, including complete post-state and all 13 counters, or have narrow reviewed DEV masks. |
| **Coverage (B4)** | regenerated ledger + reachability/coverage dashboard | Zero unverified Watcher-reachable rows across cards, monsters, relics, potions, powers, events, Neow, and map/generator invariants. |
| **Regression (B5)** | lib suite, oracle suite, architecture checks, schema fixtures | Green offline; no ignored fixed P0; pure-core builds with and without adapters. |
| **Parallel-state contract** | scalar-vs-batch determinism suite + Criterion | One immutable registry serves N independent states; no gameplay global/lock; indexed outputs are identical to scalar execution; baseline and allocations are published. |
| **Inspectability** | rebuilt ParityView/monitor trace adapter | Any divergence opens at the first mismatching action with Java/Rust values and canonical IDs/counters. |
| **Quarantine** | masks + deviations register | Every mask has a scoped DEV entry and the list remains below GOAL's reviewability guideline. |

The legacy Python training implementation is not a compatibility dependency for the boundary redesign or parity closure. Its replacement must consume the versioned adapter and supply its own contract tests before it becomes a production consumer; it must not force training/search types back into sim core.

## Sequenced next-two-month plan

The schedule assumes three to four parallel worker slots and one or more human-attended corpus mint sessions. Correctness and state contracts precede broad optimization; corpus scripting and ledger extraction can run in parallel.

| Window | Work unit and owner-sized slice | Depends on | Exit test / artifact |
|---|---|---|---|
| **Week 1** | **W1-A `combat-p0-dispatch`** — fix victory-handler completion and nested runtime reentrancy; unignore only the matching repros and add the death/exhaust/removal family matrix. | None | EDA-001/006 engine-path tests plus full lib suite green. |
| **Week 1** | **W1-B `numeric-state-design`** — inventory every narrowing write for status, card misc, relic counters, move effects, and effect state; choose widths and migration/serde plan. | None | Checked inventory in the EDA register; compile-time/newtype plan; boundary tests written ignored only where a fix is still pending. |
| **Week 1** | **W1-C `core-boundary-contract`** — specify public sim-core types, adapter direction, snapshot envelope, content IDs, scratch/event sink, and major/minor compatibility rules. | None | Small reviewed API design recorded with compile/test acceptance criteria; no runtime change bundled. |
| **Week 1-2** | **W2-A `ledger-domains`** — extend extraction with powers, events, Neow, and map/generator invariant rows; add reachability metadata without changing existing statuses. | None | Two regenerations byte-identical; 667 existing statuses/evidence preserved; new rows start honestly unverified. |
| **Week 2-3** | **W2-B `run-rng-streams`** — introduce typed 13-stream ownership and route non-combat call sites in narrow families (map/event, rewards/relics, merchant/potion, per-floor combat streams). | W1-C for ownership; source audit | Per-stream seed/counter fixtures; full suite after each family; no catch-all RNG call sites at completion. |
| **Week 2-4** | **W2-E `encounter-queues`** — replace rotating simplified pools with Java's weighted per-act weak/strong/elite queues, exclusions, and canonical encounter expansion. | W2-B `monsterRng` ownership; K1 invariant identity can land in parallel | Seed fixtures cover queue order, weights, repeat rejection, first-strong exclusions, and every canonical composition; EDA-005 is unignored. |
| **Week 2-3** | **W2-C `sim-core-cut`** — feature-gate/remove PyO3 from core, add architecture check, and move bindings/obs/search behind adapters without preserving obsolete Python shapes. | W1-C | Pure no-default-feature all-target build; Python-feature build; `check-arch`; lib suite. |
| **Week 2-4** | **W2-D `numeric-state-migration`** — implement the W1-B width changes subsystem by subsystem, beginning with card misc/status values implicated by P0s. | W1-B, W1-C boundary policy | EDA-002/003 regressions unignored; snapshot/schema compatibility fixtures; full suite after each commit. |
| **Week 3-4** | **W3-A `run-state-normalization`** — canonical deck instances, typed relic instances/cache rebuild, phase payload enum, profile snapshot for Note for Yourself. Split into deck, relic, global-profile, then phase commits. | W1-C; preferably W2-C IDs | Phase round-trip tests, duplicate-card misc test, relic cache load test, two-root isolation test. |
| **Week 3-4** | **W3-B `trace-v2-actions`** — land additive-tolerant envelope and complete reward/event/campfire/shop/key action mappings on both replay sides. | W1-C; R4 action design; can parallel W3-A behind stable DTO | Version compatibility matrix; every action variant fixture; smoke golden remains readable. |
| **Week 4-5** | **W4-A `trace-complete-state`** — emit real relic counters, normalize power IDs, compare powers/orbs/history/counters, and add negative differ fixtures. | W3-A canonical relics; W3-B schema; C3 ownership | Each single-field perturbation yields `first_divergence`; F6 acceptance list closed. |
| **Week 4-5** | **W4-B `batch-step-api`** — add in-place/legal-into APIs, external scratch/event sink, indexed parallel executor, and allocation/thread benchmarks. | W2-C core, W3-A state, W2-B RNG | Scalar-vs-1/2/N byte-equivalence; no gameplay globals; published M4 throughput/allocation baseline. |
| **Week 4-6** | **W4-C `missing-domain-verification`** — parallel source-verification workers by domain: powers/event families, Neow, and map/generator invariants. | W2-A; C3 for powers; W2-B for stochastic rows | Each row has citation + engine-path test before flip; full suite per batch. |
| **Week 5** | **W5-A `corpus-script-pack`** — write complete A0 action scripts/requests selected for row coverage, including `1776347657`; validate every action offline against Rust. | W3-B; W2-A coverage IDs | Every script parses/replays to a clear outcome; coverage report predicts rows exercised; mint manifest ready. |
| **Week 5-6, human** | **W5-H `mint-a0-corpus`** — A/B mint the script pack in attended real-game sessions; freeze goldens. | W5-A | Same-seed Java outputs byte-identical; each script has one protected golden and header metadata. |
| **Week 5-7** | **W5-B `test-health-wave`** — classify private-coupled/duplicate tests, add engine-path companions, reorganize only after fault-detection equivalence is proven. | W2-C public snapshot/builders; can begin classification earlier | Coverage index, no behavior supported only by hidden/private tests, accounted replacement map. |
| **Week 6-7** | **W6-A `corpus-divergence-waves`** — fix first divergences in dependency order: RNG counter, transition/order, state amount/owner, then representation. One source-cited repair per commit. | W5-H, W4-A, W2-B | Each wave increases matched-action prefix; all P0s get permanent tests; no masks without DEV entries. |
| **Week 7** | **W7-A `coverage-corpus-join`** — stamp `covered_by`, identify reachable unexercised rows, add short request scripts, and finish new-domain ledger rows. | W2-A, W5-H, W4-C | Zero reachable unverified rows; every uncovered row explicitly queued/quarantined. |
| **Week 7-8** | **W7-B `consumer-adapter-fixtures`** — provide stable Rust snapshot/trace DTO examples for rebuilt training and monitor teams; implement observation projection outside core only if required for acceptance. | W2-C, W3-B, W4-B | Old/current/future-minor fixtures pass; rejected-major diagnostics clear; no core dependency reversal. |
| **Week 8** | **W8-A `closure-ladder`** — run B3/B4/B5 together, complete targeted goldens, triage quarantine, rerun scalar/batch Criterion, and wire the rebuilt ParityView adapter. | All prior units | All finished-sim gates above pass from a clean checkout; remaining work, if any, is a named DEV quarantine rather than an implicit gap. |

### Critical path

```text
core contract -> sim-core cut -> canonical run state -> batch API
       |                 |                |
       +-> RNG streams --+-> trace complete state -> corpus divergence closure
       |                                  ^
ledger domains -> domain verification ----+
trace v2/actions -> corpus scripts -> human mint
```

The human mint is a scheduling dependency, not a reason to pause offline work. Before goldens arrive, workers can complete source-derived tests, RNG topology, full action parsing, trace negative fixtures, schema compatibility, ledger expansion, and deterministic batch tests. Once goldens land, work always follows the earliest unmasked divergence so downstream differences are not debugged against an already-wrong prefix.
