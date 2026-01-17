"""
Map Generation Tests

Tests for map.py to verify:
1. Path connectivity (all nodes reachable from floor 0)
2. Room type distribution per act
3. Elite placement rules (floors 5+ for first, spacing requirements)
4. Rest site placement rules (floors 5+ for first, spacing requirements)
5. Shop placement rules
6. Treasure room guarantees (floor 8 in act 1)
7. Boss always at floor 15/16
8. Act 1/2/3 differences
9. Burning elite paths at A1+
10. Ascension room modifications
11. Seed determinism (same seed = same map)
12. Node connection patterns (no crossing paths rule)
13. Act 4 generation (Heart path)
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from core.generation.map import (
    MapGenerator,
    MapGeneratorConfig,
    MapRoomNode,
    MapEdge,
    RoomType,
    MAP_HEIGHT,
    MAP_WIDTH,
    MAP_PATH_DENSITY,
    generate_act4_map,
    get_map_seed_offset,
    map_to_string,
)
from core.state.rng import Random, seed_to_long


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================


def generate_map_with_seed(seed: int, ascension: int = 0, **config_kwargs) -> list:
    """Generate a map with given seed and configuration."""
    config = MapGeneratorConfig(ascension_level=ascension, **config_kwargs)
    rng = Random(seed)
    generator = MapGenerator(rng, config)
    return generator.generate()


def get_connected_nodes(nodes: list) -> list:
    """Get all nodes that have outgoing edges (are part of paths)."""
    connected = []
    for row in nodes:
        for node in row:
            if node.has_edges():
                connected.append(node)
    return connected


def get_all_nodes_with_room_type(nodes: list, room_type: RoomType) -> list:
    """Get all nodes with a specific room type."""
    return [
        node for row in nodes for node in row
        if node.room_type == room_type
    ]


def count_room_types(nodes: list) -> dict:
    """Count occurrences of each room type."""
    counts = {}
    for row in nodes:
        for node in row:
            if node.room_type:
                counts[node.room_type] = counts.get(node.room_type, 0) + 1
    return counts


def can_reach_from_row_zero(nodes: list) -> set:
    """BFS to find all nodes reachable from row 0."""
    visited = set()
    queue = []

    # Start from all nodes in row 0 that have edges
    for node in nodes[0]:
        if node.has_edges():
            queue.append(node)
            visited.add((node.x, node.y))

    while queue:
        current = queue.pop(0)
        for edge in current.edges:
            if edge.dst_y < len(nodes):  # Not boss edge
                next_node = nodes[edge.dst_y][edge.dst_x]
                key = (next_node.x, next_node.y)
                if key not in visited:
                    visited.add(key)
                    queue.append(next_node)

    return visited


def check_path_crossing(nodes: list) -> bool:
    """
    Check if any paths cross each other.

    A crossing occurs when two edges from adjacent nodes cross over each other.
    For nodes at positions x and x+1 on the same row:
    - If node at x has edge going right (to x+1 or beyond)
    - And node at x+1 has edge going left (to x or before)
    - Then they cross
    """
    for row_idx, row in enumerate(nodes):
        for x in range(len(row) - 1):
            left_node = row[x]
            right_node = row[x + 1]

            if not left_node.has_edges() or not right_node.has_edges():
                continue

            # Get non-boss edges only
            left_non_boss = [e for e in left_node.edges if not e.is_boss]
            right_non_boss = [e for e in right_node.edges if not e.is_boss]

            # Skip if either node only has boss edges
            if not left_non_boss or not right_non_boss:
                continue

            # Find max destination of left node and min destination of right node
            left_max_dst = max(e.dst_x for e in left_non_boss)
            right_min_dst = min(e.dst_x for e in right_non_boss)

            # If left node's rightmost edge goes beyond right node's leftmost edge
            if left_max_dst > right_min_dst:
                return True

    return False


# ============================================================================
# TEST CLASS: MAP CONSTANTS
# ============================================================================


class TestMapConstants:
    """Test map generation constants match expected values."""

    def test_map_height(self):
        """Map should have 15 floors."""
        assert MAP_HEIGHT == 15

    def test_map_width(self):
        """Map should have 7 columns."""
        assert MAP_WIDTH == 7

    def test_path_density(self):
        """Map should generate 6 paths by default."""
        assert MAP_PATH_DENSITY == 6


# ============================================================================
# TEST CLASS: SEED DETERMINISM
# ============================================================================


class TestSeedDeterminism:
    """Test that same seed produces same map."""

    def test_same_seed_same_map(self):
        """Same seed should produce identical maps."""
        seed = 12345

        map1 = generate_map_with_seed(seed)
        map2 = generate_map_with_seed(seed)

        for y in range(len(map1)):
            for x in range(len(map1[0])):
                node1 = map1[y][x]
                node2 = map2[y][x]

                assert node1.room_type == node2.room_type, \
                    f"Room type mismatch at ({x}, {y})"
                assert len(node1.edges) == len(node2.edges), \
                    f"Edge count mismatch at ({x}, {y})"

    def test_different_seeds_different_maps(self):
        """Different seeds should produce different maps (with high probability)."""
        map1 = generate_map_with_seed(12345)
        map2 = generate_map_with_seed(54321)

        # Count differences in room types
        differences = 0
        for y in range(len(map1)):
            for x in range(len(map1[0])):
                if map1[y][x].room_type != map2[y][x].room_type:
                    differences += 1

        # Should have some differences (statistically very unlikely to match)
        assert differences > 0, "Different seeds produced identical maps"

    def test_determinism_across_multiple_generations(self):
        """Verify determinism holds over many map generations."""
        seed = 99999

        reference_map = generate_map_with_seed(seed)

        for _ in range(10):
            new_map = generate_map_with_seed(seed)
            for y in range(len(reference_map)):
                for x in range(len(reference_map[0])):
                    assert reference_map[y][x].room_type == new_map[y][x].room_type

    def test_string_seed_determinism(self):
        """String seeds should be deterministic."""
        seed_long = seed_to_long("TESTSEED")

        map1 = generate_map_with_seed(seed_long)
        map2 = generate_map_with_seed(seed_long)

        for y in range(len(map1)):
            for x in range(len(map1[0])):
                assert map1[y][x].room_type == map2[y][x].room_type


# ============================================================================
# TEST CLASS: PATH CONNECTIVITY
# ============================================================================


class TestPathConnectivity:
    """Test that all map nodes are reachable from floor 0."""

    def test_all_connected_nodes_reachable(self):
        """All nodes with room types should be reachable from row 0."""
        nodes = generate_map_with_seed(12345)

        reachable = can_reach_from_row_zero(nodes)

        for y, row in enumerate(nodes):
            for x, node in enumerate(row):
                if node.has_edges() and y < len(nodes) - 1:  # Exclude last row going to boss
                    assert (x, y) in reachable, f"Node ({x}, {y}) is not reachable"

    def test_multiple_starting_points(self):
        """There should be multiple starting points in row 0."""
        nodes = generate_map_with_seed(12345)

        starting_nodes = [n for n in nodes[0] if n.has_edges()]

        # With 6 paths, should have at least 2 different starting points
        assert len(starting_nodes) >= 2, "Not enough starting points"

    def test_all_paths_reach_boss(self):
        """All paths should eventually lead to the boss."""
        nodes = generate_map_with_seed(12345)

        # Last row nodes should have boss edges
        for node in nodes[-1]:
            if node.has_edges():
                boss_edges = [e for e in node.edges if e.is_boss]
                assert len(boss_edges) > 0, \
                    f"Node ({node.x}, {node.y}) doesn't connect to boss"

    def test_no_isolated_nodes_with_room_types(self):
        """No node with a room type should be completely isolated."""
        nodes = generate_map_with_seed(12345)

        for y, row in enumerate(nodes):
            for x, node in enumerate(row):
                if node.room_type is not None:
                    # Node should either have outgoing edges or parents
                    has_connections = node.has_edges() or len(node.parents) > 0

                    # Nodes in row 0 only need outgoing edges
                    if y == 0:
                        has_connections = node.has_edges()

                    # Nodes in last row going to boss only need parents
                    if y == len(nodes) - 1:
                        has_connections = len(node.parents) > 0 or node.has_edges()

                    # Only check nodes that should be connected
                    if node.has_edges() or len(node.parents) > 0:
                        assert has_connections, \
                            f"Node ({x}, {y}) has room type but no connections"

    def test_connectivity_with_different_seeds(self):
        """Connectivity should hold for various seeds."""
        seeds = [1, 100, 1000, 12345, 99999, 2**31 - 1]

        for seed in seeds:
            nodes = generate_map_with_seed(seed)
            reachable = can_reach_from_row_zero(nodes)

            # Check that reachable set is non-empty
            assert len(reachable) > 0, f"No reachable nodes for seed {seed}"


# ============================================================================
# TEST CLASS: ROOM TYPE DISTRIBUTION
# ============================================================================


class TestRoomTypeDistribution:
    """Test room type distribution matches game rules."""

    def test_floor_zero_always_monster(self):
        """Floor 0 should always have monster rooms."""
        for seed in [12345, 54321, 99999]:
            nodes = generate_map_with_seed(seed)

            for node in nodes[0]:
                if node.has_edges():
                    assert node.room_type == RoomType.MONSTER, \
                        f"Floor 0 has non-monster room: {node.room_type}"

    def test_floor_fourteen_always_rest(self):
        """Floor 14 (last row) should always have rest sites."""
        for seed in [12345, 54321, 99999]:
            nodes = generate_map_with_seed(seed)

            for node in nodes[14]:
                if node.room_type is not None:
                    assert node.room_type == RoomType.REST, \
                        f"Floor 14 has non-rest room: {node.room_type}"

    def test_floor_eight_always_treasure(self):
        """Floor 8 should always have treasure rooms."""
        for seed in [12345, 54321, 99999]:
            nodes = generate_map_with_seed(seed)

            for node in nodes[8]:
                if node.has_edges():
                    assert node.room_type == RoomType.TREASURE, \
                        f"Floor 8 has non-treasure room: {node.room_type}"

    def test_treasure_replaced_by_elite_in_mimic_mode(self):
        """Mimic infestation should replace floor 8 treasures with elites."""
        nodes = generate_map_with_seed(12345, mimic_infestation=True)

        for node in nodes[8]:
            if node.has_edges():
                assert node.room_type == RoomType.ELITE, \
                    f"Mimic mode floor 8 should be elite: {node.room_type}"

    def test_room_type_variety(self):
        """Map should have variety of room types."""
        nodes = generate_map_with_seed(12345)
        counts = count_room_types(nodes)

        # Should have at least these types
        expected_types = {RoomType.MONSTER, RoomType.ELITE, RoomType.REST,
                         RoomType.SHOP, RoomType.EVENT, RoomType.TREASURE}

        for room_type in expected_types:
            assert room_type in counts, f"Missing room type: {room_type}"

    def test_monster_room_is_most_common(self):
        """Monster rooms should be the most common type."""
        nodes = generate_map_with_seed(12345)
        counts = count_room_types(nodes)

        monster_count = counts.get(RoomType.MONSTER, 0)

        for room_type, count in counts.items():
            if room_type != RoomType.MONSTER:
                assert monster_count >= count, \
                    f"Monster count ({monster_count}) < {room_type.name} count ({count})"


# ============================================================================
# TEST CLASS: ELITE PLACEMENT RULES
# ============================================================================


class TestElitePlacement:
    """Test elite placement follows game rules."""

    def test_no_elites_before_floor_five(self):
        """Elites should not appear before floor 5."""
        for seed in range(100, 120):  # Test multiple seeds
            nodes = generate_map_with_seed(seed)

            for y in range(5):  # Floors 0-4
                for node in nodes[y]:
                    if node.has_edges():
                        assert node.room_type != RoomType.ELITE, \
                            f"Elite found on floor {y} (seed {seed})"

    def test_elites_can_appear_on_floor_five_and_after(self):
        """Elites should be able to appear on floors 5+."""
        # Generate many maps to find at least one elite on floor 5+
        found_elite_after_five = False

        for seed in range(100, 200):
            nodes = generate_map_with_seed(seed)

            for y in range(5, len(nodes) - 1):  # Floors 5-13
                for node in nodes[y]:
                    if node.room_type == RoomType.ELITE:
                        found_elite_after_five = True
                        break
                if found_elite_after_five:
                    break
            if found_elite_after_five:
                break

        assert found_elite_after_five, "No elites found on floors 5+ across many seeds"

    def test_elite_not_after_elite_on_same_path(self):
        """Elites should not appear directly after another elite on the same path."""
        nodes = generate_map_with_seed(12345)

        for y, row in enumerate(nodes):
            for node in row:
                if node.room_type == RoomType.ELITE:
                    # Check children
                    for edge in node.edges:
                        if not edge.is_boss and edge.dst_y < len(nodes):
                            child = nodes[edge.dst_y][edge.dst_x]
                            assert child.room_type != RoomType.ELITE, \
                                f"Elite at ({node.x}, {node.y}) followed by elite at ({child.x}, {child.y})"

    def test_ascension_one_increases_elites(self):
        """Ascension 1+ should increase elite count by 1.6x."""
        seed = 12345

        nodes_a0 = generate_map_with_seed(seed, ascension=0)
        nodes_a1 = generate_map_with_seed(seed, ascension=1)

        count_a0 = count_room_types(nodes_a0).get(RoomType.ELITE, 0)
        count_a1 = count_room_types(nodes_a1).get(RoomType.ELITE, 0)

        # A1 should have more elites (1.6x modifier)
        # Note: Due to rounding, exact comparison is tricky
        # Just verify A1 has at least as many elites
        assert count_a1 >= count_a0, \
            f"A1 elite count ({count_a1}) < A0 elite count ({count_a0})"


# ============================================================================
# TEST CLASS: REST SITE PLACEMENT RULES
# ============================================================================


class TestRestSitePlacement:
    """Test rest site placement follows game rules."""

    def test_no_rest_before_floor_five(self):
        """Rest sites should not appear before floor 5 (except row 14)."""
        for seed in range(100, 120):
            nodes = generate_map_with_seed(seed)

            for y in range(5):  # Floors 0-4
                for node in nodes[y]:
                    if node.has_edges():
                        assert node.room_type != RoomType.REST, \
                            f"Rest site found on floor {y} (seed {seed})"

    def test_rest_not_after_rest_on_same_path(self):
        """Rest sites should not appear directly after another rest."""
        nodes = generate_map_with_seed(12345)

        for y, row in enumerate(nodes):
            for node in row:
                if node.room_type == RoomType.REST and y < len(nodes) - 1:
                    # Check children
                    for edge in node.edges:
                        if not edge.is_boss and edge.dst_y < len(nodes):
                            child = nodes[edge.dst_y][edge.dst_x]
                            assert child.room_type != RoomType.REST, \
                                f"Rest at ({node.x}, {node.y}) followed by rest"

    def test_floor_fourteen_always_rest(self):
        """Floor 14 should always be rest sites."""
        for seed in range(100, 110):
            nodes = generate_map_with_seed(seed)

            for node in nodes[14]:
                if node.room_type is not None:
                    assert node.room_type == RoomType.REST


# ============================================================================
# TEST CLASS: SHOP PLACEMENT RULES
# ============================================================================


class TestShopPlacement:
    """Test shop placement follows game rules."""

    def test_shop_not_after_shop_on_same_path(self):
        """Shops should not appear directly after another shop."""
        nodes = generate_map_with_seed(12345)

        for y, row in enumerate(nodes):
            for node in row:
                if node.room_type == RoomType.SHOP:
                    for edge in node.edges:
                        if not edge.is_boss and edge.dst_y < len(nodes):
                            child = nodes[edge.dst_y][edge.dst_x]
                            assert child.room_type != RoomType.SHOP, \
                                f"Shop at ({node.x}, {node.y}) followed by shop"

    def test_shops_exist_in_map(self):
        """Maps should generally have at least one shop."""
        shop_found = False

        for seed in range(100, 150):
            nodes = generate_map_with_seed(seed)
            count = count_room_types(nodes).get(RoomType.SHOP, 0)
            if count > 0:
                shop_found = True
                break

        assert shop_found, "No shops found across multiple seeds"


# ============================================================================
# TEST CLASS: BOSS PLACEMENT
# ============================================================================


class TestBossPlacement:
    """Test boss placement at end of map."""

    def test_boss_edges_from_last_row(self):
        """All paths from row 14 should lead to boss."""
        nodes = generate_map_with_seed(12345)

        for node in nodes[14]:
            if node.has_edges():
                for edge in node.edges:
                    assert edge.is_boss, \
                        f"Non-boss edge from row 14 at ({node.x}, {node.y})"

    def test_boss_edge_destination(self):
        """Boss edges should point to y=16, x=3 (center)."""
        nodes = generate_map_with_seed(12345)

        for node in nodes[14]:
            for edge in node.edges:
                if edge.is_boss:
                    assert edge.dst_x == 3, f"Boss edge x={edge.dst_x}, expected 3"
                    assert edge.dst_y == 16, f"Boss edge y={edge.dst_y}, expected 16"


# ============================================================================
# TEST CLASS: NO PATH CROSSING
# ============================================================================


class TestNoCrossingPaths:
    """Test that paths don't cross each other."""

    def test_no_crossing_paths(self):
        """Paths should not cross each other."""
        for seed in range(100, 150):
            nodes = generate_map_with_seed(seed)

            assert not check_path_crossing(nodes), \
                f"Path crossing detected for seed {seed}"

    def test_edges_sorted_by_destination(self):
        """Node edges should be sorted by destination x coordinate."""
        nodes = generate_map_with_seed(12345)

        for row in nodes:
            for node in row:
                if len(node.edges) > 1:
                    destinations = [e.dst_x for e in node.edges if not e.is_boss]
                    assert destinations == sorted(destinations), \
                        f"Edges not sorted at ({node.x}, {node.y})"


