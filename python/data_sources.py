"""
Slay the Spire Data Sources

Multiple approaches to collect expert decision data:

1. **Existing Datasets** (easiest)
   - Official data dump: 380GB, 77M+ runs
   - SpireLogs: 50M+ runs with statistics
   - MaT1g3R dataset: Sample runs with JSON

2. **Streamer Run Tracking** (Baalorlord)
   - Spireblight: baalorlord.tv/runs
   - JSON format, programmatic access

3. **VOD Extraction** (most work, highest value)
   - YouTube/Twitch VODs
   - Gemini Vision for frame analysis
   - Expert commentary alignment

4. **Game Save Analysis**
   - Local run history files
   - Run History Plus mod output
"""

import os
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass

# ============ DATA SOURCE CONFIGS ============

@dataclass
class DataSource:
    name: str
    url: str
    format: str
    size: str
    watcher_runs: str
    access_method: str
    notes: str

DATA_SOURCES = {
    "official_dump": DataSource(
        name="Official Slay the Spire Data Dump",
        url="https://www.reddit.com/r/slaythespire/comments/... (search for 'data dump')",
        format="JSON (gzipped)",
        size="380GB compressed, 77M+ runs",
        watcher_runs="~20% = 15M+ Watcher runs",
        access_method="Download from MegaCrit/community hosting",
        notes="Most comprehensive but massive. Filter to Watcher A20 for manageable size."
    ),
    "spirelogs": DataSource(
        name="SpireLogs.com",
        url="https://spirelogs.com/",
        format="Web scraping / mod upload",
        size="50M+ runs",
        watcher_runs="Significant",
        access_method="No public API - use SpireLogMod for card stats",
        notes="Great for aggregate statistics, harder for individual run data."
    ),
    "spirelogs_datasets": DataSource(
        name="SpireLogs Monthly Datasets",
        url="https://drive.google.com/drive/folders/1c7MwTdLxnPgvmPbBEfNWa45YAUU53H0l",
        format="JSON",
        size="Monthly dumps",
        watcher_runs="Varies",
        access_method="Direct Google Drive download",
        notes="Used to train Slay-I (325K fights). Good starting point."
    ),
    "spireblight": DataSource(
        name="Spireblight (Baalorlord's System)",
        url="https://github.com/Spireblight/Spireblight",
        format="JSON in data/ folder",
        size="Baalorlord's runs",
        watcher_runs="All of Baalorlord's documented runs",
        access_method="Clone repo, run client.py, access data/ folder",
        notes="High-quality expert data with exact decisions. MIT licensed."
    ),
    "mat1g3r_dataset": DataSource(
        name="MaT1g3R Slay-the-Spire-data",
        url="https://github.com/MaT1g3R/Slay-the-Spire-data",
        format="JSON",
        size="~500 runs sample",
        watcher_runs="Limited",
        access_method="git clone",
        notes="Small but includes analysis code. Good for testing pipeline."
    ),
    "local_saves": DataSource(
        name="Local Game Saves",
        url="~/Library/Application Support/Steam/steamapps/common/SlayTheSpire/",
        format="Run history JSON (XOR+Base64 encoded)",
        size="Your runs",
        watcher_runs="Your Watcher runs",
        access_method="Read and decode save files",
        notes="Use Run History Plus mod for detailed per-turn data."
    ),
}

# ============ OFFICIAL DATA DUMP ============

def download_official_dump(output_dir: Path) -> Path:
    """
    Instructions for official data dump.

    The dump is too large to auto-download - user must manually fetch.
    """
    print("""
=== Official Slay the Spire Data Dump ===

The official data dump is ~380GB compressed (77M+ runs).

To get it:
1. Search Reddit r/slaythespire for "data dump" or "MegaCrit data"
2. Or contact MegaCrit directly for research access
3. Download and extract to your target directory

For Watcher A20 only, you can filter to reduce size:
- Filter: character_chosen == "WATCHER" AND ascension_level == 20
- Estimated: ~5-10GB for Watcher A20 subset

After downloading, use filter_dump_to_watcher() to extract relevant runs.
""")
    return output_dir

