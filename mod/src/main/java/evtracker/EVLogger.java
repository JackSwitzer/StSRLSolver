package evtracker;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;

import java.io.*;
import java.net.Socket;
import java.text.SimpleDateFormat;
import java.util.*;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.zip.GZIPOutputStream;

/**
 * Logs events to both file and optional socket connection.
 * Events are JSON lines for easy parsing.
 *
 * Includes deduplication to prevent double-logging on save/reload.
 */
public class EVLogger {
    private static final String LOG_DIR = System.getProperty("user.home") +
        "/Desktop/SlayTheSpireRL/logs";
    private static final int SOCKET_PORT = 9999;

    private final Gson gson;
    private PrintWriter fileWriter;
    private Socket socket;
    private PrintWriter socketWriter;
    private final BlockingQueue<String> logQueue;
    private Thread writerThread;
    private volatile boolean running = true;

    // Deduplication: track logged events to prevent double-logging on reload
    private final Set<String> loggedEventKeys = new HashSet<>();
    private Long lastSeenSeed = null;

    public EVLogger() {
        gson = new GsonBuilder().create();
        logQueue = new LinkedBlockingQueue<>();

        initFileWriter();
        tryConnectSocket();
        startWriterThread();
    }

    private void initFileWriter() {
        try {
            File logDir = new File(LOG_DIR);
            if (!logDir.exists()) {
                logDir.mkdirs();
            }

            String timestamp = new SimpleDateFormat("yyyy-MM-dd_HH-mm-ss").format(new Date());
            File logFile = new File(logDir, "evlog_" + timestamp + ".jsonl");

            fileWriter = new PrintWriter(new BufferedWriter(new FileWriter(logFile, true)));
            System.out.println("[EVTracker] Logging to: " + logFile.getAbsolutePath());
        } catch (IOException e) {
            System.err.println("[EVTracker] Failed to init file logger: " + e.getMessage());
        }
    }

    private void tryConnectSocket() {
        try {
            socket = new Socket("localhost", SOCKET_PORT);
            socketWriter = new PrintWriter(socket.getOutputStream(), true);
            System.out.println("[EVTracker] Connected to socket on port " + SOCKET_PORT);
        } catch (IOException e) {
            // Socket server not running, that's OK
            System.out.println("[EVTracker] No socket server on port " + SOCKET_PORT + " (file logging only)");
        }
    }

    private void startWriterThread() {
        writerThread = new Thread(() -> {
            while (running || !logQueue.isEmpty()) {
                try {
                    String line = logQueue.poll();
                    if (line != null) {
                        writeLine(line);
                    } else {
                        Thread.sleep(10);
                    }
                } catch (InterruptedException e) {
                    break;
                }
            }
        });
        writerThread.setDaemon(true);
        writerThread.start();
    }

    private void writeLine(String line) {
        if (fileWriter != null) {
            fileWriter.println(line);
            fileWriter.flush();
        }

        if (socketWriter != null) {
            try {
                socketWriter.println(line);
            } catch (Exception e) {
                // Socket disconnected
                socketWriter = null;
                socket = null;
            }
        }
    }

    public void log(String eventType, Object data) {
        // Check for new run (seed changed) - reset dedup cache
        Long currentSeed = Settings.seed;
        if (currentSeed != null && !currentSeed.equals(lastSeenSeed)) {
            loggedEventKeys.clear();
            lastSeenSeed = currentSeed;
        }

        // Generate dedup key based on event type and context
        String dedupKey = generateDedupKey(eventType, data);
        if (dedupKey != null && loggedEventKeys.contains(dedupKey)) {
            // Already logged this event (reload scenario)
            return;
        }
        if (dedupKey != null) {
            loggedEventKeys.add(dedupKey);
        }

        Map<String, Object> event = new HashMap<>();
        event.put("type", eventType);
        event.put("timestamp", System.currentTimeMillis());
        event.put("data", data);

        String json = gson.toJson(event);
        logQueue.offer(json);
    }

    /**
     * Generate a deduplication key for an event.
     * Returns null for events that shouldn't be deduplicated (system events).
     */
    private String generateDedupKey(String eventType, Object data) {
        // System events don't need dedup
        if ("system".equals(eventType)) {
            return null;
        }

        try {
            StringBuilder key = new StringBuilder();
            key.append(eventType).append(":");

            // Add seed
            if (Settings.seed != null) {
                key.append(Settings.seed).append(":");
            }

            // Add floor
            if (AbstractDungeon.floorNum > 0) {
                key.append("f").append(AbstractDungeon.floorNum).append(":");
            }

            // Add event-specific data for uniqueness
            if (data instanceof Map) {
                Map<?, ?> map = (Map<?, ?>) data;

                // Turn number for combat events
                if (map.containsKey("turn")) {
                    key.append("t").append(map.get("turn")).append(":");
                }

                // Card ID for card events
                if (map.containsKey("card") && map.get("card") instanceof Map) {
                    Map<?, ?> card = (Map<?, ?>) map.get("card");
                    if (card.containsKey("id")) {
                        key.append("c").append(card.get("id")).append(":");
                    }
                    if (card.containsKey("uuid")) {
                        key.append("u").append(card.get("uuid")).append(":");
                    }
                }

                // Battle number
                if (map.containsKey("battle_number")) {
                    key.append("b").append(map.get("battle_number")).append(":");
                }
            }

            return key.toString();
        } catch (Exception e) {
            // If dedup fails, allow the log
            return null;
        }
    }

    public void log(String eventType, String message) {
        Map<String, Object> data = new HashMap<>();
        data.put("message", message);
        log(eventType, data);
    }

    /**
     * Clear dedup cache (call on new run start).
     */
    public void resetDedup() {
        loggedEventKeys.clear();
        lastSeenSeed = null;
    }

    public void close() {
        running = false;
        try {
            if (writerThread != null) {
                writerThread.join(1000);
            }
        } catch (InterruptedException ignored) {}

        if (fileWriter != null) {
            fileWriter.close();
        }

        if (socket != null) {
            try {
                socket.close();
            } catch (IOException ignored) {}
        }
    }
}
