# Engine Parity Scorecard

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

Execution map for the current decompile-backed endgame:

- [`DECOMPILE_PARITY_ENDGAME.md`](./DECOMPILE_PARITY_ENDGAME.md)

## Rubric

This scorecard uses the audit-skill rubric:

- `100%`: production path matches the intended behavior, has engine-path coverage, and no placeholder or inline exception remains for that slice.
- `75%`: production path is mostly correct and tested, but still has temporary inline handling, adapters, or missing edge-case tests.
- `50%`: the surface exists and is partly wired, but important behavior is still simplified, placeholder-backed, or helper-path only.
- `25%`: architecture exists, but production behavior is still mostly missing.
- `0%`: missing.

## Scorecard

Weighted overall completion toward the target "universal gameplay runtime + decision-complete RL loop": `99%`

Weights:

- combat runtime parity: `35%`
- RL combat surface: `25%`
- run/reward/event decision parity: `25%`
- dead-system retirement: `15%`

Area scores:

- combat runtime parity: `99%`
- RL combat surface: `98%`
- run/reward/event decision parity: `99%`
- dead-system retirement: `98%`
- architecture unification snapshot: `99%`

Interpretation:

- If the target were only "combat event runtime parity," the branch is very close.
- If the target is the full user request, including reward ordering, event/run decisions, stable RL search surfaces, and legacy deletion, the branch is approaching merge quality but still blocked by breadth and cleanup, not by missing core architecture.

## Current Quantified Backlog

These counts come from the current verified production tree and are useful as a hard baseline for future worker waves:

- card files with empty `effect_data`: `7`
- card files still using `complex_hook`: `6`
- unresolved public card files (union of empty typed programs and hook-backed files): `9`
- typed event placeholder branches still using `EventProgramOp::blocked(...)`: `0`
- live production potion fallback callsites: `0`
- other live production legacy dispatch/install callsites: `0`
- confirmed Java-backed self-mutating card family still failing its dedicated engine-path suite: `0`

Empty-`effect_data` card backlog by class:

- Watcher: `1`
- Defect: `2`
- Silent: `3`
- Ironclad: `2`
- Colorless: `0`

Additional shared-file tail outside the five main class folders:

- shared card modules and temp/status/curses files: `3`

What those numbers mean:

- the card registry is broad, but the remaining file-level tail is now much smaller and concentrated in retained-state, generated-choice, orb-scaling, manual-discard, post-damage-context families, and a very short colorless / Watcher utility residue
- the currently verified unresolved public-card tail is: `Ritual Dagger`, `Scrape`, `Burning Pact`, `Dual Wield`, `Fiend Fire`, `Nightmare`, `Reflex`, `Tactician`, and `Deus Ex Machina`
- `Blizzard` has now moved onto a typed `status count × card magic` damage source, so it no longer counts as a hook-backed or empty-program public card
  - `Ritual Dagger` is no longer an empty typed-program shell; it now carries a typed damage body while its kill-scaling misc propagation stays hook-backed behind a Java-cited blocker
  - `Reflex`, `Tactician`, and `Deus Ex Machina` are now carried by verified runtime draw/discard hook coverage rather than stale blocker sentinels
  - `Escape Plan`, `Malaise`, and `Lesson Learned` are now on typed runtime surfaces; `Enlightenment`, `Reboot`, `Fission`, base `True Grit`, and `Second Wind` are now on typed runtime/declarative paths, while `Malaise` / `Lesson Learned` have moved out of the hook-backed public-card tail
  - the event runtime no longer relies on `EventProgramOp::blocked(...)` for supported content, and `Golden Wing` is now honest on the typed runtime path; `Dead Adventurer` now carries ascension-sensitive first-search normalization on the canonical typed event path
- direct relic helper-path references in `src/tests/test_relics_parity.rs` and `src/relics/mod.rs` are now at `0`; the old helper-path relic test modules and `relics/combat.rs` are deleted, the final `Runic Pyramid` / `Unceasing Top` hand-lifecycle bridges are deleted from `relics/run.rs`, and the remaining dead-system tail is now mostly ignored blocker tests plus narrow oracle cleanup
- the easiest remaining non-hook empties are now concentrated in a few real primitive families: Silent discard/queue sequencing, Ironclad exhaust/top-play, Defect frost/order, and Colorless utility/cost-mutation behavior

## Why We Are Not Done Yet

