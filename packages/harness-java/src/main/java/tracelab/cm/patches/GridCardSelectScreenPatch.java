// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  basemod.ReflectionHacks
 *  com.evacipated.cardcrawl.modthespire.lib.SpirePatch
 *  com.megacrit.cardcrawl.cards.AbstractCard
 *  com.megacrit.cardcrawl.screens.select.GridCardSelectScreen
 */
package tracelab.cm.patches;

import basemod.ReflectionHacks;
import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.screens.select.GridCardSelectScreen;

@SpirePatch(clz=GridCardSelectScreen.class, method="updateCardPositionsAndHoverLogic")
public class GridCardSelectScreenPatch {
    public static AbstractCard hoverCard;
    public static boolean replaceHoverCard;

    public static void Postfix(GridCardSelectScreen _instance) {
        if (replaceHoverCard) {
            ReflectionHacks.setPrivate((Object)_instance, GridCardSelectScreen.class, (String)"hoveredCard", (Object)hoverCard);
            GridCardSelectScreenPatch.hoverCard.hb.hovered = true;
            GridCardSelectScreenPatch.hoverCard.hb.clicked = true;
            replaceHoverCard = false;
        }
    }

    static {
        replaceHoverCard = false;
    }
}

