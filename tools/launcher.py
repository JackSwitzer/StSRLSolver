#!/usr/bin/env python3
"""
STS Simulation Launcher
A clean GUI for launching Slay the Spire simulations and RL training.
"""

import dearpygui.dearpygui as dpg
from dataclasses import dataclass
from enum import Enum
from typing import Callable
import threading
import time


class Character(Enum):
    IRONCLAD = "IRONCLAD"
    SILENT = "SILENT"
    DEFECT = "DEFECT"
    WATCHER = "WATCHER"


class Mode(Enum):
    VISUAL = "visual"
    HEADLESS = "headless"


@dataclass
class SimulationConfig:
    character: Character = Character.WATCHER
    ascension: int = 20
    total_seeds: int = 1
    runs_per_seed: int = 1
    mode: Mode = Mode.VISUAL
    seed: str = ""  # Specific seed to test (empty = random)


@dataclass
class SimulationStats:
    current_seed: int = 0
    completed_seeds: int = 0
    total_runs: int = 0
    completed_runs: int = 0
    wins: int = 0
    losses: int = 0
    running: bool = False

    @property
    def win_rate(self) -> float:
        total = self.wins + self.losses
        return (self.wins / total * 100) if total > 0 else 0.0


class SimulationLauncher:
    """Main launcher application."""

    # STS-inspired color palette
    COLORS = {
        "bg_dark": (25, 25, 35),
        "bg_panel": (35, 35, 50),
        "accent": (200, 150, 80),  # Gold
        "accent_hover": (220, 170, 100),
        "text": (220, 220, 220),
        "text_dim": (140, 140, 150),
        "success": (80, 180, 100),
        "danger": (200, 80, 80),
        "ironclad": (180, 60, 60),
        "silent": (60, 150, 80),
        "defect": (80, 120, 200),
        "watcher": (150, 80, 180),
    }

    def __init__(self):
        self.config = SimulationConfig()
        self.stats = SimulationStats()
        self._simulation_thread: threading.Thread | None = None
        self._stop_requested = False

    def setup_theme(self):
        """Create the dark STS-inspired theme."""
        with dpg.theme() as self.global_theme:
            with dpg.theme_component(dpg.mvAll):
                # Window/frame colors
                dpg.add_theme_color(
                    dpg.mvThemeCol_WindowBg, self.COLORS["bg_dark"], category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ChildBg, self.COLORS["bg_panel"], category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_FrameBg, (45, 45, 60), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_FrameBgHovered, (55, 55, 75), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_FrameBgActive, (65, 65, 85), category=dpg.mvThemeCat_Core
                )

                # Text
                dpg.add_theme_color(
                    dpg.mvThemeCol_Text, self.COLORS["text"], category=dpg.mvThemeCat_Core
                )

                # Buttons
                dpg.add_theme_color(
                    dpg.mvThemeCol_Button, (50, 50, 70), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonHovered, (65, 65, 90), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonActive, (80, 80, 110), category=dpg.mvThemeCat_Core
                )

                # Slider/progress
                dpg.add_theme_color(
                    dpg.mvThemeCol_SliderGrab, self.COLORS["accent"], category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_SliderGrabActive,
                    self.COLORS["accent_hover"],
                    category=dpg.mvThemeCat_Core,
                )

                # Headers
                dpg.add_theme_color(
                    dpg.mvThemeCol_Header, (50, 50, 70), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_HeaderHovered, (60, 60, 85), category=dpg.mvThemeCat_Core
                )

                # Border
                dpg.add_theme_color(
                    dpg.mvThemeCol_Border, (60, 60, 80), category=dpg.mvThemeCat_Core
                )

                # Checkbox
                dpg.add_theme_color(
                    dpg.mvThemeCol_CheckMark, self.COLORS["accent"], category=dpg.mvThemeCat_Core
                )

                # Rounding
                dpg.add_theme_style(dpg.mvStyleVar_FrameRounding, 4, category=dpg.mvThemeCat_Core)
                dpg.add_theme_style(dpg.mvStyleVar_WindowRounding, 6, category=dpg.mvThemeCat_Core)
                dpg.add_theme_style(dpg.mvStyleVar_ChildRounding, 4, category=dpg.mvThemeCat_Core)
                dpg.add_theme_style(
                    dpg.mvStyleVar_FramePadding, 8, 4, category=dpg.mvThemeCat_Core
                )

        # Start button theme
        with dpg.theme() as self.start_theme:
            with dpg.theme_component(dpg.mvButton):
                dpg.add_theme_color(
                    dpg.mvThemeCol_Button, (40, 100, 50), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonHovered, (50, 130, 60), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonActive, (60, 150, 70), category=dpg.mvThemeCat_Core
                )

        # Stop button theme
        with dpg.theme() as self.stop_theme:
            with dpg.theme_component(dpg.mvButton):
                dpg.add_theme_color(
                    dpg.mvThemeCol_Button, (120, 50, 50), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonHovered, (150, 60, 60), category=dpg.mvThemeCat_Core
                )
                dpg.add_theme_color(
                    dpg.mvThemeCol_ButtonActive, (180, 70, 70), category=dpg.mvThemeCat_Core
                )

        # Progress bar theme
        with dpg.theme() as self.progress_theme:
            with dpg.theme_component(dpg.mvProgressBar):
                dpg.add_theme_color(
                    dpg.mvThemeCol_PlotHistogram, self.COLORS["accent"], category=dpg.mvThemeCat_Core
                )

    def create_ui(self):
        """Build the main UI."""
        with dpg.window(tag="main_window", label="STS Simulation Launcher"):
            # Title
            dpg.add_text("STS Simulation Launcher", color=self.COLORS["accent"])
            dpg.add_separator()
            dpg.add_spacer(height=10)

            # Configuration section
            with dpg.child_window(height=200, border=True):
                dpg.add_text("Configuration", color=self.COLORS["text_dim"])
                dpg.add_spacer(height=5)

                # Row 1: Character and Ascension
                with dpg.group(horizontal=True):
                    dpg.add_text("Character:", color=self.COLORS["text"])
                    dpg.add_combo(
                        items=[c.value for c in Character],
                        default_value=self.config.character.value,
                        width=150,
                        callback=self._on_character_change,
                        tag="character_combo",
                    )
                    dpg.add_spacer(width=30)
                    dpg.add_text("Ascension:", color=self.COLORS["text"])
                    dpg.add_slider_int(
                        default_value=self.config.ascension,
                        min_value=0,
                        max_value=20,
                        width=150,
                        callback=self._on_ascension_change,
                        tag="ascension_slider",
                    )

                dpg.add_spacer(height=15)

                # Row 2: Seeds and Runs
                with dpg.group(horizontal=True):
                    dpg.add_text("Total Seeds:", color=self.COLORS["text"])
                    dpg.add_input_int(
                        default_value=self.config.total_seeds,
                        min_value=1,
                        max_value=10000,
                        min_clamped=True,
                        max_clamped=True,
                        width=100,
                        callback=self._on_seeds_change,
                        tag="seeds_input",
                    )
                    dpg.add_spacer(width=30)
                    dpg.add_text("Runs/Seed:", color=self.COLORS["text"])
                    dpg.add_input_int(
                        default_value=self.config.runs_per_seed,
                        min_value=1,
                        max_value=100,
                        min_clamped=True,
                        max_clamped=True,
                        width=100,
                        callback=self._on_runs_change,
                        tag="runs_input",
                    )

                dpg.add_spacer(height=15)

                # Row 3: Mode
                with dpg.group(horizontal=True):
                    dpg.add_text("Mode:", color=self.COLORS["text"])
                    dpg.add_radio_button(
                        items=["Visual", "Headless"],
                        default_value="Visual" if self.config.mode == Mode.VISUAL else "Headless",
                        horizontal=True,
                        callback=self._on_mode_change,
                        tag="mode_radio",
                    )

                dpg.add_spacer(height=10)

                # Estimated runs display
                with dpg.group(horizontal=True):
                    dpg.add_text("Total runs:", color=self.COLORS["text_dim"])
                    dpg.add_text(
                        str(self.config.total_seeds * self.config.runs_per_seed),
                        tag="total_runs_text",
                        color=self.COLORS["accent"],
                    )

            dpg.add_spacer(height=15)

            # Control buttons
            with dpg.group(horizontal=True):
                start_btn = dpg.add_button(
                    label="  START  ",
                    width=150,
                    height=40,
                    callback=self._on_start,
                    tag="start_btn",
                )
                dpg.bind_item_theme(start_btn, self.start_theme)

                dpg.add_spacer(width=20)

                stop_btn = dpg.add_button(
                    label="  STOP  ",
                    width=150,
                    height=40,
                    callback=self._on_stop,
                    tag="stop_btn",
                    enabled=False,
                )
                dpg.bind_item_theme(stop_btn, self.stop_theme)

            dpg.add_spacer(height=15)

            # Progress section
            with dpg.child_window(height=150, border=True):
                dpg.add_text("Progress", color=self.COLORS["text_dim"])
                dpg.add_spacer(height=5)

                # Progress bars
                with dpg.group(horizontal=True):
                    dpg.add_text("Seeds:", color=self.COLORS["text"], indent=10)
                    progress = dpg.add_progress_bar(
                        default_value=0.0, width=250, tag="seed_progress"
                    )
                    dpg.bind_item_theme(progress, self.progress_theme)
                    dpg.add_text("0/0", tag="seed_progress_text", color=self.COLORS["text_dim"])

                dpg.add_spacer(height=5)

                with dpg.group(horizontal=True):
                    dpg.add_text("Runs: ", color=self.COLORS["text"], indent=10)
                    progress = dpg.add_progress_bar(
                        default_value=0.0, width=250, tag="run_progress"
                    )
                    dpg.bind_item_theme(progress, self.progress_theme)
                    dpg.add_text("0/0", tag="run_progress_text", color=self.COLORS["text_dim"])

                dpg.add_spacer(height=15)

                # Stats
                with dpg.group(horizontal=True):
                    dpg.add_text("Wins:", color=self.COLORS["success"], indent=10)
                    dpg.add_text("0", tag="wins_text", color=self.COLORS["success"])
                    dpg.add_spacer(width=20)
                    dpg.add_text("Losses:", color=self.COLORS["danger"])
                    dpg.add_text("0", tag="losses_text", color=self.COLORS["danger"])
                    dpg.add_spacer(width=20)
                    dpg.add_text("Win Rate:", color=self.COLORS["text"])
                    dpg.add_text("--", tag="winrate_text", color=self.COLORS["accent"])

                dpg.add_spacer(height=10)

                with dpg.group(horizontal=True):
                    dpg.add_text("Status:", color=self.COLORS["text_dim"], indent=10)
                    dpg.add_text("Ready", tag="status_text", color=self.COLORS["text"])

    # Callbacks
    def _on_character_change(self, sender, value):
        self.config.character = Character(value)

    def _on_ascension_change(self, sender, value):
        self.config.ascension = value

    def _on_seeds_change(self, sender, value):
        self.config.total_seeds = value
        self._update_total_runs()

    def _on_runs_change(self, sender, value):
        self.config.runs_per_seed = value
        self._update_total_runs()

    def _on_mode_change(self, sender, value):
        self.config.mode = Mode.VISUAL if value == "Visual" else Mode.HEADLESS

    def _update_total_runs(self):
        total = self.config.total_seeds * self.config.runs_per_seed
        dpg.set_value("total_runs_text", str(total))

    def _on_start(self):
        if self.stats.running:
            return

        self._stop_requested = False
        self.stats = SimulationStats(
            total_runs=self.config.total_seeds * self.config.runs_per_seed, running=True
        )

        # Update UI state
        dpg.configure_item("start_btn", enabled=False)
        dpg.configure_item("stop_btn", enabled=True)
        dpg.set_value("status_text", "Running...")

        # Start simulation thread
        self._simulation_thread = threading.Thread(target=self._run_simulation, daemon=True)
        self._simulation_thread.start()

    def _on_stop(self):
        self._stop_requested = True
        dpg.set_value("status_text", "Stopping...")

    def _run_simulation(self):
        """Placeholder simulation loop - replace with actual simulation integration."""
        import random

        for seed_idx in range(self.config.total_seeds):
            if self._stop_requested:
                break

            self.stats.current_seed = seed_idx + 1

            for run_idx in range(self.config.runs_per_seed):
                if self._stop_requested:
                    break

                # Simulate a run (placeholder - replace with actual simulation)
                time.sleep(0.05 if self.config.mode == Mode.HEADLESS else 0.1)

                # Random win/loss for demo
                if random.random() < 0.85:
                    self.stats.wins += 1
                else:
                    self.stats.losses += 1

                self.stats.completed_runs += 1
                self._update_progress_ui()

            self.stats.completed_seeds += 1

        # Simulation complete
        self.stats.running = False
        self._finalize_simulation()

    def _update_progress_ui(self):
        """Update progress display (called from simulation thread)."""
        # Seed progress
        seed_progress = self.stats.completed_seeds / self.config.total_seeds
        dpg.set_value("seed_progress", seed_progress)
        dpg.set_value(
            "seed_progress_text", f"{self.stats.completed_seeds}/{self.config.total_seeds}"
        )

        # Run progress
        run_progress = self.stats.completed_runs / self.stats.total_runs if self.stats.total_runs else 0
        dpg.set_value("run_progress", run_progress)
        dpg.set_value("run_progress_text", f"{self.stats.completed_runs}/{self.stats.total_runs}")

        # Stats
        dpg.set_value("wins_text", str(self.stats.wins))
        dpg.set_value("losses_text", str(self.stats.losses))
        dpg.set_value("winrate_text", f"{self.stats.win_rate:.1f}%")

        # Status
        dpg.set_value("status_text", f"Seed {self.stats.current_seed}")

    def _finalize_simulation(self):
        """Clean up after simulation ends."""
        dpg.configure_item("start_btn", enabled=True)
        dpg.configure_item("stop_btn", enabled=False)

        if self._stop_requested:
            dpg.set_value("status_text", "Stopped")
        else:
            dpg.set_value("status_text", "Complete")

    def run(self):
        """Main entry point."""
        dpg.create_context()

        self.setup_theme()
        self.create_ui()

        dpg.bind_theme(self.global_theme)

        dpg.create_viewport(title="STS Simulation Launcher", width=500, height=450)
        dpg.setup_dearpygui()
        dpg.set_primary_window("main_window", True)
        dpg.show_viewport()

        dpg.start_dearpygui()
        dpg.destroy_context()


def main():
    launcher = SimulationLauncher()
    launcher.run()


if __name__ == "__main__":
    main()
