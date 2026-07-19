//! Map generation — port of Java's MapGenerator + RoomTypeAssigner.
//!
//! Generates a 15-row x 7-column dungeon map with paths and room types.
//! Floor 0 = first row (always monster), floor 8 = treasure, floor 14 = rest.
//! Terminal nodes point at Java's off-map boss destination `(3, 16)`.

use serde::{Deserialize, Serialize};

const STANDARD_BOSS_X: usize = 3;
const STANDARD_BOSS_Y: usize = 16;

// ---------------------------------------------------------------------------
// Room types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    Monster,
    Elite,
    Rest,
    Shop,
    Event,
    Treasure,
    Boss,
    None,
}

impl RoomType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomType::Monster => "monster",
            RoomType::Elite => "elite",
            RoomType::Rest => "rest",
            RoomType::Shop => "shop",
            RoomType::Event => "event",
            RoomType::Treasure => "treasure",
            RoomType::Boss => "boss",
            RoomType::None => "none",
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            RoomType::Monster => 'M',
            RoomType::Elite => 'E',
            RoomType::Rest => 'R',
            RoomType::Shop => '$',
            RoomType::Event => '?',
            RoomType::Treasure => 'T',
            RoomType::Boss => 'B',
            RoomType::None => ' ',
        }
    }
}

// ---------------------------------------------------------------------------
// Map node and edge
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapNode {
    pub x: usize,
    pub y: usize,
    pub room_type: RoomType,
    pub has_edges: bool,
    /// Edges go to (x, y) on the next row
    pub edges: Vec<(usize, usize)>,
    /// Parent nodes (x, y) from previous row
    pub parents: Vec<(usize, usize)>,
    /// Whether this node has the emerald key (elite only)
    pub has_emerald_key: bool,
}

impl MapNode {
    fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            room_type: RoomType::None,
            has_edges: false,
            edges: Vec::new(),
            parents: Vec::new(),
            has_emerald_key: false,
        }
    }

    fn add_edge(&mut self, dst_x: usize, dst_y: usize) {
        // Avoid duplicate edges
        if !self.edges.contains(&(dst_x, dst_y)) {
            self.edges.push((dst_x, dst_y));
            self.edges.sort();
            self.has_edges = true;
        }
    }
}

// ---------------------------------------------------------------------------
// DungeonMap — the full map for one act
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonMap {
    /// rows[y][x] = MapNode. y=0 is first floor, y=14 is last floor before boss.
    pub rows: Vec<Vec<MapNode>>,
    pub height: usize,
    pub width: usize,
}

impl DungeonMap {
    /// Get a node reference.
    pub fn get(&self, x: usize, y: usize) -> &MapNode {
        &self.rows[y][x]
    }

    /// Get a mutable node reference.
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut MapNode {
        &mut self.rows[y][x]
    }

    /// Get reachable next nodes from a position.
    pub fn get_next_nodes(&self, x: usize, y: usize) -> Vec<&MapNode> {
        let node = &self.rows[y][x];
        node.edges
            .iter()
            .filter_map(|&(ex, ey)| self.rows.get(ey).and_then(|row| row.get(ex)))
            .collect()
    }

    /// Get starting nodes (row 0 nodes that have edges).
    pub fn get_start_nodes(&self) -> Vec<&MapNode> {
        self.rows[0].iter().filter(|n| n.has_edges).collect()
    }

