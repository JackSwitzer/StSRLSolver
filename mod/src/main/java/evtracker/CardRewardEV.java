package evtracker;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.helpers.ImageMaster;
import com.megacrit.cardcrawl.screens.CardRewardScreen;

import java.util.*;

/**
 * Displays EV comparison for card rewards.
 *
 * Evaluates cards based on:
 * 1. Intrinsic power (card tier/quality)
 * 2. Deck synergy (what does your deck need?)
 * 3. Current run state (floor, HP, relics)
 *
 * Shows "BEST", "GOOD", "OK", "SKIP" labels on reward cards.
 */
public class CardRewardEV {

    private static Map<String, Float> cardScores = new HashMap<>();
    private static List<AbstractCard> lastRewardCards = null;
    private static String bestCardUuid = null;

    /**
     * Called when card reward screen opens
     */
    public static void onRewardOpen(List<AbstractCard> cards) {
        cardScores.clear();
        lastRewardCards = cards;
        bestCardUuid = null;

        if (cards == null || cards.isEmpty()) return;

        // Score each card
        float bestScore = Float.MIN_VALUE;
        for (AbstractCard card : cards) {
            float score = evaluateCard(card);
            cardScores.put(card.uuid.toString(), score);
            if (score > bestScore) {
                bestScore = score;
                bestCardUuid = card.uuid.toString();
            }
        }
    }

    /**
     * Render EV badges on reward cards
     */
    public static void render(SpriteBatch sb) {
        if (!isRewardScreenOpen() || lastRewardCards == null) {
            return;
        }

        // Get the cards from the reward screen
        CardRewardScreen screen = AbstractDungeon.cardRewardScreen;
        if (screen == null || screen.rewardGroup == null) return;

        for (AbstractCard card : screen.rewardGroup) {
            renderCardBadge(sb, card);
        }
    }

    private static void renderCardBadge(SpriteBatch sb, AbstractCard card) {
        String uuid = card.uuid.toString();
        Float score = cardScores.get(uuid);
        if (score == null) {
            score = evaluateCard(card);
            cardScores.put(uuid, score);
        }

        // Determine label and color
        String label;
        Color bgColor;

        if (uuid.equals(bestCardUuid)) {
            label = "BEST";
            bgColor = COLOR_BEST;
        } else if (score >= 7) {
            label = "GOOD";
            bgColor = COLOR_GOOD;
        } else if (score >= 4) {
            label = "OK";
            bgColor = COLOR_OK;
        } else if (score >= 0) {
            label = "MEH";
            bgColor = COLOR_MEH;
        } else {
            label = "SKIP";
            bgColor = COLOR_SKIP;
        }

        // Also show numeric score
        label = label + " (" + String.format("%.0f", score) + ")";

        // Position: top center of card
        float x = card.current_x;
        float y = card.current_y + (card.hb.height / 2) * 0.85f;

        // Badge size
        float badgeWidth = Math.max(60, label.length() * 10) * Settings.scale;
        float badgeHeight = 30 * Settings.scale;

        // Background
        bgColor = bgColor.cpy();
        bgColor.a = 0.9f;
        sb.setColor(bgColor);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG,
                x - badgeWidth/2, y - badgeHeight/2,
                badgeWidth, badgeHeight);