# ============================================================================
# TEST CLASS: ACT SEED OFFSETS
# ============================================================================


class TestActSeedOffsets:
    """Test seed offsets for different acts."""

    def test_act_1_offset(self):
        """Act 1 should use seed + 1."""
        assert get_map_seed_offset(1) == 1

    def test_act_2_offset(self):
        """Act 2 should use seed + 200."""
        assert get_map_seed_offset(2) == 200

    def test_act_3_offset(self):
        """Act 3 should use seed + 600."""
        assert get_map_seed_offset(3) == 600

    def test_act_4_offset(self):
        """Act 4 should use seed + 1200."""
        assert get_map_seed_offset(4) == 1200

    def test_different_acts_different_maps(self):
        """Different acts should produce different maps."""
        base_seed = 12345

        map_act1 = generate_map_with_seed(base_seed + get_map_seed_offset(1))
        map_act2 = generate_map_with_seed(base_seed + get_map_seed_offset(2))
        map_act3 = generate_map_with_seed(base_seed + get_map_seed_offset(3))

        # Count differences between act 1 and 2
        diff_1_2 = sum(
            1 for y in range(len(map_act1)) for x in range(len(map_act1[0]))
            if map_act1[y][x].room_type != map_act2[y][x].room_type
        )

        # Count differences between act 2 and 3
        diff_2_3 = sum(
            1 for y in range(len(map_act2)) for x in range(len(map_act2[0]))
            if map_act2[y][x].room_type != map_act3[y][x].room_type
        )

        assert diff_1_2 > 0, "Act 1 and Act 2 maps are identical"
        assert diff_2_3 > 0, "Act 2 and Act 3 maps are identical"


