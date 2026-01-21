package evtracker;

import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.potions.AbstractPotion;

import java.util.*;

/**
 * Tracks combat decisions for post-combat review.
 * Compares actual plays vs calculated optimal plays.
 * Tracks relics, potions, scaling, and all EV-relevant metrics.
 */
public class CombatReview {

    // Decision record
    public static class Decision {
        public int turn;
        public String cardPlayed;
        public String target;
        public float evAtTime;
        public float bestEVAtTime;
        public String bestCardAtTime;
        public int hpBefore;
        public int hpAfter;
        public int energyBefore;
        public String stanceBefore;
        public long timestamp;

        public float getEVDelta() {
            return evAtTime - bestEVAtTime;
        }

        public boolean wasOptimal() {
            return Math.abs(evAtTime - bestEVAtTime) < 0.5f;
        }
    }

    // Relic usage tracking
    public static class RelicUsage {
        public String relicId;
        public String relicName;
        public int procs;           // Times it triggered
        public String type;         // "combat_proc" or "passive"
        public String effect;       // Description of effect

        public RelicUsage(String id, String name, String type) {
            this.relicId = id;
            this.relicName = name;
            this.type = type;
            this.procs = 0;
        }
    }

    // Potion usage tracking
    public static class PotionUsage {
        public String potionId;
        public String potionName;
        public int turn;
        public String target;       // Monster name or null
        public int hpAtUse;
        public String context;      // "offensive", "defensive", "utility"
    }

    // Scaling event tracking (permanent stat gains)
    public static class ScalingEvent {
        public String source;       // Card/relic name
        public String type;         // "damage", "hp", "gold", "card_upgrade"
        public int amount;
        public int turn;
    }

    // Combat summary
    public static class CombatSummary {
        public String encounter;
        public int floor;
        public int totalTurns;
        public int hpLost;
        public int optimalHpLost;    // Estimated
        public int decisionsCount;
        public int optimalDecisions;
        public int suboptimalDecisions;
        public float totalEVLost;
        public List<Decision> decisions;
        public List<String> keyMistakes;
        public String playerComment;

        // Combat damage stats
        public int totalDamageDealt;
        public int totalDamageTaken;
        public int blockGenerated;
        public int blockWasted;      // Block that exceeded incoming damage
        public boolean wasInfinite;

        // Relic tracking
        public Map<String, RelicUsage> relicUsage;
        public List<String> combatProcRelics;   // Pen Nib, Ink Bottle, Kunai, etc.
        public List<String> passiveRelics;      // White Beast Statue, Toy Ornithopter

        // Potion tracking
        public List<PotionUsage> potionsUsed;

        // Scaling tracking
        public List<ScalingEvent> scalingEvents;
        public int permanentDamageGained;   // Ritual Dagger, Feed, etc.
        public int permanentHPGained;       // Feed
        public int goldGained;              // Bloody Idol, Golden Idol, etc.

        // Energy efficiency
        public int totalEnergyAvailable;
        public int totalEnergySpent;
        public float energyEfficiency;

        public float getOptimalityScore() {
            if (decisionsCount == 0) return 100;
            return (float) optimalDecisions / decisionsCount * 100;
        }
    }

    // Current combat tracking
    private static List<Decision> currentCombatDecisions = new ArrayList<>();
    private static String currentEncounter = "";
    private static int combatStartHP = 0;
    private static int combatFloor = 0;

    // Extended tracking for current combat
    private static int currentDamageDealt = 0;
    private static int currentBlockGenerated = 0;
    private static int currentEnergyAvailable = 0;
    private static int currentEnergySpent = 0;
    private static Map<String, RelicUsage> currentRelicUsage = new HashMap<>();
    private static List<PotionUsage> currentPotionsUsed = new ArrayList<>();
    private static List<ScalingEvent> currentScalingEvents = new ArrayList<>();
    private static Map<String, Integer> relicCounterStart = new HashMap<>();  // Track relic counters at combat start
    private static boolean infiniteDetected = false;

    // History of combat summaries
    private static List<CombatSummary> combatHistory = new ArrayList<>();

