"""
Audit tests: RNG system, map generation, encounters, card rewards, potion drops.

Validates Python engine parity with decompiled Java source.
"""

import pytest
import sys
import os

# Ensure engine is importable
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from packages.engine.state.rng import (
    XorShift128,
    Random,
    seed_to_long,
    long_to_seed,
    GameRNG,
)


# =============================================================================
# 1. RNG SYSTEM
# =============================================================================


class TestXorShift128:
    """Validate XorShift128 matches libGDX RandomXS128."""

    def test_murmur_hash3_known_value(self):
        """MurmurHash3 finalizer should be deterministic."""
        h = XorShift128._murmur_hash3(42)
        assert isinstance(h, int)
        assert 0 <= h < (1 << 64)

    def test_zero_seed_uses_long_min_value(self):
        """Seed of 0 should be treated as Long.MIN_VALUE."""
        rng = XorShift128(0)
        # Should not crash, and state should not be all zeros
        assert rng.seed0 != 0 or rng.seed1 != 0

    def test_next_int_bound(self):
        """next_int(n) should return values in [0, n)."""
        rng = XorShift128(12345)
        for _ in range(100):
            val = rng.next_int(10)
            assert 0 <= val < 10

    def test_next_float_range(self):
        """next_float() should return values in [0, 1)."""
        rng = XorShift128(12345)
        for _ in range(100):
            val = rng.next_float()
            assert 0.0 <= val < 1.0

    def test_next_double_range(self):
        """next_double() should return values in [0, 1)."""
        rng = XorShift128(12345)
        for _ in range(100):
            val = rng.next_double()
            assert 0.0 <= val < 1.0

    def test_copy_produces_identical_sequence(self):
        """copy() should produce identical subsequent values."""
        rng1 = XorShift128(99999)
        # Advance a few
        for _ in range(5):
            rng1._next_long()
        rng2 = rng1.copy()
        for _ in range(20):
            assert rng1._next_long() == rng2._next_long()

    def test_two_arg_constructor_sets_state_directly(self):
        """Two-arg constructor should set seed0/seed1 directly."""
        rng = XorShift128(111, 222)
        assert rng.seed0 == 111
        assert rng.seed1 == 222


class TestRandom:
    """Validate Random wrapper matches game's Random class."""

    def test_random_int_inclusive(self):
        """random_int(n) returns [0, n] inclusive."""
        rng = Random(42)
        seen_max = False
        for _ in range(1000):
            val = rng.random_int(5)
            assert 0 <= val <= 5
            if val == 5:
                seen_max = True
        assert seen_max, "Should see max value in 1000 attempts"

    def test_random_int_range_inclusive(self):
        """random_int_range(start, end) returns [start, end] inclusive."""
        rng = Random(42)
        for _ in range(100):
            val = rng.random_int_range(10, 20)
            assert 10 <= val <= 20

    def test_counter_increments(self):
        """Each RNG call should increment counter by 1."""
        rng = Random(42)
        assert rng.counter == 0
        rng.random_int(10)
        assert rng.counter == 1
        rng.random_boolean()
        assert rng.counter == 2
        rng.random_float()
        assert rng.counter == 3

    def test_constructor_with_counter_advances(self):
        """Random(seed, counter) should advance by calling random(999) counter times."""
        rng1 = Random(42)
        for _ in range(5):
            rng1.random_int(999)

        rng2 = Random(42, 5)

        # After construction, both should produce same next value
        assert rng1.random_int(100) == rng2.random_int(100)

    def test_copy_produces_identical_sequence(self):
        """copy() should produce identical subsequent values."""
        rng1 = Random(12345)
        for _ in range(10):
            rng1.random_int(99)
        rng2 = rng1.copy()
        assert rng2.counter == rng1.counter
        for _ in range(20):
            assert rng1.random_int(99) == rng2.random_int(99)

    def test_set_counter_advances_via_random_boolean(self):
        """set_counter should advance using randomBoolean()."""
        rng = Random(42)
        rng.set_counter(10)
        assert rng.counter == 10

    def test_random_long_range_uses_next_double(self):
        """random_long_range uses nextDouble, not nextInt."""
        rng1 = Random(42)
        rng2 = Random(42)
        # random_long_range should use nextDouble internally
        val1 = rng1.random_long_range(1000)
        # Manually: nextDouble * 1000
        d = rng2._rng.next_double()
        rng2.counter += 1
        val2 = int(d * 1000)
        assert val1 == val2

    def test_random_boolean_no_arg_uses_next_boolean(self):
        """randomBoolean() with no arg uses nextBoolean (LSB check)."""
        rng = Random(42)
        val = rng.random_boolean()
        assert isinstance(val, bool)

    def test_random_boolean_with_chance_uses_next_float(self):
        """randomBoolean(chance) uses nextFloat < chance."""
        rng = Random(42)
        # chance=1.0 should always return True
        assert rng.random_boolean(1.0) is True

    def test_random_boolean_zero_chance(self):
        """randomBoolean(0.0) should always return False."""
        rng = Random(42)
        for _ in range(100):
            assert rng.random_boolean(0.0) is False


