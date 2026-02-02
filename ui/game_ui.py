"""
Game UI - DearPyGui wrapper for GameRunner.

Provides a visual interface for playing/watching Slay the Spire runs
using the GameRunner simulation engine.

Usage:
    uv run python -m ui.game_ui
"""

from __future__ import annotations

from typing import Optional, List, Dict, Any
import dearpygui.dearpygui as dpg

import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from core.game import (
    GameRunner, GamePhase, GameAction,
    PathAction, NeowAction, CombatAction, RewardAction,
    EventAction, ShopAction, RestAction, TreasureAction, BossRewardAction
)
from core.generation.map import RoomType, map_to_string


# =============================================================================
# Constants
# =============================================================================

WINDOW_WIDTH = 1200
WINDOW_HEIGHT = 800

# Colors (RGBA 0-255)
COLOR_HP = (200, 50, 50, 255)
COLOR_HP_BG = (80, 30, 30, 255)
COLOR_GOLD = (255, 215, 0, 255)
COLOR_ENERGY = (255, 200, 50, 255)
COLOR_BLOCK = (100, 150, 200, 255)
COLOR_TEXT = (220, 220, 220, 255)
COLOR_MUTED = (150, 150, 150, 255)
COLOR_ACTION_BTN = (60, 100, 140, 255)
COLOR_ACTION_BTN_HOVER = (80, 130, 180, 255)

# Room type colors for map
ROOM_COLORS = {
    RoomType.MONSTER: (100, 100, 100, 255),
    RoomType.ELITE: (180, 50, 50, 255),
    RoomType.REST: (50, 180, 50, 255),
    RoomType.SHOP: (255, 215, 0, 255),
    RoomType.EVENT: (150, 100, 200, 255),
    RoomType.TREASURE: (255, 180, 50, 255),
    RoomType.BOSS: (200, 0, 0, 255),
}


# =============================================================================
# Game UI Class
# =============================================================================

