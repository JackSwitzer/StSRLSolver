package evtracker;

import basemod.BaseMod;
import basemod.devcommands.ConsoleCommand;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.saveAndContinue.SaveFile;

import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;

/**
 * Console commands for EVTracker mod.
 *
 * Commands:
 * - evseed <SEED> : Start new run with specific seed
 * - evresetrun    : Restart current run with same seed
 * - evfloor <N>   : Jump to floor (experimental)
 * - evkillall     : Kill all monsters immediately
 * - evdumprng     : Output current RNG state for Python sync
 */
public class ConsoleCommands {

    private static final String RNG_DUMP_PATH = "/tmp/evtracker_rng.json";

    /**
     * Register all EVTracker console commands.
     * Call this from receivePostInitialize().
     */
    public static void registerCommands() {
        ConsoleCommand.addCommand("evseed", SeedCommand.class);
        ConsoleCommand.addCommand("evresetrun", ResetRunCommand.class);
        ConsoleCommand.addCommand("evfloor", FloorCommand.class);
        ConsoleCommand.addCommand("evkillall", KillAllCommand.class);
        ConsoleCommand.addCommand("evdumprng", DumpRngCommand.class);

        System.out.println("[EVTracker] Console commands registered: evseed, evresetrun, evfloor, evkillall, evdumprng");
    }

    // ========== SEED COMMAND ==========

    /**
     * Start a new run with a specific seed.
     * Usage: evseed <SEED_STRING>
     *
     * Example: evseed 12345ABCDE
     */
    public static class SeedCommand extends ConsoleCommand {

        public SeedCommand() {
            this.minExtraTokens = 1;
            this.maxExtraTokens = 1;
            this.requiresPlayer = false;
            this.simpleCheck = false;
        }

        @Override
        protected void execute(String[] tokens, int depth) {
            if (tokens.length <= depth) {
                errorMsg();
                return;
            }

            String seedString = tokens[depth].toUpperCase();

            try {
                long seed = SeedHelper.getLong(seedString);

                // Set the seed
                Settings.seed = seed;
                Settings.seedSet = true;
                Settings.specialSeed = null;
                Settings.isDailyRun = false;
                SeedHelper.cachedSeed = null;

                // Regenerate all RNG streams
                AbstractDungeon.generateSeeds();

                System.out.println("[EVTracker] Seed set to: " + seedString + " (long: " + seed + ")");
                System.out.println("[EVTracker] Start a new run to use this seed.");

            } catch (Exception e) {
                System.err.println("[EVTracker] Invalid seed format: " + seedString);
                errorMsg();
            }
        }

        @Override
        protected void errorMsg() {
            System.out.println("[EVTracker] Usage: evseed <SEED_STRING>");
            System.out.println("[EVTracker] Example: evseed 12345ABCDE");
        }

        @Override
        public ArrayList<String> extraOptions(String[] tokens, int depth) {
            ArrayList<String> options = new ArrayList<>();
            options.add("<SEED>");
            return options;
        }
    }

    // ========== RESET RUN COMMAND ==========

    /**
     * Restart the current run with the same seed.
     * Usage: evresetrun
     */
    public static class ResetRunCommand extends ConsoleCommand {

        public ResetRunCommand() {
            this.minExtraTokens = 0;
            this.maxExtraTokens = 0;
            this.requiresPlayer = true;
            this.simpleCheck = true;
        }

        @Override
        protected void execute(String[] tokens, int depth) {
            if (Settings.seed == null) {
                System.err.println("[EVTracker] No seed set. Cannot reset run.");
                return;
            }

            long currentSeed = Settings.seed;
            String seedString = SeedHelper.getString(currentSeed);

            System.out.println("[EVTracker] Resetting run with seed: " + seedString);

            // Store current character and ascension
            AbstractPlayer.PlayerClass playerClass = AbstractDungeon.player.chosenClass;
            int ascension = AbstractDungeon.ascensionLevel;

            // Ensure seed is set for new game
            Settings.seedSet = true;

            // Regenerate seeds
            AbstractDungeon.generateSeeds();

            // Force return to main menu and start new game
            // Note: This approach depends on game state - may need adjustment
            try {
                CardCrawlGame.startOver();
            } catch (Exception e) {
                System.err.println("[EVTracker] Error resetting run: " + e.getMessage());
                System.out.println("[EVTracker] Seed preserved. Please start a new run manually.");
            }
        }

