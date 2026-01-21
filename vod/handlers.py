"""
Tool handlers that mutate VODRunState based on tool calls.

Each handler receives a VODRunState and parameters dict, applies
the appropriate state mutations, and logs the decision.
"""

from typing import Any, Callable, Optional
from dataclasses import dataclass

from core.state import create_watcher_run
from vod.state import VODRunState, DecisionType, DecisionLog


@dataclass
class ToolResult:
    """Result of a tool handler execution."""
    success: bool
    message: str
    decision: Optional[DecisionLog] = None
    error: Optional[str] = None


class ToolHandler:
    """
    Handles tool calls from the LLM and applies state mutations.

    Usage:
        handler = ToolHandler(state)
        result = handler.handle("card_reward", {...params...})
    """

    def __init__(self, state: VODRunState):
        self.state = state
        self._handlers: dict[str, Callable[[dict], ToolResult]] = {
            "set_seed": self._handle_set_seed,
            "neow": self._handle_neow,
            "path": self._handle_path,
            "combat_start": self._handle_combat_start,
            "combat_turn": self._handle_combat_turn,
            "combat_end": self._handle_combat_end,
            "card_reward": self._handle_card_reward,
            "relic_reward": self._handle_relic_reward,
            "potion_reward": self._handle_potion_reward,
            "shop": self._handle_shop,
            "rest": self._handle_rest,
            "boss_relic": self._handle_boss_relic,
            "event": self._handle_event,
            "result": self._handle_result,
        }

    def handle(self, tool_name: str, params: dict[str, Any]) -> ToolResult:
        """Handle a tool call and apply state mutations."""
        if tool_name not in self._handlers:
            return ToolResult(
                success=False,
                message=f"Unknown tool: {tool_name}",
                error=f"No handler for tool '{tool_name}'"
            )

        try:
            # Update timestamp if provided
            if "timestamp" in params:
                self.state.current_timestamp = params["timestamp"]

            return self._handlers[tool_name](params)
        except Exception as e:
            return ToolResult(
                success=False,
                message=f"Error handling {tool_name}",
                error=str(e)
            )

    def handle_batch(self, calls: list[dict[str, Any]]) -> list[ToolResult]:
        """Handle multiple tool calls in sequence."""
        results = []
        for call in calls:
            tool_name = call.get("name") or call.get("tool")
            params = call.get("arguments") or call.get("params") or {}
            results.append(self.handle(tool_name, params))
        return results

    # --- Individual handlers ---

    def _handle_set_seed(self, params: dict) -> ToolResult:
        """Set the game seed."""
        seed = params["seed"]

        # Recreate RunState with correct seed if this is the first seed detection
        if not self.state.seed_detected:
            self.state.run = create_watcher_run(
                seed=seed,
                ascension=self.state.run.ascension
            )
            self.state.seed_detected = True

        decision = self.state.log_decision(
            DecisionType.SEED,
            {"seed": seed},
            floor=0,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Seed set to {seed}",
            decision=decision,
        )

    def _handle_neow(self, params: dict) -> ToolResult:
        """Handle Neow bonus selection."""
        chosen = params["chosen"]
        drawback = params.get("drawback")

        # Log the decision
        decision = self.state.log_decision(
            DecisionType.NEOW,
            {
                "chosen": chosen,
                "drawback": drawback,
            },
            floor=0,
            timestamp=params.get("timestamp"),
        )

        # Apply known Neow effects to state
        self._apply_neow_effect(chosen, drawback)

        return ToolResult(
            success=True,
            message=f"Neow bonus: {chosen}" + (f" (drawback: {drawback})" if drawback else ""),
            decision=decision,
        )

    def _apply_neow_effect(self, chosen: str, drawback: Optional[str]) -> None:
        """Apply Neow bonus/drawback effects to state."""
        run = self.state.run
        chosen_lower = chosen.lower()

        # Common bonuses
        if "max hp" in chosen_lower:
            # Extract number if present
            import re
            match = re.search(r'(\d+)', chosen)
            if match:
                run.gain_max_hp(int(match.group(1)))
                run.heal_to_full()
        elif "100 gold" in chosen_lower or "hundred gold" in chosen_lower:
            run.add_gold(100)
        elif "250 gold" in chosen_lower:
            run.add_gold(250)
        elif "remove" in chosen_lower and "card" in chosen_lower:
            # Card removal handled by subsequent tool calls or manually
            pass

        # Common drawbacks
        if drawback:
            drawback_lower = drawback.lower()
            if "lose all gold" in drawback_lower:
                run.set_gold(0)
            elif "lose" in drawback_lower and "max hp" in drawback_lower:
                match = re.search(r'(\d+)', drawback)
                if match:
                    run.lose_max_hp(int(match.group(1)))
            elif "curse" in drawback_lower:
                # Curse added - handled by extractor noting the curse
                pass

    def _handle_path(self, params: dict) -> ToolResult:
        """Record a map path choice."""
        floor = params["floor"]
        chosen = params["chosen"]
        options = params.get("options", [])

        # Update floor
        self.state.run.floor = floor

        # Record path
        self.state.record_path(floor, chosen, options)

        decision = self.state.log_decision(
            DecisionType.PATH,
            {
                "chosen": chosen,
                "options": options,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Floor {floor}: chose {chosen}",
            decision=decision,
        )

    def _handle_combat_start(self, params: dict) -> ToolResult:
        """Start tracking a combat."""
        floor = params["floor"]
        enemy = params["enemy"]

        self.state.run.floor = floor
        self.state.start_combat(
            floor=floor,
            enemy=enemy,
            timestamp=params.get("timestamp"),
        )

        decision = self.state.log_decision(
            DecisionType.COMBAT_START,
            {"enemy": enemy},
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Combat started: {enemy} (floor {floor})",
            decision=decision,
        )

    def _handle_combat_turn(self, params: dict) -> ToolResult:
        """Record a combat turn."""
        turn = params["turn"]
        cards = params["cards"]
        potions = params.get("potions_used", [])

        if self.state.active_combat:
            self.state.active_combat.add_turn(cards)

        # Use potions from state
        for potion in potions:
            self.state.run.use_potion_by_name(potion) if hasattr(self.state.run, 'use_potion_by_name') else None

        decision = self.state.log_decision(
            DecisionType.COMBAT_TURN,
            {
                "turn": turn,
                "cards": cards,
                "potions": potions,
            },
            floor=self.state.run.floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Turn {turn}: {', '.join(cards) if cards else 'no cards'}",
            decision=decision,
        )

    def _handle_combat_end(self, params: dict) -> ToolResult:
        """End combat and update HP."""
        hp = params["hp"]
        gold_gained = params.get("gold_gained", 0)

        combat = self.state.end_combat(hp, params.get("timestamp"))

        # Update gold
        if gold_gained > 0:
            self.state.run.add_gold(gold_gained)

        # Track combat win
        self.state.run.combats_won += 1

        decision = self.state.log_decision(
            DecisionType.COMBAT_END,
            {
                "hp": hp,
                "gold_gained": gold_gained,
                "combat": combat.to_dict() if combat else None,
            },
            floor=self.state.run.floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Combat ended: {hp} HP" + (f", +{gold_gained} gold" if gold_gained else ""),
            decision=decision,
        )

    def _handle_card_reward(self, params: dict) -> ToolResult:
        """Handle card reward selection."""
        floor = params["floor"]
        options = params["options"]
        chosen = params["chosen"]
        singing_bowl = params.get("singing_bowl", False)

        self.state.run.floor = floor

        # Add card to deck if not skipped
        if chosen.upper() != "SKIP" and not singing_bowl:
            # Check if upgraded (indicated by +)
            upgraded = chosen.endswith("+")
            card_id = chosen.rstrip("+")
            self.state.run.add_card(card_id, upgraded=upgraded)
        elif singing_bowl:
            # Singing Bowl gives +2 max HP
            self.state.run.gain_max_hp(2)
            self.state.run.heal(2)

        decision = self.state.log_decision(
            DecisionType.CARD_REWARD,
            {
                "options": options,
                "chosen": chosen,
                "singing_bowl": singing_bowl,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Card reward: {chosen}" + (" (Singing Bowl)" if singing_bowl else ""),
            decision=decision,
        )

    def _handle_relic_reward(self, params: dict) -> ToolResult:
        """Handle relic obtained."""
        floor = params["floor"]
        relic = params["relic"]
        source = params.get("source", "unknown")

        self.state.run.floor = floor
        self.state.run.add_relic(relic)

        # Track elite/boss kills
        if source == "elite":
            self.state.run.elites_killed += 1
            self.state.run.elites_killed_this_act += 1
        elif source == "boss":
            self.state.run.bosses_killed += 1

        decision = self.state.log_decision(
            DecisionType.RELIC_REWARD,
            {
                "relic": relic,
                "source": source,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Relic obtained: {relic} ({source})",
            decision=decision,
        )

    def _handle_potion_reward(self, params: dict) -> ToolResult:
        """Handle potion obtained."""
        floor = params["floor"]
        potion = params["potion"]

        self.state.run.floor = floor
        self.state.run.add_potion(potion)

        decision = self.state.log_decision(
            DecisionType.POTION_REWARD,
            {"potion": potion},
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Potion obtained: {potion}",
            decision=decision,
        )

    def _handle_shop(self, params: dict) -> ToolResult:
        """Handle shop visit."""
        floor = params["floor"]
        cards_bought = params.get("cards_bought", [])
        cards_removed = params.get("cards_removed", [])
        relics_bought = params.get("relics_bought", [])
        potions_bought = params.get("potions_bought", [])
        gold_spent = params.get("gold_spent", 0)

        self.state.run.floor = floor

        # Add purchased cards
        for card in cards_bought:
            upgraded = card.endswith("+")
            card_id = card.rstrip("+")
            self.state.run.add_card(card_id, upgraded=upgraded)

        # Remove cards
        for card in cards_removed:
            self.state.run.remove_card_by_id(card)

        # Add relics
        for relic in relics_bought:
            self.state.run.add_relic(relic)

        # Add potions
        for potion in potions_bought:
            self.state.run.add_potion(potion)

        # Spend gold
        if gold_spent > 0:
            self.state.run.lose_gold(gold_spent)

        decision = self.state.log_decision(
            DecisionType.SHOP,
            {
                "cards_bought": cards_bought,
                "cards_removed": cards_removed,
                "relics_bought": relics_bought,
                "potions_bought": potions_bought,
                "gold_spent": gold_spent,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        actions = []
        if cards_bought:
            actions.append(f"bought {', '.join(cards_bought)}")
        if cards_removed:
            actions.append(f"removed {', '.join(cards_removed)}")
        if relics_bought:
            actions.append(f"bought relics {', '.join(relics_bought)}")

        return ToolResult(
            success=True,
            message=f"Shop: {'; '.join(actions) if actions else 'browsed'}",
            decision=decision,
        )

    def _handle_rest(self, params: dict) -> ToolResult:
        """Handle rest site decision."""
        floor = params["floor"]
        action = params["action"]
        card_upgraded = params.get("card_upgraded")

        self.state.run.floor = floor

        if action == "rest":
            # Heal 30% of max HP
            heal_amount = int(self.state.run.max_hp * 0.30)
            # Check for Regal Pillow (+15 more)
            if self.state.run.has_relic("Regal Pillow"):
                heal_amount += 15
            self.state.run.heal(heal_amount)
        elif action == "smith" and card_upgraded:
            # Upgrade a card - find and upgrade it
            deck = self.state.run.deck
            for i, card in enumerate(deck):
                if card.id == card_upgraded and not card.upgraded:
                    self.state.run.upgrade_card(i)
                    break
        elif action == "ruby_key":
            self.state.run.obtain_ruby_key()
        elif action == "dig":
            # Shovel - relic obtained separately
            pass
        elif action == "lift":
            # Girya - counter increment
            self.state.run.increment_relic_counter("Girya", 1)
        elif action == "toke":
            # Peace Pipe - card removed separately
            pass

        decision = self.state.log_decision(
            DecisionType.REST,
            {
                "action": action,
                "card_upgraded": card_upgraded,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        msg = f"Rest site: {action}"
        if card_upgraded:
            msg += f" ({card_upgraded})"

        return ToolResult(
            success=True,
            message=msg,
            decision=decision,
        )

    def _handle_boss_relic(self, params: dict) -> ToolResult:
        """Handle boss relic selection."""
        floor = params["floor"]
        options = params["options"]
        chosen = params["chosen"]

        self.state.run.floor = floor
        self.state.run.add_relic(chosen)

        # Advance act after boss
        if floor in [16, 33, 50]:
            self.state.run.advance_act()

        decision = self.state.log_decision(
            DecisionType.BOSS_RELIC,
            {
                "options": options,
                "chosen": chosen,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Boss relic: {chosen} (from {', '.join(options)})",
            decision=decision,
        )

    def _handle_event(self, params: dict) -> ToolResult:
        """Handle event encounter."""
        floor = params["floor"]
        name = params["name"]
        choice = params["choice"]
        outcome = params.get("outcome")

        self.state.run.floor = floor

        decision = self.state.log_decision(
            DecisionType.EVENT,
            {
                "name": name,
                "choice": choice,
                "outcome": outcome,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        return ToolResult(
            success=True,
            message=f"Event '{name}': {choice}",
            decision=decision,
        )

    def _handle_result(self, params: dict) -> ToolResult:
        """Handle run result."""
        victory = params["victory"]
        floor = params["floor"]
        heart_kill = params.get("heart_kill", False)
        final_hp = params.get("final_hp", 0)

        self.state.run.floor = floor
        self.state.run.current_hp = final_hp

        decision = self.state.log_decision(
            DecisionType.RESULT,
            {
                "victory": victory,
                "floor": floor,
                "heart_kill": heart_kill,
                "final_hp": final_hp,
            },
            floor=floor,
            timestamp=params.get("timestamp"),
        )

        result_str = "Victory!" if victory else "Defeat"
        if heart_kill:
            result_str = "Heart killed!"

        return ToolResult(
            success=True,
            message=f"{result_str} at floor {floor} with {final_hp} HP",
            decision=decision,
        )


def create_handler(state: VODRunState) -> ToolHandler:
    """Factory function to create a tool handler."""
    return ToolHandler(state)
