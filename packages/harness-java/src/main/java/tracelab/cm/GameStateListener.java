package tracelab.cm;

/**
 * Minimal stub of CommunicationMod's GameStateListener: the vendored
 * ChoiceScreenUtils/patches only signal state-change hints to it, which
 * TraceLab's own ScriptRunner quiescence logic supersedes.
 */
public class GameStateListener {
    public static void blockStateUpdate() {
    }

    public static void resumeStateUpdate() {
    }

    public static void registerStateChange() {
    }
}
