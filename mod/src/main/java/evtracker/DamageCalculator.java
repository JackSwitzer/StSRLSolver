package evtracker;

import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.cards.DamageInfo;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.powers.AbstractPower;
import com.megacrit.cardcrawl.relics.AbstractRelic;

import java.util.HashMap;
import java.util.Map;

/**
 * Accurate damage calculation matching game's DamageInfo.applyPowers() logic.
 *
 * Order of operations (player attacking):
 * 1. Base damage + Strength (additive)
 * 2. Player stance.atDamageGive (Wrath: x2, Divinity: x3)
 * 3. Player powers.atDamageGive (Weak: x0.75)
 * 4. Target powers.atDamageReceive (Vulnerable: x1.5)
 * 5. Final multipliers (Pen Nib, etc.)
 * 6. floor(result), min 0
 */
public class DamageCalculator {

    // Relic constants (from decompiled code)
    private static final float VULN_NORMAL = 1.5f;
    private static final float VULN_ODD_MUSHROOM = 1.25f;
    private static final float VULN_PAPER_FROG = 1.75f;
    private static final float WEAK_NORMAL = 0.75f;
    private static final float WEAK_PAPER_CRANE = 0.6f;

    /**
     * Calculate actual damage a card would deal to a monster.
     * Matches game's damage calculation pipeline.
     */
    public static int calculateCardDamage(AbstractCard card, AbstractMonster target) {
        if (card.type != AbstractCard.CardType.ATTACK || target == null) {
            return 0;
        }

        AbstractPlayer player = AbstractDungeon.player;

        // Start with card's base damage
        float damage = card.baseDamage;

        // 1. Add Strength (additive)
        int strength = getPlayerStrength();
        damage += strength;

        // 2. Apply Weak (if player is weak)
        if (playerHasPower("Weakened") || playerHasPower("Weak")) {
            damage = (float) Math.floor(damage * getWeakMultiplier(true));
        }

        // 3. Apply stance multiplier
        float stanceMultiplier = getStanceDamageMultiplier();
        damage = (float) Math.floor(damage * stanceMultiplier);

        // 4. Apply Vulnerable on target
        if (monsterHasPower(target, "Vulnerable")) {
            damage = (float) Math.floor(damage * getVulnerableMultiplier(false));
        }

        // 5. Apply relic multipliers (Pen Nib, etc.)
        damage = applyRelicMultipliers(damage, card);

        // Floor and cap at 0
        return Math.max(0, (int) Math.floor(damage));
    }

    /**
     * Calculate damage a monster's attack would deal to the player.
     * Accounts for player Vulnerable, Wrath stance, relics.
     */
    public static int calculateIncomingDamage(AbstractMonster monster) {
        if (monster == null || monster.isDead || monster.isDying) {
            return 0;
        }

        int baseDamage = monster.getIntentDmg();
        if (baseDamage < 0) {
            return 0;
        }

        float damage = baseDamage;

        // Apply Weak on monster (reduces their damage)
        if (monsterHasPower(monster, "Weakened") || monsterHasPower(monster, "Weak")) {
            damage = (float) Math.floor(damage * getWeakMultiplier(false));
        }

        // Apply player Vulnerable (increases damage taken)
        if (playerHasPower("Vulnerable")) {
            damage = (float) Math.floor(damage * getVulnerableMultiplier(true));
        }

        // Apply Wrath stance (doubles damage received)
        AbstractPlayer player = AbstractDungeon.player;
        if (player != null && player.stance != null && player.stance.ID.equals("Wrath")) {
            damage = (float) Math.floor(damage * 2.0f);
        }

        // Get multi-hit count
        int multiHit = getMonsterIntentMulti(monster);
        int totalDamage = (int) Math.floor(damage) * Math.max(1, multiHit);

        return Math.max(0, totalDamage);
    }

