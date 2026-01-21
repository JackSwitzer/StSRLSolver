package evtracker.search;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.annotations.SerializedName;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Response data class for combat search results.
 * Parsed from Python search server response.
 */
public class SearchResponse {
    private static final Gson gson = new GsonBuilder().create();

    public String type;

    @SerializedName("request_id")
    public String requestId;

    @SerializedName("best_line")
    public BestLine bestLine;

    @SerializedName("alternative_lines")
    public List<BestLine> alternativeLines;

    @SerializedName("search_time_ms")
    public float searchTimeMs;

    @SerializedName("nodes_explored")
    public int nodesExplored;

    public String error;

    /**
     * A recommended action sequence.
     */
    public static class BestLine {
        public List<ActionInfo> actions;

        @SerializedName("expected_outcome")
        public ExpectedOutcome expectedOutcome;

        @SerializedName("display_text")
        public String displayText;

        public float score;
        public int visits;

        /**
         * Get the display text for overlay.
         */
        public String getDisplayText() {
            if (displayText != null && !displayText.isEmpty()) {
                return displayText;
            }
            // Fallback to building from actions
            if (actions == null || actions.isEmpty()) {
                return "No action";
            }
            StringBuilder sb = new StringBuilder();
            for (ActionInfo action : actions) {
                if (sb.length() > 0) sb.append(" + ");
                sb.append(action.getDisplayText());
            }
            return sb.toString();
        }
    }

    /**
     * Information about a single action.
     */
    public static class ActionInfo {
        public String type;  // "card", "potion", "end_turn"

        @SerializedName("card_id")
        public String cardId;

        @SerializedName("card_idx")
        public Integer cardIdx;

        @SerializedName("potion_id")
        public String potionId;

        public Integer slot;  // potion slot
        public Integer target;  // target index

        @SerializedName("target_name")
        public String targetName;

        /**
         * Get display text for this action.
         */
        public String getDisplayText() {
            if ("card".equals(type)) {
                String text = cardId != null ? cardId : "Card";
                if (targetName != null) {
                    text += " -> " + targetName;
                }
                return text;
            } else if ("potion".equals(type)) {
                String text = "Potion";
                if (potionId != null) {
                    text = potionId;
                } else if (slot != null) {
                    text = "Potion [" + slot + "]";
                }
                if (targetName != null) {
                    text += " -> " + targetName;
                }
                return text;
            } else if ("end_turn".equals(type)) {
                return "End Turn";
            }
            return "?";
        }

        /**
         * Check if this is an end turn action.
         */
        public boolean isEndTurn() {
            return "end_turn".equals(type);
        }

        /**
         * Check if this is a card play action.
         */
        public boolean isCardPlay() {
            return "card".equals(type);
        }

        /**
         * Check if this is a potion use action.
         */
        public boolean isPotionUse() {
            return "potion".equals(type);
        }
    }

    /**
     * Expected outcome of an action sequence.
     */
    public static class ExpectedOutcome {
        @SerializedName("hp_lost")
        public int hpLost;

        @SerializedName("damage_dealt")
        public int damageDealt;

        @SerializedName("enemy_killed")
        public boolean enemyKilled;

        @SerializedName("player_dead")
        public boolean playerDead;

        @SerializedName("block_remaining")
        public int blockRemaining;

        @SerializedName("energy_remaining")
        public int energyRemaining;

        /**
         * Get a summary string for overlay display.
         */
        public String getSummary() {
            if (enemyKilled) {
                return "KILL";
            } else if (playerDead) {
                return "DEAD";
            } else if (hpLost == 0) {
                return "0 dmg taken";
            } else if (hpLost > 0) {
                return hpLost + " dmg taken";
            } else {
                return damageDealt + " dealt";
            }
        }
    }

    /**
     * Parse from JSON string.
     */
    public static SearchResponse fromJson(String json) {
        try {
            return gson.fromJson(json, SearchResponse.class);
        } catch (Exception e) {
            SearchResponse error = new SearchResponse();
            error.error = "Failed to parse response: " + e.getMessage();
            return error;
        }
    }

    /**
     * Check if the response is valid (no error).
     */
    public boolean isValid() {
        return error == null || error.isEmpty();
    }

    /**
     * Check if there's a best line result.
     */
    public boolean hasBestLine() {
        return bestLine != null;
    }

    /**
     * Get display text for overlay.
     */
    public String getDisplayText() {
        if (!isValid()) {
            return "Error: " + error;
        }
        if (!hasBestLine()) {
            return "No result";
        }
        return bestLine.getDisplayText();
    }

    /**
     * Get outcome summary for overlay.
     */
    public String getOutcomeSummary() {
        if (!isValid() || !hasBestLine() || bestLine.expectedOutcome == null) {
            return "";
        }
        return bestLine.expectedOutcome.getSummary();
    }

    /**
     * Get full display with outcome.
     */
    public String getFullDisplay() {
        if (!isValid()) {
            return "Error: " + error;
        }
        if (!hasBestLine()) {
            return "No result";
        }

        String outcome = getOutcomeSummary();
        if (outcome.isEmpty()) {
            return bestLine.getDisplayText();
        }
        return bestLine.getDisplayText() + " = " + outcome;
    }

    /**
     * Get search statistics string.
     */
    public String getStatsText() {
        return String.format("%.0fms, %d nodes", searchTimeMs, nodesExplored);
    }
}
