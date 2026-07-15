# Engine Deep Audit

**Snapshot:** `codex/engine-deep-audit`, 2026-07-15

**Authority:** decompiled Java for behavior; committed Java traces for integration parity

**Register size:** 35 findings — 13 P0, 13 P1, 9 P2

## Ranked top 10

| Rank | ID | Band | Finding | Why it is first | Effort |
|---:|---|:---:|---|---|:---:|
| 1 | EDA-006 | P0 | Nested runtime events are dispatched into a temporary empty runtime | Death-trigger chains silently lose downstream handlers, so ordinary multi-relic combat outcomes can be wrong. | M |
| 2 | EDA-005 | P0 | Run encounters rotate incomplete hard-coded pools instead of Java's seeded weighted queues | Legal encounters are absent, wrong-sized, and seed-independent, so full-run parity is impossible. | L |
| 3 | EDA-012 | P0 | Combat shuffle uses the wrong algorithm, stream contract, and counter accounting | Card order and RNG counters diverge at every reshuffle, poisoning all later parity evidence. | M |
| 4 | EDA-004 | P0 | Per-floor combat RNG streams are seeded and reset unlike Java | AI, shuffle, card-random, misc, and potion state start from noncanonical seeds/topology. | M |
| 5 | EDA-003 | P0 | Status amounts narrow to `i16` and wrap | Legal repeated stacking can turn a large positive power into an unrelated value. | L |
| 6 | EDA-001 | P0 | Combat-victory dispatch stops after the first handler that observes combat over | Later relic, blight, and power victory effects are skipped. | S |
| 7 | EDA-008 | P0 | Necronomicon replay bypasses the normal card-use pipeline | The copy omits card-play counters and use/exhaust/relic/power hooks. | M |
| 8 | EDA-014 | P1 | `CombatSnapshotV1` cannot deterministically resume a combat | It omits RNG streams and causal runtime/queue state while presenting itself as a restorable snapshot. | L |
| 9 | EDA-017 | P1 | PUCT is scalar and clone-heavy with no batched leaf-evaluation boundary | The primary future workload cannot share one immutable registry across efficient parallel rollouts. | L |
| 10 | EDA-022 | P1 | Boundary schemas are exact-version and non-evolvable | One additive field already breaks a consumer; the rebuilt clients need a tolerant contract before they couple again. | M |

## Audit coverage and readiness

| Audit area | Result | Evidence |
|---|---|---|
| 1. Bugs and edge cases | 13 confirmed P0s; 11 have ignored deterministic reproducers, 2 have source/code proofs | `packages/engine-rs/src/tests/test_audit_repros.rs`; EDA-001–013 |
| 2. Stale and weak tests | Four source-contradicting/weak families plus suite-wide coupling and duplication debt | EDA-029–030 |
| 3. Abstractions | Run phase, hook, representation, identity, API, and coverage-model debt mapped | EDA-020, EDA-024, EDA-027–028, EDA-031–032 |
| 4. Batching and parallelism | Registry sharing is viable; clone, scalar search, global state, and allocation boundaries remain | EDA-015–019, EDA-034 |
| 5. Contract redesign | Pure-core, tolerant schema, trace vocabulary, and consumer-owned projection are specified | EDA-014, EDA-021–026 |
| 6. Dead code purge | Executed in two focused code-cleanup commits; 106 subject-only tests accounted for | `695df979`, `6a2f2d07` |
| 7. Docs refresh | Pre-sweep quarry deleted; seven live source indexes and the deviation register refreshed | `1961ea8c`, `92ae11ba` |
| 8. Process/tooling | Dead scripts/hooks/config removed; factual guidance refreshed; contract change remains human-gated | `b84bda57`; EDA-035 |
| 9. Completion map | Seven-layer dependency map, exit gates, and an eight-week worker queue produced | `docs/work_units/sim-completion-map.md` |

The cleanup-adjusted library baseline is 2,883 passing and 11 intentionally ignored audit reproducers. The two code-cleanup commits removed exactly 106 tests whose only subjects were deleted helpers (40 plus 66). `cargo check --all-targets` is clean. The 667-row ledger remains 370 cards, 68 monsters, 43 potions, and 186 relics, all verified; that ledger does not cover events, Neow, powers, or map/generator invariants.

Criterion measurements on the audit M4 give simple full turns at roughly 4.14–4.28 us, legal-action enumeration at 96.6–104.9 ns, representative state clones at 568–776 ns, and representative turn windows at 7.87–11.13 us. These are useful scalar baselines, not batch-readiness proof: there is no allocation counter, `step_many`, thread-scaling run, or no-Python core build.

Known findings F1–F8 remain owned by `docs/goal/FINDINGS.md`. This register cross-references F1, F2, F4, F5, F6, F7, and F8 where they constrain a fix; it does not re-register them. In particular, EDA-004 is the newly confirmed stream-ownership/per-floor combat-seeding defect, distinct from F2's already-known noncombat counter-exposure defect, while EDA-033 is the corpus-volume/process gap rather than F6's already-known compare-field omissions.

