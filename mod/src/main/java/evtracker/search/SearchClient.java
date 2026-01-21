package evtracker.search;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.*;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.*;
import java.util.function.Consumer;

/**
 * Client for communicating with Python combat search server.
 * Manages socket connection and handles async search requests.
 */
public class SearchClient {
    private static final Logger logger = LogManager.getLogger(SearchClient.class);

    private static final String DEFAULT_HOST = "127.0.0.1";
    private static final int DEFAULT_PORT = 9998;
    private static final int CONNECTION_TIMEOUT_MS = 1000;
    private static final int READ_TIMEOUT_MS = 5000;

    private static SearchClient instance;

    private String host;
    private int port;
    private Socket socket;
    private DataInputStream input;
    private DataOutputStream output;
    private boolean connected = false;
    private ExecutorService executor;

    // Latest response for overlay display
    private volatile SearchResponse latestResponse;
    private volatile boolean searchInProgress = false;

    private SearchClient(String host, int port) {
        this.host = host;
        this.port = port;
        this.executor = Executors.newSingleThreadExecutor();
    }

    /**
     * Get the singleton instance.
     */
    public static synchronized SearchClient getInstance() {
        if (instance == null) {
            instance = new SearchClient(DEFAULT_HOST, DEFAULT_PORT);
        }
        return instance;
    }

    /**
     * Check if client is connected to server.
     */
    public static boolean isConnected() {
        return getInstance().connected;
    }

    /**
     * Get latest search response.
     */
    public static SearchResponse getLatestResponse() {
        return getInstance().latestResponse;
    }

    /**
     * Check if a search is currently in progress.
     */
    public static boolean isSearchInProgress() {
        return getInstance().searchInProgress;
    }

    /**
     * Connect to the search server.
     */
    public synchronized boolean connect() {
        if (connected) {
            return true;
        }

        try {
            logger.info("Connecting to search server at {}:{}", host, port);

            socket = new Socket();
            socket.connect(new java.net.InetSocketAddress(host, port), CONNECTION_TIMEOUT_MS);
            socket.setSoTimeout(READ_TIMEOUT_MS);

            input = new DataInputStream(new BufferedInputStream(socket.getInputStream()));
            output = new DataOutputStream(new BufferedOutputStream(socket.getOutputStream()));

            connected = true;
            logger.info("Connected to search server");
            return true;

        } catch (Exception e) {
            logger.warn("Failed to connect to search server: {}", e.getMessage());
            disconnect();
            return false;
        }
    }

    /**
     * Disconnect from the server.
     */
    public synchronized void disconnect() {
        connected = false;

        try {
            if (input != null) input.close();
        } catch (Exception ignored) {}

        try {
            if (output != null) output.close();
        } catch (Exception ignored) {}

        try {
            if (socket != null) socket.close();
        } catch (Exception ignored) {}

        input = null;
        output = null;
        socket = null;
    }

    /**
     * Request a search asynchronously.
     *
     * @param request The search request
     * @param callback Optional callback with response (called on background thread)
     */
    public void requestSearchAsync(SearchRequest request, Consumer<SearchResponse> callback) {
        if (!connected) {
            if (!connect()) {
                // Connection failed, return error response
                SearchResponse error = new SearchResponse();
                error.error = "Not connected to search server";
                latestResponse = error;
                if (callback != null) {
                    callback.accept(error);
                }
                return;
            }
        }

        searchInProgress = true;

        executor.submit(() -> {
            try {
                SearchResponse response = sendRequestBlocking(request);
                latestResponse = response;

                if (callback != null) {
                    callback.accept(response);
                }

            } catch (Exception e) {
                logger.error("Search request failed: {}", e.getMessage());
                SearchResponse error = new SearchResponse();
                error.error = e.getMessage();
                latestResponse = error;

                if (callback != null) {
                    callback.accept(error);
                }

                // Reconnect on error
                disconnect();

            } finally {
                searchInProgress = false;
            }
        });
    }

    /**
     * Send a search request and wait for response.
     *
     * @param request The search request
     * @return The search response
     */
    public SearchResponse sendRequestBlocking(SearchRequest request) throws IOException {
        if (!connected) {
            throw new IOException("Not connected");
        }

        String json = request.toJson();
        byte[] payload = json.getBytes(StandardCharsets.UTF_8);

        // Send length-prefixed message
        synchronized (output) {
            output.writeInt(payload.length);
            output.write(payload);
            output.flush();
        }

        logger.debug("Sent search request: {} bytes", payload.length);

        // Read response
        synchronized (input) {
            int length = input.readInt();

            if (length > 10_000_000) {
                throw new IOException("Response too large: " + length);
            }

            byte[] responseBytes = new byte[length];
            input.readFully(responseBytes);

            String responseJson = new String(responseBytes, StandardCharsets.UTF_8);
            logger.debug("Received response: {} bytes", length);

            return SearchResponse.fromJson(responseJson);
        }
    }

    /**
     * Build a search request from current game state.
     */
    public static SearchRequest buildFromCurrentState() {
        return SearchRequest.fromCurrentState();
    }

    /**
     * Convenience method to request search from current state.
     *
     * @param callback Optional callback with response
     */
    public static void requestSearch(Consumer<SearchResponse> callback) {
        SearchRequest request = buildFromCurrentState();
        // Debug: log hand size
        int handSize = request.card_piles != null ?
            ((java.util.List<?>)request.card_piles.get("hand")).size() : -1;
        int energy = request.player != null ?
            (Integer)request.player.getOrDefault("energy", -1) : -1;
        logger.info("[SearchClient] Requesting search: hand={}, energy={}", handSize, energy);
        getInstance().requestSearchAsync(request, callback);
    }

    /**
     * Clear the latest response.
     */
    public static void clearLatestResponse() {
        getInstance().latestResponse = null;
    }

    /**
     * Shutdown the client.
     */
    public void shutdown() {
        disconnect();
        executor.shutdownNow();
    }
}
