package evtracker;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.graphics.g2d.GlyphLayout;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.badlogic.gdx.graphics.glutils.ShapeRenderer;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.helpers.Hitbox;
import com.megacrit.cardcrawl.helpers.input.InputHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.rooms.AbstractRoom;

/**
 * Enhanced debug overlay with INFINITE button and EV display.
 *
 * Features:
 * - "INFINITE" button in top-right corner when infinite detected
 * - Animated text: "INF" -> "INF." -> "INF.." -> "INFINITE"
 * - Click to trigger console kill (end battle immediately)
 * - Optional EV estimate display
 */
public class DebugOverlay {

    // Button position (top-right)
    private static final float BUTTON_WIDTH = 120 * Settings.scale;
    private static final float BUTTON_HEIGHT = 40 * Settings.scale;
    private static final float BUTTON_MARGIN = 20 * Settings.scale;

    // Animation
    private static final float ANIMATION_INTERVAL = 0.3f;
    private static final String[] ANIMATION_FRAMES = {"INF", "INF.", "INF..", "INFINITE"};

    // State
    private static Hitbox buttonHb;
    private static float animationTimer = 0f;
    private static int animationFrame = 0;
    private static boolean wasClicked = false;
    private static GlyphLayout glyphLayout = new GlyphLayout();

    // Colors
    private static final Color BUTTON_BG_COLOR = new Color(0.8f, 0.1f, 0.1f, 0.9f);
    private static final Color BUTTON_HOVER_COLOR = new Color(1.0f, 0.2f, 0.2f, 1.0f);
    private static final Color BUTTON_TEXT_COLOR = Color.WHITE.cpy();

    /**
     * Initialize the overlay (call once at mod init).
     */
    public static void initialize() {
        float x = Settings.WIDTH - BUTTON_WIDTH - BUTTON_MARGIN;
        float y = Settings.HEIGHT - BUTTON_HEIGHT - BUTTON_MARGIN;
        buttonHb = new Hitbox(x, y, BUTTON_WIDTH, BUTTON_HEIGHT);
    }

    /**
     * Reset overlay state (call at combat start).
     */
    public static void reset() {
        animationTimer = 0f;
        animationFrame = 0;
        wasClicked = false;
    }

    /**
     * Update overlay state (call each frame during combat).
     */
    public static void update() {
        if (!InfiniteDetector.isInfiniteDetected()) {
            return;
        }

        // Update animation
        animationTimer += Gdx.graphics.getDeltaTime();
        if (animationTimer >= ANIMATION_INTERVAL) {
            animationTimer = 0f;
            animationFrame = (animationFrame + 1) % ANIMATION_FRAMES.length;
        }

        // Update hitbox
        buttonHb.update();

        // Check for click
        if (buttonHb.hovered && InputHelper.justClickedLeft) {
            wasClicked = true;
        }

        if (wasClicked && InputHelper.justReleasedClickLeft) {
            wasClicked = false;
            if (buttonHb.hovered) {
                triggerConsoleKill();
            }
        }
    }

    /**
     * Render the overlay (call from PostRenderSubscriber).
     */
    public static void render(SpriteBatch sb) {
        if (!shouldRender()) {
            return;
        }

        // Update detector
        InfiniteDetector.update();

        // Update overlay
        update();

        // Render infinite button if detected
        if (InfiniteDetector.isInfiniteDetected()) {
            renderInfiniteButton(sb);
        }

        // Optionally render debug info
        renderDebugInfo(sb);
    }

    private static boolean shouldRender() {
        return AbstractDungeon.getCurrRoom() != null &&
               AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMBAT &&
               AbstractDungeon.player != null &&
               !AbstractDungeon.player.isDead;
    }