# P0 — correctness and parity

## EDA-001 — Victory dispatch truncates later handlers

**Evidence:** `packages/engine-rs/src/effects/runtime.rs:405-418` breaks handler iteration when `combat_over` becomes true, while `packages/engine-rs/src/engine.rs:5117-5125` marks combat over before emitting `CombatVictory`. Java `decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java:1949-1960` calls every relic, blight, and power `onVictory` handler without such an early exit.

The first Rust handler that sees the already-ended combat can prevent all remaining victory handlers from running. This is order-dependent and affects legitimate combinations such as Burning Blood plus Face of Cleric; the problem is the dispatch contract, not either relic definition.

**Smallest repro:** ignored `EDA-001` installs Burning Blood and Face of Cleric, wins combat, and expects both healing and max-HP gain; current Rust skips the later handler.

**Proposed fix:** finish the immutable `CombatVictory` handler snapshot even when combat is over; only queue-processing rules, not event fan-out, should stop. Add order permutations and relic/blight/power family coverage.

**Effort:** S.

## EDA-002 — Card `misc` wraps at `i16`

**Evidence:** `packages/engine-rs/src/combat_types.rs:15-23` stores `CardInstance.misc` as `i16`, and `packages/engine-rs/src/effects/interpreter.rs:801-833` narrows modification results. Java `decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyDamageAction.java:21-29` operates on Java `int` card state.

Persistent card values can be modified repeatedly across a run. Narrowing a valid Java `int` to `i16` silently wraps, corrupting Rampage and any sibling mechanic using `misc`; serialization, observations, snapshots, and traces then preserve the wrong value.

**Smallest repro:** ignored `EDA-002` applies +5 to Rampage at 32,765 and expects 32,770; Rust wraps.

**Proposed fix:** migrate gameplay-significant card `misc` end-to-end to `i32`, inventory every cast/projection, and add boundary plus round-trip tests before changing the wire schema.

**Effort:** L.

## EDA-003 — Status amounts wrap at `i16`

**Evidence:** `packages/engine-rs/src/state.rs:77` stores status amounts as `i16` and `packages/engine-rs/src/state.rs:120-134` narrows `i32` writes. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/AbstractPower.java:157-164` stacks an `int`, and `decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TriplePoisonAction.java:20-24` can multiply it repeatedly.

The narrowing is systemic across player and enemy powers, not a single card bug. Legal Catalyst+ stacking can wrap a large positive Poison amount into a much smaller or negative value, changing damage, cleanup, observations, and traces.

**Smallest repro:** ignored `EDA-003` starts Poison at 9,999, applies two Catalyst+, and expects 89,991; current Rust stores 24,455.

**Proposed fix:** replace status storage and all interpreter/action intermediates with `i32`, then sweep power-specific casts and schema/snapshot encoders with positive and negative boundary tests.

**Effort:** L.

## EDA-004 — Per-floor combat RNG initialization disagrees with Java

**Evidence:** `packages/engine-rs/src/run.rs:2809-2817` constructs combat seeds with implementation-specific arithmetic, while `packages/engine-rs/src/engine.rs:183-204` adds an AI offset and aliases stream seeds. Java `decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:388-422,1737-1741` initializes/preserves the persistent streams and reseeds only monster-HP, AI, shuffle, card-random, and misc RNGs from `Settings.seed + floorNum`; potion RNG is persistent rather than reset per combat.

Even if each individual RNG implementation were correct, the streams begin from different states and the potion stream has the wrong lifetime. This guarantees later AI/card/shuffle divergence. F2 separately owns the observable symptom that 12 noncombat counters are absent from the trace surface.

**Smallest repro:** ignored `EDA-004` builds a floor combat and compares the first AI sample to Java's `seed + floor` initialization; it differs.

**Proposed fix:** introduce typed run RNG ownership, reproduce Java floor-transition reseeding exactly, pass state into combat without offsets/aliases, and assert seed plus counter zero-points for every reset and persistent stream.

**Effort:** M.

## EDA-005 — Run encounters ignore Java's seeded weighted queues

**Evidence:** `packages/engine-rs/src/run.rs:216-300` defines simplified, incomplete encounter arrays and `packages/engine-rs/src/run.rs:1656-1685` selects by monotonically rotating `index % pool.len()`. Java `decompiled/java-src/com/megacrit/cardcrawl/dungeons/Exordium.java:111-153`, `TheCity.java:89-130`, and `TheBeyond.java:86-125` build seeded weighted queues through `AbstractDungeon.java:1047-1084`.

Rust hallway and elite selection is seed-independent at a given ordinal, omits legal encounters, includes incorrect compositions (for example two rather than three Byrds in the Act 2 weak pool), and ignores Java's no-immediate/two-back-repeat rules and first-strong exclusions. The defect spans every act and directly prevents source- or trace-parity full runs.

**Smallest repro:** ignored `EDA-005` uses seed 4. Java's first `monsterRng` float is 0.45369965, which selects Jaw Worm from Exordium's four equal-weight weak encounters; Rust always selects the first rotated entry, Cultist.

**Proposed fix:** port the canonical weighted encounter names/compositions and queue generation per act, give `RunEngine` a dedicated persistent `monsterRng`, preserve Java exclusions/repeat rejection/counters, and map canonical encounter IDs through `MonsterHelper`.

**Effort:** L.

## EDA-006 — Nested runtime events are lost during dispatch

**Evidence:** `packages/engine-rs/src/engine.rs:258-266` uses `mem::take` to move the active effect runtime out of the engine while dispatching. A handler that emits another event therefore reaches a default/empty runtime. Java death flow in `decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java:925-937` synchronously notifies the installed relic/power set; `relics/MercuryHourglass.java:27-32` damage can therefore trigger `relics/GremlinHorn.java:45-51` on death.

The runtime is reentrant in Java but temporarily absent in Rust. Any handler-caused death, exhaust, power removal, or other nested event can lose downstream subscribers; Mercury Hourglass killing the first enemy and Gremlin Horn failing to grant energy/draw is one deterministic instance of a whole trigger family.

**Smallest repro:** ignored `EDA-006` starts combat with Mercury Hourglass and Gremlin Horn against a 3-HP enemy and expects the Horn's energy/draw after the Hourglass kill; neither arrives.

**Proposed fix:** make dispatch reentrant with an explicit event queue or runtime-owned frame stack, define install/remove visibility during nested emission, and add a trigger-family matrix for death, exhaust, removal, and victory.

**Effort:** M.

## EDA-007 — Devotion invents a Mantra remainder when Mantra is absent

**Evidence:** `packages/engine-rs/src/engine.rs:4781-4787` applies generic Mantra threshold/remainder logic for a large Devotion grant. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevotionPower.java:35-41` checks for absent Mantra and amount at least ten, then queues only `ChangeStanceAction`; it applies Mantra only in the other branch.

