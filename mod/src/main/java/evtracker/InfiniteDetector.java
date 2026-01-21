package evtracker;

import com.evacipated.cardcrawl.modthespire.lib.SpireInsertPatch;
import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.monsters.AbstractMonster;

import java.util.ArrayList;
import java.util.List;

/**
 * Detects infinite combos by monitoring for repeating card sequences.
 *
 * Detection criteria:
 * - Track all cards played this turn
 * - Look for a sequence of 2-5 cards that repeats at least twice
 * - This indicates a loop (infinite combo)
 *
 * Example: Playing [A, B, C, A, B, C] would detect [A, B, C] as a repeating sequence.
 */
public class InfiniteDetector {

    private static final int MIN_SEQUENCE_LENGTH = 2;
    private static final int MAX_SEQUENCE_LENGTH = 6;
    private static final int MIN_REPETITIONS = 2;

    // Track cards played this turn (by card ID)
    private static List<String> cardsPlayedThisTurn = new ArrayList<>();
    private static int lastKnownCardsPlayed = 0;

    // Detection result
    private static boolean infiniteDetected = false;
    private static String detectedSequence = "";
    private static int detectedRepetitions = 0;

    /**
     * Reset detector state at combat start.
     */
    public static void reset() {
        cardsPlayedThisTurn.clear();
        lastKnownCardsPlayed = 0;
        infiniteDetected = false;
        detectedSequence = "";
        detectedRepetitions = 0;
    }

    /**
     * Reset for new turn.
     */
    public static void onTurnStart() {
        cardsPlayedThisTurn.clear();
        lastKnownCardsPlayed = 0;
        // Don't reset infiniteDetected - keep it for the combat summary
    }

    /**
     * Called when a card is played.
     */
    public static void onCardPlayed(AbstractCard card) {
        cardsPlayedThisTurn.add(card.cardID);
        checkForInfinite();
    }

    /**
     * Called each frame to sync with game state and check for sequences.
     */
    public static void update() {
        if (infiniteDetected) {
            return; // Already detected
        }

        // Sync with game's card tracking
        if (AbstractDungeon.actionManager != null &&
            AbstractDungeon.actionManager.cardsPlayedThisTurn != null) {

            int gameCardsPlayed = AbstractDungeon.actionManager.cardsPlayedThisTurn.size();

            // If game has more cards than we've tracked, sync up
            if (gameCardsPlayed > lastKnownCardsPlayed) {
                for (int i = lastKnownCardsPlayed; i < gameCardsPlayed; i++) {
                    AbstractCard card = AbstractDungeon.actionManager.cardsPlayedThisTurn.get(i);
                    if (!cardsPlayedThisTurn.contains(card.cardID) ||
                        cardsPlayedThisTurn.size() < gameCardsPlayed) {
                        cardsPlayedThisTurn.add(card.cardID);
                    }
                }
                lastKnownCardsPlayed = gameCardsPlayed;
                checkForInfinite();
            }
        }
    }

    /**
     * Check if the card sequence contains a repeating pattern.
     */
    private static void checkForInfinite() {
        int n = cardsPlayedThisTurn.size();

        // Need at least 2*MIN_SEQUENCE_LENGTH cards to detect a repeat
        if (n < MIN_SEQUENCE_LENGTH * MIN_REPETITIONS) {
            return;
        }

        // Try different sequence lengths (2 to MAX_SEQUENCE_LENGTH)
        for (int seqLen = MIN_SEQUENCE_LENGTH; seqLen <= MAX_SEQUENCE_LENGTH; seqLen++) {
            // Need at least seqLen * MIN_REPETITIONS cards
            if (n < seqLen * MIN_REPETITIONS) {
                continue;
            }

            // Check if the last seqLen*k cards form a repeating pattern
            int repetitions = countRepetitions(seqLen);

            if (repetitions >= MIN_REPETITIONS) {
                infiniteDetected = true;
                detectedRepetitions = repetitions;

                // Build the sequence string for display
                StringBuilder sb = new StringBuilder();
                int startIdx = n - (seqLen * repetitions);
                for (int i = 0; i < seqLen; i++) {
                    if (i > 0) sb.append(" -> ");
                    sb.append(cardsPlayedThisTurn.get(startIdx + i));
                }
                detectedSequence = sb.toString();

                System.out.println("[EVTracker] INFINITE DETECTED: Sequence [" + detectedSequence +
                    "] repeated " + repetitions + " times");
                return;
            }
        }
    }

    /**
     * Count how many times a sequence of given length repeats at the end of the card list.
     */
    private static int countRepetitions(int seqLen) {
        int n = cardsPlayedThisTurn.size();
        if (n < seqLen * 2) {
            return 0;
        }

        // Get the most recent sequence
        List<String> recentSeq = new ArrayList<>();
        for (int i = n - seqLen; i < n; i++) {
            recentSeq.add(cardsPlayedThisTurn.get(i));
        }

        // Count how many times this sequence appears consecutively from the end
        int repetitions = 1;
        int checkStart = n - seqLen * 2;

        while (checkStart >= 0) {
            boolean matches = true;
            for (int i = 0; i < seqLen; i++) {
                if (!cardsPlayedThisTurn.get(checkStart + i).equals(recentSeq.get(i))) {
                    matches = false;
                    break;
                }
            }

            if (matches) {
                repetitions++;
                checkStart -= seqLen;
            } else {
                break;
            }
        }

        return repetitions;
    }

    /**
     * Check if an infinite combo has been detected.
     */
    public static boolean isInfiniteDetected() {
        return infiniteDetected;
    }

    /**
     * Get the detected repeating sequence (for display).
     */
    public static String getDetectedSequence() {
        return detectedSequence;
    }

    /**
     * Get how many times the sequence repeated.
     */
    public static int getDetectedRepetitions() {
        return detectedRepetitions;
    }

    /**
     * Get total cards played this turn.
     */
    public static int getCardsPlayedCount() {
        return cardsPlayedThisTurn.size();
    }

    /**
     * Clear the infinite flag (e.g., after using the kill button).
     */
    public static void clearInfinite() {
        infiniteDetected = false;
        detectedSequence = "";
        detectedRepetitions = 0;
    }

    // ========== PATCHES ==========

    /**
     * Patch to detect when a card is played.
     */
    @SpirePatch(
        clz = com.megacrit.cardcrawl.cards.AbstractCard.class,
        method = "use",
        paramtypez = {
            com.megacrit.cardcrawl.characters.AbstractPlayer.class,
            com.megacrit.cardcrawl.monsters.AbstractMonster.class
        }
    )
    public static class CardUsePatch {
        @SpireInsertPatch(rloc = 0)
        public static void Insert(AbstractCard __instance,
                                  com.megacrit.cardcrawl.characters.AbstractPlayer p,
                                  AbstractMonster m) {
            InfiniteDetector.onCardPlayed(__instance);
        }
    }
}