    // Combat proc relics (activate on specific triggers)
    private static final Set<String> COMBAT_PROC_RELICS = new HashSet<>(Arrays.asList(
        "PenNib",           // Double damage every 10 attacks
        "InkBottle",        // Draw card every 10 cards played
        "Kunai",            // +1 Dex every 3 attacks
        "Shuriken",         // +1 Str every 3 attacks
        "Ornamental Fan",   // Block every 3 attacks
        "Letter Opener",    // Damage every 3 skills
        "Nunchaku",         // Energy every 10 attacks
        "Happy Flower",     // Energy every 3 turns
        "Sundial",          // Draw every 3 shuffles
        "Incense Burner",   // Intangible every 6 turns
        "Inserter",         // Orb slot every 2 turns
        "The Abacus"        // Block on shuffle
    ));

    // Passive/end-of-combat relics
    private static final Set<String> PASSIVE_RELICS = new HashSet<>(Arrays.asList(
        "White Beast Statue",  // Potion on elite kill
        "Toy Ornithopter",     // Heal on potion use
        "Maw Bank",            // Gold on no-spend
        "Golden Idol",         // +25% gold
        "Bloody Idol",         // Heal when gold gained
        "Gremlin Horn",        // Energy + draw on kill
        "StrikeDummy",         // Strike damage bonus
        "Meat on the Bone",    // Heal at low HP combat end
        "Burning Blood",       // Heal on combat end
        "Black Blood",         // More heal on combat end
        "Magic Flower",        // Heal bonus
        "Regal Pillow",        // Heal bonus at rest
        "Meal Ticket"          // Heal on shop visit
    ));

    /**
     * Called at battle start
     */
    public static void onBattleStart() {
        currentCombatDecisions.clear();
        combatStartHP = AbstractDungeon.player.currentHealth;
        combatFloor = AbstractDungeon.floorNum;

        // Reset extended tracking
        currentDamageDealt = 0;
        currentBlockGenerated = 0;
        currentEnergyAvailable = 0;
        currentEnergySpent = 0;
        currentRelicUsage.clear();
        currentPotionsUsed.clear();
        currentScalingEvents.clear();
        relicCounterStart.clear();
        infiniteDetected = false;

        // Get encounter name
        if (AbstractDungeon.getMonsters() != null &&
            !AbstractDungeon.getMonsters().monsters.isEmpty()) {
            StringBuilder sb = new StringBuilder();
            for (com.megacrit.cardcrawl.monsters.AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (sb.length() > 0) sb.append(" + ");
                sb.append(m.name);
            }
            currentEncounter = sb.toString();
        }

        // Snapshot relic counters at combat start
        for (AbstractRelic r : AbstractDungeon.player.relics) {
            if (COMBAT_PROC_RELICS.contains(r.relicId) || PASSIVE_RELICS.contains(r.relicId)) {
                relicCounterStart.put(r.relicId, r.counter);
                String type = COMBAT_PROC_RELICS.contains(r.relicId) ? "combat_proc" : "passive";
                currentRelicUsage.put(r.relicId, new RelicUsage(r.relicId, r.name, type));
            }
        }
    }

    /**
     * Called when a card is played
     */
    public static void onCardPlayed(AbstractCard card, String target) {
        Decision d = new Decision();
        d.turn = AbstractDungeon.actionManager.turn;
        d.cardPlayed = card.name;
        d.target = target;
        d.hpBefore = AbstractDungeon.player.currentHealth;
        d.energyBefore = com.megacrit.cardcrawl.ui.panels.EnergyPanel.totalCount;
        d.stanceBefore = AbstractDungeon.player.stance != null ?
                        AbstractDungeon.player.stance.ID : "Neutral";
        d.timestamp = System.currentTimeMillis();

        // Track energy spent
        if (card.costForTurn > 0) {
            currentEnergySpent += card.costForTurn;
        }

        // Track damage dealt (approximate - actual damage handled by DamageCalculator)
        if (card.baseDamage > 0) {
            currentDamageDealt += card.damage;  // Use calculated damage
        }

        // Track block generated
        if (card.baseBlock > 0) {
            currentBlockGenerated += card.block;  // Use calculated block
        }

        // Calculate EV of played card
        d.evAtTime = CardEVOverlay.calculateCardEV(card);

        // Find best card at this moment
        float bestEV = d.evAtTime;
        String bestCard = card.name;
        for (AbstractCard c : AbstractDungeon.player.hand.group) {
            float ev = CardEVOverlay.calculateCardEV(c);
            if (ev > bestEV) {
                bestEV = ev;
                bestCard = c.name;
            }
        }
        d.bestEVAtTime = bestEV;
        d.bestCardAtTime = bestCard;

        currentCombatDecisions.add(d);
    }

