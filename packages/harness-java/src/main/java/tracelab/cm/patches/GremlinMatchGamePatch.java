// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  basemod.ReflectionHacks
 *  com.evacipated.cardcrawl.modthespire.lib.LineFinder
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$FieldAccessMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$MethodCallMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch
 *  com.evacipated.cardcrawl.modthespire.lib.SpirePatch
 *  com.evacipated.cardcrawl.modthespire.patcher.PatchingException
 *  com.megacrit.cardcrawl.cards.AbstractCard
 *  com.megacrit.cardcrawl.cards.CardGroup
 *  com.megacrit.cardcrawl.events.shrines.GremlinMatchGame
 *  com.megacrit.cardcrawl.helpers.Hitbox
 *  com.megacrit.cardcrawl.helpers.input.InputHelper
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
import com.megacrit.cardcrawl.cards.CardGroup;
import com.megacrit.cardcrawl.events.shrines.GremlinMatchGame;
import com.megacrit.cardcrawl.helpers.Hitbox;
import com.megacrit.cardcrawl.helpers.input.InputHelper;
import tracelab.cm.GameStateListener;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Set;
import java.util.UUID;
import javassist.CannotCompileException;
import javassist.CtBehavior;

public class GremlinMatchGamePatch {
    public static HashMap<UUID, Integer> cardPositions;
    public static CardGroup cards;
    public static Set<UUID> revealedCards;

    public static ArrayList<AbstractCard> getOrderedCards() {
        ArrayList<AbstractCard> returnedCards = new ArrayList<AbstractCard>(GremlinMatchGamePatch.cards.group);
        returnedCards.sort(Comparator.comparingInt(c -> cardPositions.get(c.uuid)));
        returnedCards.removeIf(c -> !c.isFlipped);
        return returnedCards;
    }

    @SpirePatch(clz=GremlinMatchGame.class, method="updateMatchGameLogic")
    public static class RegisterFirstFlipPatch {
        @SpireInsertPatch(locator=Locator.class)
        public static void Insert(GremlinMatchGame _instance) {
            GameStateListener.registerStateChange();
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.FieldAccessMatcher chosenMatcher = new Matcher.FieldAccessMatcher(GremlinMatchGame.class, "chosenCard");
                Matcher.FieldAccessMatcher hoveredMatcher = new Matcher.FieldAccessMatcher(GremlinMatchGame.class, "hoveredCard");
                int[] chosenMatches = LineFinder.findAllInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)chosenMatcher);
                int[] hoveredMatches = LineFinder.findAllInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)hoveredMatcher);
                for (int waitMatch : chosenMatches) {
                    for (int gameDoneMatch : hoveredMatches) {
                        if (waitMatch != gameDoneMatch) continue;
                        int[] match = new int[]{waitMatch};
                        return match;
                    }
                }
                throw new PatchingException("Could not find patching location for RegisterFirstFlipPatch in GremlinMatchGame.");
            }
        }
    }

    @SpirePatch(clz=GremlinMatchGame.class, method="updateMatchGameLogic")
    public static class CardIdentificationPatch {
        @SpireInsertPatch(locator=Locator.class, localvars={"c"})
        public static void Insert(GremlinMatchGame _instance, AbstractCard c) {
            revealedCards.add(c.uuid);
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.FieldAccessMatcher matcher = new Matcher.FieldAccessMatcher(AbstractCard.class, "isFlipped");
                int[] matches = LineFinder.findAllInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
                return Arrays.copyOfRange(matches, 1, 2);
            }
        }
    }

    @SpirePatch(clz=GremlinMatchGame.class, method="updateMatchGameLogic")
    public static class WaitForCardFlipPatch {
        @SpireInsertPatch(locator=Locator.class)
        public static void Insert(GremlinMatchGame _instance) {
            int attemptCount = (Integer)ReflectionHacks.getPrivate((Object)_instance, GremlinMatchGame.class, (String)"attemptCount");
            if (attemptCount > 0) {
                GameStateListener.registerStateChange();
            }
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.FieldAccessMatcher matcher = new Matcher.FieldAccessMatcher(GremlinMatchGame.class, "attemptCount");
                int[] result = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
                result[0] = result[0] + 1;
                return result;
            }
        }
    }

    @SpirePatch(clz=GremlinMatchGame.class, method="updateMatchGameLogic")
    public static class HoverCardPatch {
        public static boolean doHover = false;
        public static AbstractCard hoverCard = null;

        @SpireInsertPatch(locator=Locator.class, localvars={"c"})
        public static void Insert(GremlinMatchGame _instance, AbstractCard c) {
            if (doHover) {
                if (c.equals(hoverCard)) {
                    c.hb.hovered = true;
                    InputHelper.justClickedLeft = true;
                    doHover = false;
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
                Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(Hitbox.class, "update");
                int[] result = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
                result[0] = result[0] + 1;
                return result;
            }
        }
    }

    @SpirePatch(clz=GremlinMatchGame.class, method="<ctor>")
    public static class InitializeCardsPatch {
        public static void Postfix(GremlinMatchGame _instance) {
            cards = (CardGroup)ReflectionHacks.getPrivate((Object)_instance, GremlinMatchGame.class, (String)"cards");
            revealedCards = new HashSet<UUID>();
            cardPositions = new HashMap();
            for (int i = 0; i < 12; ++i) {
                AbstractCard currentCard = (AbstractCard)GremlinMatchGamePatch.cards.group.get(i);
                int target_x = i % 4;
                int target_y = i % 3;
                int position = target_x + 4 * target_y;
                cardPositions.put(currentCard.uuid, position);
            }
        }
    }
}

