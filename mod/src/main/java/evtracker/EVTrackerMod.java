package evtracker;

import basemod.BaseMod;
import basemod.interfaces.*;
import com.megacrit.cardcrawl.rooms.CampfireUI;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.evacipated.cardcrawl.modthespire.lib.SpireInitializer;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.core.Settings;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.FontHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.powers.AbstractPower;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.ui.panels.EnergyPanel;
import com.megacrit.cardcrawl.helpers.ImageMaster;

import evtracker.search.SearchClient;
import evtracker.search.SearchOverlay;
import evtracker.search.SearchResponse;
import evtracker.search.StateVerifier;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@SpireInitializer
public class EVTrackerMod implements
        PostInitializeSubscriber,
        OnCardUseSubscriber,
        OnPlayerTurnStartSubscriber,
        OnPlayerTurnStartPostDrawSubscriber,
        OnPlayerDamagedSubscriber,
        PostBattleSubscriber,
        OnStartBattleSubscriber,
        StartGameSubscriber,
        PostDeathSubscriber,
        PostDungeonInitializeSubscriber,
        PostRenderSubscriber,
        PostUpdateSubscriber,
        PostCampfireSubscriber,
        PostPotionUseSubscriber {  // Potion tracking via BaseMod

    public static final String MOD_ID = "evtracker";
    private static EVLogger logger;
    private static String currentRunId;
    private static int turnNumber;
    private static int battleNumber;

    // EV tracking stats
    private static int totalDamageDealt = 0;
    private static int totalDamageTaken = 0;
    private static int cardsPlayed = 0;
    private static int turnsPlayed = 0;
    private static int energySpent = 0;

    // Track last card target for accurate logging
    private static AbstractMonster lastCardTarget = null;

    public static void initialize() {
        new EVTrackerMod();
    }

    public EVTrackerMod() {
        BaseMod.subscribe(this);
        logger = new EVLogger();
    }

    @Override
    public void receivePostInitialize() {
        // Initialize debug overlay
        DebugOverlay.initialize();

        // Register console commands (evseed, evresetrun, evfloor, evkillall, evdumprng)
        ConsoleCommands.registerCommands();

        logger.log("system", "EVTracker initialized with console commands");
    }

    // ========== CARD TARGET TRACKING ==========

    /**
     * Set the target for the next card play.
     * Call this before receiveCardUsed to capture actual target.
     */
    public static void setLastCardTarget(AbstractMonster target) {
        lastCardTarget = target;
    }

    // ========== UPDATE LOOP ==========

    @Override
    public void receivePostUpdate() {
        // Update post-combat review UI animation
        PostCombatReviewUI.update();

        // Update infinite detector
        InfiniteDetector.update();

        // Update search overlay animation
        SearchOverlay.update(com.badlogic.gdx.Gdx.graphics.getDeltaTime());
    }

    // ========== RENDER EV PANEL ==========

    @Override
    public void receivePostRender(SpriteBatch sb) {
        // Render post-combat review (can show during any phase after combat)
        if (AbstractDungeon.player != null && PostCombatReviewUI.isShowing()) {
            PostCombatReviewUI.render(sb);
        }

        // Render card reward EV when on reward screen
        if (AbstractDungeon.player != null &&
            AbstractDungeon.screen == AbstractDungeon.CurrentScreen.CARD_REWARD) {
            CardRewardEV.render(sb);
        }

        // Combat-specific rendering: only during active combat
        if (AbstractDungeon.player == null ||
            AbstractDungeon.currMapNode == null ||
            AbstractDungeon.getCurrRoom() == null ||
            AbstractDungeon.getCurrRoom().phase != AbstractRoom.RoomPhase.COMBAT) {
            return;
        }

        // Render debug overlay (includes infinite button and detection)
        DebugOverlay.render(sb);

        // Render combat search overlay (shows best play recommendation)
        SearchOverlay.render(sb);

        // Calculate accurate damage using DamageCalculator
        int incomingDamage = DamageCalculator.calculateTotalIncomingDamage();
        int block = AbstractDungeon.player.currentBlock;
        int netDamage = DamageCalculator.calculateNetDamage();

        // Calculate turns to kill
        DamageCalculator.TurnsToKillResult ttk = DamageCalculator.calculateTurnsToKill();

        // Calculate EV: expected HP loss over remaining combat
        float combatEV = -ttk.expectedDamageTaken;
        float turnEV = -netDamage;

        // Get current modifiers
        int strength = DamageCalculator.getPlayerStrength();
        int dexterity = DamageCalculator.getPlayerDexterity();
        float stanceMult = DamageCalculator.getStanceDamageMultiplier();

        // Render EV panel - right side, below seed/mods area
        float panelWidth = 280 * Settings.scale;
        float x = Settings.WIDTH - panelWidth - 20 * Settings.scale;
        float y = Settings.HEIGHT - 180 * Settings.scale;  // Below mods/seed area
        BitmapFont font = FontHelper.tipBodyFont;

        // Semi-transparent background for readability
        sb.setColor(0, 0, 0, 0.7f);
        sb.draw(ImageMaster.WHITE_SQUARE_IMG, x - 10 * Settings.scale, y - 160 * Settings.scale,
                panelWidth + 20 * Settings.scale, 180 * Settings.scale);
        sb.setColor(Color.WHITE);

        // Title
        FontHelper.renderFontLeft(sb, font, "=== EV Tracker ===", x, y, Color.GOLD);
        y -= 25 * Settings.scale;

        // Stance warning (prominent)
        String stanceId = AbstractDungeon.player.stance != null ? AbstractDungeon.player.stance.ID : "Neutral";
        if (stanceId.equals("Wrath")) {
            FontHelper.renderFontLeft(sb, font, "!! WRATH - 2x DMG IN/OUT !!", x, y, Color.RED);
            y -= 20 * Settings.scale;
        } else if (stanceId.equals("Divinity")) {
            FontHelper.renderFontLeft(sb, font, "!!! DIVINITY - 3x DMG !!!", x, y, Color.GOLD);
            y -= 20 * Settings.scale;
        } else if (stanceId.equals("Calm")) {
            FontHelper.renderFontLeft(sb, font, "Calm (+2 energy on exit)", x, y, Color.CYAN);
            y -= 20 * Settings.scale;
        }

        // Incoming damage (with modifiers applied)
        Color dmgColor = netDamage > 0 ? Color.RED : Color.GREEN;
        FontHelper.renderFontLeft(sb, font,
            String.format("Incoming: %d (-%d net)", incomingDamage, netDamage),
            x, y, dmgColor);
        y -= 20 * Settings.scale;

        // Turn EV
        Color evColor = turnEV >= 0 ? Color.GREEN : Color.SALMON;
        FontHelper.renderFontLeft(sb, font,
            String.format("Turn EV: %.0f HP", turnEV),
            x, y, evColor);
        y -= 20 * Settings.scale;

        // Turns to kill
        String ttkStr = ttk.turnsToKill < 100 ? String.format("%.1f", ttk.turnsToKill) : "inf";
        FontHelper.renderFontLeft(sb, font,
            String.format("TTK: %s turns | Pot.Dmg: %d", ttkStr, ttk.potentialDamagePerTurn),
            x, y, Color.WHITE);
        y -= 20 * Settings.scale;

        // Combat EV (expected total HP loss)
        Color combatEvColor = combatEV >= -10 ? Color.GREEN : (combatEV >= -20 ? Color.YELLOW : Color.RED);
        FontHelper.renderFontLeft(sb, font,
            String.format("Combat EV: %.0f HP (est)", combatEV),
            x, y, combatEvColor);
        y -= 20 * Settings.scale;

        // Modifiers line
        StringBuilder modLine = new StringBuilder();
        if (strength != 0) modLine.append(String.format("Str:%+d ", strength));
        if (dexterity != 0) modLine.append(String.format("Dex:%+d ", dexterity));
        if (DamageCalculator.playerHasPower("Vulnerable")) modLine.append("VULN ");
        if (DamageCalculator.playerHasPower("Weak") || DamageCalculator.playerHasPower("Weakened")) modLine.append("WEAK ");
        if (DamageCalculator.playerHasPower("IntangiblePlayer")) modLine.append("INTANG ");

        if (modLine.length() > 0) {
            FontHelper.renderFontLeft(sb, font, modLine.toString().trim(), x, y, Color.LIGHT_GRAY);
            y -= 20 * Settings.scale;
        }

        // Best Line section (TODO: connect to Python tree search)
        y -= 5 * Settings.scale;  // Small gap
        FontHelper.renderFontLeft(sb, font, "--- Best Line ---", x, y, Color.CYAN);
        y -= 20 * Settings.scale;

        // TODO: Replace with actual tree search results from Python simulation
        // For now, show placeholder that will be populated via socket
        String bestLine = getBestLineRecommendation();
        FontHelper.renderFontLeft(sb, font, bestLine, x, y, Color.WHITE);
        y -= 20 * Settings.scale;

        // Efficiency stats
        float damageEfficiency = cardsPlayed > 0 ? (float) totalDamageDealt / cardsPlayed : 0;
        float damageTakenPerTurn = turnsPlayed > 0 ? (float) totalDamageTaken / turnsPlayed : 0;
        FontHelper.renderFontLeft(sb, font,
            String.format("Stats: %.1f dmg/card | %.1f taken/turn", damageEfficiency, damageTakenPerTurn),
            x, y, Color.GRAY);

        // Render per-card EV badges on hand
        CardEVOverlay.renderCardEVs(sb);
    }

    // Get tree search recommendation from Python server or fallback to heuristic
    private String getBestLineRecommendation() {
        // Check if we have a search result from Python
        SearchResponse response = SearchClient.getLatestResponse();
        if (response != null && response.isValid() && response.hasBestLine()) {
            return response.getFullDisplay();
        }

        // Check if search is in progress
        if (SearchClient.isSearchInProgress()) {
            return "Searching...";
        }

        // Check if we have cards in hand
        if (AbstractDungeon.player.hand.size() == 0) {
            return "No cards in hand";
        }

        // Simple heuristic fallback
        int energy = EnergyPanel.totalCount;
        if (energy == 0) {
            return "End turn (no energy)";
        }

        // Count playable attack/defense cards
        int attacks = 0, blocks = 0;
        for (AbstractCard card : AbstractDungeon.player.hand.group) {
            if (card.costForTurn <= energy) {
                if (card.type == AbstractCard.CardType.ATTACK) attacks++;
                else if (card.type == AbstractCard.CardType.SKILL && card.baseBlock > 0) blocks++;
            }
        }

        // Basic recommendation based on incoming damage
        int netDamage = DamageCalculator.calculateNetDamage();
        if (netDamage > AbstractDungeon.player.currentHealth * 0.3) {
            return String.format("Block priority (%d cards)", blocks);
        } else {
            return String.format("Attack priority (%d cards)", attacks);
        }
    }

    // ========== GAME FLOW HOOKS ==========

    @Override
    public void receiveStartGame() {
        currentRunId = String.valueOf(System.currentTimeMillis());
        battleNumber = 0;
        resetCombatStats();

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("character", CardCrawlGame.chosenCharacter != null ?
            CardCrawlGame.chosenCharacter.name() : "UNKNOWN");
        event.put("ascension", AbstractDungeon.ascensionLevel);
        event.put("seed", Settings.seed);

        logger.log("run_start", event);
    }

    @Override
    public void receivePostDungeonInitialize() {
        if (AbstractDungeon.player != null) {
            logPlayerState("dungeon_init");
        }
    }

    @Override
    public void receiveOnBattleStart(AbstractRoom room) {
        battleNumber++;
        turnNumber = 0;
        resetCombatStats();

        // Reset infinite detector and debug overlay for new combat
        InfiniteDetector.reset();
        DebugOverlay.reset();

        // Clear search state for new combat
        SearchClient.clearLatestResponse();
        StateVerifier.resetStats();

        // Initialize combat review tracking
        CombatReview.onBattleStart();

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("room_type", room.getClass().getSimpleName());
        event.put("player_state", getPlayerState());
        event.put("monsters", getMonsterStates());
        event.put("hand", getCardList(AbstractDungeon.player.hand.group));
        event.put("draw_pile_size", AbstractDungeon.player.drawPile.size());
        event.put("discard_pile_size", AbstractDungeon.player.discardPile.size());
        event.put("damage_modifiers", DamageCalculator.extractDamageModifiers());

        logger.log("battle_start", event);
    }

    @Override
    public void receiveOnPlayerTurnStart() {
        turnNumber++;
        turnsPlayed++;

        // Clear card EV cache for new turn
        CardEVOverlay.onTurnStart();

        // Reset infinite detector for new turn
        InfiniteDetector.onTurnStart();

        // Log turn start (before draw)
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("turn", turnNumber);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("player_state", getPlayerState());
        event.put("monsters", getMonsterStates());
        event.put("hand_pre_draw", getCardList(AbstractDungeon.player.hand.group));
        event.put("energy", EnergyPanel.totalCount);
        event.put("draw_pile_size", AbstractDungeon.player.drawPile.size());
        event.put("discard_pile_size", AbstractDungeon.player.discardPile.size());
        event.put("damage_modifiers", DamageCalculator.extractDamageModifiers());

        // Calculate incoming damage with all modifiers
        event.put("incoming_damage", DamageCalculator.calculateTotalIncomingDamage());
        event.put("net_damage", DamageCalculator.calculateNetDamage());

        logger.log("turn_start", event);
    }

    @Override
    public void receiveOnPlayerTurnStartPostDraw() {
        // Track energy for combat review
        CombatReview.onTurnStart(EnergyPanel.totalCount);

        // Capture full state for Python consumption (writes to /tmp/evtracker_state.json)
        TurnStateCapture.captureState();

        // Request search from Python server (async)
        SearchClient.requestSearch(response -> {
            // Update overlay when response arrives
            SearchOverlay.updateResults(response);

            // Store prediction for verification
            if (response != null && response.hasBestLine()) {
                StateVerifier.setPredictedState(response, response.getFullDisplay());
            }
        });

        // Log hand after draw completed
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("turn", turnNumber);
        event.put("hand_post_draw", getCardList(AbstractDungeon.player.hand.group));
        event.put("energy", EnergyPanel.totalCount);

        // Calculate potential damage from hand
        event.put("potential_damage", DamageCalculator.calculatePotentialHandDamage());

        // Calculate turns to kill
        DamageCalculator.TurnsToKillResult ttk = DamageCalculator.calculateTurnsToKill();
        Map<String, Object> ttkData = new HashMap<>();
        ttkData.put("turns", ttk.turnsToKill);
        ttkData.put("expected_damage_taken", ttk.expectedDamageTaken);
        ttkData.put("total_enemy_hp", ttk.totalEnemyHP);
        ttkData.put("potential_damage_per_turn", ttk.potentialDamagePerTurn);
        event.put("turns_to_kill", ttkData);

        logger.log("turn_start_post_draw", event);
    }

    @Override
    public void receiveCardUsed(AbstractCard card) {
        cardsPlayed++;
        int energyBefore = EnergyPanel.totalCount + card.costForTurn;
        energySpent += card.costForTurn;

        // Track for infinite detection
        InfiniteDetector.onCardPlayed(card);

        // Track for combat review - get target name
        String targetName = null;
        if (lastCardTarget != null) {
            targetName = lastCardTarget.name;
        } else if (card.target == AbstractCard.CardTarget.ALL_ENEMY) {
            targetName = "ALL";
        }
        CombatReview.onCardPlayed(card, targetName);

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("turn", turnNumber);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("card", getCardInfo(card));
        event.put("energy_before", energyBefore);
        event.put("energy_after", EnergyPanel.totalCount);
        event.put("player_state", getPlayerState());
        event.put("monsters", getMonsterStates());
        event.put("damage_modifiers", DamageCalculator.extractDamageModifiers());

        // Get actual target (use lastCardTarget if set, otherwise find hovered)
        AbstractMonster target = lastCardTarget;
        if (target == null && (card.target == AbstractCard.CardTarget.ENEMY ||
                               card.target == AbstractCard.CardTarget.SELF_AND_ENEMY)) {
            // Try to find the hovered monster
            target = findHoveredMonster();
        }

        if (target != null) {
            event.put("target", getMonsterState(target));
            event.put("target_modifiers", DamageCalculator.extractMonsterModifiers(target));

            // Calculate actual damage this card would deal
            if (card.type == AbstractCard.CardType.ATTACK) {
                int calculatedDamage = DamageCalculator.calculateCardDamage(card, target);
                event.put("calculated_damage", calculatedDamage);
                totalDamageDealt += calculatedDamage;
            }
        } else if (card.type == AbstractCard.CardType.ATTACK) {
            // AoE or no target - calculate against first monster
            AbstractMonster firstMonster = DamageCalculator.getFirstAliveMonster();
            if (firstMonster != null) {
                int calculatedDamage = DamageCalculator.calculateCardDamage(card, firstMonster);
                event.put("calculated_damage", calculatedDamage);
                totalDamageDealt += calculatedDamage;

                // For AoE, multiply by monster count
                if (card.target == AbstractCard.CardTarget.ALL_ENEMY ||
                    card.target == AbstractCard.CardTarget.ALL) {
                    int aliveCount = countAliveMonsters();
                    event.put("aoe_total_damage", calculatedDamage * aliveCount);
                }
            }
        }

        // Reset target tracking
        lastCardTarget = null;

        logger.log("card_played", event);

        // Verify predicted state matches actual (async, on next frame)
        StateVerifier.verifyAfterAction("card", card.cardID);
    }

    @Override
    public int receiveOnPlayerDamaged(int damage, com.megacrit.cardcrawl.cards.DamageInfo info) {
        totalDamageTaken += damage;

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("turn", turnNumber);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("damage_amount", damage);
        event.put("damage_type", info.type.name());
        event.put("player_hp_before", AbstractDungeon.player.currentHealth);
        event.put("player_block", AbstractDungeon.player.currentBlock);
        event.put("damage_modifiers", DamageCalculator.extractDamageModifiers());

        if (info.owner != null) {
            event.put("source", info.owner.name);
            if (info.owner instanceof AbstractMonster) {
                event.put("source_modifiers", DamageCalculator.extractMonsterModifiers((AbstractMonster) info.owner));
            }
        }

        logger.log("player_damaged", event);

        return damage; // Don't modify damage
    }

    @Override
    public void receivePostBattle(AbstractRoom room) {
        // Get combat review summary
        CombatReview.CombatSummary review = CombatReview.onBattleEnd();

        // Show post-combat review UI
        if (review != null) {
            PostCombatReviewUI.show(review);
        }

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("battle_number", battleNumber);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("room_type", room.getClass().getSimpleName());
        event.put("turns_taken", turnNumber);
        event.put("player_state", getPlayerState());
        event.put("victory", true);
        event.put("total_damage_dealt", totalDamageDealt);
        event.put("total_damage_taken", totalDamageTaken);
        event.put("cards_played", cardsPlayed);
        event.put("energy_spent", energySpent);

        // Efficiency metrics
        event.put("damage_per_card", cardsPlayed > 0 ? (float) totalDamageDealt / cardsPlayed : 0);
        event.put("damage_per_turn", turnsPlayed > 0 ? (float) totalDamageDealt / turnsPlayed : 0);
        event.put("damage_taken_per_turn", turnsPlayed > 0 ? (float) totalDamageTaken / turnsPlayed : 0);
        event.put("damage_per_energy", energySpent > 0 ? (float) totalDamageDealt / energySpent : 0);

        // Track if infinite was detected this combat
        event.put("infinite_detected", InfiniteDetector.isInfiniteDetected());

        // Combat review data
        if (review != null) {
            Map<String, Object> reviewData = new HashMap<>();
            reviewData.put("optimal_decisions", review.optimalDecisions);
            reviewData.put("suboptimal_decisions", review.suboptimalDecisions);
            reviewData.put("optimality_score", review.getOptimalityScore());
            reviewData.put("total_ev_lost", review.totalEVLost);
            reviewData.put("hp_lost", review.hpLost);
            reviewData.put("optimal_hp_lost", review.optimalHpLost);
            reviewData.put("key_mistakes", review.keyMistakes);
            event.put("combat_review", reviewData);
        }

        logger.log("battle_end", event);
    }

    @Override
    public void receivePostDeath() {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("player_state", getPlayerState());
        event.put("victory", false);
        event.put("total_damage_dealt", totalDamageDealt);
        event.put("total_damage_taken", totalDamageTaken);
        event.put("cards_played", cardsPlayed);

        if (battleNumber > 0) {
            event.put("battle_number", battleNumber);
            event.put("turn", turnNumber);
            event.put("monsters", getMonsterStates());
        }

        logger.log("run_end", event);
    }

    // ========== POTION TRACKING ==========

    @Override
    public void receivePostPotionUse(AbstractPotion potion) {
        // Track potion usage for combat review
        CombatReview.onPotionUsed(potion);

        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("battle_number", battleNumber);
        event.put("turn", turnNumber);
        event.put("potion_id", potion.ID);
        event.put("potion_name", potion.name);
        event.put("player_state", getPlayerState());

        // Log monster states if in combat
        if (AbstractDungeon.getCurrRoom() != null &&
            AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMBAT) {
            event.put("monsters", getMonsterStates());
        }

        logger.log("potion_used", event);
    }

    // ========== CAMPFIRE DECISION TRACKING ==========

    @Override
    public boolean receivePostCampfire() {
        // Log campfire decision with context
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("player_state", getPlayerState());
        event.put("deck", getCardList(AbstractDungeon.player.masterDeck.group));

        // Determine what action was taken by checking state changes
        // This is called AFTER the campfire action completes
        // We can't directly know which action was taken, but we log the post-state
        event.put("hp_after", AbstractDungeon.player.currentHealth);
        event.put("max_hp_after", AbstractDungeon.player.maxHealth);
        event.put("deck_size", AbstractDungeon.player.masterDeck.size());

        logger.log("campfire_action", event);

        return false; // Don't prevent other subscribers from running
    }

    // ========== CARD REWARD/UPGRADE/REMOVE TRACKING ==========

    /**
     * Called when a card is added to deck (rewards, events, etc.)
     */
    public static void onCardObtained(AbstractCard card, String source) {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("card", getCardInfoStatic(card));
        event.put("source", source);
        event.put("deck_size_after", AbstractDungeon.player.masterDeck.size());

        // Log alternatives if this was a card reward
        // (this would need to be called with the reward cards list)

        logger.log("card_obtained", event);
    }

    /**
     * Called when a card is upgraded
     */
    public static void onCardUpgraded(AbstractCard card, String source) {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("card", getCardInfoStatic(card));
        event.put("source", source);  // "campfire", "event", "relic", etc.

        logger.log("card_upgraded", event);
    }

    /**
     * Called when a card is removed from deck
     */
    public static void onCardRemoved(AbstractCard card, String source) {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("card", getCardInfoStatic(card));
        event.put("source", source);  // "shop", "event", etc.
        event.put("deck_size_after", AbstractDungeon.player.masterDeck.size());

        logger.log("card_removed", event);
    }

    /**
     * Called when a card reward is presented (for tracking what was skipped)
     */
    public static void onCardRewardPresented(List<AbstractCard> cards) {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);

        List<Map<String, Object>> cardList = new ArrayList<>();
        for (AbstractCard card : cards) {
            cardList.add(getCardInfoStatic(card));
        }
        event.put("offered_cards", cardList);

        logger.log("card_reward_presented", event);
    }

    // Static version of getCardInfo for use in static methods
    private static Map<String, Object> getCardInfoStatic(AbstractCard card) {
        Map<String, Object> info = new HashMap<>();
        info.put("id", card.cardID);
        info.put("name", card.name);
        info.put("type", card.type.name());
        info.put("cost", card.costForTurn);
        info.put("base_cost", card.cost);
        info.put("upgraded", card.upgraded);
        info.put("base_damage", card.baseDamage);
        info.put("base_block", card.baseBlock);
        info.put("magic_number", card.magicNumber);
        info.put("rarity", card.rarity.name());
        return info;
    }

    // ========== HELPER METHODS ==========

    private void resetCombatStats() {
        totalDamageDealt = 0;
        totalDamageTaken = 0;
        cardsPlayed = 0;
        turnsPlayed = 0;
        energySpent = 0;
        lastCardTarget = null;
    }

    private AbstractMonster findHoveredMonster() {
        if (AbstractDungeon.getMonsters() == null) return null;
        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            if (!m.isDead && !m.isDying && m.hb.hovered) {
                return m;
            }
        }
        return null;
    }

    private int countAliveMonsters() {
        int count = 0;
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    count++;
                }
            }
        }
        return count;
    }

    private void logPlayerState(String eventType) {
        Map<String, Object> event = new HashMap<>();
        event.put("run_id", currentRunId);
        event.put("floor", AbstractDungeon.floorNum);
        event.put("player_state", getPlayerState());
        event.put("deck", getCardList(AbstractDungeon.player.masterDeck.group));
        event.put("relics", getRelicList());
        event.put("potions", getPotionList());

        logger.log(eventType, event);
    }

    private Map<String, Object> getPlayerState() {
        AbstractPlayer p = AbstractDungeon.player;
        Map<String, Object> state = new HashMap<>();
        state.put("hp", p.currentHealth);
        state.put("max_hp", p.maxHealth);
        state.put("block", p.currentBlock);
        state.put("gold", p.gold);

        // Stance (Watcher)
        if (p.stance != null) {
            state.put("stance", p.stance.ID);
            state.put("stance_damage_mult", DamageCalculator.getStanceDamageMultiplier());
        }

        // Key powers with computed effects
        state.put("strength", DamageCalculator.getPlayerStrength());
        state.put("dexterity", DamageCalculator.getPlayerDexterity());
        state.put("vulnerable", DamageCalculator.getPlayerVulnerable());
        state.put("weak", DamageCalculator.getPlayerWeak());
        state.put("intangible", DamageCalculator.getPlayerIntangible());

        // All powers
        List<Map<String, Object>> powers = new ArrayList<>();
        for (AbstractPower power : p.powers) {
            Map<String, Object> powerInfo = new HashMap<>();
            powerInfo.put("id", power.ID);
            powerInfo.put("name", power.name);
            powerInfo.put("amount", power.amount);
            powerInfo.put("type", power.type.name());
            powers.add(powerInfo);
        }
        state.put("powers", powers);

        return state;
    }

    private List<Map<String, Object>> getMonsterStates() {
        List<Map<String, Object>> monsters = new ArrayList<>();
        if (AbstractDungeon.getMonsters() != null) {
            for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
                if (!m.isDead && !m.isDying) {
                    monsters.add(getMonsterState(m));
                }
            }
        }
        return monsters;
    }

    private Map<String, Object> getMonsterState(AbstractMonster m) {
        Map<String, Object> state = new HashMap<>();
        state.put("id", m.id);
        state.put("name", m.name);
        state.put("hp", m.currentHealth);
        state.put("max_hp", m.maxHealth);
        state.put("block", m.currentBlock);

        // Intent with calculated damage
        state.put("intent", m.intent.name());
        int baseDamage = m.getIntentDmg();
        if (baseDamage >= 0) {
            state.put("intent_base_damage", baseDamage);
            state.put("intent_multi", DamageCalculator.getMonsterIntentMulti(m));

            // Calculate actual incoming damage with all modifiers
            int actualDamage = DamageCalculator.calculateIncomingDamage(m);
            state.put("intent_calculated_damage", actualDamage);
        }

        // Key powers
        state.put("strength", DamageCalculator.getMonsterPowerAmount(m, "Strength"));
        state.put("vulnerable", DamageCalculator.getMonsterPowerAmount(m, "Vulnerable"));
        state.put("weak", DamageCalculator.getMonsterPowerAmount(m, "Weak"));

        // All powers
        List<Map<String, Object>> powers = new ArrayList<>();
        for (AbstractPower power : m.powers) {
            Map<String, Object> powerInfo = new HashMap<>();
            powerInfo.put("id", power.ID);
            powerInfo.put("name", power.name);
            powerInfo.put("amount", power.amount);
            powers.add(powerInfo);
        }
        state.put("powers", powers);

        return state;
    }

    private Map<String, Object> getCardInfo(AbstractCard card) {
        Map<String, Object> info = new HashMap<>();
        info.put("id", card.cardID);
        info.put("name", card.name);
        info.put("type", card.type.name());
        info.put("cost", card.costForTurn);
        info.put("base_cost", card.cost);
        info.put("upgraded", card.upgraded);

        // Damage with base and calculated values
        info.put("base_damage", card.baseDamage);
        info.put("display_damage", card.damage);  // After modifiers applied by game

        // Block values
        info.put("base_block", card.baseBlock);
        info.put("display_block", card.block);

        info.put("magic_number", card.magicNumber);
        info.put("exhausts", card.exhaust);
        info.put("ethereal", card.isEthereal);
        info.put("target_type", card.target.name());
        // Only check playability if we're in active combat
        try {
            if (AbstractDungeon.currMapNode != null &&
                AbstractDungeon.getCurrRoom() != null &&
                AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMBAT &&
                AbstractDungeon.player != null) {
                info.put("is_playable", card.canUse(AbstractDungeon.player, null));
            } else {
                info.put("is_playable", false);
            }
        } catch (Exception e) {
            info.put("is_playable", false);
        }

        return info;
    }

    private List<Map<String, Object>> getCardList(ArrayList<AbstractCard> cards) {
        List<Map<String, Object>> list = new ArrayList<>();
        for (AbstractCard card : cards) {
            list.add(getCardInfo(card));
        }
        return list;
    }

    private List<Map<String, Object>> getRelicList() {
        List<Map<String, Object>> list = new ArrayList<>();
        for (AbstractRelic relic : AbstractDungeon.player.relics) {
            Map<String, Object> info = new HashMap<>();
            info.put("id", relic.relicId);
            info.put("name", relic.name);
            info.put("counter", relic.counter);
            info.put("description", relic.description);
            list.add(info);
        }
        return list;
    }

    private List<Map<String, Object>> getPotionList() {
        List<Map<String, Object>> list = new ArrayList<>();
        for (AbstractPotion potion : AbstractDungeon.player.potions) {
            Map<String, Object> info = new HashMap<>();
            info.put("id", potion.ID);
            info.put("name", potion.name);
            // Only check canUse if we're in a valid room context
            if (AbstractDungeon.currMapNode != null && AbstractDungeon.getCurrRoom() != null) {
                info.put("can_use", potion.canUse());
            } else {
                info.put("can_use", false);
            }
            list.add(info);
        }
        return list;
    }
}
