package evtracker;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.helpers.ImageMaster;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.ui.panels.EnergyPanel;

import java.util.HashMap;
import java.util.Map;

/**
 * Renders EV values on cards in hand during combat.
 * Shows expected value of playing each card based on:
 * - Immediate damage/block value
 * - Resource efficiency (energy cost)
 * - Stance interactions
 * - Enemy state consideration
 */
public class CardEVOverlay {

    private static Map<String, Float> cardEVCache = new HashMap<>();
    private static int lastTurnCached = -1;
    private static int lastHandSize = -1;

    /**
     * Render EV badges on all cards in hand
     */
    public static void renderCardEVs(SpriteBatch sb) {
        if (AbstractDungeon.player == null || AbstractDungeon.player.hand == null) {
            return;
        }

        // Recalculate if hand changed
        int currentTurn = AbstractDungeon.actionManager.turn;
        int handSize = AbstractDungeon.player.hand.size();
        if (currentTurn != lastTurnCached || handSize != lastHandSize) {
            recalculateAllEVs();
            lastTurnCached = currentTurn;
            lastHandSize = handSize;
        }

        // Render EV on each card
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            renderCardEV(sb, card);
        }
    }

    /**
     * Render EV badge on a single card
     */
    private static void renderCardEV(SpriteBatch sb, AbstractCard card) {
        Float ev = cardEVCache.get(card.uuid.toString());
        if (ev == null) {
            ev = calculateCardEV(card);
            cardEVCache.put(card.uuid.toString(), ev);
        }

        // Position: top-right of card
        float x = card.current_x + (card.hb.width / 2) * 0.7f;
        float y = card.current_y + (card.hb.height / 2) * 0.8f;

        // Badge background
        float badgeSize = 40 * Settings.scale;
        Color bgColor = getEVColor(ev);
        bgColor.a = 0.85f;
        sb.setColor(bgColor);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG,
                x - badgeSize/2, y - badgeSize/2,
                badgeSize, badgeSize);

        // EV text
        String evText = formatEV(ev);
        Color textColor = ev >= 0 ? Color.WHITE : Color.WHITE;
        FontHelper.renderFontCentered(sb, FontHelper.cardEnergyFont_L,
                evText, x, y, textColor);
    }

    /**
     * Calculate EV for playing a card
     */
    public static float calculateCardEV(AbstractCard card) {
        float ev = 0;

        // Can't play = very negative
        if (!card.canUse(AbstractDungeon.player, getFirstAliveMonster())) {
            return -999;
        }

        int energy = EnergyPanel.totalCount;
        if (card.costForTurn > energy && card.costForTurn != -1) {
            return -999; // Can't afford
        }

        // Base value from damage/block
        ev += calculateDamageValue(card);
        ev += calculateBlockValue(card);

        // Energy efficiency bonus
        ev += calculateEnergyEfficiency(card);

        // Stance synergy
        ev += calculateStanceSynergy(card);

        // Special effects value
        ev += calculateSpecialEffects(card);

        // Situational modifiers
        ev += calculateSituationalValue(card);

        return ev;
    }

    private static float calculateDamageValue(AbstractCard card) {
        if (card.baseDamage <= 0) return 0;

        int damage = card.damage; // Already includes strength, etc.
        float value = damage;

        // Multi-hit bonus (check card type/target for multi-attack cards)
        if (card.target == AbstractCard.CardTarget.ALL_ENEMY) {
            // AoE cards hit all enemies
            value *= 1.5f;
        }

        // Killing blow bonus - huge value if this kills
        AbstractMonster target = getFirstAliveMonster();
        if (target != null && damage >= target.currentHealth) {
            value += 20; // Big bonus for lethal
        }

        // Vulnerable bonus (they take more)
        if (target != null && target.hasPower("Vulnerable")) {
            value *= 1.2f;
        }

        return value * 0.5f; // Scale down raw damage
    }

    private static float calculateBlockValue(AbstractCard card) {
        if (card.baseBlock <= 0) return 0;

        int block = card.block; // Already includes dexterity
        int incomingDamage = DamageCalculator.calculateTotalIncomingDamage();
        int currentBlock = AbstractDungeon.player.currentBlock;

        // Value is how much damage this actually prevents
        int neededBlock = Math.max(0, incomingDamage - currentBlock);
        int usefulBlock = Math.min(block, neededBlock);
        int wastedBlock = block - usefulBlock;

        // Useful block = full value, wasted block = reduced value
        return usefulBlock * 1.0f + wastedBlock * 0.2f;
    }

    private static float calculateEnergyEfficiency(AbstractCard card) {
        int cost = card.costForTurn;
        if (cost <= 0) return 2; // Free cards are great

        // Penalize expensive cards slightly
        return -cost * 0.5f;
    }

    private static float calculateStanceSynergy(AbstractCard card) {
        String stance = AbstractDungeon.player.stance != null ?
                       AbstractDungeon.player.stance.ID : "Neutral";
        float bonus = 0;

        // Wrath synergies
        if (stance.equals("Wrath")) {
            if (card.type == AbstractCard.CardType.ATTACK) {
                bonus += 3; // Attacks deal double
            }
            // Check if card exits wrath (safety)
            if (cardExitsWrath(card)) {
                int incomingDamage = DamageCalculator.calculateTotalIncomingDamage();
                if (incomingDamage > 0) {
                    bonus += 10; // Huge bonus for safe exit
                }
            }
        }

        // Calm synergies
        if (stance.equals("Calm")) {
            if (cardEntersWrath(card)) {
                // Check if we can kill this turn
                bonus += 2;
            }
        }

        return bonus;
    }

    private static float calculateSpecialEffects(AbstractCard card) {
        float value = 0;

        // Draw cards
        if (card.magicNumber > 0 && cardDrawsCards(card)) {
            value += card.magicNumber * 2;
        }

        // Exhaust (usually negative unless exhaust synergy)
        if (card.exhaust) {
            value -= 1;
        }

        // Ethereal (will vanish)
        if (card.isEthereal) {
            value += 0.5f; // Slight bonus to play ethereal cards
        }

        return value;
    }

    private static float calculateSituationalValue(AbstractCard card) {
        float value = 0;

        // If low HP, prioritize block
        float hpPercent = (float) AbstractDungeon.player.currentHealth /
                         AbstractDungeon.player.maxHealth;
        if (hpPercent < 0.3f && card.baseBlock > 0) {
            value += 5;
        }

        // If enemy low HP, prioritize damage
        AbstractMonster target = getFirstAliveMonster();
        if (target != null) {
            float enemyHpPercent = (float) target.currentHealth / target.maxHealth;
            if (enemyHpPercent < 0.3f && card.baseDamage > 0) {
                value += 3;
            }
        }

        return value;
    }

    // Helper methods
    private static boolean cardExitsWrath(AbstractCard card) {
        String id = card.cardID;
        return id.equals("Vigilance") || id.equals("InnerPeace") ||
               id.equals("FearNoEvil") || id.equals("EmptyMind") ||
               id.equals("Meditate") || id.equals("Tranquility");
    }

    private static boolean cardEntersWrath(AbstractCard card) {
        String id = card.cardID;
        return id.equals("Eruption") || id.equals("Tantrum") ||
               id.equals("Indignation") || id.equals("Ragnarok");
    }

    private static boolean cardDrawsCards(AbstractCard card) {
        String id = card.cardID;
        return id.equals("Scrawl") || id.equals("EmptyMind") ||
               id.equals("WheelKick") || id.equals("ThirdEye") ||
               id.equals("CutThroughFate");
    }

    private static AbstractMonster getFirstAliveMonster() {
        if (AbstractDungeon.getMonsters() == null) return null;
        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            if (!m.isDead && !m.isDying && m.currentHealth > 0) {
                return m;
            }
        }
        return null;
    }

    private static Color getEVColor(float ev) {
        if (ev >= 10) return new Color(0.2f, 0.8f, 0.2f, 1); // Green
        if (ev >= 5) return new Color(0.6f, 0.8f, 0.2f, 1);  // Yellow-green
        if (ev >= 0) return new Color(0.8f, 0.8f, 0.2f, 1);  // Yellow
        if (ev >= -5) return new Color(0.8f, 0.5f, 0.2f, 1); // Orange
        return new Color(0.8f, 0.2f, 0.2f, 1); // Red
    }

    private static String formatEV(float ev) {
        if (ev <= -100) return "X";
        if (ev >= 0) return "+" + (int)ev;
        return String.valueOf((int)ev);
    }

    private static void recalculateAllEVs() {
        cardEVCache.clear();
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            cardEVCache.put(card.uuid.toString(), calculateCardEV(card));
        }
    }

    /**
     * Clear cache on turn start
     */
    public static void onTurnStart() {
        cardEVCache.clear();
        lastTurnCached = -1;
    }
}
