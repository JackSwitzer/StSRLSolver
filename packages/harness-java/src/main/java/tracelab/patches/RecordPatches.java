package tracelab.patches;

import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.AbstractCreature;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.events.AbstractEvent;
import com.megacrit.cardcrawl.events.GenericEventDialog;
import com.megacrit.cardcrawl.events.RoomEventDialog;
import com.megacrit.cardcrawl.map.MapRoomNode;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.monsters.MonsterGroup;
import com.megacrit.cardcrawl.neow.NeowEvent;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rewards.RewardItem;
import com.megacrit.cardcrawl.rewards.chests.AbstractChest;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.screens.CardRewardScreen;
import com.megacrit.cardcrawl.screens.DeathScreen;
import com.megacrit.cardcrawl.screens.VictoryScreen;
import com.megacrit.cardcrawl.screens.select.BossRelicSelectScreen;
import com.megacrit.cardcrawl.screens.select.GridCardSelectScreen;
import com.megacrit.cardcrawl.screens.select.HandCardSelectScreen;
import com.megacrit.cardcrawl.shop.ShopScreen;
import com.megacrit.cardcrawl.shop.StorePotion;
import com.megacrit.cardcrawl.shop.StoreRelic;
import com.megacrit.cardcrawl.ui.buttons.ProceedButton;
import com.megacrit.cardcrawl.ui.campfire.DigOption;
import com.megacrit.cardcrawl.ui.campfire.LiftOption;
import com.megacrit.cardcrawl.ui.campfire.RecallOption;
import com.megacrit.cardcrawl.ui.campfire.RestOption;
import com.megacrit.cardcrawl.ui.campfire.SmithOption;
import com.megacrit.cardcrawl.ui.campfire.TokeOption;
import com.megacrit.cardcrawl.ui.panels.PotionPopUp;
import com.megacrit.cardcrawl.ui.panels.TopPanel;
import javassist.CannotCompileException;
import javassist.expr.ExprEditor;
import javassist.expr.MethodCall;
import tracelab.Recorder;

import java.util.ArrayList;
import java.util.List;

/**
 * Record-mode decision hooks (see packages/harness-java/HOOKS.md for the
 * commit-point inventory and decompiled-source citations). Every patch is a
 * pure observer: it calls Recorder.commit(...) and never changes game flow.
 */
public class RecordPatches {

    // ---------------------------------------------------------------- combat

    // decompiled AbstractPlayer.java:1356 — fires once per committed card play.
    @SpirePatch(clz = AbstractPlayer.class, method = "useCard")
    public static class UseCard {
        public static void Prefix(AbstractPlayer __instance, AbstractCard c,
                                  AbstractMonster monster, int energyOnUse) {
            if (!Recorder.active()) {
                return;
            }
            Recorder.commit("PLAY_CARD",
                    "card_id", c.cardID,
                    "upgrades", c.timesUpgraded,
                    "hand_idx", __instance.hand.group.indexOf(c),
                    "target", monsterIdx(monster),
                    "energy_on_use", energyOnUse);
        }
    }

    // decompiled AbstractRoom.java:393.
    @SpirePatch(clz = AbstractRoom.class, method = "endTurn")
    public static class EndTurn {
        public static void Prefix(AbstractRoom __instance) {
            if (Recorder.active() && __instance.phase == AbstractRoom.RoomPhase.COMBAT) {
                Recorder.commit("END_TURN");
            }
        }
    }

    // Potion use commits inside PotionPopUp.updateTargetMode (targeted,
    // decompiled PotionPopUp.java:210) and PotionPopUp.updateInput
    // (untargeted, :241). Instrument the `use` calls; the paired
    // TopPanel.destroyPotion is suppressed for that slot the same frame.
    @SpirePatch(clz = PotionPopUp.class, method = "updateTargetMode")
    public static class PotionUseTargeted {
        public static ExprEditor Instrument() {
            return potionUseEditor();
        }
    }

    @SpirePatch(clz = PotionPopUp.class, method = "updateInput")
    public static class PotionUseUntargeted {
        public static ExprEditor Instrument() {
            return potionUseEditor();
        }
    }