def filter_dump_to_watcher(
    dump_dir: Path,
    output_path: Path,
    ascension: int = 20,
    victory_only: bool = True
) -> int:
    """
    Filter massive dump to Watcher A20 wins only.

    Processes files in streaming fashion to handle large data.
    """
    import gzip

    count = 0
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as out_f:
        out_f.write("[\n")
        first = True

        for file_path in dump_dir.glob("*.json.gz"):
            print(f"Processing {file_path.name}...")

            with gzip.open(file_path, 'rt') as f:
                for line in f:
                    try:
                        run = json.loads(line)

                        # Filter criteria
                        if run.get("character_chosen") != "WATCHER":
                            continue
                        if run.get("ascension_level", 0) < ascension:
                            continue
                        if victory_only and not run.get("victory"):
                            continue

                        # Write to output
                        if not first:
                            out_f.write(",\n")
                        out_f.write(json.dumps(run))
                        first = False
                        count += 1

                        if count % 10000 == 0:
                            print(f"  Found {count} matching runs...")

                    except json.JSONDecodeError:
                        continue

        out_f.write("\n]")

    print(f"Filtered to {count} Watcher A{ascension} {'wins' if victory_only else 'runs'}")
    return count

# ============ SPIREBLIGHT (BAALORLORD) ============

def clone_spireblight(output_dir: Path) -> Path:
    """Clone Spireblight repo to access Baalorlord's run data."""
    repo_path = output_dir / "Spireblight"

    if repo_path.exists():
        print(f"Spireblight already cloned at {repo_path}")
        return repo_path

    cmd = [
        "git", "clone",
        "https://github.com/Spireblight/Spireblight.git",
        str(repo_path)
    ]
    subprocess.run(cmd, check=True)
    return repo_path

def fetch_spireblight_runs(repo_path: Path) -> List[Dict]:
    """
    Read run data from Spireblight data/ folder.

    Note: You may need to run client.py first to sync data.
    """
    data_dir = repo_path / "data"

    if not data_dir.exists():
        print(f"Data directory not found. Run Spireblight client.py to sync.")
        return []

    runs = []
    for json_file in data_dir.glob("*.json"):
        try:
            with open(json_file) as f:
                run = json.load(f)
                runs.append(run)
        except json.JSONDecodeError:
            continue

    print(f"Found {len(runs)} runs in Spireblight data")
    return runs

def scrape_baalorlord_runs(output_path: Path, max_pages: int = 10) -> List[Dict]:
    """
    Scrape run data from baalorlord.tv/runs.

    Returns list of run metadata with links to full data.
    """
    try:
        import requests
        from bs4 import BeautifulSoup
    except ImportError:
        print("Install: pip install requests beautifulsoup4")
        return []

    base_url = "https://baalorlord.tv"
    runs = []

    # Try different profile endpoints
    for profile_id in range(5):
        url = f"{base_url}/profile/{profile_id}/runs"
        print(f"Fetching {url}...")

        try:
            response = requests.get(url, timeout=10)
            if response.status_code != 200:
                continue

            soup = BeautifulSoup(response.text, 'html.parser')
            # Parse run entries - structure depends on actual HTML
            # This is a placeholder - actual parsing needs site inspection

            run_entries = soup.find_all('a', href=lambda x: x and '/run/' in x)
            for entry in run_entries:
                runs.append({
                    "url": base_url + entry['href'],
                    "profile_id": profile_id,
                })

        except Exception as e:
            print(f"Error fetching {url}: {e}")
            continue

    print(f"Found {len(runs)} run links")

    # Save for later detailed fetching
    with open(output_path, 'w') as f:
        json.dump(runs, f, indent=2)

    return runs

# ============ SPIRELOGS DATASETS ============

def download_spirelogs_dataset(output_dir: Path, dataset_name: str = "Monthly_2020_11") -> Path:
    """
    Download SpireLogs monthly dataset from Google Drive.

    Known datasets:
    - Monthly_2020_10
    - Monthly_2020_11

    Note: Requires gdown or manual download from:
    https://drive.google.com/drive/folders/1c7MwTdLxnPgvmPbBEfNWa45YAUU53H0l
    """
    try:
        import gdown
    except ImportError:
        print("Install: pip install gdown")
        print("Or download manually from Google Drive")
        return output_dir

    # Google Drive folder ID
    folder_id = "1c7MwTdLxnPgvmPbBEfNWa45YAUU53H0l"

    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Downloading SpireLogs dataset to {output_dir}...")

    # Download entire folder
    gdown.download_folder(
        f"https://drive.google.com/drive/folders/{folder_id}",
        output=str(output_dir),
        quiet=False
    )

    return output_dir

# ============ MAT1G3R DATASET ============

def clone_mat1g3r_dataset(output_dir: Path) -> Path:
    """Clone MaT1g3R's Slay-the-Spire-data repo."""
    repo_path = output_dir / "Slay-the-Spire-data"

    if repo_path.exists():
        print(f"Dataset already cloned at {repo_path}")
        return repo_path

    cmd = [
        "git", "clone",
        "https://github.com/MaT1g3R/Slay-the-Spire-data.git",
        str(repo_path)
    ]
    subprocess.run(cmd, check=True)
    return repo_path

