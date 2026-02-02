"""Compatibility shim - use packages.engine.generation.rewards directly."""
from packages.engine.generation.rewards import *  # noqa: F401,F403
from packages.engine.generation.rewards import (  # noqa: F401
    _roll_card_rarity,
    _roll_elite_relic_tier,
    _roll_normal_relic_tier,
    _roll_shop_relic_tier,
)
