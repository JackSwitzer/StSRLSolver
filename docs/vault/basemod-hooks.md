# BaseMod Hooks Reference

Complete subscriber interface reference for STS mod development.

## Subscription Pattern

```java
@SpireInitializer
public class MyMod implements PostInitializeSubscriber {
    public static void initialize() { new MyMod(); }

    public MyMod() {
        BaseMod.subscribe(this);
    }

    @Override
    public void receivePostInitialize() {
        // Handle event
    }
}
```

## Combat Hooks (Priority for EV Tracking)

### OnCardUseSubscriber
```java
void receiveCardUsed(AbstractCard card)
```
- **Fires**: When card played (before exhaust)
- **Access**: card.damage, card.block, card.costForTurn

### OnPlayerTurnStartSubscriber
```java
void receiveOnPlayerTurnStart()
```
- **Fires**: Start of turn, BEFORE draw
- **Access**: Hand before draw, energy, monsters

### OnPlayerTurnStartPostDrawSubscriber
```java
void receiveOnPlayerTurnStartPostDraw()
```
- **Fires**: After draw completes
- **Access**: Actual hand after draw
- **Critical**: Use this for hand state logging

### OnPlayerDamagedSubscriber
```java
int receiveOnPlayerDamaged(int damage, DamageInfo info)
```
- **Fires**: Before block applied
- **Return**: Can modify damage amount
- **Access**: info.type, info.owner

### OnStartBattleSubscriber
```java
void receiveOnStartBattle(AbstractRoom room)
```
- **Fires**: Battle begins
- **Access**: AbstractDungeon.getMonsters().monsters

### PostBattleSubscriber
```java
void receivePostBattle(AbstractRoom room)
```
- **Fires**: Battle ends (victory ONLY)
- **Note**: Does NOT fire on death

### PreMonsterTurnSubscriber
```java
boolean receivePreMonsterTurn(AbstractMonster monster)
```
- **Return**: false to skip monster's turn

## Game Flow Hooks

### StartGameSubscriber
```java
void receiveStartGame()
```
- **Fires**: After dungeon/player creation
- **Access**: CardCrawlGame.chosenCharacter, Settings.seed

### PostDungeonInitializeSubscriber
```java
void receivePostDungeonInitialize()
```
- **Fires**: Dungeon layout created
- **Access**: AbstractDungeon.player.masterDeck

### PostDeathSubscriber
```java
void receivePostDeath()
```
- **Fires**: Player dies or abandons
- **Use**: Log final state, run end

### StartActSubscriber
```java
void receiveStartAct()
```
- **Fires**: New act begins

## Power/Relic Hooks

### OnPowersModifiedSubscriber
```java
void receiveOnPowersModified()
```
- **Fires**: Any power applied/removed
- **Note**: Fires for both player AND enemies

### PostPowerApplySubscriber
```java
void receivePostPowerApply(AbstractPower power, AbstractCreature target, AbstractCreature source)
```
- **Fires**: Power applied via action

### RelicGetSubscriber
```java
void receiveRelicGet(AbstractRelic relic)
```
- **Fires**: Relic obtained (not starting relics)

## Render Hooks

### PostRenderSubscriber
```java
void receivePostRender(SpriteBatch sb)
```
- **Fires**: After all game rendering
- **Use**: Overlay custom UI, EV panel

```java
// Check if in combat
if (AbstractDungeon.getCurrRoom().phase == AbstractRoom.RoomPhase.COMBAT) {
    FontHelper.renderFontLeft(sb, FontHelper.tipBodyFont, text, x, y, Color.WHITE);
}
```

## Key Game State Access

```java
AbstractDungeon.player              // Player instance
AbstractDungeon.player.hand.group   // Hand cards
AbstractDungeon.player.drawPile     // Draw pile
AbstractDungeon.player.discardPile  // Discard pile
AbstractDungeon.player.stance       // Current stance
AbstractDungeon.getCurrRoom()       // Current room
AbstractDungeon.getMonsters()       // Monster group
AbstractDungeon.floorNum            // Current floor
EnergyPanel.totalCount              // Available energy
```

## EVTracker Currently Uses

```java
implements
    PostInitializeSubscriber,        // Init logging
    OnCardUseSubscriber,              // Card play tracking
    OnPlayerTurnStartSubscriber,      // Turn state (pre-draw)
    OnPlayerDamagedSubscriber,        // Damage tracking
    PostBattleSubscriber,             // Battle results
    OnStartBattleSubscriber,          // Battle setup
    StartGameSubscriber,              // Run initialization
    PostDeathSubscriber,              // Run end
    PostDungeonInitializeSubscriber,  // Dungeon setup
    PostRenderSubscriber              // EV overlay
```

## Missing Critical Hooks

**Add to EVTracker**:
- `OnPlayerTurnStartPostDrawSubscriber` - Actual hand after draw
- `StartActSubscriber` - Act boundaries
- `PreMonsterTurnSubscriber` - Monster intents before execution
- Custom SpirePatch for stance changes
