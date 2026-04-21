# Pre-merge Triage — PR #138 Parity-Audit-Fleet — 2026-04-21

**Inputs:** 9 parallel read-only audit subagents (`scratch-2026-04-21/*.md`).
**Method:** three-way compare Java decompile ↔ Rust engine ↔ Rust tests.
**Commits audited:** `55e77c89..d16cdba2` (7 commits, +4170 / −547, 24 files).
**Register baseline:** D1–D159.

## Bottom line

- **4 novel P0 merge-blockers** in training-contract determinism (F1/F2/F3/F4).
- **1 structural P1** (dead dispatch tables confirmed dead — live casualty D70).
- **2 novel P2 register rows** (D160 BookOfStabbing A18+, D161 Mugger turn-2).
- **6 novel P1 deviations** across powers/combat hooks (Flight, Angry, Buffer×2, Intangible, Thorns).
- **~230 Stage B findings still un-registered** — systemic bitrot risk, not a merge-blocker.
- **Zero new Spiker-class double-apply bugs** surfaced globally.

**Recommendation:** block merge on F1–F4; everything else is register-and-defer.

---

## Merge-blockers (P0)

These silently break determinism properties the engine already claims. Training
artifacts from this branch would drift from "true" rollouts once snapshotted.

| ID | Where | One-liner |
|---|---|---|
| **F1** | `training_contract.rs:523-555,658-728,735-803` | `CombatSnapshotV1` drops `ai_rng` state; roundtrip re-seeds from 0 → intent sequence diverges silently |
| **F2** | `training_contract.rs:506-521,737-753` | `EnemySnapshotV1` drops `move_history` → every `last_move(X)` guard returns false post-roundtrip, collapsing anti-repeat branches |
| **F3** | `search.rs:1087-1138` | `combat_state_hash` ignores `rng` + `ai_rng` → MCTS transposition merges states with divergent enemy intent streams |
| **F4** | `training_contract.rs:1522-1545` | Existing roundtrip test asserts player/hand/potion/relic surface only; never asserts RNG/history parity. Bug is uncaught today |

**Fix cost estimate:** F1–F4 are ~80 LOC + one new test. Single test reproduces
all three: snapshot → `combat_engine_from_snapshot` → two `roll_next_move` calls
→ assert `move_id` equality.

**Merge guidance:** block merge if PR #138 claims replay/search determinism
(it does — Stage D commit body explicitly cites `test_ai_rng_parity.rs`).
Otherwise, merge as-is and register F1–F3 as post-merge P0s.

---

## Novel P0 coverage gap (not merge-blocking)

| Where | One-liner |
|---|---|
| `tests/test_bosses.rs:512-515` + missing | Heart `DEBILITATE` turn-0 init asserted, but no test exercises `apply_debuff_from_enemy` path through `combat_hooks::do_enemy_turns` — a regression where the intent runs without landing WEAK/VULN/FRAIL would pass init and fail only in integration |

---

## Structural P1 (register bump recommended)

**Dead dispatch tables** — `powers::process_start_of_turn`,
`process_end_of_turn`, `process_end_of_round` at
`powers/buffs.rs:620,704,752`. Callers: **only tests**
(`buffs.rs:1061,1092,1100,1117,1120,1234` + `test_cards_defect.rs:606+`).
Engine inlines equivalent logic at `engine.rs:1085/1304/1558-1584` for the
watched powers, but:

- **Live casualty: D70 Equilibrium decrement** — `decrement_equilibrium`
  is only called from `buffs.rs:524` (inside the dead
  `process_end_of_turn`). Grep confirms no other call sites. D70 is
  therefore not "unverified" — it is **confirmed dead**. Bump D70 to P1.
- Root cause of a cluster of Stage B findings: WS7–WS11, D89 Fasting,
  D111 pre-draw/post-draw split all trace back to this.
- Recommendation (separate cleanup PR): either wire the dispatch tables
  into `start_player_turn`/`end_turn`, or delete them. Leaving them
  actively misleads new contributors.

---

## Novel P1 deviations (register as D162–D167)