    private static ExprEditor potionUseEditor() {
        return new ExprEditor() {
            @Override
            public void edit(MethodCall m) throws CannotCompileException {
                if (m.getMethodName().equals("use")) {
                    m.replace("{ tracelab.patches.RecordPatches.onPotionUse($0, $$); $proceed($$); }");
                }
            }
        };
    }

    public static void onPotionUse(Object potionObj, Object targetObj) {
        if (!Recorder.active() || !(potionObj instanceof AbstractPotion)) {
            return;
        }
        AbstractPotion potion = (AbstractPotion) potionObj;
        int slot = AbstractDungeon.player.potions.indexOf(potion);
        Recorder.potionUsedSlotThisFrame = slot;
        Recorder.commit("USE_POTION",
                "potion_id", potion.ID,
                "slot", slot,
                "target", targetObj instanceof AbstractMonster
                        ? monsterIdx((AbstractMonster) targetObj) : -1);
    }

    // decompiled TopPanel.java:530 — sole funnel for potion removal; a
    // removal not preceded by a use this frame is a discard.
    @SpirePatch(clz = TopPanel.class, method = "destroyPotion")
    public static class DestroyPotion {
        public static void Prefix(TopPanel __instance, int slot) {
            if (Recorder.active() && Recorder.potionUsedSlotThisFrame != slot) {
                AbstractPotion potion = slot >= 0 && slot < AbstractDungeon.player.potions.size()
                        ? AbstractDungeon.player.potions.get(slot) : null;
                Recorder.commit("DISCARD_POTION",
                        "slot", slot,
                        "potion_id", potion != null ? potion.ID : "UNKNOWN");
            }
        }
    }

    // ------------------------------------------------------------ navigation

    // Path commit = the nextRoomTransitionStart() call inside
    // MapRoomNode.update (decompiled MapRoomNode.java:188-193).
    @SpirePatch(clz = MapRoomNode.class, method = "update")
    public static class PathCommit {
        public static ExprEditor Instrument() {
            return new ExprEditor() {
                @Override
                public void edit(MethodCall m) throws CannotCompileException {
                    if (m.getMethodName().equals("nextRoomTransitionStart")) {
                        m.replace("{ tracelab.patches.RecordPatches.onPathCommit(this); $proceed($$); }");
                    }
                }
            };
        }
    }

    public static void onPathCommit(Object nodeObj) {
        if (!Recorder.active() || !(nodeObj instanceof MapRoomNode)) {
            return;
        }
        MapRoomNode node = (MapRoomNode) nodeObj;
        String symbol = node.getRoom() != null ? node.getRoom().getMapSymbol() : "?";
        Recorder.commit("PATH", "x", node.x, "y", node.y, "symbol", symbol);
    }

    // ProceedButton drives every "continue" transition: post-combat skip,
    // shop leave, chest skip, event exit, boss chest continue.
    @SpirePatch(clz = ProceedButton.class, method = "update")
    public static class Proceed {
        private static java.lang.reflect.Field hbField;

        public static void Prefix(ProceedButton __instance) {
            if (!Recorder.active()) {
                return;
            }
            try {
                if (hbField == null) {
                    hbField = ProceedButton.class.getDeclaredField("hb");
                    hbField.setAccessible(true);
                }
                com.megacrit.cardcrawl.helpers.Hitbox hb =
                        (com.megacrit.cardcrawl.helpers.Hitbox) hbField.get(__instance);
                if (hb != null && hb.clicked) {
                    AbstractRoom room = AbstractDungeon.currMapNode != null
                            ? AbstractDungeon.currMapNode.getRoom() : null;
                    Recorder.commit("PROCEED",
                            "room", room != null ? room.getClass().getSimpleName() : "NONE");
                }
            } catch (ReflectiveOperationException e) {
                // observer only — never break the button
            }
        }
    }

    // ---------------------------------------------------------------- events

    // decompiled NeowEvent.java:161.
    @SpirePatch(clz = NeowEvent.class, method = "buttonEffect")
    public static class NeowChoice {
        public static void Prefix(NeowEvent __instance, int buttonPressed) {
            if (Recorder.active()) {
                Recorder.commit("NEOW", "choice", buttonPressed);
            }
        }
    }