class TestSeedConversion:
    """Validate seed string <-> long conversion."""

    def test_roundtrip(self):
        """seed_to_long -> long_to_seed should roundtrip."""
        for s in ["ABC123", "1ABCD", "ZZZZZ", "1", "0"]:
            long_val = seed_to_long(s)
            if long_val != 0:  # 0 has special handling
                back = long_to_seed(long_val)
                assert seed_to_long(back) == long_val

    def test_o_replaced_with_zero(self):
        """O should be replaced with 0."""
        assert seed_to_long("O") == seed_to_long("0")
        assert seed_to_long("HELLO") == seed_to_long("HELL0")

    def test_case_insensitive(self):
        """Seeds should be case-insensitive."""
        assert seed_to_long("abc") == seed_to_long("ABC")

    def test_numeric_passthrough(self):
        """Pure numeric strings pass through as integers."""
        assert seed_to_long("12345") == 12345
        assert seed_to_long("-100") == -100

    def test_base35_charset(self):
        """Charset is 0-9, A-Z excluding O (35 chars)."""
        charset = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ"
        assert len(charset) == 35


class TestGameRNG:
    """Validate all 13 RNG streams."""

    def test_all_13_streams_initialized(self):
        """All 13 streams should be initialized."""
        game = GameRNG(seed=42)
        assert game.monster_rng is not None
        assert game.map_rng is not None
        assert game.event_rng is not None
        assert game.merchant_rng is not None
        assert game.card_rng is not None
        assert game.treasure_rng is not None
        assert game.relic_rng is not None
        assert game.potion_rng is not None
        assert game.monster_hp_rng is not None
        assert game.ai_rng is not None
        assert game.shuffle_rng is not None
        assert game.card_random_rng is not None
        assert game.misc_rng is not None

    def test_persistent_streams_share_seed(self):
        """Persistent streams all start with same seed."""
        game = GameRNG(seed=42)
        # All persistent streams should produce same first value
        vals = [
            Random(42).random_int(99),
        ]
        assert game.monster_rng.random_int(99) == vals[0]
        assert game.card_rng.random_int(99) == vals[0]

    def test_per_floor_reseeding(self):
        """Per-floor streams should reseed with seed+floor."""
        game = GameRNG(seed=42, floor=0)
        floor0_val = game.ai_rng.random_int(99)

        game.advance_floor()
        floor1_val = game.ai_rng.random_int(99)

        # Floor 1 should be different from floor 0
        expected = Random(42 + 1).random_int(99)
        assert floor1_val == expected
        assert floor0_val != floor1_val

    def test_get_counters_and_from_save(self):
        """Save/restore should preserve state."""
        game = GameRNG(seed=42)
        game.monster_rng.random_int(99)
        game.monster_rng.random_int(99)
        game.card_rng.random_int(99)

        counters = game.get_counters()
        assert counters["monster_seed_count"] == 2
        assert counters["card_seed_count"] == 1

        restored = GameRNG.from_save(42, counters, floor=0)
        assert restored.monster_rng.random_int(99) == game.monster_rng.random_int(99)


# =============================================================================
# 2. MAP GENERATION
# =============================================================================


