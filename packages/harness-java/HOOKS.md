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
| EVENT_CHOICE {choice} | `AbstractEvent.buttonEffect` is abstract — patch `GenericEventDialog`/`RoomEventDialog` option-pressed dispatch which routes to subclass buttonEffect. REFINE-AT-P2: dialog class carries the pressed index | `AbstractEvent.java:95` |
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
| SAVE_QUIT (lifecycle) | `SaveAndContinue.save(SaveFile)` Postfix when triggered from the save+quit button (not autosave) — REFINE-AT-P2: distinguish; fallback = record QUIT on `CardCrawlGame` dispose with active run | `SaveAndContinue.java:125` |
| RESUME (lifecycle) | Continue-run load path (`CardCrawlGame.loadPlayerSave` / SaveFile apply) Postfix — reopen artifacts by run-id | REFINE-AT-P2 |
| RUN_START (lifecycle) | Dungeon generation for a fresh run (existing TraceLabMod launch flow already observes this) | existing |
| RUN_END {VICTORY/DEATH/ABANDON} | `DeathScreen` ctor, `VictoryScreen` ctor, abandon via main-menu path | REFINE-AT-P2 |

## Unknown-action safety net

Any commit observed through a generic path that doesn't map to the table emits
`{"type":"UNKNOWN","raw":{class, method, args...}}` — never dropped. The
validator (P4) fails a recording containing UNKNOWN, which is the signal to add
the missing hook. This is the completeness ratchet.

## Snapshot pairing rule

One pending action at a time: a commit hook while another pending action awaits
its stable-state snapshot indicates a compound game flow (e.g. potion → card
reward); the recorder queues them FIFO and pairs each with the first stable
state after its own resolution. If the queue exceeds depth 4, dump diagnostics
and emit UNKNOWN rather than misattribute state.