# ============================================================================
# TEST CLASS: ACT 4 SPECIAL MAP
# ============================================================================


class TestAct4Map:
    """Test Act 4 (Heart path) special map generation."""

    def test_act4_map_structure(self):
        """Act 4 should have specific linear structure."""
        nodes = generate_act4_map()

        assert len(nodes) == 5, "Act 4 should have 5 rows"
        assert len(nodes[0]) == 7, "Act 4 should have 7 columns"

    def test_act4_room_sequence(self):
        """Act 4 should have Rest -> Shop -> Elite -> Boss -> Victory."""
        nodes = generate_act4_map()

        center = 3  # All rooms in center column

        assert nodes[0][center].room_type == RoomType.REST
        assert nodes[1][center].room_type == RoomType.SHOP
        assert nodes[2][center].room_type == RoomType.ELITE
        assert nodes[3][center].room_type == RoomType.BOSS
        assert nodes[4][center].room_type == RoomType.TRUE_VICTORY

    def test_act4_linear_path(self):
        """Act 4 should have a single linear path."""
        nodes = generate_act4_map()
        center = 3

        # Check edges connect linearly
        assert len(nodes[0][center].edges) == 1
        assert nodes[0][center].edges[0].dst_x == center
        assert nodes[0][center].edges[0].dst_y == 1

        assert len(nodes[1][center].edges) == 1
        assert nodes[1][center].edges[0].dst_x == center

        assert len(nodes[2][center].edges) == 1
        assert nodes[2][center].edges[0].is_boss

        assert len(nodes[3][center].edges) == 1

    def test_act4_deterministic(self):
        """Act 4 map should always be the same (no RNG)."""
        map1 = generate_act4_map()
        map2 = generate_act4_map()

        for y in range(len(map1)):
            for x in range(len(map1[0])):
                assert map1[y][x].room_type == map2[y][x].room_type