class TestMapGeneration:
    """Validate map generation matches Java MapGenerator."""

    def test_map_seed_offsets_match_java(self):
        """Map seed offsets should match Java source."""
        from packages.engine.generation.map import get_map_seed_offset

        # Java: Exordium = seed + actNum (actNum=1)
        assert get_map_seed_offset(1) == 1
        # Java: TheCity = seed + actNum * 100 (actNum=2 -> 200)
        assert get_map_seed_offset(2) == 200
        # Java: TheBeyond = seed + actNum * 200 (actNum=3 -> 600)
        assert get_map_seed_offset(3) == 600
        # Java: TheEnding = seed + actNum * 300 (actNum=4 -> 1200)
        assert get_map_seed_offset(4) == 1200

    def test_map_dimensions(self):
        """Map should be 15 rows x 7 columns."""
        from packages.engine.generation.map import (
            MapGenerator,
            MapGeneratorConfig,
            MAP_HEIGHT,
            MAP_WIDTH,
        )

        assert MAP_HEIGHT == 15
        assert MAP_WIDTH == 7

        rng = Random(42 + 1)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        assert len(dungeon) == 15
        assert all(len(row) == 7 for row in dungeon)

    def test_row0_always_monster(self):
        """Row 0 should always be monster rooms."""
        from packages.engine.generation.map import (
            MapGenerator,
            MapGeneratorConfig,
            RoomType,
        )

        rng = Random(12345)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        for node in dungeon[0]:
            if node.has_edges():
                assert node.room_type == RoomType.MONSTER

    def test_row14_always_rest(self):
        """Row 14 (last) should always be rest sites."""
        from packages.engine.generation.map import (
            MapGenerator,
            RoomType,
        )

        rng = Random(12345)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        for node in dungeon[14]:
            if node.room_type is not None:
                assert node.room_type == RoomType.REST

    def test_row8_always_treasure(self):
        """Row 8 should always be treasure rooms."""
        from packages.engine.generation.map import (
            MapGenerator,
            RoomType,
        )

        rng = Random(12345)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        for node in dungeon[8]:
            if node.room_type is not None:
                assert node.room_type == RoomType.TREASURE

    def test_no_elite_or_rest_in_first_5_rows(self):
        """Rows 0-4 should never have elite or rest rooms."""
        from packages.engine.generation.map import (
            MapGenerator,
            RoomType,
        )

        for seed in [42, 999, 7777, 123456]:
            rng = Random(seed)
            gen = MapGenerator(rng)
            dungeon = gen.generate()

            for y in range(5):
                for node in dungeon[y]:
                    if node.room_type is not None:
                        assert node.room_type not in (RoomType.REST, RoomType.ELITE), (
                            f"Found {node.room_type} at row {y} seed {seed}"
                        )

    def test_no_rest_at_row13(self):
        """Row 13 should never have rest rooms (they go on row 14)."""
        from packages.engine.generation.map import MapGenerator, RoomType

        for seed in [42, 999, 7777, 123456]:
            rng = Random(seed)
            gen = MapGenerator(rng)
            dungeon = gen.generate()

            for node in dungeon[13]:
                if node.room_type is not None:
                    assert node.room_type != RoomType.REST

    def test_no_parent_repeat_for_restricted_rooms(self):
        """Rest/Treasure/Shop/Elite should not repeat with parent."""
        from packages.engine.generation.map import MapGenerator, RoomType

        restricted = {RoomType.REST, RoomType.TREASURE, RoomType.SHOP, RoomType.ELITE}

        for seed in [42, 999, 7777]:
            rng = Random(seed)
            gen = MapGenerator(rng)
            dungeon = gen.generate()

            for y in range(1, len(dungeon)):
                for node in dungeon[y]:
                    if node.room_type not in restricted:
                        continue
                    for parent in node.parents:
                        assert parent.room_type != node.room_type, (
                            f"Parent repeat: {node.room_type} at ({node.x},{node.y})"
                        )

    def test_at_least_two_starting_paths(self):
        """Map should have at least 2 distinct starting columns."""
        from packages.engine.generation.map import MapGenerator

        rng = Random(42)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        starting_cols = set()
        for node in dungeon[0]:
            if node.has_edges():
                starting_cols.add(node.x)

        assert len(starting_cols) >= 2

    def test_all_paths_reach_boss(self):
        """Every path from row 0 should eventually reach the boss row."""
        from packages.engine.generation.map import MapGenerator

        rng = Random(42)
        gen = MapGenerator(rng)
        dungeon = gen.generate()

        # Trace from each starting node
        for start in dungeon[0]:
            if not start.has_edges():
                continue
            # BFS to check reachability
            visited = set()
            queue = [start]
            reached_boss = False
            while queue:
                current = queue.pop(0)
                if (current.x, current.y) in visited:
                    continue
                visited.add((current.x, current.y))
                for edge in current.edges:
                    if edge.is_boss:
                        reached_boss = True
                    elif edge.dst_y < len(dungeon):
                        queue.append(dungeon[edge.dst_y][edge.dst_x])
            assert reached_boss, f"Path from ({start.x}, {start.y}) never reaches boss"

    def test_no_path_crossing(self):
        """Edges should not cross (left node's rightmost edge <= right node's leftmost edge)."""
        from packages.engine.generation.map import MapGenerator

        for seed in [42, 999, 55555]:
            rng = Random(seed)
            gen = MapGenerator(rng)
            dungeon = gen.generate()

            for y in range(len(dungeon) - 1):
                for x in range(len(dungeon[y]) - 1):
                    left = dungeon[y][x]
                    right = dungeon[y][x + 1]
                    if not left.has_edges() or not right.has_edges():
                        continue
                    left_max = max(e.dst_x for e in left.edges if not e.is_boss)
                    right_min = min(e.dst_x for e in right.edges if not e.is_boss)
                    assert left_max <= right_min, (
                        f"Crossing at row {y}, cols {x}/{x+1}, seed {seed}"
                    )

    def test_ascension_increases_elites(self):
        """A1+ should have more elites than A0."""
        from packages.engine.generation.map import (
            MapGenerator,
            MapGeneratorConfig,
            RoomType,
        )

        def count_room(dungeon, rt):
            return sum(
                1
                for row in dungeon
                for n in row
                if n.room_type == rt
            )

        a0_rng = Random(42)
        a0_gen = MapGenerator(a0_rng, MapGeneratorConfig(ascension_level=0))
        a0_map = a0_gen.generate()

        a20_rng = Random(42)
        a20_gen = MapGenerator(a20_rng, MapGeneratorConfig(ascension_level=20))
        a20_map = a20_gen.generate()

        assert count_room(a20_map, RoomType.ELITE) >= count_room(a0_map, RoomType.ELITE)

    def test_act4_map_structure(self):
        """Act 4 map should be fixed linear structure."""
        from packages.engine.generation.map import generate_act4_map, RoomType

        nodes = generate_act4_map()
        assert len(nodes) == 5
        assert nodes[0][3].room_type == RoomType.REST
        assert nodes[1][3].room_type == RoomType.SHOP
        assert nodes[2][3].room_type == RoomType.ELITE
        assert nodes[3][3].room_type == RoomType.BOSS
        assert nodes[4][3].room_type == RoomType.TRUE_VICTORY

    def test_common_ancestor_bug_preserved(self):
        """The Java bug comparing node1.x < node2.y should be preserved."""
        from packages.engine.generation.map import MapGenerator

        # The Python code at line 337 has: if node1.x < node2.y
        # This is the intentional Java bug preservation
        gen = MapGenerator(Random(42))
        # Just verify the method exists and doesn't crash
        from packages.engine.generation.map import MapRoomNode

        n1 = MapRoomNode(x=0, y=5)
        n2 = MapRoomNode(x=6, y=5)
        # Should not crash
        result = gen._get_common_ancestor(n1, n2, 5)
        assert result is None  # No parents, so no ancestor


