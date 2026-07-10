# Watcher Cards Parity Audit (Rust vs Java)

Rust dir: `packages/engine-rs/src/cards/watcher/` (83 modules)
Java dir: `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/` (77 files) + `cards/tempCards/` (Beta, Omega, Miracle, Insight, Smite, Safety, Expunger, ThroughViolence)
Unified starters: `packages/engine-rs/src/cards/starters.rs` (`Strike`, `Defend`)
Cross-reference: `docs/work_units/parity-deviations-register.md` (D1–D87)

Severity vocab: `bug` = real runtime divergence; `deferred` = intentional simplification; `intentional` = known approximation for MCTS; `unverified` = needs a runtime test to confirm.

---

## Findings (W-rows)

| ID | Card | Severity | Rust ref | Java ref | Summary | Cross-ref |
|---|---|---|---|---|---|---|
| W1 | Strike / Defend (unified Watcher basics) | intentional | `packages/engine-rs/src/cards/starters.rs:27-93` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Strike_Purple.java` + `Defend_Watcher.java` | Rust collapses the 4 class-specific copies of Strike/Defend into a single pair; upgrade deltas (6→9 dmg, 5→8 block, cost 1) match Watcher's copy exactly. `is_strike()`/`is_defend()` substring helpers preserve starter-only semantics. Watcher-only training horizon makes this safe. | — |
| W2 | Deva Form — escalation formula wrong for stacks > 1 | bug | `packages/engine-rs/src/powers/defs/turn_start.rs:227-244`, `packages/engine-rs/src/cards/watcher/devaform.rs:9-10` | `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevaPower.java:31-47` | Java: `onEnergyRecharge` grants `energyGainAmount`, then `energyGainAmount += amount` (stacks). With 2 Deva Form casts (amount=2): 2, 4, 6, 8… Rust: `GainEnergy(StatusValue(DEVA_FORM)) + AddStatus(DEVA_FORM, Fixed(1))` — for 2 stacks produces 2, 3, 4, 5. Single-cast case is correct (1, 2, 3, 4 matches). Fix: scale the `AddStatus` step by current stacks (needs `AddStatus(…, StatusValue(DEVA_FORM))` or a dedicated complex hook). | new |
| W3 | Pressure Points `TriggerMarks` bypasses HP-loss pipeline | bug | `packages/engine-rs/src/effects/interpreter.rs:621-629`, `packages/engine-rs/src/cards/watcher/pressurepoints.rs` | `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MarkPower.java:34-39` (uses `LoseHPAction`) | Java routes Mark damage through `LoseHPAction(owner, null, amount, FIRE)` — goes through Buffer, Intangible (caps to 1), on-loseHP hooks (Blue Candle, Centennial Puzzle), and triggers on-death hooks correctly. Rust subtracts `mark` directly from `enemy.entity.hp`: ignores enemy Intangible, skips `on_lose_hp`/`on_death` side-effects (e.g., Spore Cloud Weak, Gremlin Horn energy), and skips `total_damage_dealt` counters that some status-resolvers read. Confirmed divergent. | new (promotes prior "unverified") |
| W4 | Fasting `ModifyMaxEnergy(-1)` persists across turns instead of reducing energy-on-recharge | deferred | `packages/engine-rs/src/cards/watcher/fasting.rs:11-13,21-23` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java` → `EnergyDownPower` | Java's `EnergyDownPower.atStartOfTurn` calls `LoseEnergyAction`, i.e. reduces available energy on the current turn only via the recharge hook (stacks if two Fastings). Rust permanently drops `max_energy` by 1. Net effect at equilibrium (every turn starts with max) is identical; observable difference is mid-turn gainEnergy relics (Runic Dome, etc.) and stacking with Ice Cream. Marked deferred — low gameplay impact for current combat-first training. | new |
| W5 | Signature Move `canUse` excludes duplicate copies of itself | bug | `packages/engine-rs/src/effects/card_runtime.rs:15-23` (`candidate.def_id != card_inst.def_id`) | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SignatureMove.java:46-57` (`c == this` by reference) | Rust's `OnlyAttackInHand` correctly wired (`runtime_meta.rs:14`, `cards/mod.rs:325-327`) for the common case (other Attacks present → blocked). Divergence: Rust excludes by `def_id` equality, so two SignatureMoves in hand each see the other as "not an attack to worry about" and BOTH are playable. Java uses Java reference equality (`c != this`) — the second SignatureMove DOES block the first. Edge case (requires 2 copies in hand via Havoc / Dual Wield / Nilry's / Armaments+ workflow), but real divergence. Fix: iterate with instance-ID or skip exactly ONE matching-def_id occurrence. | new |
| W6 | Omniscience MCTS approximation of "play twice" | intentional | `packages/engine-rs/src/cards/watcher/omniscience.rs`, `packages/engine-rs/src/engine.rs:900` (`resolve_play_card_free_from_draw`) | `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java` (uses `new UseCardAction(card)` twice) | Rust plays the selected card once + adds a cost-0 copy to hand (via resolve path at engine.rs:900), approximating Java's double-play so the policy can still re-evaluate with updated board state on the second cast. Minor behavioral gap: status/stance/damage mods applied during the first play aren't committed before the second is queued (vs Java where both actions enqueue linearly). Accepted for MCTS. | — |
| W7 | Swivel FreeAttack flag non-stacking (overwrite to 1) | bug | `packages/engine-rs/src/effects/interpreter.rs:2092-2094` (`set_status(NEXT_ATTACK_FREE, 1)`) | `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/FreeAttackPower.java:19-43` + `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Swivel.java:30` (`ApplyPowerAction(…, new FreeAttackPower(p, 1), 1)`) | Java's `ApplyPowerAction` stacks `amount` on the same power ID — two Swivels in one turn = next 2 attacks free. Rust `BoolFlag::NextAttackFree` → `set_status(NEXT_ATTACK_FREE, 1)` overwrites; only one attack is ever freed no matter how many Swivels. Fix: change interpreter to `add_status(NEXT_ATTACK_FREE, 1)` or bump counter explicitly. | new |
| W8 | Cut Through Fate draws `Magic` instead of fixed 1 | bug | `packages/engine-rs/src/cards/watcher/cutthroughfate.rs:9-10,17-18` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CutThroughFate.java:32-33` | Known from register. Rust draws 2 (base) or 3 (upgraded); Java draws 1 always. Card text says "Scry 2 / Draw 1." | **D16** |
| W9 | Spirit Shield block formula uses `hand_size` repeated N times (not `(hand_size-1)*magic` once) | bug | `packages/engine-rs/src/cards/watcher/spiritshield.rs:11-13,21-24` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SpiritShield.java:32-42` | Known from register. Large magnitude (4 hand → Java 9 block, Rust 36). | **D17** |
| W10 | Miracle missing `selfRetain` trait | bug | `packages/engine-rs/src/cards/watcher/miracle.rs:3-21`, `packages/engine-rs/src/cards/runtime_meta.rs:113-141` | `decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Miracle.java:29` (`this.selfRetain = true;`) | Java Miracle sets `selfRetain = true` (so the tempCard is retained at end of turn, not discarded). Rust Miracle is **not** in the `retain` list at `runtime_meta.rs:113-141` — held Miracles get discarded at end of turn. Critical for Collect and Omniscience flows where cached Miracles are expected to survive. Fix: add `"Miracle" \| "Miracle+"` to retain list. | new |
| W11 | Collect spawns non-upgraded Miracles even on Collect+ | bug | `packages/engine-rs/src/engine.rs:1226-1230` (`add_temp_cards_to_hand("Miracle", miracles)`), `packages/engine-rs/src/cards/watcher/collect.rs` | `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/CollectPower.java::onEnergyRecharge` (does `Miracle card = new Miracle(); card.upgrade();`) | Java's CollectPower always upgrades the Miracle it produces (makes the generated Miracles give 2 energy each). Rust calls `add_temp_cards_to_hand` with literal ID `"Miracle"` — spawns base 1-energy Miracles regardless of Collect / Collect+ distinction. Fix: spawn `"Miracle+"`. Compounds with W10 since the spawned Miracles currently don't retain. | new |
| W12 | Wallop uses `TotalUnblockedDamage` vs Java `target.lastDamageTaken` | unverified | `packages/engine-rs/src/cards/watcher/wallop.rs:11,20` | `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/WallopAction.java:38-43` | Known from register. Single-target parity holds unless target is Intangible or incidental damage occurs during play. | **D19** |
| W13 | Conjure Blade Chemical X bonus missing | unverified | `packages/engine-rs/src/cards/watcher/conjureblade.rs` | `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ConjureBladeAction.java:34-37` | Known from register. Rust `XCostPlus(0/1)` doesn't apply the Chemical X relic's +2. | **D20** |
| W14 | FearNoEvil `EnemyAttacking` condition coverage | unverified | `packages/engine-rs/src/cards/watcher/fearnoevil.rs:10,19` | `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/FearNoEvilAction.java:25` | Known from register. Needs verification that `Cond::EnemyAttacking` covers ATTACK, ATTACK_BUFF, ATTACK_DEBUFF, ATTACK_DEFEND. | **D23** |
| W15 | Lesson Learned upgrades from draw+discard only, not master deck | unverified | `packages/engine-rs/src/cards/watcher/lessonlearned.rs:7-10` | `decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java` | Known from register. Java likely upgrades from the master deck (persists post-combat); Rust upgrades from `[Draw, Discard]` piles (in-combat only), so a card in hand at kill-time is skipped and upgrades may not persist. | **D24** |
| W16 | Establishment+ only flips `innate`, magic stays 1 | intentional | `packages/engine-rs/src/cards/watcher/establishment.rs:8,16` + `packages/engine-rs/src/cards/runtime_meta.rs:109` (`"Establishment+"` innate) | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Establishment.java:30-37` | Matches Java. Recorded here because the upgrade zero-delta on magic is suspicious at a glance. | **D25** |
| W17 | Vault end-turn skip + debuff-decrement timing | intentional | `packages/engine-rs/src/engine.rs:1545-1582` | `decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java:323-326` | Closed — fixed in commit `2db11718`. Rust now decrements debuffs only on the "enemy actually acts" branch; Vault no longer eats a Weak stack. | **D50 (closed)** |
| W18 | Foresight — ID is `Wireheading` in Java | intentional | `packages/engine-rs/src/cards/watcher/wireheading.rs` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Foresight.java:17` (`public static final String ID = "Wireheading";`) | Java card class `Foresight.java` declares the localized name "Foresight" but keeps the internal ID "Wireheading". Rust module is named `wireheading.rs` and uses ID/name pair that matches Java's ID-first convention. No behavioral divergence. | — |
| W19 | Rushdown / Adaptation name swap (class vs ID) | intentional | `packages/engine-rs/src/cards/watcher/adaptation.rs` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Rushdown.java:17` (`ID = "Adaptation";`) | Java class `Rushdown.java` defines ID="Adaptation"; Rust module named `adaptation.rs` registers with ID="Adaptation". `Rushdown` is the power ID applied; they're two sides of the same card. Confirmed. | — |
| W20 | Simmering Fury — Java class name, ID "Vengeance" | intentional | `packages/engine-rs/src/cards/watcher/vengeance.rs:5-10` | `decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SimmeringFury.java:17` (`ID = "Vengeance";`) | Same class-name vs ID swap. Rust uses ID "Vengeance", card name "Simmering Fury". Turn-start hook at `engine.rs:1279-1287` applies `ChangeStance(Wrath)` + `draw_cards(status)` + clears `SIMMERING_FURY`. | — |