    /**
     * Called after card resolves to update HP
     */
    public static void onCardResolved() {
        if (!currentCombatDecisions.isEmpty()) {
            Decision d = currentCombatDecisions.get(currentCombatDecisions.size() - 1);
            d.hpAfter = AbstractDungeon.player.currentHealth;
        }
    }

    /**
     * Called at battle end
     */
    public static CombatSummary onBattleEnd() {
        CombatSummary summary = new CombatSummary();
        summary.encounter = currentEncounter;
        summary.floor = combatFloor;
        summary.totalTurns = AbstractDungeon.actionManager.turn;
        summary.hpLost = combatStartHP - AbstractDungeon.player.currentHealth;
        summary.decisions = new ArrayList<>(currentCombatDecisions);
        summary.decisionsCount = currentCombatDecisions.size();

        // Extended damage stats
        summary.totalDamageDealt = currentDamageDealt;
        summary.totalDamageTaken = Math.max(0, combatStartHP - AbstractDungeon.player.currentHealth);
        summary.blockGenerated = currentBlockGenerated;
        summary.wasInfinite = infiniteDetected || InfiniteDetector.isInfiniteDetected();

        // Energy tracking
        summary.totalEnergySpent = currentEnergySpent;
        summary.totalEnergyAvailable = currentEnergyAvailable;
        summary.energyEfficiency = currentEnergyAvailable > 0 ?
            (float) currentEnergySpent / currentEnergyAvailable : 1.0f;

        // Analyze relic usage by comparing counter changes
        summary.relicUsage = new HashMap<>(currentRelicUsage);
        summary.combatProcRelics = new ArrayList<>();
        summary.passiveRelics = new ArrayList<>();

        for (AbstractRelic r : AbstractDungeon.player.relics) {
            if (currentRelicUsage.containsKey(r.relicId)) {
                RelicUsage usage = currentRelicUsage.get(r.relicId);
                Integer startCounter = relicCounterStart.get(r.relicId);
                if (startCounter != null) {
                    // Estimate procs from counter change (heuristic)
                    int counterChange = Math.abs(r.counter - startCounter);
                    usage.procs = counterChange;
                }

                if (COMBAT_PROC_RELICS.contains(r.relicId)) {
                    summary.combatProcRelics.add(r.name);
                } else {
                    summary.passiveRelics.add(r.name);
                }
            }
        }

        // Potion tracking
        summary.potionsUsed = new ArrayList<>(currentPotionsUsed);

        // Scaling tracking
        summary.scalingEvents = new ArrayList<>(currentScalingEvents);
        summary.permanentDamageGained = 0;
        summary.permanentHPGained = 0;
        summary.goldGained = 0;
        for (ScalingEvent se : currentScalingEvents) {
            switch (se.type) {
                case "damage": summary.permanentDamageGained += se.amount; break;
                case "hp": summary.permanentHPGained += se.amount; break;
                case "gold": summary.goldGained += se.amount; break;
            }
        }

        // Analyze decisions
        summary.optimalDecisions = 0;
        summary.suboptimalDecisions = 0;
        summary.totalEVLost = 0;
        summary.keyMistakes = new ArrayList<>();

        for (Decision d : currentCombatDecisions) {
            if (d.wasOptimal()) {
                summary.optimalDecisions++;
            } else {
                summary.suboptimalDecisions++;
                summary.totalEVLost += d.getEVDelta();

                // Track significant mistakes
                if (d.getEVDelta() < -5) {
                    summary.keyMistakes.add(String.format(
                        "Turn %d: Played %s (EV: %.0f) instead of %s (EV: %.0f)",
                        d.turn, d.cardPlayed, d.evAtTime, d.bestCardAtTime, d.bestEVAtTime
                    ));
                }
            }
        }

        // Estimate optimal HP loss (rough heuristic)
        summary.optimalHpLost = Math.max(0, summary.hpLost + (int)(summary.totalEVLost * 0.5f));

        combatHistory.add(summary);
        return summary;
    }

