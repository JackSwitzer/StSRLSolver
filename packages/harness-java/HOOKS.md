# Record-Mode Hook Inventory (P1)

Every player decision commit-point, with the SpirePatch target that observes it.
Convention: hooks capture the DECISION at commit time (push a pending action);
the post-action state snapshot is taken at the next stable state (actionManager
idle, no screen transition — same gate ScriptRunner already uses), pairing the
pending action with its resulting state record.

Signatures verified against `decompiled/java-src/com/megacrit/cardcrawl/` on
2026-07-18 unless marked REFINE-AT-P2 (candidate needs line-level confirmation
during implementation).

| v2 action | Commit point (SpirePatch target) | Verified |
|---|---|---|
| PLAY_CARD {card_id, hand_idx, target} | `AbstractPlayer.useCard(AbstractCard, AbstractMonster, int)` Prefix — fires once per committed play, after targeting; hand_idx resolved from `player.hand` before removal | `AbstractPlayer.java:1356` |
| END_TURN | `AbstractRoom.endTurn()` Prefix; suppress when initiated by game effects (track `PressEndTurnButtonAction` origin vs button) | `AbstractRoom.java:393` |
| USE_POTION {slot, target} | `AbstractPotion.use` is abstract & overridden — patch the invoker instead: `PotionPopUp` confirm path (calls `use(target)` then discards). REFINE-AT-P2: exact private method | base abstract confirmed |
| DISCARD_POTION {slot} | `TopPanel` discard path / `AbstractPlayer.removePotion` Prefix with in-combat-use disambiguation flag. REFINE-AT-P2 | — |
| NEOW {choice} | `NeowEvent.buttonEffect(int)` Prefix — matches existing ScriptRunner semantics (only the real multi-option choice recorded) | `NeowEvent.java:161` |
| EVENT_CHOICE {choice} | `RoomEventDialog.update()` Prefix+Postfix + `GenericEventDialog.update()` Prefix+Postfix — the two dialog widgets every event option click bottoms out in (24 of ~50 event classes override `AbstractEvent.update()` and all still funnel through one of these two `getSelectedOption()`-consuming loops); detects the `waitForInput` true->false edge (set exactly once per click) and reads `AbstractDungeon.getCurrRoom().event` for the class name. Replaces the old `AbstractEvent.update()` instrument, which missed every image-type event (`AbstractImageEvent` overrides `update()` entirely) — pilot floors 4, 5, 22. NeowEvent shares `RoomEventDialog`, so its clicks are filtered out here (already recorded via `NeowEvent.buttonEffect`) | `RoomEventDialog.java:62-70`, `GenericEventDialog.java:84-92`, `AbstractImageEvent.java:52-54` |
| PATH {choice → node x,y} | `DungeonMapScreen` node click commit → `MapRoomNode` chosen / `AbstractDungeon.nextRoom` set. REFINE-AT-P2: patch point that fires exactly once | — |
| REWARD_TAKE {item} | `RewardItem.claimReward()` Prefix (returns boolean; record on true via Postfix result check) | `RewardItem.java:255` |
| REWARD_SKIP / PROCEED | `ProceedButton` click commit (room-type-aware; this is also CHEST_SKIP, SHOP_LEAVE, and post-combat skip) | REFINE-AT-P2 |
| CARD_REWARD {pick / SKIP / BOWL} | `CardRewardScreen.acquireCard` Prefix; Singing Bowl + Skip buttons in same screen. REFINE-AT-P2: bowl/skip button methods | — |
| CAMPFIRE {REST/SMITH/LIFT/TOKE/DIG/RECALL} | `AbstractCampfireOption.useOption()` — abstract, overridden per option class; patch each concrete `*Option.useOption` (RestOption, SmithOption, LiftOption, TokeOption, DigOption, RecallOption) — small closed set | `AbstractCampfireOption.java:84` |
| SHOP_BUY_CARD {idx} | `ShopScreen.purchaseCard(AbstractCard)` Prefix (private — SpirePatch handles) | `ShopScreen.java:589` |
| SHOP_BUY_RELIC / SHOP_BUY_POTION {idx} | `StoreRelic.purchaseRelic()` / `StorePotion.purchasePotion()`. REFINE-AT-P2: confirm names | — |
| SHOP_REMOVE | `ShopScreen.purchasePurge()` Prefix | `ShopScreen.java:966` |
| CHEST_OPEN | `AbstractChest.open(boolean)` Prefix | `AbstractChest.java:61` |
| BOSS_RELIC {pick} | Two commit points, deduped by relic instance in `onBossRelic`: (1) `instantObtain` inside `BossRelicSelectScreen.update()`, for the 4 relics that skip the normal `obtain()` (Black Blood, Ring of the Serpent, FrozenCore, HolyWater); (2) `AbstractRelic.bossObtainLogic()` Postfix, for every other relic — the mouse-confirm path (`AbstractRelic.update()` on `hb.clicked`) and the touch-confirm path (`BossRelicSelectScreen.updateConfirmButton()`) both call this method directly. (1) alone missed ordinary relics entirely (pilot: Empty Cage, idx 67->68, no BOSS_RELIC action) since `r.update()`'s call chain is separate bytecode from `BossRelicSelectScreen.update()`'s own body that the `instantObtain` `ExprEditor` never sees. SKIP is not recorded (no state change results from it) | `BossRelicSelectScreen.java:199-201,225-235`, `AbstractRelic.java:355-365,387-394` |
| CHOOSE {indices} (grid/hand selects: Omniscience, discard picks, transforms, scry…) | `GridCardSelectScreen` confirm commit + `HandCardSelectScreen` confirm (`selectedCards` at confirm) + scry screen equivalent. Coordinate names with Wave-2 schema | classes confirmed |
| SAVE_QUIT / ABANDON (lifecycle, classification) | Not a commit hook — decided in `Recorder.update()` the frame `CardCrawlGame.isInARun()` goes false with an open recording (death/victory already closed the recorder via their own patches by then and never reach this branch). Calls `RecordWriter.hasUsableSave()`, which mirrors the file-existence half of `SaveAndContinue.saveExistsAndNotCorrupted(AbstractPlayer)` — SAVE_QUIT if a save file exists for the run's character, ABANDON otherwise. Replaces a 600-frame-since-last-save heuristic that misclassified pilot sitting 1 (a real save-and-quit recorded as ABANDON). `AbstractDungeon.player` is already null by this point (`MainMenuScreen`'s constructor nulls it), so the character comes from the run's own `meta.json`, not a live player instance | `SaveAndContinue.java:51-68`, `MainMenuScreen.java:116` |
| RESUME (lifecycle) | `CardCrawlGame.loadPlayerSave` Postfix sets `Recorder.resumeDetected`; `RecordWriter.open()` then calls `findReattachable`, which matches the most-recent run directory (same seed_long + character) with status `in_progress` (crash case) OR `SAVE_QUIT` (normal continue case — was previously ONLY matched on `in_progress`, which meant a save-and-quit could never reattach and instead split into a second run directory on resume, as pilot sitting 1/2 did). A `SAVE_QUIT` match is flipped back to `in_progress`, gets a new sitting appended, and a RESUME lifecycle record is written | `CardCrawlGame.java:855` |
| RUN_START (lifecycle) | Dungeon generation for a fresh run (existing TraceLabMod launch flow already observes this) | existing |
| RUN_END {VICTORY/DEATH/SAVE_QUIT/ABANDON} | `DeathScreen` ctor, `VictoryScreen` ctor close the recorder directly; SAVE_QUIT/ABANDON classification is the `Recorder.update()` branch described above | resolved |