        @Override
        protected void errorMsg() {
            System.out.println("[EVTracker] Usage: evresetrun");
            System.out.println("[EVTracker] Restarts the current run with the same seed.");
        }
    }

    // ========== FLOOR COMMAND ==========

    /**
     * Jump to a specific floor (experimental).
     * Usage: evfloor <N>
     *
     * Note: This is experimental and may cause issues with game state.
     * Works best for simple floor jumps within the same act.
     */
    public static class FloorCommand extends ConsoleCommand {

        public FloorCommand() {
            this.minExtraTokens = 1;
            this.maxExtraTokens = 1;
            this.requiresPlayer = true;
            this.simpleCheck = false;
        }

        @Override
        protected void execute(String[] tokens, int depth) {
            if (tokens.length <= depth) {
                errorMsg();
                return;
            }

            try {
                int targetFloor = Integer.parseInt(tokens[depth]);

                if (targetFloor < 0 || targetFloor > 60) {
                    System.err.println("[EVTracker] Floor must be between 0 and 60");
                    return;
                }

                int currentFloor = AbstractDungeon.floorNum;

                System.out.println("[EVTracker] Attempting to jump from floor " + currentFloor + " to floor " + targetFloor);
                System.out.println("[EVTracker] WARNING: Floor jumping is experimental and may cause issues.");

                // Set the floor number
                AbstractDungeon.floorNum = targetFloor;

                // Update act based on floor
                if (targetFloor <= 17) {
                    AbstractDungeon.actNum = 1;
                } else if (targetFloor <= 34) {
                    AbstractDungeon.actNum = 2;
                } else if (targetFloor <= 52) {
                    AbstractDungeon.actNum = 3;
                } else {
                    AbstractDungeon.actNum = 4;
                }

                System.out.println("[EVTracker] Floor set to: " + targetFloor + " (Act " + AbstractDungeon.actNum + ")");
                System.out.println("[EVTracker] Note: You may need to proceed to the next node for changes to take effect.");

            } catch (NumberFormatException e) {
                System.err.println("[EVTracker] Invalid floor number: " + tokens[depth]);
                errorMsg();
            }
        }

        @Override
        protected void errorMsg() {
            System.out.println("[EVTracker] Usage: evfloor <N>");
            System.out.println("[EVTracker] Example: evfloor 15");
            System.out.println("[EVTracker] Note: Experimental feature - use with caution.");
        }

        @Override
        public ArrayList<String> extraOptions(String[] tokens, int depth) {
            ArrayList<String> options = new ArrayList<>();
            options.add("<FLOOR_NUMBER>");
            return options;
        }
    }

    // ========== KILL ALL COMMAND ==========

    /**
     * Kill all monsters immediately.
     * Usage: evkillall
     *
     * Useful for skipping combat during testing or ending infinite combos.
     */
    public static class KillAllCommand extends ConsoleCommand {

        public KillAllCommand() {
            this.minExtraTokens = 0;
            this.maxExtraTokens = 0;
            this.requiresPlayer = true;
            this.simpleCheck = true;
        }

        @Override
        protected void execute(String[] tokens, int depth) {
            if (AbstractDungeon.getCurrRoom() == null) {
                System.err.println("[EVTracker] Not in a room.");
                return;
            }

            if (AbstractDungeon.getCurrRoom().phase != AbstractRoom.RoomPhase.COMBAT) {
                System.err.println("[EVTracker] Not in combat.");
                return;
            }

            if (AbstractDungeon.getMonsters() == null) {
                System.err.println("[EVTracker] No monsters present.");
                return;
            }

            int killCount = 0;
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    m.currentHealth = 0;
                    m.die();
                    killCount++;
                }
            }