---

## Missing Watcher cards (intentional non-registration)

| Java class | Reason |
|---|---|
| `Discipline.java` | DEPRECATED beta content; never shown in retail card pools. Correctly omitted. |
| `Unraveling.java` | Beta / cut content; not in live card generator. Correctly omitted. |

Both inspected; no divergence.

---

## Extra / tempCard registrations in Rust

These are Watcher-adjacent tempCards (`cards/tempCards/*.java` in Java) which Rust correctly registers as their own modules to be produceable at runtime. All 11 legitimate — one Rust module per Java tempCard:

| Rust module | Java file | Note |
|---|---|---|
| `beta.rs` | `cards/tempCards/Beta.java` | Alpha → shuffles Beta into draw |
| `omega.rs` | `cards/tempCards/Omega.java` | Beta → shuffles Omega into draw |
| `miracle.rs` | `cards/tempCards/Miracle.java` | Collect spawns; Miracle relic; scry etc. |
| `insight.rs` | `cards/tempCards/Insight.java` | Pray → shuffles Insight |
| `smite.rs` | `cards/tempCards/Smite.java` | Carve Reality → tempCard |
| `safety.rs` | `cards/tempCards/Safety.java` | Beyond event etc. |
| `expunger.rs` | (no direct file, generated by ConjureBlade) | Conjure Blade X-cost attack token |
| `throughviolence.rs` | `cards/tempCards/ThroughViolence.java` | Reach Heaven → shuffles |
| `vengeance.rs` | `SimmeringFury.java` | Already covered W20 |
| `wireheading.rs` | `Foresight.java` | Already covered W18 |
| `holywater.rs` | `HolyWater.java` | Watcher-specific, not in tempCards/ |

