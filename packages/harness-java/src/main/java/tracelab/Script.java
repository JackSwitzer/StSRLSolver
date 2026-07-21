package tracelab;

import com.google.gson.Gson;

import java.io.FileReader;
import java.io.IOException;
import java.util.List;

/**
 * Trace script per docs/goal/TOOLING.md T2. Action vocabulary maps 1:1 onto
 * the Rust engine's RunAction (packages/engine-rs/src/run.rs).
 */
public class Script {
    public int v = 1;
    public String seed;
    public String character = "WATCHER";
    public int ascension = 0;
    public String mode = "script";
    public Stop stop;
    public List<Action> actions;

    public static class Stop {
        public Integer max_floor;
    }

    public static class Action {
        public String type;
        public Integer hand_idx;
        public Integer target;
        public Integer choice;
        public Integer item;
        public Integer idx;
        public String card_id;
        public String choice_name;
    }

    public static Script load(String path) throws IOException {
        try (FileReader reader = new FileReader(path)) {
            Script s = new Gson().fromJson(reader, Script.class);
            if (s == null || s.seed == null || (s.actions == null && !"auto".equals(s.mode))) {
                throw new IOException("tracelab script missing seed or actions: " + path);
            }
            if (s.v != 1) {
                throw new IOException("tracelab script version " + s.v + " unsupported (want 1)");
            }
            return s;
        }
    }
}
