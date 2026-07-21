// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  basemod.ReflectionHacks
 *  com.evacipated.cardcrawl.modthespire.lib.LineFinder
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$MethodCallMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch
 *  com.evacipated.cardcrawl.modthespire.lib.SpirePatch
 *  com.evacipated.cardcrawl.modthespire.patcher.PatchingException
 *  com.megacrit.cardcrawl.cards.AbstractCard
 *  com.megacrit.cardcrawl.shop.ShopScreen
 *  javassist.CannotCompileException
 *  javassist.CtBehavior
 */
package tracelab.cm.patches;

import basemod.ReflectionHacks;
import com.evacipated.cardcrawl.modthespire.lib.LineFinder;
import com.evacipated.cardcrawl.modthespire.lib.Matcher;
import com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator;
import com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch;
import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.evacipated.cardcrawl.modthespire.patcher.PatchingException;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.shop.ShopScreen;
import tracelab.cm.GameStateListener;
import java.util.ArrayList;
import javassist.CannotCompileException;
import javassist.CtBehavior;

public class ShopScreenPatch {
    public static boolean doHover = false;
    public static AbstractCard hoverCard;

    @SpirePatch(clz=ShopScreen.class, method="update")
    public static class HoverCardPatch {
        @SpireInsertPatch(locator=Locator.class)
        public static void Insert(ShopScreen _instance) {
            if (doHover) {
                ArrayList coloredCards = (ArrayList)ReflectionHacks.getPrivate((Object)_instance, ShopScreen.class, (String)"coloredCards");
                ArrayList colorlessCards = (ArrayList)ReflectionHacks.getPrivate((Object)_instance, ShopScreen.class, (String)"colorlessCards");
                for (AbstractCard card : (java.util.ArrayList<AbstractCard>)coloredCards) {
                    card.hb.hovered = card == hoverCard;
                }
                for (AbstractCard card : (java.util.ArrayList<AbstractCard>)colorlessCards) {
                    card.hb.hovered = card == hoverCard;
                }
                doHover = false;
            }
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(ShopScreen.class, "updateHand");
                return LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
            }
        }
    }

    @SpirePatch(clz=ShopScreen.class, method="purgeCard")
    public static class PurgeCardPatch {
        public static void Postfix() {
            GameStateListener.resumeStateUpdate();
        }
    }
}

