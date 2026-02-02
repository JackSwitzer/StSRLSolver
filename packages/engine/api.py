"""
Slay the Spire Core - Simple API Entry Point

This module provides a simplified, curated set of exports for common use cases.
For the full API, use `from core import <symbol>` directly.

Quick Start Examples:

1. Predict encounters for a seed:
    ```python
    from core.api import predict_all_acts, predict_all_bosses_extended

    acts = predict_all_acts("MY_SEED")
    bosses = predict_all_bosses_extended("MY_SEED")
    ```

2. Run a game simulation:
    ```python
    from core.api import GameRunner, GamePhase

    runner = GameRunner(seed="TEST123", ascension=20)
    while not runner.game_over:
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
    ```

3. RNG utilities:
    ```python
    from core.api import Random, seed_to_long, long_to_seed

    rng = Random(seed_to_long("MY_SEED"))
    value = rng.random()  # Float [0, 1)
    ```
"""

# =============================================================================
# RNG System
# =============================================================================
from .state.rng import (
    Random,
    seed_to_long,
    long_to_seed,
    XorShift128,
    GameRNG,
)

# =============================================================================
# Encounter Prediction
# =============================================================================
from .generation.encounters import (
    # Main prediction functions
    predict_all_acts,
    predict_all_bosses,
    predict_all_bosses_extended,
    predict_act_encounters,
    # Generation functions
    generate_exordium_encounters,
    generate_city_encounters,
    generate_beyond_encounters,
    generate_ending_encounters,
    # Boss lists
    EXORDIUM_BOSSES,
    CITY_BOSSES,
    BEYOND_BOSSES,
)

# =============================================================================
# Map Generation
# =============================================================================
from .generation.map import (
    MapGenerator,
    MapGeneratorConfig,
    MapRoomNode,
    MapEdge,
    RoomType,
    generate_act4_map,
    map_to_string,
)

# =============================================================================
# Game Runner
# =============================================================================
from .game import (
    GameRunner,
    GamePhase,
    GameAction,
    PathAction,
    NeowAction,
    CombatAction,
    RewardAction,
    EventAction,
    ShopAction,
    RestAction,
    TreasureAction,
    BossRewardAction,
    DecisionLogEntry,
)

# =============================================================================
# Combat Simulation
# =============================================================================
from .calc.combat_sim import (
    CombatSimulator,
    ActionType,
    Action,
    CombatResult,
)

# =============================================================================
# Content: Cards
# =============================================================================
from .content.cards import (
    Card,
    CardType,
    CardRarity,
    CardTarget,
    CardColor,
    get_card,
    get_starting_deck,
    WATCHER_CARDS,
)

# =============================================================================
# Content: Enemies
# =============================================================================
from .content.enemies import (
    Enemy,
    EnemyState,
    MoveInfo,
    Intent,
    EnemyType,
    create_enemy,
    ENEMY_CLASSES,
    # Common enemies
    JawWorm,
    Cultist,
    GremlinNob,
    Lagavulin,
    SlimeBoss,
)

# =============================================================================
# Content: Relics
# =============================================================================
from .content.relics import (
    Relic,
    RelicTier,
    get_relic,
    get_relics_by_tier,
    get_starter_relic,
    ALL_RELICS,
    BOSS_RELICS,
)

# =============================================================================
# Content: Potions
# =============================================================================
from .content.potions import (
    Potion,
    PotionRarity,
    get_potion_by_id,
    get_potion_pool,
    ALL_POTIONS,
)

__all__ = [
    # RNG
    "Random", "seed_to_long", "long_to_seed", "XorShift128", "GameRNG",
    # Encounters
    "predict_all_acts", "predict_all_bosses", "predict_all_bosses_extended",
    "predict_act_encounters",
    "generate_exordium_encounters", "generate_city_encounters",
    "generate_beyond_encounters", "generate_ending_encounters",
    "EXORDIUM_BOSSES", "CITY_BOSSES", "BEYOND_BOSSES",
    # Map
    "MapGenerator", "MapGeneratorConfig", "MapRoomNode", "MapEdge", "RoomType",
    "generate_act4_map", "map_to_string",
    # Game Runner
    "GameRunner", "GamePhase", "GameAction",
    "PathAction", "NeowAction", "CombatAction", "RewardAction",
    "EventAction", "ShopAction", "RestAction", "TreasureAction", "BossRewardAction",
    "DecisionLogEntry",
    # Combat
    "CombatSimulator", "ActionType", "Action", "CombatResult",
    # Cards
    "Card", "CardType", "CardRarity", "CardTarget", "CardColor",
    "get_card", "get_starting_deck", "WATCHER_CARDS",
    # Enemies
    "Enemy", "EnemyState", "MoveInfo", "Intent", "EnemyType",
    "create_enemy", "ENEMY_CLASSES",
    "JawWorm", "Cultist", "GremlinNob", "Lagavulin", "SlimeBoss",
    # Relics
    "Relic", "RelicTier", "get_relic", "get_relics_by_tier", "get_starter_relic",
    "ALL_RELICS", "BOSS_RELICS",
    # Potions
    "Potion", "PotionRarity", "get_potion_by_id", "get_potion_pool", "ALL_POTIONS",
]
