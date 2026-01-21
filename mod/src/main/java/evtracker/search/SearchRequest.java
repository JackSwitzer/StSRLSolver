package evtracker.search;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.relics.AbstractRelic;

import java.util.*;

/**
 * Request data class for combat search.
 * Captures all state needed for Python simulation.
 */
public class SearchRequest {
    private static final Gson gson = new GsonBuilder().create();

    public String type = "search_request";
    public String request_id;
    public Map<String, Object> seed;
    public Map<String, Object> rng_state;
    public Map<String, Object> player;
    public Map<String, Object> card_piles;
    public List<Map<String, Object>> enemies;
    public List<Map<String, Object>> potions;
    public List<Map<String, Object>> relics;
    public Map<String, Object> search_params;

    public SearchRequest() {
        this.request_id = UUID.randomUUID().toString();
        this.search_params = new HashMap<>();
    }

    /**
     * Build a search request from current game state.
     */
    public static SearchRequest fromCurrentState() {
        SearchRequest req = new SearchRequest();

        // Seed info
        req.seed = new HashMap<>();
        if (Settings.seed != null) {
            req.seed.put("seed_long", Settings.seed);
            req.seed.put("seed_string", SeedHelper.getString(Settings.seed));
        }

        // RNG state
        req.rng_state = getRngState();

        // Player state
        req.player = capturePlayerState();

        // Card piles
        req.card_piles = captureCardPiles();

        // Enemies
        req.enemies = captureEnemies();

        // Potions
        req.potions = capturePotions();

        // Relics
        req.relics = captureRelics();

        // Default search params
        req.search_params.put("budget_ms", 2000);
        req.search_params.put("iterations", 1000);
        req.search_params.put("algorithm", "mcts");

        return req;
    }

    private static Map<String, Object> getRngState() {
        Map<String, Object> state = new HashMap<>();
        Map<String, Object> counters = new HashMap<>();

        if (AbstractDungeon.cardRng != null)
            counters.put("cardRng", AbstractDungeon.cardRng.counter);
        if (AbstractDungeon.monsterRng != null)
            counters.put("monsterRng", AbstractDungeon.monsterRng.counter);
        if (AbstractDungeon.eventRng != null)
            counters.put("eventRng", AbstractDungeon.eventRng.counter);
        if (AbstractDungeon.relicRng != null)
            counters.put("relicRng", AbstractDungeon.relicRng.counter);
        if (AbstractDungeon.treasureRng != null)
            counters.put("treasureRng", AbstractDungeon.treasureRng.counter);
        if (AbstractDungeon.potionRng != null)
            counters.put("potionRng", AbstractDungeon.potionRng.counter);
        if (AbstractDungeon.merchantRng != null)
            counters.put("merchantRng", AbstractDungeon.merchantRng.counter);
        if (AbstractDungeon.monsterHpRng != null)
            counters.put("monsterHpRng", AbstractDungeon.monsterHpRng.counter);
        if (AbstractDungeon.aiRng != null)
            counters.put("aiRng", AbstractDungeon.aiRng.counter);
        if (AbstractDungeon.shuffleRng != null)
            counters.put("shuffleRng", AbstractDungeon.shuffleRng.counter);
        if (AbstractDungeon.cardRandomRng != null)
            counters.put("cardRandomRng", AbstractDungeon.cardRandomRng.counter);
        if (AbstractDungeon.miscRng != null)
            counters.put("miscRng", AbstractDungeon.miscRng.counter);

        state.put("rng_counters", counters);
        if (Settings.seed != null) {
            state.put("seed", Settings.seed);
            state.put("seed_string", SeedHelper.getString(Settings.seed));
        }

        return state;
    }

    private static Map<String, Object> capturePlayerState() {
        Map<String, Object> state = new HashMap<>();
        AbstractPlayer p = AbstractDungeon.player;

        if (p == null) return state;

        state.put("current_hp", p.currentHealth);
        state.put("max_hp", p.maxHealth);
        state.put("block", p.currentBlock);
        state.put("energy", com.megacrit.cardcrawl.ui.panels.EnergyPanel.totalCount);
        state.put("max_energy", p.energy.energy);

        // Stance
        if (p.stance != null) {
            state.put("stance_id", p.stance.ID);
            state.put("stance_name", p.stance.name);
        } else {
            state.put("stance_id", "Neutral");
            state.put("stance_name", "Neutral");
        }

        // Powers
        List<Map<String, Object>> powers = new ArrayList<>();
        for (com.megacrit.cardcrawl.powers.AbstractPower power : p.powers) {
            Map<String, Object> powerMap = new HashMap<>();
            powerMap.put("id", power.ID);
            powerMap.put("name", power.name);
            powerMap.put("amount", power.amount);
            powerMap.put("type", power.type.toString());
            powers.add(powerMap);
        }
        state.put("powers", powers);

        // Common status shortcuts
        state.put("strength", getPlayerPowerAmount("Strength"));
        state.put("dexterity", getPlayerPowerAmount("Dexterity"));
        state.put("vulnerable", getPlayerPowerAmount("Vulnerable"));
        state.put("weak", getPlayerPowerAmount("Weak"));
        state.put("frail", getPlayerPowerAmount("Frail"));
        state.put("intangible", getPlayerPowerAmount("Intangible"));

        return state;
    }

