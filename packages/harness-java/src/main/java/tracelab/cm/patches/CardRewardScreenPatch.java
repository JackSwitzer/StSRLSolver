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
 *  com.megacrit.cardcrawl.cards.AbstractCard
 *  com.megacrit.cardcrawl.screens.CardRewardScreen
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
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.screens.CardRewardScreen;
import java.util.ArrayList;
import javassist.CannotCompileException;
import javassist.CtBehavior;

public class CardRewardScreenPatch {
    public static boolean doHover = false;
    public static AbstractCard hoverCard;

    @SpirePatch(clz=CardRewardScreen.class, method="cardSelectUpdate")
    public static class AcquireCardPatch {
        @SpireInsertPatch(locator=Locator.class)
        public static void Insert(CardRewardScreen _instance) {
            doHover = false;
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.FieldAccessMatcher matcher = new Matcher.FieldAccessMatcher(CardRewardScreen.class, "skipButton");
                return LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
            }
        }
    }

    @SpirePatch(clz=CardRewardScreen.class, method="cardSelectUpdate")
    public static class HoverCardPatch {
        @SpireInsertPatch(locator=Locator.class, localvars={"c"})
        public static void Insert(CardRewardScreen _instance, AbstractCard c) {
            if (doHover) {
                if (c.equals(hoverCard)) {
                    CardRewardScreenPatch.hoverCard.hb.hovered = true;
                } else {
                    c.hb.hovered = false;
                }
            }
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(AbstractCard.class, "updateHoverLogic");
                int[] match = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
                match[0] = match[0] + 1;
                return match;
            }
        }
    }
}

