package evtracker;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.helpers.ImageMaster;
import com.megacrit.cardcrawl.helpers.input.InputHelper;

import java.util.List;

/**
 * Renders a post-combat review panel showing:
 * - Damage dealt/taken
 * - Relics used (combat proc vs passive)
 * - Potions used
 * - Scaling events
 * - Optimality score
 * - Key mistakes
 */
public class PostCombatReviewUI {

    private static boolean showReview = false;
    private static CombatReview.CombatSummary currentSummary = null;
    private static float alpha = 0f;
    private static final float FADE_SPEED = 3f;

    // Panel dimensions
    private static final float PANEL_WIDTH = 450f;
    private static final float PANEL_HEIGHT = 500f;

    /**
     * Show the review panel for the given combat summary
     */
    public static void show(CombatReview.CombatSummary summary) {
        currentSummary = summary;
        showReview = true;
        alpha = 0f;
    }

    /**
     * Hide the review panel
     */
    public static void hide() {
        showReview = false;
    }

    /**
     * Check if review is currently showing
     */
    public static boolean isShowing() {
        return showReview && alpha > 0.1f;
    }

    /**
     * Update (handle fade in/out)
     */
    public static void update() {
        if (showReview && alpha < 1f) {
            alpha = Math.min(1f, alpha + com.badlogic.gdx.Gdx.graphics.getDeltaTime() * FADE_SPEED);
        } else if (!showReview && alpha > 0f) {
            alpha = Math.max(0f, alpha - com.badlogic.gdx.Gdx.graphics.getDeltaTime() * FADE_SPEED);
        }

        // Click to dismiss
        if (showReview && InputHelper.justClickedLeft) {
            hide();
        }
    }

    /**
     * Render the review panel
     */
    public static void render(SpriteBatch sb) {
        if (currentSummary == null || alpha <= 0.01f) return;

        float panelW = PANEL_WIDTH * Settings.scale;
        float panelH = PANEL_HEIGHT * Settings.scale;
        float x = (Settings.WIDTH - panelW) / 2f;
        float y = (Settings.HEIGHT - panelH) / 2f;

        // Background panel
        sb.setColor(0f, 0f, 0f, 0.9f * alpha);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x, y, panelW, panelH);