Java's missing-power fast path deliberately enters Divinity without constructing Mantra at all. Rust instead applies its generic threshold/remainder rule, so amounts above ten invent leftover Mantra and accelerate a later stance entry.

**Smallest repro:** ignored `EDA-007` gives Devotion 12 to a player without Mantra and expects Divinity with no spurious remainder under the Java action sequence; Rust leaves Mantra 2.

**Proposed fix:** implement Java's explicit “Mantra absent and amount at least ten” direct stance-change branch; use normal Mantra stacking otherwise, and test 9, 10, 12, existing-Mantra, and repeated-trigger cases.

**Effort:** S.

## EDA-008 — Necronomicon copy bypasses card-use semantics

**Evidence:** `packages/engine-rs/src/engine.rs:2890-2910` directly reapplies the copied card effect. Java `decompiled/java-src/com/megacrit/cardcrawl/relics/Necronomicon.java:56-73` queues a copied card through the action manager; `decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java:188-277` and `actions/utility/UseCardAction.java:30-88` perform the normal counters and power/relic/card hooks.

Direct effect replay can match damage while omitting played-card counters, relic/power callbacks, exhaust/discard handling, and nested copy guards. Damage-only tests therefore overstate coverage.

**Smallest repro:** ignored `EDA-008` plays a qualifying attack with card-play/block hooks and expects two complete uses; current Rust reports only one played-card/hook pass.

**Proposed fix:** represent the Necronomicon copy as a queued free-to-play card instance through the canonical use pipeline with the Java copy guard, then test counters, hooks, destination, and X-cost/free-play siblings.

**Effort:** M.

## EDA-009 — Enemy Ritual strength is applied at the wrong trigger

**Evidence:** `packages/engine-rs/src/powers/defs/enemy.rs:20-56` registers enemy Ritual on `EnemyTurnStart`. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/RitualPower.java:19,46-54` uses `atEndOfRound` and a `skipFirst` flag.

Applying Strength at turn start changes the Cultist's first Dark Strike and shifts every later round relative to Java. The apparent final strength can look plausible while damage timing is wrong.

**Smallest repro:** ignored `EDA-009` advances a fresh Cultist through its first Dark Strike and expects 6 damage followed by Strength 3; Rust attacks for 9.

**Proposed fix:** model Ritual as an owner end-of-round handler with `skipFirst` state, and cover application before/after the owner's turn plus multiple owners.

**Effort:** S.

## EDA-010 — Static Discharge bypasses the canonical orb pipeline

**Evidence:** `packages/engine-rs/src/combat_hooks.rs:398-459` performs a direct/raw Lightning channel effect when the orb slots are full. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/StaticDischargePower.java:29-34` queues `ChannelAction`, which uses the normal Lightning/orb-slot/evocation and Electrodynamics targeting pipeline.

The shortcut omits canonical full-slot eviction, channel counters/hooks, focus/orb state, and multi-target behavior. It can look correct in a single-target damage assertion while producing different causal state.

