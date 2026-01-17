"""
Java HashMap Iteration Order Replication

Replicates Java's HashMap iteration order in Python for seed-deterministic
card pool ordering in Slay the Spire.

Java HashMap internals:
- Default capacity: 16 (always power of 2)
- Load factor: 0.75
- Bucket index: hash & (capacity - 1)
- Iteration: buckets 0 to n-1, linked list order within buckets

References:
- https://peterchng.com/blog/2022/06/17/what-iteration-order-can-you-expect-from-a-java-hashmap/
- https://www.geeksforgeeks.org/java/internal-working-of-hashmap-java/
- https://gist.github.com/hanleybrand/5224673
"""

from typing import List, Dict, Any, Tuple, Optional
from dataclasses import dataclass, field


def java_string_hashcode(s: str) -> int:
    """
    Replicate Java's String.hashCode() method.

    Formula: s[0]*31^(n-1) + s[1]*31^(n-2) + ... + s[n-1]

    Uses Horner's rule: h = 31*h + c for each character.
    Result is a signed 32-bit integer (can be negative).
    """
    h = 0
    for c in s:
        # Multiply by 31 and add character, keeping as 32-bit unsigned
        h = (31 * h + ord(c)) & 0xFFFFFFFF

    # Convert to signed 32-bit integer (Java's int is signed)
    if h >= 0x80000000:
        h -= 0x100000000

    return h


def java_hashmap_hash(key_hashcode: int) -> int:
    """
    Java HashMap's internal hash() function.

    Spreads the impact of higher bits downward to reduce collisions
    for hash codes that differ only in upper bits.

    From HashMap.java:
        static final int hash(Object key) {
            int h;
            return (key == null) ? 0 : (h = key.hashCode()) ^ (h >>> 16);
        }
    """
    h = key_hashcode & 0xFFFFFFFF  # Treat as unsigned for shift
    # XOR with upper 16 bits shifted down (unsigned right shift)
    return (key_hashcode ^ (h >> 16)) & 0xFFFFFFFF


def bucket_index(hash_value: int, capacity: int) -> int:
    """
    Calculate bucket index from hash.

    Java: (n - 1) & hash
    """
    # Ensure we're working with the right bits
    return (capacity - 1) & (hash_value & 0xFFFFFFFF)


@dataclass
class HashMapEntry:
    """Entry in a HashMap bucket."""
    key: str
    value: Any
    hash_value: int
    next: Optional['HashMapEntry'] = None


class JavaHashMap:
    """
    Simulates Java HashMap iteration order.

    This class replicates the exact iteration order of Java's HashMap
    by simulating its bucket structure and insertion behavior.
    """

    DEFAULT_INITIAL_CAPACITY = 16
    MAXIMUM_CAPACITY = 1 << 30
    DEFAULT_LOAD_FACTOR = 0.75

    def __init__(self, initial_capacity: int = DEFAULT_INITIAL_CAPACITY):
        """Initialize with given capacity (will be rounded up to power of 2)."""
        self.capacity = self._table_size_for(initial_capacity)
        self.table: List[Optional[HashMapEntry]] = [None] * self.capacity
        self.size = 0
        self.threshold = int(self.capacity * self.DEFAULT_LOAD_FACTOR)

    @staticmethod
    def _table_size_for(cap: int) -> int:
        """Returns a power of two size for the given target capacity."""
        n = cap - 1
        n |= n >> 1
        n |= n >> 2
        n |= n >> 4
        n |= n >> 8
        n |= n >> 16
        if n < 0:
            return 1
        if n >= JavaHashMap.MAXIMUM_CAPACITY:
            return JavaHashMap.MAXIMUM_CAPACITY
        return n + 1

    def put(self, key: str, value: Any) -> None:
        """Insert a key-value pair."""
        key_hash = java_string_hashcode(key)
        hash_val = java_hashmap_hash(key_hash)
        idx = bucket_index(hash_val, self.capacity)

        # Check if key already exists
        entry = self.table[idx]
        prev = None
        while entry is not None:
            if entry.key == key:
                entry.value = value  # Update existing
                return
            prev = entry
            entry = entry.next

        # Add new entry
        new_entry = HashMapEntry(key=key, value=value, hash_value=hash_val)
        if prev is None:
            self.table[idx] = new_entry
        else:
            prev.next = new_entry

        self.size += 1

        # Resize if needed
        if self.size > self.threshold:
            self._resize()

    def _resize(self) -> None:
        """
        Double the capacity and rehash all entries.

        Java 8 HashMap resize maintains entry order within each bucket by
        splitting entries into "lo" (stays at same index) and "hi" (moves
        to index + oldCap) lists, preserving insertion order.
        """
        old_table = self.table
        old_capacity = self.capacity

        new_capacity = old_capacity * 2
        if new_capacity > self.MAXIMUM_CAPACITY:
            new_capacity = self.MAXIMUM_CAPACITY

        self.capacity = new_capacity
        self.threshold = int(new_capacity * self.DEFAULT_LOAD_FACTOR)
        self.table = [None] * new_capacity

        # Java 8 resize: split each bucket into lo/hi lists maintaining order
        for j in range(old_capacity):
            entry = old_table[j]
            if entry is None:
                continue

            # Collect entries into lo (stays at j) and hi (goes to j + oldCap)
            lo_head = lo_tail = None
            hi_head = hi_tail = None

            while entry is not None:
                next_entry = entry.next
                # Check if entry stays in lo bucket or moves to hi bucket
                # Java: (e.hash & oldCap) == 0 means stays in lo
                if (entry.hash_value & old_capacity) == 0:
                    if lo_tail is None:
                        lo_head = entry
                    else:
                        lo_tail.next = entry
                    lo_tail = entry
                else:
                    if hi_tail is None:
                        hi_head = entry
                    else:
                        hi_tail.next = entry
                    hi_tail = entry
                entry = next_entry

            # Set bucket heads and terminate lists
            if lo_tail is not None:
                lo_tail.next = None
                self.table[j] = lo_head
            if hi_tail is not None:
                hi_tail.next = None
                self.table[j + old_capacity] = hi_head

    def keys_in_iteration_order(self) -> List[str]:
        """
        Return keys in Java HashMap iteration order.

        Iterates buckets from 0 to capacity-1, visiting all entries
        in each bucket's linked list before moving to the next bucket.
        """
        result = []
        for bucket in self.table:
            entry = bucket
            while entry is not None:
                result.append(entry.key)
                entry = entry.next
        return result

    def values_in_iteration_order(self) -> List[Any]:
        """Return values in Java HashMap iteration order."""
        result = []
        for bucket in self.table:
            entry = bucket
            while entry is not None:
                result.append(entry.value)
                entry = entry.next
        return result

    def items_in_iteration_order(self) -> List[Tuple[str, Any]]:
        """Return (key, value) pairs in Java HashMap iteration order."""
        result = []
        for bucket in self.table:
            entry = bucket
            while entry is not None:
                result.append((entry.key, entry.value))
                entry = entry.next
        return result


