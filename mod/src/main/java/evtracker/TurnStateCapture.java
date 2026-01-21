package evtracker;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.powers.AbstractPower;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.ui.panels.EnergyPanel;

import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Captures full combat state at turn start for Python consumption.
 *
 * Outputs JSON to /tmp/evtracker_state.json containing:
 * - Player: HP, block, energy, hand, draw pile, discard pile, exhaust pile
 * - Monsters: HP, block, intent, powers, debuffs
 * - Run state: relics, potions, gold, floor, seed, RNG counters
 *
 * Usage:
 * - Call captureState() at turn start (OnPlayerTurnStartPostDraw)
 * - Returns JSON string for logging/socket
 * - Also writes to file for Python to read
 */
public class TurnStateCapture {

    private static final String STATE_FILE_PATH = "/tmp/evtracker_state.json";
    private static final Gson gson = new GsonBuilder().setPrettyPrinting().create();

    /**
     * Capture full combat state and write to file.
     * Call this at the start of each player turn (after draw).
     *
     * @return JSON string of the captured state
     */
    public static String captureState() {
        Map<String, Object> state = captureStateMap();

        String json = gson.toJson(state);

        // Write to file for Python
        writeToFile(json);

        return json;
    }

    /**
     * Capture state as a Map (for embedding in other logs).
     */
    public static Map<String, Object> captureStateMap() {
        Map<String, Object> state = new HashMap<>();

        // Metadata
        state.put("timestamp", System.currentTimeMillis());
        state.put("capture_type", "turn_state");

        // Run info
        state.put("run_info", captureRunInfo());

        // Player state
        state.put("player", capturePlayerState());

        // Card piles
        state.put("hand", captureCardPile(AbstractDungeon.player.hand.group, true));
        state.put("draw_pile", captureCardPile(AbstractDungeon.player.drawPile.group, false));
        state.put("discard_pile", captureCardPile(AbstractDungeon.player.discardPile.group, true));
        state.put("exhaust_pile", captureCardPile(AbstractDungeon.player.exhaustPile.group, true));

        // Monster states
        state.put("monsters", captureMonsterStates());

        // Relics and potions
        state.put("relics", captureRelics());
        state.put("potions", capturePotions());

        // RNG state for sync
        state.put("rng_state", ConsoleCommands.getRngState());

        // Combat calculations
        state.put("combat_analysis", captureCombatAnalysis());

        return state;
    }

    /**
     * Write state to file for Python consumption.
     */
    private static void writeToFile(String json) {
        try (PrintWriter writer = new PrintWriter(new FileWriter(STATE_FILE_PATH))) {
            writer.print(json);
        } catch (IOException e) {
            System.err.println("[EVTracker] Failed to write state: " + e.getMessage());
        }
    }

    // ========== RUN INFO ==========

    private static Map<String, Object> captureRunInfo() {
        Map<String, Object> info = new HashMap<>();

        info.put("floor", AbstractDungeon.floorNum);
        info.put("act", AbstractDungeon.actNum);
        info.put("gold", AbstractDungeon.player.gold);

        if (Settings.seed != null) {
            info.put("seed", Settings.seed);
            info.put("seed_string", SeedHelper.getString(Settings.seed));
        }

        info.put("ascension_level", AbstractDungeon.ascensionLevel);
        info.put("character", AbstractDungeon.player.chosenClass.name());

        // Room info
        if (AbstractDungeon.getCurrRoom() != null) {
            AbstractRoom room = AbstractDungeon.getCurrRoom();
            info.put("room_type", room.getClass().getSimpleName());
            info.put("room_phase", room.phase.name());
        }

        return info;
    }

    // ========== PLAYER STATE ==========

    private static Map<String, Object> capturePlayerState() {
        AbstractPlayer p = AbstractDungeon.player;
        Map<String, Object> state = new HashMap<>();

        // Health
        state.put("current_hp", p.currentHealth);
        state.put("max_hp", p.maxHealth);
        state.put("block", p.currentBlock);

        // Energy
        state.put("energy", EnergyPanel.totalCount);
        state.put("max_energy", EnergyPanel.getCurrentEnergy());

        // Orbs (Defect)
        state.put("orb_slots", p.orbs != null ? p.orbs.size() : 0);
        if (p.orbs != null && !p.orbs.isEmpty()) {
            List<Map<String, Object>> orbs = new ArrayList<>();
            for (com.megacrit.cardcrawl.orbs.AbstractOrb orb : p.orbs) {
                Map<String, Object> orbInfo = new HashMap<>();
                orbInfo.put("id", orb.ID);
                orbInfo.put("name", orb.name);
                orbInfo.put("passive_amount", orb.passiveAmount);
                orbInfo.put("evoke_amount", orb.evokeAmount);
                orbs.add(orbInfo);
            }
            state.put("orbs", orbs);
        }

        // Stance (Watcher)
        if (p.stance != null) {
            state.put("stance_id", p.stance.ID);
            state.put("stance_name", p.stance.name);
        }

        // Powers
        state.put("powers", capturePowers(p.powers));

        // Key combat stats
        state.put("strength", DamageCalculator.getPlayerStrength());
        state.put("dexterity", DamageCalculator.getPlayerDexterity());
        state.put("vulnerable", DamageCalculator.getPlayerVulnerable());
        state.put("weak", DamageCalculator.getPlayerWeak());
        state.put("intangible", DamageCalculator.getPlayerIntangible());

        return state;
    }

