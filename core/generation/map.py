"""
Map Generation Algorithm - Exact replication of game's MapGenerator.

Generates dungeon maps matching Slay the Spire's algorithm exactly.
Uses the same RNG patterns and room distribution rules.

Reference: decompiled/java-src/com/megacrit/cardcrawl/map/MapGenerator.java
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Set, Tuple
import random as py_random

from ..state.rng import Random


class RoomType(Enum):
    """Room types in the dungeon."""
    MONSTER = "M"
    ELITE = "E"
    REST = "R"
    SHOP = "$"
    EVENT = "?"
    TREASURE = "T"
    BOSS = "B"
    # Act 4 specific
    TRUE_VICTORY = "V"


@dataclass
class MapEdge:
    """Edge connecting two map nodes."""
    src_x: int
    src_y: int
    dst_x: int
    dst_y: int
    is_boss: bool = False

    def __hash__(self):
        return hash((self.src_x, self.src_y, self.dst_x, self.dst_y))

    def __eq__(self, other):
        if not isinstance(other, MapEdge):
            return False
        return (self.src_x == other.src_x and self.src_y == other.src_y and
                self.dst_x == other.dst_x and self.dst_y == other.dst_y)


@dataclass
class MapRoomNode:
    """A node on the dungeon map."""
    x: int
    y: int
    room_type: Optional[RoomType] = None
    edges: List[MapEdge] = field(default_factory=list)
    parents: List['MapRoomNode'] = field(default_factory=list)
    has_emerald_key: bool = False

    # Visual offsets (not used for logic, but tracked for accuracy)
    offset_x: float = 0.0
    offset_y: float = 0.0

    def has_edges(self) -> bool:
        """Check if node has any outgoing edges."""
        return len(self.edges) > 0

    def add_edge(self, edge: MapEdge):
        """Add an edge if not duplicate."""
        for existing in self.edges:
            if existing == edge:
                return
        self.edges.append(edge)
        self.edges.sort(key=lambda e: (e.dst_x, e.dst_y))

    def del_edge(self, edge: MapEdge):
        """Remove an edge."""
        if edge in self.edges:
            self.edges.remove(edge)

    def add_parent(self, parent: 'MapRoomNode'):
        """Add a parent node."""
        if parent not in self.parents:
            self.parents.append(parent)

    def is_connected_to(self, node: 'MapRoomNode') -> bool:
        """Check if this node has an edge to another node."""
        for edge in self.edges:
            if edge.dst_x == node.x and edge.dst_y == node.y:
                return True
        return False

    def get_symbol(self) -> str:
        """Get map symbol for this room."""
        if self.room_type is None:
            return " "
        return self.room_type.value

    def __hash__(self):
        return hash((self.x, self.y))

    def __eq__(self, other):
        if not isinstance(other, MapRoomNode):
            return False
        return self.x == other.x and self.y == other.y


# Constants matching the game
MAP_HEIGHT = 15
MAP_WIDTH = 7
MAP_PATH_DENSITY = 6

# Room placement rules
MIN_ANCESTOR_GAP = 3
MAX_ANCESTOR_GAP = 5

# Ascension thresholds
ELITE_ASCENSION_THRESHOLD = 1


@dataclass
class MapGeneratorConfig:
    """Configuration for map generation."""
    # Base probabilities
    shop_room_chance: float = 0.05
    rest_room_chance: float = 0.12
    treasure_room_chance: float = 0.0  # Fixed at floor 8
    event_room_chance: float = 0.22
    elite_room_chance: float = 0.08

    # Modifiers
    ascension_level: int = 0
    elite_swarm_mod: bool = False
    uncertain_future_mod: bool = False
    mimic_infestation: bool = False  # Endless mode blight

    # Act 4 (Final Act) options
    is_final_act_available: bool = False
    has_emerald_key: bool = False


class MapGenerator:
    """
    Generates dungeon maps matching the game's algorithm.

    Usage:
        rng = Random(seed + act_num * act_offset)
        generator = MapGenerator(rng, config)
        dungeon_map = generator.generate()
    """

    def __init__(self, rng: Random, config: Optional[MapGeneratorConfig] = None):
        """
        Initialize map generator.

        Args:
            rng: Random instance seeded appropriately for the act
            config: Generation configuration
        """
        self.rng = rng
        self.config = config or MapGeneratorConfig()

    def generate(self) -> List[List[MapRoomNode]]:
        """
        Generate a complete dungeon map.

        Returns:
            2D list of MapRoomNodes [row][column]
        """
        # Create node grid
        nodes = self._create_nodes(MAP_HEIGHT, MAP_WIDTH)

        # Create paths through the dungeon
        path_density = 1 if self.config.uncertain_future_mod else MAP_PATH_DENSITY
        nodes = self._create_paths(nodes, path_density)

        # Remove redundant edges from first row
        nodes = self._filter_redundant_edges(nodes)

        # Assign room types
        nodes = self._assign_room_types(nodes)

        # Place emerald key elite if applicable
        if self.config.is_final_act_available and not self.config.has_emerald_key:
            self._set_emerald_elite(nodes)

        return nodes

    def _create_nodes(self, height: int, width: int) -> List[List[MapRoomNode]]:
        """Create the base node grid.

        NOTE: In the actual game, node offset jitter uses MathUtils.random()
        (libGDX's global static Random), NOT mapRng. This means jitter is
        non-deterministic from the seed. For accurate map generation matching
        the game's room layout and connections, we skip jitter RNG to avoid
        consuming mapRng calls that would desync the shuffle.

        Visual offsets are set to 0 for deterministic map generation.
        """
        nodes = []
        for y in range(height):
            row = []
            for x in range(width):
                node = MapRoomNode(x=x, y=y)
                # Set offsets to 0 - actual game uses non-deterministic MathUtils.random()
                node.offset_x = 0
                node.offset_y = 0
                row.append(node)
            nodes.append(row)
        return nodes

    def _rand_range(self, min_val: int, max_val: int) -> int:
        """Random int in [min, max] inclusive."""
        return self.rng.random(max_val - min_val) + min_val

    def _create_paths(self, nodes: List[List[MapRoomNode]], path_density: int) -> List[List[MapRoomNode]]:
        """Generate paths through the dungeon."""
        row_size = len(nodes[0]) - 1
        first_starting_node = -1

        for i in range(path_density):
            starting_node = self._rand_range(0, row_size)

            if i == 0:
                first_starting_node = starting_node
            # Second path must start from different node
            while starting_node == first_starting_node and i == 1:
                starting_node = self._rand_range(0, row_size)

            # Create initial edge and propagate path
            initial_edge = MapEdge(
                src_x=starting_node, src_y=-1,
                dst_x=starting_node, dst_y=0
            )
            self._create_path(nodes, initial_edge)

        return nodes

    def _create_path(self, nodes: List[List[MapRoomNode]], edge: MapEdge):
        """Recursively create a path through the dungeon."""
        current_node = nodes[edge.dst_y][edge.dst_x]

        # If at last row, add boss edge
        if edge.dst_y + 1 >= len(nodes):
            boss_edge = MapEdge(
                src_x=edge.dst_x, src_y=edge.dst_y,
                dst_x=3, dst_y=edge.dst_y + 2,  # Boss at center
                is_boss=True
            )
            current_node.add_edge(boss_edge)
            return

        row_width = len(nodes[edge.dst_y])
        row_end = row_width - 1

        # Determine valid x range for next node
        if edge.dst_x == 0:
            min_x, max_x = 0, 1
        elif edge.dst_x == row_end:
            min_x, max_x = -1, 0
        else:
            min_x, max_x = -1, 1

        new_edge_x = edge.dst_x + self._rand_range(min_x, max_x)
        new_edge_y = edge.dst_y + 1

        target_candidate = nodes[new_edge_y][new_edge_x]

        # Check for common ancestors (prevent paths merging too quickly)
        parents = target_candidate.parents
        if parents:
            for parent in parents:
                if parent == current_node:
                    continue
                ancestor = self._get_common_ancestor(parent, current_node, MAX_ANCESTOR_GAP)
                if ancestor is not None:
                    ancestor_gap = new_edge_y - ancestor.y
                    if ancestor_gap < MIN_ANCESTOR_GAP:
                        # Redirect the path
                        if target_candidate.x > current_node.x:
                            new_edge_x = edge.dst_x + self._rand_range(-1, 0)
                            if new_edge_x < 0:
                                new_edge_x = edge.dst_x
                        elif target_candidate.x == current_node.x:
                            new_edge_x = edge.dst_x + self._rand_range(-1, 1)
                            if new_edge_x > row_end:
                                new_edge_x = edge.dst_x - 1
                            elif new_edge_x < 0:
                                new_edge_x = edge.dst_x + 1
                        else:
                            new_edge_x = edge.dst_x + self._rand_range(0, 1)
                            if new_edge_x > row_end:
                                new_edge_x = edge.dst_x

                        target_candidate = nodes[new_edge_y][new_edge_x]
                    elif ancestor_gap < MAX_ANCESTOR_GAP:
                        continue  # Skip this parent, gap not large enough

        # Prevent path crossing with left neighbor
        if edge.dst_x != 0:
            left_node = nodes[edge.dst_y][edge.dst_x - 1]
            if left_node.has_edges():
                max_edge = max(left_node.edges, key=lambda e: e.dst_x)
                if max_edge.dst_x > new_edge_x:
                    new_edge_x = max_edge.dst_x

        # Prevent path crossing with right neighbor
        if edge.dst_x < row_end:
            right_node = nodes[edge.dst_y][edge.dst_x + 1]
            if right_node.has_edges():
                min_edge = min(right_node.edges, key=lambda e: e.dst_x)
                if min_edge.dst_x < new_edge_x:
                    new_edge_x = min_edge.dst_x

        target_candidate = nodes[new_edge_y][new_edge_x]

        # Create the edge
        new_edge = MapEdge(
            src_x=edge.dst_x, src_y=edge.dst_y,
            dst_x=new_edge_x, dst_y=new_edge_y
        )
        current_node.add_edge(new_edge)
        target_candidate.add_parent(current_node)

        # Recurse
        self._create_path(nodes, new_edge)

    def _get_common_ancestor(
        self, node1: MapRoomNode, node2: MapRoomNode, max_depth: int
    ) -> Optional[MapRoomNode]:
        """Find common ancestor of two nodes within max_depth."""
        if node1.y != node2.y or node1 == node2:
            return None

        # NOTE: Java compares node1.x < node2.y (x to y) - this appears to be
        # a bug in the original game, but we match it exactly for parity
        if node1.x < node2.y:
            l_node, r_node = node1, node2
        else:
            l_node, r_node = node2, node1

        for current_y in range(node1.y, max(node1.y - max_depth - 1, -1), -1):
            if not l_node.parents or not r_node.parents:
                return None

            # Get rightmost parent of left node
            l_node = max(l_node.parents, key=lambda n: n.x)
            # Get leftmost parent of right node
            r_node = min(r_node.parents, key=lambda n: n.x)

            if l_node == r_node:
                return l_node

        return None

    def _filter_redundant_edges(self, nodes: List[List[MapRoomNode]]) -> List[List[MapRoomNode]]:
        """Remove duplicate edges to same destination from row 0."""
        existing_edges: Set[Tuple[int, int]] = set()
        delete_list = []

        for node in nodes[0]:
            if not node.has_edges():
                continue

            for edge in node.edges:
                dest = (edge.dst_x, edge.dst_y)
                if dest in existing_edges:
                    delete_list.append((node, edge))
                else:
                    existing_edges.add(dest)

            for node_to_del, edge_to_del in delete_list:
                if node_to_del == node:
                    node.del_edge(edge_to_del)
            delete_list.clear()

        return nodes

    def _assign_room_types(self, nodes: List[List[MapRoomNode]]) -> List[List[MapRoomNode]]:
        """Assign room types to all connected nodes."""
        # Count connected nodes (excluding row 14 which goes to boss)
        available_count = sum(
            1 for row_idx, row in enumerate(nodes)
            for node in row
            if node.has_edges() and row_idx != len(nodes) - 2
        )

        # Generate room list
        room_list = self._generate_room_list(available_count)

        # Shuffle room list using RNG
        self._shuffle_with_rng(room_list)

        # Fixed room assignments
        # Row 14 (last row): Rest Sites
        for node in nodes[len(nodes) - 1]:
            if node.room_type is None:
                node.room_type = RoomType.REST

        # Row 0: Monster Rooms
        for node in nodes[0]:
            if node.room_type is None:
                node.room_type = RoomType.MONSTER

        # Row 8: Treasure Rooms (or Elite if mimic infestation)
        treasure_type = RoomType.ELITE if self.config.mimic_infestation else RoomType.TREASURE
        for node in nodes[8]:
            if node.room_type is None:
                node.room_type = treasure_type

        # Distribute remaining rooms
        self._distribute_rooms(nodes, room_list)

        # Fill any unassigned nodes with Monster rooms
        for row in nodes:
            for node in row:
                if node.has_edges() and node.room_type is None:
                    node.room_type = RoomType.MONSTER

        return nodes

    def _generate_room_list(self, available_count: int) -> List[RoomType]:
        """Generate the list of rooms to distribute."""
        room_list = []

        shop_count = round(available_count * self.config.shop_room_chance)
        rest_count = round(available_count * self.config.rest_room_chance)
        treasure_count = round(available_count * self.config.treasure_room_chance)
        event_count = round(available_count * self.config.event_room_chance)

        # Elite count with modifiers
        if self.config.elite_swarm_mod:
            elite_count = round(available_count * self.config.elite_room_chance * 2.5)
        elif self.config.ascension_level >= ELITE_ASCENSION_THRESHOLD:
            elite_count = round(available_count * self.config.elite_room_chance * 1.6)
        else:
            elite_count = round(available_count * self.config.elite_room_chance)

        # Add rooms to list
        room_list.extend([RoomType.SHOP] * shop_count)
        room_list.extend([RoomType.REST] * rest_count)
        room_list.extend([RoomType.ELITE] * elite_count)
        room_list.extend([RoomType.EVENT] * event_count)

        # Fill remainder with monsters
        while len(room_list) < available_count:
            room_list.append(RoomType.MONSTER)

        return room_list

    def _shuffle_with_rng(self, items: list):
        """Shuffle list using game's RNG (matches Collections.shuffle).

        Java's Collections.shuffle implementation:
        for (int i = size; i > 1; i--)
            swap(arr, i-1, rnd.nextInt(i));

        Where nextInt(i) returns [0, i) exclusive.
        """
        for i in range(len(items) - 1, 0, -1):
            j = self.rng._rng.next_int(i + 1)  # [0, i+1) = [0, i] inclusive
            self.rng.counter += 1  # Track RNG counter
            items[i], items[j] = items[j], items[i]

    def _distribute_rooms(self, nodes: List[List[MapRoomNode]], room_list: List[RoomType]):
        """Distribute rooms to nodes following placement rules."""
        for row in nodes:
            for node in row:
                if not node.has_edges() or node.room_type is not None:
                    continue

                room_type = self._get_valid_room_type(nodes, node, room_list)
                if room_type is not None:
                    node.room_type = room_type
                    room_list.remove(room_type)

    def _get_valid_room_type(
        self, nodes: List[List[MapRoomNode]], node: MapRoomNode, room_list: List[RoomType]
    ) -> Optional[RoomType]:
        """Get a valid room type for a node based on placement rules."""
        parents = node.parents
        siblings = self._get_siblings(nodes, parents, node)

        for room_type in room_list:
            # Check row restrictions
            if not self._rule_assignable_to_row(node, room_type):
                continue

            # Check parent/sibling rules
            if node.y != 0:  # First row bypasses these rules
                if self._rule_parent_matches(parents, room_type):
                    continue
                if self._rule_sibling_matches(siblings, room_type):
                    continue

            return room_type

        return None

    def _get_siblings(
        self, nodes: List[List[MapRoomNode]], parents: List[MapRoomNode], node: MapRoomNode
    ) -> List[MapRoomNode]:
        """Get sibling nodes (other children of same parents)."""
        siblings = []
        for parent in parents:
            for edge in parent.edges:
                sibling = nodes[edge.dst_y][edge.dst_x]
                if sibling != node and sibling not in siblings:
                    siblings.append(sibling)
        return siblings

    def _rule_assignable_to_row(self, node: MapRoomNode, room_type: RoomType) -> bool:
        """Check if room type can be placed on this row."""
        # Rest and Elite can't appear in first 5 floors
        if node.y <= 4 and room_type in (RoomType.REST, RoomType.ELITE):
            return False

        # Rest can't appear on floors 13+ (it's forced on 14 anyway)
        if node.y >= 13 and room_type == RoomType.REST:
            return False

        return True

    def _rule_parent_matches(self, parents: List[MapRoomNode], room_type: RoomType) -> bool:
        """Check if parent has same room type (for restricted types)."""
        applicable = {RoomType.REST, RoomType.TREASURE, RoomType.SHOP, RoomType.ELITE}

        if room_type not in applicable:
            return False

        for parent in parents:
            if parent.room_type == room_type:
                return True

        return False

    def _rule_sibling_matches(self, siblings: List[MapRoomNode], room_type: RoomType) -> bool:
        """Check if sibling has same room type (for restricted types)."""
        applicable = {RoomType.REST, RoomType.MONSTER, RoomType.EVENT, RoomType.ELITE, RoomType.SHOP}

        if room_type not in applicable:
            return False

        for sibling in siblings:
            if sibling.room_type == room_type:
                return True

        return False

    def _set_emerald_elite(self, nodes: List[List[MapRoomNode]]):
        """Mark one random elite as the burning elite (emerald key holder)."""
        elite_nodes = [
            node for row in nodes for node in row
            if node.room_type == RoomType.ELITE
        ]

        if elite_nodes:
            chosen = elite_nodes[self.rng.random(len(elite_nodes) - 1)]
            chosen.has_emerald_key = True


def generate_act4_map() -> List[List[MapRoomNode]]:
    """
    Generate the special Act 4 (The Ending) map.

    Act 4 has a fixed linear structure:
    - Floor 0: Rest
    - Floor 1: Shop
    - Floor 2: Elite (Shield and Spear)
    - Floor 3: Boss (The Heart)
    - Floor 4: True Victory

    Returns:
        2D list of MapRoomNodes
    """
    nodes = []

    for y in range(5):
        row = []
        for x in range(7):
            node = MapRoomNode(x=x, y=y)
            row.append(node)
        nodes.append(row)

    # Set up the linear path in column 3
    rest_node = nodes[0][3]
    rest_node.room_type = RoomType.REST

    shop_node = nodes[1][3]
    shop_node.room_type = RoomType.SHOP

    elite_node = nodes[2][3]
    elite_node.room_type = RoomType.ELITE

    boss_node = nodes[3][3]
    boss_node.room_type = RoomType.BOSS

    victory_node = nodes[4][3]
    victory_node.room_type = RoomType.TRUE_VICTORY

    # Connect nodes
    rest_node.add_edge(MapEdge(3, 0, 3, 1))
    shop_node.add_edge(MapEdge(3, 1, 3, 2))
    elite_node.add_edge(MapEdge(3, 2, 3, 3, is_boss=True))
    boss_node.add_edge(MapEdge(3, 3, 3, 4))

    return nodes


def get_map_seed_offset(act_num: int) -> int:
    """
    Get the seed offset for a given act number.

    Args:
        act_num: Act number (1-4)

    Returns:
        Offset to add to base seed for map generation
    """
    offsets = {
        1: 1,    # Exordium: seed + 1
        2: 200,  # The City: seed + 200
        3: 600,  # The Beyond: seed + 600
        4: 1200, # The Ending: seed + 1200
    }
    return offsets.get(act_num, 0)


def map_to_string(nodes: List[List[MapRoomNode]], show_rooms: bool = True) -> str:
    """
    Convert map to ASCII string representation.

    Args:
        nodes: 2D list of map nodes
        show_rooms: Whether to show room symbols

    Returns:
        ASCII string representation of the map
    """
    lines = []
    left_padding = "     "

    for row_num in range(len(nodes) - 1, -1, -1):
        # Draw edges going up
        edge_line = f" {left_padding}"
        for node in nodes[row_num]:
            left = " "
            mid = " "
            right = " "
            for edge in node.edges:
                if edge.dst_x < node.x:
                    left = "\\"
                if edge.dst_x == node.x:
                    mid = "|"
                if edge.dst_x > node.x:
                    right = "/"
            edge_line += f"{left}{mid}{right}"
        lines.append(edge_line)

        # Draw nodes
        row_str = str(row_num).rjust(2) + " " + left_padding[:-2]
        for node in nodes[row_num]:
            if row_num == len(nodes) - 1:
                # Last row - check if any lower node connects here
                has_connection = False
                if row_num > 0:
                    for lower_node in nodes[row_num - 1]:
                        for edge in lower_node.edges:
                            if edge.dst_x == node.x:
                                has_connection = True
                                break
                symbol = node.get_symbol() if (has_connection or node.has_edges()) and show_rooms else " "
            else:
                symbol = node.get_symbol() if node.has_edges() and show_rooms else " "
            row_str += f" {symbol} "
        lines.append(row_str)

    return "\n".join(lines)


# ============ TESTING ============

if __name__ == "__main__":
    from ..state.rng import Random, seed_to_long

    # Test with a known seed
    seed = seed_to_long("ABC123")
    print(f"Seed: {seed}")

    # Generate Act 1 map
    config = MapGeneratorConfig(ascension_level=20)
    map_rng = Random(seed + get_map_seed_offset(1))
    generator = MapGenerator(map_rng, config)

    dungeon = generator.generate()

    print("\n=== Generated Map (Act 1, A20) ===")
    print(map_to_string(dungeon))

    # Count room types
    counts = {}
    for row in dungeon:
        for node in row:
            if node.room_type:
                counts[node.room_type] = counts.get(node.room_type, 0) + 1

    print("\n=== Room Distribution ===")
    for room_type, count in sorted(counts.items(), key=lambda x: x[0].value):
        print(f"  {room_type.value} ({room_type.name}): {count}")

    # Test Act 4 map
    print("\n=== Act 4 Map ===")
    act4_map = generate_act4_map()
    print(map_to_string(act4_map))
