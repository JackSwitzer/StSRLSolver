package evtracker.search;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.rooms.AbstractRoom;

/**
 * Overlay for displaying search results during combat.
 * Shows best play recommendation and search statistics.
 */
public class SearchOverlay {
    private static final float X_POS = 20f * Settings.scale;
    private static final float Y_POS = Settings.HEIGHT - 180f * Settings.scale;
    private static final float LINE_HEIGHT = 30f * Settings.scale;

    private static final Color BACKGROUND_COLOR = new Color(0, 0, 0, 0.7f);
    private static final Color TITLE_COLOR = new Color(1f, 0.8f, 0.2f, 1f);  // Gold
    private static final Color TEXT_COLOR = Color.WHITE;
    private static final Color GOOD_COLOR = new Color(0.4f, 1f, 0.4f, 1f);  // Green
    private static final Color BAD_COLOR = new Color(1f, 0.4f, 0.4f, 1f);   // Red
    private static final Color SEARCH_COLOR = new Color(0.5f, 0.7f, 1f, 1f); // Blue

    private static boolean enabled = true;
    private static boolean showAlternatives = false;
    private static float flashTimer = 0f;

    /**
     * Toggle overlay visibility.
     */
    public static void toggle() {
        enabled = !enabled;
    }

    /**
     * Check if overlay is enabled.
     */
    public static boolean isEnabled() {
        return enabled;
    }

    /**
     * Set overlay enabled state.
     */
    public static void setEnabled(boolean value) {
        enabled = value;
    }

    /**
     * Toggle showing alternative lines.
     */
    public static void toggleAlternatives() {
        showAlternatives = !showAlternatives;
    }

    /**
     * Update the overlay (called each frame).
     */
    public static void update(float deltaTime) {
        if (flashTimer > 0) {
            flashTimer -= deltaTime;
        }
    }

    /**
     * Update with new search results.
     */
    public static void updateResults(SearchResponse response) {
        // Flash when new result arrives
        flashTimer = 0.5f;
    }

    /**
     * Render the overlay.
     */
    public static void render(SpriteBatch sb) {
        if (!enabled) return;
        if (!isInCombat()) return;

        SearchResponse response = SearchClient.getLatestResponse();
        boolean searching = SearchClient.isSearchInProgress();

        float y = Y_POS;

        // Title
        Color titleColor = flashTimer > 0 ?
            TITLE_COLOR.cpy().lerp(Color.WHITE, flashTimer * 2) :
            TITLE_COLOR;

        FontHelper.renderFontLeftTopAligned(
            sb,
            FontHelper.tipBodyFont,
            "== Combat Search ==",
            X_POS,
            y,
            titleColor
        );
        y -= LINE_HEIGHT;

        // Connection status
        if (!SearchClient.isConnected()) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "Not connected",
                X_POS,
                y,
                BAD_COLOR
            );
            return;
        }

        // Search status
        if (searching) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "Searching...",
                X_POS,
                y,
                SEARCH_COLOR
            );
            return;
        }

        // No results yet
        if (response == null) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "Waiting for turn start",
                X_POS,
                y,
                TEXT_COLOR
            );
            return;
        }

        // Error
        if (!response.isValid()) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "Error: " + truncate(response.error, 40),
                X_POS,
                y,
                BAD_COLOR
            );
            return;
        }

        // No best line
        if (!response.hasBestLine()) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "No recommendation",
                X_POS,
                y,
                TEXT_COLOR
            );
            return;
        }

        // Best play
        String bestPlay = response.getDisplayText();
        Color playColor = getOutcomeColor(response);

        FontHelper.renderFontLeftTopAligned(
            sb,
            FontHelper.tipBodyFont,
            "Best: " + truncate(bestPlay, 50),
            X_POS,
            y,
            playColor
        );
        y -= LINE_HEIGHT;

        // Outcome
        String outcome = response.getOutcomeSummary();
        if (!outcome.isEmpty()) {
            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "-> " + outcome,
                X_POS,
                y,
                playColor
            );
            y -= LINE_HEIGHT;
        }

        // Statistics
        FontHelper.renderFontLeftTopAligned(
            sb,
            FontHelper.tipBodyFont,
            response.getStatsText(),
            X_POS,
            y,
            Color.GRAY
        );
        y -= LINE_HEIGHT;

        // Alternative lines (if enabled)
        if (showAlternatives && response.alternativeLines != null) {
            y -= LINE_HEIGHT * 0.5f;

            FontHelper.renderFontLeftTopAligned(
                sb,
                FontHelper.tipBodyFont,
                "Alternatives:",
                X_POS,
                y,
                Color.LIGHT_GRAY
            );
            y -= LINE_HEIGHT;

            for (int i = 0; i < Math.min(3, response.alternativeLines.size()); i++) {
                SearchResponse.BestLine alt = response.alternativeLines.get(i);
                String altText = alt.getDisplayText();

                FontHelper.renderFontLeftTopAligned(
                    sb,
                    FontHelper.tipBodyFont,
                    "  " + (i + 1) + ". " + truncate(altText, 45),
                    X_POS,
                    y,
                    Color.LIGHT_GRAY
                );
                y -= LINE_HEIGHT;
            }
        }
    }

    /**
     * Get color based on outcome.
     */
    private static Color getOutcomeColor(SearchResponse response) {
        if (!response.hasBestLine() || response.bestLine.expectedOutcome == null) {
            return TEXT_COLOR;
        }

        SearchResponse.ExpectedOutcome outcome = response.bestLine.expectedOutcome;

        if (outcome.enemyKilled) {
            return GOOD_COLOR;
        } else if (outcome.playerDead) {
            return BAD_COLOR;
        } else if (outcome.hpLost == 0) {
            return GOOD_COLOR;
        } else if (outcome.hpLost > 10) {
            return BAD_COLOR;
        }

        return TEXT_COLOR;
    }

    /**
     * Check if we're in combat.
     */
    private static boolean isInCombat() {
        if (AbstractDungeon.getCurrRoom() == null) return false;
        if (AbstractDungeon.getCurrRoom().phase != AbstractRoom.RoomPhase.COMBAT) return false;
        if (AbstractDungeon.actionManager.turnHasEnded) return false;
        return true;
    }

    /**
     * Truncate string with ellipsis.
     */
    private static String truncate(String s, int maxLen) {
        if (s == null) return "";
        if (s.length() <= maxLen) return s;
        return s.substring(0, maxLen - 3) + "...";
    }
}