    // AbstractEvent.update() dispatches option clicks via
    // `this.buttonEffect(this.roomEventText.getSelectedOption())`
    // (decompiled AbstractEvent.java:112-114), but 24 of the ~50 event
    // classes OVERRIDE update() themselves (all of them still end by calling
    // super.update() or, for image events, AbstractImageEvent.update()'s own
    // `this.buttonEffect(GenericEventDialog.getSelectedOption())`,
    // decompiled AbstractImageEvent.java:52-54). A patch on
    // AbstractEvent.update() alone never runs for an overridden update() —
    // that's why every image-type event (floors 4, 5, 22 in the pilot) went
    // unrecorded. Every path, without exception, bottoms out in exactly one
    // of two dialog widgets consuming a button press: RoomEventDialog.update
    // (decompiled RoomEventDialog.java:62-70, sets `selectedOption` and
    // flips `waitForInput` false->true) or GenericEventDialog.update
    // (decompiled GenericEventDialog.java:84-92, same pattern). Hooking both
    // dialogs directly supersedes the old AbstractEvent.update instrument,
    // so that patch is replaced rather than stacked (RecordPatches no longer
    // has an AbstractEvent.update hook at all).
    //
    // NeowEvent reuses its own RoomEventDialog instance for its choice UI
    // (decompiled neow/NeowEvent.java:139-140), so RoomEventDialog.update
    // fires for Neow choices too; onEventDialogChoice defers to the existing
    // NeowEvent.buttonEffect Prefix (NeowChoice, above) for those so a Neow
    // pick is never double-recorded as EVENT_CHOICE.
    @SpirePatch(clz = RoomEventDialog.class, method = "update")
    public static class RoomEventChoice {
        private static boolean wasWaitingForInput;

        public static void Prefix() {
            wasWaitingForInput = RoomEventDialog.waitForInput;
        }

        public static void Postfix() {
            if (wasWaitingForInput && !RoomEventDialog.waitForInput) {
                onEventDialogChoice(RoomEventDialog.selectedOption);
            }
        }
    }

    @SpirePatch(clz = GenericEventDialog.class, method = "update")
    public static class ImageEventChoice {
        private static boolean wasWaitingForInput;

        public static void Prefix() {
            wasWaitingForInput = GenericEventDialog.waitForInput;
        }

        public static void Postfix() {
            if (wasWaitingForInput && !GenericEventDialog.waitForInput) {
                onEventDialogChoice(GenericEventDialog.selectedOption);
            }
        }
    }

    private static void onEventDialogChoice(int choice) {
        if (!Recorder.active()) {
            return;
        }
        AbstractRoom room = AbstractDungeon.getCurrRoom();
        AbstractEvent event = room != null ? room.event : null;
        if (event == null) {
            return;
        }
        onEventChoice(event, choice);
    }

    public static void onEventChoice(Object event, int choice) {
        if (Recorder.active() && !(event instanceof NeowEvent)) {
            Recorder.commit("EVENT_CHOICE",
                    "choice", choice,
                    "event", event.getClass().getSimpleName());
        }
    }

    // --------------------------------------------------------------- rewards

    // decompiled RewardItem.java:255 — claimReward sets isDone on success, so
    // a plain Postfix on the instance avoids MTS's return-value param rules.
    @SpirePatch(clz = RewardItem.class, method = "claimReward")
    public static class ClaimReward {
        public static void Postfix(RewardItem __instance) {
            if (!Recorder.active() || !__instance.isDone) {
                return;
            }
            String id = "";
            if (__instance.relic != null) {
                id = __instance.relic.relicId;
            } else if (__instance.potion != null) {
                id = __instance.potion.ID;
            }
            Recorder.commit("REWARD_TAKE",
                    "reward_type", __instance.type != null ? __instance.type.name() : "UNKNOWN",
                    "id", id);
        }
    }

    @SpirePatch(clz = CardRewardScreen.class, method = "acquireCard")
    public static class CardRewardPick {
        public static void Prefix(CardRewardScreen __instance, AbstractCard hoveredCard) {
            if (Recorder.active()) {
                Recorder.commit("CARD_REWARD",
                        "card_id", hoveredCard.cardID,
                        "upgrades", hoveredCard.timesUpgraded);
            }
        }
    }

