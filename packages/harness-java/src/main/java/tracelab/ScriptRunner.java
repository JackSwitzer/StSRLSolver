package tracelab;

import com.badlogic.gdx.Gdx;
import com.megacrit.cardcrawl.actions.GameActionManager;
import com.megacrit.cardcrawl.actions.watcher.PressEndTurnButtonAction;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.cards.CardQueueItem;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.events.AbstractEvent;
import com.megacrit.cardcrawl.events.RoomEventDialog;
import com.megacrit.cardcrawl.map.MapEdge;
import com.megacrit.cardcrawl.map.MapRoomNode;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import tracelab.cm.ChoiceScreenUtils;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Feeds script actions when the game reaches a stable state, then records the
 * post-action state. Stability semantics follow CommunicationMod's
 * GameStateListener approach (docs/vault/communication-mod-api.md): empty
 * action queues, no fades, and per-action-type readiness.
 */
public class ScriptRunner {

    private static final int STABLE_FRAMES_REQUIRED = 8;
    private static final boolean DEBUG = !"0".equals(System.getProperty("tracelab.debug", "1"));
    private static final int MAX_AUTO_ADVANCE = 8;
    private static int frameCounter = 0;
    private static int autoAdvancePresses = 0;

    private static Script script;
    private static boolean active = false;
    private static boolean finished = false;
    private static int nextAction = 0;
    private static int recordIdx = 0;
    private static int stableFrames = 0;
    private static boolean awaitingRecord = false;
    private static Script.Action lastAction = null;
    private static int framesSinceExecute = 0;
    private static java.lang.reflect.Field animWaitTimerField;

    private static boolean auto = false;
    private static boolean record = false;
    private static com.megacrit.cardcrawl.map.MapRoomNode lastMapNode = null;
    private static List<MapRoomNode> lastMapOptions = null;

    public static boolean recording() {
        return record && active && !finished;
    }

    public static void recordAction(Script.Action a) {
        if (!recording()) {
            return;
        }
        if (awaitingRecord) {
            TraceWriter.writeRecord(recordIdx, lastAction);
            recordIdx++;
        }
        System.out.println("[TraceLab] recorded " + a.type
                + (a.card_id != null ? " " + a.card_id : "")
                + (a.choice != null ? " choice=" + a.choice : "")
                + (a.choice_name != null ? " " + a.choice_name : ""));
        lastAction = a;
        awaitingRecord = true;
        stableFrames = 0;
    }

    public static void start(Script s) {
        script = s;
        auto = "auto".equals(s.mode);
        record = "record".equals(s.mode);
        if (record && s.actions == null) {
            s.actions = new ArrayList<Script.Action>();
        }
        if (auto && s.actions == null) {
            s.actions = new ArrayList<Script.Action>();
        }
        active = true;
    }

    public static void update() {
        if (!active || finished) {
            return;
        }
        if (!CardCrawlGame.isInARun() || AbstractDungeon.player == null) {
            return;
        }
        if (AbstractDungeon.player.isDead || AbstractDungeon.player.isDying) {
            finish("player_dead");
            return;
        }

        frameCounter++;
        if (DEBUG && frameCounter % 180 == 0) {
            logState();
        }

        if (record) {
            recordModeUpdate();
            return;
        }

        framesSinceExecute++;
        if (!isQuiescent()) {
            stableFrames = 0;
            return;
        }
        stableFrames++;
        if (stableFrames == 3600) {
            logState();
            finish("stuck_next_" + nextAction);
            return;
        }
        if (stableFrames < STABLE_FRAMES_REQUIRED) {
            return;
        }

        if (awaitingRecord) {
            TraceWriter.writeRecord(recordIdx, lastAction);
            recordIdx++;
            awaitingRecord = false;
        }

        if (script.stop != null && script.stop.max_floor != null
                && AbstractDungeon.floorNum > script.stop.max_floor) {
            finish("max_floor");
            return;
        }
        if (nextAction >= script.actions.size()) {
            if (!auto) {
                finish("script_exhausted");
                return;
            }
            Script.Action synth = autoPolicy();
            if (synth == null) {
                return;
            }
            script.actions.add(synth);
        }

        Script.Action a = script.actions.get(nextAction);

        // Single-option event screens (Neow intro filler, forced continues) are
        // skipped, not recorded — Rust models Neow as one NEOW action, so only the
        // real multi-option choice becomes a trace record.
        AbstractRoom evRoom = currRoom();
        if (evRoom != null && evRoom.event != null && evRoom.event.waitTimer <= 0.0f
                && RoomEventDialog.waitForInput
                && eventOptions(evRoom.event).size() == 1) {
            if (autoAdvancePresses >= MAX_AUTO_ADVANCE) {
                finish("event_auto_advance_loop");
                return;
            }
            autoAdvancePresses++;
            System.out.println("[TraceLab] event auto-advance press " + autoAdvancePresses);
            pressEventOption(0);
            stableFrames = 0;
            return;
        }

        if (!readyFor(a)) {
            return;
        }
        System.out.println("[TraceLab] action " + nextAction + ": " + a.type);
        boolean ok = execute(a);
        if (!ok) {
            finish("illegal_action_" + nextAction + "_" + a.type);
            return;
        }
        lastAction = a;
        nextAction++;
        awaitingRecord = true;
        stableFrames = 0;
        framesSinceExecute = 0;
        autoAdvancePresses = 0;
    }

