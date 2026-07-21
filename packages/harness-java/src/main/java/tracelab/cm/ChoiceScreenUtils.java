// Vendored from CommunicationMod (MIT, github.com/ForgottenArbiter/CommunicationMod),
// decompiled via CFR 0.152 from Steam workshop item 2131373661; package-renamed for TraceLab.
/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  basemod.ReflectionHacks
 *  com.badlogic.gdx.Gdx
 *  com.megacrit.cardcrawl.cards.AbstractCard
 *  com.megacrit.cardcrawl.cards.CardGroup
 *  com.megacrit.cardcrawl.core.CardCrawlGame
 *  com.megacrit.cardcrawl.core.Settings
 *  com.megacrit.cardcrawl.dungeons.AbstractDungeon
 *  com.megacrit.cardcrawl.dungeons.AbstractDungeon$CurrentScreen
 *  com.megacrit.cardcrawl.events.AbstractImageEvent
 *  com.megacrit.cardcrawl.events.GenericEventDialog
 *  com.megacrit.cardcrawl.events.RoomEventDialog
 *  com.megacrit.cardcrawl.events.shrines.GremlinMatchGame
 *  com.megacrit.cardcrawl.events.shrines.GremlinWheelGame
 *  com.megacrit.cardcrawl.helpers.Hitbox
 *  com.megacrit.cardcrawl.helpers.input.InputHelper
 *  com.megacrit.cardcrawl.map.MapRoomNode
 *  com.megacrit.cardcrawl.relics.AbstractRelic
 *  com.megacrit.cardcrawl.rewards.RewardItem
 *  com.megacrit.cardcrawl.rewards.chests.AbstractChest
 *  com.megacrit.cardcrawl.rooms.AbstractRoom$RoomPhase
 *  com.megacrit.cardcrawl.rooms.CampfireUI
 *  com.megacrit.cardcrawl.rooms.RestRoom
 *  com.megacrit.cardcrawl.rooms.ShopRoom
 *  com.megacrit.cardcrawl.rooms.TreasureRoom
 *  com.megacrit.cardcrawl.rooms.TreasureRoomBoss
 *  com.megacrit.cardcrawl.screens.CardRewardScreen
 *  com.megacrit.cardcrawl.screens.mainMenu.MenuCancelButton
 *  com.megacrit.cardcrawl.screens.select.BossRelicSelectScreen
 *  com.megacrit.cardcrawl.screens.select.GridCardSelectScreen
 *  com.megacrit.cardcrawl.screens.select.HandCardSelectScreen
 *  com.megacrit.cardcrawl.shop.ShopScreen
 *  com.megacrit.cardcrawl.shop.StorePotion
 *  com.megacrit.cardcrawl.shop.StoreRelic
 *  com.megacrit.cardcrawl.ui.buttons.CardSelectConfirmButton
 *  com.megacrit.cardcrawl.ui.buttons.GridSelectConfirmButton
 *  com.megacrit.cardcrawl.ui.buttons.LargeDialogOptionButton
 *  com.megacrit.cardcrawl.ui.buttons.ProceedButton
 *  com.megacrit.cardcrawl.ui.buttons.SingingBowlButton
 *  com.megacrit.cardcrawl.ui.buttons.SkipCardButton
 *  com.megacrit.cardcrawl.ui.campfire.AbstractCampfireOption
 *  org.apache.logging.log4j.LogManager
 *  org.apache.logging.log4j.Logger
 */
package tracelab.cm;