            System.out.println("[EVTracker] Killed " + killCount + " monster(s).");
        }

        @Override
        protected void errorMsg() {
            System.out.println("[EVTracker] Usage: evkillall");
            System.out.println("[EVTracker] Kills all monsters in the current combat.");
        }
    }

    // ========== DUMP RNG COMMAND ==========

    /**
     * Dump current RNG state for Python sync.
     * Usage: evdumprng
     *
     * Outputs all RNG stream counters to /tmp/evtracker_rng.json
     */
    public static class DumpRngCommand extends ConsoleCommand {

        public DumpRngCommand() {
            this.minExtraTokens = 0;
            this.maxExtraTokens = 0;
            this.requiresPlayer = false;
            this.simpleCheck = true;
        }

        @Override
        protected void execute(String[] tokens, int depth) {
            Map<String, Object> rngState = getRngState();

            if (rngState.isEmpty()) {
                System.err.println("[EVTracker] RNG state not available (not in dungeon).");
                return;
            }

            // Write to file
            try (PrintWriter writer = new PrintWriter(new FileWriter(RNG_DUMP_PATH))) {
                com.google.gson.Gson gson = new com.google.gson.GsonBuilder()
                    .setPrettyPrinting()
                    .create();
                writer.println(gson.toJson(rngState));
                System.out.println("[EVTracker] RNG state dumped to: " + RNG_DUMP_PATH);
            } catch (IOException e) {
                System.err.println("[EVTracker] Failed to write RNG state: " + e.getMessage());
            }

            // Also print to console
            System.out.println("[EVTracker] RNG State:");
            System.out.println("  Seed: " + (Settings.seed != null ? SeedHelper.getString(Settings.seed) : "null"));
            System.out.println("  Seed (long): " + Settings.seed);

            if (AbstractDungeon.cardRng != null) {
                System.out.println("  cardRng counter: " + AbstractDungeon.cardRng.counter);
            }
            if (AbstractDungeon.monsterRng != null) {
                System.out.println("  monsterRng counter: " + AbstractDungeon.monsterRng.counter);
            }
            if (AbstractDungeon.relicRng != null) {
                System.out.println("  relicRng counter: " + AbstractDungeon.relicRng.counter);
            }
            if (AbstractDungeon.eventRng != null) {
                System.out.println("  eventRng counter: " + AbstractDungeon.eventRng.counter);
            }
            if (AbstractDungeon.potionRng != null) {
                System.out.println("  potionRng counter: " + AbstractDungeon.potionRng.counter);
            }
        }

        @Override
        protected void errorMsg() {
            System.out.println("[EVTracker] Usage: evdumprng");
            System.out.println("[EVTracker] Dumps RNG state to " + RNG_DUMP_PATH);
        }
    }

    // ========== HELPER METHODS ==========

    /**
     * Get current RNG state as a map for JSON serialization.
     */
    public static Map<String, Object> getRngState() {
        Map<String, Object> state = new HashMap<>();

        // Seed info
        if (Settings.seed != null) {
            state.put("seed", Settings.seed);
            state.put("seed_string", SeedHelper.getString(Settings.seed));
            state.put("seed_set", Settings.seedSet);
        }

        // Floor info
        state.put("floor_num", AbstractDungeon.floorNum);
        state.put("act_num", AbstractDungeon.actNum);

        // RNG counters (all 13 streams)
        Map<String, Integer> counters = new HashMap<>();

        if (AbstractDungeon.cardRng != null) {
            counters.put("cardRng", AbstractDungeon.cardRng.counter);
        }
        if (AbstractDungeon.monsterRng != null) {
            counters.put("monsterRng", AbstractDungeon.monsterRng.counter);
        }
        if (AbstractDungeon.eventRng != null) {
            counters.put("eventRng", AbstractDungeon.eventRng.counter);
        }
        if (AbstractDungeon.relicRng != null) {
            counters.put("relicRng", AbstractDungeon.relicRng.counter);
        }
        if (AbstractDungeon.treasureRng != null) {
            counters.put("treasureRng", AbstractDungeon.treasureRng.counter);
        }
        if (AbstractDungeon.potionRng != null) {
            counters.put("potionRng", AbstractDungeon.potionRng.counter);
        }
        if (AbstractDungeon.merchantRng != null) {
            counters.put("merchantRng", AbstractDungeon.merchantRng.counter);
        }
        if (AbstractDungeon.monsterHpRng != null) {
            counters.put("monsterHpRng", AbstractDungeon.monsterHpRng.counter);
        }
        if (AbstractDungeon.aiRng != null) {
            counters.put("aiRng", AbstractDungeon.aiRng.counter);
        }
        if (AbstractDungeon.shuffleRng != null) {
            counters.put("shuffleRng", AbstractDungeon.shuffleRng.counter);
        }
        if (AbstractDungeon.cardRandomRng != null) {
            counters.put("cardRandomRng", AbstractDungeon.cardRandomRng.counter);
        }
        if (AbstractDungeon.miscRng != null) {
            counters.put("miscRng", AbstractDungeon.miscRng.counter);
        }

        state.put("rng_counters", counters);
        state.put("timestamp", System.currentTimeMillis());

        return state;
    }
}
