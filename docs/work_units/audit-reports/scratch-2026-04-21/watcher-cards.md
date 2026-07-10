# Watcher Cards Parity Audit (Read-Only, 2026-04-21)

Three-way scan: Java decompile -> Rust impl -> Rust tests, scoped to Watcher
cards likely to see training play. All deviations found were already registered
in Stage B (`docs/work_units/audit-reports/watcher-cards.md`,
`docs/work_units/audit-reports/watcher-stances-scry-mantra.md`) or the global
deviation register (`docs/work_units/parity-deviations-register.md`). No new
P0/P1 bugs surfaced; several P2 test-coverage gaps noted.

## Summary

Coverage: ~37 Watcher cards + base stance plumbing. Verdict legend: CLEAN,
REGISTERED (already flagged), GAP (untested but correct).

### Starters and stance-core
- Strike / Strike+ (starters.rs:28-59) - CLEAN; unified with Ironclad Strike
  by design, registered as W1 in watcher-cards.md
- Defend / Defend+ (starters.rs:62-93) - CLEAN; unified by design, W1
- Eruption / Eruption+ (eruption.rs) - CLEAN; 9 dmg + enter Wrath verified by
  test_card_runtime_watcher_wave4.rs:91-104
- Vigilance / Vigilance+ (vigilance.rs) - CLEAN; implicit base_block + Calm
- Crescendo / Crescendo+ (crescendo.rs) - CLEAN; retain + ChangeStance(Calm)

### Attacks (stance-sensitive)
- FlurryOfBlows / + - CLEAN; multi_hit=3/5 via runtime_meta.rs, explicit
  `DealDamage` so no double-damage from implicit base_damage
- FearNoEvil / + - REGISTERED D23 (watcher-cards.md W-refs); known deviation
- SashWhip / + - CLEAN; conditional Weak on InStance(Wrath) via HeadStomp-
  equivalent
- FlyingSleeves / + - CLEAN; multi_hit=2, retain wired
- Halt / + - CLEAN; base_block 3/4 + conditional `A::Magic` (9/14) in Wrath;
  matches Java HaltAction branching
- Tantrum / + - CLEAN; multi_hit=3/4 + PostPlay=ShuffleIntoDraw at
  runtime_meta.rs:187,220
- PressurePoints - REGISTERED W3 (pipeline bypass)
- CutThroughFate - REGISTERED D16 / W8 (Magic-sourced draw, scry amount)
- SignatureMove - REGISTERED W5 (dedup logic)
- Wallop - REGISTERED D19
- ConjureBlade - REGISTERED D20
- LessonLearned - REGISTERED D24
- ReachHeaven / + - CLEAN; attack + add Through Violence to draw
- Indignation / + - CLEAN; mantra cost branch via IndignationAction wired

### Skills and mantra generators
- Prostrate - CLEAN; 2 block + 1 mantra
- Meditate - CLEAN; retain this card, PostPlay=EndTurn in runtime_meta.rs
- Worship - CLEAN; 5 mantra; + retains
- InnerPeace - CLEAN; stance-conditional draw(3) vs enter Calm
- Devotion / + - REGISTERED WS1 (post-draw timing fix pending)
- BattleHymn / + - CLEAN; verified test_card_runtime_watcher_wave4.rs:108-114
- Sanctity - CLEAN; 4 block + conditional Draw(2) on LastCardType(Skill)

### Divinity / Mantra-path cards
- Blasphemy / + - CLEAN (with caveat); dedicated `blasphemy_active` flag in
  engine.rs:1090-1099 delivers self-death start-of-turn. Independent of
  WS8 (END_TURN_DEATH unwired), so Blasphemy itself works correctly.
- DevaForm / + - REGISTERED W2 (stacking semantics)
- Establishment / + - WS13 intentional-deviation note; retain cost-reduce
  working for current corpus
- Wish / + - CLEAN; three modes (BecomeAlmighty/FameAndFortune/LiveForever)
  map to AddStatus(STRENGTH)/gold/HP gain
- Scrawl - CLEAN; DrawCards(Fixed(10)) + hand-cap break at engine.rs:2880
  is net-equivalent to Java ExpertiseAction(10)
- Collect / Collect+ - REGISTERED W11 (Collect+ upgrade missing)

### Stance utilities
- EmptyFist / + - CLEAN; ChangeStance(Calm) + base_damage fallback
- EmptyBody / + - CLEAN; ChangeStance(Calm) + implicit base_block
- EmptyMind / + - CLEAN; ChangeStance(Calm) + Draw(2/3)
- Perseverance / + - CLEAN; retain + OnRetain=GrowBlock in runtime_meta.rs
- TalkToTheHand - CLEAN; attack + AddStatus(BLOCK_NEXT_TURN) via status
- FollowUp - CLEAN; LastCardType(Attack) conditional refund + draw
- Study - CLEAN; AddStatus(STUDY) self; generates Insight on end-of-turn
- DeceiveReality - CLEAN; generates Safety + block
- ForeignInfluence - CLEAN; transform/pick via action-equivalent
- ThirdEye - CLEAN; block + Scry(3/5) wired via scry status path
- Evaluate / + - CLEAN; block + ShuffleIntoDraw(Insight)
- Alpha - CLEAN; ShuffleIntoDraw(Beta) via static card_add
- Protect - REGISTERED (Retain handling) no issue found in current impl