import basemod.ReflectionHacks;
import com.badlogic.gdx.Gdx;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.cards.CardGroup;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.events.AbstractImageEvent;
import com.megacrit.cardcrawl.events.GenericEventDialog;
import com.megacrit.cardcrawl.events.RoomEventDialog;
import com.megacrit.cardcrawl.events.shrines.GremlinMatchGame;
import com.megacrit.cardcrawl.events.shrines.GremlinWheelGame;
import com.megacrit.cardcrawl.helpers.Hitbox;
import com.megacrit.cardcrawl.helpers.input.InputHelper;
import com.megacrit.cardcrawl.map.MapRoomNode;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rewards.RewardItem;
import com.megacrit.cardcrawl.rewards.chests.AbstractChest;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.rooms.CampfireUI;
import com.megacrit.cardcrawl.rooms.RestRoom;
import com.megacrit.cardcrawl.rooms.ShopRoom;
import com.megacrit.cardcrawl.rooms.TreasureRoom;
import com.megacrit.cardcrawl.rooms.TreasureRoomBoss;
import com.megacrit.cardcrawl.screens.CardRewardScreen;
import com.megacrit.cardcrawl.screens.mainMenu.MenuCancelButton;
import com.megacrit.cardcrawl.screens.select.BossRelicSelectScreen;
import com.megacrit.cardcrawl.screens.select.GridCardSelectScreen;
import com.megacrit.cardcrawl.screens.select.HandCardSelectScreen;
import com.megacrit.cardcrawl.shop.ShopScreen;
import com.megacrit.cardcrawl.shop.StorePotion;
import com.megacrit.cardcrawl.shop.StoreRelic;
import com.megacrit.cardcrawl.ui.buttons.CardSelectConfirmButton;
import com.megacrit.cardcrawl.ui.buttons.GridSelectConfirmButton;
import com.megacrit.cardcrawl.ui.buttons.LargeDialogOptionButton;
import com.megacrit.cardcrawl.ui.buttons.ProceedButton;
import com.megacrit.cardcrawl.ui.buttons.SingingBowlButton;
import com.megacrit.cardcrawl.ui.buttons.SkipCardButton;
import com.megacrit.cardcrawl.ui.campfire.AbstractCampfireOption;
import tracelab.cm.GameStateListener;
import tracelab.cm.patches.AbstractRelicUpdatePatch;
import tracelab.cm.patches.CardRewardScreenPatch;
import tracelab.cm.patches.DungeonMapPatch;
import tracelab.cm.patches.GremlinMatchGamePatch;
import tracelab.cm.patches.GridCardSelectScreenPatch;
import tracelab.cm.patches.MapRoomNodeHoverPatch;
import tracelab.cm.patches.MerchantPatch;
import tracelab.cm.patches.ShopScreenPatch;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public class ChoiceScreenUtils {
    private static final Logger logger = LogManager.getLogger((String)ChoiceScreenUtils.class.getName());

    public static ChoiceType getCurrentChoiceType() {
        if (!AbstractDungeon.isScreenUp) {
            if (AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.EVENT || AbstractDungeon.getCurrRoom().event != null && AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMPLETE) {
                return ChoiceType.EVENT;
            }
            if (AbstractDungeon.getCurrRoom() instanceof TreasureRoomBoss || AbstractDungeon.getCurrRoom() instanceof TreasureRoom) {
                return ChoiceType.CHEST;
            }
            if (AbstractDungeon.getCurrRoom() instanceof ShopRoom) {
                return ChoiceType.SHOP_ROOM;
            }
            if (AbstractDungeon.getCurrRoom() instanceof RestRoom) {
                return ChoiceType.REST;
            }
            if (AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMPLETE && AbstractDungeon.actionManager.isEmpty() && !AbstractDungeon.isFadingOut) {
                if (AbstractDungeon.getCurrRoom().event == null || !(AbstractDungeon.getCurrRoom().event instanceof AbstractImageEvent) && !AbstractDungeon.getCurrRoom().event.hasFocus) {
                    return ChoiceType.COMPLETE;
                }
            } else {
                return ChoiceType.NONE;
            }
        }
        AbstractDungeon.CurrentScreen screen = AbstractDungeon.screen;
        switch (screen) {
            case CARD_REWARD: {
                return ChoiceType.CARD_REWARD;
            }
            case COMBAT_REWARD: {
                return ChoiceType.COMBAT_REWARD;
            }
            case MAP: {
                return ChoiceType.MAP;
            }
            case BOSS_REWARD: {
                return ChoiceType.BOSS_REWARD;
            }
            case SHOP: {
                return ChoiceType.SHOP_SCREEN;
            }
            case GRID: {
                return ChoiceType.GRID;
            }
            case HAND_SELECT: {
                return ChoiceType.HAND_SELECT;
            }
            case DEATH: 
            case VICTORY: 
            case UNLOCK: 
            case NEOW_UNLOCK: {
                return ChoiceType.GAME_OVER;
            }
        }
        return ChoiceType.NONE;
    }

    public static ArrayList<String> getCurrentChoiceList() {
        ArrayList<String> choices;
        ChoiceType choiceType = ChoiceScreenUtils.getCurrentChoiceType();
        switch (choiceType) {
            case EVENT: {
                choices = ChoiceScreenUtils.getEventScreenChoices();
                break;
            }
            case CHEST: {
                choices = ChoiceScreenUtils.getChestRoomChoices();
                break;
            }
            case SHOP_ROOM: {
                choices = ChoiceScreenUtils.getShopRoomChoices();
                break;
            }
            case REST: {
                choices = ChoiceScreenUtils.getRestRoomChoices();
                break;
            }
            case CARD_REWARD: {
                choices = ChoiceScreenUtils.getCardRewardScreenChoices();
                break;
            }
            case COMBAT_REWARD: {
                choices = ChoiceScreenUtils.getCombatRewardScreenChoices();
                break;
            }
            case MAP: {
                choices = ChoiceScreenUtils.getMapScreenChoices();
                break;
            }
            case BOSS_REWARD: {
                choices = ChoiceScreenUtils.getBossRewardScreenChoices();
                break;
            }
            case SHOP_SCREEN: {
                choices = ChoiceScreenUtils.getShopScreenChoices();
                break;
            }
            case GRID: {
                choices = ChoiceScreenUtils.getGridScreenChoices();
                break;
            }
            case HAND_SELECT: {
                choices = ChoiceScreenUtils.getHandSelectScreenChoices();
                break;
            }
            default: {
                return new ArrayList<String>();
            }
        }
        ArrayList<String> lowerCaseChoices = new ArrayList<String>();
        for (String item : choices) {
            lowerCaseChoices.add(item.toLowerCase());
        }
        return lowerCaseChoices;
    }

    public static void executeChoice(int choice_index) {
        ChoiceType choiceType = ChoiceScreenUtils.getCurrentChoiceType();
        switch (choiceType) {
            case EVENT: {
                ChoiceScreenUtils.makeEventChoice(choice_index);
                return;
            }
            case CHEST: {
                ChoiceScreenUtils.makeChestRoomChoice(choice_index);
                return;
            }
            case SHOP_ROOM: {
                ChoiceScreenUtils.makeShopRoomChoice(choice_index);
                return;
            }
            case REST: {
                ChoiceScreenUtils.makeRestRoomChoice(choice_index);
                return;
            }
            case CARD_REWARD: {
                ChoiceScreenUtils.makeCardRewardChoice(choice_index);
                return;
            }
            case COMBAT_REWARD: {
                ChoiceScreenUtils.makeCombatRewardChoice(choice_index);
                return;
            }
            case MAP: {
                ChoiceScreenUtils.makeMapChoice(choice_index);
                return;
            }
            case BOSS_REWARD: {
                ChoiceScreenUtils.makeBossRewardChoice(choice_index);
                return;
            }
            case SHOP_SCREEN: {
                ChoiceScreenUtils.makeShopScreenChoice(choice_index);
                return;
            }
            case GRID: {
                ChoiceScreenUtils.makeGridScreenChoice(choice_index);
                return;
            }
            case HAND_SELECT: {
                ChoiceScreenUtils.makeHandSelectScreenChoice(choice_index);
                return;
            }
        }
        logger.info("Unimplemented choice.");
    }

    private static boolean isCancelButtonAvailable(ChoiceType choiceType) {
        switch (choiceType) {
            case EVENT: {
                return false;
            }
            case CHEST: {
                return false;
            }
            case SHOP_ROOM: {
                return false;
            }
            case REST: {
                return false;
            }
            case CARD_REWARD: {
                return ChoiceScreenUtils.isCardRewardSkipAvailable();
            }
            case COMBAT_REWARD: {
                return false;
            }
            case MAP: {
                return AbstractDungeon.dungeonMapScreen.dismissable;
            }
            case BOSS_REWARD: {
                return true;
            }
            case SHOP_SCREEN: {
                return true;
            }
            case GRID: {
                return ChoiceScreenUtils.isGridScreenCancelAvailable();
            }
            case HAND_SELECT: {
                return false;
            }
            case GAME_OVER: {
                return false;
            }
            case COMPLETE: {
                return false;
            }
        }
        return false;
    }

    public static boolean isCancelButtonAvailable() {
        return ChoiceScreenUtils.isCancelButtonAvailable(ChoiceScreenUtils.getCurrentChoiceType());
    }

    private static String getCancelButtonText(ChoiceType choiceType) {
        switch (choiceType) {
            case CARD_REWARD: {
                return "skip";
            }
            case MAP: {
                return "return";
            }
            case BOSS_REWARD: {
                return "skip";
            }
            case SHOP_SCREEN: {
                return "leave";
            }
            case GRID: {
                return "cancel";
            }
        }
        return "cancel";
    }

    public static String getCancelButtonText() {
        return ChoiceScreenUtils.getCancelButtonText(ChoiceScreenUtils.getCurrentChoiceType());
    }

    private static void pressCancelButton(ChoiceType choiceType) {
        switch (choiceType) {
            case CARD_REWARD: {
                AbstractDungeon.closeCurrentScreen();
                return;
            }
            case MAP: {
                ChoiceScreenUtils.clickCancelButton();
                return;
            }
            case BOSS_REWARD: {
                MenuCancelButton button = (MenuCancelButton)ReflectionHacks.getPrivate((Object)AbstractDungeon.bossRelicScreen, BossRelicSelectScreen.class, (String)"cancelButton");
                button.hb.clicked = true;
                return;
            }
            case SHOP_SCREEN: {
                ChoiceScreenUtils.clickCancelButton();
                return;
            }
            case GRID: {
                ChoiceScreenUtils.clickCancelButton();
            }
        }
    }

    public static void pressCancelButton() {
        ChoiceScreenUtils.pressCancelButton(ChoiceScreenUtils.getCurrentChoiceType());
    }

    private static boolean isConfirmButtonAvailable(ChoiceType choiceType) {
        switch (choiceType) {
            case EVENT: {
                return false;
            }
            case CHEST: {
                return true;
            }
            case SHOP_ROOM: {
                return true;
            }
            case REST: {
                return ChoiceScreenUtils.isRestRoomProceedAvailable();
            }
            case CARD_REWARD: {
                return false;
            }
            case COMBAT_REWARD: {
                return true;
            }
            case MAP: {
                return false;
            }
            case BOSS_REWARD: {
                return false;
            }
            case SHOP_SCREEN: {
                return false;
            }
            case GRID: {
                return ChoiceScreenUtils.isGridScreenConfirmAvailable();
            }
            case HAND_SELECT: {
                return ChoiceScreenUtils.isHandSelectConfirmButtonEnabled();
            }
            case GAME_OVER: {
                return true;
            }
            case COMPLETE: {
                return true;
            }
        }
        return false;
    }

    public static boolean isConfirmButtonAvailable() {
        return ChoiceScreenUtils.isConfirmButtonAvailable(ChoiceScreenUtils.getCurrentChoiceType());
    }

    private static String getConfirmButtonText(ChoiceType choiceType) {
        switch (choiceType) {
            case CHEST: {
                return "proceed";
            }
            case SHOP_ROOM: {
                return "proceed";
            }
            case REST: {
                return "proceed";
            }
            case COMBAT_REWARD: {
                return "proceed";
            }
            case GRID: {
                return "confirm";
            }
            case HAND_SELECT: {
                return "confirm";
            }
            case GAME_OVER: {
                return "proceed";
            }
            case COMPLETE: {
                return "proceed";
            }
        }
        return "confirm";
    }

    public static String getConfirmButtonText() {
        return ChoiceScreenUtils.getConfirmButtonText(ChoiceScreenUtils.getCurrentChoiceType());
    }

    public static void pressConfirmButton(ChoiceType choiceType) {
        switch (choiceType) {
            case CHEST: {
                ChoiceScreenUtils.clickProceedButton();
                return;
            }
            case SHOP_ROOM: {
                ChoiceScreenUtils.clickProceedButton();
                return;
            }
            case REST: {
                ChoiceScreenUtils.clickProceedButton();
                return;
            }
            case COMBAT_REWARD: {
                ChoiceScreenUtils.clickProceedButton();
                return;
            }
            case GRID: {
                ChoiceScreenUtils.clickGridScreenConfirmButton();
                return;
            }
            case HAND_SELECT: {
                ChoiceScreenUtils.clickHandSelectScreenConfirmButton();
                return;
            }
            case GAME_OVER: {
                ChoiceScreenUtils.clickGameOverReturnButton();
                return;
            }
            case COMPLETE: {
                ChoiceScreenUtils.clickProceedButton();
            }
        }
    }

    public static void pressConfirmButton() {
        ChoiceScreenUtils.pressConfirmButton(ChoiceScreenUtils.getCurrentChoiceType());
    }

    public static ArrayList<String> getCardRewardScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        for (AbstractCard card : AbstractDungeon.cardRewardScreen.rewardGroup) {
            choices.add(card.name.toLowerCase());
        }
        if (ChoiceScreenUtils.isBowlAvailable()) {
            choices.add("bowl");
        }
        return choices;
    }

    public static boolean isBowlAvailable() {
        SingingBowlButton bowlButton = (SingingBowlButton)ReflectionHacks.getPrivate((Object)AbstractDungeon.cardRewardScreen, CardRewardScreen.class, (String)"bowlButton");
        return (Boolean)ReflectionHacks.getPrivate((Object)bowlButton, SingingBowlButton.class, (String)"isHidden") == false;
    }

    public static boolean isCardRewardSkipAvailable() {
        SkipCardButton skipButton = (SkipCardButton)ReflectionHacks.getPrivate((Object)AbstractDungeon.cardRewardScreen, CardRewardScreen.class, (String)"skipButton");
        return (Boolean)ReflectionHacks.getPrivate((Object)skipButton, SkipCardButton.class, (String)"isHidden") == false;
    }

    public static void makeCardRewardChoice(int choice) {
        ArrayList<String> choices = ChoiceScreenUtils.getCardRewardScreenChoices();
        if (choices.get(choice).equals("bowl")) {
            SingingBowlButton bowlButton = (SingingBowlButton)ReflectionHacks.getPrivate((Object)AbstractDungeon.cardRewardScreen, CardRewardScreen.class, (String)"bowlButton");
            bowlButton.onClick();
            AbstractDungeon.cardRewardScreen.closeFromBowlButton();
            AbstractDungeon.closeCurrentScreen();
        } else {
            AbstractCard selectedCard = (AbstractCard)AbstractDungeon.cardRewardScreen.rewardGroup.get(choice);
            CardRewardScreenPatch.doHover = true;
            CardRewardScreenPatch.hoverCard = selectedCard;
            selectedCard.hb.clicked = true;
        }
    }

    public static ArrayList<String> getHandSelectScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        HandCardSelectScreen screen = AbstractDungeon.handCardSelectScreen;
        if (screen.numCardsToSelect == screen.selectedCards.group.size()) {
            return choices;
        }
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            choices.add(card.name.toLowerCase());
        }
        return choices;
    }

    public static void makeHandSelectScreenChoice(int choice) {
        HandCardSelectScreen screen = AbstractDungeon.handCardSelectScreen;
        screen.hoveredCard = (AbstractCard)AbstractDungeon.player.hand.group.get(choice);
        screen.hoveredCard.setAngle(0.0f, false);
        try {
            Method hotkeyCheck = HandCardSelectScreen.class.getDeclaredMethod("selectHoveredCard", new Class[0]);
            hotkeyCheck.setAccessible(true);
            hotkeyCheck.invoke(screen, new Object[0]);
        }
        catch (IllegalAccessException | NoSuchMethodException | InvocationTargetException e) {
            e.printStackTrace();
            throw new RuntimeException("selectHoveredCard method somehow can't be called.");
        }
    }

    private static void clickHandSelectScreenConfirmButton() {
        HandCardSelectScreen screen = AbstractDungeon.handCardSelectScreen;
        screen.button.hb.clicked = true;
    }

    private static boolean isHandSelectConfirmButtonEnabled() {
        CardSelectConfirmButton button = AbstractDungeon.handCardSelectScreen.button;
        boolean isHidden = (Boolean)ReflectionHacks.getPrivate((Object)button, CardSelectConfirmButton.class, (String)"isHidden");
        boolean isDisabled = button.isDisabled;
        return !isHidden && !isDisabled;
    }

    public static ArrayList<AbstractCard> getGridScreenCards() {
        GridCardSelectScreen screen = AbstractDungeon.gridSelectScreen;
        CardGroup cards = (CardGroup)ReflectionHacks.getPrivate((Object)screen, GridCardSelectScreen.class, (String)"targetGroup");
        return cards.group;
    }

    public static ArrayList<String> getGridScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        if (AbstractDungeon.gridSelectScreen.confirmScreenUp || AbstractDungeon.gridSelectScreen.isJustForConfirming) {
            return choices;
        }
        for (AbstractCard card : ChoiceScreenUtils.getGridScreenCards()) {
            choices.add(card.name.toLowerCase());
        }
        return choices;
    }

    public static void makeGridScreenChoice(int choice) {
        GridCardSelectScreen screen = AbstractDungeon.gridSelectScreen;
        GridCardSelectScreenPatch.hoverCard = ChoiceScreenUtils.getGridScreenCards().get(choice);
        GridCardSelectScreenPatch.replaceHoverCard = true;
    }

    private static void clickGridScreenConfirmButton() {
        GridCardSelectScreen screen = AbstractDungeon.gridSelectScreen;
        screen.confirmButton.hb.clicked = true;
        if (AbstractDungeon.previousScreen == AbstractDungeon.CurrentScreen.SHOP) {
            GameStateListener.blockStateUpdate();
        }
    }

    private static boolean isGridScreenCancelAvailable() {
        GridCardSelectScreen screen = AbstractDungeon.gridSelectScreen;
        boolean canCancel = (Boolean)ReflectionHacks.getPrivate((Object)screen, GridCardSelectScreen.class, (String)"canCancel");
        if (canCancel && (screen.forPurge || screen.forTransform || screen.forUpgrade || AbstractDungeon.previousScreen == AbstractDungeon.CurrentScreen.SHOP)) {
            return true;
        }
        return screen.confirmScreenUp;
    }

    private static boolean isGridScreenConfirmAvailable() {
        GridCardSelectScreen screen = AbstractDungeon.gridSelectScreen;
        if (screen.confirmScreenUp || screen.isJustForConfirming) {
            return true;
        }
        return !screen.confirmButton.isDisabled && (Boolean)ReflectionHacks.getPrivate((Object)screen.confirmButton, GridSelectConfirmButton.class, (String)"isHidden") == false && (screen.forUpgrade || screen.forTransform || screen.forPurge || screen.anyNumber);
    }

    public static ArrayList<String> getCombatRewardScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        for (RewardItem reward : AbstractDungeon.combatRewardScreen.rewards) {
            choices.add(reward.type.name().toLowerCase());
        }
        return choices;
    }

    public static void makeCombatRewardChoice(int choice) {
        RewardItem reward = (RewardItem)AbstractDungeon.combatRewardScreen.rewards.get(choice);
        reward.isDone = true;
    }

    public static ArrayList<String> getBossRewardScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        for (AbstractRelic relic : AbstractDungeon.bossRelicScreen.relics) {
            choices.add(relic.name);
        }
        return choices;
    }

    public static void makeBossRewardChoice(int choice) {
        AbstractRelic chosenRelic = (AbstractRelic)AbstractDungeon.bossRelicScreen.relics.get(choice);
        AbstractRelicUpdatePatch.doHover = true;
        AbstractRelicUpdatePatch.hoverRelic = chosenRelic;
        InputHelper.justClickedLeft = true;
    }

    public static ArrayList<String> getChestRoomChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        AbstractChest chest = null;
        if (AbstractDungeon.getCurrRoom() instanceof TreasureRoomBoss) {
            chest = ((TreasureRoomBoss)AbstractDungeon.getCurrRoom()).chest;
        } else if (AbstractDungeon.getCurrRoom() instanceof TreasureRoom) {
            chest = ((TreasureRoom)AbstractDungeon.getCurrRoom()).chest;
        }
        if (chest != null && !chest.isOpen) {
            choices.add("open");
        }
        return choices;
    }

    public static void makeChestRoomChoice(int choice) {
        if (AbstractDungeon.getCurrRoom() instanceof TreasureRoomBoss) {
            AbstractChest chest = ((TreasureRoomBoss)AbstractDungeon.getCurrRoom()).chest;
            chest.isOpen = true;
            chest.open(false);
        } else if (AbstractDungeon.getCurrRoom() instanceof TreasureRoom) {
            AbstractChest chest = ((TreasureRoom)AbstractDungeon.getCurrRoom()).chest;
            chest.isOpen = true;
            chest.open(false);
        }
    }

    public static ArrayList<String> getShopRoomChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        choices.add("shop");
        return choices;
    }

    public static void makeShopRoomChoice(int choice) {
        MerchantPatch.visitMerchant = true;
    }

    public static ArrayList<String> getShopScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        ArrayList<Object> shopItems = ChoiceScreenUtils.getAvailableShopItems();
        for (Object item : shopItems) {
            if (item instanceof String) {
                choices.add((String)item);
                continue;
            }
            if (item instanceof AbstractCard) {
                choices.add(((AbstractCard)item).name.toLowerCase());
                continue;
            }
            if (item instanceof StoreRelic) {
                choices.add(((StoreRelic)item).relic.name);
                continue;
            }
            if (!(item instanceof StorePotion)) continue;
            choices.add(((StorePotion)item).potion.name);
        }
        return choices;
    }

    public static ArrayList<AbstractCard> getShopScreenCards() {
        ArrayList<AbstractCard> cards = new ArrayList<AbstractCard>();
        ShopScreen screen = AbstractDungeon.shopScreen;
        ArrayList coloredCards = (ArrayList)ReflectionHacks.getPrivate((Object)screen, ShopScreen.class, (String)"coloredCards");
        ArrayList colorlessCards = (ArrayList)ReflectionHacks.getPrivate((Object)screen, ShopScreen.class, (String)"colorlessCards");
        cards.addAll(coloredCards);
        cards.addAll(colorlessCards);
        return cards;
    }

    public static ArrayList<StoreRelic> getShopScreenRelics() {
        ShopScreen screen = AbstractDungeon.shopScreen;
        return (ArrayList)ReflectionHacks.getPrivate((Object)screen, ShopScreen.class, (String)"relics");
    }

    public static ArrayList<StorePotion> getShopScreenPotions() {
        ShopScreen screen = AbstractDungeon.shopScreen;
        return (ArrayList)ReflectionHacks.getPrivate((Object)screen, ShopScreen.class, (String)"potions");
    }

    private static ArrayList<Object> getAvailableShopItems() {
        ArrayList<Object> choices = new ArrayList<Object>();
        ShopScreen screen = AbstractDungeon.shopScreen;
        if (screen.purgeAvailable && AbstractDungeon.player.gold >= ShopScreen.actualPurgeCost) {
            choices.add("purge");
        }
        for (AbstractCard card : ChoiceScreenUtils.getShopScreenCards()) {
            if (card.price > AbstractDungeon.player.gold) continue;
            choices.add(card);
        }
        for (StoreRelic relic : ChoiceScreenUtils.getShopScreenRelics()) {
            if (relic.price > AbstractDungeon.player.gold) continue;
            choices.add(relic);
        }
        for (StorePotion potion : ChoiceScreenUtils.getShopScreenPotions()) {
            if (potion.price > AbstractDungeon.player.gold) continue;
            choices.add(potion);
        }
        return choices;
    }

    public static void makeShopScreenChoice(int choice) {
        ArrayList<Object> shopItems = ChoiceScreenUtils.getAvailableShopItems();
        Object shopItem = shopItems.get(choice);
        if (shopItem instanceof String) {
            AbstractDungeon.previousScreen = AbstractDungeon.CurrentScreen.SHOP;
            AbstractDungeon.gridSelectScreen.open(CardGroup.getGroupWithoutBottledCards((CardGroup)AbstractDungeon.player.masterDeck.getPurgeableCards()), 1, ShopScreen.NAMES[13], false, false, true, true);
        } else if (shopItem instanceof AbstractCard) {
            AbstractCard card = (AbstractCard)shopItem;
            ShopScreenPatch.doHover = true;
            ShopScreenPatch.hoverCard = card;
            card.hb.clicked = true;
        } else if (shopItem instanceof StoreRelic) {
            StoreRelic relic = (StoreRelic)shopItem;
            relic.relic.hb.clicked = true;
        } else if (shopItem instanceof StorePotion) {
            StorePotion potion = (StorePotion)shopItem;
            potion.potion.hb.clicked = true;
        }
    }

    private static void clickProceedButton() {
        AbstractDungeon.overlayMenu.proceedButton.show();
        Hitbox hb = (Hitbox)ReflectionHacks.getPrivate((Object)AbstractDungeon.overlayMenu.proceedButton, ProceedButton.class, (String)"hb");
        hb.clicked = true;
    }

    private static void clickCancelButton() {
        AbstractDungeon.overlayMenu.cancelButton.hb.clicked = true;
    }

    private static void setCursorPosition(float x, float y) {
        Gdx.input.setCursorPosition((int)x, (int)y);
        InputHelper.updateFirst();
    }

    public static boolean bossNodeAvailable() {
        MapRoomNode currMapNode = AbstractDungeon.getCurrMapNode();
        return currMapNode.y == 14 || AbstractDungeon.id.equals("TheEnding") && currMapNode.y == 2;
    }

    public static ArrayList<String> getMapScreenChoices() {
        ArrayList<String> choices = new ArrayList<String>();
        MapRoomNode currMapNode = AbstractDungeon.getCurrMapNode();
        if (ChoiceScreenUtils.bossNodeAvailable()) {
            choices.add("boss");
            return choices;
        }
        ArrayList<MapRoomNode> availableNodes = ChoiceScreenUtils.getMapScreenNodeChoices();
        for (MapRoomNode node : availableNodes) {
            choices.add(String.format("x=%d", node.x).toLowerCase());
        }
        return choices;
    }

    public static ArrayList<MapRoomNode> getMapScreenNodeChoices() {
        ArrayList<MapRoomNode> choices = new ArrayList<MapRoomNode>();
        MapRoomNode currMapNode = AbstractDungeon.getCurrMapNode();
        ArrayList map = AbstractDungeon.map;
        if (!AbstractDungeon.firstRoomChosen) {
            for (MapRoomNode node : (java.util.ArrayList<MapRoomNode>)map.get(0)) {
                if (!node.hasEdges()) continue;
                choices.add(node);
            }
        } else {
            for (ArrayList rows : (java.util.ArrayList<ArrayList>)map) {
                for (MapRoomNode node : (java.util.ArrayList<MapRoomNode>)rows) {
                    if (!node.hasEdges()) continue;
                    boolean normalConnection = currMapNode.isConnectedTo(node);
                    boolean wingedConnection = currMapNode.wingedIsConnectedTo(node);
                    if (!normalConnection && !wingedConnection) continue;
                    choices.add(node);
                }
            }
        }
        return choices;
    }

    public static void makeMapChoice(int choice) {
        MapRoomNode currMapNode = AbstractDungeon.getCurrMapNode();
        if (currMapNode.y == 14 || AbstractDungeon.id.equals("TheEnding") && currMapNode.y == 2) {
            if (choice == 0) {
                DungeonMapPatch.doBossHover = true;
                return;
            }
            throw new IndexOutOfBoundsException("Only a boss node can be chosen here.");
        }
        ArrayList<MapRoomNode> nodeChoices = ChoiceScreenUtils.getMapScreenNodeChoices();
        MapRoomNodeHoverPatch.hoverNode = nodeChoices.get(choice);
        MapRoomNodeHoverPatch.doHover = true;
        AbstractDungeon.dungeonMapScreen.clicked = true;
    }

    public static String getOptionName(String input) {
        String unformatted = input.replaceAll("#.|NL", "");
        Pattern regex = Pattern.compile("\\[(.*?)\\]");
        Matcher matcher = regex.matcher(unformatted);
        if (matcher.find()) {
            return matcher.group(1).trim();
        }
        return unformatted.trim();
    }

    public static EventDialogType getEventDialogType() {
        boolean genericShown = (Boolean)ReflectionHacks.getPrivateStatic(GenericEventDialog.class, (String)"show");
        if (genericShown) {
            return EventDialogType.IMAGE;
        }
        boolean roomShown = (Boolean)ReflectionHacks.getPrivate((Object)AbstractDungeon.getCurrRoom().event.roomEventText, RoomEventDialog.class, (String)"show");
        if (roomShown) {
            return EventDialogType.ROOM;
        }
        return EventDialogType.NONE;
    }

    public static ArrayList<LargeDialogOptionButton> getEventButtons() {
        EventDialogType eventType = ChoiceScreenUtils.getEventDialogType();
        switch (eventType) {
            case IMAGE: {
                return AbstractDungeon.getCurrRoom().event.imageEventText.optionList;
            }
            case ROOM: {
                return RoomEventDialog.optionList;
            }
        }
        return new ArrayList<LargeDialogOptionButton>();
    }

    public static ArrayList<LargeDialogOptionButton> getActiveEventButtons() {
        ArrayList<LargeDialogOptionButton> buttons = ChoiceScreenUtils.getEventButtons();
        ArrayList<LargeDialogOptionButton> activeButtons = new ArrayList<LargeDialogOptionButton>();
        for (LargeDialogOptionButton button : buttons) {
            if (button.isDisabled) continue;
            activeButtons.add(button);
        }
        return activeButtons;
    }

    public static ArrayList<String> getEventScreenChoices() {
        ArrayList<String> choiceList;
        block4: {
            block5: {
                block3: {
                    choiceList = new ArrayList<String>();
                    ArrayList<LargeDialogOptionButton> activeButtons = ChoiceScreenUtils.getActiveEventButtons();
                    if (activeButtons.size() <= 0) break block3;
                    for (LargeDialogOptionButton button : activeButtons) {
                        choiceList.add(ChoiceScreenUtils.getOptionName(button.msg).toLowerCase());
                    }
                    break block4;
                }
                if (!(AbstractDungeon.getCurrRoom().event instanceof GremlinWheelGame)) break block5;
                choiceList.add("spin");
                break block4;
            }
            if (!(AbstractDungeon.getCurrRoom().event instanceof GremlinMatchGame)) break block4;
            ArrayList<AbstractCard> pickableCards = GremlinMatchGamePatch.getOrderedCards();
            for (AbstractCard c : pickableCards) {
                if (GremlinMatchGamePatch.revealedCards.contains(c.uuid)) {
                    choiceList.add(c.cardID);
                    continue;
                }
                choiceList.add(String.format("card%d", GremlinMatchGamePatch.cardPositions.get(c.uuid)));
            }
        }
        return choiceList;
    }

    public static void makeEventChoice(int choice) {
        ArrayList<LargeDialogOptionButton> activeButtons = ChoiceScreenUtils.getActiveEventButtons();
        if (activeButtons.size() > 0) {
            activeButtons.get((int)choice).pressed = true;
        } else if (AbstractDungeon.getCurrRoom().event instanceof GremlinWheelGame) {
            GremlinWheelGame event = (GremlinWheelGame)AbstractDungeon.getCurrRoom().event;
            ReflectionHacks.setPrivate((Object)event, GremlinWheelGame.class, (String)"buttonPressed", (Object)true);
            CardCrawlGame.sound.play("WHEEL");
        } else if (AbstractDungeon.getCurrRoom().event instanceof GremlinMatchGame) {
            ArrayList<AbstractCard> pickable = GremlinMatchGamePatch.getOrderedCards();
            GremlinMatchGamePatch.HoverCardPatch.hoverCard = pickable.get(choice);
            GremlinMatchGamePatch.HoverCardPatch.doHover = true;
        }
    }

    public static ArrayList<String> getRestRoomChoices() {
        ArrayList<String> choiceList = new ArrayList<String>();
        ArrayList<AbstractCampfireOption> buttons = ChoiceScreenUtils.getValidRestRoomButtons();
        for (AbstractCampfireOption button : (java.util.ArrayList<AbstractCampfireOption>)buttons) {
            choiceList.add(ChoiceScreenUtils.getCampfireOptionName(button));
        }
        return choiceList;
    }

    public static void makeRestRoomChoice(int choice_index) {
        ArrayList<AbstractCampfireOption> buttons = ChoiceScreenUtils.getValidRestRoomButtons();
        AbstractCampfireOption button = buttons.get(choice_index);
        RestRoom room = (RestRoom)AbstractDungeon.getCurrRoom();
        button.useOption();
        room.campfireUI.somethingSelected = true;
    }

    private static boolean isRestRoomProceedAvailable() {
        return AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMPLETE;
    }

    private static ArrayList<AbstractCampfireOption> getValidRestRoomButtons() {
        ArrayList<AbstractCampfireOption> choiceList = new ArrayList<AbstractCampfireOption>();
        RestRoom room = (RestRoom)AbstractDungeon.getCurrRoom();
        if (!ChoiceScreenUtils.isRestRoomProceedAvailable()) {
            ArrayList buttons = (ArrayList)ReflectionHacks.getPrivate((Object)room.campfireUI, CampfireUI.class, (String)"buttons");
            for (AbstractCampfireOption button : (java.util.ArrayList<AbstractCampfireOption>)buttons) {
                if (!button.usable) continue;
                choiceList.add(button);
            }
        }
        return choiceList;
    }

    private static String getCampfireOptionName(AbstractCampfireOption option) {
        String classname = option.getClass().getSimpleName();
        String nameWithoutOption = classname.substring(0, classname.length() - "Option".length());
        return nameWithoutOption.toLowerCase();
    }

    private static void clickGameOverReturnButton() {
        AbstractDungeon.unlocks.clear();
        Settings.isTrial = false;
        Settings.isDailyRun = false;
        Settings.isEndless = false;
        CardCrawlGame.trial = null;
        CardCrawlGame.startOver();
    }

    public static enum EventDialogType {
        IMAGE,
        ROOM,
        NONE;

    }

    public static enum ChoiceType {
        EVENT,
        CHEST,
        SHOP_ROOM,
        REST,
        CARD_REWARD,
        COMBAT_REWARD,
        MAP,
        BOSS_REWARD,
        SHOP_SCREEN,
        GRID,
        HAND_SELECT,
        GAME_OVER,
        COMPLETE,
        NONE;

    }
}