No spurious modules found.

---

## Items verified clean (no divergence)

Sampled cross-check (Rust file ↔ Java file), `bug` ruled out:

Strike/Defend (unified) · Alpha · BattleHymn · Blasphemy · BowlingBash · Brilliance · CarveReality · Conclude · Consecrate · Crescendo · CrushJoints · DeusExMachina · Devotion · EmptyBody · EmptyFist · EmptyMind · Eruption · Evaluate · FearNoEvil (shape; see W14 unverified) · FlurryOfBlows (stance-change return-from-discard wired at `engine.rs:2986`) · FlyingSleeves · FollowUp · ForeignInfluence · Halt (base+wrath formula matches, `base_block + magic` in Wrath) · HolyWater · Indignation · InnerPeace · JustLucky · LikeWater · MasterReality · Meditate (retain-on-return wired at `interpreter.rs:1234-1240`) · MentalFortress · Nirvana · Perseverance (`OnRetain::GrowBlock`) · Pray · Prostrate · Protect · Ragnarok (5/6 random hits match) · ReachHeaven · Safety · Sanctity (Cond::LastCardType(Skill) matches Java `size-2` semantics) · SandsOfTime (`OnRetain::ReduceCost`) · SashWhip (Cond::LastCardType(Attack), Weak 1→2) · Scrawl (hand-cap at `engine.rs:2880` naturally mirrors Java `10 - hand.size`) · Smite · Study · Swivel (stats only; see W7 for stacking bug) · Tantrum (multi-hit via `multi_hit` hint + PostPlay::ShuffleIntoDraw) · ThirdEye · ThroughViolence · Tranquility · Vault · Vengeance/SimmeringFury · Vigilance · Wallop (stats; see W12 unverified) · WaveOfTheHand · Weave (Scry-return at `engine.rs:609`) · WheelKick · WindmillStrike (`OnRetain::GrowDamage` + Modify damage) · Wish (3-way `ChooseScaledNamedOptions` matches) · Worship (Worship+ retain set) · WreathOfFlame (Vigor consumed on first attack at `card_effects.rs:271-280`)

