package evtracker.search;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import evtracker.TurnStateCapture;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.text.SimpleDateFormat;
import java.util.*;

/**
 * Verifies Python predictions match actual Java game state.
 * Logs mismatches for debugging simulation accuracy.
 */
public class StateVerifier {
    private static final Logger logger = LogManager.getLogger(StateVerifier.class);
    private static final Gson gson = new GsonBuilder().setPrettyPrinting().create();

    // Stored prediction from last search response
    private static Map<String, Object> predictedState;
    private static String lastActionDescription;

    // Statistics
    private static int totalVerifications = 0;
    private static int totalMismatches = 0;
    private static Map<String, Integer> mismatchByField = new HashMap<>();

    // Log directory
    private static final String LOG_DIR = System.getProperty("user.home") +
        "/Desktop/SlayTheSpireRL/logs/verification";

    static {
        // Ensure log directory exists
        new File(LOG_DIR).mkdirs();
    }

    /**
     * Store the predicted outcome for verification after action.
     */
    public static void setPredictedState(SearchResponse response, String actionDescription) {
        if (response == null || !response.hasBestLine()) {
            predictedState = null;
            return;
        }

        lastActionDescription = actionDescription;

        // Store expected outcomes
        predictedState = new HashMap<>();
        SearchResponse.ExpectedOutcome outcome = response.bestLine.expectedOutcome;

        if (outcome != null) {
            predictedState.put("hp_after_action", getCurrentPlayerHp() - outcome.hpLost);
            predictedState.put("block_remaining", outcome.blockRemaining);
            predictedState.put("energy_remaining", outcome.energyRemaining);

            // Store enemy HP predictions if we have them
            if (outcome.damageDealt > 0) {
                predictedState.put("damage_dealt", outcome.damageDealt);
            }
        }
    }

    /**
     * Verify the predicted state matches actual state after an action.
     *
     * @param actionType Type of action (card, potion, end_turn)
     * @param actionDetail Details about the action
     */
    public static void verifyAfterAction(String actionType, String actionDetail) {
        if (predictedState == null) {
            return;
        }

        totalVerifications++;

        List<Map<String, Object>> mismatches = new ArrayList<>();

        // Get actual state
        Map<String, Object> actualState = captureActualState();

        // Compare HP
        Integer predictedHp = (Integer) predictedState.get("hp_after_action");
        Integer actualHp = (Integer) actualState.get("player_hp");
        if (predictedHp != null && actualHp != null && !predictedHp.equals(actualHp)) {
            Map<String, Object> mismatch = new HashMap<>();
            mismatch.put("field", "player.hp");
            mismatch.put("predicted", predictedHp);
            mismatch.put("actual", actualHp);
            mismatch.put("diff", actualHp - predictedHp);
            mismatch.put("diagnosis", diagnoseHpMismatch(actualHp - predictedHp));
            mismatches.add(mismatch);
            incrementMismatchCount("player.hp");
        }

        // Compare block
        Integer predictedBlock = (Integer) predictedState.get("block_remaining");
        Integer actualBlock = (Integer) actualState.get("player_block");
        if (predictedBlock != null && actualBlock != null && !predictedBlock.equals(actualBlock)) {
            Map<String, Object> mismatch = new HashMap<>();
            mismatch.put("field", "player.block");
            mismatch.put("predicted", predictedBlock);
            mismatch.put("actual", actualBlock);
            mismatch.put("diff", actualBlock - predictedBlock);
            mismatch.put("diagnosis", diagnoseBlockMismatch(actualBlock - predictedBlock));
            mismatches.add(mismatch);
            incrementMismatchCount("player.block");
        }

        // Compare energy
        Integer predictedEnergy = (Integer) predictedState.get("energy_remaining");
        Integer actualEnergy = (Integer) actualState.get("energy");
        if (predictedEnergy != null && actualEnergy != null && !predictedEnergy.equals(actualEnergy)) {
            Map<String, Object> mismatch = new HashMap<>();
            mismatch.put("field", "energy");
            mismatch.put("predicted", predictedEnergy);
            mismatch.put("actual", actualEnergy);
            mismatch.put("diff", actualEnergy - predictedEnergy);
            mismatch.put("diagnosis", diagnoseEnergyMismatch(actualEnergy - predictedEnergy));
            mismatches.add(mismatch);
            incrementMismatchCount("energy");
        }

        // Log mismatches
        if (!mismatches.isEmpty()) {
            totalMismatches++;
            logMismatch(actionType, actionDetail, mismatches, actualState);
            logger.warn("State verification failed: {} mismatches", mismatches.size());
        }

        // Clear prediction
        predictedState = null;
    }