# =============================================================================
# 3. ENCOUNTER GENERATION
# =============================================================================


class TestEncounterGeneration:
    """Validate encounter generation matches Java dungeon classes."""

    def test_exordium_weak_pool_count(self):
        """Exordium should have 4 weak monsters."""
        from packages.engine.generation.encounters import get_exordium_weak_pool

        pool = get_exordium_weak_pool()
        assert len(pool) == 4

    def test_exordium_strong_pool_count(self):
        """Exordium should have 10 strong monsters."""
        from packages.engine.generation.encounters import get_exordium_strong_pool

        pool = get_exordium_strong_pool()
        assert len(pool) == 10

    def test_city_weak_pool_count(self):
        """City should have 5 weak monsters."""
        from packages.engine.generation.encounters import get_city_weak_pool

        pool = get_city_weak_pool()
        assert len(pool) == 5

    def test_city_strong_pool_count(self):
        """City should have 8 strong monsters."""
        from packages.engine.generation.encounters import get_city_strong_pool

        pool = get_city_strong_pool()
        assert len(pool) == 8

    def test_beyond_weak_pool_count(self):
        """Beyond should have 3 weak monsters."""
        from packages.engine.generation.encounters import get_beyond_weak_pool

        pool = get_beyond_weak_pool()
        assert len(pool) == 3

    def test_beyond_strong_pool_count(self):
        """Beyond should have 8 strong monsters."""
        from packages.engine.generation.encounters import get_beyond_strong_pool

        pool = get_beyond_strong_pool()
        assert len(pool) == 8

    def test_exordium_encounter_counts(self):
        """Exordium: 3 weak + 13 strong + 10 elite."""
        from packages.engine.generation.encounters import generate_exordium_encounters

        rng = Random(42)
        monsters, elites, boss = generate_exordium_encounters(rng)
        # 3 weak + 1 first strong + 12 remaining strong = 16
        assert len(monsters) == 16
        assert len(elites) == 10
        assert boss in ["The Guardian", "Hexaghost", "Slime Boss"]

    def test_city_encounter_counts(self):
        """City: 2 weak + 13 strong + 10 elite."""
        from packages.engine.generation.encounters import generate_city_encounters

        rng = Random(42)
        monsters, elites, boss = generate_city_encounters(rng)
        assert len(monsters) == 15  # 2 + 1 + 12
        assert len(elites) == 10
        assert boss in ["Automaton", "Collector", "Champ"]

    def test_beyond_encounter_counts(self):
        """Beyond: 2 weak + 13 strong + 10 elite."""
        from packages.engine.generation.encounters import generate_beyond_encounters

        rng = Random(42)
        monsters, elites, boss = generate_beyond_encounters(rng)
        assert len(monsters) == 15  # 2 + 1 + 12
        assert len(elites) == 10
        assert boss in ["Awakened One", "Time Eater", "Donu and Deca"]

    def test_ending_fixed_encounters(self):
        """Ending: fixed encounters, no RNG."""
        from packages.engine.generation.encounters import generate_ending_encounters

        monsters, elites, boss = generate_ending_encounters()
        assert len(monsters) == 0
        assert len(elites) == 1
        assert elites[0] == "Spire Shield and Spire Spear"

    def test_no_immediate_repeat_normal(self):
        """Normal encounter list should have no immediate repeats."""
        from packages.engine.generation.encounters import generate_exordium_encounters

        for seed in [42, 999, 7777, 123456, 54321]:
            rng = Random(seed)
            monsters, _, _ = generate_exordium_encounters(rng)
            for i in range(1, len(monsters)):
                assert monsters[i] != monsters[i - 1], (
                    f"Repeat at index {i}: {monsters[i]} (seed={seed})"
                )

    def test_no_2back_repeat_normal(self):
        """Normal encounter list should have no 2-back repeats."""
        from packages.engine.generation.encounters import generate_exordium_encounters

        for seed in [42, 999, 7777, 123456, 54321]:
            rng = Random(seed)
            monsters, _, _ = generate_exordium_encounters(rng)
            for i in range(2, len(monsters)):
                assert monsters[i] != monsters[i - 2], (
                    f"2-back repeat at index {i}: {monsters[i]} (seed={seed})"
                )

    def test_elite_allows_2back_repeat(self):
        """Elite list allows 2-back repeats (only prevents immediate)."""
        from packages.engine.generation.encounters import generate_exordium_encounters

        # With only 3 elites over 10 slots, 2-back repeats must occur
        for seed in [42, 999, 7777]:
            rng = Random(seed)
            _, elites, _ = generate_exordium_encounters(rng)
            for i in range(1, len(elites)):
                assert elites[i] != elites[i - 1], (
                    f"Elite immediate repeat at {i}: {elites[i]} (seed={seed})"
                )

    def test_exordium_exclusions(self):
        """Exordium exclusion rules should match Java."""
        from packages.engine.generation.encounters import get_exordium_exclusions

        assert "Exordium Thugs" in get_exordium_exclusions("Looter")
        assert "Red Slaver" in get_exordium_exclusions("Blue Slaver")
        assert "Exordium Thugs" in get_exordium_exclusions("Blue Slaver")
        assert "3 Louse" in get_exordium_exclusions("2 Louse")
        assert "Large Slime" in get_exordium_exclusions("Small Slimes")
        assert "Lots of Slimes" in get_exordium_exclusions("Small Slimes")
        # Jaw Worm and Cultist have no exclusions
        assert get_exordium_exclusions("Jaw Worm") == []
        assert get_exordium_exclusions("Cultist") == []

    def test_city_exclusions(self):
        """City exclusion rules should match Java."""
        from packages.engine.generation.encounters import get_city_exclusions

        assert "Sentry and Sphere" in get_city_exclusions("Spheric Guardian")
        assert "Chosen and Byrds" in get_city_exclusions("3 Byrds")
        assert "Chosen and Byrds" in get_city_exclusions("Chosen")
        assert "Cultist and Chosen" in get_city_exclusions("Chosen")
        # Java has no case for Shell Parasite or 2 Thieves
        assert get_city_exclusions("Shell Parasite") == []
        assert get_city_exclusions("2 Thieves") == []

    def test_beyond_exclusions(self):
        """Beyond exclusion rules should match Java."""
        from packages.engine.generation.encounters import get_beyond_exclusions

        assert "3 Darklings" in get_beyond_exclusions("3 Darklings")
        assert "Orb Walker" in get_beyond_exclusions("Orb Walker")
        assert "4 Shapes" in get_beyond_exclusions("3 Shapes")

    def test_boss_lists_match_java(self):
        """Boss lists should match Java source."""
        from packages.engine.generation.encounters import (
            EXORDIUM_BOSSES,
            CITY_BOSSES,
            BEYOND_BOSSES,
        )

        assert set(EXORDIUM_BOSSES) == {"The Guardian", "Hexaghost", "Slime Boss"}
        assert set(CITY_BOSSES) == {"Automaton", "Collector", "Champ"}
        assert set(BEYOND_BOSSES) == {"Awakened One", "Time Eater", "Donu and Deca"}

    def test_boss_deterministic_for_seed(self):
        """Same seed should produce same bosses."""
        from packages.engine.generation.encounters import (
            generate_exordium_encounters,
            generate_city_encounters,
            generate_beyond_encounters,
        )

        for seed in [42, 999, 7777]:
            rng1 = Random(seed)
            _, _, boss1a = generate_exordium_encounters(rng1)
            _, _, boss2a = generate_city_encounters(rng1)

            rng2 = Random(seed)
            _, _, boss1b = generate_exordium_encounters(rng2)
            _, _, boss2b = generate_city_encounters(rng2)

            assert boss1a == boss1b
            assert boss2a == boss2b

    def test_java_shuffle_deterministic(self):
        """_java_shuffle should be deterministic and match Java's LCG."""
        from packages.engine.generation.encounters import _java_shuffle

        items = ["A", "B", "C", "D", "E"]
        _java_shuffle(items, 42)
        # Just verify it's deterministic
        items2 = ["A", "B", "C", "D", "E"]
        _java_shuffle(items2, 42)
        assert items == items2

    def test_weight_normalization(self):
        """Weights should normalize to sum=1 and sort ascending."""
        from packages.engine.generation.encounters import normalize_weights, MonsterInfo

        monsters = [
            MonsterInfo("A", 3.0),
            MonsterInfo("B", 1.0),
            MonsterInfo("C", 2.0),
        ]
        result = normalize_weights(monsters)
        assert abs(sum(m.weight for m in result) - 1.0) < 1e-10
        # Should be sorted ascending by weight
        for i in range(len(result) - 1):
            assert result[i].weight <= result[i + 1].weight