| Proposed ID | File:line | One-liner | Source |
|---|---|---|---|
| D162 | `engine.rs:2479-2485` | **Flight** has no `atStartOfTurn` restoration of `storedAmount`; Byrds/BanditPointy/Darkling permanently degrade across turns (~2× player damage upside) | powers-relics |
| D163 | `combat_hooks.rs:578-583` | **Angry** fires on Poison/Thorns/HP_LOSS without Java's `type != HP_LOSS && type != THORNS` gate; Red Louse A17+ over-triggers | combat-hooks |
| D164 | `combat_hooks.rs:162-166` | **Buffer** `continue` skips Thorns retaliation — Java's `ThornsPower.onAttacked` fires regardless of Buffer absorption | combat-hooks |
| D165 | `engine.rs:1578-1583` + Intangible apply sites | **Player Intangible** drops one turn early — no `justApplied` flag set in `add_status(sid::INTANGIBLE)` call sites, Java sets `justApplied=true` in IntangiblePower constructor; matters for Apotheosis / Wraith Form | combat-hooks |
| D166 | `combat_hooks.rs:240-251` | **Thorns retaliation** doesn't cap at 1 against an attacker with INTANGIBLE; Nemesis start-of-turn Intangible = 1 is the live case | combat-hooks |
| D167 | `combat_hooks.rs:160-166` | **Buffer** no `damageAmount > 0` guard — 0-damage attacks (debuff-only enemy moves, dagger-spray-into-intangible) eat a Buffer stack | powers-relics |

All six are **already known** in Stage B reports (P4/P11 powers audit) or
implicit in the dead-dispatch cluster, but none have D-numbers yet.

---

## Novel P2 deviations (register as D168–D169)

| Proposed ID | File:line | One-liner | Source |
|---|---|---|---|
| D168 | `enemies/act2.rs:233-255` | **BookOfStabbing** A18+ `stabCount` increment on BigStab path missing (Java L128-146 increments on both Stab AND BigStab branches at A18+; Rust bumps only on Stab) | enemy-ai |
| D169 | `enemies/act2.rs:44-59` | **Mugger** turn-2 collapses Java's 50/50 Escape-block / BigSwipe into deterministic BigSwipe; also fabricates SmokeBomb move; A17+ escape-block +6 bump missing | enemy-ai |

---

## AI-RNG-wiring P1s already Stage-D DEFERRED (confirmation)

Per ai-rng-wiring audit — all known, all deferred:

| ID | Kind |
|---|---|
| F5 | PRNG algorithm mismatch (Rust xorshift128+ vs Java `java.util.Random`) |
| F6 | `create_enemy` skips Java's init `rollMove()` → stream offset by N_enemies |
| F7 | Recursive sub-rolls collapsed to deterministic fallback (JawWorm/TimeEater/Darkling/WrithingMass/Reptomancer/GremlinLeader/ShelledParasite) |
| F8 | Champ mid-turn re-roll double-pushes `move_history` |
| F9 | WrithingMass Reactive re-roll skips `ai_rng` + `move_history` |

These are register-and-defer per Stage-D narrative — no action in this PR.

---

## Training-infra findings

**Verdict: yellow.** No P0s. Nothing will corrupt data. 15 total findings
(4 P1, 11 P2).

| Sev | Where | One-liner |
|---|---|---|
| P1 | `stage2_pipeline.py:442` | `synthetic_target = round(total_cases * 0.84)` — the single most consequential corpus ratio stayed hardcoded despite Stage E's "surface constants" claim |
| P1 | `stage2_pipeline.py:556` | `multiplier = 1 if pass==0 else 2 if pass==1 else 4` — collection-pass visit-scaling ramp hardcoded |
| P1 | `training.sh:47-66` | `smoke` subcommand called directly (not via `launch --with-smoke`) has no pid/lock guard against a live overnight still writing to `logs/active` |
| P1 | `cli.py:701` / `stage2_pipeline.py:45` | `COLLECTION_WORKER_COUNT = 1` reported as truth; should link to the PyO3 single-thread constraint that forces it, not just state as fact |

