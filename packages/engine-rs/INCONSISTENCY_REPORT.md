# Comprehensive Parity Inconsistency Report

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

## 1. Executive Summary

Fresh audit result:

- supported-scope engine/runtime parity is effectively `99%`
- all-content parity including explicit unsupported backlog and post-merge semantic families is closer to `97-98%`
- the supported-scope merge-blocking gameplay tail is currently `0`
- the remaining work is now dominated by explicit unsupported scope, action-layer breadth, and a small set of Java-cited semantic families

Fresh verification completed during this audit:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- green-core suites:
  - `test_events_parity` `7 passed`
  - `test_event_runtime_wave18` `1 passed`
  - `test_reward_runtime` `10 passed`
  - `test_relic_runtime_java_green1` `8 passed`
  - `test_dead_system_cleanup_wave22` `1 passed`
  - `test_potion_runtime_action_path` `15 passed`
  - `test_generated_choice_java_wave3` `7 passed`
  - `test_orb_runtime_java_wave1` `7 passed, 2 ignored`
  - `test_damage_followup_java_wave1` `11 passed`
  - `test_card_play_timing_java_wave1` `8 passed`
  - `test_card_runtime_nonplay_triggers_wave1` `4 passed`
  - `test_rl_contract` `12 passed`
  - `test_rl_reward_obs_wave1` `2 passed`
  - `test_search_harness` `5 passed`
- live blocker inventory:
  - `test_card_runtime_generated_choice_wave5` `2 passed, 3 ignored`
  - `test_card_runtime_watcher_wave18` `1 passed, 5 ignored`
  - `test_card_runtime_watcher_wave19` `2 passed, 2 ignored`
  - `test_card_runtime_watcher_wave21` `2 passed, 3 ignored`
  - `test_card_runtime_defect_wave12` `3 passed, 3 ignored`
  - `test_card_runtime_colorless_wave8` `4 passed, 1 ignored`
  - `test_potion_runtime_wave8` `4 passed, 2 ignored`
  - `test_relic_runtime_wave17` `0 passed, 1 ignored`

Conclusion:

- If the merge claim is `supported runtime parity complete`, the branch is merge-ready after audit/doc cleanup.
- If the merge claim is `all gameplay content complete`, that stronger claim is still false.
- The remaining `1%` is mostly semantic-family breadth and stale/noisy audit debt, not missing core architecture.

Priority task queue from this audit:

1. Clean stale solved `#[ignore]` tests so the backlog is honest.
2. Keep PR/docs explicit that `Scrap Ooze` remains unsupported and that `Match and Keep!` is currently a temporary fixed-card approximation rather than the Java minigame.
3. If the stronger all-content claim is needed now, finish the remaining semantic families in section 6 first: `Neow`, potion legality / choose-one, `Emotion Chip`, `Scrap Ooze`, and full `Match and Keep!`.
4. After merge readiness is settled, pivot to training-system redesign with the appendix recommendations in section 7.

## 2. Quantified Baseline

Current live counts from the verified tree:

- raw public card files with empty `effect_data`: `3`
- raw public card files with `complex_hook: Some(...)`: `0`
- unresolved public gameplay-gap files: `0`
- blocked supported event ops: `0`
- unsupported blocked event branches still present in source: `1`
- total ignored tests in `packages/engine-rs/src/tests`: `79`

Ignored-test bucket totals from the full classifier pass:

- `26` confirmed active parity blockers
- `37` stale solved / obsolete noisy tests
- `11` post-merge enhancements
- `4` cleanup-only shell/accounting lines
- `1` unsupported/out-of-scope line

Those `26` active blockers are the full all-content backlog inventory. They are not the same thing as supported-scope merge blockers, which remain `0` in section 3.

The raw empty public-card files are cleanup-only shells:

- [`silent/reflex.rs`](./src/cards/silent/reflex.rs)
- [`silent/tactician.rs`](./src/cards/silent/tactician.rs)
- [`watcher/deusexmachina.rs`](./src/cards/watcher/deusexmachina.rs)

Engine-path proof already exists for those behaviors in [`test_card_runtime_nonplay_triggers_wave1.rs`](./src/tests/test_card_runtime_nonplay_triggers_wave1.rs).

Ignored-test counts by family:

- `Colorless/Choice`: `38`
- `Watcher`: `23`
- `Defect/Orb`: `9`
- `Relics/DeadSystem`: `7`
- `Other`: `7`
- `Ironclad/Zone`: `5`
- `Potions`: `3`