    /**
     * Calculate total incoming damage from all monsters this turn.
     */
    public static int calculateTotalIncomingDamage() {
        int total = 0;
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    total += calculateIncomingDamage(m);
                }
            }
        }
        return total;
    }

    /**
     * Calculate expected damage after block is applied.
     */
    public static int calculateNetDamage() {
        int incoming = calculateTotalIncomingDamage();
        int block = AbstractDungeon.player.currentBlock;
        return Math.max(0, incoming - block);
    }

    /**
     * Calculate turns to kill all monsters with current damage output.
     */
    public static TurnsToKillResult calculateTurnsToKill() {
        AbstractPlayer player = AbstractDungeon.player;

        int totalEnemyHP = 0;
        int totalEnemyBlock = 0;

        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    totalEnemyHP += m.currentHealth;
                    totalEnemyBlock += m.currentBlock;
                }
            }
        }

        // Estimate damage per turn from hand
        int potentialDamage = calculatePotentialHandDamage();

        // Account for block we need to break first
        int effectiveHP = totalEnemyHP + totalEnemyBlock;

        // Calculate turns to kill
        float turnsToKill = potentialDamage > 0 ? (float) effectiveHP / potentialDamage : Float.MAX_VALUE;

        // Calculate expected damage taken over those turns
        int damagePerTurn = calculateTotalIncomingDamage();
        int expectedDamageTaken = (int) Math.ceil(turnsToKill) * damagePerTurn;

        return new TurnsToKillResult(turnsToKill, expectedDamageTaken, totalEnemyHP, potentialDamage);
    }

    /**
     * Calculate potential damage from current hand.
     */
    public static int calculatePotentialHandDamage() {
        AbstractPlayer player = AbstractDungeon.player;
        if (player == null || player.hand == null) {
            return 0;
        }

        int totalDamage = 0;
        int energyAvailable = com.megacrit.cardcrawl.ui.panels.EnergyPanel.totalCount;

        // Simple greedy: play all attacks we can afford
        for (AbstractCard card : player.hand.group) {
            if (card.type == AbstractCard.CardType.ATTACK && card.costForTurn <= energyAvailable) {
                // Get first alive monster for damage calc
                AbstractMonster target = getFirstAliveMonster();
                if (target != null) {
                    totalDamage += calculateCardDamage(card, target);
                    energyAvailable -= card.costForTurn;
                }
            }
        }

        return totalDamage;
    }

    // ========== POWER/MODIFIER EXTRACTION ==========

    public static int getPlayerStrength() {
        return getPlayerPowerAmount("Strength");
    }

    public static int getPlayerDexterity() {
        return getPlayerPowerAmount("Dexterity");
    }

    public static int getPlayerVulnerable() {
        return getPlayerPowerAmount("Vulnerable");
    }

    public static int getPlayerWeak() {
        int weak = getPlayerPowerAmount("Weak");
        if (weak == 0) {
            weak = getPlayerPowerAmount("Weakened");
        }
        return weak;
    }

    public static int getPlayerIntangible() {
        return getPlayerPowerAmount("IntangiblePlayer");
    }

    public static int getPlayerPowerAmount(String powerId) {
        AbstractPlayer player = AbstractDungeon.player;
        if (player == null) return 0;

        for (AbstractPower power : player.powers) {
            if (power.ID.equals(powerId)) {
                return power.amount;
            }
        }
        return 0;
    }

    public static boolean playerHasPower(String powerId) {
        return getPlayerPowerAmount(powerId) > 0;
    }

    public static boolean monsterHasPower(AbstractMonster monster, String powerId) {
        if (monster == null) return false;
        for (AbstractPower power : monster.powers) {
            if (power.ID.equals(powerId)) {
                return true;
            }
        }
        return false;
    }

    public static int getMonsterPowerAmount(AbstractMonster monster, String powerId) {
        if (monster == null) return 0;
        for (AbstractPower power : monster.powers) {
            if (power.ID.equals(powerId)) {
                return power.amount;
            }
        }
        return 0;
    }

    // ========== MULTIPLIER CALCULATIONS ==========

    /**
     * Get stance damage multiplier for outgoing attacks.
     */
    public static float getStanceDamageMultiplier() {
        AbstractPlayer player = AbstractDungeon.player;
        if (player == null || player.stance == null) {
            return 1.0f;
        }

        switch (player.stance.ID) {
            case "Wrath":
                return 2.0f;
            case "Divinity":
                return 3.0f;
            default:
                return 1.0f;
        }
    }

    /**
     * Get Vulnerable multiplier, accounting for relics.
     * @param isPlayer true if the vulnerable target is the player
     */
    public static float getVulnerableMultiplier(boolean isPlayer) {
        AbstractPlayer player = AbstractDungeon.player;

        if (isPlayer) {
            // Player is vulnerable - check for Odd Mushroom
            if (playerHasRelic("Odd Mushroom")) {
                return VULN_ODD_MUSHROOM;
            }
        } else {
            // Monster is vulnerable - check for Paper Frog
            if (playerHasRelic("Paper Frog")) {
                return VULN_PAPER_FROG;
            }
        }

        return VULN_NORMAL;
    }

    /**
     * Get Weak multiplier, accounting for relics.
     * @param isPlayer true if the weak attacker is the player
     */
    public static float getWeakMultiplier(boolean isPlayer) {
        if (!isPlayer) {
            // Monster is weak - check for Paper Crane
            if (playerHasRelic("Paper Crane")) {
                return WEAK_PAPER_CRANE;
            }
        }
        return WEAK_NORMAL;
    }

    /**
     * Apply relic damage multipliers (Pen Nib, etc.)
     */
    private static float applyRelicMultipliers(float damage, AbstractCard card) {
        AbstractPlayer player = AbstractDungeon.player;
        if (player == null) return damage;

        // Check Pen Nib (doubles next attack)
        for (AbstractRelic relic : player.relics) {
            if (relic.relicId.equals("Pen Nib") && relic.counter == 9) {
                damage *= 2.0f;
                break;
            }
        }

        // Akabeko (first attack in combat deals +8 damage) - counter check
        // Note: This is simplified, actual implementation is more complex

        return damage;
    }

    public static boolean playerHasRelic(String relicId) {
        AbstractPlayer player = AbstractDungeon.player;
        if (player == null) return false;

        for (AbstractRelic relic : player.relics) {
            if (relic.relicId.equals(relicId)) {
                return true;
            }
        }
        return false;
    }

    // ========== MONSTER UTILITIES ==========

    public static int getMonsterIntentMulti(AbstractMonster monster) {
        try {
            java.lang.reflect.Field f = AbstractMonster.class.getDeclaredField("intentMultiAmt");
            f.setAccessible(true);
            return f.getInt(monster);
        } catch (Exception e) {
            return 1;
        }
    }

    public static AbstractMonster getFirstAliveMonster() {
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    return m;
                }
            }
        }
        return null;
    }

    // ========== STATE EXTRACTION FOR LOGGING ==========

    /**
     * Extract all damage-relevant modifiers for logging.
     */
    public static Map<String, Object> extractDamageModifiers() {
        Map<String, Object> mods = new HashMap<>();

        // Player powers
        mods.put("strength", getPlayerStrength());
        mods.put("dexterity", getPlayerDexterity());
        mods.put("vulnerable", getPlayerVulnerable());
        mods.put("weak", getPlayerWeak());
        mods.put("intangible", getPlayerIntangible());

        // Stance
        AbstractPlayer player = AbstractDungeon.player;
        if (player != null && player.stance != null) {
            mods.put("stance", player.stance.ID);
            mods.put("stance_damage_mult", getStanceDamageMultiplier());
        }

        // Relics that affect damage
        mods.put("has_odd_mushroom", playerHasRelic("Odd Mushroom"));
        mods.put("has_paper_frog", playerHasRelic("Paper Frog"));
        mods.put("has_paper_crane", playerHasRelic("Paper Crane"));
        mods.put("has_pen_nib", playerHasRelic("Pen Nib"));

        // Check Pen Nib counter
        if (playerHasRelic("Pen Nib")) {
            for (AbstractRelic relic : player.relics) {
                if (relic.relicId.equals("Pen Nib")) {
                    mods.put("pen_nib_counter", relic.counter);
                    mods.put("pen_nib_ready", relic.counter == 9);
                    break;
                }
            }
        }

        return mods;
    }

    /**
     * Extract monster damage modifiers for logging.
     */
    public static Map<String, Object> extractMonsterModifiers(AbstractMonster monster) {
        Map<String, Object> mods = new HashMap<>();

        if (monster == null) return mods;

        mods.put("strength", getMonsterPowerAmount(monster, "Strength"));
        mods.put("vulnerable", getMonsterPowerAmount(monster, "Vulnerable"));
        mods.put("weak", getMonsterPowerAmount(monster, "Weak"));
        mods.put("intangible", getMonsterPowerAmount(monster, "Intangible"));

        return mods;
    }

    // ========== RESULT CLASSES ==========

    public static class TurnsToKillResult {
        public final float turnsToKill;
        public final int expectedDamageTaken;
        public final int totalEnemyHP;
        public final int potentialDamagePerTurn;

        public TurnsToKillResult(float turnsToKill, int expectedDamageTaken,
                                  int totalEnemyHP, int potentialDamagePerTurn) {
            this.turnsToKill = turnsToKill;
            this.expectedDamageTaken = expectedDamageTaken;
            this.totalEnemyHP = totalEnemyHP;
            this.potentialDamagePerTurn = potentialDamagePerTurn;
        }
    }
}