    // ========== CARD PILES ==========

    /**
     * Capture a card pile.
     * @param cards The cards to capture
     * @param includeDetails If true, include full card details. If false, just ID/name (for draw pile).
     */
    private static List<Map<String, Object>> captureCardPile(ArrayList<AbstractCard> cards, boolean includeDetails) {
        List<Map<String, Object>> pile = new ArrayList<>();

        for (AbstractCard card : cards) {
            Map<String, Object> cardInfo = new HashMap<>();

            cardInfo.put("id", card.cardID);
            cardInfo.put("name", card.name);
            cardInfo.put("uuid", card.uuid.toString());

            if (includeDetails) {
                cardInfo.put("type", card.type.name());
                cardInfo.put("rarity", card.rarity.name());
                cardInfo.put("color", card.color.name());

                // Cost
                cardInfo.put("cost", card.cost);
                cardInfo.put("cost_for_turn", card.costForTurn);

                // Damage/Block values
                cardInfo.put("base_damage", card.baseDamage);
                cardInfo.put("damage", card.damage);
                cardInfo.put("base_block", card.baseBlock);
                cardInfo.put("block", card.block);
                cardInfo.put("magic_number", card.magicNumber);

                // State
                cardInfo.put("upgraded", card.upgraded);
                cardInfo.put("timesUpgraded", card.timesUpgraded);
                cardInfo.put("exhausts", card.exhaust);
                cardInfo.put("ethereal", card.isEthereal);
                cardInfo.put("innate", card.isInnate);
                cardInfo.put("retain", card.retain);

                // Target type
                cardInfo.put("target", card.target.name());

                // Playability
                cardInfo.put("can_use", card.canUse(AbstractDungeon.player, null));
                cardInfo.put("has_enough_energy", card.costForTurn <= EnergyPanel.totalCount || card.freeToPlay());
            }

            pile.add(cardInfo);
        }

        return pile;
    }

    // ========== MONSTER STATES ==========

    private static List<Map<String, Object>> captureMonsterStates() {
        List<Map<String, Object>> monsters = new ArrayList<>();

        if (AbstractDungeon.getMonsters() == null) {
            return monsters;
        }

        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            Map<String, Object> state = new HashMap<>();

            state.put("id", m.id);
            state.put("name", m.name);
            state.put("index", AbstractDungeon.getMonsters().monsters.indexOf(m));

            // Alive status
            state.put("is_dead", m.isDead);
            state.put("is_dying", m.isDying);
            state.put("half_dead", m.halfDead);
            state.put("escaped", m.escaped);

            if (!m.isDead && !m.isDying) {
                // Health
                state.put("current_hp", m.currentHealth);
                state.put("max_hp", m.maxHealth);
                state.put("block", m.currentBlock);

                // Intent
                state.put("intent", m.intent.name());
                int intentDamage = m.getIntentDmg();
                if (intentDamage >= 0) {
                    state.put("intent_base_damage", intentDamage);
                    state.put("intent_multi", DamageCalculator.getMonsterIntentMulti(m));

                    // Calculated damage with all modifiers
                    int actualDamage = DamageCalculator.calculateIncomingDamage(m);
                    state.put("intent_calculated_damage", actualDamage);
                }

                // Powers
                state.put("powers", capturePowers(m.powers));

                // Key combat stats
                state.put("strength", DamageCalculator.getMonsterPowerAmount(m, "Strength"));
                state.put("vulnerable", DamageCalculator.getMonsterPowerAmount(m, "Vulnerable"));
                state.put("weak", DamageCalculator.getMonsterPowerAmount(m, "Weak"));

                // Move history
                List<Integer> moveHistory = new ArrayList<>();
                for (Byte move : m.moveHistory) {
                    moveHistory.add(move.intValue());
                }
                state.put("move_history", moveHistory);
            }

            monsters.add(state);
        }