    /**
     * Capture current actual state for comparison.
     */
    private static Map<String, Object> captureActualState() {
        Map<String, Object> state = new HashMap<>();

        AbstractPlayer p = AbstractDungeon.player;
        if (p == null) return state;

        state.put("player_hp", p.currentHealth);
        state.put("player_max_hp", p.maxHealth);
        state.put("player_block", p.currentBlock);
        state.put("energy", com.megacrit.cardcrawl.ui.panels.EnergyPanel.totalCount);

        if (p.stance != null) {
            state.put("stance", p.stance.ID);
        }

        // Enemy HP
        if (AbstractDungeon.getCurrRoom() != null &&
            AbstractDungeon.getCurrRoom().monsters != null) {
            List<Integer> enemyHps = new ArrayList<>();
            for (AbstractMonster m : AbstractDungeon.getCurrRoom().monsters.monsters) {
                if (!m.isDead && !m.halfDead) {
                    enemyHps.add(m.currentHealth);
                }
            }
            state.put("enemy_hps", enemyHps);
        }

        state.put("hand_size", p.hand.size());

        return state;
    }

    private static int getCurrentPlayerHp() {
        AbstractPlayer p = AbstractDungeon.player;
        return p != null ? p.currentHealth : 0;
    }

    private static String diagnoseHpMismatch(int diff) {
        if (diff > 0) {
            return "Player took less damage than predicted. Check: Block calc, enemy Weak, Intangible";
        } else {
            return "Player took more damage than predicted. Check: Vulnerable, Wrath damage mult";
        }
    }

    private static String diagnoseBlockMismatch(int diff) {
        if (diff > 0) {
            return "More block than predicted. Check: Dexterity, block relics";
        } else {
            return "Less block than predicted. Check: Frail, block removal effects";
        }
    }

    private static String diagnoseEnergyMismatch(int diff) {
        if (diff > 0) {
            return "More energy than predicted. Check: Calm exit, energy relics";
        } else {
            return "Less energy than predicted. Check: Card costs, X-cost cards";
        }
    }

    private static void incrementMismatchCount(String field) {
        mismatchByField.put(field, mismatchByField.getOrDefault(field, 0) + 1);
    }

    /**
     * Log mismatch to file.
     */
    private static void logMismatch(
        String actionType,
        String actionDetail,
        List<Map<String, Object>> mismatches,
        Map<String, Object> actualState
    ) {
        String timestamp = new SimpleDateFormat("yyyyMMdd_HHmmss").format(new Date());
        String filename = String.format("mismatch_%04d_%s.json", totalVerifications, timestamp);
        File logFile = new File(LOG_DIR, filename);

        Map<String, Object> logData = new HashMap<>();
        logData.put("verification_id", totalVerifications);
        logData.put("timestamp", System.currentTimeMillis());
        logData.put("action_type", actionType);
        logData.put("action_detail", actionDetail);
        logData.put("action_description", lastActionDescription);
        logData.put("predicted_state", predictedState);
        logData.put("actual_state", actualState);
        logData.put("mismatches", mismatches);

        // Add full game state for debugging
        try {
            logData.put("full_state", TurnStateCapture.captureStateMap());
        } catch (Exception e) {
            logger.warn("Failed to capture full state: {}", e.getMessage());
        }

        try (Writer writer = new BufferedWriter(
            new OutputStreamWriter(new FileOutputStream(logFile), StandardCharsets.UTF_8))) {
            gson.toJson(logData, writer);
            logger.info("Mismatch logged to: {}", logFile.getAbsolutePath());
        } catch (IOException e) {
            logger.error("Failed to write mismatch log: {}", e.getMessage());
        }
    }

    /**
     * Get verification statistics.
     */
    public static Map<String, Object> getStats() {
        Map<String, Object> stats = new HashMap<>();
        stats.put("total_verifications", totalVerifications);
        stats.put("total_mismatches", totalMismatches);
        stats.put("accuracy", totalVerifications > 0 ?
            (float)(totalVerifications - totalMismatches) / totalVerifications : 1.0f);
        stats.put("mismatch_by_field", new HashMap<>(mismatchByField));
        return stats;
    }

    /**
     * Get statistics as string for display.
     */
    public static String getStatsString() {
        Map<String, Object> stats = getStats();
        float accuracy = (float) stats.get("accuracy");
        return String.format("Verification: %d/%d (%.1f%% accurate)",
            totalVerifications - totalMismatches,
            totalVerifications,
            accuracy * 100);
    }

    /**
     * Reset statistics.
     */
    public static void resetStats() {
        totalVerifications = 0;
        totalMismatches = 0;
        mismatchByField.clear();
    }
}