Important note:

- raw counts are noisier than classified counts
- the ignored backlog includes a mix of active blockers, stale solved tests, cleanup-shell/accounting debt, and post-merge enhancement work
- unsupported scope is mostly expressed in source as explicit blocked event branches, not as ignored tests

## 3. Merge-Gating Inconsistencies

No confirmed supported-scope semantic gameplay blockers remain after the fresh matrix verification.

The current merge gate is scope honesty, not missing supported runtime behavior:

- supported public gameplay-gap cards: complete
- supported event branches: unblocked
- public `complex_hook` gameplay tail: eliminated
- potion/relic/reward/RL surfaces: green on the representative core suites above

The branch is therefore merge-ready for the supported scope, with two caveats:

- unsupported content must stay explicitly marked unsupported
- stale ignored tests and cleanup shells must not be misrepresented as live gameplay gaps

## 4. Stale / Noisy Debt

### Finding S1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_card_runtime_shared_primitive_wave1.rs:70`](./src/tests/test_card_runtime_shared_primitive_wave1.rs#L70), [`test_card_runtime_tiny_primitive_wave2.rs:160`](./src/tests/test_card_runtime_tiny_primitive_wave2.rs#L160), [`test_card_runtime_colorless_wave10.rs`](./src/tests/test_card_runtime_colorless_wave10.rs)
- Problem: older ignored tests still claim `Enlightenment` is blocked, even though the current runtime has typed proof for the base and upgraded card on the canonical path.
- Recommended fix: delete or unignore the old blocker sentinels and keep `test_card_runtime_colorless_wave10` as the living proof.
- Test mapping: keep `test_card_runtime_colorless_wave10`; retire the two stale ignores.
- Worker slice: colorless ignored-test hygiene

