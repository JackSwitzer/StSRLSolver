package tracelab.patches;

import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.AbstractCreature;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.map.MapRoomNode;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.monsters.MonsterGroup;
import com.megacrit.cardcrawl.neow.NeowEvent;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rewards.RewardItem;
import com.megacrit.cardcrawl.rewards.chests.AbstractChest;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.saveAndContinue.SaveAndContinue;
import com.megacrit.cardcrawl.saveAndContinue.SaveFile;
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

    // All ordinary events route option clicks through
    // AbstractEvent.update -> this.buttonEffect(i) (decompiled
    // AbstractEvent.java:102). NeowEvent overrides update, so no dedupe needed.
    @SpirePatch(clz = com.megacrit.cardcrawl.events.AbstractEvent.class, method = "update")
    public static class EventChoice {
        public static ExprEditor Instrument() {
            return new ExprEditor() {
                @Override
                public void edit(MethodCall m) throws CannotCompileException {
                    if (m.getMethodName().equals("buttonEffect")) {
                        m.replace("{ tracelab.patches.RecordPatches.onEventChoice(this, $1); $proceed($$); }");
                    }
                }
            };
        }
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

    // Boss relic commit = instantObtain inside BossRelicSelectScreen.update
    // (decompiled BossRelicSelectScreen.java:200).
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

    public static void onBossRelic(Object relicObj) {
        if (Recorder.active() && relicObj instanceof AbstractRelic) {
            Recorder.commit("BOSS_RELIC", "relic_id", ((AbstractRelic) relicObj).relicId);
        }
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

    @SpirePatch(clz = SaveAndContinue.class, method = "save")
    public static class NoteSave {
        public static void Postfix(SaveFile save) {
            Recorder.noteSave();
        }
    }

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
