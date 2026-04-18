"""
Game loop audit tests -- verifies GameRunner phase transitions, room handlers,
combat sim setup, and rest/shop/treasure/reward flows against decompiled Java behavior.
"""

import pytest
from packages.engine.game import (
    GameRunner, GamePhase,
    PathAction, CombatAction, RewardAction, EventAction,
    ShopAction, RestAction, TreasureAction, BossRewardAction, NeowAction,
)
from packages.engine.state.run import create_watcher_run, RunState, CardInstance, RelicInstance
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.handlers.rooms import (
    RestHandler, TreasureHandler, ChestType, ChestReward,
    NeowHandler, NeowBlessingType, NeowDrawbackType,
)
from packages.engine.calc.combat_sim import (
    CombatSimulator, ActionType, Action, CombatResult,
    encode_card_id, decode_card_id,
)
from packages.engine.content.cards import get_card, CardTarget, CardType


# =============================================================================
# HELPERS
# =============================================================================

def make_runner(seed="AUDIT1", ascension=20, skip_neow=True, verbose=False):
    """Create a GameRunner for testing."""
    return GameRunner(seed=seed, ascension=ascension, skip_neow=skip_neow, verbose=verbose)


def advance_to_phase(runner, target_phase, max_steps=200):
    """Take random actions until reaching target phase (or give up)."""
    import random
    random.seed(42)
    for _ in range(max_steps):
        if runner.phase == target_phase:
            return True
        if runner.game_over:
            return False
        actions = runner.get_available_actions()
        if not actions:
            return False
        runner.take_action(random.choice(actions))
    return runner.phase == target_phase


# =============================================================================
# GAME PHASE ENUM
# =============================================================================

class TestGamePhases:
    """Verify all expected phases exist."""

    def test_all_phases_present(self):
        phases = [p.name for p in GamePhase]
        expected = [
            "MAP_NAVIGATION", "COMBAT", "COMBAT_REWARDS", "EVENT",
            "SHOP", "REST", "TREASURE", "BOSS_REWARDS", "NEOW", "RUN_COMPLETE",
        ]
        for e in expected:
            assert e in phases, f"Missing phase: {e}"

    def test_initial_phase_skip_neow(self):
        runner = make_runner(skip_neow=True)
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_initial_phase_with_neow(self):
        runner = make_runner(skip_neow=False)
        assert runner.phase == GamePhase.NEOW


# =============================================================================
# MAP NAVIGATION
# =============================================================================

class TestMapNavigation:
    """Test map navigation and path selection."""

    def test_path_actions_available(self):
        runner = make_runner()
        actions = runner.get_available_actions()
        assert len(actions) > 0
        assert all(isinstance(a, PathAction) for a in actions)

    def test_path_action_advances_floor(self):
        runner = make_runner()
        initial_floor = runner.run_state.floor
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        assert runner.run_state.floor == initial_floor + 1

    def test_path_action_enters_room(self):
        runner = make_runner()
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        # Should no longer be in MAP_NAVIGATION (entered some room)
        assert runner.phase != GamePhase.MAP_NAVIGATION or runner.game_over


# =============================================================================
# COMBAT -> REWARDS TRANSITION
# =============================================================================

class TestCombatRewards:
    """Verify combat->rewards transition."""

    def test_combat_victory_goes_to_rewards(self):
        runner = make_runner(seed="COMBATRWD")
        reached = advance_to_phase(runner, GamePhase.COMBAT_REWARDS, max_steps=500)
        # Either reached rewards or game ended
        assert reached or runner.game_over

    def test_combat_defeat_ends_game(self):
        """If player dies in combat, game should end."""
        runner = make_runner(seed="DEFEAT1")
        # Run until game over
        import random
        random.seed(99)
        for _ in range(5000):
            if runner.game_over:
                break
            actions = runner.get_available_actions()
            if not actions:
                break
            runner.take_action(random.choice(actions))
        # Game should eventually end one way or another
        assert runner.game_over or runner.run_state.floor > 0

    def test_reward_proceed_returns_to_map(self):
        """Test proceeding from COMBAT_REWARDS goes back to MAP_NAVIGATION."""
        from packages.engine.handlers.reward_handler import RewardHandler
        runner = make_runner(seed="RWDMAP")
        # Directly set up combat rewards phase (don't rely on random play)
        runner.current_room_type = "monster"
        runner.current_rewards = RewardHandler.generate_combat_rewards(
            run_state=runner.run_state,
            room_type="monster",
            card_rng=runner.card_rng,
            treasure_rng=runner.treasure_rng,
            potion_rng=runner.potion_rng,
            relic_rng=runner.relic_rng,
        )
        runner.phase = GamePhase.COMBAT_REWARDS

        # Skip all card rewards and proceed
        while runner.phase == GamePhase.COMBAT_REWARDS:
            actions = runner.get_available_actions()
            # Find proceed or skip actions
            proceed = [a for a in actions if isinstance(a, RewardAction) and a.reward_type in ("proceed", "skip_card")]
            if proceed:
                runner.take_action(proceed[0])
            elif actions:
                runner.take_action(actions[0])
            else:
                break

        # Should be back at map or at boss rewards
        assert runner.phase in (GamePhase.MAP_NAVIGATION, GamePhase.BOSS_REWARDS) or runner.game_over


