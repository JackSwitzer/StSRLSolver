# Watcher A0 Oracle Closure Scorecard (90% Engine-Ready)

> **Historical Wave 3 snapshot.** This file preserves the state before the
> Wave 3b systemic replay work. Its 3,110-test and 51/607-prefix figures are not
> current. Use [`../oracle-replay-wave3b-appendix.md`](../oracle-replay-wave3b-appendix.md)
> for the live 3,269-test, 2,013-action, exact-omission, assumption, and
> certification scorecard.

**Branch:** `codex/oracle-loop-wave3`

**Authority:** decompiled Java for behavior; human-minted Java traces for
end-to-end certification

## Decision Dashboard

| Gate | Result | Evidence |
| --- | ---: | --- |
| Native Rust simulator regression | 3,110 tests, 0 failed, 0 skipped | `./scripts/test_engine_rs.sh test --lib` |
| All Rust targets | green | `./scripts/test_engine_rs.sh check --all-targets` |
| Canonical run actions | 27/27 serialized and documented | `test_trace_schema_v2` |
| Deterministic v2 replay | green; header + 4 transitions + end | `smoke-v2-neow-floor1.json` |
| V1 differ field coverage | complete for every serialized field | one-field mutation tests in `test_trace_oracle` |
| Generic production RNG | zero matches | `rand::`, `RngCore`, `SliceRandom` source scan |
| Native RNG source audit | no algorithm/overload mismatch found | `RandomXS128`, StS `Random`, seeded JDK LCG fixture matrix |
| Process-global RNG lifecycle | modeled semantic calls green; desktop cadence remains external | shop speech/reset tests plus per-checkpoint ambient recorder requirement |
| Targetable enemy catalog | 66/66 canonical Java IDs | ledger-derived construct/export/roll tests |
| Ambient RNG initial state | external oracle input required | `new_with_ambient_states`; `wave3-recorder-needs.md` |
| Recording corpus intake | 14/14 attempted; 13 readable, 1 truncated | `test_recording_bundles` + rebuilt CLI replay |
| Deepest comparable state prefix | 51/607 on direct-reward A0 victory; 42 direct checkpoints | bundle first-divergence report |
| Strict Java-certified prefix | 0 for every bundle | every readable bundle lacks authoritative profile initialization |
| Java power projection | common static status-backed IDs/amounts mapped; false dynamic projection suppressed | trace projection tests; direct-add/dynamic powers remain F16 |
| Pure-core boundary | no observation/search/training/PyO3 module or dependency | `lib.rs`, `Cargo.toml`, production source scan |

## Rubric-Backed Readiness

This score measures whether the Rust core is a sound base for the next training
layer. It does **not** convert a partially replayable corpus into a parity claim.

| Area | Weight | Score | Basis |
| --- | ---: | ---: | --- |
| Native RNG and counted stream ownership | 20% | 100% | Java-derived vectors, overload/counter/lifecycle audit, and no generic gameplay RNG; desktop ambient cadence is an external oracle input |
| Combat and verified content | 30% | 94% | 667/667 source-derived ledger rows, 66 targetable enemy identities, and zero-skip engine-path suite; relic startup queue order and dynamic power identity remain open |
| Run systems and canonical actions | 30% | 82% | action/runtime surfaces and process-global RNG lifecycle tests are green; current recorder cannot prove uninterrupted Neow-to-Heart continuation |
| Causal checkpoints and trace tooling | 15% | 78% | deterministic restore, paired deck/power order, and hardened bundle adapter; profile, ambient state, dynamic power identity, and settled checkpoints are incomplete in the corpus |
| Dead-system retirement | 5% | 100% | consumer observations, search policies, training rewards, bindings, and generic RNG are disconnected from the core |
| **Weighted engine-side readiness** | **100%** | **90%** | rounded from 89.5%; dominated by unverified full-run continuation and exact power-list/oracle projection, not by content-row work |

The corpus-certification answer is deliberately not expressed as a percentage.
The deepest comparable prefix is `51/607`, but the strict Java-certified prefix
is `0` because every readable legacy bundle lacks authoritative profile
initialization. Missing profile inputs and recorder actions make the remaining
denominator non-comparable rather than proving 556 actions wrong.

## Completion Read