**Smallest repro:** ignored `EDA-010` fills orb slots, enables Electrodynamics, takes unblocked attack damage, and expects the oldest orb to evoke plus one canonical Lightning channel and all-target damage; Rust performs only a raw first-target effect.

**Proposed fix:** enqueue the standard channel operation and remove the special raw path; test empty/full slots, Electrodynamics, focus, evoke hooks, and multiple discharge stacks.

**Effort:** M.

## EDA-011 — Spore Cloud fires after the final enemy ends combat

**Evidence:** `packages/engine-rs/src/engine.rs:3788-3793` applies Spore Cloud's Vulnerable unconditionally on death. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/SporeCloudPower.java:36-43` returns when the room's `isBattleEnding()` is true; `rooms/AbstractRoom.java:640-647` delegates that state to `monsters/MonsterGroup.java:91-97` when needed.

Killing the last Fungi Beast should not add a post-combat player debuff. The current outcome contaminates combat-end snapshots and any victory handler that inspects powers.

**Smallest repro:** ignored `EDA-011` kills the sole Fungi Beast and expects no Vulnerable; Rust applies 2.

**Proposed fix:** gate the death effect on the canonical “other monsters remain” condition before applying the debuff, with sole-enemy and multi-enemy ordering tests.

**Effort:** S.

## EDA-012 — Combat shuffle uses the wrong RNG algorithm and accounting

**Evidence:** `packages/engine-rs/src/engine.rs:414,4456,4548` calls Rust `SliceRandom::shuffle` directly on `StsRandom`, consuming raw generator words without the Java counter contract. Java `decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java:550-555` consumes one `shuffleRng.randomLong()` and then calls `Collections.shuffle` with a new `java.util.Random(seed)`. A Java-compatible helper already exists at `packages/engine-rs/src/seed.rs:214-249`.

The shuffle differs in three linked ways: random algorithm, stream consumption shape, and exposed counter ticks. Thus draw order and every later shuffle-stream counter diverge even if the initial seed matches.

**Smallest repro:** construct a fixed discard pile and `shuffleRng` seed, run the Java two-stage algorithm and the engine reshuffle, and assert both pile order and exactly one outer-stream counter tick; current Rust fails both contracts.

**Proposed fix:** draw one canonical `randomLong`, pass it to `java_util_shuffle`, and route every combat shuffle/reshuffle call through that operation; add sizes 0, 1, 2, and representative piles.

**Effort:** M.

## EDA-013 — Player Poison ticks on the wrong owner's phase

**Evidence:** `packages/engine-rs/src/engine.rs:2184-2193` processes player Poison at player turn end. Java `decompiled/java-src/com/megacrit/cardcrawl/powers/PoisonPower.java:58-64` triggers at the poisoned owner's turn start.

The generic power model is incorrect for a poisoned player: damage, death timing, artifact/power interactions, and action ordering move by a full phase. No currently identified vanilla Watcher path applies Poison to the player, so this is a confirmed semantic defect with a current reachability caveat, not a claimed full-run divergence.

**Smallest repro:** place Poison on the player, advance end turn and the next owner turn start, and assert no end-turn loss followed by start-turn damage/decrement; `packages/engine-rs/src/tests/test_integration.rs:1306-1347` currently enshrines the opposite timing.

**Proposed fix:** make power ticks owner/event-driven, replace the stale synthetic test, and retain the scenario as a generic engine invariant even if vanilla reachability remains absent.

**Effort:** S.

# P1 — performance, batching, and contracts

## EDA-014 — `CombatSnapshotV1` is not a deterministic resume snapshot

**Evidence:** `packages/engine-rs/src/training_contract.rs:526-559` defines only one RNG tuple; capture at `:663-733` records only `engine.rng`; restore at `:740-808` recreates other streams from seed zero and restores neither move history nor effect-runtime, pending-choice, program, or event-queue state.

The type can reconstruct a shape that admits legal actions but cannot guarantee the same future. Treating it as a resume/checkpoint boundary makes rollouts, distributed workers, and parity replay nondeterministic.

**Smallest repro:** snapshot after consuming different AI/shuffle RNG counts and installing hidden effect state, restore, apply the same action, and compare intent, piles, legal actions, counters, and events; the futures differ.

**Proposed fix:** define a causal `CombatSnapshot` separate from observation/diagnostics, include every RNG and queued/runtime state that affects the future, and prove round-trip continuation byte-equivalence.

**Effort:** L.

## EDA-015 — Effect runtime is rebuilt and cloned on every action/event

**Evidence:** `packages/engine-rs/src/engine.rs:525-530` rebuilds the runtime unconditionally; `packages/engine-rs/src/effects/runtime.rs:272-379` rescans definitions, `:252-254` clones handlers, and `:386-418` clones handler vectors for dispatch. Relic/potion resolution also repeatedly scans and normalizes IDs.

