package evtracker.patches;

import com.evacipated.cardcrawl.modthespire.lib.*;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.cards.CardGroup;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.rewards.RewardItem;
import com.megacrit.cardcrawl.screens.CardRewardScreen;
import evtracker.CardRewardEV;
import evtracker.EVTrackerMod;

import java.util.ArrayList;

/**
 * Patches for tracking key game decisions.
 *
 * Note: Use BaseMod subscriber interfaces where possible (more stable across versions).
 * Only use SpirePatch for things BaseMod doesn't expose.
 */
public class DecisionTrackingPatches {

    // ========== CARD REWARD TRACKING ==========

    /**
     * Hook when card reward screen opens to log what cards were offered
     */
    @SpirePatch(
        clz = CardRewardScreen.class,
        method = "open",
        paramtypez = {ArrayList.class, RewardItem.class, String.class}
    )
    public static class CardRewardOpenPatch {
        @SpirePostfixPatch
        public static void Postfix(CardRewardScreen __instance, ArrayList<AbstractCard> cards,
                                   RewardItem rItem, String header) {
            if (cards != null && !cards.isEmpty()) {
                EVTrackerMod.onCardRewardPresented(cards);
                // Initialize card reward EV display
                CardRewardEV.onRewardOpen(cards);
            }
        }
    }

    /**
     * Hook when a card is acquired from rewards
     */
    @SpirePatch(
        clz = CardRewardScreen.class,
        method = "acquireCard"
    )
    public static class CardRewardAcquirePatch {
        @SpirePostfixPatch
        public static void Postfix(CardRewardScreen __instance, AbstractCard hoveredCard) {
            if (hoveredCard != null) {
                EVTrackerMod.onCardObtained(hoveredCard, "reward");
            }
        }
    }

    // ========== CARD REMOVAL TRACKING ==========

    /**
     * Hook when a card is removed from master deck
     */
    @SpirePatch(
        clz = CardGroup.class,
        method = "removeCard",
        paramtypez = {AbstractCard.class}
    )
    public static class CardRemovePatch {
        @SpirePrefixPatch
        public static void Prefix(CardGroup __instance, AbstractCard c) {
            // Only track master deck removals
            if (AbstractDungeon.player != null &&
                __instance == AbstractDungeon.player.masterDeck && c != null) {
                String source = "unknown";
                if (AbstractDungeon.getCurrRoom() != null) {
                    String roomClass = AbstractDungeon.getCurrRoom().getClass().getSimpleName();
                    if (roomClass.equals("ShopRoom")) {
                        source = "shop";
                    } else if (roomClass.contains("Event")) {
                        source = "event";
                    }
                }
                EVTrackerMod.onCardRemoved(c, source);
            }
        }
    }

    // Note: Card upgrade tracking is done via AbstractCard.upgrade() but that method
    // is called very frequently during card initialization. Better to track via
    // specific upgrade sources (campfire UI, events) if needed.

    // Note: Potion use is tracked via BaseMod's PostPotionUseSubscriber in EVTrackerMod

    // Note: Scaling cards (Feed, Ritual Dagger) are tracked via their specific
    // onMonsterDeath callbacks, but those require knowing exact card implementations.
    // For now, we track damage/HP changes at a higher level.
}