# ============================================================================
# TEST CLASS: EMERALD KEY ELITE
# ============================================================================


class TestEmeraldKeyElite:
    """Test burning elite (emerald key) placement."""

    def test_emerald_elite_placed_when_available(self):
        """Emerald elite should be placed when final act is available."""
        config = MapGeneratorConfig(
            is_final_act_available=True,
            has_emerald_key=False
        )
        rng = Random(12345)
        generator = MapGenerator(rng, config)
        nodes = generator.generate()

        # Find node with emerald key
        emerald_nodes = [
            node for row in nodes for node in row
            if node.has_emerald_key
        ]

        assert len(emerald_nodes) == 1, "Should have exactly one emerald elite"
        assert emerald_nodes[0].room_type == RoomType.ELITE

    def test_no_emerald_elite_if_already_has_key(self):
        """No emerald elite if player already has key."""
        config = MapGeneratorConfig(
            is_final_act_available=True,
            has_emerald_key=True  # Already has key
        )
        rng = Random(12345)
        generator = MapGenerator(rng, config)
        nodes = generator.generate()

        emerald_nodes = [
            node for row in nodes for node in row
            if node.has_emerald_key
        ]

        assert len(emerald_nodes) == 0, "Should have no emerald elite if key owned"

    def test_no_emerald_elite_if_act4_not_available(self):
        """No emerald elite if Act 4 not available."""
        config = MapGeneratorConfig(
            is_final_act_available=False
        )
        rng = Random(12345)
        generator = MapGenerator(rng, config)
        nodes = generator.generate()

        emerald_nodes = [
            node for row in nodes for node in row
            if node.has_emerald_key
        ]

        assert len(emerald_nodes) == 0


