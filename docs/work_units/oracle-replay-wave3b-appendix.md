# Wave 3b Oracle Replay Appendix

**Status:** diagnostic replay only; **uncertified**

This appendix records the recorder omissions observed while replaying three
Watcher victories (607, 516, and 890 recorder actions). It is intentionally
separate from engine parity claims: the adapter reaches all 2,013/2,013
recorded actions with no first divergence among the projected fields it can
compare, but none of the recordings contains enough causal information to
certify an independently generated run.

## Evidence snapshot

- Bundle:
  `data/traces/recordings/-5884681071377138867-WATCHER-20260720-194423/`
- Seed (signed long): `-5884681071377138867`
- Character / ascension / outcome: Watcher / A0 / victory
- Final local replay report: `/tmp/sts-wave3b-final.json`
- Report SHA-256:
  `59f1a01e14d29e55094e040a750f7c6bf94f71ed9acb7e6781a14770150da023`
- Report result: `uncertified`; 607 total and comparable recorder actions,
  571 replayed actions, 465 directly matched checkpoints, 142 coupled
  checkpoints, 84 actions with unverified semantics, and no first divergence
  among compared projected fields. The terminal `RUN_END` witness also proves
  the omitted final True Victory `Proceed`.

The original requested 48-row extraction is frozen below so the recorder patch
can be implemented against an exact list. Subsequent full-engine work exposed
five purge-grid selections and the terminal True Victory `Proceed`; those six
additional omissions are listed separately. Evidence classes must not be added
together as if every callback or reconstruction were another player decision:

| Evidence class | Count | Meaning |
|---|---:|---|
| Original inferred canonical actions | 48 | The exact omissions initially requested for recorder repair. |
| Additional inferred canonical actions | 6 | Five selected-card purge commits plus the terminal True Victory `Proceed`, exposed by later systemic replay work. |
| Ignored recorder callbacks | 36 | The recorder emitted a UI opener or internal-engine callback that is not another agent decision. |
| Minimal state reconstructions | 4 | Three missing realized relic-pool entries plus one missing process-global `Collections.shuffle` outcome. These prevent certification even though replay continued. |

## Recorder hook keys

The final column of the 48-row table refers to these recommended recorder
contracts:

| Key | Required semantic hook and payload |
|---|---|
| `R` | Hook inside the actual `ProceedButton.update()` commit branch, after its hitbox has updated, and emit `LEAVE_REWARDS` with `source_screen`, room type, and floor. The current Prefix checks `hb.clicked` before `hb.update()` can produce the click. |
| `S` | Use the same `ProceedButton` commit hook, classified from the pre-transition screen/room as `SHOP_LEAVE`; include floor and shop/source screen. |
| `G` | Hook the grid-confirm commit after the confirm hitbox updates. Emit the source (`event`, campfire, or shop), deck index, stable card-instance identity/UUID, card ID, upgrades, and misc. A Prefix on `GridCardSelectScreen.update()` has the same premature-hitbox problem as the Proceed hook. |
| `B` | Emit a lossless boss-reward compound action with chest kind, reward item index, option index, relic instance identity, and relic ID, or emit separate `CHEST_OPEN` and typed item/option selection actions carrying those fields. |
| `P` | Patch the boss-icon branch in `DungeonMap.update()` around `bossHb` and `AbstractDungeon.nextRoomTransitionStart()`. The existing `MapRoomNode.update()` hook cannot see boss-icon clicks. Emit destination coordinates, node kind, and boss ID. |
| `T` | Hook the actual `ProceedButton` commit branch and classify the pre-transition state as `PROCEED`; include transition kind plus source and destination acts/rooms. |
| `W` | Hook the `chooseOne` branch of `CardRewardScreen.cardSelectUpdate()` and emit the originating card/action, option index, and option card ID. Wish uses `ChooseOneAction`/`chooseOneOpen`, not the grid or hand selection screens. |
| `U` | Treat `ShopScreen.purchasePurge()` as an opener only. Emit a separate semantic purge commit from the grid-confirm path with shop/card instance IDs, deck index, card ID, upgrades, and misc. |

For grid and hand selections, a selected card ID alone is insufficient because
duplicate cards are legal. The payload must carry deck position plus a stable
card-instance identity. For every hook above, commit only the semantic player
decision and pair it with a checkpoint after the action queue and screen
transition settle.

## Exact 48 inferred actions

`before_idx` means the adapter executed the missing canonical action immediately
before the recorder action with that index. Multiple entries at the same index
are ordered exactly as they appeared in the report.

