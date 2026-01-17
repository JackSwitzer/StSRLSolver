# Headless ModTheSpire Launch Guide

## The Problem

The Steam Workshop ModTheSpire (v3.6.3) is from December 2018 and lacks:
- `--mods` flag for specifying mods via CLI
- `--skip-launcher` flag (present but non-functional)

This means mods can only be loaded via the GUI launcher popup.

## The Solution

Build ModTheSpire v3.30.3 from the GitHub master branch, which has full CLI support.

### Build Steps

1. **Clone the repo:**
```bash
cd /Users/jackswitzer/Desktop/SlayTheSpireRL
git clone --depth 1 https://github.com/kiooeht/ModTheSpire.git mts-source
```

2. **Copy game JAR for dependency:**
```bash
mkdir -p lib
cp "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/desktop-1.0.jar" lib/
```

3. **Get JDK 8 (required for sun.* internal classes):**
```bash
# Download x64 JDK 8 for macOS (works with Rosetta)
curl -sL "https://api.adoptium.net/v3/binary/latest/8/ga/mac/x64/jdk/hotspot/normal/eclipse" -o /tmp/jdk8.tar.gz
mkdir -p jdk8
tar -xzf /tmp/jdk8.tar.gz -C jdk8 --strip-components=2
```

4. **Build:**
```bash
cd mts-source
JAVA_HOME=/Users/jackswitzer/Desktop/SlayTheSpireRL/jdk8/Home /opt/homebrew/bin/mvn clean package -DskipTests
```

5. **Install:**
```bash
cp _ModTheSpire/ModTheSpire.jar "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/ModTheSpire.jar"
```

## Usage

### Launch with mods (no GUI):
```bash
cd "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
./jre/bin/java -Xmx1G -jar ModTheSpire.jar --mods basemod,stslib,evtracker --skip-intro
```

### Available CLI flags (v3.30.3):
- `--mods MOD_ID1,MOD_ID2,...` - Load specific mods by ID (auto-skips launcher)
- `--skip-launcher` - Skip the mod selection GUI
- `--skip-intro` - Skip the game intro sequence
- `--profile PROFILE_NAME` - Load a saved mod profile
- `--debug` - Enable debug mode
- `--close-when-finished` - Close MTS when game exits
- `--allow-beta` - Allow beta mods

### Mod IDs (from ModTheSpire.json in each JAR):
- BaseMod.jar → `basemod`
- StSLib.jar → `stslib`
- EVTracker.jar → `evtracker`

## Key Files

| File | Purpose |
|------|---------|
| `ModTheSpire.jar` | The launcher (v3.30.3 required for CLI) |
| `mods/*.jar` | Installed mods |
| `~/Library/Preferences/ModTheSpire/mod_lists.json` | Saved mod profiles |

## Verification

Check mods are loading:
```bash
# Look for "Mod list:" section in output
grep -A5 "Mod list:" game_stdout.log

# Expected:
# Mod list:
#  - basemod (5.56.0)
#  - stslib (2.12.0)
#  - evtracker (1.0.0)
```

## Troubleshooting

**"Unknown option: --mods"**
- The installed ModTheSpire is old (v3.6.3). Build from source.

**Rosetta errors**
- Harmless warnings on Apple Silicon. Game still runs.

**No mods loading**
- Check mod IDs match exactly (case-sensitive)
- Verify mods are in `mods/` directory
- Check ModTheSpire version: look for "ModTheSpire (3.30.3)" in output