    // Boss relic has TWO commit points that both need instrumenting:
    //  1. instantObtain inside BossRelicSelectScreen.update, for the 4 relics
    //     that skip the normal obtain() call (Black Blood, Ring of the
    //     Serpent, FrozenCore, HolyWater — decompiled
    //     BossRelicSelectScreen.java:199-201, relicObtainLogic).
    //  2. AbstractRelic.bossObtainLogic() for every OTHER relic (e.g. Empty
    //     Cage) — the mouse-confirm path: AbstractRelic.update() calls
    //     `this.bossObtainLogic()` directly on hb.clicked when not on a
    //     touchscreen (decompiled AbstractRelic.java:355-365), and
    //     bossObtainLogic() itself calls `this.obtain()` for anything not in
    //     the special-4 set (decompiled AbstractRelic.java:387-394). The
    //     touch-confirm path (BossRelicSelectScreen.updateConfirmButton ->
    //     touchRelic.bossObtainLogic(), decompiled
    //     BossRelicSelectScreen.java:225-235) calls the exact same method,
    //     so patching bossObtainLogic covers both. The prior instrument only
    //     rewrote `instantObtain` calls textually present in
    //     BossRelicSelectScreen.update()'s own bytecode; r.update() (and the
    //     bossObtainLogic() it calls) is a different method's bytecode
    //     entirely, so it was never touched — that's the Empty Cage gap
    //     (pilot sitting 2, idx 67->68, no BOSS_RELIC action).
    //
    // For the special 4, BOTH hooks fire on the same frame (r.update() runs
    // bossObtainLogic() first, which still executes even though it skips
    // obtain(); relicObtainLogic's instantObtain call follows immediately in
    // the same BossRelicSelectScreen.update() iteration) — onBossRelic
    // dedupes by relic instance identity so only one BOSS_RELIC commit lands
    // per pick, regardless of which hook observes it first.
    @SpirePatch(clz = BossRelicSelectScreen.class, method = "update")
    public static class BossRelicPick {
        public static ExprEditor Instrument() {
            return new ExprEditor() {
                @Override
                public void edit(MethodCall m) throws CannotCompileException {
                    if (m.getMethodName().equals("instantObtain")) {
                        m.replace("{ tracelab.patches.RecordPatches.onBossRelic($0); $proceed($$); }");
                    }
                }
            };
        }
    }

    @SpirePatch(clz = AbstractRelic.class, method = "bossObtainLogic")
    public static class BossRelicObtain {
        public static void Postfix(AbstractRelic __instance) {
            onBossRelic(__instance);
        }
    }

    private static Object lastBossRelicCommitted = null;

    public static void onBossRelic(Object relicObj) {
        if (!Recorder.active() || !(relicObj instanceof AbstractRelic) || relicObj == lastBossRelicCommitted) {
            return;
        }
        lastBossRelicCommitted = relicObj;
        Recorder.commit("BOSS_RELIC", "relic_id", ((AbstractRelic) relicObj).relicId);
    }

    // decompiled AbstractChest.java:61.
    @SpirePatch(clz = AbstractChest.class, method = "open")
    public static class ChestOpen {
        public static void Prefix(AbstractChest __instance, boolean bossChest) {
            if (Recorder.active()) {
                Recorder.commit("CHEST_OPEN", "chest", __instance.getClass().getSimpleName());
            }
        }
    }

    // -------------------------------------------------------------- campfire

    @SpirePatch(clz = RestOption.class, method = "useOption")
    public static class CampfireRest {
        public static void Prefix() { campfire("REST"); }
    }

    @SpirePatch(clz = SmithOption.class, method = "useOption")
    public static class CampfireSmith {
        public static void Prefix() { campfire("SMITH"); }
    }

    @SpirePatch(clz = LiftOption.class, method = "useOption")
    public static class CampfireLift {
        public static void Prefix() { campfire("LIFT"); }
    }

    @SpirePatch(clz = TokeOption.class, method = "useOption")
    public static class CampfireToke {
        public static void Prefix() { campfire("TOKE"); }
    }

    @SpirePatch(clz = DigOption.class, method = "useOption")
    public static class CampfireDig {
        public static void Prefix() { campfire("DIG"); }
    }

    @SpirePatch(clz = RecallOption.class, method = "useOption")
    public static class CampfireRecall {
        public static void Prefix() { campfire("RECALL"); }
    }