| # | `before_idx` | Inferred canonical action | Evidence / cause | Hook |
|---:|---:|---|---|:---:|
| 1 | 17 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 2 | 19 | `SelectRewardItem(0)` | The second Purification Shrine callback collapses the typed grid opener and card selection. | `G` |
| 3 | 30 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 4 | 33 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 5 | 35 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 6 | 46 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 7 | 49 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 8 | 53 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 9 | 67 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 10 | 90 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 11 | 105 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 12 | 121 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 13 | 123 | `ChoosePath(0)` | The combat checkpoint and sole synthetic boss destination prove the omitted boss-icon click. | `P` |
| 14 | 135 | `LeaveRewards` | `BOSS_RELIC` proves the boss-combat reward exit. | `R` |
| 15 | 135 | `OpenChest` | `BOSS_RELIC` proves the intervening Boss Chest open. | `B` |
| 16 | 135 | `SelectRewardItem(0)` | This is the uniquely legal typed opener for `BossRelicSelectScreen`. | `B` |
| 17 | 136 | `Proceed` | The next-act `PATH` proves the omitted `DungeonTransitionScreen` Proceed. | `T` |
| 18 | 151 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 19 | 153 | `SelectRewardItem(0)` | The second Back to Basics callback collapses the typed grid opener and card selection. | `G` |
| 20 | 156 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 21 | 173 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 22 | 203 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 23 | 208 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 24 | 225 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 25 | 256 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 26 | 277 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 27 | 279 | `ChoosePath(0)` | The combat checkpoint and sole synthetic boss destination prove the omitted boss-icon click. | `P` |
| 28 | 318 | `LeaveRewards` | `BOSS_RELIC` proves the boss-combat reward exit. | `R` |
| 29 | 318 | `OpenChest` | `BOSS_RELIC` proves the intervening Boss Chest open. | `B` |
| 30 | 318 | `SelectRewardItem(0)` | This is the uniquely legal typed opener for `BossRelicSelectScreen`. | `B` |
| 31 | 319 | `Proceed` | The next-act `PATH` proves the omitted `DungeonTransitionScreen` Proceed. | `T` |
| 32 | 337 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 33 | 355 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 34 | 370 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 35 | 391 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 36 | 393 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 37 | 399 | `SelectRewardItem(0)` | The second Bonfire callback collapses the typed grid opener and card selection. | `G` |
| 38 | 406 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 39 | 429 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 40 | 434 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 41 | 455 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 42 | 470 | `LeaveRewards` | The following `PATH` proves the completed reward-screen exit. | `R` |
| 43 | 472 | `ChoosePath(0)` | The combat checkpoint and sole synthetic boss destination prove the omitted boss-icon click. | `P` |
| 44 | 498 | `Proceed` | The Spire Heart event callback proves the preceding boss-room transition Proceed. | `T` |
| 45 | 505 | `ShopLeave` | The following `PATH` proves the active-shop exit. | `S` |
| 46 | 526 | `LeaveRewards` | The following combat checkpoint proves the completed reward-screen exit. | `R` |
| 47 | 526 | `ChoosePath(0)` | The combat checkpoint and sole synthetic boss destination prove the omitted boss-icon click. | `P` |
| 48 | 527 | `CombatAction::Choose(0)` | Settled powers and gold uniquely identify Wish's omitted choice between `PLAY_CARD` 526 and the next recorded action. | `W` |

The action totals cross-check as follows: 26 `LeaveRewards`, 7 `ShopLeave`,
5 `SelectRewardItem`, 4 `ChoosePath`, 3 `Proceed`, 2 `OpenChest`, and 1 Wish
choice = 48.

## Six omissions exposed after the original 48-row extraction

These do not change the requested table above. They are additional recorder
repairs found while extending the same run through the completed terminal
lifecycle:

| # | `before_idx` | Inferred canonical action | Evidence / cause | Hook |
|---:|---:|---|---|:---:|
| 49 | 32 | `ShopRemoveCard(4)` | The following merchant purchase checkpoint exactly proves the omitted purge-grid selection. | `U` |
| 50 | 34 | `ShopRemoveCard(4)` | The following ordered deck exactly proves the omitted selected card. | `U` |
| 51 | 155 | `ShopRemoveCard(3)` | The following ordered deck exactly proves the omitted selected card. | `U` |
| 52 | 392 | `ShopRemoveCard(0)` | The following ordered deck exactly proves the omitted selected card. | `U` |
| 53 | 431 | `ShopRemoveCard(0)` | The following ordered deck exactly proves the omitted selected card. | `U` |
| 54 | 607 | `Proceed` | Authoritative `RUN_END {status:VICTORY,floor:56}` and the sole terminal progression action prove `ProceedButton.goToTrueVictoryRoom`. Optional potion discards remain legal but are not inferred. | `T` |

