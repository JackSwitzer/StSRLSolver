#!/usr/bin/env python3
"""
STS Game State Viewer - Shows predictions and game state with actual assets.
"""
import dearpygui.dearpygui as dpg
import threading
import time
import base64
import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict, Optional
import sys

sys.path.insert(0, str(Path(__file__).parent.parent))

from core.generation.encounters import predict_all_acts, predict_all_bosses_extended

SAVE_DIR = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/saves"
ASSETS_DIR = Path(__file__).parent.parent / "assets"
XOR_KEY = b"key"

# Relic ID to filename mapping (common ones)
RELIC_FILENAMES = {
    "PureWater": "pureWater.png",
    "NeowsBlessing": "neow.png",
    "Akabeko": "akabeko.png",
    "Anchor": "anchor.png",
    "Lantern": "lantern.png",
    "BurningBlood": "burningBlood.png",
    "CrackedCore": "core.png",
    "RingOfTheSnake": "snake.png",
    "Vajra": "vajra.png",
    "Boot": "boot.png",
    "Nunchaku": "nunchaku.png",
    "OddlySmoothStone": "oddlySmooth.png",
    "BronzeScales": "bronzeScales.png",
    "PenNib": "penNib.png",
    # Add more as needed
}


@dataclass
class GameState:
    seed: str = ""
    floor: int = 0
    act: int = 1
    hp: int = 80
    max_hp: int = 80
    gold: int = 99
    character: str = "WATCHER"
    ascension: int = 20
    boss: str = ""
    monsters: List[str] = field(default_factory=list)
    elites: List[str] = field(default_factory=list)
    relics: List[str] = field(default_factory=list)
    deck: List[str] = field(default_factory=list)
    potions: List[str] = field(default_factory=list)
    path: List[str] = field(default_factory=list)


def decrypt_save(data: bytes) -> dict:
    decrypted = bytes([data[i] ^ XOR_KEY[i % len(XOR_KEY)] for i in range(len(data))])
    return json.loads(decrypted.decode('utf-8'))


def load_save(character: str = "WATCHER") -> Optional[dict]:
    save_path = SAVE_DIR / f"{character}.autosave"
    if not save_path.exists():
        return None
    with open(save_path, "rb") as f:
        return decrypt_save(base64.b64decode(f.read()))


