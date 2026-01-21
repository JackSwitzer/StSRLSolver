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

import java.util.*;

/**
 * Combat-aware EV overlay for cards in hand.
 *
 * Key insight: EV depends on combat context, not individual card stats.
 *
 * Decision tree:
 * 1. CAN WE LETHAL? → Find efficient kill line, block is worthless
 * 2. NO LETHAL → Optimize damage/block balance for HP preservation
 *
 * Shows:
 * - "KILL" on cards that achieve lethal
 * - "+" for cards contributing to optimal play
 * - "-" for suboptimal/wasteful plays
 */
public class CardEVOverlay {

    // Combat state analysis (cached per turn)
    private static boolean canLethal = false;
    private static int totalPotentialDamage = 0;
    private static int totalEnemyHP = 0;
    private static int incomingDamage = 0;
    private static int currentBlock = 0;
    private static int netDamage = 0;
    private static Set<String> lethalCards = new HashSet<>();  // UUIDs of cards needed for kill
    private static Map<String, String> cardLabels = new HashMap<>();  // UUID -> display label
    private static Map<String, Color> cardColors = new HashMap<>();   // UUID -> badge color

    private static int lastTurnCached = -1;
    private static int lastHandSize = -1;
    private static int lastEnemyHP = -1;

    /**
     * Render EV badges on all cards in hand
     */
    public static void renderCardEVs(SpriteBatch sb) {
        if (AbstractDungeon.player == null || AbstractDungeon.player.hand == null) {
            return;
        }

        // Recalculate if state changed
        int currentTurn = AbstractDungeon.actionManager.turn;
        int handSize = AbstractDungeon.player.hand.size();
        int enemyHP = getTotalEnemyHP();

        if (currentTurn != lastTurnCached || handSize != lastHandSize || enemyHP != lastEnemyHP) {
            analyzeCombatState();
            lastTurnCached = currentTurn;
            lastHandSize = handSize;
            lastEnemyHP = enemyHP;
        }

        // Render badge on each card
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            renderCardBadge(sb, card);
        }
    }

    /**
     * Analyze full combat state and determine card values
     */
    private static void analyzeCombatState() {
        lethalCards.clear();
        cardLabels.clear();
        cardColors.clear();

        // Get combat state
        totalEnemyHP = getTotalEnemyHP();
        incomingDamage = DamageCalculator.calculateTotalIncomingDamage();
        currentBlock = AbstractDungeon.player.currentBlock;
        netDamage = Math.max(0, incomingDamage - currentBlock);
        int baseEnergy = EnergyPanel.totalCount;

        // First pass: find energy generators to calculate effective energy
        int bonusEnergy = 0;
        List<AbstractCard> energyGens = new ArrayList<>();
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            if (!card.canUse(AbstractDungeon.player, getFirstAliveMonster())) continue;
            int cost = card.costForTurn == -1 ? baseEnergy : card.costForTurn;

            // Energy generators that are free or cheap
            int energyGen = getEnergyGenerated(card);
            if (energyGen > 0 && cost <= 1) {
                bonusEnergy += energyGen - cost; // Net energy gain
                energyGens.add(card);
            }
            // Calm exit energy
            if (cardExitsCalmForEnergy(card) && cost <= baseEnergy) {
                bonusEnergy += 2 - cost; // Calm gives +2 on exit
            }
        }

        int effectiveEnergy = baseEnergy + bonusEnergy;

        // Calculate total potential damage from hand with effective energy
        totalPotentialDamage = 0;
        List<CardDamageInfo> damageCards = new ArrayList<>();
        List<CardBlockInfo> blockCards = new ArrayList<>();

        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            if (!card.canUse(AbstractDungeon.player, getFirstAliveMonster())) {
                cardLabels.put(card.uuid.toString(), "X");
                cardColors.put(card.uuid.toString(), COLOR_UNPLAYABLE);
                continue;
            }

            int cost = card.costForTurn == -1 ? effectiveEnergy : card.costForTurn;

            // For initial unplayable check, use base energy (before generators played)
            int checkCost = card.costForTurn == -1 ? baseEnergy : card.costForTurn;
            if (checkCost > effectiveEnergy && card.costForTurn != -1 && !energyGens.contains(card)) {
                cardLabels.put(card.uuid.toString(), "$");
                cardColors.put(card.uuid.toString(), COLOR_UNPLAYABLE);
                continue;
            }

            // Track damage cards
            if (card.baseDamage > 0) {
                int dmg = calculateEffectiveDamage(card);
                damageCards.add(new CardDamageInfo(card, dmg, cost));
                totalPotentialDamage += dmg;
            }

            // Track block cards
            if (card.baseBlock > 0) {
                int blk = card.block;
                blockCards.add(new CardBlockInfo(card, blk, cost));
            }

            // Mark energy generators specially
            if (isEnergyGenerator(card)) {
                int netGain = getEnergyGenerated(card) - card.costForTurn;
                if (cardExitsCalmForEnergy(card)) netGain = 2 - card.costForTurn;
                if (netGain > 0 && !cardLabels.containsKey(card.uuid.toString())) {
                    cardLabels.put(card.uuid.toString(), "+" + netGain + "E");
                    cardColors.put(card.uuid.toString(), COLOR_GOOD);
                }
            }
        }

        // === LETHAL CHECK (with effective energy from generators) ===
        canLethal = checkLethalWithEnergy(damageCards, totalEnemyHP, effectiveEnergy);

        if (canLethal) {
            // Find efficient lethal line
            findEfficientLethal(damageCards, totalEnemyHP, effectiveEnergy);

            // Mark non-lethal cards
            for (AbstractCard card : AbstractDungeon.player.hand.group) {
                String uuid = card.uuid.toString();
                if (!cardLabels.containsKey(uuid)) {
                    if (card.baseBlock > 0 && card.baseDamage <= 0) {
                        // Pure block card when we can kill = bad
                        cardLabels.put(uuid, "0");
                        cardColors.put(uuid, COLOR_WASTE);
                    } else if (card.baseDamage > 0 && !lethalCards.contains(uuid)) {
                        // Damage card not needed for kill = overkill
                        cardLabels.put(uuid, "+");
                        cardColors.put(uuid, COLOR_NEUTRAL);
                    } else if (card.baseDamage <= 0 && card.baseBlock <= 0) {
                        // Utility card during lethal turn
                        cardLabels.put(uuid, "?");
                        cardColors.put(uuid, COLOR_NEUTRAL);
                    }
                }
            }
        } else {
            // === NO LETHAL - SURVIVAL MODE ===
            evaluateSurvivalValue(damageCards, blockCards, effectiveEnergy);
        }
    }

    /**
     * Check if lethal is possible with available energy
     */
    private static boolean checkLethalWithEnergy(List<CardDamageInfo> damageCards, int targetHP, int availableEnergy) {
        if (targetHP <= 0) return true;

        // Sort by efficiency to greedily find if we can kill
        List<CardDamageInfo> sorted = new ArrayList<>(damageCards);
        sorted.sort((a, b) -> {
            float effA = a.cost == 0 ? a.damage * 10 : (float) a.damage / a.cost;
            float effB = b.cost == 0 ? b.damage * 10 : (float) b.damage / b.cost;
            return Float.compare(effB, effA);
        });

        int remainingHP = targetHP;
        int remainingEnergy = availableEnergy;

        for (CardDamageInfo cdi : sorted) {
            if (remainingHP <= 0) return true;
            if (cdi.cost <= remainingEnergy) {
                remainingHP -= cdi.damage;
                remainingEnergy -= cdi.cost;
            }
        }

        return remainingHP <= 0;
    }

    /**
     * Find minimum cards needed to achieve lethal
     */
    private static void findEfficientLethal(List<CardDamageInfo> damageCards, int targetHP, int energy) {
        // Sort by damage efficiency (damage per energy, prefer high damage)
        damageCards.sort((a, b) -> {
            float effA = a.cost == 0 ? a.damage * 10 : (float) a.damage / a.cost;
            float effB = b.cost == 0 ? b.damage * 10 : (float) b.damage / b.cost;
            return Float.compare(effB, effA);
        });

        int remainingHP = targetHP;
        int remainingEnergy = energy;
        List<CardDamageInfo> killLine = new ArrayList<>();

        // Greedy selection of most efficient damage
        for (CardDamageInfo cdi : damageCards) {
            if (remainingHP <= 0) break;
            if (cdi.cost <= remainingEnergy) {
                killLine.add(cdi);
                remainingHP -= cdi.damage;
                remainingEnergy -= cdi.cost;
            }
        }

        // Mark kill cards
        boolean firstKill = true;
        for (CardDamageInfo cdi : killLine) {
            String uuid = cdi.card.uuid.toString();
            lethalCards.add(uuid);
            if (firstKill && remainingHP <= 0) {
                // This card delivers lethal
                cardLabels.put(uuid, "KILL");
                cardColors.put(uuid, COLOR_KILL);
                firstKill = false;
            } else {
                cardLabels.put(uuid, "DMG");
                cardColors.put(uuid, COLOR_GOOD);
            }
        }
    }

    /**
     * Evaluate cards for survival when lethal not possible
     */
    private static void evaluateSurvivalValue(List<CardDamageInfo> damageCards,
                                               List<CardBlockInfo> blockCards, int energy) {
        int blockNeeded = netDamage;
        int blockAvailable = 0;
        for (CardBlockInfo cbi : blockCards) {
            blockAvailable += cbi.block;
        }

        // Calculate HP loss this turn
        int projectedHPLoss = Math.max(0, netDamage - blockAvailable);

        // Calculate turns to kill (rough estimate)
        float avgDamagePerTurn = totalPotentialDamage * 0.7f; // Account for energy/draw variance
        float turnsToKill = avgDamagePerTurn > 0 ? totalEnemyHP / avgDamagePerTurn : 99;

        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            String uuid = card.uuid.toString();
            if (cardLabels.containsKey(uuid)) continue; // Already labeled

            int cost = card.costForTurn == -1 ? energy : card.costForTurn;

            // === BLOCK CARDS ===
            if (card.baseBlock > 0) {
                int block = card.block;
                int usefulBlock = Math.min(block, blockNeeded);
                int wastedBlock = block - usefulBlock;

                if (usefulBlock > 0) {
                    // Block that prevents damage
                    float efficiency = (float) usefulBlock / Math.max(1, cost);
                    if (efficiency >= 5) {
                        cardLabels.put(uuid, "+" + usefulBlock);
                        cardColors.put(uuid, COLOR_GOOD);
                    } else {
                        cardLabels.put(uuid, "+" + usefulBlock);
                        cardColors.put(uuid, COLOR_OK);
                    }
                    blockNeeded -= usefulBlock;
                } else {
                    // Wasted block
                    cardLabels.put(uuid, "0");
                    cardColors.put(uuid, COLOR_WASTE);
                }
            }
            // === DAMAGE CARDS ===
            else if (card.baseDamage > 0) {
                int dmg = calculateEffectiveDamage(card);
                float efficiency = cost == 0 ? dmg * 2 : (float) dmg / cost;

                // Value based on contribution to kill
                if (efficiency >= 6) {
                    cardLabels.put(uuid, "+" + dmg);
                    cardColors.put(uuid, COLOR_GOOD);
                } else if (efficiency >= 3) {
                    cardLabels.put(uuid, "+" + dmg);
                    cardColors.put(uuid, COLOR_OK);
                } else {
                    cardLabels.put(uuid, dmg + "");
                    cardColors.put(uuid, COLOR_NEUTRAL);
                }
            }
            // === UTILITY CARDS ===
            else {
                // Powers, skills without block/damage
                float value = calculateUtilityValue(card);
                if (value >= 5) {
                    cardLabels.put(uuid, "+");
                    cardColors.put(uuid, COLOR_GOOD);
                } else if (value >= 0) {
                    cardLabels.put(uuid, "~");
                    cardColors.put(uuid, COLOR_NEUTRAL);
                } else {
                    cardLabels.put(uuid, "-");
                    cardColors.put(uuid, COLOR_BAD);
                }
            }
        }
    }

    /**
     * Calculate effective damage considering multi-target, vulnerable, etc.
     */
    private static int calculateEffectiveDamage(AbstractCard card) {
        int damage = card.damage;

        // Multi-target multiplier
        if (card.target == AbstractCard.CardTarget.ALL_ENEMY) {
            int aliveCount = 0;
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) aliveCount++;
            }
            damage *= aliveCount;
        }

        // Stance multiplier (already in card.damage for display)
        // But we need to check Wrath for double damage potential
        String stance = AbstractDungeon.player.stance != null ?
                       AbstractDungeon.player.stance.ID : "Neutral";
        if (stance.equals("Calm") && cardEntersWrath(card)) {
            damage *= 2; // Will deal double after entering Wrath
        }

        return damage;
    }

    /**
     * Calculate utility value for non-damage/block cards
     */
    private static float calculateUtilityValue(AbstractCard card) {
        float value = 0;
        String id = card.cardID;

        // Draw is valuable
        if (cardDrawsCards(card)) {
            value += card.magicNumber * 2;
        }

        // Scry is situationally good
        if (id.equals("ThirdEye") || id.equals("CutThroughFate")) {
            value += 1;
        }

        // Stance changes
        String stance = AbstractDungeon.player.stance != null ?
                       AbstractDungeon.player.stance.ID : "Neutral";
        if (stance.equals("Wrath") && cardExitsWrath(card) && incomingDamage > 0) {
            value += 10; // Exit Wrath before getting hit
        }
        if (stance.equals("Calm") && cardEntersWrath(card) && canLethal) {
            value += 5; // Enter Wrath to burst
        }

        // Powers are generally good if we survive
        if (card.type == AbstractCard.CardType.POWER) {
            if (netDamage > AbstractDungeon.player.currentHealth * 0.5f) {
                value -= 5; // Don't play powers when about to die
            } else {
                value += 3;
            }
        }

        // Exhaust synergy check
        if (card.exhaust) {
            value -= 1;
        }

        return value;
    }

    /**
     * Render badge on a single card
     */
    private static void renderCardBadge(SpriteBatch sb, AbstractCard card) {
        String uuid = card.uuid.toString();
        String label = cardLabels.getOrDefault(uuid, "?");
        Color bgColor = cardColors.getOrDefault(uuid, COLOR_NEUTRAL);

        // Skip unplayable markers if card is faded
        if (label.equals("X") || label.equals("$")) {
            return; // Don't clutter with X marks
        }

        // Position: top-right of card, smaller and less intrusive
        float x = card.current_x + (card.hb.width / 2) * 0.6f;
        float y = card.current_y + (card.hb.height / 2) * 0.7f;

        // Badge size based on label length
        float badgeWidth = Math.max(35, label.length() * 12) * Settings.scale;
        float badgeHeight = 28 * Settings.scale;

        // Semi-transparent background
        bgColor = bgColor.cpy();
        bgColor.a = 0.9f;
        sb.setColor(bgColor);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG,
                x - badgeWidth/2, y - badgeHeight/2,
                badgeWidth, badgeHeight);

        // Border
        sb.setColor(0, 0, 0, 0.5f);
        float border = 2 * Settings.scale;
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x - badgeWidth/2 - border, y - badgeHeight/2 - border,
                badgeWidth + border*2, border); // Top
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x - badgeWidth/2 - border, y + badgeHeight/2,
                badgeWidth + border*2, border); // Bottom

        // Text
        FontHelper.renderFontCentered(sb, FontHelper.tipBodyFont,
                label, x, y, Color.WHITE);
    }

    // === HELPER METHODS ===

    private static int getTotalEnemyHP() {
        int total = 0;
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    total += m.currentHealth;
                }
            }
        }
        return total;
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
               id.equals("CutThroughFate") || id.equals("InnerPeace");
    }

    /**
     * Get energy generated by playing this card (Miracle, etc.)
     */
    private static int getEnergyGenerated(AbstractCard card) {
        String id = card.cardID;
        // Watcher energy cards
        if (id.equals("Miracle")) return 1;
        if (id.equals("Collect") || id.equals("Collect+")) return card.magicNumber; // 2 or 3
        if (id.equals("Worship")) return card.magicNumber; // Mantra, but can trigger Divinity
        if (id.equals("Prostrate")) return 0; // Mantra, not direct energy

        // Universal energy cards
        if (id.equals("Adrenaline")) return 1;
        if (id.equals("Enlightenment")) return 0; // Cost reduction, not energy
        if (id.equals("Seeing Red")) return 2;
        if (id.equals("Offering")) return 2;
        if (id.equals("Bloodletting")) return 2;

        // Calm exit gives +2 (handled separately)
        return 0;
    }

    /**
     * Check if card generates energy (for smarter sequencing)
     */
    private static boolean isEnergyGenerator(AbstractCard card) {
        return getEnergyGenerated(card) > 0 || cardExitsCalmForEnergy(card);
    }

    private static boolean cardExitsCalmForEnergy(AbstractCard card) {
        String stance = AbstractDungeon.player.stance != null ?
                       AbstractDungeon.player.stance.ID : "Neutral";
        if (!stance.equals("Calm")) return false;

        // Cards that exit Calm (gives +2 energy)
        String id = card.cardID;
        return id.equals("Eruption") || id.equals("Tantrum") ||
               id.equals("Indignation") || id.equals("Ragnarok") ||
               id.equals("Vigilance") || id.equals("InnerPeace") ||
               id.equals("FearNoEvil") || id.equals("EmptyMind") ||
               id.equals("Meditate") || id.equals("Tranquility");
    }

    /**
     * Clear cache on turn start
     */
    public static void onTurnStart() {
        cardLabels.clear();
        cardColors.clear();
        lethalCards.clear();
        lastTurnCached = -1;
    }

    /**
     * Legacy method for CombatReview compatibility
     */
    public static float calculateCardEV(AbstractCard card) {
        // Return simple heuristic for backward compatibility
        float ev = 0;
        if (card.baseDamage > 0) ev += card.damage * 0.5f;
        if (card.baseBlock > 0) ev += card.block * 0.5f;
        return ev;
    }

    // === DATA CLASSES ===

    private static class CardDamageInfo {
        AbstractCard card;
        int damage;
        int cost;
        CardDamageInfo(AbstractCard c, int d, int cost) {
            this.card = c;
            this.damage = d;
            this.cost = cost;
        }
    }

    private static class CardBlockInfo {
        AbstractCard card;
        int block;
        int cost;
        CardBlockInfo(AbstractCard c, int b, int cost) {
            this.card = c;
            this.block = b;
            this.cost = cost;
        }
    }

    // === COLORS ===
    private static final Color COLOR_KILL = new Color(1.0f, 0.84f, 0.0f, 1);      // Gold - lethal blow
    private static final Color COLOR_GOOD = new Color(0.2f, 0.7f, 0.2f, 1);       // Green - efficient
    private static final Color COLOR_OK = new Color(0.5f, 0.7f, 0.3f, 1);         // Yellow-green
    private static final Color COLOR_NEUTRAL = new Color(0.5f, 0.5f, 0.5f, 1);    // Gray
    private static final Color COLOR_WASTE = new Color(0.7f, 0.3f, 0.2f, 1);      // Red-orange - wasteful
    private static final Color COLOR_BAD = new Color(0.7f, 0.2f, 0.2f, 1);        // Red
    private static final Color COLOR_UNPLAYABLE = new Color(0.3f, 0.3f, 0.3f, 1); // Dark gray
}
