//! Map generation — port of Java's MapGenerator + RoomTypeAssigner.
//!
//! Generates a 15-row x 7-column dungeon map with paths and room types.
//! Floor 0 = first row (always monster), floor 8 = treasure, floor 14 = rest.
//! Boss fight is floor 15 (off-map).

use rand::Rng;
use serde::{Deserialize, Serialize};

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
            .map(|&(ex, ey)| &self.rows[ey][ex])
            .collect()
    }

    /// Get starting nodes (row 0 nodes that have edges).
    pub fn get_start_nodes(&self) -> Vec<&MapNode> {
        self.rows[0]
            .iter()
            .filter(|n| n.has_edges)
            .collect()
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

    map
}

/// Simple RNG wrapper matching Java's Random.random(range) behavior.
struct MapRng {
    rng: rand::rngs::SmallRng,
}

impl MapRng {
    fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    /// Returns a random int in [min, max] inclusive (matches Java randRange).
    fn rand_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng.gen_range(min..=max)
    }

    /// Shuffle a slice in place.
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        // Fisher-Yates shuffle
        let len = slice.len();
        for i in (1..len).rev() {
            let j = self.rng.gen_range(0..=i);
            slice.swap(i, j);
        }
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
                let min_edge_x = right.edges.iter().map(|e| e.0).min().unwrap_or(map.width - 1);
                if min_edge_x < next_x {
                    next_x = min_edge_x;
                }
            }
        }

        // Clamp
        next_x = next_x.min(map.width - 1);

        let next_y = y + 1;

        // Add edge and parent
        map.rows[y][current_x].add_edge(next_x, next_y);
        let parent = (current_x, y);
        if !map.rows[next_y][next_x].parents.contains(&parent) {
            map.rows[next_y][next_x].parents.push(parent);
        }

        current_x = next_x;
    }

    // Mark the starting node as connected
    map.rows[0][start_x].has_edges = true;
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

fn assign_room_types(map: &mut DungeonMap, ascension: i32, rng: &mut MapRng) {
    // Fixed rows:
    // Row 0 = always Monster
    // Row 8 = always Treasure
    // Row 14 (last) = always Rest (before boss)
    for x in 0..map.width {
        if map.rows[0][x].has_edges {
            map.rows[0][x].room_type = RoomType::Monster;
        }
    }
    for x in 0..map.width {
        // Row 8 = treasure for connected nodes
        let has_parent = !map.rows[8][x].parents.is_empty();
        let has_edge = map.rows[8][x].has_edges;
        if has_parent || has_edge {
            map.rows[8][x].room_type = RoomType::Treasure;
        }
    }
    for x in 0..map.width {
        let has_parent = !map.rows[14][x].parents.is_empty();
        let has_edge = map.rows[14][x].has_edges;
        if has_parent || has_edge {
            map.rows[14][x].room_type = RoomType::Rest;
        }
    }

    // Count assignable nodes (connected but not yet assigned, excluding row 14 top)
    let mut assignable = 0;
    for y in 1..map.height {
        for x in 0..map.width {
            let node = &map.rows[y][x];
            if (node.has_edges || !node.parents.is_empty()) && node.room_type == RoomType::None {
                assignable += 1;
            }
        }
    }

    // Generate room list using Exordium ratios
    let shop_chance: f32 = 0.05;
    let rest_chance: f32 = 0.12;
    let event_chance: f32 = 0.22;
    let elite_chance: f32 = 0.08;

    let shop_count = (assignable as f32 * shop_chance).round() as usize;
    let rest_count = (assignable as f32 * rest_chance).round() as usize;
    let elite_count = if ascension >= 1 {
        (assignable as f32 * elite_chance * 1.6).round() as usize
    } else {
        (assignable as f32 * elite_chance).round() as usize
    };
    let event_count = (assignable as f32 * event_chance).round() as usize;
    // Remainder = monsters
    let special_total = shop_count + rest_count + elite_count + event_count;
    let monster_count = if assignable > special_total {
        assignable - special_total
    } else {
        0
    };

    // Build shuffled room list
    let mut room_list: Vec<RoomType> = Vec::with_capacity(assignable);
    for _ in 0..shop_count {
        room_list.push(RoomType::Shop);
    }
    for _ in 0..rest_count {
        room_list.push(RoomType::Rest);
    }
    for _ in 0..elite_count {
        room_list.push(RoomType::Elite);
    }
    for _ in 0..event_count {
        room_list.push(RoomType::Event);
    }
    for _ in 0..monster_count {
        room_list.push(RoomType::Monster);
    }
    // Pad with monsters if needed
    while room_list.len() < assignable {
        room_list.push(RoomType::Monster);
    }

    rng.shuffle(&mut room_list);

    // Assign rooms respecting rules
    let mut room_idx = 0;
    for y in 1..map.height {
        for x in 0..map.width {
            if map.rows[y][x].room_type != RoomType::None {
                continue;
            }
            let connected = map.rows[y][x].has_edges || !map.rows[y][x].parents.is_empty();
            if !connected {
                continue;
            }

            // Find a valid room from the list
            let start_idx = room_idx;
            loop {
                if room_idx >= room_list.len() {
                    // Fallback: monster
                    map.rows[y][x].room_type = RoomType::Monster;
                    break;
                }

                let candidate = room_list[room_idx];
                if is_valid_room_placement(map, x, y, candidate) {
                    map.rows[y][x].room_type = candidate;
                    room_list.remove(room_idx);
                    break;
                }

                room_idx += 1;
                if room_idx >= room_list.len() {
                    room_idx = 0;
                }
                if room_idx == start_idx {
                    // No valid room found, use monster
                    map.rows[y][x].room_type = RoomType::Monster;
                    break;
                }
            }
        }
    }

    // Last minute check: any connected node without a room gets Monster
    for y in 0..map.height {
        for x in 0..map.width {
            let node = &map.rows[y][x];
            if (node.has_edges || !node.parents.is_empty()) && node.room_type == RoomType::None {
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
        assert!(starts.len() >= 2, "Expected at least 2 start nodes, got {}", starts.len());

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
        assert!(differences > 0, "Different seeds should produce different maps");
    }
}