    private static void campfire(String choice) {
        if (Recorder.active()) {
            Recorder.commit("CAMPFIRE", "choice", choice);
        }
    }

    // ------------------------------------------------------------------ shop

    // decompiled ShopScreen.java:589 (private; SpirePatch handles).
    @SpirePatch(clz = ShopScreen.class, method = "purchaseCard")
    public static class ShopBuyCard {
        public static void Prefix(ShopScreen __instance, AbstractCard hoveredCard) {
            if (Recorder.active()) {
                Recorder.commit("SHOP_BUY_CARD",
                        "card_id", hoveredCard.cardID,
                        "upgrades", hoveredCard.timesUpgraded);
            }
        }
    }

    // decompiled ShopScreen.java:966.
    @SpirePatch(clz = ShopScreen.class, method = "purchasePurge")
    public static class ShopRemove {
        public static void Prefix() {
            if (Recorder.active()) {
                Recorder.commit("SHOP_REMOVE");
            }
        }
    }

    // decompiled StoreRelic.java:87 / StorePotion.java:78.
    @SpirePatch(clz = StoreRelic.class, method = "purchaseRelic")
    public static class ShopBuyRelic {
        public static void Prefix(StoreRelic __instance) {
            if (Recorder.active()) {
                Recorder.commit("SHOP_BUY_RELIC",
                        "relic_id", __instance.relic != null ? __instance.relic.relicId : "UNKNOWN");
            }
        }
    }

    @SpirePatch(clz = StorePotion.class, method = "purchasePotion")
    public static class ShopBuyPotion {
        public static void Prefix(StorePotion __instance) {
            if (Recorder.active()) {
                Recorder.commit("SHOP_BUY_POTION",
                        "potion_id", __instance.potion != null ? __instance.potion.ID : "UNKNOWN");
            }
        }
    }

    // ---------------------------------------------------- card select screens

    // Grid/hand select confirms (Omniscience, discards, transforms...).
    // hb.clicked is observed in Prefix before the screen consumes it.
    @SpirePatch(clz = GridCardSelectScreen.class, method = "update")
    public static class GridChoose {
        public static void Prefix(GridCardSelectScreen __instance) {
            if (!Recorder.active() || !__instance.confirmButton.hb.clicked) {
                return;
            }
            Recorder.commit("CHOOSE",
                    "screen", "GRID",
                    "cards", cardIdList(__instance.selectedCards));
        }
    }

    @SpirePatch(clz = HandCardSelectScreen.class, method = "update")
    public static class HandChoose {
        public static void Prefix(HandCardSelectScreen __instance) {
            if (!Recorder.active() || !__instance.button.hb.clicked) {
                return;
            }
            Recorder.commit("CHOOSE",
                    "screen", "HAND",
                    "cards", cardIdList(__instance.selectedCards.group));
        }
    }

    // -------------------------------------------------------------- lifecycle

    // Private (decompiled CardCrawlGame.java:855); marks the next dungeon
    // attach as a resumed run.
    @SpirePatch(clz = CardCrawlGame.class, method = "loadPlayerSave")
    public static class LoadSave {
        public static void Postfix(CardCrawlGame __instance, AbstractPlayer p) {
            Recorder.resumeDetected = true;
        }
    }

    @SpirePatch(clz = DeathScreen.class, method = SpirePatch.CONSTRUCTOR)
    public static class Death {
        public static void Postfix(DeathScreen __instance, MonsterGroup m) {
            Recorder.closeRunFromPatch("DEATH");
        }
    }

    @SpirePatch(clz = VictoryScreen.class, method = SpirePatch.CONSTRUCTOR)
    public static class Victory {
        public static void Postfix(VictoryScreen __instance, MonsterGroup m) {
            Recorder.closeRunFromPatch("VICTORY");
        }
    }

    // ---------------------------------------------------------------- helpers

    private static int monsterIdx(AbstractMonster monster) {
        if (monster == null || AbstractDungeon.getMonsters() == null) {
            return -1;
        }
        return AbstractDungeon.getMonsters().monsters.indexOf(monster);
    }

    private static List<String> cardIdList(Iterable<AbstractCard> cards) {
        List<String> ids = new ArrayList<String>();
        if (cards != null) {
            for (AbstractCard c : cards) {
                ids.add(c.cardID);
            }
        }
        return ids;
    }
}