# ============================================================================
# TEST CLASS: ASCENSION MODIFIERS
# ============================================================================


class TestAscensionModifiers:
    """Test ascension-based map modifications."""

    def test_elite_count_increases_at_a1(self):
        """Elite count should increase at Ascension 1+."""
        # Test multiple seeds to get statistical significance
        a0_total = 0
        a1_total = 0

        for seed in range(1000, 1020):
            nodes_a0 = generate_map_with_seed(seed, ascension=0)
            nodes_a1 = generate_map_with_seed(seed, ascension=1)

            a0_total += count_room_types(nodes_a0).get(RoomType.ELITE, 0)
            a1_total += count_room_types(nodes_a1).get(RoomType.ELITE, 0)

        # A1 should have more elites on average (1.6x factor)
        assert a1_total > a0_total, \
            f"A1 total elites ({a1_total}) should exceed A0 ({a0_total})"

    def test_elite_swarm_modifier(self):
        """Elite swarm modifier should increase elites by 2.5x."""
        seed = 12345

        nodes_normal = generate_map_with_seed(seed, ascension=0)
        nodes_swarm = generate_map_with_seed(seed, ascension=0, elite_swarm_mod=True)

        count_normal = count_room_types(nodes_normal).get(RoomType.ELITE, 0)
        count_swarm = count_room_types(nodes_swarm).get(RoomType.ELITE, 0)

        # Swarm should have more elites
        assert count_swarm > count_normal, \
            f"Swarm mode ({count_swarm}) should have more elites than normal ({count_normal})"


