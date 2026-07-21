// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.evacipated.cardcrawl.modthespire.lib.LineFinder
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher
 *  com.evacipated.cardcrawl.modthespire.lib.Matcher$MethodCallMatcher
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertLocator
 *  com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch
 *  com.evacipated.cardcrawl.modthespire.lib.SpirePatch
 *  com.evacipated.cardcrawl.modthespire.patcher.PatchingException
 *  com.megacrit.cardcrawl.helpers.Hitbox
 *  com.megacrit.cardcrawl.helpers.input.InputHelper
 *  com.megacrit.cardcrawl.shop.Merchant
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
import com.megacrit.cardcrawl.helpers.input.InputHelper;
import com.megacrit.cardcrawl.shop.Merchant;
import java.util.ArrayList;
import javassist.CannotCompileException;
import javassist.CtBehavior;

public class MerchantPatch {
    public static boolean visitMerchant = false;

    @SpirePatch(clz=Merchant.class, method="update")
    public static class MerchantUpdatePatch {
        @SpireInsertPatch(locator=Locator.class)
        public static void Insert(Merchant _instance) {
            if (visitMerchant) {
                _instance.hb.hovered = true;
                InputHelper.justClickedLeft = true;
                visitMerchant = false;
            }
        }

        private static class Locator
        extends SpireInsertLocator {
            private Locator() {
            }

            public int[] Locate(CtBehavior ctMethodToPatch) throws CannotCompileException, PatchingException {
                Matcher.MethodCallMatcher matcher = new Matcher.MethodCallMatcher(Hitbox.class, "update");
                int[] results = LineFinder.findInOrder((CtBehavior)ctMethodToPatch, new ArrayList(), (Matcher)matcher);
                results[0] = results[0] + 1;
                return results;
            }
        }
    }
}