    /// Get all connected nodes at a given floor.
    pub fn get_nodes_at_floor(&self, floor: usize) -> Vec<&MapNode> {
        if floor >= self.height {
            return Vec::new();
        }
        self.rows[floor]
            .iter()
            .filter(|n| n.has_edges || !n.parents.is_empty())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Map generation (port of Java MapGenerator)
// ---------------------------------------------------------------------------

/// Generate a dungeon map for one act.
///
/// Standard parameters: height=15, width=7, path_density=6.
/// Room type distribution matches Exordium at A20:
///   shop=5%, rest=12%, treasure=0%, elite=8%*1.6 (A1+), event=22%
pub fn generate_map(seed: u64, ascension: i32) -> DungeonMap {
    generate_map_with_rng(seed, ascension).0
}

/// Generate a map and return the consumed map RNG state for trace accounting.
pub(crate) fn generate_map_with_rng(
    seed: u64,
    ascension: i32,
) -> (DungeonMap, crate::seed::StsRandom) {
    generate_map_with_rng_for_run(seed, ascension, true)
}

pub(crate) fn generate_map_with_rng_for_run(
    seed: u64,
    ascension: i32,
    place_emerald_elite: bool,
) -> (DungeonMap, crate::seed::StsRandom) {
    let mut rng = MapRng::new(seed);

    let height = 15;
    let width = 7;
    let path_density = 6;

    // Step 1: Create empty nodes
    let mut map = DungeonMap {
        rows: (0..height)
            .map(|y| (0..width).map(|x| MapNode::new(x, y)).collect())
            .collect(),
        height,
        width,
    };

    // Step 2: Create paths
    create_paths(&mut map, path_density, &mut rng);

    // Step 3: Filter redundant edges from row 0
    filter_redundant_row0(&mut map);

    // Step 4: Assign room types
    assign_room_types(&mut map, ascension, &mut rng);

    // Step 5: The standard run starts with Act 4 available and no Emerald
    // Key, so AbstractDungeon.setEmeraldElite consumes one counted mapRng
    // draw and marks exactly one elite node.
    if place_emerald_elite {
        set_emerald_elite(&mut map, &mut rng);
    }

    (map, rng.into_inner())
}

/// Simple RNG wrapper matching Java's `Random.random(range)` behavior.
struct MapRng {
    rng: crate::seed::StsRandom,
}

impl MapRng {
    fn new(seed: u64) -> Self {
        Self {
            rng: crate::seed::StsRandom::new(seed),
        }
    }

    /// Returns a random int in [min, max] inclusive (matches Java randRange).
    fn rand_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        // MapGenerator.randRange calls Random.random(max - min) + min, so
        // path-generation draws advance the public mapRng counter.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/map/MapGenerator.java
        self.rng.random_int(max - min) + min
    }

    /// Shuffle a slice in place.
    fn shuffle_room_assignments<T>(&mut self, slice: &mut [T]) {
        // RoomTypeAssigner passes the wrapped RandomXS128 directly to
        // Collections.shuffle, bypassing the public wrapper counter.
        // Java: RoomTypeAssigner.java:127-136.
        self.rng.shuffle_with_inner(slice);
    }

    fn into_inner(self) -> crate::seed::StsRandom {
        self.rng
    }
}

fn create_paths(map: &mut DungeonMap, path_density: usize, rng: &mut MapRng) {
    let row_size = map.width as i32 - 1;
    let mut first_start: i32 = -1;

    for i in 0..path_density {
        let mut start = rng.rand_range(0, row_size);
        if i == 0 {
            first_start = start;
        }
        // Second path must start at a different node
        while i == 1 && start == first_start {
            start = rng.rand_range(0, row_size);
        }
        create_single_path(map, start as usize, rng);
    }
}

fn create_single_path(map: &mut DungeonMap, start_x: usize, rng: &mut MapRng) {
    let mut current_x = start_x;

    for y in 0..(map.height - 1) {
        let row_end = map.width as i32 - 1;
        let cx = current_x as i32;

        // Determine valid range for next node
        let (min_off, max_off) = if cx == 0 {
            (0, 1)
        } else if cx == row_end {
            (-1, 0)
        } else {
            (-1, 1)
        };

        let mut next_x = (cx + rng.rand_range(min_off, max_off)) as usize;
        let next_y = y + 1;

        // Java rerolls paths that would rejoin a sibling branch fewer than
        // three rows after their common ancestor. The original candidate's
        // parent snapshot remains the loop source even if a reroll changes
        // the candidate node.
        // Source: decompiled/java-src/com/megacrit/cardcrawl/map/MapGenerator.java
        let candidate_parents = map.rows[next_y][next_x].parents.clone();
        for parent in candidate_parents {
            if parent == (current_x, y) {
                continue;
            }
            let Some(ancestor) = common_ancestor(map, parent, (current_x, y), 5) else {
                continue;
            };
            if next_y - ancestor.1 >= 3 {
                continue;
            }

            let target_x = next_x as i32;
            let rerolled = if target_x > cx {
                let candidate = cx + rng.rand_range(-1, 0);
                if candidate < 0 {
                    cx
                } else {
                    candidate
                }
            } else if target_x == cx {
                let candidate = cx + rng.rand_range(-1, 1);
                if candidate > row_end {
                    cx - 1
                } else if candidate < 0 {
                    cx + 1
                } else {
                    candidate
                }
            } else {
                let candidate = cx + rng.rand_range(0, 1);
                if candidate > row_end {
                    cx
                } else {
                    candidate
                }
            };
            next_x = rerolled as usize;
        }

        // Anti-crossing: don't cross existing edges from neighbors
        // Check left neighbor
        if current_x > 0 {
            let left = &map.rows[y][current_x - 1];
            if !left.edges.is_empty() {
                let max_edge_x = left.edges.iter().map(|e| e.0).max().unwrap_or(0);
                if max_edge_x > next_x {
                    next_x = max_edge_x;
                }
            }
        }
        // Check right neighbor
        if current_x < map.width - 1 {
            let right = &map.rows[y][current_x + 1];
            if !right.edges.is_empty() {
                let min_edge_x = right
                    .edges
                    .iter()
                    .map(|e| e.0)
                    .min()
                    .unwrap_or(map.width - 1);
                if min_edge_x < next_x {
                    next_x = min_edge_x;
                }
            }
        }

        // Clamp
        next_x = next_x.min(map.width - 1);

        // Add edge and parent
        map.rows[y][current_x].add_edge(next_x, next_y);
        let parent = (current_x, y);
        // MapRoomNode.addParent appends unconditionally even when addEdge
        // rejects a duplicate edge; common-ancestor traversal observes that
        // exact parent list.
        map.rows[next_y][next_x].parents.push(parent);

        current_x = next_x;
    }

    // MapGenerator adds a terminal edge from every row-14 endpoint to the
    // off-map boss destination. The destination is intentionally not a node
    // in the 15-row dungeon map.
    // Java: MapGenerator.java::_createPaths.
    map.rows[map.height - 1][current_x].add_edge(STANDARD_BOSS_X, STANDARD_BOSS_Y);

    // Mark the starting node as connected
    map.rows[0][start_x].has_edges = true;
}

fn common_ancestor(
    map: &DungeonMap,
    node1: (usize, usize),
    node2: (usize, usize),
    max_depth: usize,
) -> Option<(usize, usize)> {
    debug_assert_eq!(node1.1, node2.1);
    debug_assert_ne!(node1, node2);

    // Preserve MapGenerator's shipped comparison exactly: it compares the
    // first node's x against the second node's y when choosing left/right.
    let (mut left, mut right) = if node1.0 < node2.1 {
        (node1, node2)
    } else {
        (node2, node1)
    };

    for _ in 0..=max_depth {
        let left_parents = &map.rows[left.1][left.0].parents;
        let right_parents = &map.rows[right.1][right.0].parents;
        if left_parents.is_empty() || right_parents.is_empty() {
            return None;
        }
        left = *left_parents
            .iter()
            .max_by_key(|parent| parent.0)
            .expect("non-empty left parent list");
        right = *right_parents
            .iter()
            .min_by_key(|parent| parent.0)
            .expect("non-empty right parent list");
        if left == right {
            return Some(left);
        }
    }
    None
}

fn filter_redundant_row0(map: &mut DungeonMap) {
    // Remove duplicate destination edges from row 0 (keep first occurrence)
    let mut seen_dsts: Vec<(usize, usize)> = Vec::new();
    for x in 0..map.width {
        let node = &mut map.rows[0][x];
        let mut keep = Vec::new();
        for &edge in &node.edges {
            if !seen_dsts.contains(&edge) {
                seen_dsts.push(edge);
                keep.push(edge);
            }
        }
        node.edges = keep;
        node.has_edges = !node.edges.is_empty();
    }
}

// ---------------------------------------------------------------------------
// Room type assignment (port of Java RoomTypeAssigner)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RoomRequestCounts {
    shop: usize,
    rest: usize,
    elite: usize,
    event: usize,
}