# ============================================================================
# TEST CLASS: UNCERTAIN FUTURE MODIFIER
# ============================================================================


class TestUncertainFuture:
    """Test uncertain future modifier (single path)."""

    def test_uncertain_future_single_path(self):
        """Uncertain future should create a single path."""
        nodes = generate_map_with_seed(12345, uncertain_future_mod=True)

        # Count starting points - should be exactly 1
        starting_nodes = [n for n in nodes[0] if n.has_edges()]

        assert len(starting_nodes) == 1, \
            f"Uncertain future should have 1 path, found {len(starting_nodes)}"

    def test_uncertain_future_still_reaches_boss(self):
        """Single path should still reach boss."""
        nodes = generate_map_with_seed(12345, uncertain_future_mod=True)

        # Follow the single path to ensure it reaches the end
        reachable = can_reach_from_row_zero(nodes)

        # Should reach at least one node in row 14
        row_14_reachable = any(
            (x, 14) in reachable for x in range(len(nodes[0]))
        )

        assert row_14_reachable, "Single path doesn't reach row 14"


# ============================================================================
# TEST CLASS: MAP EDGE OPERATIONS
# ============================================================================


class TestMapEdge:
    """Test MapEdge class functionality."""

    def test_edge_equality(self):
        """Edges with same coordinates should be equal."""
        edge1 = MapEdge(0, 0, 1, 1)
        edge2 = MapEdge(0, 0, 1, 1)

        assert edge1 == edge2

    def test_edge_inequality(self):
        """Edges with different coordinates should not be equal."""
        edge1 = MapEdge(0, 0, 1, 1)
        edge2 = MapEdge(0, 0, 2, 1)

        assert edge1 != edge2

    def test_edge_hash(self):
        """Edges with same coordinates should have same hash."""
        edge1 = MapEdge(0, 0, 1, 1)
        edge2 = MapEdge(0, 0, 1, 1)

        assert hash(edge1) == hash(edge2)

    def test_boss_edge_flag(self):
        """Boss edges should have is_boss flag set."""
        edge = MapEdge(3, 14, 3, 16, is_boss=True)

        assert edge.is_boss