This design supports tests that mutate internals directly, but it converts immutable content into per-step heap work and complicates reentrancy. For N-state MCTS, runtime definitions should be shared and state-local instances compact.

**Smallest repro:** allocation-count one effect-heavy turn and compare handler-definition scans/clones with an equivalent precompiled registry path.

**Proposed fix:** compile immutable handlers once in `ContentDb`, keep only instance state/subscriptions in each combat, introduce reusable dispatch scratch, and make fixture mutation explicit rather than rebuilding production state.

**Effort:** L.

## EDA-016 — Scalar run `step` eagerly builds an RL-rich result

**Evidence:** `packages/engine-rs/src/run.rs:1464-1521` validates legality before/after, constructs decisions/context/events, and copies an observation for every `step_with_result`. Search at `packages/engine-rs/src/search.rs:911-919` consumes only reward in this path.

Core simulation pays for consumer projections even when rollout search needs only mutation, terminal/reward, and perhaps emitted deltas. This boundary makes allocation reduction and alternative observation batching harder.

**Smallest repro:** benchmark identical run actions through the current result builder and a temporary in-place internal path while counting allocations and bytes copied.

**Proposed fix:** expose `step_in_place` with optional event sink/scratch; implement the rich adapter as an outer projection and batch observations only at policy-evaluation boundaries.

**Effort:** M.

## EDA-017 — PUCT has no batched or parallel leaf boundary

**Evidence:** `packages/engine-rs/src/search.rs:158-171` stores full engines/nodes; `:280-289` accepts one mutable evaluator; `:328-336` performs one simulation/evaluation per loop; expansion at `:460-507` clones parent state into children. No `step_many` or indexed batch evaluator exists.

The implementation serializes the most expensive policy boundary and deep-clones per child. It cannot exploit one immutable registry and N independent states or an accelerator-friendly leaf batch while guaranteeing stable index ordering.

**Smallest repro:** benchmark K rollouts with a fixed deterministic evaluator and report scalar clone/eval time versus a prototype that gathers K leaves and evaluates one ordered batch.

**Proposed fix:** separate tree metadata from compact causal states, add deterministic indexed leaf batches and in-place stepping, parallelize independent simulations with state-local RNG/scratch, and regression-test scalar-versus-1/2/N equality.

**Effort:** L.

## EDA-018 — State clones include diagnostics and avoidable heap state

**Evidence:** `packages/engine-rs/src/engine.rs:546-573` clones the runtime, pending/program queues, event log, cards, enemies, strings, and histories. Criterion measured representative clones at 568–776 ns; `has_relic` appears at 71 source call sites.

The baseline is already sub-microsecond, but MCTS multiplies it by nodes and workers. Diagnostic logs, immutable definitions, repeated strings, and reusable scratch do not belong in every causal clone.

**Smallest repro:** benchmark clones with effect-heavy enemies, deep move history, populated queues/logs, relics, and potions while counting heap allocations; current benches do not stress all of these.

**Proposed fix:** split causal snapshot from diagnostics/scratch, intern typed IDs, share immutable content, bound/compact histories, and set representative clone/allocation gates before optimizing.

**Effort:** M.

## EDA-019 — Note for Yourself is process-global mutable gameplay state

**Evidence:** `packages/engine-rs/src/run.rs:27-43` declares `OnceLock<Mutex<String>>`; production writes at `:4181-4184` and reads at `:7014-7018` and `:7410-7412`.

Two simulations in one process can influence each other's event outcome through a lock outside their snapshots. This violates deterministic independent rollout state even though the registries themselves are immutable.

**Smallest repro:** initialize two roots with different stored cards, interleave the event choices on two threads, and show that one root observes the other's process-global value.

**Proposed fix:** supply immutable profile/save inputs to each root and emit explicit profile-update outputs outside rollout state; prohibit mutable gameplay globals with an architecture check.

**Effort:** S.

## EDA-020 — Relics have three diverging representations

**Evidence:** `packages/engine-rs/src/run.rs:338-390` stores relic strings, skipped-serialization `RelicFlags`, and runtime/counter state; `:4373-4397` manually rebuilds flags, while combat transfer at `:2804-2817` and later code copy only selected pieces. Observation paths read different counter sources.

Capability flags, counters, and runtime hidden state can disagree after deserialize, run-to-combat transfer, or clone. Hot hooks also repeatedly compare strings instead of resolving one stable instance.

**Smallest repro:** serialize/deserialize a run with a counter/stateful relic, enter combat, and compare `has_relic`, flag behavior, runtime hidden value, trace counter, and observation token; at least one representation is lost or stale.

**Proposed fix:** use canonical typed relic instances with one state/counter payload; derive caches on load and project trace/obs from that single source.

**Effort:** L.

## EDA-021 — PyO3 is an unconditional dependency of simulation core

**Evidence:** `packages/engine-rs/Cargo.toml:12-24` enables PyO3 unconditionally; `packages/engine-rs/src/lib.rs:43` and core action/state/engine modules import binding types/attributes.