class GameViewer:
    COLORS = {
        "bg": (15, 15, 25),
        "panel": (25, 25, 40),
        "gold": (212, 175, 55),
        "red": (200, 60, 60),
        "green": (60, 180, 80),
        "blue": (80, 140, 220),
        "purple": (160, 80, 200),
        "text": (230, 230, 230),
        "dim": (130, 130, 150),
        "hp_red": (180, 50, 50),
        "hp_bg": (60, 30, 30),
    }

    def __init__(self):
        self.state = GameState()
        self.predictions: Dict = {}
        self._running = True
        self._watch_thread: Optional[threading.Thread] = None
        self._loaded_textures: Dict[str, int] = {}

    def _load_texture(self, path: Path, tag: str) -> bool:
        """Load a texture from file."""
        if not path.exists():
            return False
        try:
            width, height, channels, data = dpg.load_image(str(path))
            with dpg.texture_registry():
                dpg.add_static_texture(width, height, data, tag=tag)
            self._loaded_textures[tag] = (width, height)
            return True
        except Exception as e:
            return False

    def _load_assets(self):
        """Pre-load common assets."""
        # Load UI assets
        ui_dir = ASSETS_DIR / "ui"
        for name in ["gold.png", "panelHeart.png"]:
            path = ui_dir / name
            if path.exists():
                self._load_texture(path, f"ui_{name.replace('.png', '')}")

        # Load some relic assets
        relic_dir = ASSETS_DIR / "relics"
        if relic_dir.exists():
            for relic_file in list(relic_dir.glob("*.png"))[:50]:  # Load first 50
                tag = f"relic_{relic_file.stem}"
                self._load_texture(relic_file, tag)

        # Load potion assets
        potion_dir = ASSETS_DIR / "potions"
        if potion_dir.exists():
            for potion_file in potion_dir.glob("*.png"):
                tag = f"potion_{potion_file.stem}"
                self._load_texture(potion_file, tag)

    def run(self):
        dpg.create_context()
        dpg.create_viewport(title="STS Game Viewer", width=1400, height=900)

        # Load assets
        self._load_assets()

        # Theme
        with dpg.theme() as global_theme:
            with dpg.theme_component(dpg.mvAll):
                dpg.add_theme_color(dpg.mvThemeCol_WindowBg, self.COLORS["bg"])
                dpg.add_theme_color(dpg.mvThemeCol_ChildBg, self.COLORS["panel"])
                dpg.add_theme_color(dpg.mvThemeCol_Text, self.COLORS["text"])
                dpg.add_theme_color(dpg.mvThemeCol_Border, (50, 50, 70))
                dpg.add_theme_color(dpg.mvThemeCol_FrameBg, (40, 40, 60))
                dpg.add_theme_color(dpg.mvThemeCol_Button, (50, 50, 80))
                dpg.add_theme_color(dpg.mvThemeCol_ButtonHovered, (70, 70, 100))
                dpg.add_theme_color(dpg.mvThemeCol_SliderGrab, self.COLORS["gold"])
                dpg.add_theme_style(dpg.mvStyleVar_WindowRounding, 0)
                dpg.add_theme_style(dpg.mvStyleVar_ChildRounding, 8)
                dpg.add_theme_style(dpg.mvStyleVar_FrameRounding, 4)
                dpg.add_theme_style(dpg.mvStyleVar_WindowPadding, 10, 10)

        dpg.bind_theme(global_theme)
        self._create_ui()

        dpg.setup_dearpygui()
        dpg.show_viewport()
        dpg.set_primary_window("main", True)

        # Start watch thread
        self._watch_thread = threading.Thread(target=self._watch_save, daemon=True)
        self._watch_thread.start()

        while dpg.is_dearpygui_running() and self._running:
            dpg.render_dearpygui_frame()

        dpg.destroy_context()

    def _create_ui(self):
        with dpg.window(tag="main", label="STS Game Viewer", no_title_bar=True):
            # Header bar
            with dpg.child_window(height=50, border=False):
                with dpg.group(horizontal=True):
                    dpg.add_text("STS GAME VIEWER", color=self.COLORS["gold"])
                    dpg.add_spacer(width=50)
                    dpg.add_text("Seed:", color=self.COLORS["dim"])
                    dpg.add_input_text(
                        tag="seed_input",
                        default_value="1234567890",
                        width=180,
                        callback=self._on_seed_change
                    )
                    dpg.add_button(label="Generate", callback=self._generate_predictions, width=100)
                    dpg.add_spacer(width=20)
                    dpg.add_text("", tag="status_text", color=self.COLORS["green"])

            dpg.add_separator()

            # Main content - 3 columns
            with dpg.group(horizontal=True):
                # LEFT COLUMN - Current State
                with dpg.child_window(width=380, border=True, tag="left_panel"):
                    self._create_state_panel()

                dpg.add_spacer(width=8)

                # MIDDLE COLUMN - Predictions
                with dpg.child_window(width=480, border=True, tag="middle_panel"):
                    self._create_predictions_panel()

                dpg.add_spacer(width=8)

                # RIGHT COLUMN - Parity
                with dpg.child_window(border=True, tag="right_panel"):
                    self._create_parity_panel()

        self._update_display()

    def _create_state_panel(self):
        dpg.add_text("CURRENT STATE", color=self.COLORS["gold"])
        dpg.add_separator()
        dpg.add_spacer(height=5)

        # Floor/Act/HP row
        with dpg.group(horizontal=True):
            dpg.add_text("", tag="floor_text", color=self.COLORS["text"])
            dpg.add_spacer(width=20)
            dpg.add_text("", tag="hp_text", color=self.COLORS["red"])
            dpg.add_spacer(width=20)
            dpg.add_text("", tag="gold_text", color=self.COLORS["gold"])

        dpg.add_spacer(height=5)
        dpg.add_text("", tag="boss_text", color=self.COLORS["purple"])

        # HP Bar
        dpg.add_spacer(height=5)
        dpg.add_progress_bar(tag="hp_bar", default_value=1.0, width=-1)

        # Relics section
        dpg.add_spacer(height=15)
        dpg.add_text("RELICS", color=self.COLORS["dim"])
        dpg.add_separator()
        with dpg.group(horizontal=True, tag="relics_group"):
            dpg.add_text("None", tag="relics_placeholder", color=self.COLORS["dim"])

        # Potions section
        dpg.add_spacer(height=15)
        dpg.add_text("POTIONS", color=self.COLORS["dim"])
        dpg.add_separator()
        dpg.add_text("", tag="potions_text", wrap=360)

        # Deck section
        dpg.add_spacer(height=15)
        dpg.add_text("DECK", color=self.COLORS["dim"])
        dpg.add_separator()
        dpg.add_text("", tag="deck_text", wrap=360)

        # Path section
        dpg.add_spacer(height=15)
        dpg.add_text("PATH TAKEN", color=self.COLORS["dim"])
        dpg.add_separator()
        dpg.add_text("", tag="path_text", wrap=360)

    def _create_predictions_panel(self):
        dpg.add_text("PREDICTIONS", color=self.COLORS["gold"])
        dpg.add_separator()

        with dpg.tab_bar():
            for act in [1, 2, 3]:
                with dpg.tab(label=f"Act {act}"):
                    dpg.add_spacer(height=5)

                    # Boss
                    dpg.add_text("BOSS", color=self.COLORS["purple"])
                    dpg.add_text("", tag=f"pred_boss_{act}", color=self.COLORS["red"])

                    dpg.add_spacer(height=10)

                    # Monsters
                    dpg.add_text("MONSTERS (next 10)", color=self.COLORS["dim"])
                    dpg.add_text("", tag=f"pred_monsters_{act}", wrap=460)

                    dpg.add_spacer(height=10)

                    # Elites
                    dpg.add_text("ELITES", color=self.COLORS["dim"])
                    dpg.add_text("", tag=f"pred_elites_{act}", wrap=460)

    def _create_parity_panel(self):
        dpg.add_text("PARITY CHECK", color=self.COLORS["gold"])
        dpg.add_separator()

        dpg.add_spacer(height=10)
        dpg.add_text("", tag="parity_status", color=self.COLORS["green"])

        dpg.add_spacer(height=15)
        dpg.add_text("Monster Comparison:", color=self.COLORS["dim"])
        dpg.add_text("", tag="monster_parity", wrap=400)

        dpg.add_spacer(height=15)
        dpg.add_text("Elite Comparison:", color=self.COLORS["dim"])
        dpg.add_text("", tag="elite_parity", wrap=400)

        dpg.add_spacer(height=15)
        dpg.add_text("Boss:", color=self.COLORS["dim"])
        dpg.add_text("", tag="boss_parity")

        dpg.add_spacer(height=20)
        dpg.add_separator()
        dpg.add_text("RNG COUNTERS", color=self.COLORS["dim"])
        dpg.add_text("", tag="rng_counters", wrap=400)

    def _on_seed_change(self, sender, app_data):
        self.state.seed = app_data

    def _generate_predictions(self):
        seed = dpg.get_value("seed_input")
        if not seed:
            dpg.set_value("status_text", "Enter a seed!")
            dpg.configure_item("status_text", color=self.COLORS["red"])
            return

        dpg.set_value("status_text", "Generating...")
        dpg.configure_item("status_text", color=self.COLORS["gold"])

        try:
            self.predictions = predict_all_acts(seed, include_act4=True)

            for act in [1, 2, 3]:
                act_data = self.predictions.get(f"act{act}", {})
                monsters = act_data.get("monsters", [])
                elites = act_data.get("elites", [])
                boss = act_data.get("boss", "Unknown")

                monster_text = "\n".join(f"{i+1:2}. {m}" for i, m in enumerate(monsters[:10]))
                elite_text = "\n".join(f"{i+1}. {e}" for i, e in enumerate(elites[:5]))

                dpg.set_value(f"pred_boss_{act}", boss)
                dpg.set_value(f"pred_monsters_{act}", monster_text or "None")
                dpg.set_value(f"pred_elites_{act}", elite_text or "None")

            dpg.set_value("status_text", f"✓ Predictions ready for: {seed}")
            dpg.configure_item("status_text", color=self.COLORS["green"])
        except Exception as e:
            dpg.set_value("status_text", f"Error: {e}")
            dpg.configure_item("status_text", color=self.COLORS["red"])

    def _watch_save(self):
        last_mtime = 0
        while self._running:
            try:
                save_path = SAVE_DIR / "WATCHER.autosave"
                if save_path.exists():
                    mtime = save_path.stat().st_mtime
                    if mtime > last_mtime:
                        last_mtime = mtime
                        save = load_save("WATCHER")
                        if save:
                            self._update_from_save(save)
            except Exception:
                pass
            time.sleep(1)

    def _update_from_save(self, save: dict):
        self.state.seed = str(save.get('seed', ''))
        self.state.floor = save.get('floor_num', 0)
        self.state.hp = save.get('current_health', 0)
        self.state.max_hp = save.get('max_health', 80)
        self.state.gold = save.get('gold', 0)
        self.state.boss = save.get('boss', '')
        self.state.monsters = save.get('monster_list', [])
        self.state.elites = save.get('elite_list', save.get('elite_monster_list', []))
        self.state.relics = save.get('relics', [])
        self.state.deck = [
            c.get('id', '') + ('+' if c.get('upgrades', 0) > 0 else '')
            for c in save.get('cards', [])
        ]
        self.state.potions = [p for p in save.get('potions', []) if p != "Potion Slot"]
        self.state.path = save.get('metric_path_per_floor', [])

        act_map = {"Exordium": 1, "TheCity": 2, "TheBeyond": 3, "TheEnding": 4}
        self.state.act = act_map.get(save.get('level_name', 'Exordium'), 1)

        self._update_display()
        self._check_parity(save)

    def _update_display(self):
        dpg.set_value("floor_text", f"Floor {self.state.floor} | Act {self.state.act}")
        dpg.set_value("hp_text", f"♥ {self.state.hp}/{self.state.max_hp}")
        dpg.set_value("gold_text", f"⬡ {self.state.gold}")
        dpg.set_value("boss_text", f"Boss: {self.state.boss}" if self.state.boss else "")

        # HP bar
        hp_pct = self.state.hp / self.state.max_hp if self.state.max_hp > 0 else 0
        dpg.set_value("hp_bar", hp_pct)

        # Relics as text (could show images if loaded)
        relic_text = ", ".join(self.state.relics) if self.state.relics else "None"
        if dpg.does_item_exist("relics_placeholder"):
            dpg.set_value("relics_placeholder", relic_text)

        # Potions
        potion_text = ", ".join(self.state.potions) if self.state.potions else "Empty"
        dpg.set_value("potions_text", potion_text)

        # Deck (show count and first few)
        deck_count = len(self.state.deck)
        deck_preview = ", ".join(self.state.deck[:15])
        if deck_count > 15:
            deck_preview += f"... (+{deck_count - 15} more)"
        dpg.set_value("deck_text", f"({deck_count} cards) {deck_preview}" if self.state.deck else "Empty")

        # Path
        path_text = " → ".join(self.state.path) if self.state.path else "Not started"
        dpg.set_value("path_text", path_text)

        if self.state.seed:
            dpg.set_value("seed_input", self.state.seed)

    def _check_parity(self, save: dict):
        if not self.predictions:
            # Auto-generate if we have a seed
            if self.state.seed:
                self._generate_predictions()
            return

        act_key = f"act{self.state.act}"
        if act_key not in self.predictions:
            return

        pred = self.predictions[act_key]
        pred_monsters = pred.get("monsters", [])
        pred_elites = pred.get("elites", [])
        pred_boss = pred.get("boss", "")

        # Monster comparison
        monster_lines = []
        m_matches = 0
        for i in range(min(len(pred_monsters), len(self.state.monsters), 13)):
            p, a = pred_monsters[i], self.state.monsters[i]
            match = p == a
            if match:
                m_matches += 1
            symbol = "✓" if match else "✗"
            monster_lines.append(f"{i+1:2}. {p[:18]:<18} vs {a[:18]:<18} {symbol}")

        # Elite comparison
        elite_lines = []
        e_matches = 0
        for i in range(min(len(pred_elites), len(self.state.elites), 10)):
            p, a = pred_elites[i], self.state.elites[i]
            match = p == a
            if match:
                e_matches += 1
            symbol = "✓" if match else "✗"
            elite_lines.append(f"{i+1}. {p[:20]:<20} vs {a[:20]:<20} {symbol}")

        # Boss
        boss_match = pred_boss == self.state.boss
        boss_symbol = "✓" if boss_match else "✗"

        # Calculate totals
        total = len(self.state.monsters) + len(self.state.elites) + 1
        matches = m_matches + e_matches + (1 if boss_match else 0)
        pct = (matches / total * 100) if total > 0 else 0

        # Update UI
        color = self.COLORS["green"] if pct == 100 else self.COLORS["red"] if pct < 80 else self.COLORS["gold"]
        dpg.set_value("parity_status", f"PARITY: {matches}/{total} ({pct:.0f}%)")
        dpg.configure_item("parity_status", color=color)

        dpg.set_value("monster_parity", "\n".join(monster_lines) or "No data")
        dpg.set_value("elite_parity", "\n".join(elite_lines) or "No data")
        dpg.set_value("boss_parity", f"{pred_boss} vs {self.state.boss} {boss_symbol}")

        # RNG counters
        rng_text = f"""card_seed_count: {save.get('card_seed_count', 'N/A')}
monster_seed_count: {save.get('monster_seed_count', 'N/A')}
relic_seed_count: {save.get('relic_seed_count', 'N/A')}
potion_seed_count: {save.get('potion_seed_count', 'N/A')}
event_seed_count: {save.get('event_seed_count', 'N/A')}"""
        dpg.set_value("rng_counters", rng_text)


def main():
    viewer = GameViewer()
    viewer.run()


if __name__ == "__main__":
    main()