## The 36 ignored callbacks are not omitted decisions

These records are present in the legacy stream but were ignored because they
are duplicate UI hooks or internal engine callbacks rather than additional
agent choices:

| Recorder-only callback class | Count | Recorder indices | Required repair |
|---|---:|---|---|
| Unindexed card-reward preview/opener with no persistent choice | 25 | 16, 29, 45, 89, 104, 120, 150, 202, 224, 255, 276, 314, 333, 334, 353, 354, 368, 369, 386, 387, 428, 454, 468, 469, 523 | Emit authoritative reward `item_index`, then emit semantic choose/skip/leave separately. |
| Distilled Chaos internal `PLAY_CARD` callback | 3 | 70, 71, 72 | Suppress automatic card-use callbacks or mark `origin=distilled_chaos` plus a parent action ID. |
| Unavailable linked Sapphire Key callback | 2 | 52, 207 | Do not emit a claim after the linked relic has made the key unavailable; include reward item index/status. |
| Cancelled Smith grid opener | 1 | 394 | Distinguish open, confirm, and cancel; only confirm mutates the deck. |
| Disabled chest relic callback | 1 | 405 | Do not emit the linked relic after Sapphire Key claim disables it; carry reward item status/index. |
| Shop purge grid opener callback | 2 | 31, 430 | Treat `ShopScreen.purchasePurge()` as an opener/context marker, not the semantic removal. |
| Time Warp internal end-turn callback | 1 | 485 | Suppress effect-driven `END_TURN`, or annotate its origin and parent card action. |
| Burn end-turn internal `useCard` callback | 1 | 595 | Suppress status-card auto-use (`dontTriggerOnUseCard`) or annotate automatic origin. |

## The four legacy-run reconstructions

The old device/profile data did not include realized ordered relic pools. The
adapter therefore made three diagnostic, same-tier replacements before the
listed recorder action:

| `before_idx` | Recorder-proven relic | Generated relic before reconstruction | Why uncertified |
|---:|---|---|---|
| 51 | `Turnip` | `StoneCalendar` | The claimed identity is known, but the missing ordered pool cannot prove how Java reached it. |
| 254 | `Prayer Wheel` | `Girya` | The claimed identity is known, but the missing ordered pool cannot prove how Java reached it. |
| 427 | `TeardropLocket` | `Bottled Tornado` | The claimed identity is known, but the missing ordered pool cannot prove how Java reached it. |

These are evidence quarantines, not Rust generation fixes. A reminted golden
must provide the complete all-unlocked profile and realized pool/RNG inputs so
no reconstruction is needed.

At index 596, the source-correct two-phase end-turn path preserves the original
unshuffled whole hand for status effects and shuffles the later retained-card
snapshot before Ethereal exhaust. The trace proves the same trailing
`Dazed`/`Void` multiset but the reverse order. Because the legacy envelope did
not capture the raw process-global `java.util.Collections` default `Random`
state, the adapter minimally restores the witnessed two-card suffix order for
continued diagnostics. It does not invent or advance an RNG state.

## Omission classes confirmed in the two newer recordings

The two newer Watcher victories declare all cards and relics unlocked and are
highly useful for extending behavior coverage, but they are not yet
initialization-certified: their headers omit Note for Yourself state and do not
prove an all-character unlock roster, while their recorder dialect still omits
semantic payloads and process-global inputs. The simulator's broader
all-unlocked compatibility policy is therefore an explicit assumption, not a
fact inferred from these runs. The observations below come from raw-corpus
inspection and must not be read as parity certification of every later row.

The comparator now makes that distinction mechanically. A recording can reach
every action without a projected-state difference and still cannot become
`Match` unless it supplies an authoritative profile, release/standard execution
environment, complete initialization witness, complete checkpoint schema, and
an uninterrupted lifecycle. Existing v1 checkpoint objects are always
quarantined as either `incomplete_v1_checkpoint_shape` or
`partial_v1_checkpoint_schema`; a distinct required-field checkpoint version is
needed before certification is possible.

### Seed `52YDKL7ZDRXZZ` (890 actions)

