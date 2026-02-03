package evtracker;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.helpers.Hitbox;
import com.megacrit.cardcrawl.helpers.ImageMaster;
import com.megacrit.cardcrawl.helpers.input.InputHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * "Kill All" button that appears above End Turn when an infinite combo is detected.
 * Uses the same End Turn button texture for consistent look.
 */
public class KillAllButton {
    private static final Logger logger = LogManager.getLogger(KillAllButton.class);

    // Same dimensions as End Turn button: 256x256 texture, drawn centered
    private static final float TEX_SIZE = 256f;
    // Center position: same X as End Turn, offset Y above it
    private static final float CX = 1640f * Settings.xScale;
    private static final float CY = 210f * Settings.yScale + 130f * Settings.scale;

    // Hitbox matches End Turn: 230x110
    private static final float HB_W = 230f * Settings.scale;
    private static final float HB_H = 110f * Settings.scale;
    private static final Hitbox hb = new Hitbox(CX - HB_W / 2f, CY - HB_H / 2f, HB_W, HB_H);

    private static final Color TEXT_COLOR = new Color(1f, 0.3f, 0.3f, 1f);
    private static final Color TEXT_HOVER = new Color(1f, 0.7f, 0.2f, 1f);

    public static boolean shouldShow() {
        return InfiniteDetector.isInfiniteDetected()
            && AbstractDungeon.getCurrRoom() != null
            && AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMBAT
            && AbstractDungeon.getMonsters() != null;
    }

    public static void update() {
        if (!shouldShow()) return;

        hb.update();

        if (hb.hovered && InputHelper.justClickedLeft) {
            InputHelper.justClickedLeft = false;
            killAll();
        }
    }

    public static void render(SpriteBatch sb) {
        if (!shouldShow()) return;

        // Draw button texture (same as End Turn)
        sb.setColor(Color.WHITE);
        Texture btnTex = hb.hovered ? ImageMaster.END_TURN_HOVER : ImageMaster.END_TURN_BUTTON;
        sb.draw(btnTex,
                CX - TEX_SIZE / 2f, CY - TEX_SIZE / 2f,       // position
                TEX_SIZE / 2f, TEX_SIZE / 2f,                   // origin
                TEX_SIZE, TEX_SIZE,                              // size
                Settings.scale, Settings.scale,                  // scale
                0f,                                              // rotation
                0, 0, (int) TEX_SIZE, (int) TEX_SIZE,           // src rect
                false, false);

        // Text
        FontHelper.renderFontCentered(sb, FontHelper.panelEndTurnFont,
                "KILL ALL",
                CX,
                CY - 3f * Settings.scale,
                hb.hovered ? TEXT_HOVER : TEXT_COLOR);

        hb.render(sb);
    }

    private static void killAll() {
        // Disable cannotLose so endBattle fires for multi-phase bosses (Awakened One, Heart, etc.)
        AbstractDungeon.getCurrRoom().cannotLose = false;

        int killCount = 0;
        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            if (!m.isDead && !m.isDying) {
                m.currentHealth = 0;
                m.die();
                killCount++;
            }
        }
        logger.info("[EVTracker] Kill All button: killed " + killCount + " monster(s).");
        InfiniteDetector.clearInfinite();
    }
}