    // ========== TRACKING HOOKS ==========

    /**
     * Called when a potion is used
     */
    public static void onPotionUsed(AbstractPotion potion, String targetName) {
        PotionUsage usage = new PotionUsage();
        usage.potionId = potion.ID;
        usage.potionName = potion.name;
        usage.turn = AbstractDungeon.actionManager.turn;
        usage.target = targetName;
        usage.hpAtUse = AbstractDungeon.player.currentHealth;

        // Categorize potion type
        if (potion.isThrown) {
            usage.context = "offensive";
        } else if (potion.ID.contains("Block") || potion.ID.contains("Fairy") ||
                   potion.ID.contains("Regen") || potion.ID.contains("Blood")) {
            usage.context = "defensive";
        } else {
            usage.context = "utility";
        }

        currentPotionsUsed.add(usage);
    }

    /**
     * Called when permanent scaling occurs (Ritual Dagger, Feed, etc.)
     */
    public static void onScalingEvent(String source, String type, int amount) {
        ScalingEvent event = new ScalingEvent();
        event.source = source;
        event.type = type;
        event.amount = amount;
        event.turn = AbstractDungeon.actionManager.turn;
        currentScalingEvents.add(event);
    }

    /**
     * Track energy available at turn start
     */
    public static void onTurnStart(int energyAvailable) {
        currentEnergyAvailable += energyAvailable;
    }

    /**
     * Mark that an infinite was detected
     */
    public static void setInfiniteDetected() {
        infiniteDetected = true;
    }

    /**
     * Get the most recent combat summary
     */
    public static CombatSummary getLastCombatSummary() {
        if (combatHistory.isEmpty()) return null;
        return combatHistory.get(combatHistory.size() - 1);
    }

    /**
     * Get all combat summaries for the current run
     */
    public static List<CombatSummary> getCombatHistory() {
        return new ArrayList<>(combatHistory);
    }

    /**
     * Add player comment to last combat
     */
    public static void addComment(String comment) {
        if (!combatHistory.isEmpty()) {
            combatHistory.get(combatHistory.size() - 1).playerComment = comment;
        }
    }

    /**
     * Clear history (on new run)
     */
    public static void clearHistory() {
        combatHistory.clear();
        currentCombatDecisions.clear();
    }

    /**
     * Get run statistics
     */
    public static Map<String, Object> getRunStats() {
        Map<String, Object> stats = new HashMap<>();

        int totalDecisions = 0;
        int totalOptimal = 0;
        int totalHPLost = 0;
        int totalOptimalHPLost = 0;
        float totalEVLost = 0;

        for (CombatSummary s : combatHistory) {
            totalDecisions += s.decisionsCount;
            totalOptimal += s.optimalDecisions;
            totalHPLost += s.hpLost;
            totalOptimalHPLost += s.optimalHpLost;
            totalEVLost += s.totalEVLost;
        }

        stats.put("total_combats", combatHistory.size());
        stats.put("total_decisions", totalDecisions);
        stats.put("optimal_decisions", totalOptimal);
        stats.put("optimality_rate", totalDecisions > 0 ?
                 (float) totalOptimal / totalDecisions * 100 : 100);
        stats.put("total_hp_lost", totalHPLost);
        stats.put("optimal_hp_lost", totalOptimalHPLost);
        stats.put("hp_wasted", totalHPLost - totalOptimalHPLost);
        stats.put("total_ev_lost", totalEVLost);

        return stats;
    }
}