The native core remains highly implemented, but full-run Java certification is
still open. The strongest real-game statement is currently a **51-action
comparable prefix** on one A0 victory. It stops at a relic identity that depends
on the missing profile/unlock snapshot (`Turnip` in Java versus
`StoneCalendar` under Rust's compatibility fallback), so continuing under an
all-unlocked guess would manufacture evidence. The prefix contains 42 direct
state checkpoints plus coupled callbacks; the report records 10 inferred
actions, 10 coupled callbacks, and 6 actions with incomplete semantic identity.
The legacy Neow action has matching effects but no semantic option payload, so
that identity remains explicitly unverified. Four other A0 terminal bundles
stop after two records because their Neow grid selections are absent from the
recorder dialect. No implementation percentage should be presented as corpus
certification.

## Recording Bundle Matrix

| Bundle group | Count | Deepest comparable action | First hard boundary |
| --- | ---: | ---: | --- |
| Direct-reward A0 terminal (`-588468...194423`) | 1 | 51/607 | profile/unlock snapshot is absent, so the relic pool cannot be certified |
| Neow-grid A0 terminal (`-836201...174739`, `367900...190036`, `473375...181902`, `634121...193226`) | 4 | 2 | recorder omits selected deck-card identity |
| Zero-action A0 abandon | 3 | 0 | no recorded gameplay action exists to replay |
| Mid-run resume without a pre-action checkpoint | 3 | 0 | first action is an event choice while Rust must still be at Neow |
| Non-A0/profile initial state not represented | 2 | 0 | Java deck contains `AscendersBane`; recorder metadata cannot reconstruct the run |
| Truncated gzip (`-635665...214546`) | 1 | n/a | unexpected end of file |

No bundle uses a `DEV-` divergence mask. All 13 readable reports carry the
`missing_profile_snapshot` initialization quarantine, and no recorder omission
is credited as a state match. The ledger remains `667 verified, 0 unverified`
because this wave reopened system-level certification rather than a
source-verified content row.

## Fixed And Proven

| Item | Status | Proof |
| --- | --- | --- |
| Full v1 differ surface | FIXED-PROVEN | Record identity/action, complete nested post-state, and RNG-first ordering have corruption tests. |
| Canonical v2 action script | FIXED-PROVEN | Direct `GameAction` serialization, seed identity validation, illegal-action rejection, causal chaining, and deterministic replay pass. |
| V2 CLI replay | FIXED-PROVEN | `trace_replay --script ... --out ...` emits six valid JSONL envelopes for the smoke fixture. |
| Schema/document synchronization | FIXED-PROVEN | A test fails if any serialized `GameAction` variant is absent from `script-schema-v2.md`. |
| False Java-certification guard | FIXED-PROVEN | V2 CLI rejects Java diff flags until a language-neutral projection exists. |
| Language-neutral oracle state | FIXED-PROVEN | Mandatory validated schema, selected Neow witness, typed state, and all 13 RNG fields have serde and corruption tests. |
| Exact RNG continuation state | FIXED-PROVEN | Oracle minor 1 requires both raw `RandomXS128` words for all 13 named streams plus Neow, the ambient MathUtils words, and the 48-bit Collections LCG; duplicated counters must agree or decoding fails. |
| Causal checkpoint compatibility | FIXED-PROVEN | Required power order is serialized under `sts.core_checkpoint` major 2; major 1 and the prior semantics fingerprint are rejected explicitly before continuation. |
| Bundle intake/diff | FIXED-PROVEN | Concatenated gzip members, full action payloads, schema/index alignment, and unique identity mapping fail hard; skipped/coupled/inferred/profile-incomplete reports are `uncertified`, never `match`. |
| RNG API honesty | FIXED-PROVEN | Standalone combat fixtures are crate-only, benchmark fixtures are explicitly named, RNG-less enemy rolling is test-only, and both ambient states are injectable. |
| Watcher starter identity | FIXED-PROVEN | Persistent/combat state now uses Java `Strike_P`/`Defend_P`; all comparable fields in the five A0 action-0 states align. |
| Starter-system integration | FIXED-PROVEN | Canonical starter rarity/color now drives Neow transform, bottled-relic spawn legality, NoteForYourself storage, and Pandora's Box removal. |
| Pre-map coordinate | FIXED-PROVEN | Rust now models Java's synthetic `(0,-1)` node. |
| End-turn discard order | FIXED-PROVEN | Non-retained cards discard top-to-bottom, preserving Java future shuffle input. |
| Oracle intent/history view | FIXED-PROVEN | Dynamic intent damage, Java move-history view, visible power IDs, and internal AI-counter filtering are covered by source-derived trace tests and the 51-action comparable prefix. |
| Static status-backed Java power projection | FIXED-PROVEN WITH OPEN TAIL | Common Java IDs, marker/compound amounts, required checkpoint order, sorted/direct mutation modes, all audited Watcher A0 direct-add sites, and separate `Energized`/`EnergizedBlue` identities are tested. The inaccurate aggregate Bomb projection is suppressed. Relic startup queue order and dynamic/non-status powers remain F16. |
| Java float intent projection | FIXED-PROVEN | Weak, Vulnerable, stance, BackAttack cast, and Intangible retain one float pipeline until Java's final floor. |
| Process-global RNG continuity | FIXED-PROVEN | Successful shop purchases consume speech-timer where applicable, voice, buy-message, side, and position draws; Masked Bandits pays one ambient target draw per gold; reset preserves both MathUtils and Collections states. |
| Resolved combat room identity | FIXED-PROVEN | Active combat retains the concrete Monster/Elite/Boss/Event room through checkpoints, so `?` monster fights receive normal MonsterRoom rewards including Prayer Wheel. |
| Enemy identity/catalog | FIXED-PROVEN | All 66 targetable ledger monsters construct, export, and roll under canonical Java IDs; aliases normalize before state creation and unknown IDs fail closed. |
| Event combat rewards | FIXED-PROVEN | Masked Bandits and Mushrooms use explicit follow-up decisions and claimable random-gold/relic reward screens. |
| Event constructor RNG and recursive state | FIXED-PROVEN | World of Goop, Shining Light, Mausoleum, Fountain, Falling, Sensory Stone, Designer, We Meet Again, and Dead Adventurer have Java-cited constructor/action tests. Dead Adventurer preserves its Java-shuffled reward order and encounter key, exposes 25/50/75% risk, returns an explicit Fight decision, and restores deterministically. |
| Pandora checkpoint identity | FIXED-PROVEN | Pandora removal keeps deck names and card instances aligned across capture/restore. |
| Typed choice cleanup | FIXED-PROVEN | Untyped named-choice/no-op Gold and first-target free-play adapters are deleted; Wish and Omniscience use typed Java paths. |
| Terminal-rollout test honesty | FIXED-PROVEN | The legal-action rollout must now reach an actual terminal state before its cap; the old tautological “terminal or hit cap” assertion was removed. This is not presented as Heart replay evidence. |

## Remaining Merge Gates

1. Land one uninterrupted canonical v2 Neow-to-Heart replay. Current tests
   prove every inspected player-decision family and the Act 4 route, but the
   core-action test stops at the first combat and Heart tests inject Act 3
   state plus forced combat outcomes. This is a coverage gate, not a known
   missing `GameAction` variant.
2. Fulfill `data/traces/requests/wave3-recorder-needs.md`, especially semantic
   Neow payloads, nested card selections, omitted skip/leave actions, stable
   causal checkpoints, and both ambient RNG states at every checkpoint.
3. Close F16's remaining power tail: preserve cross-relic `addToTop`,
   `addToBot`, and direct startup queue order, and serialize dynamic/non-status
   power identities such as The Bomb, Minion, BackAttack, Stasis, and Pen Nib.
4. Re-record the truncated bundle and provide pre-action resume checkpoints.
5. Replay every repaired A0 bundle and continue fixing the earliest
   source-confirmed divergence until each full run matches.
6. Capture profile/final-act/key and ambient RNG initial conditions so a full
   match does not rely on hidden defaults.

## Stacked PR State

| Layer | Branch | State |
| --- | --- | --- |
| Native Java RNG | `codex/rng-core-java` | published draft |
| Run generation | `codex/run-generation-parity` | published draft |
| Canonical actions | `codex/full-run-actions-v2` | published draft |
| Causal checkpoints | `codex/core-checkpoint-normalization` | published draft |
| Pure simulation boundary | `codex/pure-sim-freeze` | published draft |
| Oracle closure | `codex/watcher-a0-oracle-closure` | engine-side base complete |
| Real recording loop | `codex/oracle-loop-wave3` | 51-action prefix; profile/recorder blockers filed |

## Verdict

The offline oracle loop now works and has already found substantive identity
and discard-order defects. The stack remains draft: current recordings omit
decisions and initial conditions required for deterministic continuation, so
Neow-to-Heart parity is not yet certified. No recorder omission is masked or
treated as an engine match.
