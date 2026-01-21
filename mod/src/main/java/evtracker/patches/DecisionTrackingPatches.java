package evtracker.patches;

import com.evacipated.cardcrawl.modthespire.lib.*;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.cards.CardGroup;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.rewards.RewardItem;
import com.megacrit.cardcrawl.screens.CardRewardScreen;
import evtracker.EVTrackerMod;
import evtracker.CombatReview;

import java.util.ArrayList;

/**
 * Patches for tracking key game decisions:
 * - Card rewards (what was offered, what was picked/skipped)
 * - Card upgrades
 * - Card removals
 */
public class DecisionTrackingPatches {

    // ========== CARD REWARD TRACKING ==========

    /**
     * Hook when card reward screen opens to log what cards were offered
     */
    @SpirePatch(
        clz = CardRewardScreen.class,
        method = "open"
    )
    public static class CardRewardOpenPatch {
        @SpirePostfixPatch
        public static void Postfix(CardRewardScreen __instance, ArrayList<AbstractCard> cards,
                                   RewardItem rItem, String header) {
            if (cards != null && !cards.isEmpty()) {
                EVTrackerMod.onCardRewardPresented(cards);
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

    // ========== CARD UPGRADE TRACKING ==========

    /**
     * Hook when any card is upgraded
     */
    @SpirePatch(
        clz = AbstractCard.class,
        method = "upgrade"
    )
    public static class CardUpgradePatch {
        @SpirePostfixPatch
        public static void Postfix(AbstractCard __instance) {
            // Determine source based on current game state
            String source = "unknown";
            if (AbstractDungeon.getCurrRoom() != null) {
                String roomClass = AbstractDungeon.getCurrRoom().getClass().getSimpleName();
                if (roomClass.equals("RestRoom")) {
                    source = "campfire";
                } else if (roomClass.contains("Event")) {
                    source = "event";
                } else if (roomClass.equals("ShopRoom")) {
                    source = "shop";
                }
            }
            EVTrackerMod.onCardUpgraded(__instance, source);
        }
    }

    // ========== CARD REMOVAL TRACKING ==========

    /**
     * Hook when a card is removed from deck via CardGroup.removeCard
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
            if (__instance == AbstractDungeon.player.masterDeck && c != null) {
                // Determine source
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

    // ========== CARD TRANSFORM TRACKING ==========

    /**
     * Hook for card transforms (Astrolabe, events, etc.)
     * This catches when a card is transformed into another
     */
    @SpirePatch(
        clz = AbstractDungeon.class,
        method = "transformCard",
        paramtypez = {AbstractCard.class, boolean.class, com.badlogic.gdx.math.Vector2.class}
    )
    public static class CardTransformPatch {
        @SpirePrefixPatch
        public static void Prefix(AbstractCard c, boolean autoUpgrade,
                                  com.badlogic.gdx.math.Vector2 transformPostion) {
            if (c != null) {
                EVTrackerMod.onCardRemoved(c, "transform");
            }
        }

        @SpirePostfixPatch
        public static void Postfix(AbstractCard c, boolean autoUpgrade,
                                   com.badlogic.gdx.math.Vector2 transformPostion) {
            // The new card would be tracked via onCardObtained when added to deck
        }
    }

    // ========== POTION USE TRACKING ==========

    /**
     * Hook when a potion is used
     */
    @SpirePatch(
        clz = AbstractPotion.class,
        method = "use",
        paramtypez = {AbstractMonster.class}
    )
    public static class PotionUsePatch {
        @SpirePostfixPatch
        public static void Postfix(AbstractPotion __instance, AbstractMonster target) {
            String targetName = target != null ? target.name : null;
            CombatReview.onPotionUsed(__instance, targetName);
        }
    }

    // ========== SCALING CARD TRACKING ==========

    /**
     * Hook for Ritual Dagger permanent damage increase
     * Note: This requires checking the specific card implementations
     * For now we hook onMonsterDeath for scaling cards
     */
    @SpirePatch(
        clz = AbstractMonster.class,
        method = "die",
        paramtypez = {}  // The no-arg version of die()
    )
    public static class MonsterDeathScalingPatch {
        @SpirePostfixPatch
        public static void Postfix(AbstractMonster __instance) {
            // Check for Feed in player's exhaust pile (just played)
            // Feed adds 3 HP on killing blow
            for (AbstractCard c : AbstractDungeon.player.exhaustPile.group) {
                if (c.cardID.equals("Feed") && !c.upgraded) {
                    CombatReview.onScalingEvent("Feed", "hp", 3);
                } else if (c.cardID.equals("Feed") && c.upgraded) {
                    CombatReview.onScalingEvent("Feed+", "hp", 4);
                }
            }

            // Check for Ritual Dagger
            for (AbstractCard c : AbstractDungeon.player.exhaustPile.group) {
                if (c.cardID.equals("RitualDagger")) {
                    CombatReview.onScalingEvent("Ritual Dagger", "damage", 3);
                }
            }

            // Check for Lesson Learned
            for (AbstractCard c : AbstractDungeon.player.exhaustPile.group) {
                if (c.cardID.equals("Lesson Learned")) {
                    CombatReview.onScalingEvent("Lesson Learned", "card_upgrade", 1);
                }
            }

            // Hand of Greed gold gain
            for (AbstractCard c : AbstractDungeon.player.exhaustPile.group) {
                if (c.cardID.equals("HandOfGreed") && !c.upgraded) {
                    CombatReview.onScalingEvent("Hand of Greed", "gold", 20);
                } else if (c.cardID.equals("HandOfGreed") && c.upgraded) {
                    CombatReview.onScalingEvent("Hand of Greed+", "gold", 25);
                }
            }
        }
    }
}