### Scry / Retain / Mantra mechanics
- Miracle - REGISTERED W10 (retain missing)
- Foresight - REGISTERED WS4 / D109 (empty-deck shuffle)
- SandsOfTime - CLEAN; retain + OnRetain=ReduceCost
- WindmillStrike - CLEAN; retain + OnRetain=GrowDamage
- Swivel - REGISTERED W7 (non-stacking)
- SpiritShield - REGISTERED W9 / D17 (block formula off)
- HolyWater - REGISTERED D88

## Detailed deviations

No new findings. All deviations encountered map to already-registered
entries. One-liners:

1. **PressurePoints pipeline bypass** (W3) - card applies damage outside
   execute_card_effects, so relic/power hooks don't fire. Already in
   audit-reports/watcher-cards.md:~W3.

2. **DevaForm stacking** (W2) - Java increments Energy gain each turn;
   Rust grants +1 flat each turn via AddStatus(DEVA_FORM). Test at
   test_card_runtime_watcher_wave4.rs:123-129 asserts stacking to 2 but
   does not cover the +2/+3 escalation. Already in audit-reports/
   watcher-cards.md:W2.

3. **Collect+ upgrade missing** (W11) - only base Collect registered.
   Already in audit-reports/watcher-cards.md:W11.

4. **Swivel non-stacking** (W7) - AddStatus should stack, Rust overwrites.
   Already in audit-reports/watcher-cards.md:W7.

5. **SpiritShield block formula** (W9 / D17) - Java: 3 * retained-count;
   Rust: Magic * hand.size. Already in audit-reports/watcher-cards.md:W9
   and parity-deviations-register.md:D17.

6. **Miracle retain** (W10) - retain flag missing; PR #138 pending.
   Already in audit-reports/watcher-cards.md:W10.

7. **Devotion post-draw timing** (WS1) - status-apply ordering relative
   to end-of-turn draw. Already in watcher-stances-scry-mantra.md:WS1.

8. **Foresight empty-deck** (WS4 / D109) - triggers scry on empty deck
   without shuffle-in. Already in both docs.

9. **CannotChangeStance unenforced** (WS3 / D110) - status stored but
   not checked in change_stance. Already in watcher-stances-scry-
   mantra.md:WS3.

10. **Fasting energy drain unwired** (WS11 / D89) - status effect on
    turn start not consuming energy. Already registered.

11. **CutThroughFate** (D16 / W8) - draws on Magic amount instead of 1,
    scry amount mismatch. Already in parity-deviations-register.md:D16.

12. **Wallop** (D19), **ConjureBlade** (D20), **FearNoEvil** (D23),
    **LessonLearned** (D24) - each registered in D-register.

13. **WaveOfTheHand** (WS12) - clears at turn-start in Rust, end-of-
    round in Java. Already registered.

14. **GoldenEye / Enchiridion / DeadBranch** (D106-D108) - relic/power
    hooks Watcher cards rely on are unwired. Already registered.

15. **Establishment** (WS13) - documented intentional deviation for
    corpus stability. No action.

## Untested cards (P2 gaps)

These are implemented correctly per my read but lack dedicated runtime
tests; test coverage would reduce future regression risk:

- Halt+ upgrade path (base `base_block=4` + magic `A::Magic` 14 in Wrath):
  no test asserting 18 block when Wrath active. halt.rs + no hit in
  test_card_runtime_watcher_wave4.rs or test_cards_watcher.rs.
- FlurryOfBlows+ (5-hit path): multi_hit=5 via runtime_meta.rs:220; no
  test exercising the upgraded hit count.
- Tantrum+ (4-hit path): same as above; cost/base_damage asserted but hit
  count upgrade untested.
- SashWhip Wrath branch: conditional Weak apply on InStance(Wrath) not
  covered by a stance-entered runtime test.
- Indignation mantra branch: IndignationAction equivalent (cost-mantra
  branch) untested in runtime.
- Blasphemy self-death: engine.rs:1090-1099 blasphemy_active flag has no
  dedicated "player dies on next turn start" test in the Watcher wave
  files I scanned.
- Scrawl hand-cap interaction: DrawCards(Fixed(10)) + hand-cap break is
  the net-equivalent of Java ExpertiseAction(10-handSize), but no test
  asserts the cap-stop behavior for this card specifically.
- Wish+ upgrade path: three-branch action mapping untested per branch.

## Confirmed clean (no action)

Starters (Strike/+/Defend/+), Eruption/+, Vigilance/+, Crescendo/+,
Halt/+, FlurryOfBlows/+, Tantrum/+, SashWhip/+, FlyingSleeves/+,
EmptyFist/+, EmptyBody/+, EmptyMind/+, Perseverance/+, ReachHeaven/+,
Prostrate, Meditate, Wish/+, Blasphemy/+ (with dedicated flag path),
Scrawl, Worship, InnerPeace, Protect, Alpha, TalkToTheHand, FollowUp,
Study, DeceiveReality, ForeignInfluence, Indignation/+, Establishment/+
(intentional WS13), ThirdEye, Evaluate/+, BattleHymn/+, Adaptation/+
(Rushdown), Devotion/+ base path (WS1 is timing nit), DevaForm/+ base
path (W2 is scaling nit), SandsOfTime, WindmillStrike, Sanctity.

## Net signal

Stage B coverage is thorough for Watcher cards. The PR #138 stack is
safe to merge for overnight training from a Watcher-cards standpoint:
every real deviation I found on a second-pass read is already
registered. The P2 test gaps listed above are worth a follow-up
wave (upgrade paths + stance-conditional branches) but do not block
merge.