Core compilation, tests, and benchmarks inherit Python/framework linkage even though rollout state and mechanics do not require Python. Dependency direction also lets obsolete consumer shapes leak into core types.

**Smallest repro:** `cargo check --manifest-path packages/engine-rs/Cargo.toml --all-targets --no-default-features` cannot produce a genuinely Python-free core because no such feature boundary exists.

**Proposed fix:** extract a zero-Python core crate or enforced module, put bindings/obs/search adapters behind opt-in features/crates, and add a dependency-direction/no-default build gate.

**Effort:** M.

## EDA-022 — Boundary schemas reject additive evolution

**Evidence:** observation functions are named v2 while exporting version 4 at `packages/engine-rs/src/obs.rs:584-611`; training contracts use exact version constants; `packages/engine-rs/src/trace.rs:49-60` rejects anything except version 1. Adding `power_cards_played_this_combat` already makes the old strict Python constructor fail.

Exact struct construction and monolithic integer versions turn harmless additive fields into breaking changes. The future trainer and monitor are being rebuilt, so this is the moment to define a durable boundary rather than preserve the obsolete one.

**Smallest repro:** decode a current payload with a consumer that knows the prior minor schema but projects only known fields; today the extra field/type constructor fails instead of being tolerated.

**Proposed fix:** use an envelope such as `{schema:{name,major,minor}, capabilities, payload}`, tolerate unknown minor fields, default missing additive fields, reject incompatible majors, and keep core state separate from wire DTOs.

**Effort:** M.

## EDA-023 — Trace replay cannot express a full run

**Evidence:** `packages/engine-rs/src/trace.rs:706-733` maps only a subset of `RunAction` and rejects reward, event, campfire, shop, key, and other transition choices. F7 owns already-known action-schema nits, and F4 owns the Neow mapping mismatch.

The only end-to-end oracle cannot drive the decisions needed for a Watcher A0 run through Heart. This is a vocabulary/adapter gap, not a mechanics finding.

**Smallest repro:** serialize and replay one action of each missing run decision variant; the mapper returns unsupported before simulation.

**Proposed fix:** define one stable typed action vocabulary shared by script validation and Java/Rust adapters, complete every run phase, and add one compatibility fixture per variant.

**Effort:** M.

## EDA-024 — Hot causal state uses strings and linear lookup

**Evidence:** `packages/engine-rs/src/state.rs:141-155,304-340` and `packages/engine-rs/src/run.rs:338-355` own content names as `String`; `has_relic` has 71 call sites, and registry normalization allocates/scans.

Names are appropriate at serialization/debug boundaries but waste clone bandwidth and make typo/alias behavior part of hot mechanics. The same identity can appear spaced, unspaced, normalized, or as a capability flag.

**Smallest repro:** allocation-profile a relic-heavy damage turn and a state clone; count string clones, normalizations, and linear comparisons.

**Proposed fix:** assign stable typed content IDs and instance IDs in immutable `ContentDb`, store compact IDs in causal state, and convert names only at adapters.

**Effort:** L.

## EDA-025 — Observation code depends on private effect internals

**Evidence:** `packages/engine-rs/src/obs.rs:584-611` defines a fixed legacy shape while `:825` and `:837` read hidden effect values directly; run adapters at `packages/engine-rs/src/run.rs:676` and `:1496` expose that projection.

The core/runtime cannot change its private state layout without changing an RL-specific vector, and the “v2” name/version mismatch obscures compatibility. Because both consumers are being rebuilt, observation encoding should be downstream of a public causal DTO.

**Smallest repro:** replace one hidden runtime representation with an equivalent typed public counter and observe that the fixed encoder must still reach into the old internal path.

**Proposed fix:** expose typed snapshot/events sufficient for projections, move vector/token construction to a consumer adapter, and version that adapter independently.

**Effort:** M.

## EDA-026 — Parallel deck vectors can attach persistent state to the wrong duplicate

**Evidence:** `packages/engine-rs/src/run.rs:339-351` stores `deck: Vec<String>` beside `deck_card_states`, and reconciliation at `:616-638` searches by base name rather than stable instance identity.

With duplicate cards, upgrades/removals/reordering can cause `misc` or other persistent state to migrate to the wrong copy. Correct lengths do not prove identity preservation.

**Smallest repro:** create two same-base cards with different upgrade/`misc` state, reorder/remove one through a run transition, reconcile, and assert each original instance retains its state.

**Proposed fix:** replace both vectors with one canonical deck of stable card instances and serialize instance IDs/state together.

**Effort:** M.

# P2 — test health and abstraction debt

## EDA-027 — `RunEngine` is a phase god-struct

**Evidence:** `packages/engine-rs/src/run.rs:758-811` combines stable run state, combat, reward, shop, event, Neow, map, and numerous `pending_*` booleans/sidecars in one cloneable struct.

Invalid combinations are representable and every new relic/event continuation expands the root type. Clones pay for inactive phases, while transition ownership and serialization invariants remain implicit.

