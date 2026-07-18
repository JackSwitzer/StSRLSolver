package tracelab;

import basemod.BaseMod;
import basemod.interfaces.PostInitializeSubscriber;
import basemod.interfaces.PostUpdateSubscriber;
import com.evacipated.cardcrawl.modthespire.lib.SpireInitializer;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.screens.charSelect.CharacterOption;
import com.megacrit.cardcrawl.screens.charSelect.CharacterSelectScreen;
import com.megacrit.cardcrawl.screens.mainMenu.MainMenuScreen;

/**
 * Seeded scripted launch cribbed from PracticeLabMod.applyPendingLaunch
 * (reference/practicelab), generalized to script-driven character/ascension.
 */
@SpireInitializer
public class TraceLabMod implements PostInitializeSubscriber, PostUpdateSubscriber {

    private static Script script;
    private static boolean launchPending = false;
    private static boolean charSelectOpened = false;
    private static int menuSettleFrames = 0;
    private static final int MENU_SETTLE_REQUIRED = 90;

    public TraceLabMod() {
    }

    public static void initialize() {
        BaseMod.subscribe(new TraceLabMod());
    }

    @Override
    public void receivePostInitialize() {
        String scriptPath = System.getProperty("tracelab.script");
        String outPath = System.getProperty("tracelab.out");
        if (scriptPath == null || outPath == null) {
            if (!"0".equals(System.getProperty("tracelab.record", "1"))) {
                Recorder.enable(System.getProperty("tracelab.recorddir", "tracelab-recordings"));
            } else {
                System.out.println("[TraceLab] no script and record-mode off; idle.");
            }
            return;
        }
        try {
            script = Script.load(scriptPath);
            TraceWriter.init(outPath, script);
            ScriptRunner.start(script);
            launchPending = true;
            System.out.println("[TraceLab] loaded script " + scriptPath + " seed=" + script.seed
                    + " actions=" + script.actions.size() + " -> " + outPath);
        } catch (Exception e) {
            System.err.println("[TraceLab] failed to load script: " + e);
            e.printStackTrace();
        }
    }

    @Override
    public void receivePostUpdate() {
        ScriptRunner.update();
        Recorder.update();
    }

    public static void updateMainMenu(MainMenuScreen screen) {
        if (!launchPending || charSelectOpened || CardCrawlGame.isInARun()) {
            return;
        }
        if (menuSettleFrames < MENU_SETTLE_REQUIRED) {
            menuSettleFrames++;
            return;
        }
        charSelectOpened = true;
        System.out.println("[TraceLab] opening character select");
        screen.charSelectScreen.open(false);
    }

    public static void applyPendingLaunch(CharacterSelectScreen screen) {
        if (!launchPending || !charSelectOpened) {
            return;
        }

        AbstractPlayer.PlayerClass wanted;
        try {
            wanted = AbstractPlayer.PlayerClass.valueOf(script.character.toUpperCase());
        } catch (IllegalArgumentException e) {
            System.err.println("[TraceLab] unknown character " + script.character);
            launchPending = false;
            return;
        }

        CharacterOption pick = null;
        for (CharacterOption option : screen.options) {
            if (option.c != null && option.c.chosenClass == wanted && !option.locked) {
                pick = option;
                break;
            }
        }
        if (pick == null) {
            return;
        }

        screen.deselectOtherOptions(pick);
        pick.selected = true;
        screen.justSelected();
        CardCrawlGame.chosenCharacter = wanted;
        screen.confirmButton.isDisabled = false;
        screen.confirmButton.show();

        if (script.ascension > 0) {
            screen.isAscensionMode = true;
            screen.ascensionLevel = script.ascension;
            screen.ascLevelInfoString = CharacterSelectScreen.A_TEXT[script.ascension - 1];
        } else {
            screen.isAscensionMode = false;
            screen.ascensionLevel = 0;
        }

        Settings.seed = parseSeed(script.seed);
        Settings.seedSet = true;
        Settings.specialSeed = null;

        System.out.println("[TraceLab] launching " + wanted + " asc=" + script.ascension
                + " seed=" + script.seed + " (" + Settings.seed + ")");
        screen.confirmButton.hb.clicked = true;
        launchPending = false;
    }

    static long parseSeed(String raw) {
        String trimmed = raw.trim();
        try {
            return Long.parseLong(trimmed);
        } catch (NumberFormatException e) {
            return SeedHelper.getLong(trimmed.toUpperCase());
        }
    }
}