class GameUI:
    """
    DearPyGui-based UI for GameRunner.

    Provides visual interface for:
    - Game state display (HP, gold, floor, etc.)
    - Map navigation
    - Combat view
    - Action selection
    - Deck display
    """

    def __init__(self):
        """Initialize the UI (but don't start the game yet)."""
        self.runner: Optional[GameRunner] = None

        # UI state
        self._action_buttons: List[int] = []
        self._log_messages: List[str] = []

        # DearPyGui setup
        dpg.create_context()
        self._setup_theme()
        self._setup_fonts()
        self._build_ui()

    def _setup_theme(self):
        """Set up the visual theme."""
        with dpg.theme() as self.global_theme:
            with dpg.theme_component(dpg.mvAll):
                dpg.add_theme_style(dpg.mvStyleVar_FrameRounding, 4)
                dpg.add_theme_style(dpg.mvStyleVar_WindowRounding, 6)
                dpg.add_theme_style(dpg.mvStyleVar_FramePadding, 8, 6)
                dpg.add_theme_style(dpg.mvStyleVar_ItemSpacing, 10, 8)

        with dpg.theme() as self.action_btn_theme:
            with dpg.theme_component(dpg.mvButton):
                dpg.add_theme_color(dpg.mvThemeCol_Button, COLOR_ACTION_BTN)
                dpg.add_theme_color(dpg.mvThemeCol_ButtonHovered, COLOR_ACTION_BTN_HOVER)
                dpg.add_theme_style(dpg.mvStyleVar_FrameRounding, 6)

    def _setup_fonts(self):
        """Set up fonts (uses default for now)."""
        # Could add custom fonts here if needed
        pass

    def _build_ui(self):
        """Build the main UI layout."""
        dpg.create_viewport(
            title="Slay the Spire RL",
            width=WINDOW_WIDTH,
            height=WINDOW_HEIGHT,
            resizable=True
        )

        with dpg.window(tag="main_window"):
            # Top section: Seed input / Start game
            with dpg.group(horizontal=True):
                dpg.add_text("Seed:")
                dpg.add_input_text(
                    tag="seed_input",
                    default_value="TEST123",
                    width=150,
                    on_enter=True,
                    callback=self._on_start_game
                )
                dpg.add_text("Ascension:")
                dpg.add_input_int(
                    tag="ascension_input",
                    default_value=20,
                    width=80,
                    min_value=0,
                    max_value=20,
                    min_clamped=True,
                    max_clamped=True
                )
                dpg.add_button(
                    label="Start Game",
                    callback=self._on_start_game
                )
                dpg.add_button(
                    label="Random Action",
                    tag="random_action_btn",
                    callback=self._on_random_action,
                    enabled=False
                )
                dpg.add_button(
                    label="Auto-Play 10",
                    tag="auto_play_btn",
                    callback=self._on_auto_play,
                    enabled=False
                )

            dpg.add_separator()

            # Main content: two columns
            with dpg.group(horizontal=True):
                # Left column: Game state
                with dpg.child_window(tag="state_panel", width=350, height=450):
                    dpg.add_text("GAME STATE", color=COLOR_MUTED)
                    dpg.add_separator()

                    # HP bar
                    with dpg.group(horizontal=True):
                        dpg.add_text("HP:", color=COLOR_HP)
                        dpg.add_text("--/--", tag="hp_text")
                    dpg.add_progress_bar(
                        tag="hp_bar",
                        default_value=1.0,
                        width=-1,
                        overlay="HP"
                    )

                    dpg.add_spacer(height=10)

                    # Gold, Floor, Act, Phase
                    with dpg.group(horizontal=True):
                        dpg.add_text("Gold:", color=COLOR_GOLD)
                        dpg.add_text("--", tag="gold_text")

                    with dpg.group(horizontal=True):
                        dpg.add_text("Floor:")
                        dpg.add_text("--", tag="floor_text")

                    with dpg.group(horizontal=True):
                        dpg.add_text("Act:")
                        dpg.add_text("--", tag="act_text")

                    with dpg.group(horizontal=True):
                        dpg.add_text("Phase:")
                        dpg.add_text("--", tag="phase_text", color=COLOR_MUTED)

                    dpg.add_separator()

                    # Relics
                    dpg.add_text("RELICS", color=COLOR_MUTED)
                    dpg.add_text("--", tag="relics_text", wrap=340)

                    dpg.add_separator()

                    # Potions
                    dpg.add_text("POTIONS", color=COLOR_MUTED)
                    dpg.add_text("--", tag="potions_text")

                    dpg.add_separator()

                    # Keys
                    dpg.add_text("KEYS", color=COLOR_MUTED)
                    dpg.add_text("--", tag="keys_text")

                # Right column: Map/Combat view
                with dpg.child_window(tag="main_panel", width=-1, height=450):
                    dpg.add_text("MAP / COMBAT VIEW", color=COLOR_MUTED)
                    dpg.add_separator()

                    # Map display (text-based for now)
                    dpg.add_text(
                        "Start a game to see the map",
                        tag="map_text",
                        wrap=-1
                    )

                    dpg.add_separator()

                    # Combat info (when in combat)
                    dpg.add_text("", tag="combat_info", wrap=-1)

            dpg.add_separator()

            # Actions panel
            with dpg.child_window(tag="actions_panel", height=120):
                dpg.add_text("AVAILABLE ACTIONS", color=COLOR_MUTED)
                dpg.add_separator()
                with dpg.group(tag="actions_group", horizontal=True):
                    dpg.add_text("Start a game to see actions", tag="actions_placeholder")

            dpg.add_separator()

            # Deck panel
            with dpg.child_window(tag="deck_panel", height=100):
                dpg.add_text("DECK", color=COLOR_MUTED)
                dpg.add_separator()
                dpg.add_text("--", tag="deck_text", wrap=-1)

            # Log panel (collapsible)
            with dpg.collapsing_header(label="Game Log", default_open=False):
                dpg.add_text("", tag="log_text", wrap=-1)

        dpg.bind_theme(self.global_theme)
        dpg.set_primary_window("main_window", True)

    # =========================================================================
    # Game Control Methods
    # =========================================================================

    def start_game(self, seed: str, ascension: int = 20):
        """
        Start a new game with the given seed.

        Args:
            seed: Seed string (e.g., "TEST123")
            ascension: Ascension level (0-20)
        """
        self.runner = GameRunner(seed=seed, ascension=ascension, verbose=False, skip_neow=False)
        self._log(f"=== Game Started ===")
        self._log(f"Seed: {seed}, Ascension: {ascension}")
        self._update_display()

        # Enable action buttons
        dpg.configure_item("random_action_btn", enabled=True)
        dpg.configure_item("auto_play_btn", enabled=True)

    def take_action(self, action_idx: int):
        """
        Take an action by index.

        Args:
            action_idx: Index into available actions list
        """
        if not self.runner or self.runner.game_over:
            return

        actions = self.runner.get_available_actions()
        if action_idx < 0 or action_idx >= len(actions):
            self._log(f"Invalid action index: {action_idx}")
            return

        action = actions[action_idx]
        self._log(f"Action: {self._action_to_string(action)}")
        self.runner.take_action(action)
        self._update_display()

    def take_random_action(self):
        """Take a random valid action."""
        if not self.runner or self.runner.game_over:
            return

        import random
        actions = self.runner.get_available_actions()
        if actions:
            action = random.choice(actions)
            self._log(f"Random: {self._action_to_string(action)}")
            self.runner.take_action(action)
            self._update_display()

    # =========================================================================
    # Display Update Methods
    # =========================================================================

    def _update_display(self):
        """Update all UI elements to reflect current game state."""
        if not self.runner:
            return

        state = self.runner.run_state

        # Update HP
        hp_text = f"{state.current_hp}/{state.max_hp}"
        hp_ratio = state.current_hp / state.max_hp if state.max_hp > 0 else 0
        dpg.set_value("hp_text", hp_text)
        dpg.set_value("hp_bar", hp_ratio)

        # Update resources
        dpg.set_value("gold_text", str(state.gold))
        dpg.set_value("floor_text", str(state.floor))
        dpg.set_value("act_text", str(state.act))
        dpg.set_value("phase_text", self.runner.phase.name)

        # Update relics
        if state.relics:
            relic_str = ", ".join(str(r) for r in state.relics)
        else:
            relic_str = "None"
        dpg.set_value("relics_text", relic_str)

        # Update potions
        potions = state.get_potions()
        if potions:
            potion_str = ", ".join(potions)
        else:
            potion_str = f"Empty ({state.count_empty_potion_slots()} slots)"
        dpg.set_value("potions_text", potion_str)

        # Update keys
        keys = []
        if state.has_ruby_key:
            keys.append("Ruby")
        if state.has_emerald_key:
            keys.append("Emerald")
        if state.has_sapphire_key:
            keys.append("Sapphire")
        dpg.set_value("keys_text", ", ".join(keys) if keys else "None")

        # Update map/combat view
        self._update_map_view()

        # Update actions
        self._update_actions()

        # Update deck
        self._update_deck()

        # Check game over
        if self.runner.game_over:
            stats = self.runner.get_run_statistics()
            if self.runner.game_won:
                self._log(f"=== VICTORY === Floor {stats['final_floor']}, {stats['deck_size']} cards, {stats['relic_count']} relics")
            else:
                self._log(f"=== DEFEAT === Floor {stats['final_floor']}, HP: {stats['final_hp']}/{stats['final_max_hp']}")
            dpg.configure_item("random_action_btn", enabled=False)
            dpg.configure_item("auto_play_btn", enabled=False)

    def _update_map_view(self):
        """Update the map/combat view panel."""
        if not self.runner:
            return

        state = self.runner.run_state
        phase = self.runner.phase

        # Build map text
        current_map = state.get_current_map()
        if current_map:
            map_str = map_to_string(current_map)
            pos = state.map_position
            position_str = f"\nCurrent position: ({pos.x}, {pos.y})" if hasattr(pos, 'x') else f"\nPosition: {pos}"

            # Show visited path
            visited = getattr(state, 'visited_nodes', None)
            visited_str = ""
            if visited:
                visited_str = f"\nVisited: {len(visited)} nodes"

            # Add available paths info
            paths = state.get_available_paths()
            if paths:
                path_info = "\n\nNext rooms (choose one):"
                for i, node in enumerate(paths):
                    path_info += f"\n  [{i}] {node.room_type.name} at ({node.x}, {node.y})"
            else:
                path_info = ""

            dpg.set_value("map_text", f"Act {state.act} Map:\n{map_str}{position_str}{visited_str}{path_info}")
        else:
            dpg.set_value("map_text", "No map generated")

        # Update combat info based on phase
        combat_info = ""
        if phase == GamePhase.COMBAT:
            combat_info = self._build_combat_info()
        elif phase == GamePhase.COMBAT_REWARDS:
            combat_info = self._build_rewards_info()
        elif phase == GamePhase.EVENT:
            combat_info = self._build_event_info()
        elif phase == GamePhase.SHOP:
            combat_info = self._build_shop_info()
        elif phase == GamePhase.REST:
            combat_info = f"REST SITE\nCurrent HP: {state.current_hp}/{state.max_hp}"
        elif phase == GamePhase.TREASURE:
            combat_info = "TREASURE ROOM"
        elif phase == GamePhase.BOSS_REWARDS:
            combat_info = self._build_boss_rewards_info()
        elif phase == GamePhase.NEOW:
            combat_info = self._build_neow_info()
        elif phase == GamePhase.RUN_COMPLETE:
            combat_info = self._build_game_over_info()

        dpg.set_value("combat_info", combat_info)

    def _build_combat_info(self) -> str:
        """Build combat display string."""
        runner = self.runner
        if not runner or not runner.current_combat:
            return "IN COMBAT (no engine)"

        cs = runner.current_combat.state
        lines = ["=== COMBAT ==="]
        lines.append(f"Turn: {cs.turn}  |  Energy: {cs.energy}/{cs.max_energy}  |  Stance: {cs.stance}")
        lines.append(f"Player: {cs.player.hp}/{cs.player.max_hp} HP  |  Block: {cs.player.block}")

        # Player statuses
        statuses = [f"{k}: {v}" for k, v in cs.player.statuses.items() if v != 0]
        if statuses:
            lines.append(f"Statuses: {', '.join(statuses)}")

        lines.append("")

        # Enemies
        for i, e in enumerate(cs.enemies):
            alive = "DEAD" if e.is_dead else ""
            if e.is_dead:
                lines.append(f"  [{i}] {e.name} - DEAD")
                continue
            intent = ""
            if e.move_damage > 0:
                dmg = e.move_damage
                hits = e.move_hits
                intent = f"ATK {dmg}x{hits}" if hits > 1 else f"ATK {dmg}"
            elif e.move_block > 0:
                intent = f"BLOCK {e.move_block}"
            else:
                intent = "BUFF/DEBUFF"
            enemy_statuses = [f"{k}:{v}" for k, v in e.statuses.items() if v != 0]
            status_str = f"  ({', '.join(enemy_statuses)})" if enemy_statuses else ""
            lines.append(f"  [{i}] {e.name}: {e.hp}/{e.max_hp} HP  Block:{e.block}  Intent:{intent}{status_str}")

        lines.append("")

        # Hand
        lines.append(f"Hand ({len(cs.hand)} cards):")
        for i, card_id in enumerate(cs.hand):
            lines.append(f"  [{i}] {card_id}")

        # Potions
        active_potions = [(i, p) for i, p in enumerate(cs.potions) if p]
        if active_potions:
            lines.append(f"\nPotions: {', '.join(f'[{i}] {p}' for i, p in active_potions)}")

        # Draw/Discard/Exhaust counts
        lines.append(f"\nDraw: {len(cs.draw_pile)}  Discard: {len(cs.discard_pile)}  Exhaust: {len(cs.exhaust_pile)}")

        return "\n".join(lines)

    def _build_rewards_info(self) -> str:
        """Build combat rewards display."""
        runner = self.runner
        lines = ["=== COMBAT REWARDS ==="]

        rewards = runner.current_rewards
        if not rewards:
            lines.append("No rewards available. Proceed.")
            return "\n".join(lines)

        if rewards.gold and rewards.gold.claimed:
            lines.append(f"Gold: {rewards.gold.amount} (claimed)")
        elif rewards.gold:
            lines.append(f"Gold: {rewards.gold.amount}")

        if rewards.potion:
            status = "claimed" if rewards.potion.claimed else ("skipped" if rewards.potion.skipped else "available")
            lines.append(f"Potion: {rewards.potion.potion.name} ({status})")

        if rewards.relic:
            status = "claimed" if rewards.relic.claimed else "available"
            lines.append(f"Relic: {rewards.relic.relic.name} ({status})")

        if rewards.emerald_key:
            status = "claimed" if rewards.emerald_key.claimed else "available"
            lines.append(f"Emerald Key ({status})")

        for i, card_reward in enumerate(rewards.card_rewards):
            if card_reward.is_resolved:
                lines.append(f"Card reward {i}: resolved")
            else:
                card_names = [c.name for c in card_reward.cards]
                lines.append(f"Card reward {i}: {', '.join(card_names)}")

        return "\n".join(lines)

    def _build_event_info(self) -> str:
        """Build event display."""
        runner = self.runner
        lines = ["=== EVENT ==="]

        es = runner.current_event_state
        if not es:
            lines.append("No event active.")
            return "\n".join(lines)

        # Get event name
        event_def = runner.event_handler._get_event_definition(es.event_id)
        event_name = event_def.name if event_def else es.event_id
        lines.append(f"Event: {event_name}")

        if event_def and hasattr(event_def, 'flavor_text') and event_def.flavor_text:
            lines.append(f"\n{event_def.flavor_text}")

        lines.append("")

        # Show choices
        choices = runner.event_handler.get_available_choices(es, runner.run_state)
        for choice in choices:
            lines.append(f"  [{choice.index}] {choice.text}")

        return "\n".join(lines)

    def _build_shop_info(self) -> str:
        """Build shop display."""
        runner = self.runner
        lines = ["=== SHOP ==="]
        lines.append(f"Your gold: {runner.run_state.gold}")
        lines.append("")

        shop = runner.current_shop
        if not shop:
            lines.append("No shop data.")
            return "\n".join(lines)

        # Colored cards
        lines.append("-- Colored Cards --")
        for c in shop.colored_cards:
            if not c.purchased:
                sale = " [SALE]" if c.on_sale else ""
                affordable = " *" if c.price <= runner.run_state.gold else ""
                lines.append(f"  {c.card.name} ({c.card.rarity.name}): {c.price}g{sale}{affordable}")

        # Colorless cards
        lines.append("-- Colorless Cards --")
        for c in shop.colorless_cards:
            if not c.purchased:
                affordable = " *" if c.price <= runner.run_state.gold else ""
                lines.append(f"  {c.card.name} ({c.card.rarity.name}): {c.price}g{affordable}")

        # Relics
        lines.append("-- Relics --")
        for r in shop.relics:
            if not r.purchased:
                affordable = " *" if r.price <= runner.run_state.gold else ""
                lines.append(f"  {r.relic.name} ({r.relic.tier.name}): {r.price}g{affordable}")

        # Potions
        lines.append("-- Potions --")
        for p in shop.potions:
            if not p.purchased:
                affordable = " *" if p.price <= runner.run_state.gold else ""
                lines.append(f"  {p.potion.name}: {p.price}g{affordable}")

        # Card removal
        if shop.purge_available:
            affordable = " *" if shop.purge_cost <= runner.run_state.gold else ""
            lines.append(f"\nCard Removal: {shop.purge_cost}g{affordable}")
        else:
            lines.append("\nCard Removal: used")

        lines.append("\n(* = affordable)")

        return "\n".join(lines)

    def _build_boss_rewards_info(self) -> str:
        """Build boss relic choice display."""
        runner = self.runner
        lines = ["=== BOSS REWARDS ===", "Choose a boss relic:", ""]

        if runner.current_rewards and runner.current_rewards.boss_relics:
            boss_relics = runner.current_rewards.boss_relics
            for i, relic in enumerate(boss_relics.relics):
                desc = relic.description if hasattr(relic, 'description') else ""
                lines.append(f"  [{i}] {relic.name}: {desc}")
        else:
            lines.append("  (no boss relics generated)")

        return "\n".join(lines)

    def _build_neow_info(self) -> str:
        """Build Neow blessing display."""
        runner = self.runner
        lines = ["=== NEOW'S BLESSING ===", "Choose a blessing:", ""]

        if runner.neow_blessings:
            for i, blessing in enumerate(runner.neow_blessings):
                lines.append(f"  [{i}] {blessing.description}")
        else:
            lines.append("  (blessings not yet generated)")

        return "\n".join(lines)

    def _build_game_over_info(self) -> str:
        """Build game over display."""
        runner = self.runner
        state = runner.run_state
        if runner.game_won:
            lines = ["=== VICTORY ===", ""]
        else:
            lines = ["=== DEFEAT ===", ""]

        stats = runner.get_run_statistics()
        lines.append(f"Floor reached: {stats['final_floor']}")
        lines.append(f"Act: {stats['final_act']}")
        lines.append(f"HP: {stats['final_hp']}/{stats['final_max_hp']}")
        lines.append(f"Gold: {stats['final_gold']}")
        lines.append(f"Deck size: {stats['deck_size']}")
        lines.append(f"Relics: {stats['relic_count']}")
        lines.append(f"Combats won: {stats['combats_won']}")
        lines.append(f"Decisions made: {stats['decisions_made']}")

        return "\n".join(lines)

    def _update_actions(self):
        """Update the available actions panel."""
        # Clear existing action buttons
        for btn_id in self._action_buttons:
            if dpg.does_item_exist(btn_id):
                dpg.delete_item(btn_id)
        self._action_buttons.clear()

        # Remove placeholder if exists
        if dpg.does_item_exist("actions_placeholder"):
            dpg.delete_item("actions_placeholder")

        if not self.runner or self.runner.game_over:
            with dpg.group(parent="actions_group"):
                txt = dpg.add_text("Game over" if self.runner else "No game running")
                self._action_buttons.append(txt)
            return

        actions = self.runner.get_available_actions()
        if not actions:
            with dpg.group(parent="actions_group"):
                txt = dpg.add_text("No actions available")
                self._action_buttons.append(txt)
            return

        # Create buttons for each action (wrap after some number)
        for i, action in enumerate(actions):
            label = f"[{i}] {self._action_to_string(action)}"
            btn = dpg.add_button(
                label=label,
                parent="actions_group",
                callback=self._on_action_clicked,
                user_data=i,
                width=200
            )
            dpg.bind_item_theme(btn, self.action_btn_theme)
            self._action_buttons.append(btn)

    def _update_deck(self):
        """Update the deck display."""
        if not self.runner:
            dpg.set_value("deck_text", "--")
            return

        state = self.runner.run_state

        # Count cards by type
        card_counts: Dict[str, int] = {}
        for card in state.deck:
            key = str(card)
            card_counts[key] = card_counts.get(key, 0) + 1

        # Format as "Card x2, Card x1, ..."
        deck_parts = []
        for card_name, count in sorted(card_counts.items()):
            if count > 1:
                deck_parts.append(f"{card_name} x{count}")
            else:
                deck_parts.append(card_name)

        deck_str = f"({len(state.deck)} cards): " + ", ".join(deck_parts)
        dpg.set_value("deck_text", deck_str)

    def _update_log(self):
        """Update the log display."""
        log_str = "\n".join(self._log_messages[-50:])  # Keep last 50 messages
        dpg.set_value("log_text", log_str)

    # =========================================================================
    # Callbacks
    # =========================================================================

    def _on_start_game(self, sender=None, app_data=None):
        """Callback for Start Game button."""
        seed = dpg.get_value("seed_input")
        ascension = dpg.get_value("ascension_input")
        self.start_game(seed, ascension)

    def _on_action_clicked(self, sender, app_data, user_data):
        """Callback for action buttons."""
        action_idx = user_data
        self.take_action(action_idx)

    def _on_random_action(self, sender=None, app_data=None):
        """Callback for Random Action button."""
        self.take_random_action()

    def _on_auto_play(self, sender=None, app_data=None):
        """Callback for Auto-Play button (play 10 random actions)."""
        for _ in range(10):
            if self.runner and not self.runner.game_over:
                self.take_random_action()

    # =========================================================================
    # Utility Methods
    # =========================================================================

    def _action_to_string(self, action: GameAction) -> str:
        """Convert an action to a human-readable string."""
        if isinstance(action, PathAction):
            if self.runner:
                paths = self.runner.run_state.get_available_paths()
                if action.node_index < len(paths):
                    node = paths[action.node_index]
                    return f"Go to {node.room_type.name} ({node.x},{node.y})"
            return f"Path {action.node_index}"

        elif isinstance(action, NeowAction):
            if self.runner and self.runner.neow_blessings and action.choice_index < len(self.runner.neow_blessings):
                return f"Neow: {self.runner.neow_blessings[action.choice_index].description}"
            return f"Neow option {action.choice_index}"

        elif isinstance(action, CombatAction):
            if action.action_type == "end_turn":
                return "End Turn"
            elif action.action_type == "play_card":
                card_name = f"card {action.card_idx}"
                if self.runner and self.runner.current_combat:
                    hand = self.runner.current_combat.state.hand
                    if 0 <= action.card_idx < len(hand):
                        card_name = hand[action.card_idx]
                target_str = ""
                if action.target_idx >= 0 and self.runner and self.runner.current_combat:
                    enemies = self.runner.current_combat.state.enemies
                    if action.target_idx < len(enemies):
                        target_str = f" -> {enemies[action.target_idx].name}"
                return f"Play {card_name}{target_str}"
            elif action.action_type == "use_potion":
                potion_name = f"potion {action.potion_idx}"
                if self.runner and self.runner.current_combat:
                    potions = self.runner.current_combat.state.potions
                    if action.potion_idx < len(potions):
                        potion_name = potions[action.potion_idx] or potion_name
                return f"Use {potion_name}"
            return action.action_type

        elif isinstance(action, RewardAction):
            if action.reward_type == "proceed":
                return "Proceed"
            elif action.reward_type == "gold":
                if self.runner and self.runner.current_rewards and self.runner.current_rewards.gold:
                    return f"Take {self.runner.current_rewards.gold.amount} gold"
                return "Take gold"
            elif action.reward_type == "potion":
                if self.runner and self.runner.current_rewards and self.runner.current_rewards.potion:
                    return f"Take {self.runner.current_rewards.potion.potion.name}"
                return "Take potion"
            elif action.reward_type == "skip_potion":
                return "Skip potion"
            elif action.reward_type == "card":
                cr_idx = action.choice_index // 100
                card_idx = action.choice_index % 100
                if self.runner and self.runner.current_rewards:
                    rewards = self.runner.current_rewards
                    if cr_idx < len(rewards.card_rewards):
                        cr = rewards.card_rewards[cr_idx]
                        if card_idx < len(cr.cards):
                            return f"Take {cr.cards[card_idx].name}"
                return f"Take card {action.choice_index}"
            elif action.reward_type == "skip_card":
                return "Skip card reward"
            elif action.reward_type == "singing_bowl":
                return "Singing Bowl (+2 Max HP)"
            elif action.reward_type == "relic":
                if self.runner and self.runner.current_rewards and self.runner.current_rewards.relic:
                    return f"Take {self.runner.current_rewards.relic.relic.name}"
                return "Take relic"
            elif action.reward_type == "emerald_key":
                return "Take Emerald Key"
            elif action.reward_type == "skip_emerald_key":
                return "Skip Emerald Key"
            return f"Reward: {action.reward_type}"

        elif isinstance(action, EventAction):
            if self.runner and self.runner.current_event_state:
                choices = self.runner.event_handler.get_available_choices(
                    self.runner.current_event_state, self.runner.run_state
                )
                for c in choices:
                    if c.index == action.choice_index:
                        return f"[{c.index}] {c.text}"
            return f"Event option {action.choice_index}"

        elif isinstance(action, ShopAction):
            if action.action_type == "leave":
                return "Leave shop"
            shop = self.runner.current_shop if self.runner else None
            if shop:
                if action.action_type == "buy_colored_card":
                    for c in shop.colored_cards:
                        if c.slot_index == action.item_index and not c.purchased:
                            return f"Buy {c.card.name} ({c.price}g)"
                elif action.action_type == "buy_colorless_card":
                    for c in shop.colorless_cards:
                        if c.slot_index == action.item_index and not c.purchased:
                            return f"Buy {c.card.name} ({c.price}g)"
                elif action.action_type == "buy_relic":
                    for r in shop.relics:
                        if r.slot_index == action.item_index and not r.purchased:
                            return f"Buy {r.relic.name} ({r.price}g)"
                elif action.action_type == "buy_potion":
                    for p in shop.potions:
                        if p.slot_index == action.item_index and not p.purchased:
                            return f"Buy {p.potion.name} ({p.price}g)"
                elif action.action_type == "remove_card":
                    if action.item_index < len(self.runner.run_state.deck):
                        card = self.runner.run_state.deck[action.item_index]
                        return f"Remove {card} ({shop.purge_cost}g)"
            return f"Shop: {action.action_type} {action.item_index}"

        elif isinstance(action, RestAction):
            if action.action_type == "rest":
                return "Rest (heal)"
            elif action.action_type == "upgrade":
                if self.runner and action.card_index >= 0:
                    card = self.runner.run_state.deck[action.card_index]
                    return f"Upgrade {card}"
                return f"Upgrade card {action.card_index}"
            return action.action_type.title()

        elif isinstance(action, TreasureAction):
            if action.action_type == "take_relic":
                return "Take relic"
            elif action.action_type == "sapphire_key":
                return "Take Sapphire Key"
            return action.action_type

        elif isinstance(action, BossRewardAction):
            if self.runner and self.runner.current_rewards and self.runner.current_rewards.boss_relics:
                relics = self.runner.current_rewards.boss_relics.relics
                if action.relic_index < len(relics):
                    return f"Boss relic: {relics[action.relic_index].name}"
            return f"Boss relic {action.relic_index}"

        return str(action)

    def _log(self, message: str):
        """Add a message to the log."""
        self._log_messages.append(message)
        self._update_log()

    # =========================================================================
    # Main Loop
    # =========================================================================

    def run(self):
        """Run the DearPyGui main loop."""
        dpg.setup_dearpygui()
        dpg.show_viewport()
        dpg.start_dearpygui()
        dpg.destroy_context()


# =============================================================================
# Main Entry Point
# =============================================================================

def main():
    """Launch the Game UI."""
    ui = GameUI()
    ui.run()


if __name__ == "__main__":
    main()
