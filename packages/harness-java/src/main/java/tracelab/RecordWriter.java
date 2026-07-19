package tracelab;

import com.badlogic.gdx.Gdx;
import com.google.gson.Gson;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.saveAndContinue.SaveAndContinue;

import java.io.File;
import java.io.FileOutputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.zip.GZIPOutputStream;

/**
 * Artifact writer for record-mode (SPEC-tracelab-record-mode.md).
 *
 * Per run directory under the recordings root:
 *   meta.json       — run identity, sittings, status, record count
 *   script.jsonl    — one action per line, in play order
 *   trace.jsonl.gz  — one full-state record per action (gzip, per-line flush;
 *                     resume appends a new gzip member, which zcat handles)
 */
final class RecordWriter {

    private static final Gson GSON = new Gson();
    private static final SimpleDateFormat TS = new SimpleDateFormat("yyyyMMdd-HHmmss");

    private final File dir;
    private final Map<String, Object> meta;
    private final PrintWriter scriptOut;
    private final GZIPOutputStream traceGzip;
    private final Writer traceOut;
    private int idx;

    private RecordWriter(File dir, Map<String, Object> meta, boolean append) throws IOException {
        this.dir = dir;
        this.meta = meta;
        this.idx = meta.get("records") instanceof Number
                ? ((Number) meta.get("records")).intValue() : 0;
        this.scriptOut = new PrintWriter(new FileWriter(new File(dir, "script.jsonl"), append));
        this.traceGzip = new GZIPOutputStream(
                new FileOutputStream(new File(dir, "trace.jsonl.gz"), append), 8192, true);
        this.traceOut = new OutputStreamWriter(traceGzip, StandardCharsets.UTF_8);
    }

    static RecordWriter open(String baseDir, boolean resume) throws IOException {
        long seedLong = Settings.seed != null ? Settings.seed : 0L;
        String character = AbstractDungeon.player.chosenClass.name();
        File root = new File(baseDir);
        if (!root.isDirectory() && !root.mkdirs()) {
            throw new IOException("cannot create recordings root " + root);
        }

        if (resume) {
            RecordWriter reopened = findReattachable(root, seedLong, character);
            if (reopened != null) {
                reopened.addSitting();
                return reopened;
            }
            System.err.println("[TraceLab] resume detected but no in-progress/save-quit recording for seed "
                    + seedLong + "; starting fresh artifacts");
        }

        File dir = new File(root, seedLong + "-" + character + "-" + TS.format(new Date()));
        if (!dir.mkdirs()) {
            throw new IOException("cannot create recording dir " + dir);
        }
        Map<String, Object> meta = new LinkedHashMap<String, Object>();
        meta.put("v", 1);
        meta.put("run_id", dir.getName());
        meta.put("seed_long", seedLong);
        meta.put("seed_display", SeedHelper.getString(seedLong));
        meta.put("character", character);
        meta.put("ascension", AbstractDungeon.ascensionLevel);
        meta.put("game_version", CardCrawlGame.TRUE_VERSION_NUM);
        meta.put("status", "in_progress");
        meta.put("records", 0);
        List<String> sittings = new ArrayList<String>();
        sittings.add(TS.format(new Date()));
        meta.put("sittings", sittings);

        RecordWriter writer = new RecordWriter(dir, meta, false);
        writer.flushMeta();
        System.out.println("[TraceLab] recording to " + dir);
        return writer;
    }

    // Fix 3(b): reattach on resume. "in_progress" is the crash-recovery case
    // (the previous sitting never got a chance to close the recording at
    // all); "SAVE_QUIT" is the normal case — the player saved-and-quit, and
    // CardCrawlGame.loadPlayerSave (decompiled CardCrawlGame.java:855) fires
    // when they hit Continue, so the SAME run directory must be reopened
    // rather than split into a second one. When multiple SAVE_QUIT
    // directories match the same seed+character (e.g. the same seed replayed
    // across sessions), the most recent one wins — directory names embed a
    // yyyyMMdd-HHmmss suffix (see `open`), so lexical order is chronological
    // order. An in_progress match always wins over a SAVE_QUIT match: it
    // means a prior sitting is still (spuriously) open and is the more
    // authoritative target to reattach to.
    @SuppressWarnings("unchecked")
    private static RecordWriter findReattachable(File root, long seedLong, String character) throws IOException {
        File[] candidates = root.listFiles();
        if (candidates == null) {
            return null;
        }
        File bestDir = null;
        Map<String, Object> bestMeta = null;
        boolean bestInProgress = false;
        for (File dir : candidates) {
            File metaFile = new File(dir, "meta.json");
            if (!metaFile.isFile()) {
                continue;
            }
            try {
                String raw = new String(Files.readAllBytes(metaFile.toPath()), StandardCharsets.UTF_8);
                Map<String, Object> meta = GSON.fromJson(raw, LinkedHashMap.class);
                String status = (String) meta.get("status");
                boolean inProgress = "in_progress".equals(status);
                boolean reattachable = inProgress || "SAVE_QUIT".equals(status);
                boolean match = reattachable
                        && meta.get("seed_long") instanceof Number
                        && ((Number) meta.get("seed_long")).longValue() == seedLong
                        && character.equals(meta.get("character"));
                if (!match) {
                    continue;
                }
                boolean better = bestDir == null
                        || (inProgress && !bestInProgress)
                        || (inProgress == bestInProgress && dir.getName().compareTo(bestDir.getName()) > 0);
                if (better) {
                    bestDir = dir;
                    bestMeta = meta;
                    bestInProgress = inProgress;
                }
            } catch (Exception e) {
                System.err.println("[TraceLab] skipping unreadable recording meta " + metaFile + ": " + e);
            }
        }
        if (bestDir == null) {
            return null;
        }
        RecordWriter writer = new RecordWriter(bestDir, bestMeta, true);
        if (!bestInProgress) {
            // Was SAVE_QUIT: this sitting resumes it, so it's open again.
            writer.meta.put("status", "in_progress");
        }
        System.out.println("[TraceLab] resuming recording " + bestDir
                + " (was " + (bestInProgress ? "in_progress" : "SAVE_QUIT") + ")");
        return writer;
    }