P2s (11): smoke dir 1-sec collision, uv run exit-code trap fragility,
non-atomic pid write, non-default --pid-file escape hatch, missing
disk-space preflight, `ROOM_KIND_CORPUS_WEIGHTS` normalization contract,
`_ENCOUNTER_POOL_SHUFFLE_SEED = 20260421` magic date, `buckets = 8`
hidden knob, hardcoded --target-cases 24 in smoke, undocumented archive /
smoke / --with-smoke subcommands, `max(12, ...)` HP floor unexplained.

---

## Changelog meta-concerns

1. **Stage D commit (`9984bd86`) title overstates coverage** — "Act 1 (10 enemies)" is misleading; ~10 enemies still discard `_num` (GremlinWizard, Lagavulin, Sentry, Guardian, Hexaghost, SlimeBoss, BanditLeader/Bear, FungiBeast, SpikeSlime_{M,L}). Stage G (D146-D157) correctly registers the gaps.
2. **Stage D introduced the Spiker bug that `d16cdba2` later fixes** — `act3.rs` roll-time `sid::THORNS += 2` was added in `9984bd86`, double-applied with `combat_hooks`' `mfx::THORNS`, with the Stage D test asserting the bug as "parity" (`THORNS == 5`). A real parity regression introduced and cleaned up in the same PR. Worth calling out explicitly.
3. **D140 / `writhing_mass_reactive_reroll` is inert production code** — declared `pub fn` with detailed comment, but only called from `test_enemy_ai.rs:720`. Stage D commit could be read as "implemented"; in reality it is dead.
4. **Register D1 "fixed for core branch logic"** — subjective. Consider linking to D146-D157 from D1 so readers know the ~10 enemies that still ignore `num` are deliberate scope cuts.
5. **Stage E "12-worker" is a docs fix, not performance fix** — prior `worker_count=12` was a manifest lie; `COLLECTION_WORKER_COUNT=1` is the correct answer because PUCT collection is single-process. Commit body should clarify.
6. **Duplicate near-duplicate tests pre-exist the PR** (D159 deferred). Several Stage G findings (D129/D130 pre-Stage-F) hid inside tests that encoded the bug as parity. The 35% LOC reduction in D159 would reduce the chance of future hidden drift.

---

## Register-and-defer: 230 Stage B findings never landed

Per stage-b-crossref audit: of ~302 Stage B findings, only ~68 became
register rows (D88-D128 + D132-D159). Net ~230 findings live **only**
inside per-area reports. Breakdown of the un-registered tail:

- **Enemy AI Act 1 tail** — ~40+ E*A1 rows (AcidSlime_L split, Cultist ascension, Hexaghost A19+ inferno scaling, GremlinNob Bellow cap, JawWorm upgraded ATTACK curve, Louse HP pools, Looter steal-gold pipeline, etc.)
- **Enemy AI Act 2/3/4 tail** — ~25 rows (Centurion/Healer timing, Looter vs Mugger Smoke Bomb split, Collector A19+ scaling constants `strAmt=5/megaDebuffAmt=5/rakeDmg=21`, Collector Revive move id 5, Chosen hex+strip STR, Maw/Donu edge cases, SphericGuardian inverse-wave, Heart scry-barrier interaction, Spear beam damage table)
- **Powers coverage** — ~40 of P4-P50 (Envenom dual routes, FireBreathing trigger, NoxiousFumes Artifact, TheBomb vs Intangible, ElectroDynamics pierce, Strength/Focus/Dex stacking, CorruptionPower makes-all-Skills-exhaust, DemonForm end-of-turn timing, FeelNoPain exhaust trigger, BarricadePower keeps block)
- **Missing relics** — ~17 (Dolly's Mirror onEquip transform, Necronomicon free first Attack, Runic Capacitor orb slots, Inserter energy-every-2-turns, Tough Bandages, Shovel, N'loth, WingBoots, Peace Pipe, Nuclear Battery, etc.)
- **Potions** — PT2 EntropicBrew pool, PT5 SwiftPotion turn-end return, PT6 FairyInABottle heal threshold, PT7 DistilledChaos ordering, PT9 LiquidBronze damage type, PT11 StrengthPotion decay
- **Damage pipeline** — DM2 Thorns pipeline, DM3 Plated vs block-retention, DM5 multi-hit fusion, DM6 Buffer vs block, DM7 Intangible cap layering, DM8 Vulnerable 1.5× truncation, DM9 Torii gating, DM10 Tungsten gating, DM11 HP-loss triggers, DM12 Angry sequence, DM13 Static Discharge vs Flight halving
- **Tests** — ~15 tests that encode bugs as parity (Stage G found ~25 instances; only some registered as D-rows)
- **Events** — RN-MAP-01, RN-EV-03 (GhostGold no card removal), RN-EV-05 (DeadAdventurer monster rolling), RN-NEOW-01 ONE_RANDOM_RARE path, RN-SHOP-05 card-removal scaling, RN-REWARD-02 count gate

