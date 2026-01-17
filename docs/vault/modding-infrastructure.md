# StS Modding Infrastructure

## Core Stack

| Library | Purpose |
|---------|---------|
| ModTheSpire | Mod loader + bytecode patching |
| BaseMod | High-level API + hooks |
| StSLib | Extended interfaces + damage modifiers |

## SpirePatch (ModTheSpire)

```java
@SpirePatch(clz = AbstractDungeon.class, method = "update")
public class MyPatch {
    @SpirePrefixPatch
    public static SpireReturn<Void> Prefix(AbstractDungeon __instance) {
        if (condition) return SpireReturn.Return();
        return SpireReturn.Continue();
    }

    @SpirePostfixPatch
    public static void Postfix(AbstractDungeon __instance) {
        // After method
    }
}
```

Patch types (order): Insert -> Instrument -> Replace -> Prefix -> Postfix -> Raw

## BaseMod Subscribers

### Combat Hooks
- `OnCardUseSubscriber` - Card played
- `OnPlayerTurnStartSubscriber` - Turn start
- `OnPlayerTurnStartPostDrawSubscriber` - After draw
- `OnPlayerDamagedSubscriber` - Before block, can modify
- `PostBattleSubscriber` - Battle ends
- `OnStartBattleSubscriber` - Battle begins
- `PreMonsterTurnSubscriber` - Before monster, return false skips

### Game Flow
- `PostInitializeSubscriber` - Game ready
- `StartGameSubscriber` - After player gen
- `StartActSubscriber` - New act
- `PostDeathSubscriber` - Player died

### Render/Update
- `PostRenderSubscriber` - Above everything
- `PreRoomRenderSubscriber` - Under player, over background
- `PostUpdateSubscriber` - After updates

## CommunicationMod Protocol

External process via stdin/stdout JSON.

**Handshake:** Process sends "ready\n"

**Commands:**
- `START PlayerClass [Ascension] [Seed]`
- `PLAY CardIndex [TargetIndex]` (1-indexed)
- `END` - End turn
- `POTION Use|Discard Slot [Target]`
- `CHOOSE Index|Name`
- `PROCEED`, `RETURN`, `STATE`, `WAIT`

**State JSON:**
```json
{
  "available_commands": [...],
  "ready_for_command": true,
  "game_state": {
    "combat_state": {
      "player": {"current_hp": 50, "block": 10, "energy": 3},
      "monsters": [...],
      "hand": [...], "draw_pile": [...], "discard_pile": [...]
    },
    "deck": [...], "relics": [...], "potions": [...]
  }
}
```

## Watcher Stance Hooks

```java
@SpirePatch(clz = AbstractPlayer.class, method = "switchedStance")
public class StanceChangePatch {
    public static void Postfix(AbstractPlayer __instance) {
        AbstractStance stance = __instance.stance;
        // CalmStance, WrathStance, DivinityStance, NeutralStance
    }
}
```

Access current: `AbstractDungeon.player.stance.ID`

## Key Game Classes

```java
AbstractDungeon.player                    // Player
AbstractDungeon.getCurrRoom()             // Current room
AbstractDungeon.getCurrRoom().monsters    // Combat monsters
AbstractDungeon.player.hand.group         // Hand
AbstractDungeon.player.drawPile.group     // Draw pile
AbstractDungeon.player.discardPile.group  // Discard
AbstractDungeon.player.exhaustPile.group  // Exhaust
EnergyPanel.totalCount                    // Current energy
AbstractDungeon.actionManager.addToBottom(action) // Queue action
```

## Build Setup

Maven with system-scoped deps:
- `desktop-1.0.jar` (game)
- `ModTheSpire.jar`
- `BaseMod.jar`
- `StSLib.jar` (optional)

Java 8 required (not 9+).

```xml
<properties>
    <maven.compiler.source>1.8</maven.compiler.source>
    <maven.compiler.target>1.8</maven.compiler.target>
</properties>
```

## Utility Mods Reference

- **InfoMod** - Potion/event probabilities
- **Relic Stats** - Per-relic stat tracking
- **MintySpire** - Damage sum, Pen Nib indicator
- **Run History Plus** - Enhanced run logging