        return monsters;
    }

    // ========== POWERS ==========

    private static List<Map<String, Object>> capturePowers(ArrayList<AbstractPower> powers) {
        List<Map<String, Object>> powerList = new ArrayList<>();

        for (AbstractPower power : powers) {
            Map<String, Object> powerInfo = new HashMap<>();

            powerInfo.put("id", power.ID);
            powerInfo.put("name", power.name);
            powerInfo.put("amount", power.amount);
            powerInfo.put("type", power.type.name()); // BUFF or DEBUFF

            powerList.add(powerInfo);
        }

        return powerList;
    }

    // ========== RELICS ==========

    private static List<Map<String, Object>> captureRelics() {
        List<Map<String, Object>> relics = new ArrayList<>();

        for (AbstractRelic relic : AbstractDungeon.player.relics) {
            Map<String, Object> info = new HashMap<>();

            info.put("id", relic.relicId);
            info.put("name", relic.name);
            info.put("counter", relic.counter);
            info.put("tier", relic.tier.name());
            info.put("used_up", relic.usedUp);

            // Specific relic state (e.g., Pen Nib at 9 = next attack doubled)
            if (relic.relicId.equals("Pen Nib")) {
                info.put("ready", relic.counter == 9);
            } else if (relic.relicId.equals("Incense Burner")) {
                info.put("ready", relic.counter == 5);
            } else if (relic.relicId.equals("Nunchaku")) {
                info.put("ready", relic.counter == 9);
            } else if (relic.relicId.equals("Happy Flower")) {
                info.put("ready", relic.counter == 2);
            }

            relics.add(info);
        }

        return relics;
    }

    // ========== POTIONS ==========

    private static List<Map<String, Object>> capturePotions() {
        List<Map<String, Object>> potions = new ArrayList<>();

        for (AbstractPotion potion : AbstractDungeon.player.potions) {
            Map<String, Object> info = new HashMap<>();

            info.put("id", potion.ID);
            info.put("name", potion.name);
            info.put("slot", AbstractDungeon.player.potions.indexOf(potion));
            info.put("can_use", potion.canUse());
            info.put("can_discard", potion.canDiscard());
            info.put("rarity", potion.rarity.name());
            info.put("is_placeholder", potion.ID.equals("Potion Slot"));

            potions.add(info);
        }

        return potions;
    }

    // ========== COMBAT ANALYSIS ==========

    private static Map<String, Object> captureCombatAnalysis() {
        Map<String, Object> analysis = new HashMap<>();

        // Incoming damage this turn
        analysis.put("incoming_damage", DamageCalculator.calculateTotalIncomingDamage());
        analysis.put("net_damage", DamageCalculator.calculateNetDamage());

        // Potential damage from hand
        analysis.put("potential_hand_damage", DamageCalculator.calculatePotentialHandDamage());

        // Turns to kill
        DamageCalculator.TurnsToKillResult ttk = DamageCalculator.calculateTurnsToKill();
        Map<String, Object> ttkInfo = new HashMap<>();
        ttkInfo.put("turns", ttk.turnsToKill);
        ttkInfo.put("expected_damage_taken", ttk.expectedDamageTaken);
        ttkInfo.put("total_enemy_hp", ttk.totalEnemyHP);
        ttkInfo.put("potential_damage_per_turn", ttk.potentialDamagePerTurn);
        analysis.put("turns_to_kill", ttkInfo);

        // Damage modifiers
        analysis.put("damage_modifiers", DamageCalculator.extractDamageModifiers());

        // Stance info (Watcher)
        if (AbstractDungeon.player.stance != null) {
            analysis.put("stance_damage_multiplier", DamageCalculator.getStanceDamageMultiplier());
        }

        return analysis;
    }

    // ========== MINIMAL STATE (for frequent updates) ==========

    /**
     * Capture minimal state for frequent updates (e.g., after each action).
     * Less data than full captureState() for performance.
     */
    public static String captureMinimalState() {
        Map<String, Object> state = new HashMap<>();

        state.put("timestamp", System.currentTimeMillis());
        state.put("floor", AbstractDungeon.floorNum);

        // Player vitals
        AbstractPlayer p = AbstractDungeon.player;
        state.put("player_hp", p.currentHealth);
        state.put("player_block", p.currentBlock);
        state.put("energy", EnergyPanel.totalCount);
        state.put("hand_size", p.hand.size());

        // Monster vitals
        List<Map<String, Object>> monsters = new ArrayList<>();
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    Map<String, Object> monster = new HashMap<>();
                    monster.put("id", m.id);
                    monster.put("hp", m.currentHealth);
                    monster.put("block", m.currentBlock);
                    monster.put("intent", m.intent.name());
                    monsters.add(monster);
                }
            }
        }
        state.put("monsters", monsters);

        return gson.toJson(state);
    }
}