        // Border
        sb.setColor(0.4f, 0.4f, 0.6f, alpha);
        float borderWidth = 3 * Settings.scale;
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x, y, panelW, borderWidth);  // Bottom
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x, y + panelH - borderWidth, panelW, borderWidth);  // Top
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x, y, borderWidth, panelH);  // Left
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x + panelW - borderWidth, y, borderWidth, panelH);  // Right

        // Content
        float textX = x + 20 * Settings.scale;
        float textY = y + panelH - 30 * Settings.scale;
        float lineHeight = 22 * Settings.scale;

        Color textColor = new Color(1f, 1f, 1f, alpha);
        Color headerColor = new Color(1f, 0.85f, 0.3f, alpha);
        Color goodColor = new Color(0.4f, 1f, 0.4f, alpha);
        Color badColor = new Color(1f, 0.4f, 0.4f, alpha);
        Color neutralColor = new Color(0.7f, 0.7f, 0.9f, alpha);

        // Title
        FontHelper.renderFontLeft(sb, FontHelper.tipHeaderFont,
            "=== COMBAT REVIEW ===", textX, textY, headerColor);
        textY -= lineHeight * 1.5f;

        // Encounter info
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            String.format("Floor %d: %s", currentSummary.floor, currentSummary.encounter),
            textX, textY, textColor);
        textY -= lineHeight;

        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            String.format("Turns: %d | Decisions: %d", currentSummary.totalTurns, currentSummary.decisionsCount),
            textX, textY, textColor);
        textY -= lineHeight * 1.3f;

        // Damage section
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            "--- DAMAGE ---", textX, textY, headerColor);
        textY -= lineHeight;

        Color dmgColor = currentSummary.wasInfinite ? goodColor : textColor;
        String dmgText = currentSummary.wasInfinite ?
            String.format("Dealt: %d (INFINITE)", currentSummary.totalDamageDealt) :
            String.format("Dealt: %d", currentSummary.totalDamageDealt);
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont, dmgText, textX, textY, dmgColor);
        textY -= lineHeight;

        Color hpColor = currentSummary.hpLost <= 0 ? goodColor :
                       (currentSummary.hpLost <= 5 ? neutralColor : badColor);
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            String.format("HP Lost: %d (optimal est: %d)", currentSummary.hpLost, currentSummary.optimalHpLost),
            textX, textY, hpColor);
        textY -= lineHeight;

        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            String.format("Block Generated: %d", currentSummary.blockGenerated),
            textX, textY, textColor);
        textY -= lineHeight * 1.3f;

        // Relic section
        if (!currentSummary.combatProcRelics.isEmpty() || !currentSummary.passiveRelics.isEmpty()) {
            FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                "--- RELICS ---", textX, textY, headerColor);
            textY -= lineHeight;

            if (!currentSummary.combatProcRelics.isEmpty()) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    "Proc: " + String.join(", ", currentSummary.combatProcRelics),
                    textX, textY, neutralColor);
                textY -= lineHeight;
            }

            if (!currentSummary.passiveRelics.isEmpty()) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    "Passive: " + String.join(", ", currentSummary.passiveRelics),
                    textX, textY, neutralColor);
                textY -= lineHeight;
            }
            textY -= lineHeight * 0.3f;
        }

        // Potion section
        if (!currentSummary.potionsUsed.isEmpty()) {
            FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                "--- POTIONS USED ---", textX, textY, headerColor);
            textY -= lineHeight;

            for (CombatReview.PotionUsage pu : currentSummary.potionsUsed) {
                String potionText = String.format("T%d: %s (%s)",
                    pu.turn, pu.potionName, pu.context);
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont, potionText, textX, textY, neutralColor);
                textY -= lineHeight;
            }
            textY -= lineHeight * 0.3f;
        }

        // Scaling section
        if (!currentSummary.scalingEvents.isEmpty()) {
            FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                "--- SCALING ---", textX, textY, headerColor);
            textY -= lineHeight;

            if (currentSummary.permanentDamageGained > 0) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    String.format("+%d permanent damage", currentSummary.permanentDamageGained),
                    textX, textY, goodColor);
                textY -= lineHeight;
            }
            if (currentSummary.permanentHPGained > 0) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    String.format("+%d max HP", currentSummary.permanentHPGained),
                    textX, textY, goodColor);
                textY -= lineHeight;
            }
            if (currentSummary.goldGained > 0) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    String.format("+%d gold", currentSummary.goldGained),
                    textX, textY, goodColor);
                textY -= lineHeight;
            }
            textY -= lineHeight * 0.3f;
        }

        // Optimality section
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            "--- OPTIMALITY ---", textX, textY, headerColor);
        textY -= lineHeight;

        float score = currentSummary.getOptimalityScore();
        Color scoreColor = score >= 80 ? goodColor : (score >= 50 ? neutralColor : badColor);
        FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
            String.format("Score: %.0f%% (%d/%d optimal)",
                score, currentSummary.optimalDecisions, currentSummary.decisionsCount),
            textX, textY, scoreColor);
        textY -= lineHeight;

        if (currentSummary.totalEVLost < -1) {
            FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                String.format("EV Lost: %.1f", currentSummary.totalEVLost),
                textX, textY, badColor);
            textY -= lineHeight;
        }
        textY -= lineHeight * 0.3f;

        // Key mistakes (limit to 3)
        if (!currentSummary.keyMistakes.isEmpty()) {
            FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                "--- KEY MISTAKES ---", textX, textY, headerColor);
            textY -= lineHeight;

            List<String> mistakes = currentSummary.keyMistakes;
            int shown = Math.min(3, mistakes.size());
            for (int i = 0; i < shown; i++) {
                // Truncate long mistake descriptions
                String mistake = mistakes.get(i);
                if (mistake.length() > 45) {
                    mistake = mistake.substring(0, 42) + "...";
                }
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont, mistake, textX, textY, badColor);
                textY -= lineHeight;
            }
            if (mistakes.size() > 3) {
                FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont,
                    String.format("...and %d more", mistakes.size() - 3),
                    textX, textY, neutralColor);
            }
        }

        // Dismiss hint
        float hintY = y + 15 * Settings.scale;
        FontHelper.renderFontCentered(sb, FontHelper.tipBodyFont,
            "(Click to dismiss)", x + panelW / 2, hintY, new Color(0.5f, 0.5f, 0.5f, alpha));
    }
}