def load_mat1g3r_runs(repo_path: Path) -> List[Dict]:
    """Load runs from MaT1g3R dataset."""
    results_dir = repo_path / "results"
    runs = []

    if not results_dir.exists():
        print(f"Results directory not found at {results_dir}")
        return runs

    for subdir in results_dir.iterdir():
        if not subdir.is_dir():
            continue

        for json_file in subdir.glob("*.json"):
            try:
                with open(json_file) as f:
                    run = json.load(f)
                    runs.append(run)
            except json.JSONDecodeError:
                continue

    print(f"Loaded {len(runs)} runs from MaT1g3R dataset")
    return runs

# ============ LOCAL SAVES ============

def find_local_saves() -> Path:
    """Find local Slay the Spire save directory."""
    import platform

    system = platform.system()

    if system == "Darwin":  # macOS
        base = Path.home() / "Library/Application Support/Steam/steamapps/common/SlayTheSpire"
        # Check both .app bundle and direct
        paths = [
            base / "SlayTheSpire.app/Contents/Resources/preferences",
            base / "preferences",
        ]
    elif system == "Windows":
        paths = [
            Path.home() / "AppData/Local/SlayTheSpire",
            Path("C:/Program Files (x86)/Steam/steamapps/common/SlayTheSpire"),
        ]
    else:  # Linux
        paths = [
            Path.home() / ".local/share/Steam/steamapps/common/SlayTheSpire",
        ]

    for path in paths:
        if path.exists():
            return path

    print("Could not find Slay the Spire save directory")
    return Path(".")

def decode_save_file(encoded: str) -> Dict:
    """
    Decode XOR+Base64 encoded save data.

    StS uses simple XOR with key "key" then Base64.
    """
    import base64

    # Base64 decode
    decoded_bytes = base64.b64decode(encoded)

    # XOR with "key"
    key = b"key"
    decrypted = bytes([b ^ key[i % len(key)] for i, b in enumerate(decoded_bytes)])

    # Parse JSON
    return json.loads(decrypted.decode('utf-8'))

def load_local_runs(saves_dir: Path) -> List[Dict]:
    """Load and decode local run history."""
    runs = []

    # Look for run history files
    for pattern in ["*RUN*", "*run*", "*.run"]:
        for file_path in saves_dir.glob(pattern):
            try:
                with open(file_path) as f:
                    content = f.read()

                # Try to decode
                if content.startswith('{'):
                    run = json.loads(content)
                else:
                    run = decode_save_file(content)

                runs.append(run)
            except Exception as e:
                continue

    print(f"Loaded {len(runs)} local runs")
    return runs

# ============ UNIFIED LOADER ============

def collect_all_data(output_dir: Path) -> Dict[str, List[Dict]]:
    """
    Collect data from all available sources.

    Returns dict mapping source name to list of runs.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    all_data = {}

    # 1. MaT1g3R (small, quick)
    print("\n=== MaT1g3R Dataset ===")
    try:
        mat1g3r_path = clone_mat1g3r_dataset(output_dir)
        all_data["mat1g3r"] = load_mat1g3r_runs(mat1g3r_path)
    except Exception as e:
        print(f"MaT1g3R failed: {e}")

    # 2. Spireblight (Baalorlord)
    print("\n=== Spireblight (Baalorlord) ===")
    try:
        spireblight_path = clone_spireblight(output_dir)
        all_data["spireblight"] = fetch_spireblight_runs(spireblight_path)
    except Exception as e:
        print(f"Spireblight failed: {e}")

    # 3. Local saves
    print("\n=== Local Saves ===")
    try:
        saves_dir = find_local_saves()
        all_data["local"] = load_local_runs(saves_dir)
    except Exception as e:
        print(f"Local saves failed: {e}")

    # Summary
    print("\n=== Data Collection Summary ===")
    total = 0
    for source, runs in all_data.items():
        print(f"  {source}: {len(runs)} runs")
        total += len(runs)
    print(f"  TOTAL: {total} runs")

    return all_data

# ============ TESTING ============

if __name__ == "__main__":
    print("=== Slay the Spire Data Sources ===\n")

    print("Available data sources:")
    for key, source in DATA_SOURCES.items():
        print(f"\n{source.name}")
        print(f"  URL: {source.url}")
        print(f"  Format: {source.format}")
        print(f"  Size: {source.size}")
        print(f"  Watcher runs: {source.watcher_runs}")
        print(f"  Access: {source.access_method}")

    print("\n" + "="*50)
    print("To collect data, run:")
    print("  python data_sources.py --collect --output ./data")