        // Border
        sb.setColor(0, 0, 0, 0.6f);
        float border = 2 * Settings.scale;
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x - badgeWidth/2 - border, y - badgeHeight/2 - border,
                badgeWidth + border*2, border);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x - badgeWidth/2 - border, y + badgeHeight/2,
                badgeWidth + border*2, border);

        // Text
        FontHelper.renderFontCentered(sb, FontHelper.tipBodyFont, label, x, y, Color.WHITE);
    }

    /**
     * Evaluate a card for the current deck/run state
     */
    private static float evaluateCard(AbstractCard card) {
        float score = 0;

        // === BASE POWER ===
        score += getIntrinsicPower(card);

        // === DECK SYNERGY ===
        score += getDeckSynergy(card);

        // === RUN STATE ===
        score += getRunStateModifier(card);

        // === RARITY BONUS ===
        if (card.rarity == AbstractCard.CardRarity.RARE) {
            score += 2;
        } else if (card.rarity == AbstractCard.CardRarity.UNCOMMON) {
            score += 0.5f;
        }

        // === UPGRADED BONUS ===
        if (card.upgraded) {
            score += 1;
        }

        return score;
    }

    /**
     * Intrinsic card power (independent of deck)
     */
    private static float getIntrinsicPower(AbstractCard card) {
        String id = card.cardID;
        float power = 0;

        // === WATCHER TOP TIER ===
        if (id.equals("Ragnarok") || id.equals("Scrawl") || id.equals("Vault") ||
            id.equals("Omniscience") || id.equals("Blasphemy") || id.equals("DevaForm") ||
            id.equals("LessonLearned") || id.equals("Wish")) {
            power = 9;
        }
        // === WATCHER HIGH TIER ===
        else if (id.equals("TalkToTheHand") || id.equals("MentalFortress") ||
                 id.equals("Rushdown") || id.equals("Tantrum") || id.equals("Wallop") ||
                 id.equals("WheelKick") || id.equals("Conclude") || id.equals("FearNoEvil") ||
                 id.equals("SandsOfTime") || id.equals("Establishment")) {
            power = 7;
        }
        // === WATCHER MID TIER ===
        else if (id.equals("Eruption") || id.equals("Vigilance") || id.equals("CutThroughFate") ||
                 id.equals("EmptyFist") || id.equals("InnerPeace") || id.equals("ThirdEye") ||
                 id.equals("FlurryOfBlows") || id.equals("Swivel") || id.equals("Worship")) {
            power = 5;
        }
        // === WATCHER LOW TIER ===
        else if (id.equals("Crescendo") || id.equals("Tranquility") || id.equals("Halt") ||
                 id.equals("Prostrate") || id.equals("Protect") || id.equals("JustLucky")) {
            power = 3;
        }
        // === DEFAULT BY TYPE ===
        else {
            if (card.type == AbstractCard.CardType.POWER) {
                power = 5; // Powers generally valuable
            } else if (card.type == AbstractCard.CardType.ATTACK) {
                power = 3 + (card.baseDamage / 10f);
            } else if (card.type == AbstractCard.CardType.SKILL) {
                power = 3 + (card.baseBlock / 10f);
            } else {
                power = 2;
            }
        }

        // Energy efficiency adjustment
        int cost = card.cost;
        if (cost == 0) power += 1;
        else if (cost >= 3) power -= 1;

        return power;
    }

    /**
     * How well does this card synergize with current deck?
     */
    private static float getDeckSynergy(AbstractCard card) {
        if (AbstractDungeon.player == null) return 0;

        float synergy = 0;
        List<AbstractCard> deck = AbstractDungeon.player.masterDeck.group;
        String id = card.cardID;

        // Count deck composition
        int attackCount = 0, skillCount = 0, powerCount = 0;
        int stanceCards = 0, retainCards = 0, scryCards = 0;
        int totalCards = deck.size();

        for (AbstractCard c : deck) {
            if (c.type == AbstractCard.CardType.ATTACK) attackCount++;
            else if (c.type == AbstractCard.CardType.SKILL) skillCount++;
            else if (c.type == AbstractCard.CardType.POWER) powerCount++;

            String cid = c.cardID;
            if (cid.equals("Eruption") || cid.equals("Tantrum") || cid.equals("Indignation") ||
                cid.equals("Vigilance") || cid.equals("InnerPeace") || cid.equals("FearNoEvil")) {
                stanceCards++;
            }
            if (c.retain || cid.contains("Retain") || cid.equals("Establishment")) {
                retainCards++;
            }
            if (cid.equals("ThirdEye") || cid.equals("CutThroughFate") || cid.equals("Foresight")) {
                scryCards++;
            }
        }

        // === DECK SIZE CONSIDERATIONS ===
        // Small deck: avoid adding cards unless very good
        if (totalCards <= 15) {
            synergy -= 1; // Penalize deck bloat
        }
        // Large deck: card draw more valuable
        if (totalCards >= 25 && cardDrawsCards(card)) {
            synergy += 2;
        }

        // === ATTACK/SKILL BALANCE ===
        float attackRatio = (float) attackCount / Math.max(1, totalCards);
        if (card.type == AbstractCard.CardType.ATTACK && attackRatio > 0.6) {
            synergy -= 1; // Too many attacks already
        }
        if (card.type == AbstractCard.CardType.SKILL && attackRatio < 0.3) {
            synergy -= 1; // Too few attacks
        }

        // === STANCE SYNERGY ===
        if (isStanceCard(card)) {
            if (stanceCards < 3) {
                synergy += 2; // Need more stance cards
            } else if (stanceCards > 6) {
                synergy -= 1; // Too many stance cards
            }
        }

        // === SPECIFIC SYNERGIES ===
        // Rushdown + stance changers
        if (id.equals("Rushdown") && stanceCards >= 2) {
            synergy += 3;
        }
        if (isStanceCard(card) && hasPower(deck, "Rushdown")) {
            synergy += 2;
        }

        // Establishment + retain cards
        if (id.equals("Establishment") && retainCards >= 2) {
            synergy += 2;
        }

        // Mental Fortress + stance changers
        if (id.equals("MentalFortress") && stanceCards >= 3) {
            synergy += 2;
        }

        return synergy;
    }

    /**
     * Modifiers based on current run state (floor, HP, relics)
     */
    private static float getRunStateModifier(AbstractCard card) {
        if (AbstractDungeon.player == null) return 0;

        float modifier = 0;
        int floor = AbstractDungeon.floorNum;
        float hpPercent = (float) AbstractDungeon.player.currentHealth / AbstractDungeon.player.maxHealth;

        // === FLOOR/ACT CONSIDERATIONS ===
        // Act 1: prioritize damage for hallway fights
        if (floor <= 17) {
            if (card.type == AbstractCard.CardType.ATTACK && card.baseDamage >= 10) {
                modifier += 1;
            }
        }
        // Act 2+: need scaling/powers
        if (floor > 17) {
            if (card.type == AbstractCard.CardType.POWER) {
                modifier += 1;
            }
        }

        // === HP CONSIDERATIONS ===
        // Low HP: prioritize block/healing
        if (hpPercent < 0.5f && card.baseBlock >= 10) {
            modifier += 1;
        }

        // === RELIC SYNERGIES ===
        // Pen Nib: attacks more valuable
        if (hasRelic("PenNib") && card.type == AbstractCard.CardType.ATTACK) {
            modifier += 0.5f;
        }
        // Kunai/Shuriken: attacks more valuable
        if ((hasRelic("Kunai") || hasRelic("Shuriken")) &&
            card.type == AbstractCard.CardType.ATTACK) {
            modifier += 0.5f;
        }
        // Violet Lotus: stance changes more valuable
        if (hasRelic("Violet Lotus") && isStanceCard(card)) {
            modifier += 1;
        }

        return modifier;
    }

    // === HELPER METHODS ===

    private static boolean isRewardScreenOpen() {
        return AbstractDungeon.cardRewardScreen != null &&
               AbstractDungeon.cardRewardScreen.rewardGroup != null &&
               !AbstractDungeon.cardRewardScreen.rewardGroup.isEmpty() &&
               AbstractDungeon.screen == AbstractDungeon.CurrentScreen.CARD_REWARD;
    }

    private static boolean isStanceCard(AbstractCard card) {
        String id = card.cardID;
        return id.equals("Eruption") || id.equals("Tantrum") || id.equals("Indignation") ||
               id.equals("Ragnarok") || id.equals("Vigilance") || id.equals("InnerPeace") ||
               id.equals("FearNoEvil") || id.equals("EmptyMind") || id.equals("Meditate") ||
               id.equals("Tranquility") || id.equals("Crescendo");
    }

    private static boolean cardDrawsCards(AbstractCard card) {
        String id = card.cardID;
        return id.equals("Scrawl") || id.equals("EmptyMind") || id.equals("WheelKick") ||
               id.equals("ThirdEye") || id.equals("CutThroughFate") || id.equals("InnerPeace");
    }

    private static boolean hasPower(List<AbstractCard> deck, String cardId) {
        for (AbstractCard c : deck) {
            if (c.cardID.equals(cardId)) return true;
        }
        return false;
    }

    private static boolean hasRelic(String relicId) {
        if (AbstractDungeon.player == null) return false;
        return AbstractDungeon.player.hasRelic(relicId);
    }

    // === COLORS ===
    private static final Color COLOR_BEST = new Color(1.0f, 0.84f, 0.0f, 1);   // Gold
    private static final Color COLOR_GOOD = new Color(0.2f, 0.7f, 0.2f, 1);    // Green
    private static final Color COLOR_OK = new Color(0.6f, 0.7f, 0.3f, 1);      // Yellow-green
    private static final Color COLOR_MEH = new Color(0.6f, 0.6f, 0.6f, 1);     // Gray
    private static final Color COLOR_SKIP = new Color(0.7f, 0.3f, 0.2f, 1);    // Red
}