Meta: the register is the correct disposition channel (reports = raw
ore, register = smelted rows). ~230 report-only rows is intentional per
Stage B narrative. Main risk: as code changes, per-report file:line
citations slide off. Suggested cadence: **promote 10 P1 rows per PR**
rather than one mega-promotion.

---

## Test-suite structural weaknesses (beyond D159)

Per enemy-tests audit:

1. **`test_enemies.rs` weak asserts** — lines 132, 157, 492, 507, 527, 535, 551, 573, 611 use `>=` / `is_some()` / `|| move_id` disjunctions where Java has exact values. Rewrite to equality.
2. **`test_enemies.rs` fresh-RNG-per-call** — `StsRandom::new(0)` yields deterministic num, so every `roll_next_move` call in this file pulls the same number; half the Java branch-space never runs for probabilistic enemies (Nob, Lagavulin, Nemesis, Automaton, CorruptHeart).
3. **Zero ascension coverage in `test_enemies.rs`** — every `create_enemy(base_hp)` uses A0 HP. A2/A4/A9/A19 branches untested. Recommended addition: at least one common per-act at A19 (Nob A2 StrengthDown, Snecko A17 Bite, Spiker A17 Thorns, SpireShield A19).
4. **Duplicated boss coverage** — `test_enemies.rs` 337-388 / 392-433 / 437-472 / 562-578 / 582-594 / 598-618 duplicate `test_bosses.rs` with weaker asserts. Delete the weaker copies.
5. **Missing post-execute integration** — Heart Debilitate, Reptomancer REPTO_SPAWN, Collector COLL_SPAWN, Automaton BA_SPAWN_ORBS, Lagavulin wake-on-damage, Nemesis per-turn Intangible, WrithingMass REACTIVE/MALLEABLE activation, CorruptHeart BEAT_OF_DEATH. All init-only.

---

## Already-in-register re-confirmations (this audit)