    private static int getPlayerPowerAmount(String powerId) {
        AbstractPlayer p = AbstractDungeon.player;
        if (p == null) return 0;

        for (com.megacrit.cardcrawl.powers.AbstractPower power : p.powers) {
            if (power.ID.equals(powerId)) {
                return power.amount;
            }
        }
        return 0;
    }

    private static Map<String, Object> captureCardPiles() {
        Map<String, Object> piles = new HashMap<>();
        AbstractPlayer p = AbstractDungeon.player;

        if (p == null) return piles;

        // Hand - full details
        List<Map<String, Object>> hand = new ArrayList<>();
        for (AbstractCard card : p.hand.group) {
            hand.add(captureCard(card));
        }
        piles.put("hand", hand);

        // Draw pile - just IDs (hidden during combat)
        List<Map<String, Object>> drawPile = new ArrayList<>();
        for (AbstractCard card : p.drawPile.group) {
            Map<String, Object> cardMap = new HashMap<>();
            cardMap.put("id", card.cardID);
            cardMap.put("name", card.name);
            cardMap.put("uuid", card.uuid.toString());
            drawPile.add(cardMap);
        }
        piles.put("draw_pile", drawPile);

        // Discard pile - full details
        List<Map<String, Object>> discardPile = new ArrayList<>();
        for (AbstractCard card : p.discardPile.group) {
            discardPile.add(captureCard(card));
        }
        piles.put("discard_pile", discardPile);

        // Exhaust pile - full details
        List<Map<String, Object>> exhaustPile = new ArrayList<>();
        for (AbstractCard card : p.exhaustPile.group) {
            exhaustPile.add(captureCard(card));
        }
        piles.put("exhaust_pile", exhaustPile);

        return piles;
    }

    private static Map<String, Object> captureCard(AbstractCard card) {
        Map<String, Object> cardMap = new HashMap<>();

        cardMap.put("id", card.cardID);
        cardMap.put("name", card.name);
        cardMap.put("uuid", card.uuid.toString());
        cardMap.put("type", card.type.toString());
        cardMap.put("rarity", card.rarity.toString());
        cardMap.put("color", card.color.toString());
        cardMap.put("cost", card.cost);
        cardMap.put("cost_for_turn", card.costForTurn);
        cardMap.put("base_damage", card.baseDamage);
        cardMap.put("damage", card.damage);
        cardMap.put("base_block", card.baseBlock);
        cardMap.put("block", card.block);
        cardMap.put("magic_number", card.magicNumber);
        cardMap.put("upgraded", card.upgraded);
        cardMap.put("timesUpgraded", card.timesUpgraded);
        cardMap.put("exhausts", card.exhaust);
        cardMap.put("ethereal", card.isEthereal);
        cardMap.put("innate", card.isInnate);
        cardMap.put("retain", card.retain || card.selfRetain);
        cardMap.put("target", card.target.toString());
        cardMap.put("can_use", card.canUse(AbstractDungeon.player, null));
        cardMap.put("has_enough_energy", card.costForTurn <= com.megacrit.cardcrawl.ui.panels.EnergyPanel.totalCount);

        return cardMap;
    }

