package tracelab;

import com.megacrit.cardcrawl.actions.GameActionManager;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.rooms.AbstractRoom;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Record-mode core (SPEC-tracelab-record-mode.md P2).
 *
 * Patches in RecordPatches call {@link #commit} at the moment the game commits
 * a player decision; this class pairs each pending action with the first
 * stable post-action state and hands the pair to RecordWriter. One frame
 * driver: TraceLabMod.receivePostUpdate() -> update().
 */
public final class Recorder {

    private static final int SETTLE_FRAMES = 3;
    private static final int MAX_PENDING = 8;

    private static boolean enabled = false;
    private static String baseDir;
    private static RecordWriter writer;
    private static final Deque<Map<String, Object>> pending = new ArrayDeque<Map<String, Object>>();
    private static int settleCount = 0;

    /** Set by RecordPatches when CardCrawlGame.loadPlayerSave runs (continue run). */
    public static boolean resumeDetected = false;
    /** Frame counter of the last SaveAndContinue.save call, for SAVE_QUIT vs ABANDON. */
    private static long lastSaveFrame = Long.MIN_VALUE;
    private static long frame = 0;
    /** Set within a frame when a potion `use` was recorded, so the paired
     *  TopPanel.destroyPotion is not double-recorded as a discard. */
    public static int potionUsedSlotThisFrame = -1;

    private Recorder() {
    }

    static void enable(String dir) {
        enabled = true;
        baseDir = dir;
        Runtime.getRuntime().addShutdownHook(new Thread(new Runnable() {
            @Override
            public void run() {
                onShutdown();
            }
        }, "tracelab-recorder-shutdown"));
        System.out.println("[TraceLab] record-mode enabled -> " + dir);
    }

    public static boolean active() {
        return enabled;
    }

    /** Called by patches at decision commit time. kv = alternating key,value. */
    public static void commit(String type, Object... kv) {
        if (!enabled) {
            return;
        }
        Map<String, Object> action = new LinkedHashMap<String, Object>();
        action.put("type", type);
        for (int i = 0; i + 1 < kv.length; i += 2) {
            action.put(String.valueOf(kv[i]), kv[i + 1]);
        }
        pending.addLast(action);
    }

    public static void noteSave() {
        lastSaveFrame = frame;
    }

    static void update() {
        if (!enabled) {
            return;
        }
        frame++;
        potionUsedSlotThisFrame = -1;

        if (writer == null) {
            maybeOpenRun();
            if (writer == null) {
                pending.clear();
                return;
            }
        }

        if (!CardCrawlGame.isInARun()) {
            // Back at the menu with an open recording: save-and-quit if the
            // game saved just before leaving, otherwise the run was abandoned.
            // Death/victory close earlier via their own patches.
            drainAll();
            boolean savedRecently = frame - lastSaveFrame < 600;
            closeRun(savedRecently ? "SAVE_QUIT" : "ABANDON");
            return;
        }

        if (pending.isEmpty()) {
            return;
        }
        if (pending.size() > MAX_PENDING) {
            System.err.println("[TraceLab] recorder pending overflow (" + pending.size()
                    + "), draining with current state");
            drainAll();
            return;
        }
        if (stable()) {
            settleCount++;
            if (settleCount >= SETTLE_FRAMES) {
                writer.writeAction(pending.removeFirst());
                settleCount = 0;
            }
        } else {
            settleCount = 0;
        }
    }

    private static void maybeOpenRun() {
        if (!CardCrawlGame.isInARun() || AbstractDungeon.player == null) {
            return;
        }
        try {
            writer = RecordWriter.open(baseDir, resumeDetected);
            writer.writeLifecycle(resumeDetected ? "RESUME" : "RUN_START");
            resumeDetected = false;
        } catch (Exception e) {
            System.err.println("[TraceLab] failed to open recording, record-mode off: " + e);
            e.printStackTrace();
            enabled = false;
        }
    }

    /** Death/victory patches call this directly so state is still inspectable. */
    public static void closeRunFromPatch(String status) {
        closeRun(status);
    }

    static void closeRun(String status) {
        if (writer == null) {
            return;
        }
        drainAll();
        writer.writeLifecycle("RUN_END", "status", status);
        writer.close(status);
        writer = null;
        System.out.println("[TraceLab] recording closed: " + status);
    }

    private static void drainAll() {
        while (!pending.isEmpty() && writer != null) {
            writer.writeAction(pending.removeFirst());
        }
    }

    private static boolean stable() {
        GameActionManager am = AbstractDungeon.actionManager;
        if (am == null) {
            return true;
        }
        if (!am.actions.isEmpty() || !am.cardQueue.isEmpty()) {
            return false;
        }
        AbstractRoom room = AbstractDungeon.currMapNode != null
                ? AbstractDungeon.currMapNode.getRoom() : null;
        if (room != null && room.phase == AbstractRoom.RoomPhase.COMBAT) {
            return am.phase == GameActionManager.Phase.WAITING_ON_USER;
        }
        return true;
    }

    private static void onShutdown() {
        if (writer != null) {
            drainAll();
            writer.writeLifecycle("RUN_END", "status", "SAVE_QUIT");
            writer.close("SAVE_QUIT");
            writer = null;
        }
    }
}
