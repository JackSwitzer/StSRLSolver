# Slay the Spire Direct Launch (macOS)

## Game Location

```
/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/
```

Key files:
- `desktop-1.0.jar` - Main game
- `ModTheSpire.jar` - Mod loader
- `jre/` - Bundled OpenJDK 1.8.0_252 (x86_64)

## Quick Launch Commands

**Vanilla game:**
```bash
cd "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
./jre/bin/java -Xmx1G -jar desktop-1.0.jar
```

**With mods (for RL training):**
```bash
cd "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
./jre/bin/java -Xmx1G -jar ModTheSpire.jar --skip-launcher --skip-intro --mods basemod,CommunicationMod
```

## ModTheSpire CLI Args

| Flag | Description |
|------|-------------|
| `--skip-launcher` | Skip mod selection UI, use last mods |
| `--skip-intro` | Skip intro splash |
| `--mods <ids>` | Comma-separated mod IDs |
| `--profile <name>` | Use specific mod profile |
| `--debug` | Enable debug mode |

## DRM-Free

Game is DRM-free. `steam_appid.txt` (646570) already present. Works without Steam running.

## Headless Considerations

- libGDX requires display - true headless needs Linux + Xvfb
- macOS: run minimized or use headless Python clone (decapitate-the-spire)
- For training: use CommunicationMod + SuperFastMode + in-game Fast Mode

## Apple Silicon

Bundled JRE is x86_64 (Rosetta 2). For native ARM64:
- Install Azul Zulu/Liberica JDK 8 ARM64
- May need LWJGL3 ARM natives

## RL Training Script

```bash
#!/bin/bash
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
cd "$GAME_DIR"
./jre/bin/java -Xmx1G -Xms512m -jar ModTheSpire.jar \
    --skip-launcher --skip-intro \
    --mods basemod,stslib,CommunicationMod
```
