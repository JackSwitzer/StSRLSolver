# Watcher Stance Mechanics

Complete reference for Watcher stance system from game decompilation.

## Stance Summary

| Stance | Damage Dealt | Damage Taken | On Enter | On Exit |
|--------|-------------|--------------|----------|---------|
| Neutral | x1 | x1 | - | - |
| Calm | x1 | x1 | - | +2 Energy |
| Wrath | x2 | x2 | - | - |
| Divinity | x3 | x1 | +3 Energy | Auto-exit at turn start |

## Calm Stance

**Effect**: Exit grants +2 energy

**Code** (`CalmStance.class`):
```java
@Override
public void onExitStance() {
    this.addToBot(new GainEnergyAction(2));
}
```

**Strategy**: Use for energy banking. Enter Calm on safe turns, exit to Wrath on lethal turns.

## Wrath Stance

**Effect**: 2x damage dealt AND taken (NORMAL damage only)

**Code** (`WrathStance.class`):
```java
@Override
public float atDamageGive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        return damage * 2.0f;
    }
    return damage;
}

@Override
public float atDamageReceive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        return damage * 2.0f;
    }
    return damage;
}
```

**Strategy**: Enter only when:
- Can kill all enemies this turn
- Enemies not attacking
- Have enough block to survive doubled damage

## Divinity Stance

**Effect**: 3x damage dealt, +3 energy on enter, auto-exits at start of next turn

**Code** (`DivinityStance.class`):
```java
@Override
public void onEnterStance() {
    this.addToBot(new GainEnergyAction(3));
}

@Override
public float atDamageGive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        return damage * 3.0f;
    }
    return damage;
}

@Override
public void atStartOfTurn() {
    this.addToBot(new ChangeStanceAction("Neutral"));
}
```

## Mantra System

**Threshold**: 10 Mantra = automatic Divinity entry

**Code** (`MantraPower.class`):
```java
@Override
public void stackPower(int stackAmount) {
    this.amount += stackAmount;
    if (this.amount >= 10) {
        this.addToBot(new ChangeStanceAction("Divinity"));
        this.amount -= 10;
        if (this.amount <= 0) {
            this.addToBot(new RemoveSpecificPowerAction(this.owner, this.owner, POWER_ID));
        }
    }
}
```

**Mantra Cards**:
- Pray: +3 Mantra (+4 upgraded)
- Worship: +5 Mantra
- Prostrate: +2 Mantra (+3 upgraded) + 4 block

## Blasphemy (Death Mechanic)

**Effect**: Enter Divinity + die at start of next turn

**Code** (`Blasphemy.class`):
```java
@Override
public void use(AbstractPlayer p, AbstractMonster m) {
    this.addToBot(new ChangeStanceAction("Divinity"));
    this.addToBot(new ApplyPowerAction(p, p, new EndTurnDeathPower(p)));
}
```

**EndTurnDeathPower** executes at turn start:
```java
@Override
public void atStartOfTurn() {
    this.addToBot(new LoseHPAction(this.owner, this.owner, 99999));
}
```

**Survival**: Must win combat before next turn starts.

## Stance Change Action Flow

**Code** (`ChangeStanceAction.class`):
```
1. Check CannotChangeStancePower → abort if present
2. Validate new stance differs from current
3. Call onChangeStance() on all powers
4. Call onChangeStance() on all relics
5. Execute oldStance.onExitStance()
6. Set player.stance = newStance
7. Execute newStance.onEnterStance()
8. Track in uniqueStancesThisCombat
9. Call player.switchedStance()
```

## Key Stance Cards

**Enter Wrath**:
- Eruption: 2 cost (1 upgraded), 9 damage + Wrath
- Tantrum: 1 cost, 3x3 damage + Wrath + shuffle back

**Enter Calm**:
- Vigilance: 2 cost, 8 block (12 upgraded) + Calm

**Exit to Neutral**:
- Inner Peace: If in Calm → draw 3, else → Calm
- Empty Fist: 1 cost, 9 damage, exit stance

## EV Considerations

1. **Calm Exit Value**: +2 energy = ~6-8 damage equivalent
2. **Wrath Risk**: Doubled incoming damage must be weighed against kill potential
3. **Divinity Window**: 3x damage + 3 energy is massive but timing critical
4. **Mantra Tracking**: Predict Divinity entry by tracking Mantra counter