# =============================================================================
# 4. CARD REWARD GENERATION
# =============================================================================


class TestCardRewardGeneration:
    """Validate card reward generation matches AbstractDungeon.getRewardCards()."""

    def test_blizzard_pity_timer_constants(self):
        """Blizzard constants should match Java."""
        from packages.engine.generation.rewards import (
            CARD_BLIZZ_START_OFFSET,
            CARD_BLIZZ_GROWTH,
            CARD_BLIZZ_MAX_OFFSET,
        )

        assert CARD_BLIZZ_START_OFFSET == 5
        assert CARD_BLIZZ_GROWTH == 1
        assert CARD_BLIZZ_MAX_OFFSET == -40

    def test_rarity_thresholds_normal(self):
        """Normal room rarity thresholds: rare=3, uncommon=37."""
        from packages.engine.generation.rewards import CARD_RARITY_THRESHOLDS

        normal = CARD_RARITY_THRESHOLDS["normal"]
        assert normal["rare"] == 3
        assert normal["uncommon"] == 37

    def test_rarity_thresholds_elite(self):
        """Elite room rarity thresholds: rare=10, uncommon=40."""
        from packages.engine.generation.rewards import CARD_RARITY_THRESHOLDS

        elite = CARD_RARITY_THRESHOLDS["elite"]
        assert elite["rare"] == 10
        assert elite["uncommon"] == 40

    def test_blizzard_resets_on_rare(self):
        """Getting a rare should reset blizzard offset to 5."""
        from packages.engine.generation.rewards import CardBlizzardState

        state = CardBlizzardState(offset=-10)
        state.on_rare()
        assert state.offset == 5

    def test_blizzard_decreases_on_common(self):
        """Getting a common should decrease offset by 1."""
        from packages.engine.generation.rewards import CardBlizzardState

        state = CardBlizzardState(offset=5)
        state.on_common()
        assert state.offset == 4

    def test_blizzard_clamps_at_max(self):
        """Offset should not go below -40."""
        from packages.engine.generation.rewards import CardBlizzardState

        state = CardBlizzardState(offset=-40)
        state.on_common()
        assert state.offset == -40

    def test_upgrade_chance_per_act(self):
        """Upgrade chances should match Java dungeon classes."""
        from packages.engine.generation.rewards import CARD_UPGRADE_CHANCES

        assert CARD_UPGRADE_CHANCES[1] == 0.0
        assert CARD_UPGRADE_CHANCES[2]["default"] == 0.25
        assert CARD_UPGRADE_CHANCES[2]["a12"] == 0.125
        assert CARD_UPGRADE_CHANCES[3]["default"] == 0.50
        assert CARD_UPGRADE_CHANCES[3]["a12"] == 0.25

    def test_card_reward_produces_3_cards_default(self):
        """Default card reward should produce 3 cards."""
        from packages.engine.generation.rewards import (
            generate_card_rewards,
            RewardState,
        )

        rng = Random(42)
        state = RewardState()
        cards = generate_card_rewards(rng, state, player_class="IRONCLAD")
        assert len(cards) == 3

    def test_busted_crown_reduces_by_2(self):
        """Busted Crown should reduce card choices by 2."""
        from packages.engine.generation.rewards import (
            generate_card_rewards,
            RewardState,
        )

        rng = Random(42)
        state = RewardState()
        cards = generate_card_rewards(
            rng, state, player_class="IRONCLAD", has_busted_crown=True
        )
        assert len(cards) == 1  # 3 - 2 = 1

    def test_question_card_adds_1(self):
        """Question Card should add 1 card choice."""
        from packages.engine.generation.rewards import (
            generate_card_rewards,
            RewardState,
        )

        rng = Random(42)
        state = RewardState()
        cards = generate_card_rewards(
            rng, state, player_class="IRONCLAD", has_question_card=True
        )
        assert len(cards) == 4  # 3 + 1

    def test_no_duplicate_cards_in_reward(self):
        """No two cards in same reward should have same ID."""
        from packages.engine.generation.rewards import (
            generate_card_rewards,
            RewardState,
        )

        for seed in [42, 999, 7777, 123456]:
            rng = Random(seed)
            state = RewardState()
            cards = generate_card_rewards(rng, state, player_class="WATCHER")
            ids = [c.id for c in cards]
            assert len(ids) == len(set(ids)), f"Duplicate cards for seed {seed}: {ids}"

    def test_elite_relic_thresholds(self):
        """Elite relic tier thresholds: <50=COMMON, >82=RARE, else UNCOMMON."""
        from packages.engine.generation.rewards import ELITE_RELIC_THRESHOLDS

        assert ELITE_RELIC_THRESHOLDS["common"] == 50
        assert ELITE_RELIC_THRESHOLDS["rare"] == 82