    private static void renderInfiniteButton(SpriteBatch sb) {
        // Draw button background
        sb.end();

        ShapeRenderer shapeRenderer = new ShapeRenderer();
        shapeRenderer.setProjectionMatrix(sb.getProjectionMatrix());
        shapeRenderer.begin(ShapeRenderer.ShapeType.Filled);

        Color bgColor = buttonHb.hovered ? BUTTON_HOVER_COLOR : BUTTON_BG_COLOR;
        shapeRenderer.setColor(bgColor);
        shapeRenderer.rect(buttonHb.x, buttonHb.y, buttonHb.width, buttonHb.height);

        shapeRenderer.end();

        // Draw button border
        shapeRenderer.begin(ShapeRenderer.ShapeType.Line);
        shapeRenderer.setColor(Color.WHITE);
        shapeRenderer.rect(buttonHb.x, buttonHb.y, buttonHb.width, buttonHb.height);
        shapeRenderer.end();

        shapeRenderer.dispose();

        sb.begin();

        // Draw button text
        String text = ANIMATION_FRAMES[animationFrame];
        BitmapFont font = FontHelper.buttonLabelFont;

        glyphLayout.setText(font, text);
        float textX = buttonHb.cX - glyphLayout.width / 2f;
        float textY = buttonHb.cY + glyphLayout.height / 2f;

        FontHelper.renderFont(sb, font, text, textX, textY, BUTTON_TEXT_COLOR);

        // Render hitbox for debugging
        buttonHb.render(sb);
    }

    private static void renderDebugInfo(SpriteBatch sb) {
        // Show infinite detection info
        int cardsPlayed = InfiniteDetector.getCardsPlayedCount();

        // Only show if cards have been played this turn or infinite detected
        if (cardsPlayed > 0 || InfiniteDetector.isInfiniteDetected()) {
            float x = Settings.WIDTH - 250 * Settings.scale;
            float y = Settings.HEIGHT - 80 * Settings.scale;
            BitmapFont font = FontHelper.tipBodyFont;

            // Cards played this turn
            Color cardColor = cardsPlayed >= 8 ? Color.YELLOW : Color.WHITE;
            if (cardsPlayed >= 15) cardColor = Color.ORANGE;
            FontHelper.renderFontLeft(sb, font,
                String.format("Cards this turn: %d", cardsPlayed),
                x, y, cardColor);
            y -= 20 * Settings.scale;

            // Infinite status
            if (InfiniteDetector.isInfiniteDetected()) {
                FontHelper.renderFontLeft(sb, font,
                    "INFINITE DETECTED!",
                    x, y, Color.RED);
                y -= 20 * Settings.scale;

                // Show the repeating sequence
                String seq = InfiniteDetector.getDetectedSequence();
                if (seq.length() > 30) {
                    seq = seq.substring(0, 27) + "...";
                }
                FontHelper.renderFontLeft(sb, font,
                    String.format("Loop: %s (x%d)",
                        seq, InfiniteDetector.getDetectedRepetitions()),
                    x, y, Color.ORANGE);
            }
        }
    }

    /**
     * Kill all monsters and end the battle immediately.
     * Used when infinite combo is detected and player clicks the button.
     */
    private static void triggerConsoleKill() {
        System.out.println("[EVTracker] Console kill triggered - ending battle");

        if (AbstractDungeon.getMonsters() == null) {
            return;
        }

        // Kill all monsters
        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            if (!m.isDead) {
                m.currentHealth = 0;
                m.isDying = false;
                m.isDead = true;
            }
        }

        // Clear action queue to prevent any pending actions
        if (AbstractDungeon.actionManager != null) {
            AbstractDungeon.actionManager.actions.clear();
            AbstractDungeon.actionManager.cardQueue.clear();
        }

        // End the battle
        AbstractDungeon.getCurrRoom().endBattle();

        // Clear infinite detection state
        InfiniteDetector.clearInfinite();
    }

    /**
     * Get current EV estimate for display (delegates to DamageCalculator).
     */
    public static float getCurrentEV() {
        DamageCalculator.TurnsToKillResult ttk = DamageCalculator.calculateTurnsToKill();
        return -ttk.expectedDamageTaken;
    }
}