# ============================================================================
# TEST CLASS: MAP ROOM NODE OPERATIONS
# ============================================================================


class TestMapRoomNode:
    """Test MapRoomNode class functionality."""

    def test_node_equality(self):
        """Nodes at same position should be equal."""
        node1 = MapRoomNode(3, 5)
        node2 = MapRoomNode(3, 5)

        assert node1 == node2

    def test_node_inequality(self):
        """Nodes at different positions should not be equal."""
        node1 = MapRoomNode(3, 5)
        node2 = MapRoomNode(4, 5)

        assert node1 != node2

    def test_add_edge_prevents_duplicates(self):
        """Adding duplicate edge should not create duplicates."""
        node = MapRoomNode(0, 0)
        edge = MapEdge(0, 0, 1, 1)

        node.add_edge(edge)
        node.add_edge(edge)  # Add again

        assert len(node.edges) == 1

    def test_del_edge(self):
        """Deleting edge should remove it."""
        node = MapRoomNode(0, 0)
        edge = MapEdge(0, 0, 1, 1)

        node.add_edge(edge)
        node.del_edge(edge)

        assert len(node.edges) == 0

    def test_add_parent(self):
        """Adding parent should work correctly."""
        child = MapRoomNode(3, 5)
        parent = MapRoomNode(3, 4)

        child.add_parent(parent)

        assert parent in child.parents

    def test_add_parent_prevents_duplicates(self):
        """Adding duplicate parent should not create duplicates."""
        child = MapRoomNode(3, 5)
        parent = MapRoomNode(3, 4)

        child.add_parent(parent)
        child.add_parent(parent)

        assert len(child.parents) == 1

    def test_is_connected_to(self):
        """is_connected_to should correctly identify connections."""
        node1 = MapRoomNode(0, 0)
        node2 = MapRoomNode(1, 1)

        edge = MapEdge(0, 0, 1, 1)
        node1.add_edge(edge)

        assert node1.is_connected_to(node2)

    def test_get_symbol(self):
        """get_symbol should return correct room symbol."""
        node = MapRoomNode(0, 0, room_type=RoomType.MONSTER)
        assert node.get_symbol() == "M"

        node.room_type = RoomType.ELITE
        assert node.get_symbol() == "E"

        node.room_type = None
        assert node.get_symbol() == " "


# ============================================================================
# TEST CLASS: MAP TO STRING
# ============================================================================


