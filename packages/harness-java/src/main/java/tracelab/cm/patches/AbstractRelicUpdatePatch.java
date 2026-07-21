// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.evacipated.cardcrawl.modthespire.lib.LineFinder
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$FieldAccessMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$MethodCallMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch
 *  com.evacipated.cardcrawl.modthespire.lib.SpirePatch
 *  com.evacipated.cardcrawl.modthespire.patcher.PatchingException
 *  com.megacrit.cardcrawl.helpers.Hitbox
 *  com.megacrit.cardcrawl.relics.AbstractRelic
 *  javassist.CannotCompileException
 *  javassist.CtBehavior
 */
package tracelab.cm.patches;

import com.evacipated.cardcrawl.modthespire.lib.LineFinder;
import com.evacipated.cardcrawl.modthespire.lib.Matcher;
import com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator;
import com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch;
import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.evacipated.cardcrawl.modthespire.patcher.PatchingException;
import com.megacrit.cardcrawl.helpers.Hitbox;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import tracelab.cm.GameStateListener;
import java.util.ArrayList;
import javassist.CannotCompileException;
import javassist.CtBehavior;

@SpirePatch(clz=AbstractRelic.class, method="update")
public class AbstractRelicUpdatePatch {
    public static AbstractRelic hoverRelic;
    public static boolean doHover;

    @SpireInsertPatch(locator=ObtainedLocator.class)
    public static void BlockStateChange(AbstractRelic _instance) {
        if (_instance.isObtained) {
            GameStateListener.blockStateUpdate();
        }
    }

    @SpireInsertPatch(locator=EquipLocator.class)
    public static void ResumeStateChange(AbstractRelic _instance) {
        GameStateListener.resumeStateUpdate();
    }

    @SpireInsertPatch(locator=HitboxLocator.class)
    public static void DoHitboxHover(AbstractRelic _instance) {
        if (doHover) {
            if (hoverRelic == _instance) {
                _instance.hb.hovered = true;
                _instance.hb.clicked = true;
                doHover = false;
            } else {
                _instance.hb.hovered = false;
            }
        }
    }

    static {
        doHover = false;
    }

    private static class HitboxLocator
    extends SpireInsertLocator {
        private HitboxLocator() {
        }

        public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
            Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(Hitbox.class, "update");
            int[] results = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
            results[0] = results[0] + 1;
            return results;
        }
    }

    private static class EquipLocator
    extends SpireInsertLocator {
        private EquipLocator() {
        }

        public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
            Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(AbstractRelic.class, "onEquip");
            return LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
        }
    }

    private static class ObtainedLocator
    extends SpireInsertLocator {
        private ObtainedLocator() {
        }

        public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
            Matcher.FieldAccessMatcher matcher = new Matcher.FieldAccessMatcher(AbstractRelic.class, "isObtained");
            int[] results = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
            results[0] = results[0] + 1;
            return results;
        }
    }
}