- Final report: `/tmp/sts-new-golden-890-final.json`, SHA-256
  `09fb8933c60dbd2f9c5b963127ef75a5cacb4fced13997e9aebb7a265f7b5d56`.
  Result: `uncertified`; 890/890 comparable actions, 822 replayed actions,
  689 direct checkpoints, 201 coupled callbacks, 136 inferred actions, 68
  ignored callbacks, two explicit reconstructions, 150 unverified semantic
  identities, and no first divergence.
- The profile declares empty locked card/relic lists but omits
  `note_for_yourself_card`. Its alleged post-generation initial envelope has
  map RNG counter -1 while the first settled Neow checkpoint has 98, so Rust
  now rejects its ambient MathUtils seeds. The envelope also omits the raw
  default `Collections.shuffle` state and process-global Bomb identity state.
- There are no `PROCEED` or `CHOOSE` actions in the stream, confirming the
  premature Prefix hook behavior described above.
- Five `SHOP_REMOVE` records omit the selected card instance at indices 39,
  346, 366, 551, and 692. The next checkpoints show base-card removals
  `Defend_P`, `Strike_P`, `Strike_P`, `Strike_P`, and `Defend_P`, respectively,
  but duplicate copies make instance identity unprovable.
- Distilled Chaos emits three internal `PLAY_CARD hand_idx=-1` callbacks at
  808--810. They must remain children of the potion action, not agent actions.
- Repeated event callbacks lack a semantic stage and selected-card payload,
  including Purification Shrine 13--14, Drug Dealer 181--182, Ghosts 184--185,
  The Library 209--210, Mysterious Sphere 465--466, Winding Halls 587--589,
  and Spire Heart 756--759.
- Twenty-seven blank `REWARD_TAKE {reward_type:CARD,id:""}` opener records lack
  an authoritative item index. Some lead to a card pick; skipped previews still
  omit the semantic skip/leave action.
- The two diagnostic reconstructions are explicit and narrow: 95 equal-bound
  `cardRandom` UI refresh draws before Discovery selection at index 378, and
  the `Void`/`Dazed` Ethereal exhaust suffix order at index 829 when the raw
  `java.util.Collections` state is absent.

### Seed `14W5RL9UTWNCN` (516 actions)

- Final report: `/tmp/sts-new-golden-516-final.json`, SHA-256
  `b7bb694098e0b19fb2ef1a65650f20ecc7ea212904fd63df99392a720f230943`.
  Result: `uncertified`; 516/516 comparable actions, 464 replayed actions,
  393 direct checkpoints, 123 coupled callbacks, 50 inferred actions, 52
  ignored callbacks, one explicit reconstruction, 97 unverified semantic
  identities, and no first divergence.
- The same profile/process-global omissions apply. Its alleged post-generation
  envelope has map RNG counter 0 while the first settled Neow checkpoint has
  94, so its ambient MathUtils seeds are also rejected rather than silently
  trusted. Note for Yourself, raw default `Collections.shuffle`, and
  process-global Bomb state remain absent.
- The replay currently needs one diagnostic reconstruction before index 171:
  the later checkpoint proves a `SmokeBomb` reward and the potion RNG endpoint,
  but the recording omits the runtime/debug settings and ordered reward list
  needed to prove Java's release-build four-item reward-count gate. This is a
  quarantine, not a production generation rule, and prevents certification.
- There are again no `PROCEED` or `CHOOSE` actions.
- Five `SHOP_REMOVE` records omit the selected card instance at indices 57,
  125, 271, 333, and 351. The following checkpoints show removals
  `Defend_P`, `Defend_P`, `Strike_P`, `Strike_P`, and `Strike_P`, but not which
  duplicate instance was selected.
- Necronomicon emits 22 automatic `PLAY_CARD hand_idx=-1` callbacks at indices
  137, 152, 167, 198, 227, 241, 258, 275, 297, 312, 339, 356, 387, 402, 413,
  440, 447, 454, 460, 467, 488, and 507. Each needs `origin=necronomicon` and a
  parent semantic play ID, or suppression from the agent-action stream.
- The final Through Violence records at 514 and 515 have identical payloads
  and settled lethal state, but replay proves that they are two distinct legal
  copies occupying the same hand index after the first copy exhausts. Do not
  deduplicate from payload/state alone: emit stable card-instance and semantic
  action IDs so these causal plays remain distinguishable.
