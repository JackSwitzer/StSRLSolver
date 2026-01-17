"""
Debug script to trace exact boss relic pool order.
"""
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.utils.java_hashmap import JavaHashMap
from core.generation.relics import java_collections_shuffle
from core.state.rng import Random, seed_to_long

# All shared relics in RelicLibrary.initialize() order with their IDs and tiers
# Format: (ID, TIER)
SHARED_RELICS = [
    ("Abacus", "UNCOMMON"),
    ("Akabeko", "COMMON"),
    ("Anchor", "COMMON"),
    ("Ancient Tea Set", "COMMON"),
    ("Art of War", "UNCOMMON"),
    ("Astrolabe", "BOSS"),
    ("Bag of Marbles", "COMMON"),
    ("Bag of Preparation", "UNCOMMON"),
    ("Bird Faced Urn", "RARE"),
    ("Black Star", "BOSS"),
    ("Blood Vial", "COMMON"),
    ("Bloody Idol", "RARE"),
    ("Blue Candle", "UNCOMMON"),
    ("The Boot", "COMMON"),
    ("Bottled Flame", "UNCOMMON"),
    ("Bottled Lightning", "UNCOMMON"),
    ("Bottled Tornado", "UNCOMMON"),
    ("Bronze Scales", "COMMON"),
    ("Busted Crown", "BOSS"),
    ("Calipers", "RARE"),
    ("Calling Bell", "BOSS"),
    ("CaptainsWheel", "RARE"),
    ("Cauldron", "SHOP"),
    ("Centennial Puzzle", "UNCOMMON"),
    ("Ceramic Fish", "COMMON"),
    ("Chemical X", "RARE"),
    ("Clockwork Souvenir", "SHOP"),
    ("Coffee Dripper", "BOSS"),
    ("Courier", "SHOP"),
    ("CultistMask", "SPECIAL"),
    ("Cursed Key", "BOSS"),
    ("Darkstone Periapt", "UNCOMMON"),
    ("Dead Branch", "RARE"),
    ("Dollys Mirror", "SHOP"),
    ("Dream Catcher", "COMMON"),
    ("Du-Vu Doll", "RARE"),
    ("Ectoplasm", "BOSS"),
    ("Empty Cage", "BOSS"),
    ("Enchiridion", "SPECIAL"),
    ("Eternal Feather", "UNCOMMON"),
    ("FaceOfCleric", "RARE"),
    ("Fossilized Helix", "RARE"),
    ("Frozen Egg 2", "UNCOMMON"),
    ("Frozen Eye", "SHOP"),
    ("Fusion Hammer", "BOSS"),
    ("Gambling Chip", "UNCOMMON"),
    ("Ginger", "RARE"),
    ("Girya", "RARE"),
    ("Golden Idol", "COMMON"),
    ("Gremlin Horn", "UNCOMMON"),
    ("GremlinMask", "SPECIAL"),
    ("HandDrill", "UNCOMMON"),
    ("Happy Flower", "COMMON"),
    ("HornCleat", "COMMON"),
    ("Ice Cream", "RARE"),
    ("Incense Burner", "RARE"),
    ("Ink Bottle", "UNCOMMON"),
    ("Juzu Bracelet", "UNCOMMON"),
    ("Kunai", "UNCOMMON"),
    ("Lantern", "COMMON"),
    ("Letter Opener", "UNCOMMON"),
    ("Lizard Tail", "RARE"),
    ("Mango", "RARE"),
    ("Mark of the Bloom", "SPECIAL"),
    ("Matryoshka", "RARE"),
    ("Maw Bank", "COMMON"),
    ("Meal Ticket", "COMMON"),
    ("Meat on the Bone", "UNCOMMON"),
    ("Medical Kit", "RARE"),
    ("Membership Card", "SHOP"),
    ("Mercury Hourglass", "UNCOMMON"),
    ("Molten Egg 2", "UNCOMMON"),
    ("Mummified Hand", "RARE"),
    ("MutagenicStrength", "SPECIAL"),
    ("Necronomicon", "RARE"),
    ("NeowsBlessing", "SPECIAL"),
    ("Nilrys Codex", "RARE"),
    ("Nloth's Gift", "SPECIAL"),
    ("N'loth's Hungry Face", "SPECIAL"),
    ("Nunchaku", "UNCOMMON"),
    ("Oddly Smooth Stone", "COMMON"),
    ("Odd Mushroom", "UNCOMMON"),
    ("Old Coin", "COMMON"),
    ("Omamori", "RARE"),
    ("Orange Pellets", "RARE"),
    ("Orichalcum", "COMMON"),
    ("Ornamental Fan", "UNCOMMON"),
    ("Orrery", "SHOP"),
    ("Pandora's Box", "BOSS"),
    ("Pantograph", "RARE"),
    ("Peace Pipe", "RARE"),
    ("Pear", "RARE"),
    ("Pen Nib", "COMMON"),
    ("Philosopher's Stone", "BOSS"),
    ("Pocketwatch", "UNCOMMON"),
    ("Potion Belt", "COMMON"),
    ("Prayer Wheel", "RARE"),
    ("PreservedInsect", "UNCOMMON"),
    ("PrismaticShard", "SPECIAL"),
    ("Question Card", "SHOP"),
    ("Red Mask", "SPECIAL"),
    ("Regal Pillow", "COMMON"),
    ("Runic Dome", "BOSS"),
    ("Runic Pyramid", "BOSS"),
    ("SacredBark", "BOSS"),
    ("Shovel", "RARE"),
    ("Shuriken", "UNCOMMON"),
    ("Singing Bowl", "RARE"),
    ("SlaversCollar", "BOSS"),
    ("Sling", "COMMON"),
    ("Smiling Mask", "COMMON"),
    ("Snecko Eye", "BOSS"),
    ("Sozu", "BOSS"),
    ("Spirit Poop", "SPECIAL"),
    ("SsserpentHead", "SPECIAL"),
    ("StoneCalendar", "RARE"),
    ("Strange Spoon", "RARE"),
    ("Strawberry", "RARE"),
    ("StrikeDummy", "SPECIAL"),
    ("Sundial", "UNCOMMON"),
    ("Thread and Needle", "UNCOMMON"),
    ("Tiny Chest", "UNCOMMON"),
    ("Tiny House", "BOSS"),
    ("Toolbox", "RARE"),
    ("Torii", "UNCOMMON"),
    ("Toxic Egg 2", "UNCOMMON"),
    ("Toy Ornithopter", "COMMON"),
    ("Tungsten Rod", "RARE"),
    ("Turnip", "RARE"),
    ("Unceasing Top", "UNCOMMON"),
    ("Vajra", "COMMON"),
    ("Velvet Choker", "BOSS"),
    ("Waffle", "RARE"),
    ("War Paint", "UNCOMMON"),
    ("WarpedTongs", "UNCOMMON"),
    ("Whetstone", "UNCOMMON"),
    ("White Beast Statue", "RARE"),
    ("Wing Boots", "RARE"),
]