| Dn | Status | Notes |
|---|---|---|
| D1 | partial-fix; ~10 enemies still ignore `_num` | Register correctly acknowledges deferrals via D146-D157 |
| D55/D79 | open; player poison ticks at end-of-player-turn | Confirmed; Java `atStartOfTurn` |
| D70 | open → **P1 bump recommended** | Equilibrium decrement confirmed dead (only called from dead `process_end_of_turn`) |
| D88 | open; HolyWater generates wrong cards | `relics/defs/holy_water.rs:7-11` still adds 3 literal HolyWater cards |
| D89 | open; Fasting energy drain never fires | Chains to dead dispatch |
| D90 | open; Malleable no reset | `engine.rs:2520-2527` confirmed |
| D91 | open; `deal_damage_to_player` bypasses pipeline | Engine-wide, 30+ call sites |
| D100 | open; Collector REVIVE missing | Explicit "deferred: needs minion-dead signal" comment |
| D111 | open; pre-draw/post-draw conflation | Chains to dead dispatch |
| D112 | open; SadisticPower filters | Confirmed |
| D123 | open; DevaForm pre-increment | Confirmed |
| D124 | open; Pressure Points bypasses pipeline | `effects/interpreter.rs:621-629` subtracts directly from entity.hp |
| D129/D130 | **closed** (Stage F) | Both cleanly closed |
| D131 | deferred; JawWorm sub-roll | Dominant branch fallback comment at `act1.rs:26-28` |
| D132 | open; Byrd grounded Fly-Up | Unchanged |
| D133 | open; BronzeAutomaton HyperBeam turn-5 | `act2.rs:343-344` `len() >= 4` includes SPAWN |
| D134/D135 | open; Bandit opener re-emit | Confirmed |
| D136 | open; GremlinLeader ignores aliveCount | Confirmed |
| D137/D138 | open; Champ Anger/Defensive | Confirmed |
| D139 | open; Snecko threshold | Confirmed |
| D140 | open; WrithingMass Reactive never wired | Confirmed — `writhing_mass_reactive_reroll` dead production code |
| D141 | open; Transient damage direction reversed | Confirmed |
| D142 | open; Exploder skips UNKNOWN turn-2 | Confirmed |
| D143 | open; CorruptHeart slot 0 deterministic Blood Shots | Confirmed — Watcher A0 final boss |
| D144 | open; Heart A4-A8 scaling | Confirmed — Watcher A0 final boss |
| D145 | open; Darkling turn-1 | Confirmed |
| D146-D148 | open; FungiBeast + SpikeSlime_{M,L} ignore num | Confirmed signatures `_num` |
| D149 | open; AcidSlime_S threshold + fabricated guard | Confirmed |
| D150 | open; Louse single-vs-double move | Confirmed |
| D151 | open; SlaverBlue extra guard | Confirmed |
| D152 | open; GremlinWizard 2-turn vs Java 3 (P0) | Confirmed — needs `currentCharge == 3` gating |
| D153 | open; GremlinTsundere empty arm | Confirmed `mod.rs:853` empty match arm |
| D154 | open; Lagavulin 1:1 vs Java 2:1 (P0) | Confirmed — no `debuffTurnCount` status |
| D155/D156 | open; Sentry BOLT/BEAM + first-move positional (P0) | Confirmed — `mod.rs` create_enemy seeds BOLT regardless of index |
| D157 | open; Looter stream off-by-one | Confirmed |
| D158 | **closed** | Tautology removed, breadcrumb comment at `test_enemies.rs:603-607` |
| D159 | deferred | Consolidation PR |

---

## Recommended fix order (post-merge)

**Wave 1 — determinism P0s (F1-F4):** 1 PR, ~80 LOC + 1 test. Ship before any
overnight training relies on snapshot replay.

**Wave 2 — dead-dispatch decision:** delete or wire. Prerequisite for
closing D70, D89, D111, WS7-WS11. Touches `powers/buffs.rs` and
`engine.rs` turn-phase sites.

**Wave 3 — novel P1 register promotions:** D162-D167 (Flight,
Angry-type-gate, Buffer×2, Intangible-justApplied, Thorns-Intangible).
Group with D163-D164 as "enemy-damage edge cases" PR.

**Wave 4 — enemy AI P0 closures:** D143 (Heart slot 0 Blood Shots), D144
(Heart A4-A8 scaling), D140 (WrithingMass Reactive wiring), D152
(GremlinWizard 2-turn→3), D154 (Lagavulin 2:1), D155/D156 (Sentry
BOLT/BEAM). Watcher-A0 training-path impact.

**Wave 5 — damage pipeline P0:** D91 `deal_damage_to_player` bypass,
D124 Pressure Points bypass. Engine-wide fix; ~30 call sites.

**Wave 6 — test hygiene:** `test_enemies.rs` weak-assert rewrite,
ascension coverage (per-act common × 4), duplicated boss coverage
deletion, ~5 post-execute integration tests (Debilitate, REPTO_SPAWN,
BEAT_OF_DEATH, Nemesis Intangible, WrithingMass REACTIVE).

**Wave 7 — Stage B tail promotion:** ~10 P1 rows per PR per
stage-b-crossref recommendation.

---

## One more consolidation + full audit

Per user directive: after the above fix wave, one more consolidation
pass + full audit is scheduled. Inputs for that pass:

- Re-verify F1-F4 closure with new snapshot-RNG-roundtrip test.
- Re-audit combat_hooks for any regression introduced by dead-dispatch
  wiring (if taken).
- Spot-check Stage B tail promotions for file:line drift (bitrot risk).
- Re-check training-infra 0.84 / 1/2/4 surfacing.
- Diff-audit any subsequent commits against this triage doc.
