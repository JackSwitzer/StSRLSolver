"""Data-driven passive relic definitions.

These relics modify game behavior through flags rather than triggers.
The game loop checks these flags at appropriate times.
"""

# Passive relic effects - game code checks these flags
PASSIVE_RELICS = {
    # Prevent specific powers from being applied
    "Ginger": {"prevent_power": "Weakened"},
    "Turnip": {"prevent_power": "Frail"},

    # Prevent specific actions
    "Mark of the Bloom": {"prevent_heal": True},
    "Coffee Dripper": {"prevent_rest": True},
    "Fusion Hammer": {"prevent_upgrade": True},
    "Sozu": {"prevent_potions": True},
    "Ectoplasm": {"prevent_gold_gain": True},

    # Modify game constants
    "Runic Pyramid": {"no_discard_at_turn_end": True},
    "Sacred Bark": {"double_potion_effect": True},
    "Membership Card": {"shop_discount": 0.5},
    "The Courier": {"shop_always_has_relic": True},
    "Smiling Mask": {"card_remove_cost": 0},

    # Damage/block modifiers (checked in calc pipeline)
    "Odd Mushroom": {"player_vulnerable_multiplier": 1.25},  # Instead of 1.5
    "Paper Frog": {"enemy_vulnerable_multiplier": 1.75},     # Instead of 1.5
    "Paper Crane": {"enemy_weak_multiplier": 0.60},          # Instead of 0.75
    "Tungsten Rod": {"reduce_hp_loss": 1},

    # Card play modifiers
    "Blue Candle": {"can_play_curses": True, "curse_play_damage": 1},
    "Medical Kit": {"can_play_statuses": True},
    "Ice Cream": {"energy_persists": True},

    # Rest site modifiers
    "Regal Pillow": {"rest_heal_bonus": 15},
    "Dream Catcher": {"card_reward_on_rest": True},
    "Girya": {"lift_option": True, "lift_uses": 3},
    "Peace Pipe": {"toke_option": True},
    "Shovel": {"dig_option": True},
    "Golden Eye": {"scry_on_rest": 5},  # Watcher only
    "Melange": {"scry_on_rest": 3},  # Watcher only

    # Event modifiers
    "Juzu Bracelet": {"no_monster_rooms": True},
    "N'loth's Gift": {"double_event_rewards": True},  # One-time
    "Wing Boots": {"fly_uses": 3},

    # Gold modifiers
    "Golden Idol": {"gold_bonus_percent": 25},

    # Chest/reward modifiers
    "Black Star": {"double_elite_relics": True},
    "Cursed Key": {"curse_on_chest": True},
}


def has_passive_effect(run_state, effect_type: str) -> bool:
    """Check if player has a relic with the given passive effect."""
    for relic_id in run_state.relics:
        if relic_id in PASSIVE_RELICS:
            if effect_type in PASSIVE_RELICS[relic_id]:
                return True
    return False


def get_passive_value(run_state, effect_type: str, default=None):
    """Get the value of a passive effect from equipped relics."""
    for relic_id in run_state.relics:
        if relic_id in PASSIVE_RELICS:
            effects = PASSIVE_RELICS[relic_id]
            if effect_type in effects:
                return effects[effect_type]
    return default


def get_prevented_power(run_state) -> str:
    """Get the power name that should be prevented, if any."""
    for relic_id in run_state.relics:
        if relic_id in PASSIVE_RELICS:
            effects = PASSIVE_RELICS[relic_id]
            if "prevent_power" in effects:
                return effects["prevent_power"]
    return None