def get_java_iteration_order(keys: List[str], initial_capacity: int = 16) -> List[str]:
    """
    Given a list of string keys, return them in Java HashMap iteration order.

    This simulates adding all keys to a Java HashMap and then iterating
    over the entrySet().

    IMPORTANT: Uses default initial capacity (16) to match Java's default
    HashMap behavior, including resize operations as entries are added.
    The resize process affects iteration order!

    Args:
        keys: List of string keys to insert
        initial_capacity: Initial capacity (default 16 to match Java)

    Returns:
        Keys in the order Java HashMap.entrySet() would iterate them
    """
    hashmap = JavaHashMap(initial_capacity=initial_capacity)
    for key in keys:
        hashmap.put(key, key)  # Value doesn't matter for ordering

    return hashmap.keys_in_iteration_order()


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    print("=== Java HashMap Iteration Order Tests ===\n")

    # Test String.hashCode()
    test_strings = ["Alpha", "BattleHymn", "Blasphemy", "BowlingBash", "LikeWater"]
    print("String.hashCode() values:")
    for s in test_strings:
        h = java_string_hashcode(s)
        print(f"  '{s}' -> {h}")

    print()

    # Test HashMap iteration order with a small set
    small_keys = ["C", "A", "B", "D"]
    print(f"Input order: {small_keys}")
    print(f"HashMap iteration order: {get_java_iteration_order(small_keys)}")

    print()

    # Test with Watcher card IDs
    watcher_uncommon_ids = [
        "BattleHymn", "CarveReality", "Collect", "Conclude", "DeceiveReality",
        "EmptyMind", "Fasting", "FearNoEvil", "ForeignInfluence", "Foresight",
        "Indignation", "InnerPeace", "LikeWater", "Meditate", "MentalFortress",
        "Nirvana", "Perseverance", "Pray", "ReachHeaven", "Rushdown",
        "Sanctity", "SandsOfTime", "SignatureMove", "Study", "Swivel",
        "TalkToTheHand", "Tantrum", "Vengeance", "Wallop", "WaveOfTheHand",
        "Weave", "WheelKick", "WindmillStrike", "Worship", "WreathOfFlame"
    ]

    print(f"Watcher UNCOMMON cards ({len(watcher_uncommon_ids)} cards)")
    print("Alphabetical order (first 10):", watcher_uncommon_ids[:10])

    hashmap_order = get_java_iteration_order(watcher_uncommon_ids)
    print("HashMap iteration order (first 10):", hashmap_order[:10])

    # Show full HashMap order
    print("\nFull HashMap iteration order:")
    for i, card_id in enumerate(hashmap_order):
        print(f"  {i}: {card_id}")

    # Find positions of specific cards
    print("\nPositions of test cards:")
    for card in ["LikeWater", "DeceiveReality", "Foresight"]:
        pos = hashmap_order.index(card) if card in hashmap_order else -1
        print(f"  {card}: index {pos}")