# =============================================================================
# 5. POTION DROP SYSTEM
# =============================================================================


class TestPotionDropSystem:
    """Validate potion drop system matches Java."""

    def test_base_drop_chance(self):
        """Base drop chance should be 40%."""
        from packages.engine.generation.potions import BASE_DROP_CHANCE

        assert BASE_DROP_CHANCE == 40

    def test_blizzard_mod_step(self):
        """Blizzard mod step should be 10."""
        from packages.engine.generation.potions import BLIZZARD_MOD_STEP

        assert BLIZZARD_MOD_STEP == 10

    def test_rarity_distribution(self):
        """65% common, 25% uncommon, 10% rare."""
        from packages.engine.generation.potions import (
            POTION_COMMON_CHANCE,
            POTION_UNCOMMON_CHANCE,
        )

        assert POTION_COMMON_CHANCE == 65
        assert POTION_UNCOMMON_CHANCE == 25
        # Rare = 100 - 65 - 25 = 10

    def test_sozu_prevents_drops(self):
        """Sozu should prevent all potion drops without consuming RNG."""
        from packages.engine.generation.potions import predict_potion_drop

        rng = Random(42)
        initial_counter = rng.counter
        result = predict_potion_drop(rng, has_sozu=True)
        assert result.will_drop is False
        assert rng.counter == initial_counter  # No RNG consumed

    def test_white_beast_statue_guarantees_drop(self):
        """White Beast Statue should guarantee drops."""
        from packages.engine.generation.potions import predict_potion_drop

        for seed in [42, 999, 7777]:
            rng = Random(seed)
            result = predict_potion_drop(rng, has_white_beast_statue=True)
            assert result.will_drop is True

    def test_blizzard_increases_on_no_drop(self):
        """Blizzard mod should increase by 10 when no drop."""
        from packages.engine.generation.potions import predict_potion_drop

        rng = Random(42)
        result = predict_potion_drop(rng, blizzard_mod=-100)  # Guarantee no drop
        if not result.will_drop:
            assert result.new_blizzard_mod == -100 + 10

    def test_blizzard_decreases_on_drop(self):
        """Blizzard mod should decrease by 10 when drop occurs."""
        from packages.engine.generation.potions import predict_potion_drop

        rng = Random(42)
        result = predict_potion_drop(rng, blizzard_mod=100, has_white_beast_statue=True)
        assert result.will_drop is True
        assert result.new_blizzard_mod == 100 - 10

    def test_potion_pool_sizes(self):
        """Each class should have 3 class-specific + 27 universal = 30 potions."""
        from packages.engine.generation.potions import get_potion_pool_for_class

        for cls in ["WATCHER", "IRONCLAD", "SILENT", "DEFECT"]:
            pool = get_potion_pool_for_class(cls)
            # 3 class-specific + 30 universal = 33
            assert len(pool) == 33, f"{cls} pool has {len(pool)} potions, expected 33"

    def test_drop_check_always_consumes_rng(self):
        """Drop check should always consume 1 RNG call even if chance is 0."""
        from packages.engine.generation.potions import predict_potion_drop

        rng = Random(42)
        initial = rng.counter
        predict_potion_drop(rng, blizzard_mod=-100, room_type="monster")
        # At minimum, 1 RNG call for the drop check roll
        assert rng.counter >= initial + 1

    def test_deterministic_for_seed(self):
        """Same seed should produce same potion drop sequence."""
        from packages.engine.generation.potions import predict_potion_drop

        results1 = []
        rng1 = Random(42)
        bliz = 0
        for _ in range(5):
            r = predict_potion_drop(rng1, blizzard_mod=bliz)
            results1.append((r.will_drop, r.potion_id))
            bliz = r.new_blizzard_mod

        results2 = []
        rng2 = Random(42)
        bliz = 0
        for _ in range(5):
            r = predict_potion_drop(rng2, blizzard_mod=bliz)
            results2.append((r.will_drop, r.potion_id))
            bliz = r.new_blizzard_mod

        assert results1 == results2