**Smallest repro:** construct or deserialize two mutually incompatible pending flags and observe that the type system cannot identify the single active decision frame.

**Proposed fix:** split stable `RunSnapshot` from an exhaustive phase enum with typed continuation payloads, migrating one phase family per source-tested commit.

**Effort:** L.

## EDA-028 — Twenty-four cards retain unrestricted `complex_hook` escape hatches

**Evidence:** 24 `complex_hook: Some(...)` definition occurrences—12 base/upgraded card pairs—remain under `packages/engine-rs/src/cards/`; `packages/engine-rs/src/effects/types.rs` gives the hook broad mutable `CombatEngine` access, and `card_effects.rs` executes it beside declarative ops.

Some mechanics may be inherently imperative, but the current interface does not distinguish that from definitions that simply have not been modeled. It bypasses typed operation invariants and makes family-level auditing harder.

**Smallest repro:** generate an inventory asserting the exact allowlist of 24 hooks and fail on growth; classify each by required capability and test its production path.

**Proposed fix:** migrate expressible hooks to typed ops, replace the remainder with narrow capability-specific commands, and retain a reviewed non-growth allowlist.

**Effort:** M.

## EDA-029 — Several tests encode or fail to observe the Java behavior

**Evidence:** `packages/engine-rs/src/tests/test_entity_runtime.rs:971-998` expects sole-enemy Spore Cloud Vulnerable; `test_integration.rs:1306-1347` expects player Poison at end turn; `test_integration.rs:2101-2120` says Like Water block cannot be observed and only asserts status even though stronger card tests now exist. `engine.rs:5900-5912` tests shuffle zone sizes but not order/counters, while Necronomicon wave tests assert damage only.

These are distinct examples of the F1 failure pattern: a green test may assert draft behavior, a weak proxy, or only one visible consequence while the production pipeline is wrong. The finding is the identified test families, not a claim that every direct state setup is invalid.

**Smallest repro:** apply the cited Java-derived expectations or a mutation that removes counters/hooks/order; each named test either fails against Java or stays green despite the behavioral regression.

**Proposed fix:** replace wrong expectations when the matching P0 lands, add engine-path companions that observe causal state and RNG, and require Java/golden citations for gameplay oracles.

**Effort:** M.

## EDA-030 — The test corpus is private-coupled and physically duplicated

**Evidence:** `packages/engine-rs/src/tests/` has 213 `test*.rs` files (215 Rust files including `mod.rs` and `support.rs`) and 60,616 lines; 19 files reference hidden-effect access and more than 700 assignments directly mutate `engine.state.*`. `test_reward_relic_runtime_wave3.rs` is 5,495 lines, `test_integration.rs` 3,254, and `test_cards_watcher.rs` 2,757.

Direct fixture setup is sometimes legitimate, but the wave organization hides behavioral ownership and makes it difficult to know whether a private-helper test is the sole evidence for a production path. Duplicate-looking tests cannot be deleted safely without a fault-detection map.

**Smallest repro:** classify a sample by behavior/source/event path and inject one mutation in dispatch, RNG, and amount; record which nominally related tests fail and which remain green.

**Proposed fix:** build a source-to-behavior coverage index, add engine-path companions before hiding internals, and consolidate only when a retained test detects the same mutation.

**Effort:** L.

## EDA-031 — The crate's public surface is not an intentional boundary

**Evidence:** a source scan finds 253 externally public modules and roughly 2,194 externally public declarations under `packages/engine-rs/src`, dominated by per-content modules and organically exposed helpers. No architecture/API-surface test constrains growth.

The public namespace exposes implementation details well beyond construct/step/snapshot/legal/trace entry points, making future refactors appear breaking and allowing adapters to depend inward.

**Smallest repro:** generate rustdoc/public-item inventory and compare it with the desired core API; most card modules and helper functions have no external boundary role.

**Proposed fix:** define facade crates/modules, default internals to `pub(crate)` or private, expose content through `ContentDb`, and add a checked public-API/dependency allowlist.

**Effort:** M.

## EDA-032 — The verification ledger omits whole behavior domains

**Evidence:** `docs/goal/ledger.json` contains exactly 370 card, 68 monster, 43 potion, and 186 relic rows; it has no event, Neow, map/generator, or power rows and no machine-readable Watcher reachability/`covered_by` data.

“667 verified” proves the completed four-kind sweep, not the GOAL definition for a full run. Run transitions, generators, events, and shared power semantics have no equivalent source-citation/evidence ledger.

**Smallest repro:** query the ledger for `event`, `power`, `neow`, and `map` kinds and for reachability metadata; all are absent.

**Proposed fix:** extend extraction deterministically with class rows and named generator invariants, preserve existing statuses, compute reachability, and require Java citation plus engine-path test before each new flip.

**Effort:** L.

## EDA-033 — One smoke golden cannot certify the target corpus

