package tracelab.patches;

import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.events.AbstractEvent;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.vfx.campfire.CampfireSleepEffect;
import com.megacrit.cardcrawl.vfx.campfire.CampfireSmithEffect;
import tracelab.Script;
import tracelab.ScriptRunner;

/**
 * Record-mode hooks: capture the human player's actions so TraceLab writes
 * the same per-action golden records as scripted/auto mode. Active only when
 * the script has mode:"record".
 */
public class RecordHooks {

    @SpirePatch(clz = AbstractPlayer.class, method = "useCard")
    public static class UseCardPatch {
        public static void Prefix(AbstractPlayer __instance, AbstractCard c,
                                  AbstractMonster monster, int energyOnUse) {
            if (!ScriptRunner.recording()) {
                return;
            }
            int handIdx = __instance.hand.group.indexOf(c);
            if (c.dontTriggerOnUseCard || handIdx < 0) {
                return;
            }
            Script.Action a = new Script.Action();
            a.type = "PLAY_CARD";
            a.hand_idx = handIdx;
            a.card_id = c.cardID;
            a.target = monster != null && AbstractDungeon.getMonsters() != null
                    ? AbstractDungeon.getMonsters().monsters.indexOf(monster) : null;
            ScriptRunner.recordAction(a);
        }
    }

    @SpirePatch(clz = com.megacrit.cardcrawl.ui.panels.TopPanel.class, method = "destroyPotion")
    public static class UsePotionPatch {
        public static void Prefix(com.megacrit.cardcrawl.ui.panels.TopPanel __instance, int slot) {
            if (!ScriptRunner.recording() || AbstractDungeon.player == null
                    || slot < 0 || slot >= AbstractDungeon.player.potions.size()) {
                return;
            }
            AbstractPotion potion = AbstractDungeon.player.potions.get(slot);
            if (potion == null || "Potion Slot".equals(potion.ID)) {
                return;
            }
            Script.Action a = new Script.Action();
            a.type = "USE_POTION";
            a.idx = slot;
            a.card_id = potion.ID;
            ScriptRunner.recordAction(a);
        }
    }

    @SpirePatch(clz = com.megacrit.cardcrawl.events.RoomEventDialog.class,
            method = "getSelectedOption")
    public static class RoomEventChoicePatch {
        public static void Postfix(int __result) {
            recordEventChoice(__result);
        }
    }

    @SpirePatch(clz = com.megacrit.cardcrawl.events.GenericEventDialog.class,
            method = "getSelectedOption")
    public static class GenericEventChoicePatch {
        public static void Postfix(int __result) {
            recordEventChoice(__result);
        }
    }

    private static void recordEventChoice(int choice) {
        if (!ScriptRunner.recording() || choice < 0) {
            return;
        }
        Script.Action a = new Script.Action();
        AbstractRoom room = ScriptRunner.currRoom();
        a.type = room != null && room.getClass().getSimpleName().equals("NeowRoom")
                ? "NEOW" : "EVENT_CHOICE";
        a.choice = choice;
        ScriptRunner.recordAction(a);
    }

    @SpirePatch(clz = CampfireSleepEffect.class, method = SpirePatch.CONSTRUCTOR)
    public static class CampfireRestPatch {
        public static void Prefix() {
            recordCampfire("REST");
        }
    }

    @SpirePatch(clz = CampfireSmithEffect.class, method = SpirePatch.CONSTRUCTOR)
    public static class CampfireSmithPatch {
        public static void Prefix() {
            recordCampfire("SMITH");
        }
    }

    private static void recordCampfire(String what) {
        if (!ScriptRunner.recording()) {
            return;
        }
        Script.Action a = new Script.Action();
        a.type = "CAMPFIRE";
        a.choice_name = what;
        ScriptRunner.recordAction(a);
    }

    @SpirePatch(clz = AbstractRoom.class, method = "endTurn")
    public static class EndTurnPatch {
        public static void Prefix(AbstractRoom __instance) {
            if (!ScriptRunner.recording()
                    || __instance.phase != AbstractRoom.RoomPhase.COMBAT
                    || AbstractDungeon.player == null || AbstractDungeon.player.isDead) {
                return;
            }
            Script.Action a = new Script.Action();
            a.type = "END_TURN";
            ScriptRunner.recordAction(a);
        }
    }
}