    /** Fix 3(a): whether a usable save exists for this run's character —
     * used to classify a return-to-menu as SAVE_QUIT vs ABANDON. Mirrors the
     * file-existence half of SaveAndContinue.saveExistsAndNotCorrupted
     * (decompiled SaveAndContinue.java:51-68), the same signal the game's
     * own main menu uses to decide whether "Continue" is available. The
     * corruption-recovery half of that method needs a live AbstractPlayer
     * and, on a corrupt file, falls into loadSaveFile(PlayerClass), which
     * calls Gdx.app.exit() on failure — not safe to invoke from an observer
     * patch that must never affect game flow, so we resolve the save path
     * from the run's recorded character instead of a live player instance
     * (AbstractDungeon.player is already null by this point — MainMenuScreen
     * nulls it in its constructor, decompiled MainMenuScreen.java:116).
     */
    boolean hasUsableSave() {
        Object character = meta.get("character");
        if (!(character instanceof String)) {
            return false;
        }
        try {
            AbstractPlayer.PlayerClass cls = AbstractPlayer.PlayerClass.valueOf((String) character);
            return Gdx.files.local(SaveAndContinue.getPlayerSavePath(cls)).exists();
        } catch (IllegalArgumentException e) {
            return false;
        }
    }

    void writeAction(Map<String, Object> action) {
        Map<String, Object> scriptLine = new LinkedHashMap<String, Object>();
        scriptLine.put("idx", idx);
        scriptLine.putAll(action);
        scriptOut.println(GSON.toJson(scriptLine));
        scriptOut.flush();

        Map<String, Object> rec = new LinkedHashMap<String, Object>();
        rec.put("v", 1);
        rec.put("idx", idx);
        rec.put("floor", AbstractDungeon.floorNum);
        rec.put("act", AbstractDungeon.actNum);
        rec.put("turn", AbstractDungeon.actionManager != null ? AbstractDungeon.actionManager.turn : 0);
        rec.put("phase", TraceWriter.currentPhase());
        rec.put("screen", AbstractDungeon.screen != null ? AbstractDungeon.screen.name() : "NONE");
        if (AbstractDungeon.currMapNode != null) {
            Map<String, Object> node = new LinkedHashMap<String, Object>();
            node.put("x", AbstractDungeon.currMapNode.x);
            node.put("y", AbstractDungeon.currMapNode.y);
            rec.put("map", node);
        }
        rec.put("action", action);
        rec.put("post", TraceWriter.postState());
        rec.put("deck", TraceWriter.cardIds(AbstractDungeon.player.masterDeck.group));
        writeTraceLine(rec);

        idx++;
        meta.put("records", idx);
        flushMeta();
    }

    void writeLifecycle(String type, Object... kv) {
        Map<String, Object> rec = new LinkedHashMap<String, Object>();
        rec.put("v", 1);
        rec.put("kind", "lifecycle");
        rec.put("type", type);
        for (int i = 0; i + 1 < kv.length; i += 2) {
            rec.put(String.valueOf(kv[i]), kv[i + 1]);
        }
        rec.put("floor", AbstractDungeon.floorNum);
        writeTraceLine(rec);

        Map<String, Object> scriptLine = new LinkedHashMap<String, Object>();
        scriptLine.put("lifecycle", type);
        scriptOut.println(GSON.toJson(scriptLine));
        scriptOut.flush();
    }

    void close(String status) {
        meta.put("status", "in_progress".equals(status) ? "in_progress" : status);
        flushMeta();
        try {
            traceOut.flush();
            traceGzip.finish();
            traceOut.close();
        } catch (IOException e) {
            System.err.println("[TraceLab] trace close failed: " + e);
        }
        scriptOut.close();
    }

    private void addSitting() {
        Object sittings = meta.get("sittings");
        if (sittings instanceof List) {
            ((List<String>) sittings).add(TS.format(new Date()));
        }
        flushMeta();
    }

    private void writeTraceLine(Map<String, Object> rec) {
        try {
            traceOut.write(GSON.toJson(rec));
            traceOut.write('\n');
            traceOut.flush();
        } catch (IOException e) {
            System.err.println("[TraceLab] trace write failed: " + e);
        }
    }

    private void flushMeta() {
        try {
            PrintWriter out = new PrintWriter(new FileWriter(new File(dir, "meta.json"), false));
            out.println(GSON.toJson(meta));
            out.close();
        } catch (IOException e) {
            System.err.println("[TraceLab] meta write failed: " + e);
        }
    }
}