**Evidence:** only `data/traces/scripts/smoke-neow-floor1.json` and its one protected golden exist, while `docs/goal/GOAL.md` and `docs/goal/UNITS.md` target roughly ten complete A0 runs plus seed `1776347657`. F6 separately owns omitted comparison fields.

The oracle plumbing exists, but a single short trajectory cannot expose the combinatorial ordering, RNG, event, reward, boss, Act 4, and long-run state interactions required by the finish gate.

**Smallest repro:** list scripts/goldens and compare their exercised floors/content against the target coverage set; only the smoke case is available.

**Proposed fix:** complete trace action/state contracts, write coverage-selected request scripts, have a human A/B mint immutable goldens, and burn down earliest unmasked divergences.

**Effort:** L plus attended minting.

## EDA-034 — Benchmarks omit allocation and batch-readiness gates

**Evidence:** `packages/engine-rs/benches/combat_bench.rs` and `real_world_bench.rs` measure scalar turns, legal actions, and cloning, but not allocations, effect-heavy dispatch, run decisions, `step_many`, thread scaling, or scalar-versus-batch determinism.

The current nanosecond/microsecond numbers are a baseline, not a workload acceptance test. Optimizations can shift allocations or cloning to unmeasured states, and parallel implementations can be fast but nondeterministic.

**Smallest repro:** add measurement-only fixtures for deep logs/queues, effect-heavy multi-enemy deaths, run phases, and indexed 1/2/N batches; no current benchmark reports those dimensions.

**Proposed fix:** publish representative allocation counts, scalar/batch throughput, clone/legal costs, and deterministic schedule checks in a stable performance job.

**Effort:** S.

## EDA-035 — The completed sweep has no approved successor process contract

**Evidence:** repository facts were refreshed in `AGENTS.md`, `README.md`, and `CLAUDE.md`, but the external `~/.codex/prompts/goal.md` still describes the completed row-flipping loop. Protected canonical `docs/goal/INVENTORY.md` and `UNITS.md` retain stale deleted-doc links/counts. `scripts/play.sh` remains referenced by protected `docs/goal/TOOLING.md` but targets the historical untracked EVTracker flow.

The old loop is complete, while the next mission requires corpus/oracle closure, run RNG topology, missing ledger domains, P0 repro repair, and boundary extraction. Factual staleness was safe to fix; redefining `/goal` or protected canonical plans requires explicit human approval.

**Smallest repro:** compare `scripts/ledger.sh status` (zero unverified) with the external prompt's “next unverified row” instruction and resolve its deleted links.

**Proposed fix:** after human review, replace the prompt with the proposal below, refresh protected inventories/units, and either deliberately restore/rebuild the referenced play UI or retire the reference and script together.

**Effort:** S for contract approval/update; larger UI work is separately scoped.

## Proposed `/goal` successor — HUMAN APPROVAL REQUIRED

> Close the highest-priority ready simulator-completion gap, not merely the next legacy ledger row. Read `AGENTS.md`, `docs/goal/GOAL.md`, `docs/goal/FINDINGS.md`, this register, and `docs/work_units/sim-completion-map.md`. Prefer confirmed P0 repros, then run RNG/core-oracle dependencies, then new reachable ledger domains. Ground every behavior change in decompiled Java, turn every confirmed engine finding into an engine-path regression, and keep behavioral fixes separate from architecture work. For ambiguous real-game evidence, write a trace request and continue offline; never launch the game. Run the full library suite after every commit and the relevant frozen-golden diff where available. Update the EDA entry/map when a unit closes. Stop only when the selected unit's exit tests pass, or when a named external/human dependency is recorded with the next ready unit identified.

## Leads not run down

These are hypotheses, not findings: stance-change feedback loops across multiple simultaneous powers; sibling X-cost/free-play/copy combinations beyond the confirmed Necronomicon path; Artifact/Intangible ordering outside already source-verified cases; and recursive exhaust/power-removal chains beyond the confirmed runtime-reentrancy reproducer. Each should begin with a Java method-family inventory and a production-path test, not with an EDA ID.

## Executed cleanup summary

| Commit | Area | Accounted result |
|---|---|---|
| `695df979` | Dead code | Removed superseded helper runtimes and seven zero-caller effect modules; 40 tests removed only with their deleted subjects. |
| `6a2f2d07` | Dead code | Removed the potion compatibility executor and state-only shims; 66 subject-only tests removed. |
| `1961ea8c` | Docs | Deleted pre-sweep audit/quarry/research material; git history remains the archive. |
| `92ae11ba` | Docs | Refreshed live parity source indexes, reset the current deviation register, and added the F8 cross-reference. |
| `b84bda57` | Process/tooling | Removed dead scripts/hooks/config and refreshed factual repository guidance without redefining the loop contract. |

The dependency graph, layer exit tests, and owner-sized next-two-month queue are in `docs/work_units/sim-completion-map.md`. It sequences combat P0s, pure-core/state ownership, the 13-stream run RNG model, missing content domains, trace-v2/corpus minting, batch stepping, test-strength repair, and final B3/B4/B5 closure.