fn generation_room_count(map: &DungeonMap) -> usize {
    // AbstractDungeon.generateMap counts outgoing-edge nodes before assigning
    // the three fixed rows, but excludes row 13 from the ratio base.
    // Java: AbstractDungeon.java::generateMap.
    map.rows
        .iter()
        .flat_map(|row| row.iter())
        .filter(|node| node.has_edges && node.y != map.height - 2)
        .count()
}

fn requested_room_counts(available_room_count: usize, ascension: i32) -> RoomRequestCounts {
    let shop = (available_room_count as f32 * 0.05_f32).round() as usize;
    let rest = (available_room_count as f32 * 0.12_f32).round() as usize;
    let elite = if ascension >= 1 {
        (available_room_count as f32 * 0.08_f32 * 1.6_f32).round() as usize
    } else {
        (available_room_count as f32 * 0.08_f32).round() as usize
    };
    let event = (available_room_count as f32 * 0.22_f32).round() as usize;

    RoomRequestCounts {
        shop,
        rest,
        elite,
        event,
    }
}

fn assign_fixed_rows(map: &mut DungeonMap) {
    // AbstractDungeon assigns every node in these rows, including nodes that
    // are not part of a generated path.
    // Java: AbstractDungeon.java::generateMap and
    // RoomTypeAssigner.java::assignRowAsRoomType.
    for node in &mut map.rows[map.height - 1] {
        node.room_type = RoomType::Rest;
    }
    for node in &mut map.rows[0] {
        node.room_type = RoomType::Monster;
    }
    for node in &mut map.rows[8] {
        node.room_type = RoomType::Treasure;
    }
}