## Unknown-action safety net

Any commit observed through a generic path that doesn't map to the table emits
`{"type":"UNKNOWN","raw":{class, method, args...}}` — never dropped. The
validator (P4) fails a recording containing UNKNOWN, which is the signal to add
the missing hook. This is the completeness ratchet.

This only catches decisions that reach a commit hook of *some* kind. A
decision with NO hook at all (like Empty Cage's `BOSS_RELIC` before the fix
above) produces no record whatsoever, so UNKNOWN can't see it — its effect
just silently rides along on the state snapshot of whatever the next
recorded action happens to be. `scripts/validate_recording.sh` complements
the UNKNOWN check with a state-delta cross-check: it diffs `post.relics`
length and `deck` length between consecutive action records and fails if
either changes on an action type that can't legitimately explain it (see the
allowlist and its derivation comment in `validate_recording.sh`). This is
how the Empty Cage gap was originally found (pilot sitting 2, idx 67->68) and
is now the regression test for it.

## Snapshot pairing rule

One pending action at a time: a commit hook while another pending action awaits
its stable-state snapshot indicates a compound game flow (e.g. potion → card
reward); the recorder queues them FIFO and pairs each with the first stable
state after its own resolution. If the queue exceeds depth 4, dump diagnostics
and emit UNKNOWN rather than misattribute state.