- Stage-unlabeled repeated event callbacks include Golden Wing 28--30,
  Shining Light 32--33, Golden Idol 35--37, Cleric 41--42, Cursed Tome 127--131,
  Ghosts 181--182, The Mausoleum 192--193, Falling 267--269, Moai Head 325--326,
  and Spire Heart 420--423. Golden Wing demonstrates the issue most clearly:
  damage, grid open, selected card, and exit cannot be recovered from three
  identical `EVENT_CHOICE` payloads alone.
- Nineteen blank card-reward opener records omit authoritative reward item
  indices and do not explicitly distinguish choose, skip, and leave.

## Completion, assumptions, and translation discipline

There is no honest single “engine complete” percentage: the completed 667-row
content ledger and this three-run dynamic corpus measure different scopes from
global run-system parity. The PR reports each gate separately:

| Gate | Completion | Interpretation |
|---|---:|---|
| Engine library regression | 3,269/3,269 (100%) | The canonical library suite passes with zero failures and zero ignored tests. |
| Source-verified content ledger | 667/667 (100%) | Cards, monsters, relics, and potions in the ledger are source-verified; this does not certify every run-level system. |
| Observed golden action reachability | 2,013/2,013 (100%) | Every action in the three non-empty Watcher victories is comparable and no report has a first divergence. |
| Direct action checkpoints | 1,547/2,013 (76.85%) | These checkpoints compare directly; the remaining 466 (23.15%) are recorder-FIFO callbacks coupled to an exact later checkpoint. |
| Independently certified goldens | 0/3 (0%) | All three runs require one or more recorder assumptions, inferred decisions, unverified identities, or reconstructions. |
| Observed systemic engine divergences | 0 remaining in this corpus | This is a corpus-scoped result, not proof that unobserved Java paths are complete. |

Assumptions retained deliberately:

1. The ordinary engine API uses an all-unlocked compatibility profile. The two
   new headers corroborate this for cards and relics (`locked_*=[]`, highest
   unlocked ascension 20, all bosses seen), but the legacy 607-action run has
   no authoritative profile and remains quarantined.
2. Only Watcher A0 victories are dynamically exercised here. Unlocking all
   characters/ascensions does not substitute for traces that actually execute
   their ascension- and character-specific paths.
3. Missing Note for Yourself uses Java's historical `IronWave` fallback only
   for diagnostic replay. It is not treated as captured device state. A strict
   profile must separately record `note_for_yourself_upgrades`; values 0 and 1
   are modeled, while repeated Searing Blow upgrades remain unsupported.
4. Recorder-omitted decisions are replayed only when a later full checkpoint
   forces one canonical route. Ambiguous instance identity remains explicitly
   unverified; duplicate cards are never guessed by “first match.”
5. Process-global RNG is never fabricated. Contradictory “post-generation”
   envelopes are rejected, and absent `java.util.Collections` state is reported
   as a reconstruction rather than hidden behind an engine change.

Translation best practices preserved in this wave:

- Decompiled Java controls values, ordering, queue behavior, and RNG draw
  counts; existing Rust/tests are not treated as ground truth.
- Every confirmed engine correction has a focused source-cited regression test,
  including lethal `clearPostCombatActions`, the two-phase end-turn hand order,
  Meditate/Time Warp callback timing, event-combat entry, terminal True Victory,
  and trace-header integrity.
- Production engine rules remain separate from diagnostic adapter bridges.
  Every inferred action, ignored callback, and reconstructed field is serialized
  in the report and prevents certification.
- RNG streams remain distinct. Native streams, process-global MathUtils, and
  default `Collections.shuffle` are neither conflated nor aligned by silently
  burning unrelated draws.
- Intake fails closed on duplicate candidates, script/trace payload mismatch,
  repeated lifecycle records, unknown trace record kinds, and header/meta
  disagreement.
- Certification also fails closed for daily/trial/custom/debug runs, unexpected
  loaded mods, missing or altered native/ambient RNG witnesses, altered realized
  pools/queues/map/Neow options, any `RESUME`, a missing terminal `RUN_END`, and
  every v1 checkpoint schema. The accepted environment is the exact release
  standard-run allowlist `BaseMod`, `StSLib`, and `TraceLab`.

## Final Java-to-Rust closure review

The last review followed the decompiled Java action-manager chain through
combat start, player end turn, enemy/end-of-round processing, the next turn's
energy recharge, relic/power/orb callbacks, post-draw callbacks, and every
choice continuation. It found and closed the following systemic translation
errors rather than adding per-recording patches:

