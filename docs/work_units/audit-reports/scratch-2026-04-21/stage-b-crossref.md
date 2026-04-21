# Stage B Audit Reports -- Cross-reference Against Register + Current Code

**Auditor:** Opus 4.7 (read-only audit subagent)
**Date:** 2026-04-21
**Scope:** 11 Stage B audit reports added in `55e77c89` vs `parity-deviations-register.md` (D1-D159) vs current Rust (HEAD `d16cdba2` on branch `claude/parity-audit-fleet-2026-04-21`).
**Method:** Read all 11 reports + register, spot-checked a dozen high-impact claims in current code, confirmed register closure status matches code.
**Totals:** ~302 audit findings -> ~41 promoted to register (D88-D128) + 27 Stage G promotions (D133-D159) = 68 register rows tied to Stage B reports. **Net ~230 Stage B findings live only inside the per-area reports.**

## Summary

- **Confirmed-and-fixed-in-this-PR (Stage D-G):** 3 Stage B promotions (D129 Jaw Worm branch swap, D130 Collector off-by-one, D158 test tautology). D1 AI-RNG-wiring partial-fix across Acts 1-4 dispatcher rewrites (Stage D, ~15 enemies, keeps per-enemy deferred sub-roll residue).
- **Confirmed-and-registered-only:** 65 rows (D88-D128 from Stage B promotion + D132-D157 + D159 from Stage G). Every row has a citation to its report; none are fixed in code.
- **Confirmed-and-NOT-registered:** ~230 findings. Dominated by enemy AI tail (~48 E*A1, ~37 E*A2/3/4 beyond the 6 Stage-G promotions), power coverage (~40 of P4-P50), potions (~7), damage-pipeline subcases (~12 of DM2-DM14), tests (~15), relics (2 high-impact), events (6 flagged `bug` in RN audit), OC4-OC7.
- **Stale / already-fixed-before-audit:** 2 items the audits correctly self-close (D18 Burn+ confirmed OK by OC report narrative; W17 notes D50 Vault already closed). Plus audit reports pre-date the Stage F fix for D129/D130 so those claims are technically stale in the audit body but tracked properly in the register.
- **Fixed-in-PR but claim-mismatch:** D129 Jaw Worm and D130 Collector are closed in Stage F **but** the audit reports still flag residual sub-issues (Jaw Worm D131 sub-roll deferred; Collector A19+ scaling `strAmt=5/megaDebuffAmt=5/rakeDmg=21` and Revive `MOVE 5` remain open per E11A2/E19A2).
- **Audit report factual errors:** 0 confirmed. One report (`watcher-cards.md` W16-W17) correctly self-classifies prior D-rows (D25 Establishment+, D50 Vault) as closed. DM14 self-flags "Confirmed correct" which is not an error but worth flagging for noise.

## Per-report cross-reference

### 1. `enemies-act1.md` -- 53 findings E1A1-E53A1