    private static boolean isQuiescent() {
        GameActionManager am = AbstractDungeon.actionManager;
        if (am == null) {
            return false;
        }
        ChoiceScreenUtils.ChoiceType sct = safeChoiceType();
        if (sct == ChoiceScreenUtils.ChoiceType.GRID
                || sct == ChoiceScreenUtils.ChoiceType.HAND_SELECT
                || sct == ChoiceScreenUtils.ChoiceType.CARD_REWARD
                || sct == ChoiceScreenUtils.ChoiceType.BOSS_REWARD) {
            return !AbstractDungeon.isFadingOut && !AbstractDungeon.isFadingIn;
        }
        if (!am.actions.isEmpty() || !am.cardQueue.isEmpty() || !am.preTurnActions.isEmpty()
                || am.currentAction != null) {
            return false;
        }
        if (AbstractDungeon.isFadingOut || AbstractDungeon.isFadingIn) {
            return false;
        }
        AbstractRoom room = currRoom();
        if (room != null && room.phase == AbstractRoom.RoomPhase.COMBAT) {
            if (am.phase != GameActionManager.Phase.WAITING_ON_USER || am.turnHasEnded) {
                return false;
            }
            if (AbstractDungeon.getMonsters() != null) {
                for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                    if (m.isDying && !m.isDead) {
                        return false;
                    }
                }
            }
        }
        // In an event, a press is mid-flight until the dialog re-arms
        // (waitForInput true). Acting sooner clobbers selectedOption.
        if (room != null && room.phase == AbstractRoom.RoomPhase.EVENT
                && room.event != null && !RoomEventDialog.waitForInput) {
            return false;
        }
        return true;
    }

    static ChoiceScreenUtils.ChoiceType safeChoiceType() {
        if (!CardCrawlGame.isInARun() || AbstractDungeon.currMapNode == null
                || AbstractDungeon.player == null) {
            return ChoiceScreenUtils.ChoiceType.NONE;
        }
        try {
            return ChoiceScreenUtils.getCurrentChoiceType();
        } catch (Exception e) {
            return ChoiceScreenUtils.ChoiceType.NONE;
        }
    }

    public static AbstractRoom currRoom() {
        if (!CardCrawlGame.isInARun() || AbstractDungeon.currMapNode == null) {
            return null;
        }
        return AbstractDungeon.getCurrRoom();
    }

    private static boolean readyFor(Script.Action a) {
        AbstractRoom room = currRoom();
        switch (a.type) {
            case "PLAY_CARD":
            case "END_TURN":
            case "USE_POTION":
                return room != null && room.phase == AbstractRoom.RoomPhase.COMBAT;
            case "NEOW":
            case "EVENT_CHOICE": {
                if (room == null || room.event == null) {
                    return false;
                }
                AbstractEvent event = room.event;
                return event.waitTimer <= 0.0f && RoomEventDialog.waitForInput
                        && !eventOptions(event).isEmpty();
            }
            case "PATH":
                return AbstractDungeon.screen == AbstractDungeon.CurrentScreen.MAP;
            case "CHOOSE": {
                ChoiceScreenUtils.ChoiceType ct = safeChoiceType();
                return ct != ChoiceScreenUtils.ChoiceType.NONE
                        && ct != ChoiceScreenUtils.ChoiceType.COMPLETE
                        && ct != ChoiceScreenUtils.ChoiceType.GAME_OVER;
            }
            case "PROCEED":
                return ChoiceScreenUtils.isConfirmButtonAvailable();
            case "CANCEL":
                return ChoiceScreenUtils.isCancelButtonAvailable();
            default:
                return false;
        }
    }

    private static boolean execute(Script.Action a) {
        switch (a.type) {
            case "PLAY_CARD":
                return playCard(a);
            case "END_TURN":
                AbstractDungeon.actionManager.addToBottom(new PressEndTurnButtonAction());
                return true;
            case "USE_POTION":
                return usePotion(a);
            case "NEOW":
            case "EVENT_CHOICE":
                return pressEventOption(a.choice == null ? 0 : a.choice);
            case "PATH":
                return choosePath(a.choice == null ? 0 : a.choice);
            case "CHOOSE":
                ChoiceScreenUtils.executeChoice(a.choice == null ? 0 : a.choice);
                return true;
            case "PROCEED":
                ChoiceScreenUtils.pressConfirmButton();
                return true;
            case "CANCEL":
                ChoiceScreenUtils.pressCancelButton();
                return true;
            default:
                System.err.println("[TraceLab] unsupported action type " + a.type);
                return false;
        }
    }

    private static boolean playCard(Script.Action a) {
        if (a.hand_idx == null || a.hand_idx < 0
                || a.hand_idx >= AbstractDungeon.player.hand.group.size()) {
            System.err.println("[TraceLab] PLAY_CARD bad hand_idx " + a.hand_idx);
            return false;
        }
        AbstractCard card = AbstractDungeon.player.hand.group.get(a.hand_idx);
        AbstractMonster target = null;
        if (a.target != null && a.target >= 0 && AbstractDungeon.getMonsters() != null
                && a.target < AbstractDungeon.getMonsters().monsters.size()) {
            target = AbstractDungeon.getMonsters().monsters.get(a.target);
        }
        if (!card.canUse(AbstractDungeon.player, target)) {
            System.err.println("[TraceLab] PLAY_CARD not playable: " + card.cardID);
            return false;
        }
        AbstractDungeon.actionManager.cardQueue.add(new CardQueueItem(card, target));
        return true;
    }

    private static boolean usePotion(Script.Action a) {
        if (a.idx == null || a.idx < 0 || a.idx >= AbstractDungeon.player.potions.size()) {
            System.err.println("[TraceLab] USE_POTION bad idx " + a.idx);
            return false;
        }
        AbstractPotion potion = AbstractDungeon.player.potions.get(a.idx);
        if (!potion.canUse()) {
            System.err.println("[TraceLab] USE_POTION not usable: " + potion.ID);
            return false;
        }
        AbstractMonster target = null;
        if (a.target != null && a.target >= 0 && AbstractDungeon.getMonsters() != null
                && a.target < AbstractDungeon.getMonsters().monsters.size()) {
            target = AbstractDungeon.getMonsters().monsters.get(a.target);
        }
        potion.use(target);
        AbstractDungeon.player.removePotion(potion);
        return true;
    }

    private static List<Object> eventOptions(AbstractEvent event) {
        List<Object> opts = new ArrayList<Object>();
        if (event.roomEventText != null && !event.roomEventText.optionList.isEmpty()) {
            opts.addAll(event.roomEventText.optionList);
        } else if (event.imageEventText != null && !event.imageEventText.optionList.isEmpty()) {
            opts.addAll(event.imageEventText.optionList);
        }
        return opts;
    }

    private static boolean pressEventOption(int choice) {
        AbstractRoom room = currRoom();
        if (room == null || room.event == null) {
            System.err.println("[TraceLab] no event room for choice");
            return false;
        }
        int available = eventOptions(room.event).size();
        if (choice < 0 || choice >= available) {
            System.err.println("[TraceLab] event choice " + choice + " out of range " + available);
            return false;
        }
        // Canonical automation path: RoomEventDialog exposes selectedOption /
        // waitForInput as static fields (events/RoomEventDialog.java:50-51). The
        // event's next update() fires buttonEffect(getSelectedOption()) when
        // waitForInput is false, without needing a hitbox click to register.
        RoomEventDialog.selectedOption = choice;
        RoomEventDialog.waitForInput = false;
        return true;
    }

    private static boolean choosePath(int choice) {
        List<MapRoomNode> options = availablePathNodes();
        if (choice < 0 || choice >= options.size()) {
            System.err.println("[TraceLab] PATH choice " + choice + " out of range " + options.size());
            return false;
        }
        MapRoomNode node = options.get(choice);
        System.out.println("[TraceLab] PATH -> (" + node.x + "," + node.y + ") "
                + node.getRoomSymbol(true));
        // The input path (hb.hovered + dungeonMapScreen.clicked) is unreachable:
        // DungeonMapScreen.updateMouse resets `clicked` from real input before
        // nodes read it. Instead trigger the node's own transition countdown by
        // setting its private animWaitTimer; MapRoomNode.update then runs the full
        // setCurrMapNode/nextRoomTransitionStart path (map/MapRoomNode.java:172).
        try {
            if (animWaitTimerField == null) {
                animWaitTimerField = MapRoomNode.class.getDeclaredField("animWaitTimer");
                animWaitTimerField.setAccessible(true);
            }
            animWaitTimerField.setFloat(node, 0.05f);
            return true;
        } catch (ReflectiveOperationException e) {
            System.err.println("[TraceLab] PATH reflection failed: " + e);
            return false;
        }
    }

    private static List<MapRoomNode> availablePathNodes() {
        List<MapRoomNode> options = new ArrayList<MapRoomNode>();
        MapRoomNode curr = AbstractDungeon.getCurrMapNode();
        if (curr == null || !AbstractDungeon.firstRoomChosen) {
            for (MapRoomNode node : AbstractDungeon.map.get(0)) {
                if (node.hasEdges()) {
                    options.add(node);
                }
            }
        } else {
            for (MapEdge edge : curr.getEdges()) {
                if (edge.dstY >= 0 && edge.dstY < AbstractDungeon.map.size()
                        && edge.dstX >= 0 && edge.dstX < AbstractDungeon.map.get(edge.dstY).size()) {
                    options.add(AbstractDungeon.map.get(edge.dstY).get(edge.dstX));
                }
            }
        }
        options.sort(Comparator.comparingInt(n -> n.x));
        return options;
    }

    private static void recordModeUpdate() {
        if (AbstractDungeon.screen == AbstractDungeon.CurrentScreen.MAP) {
            lastMapNode = AbstractDungeon.getCurrMapNode();
            lastMapOptions = availablePathNodes();
        }
        com.megacrit.cardcrawl.map.MapRoomNode now = AbstractDungeon.currMapNode;
        if (now != null && lastMapNode != null && now != lastMapNode && lastMapOptions != null) {
            int idx = -1;
            for (int i = 0; i < lastMapOptions.size(); i++) {
                if (lastMapOptions.get(i).x == now.x && lastMapOptions.get(i).y == now.y) {
                    idx = i;
                    break;
                }
            }
            Script.Action a = new Script.Action();
            a.type = "PATH";
            a.choice = idx >= 0 ? idx : null;
            a.choice_name = now.x + "," + now.y;
            lastMapNode = now;
            lastMapOptions = null;
            recordAction(a);
        }
        if (now != null && lastMapNode == null) {
            lastMapNode = now;
        }

        if (awaitingRecord && isQuiescent()) {
            stableFrames++;
            if (stableFrames >= STABLE_FRAMES_REQUIRED) {
                TraceWriter.writeRecord(recordIdx, lastAction);
                recordIdx++;
                awaitingRecord = false;
            }
        } else if (awaitingRecord) {
            stableFrames = 0;
        }

        if (script.stop != null && script.stop.max_floor != null
                && AbstractDungeon.floorNum > script.stop.max_floor) {
            finish("max_floor");
        }
    }

    private static Script.Action autoPolicy() {
        AbstractRoom room = currRoom();
        if (room != null && room.phase == AbstractRoom.RoomPhase.COMBAT) {
            int target = -1;
            if (AbstractDungeon.getMonsters() != null) {
                for (int i = 0; i < AbstractDungeon.getMonsters().monsters.size(); i++) {
                    AbstractMonster m = AbstractDungeon.getMonsters().monsters.get(i);
                    if (!m.isDead && !m.isDying && !m.escaped) {
                        target = i;
                        break;
                    }
                }
            }
            for (int i = 0; i < AbstractDungeon.player.hand.group.size(); i++) {
                AbstractCard c = AbstractDungeon.player.hand.group.get(i);
                AbstractMonster m = target >= 0
                        ? AbstractDungeon.getMonsters().monsters.get(target) : null;
                if (c.canUse(AbstractDungeon.player, m)) {
                    Script.Action a = new Script.Action();
                    a.type = "PLAY_CARD";
                    a.hand_idx = i;
                    a.target = target;
                    return a;
                }
            }
            Script.Action a = new Script.Action();
            a.type = "END_TURN";
            return a;
        }

        ChoiceScreenUtils.ChoiceType ct = safeChoiceType();
        Script.Action a = new Script.Action();
        switch (ct) {
            case MAP:
                a.type = "PATH";
                a.choice = 0;
                return a;
            case EVENT:
                a.type = (room != null && room.getClass().getSimpleName().equals("NeowRoom"))
                        ? "NEOW" : "EVENT_CHOICE";
                a.choice = 0;
                return a;
            case REST:
                a.type = "CHOOSE";
                a.choice = 0;
                return a;
            case GAME_OVER:
                finish("game_over");
                return null;
            case COMBAT_REWARD:
            case CARD_REWARD:
            case BOSS_REWARD:
            case CHEST:
            case SHOP_ROOM:
            case SHOP_SCREEN:
            case GRID:
            case HAND_SELECT:
            case COMPLETE:
            case NONE:
            default:
                if (ChoiceScreenUtils.isConfirmButtonAvailable()) {
                    a.type = "PROCEED";
                    return a;
                }
                if (ChoiceScreenUtils.isCancelButtonAvailable()) {
                    a.type = "CANCEL";
                    return a;
                }
                if (ct == ChoiceScreenUtils.ChoiceType.GRID
                        || ct == ChoiceScreenUtils.ChoiceType.HAND_SELECT) {
                    a.type = "CHOOSE";
                    a.choice = 0;
                    return a;
                }
                return null;
        }
    }

    private static void logState() {
        StringBuilder sb = new StringBuilder("[TraceLab] hb");
        sb.append(" floor=").append(AbstractDungeon.floorNum);
        sb.append(" screen=").append(AbstractDungeon.screen);
        AbstractRoom room = currRoom();
        sb.append(" room=").append(room == null ? "null" : room.getClass().getSimpleName());
        sb.append(" phase=").append(room == null || room.phase == null ? "-" : room.phase.name());
        if (room != null && room.event != null) {
            sb.append(" event=").append(room.event.getClass().getSimpleName());
            sb.append(" opts=").append(eventOptions(room.event).size());
            sb.append(" wait=").append(room.event.waitTimer);
        }
        GameActionManager am = AbstractDungeon.actionManager;
        if (am != null) {
            sb.append(" amPhase=").append(am.phase);
            sb.append(" acts=").append(am.actions.size());
            sb.append(" cardQ=").append(am.cardQueue.size());
            sb.append(" turnEnded=").append(am.turnHasEnded);
        }
        sb.append(" fading=").append(AbstractDungeon.isFadingOut).append('/').append(AbstractDungeon.isFadingIn);
        sb.append(" stable=").append(stableFrames);
        sb.append(" next=").append(nextAction);
        sb.append(" awaitRec=").append(awaitingRecord);
        System.out.println(sb);
    }

    private static void finish(String status) {
        if (finished) {
            return;
        }
        finished = true;
        active = false;
        System.out.println("[TraceLab] finished: " + status + " (recorded " + recordIdx + " actions)");
        TraceWriter.writeFinal(status, recordIdx, nextAction, script.actions.size());
        TraceWriter.close();
        if (!"0".equals(System.getProperty("tracelab.exit", "1"))) {
            Gdx.app.exit();
        }
    }
}