    private static List<Map<String, Object>> captureEnemies() {
        List<Map<String, Object>> enemies = new ArrayList<>();

        if (AbstractDungeon.getCurrRoom() == null ||
            AbstractDungeon.getCurrRoom().monsters == null) {
            return enemies;
        }

        for (AbstractMonster m : AbstractDungeon.getCurrRoom().monsters.monsters) {
            Map<String, Object> enemy = new HashMap<>();

            enemy.put("id", m.id);
            enemy.put("name", m.name);
            enemy.put("index", AbstractDungeon.getCurrRoom().monsters.monsters.indexOf(m));
            enemy.put("is_dead", m.isDead);
            enemy.put("is_dying", m.isDying);
            enemy.put("half_dead", m.halfDead);
            enemy.put("escaped", m.escaped);
            enemy.put("current_hp", m.currentHealth);
            enemy.put("max_hp", m.maxHealth);
            enemy.put("block", m.currentBlock);
            enemy.put("intent", m.intent.toString());
            enemy.put("intent_base_damage", m.getIntentBaseDmg());
            enemy.put("intent_multi", getIntentMulti(m));
            enemy.put("intent_calculated_damage", calculateIntentDamage(m));

            // Move history for AI prediction
            List<Byte> moveHistory = new ArrayList<>();
            if (m.moveHistory != null) {
                for (Byte move : m.moveHistory) {
                    moveHistory.add(move);
                }
            }
            enemy.put("move_history", moveHistory);

            // Powers
            List<Map<String, Object>> powers = new ArrayList<>();
            for (com.megacrit.cardcrawl.powers.AbstractPower power : m.powers) {
                Map<String, Object> powerMap = new HashMap<>();
                powerMap.put("id", power.ID);
                powerMap.put("name", power.name);
                powerMap.put("amount", power.amount);
                powerMap.put("type", power.type.toString());
                powers.add(powerMap);
            }
            enemy.put("powers", powers);

            // Common status shortcuts
            enemy.put("strength", getEnemyPowerAmount(m, "Strength"));
            enemy.put("vulnerable", getEnemyPowerAmount(m, "Vulnerable"));
            enemy.put("weak", getEnemyPowerAmount(m, "Weak"));

            enemies.add(enemy);
        }

        return enemies;
    }

    private static int getIntentMulti(AbstractMonster m) {
        try {
            java.lang.reflect.Field field = AbstractMonster.class.getDeclaredField("intentMultiAmt");
            field.setAccessible(true);
            return field.getInt(m);
        } catch (Exception e) {
            return -1;
        }
    }

    private static int calculateIntentDamage(AbstractMonster m) {
        int baseDmg = m.getIntentBaseDmg();
        if (baseDmg < 0) return -1;

        // Apply player modifiers
        AbstractPlayer p = AbstractDungeon.player;
        float damage = baseDmg;

        // Apply Wrath stance (2x incoming damage)
        if (p.stance != null && p.stance.ID.equals("Wrath")) {
            damage *= 2.0f;
        }

        // Apply Vulnerable (50% more damage taken)
        if (p.hasPower("Vulnerable")) {
            damage *= 1.5f;
        }

        // Apply Weak on enemy (25% less damage)
        if (m.hasPower("Weak")) {
            damage *= 0.75f;
        }

        return (int) Math.floor(damage);
    }

    private static int getEnemyPowerAmount(AbstractMonster m, String powerId) {
        for (com.megacrit.cardcrawl.powers.AbstractPower power : m.powers) {
            if (power.ID.equals(powerId)) {
                return power.amount;
            }
        }
        return 0;
    }

    private static List<Map<String, Object>> capturePotions() {
        List<Map<String, Object>> potions = new ArrayList<>();
        AbstractPlayer p = AbstractDungeon.player;

        if (p == null) return potions;

        for (int i = 0; i < p.potions.size(); i++) {
            AbstractPotion potion = p.potions.get(i);
            Map<String, Object> potionMap = new HashMap<>();

            potionMap.put("id", potion.ID);
            potionMap.put("name", potion.name);
            potionMap.put("slot", i);
            potionMap.put("can_use", potion.canUse());
            potionMap.put("can_discard", potion.canDiscard());
            potionMap.put("rarity", potion.rarity.toString());
            potionMap.put("is_placeholder", potion.ID.equals("Potion Slot"));

            potions.add(potionMap);
        }

        return potions;
    }

    private static List<Map<String, Object>> captureRelics() {
        List<Map<String, Object>> relics = new ArrayList<>();
        AbstractPlayer p = AbstractDungeon.player;

        if (p == null) return relics;

        for (AbstractRelic relic : p.relics) {
            Map<String, Object> relicMap = new HashMap<>();

            relicMap.put("id", relic.relicId);
            relicMap.put("name", relic.name);
            relicMap.put("counter", relic.counter);
            relicMap.put("tier", relic.tier.toString());
            relicMap.put("used_up", relic.usedUp);

            relics.add(relicMap);
        }

        return relics;
    }

    /**
     * Serialize to JSON string.
     */
    public String toJson() {
        return gson.toJson(this);
    }
}
