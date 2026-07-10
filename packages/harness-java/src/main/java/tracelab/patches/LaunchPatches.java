package tracelab.patches;

import com.evacipated.cardcrawl.modthespire.lib.SpirePatch;
import com.megacrit.cardcrawl.screens.charSelect.CharacterSelectScreen;
import com.megacrit.cardcrawl.screens.mainMenu.MainMenuScreen;
import tracelab.TraceLabMod;

public class LaunchPatches {
    @SpirePatch(clz = MainMenuScreen.class, method = "update")
    public static class MainMenuUpdatePatch {
        public static void Postfix(MainMenuScreen __instance) {
            TraceLabMod.updateMainMenu(__instance);
        }
    }

    @SpirePatch(clz = CharacterSelectScreen.class, method = "updateButtons")
    public static class CharacterSelectUpdateButtonsPatch {
        public static void Postfix(CharacterSelectScreen __instance) {
            TraceLabMod.applyPendingLaunch(__instance);
        }
    }
}