- one serializable queue now preserves Java top/bottom/direct placement across
  combat-start and turn-start callbacks, including within-callback batch order;
- Toolbox consumes `cardRandom` only when `ChooseOneColorless` drains, after
  earlier callback-time generators, and choice resolution resumes the remaining
  queue exactly once;
- energy recharge, Divinity exit, delayed Blasphemy death, EnergyDown, ordered
  powers/relics, Plasma/Cables/Loop/Emotion Chip, and finite turn relics resolve
  in Java order;
- the end-turn path preserves Java's pre/post-card phases, status-card removal,
  retain selection, orb/relic/power ordering, and `clearPostCombatActions`
  survivor classes after lethal damage;
- ApplyPower versus direct-add order and dynamic identities for The Bomb, Night
  Terror, Minion, BackAttack, Stasis, and Pen Nib survive trace projection and
  checkpoint continuation; and
- checkpoint semantics revision v7 rejects v6, validates hidden queued card
  identities, and rejects impossible external continuation boundaries.

The post-fix review found no remaining P0/P1 engine defect in that chain. F16 is
therefore resolved. This is deliberately narrower than general game
certification: F9, F10, and F15 still require better recorder data and broader
dynamic coverage.

The final unfiltered suite found one additional production omission outside
that chain: `EternalFeather.onEnterRoom(RestRoom)` had been dropped during the
run-layer rewrite. Its exact `floor(masterDeck.size / 5) * 3` heal is restored
before the campfire choice under the existing source-derived regression. The
same pass reconciled 17 stale tests with the source-correct queue, reward, RNG,
event, power, and relic behavior; none required reverting the new engine rules.
A bounded reward-settlement helper now fails fast if a legal but unclaimable
potion reward makes no progress, rather than hiding later failures in an
unbounded test loop.

## Remaining work units

No state divergence remains in the supplied non-empty corpus, so there is no
known engine-rule work unit left from these three replays. This is not a claim
that every unobserved Java branch is complete. The remaining named global units
are F9 (semantic recorder actions/checkpoints), F10 (ambient/process-global RNG
witnesses), and F15 (an uninterrupted canonical replay). Concretely:

1. Record the exact 48 original omitted actions plus the six addendum omissions
   above through the eight semantic hook contracts.
2. Capture the complete settled initialization state after generation: Note for
   Yourself, all native stream cursors/raw states, ambient MathUtils, raw default
   `java.util.Collections` state, Bomb/process-global identity, and any realized
   ordered pools that affect rewards.
3. Emit stable action, parent, item, card-instance, and selection identities;
   separate UI open/cancel callbacks and automatic engine children from agent
   decisions.
4. Introduce a distinct complete checkpoint schema/version whose phase-relevant
   fields are required rather than silently omittable.
5. Remint at least one non-empty golden that yields zero inferred actions, zero
   ignored/unclassified callbacks, zero reconstructions, zero unverified
   semantics, and a matching terminal lifecycle.
6. Add coverage for other characters and ascension-sensitive run paths before
   describing the simulator as generally training-certified.

## Recorder acceptance criteria for the next golden

A repaired recorder should make the adapter report zero inferred actions, zero
unclassified legacy callbacks, zero state reconstructions, and zero unverified
semantics. Internal callbacks may instead be emitted outside the agent-action
stream, or explicitly marked `semantic=false` with origin and parent identity.
At minimum it must:

1. Hook decisions at the actual commit branch, not a frame-early method Prefix.
2. Emit semantic screen stage plus authoritative item/option/deck indices and
   stable instance identities.
3. Suppress internal automatic callbacks from the agent-action stream, or give
   them explicit origin, parent action ID, and semantic/non-semantic status.
4. Emit every reward/shop/transition leave and every boss-icon path click.
5. Capture the complete all-unlocked profile, Note for Yourself, all native and
   ambient RNG states, process-global Bomb identity state, and realized ordered
   pools required by the initialization contract.
6. Snapshot only after the action queue and screen transition settle.

Until those conditions hold, a full diagnostic replay is useful evidence but
must remain labeled `uncertified`.

At the final replay snapshot, all three non-empty recordings traverse their
complete 2,013-action corpus without a projected-state divergence. They remain
diagnostic rather than certified because recorder omissions, unverified action
identity, inconsistent initialization envelopes, and seven explicitly reported
state reconstructions remain. The two later zero-action `ABANDON` siblings are
excluded from the evidence totals because they contain no action checkpoint.