# Purple (Watcher) relics
PURPLE_RELICS = [
    ("CloakClasp", "RARE"),
    ("Damaru", "COMMON"),
    ("GoldenEye", "RARE"),
    ("HolyWater", "BOSS"),
    ("Melange", "SHOP"),
    ("PureWater", "STARTER"),
    ("VioletLotus", "BOSS"),
    ("TeardropLocket", "UNCOMMON"),
    ("Yang", "UNCOMMON"),
]


def simulate_boss_pool_order():
    """Simulate the boss relic pool order by building full HashMaps."""

    # Build sharedRelics HashMap
    shared_map = JavaHashMap(16)
    for relic_id, tier in SHARED_RELICS:
        shared_map.put(relic_id, tier)

    # Build purpleRelics HashMap
    purple_map = JavaHashMap(16)
    for relic_id, tier in PURPLE_RELICS:
        purple_map.put(relic_id, tier)

    print(f"Shared HashMap: {shared_map.size} entries, capacity {shared_map.capacity}")
    print(f"Purple HashMap: {purple_map.size} entries, capacity {purple_map.capacity}")

    # Simulate populateRelicPool for BOSS tier
    # 1. Iterate sharedRelics HashMap
    # 2. Iterate purpleRelics HashMap (for Watcher)
    # 3. Filter to BOSS tier only

    boss_pool = []

    # From sharedRelics
    for relic_id, tier in shared_map.items_in_iteration_order():
        if tier == "BOSS":
            boss_pool.append(relic_id)

    # From purpleRelics (Watcher class-specific)
    for relic_id, tier in purple_map.items_in_iteration_order():
        if tier == "BOSS":
            boss_pool.append(relic_id)

    return boss_pool


def main():
    print("=" * 60)
    print("BOSS RELIC POOL ORDER SIMULATION")
    print("=" * 60)

    # Get pool order
    boss_pool = simulate_boss_pool_order()

    print(f"\nBoss pool before shuffle ({len(boss_pool)} relics):")
    for i, relic in enumerate(boss_pool):
        print(f"  {i:2}: {relic}")

    # Now shuffle with TEST123 seed
    seed = seed_to_long("TEST123")
    rng = Random(seed)

    # Skip 4 randomLong calls (common, uncommon, rare, shop shuffles)
    for _ in range(4):
        rng.random_long()

    boss_shuffle_seed = rng.random_long()
    print(f"\nBoss shuffle seed: {boss_shuffle_seed}")

    # Shuffle
    shuffled = java_collections_shuffle(boss_pool.copy(), boss_shuffle_seed)

    print(f"\nBoss pool after shuffle:")
    for i, relic in enumerate(shuffled[:10]):
        marker = " <-- NEOW BOSS SWAP" if i == 0 else ""
        print(f"  {i:2}: {relic}{marker}")
    print("  ...")

    print(f"\nPredicted Neow boss swap: {shuffled[0]}")
    print(f"Coffee Dripper position: {shuffled.index('Coffee Dripper')}")


if __name__ == "__main__":
    main()