That's ~60 clean cards. Together with 10 W-rows flagged and 5 cross-ref carries to existing D-entries, coverage is ~95% of registered Watcher cards.

---

## Follow-up questions

1. **W2 Deva Form**: does our Watcher training corpus actually include multi-Deva scenarios? If Deva Form is almost always single-cast, this is low-priority for policy learning. If any training run plays 2+ Devas, fix before next sweep.
2. **W3 Pressure Points**: confirm via regression test — build a combat vs a Wraith (Intangible enemy), apply Mark 10 with Pressure Points, verify Java caps Mark damage at 1 while Rust deals 10.
3. **W7 Swivel**: agent policy currently doesn't consider stacking Swivels (only 1 per turn is normal play). Still a visible bug for PUCT when exploring two-Swivel branches.
4. **W10 + W11** compound: Miracle retain + Collect upgrade are both needed for the Collect strategy to actually work as Java plays it. Fix together.
5. **W5 Signature Move**: common case works (`OnlyAttackInHand` wired through `card_runtime.rs:15-23`). The `def_id != card_inst.def_id` filter correctly excludes the card being played — but ALSO excludes all other copies of itself. If training never produces two SignatureMoves in one hand, this is benign; if it does (Havoc, Nilry's Codex, Dual Wield, etc. — none of which Watcher normally has), Rust lets both play when Java would block the second. Downgrade to `intentional` if agent policy never reaches that state.