### Finding S2
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_zone_batch_java_wave3.rs:117`](./src/tests/test_zone_batch_java_wave3.rs#L117), [`test_zone_batch_java_wave3.rs:138`](./src/tests/test_zone_batch_java_wave3.rs#L138), [`test_zone_batch_java_wave1.rs`](./src/tests/test_zone_batch_java_wave1.rs#L76), [`test_card_runtime_colorless_wave10.rs`](./src/tests/test_card_runtime_colorless_wave10.rs#L56)
- Problem: older ignored blockers still say `Headbutt` and `Violence` need primitives, but both cards now have green typed/runtime proof in newer suites.
- Recommended fix: remove or rewrite the stale blockers so they stop inflating the backlog.
- Test mapping: keep `test_zone_batch_java_wave1` and `test_card_runtime_colorless_wave10` as the engine-path evidence.
- Worker slice: zone/colorless ignored-test cleanup

### Finding S3
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_card_runtime_watcher_wave18.rs:97`](./src/tests/test_card_runtime_watcher_wave18.rs#L97), [`test_card_runtime_post_damage_wave1.rs`](./src/tests/test_card_runtime_post_damage_wave1.rs#L22)
- Problem: `Wallop` still has an old ignored blocker in a Watcher wave file, but its typed post-damage block-gain proof already exists elsewhere.
- Recommended fix: retire the stale ignore and let the post-damage suite own the behavior.
- Test mapping: `test_card_runtime_post_damage_wave1`
- Worker slice: Watcher ignored-test cleanup

### Finding S4
- Area: parity
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_card_runtime_watcher_wave16.rs:121`](./src/tests/test_card_runtime_watcher_wave16.rs#L121), [`watcher/deusexmachina.rs`](./src/cards/watcher/deusexmachina.rs), [`test_card_runtime_nonplay_triggers_wave1.rs`](./src/tests/test_card_runtime_nonplay_triggers_wave1.rs#L60)
- Problem: `Deus Ex Machina` still carries stale blocker wording even though the raw source file is explicitly a cleanup-only shell and the behavior is already proven through the non-play trigger runtime.
- Recommended fix: convert the old ignore into a cleanup-shell note or delete it entirely.
- Test mapping: `test_card_runtime_nonplay_triggers_wave1`
- Worker slice: non-play cleanup-shell hygiene

### Finding S5
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_card_runtime_watcher_wave18.rs:80`](./src/tests/test_card_runtime_watcher_wave18.rs#L80), [`test_card_runtime_watcher_wave21.rs:24`](./src/tests/test_card_runtime_watcher_wave21.rs#L24), [`test_card_runtime_watcher_wave24.rs`](./src/tests/test_card_runtime_watcher_wave24.rs#L16), Java oracles `CollectAction.java`, `ConjureBladeAction.java`, `Fasting.java`
- Problem: older ignored tests still claim `Collect`, `Conjure Blade`, and `Fasting` are blocked, but the newer Watcher wave has green typed/runtime proof for those cards.
- Recommended fix: remove the stale ignores and keep the newer Watcher suite as the owning parity evidence.
- Test mapping: `test_card_runtime_watcher_wave24`
- Worker slice: Watcher stale-blocker cleanup

### Finding S6
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: [`test_card_runtime_watcher_wave18.rs:96`](./src/tests/test_card_runtime_watcher_wave18.rs#L96), [`test_card_runtime_post_damage_wave1.rs`](./src/tests/test_card_runtime_post_damage_wave1.rs#L95), [`test_card_runtime_watcher_wave19.rs:52`](./src/tests/test_card_runtime_watcher_wave19.rs#L52), [`test_card_runtime_watcher_wave20.rs`](./src/tests/test_card_runtime_watcher_wave20.rs#L12), [`test_dead_system_cleanup_wave9.rs:48`](./src/tests/test_dead_system_cleanup_wave9.rs#L48), [`test_relic_runtime_wave15.rs`](./src/tests/test_relic_runtime_wave15.rs#L68)
- Problem: `Wallop`, `Judgement`, and `Mutagenic Strength` still appear in stale ignored blocker text even though newer engine-path suites already prove them.
- Recommended fix: delete or rewrite the old ignores so the backlog only reflects live semantic debt.
- Test mapping: `test_card_runtime_post_damage_wave1`, `test_card_runtime_watcher_wave20`, `test_relic_runtime_wave15`
- Worker slice: watcher/relic stale-blocker cleanup

### Finding S7
- Area: parity
- Severity: low
- Confidence: medium
- Scope: cleanup-only
- Evidence: [`test_card_runtime_watcher_wave18.rs:94`](./src/tests/test_card_runtime_watcher_wave18.rs#L94), [`cards/watcher/pressurepoints.rs`](./src/cards/watcher/pressurepoints.rs#L11), [`effects/runtime.rs:1122`](./src/effects/runtime.rs#L1122), [`test_integration.rs:1758`](./src/tests/test_integration.rs#L1758)
- Problem: `Pressure Points` still has blocker wording in older Watcher suites, but the current typed/runtime path already applies Mark and then triggers HP loss across marked enemies. It should only remain open if a concrete failing behavioral case can be reproduced.
- Recommended fix: either produce a specific failing engine-path case or retire the stale blocker text.
- Test mapping: current integration coverage plus any future reproducer
- Worker slice: Watcher blocker verification

### Finding S8
- Area: dead-system
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [`effects/registry.rs:416`](./src/effects/registry.rs#L416), [`effects/registry.rs:440`](./src/effects/registry.rs#L440), [`effects/registry.rs:462`](./src/effects/registry.rs#L462), [`effects/registry.rs:534`](./src/effects/registry.rs#L534), [`effects/mod.rs:39`](./src/effects/mod.rs#L39)
- Problem: dead or effectively dead dispatch exports still remain in the effect registry and module reexports, which makes the architecture look less unified than it is.
- Recommended fix: prune unused exports once the final audit/doc pass is stable, or clearly mark them as legacy-only cleanup targets.
- Test mapping: `test_dead_system_cleanup_wave22`
- Worker slice: effect-registry cleanup

### Finding S9
- Area: architecture
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [`engine.rs:2038`](./src/engine.rs#L2038), [`effects/registry.rs:520`](./src/effects/registry.rs#L520), [`effects/hooks_draw.rs:18`](./src/effects/hooks_draw.rs#L18)
- Problem: `dispatch_on_draw` is the one remaining live draw-trigger seam, primarily to support the `Deus Ex Machina` draw behavior.
- Recommended fix: keep it explicit in the audit as the last real helper seam, then decide later whether to normalize it into the canonical runtime.
- Test mapping: `test_card_runtime_nonplay_triggers_wave1`
- Worker slice: draw-trigger unification

## 5. Unsupported Backlog

### Finding U1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: unsupported
- Evidence: [`events/exordium.rs:177`](./src/events/exordium.rs#L177), [`events/exordium.rs:179`](./src/events/exordium.rs#L179)
- Problem: `Scrap Ooze` still contains an explicit blocked branch for the relic-gain line.
- Recommended fix: keep it called out as unsupported unless the event branch is actually implemented.
- Test mapping: explicit source classification only; no supported-scope blocker suite currently fails here.
- Worker slice: unsupported event backlog

## 6. Post-Merge Backlog

These are real Java-cited semantic families, but they do not currently block a supported-scope merge if scope stays honest.

### Finding P1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`test_orb_runtime_java_wave1.rs`](./src/tests/test_orb_runtime_java_wave1.rs#L211), [`test_card_runtime_defect_wave12.rs`](./src/tests/test_card_runtime_defect_wave12.rs#L151), Java oracles `EmotionChip.java`, `BarrageAction.java`, `RipAndTearAction.java`, `ThunderStrikeAction.java`
- Problem: orb timing and multi-hit timing still miss some exact Java semantics. `Emotion Chip` is the sharpest live timing mismatch, and the Defect multi-hit cards still approximate zero-orb and per-hit retarget edge behavior.
- Recommended fix: split this into `Emotion Chip` timing and multi-hit target/zero-count parity, then retire the old ignored tests once concrete engine-path proof exists.
- Test mapping: `test_orb_runtime_java_wave1`, `test_card_runtime_defect_wave12`
- Worker slice: orb timing / multi-hit family

### Finding P2
- Area: parity
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`test_potion_runtime_wave8.rs`](./src/tests/test_potion_runtime_wave8.rs#L170), Java oracle files `StancePotion.java` and `SmokeBomb.java`
- Problem: potion legality and choose-one edges are thinner than the main activation path. `Stance Potion` still wants a real choose-one decision, and `Smoke Bomb` legality still belongs to action enumeration edge handling.
- Recommended fix: add a real choose-one potion decision primitive and tighten legality enumeration for edge-restricted potions.
- Test mapping: `test_potion_runtime_wave8`
- Worker slice: potion legality family

### Finding P3
- Area: parity
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`test_relic_runtime_wave17.rs`](./src/tests/test_relic_runtime_wave17.rs), [`src/tests/test_run_parity.rs`](./src/tests/test_run_parity.rs), [`src/status_ids.rs:214`](./src/status_ids.rs#L214), Java oracle files `NeowsLament.java`, `NeowEvent.java`, `NeowReward.java`
- Problem: `Neow's Lament` / `NeowsBlessing` combat-start HP reduction remains blocked, and the run action layer still starts post-Neow instead of exposing a real four-option start decision surface for bots.
- Recommended fix: add a real `Neow` run/start phase that always exposes four choices for the RL surface, then back the `NeowsBlessing` reward path with a runtime-complete combat-start relic implementation.
- Test mapping: `test_relic_runtime_wave17`, `test_run_parity`, follow-up `test_rl_contract`
- Worker slice: Neow start/action-layer family

### Finding P4
- Area: parity
- Severity: low
- Confidence: high
- Scope: post-merge
- Evidence: [`test_generated_choice_java_wave2.rs`](./src/tests/test_generated_choice_java_wave2.rs), [`test_card_runtime_colorless_wave3.rs`](./src/tests/test_card_runtime_colorless_wave3.rs), [`test_card_runtime_colorless_wave4.rs`](./src/tests/test_card_runtime_colorless_wave4.rs), [`test_card_runtime_colorless_wave5.rs`](./src/tests/test_card_runtime_colorless_wave5.rs), [`test_card_runtime_colorless_wave6.rs`](./src/tests/test_card_runtime_colorless_wave6.rs), [`test_card_runtime_colorless_wave7.rs`](./src/tests/test_card_runtime_colorless_wave7.rs), [`test_card_runtime_watcher_wave11.rs`](./src/tests/test_card_runtime_watcher_wave11.rs)
- Problem: a long-tail of ignored tests remains for real but lower-priority enhancement families such as `Forethought`, `Impatience`, `Madness`, `Mind Blast`, retained-state Watcher cards, and some older discovery-style approximations.
- Recommended fix: keep these in a post-merge enhancement queue rather than pretending they are merge blockers for the current supported branch.
- Test mapping: the suites above remain the active backlog inventory
- Worker slice: post-merge enhancement backlog

### Finding P5
- Area: parity
- Severity: low
- Confidence: high
- Scope: post-merge
- Evidence: [`src/events/shrines.rs`](./src/events/shrines.rs), [`src/tests/test_event_runtime_wave19.rs`](./src/tests/test_event_runtime_wave19.rs), Java oracle `GremlinMatchGame.java`
- Problem: `Match and Keep!` is now on the canonical event reward runtime, but only as a temporary fixed `Rushdown+` / `Adaptation+` reward instead of the Java card-matching minigame.
- Recommended fix: keep the temporary path for starter-seed breadth, but replace it with a real minigame plus nested keep/discard resolution before claiming full Java parity for all shrine content.
- Test mapping: `test_event_runtime_wave19`
- Worker slice: shrine minigame parity

## 7. Training Appendix

The engine audit no longer shows core gameplay architecture as the primary risk. Training-system quality is now the more important next investment.

### Finding T1
- Area: training
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`logs/runs/run_20260414_singlecfg_smoke/status.json`](../logs/runs/run_20260414_singlecfg_smoke/status.json), [`logs/runs/run_20260414_singlecfg_smoke/summary.json`](../logs/runs/run_20260414_singlecfg_smoke/summary.json), [`scripts/training.sh`](../scripts/training.sh), [`packages/training/training_runner.py`](../packages/training/training_runner.py)
- Problem: the training stack is viable enough to launch and write artifacts, but the smoke run only proves startup and collection, not meaningful learning.
- Recommended fix: use one interpretable overnight baseline first, then revisit architecture with real run data in hand.
- Test mapping: validated by the smoke runs and `tests/training/test_runtime_hardening.py`
- Worker slice: overnight baseline execution

### Finding T2
- Area: training
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`packages/training/episode_log.py`](../packages/training/episode_log.py), [`packages/training/training_runner.py:211`](../packages/training/training_runner.py#L211), [`packages/training/training_runner.py:1317`](../packages/training/training_runner.py#L1317), [`packages/training/strategic_trainer.py:425`](../packages/training/strategic_trainer.py#L425)
- Problem: RL logging is already qualitatively strong, but provenance and persisted trainer diagnostics are too thin for long-term trustworthy comparison across runs.
- Recommended fix: add a durable `run_manifest.json`, persist the full trainer diagnostics (`explained_variance`, `kl_divergence`, `mean_advantage`, `mean_return`, `num_transitions`), and record checkpoint lineage/replay summaries.
- Test mapping: no engine-path test; this belongs to the training-infra rewrite queue
- Worker slice: training provenance/logging

### Finding T3
- Area: training
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`packages/app/SpireMonitor/DataLayer/StatusPoller.swift`](../packages/app/SpireMonitor/DataLayer/StatusPoller.swift), [`packages/training/worker.py:316`](../packages/training/worker.py#L316), [`packages/app/SpireMonitor/Views/Live/WorkerGridView.swift`](../packages/app/SpireMonitor/Views/Live/WorkerGridView.swift)
- Problem: monitoring is usable tonight but still has blind spots, especially worker freshness, stale-status detection, and the gap between computed diagnostics and persisted diagnostics.
- Recommended fix: keep using the monitor/watchdog tonight, but queue worker staleness and richer diagnostics as part of the training-system rewrite.
- Test mapping: smoke-run and manual monitor inspection
- Worker slice: training monitoring hardening

### Finding T4
- Area: training
- Severity: medium
- Confidence: high
- Scope: post-merge
- Evidence: [`packages/training/worker.py:331`](../packages/training/worker.py#L331), [`packages/training/turn_solver.py:1103`](../packages/training/turn_solver.py#L1103), [`packages/training/strategic_mcts.py:150`](../packages/training/strategic_mcts.py#L150)
- Problem: a restricted Watcher A0 search-first baseline is plausible, but only as a combat-search evaluation harness. The current strategic layer is not yet strong enough to count as a pure-search full-run planner.
- Recommended fix: treat `Watcher A0 / no card adds / remove-only shop / smith-first rest` as a later evaluation mode, not tonight’s main PPO run.
- Test mapping: planning recommendation only
- Worker slice: search-first evaluation harness

Tonight's naive baseline recommendation:

```bash
./scripts/training.sh start --games 4000 --workers 4 --batch 256 --asc 0 --headless --watchdog --sweep-config baseline_control
```

What to watch overnight:

- `status.json` freshness
- `games_per_min`
- `inference.total_requests`, queue wait, forward latency
- `train_steps`, `buffer_size`, `replay_buffer`
- checkpoint files
- `perf_log.jsonl` and `metrics_history.jsonl`
- milestone/watchdog output