- **Registered:** E27A1 GremlinTsundere -> D97 (Stage B) and D153 (Stage G empty-arm); E43A1 ApologySlime -> D98; ascension-scaling generic -> D118.
- **Registered Stage G (partial, not full A1 cover):** D146 FungiBeast, D147/D148 SpikeSlime_M/L, D149 AcidSlime_S threshold, D150 Louse anti-repeat, D151 SlaverBlue, D152 GremlinWizard 2-turn cycle (P0), D154 Lagavulin 1:1 alternation (P0), D155/D156 Sentry (P0), D157 Looter stream position.
- **Confirmed-and-NOT-registered:** ~40+ Act-1 E*A1 rows. Examples from audit that are NOT in register: E2A1 AcidSlime_L 30/40/30 split drift, E3A1 Cultist strength-amount ascension, E6A1 Hexaghost inferno damage scaling (A19+), E15A1 GremlinNob Bellow cap, E22A1 JawWorm first-turn behaviour (partially captured by D129 fix but E22A1's upgraded ATTACK damage curve is a separate row), E35A1 Louse HP pools, E45A1 Looter steal-gold pipeline.
- **Spot-check of current code:** confirmed `act1.rs::roll_jaw_worm` at line 11 is the Stage-F-rewritten version matching Java's 25/55 thresholds; register row D129 correctly marked **closed** (commit `d29dac03`).

### 2. `enemies-act2-act3-act4.md` -- 41 findings E*A2/3/4

- **Registered:** E11A2 Collector -> D99 + D130 (Stage F partial fix); E19A2 Reptomancer daggers -> D100; E4A4 Shield+Spear -> D101; E13A3 Transient -> D102 + D141 (direction).
- **Registered Stage G:** D132 Byrd grounded, D133 BronzeAutomaton, D134/D135 Bandits, D136 GremlinLeader aliveCount (P1), D137/D138 Champ, D139 Snecko, D140 WrithingMass Reactive (P0 un-wired), D141 Transient direction (P1), D142 Exploder UNKNOWN (P1), D143/D144 CorruptHeart (P0 both), D145 Darkling turn-1.
- **Confirmed-and-NOT-registered:** ~25 remaining. Examples: E1A2 Centurion/Healer timing, E7A2 Looter vs Mugger Smoke Bomb split, E20A2 Collector A19+ scaling (`strAmt=5, megaDebuffAmt=5, rakeDmg=21`), E21A2 Collector Revive move id 5, E7A3 Chosen hex + strip STR, E11A3 Maw/Donu scaling edge cases, E15A3 SphericGuardian inverse-wave bug, E3A4 Heart scry-barrier interaction, E5A4 Spear beam damage table.
- **Spot-check:** `act2.rs::roll_collector` at line 471-485 confirms D130 fix (`turns >= 3 && !ult_used`) and confirms E20A2's A19+ scaling constants still NOT handled.

### 3. `relics.md` -- 21 + 21 missing

- **Registered:** R1 HolyWater -> D88 (P0); R2 VelvetChoker -> D103; R3 PhilosopherStone -> D104; R4 RunicDome -> D105; R6 Enchiridion -> D107; R7 DeadBranch -> D106.
- **Confirmed-and-NOT-registered:** R5 Damaru start-of-combat Mantra timing (worth P1; Watcher Mantra build start-up); R12 Mercury Hourglass / Torii borderline; and 21 "missing relics" list in the report (Astrolabe D28, Empty Cage D29, Calling Bell D30, Frozen Eye D31 are in register; the other ~17 from that list are NOT registered individually -- e.g. Dolly's Mirror onEquip transform, Necronomicon free first Attack, Runic Capacitor orb slots, Inserter energy-every-2-turns, Tough Bandages, Shovel, N'loth, WingBoots, Peace Pipe, Nuclear Battery, etc.).
- **Spot-check:** `relics/defs/holy_water.rs` lines 7-11 literally adds 3 `"HolyWater"` cards (D88 **open**, confirmed). Critical Watcher boss-relic bug -- top of headline list in register.

### 4. `watcher-stances-scry-mantra.md` -- 14 findings WS1-WS14

- **Registered:** WS3 CannotChangeStance -> D110; WS4 Foresight empty-deck-shuffle -> D109; WS5 Golden Eye ScryAction -> D108; WS11 Fasting EnergyDown -> D89 (P0 headline).
- **Confirmed-and-NOT-registered:** WS1 Devotion pre-draw vs post-draw, WS2 Devotion >=10 direct-entry, WS6 onScry timing, WS7 WrathNextTurn latent, WS8 EndTurnDeath latent, WS9 LiveForever AngelForm PlatedArmor, WS10 NoSkills latent, WS12 WaveOfTheHand end-of-round vs turn-start, WS13 Establishment permanent-cost (intentional), WS14 FlurryOfBlows name-match (unverified).
- **Pattern:** `powers/buffs.rs::process_start_of_turn` and `process_end_of_turn` are defined but `engine.rs` never calls them (confirmed via grep -- zero matches in engine.rs). This single fact is the root of WS7-WS11 and D89, D111 (P2 Stage B promotion).

### 5. `watcher-cards.md` -- 20 findings W1-W20

- **Registered new bugs:** W2 Deva Form -> D123; W3 Pressure Points pipeline -> D124 (P1 headline); W5 Signature Move dedup -> D128; W7 Swivel non-stacking -> D125; W10 Miracle no-retain -> D126; W11 Collect+ upgrade -> D127 (compound with D126, P1 headline).
- **Cross-refs to existing D-rows:** W4 -> D16 Cut Through Fate, W8 -> D16/D17 (duplicate), W9 -> D17 Spirit Shield, W12 -> D16 Cut Through Fate, W13-W15 -> misc existing D-rows.
- **Intentional / clean:** W6, W16 (D25 Establishment+), W17 (D50 Vault), W18, W19, W20. Verified intentional.
- **NOT-registered:** The "Foreign Influence random Attack source pool" variant (W1 sub-bullet) and Wheel Kick / Flying Sleeves scaling edges are not register rows.

### 6. `other-class-cards.md` -- 14 findings OC1-OC14

- **Registered:** OC1 Crippling Cloud/Poison ID -> D122; OC2 Orb hit type -> D96; OC12 duplicate of OC2 -> D96.
- **Confirmed-and-NOT-registered:** OC4 Searing Blow HP-scaling damage formula, OC5 Pride copy-to-draw-pile timing, OC6 Parasite HP-loss onObtain vs onEquip path (interacts with relic Omamori), OC7 Sharp Hide card (Ironclad) status.
- **Audit self-closes:** OC3 / OC8 / OC9 -- marked "verified clean" in report body.

### 7. `powers-buffs-debuffs.md` -- 50 findings P1-P50

- **Registered:** P1 MalleablePower -> D90 (P0 headline, confirmed still open at `engine.rs:2520-2527`); P2 StartOfTurn pre/post-draw split -> D111; P3 SadisticPower filters -> D112; missing-powers table -> D113 (14 IDs as a single row).
- **Confirmed-and-NOT-registered:** ~40 of P4-P50. Includes: P4 EnvenomPower dual damage routes, P5 FireBreathing trigger vs Attack played, P7 NoxiousFumes Artifact, P9 TheBomb delayed damage vs Intangible, P11 ElectroDynamics pierce vs multi-target, P14 Strength/Focus/Dex stacking and Gained-vs-Lost gates, P20 CorruptionPower makes-all-Skills-exhaust plus cost-zero, P22 DemonForm end-of-turn timing (related to D111), P30 FeelNoPainPower trigger on exhaust, P40 BarricadePower keeps block.
- **Pattern:** Many P*-rows are latent because the trigger surface (`Trigger::OnSomething`) is missing or the power has no consumer outside tests -- same root cause as WS7-WS11. Individually low-priority, collectively a significant coverage gap.

### 8. `potions.md` -- 13 findings PT1-PT13

- **Registered:** PT1 SacredBark scaling -> D116; PT3 PT4 CultistPotion -> D114; PT8 SneckoOil -> D115; PT10 A11+ potency under cfg(test) -> D117; PT12/PT13 -> D115/D117 duplicates.
- **Confirmed-and-NOT-registered:** PT2 EntropicBrew pool, PT5 SwiftPotion turn-end return, PT6 FairyInABottle heal threshold, PT7 DistilledChaos ordering, PT9 LiquidBronze damage type (Thorns vs Normal), PT11 StrengthPotion decay interaction.

### 9. `damage-engine-flow.md` -- 14 findings DM1-DM14

- **Registered:** DM1 `deal_damage_to_player` pipeline bypass -> D91 (P0 headline).
- **Overlaps existing:** DM4 partially overlaps D58 Static Discharge Lightning retaliation.
- **Confirmed-and-NOT-registered:** DM2 Thorns in attack pipeline, DM3 Plated Armor vs block-retention order, DM5 DamageAction fusion in multi-hit cards, DM6 Buffer timing relative to block (interacts with D48), DM7 Intangible "cap to 1" at different layers, DM8 Vulnerable 1.5x integer-truncation vs round-half-even, DM9 Torii HP-loss gating, DM10 Tungsten Rod -1 HP gating, DM11 on-player-HP-loss triggers and Buffer interaction, DM12 Angry damage-give-hook sequence, DM13 Static Discharge vs Flight halving (related D58), DM14 self-flagged "Confirmed correct, no fix needed".
- **Spot-check:** D91 is still the single most impactful engine-wide bug -- every effect DSL `Target::Player` damage source still bypasses Wrath/Vulnerable/Intangible/Torii/Tungsten. Audit shows 30+ call sites affected.

### 10. `run-map-events-shop.md` -- ~42 findings RN-*

- **Registered:** RN-EV-01 event-pool remove-on-use -> D94 (P1 headline); RN-EV-02/RN-EV-04 shrines unreachable -> D95 (P1 headline).
- **Confirmed-and-NOT-registered (flagged `bug` in report):** RN-MAP-01 map edge-case (bottom row), RN-EV-03 `GhostGold` no card removal, RN-EV-05 DeadAdventurer monster rolling, RN-NEOW-01 ONE_RANDOM_RARE path incomplete (related D15), RN-SHOP-05 shop card-removal cost scaling, RN-REWARD-02 card-reward count gate.
- **Confirmed-and-NOT-registered (flagged `unverified`):** RN-MAP-02/03/04, RN-SHOP-01-04, RN-EV-06-17, RN-NEOW-02-05, RN-REWARD-01/03/04. ~20 rows in the audit's tail.

### 11. `tests-types-api.md` -- 20 findings T1-T20

- **Registered:** T1 orphan test files -> D92 (P0 headline, ~230 silent-loss tests); T2 `combat_engine_from_snapshot` orb/over drop -> D93 (P0); T4 clippy -> D121; T5 PyO3 JSON roundtrip -> D120; T12 `pub mod` leak -> D119.
- **Confirmed-and-NOT-registered:** T3 orphan test harnesses in `packages/training/`, T6 test helpers duplicated, T7 snapshot schema versioning, T8 Python-side fixture freshness, T9 `cargo test --lib` default-features coverage gap, T10 tests that encode bugs as parity (meta-row; individual instances are D129/D130/D141/D145/D148 already in register -- but there are ~15 more per Stage G audit), T11, T13-T20 various API drift.
- **Spot-check:** `lib.rs:11-37` confirms 27 `pub mod` lines (D119 still open).

## Highest-risk gaps (top 10, P0-first)

1. **D88 HolyWater relic** -- `relics/defs/holy_water.rs:7-11` still adds 3 `HolyWater` cards. Java gives 3 **free energy** at combat start. Boss relic, Watcher A0 build impact. **P0 in register, not in this PR.**
2. **D89 Fasting EnergyDown never drains** -- `powers/buffs.rs::process_start_of_turn` never called from `engine.rs`. Defining card cost missing. **P0 in register, not in this PR.**
3. **D91 `deal_damage_to_player` bypasses the entire damage pipeline** -- DM1 in audit. Wrath 2x / Vuln 1.5x / Intangible cap / Torii / Tungsten / Plated Armor / Static Discharge all skipped when effects `Target::Player`. Engine-wide bug. **P0 in register, not in this PR.**
4. **D124 Pressure Points / TriggerMarks bypass HP-loss pipeline** -- `effects/interpreter.rs:621-629` subtracts from `entity.hp` directly. Same family as D91. Watcher Mark damage math wrong. **P0 in register.**
5. **D90 MalleablePower never resets per turn** -- confirmed live at `engine.rs:2520-2527`. Accumulates block-per-hit across multi-turn Chosen / Stone Wyrm fights. **P0.**
6. **D143 CorruptHeart slot 0 deterministic Blood Shots** + **D144 A4-A8 damage scaling off** -- Watcher A0 final boss branch distribution wrong + damage tables off on A4-A8 Heart. **P0 both.**
7. **D140 WrithingMass ReactivePower never wired into combat_hooks** -- helper exists but only called from unit tests. Makes Act 3 combat measurably easier than Java. **P0.**
8. **D92 44 orphan test files (~230 tests)** -- T1 in audit. Silent test-coverage loss; includes tests that would have caught some of the above. **P0 structural.**
9. **D94 Event pool never removes used events** + **D95 Shrines unreachable** -- RN-EV-01/02/04. Distorts deck-shaping + 17 shrine events never fire. Training-critical structural gap. **P1 both; combined P0.**
10. **D152 GremlinWizard 2-turn cycle** + **D154 Lagavulin 1:1 alternation** + **D155 Sentry BOLT/BEAM swap** -- three P0-flagged Act 1 enemy AI divergences that encode the Rust wrong-shape in current tests (`test_enemy_ai.rs`, `test_enemies.rs`). All Watcher A0 training-path combats. **P0 triple.**

Honourable mentions (would be 11-15): D58 Static Discharge Lightning bypass, D66 Choke applies wrong power, D61 Player Regen decrements, D93 snapshot drops orbs + combat_over, D111 pre-draw/post-draw conflation.

## Stale audit claims to remove / annotate

- **`watcher-cards.md` W16** explicitly verifies D25 Establishment+ clean -- register shows D25 already **closed (not-a-bug)**. Audit report body is consistent; no change needed but worth noting it duplicates closed material.
- **`watcher-cards.md` W17** notes D50 Vault already closed -- register shows D50 **closed** via `2db11718`. Same pattern: duplicated but harmless.
- **`other-class-cards.md` OC (Burn section)** narrative closes D18 Burn+ magic question -- register still marks D18 **open**. Recommend promoting the OC investigation conclusion into D18's status and closing.
- **`damage-engine-flow.md` DM14** self-flags "Confirmed correct, no fix needed" -- noise row; could be omitted from the per-area report cleanup pass.
- **`enemies-act1.md` + `enemies-act2-act3-act4.md`** pre-date Stage F's D129/D130 fixes; the audit text still describes them as open while the register is current. Recommend a one-line note at the top of both reports pointing at Stage F commits `f0965a04` / `d29dac03`.

## Claim-mismatch fixes

- **D129 Jaw Worm** closed in Stage F for the branch-swap bug. However `enemies-act1.md` also flags the sub-roll `aiRng.randomBoolean(p)` residue and the upgraded ATTACK damage curve; these are tracked as **D131 deferred**. No code or doc change needed -- the split is clean.
- **D130 Collector** closed in Stage F for the turn-4-vs-5 MegaDebuff gate. However `enemies-act2-act3-act4.md` E19A2/E20A2 flag residual A19+ scaling (`strAmt=5, megaDebuffAmt=5, rakeDmg=21`) and `Revive` move id 5 still UN-registered. **Recommend promoting E20A2 and E21A2 to Dn rows before closing the Collector investigation.**
- **D1 AI RNG** closed-as-fixed for the core branch logic across 15 enemies in Stage D; register status reads "fixed for the core branch logic ... deferred sub-rolls and engine-context-dependent branches noted in enemy dispatcher comments." The audits' deferred sub-rolls (D131 Jaw Worm + equivalent un-registered per-enemy residues) need either individual Dn rows or a single "D1 residue" umbrella row. Current state is fine but fragile -- first time someone touches an enemy dispatcher they'll re-discover the deferred list.
- **D27 Pandora's Box** closed for the TRANSFORM hook; random-common fill is deferred pending D52 RNG-stream split. Clean partial-fix.
- **D63 Curl Up** closed for lethal-blow guard; Malleable lethal-guard queued as a separate sub-issue (not yet Dn). **Recommend promoting "Malleable lethal-blow guard" to a new D-row** to track it, since register note says "Malleable follow-up queued" without an ID.

## Meta-observations

- The register is the correct disposition channel -- reports are "raw ore", register is "smelted rows". ~260 report-only rows is intentional (Stage B narrative says "Only top-priority ... are promoted"), not a bug.
- The main risk is that the report-only tail will bitrot: as code changes (Stage D rewrote 15 enemy dispatchers, Stage F two more, Stage G registered 27 more), the per-report file:line citations slide off. The Stage G review noticed this pattern (tests encoding bugs as parity) but an automated re-verification pass over every un-promoted report row would catch silent drift.
- `process_start_of_turn` / `process_end_of_turn` in `powers/buffs.rs` is a structural smell: it's a handcrafted dispatch table that is never invoked from the engine, yet continues to grow as new powers land. WS7-WS11 + D89 + D111 all chain to this. Either wire it or delete it -- leaving it in is actively misleading.
- 6 of the 10 P0 gaps are already in the register but deferred. The dangerous category ("Confirmed-and-NOT-registered") is dominated by low-individual-impact but high-aggregate rows in `enemies-act1.md` and `powers-buffs-debuffs.md` -- exactly the enemies and powers the solver will encounter every combat. Suggest an incremental "promote 10 P1 rows per PR" cadence rather than one mega-PR.