fn distributable_room_count(map: &DungeonMap) -> usize {
    map.rows
        .iter()
        .flat_map(|row| row.iter())
        .filter(|node| node.has_edges && node.room_type == RoomType::None)
        .count()
}

fn assign_rooms_to_nodes(map: &mut DungeonMap, room_list: &mut Vec<RoomType>) {
    for y in 0..map.height {
        for x in 0..map.width {
            if !map.rows[y][x].has_edges || map.rows[y][x].room_type != RoomType::None {
                continue;
            }

            // getNextRoomTypeAccordingToRules iterates from list index zero
            // independently for every node. Removing the selected element is
            // the only cursor-like mutation.
            // Java: RoomTypeAssigner.java::getNextRoomTypeAccordingToRules.
            let Some(room_idx) = room_list
                .iter()
                .position(|&candidate| is_valid_room_placement(map, x, y, candidate))
            else {
                continue;
            };
            map.rows[y][x].room_type = room_list.remove(room_idx);
        }
    }
}

fn assign_room_types(map: &mut DungeonMap, ascension: i32, rng: &mut MapRng) {
    let available_room_count = generation_room_count(map);
    let requested = requested_room_counts(available_room_count, ascension);

    // Java fixes these rows only after deriving the requested room counts.
    assign_fixed_rows(map);
    let distributable = distributable_room_count(map);

    // Build shuffled room list
    let mut room_list: Vec<RoomType> = Vec::with_capacity(distributable);
    for _ in 0..requested.shop {
        room_list.push(RoomType::Shop);
    }
    for _ in 0..requested.rest {
        room_list.push(RoomType::Rest);
    }
    for _ in 0..requested.elite {
        room_list.push(RoomType::Elite);
    }
    for _ in 0..requested.event {
        room_list.push(RoomType::Event);
    }

    // distributeRoomsAcrossMap pads against the post-fix connected-node
    // count, not the earlier ratio base.
    while room_list.len() < distributable {
        room_list.push(RoomType::Monster);
    }

    rng.shuffle_room_assignments(&mut room_list);
    assign_rooms_to_nodes(map, &mut room_list);

    // Last minute check: any connected node without a room gets Monster
    for y in 0..map.height {
        for x in 0..map.width {
            let node = &map.rows[y][x];
            if node.has_edges && node.room_type == RoomType::None {
                map.rows[y][x].room_type = RoomType::Monster;
            }
        }
    }
}