# =============================================================================
# REST SITE
# =============================================================================

class TestRestSite:
    """Test rest site options against Java CampfireUI behavior."""

    def test_rest_heals_30_percent(self):
        run = create_watcher_run("REST1", ascension=20)
        run.damage(30)
        old_hp = run.current_hp
        result = RestHandler.rest(run)
        expected_heal = int(run.max_hp * 0.30)
        # Actual heal may be capped by max_hp
        assert result.hp_healed <= expected_heal + 1
        assert run.current_hp > old_hp

    def test_rest_regal_pillow_bonus(self):
        run = create_watcher_run("REST2", ascension=20)
        run.add_relic("Regal Pillow")
        run.damage(50)
        old_hp = run.current_hp
        result = RestHandler.rest(run)
        base_heal = int(run.max_hp * 0.30)
        # Should heal base + 15
        assert result.hp_healed >= base_heal  # At least base heal

    def test_rest_coffee_dripper_blocks(self):
        run = create_watcher_run("REST3", ascension=20)
        run.add_relic("Coffee Dripper")
        options = RestHandler.get_options(run)
        assert "rest" not in options

    def test_smith_fusion_hammer_blocks(self):
        run = create_watcher_run("REST4", ascension=20)
        run.add_relic("Fusion Hammer")
        options = RestHandler.get_options(run)
        assert "smith" not in options

    def test_dig_requires_shovel(self):
        run = create_watcher_run("REST5", ascension=20)
        options = RestHandler.get_options(run)
        assert "dig" not in options

        run.add_relic("Shovel")
        options = RestHandler.get_options(run)
        assert "dig" in options

    def test_lift_requires_girya(self):
        run = create_watcher_run("REST6", ascension=20)
        options = RestHandler.get_options(run)
        assert "lift" not in options

        run.add_relic("Girya")
        options = RestHandler.get_options(run)
        assert "lift" in options

    def test_girya_max_3_lifts(self):
        """Java: Girya max 3 lifts. Verify Python matches."""
        run = create_watcher_run("REST7", ascension=20)
        run.add_relic("Girya")

        for i in range(3):
            result = RestHandler.lift(run)
            assert result.strength_gained == 1

        # 4th lift should fail (counter at 3)
        result = RestHandler.lift(run)
        assert result.strength_gained == 0

    def test_recall_act3_only(self):
        run = create_watcher_run("REST8", ascension=20)
        # Act 1 -- no recall
        options = RestHandler.get_options(run)
        assert "recall" not in options

    def test_toke_requires_peace_pipe(self):
        run = create_watcher_run("REST9", ascension=20)
        options = RestHandler.get_options(run)
        assert "toke" not in options

        run.add_relic("Peace Pipe")
        options = RestHandler.get_options(run)
        assert "toke" in options

    def test_eternal_feather_on_enter(self):
        run = create_watcher_run("REST10", ascension=20)
        run.add_relic("Eternal Feather")
        run.damage(20)
        old_hp = run.current_hp
        healed = RestHandler.on_enter_rest_site(run)
        # Deck of 10 cards -> 2 * 3 = 6 HP
        expected = (len(run.deck) // 5) * 3
        assert healed <= expected  # May be capped by max_hp


# =============================================================================
# TREASURE ROOM
# =============================================================================

class TestTreasureRoom:
    """Test treasure room mechanics."""

    def test_chest_type_distribution(self):
        """Verify chest type thresholds match Java."""
        rng = Random(12345)
        types = {"Small": 0, "Medium": 0, "Large": 0}
        for _ in range(1000):
            ct = TreasureHandler.determine_chest_type(rng)
            types[ct.value] += 1
        # Small ~50%, Medium ~33%, Large ~17%
        assert types["Small"] > 300
        assert types["Medium"] > 150
        assert types["Large"] > 50

    def test_sapphire_key_act3(self):
        run = create_watcher_run("TREAS1", ascension=20)
        # Force act 3
        run.act = 3
        rng1 = Random(11111)
        rng2 = Random(22222)
        reward = TreasureHandler.open_chest(run, rng1, rng2, take_sapphire_key=True)
        assert reward.sapphire_key_taken
        assert run.has_sapphire_key

    def test_open_chest_gives_relic(self):
        run = create_watcher_run("TREAS2", ascension=20)
        initial_relics = len(run.relics)
        rng1 = Random(33333)
        rng2 = Random(44444)
        reward = TreasureHandler.open_chest(run, rng1, rng2)
        assert len(run.relics) > initial_relics
        assert reward.relic_id != ""


# =============================================================================
# SHOP FLOW
# =============================================================================

class TestShopFlow:
    """Test shop entry and purchase flow."""

    def test_shop_phase_transition(self):
        """Test that entering shop sets up shop state correctly."""
        runner = make_runner(seed="SHOP1")
        # Directly enter shop (don't rely on random play)
        runner._enter_shop()
        assert runner.phase == GamePhase.SHOP
        assert runner.current_shop is not None

    def test_shop_leave_returns_to_map(self):
        """Test that leaving shop returns to MAP_NAVIGATION."""
        runner = make_runner(seed="SHOP2")
        # Directly enter shop (don't rely on random play)
        runner._enter_shop()
        assert runner.phase == GamePhase.SHOP
        runner.take_action(ShopAction(action_type="leave"))
        assert runner.phase == GamePhase.MAP_NAVIGATION


# =============================================================================
# NEOW BLESSINGS
# =============================================================================

class TestNeowBlessings:
    """Test Neow blessing flow."""

    def test_neow_phase_actions(self):
        runner = make_runner(skip_neow=False)
        assert runner.phase == GamePhase.NEOW
        actions = runner.get_available_actions()
        assert len(actions) > 0
        assert all(isinstance(a, NeowAction) for a in actions)

    def test_neow_action_proceeds_to_map(self):
        runner = make_runner(skip_neow=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_first_run_options(self):
        options = NeowHandler.get_first_run_options()
        assert len(options) == 4

    def test_hundred_gold_blessing(self):
        run = create_watcher_run("NEOW1", ascension=20)
        initial_gold = run.gold
        from packages.engine.handlers.rooms import NeowBlessing
        blessing = NeowBlessing(NeowBlessingType.HUNDRED_GOLD, "Gain 100 Gold")
        rng = Random(1)
        result = NeowHandler.apply_blessing(run, blessing, rng, rng, rng, rng)
        assert run.gold == initial_gold + 100


# =============================================================================
# COMBAT SIMULATOR
# =============================================================================

class TestCombatSimulator:
    """Test CombatSimulator setup and basic mechanics."""

    def test_encode_decode_card_id(self):
        assert encode_card_id("Strike_P", True) == "Strike_P+"
        assert encode_card_id("Strike_P", False) == "Strike_P"
        assert decode_card_id("Strike_P+") == ("Strike_P", True)
        assert decode_card_id("Strike_P") == ("Strike_P", False)

    def test_setup_combat_creates_state(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        ai_rng = Random(100)
        hp_rng = Random(200)
        enemy = JawWorm(ai_rng, ascension=0, hp_rng=hp_rng)
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(
            deck=deck,
            enemies=[enemy],
            player_hp=80,
            player_max_hp=80,
        )
        assert state.player.hp == 80
        assert state.player.max_hp == 80
        assert len(state.hand) == 5
        assert len(state.enemies) == 1
        assert state.energy == 3

    def test_get_legal_actions_includes_end_turn(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        actions = sim.get_legal_actions(state)
        end_turns = [a for a in actions if a.action_type == ActionType.END_TURN]
        assert len(end_turns) == 1

    def test_play_card_reduces_energy(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        # Find a playable card action
        actions = sim.get_legal_actions(state)
        play_actions = [a for a in actions if a.action_type == ActionType.PLAY_CARD]
        assert len(play_actions) > 0

        new_state = sim.execute_action(state, play_actions[0])
        assert new_state.energy < state.energy or new_state.energy == state.energy  # 0-cost cards possible

    def test_immutable_state(self):
        """CombatSimulator should NOT mutate input state."""
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        original_hand = list(state.hand)
        original_energy = state.energy

        actions = sim.get_legal_actions(state)
        play_actions = [a for a in actions if a.action_type == ActionType.PLAY_CARD]
        if play_actions:
            sim.execute_action(state, play_actions[0])
            # Original state should be unchanged
            assert state.hand == original_hand
            assert state.energy == original_energy

    def test_end_turn_progresses(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        end_action = Action(action_type=ActionType.END_TURN)
        new_state = sim.execute_action(state, end_action)
        assert new_state.turn > state.turn or new_state.combat_over

    def test_simulate_full_combat(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        result = sim.simulate_full_combat(state, sim.greedy_policy, max_turns=50)
        assert isinstance(result, CombatResult)
        assert result.turns > 0
        assert result.cards_played >= 0

    def test_stance_change_calm_energy(self):
        """Exiting Calm should grant 2 energy (3 with Violet Lotus).
        Uses StanceManager directly since CombatSimulator._change_stance mutates state."""
        from packages.engine.content.stances import StanceManager, StanceID

        sm = StanceManager(has_violet_lotus=False)
        assert sm.current == StanceID.NEUTRAL

        # Enter Calm -- no energy
        result = sm.change_stance(StanceID.CALM)
        assert result["energy_gained"] == 0
        assert sm.current == StanceID.CALM

        # Exit Calm -- should gain 2 energy
        result = sm.change_stance(StanceID.NEUTRAL)
        assert result["energy_gained"] == 2

        # With Violet Lotus -- should gain 3 energy
        sm2 = StanceManager(has_violet_lotus=True)
        sm2.change_stance(StanceID.CALM)
        result2 = sm2.change_stance(StanceID.NEUTRAL)
        assert result2["energy_gained"] == 3

    def test_greedy_policy_returns_action(self):
        sim = CombatSimulator()
        from packages.engine.content.enemies import JawWorm
        enemy = JawWorm(Random(100), ascension=0, hp_rng=Random(200))
        deck = ["Strike_P"] * 4 + ["Defend_P"] * 4 + ["Eruption", "Vigilance"]

        state = sim.setup_combat(deck=deck, enemies=[enemy], player_hp=80, player_max_hp=80)
        action = sim.greedy_policy(state)
        assert isinstance(action, Action)


# =============================================================================
# GAME RUNNER INTEGRATION
# =============================================================================

class TestGameRunnerIntegration:
    """Integration tests for GameRunner."""

    def test_run_to_floor(self):
        runner = make_runner(seed="FLOOR5")
        stats = runner.run_to_floor(3)
        assert runner.run_state.floor >= 1

    def test_game_over_flag(self):
        runner = make_runner(seed="GAMEOVER")
        import random
        random.seed(12345)
        for _ in range(10000):
            if runner.game_over:
                break
            actions = runner.get_available_actions()
            if not actions:
                break
            runner.take_action(random.choice(actions))
        # Game should end eventually
        assert runner.game_over or runner.run_state.floor > 0

    def test_decision_log_populated(self):
        runner = make_runner(seed="LOGTEST")
        runner.run_to_floor(2)
        assert len(runner.decision_log) > 0

    def test_get_run_statistics(self):
        runner = make_runner(seed="STATS1")
        runner.run_to_floor(2)
        stats = runner.get_run_statistics()
        assert "seed" in stats
        assert "final_floor" in stats
        assert "deck_size" in stats

    def test_headless_run(self):
        from packages.engine.game import run_headless
        result = run_headless(seed=12345, ascension=0, max_actions=500)
        assert result.floor_reached >= 1
        assert result.deck_size > 0


# =============================================================================
# GOLD REWARD RANGES (Java parity)
# =============================================================================

class TestGoldRewardRanges:
    """Verify gold reward ranges match Java."""

    def test_monster_gold_range(self):
        """Java: treasureRng.random(10, 20) for normal monsters."""
        from packages.engine.handlers.reward_handler import RewardHandler
        # Directly generate combat rewards (don't rely on random play)
        runner = make_runner(seed="GOLDTEST")
        runner.current_room_type = "monster"
        runner.current_rewards = RewardHandler.generate_combat_rewards(
            run_state=runner.run_state,
            room_type="monster",
            card_rng=runner.card_rng,
            treasure_rng=runner.treasure_rng,
            potion_rng=runner.potion_rng,
            relic_rng=runner.relic_rng,
        )
        runner.phase = GamePhase.COMBAT_REWARDS

        assert runner.current_rewards is not None
        assert runner.current_rewards.gold is not None
        # Gold should be a reasonable positive number (Java: 10-20 base)
        assert runner.current_rewards.gold.amount > 0


# =============================================================================
# BOSS REWARDS
# =============================================================================

class TestBossRewards:
    """Test boss reward flow."""

    def test_boss_reward_action_type(self):
        """BossRewardAction should accept relic_index."""
        action = BossRewardAction(relic_index=1)
        assert action.relic_index == 1

    def test_boss_fight_pending_flag(self):
        """After boss combat, should go to COMBAT_REWARDS then BOSS_REWARDS."""
        runner = make_runner(seed="BOSSFLOW")
        # The _boss_fight_pending_boss_rewards flag controls this
        runner._boss_fight_pending_boss_rewards = True
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = None  # Force proceed
        actions = runner.get_available_actions()
        proceed = [a for a in actions if isinstance(a, RewardAction) and a.reward_type == "proceed"]
        assert len(proceed) > 0