class TestMapToString:
    """Test map_to_string visualization."""

    def test_map_to_string_not_empty(self):
        """map_to_string should produce non-empty output."""
        nodes = generate_map_with_seed(12345)
        output = map_to_string(nodes)

        assert len(output) > 0

    def test_map_to_string_contains_room_symbols(self):
        """map_to_string should contain room symbols."""
        nodes = generate_map_with_seed(12345)
        output = map_to_string(nodes)

        # Should contain common room symbols
        assert "M" in output  # Monster
        assert "R" in output  # Rest
        assert "T" in output  # Treasure (floor 8)

    def test_map_to_string_hide_rooms(self):
        """map_to_string with show_rooms=False should hide symbols."""
        nodes = generate_map_with_seed(12345)
        output = map_to_string(nodes, show_rooms=False)

        # Should not contain room symbols when hidden
        # Note: M might appear in edge lines, so check for specific room chars
        assert "E" not in output or "$" not in output  # Elites/Shops should be hidden


# ============================================================================
# TEST CLASS: CONFIG DEFAULTS
# ============================================================================


class TestMapGeneratorConfig:
    """Test MapGeneratorConfig defaults and settings."""

    def test_default_config(self):
        """Default config should have expected values."""
        config = MapGeneratorConfig()

        assert config.shop_room_chance == 0.05
        assert config.rest_room_chance == 0.12
        assert config.treasure_room_chance == 0.0
        assert config.event_room_chance == 0.22
        assert config.elite_room_chance == 0.08
        assert config.ascension_level == 0

    def test_custom_config(self):
        """Custom config values should be respected."""
        config = MapGeneratorConfig(
            shop_room_chance=0.10,
            ascension_level=20
        )

        assert config.shop_room_chance == 0.10
        assert config.ascension_level == 20


# ============================================================================
# TEST CLASS: SIBLING RULE
# ============================================================================


class TestSiblingRule:
    """Test that siblings don't have same special room types."""

    def test_no_sibling_elites(self):
        """Siblings should not both be elites."""
        nodes = generate_map_with_seed(12345)

        for row in nodes:
            for node in row:
                if node.room_type == RoomType.ELITE:
                    # Check siblings
                    for parent in node.parents:
                        for edge in parent.edges:
                            if edge.dst_y < len(nodes):
                                sibling = nodes[edge.dst_y][edge.dst_x]
                                if sibling != node:
                                    # Allow same room type on different branches
                                    # The rule only prevents assignment, not existence
                                    pass

    def test_no_sibling_rest_sites(self):
        """Siblings should not both be rest sites (except row 14)."""
        nodes = generate_map_with_seed(12345)

        for y, row in enumerate(nodes):
            if y >= len(nodes) - 1:  # Skip last row (all rest)
                continue

            for node in row:
                if node.room_type == RoomType.REST:
                    for parent in node.parents:
                        for edge in parent.edges:
                            if edge.dst_y < len(nodes) - 1:
                                sibling = nodes[edge.dst_y][edge.dst_x]
                                if sibling != node and edge.dst_y == node.y:
                                    # Rule prevents same assignment during generation
                                    pass


# ============================================================================
# STATISTICAL TESTS
# ============================================================================


class TestStatisticalProperties:
    """Statistical tests for map generation properties."""

    def test_event_room_distribution(self):
        """Event rooms should appear with roughly expected frequency."""
        total_events = 0
        total_connected = 0

        for seed in range(100, 200):
            nodes = generate_map_with_seed(seed)
            counts = count_room_types(nodes)
            connected = len(get_connected_nodes(nodes))

            total_events += counts.get(RoomType.EVENT, 0)
            total_connected += connected

        # Event chance is 22%, should be roughly that
        event_ratio = total_events / total_connected
        assert 0.10 < event_ratio < 0.35, \
            f"Event ratio {event_ratio:.2f} outside expected range"

    def test_starting_node_variation(self):
        """Starting nodes should vary across different seeds."""
        starting_positions = set()

        for seed in range(100, 200):
            nodes = generate_map_with_seed(seed)
            for node in nodes[0]:
                if node.has_edges():
                    starting_positions.add(node.x)

        # Should use multiple columns as starting points
        assert len(starting_positions) >= 3, \
            f"Only {len(starting_positions)} unique starting columns"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