fn is_valid_room_placement(map: &DungeonMap, x: usize, y: usize, room: RoomType) -> bool {
    // Rule: no elites or rests on rows 0-4
    if y <= 4 && (room == RoomType::Elite || room == RoomType::Rest) {
        return false;
    }

    // Rule: no rests on row 13+ (row 14 is already assigned)
    if y >= 13 && room == RoomType::Rest {
        return false;
    }

    // Rule: parent can't be same room type for special rooms
    let restricted = matches!(
        room,
        RoomType::Rest | RoomType::Treasure | RoomType::Shop | RoomType::Elite
    );
    if restricted {
        for &(px, py) in &map.rows[y][x].parents {
            if map.rows[py][px].room_type == room {
                return false;
            }
        }
    }

    // Rule: siblings can't be same type for certain rooms
    let sibling_restricted = matches!(
        room,
        RoomType::Rest | RoomType::Monster | RoomType::Event | RoomType::Elite | RoomType::Shop
    );
    if sibling_restricted {
        for &(px, py) in &map.rows[y][x].parents {
            for &(sx, sy) in &map.rows[py][px].edges {
                if sx == x && sy == y {
                    continue;
                }
                if sy < map.rows.len() && sx < map.rows[0].len() {
                    if map.rows[sy][sx].room_type == room {
                        return false;
                    }
                }
            }
        }
    }

    true
}

