<task>
Wave 2: make a complete Watcher A0 run (Neow → Heart) fully expressible, executable, and trace-mappable in the Rust engine, action-by-action. Base: branch `claude/engine-package` (2,927 lib tests green). Create `codex/run-vocab-wave2` from it and work there.

Ground truth: `decompiled/java-src/com/megacrit/cardcrawl/` (distillation: `reference/extracted/methods/`). Register refs: EDA-023 (trace replay cannot express a full run), R4/R5 in `docs/work_units/sim-completion-map.md`, FINDINGS F7 (harness contract nits). This wave is the ENGINE side of the script contract; the Java mod side is scoped separately and must not be touched (`packages/harness-java/` is read-reference only this wave).

Motivating constraint: human game sessions are the scarcest resource in this project, and past errors have mostly been Java-misreading errors. Therefore every behavior lands verification-first — proven against re-derived decompiled-Java values BEFORE it can ever waste a human minting session.

Scope, in order:

1. CANONICAL ACTION VOCABULARY (script schema v2). Define the complete typed action set covering every decision a player makes in a whole run. Start from the current `RunAction` and the T2 example in `docs/goal/TOOLING.md`, then enumerate against Java screens until exhaustive — at minimum: NEOW, PATH, PLAY_CARD, END_TURN, USE_POTION, DISCARD_POTION, REWARD_TAKE / REWARD_SKIP (gold, potion, relic, key, card-reward entry), CARD_REWARD pick / skip (+ Singing Bowl), CAMPFIRE (REST / SMITH / LIFT / TOKE / DIG / RECALL), SHOP_BUY (card / relic / potion slots), SHOP_REMOVE, SHOP_LEAVE, EVENT_CHOICE, CHEST_OPEN / CHEST_SKIP, BOSS_RELIC pick / skip, PROCEED/transition where Java requires an explicit confirm. Write the schema down in `docs/goal/TOOLING.md`-compatible form as `docs/work_units/script-schema-v2.md` — field names, types, and the Java screen each action drives. This document is the shared contract the Java mod will implement later; treat naming stability as an API commitment.

2. SHOP SHAPE. Full Java `ShopScreen`/`Merchant` inventory generation and pricing: 5 colored cards (rarity rolls, duplicate rules, ordering), colorless slot(s), 3 relics including the shop-relic slot, 3 potions, purge pricing and availability, sale card discount, Membership Card / Courier / Smiling Mask interactions (some already exist — verify, don't assume), and exact `merchantRng` consumption order. Cite `ShopScreen.java`, `StoreCreature` sources.

3. REWARD SHAPE. Combat reward generation per Java `AbstractRoom`/`AbstractDungeon`: gold ranges by room type and ascension, card reward count/rarity modifiers (Question Card, Busted Crown, Prayer Wheel second reward, Nloth's Gift odds), the potion-drop chance chain and its rng accounting, elite relic drops, emerald key on burning elite, sapphire key at chests, ruby key via campfire RECALL. Exact reward ordering in the reward screen list.

0. P0 CARRY-IN — Secret Portal floor desync (found by independent verification of wave 1). `run.rs:4514` and `run.rs:8113-8115` force `run_state.floor = boss_floor_for_act(act)` when the Act 3 Secret Portal event starts the boss combat. Java `SecretPortal.java:63-75` never touches `floorNum` — it only redirects `nextRoom` to a `MonsterRoomBoss`; the floor increments normally (+1 from the event's actual floor) and the per-floor combat streams reseed from `Settings.seed + floorNum` per `AbstractDungeon.java:1737-1741`. Fix: stop forcing the floor; let the natural increment drive the reseed. Strengthen `secret_portal_transitions_into_boss_combat` (test_event_runtime_wave9.rs) to trigger the portal from a mid-act floor and assert the boss combat seeds from event_floor+1, not 50.

4. BOSS SHAPE. Boss chest (3 boss relics, pick one or skip), act transition per Java (full heal rules and ascension modifiers, which RNG streams carry over vs re-seed — wave 1 landed part of this in commit 928381cc; verify and extend), Act 3→4 gating on all three keys, Act 4 room sequence (elite, campfire, Heart), and A20 double-boss ordering (already tested — extend if gaps).

5. TRACE MAPPER COMPLETENESS. Every action type in the v2 schema maps 1:1 in `trace.rs` both directions; `trace_replay` accepts a full-run script with zero "unsupported action" rejections. Reconcile the F7 nits in the Rust-side types (CAMPFIRE choice as string, `max_actions` vs `max_floor` semantics) and record the decision in the schema doc.

6. LEGAL-ACTION ENUMERATION. For every new phase/action, `legal_actions` (and the decision stack) exposes exactly the legal set, and illegal actions are rejected deterministically WITHOUT consuming RNG.
</task>

<verification_first_protocol>
Per item, in this order — no exceptions:
1. Read the Java; write the expected behavior as source-derived test values WITH the citation.
2. INDEPENDENT RE-DERIVATION: before implementing, re-read the cited Java a second time specifically hunting for misreads (wrong overload, ascension branch, iterator order, rng call inside a condition). Record in the test comment: "re-verified: <one line on the subtle point>". If the two readings disagree, resolve against the source before writing any engine code.
3. Implement through the production engine path.
4. Test passes; full suite green.
This replaces trusting first readings — the project's error history is Java-misreading, not Rust bugs.
</verification_first_protocol>

<gates>
- G1: `./scripts/test_engine_rs.sh test --lib` green after every commit; count only rises from 2,927.
- G2: a new integration test drives a complete synthetic Watcher A0 run Neow→Heart on a fixed seed purely via v2 typed actions — terminates in victory or death with zero unsupported-action errors and zero panics; a second test proves determinism (same seed + actions twice → byte-identical final state and RNG counters).
- G3: `trace_replay` parses and replays a full-run v2 script end-to-end (script may be the synthetic one; no golden required this wave).
- G4: bench bounds hold: full_turn_cycle < 5.2µs, clone_for_mcts < 800ns, get_legal_actions < 130ns (DYLD_FRAMEWORK_PATH=/Applications/Xcode.app/Contents/Developer/Library/Frameworks).
- G5: hygiene — cargo check --all-targets clean; protected paths untouched (`data/traces/java/`, `decompiled/`, `packages/training/`, `packages/harness-java/`, `logs/`, `runs/`, `docs/goal/` except FINDINGS cross-refs); every behavior commit cites Java file+lines.
</gates>

<anti_gaming_rules>
- Never weaken or delete an existing test without a contradicting Java citation in the same commit.
- The G2 integration test must make real decisions (take cards, buy, rest, fight) — a run that skips every reward and never shops does not satisfy the gate.
- Schema naming in script-schema-v2.md may not silently drift from the implemented types; the doc is generated-or-checked against the Rust enum in a test.
</anti_gaming_rules>

<judge_pass>
Final ~15% of budget: independently re-run G1–G5, write `docs/work_units/audit-reports/wave2-scorecard.md` (same format as wave 1: per-item FIXED-PROVEN / ATTEMPTED-REVERTED / NOT-ATTEMPTED table with exact commands and exit codes, corrections section, morning handoff with the 3 highest-value next actions). Revert anything FIXED-UNPROVEN before finishing. The scorecard is the final commit.
</judge_pass>

<default_follow_through_policy>
Do not stop for routine questions. Blocked items get documented and skipped. Halt only if the base branch is broken.
</default_follow_through_policy>