### 1. Reward flow is ordered and substantially runtime-native, but still not fully universal

The branch now has a real ordered reward screen with claim/open, choose, and skip semantics for selectable rewards. Combat potion rewards, boss relic choice screens, event-generated reward items, and treasure/chest rewards now all route through that runtime, and automatic combat gold stays outside the action space where it belongs. The remaining gap is breadth rather than architecture: the last event-specific reward branches, rarity-conditioned outcomes, and some compact reward-source observability still need wider engine-path coverage.

- [`src/run.rs:1134`](./src/run.rs#L1134)
- [`src/run.rs:1161`](./src/run.rs#L1161)
- [`src/run.rs:1188`](./src/run.rs#L1188)
- [`src/run.rs:1538`](./src/run.rs#L1538)
- [`src/decision.rs:151`](./src/decision.rs#L151)

Why this blocks parity:

- reward branching by rarity or multi-step event flow still needs broader engine-path coverage
- a few RL-facing reward-source and relic-counter details are still coarser than the underlying runtime state
- reward mutation across more exotic relic stacks still needs broader engine-path coverage

### 2. Reward-affecting relic semantics are better covered, but not finished

The run reward path can now express ordered claim-before-choose behavior, and key same-screen ordering cases are possible. `Matryoshka`, combat relic ordering, and several same-screen mutation cases now have engine-path coverage. The remaining problem is breadth: not every reward-affecting relic and event/chest interaction has equivalent coverage yet, and a few runtime counters are still more canonical than they are compactly observable.

- [`src/relic_flags.rs`](./src/relic_flags.rs)
- [`src/run.rs:1161`](./src/run.rs#L1161)
- [`src/tests/test_reward_runtime.rs`](./src/tests/test_reward_runtime.rs)
- [`src/tests/test_event_runtime_wave14.rs`](./src/tests/test_event_runtime_wave14.rs)

Examples still blocked or incomplete:

- rarity-aware shrine/event outcomes still need shared reward branching primitives
- some potion-drop and chest mutation stacks still lack the same ordered-runtime treatment
- reward-source observability was recently improved for treasure screens, but related compact-observation coverage is still thinner than combat/event reward coverage

### 3. A few gameplay islands still bypass the runtime

The owner-aware runtime is real, and most former helper seams are gone, but it is not yet the only production behavior path.

- [`src/engine.rs:142`](./src/engine.rs#L142)
- [`src/engine.rs:186`](./src/engine.rs#L186)
- [`src/engine.rs:1474`](./src/engine.rs#L1474)
- [`src/engine.rs:1575`](./src/engine.rs#L1575)

Still-inline examples:

- combat-start orb channel setup
- some orb passive/lifecycle handling that still reads direct engine state
- combat choice handling still centered on a single active choice in `CombatEngine`

### 4. Played-card instance state is covered, and the remaining timing tail is narrower

The highest-signal self-mutating Java card family is no longer a blocker in the current tree. `Streamline`, `Rampage`, `Steam Barrier`, `Glass Knife`, `Genetic Algorithm`, and `Ritual Dagger` now have dedicated engine-path coverage proving that mutation is carried on the played `CardInstance`, not broadcast through player-global status hacks. The recent timing wave also moved `Time Warp` onto the normal `OnAfterUseCard` runtime path. The remaining combat gap has moved up a level: a smaller set of replay, orb, and generated-choice edges still need tighter phase coverage.

- [`src/tests/test_defect_java_wave1.rs`](./src/tests/test_defect_java_wave1.rs)
- [`src/tests/test_played_card_instance_state.rs`](./src/tests/test_played_card_instance_state.rs)
- [`src/tests/test_card_runtime_scaling_wave1.rs`](./src/tests/test_card_runtime_scaling_wave1.rs)
- [`src/card_effects.rs:140`](./src/card_effects.rs#L140)
- [`src/effects/hooks_complex.rs`](./src/effects/hooks_complex.rs)
- [`src/effects/hooks_damage.rs:50`](./src/effects/hooks_damage.rs#L50)
- [`src/tests/test_card_play_timing_java_wave1.rs`](./src/tests/test_card_play_timing_java_wave1.rs)

Why this matters:

- Java `Streamline`, `Rampage`, `Steam Barrier`, `Glass Knife`, `Genetic Algorithm`, and `Ritual Dagger` mutate the played card instance, not all copies or a player-global scalar
- the current Rust tree now reflects that for the dedicated family suite, which is a meaningful architecture win for copy semantics and future card-generation parity
- the remaining combat timing gap is now mostly about when card-play reactions fire, not where played-card mutation is stored

This means the next highest-signal combat architecture blocker is now the long-tail use of hooks and empty programs, not the absence of the core play-phase split itself.

### 5. Choice handling is stack-based for run/reward/event flow, but combat still has a local simplification

The run/reward/event surface now uses a real decision stack, but combat choices still collapse to the single active choice model inside `CombatEngine`.

- [`src/engine.rs:345`](./src/engine.rs#L345)
- [`src/engine.rs:395`](./src/engine.rs#L395)

Why this matters:

- nested/generated combat choices are fragile
- search cannot cleanly reason about chained choice transitions

### 6. Events are typed and complete for supported content

There is now a typed event catalog and `RunEngine` executes typed event programs instead of matching the legacy `EventEffect` enum. The remaining problem is completeness: only one `EventProgramOp::blocked(...)` case remains, but it still covers an important run-edge semantic.

What improved in the recent waves:

- `Secret Portal` and `Wheel of Change` are now supported through the shared event runtime
- scripted event combat now exists as a reusable typed primitive with explicit on-win continuation
- event-generated reward choices now share the ordered reward runtime instead of bypassing it
- `Colosseum` now runs through shared sequential event continuation and multi-combat handling
- `Cursed Tome` now runs through shared multi-page event continuation with progressive HP-loss and final relic reward handling
- search hashing includes pending scripted event combat continuation so replay/search does not alias those branches

- [`src/tests/test_events_parity.rs:43`](./src/tests/test_events_parity.rs#L43)
- [`src/tests/test_events_parity.rs:68`](./src/tests/test_events_parity.rs#L68)
- [`src/tests/test_events_parity.rs:78`](./src/tests/test_events_parity.rs#L78)
- [`src/tests/test_events_parity.rs:83`](./src/tests/test_events_parity.rs#L83)
- [`src/tests/test_events_parity.rs:103`](./src/tests/test_events_parity.rs#L103)
- [`src/run.rs:1404`](./src/run.rs#L1404)

The remaining blocked-op set for supported content is now empty, including `Spire Heart`, which now runs through the canonical final-act resolution path instead of an honest blocker. `Dead Adventurer` now resolves its ascension-sensitive search roll through the canonical typed runtime instead of an opaque placeholder op.

### 7. Dead systems are reduced and mostly pruned, but one oracle family still remains

`dispatch_trigger()` is effectively superseded, and the remaining legacy helpers are now overwhelmingly oracle-only or dead-code cleanup rather than live production dependencies.

- [`src/effects/dispatch.rs:1`](./src/effects/dispatch.rs#L1)
- [`src/relics/mod.rs:11`](./src/relics/mod.rs#L11)
- [`src/relics/mod.rs:35`](./src/relics/mod.rs#L35)
- [`src/powers/mod.rs:26`](./src/powers/mod.rs#L26)

What improved in the recent waves:

- `Pen Nib` and `Velvet Choker` helper functions are gone from the relic oracle surface
- the engine-path relic bundle now covers `Orange Pellets`, `Pocketwatch`, `Pen Nib`, `Velvet Choker`, `Nunchaku`, `Ink Bottle`, `Happy Flower`, and `Incense Burner`
- `dispatch_modify_damage`, `dispatch_on_discard`, `dispatch_can_play`, `dispatch_modify_cost`, and live potion fallback callsites are gone from production
- `powers/hooks.rs` is deleted, and the old hook-table registry/helpers are removed from `powers/registry.rs`
- helper-path parity assertions for `Twisted Funnel`, `Snecko Eye`, `Sling`, `Preserved Insect`, `Du-Vu Doll`, `Girya`, `Red Skull`, `Teardrop Locket`, and `Pantograph` are now replaced by engine-path suites
- the final `Runic Pyramid` / `Unceasing Top` hand-lifecycle bridge helpers are now deleted from `relics/run.rs`
- remaining references are oracle assertions, ignored blocker tests, or dead exports only

## What Tests Already Prove

### Strong coverage now

- owner-aware combat runtime:
  - [`src/tests/test_entity_runtime.rs`](./src/tests/test_entity_runtime.rs)
- player power runtime:
  - [`src/tests/test_power_runtime_turn_start.rs`](./src/tests/test_power_runtime_turn_start.rs)
  - [`src/tests/test_power_runtime_card_play.rs`](./src/tests/test_power_runtime_card_play.rs)
  - [`src/tests/test_power_runtime_complex.rs`](./src/tests/test_power_runtime_complex.rs)
  - [`src/tests/test_power_runtime_end_to_end.rs`](./src/tests/test_power_runtime_end_to_end.rs)
  - [`src/tests/test_power_runtime_replay.rs`](./src/tests/test_power_runtime_replay.rs)
  - [`src/tests/test_power_runtime_debuff_enemy.rs`](./src/tests/test_power_runtime_debuff_enemy.rs)
- potion action/runtime path:
  - [`src/tests/test_potion_runtime_action_path.rs`](./src/tests/test_potion_runtime_action_path.rs)
- generated-choice parity:
  - [`src/tests/test_generated_choice_java_wave3.rs`](./src/tests/test_generated_choice_java_wave3.rs)
- orb/runtime parity:
  - [`src/tests/test_orb_runtime_java_wave1.rs`](./src/tests/test_orb_runtime_java_wave1.rs)
  - [`src/tests/test_card_runtime_defect_wave11.rs`](./src/tests/test_card_runtime_defect_wave11.rs)
  - [`src/tests/test_card_runtime_defect_wave14.rs`](./src/tests/test_card_runtime_defect_wave14.rs)
  - [`src/tests/test_card_runtime_defect_wave16.rs`](./src/tests/test_card_runtime_defect_wave16.rs)
- damage-followup parity:
  - [`src/tests/test_damage_followup_java_wave1.rs`](./src/tests/test_damage_followup_java_wave1.rs)
- RL combat contract and deterministic search scaffolding:
  - [`src/tests/test_rl_contract.rs`](./src/tests/test_rl_contract.rs)
  - [`src/tests/test_search_harness.rs`](./src/tests/test_search_harness.rs)
- reward runtime and ordered reward decisions:
  - [`src/tests/test_reward_runtime.rs`](./src/tests/test_reward_runtime.rs)
- dead-system cleanup coverage:
  - [`src/tests/test_dead_system_cleanup_wave22.rs`](./src/tests/test_dead_system_cleanup_wave22.rs)
- typed event catalog parity and blocked-branch reporting:
  - [`src/tests/test_events_parity.rs`](./src/tests/test_events_parity.rs)
  - [`src/tests/test_event_runtime_wave18.rs`](./src/tests/test_event_runtime_wave18.rs)
- newer card/runtime waves:
  - [`src/tests/test_card_runtime_backend_wave3.rs`](./src/tests/test_card_runtime_backend_wave3.rs)
  - [`src/tests/test_card_runtime_backend_wave1.rs`](./src/tests/test_card_runtime_backend_wave1.rs)
  - [`src/tests/test_card_runtime_backend_wave2.rs`](./src/tests/test_card_runtime_backend_wave2.rs)
  - [`src/tests/test_card_runtime_defect_wave1.rs`](./src/tests/test_card_runtime_defect_wave1.rs)
  - [`src/tests/test_card_runtime_defect_wave2.rs`](./src/tests/test_card_runtime_defect_wave2.rs)
  - [`src/tests/test_card_runtime_defect_wave3.rs`](./src/tests/test_card_runtime_defect_wave3.rs)
  - [`src/tests/test_card_runtime_defect_wave4.rs`](./src/tests/test_card_runtime_defect_wave4.rs)
  - [`src/tests/test_card_runtime_ironclad_wave1.rs`](./src/tests/test_card_runtime_ironclad_wave1.rs)
  - [`src/tests/test_card_runtime_ironclad_wave2.rs`](./src/tests/test_card_runtime_ironclad_wave2.rs)
  - [`src/tests/test_card_runtime_ironclad_wave3.rs`](./src/tests/test_card_runtime_ironclad_wave3.rs)
  - [`src/tests/test_card_runtime_ironclad_wave4.rs`](./src/tests/test_card_runtime_ironclad_wave4.rs)
  - [`src/tests/test_card_runtime_ironclad_wave7.rs`](./src/tests/test_card_runtime_ironclad_wave7.rs)
  - [`src/tests/test_card_runtime_silent_wave1.rs`](./src/tests/test_card_runtime_silent_wave1.rs)
  - [`src/tests/test_card_runtime_silent_wave2.rs`](./src/tests/test_card_runtime_silent_wave2.rs)
  - [`src/tests/test_card_runtime_silent_wave3.rs`](./src/tests/test_card_runtime_silent_wave3.rs)
  - [`src/tests/test_card_runtime_silent_wave7.rs`](./src/tests/test_card_runtime_silent_wave7.rs)
  - [`src/tests/test_card_runtime_watcher_wave1.rs`](./src/tests/test_card_runtime_watcher_wave1.rs)
  - [`src/tests/test_card_runtime_watcher_wave2.rs`](./src/tests/test_card_runtime_watcher_wave2.rs)
  - [`src/tests/test_card_runtime_watcher_wave3.rs`](./src/tests/test_card_runtime_watcher_wave3.rs)
  - [`src/tests/test_card_runtime_watcher_wave4.rs`](./src/tests/test_card_runtime_watcher_wave4.rs)
  - [`src/tests/test_card_play_timing_java_wave1.rs`](./src/tests/test_card_play_timing_java_wave1.rs)
- relic runtime threshold coverage:
  - [`src/tests/test_relic_runtime_wave3.rs`](./src/tests/test_relic_runtime_wave3.rs)
  - [`src/tests/test_relic_runtime_wave4.rs`](./src/tests/test_relic_runtime_wave4.rs)
  - [`src/tests/test_relic_runtime_wave5.rs`](./src/tests/test_relic_runtime_wave5.rs)
- dead-system cleanup coverage:
  - [`src/tests/test_dead_system_cleanup_wave1.rs`](./src/tests/test_dead_system_cleanup_wave1.rs)
- runtime inline cutover coverage:
  - [`src/tests/test_runtime_inline_cutover_wave1.rs`](./src/tests/test_runtime_inline_cutover_wave1.rs)
  - [`src/tests/test_runtime_inline_cutover_wave2.rs`](./src/tests/test_runtime_inline_cutover_wave2.rs)
  - [`src/tests/test_runtime_inline_cutover_wave3.rs`](./src/tests/test_runtime_inline_cutover_wave3.rs)
- newer event/runtime coverage:
  - [`src/tests/test_event_runtime_wave2.rs`](./src/tests/test_event_runtime_wave2.rs)
  - [`src/tests/test_event_runtime_wave3.rs`](./src/tests/test_event_runtime_wave3.rs)
  - [`src/tests/test_event_runtime_wave4.rs`](./src/tests/test_event_runtime_wave4.rs)
  - [`src/tests/test_event_runtime_wave5.rs`](./src/tests/test_event_runtime_wave5.rs)
  - [`src/tests/test_event_runtime_wave9.rs`](./src/tests/test_event_runtime_wave9.rs)
- newer potion/runtime coverage:
  - [`src/tests/test_potion_runtime_wave2.rs`](./src/tests/test_potion_runtime_wave2.rs)
  - [`src/tests/test_potion_runtime_wave3.rs`](./src/tests/test_potion_runtime_wave3.rs)
  - [`src/tests/test_potion_runtime_wave4.rs`](./src/tests/test_potion_runtime_wave4.rs)
  - [`src/tests/test_potion_runtime_wave5.rs`](./src/tests/test_potion_runtime_wave5.rs)
- decompile-backed focused suites:
  - [`src/tests/test_card_runtime_silent_wave_java1.rs`](./src/tests/test_card_runtime_silent_wave_java1.rs)
  - [`src/tests/test_generated_choice_java_wave1.rs`](./src/tests/test_generated_choice_java_wave1.rs)
  - [`src/tests/test_relic_runtime_java_green1.rs`](./src/tests/test_relic_runtime_java_green1.rs)
  - [`src/tests/test_defect_java_wave1.rs`](./src/tests/test_defect_java_wave1.rs)
  - [`src/tests/test_generated_choice_java_wave3.rs`](./src/tests/test_generated_choice_java_wave3.rs)
- power metadata and legacy-cutover coverage:
  - [`src/tests/test_power_runtime_metadata_wave1.rs`](./src/tests/test_power_runtime_metadata_wave1.rs)

### Coverage that still does not prove full parity

- `Dead Adventurer` is now behavioral:
  - the typed program skeleton, runtime normalization, and ascension-sensitive first search roll all run through the canonical typed path
- a short card-tail still relies on hooks or empty programs:
  - the remaining files are concentrated in discard sequencing, orb-ordering, damage-follow-up, and Colorless utility/scaling families rather than broad architectural gaps

## Missing Tests We Need Next

These are the next required engine-path scenarios. Each one corresponds to a known architectural gap.

- `reward_screen_black_star_second_elite_relic_survives_first_claim_and_mutates_next_item_order`
  - elite multi-relic ordering needs broader coverage
- `reward_screen_matryoshka_chest_rewards_use_same_item_runtime`
  - chest rewards should stop being a separate future system
- `nested_choice_stack_preserves_generated_choice_order`
  - choice sequencing must be explicit and replayable
- `dead_adventurer_first_search_roll_uses_ascension_sensitive_base_chance`
  - the final supported-event blocker should stay explicit until the shared search-state primitive lands
- `event_card_reward_uses_event_reward_screen_and_not_flat_deck_gain`
  - event card rewards should use the same ordered screen as combat rewards
- `reward_screen_active_item_is_visible_in_rl_context_and_features_after_open`
  - guards the exact reward-surface regression fixed in this wave

## Dead-System Removal Ladder

### Ready to remove soon

- `src/effects/dispatch.rs`
- selected dead exports in `src/powers/registry.rs`
- selected legacy compatibility exports in `src/powers/mod.rs`

Precondition:

- grep confirmation that production no longer calls them
- any useful unit coverage moved to runtime-path tests
- no broad audit depends on them as the primary parity signal

## Next Worker Waves

### Wave 1: small remaining card primitives

Own:

- `src/cards/colorless/**`
- `src/cards/ironclad/**`
- `src/cards/silent/**`
- shared effect/runtime files only for the owned primitive
- wave-specific Java-backed suites

Goal:

- finish the remaining low-risk utility/damage-follow-up cards that no longer need big architectural work
- keep `Ritual Dagger`, `Nightmare`, and the remaining large sequencing cards queued behind their actual missing primitives

### Wave 2: final supported-event blocker

Own:

- `src/events/exordium.rs`
- `src/run.rs`
- event engine-path tests

Goal:

- `Dead Adventurer` is complete on the supported typed path, including the ascension-sensitive first-search roll and existing search ramp / reward-order skeleton

### Wave 3: broad audit before final parity sweep

Own:

- `AUDIT_PARITY_STATUS.md`
- `DECOMPILE_PARITY_ENDGAME.md`
- representative focused suites across cards, relics, events, potions, and RL reward/observation surfaces

Goal:

- recount the true tail after the next 1-2 landings
- make the final broad parity sweep honest and targeted rather than exploratory

Goal:

- retire the ignored `Emotion Chip`, `Liquid Memories`, `Blizzard`, `Fission`, and `Scrape` blocker cases
- continue shrinking the `complex_hook` tail by adding shared primitives instead of bespoke patches

### Wave 4: final dead-export cleanup

Own:

- `src/effects/dispatch.rs`
- `src/powers/registry.rs`
- `src/powers/mod.rs`
- any remaining dead-export cleanup tests

Goal:

- delete dead oracle-era exports once the broad audit confirms they are no longer primary parity evidence
- reduce residual dead-code warning surfaces without reintroducing helper-path logic

## Latest Confirmed Gaps

These are the remaining coordinator-confirmed blockers on the integrated branch.

### No live production legacy blockers remain

- Production no longer calls legacy dispatch helpers for potion fallback, damage modification, card cost/legality, or card-play relic checks.
- The remaining legacy code is oracle-only, helper-test-only, or dead-export cleanup.

### Legacy kept as oracle only

- `src/powers/registry.rs`
  - Reduced to live production query helpers only (`status_is_debuff`, `active_player_power_count`) plus any still-needed residual exports.
  - Removal precondition: those remaining live query helpers are moved or inlined into canonical runtime-owned homes.

### Current queued blockers, not red integrated tests

- `src/tests/test_orb_runtime_java_wave1.rs`
  - `Emotion Chip` and `Liquid Memories` still need richer orb/choice timing primitives and remain explicit `#[ignore]` blockers.
- `src/tests/test_power_runtime_debuff_enemy.rs`
  - Still carries a legacy Time Warp expectation and should be migrated to the Java oracle at `decompiled/java-src/com/megacrit/cardcrawl/powers/TimeWarpPower.java`.
- `src/events/exordium.rs`
  - `Dead Adventurer` now runs on the ascension-sensitive first-search-roll path, and blocked event ops remain at zero for supported content.