fn set_emerald_elite(map: &mut DungeonMap, rng: &mut MapRng) {
    let elite_nodes: Vec<(usize, usize)> = map
        .rows
        .iter()
        .flat_map(|row| row.iter())
        .filter(|node| node.room_type == RoomType::Elite)
        .map(|node| (node.x, node.y))
        .collect();
    if elite_nodes.is_empty() {
        return;
    }
    let chosen = rng.rand_range(0, elite_nodes.len() as i32 - 1) as usize;
    let (x, y) = elite_nodes[chosen];
    map.rows[y][x].has_emerald_key = true;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_map_basic() {
        let map = generate_map(42, 20);
        assert_eq!(map.height, 15);
        assert_eq!(map.width, 7);

        // Row 0 should have at least 2 connected nodes (path_density=6)
        let starts = map.get_start_nodes();
        assert!(
            starts.len() >= 2,
            "Expected at least 2 start nodes, got {}",
            starts.len()
        );

        // All start nodes should be monsters
        for node in &starts {
            assert_eq!(node.room_type, RoomType::Monster);
        }

        // Row 8 connected nodes should be treasure
        for x in 0..7 {
            let node = &map.rows[8][x];
            if node.has_edges || !node.parents.is_empty() {
                assert_eq!(node.room_type, RoomType::Treasure);
            }
        }

        // Row 14 connected nodes should be rest
        for x in 0..7 {
            let node = &map.rows[14][x];
            if node.has_edges || !node.parents.is_empty() {
                assert_eq!(node.room_type, RoomType::Rest);
            }
        }
    }

    #[test]
    fn test_map_has_valid_paths() {
        let map = generate_map(123, 20);

        // Every connected node (except row 14) should have at least one edge
        for y in 0..14 {
            for x in 0..7 {
                let node = &map.rows[y][x];
                if !node.parents.is_empty() || (y == 0 && node.has_edges) {
                    assert!(
                        node.has_edges,
                        "Node ({},{}) is connected but has no edges",
                        x, y
                    );
                }
            }
        }
    }

    #[test]
    fn test_no_elites_in_early_floors() {
        let map = generate_map(42, 20);
        for y in 0..=4 {
            for x in 0..7 {
                assert_ne!(
                    map.rows[y][x].room_type,
                    RoomType::Elite,
                    "Found elite on floor {}, should not be possible",
                    y
                );
            }
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let map1 = generate_map(42, 20);
        let map2 = generate_map(42, 20);

        for y in 0..15 {
            for x in 0..7 {
                assert_eq!(map1.rows[y][x].room_type, map2.rows[y][x].room_type);
                assert_eq!(map1.rows[y][x].edges, map2.rows[y][x].edges);
            }
        }
    }

    #[test]
    fn test_different_seeds_different_maps() {
        let map1 = generate_map(42, 20);
        let map2 = generate_map(99, 20);

        // At least some room types should differ
        let mut differences = 0;
        for y in 0..15 {
            for x in 0..7 {
                if map1.rows[y][x].room_type != map2.rows[y][x].room_type {
                    differences += 1;
                }
            }
        }
        assert!(
            differences > 0,
            "Different seeds should produce different maps"
        );
    }

    #[test]
    fn smoke_seed_room_request_counts_match_java() {
        // AbstractDungeon derives ratios from every outgoing-edge node except
        // row 13 before assigning the fixed rows. For this known map seed,
        // Java reports N=57 and requests 3/7/5/13 special rooms at A0.
        // Sources:
        // - decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        // - decompiled/java-src/com/megacrit/cardcrawl/map/RoomTypeAssigner.java
        let map = generate_map(57_554_006_467, 0);
        let available = generation_room_count(&map);

        assert_eq!(available, 57);
        assert_eq!(
            requested_room_counts(available, 0),
            RoomRequestCounts {
                shop: 3,
                rest: 7,
                elite: 5,
                event: 13,
            }
        );
    }

    #[test]
    fn room_assignment_restarts_scan_at_head() {
        // The first node cannot take Rest and therefore removes Event from
        // index one. Java restarts the next node at index zero, allowing it to
        // take the remaining Rest rather than falling back to Monster.
        // Java: RoomTypeAssigner.java::getNextRoomTypeAccordingToRules.
        let height = 6;
        let width = 1;
        let mut map = DungeonMap {
            rows: (0..height)
                .map(|y| (0..width).map(|x| MapNode::new(x, y)).collect())
                .collect(),
            height,
            width,
        };
        map.rows[1][0].add_edge(0, 2);
        map.rows[5][0].add_edge(STANDARD_BOSS_X, STANDARD_BOSS_Y);
        let mut room_list = vec![RoomType::Rest, RoomType::Event];

        assign_rooms_to_nodes(&mut map, &mut room_list);

        assert_eq!(map.rows[1][0].room_type, RoomType::Event);
        assert_eq!(map.rows[5][0].room_type, RoomType::Rest);
        assert!(room_list.is_empty());
    }

    #[test]
    fn standard_map_terminal_edges_match_java() {
        // MapGenerator terminates every generated path with an edge from row
        // 14 to the off-map boss destination `(3, 16)`.
        // Java: MapGenerator.java::_createPaths.
        let map = generate_map(57_554_006_467, 0);
        let terminal_nodes: Vec<&MapNode> = map.rows[14]
            .iter()
            .filter(|node| !node.parents.is_empty())
            .collect();

        assert!(!terminal_nodes.is_empty());
        for node in terminal_nodes {
            assert_eq!(node.edges, vec![(STANDARD_BOSS_X, STANDARD_BOSS_Y)]);
            assert!(map.get_next_nodes(node.x, node.y).is_empty());
        }
    }

    #[test]
    fn map_path_rerolls_match_java_counter_for_smoke_seed() {
        // The shipped MapGenerator consumes 93 calls for this seed after
        // three short common-ancestor rerolls. AbstractDungeon then consumes
        // one more call while choosing the Emerald elite.
        // Sources:
        // - decompiled/java-src/com/megacrit/cardcrawl/map/MapGenerator.java
        // - decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
        let (map, rng) = generate_map_with_rng(57_554_006_466_u64.wrapping_add(1), 0);
        assert_eq!(rng.counter, 94);
        assert_eq!(
            map.rows
                .iter()
                .flat_map(|row| row.iter())
                .filter(|node| node.has_emerald_key)
                .count(),
            1
        );
    }
}
