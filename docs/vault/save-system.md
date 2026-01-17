# STS Save System Architecture

JSON serialization with XOR obfuscation for deterministic replay.

## File Structure

```
saves/
├── IRONCLAD.autosave        (slot 0)
├── IRONCLAD.autosave.backUp
├── 1_THE_SILENT.autosave    (slot 1)
├── 2_DEFECT.autosave        (slot 2)
└── 3_WATCHER.autosave       (slot 3)
```

## Obfuscation

**Algorithm** (`SaveFileObfuscator.class`):
```java
encode(data, key): Base64(XOR(data, "key"))
decode(data, key): XOR(Base64_Decode(data), "key")
```

**Key**: Hardcoded `"key"` (NOT cryptographic - just obfuscation)

**Detection**: Obfuscated files won't contain `{` character

## SaveFile Structure

### Player State
```java
String name;
int current_health, max_health;
int gold, hand_size;
int red, green, blue;  // Energy
```

### Deck/Relics/Potions
```java
ArrayList<CardSave> cards;           // {id, upgrades, misc}
ArrayList<String> relics;
ArrayList<Integer> relic_counters;
ArrayList<String> potions;
```

### Progression
```java
int floor_num, act_num;
int room_x, room_y;
ArrayList<Integer> path_x, path_y;   // Full path taken
String current_room;                  // Class name
```

### RNG State (Critical for Replay)
```java
long seed;                    // Master seed
int monster_seed_count;       // Monster RNG calls
int event_seed_count;         // Event RNG calls
int merchant_seed_count;      // Shop RNG calls
int card_seed_count;          // Card selection RNG calls
int treasure_seed_count;
int relic_seed_count;
int potion_seed_count;
int ai_seed_count;            // Enemy AI RNG calls
int shuffle_seed_count;       // Deck shuffle RNG calls
int card_random_seed_count;
```

**Determinism**: With seed + all counters, exact game sequence is reproducible.

### Mode Flags
```java
boolean is_ascension_mode;
int ascension_level;
boolean is_endless_mode, is_daily, is_trial;
boolean is_final_act_on;
```

## Save Triggers

```java
enum SaveType {
    ENTER_ROOM,          // Entering any room
    POST_NEOW,           // After Neow choice
    POST_COMBAT,         // After combat, before reward
    AFTER_BOSS_RELIC,    // After boss relic choice
    ENDLESS_NEOW         // Endless mode Neow
}
```

## CardSave Format

```java
public class CardSave {
    public String id;        // "Strike_P"
    public int upgrades;     // Times upgraded
    public int misc;         // Card-specific data
}
```

## Load Flow

```
File → ReadString() → isObfuscated()?
    → Yes: XOR decode + Base64 decode
    → Gson.fromJson() → SaveFile object
```

## Save Flow

```
Game state → HashMap (100+ fields)
    → Gson.toJson() (pretty print)
    → XOR encode + Base64 encode
    → AsyncSaver (separate thread)
    → Write .autosave + .backUp
```

## Error Handling

1. Load failure → try .backUp
2. Both corrupt → move to `sendToDevs/`
3. Async I/O prevents frame drops

## Replay Use Cases

**Create test scenario**:
1. Load SaveFile JSON
2. Modify: RNG counters, HP, deck, position
3. Re-obfuscate and save
4. Game loads modified state

**Deterministic replay**:
1. Load save with specific seed
2. Reset RNG counters to 0
3. Replay exact sequence