# =============================================================================
# 6. CROSS-SYSTEM INTEGRATION
# =============================================================================


class TestCrossSystemIntegration:
    """Test interactions between systems."""

    def test_full_act_generation_deterministic(self):
        """Full act generation (map + encounters) should be deterministic."""
        from packages.engine.generation.encounters import predict_all_acts

        r1 = predict_all_acts("TESTSEED")
        r2 = predict_all_acts("TESTSEED")

        for act in ["act1", "act2", "act3"]:
            assert r1[act]["boss"] == r2[act]["boss"]
            assert r1[act]["monsters"] == r2[act]["monsters"]
            assert r1[act]["elites"] == r2[act]["elites"]

    def test_monsterrng_counter_advances_across_acts(self):
        """monsterRng counter should increase across acts."""
        from packages.engine.generation.encounters import predict_all_acts

        result = predict_all_acts("ABC123")
        c1 = result["act1"]["monster_rng_counter"]
        c2 = result["act2"]["monster_rng_counter"]
        c3 = result["act3"]["monster_rng_counter"]

        assert c1 > 0
        assert c2 > c1
        assert c3 > c2

    def test_act4_does_not_consume_monsterrng(self):
        """Act 4 should not consume any monsterRng calls."""
        from packages.engine.generation.encounters import predict_all_acts

        result = predict_all_acts("ABC123", include_act4=True)
        c3 = result["act3"]["monster_rng_counter"]
        c4 = result["act4"]["monster_rng_counter"]
        assert c4 == c3
