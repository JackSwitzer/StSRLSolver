package tracelab.patches;

import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.events.AbstractEvent;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.ui.campfire.AbstractCampfireOption;
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
            Script.Action a = new Script.Action();
            a.type = "PLAY_CARD";
            a.hand_idx = __instance.hand.group.indexOf(c);
            a.card_id = c.cardID;
            a.target = monster != null && AbstractDungeon.getMonsters() != null
                    ? AbstractDungeon.getMonsters().monsters.indexOf(monster) : null;
            ScriptRunner.recordAction(a);
        }
    }

    @SpirePatch(clz = AbstractPotion.class, method = "use")
    public static class UsePotionPatch {
        public static void Prefix(AbstractPotion __instance, AbstractMonster m) {
            if (!ScriptRunner.recording()) {
                return;
            }
            Script.Action a = new Script.Action();
            a.type = "USE_POTION";
            a.idx = AbstractDungeon.player != null
                    ? AbstractDungeon.player.potions.indexOf(__instance) : null;
            a.card_id = __instance.ID;
            a.target = m != null && AbstractDungeon.getMonsters() != null
                    ? AbstractDungeon.getMonsters().monsters.indexOf(m) : null;
            ScriptRunner.recordAction(a);
        }
    }

    @SpirePatch(clz = AbstractEvent.class, method = "buttonEffect")
    public static class EventChoicePatch {
        public static void Prefix(AbstractEvent __instance, int buttonPressed) {
            if (!ScriptRunner.recording()) {
                return;
            }
            Script.Action a = new Script.Action();
            AbstractRoom room = ScriptRunner.currRoom();
            a.type = room != null && room.getClass().getSimpleName().equals("NeowRoom")
                    ? "NEOW" : "EVENT_CHOICE";
            a.choice = buttonPressed;
            ScriptRunner.recordAction(a);
        }
    }

    @SpirePatch(clz = AbstractCampfireOption.class, method = "useOption")
    public static class CampfirePatch {
        public static void Prefix(AbstractCampfireOption __instance) {
            if (!ScriptRunner.recording()) {
                return;
            }
            Script.Action a = new Script.Action();
            a.type = "CAMPFIRE";
            a.choice_name = __instance.getClass().getSimpleName().replace("Option", "").toUpperCase();
            ScriptRunner.recordAction(a);
        }
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
