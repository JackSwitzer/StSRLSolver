"""
Training viewer API server.
Reads training log files from a configurable directory and serves them as JSON.
"""
from __future__ import annotations
import json
import os
import time
from pathlib import Path
from typing import Any

from fastapi import FastAPI, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

app = FastAPI(title="Spire Training Viewer")
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_methods=["*"], allow_headers=["*"])


def get_logs_dir() -> Path:
    """Resolve logs directory from env, config file, or default."""
    if env := os.environ.get("SPIRE_LOGS_DIR"):
        return Path(env)
    # Check viz.config.json
    config_path = Path(__file__).parent / "viz.config.json"
    if config_path.exists():
        cfg = json.loads(config_path.read_text())
        if d := cfg.get("logsDir"):
            return Path(d)
    # Check .spire-monitor.json in repo root
    repo_root = Path(__file__).parent.parent.parent
    monitor_cfg = repo_root / ".spire-monitor.json"
    if monitor_cfg.exists():
        cfg = json.loads(monitor_cfg.read_text())
        if d := cfg.get("logsPath"):
            return repo_root / d
    return repo_root / "logs" / "active"


LOGS_DIR = get_logs_dir()

# -- File readers with caching --

_cache: dict[str, tuple[float, Any]] = {}


def read_json_cached(path: Path, max_age: float = 2.0) -> Any:
    key = str(path)
    now = time.time()
    if key in _cache:
        ts, data = _cache[key]
        if now - ts < max_age:
            return data
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text())
        _cache[key] = (now, data)
        return data
    except (json.JSONDecodeError, OSError):
        return None


def read_jsonl_cached(path: Path, max_age: float = 5.0) -> list[dict]:
    key = str(path)
    now = time.time()
    if key in _cache:
        ts, data = _cache[key]
        if now - ts < max_age:
            return data
    if not path.exists():
        return []
    try:
        lines = path.read_text().strip().split("\n")
        data = [json.loads(line) for line in lines if line.strip()]
        _cache[key] = (now, data)
        return data
    except (json.JSONDecodeError, OSError):
        return []


# -- Adapters: convert file format to contract format --


def adapt_status(raw: dict | None) -> dict | None:
    """Adapt status.json (whatever format) to TrainingStatus contract."""
    if not raw:
        return None
    return {
        "timestamp": raw.get("timestamp", ""),
        "elapsedHours": raw.get("elapsed_hours", 0),
        "totalGames": raw.get("total_games", 0),
        "totalWins": raw.get("total_wins", 0),
        "winRate": raw.get("win_rate_100", 0),
        "avgFloor": raw.get("avg_floor_100", 0),
        "peakFloor": raw.get("peak_floor", 0),
        "gamesPerMin": raw.get("games_per_min", 0),
        "trainSteps": raw.get("train_steps", 0),
        "loss": {
            "total": raw.get("total_loss", 0),
            "policy": raw.get("policy_loss", 0),
            "value": raw.get("value_loss", 0),
        },
        "entropy": raw.get("entropy", 0),
        "diagnostics": {
            "explainedVariance": raw.get("explained_variance", 0),
            "klDivergence": raw.get("kl_divergence", 0),
            "meanAdvantage": raw.get("mean_advantage", 0),
            "clipFraction": raw.get("clip_fraction", 0),
        },
        "configName": raw.get("config_name", ""),
        "gpuPercent": raw.get("gpu_percent"),
    }


def adapt_episode(raw: dict) -> dict:
    """Adapt episode JSONL record to Episode contract."""
    combats = []
    for c in raw.get("combats", []):
        turns = []
        for t in c.get("turns_detail", []):
            enemies = []
            for e in t.get("enemies", []):
                enemies.append(
                    {
                        "id": e.get("name", ""),
                        "name": e.get("name", ""),
                        "hp": e.get("hp", 0),
                        "maxHp": e.get("max_hp", 0),
                        "block": e.get("block", 0),
                        "intent": {"kind": e.get("intent", "unknown")},
                    }
                )
            turns.append(
                {
                    "turn": t.get("turn", 0),
                    "cardsPlayed": t.get("cards", []),
                    "energyUsed": 3 - t.get("energy_left", 0),
                    "energyLeft": t.get("energy_left", 0),
                    "playerHp": t.get("player_hp", 0),
                    "playerBlock": t.get("player_block", 0),
                    "stance": (t.get("stance", "neutral") or "neutral").lower(),
                    "enemies": enemies,
                    "handAtEnd": t.get("hand_at_end", []),
                    "unplayedPlayable": t.get("playable_unplayed", 0),
                    "solverScores": t.get("solver_scores", []),
                }
            )
        combats.append(
            {
                "floor": c.get("floor", 0),
                "roomType": c.get("room_type", "monster"),
                "encounterName": c.get("encounter_name", ""),
                "hpBefore": c.get("hp_lost", 0)
                + (turns[-1]["playerHp"] if turns else 0),
                "hpAfter": turns[-1]["playerHp"] if turns else 0,
                "turns": turns,
                "cardsPlayed": c.get("cards_played", 0),
                "potionsUsed": c.get("potions_used", 0),
                "stanceChanges": c.get("stance_changes", 0),
                "durationMs": c.get("duration_ms", 0),
                "solverMs": c.get("solver_ms", 0),
            }
        )

    path_choices = []
    for p in raw.get("path_choices", []):
        options = [
            {
                "x": o.get("x", 0),
                "y": o.get("y", 0),
                "roomType": o.get("room_type", "monster").lower(),
            }
            for o in p.get("options", [])
        ]
        path_choices.append(
            {
                "floor": p.get("floor", 0),
                "options": options,
                "chosen": p.get("chosen", 0),
            }
        )

    # Handle dual formats: episodes.jsonl uses floor/duration_s/deck_final,
    # recent_episodes.json uses floors_reached/duration/deck_changes
    floor = raw.get("floor") or raw.get("floors_reached") or raw.get("death_floor") or 0
    duration_s = raw.get("duration_s") or raw.get("duration") or 0
    hp = raw.get("hp") if raw.get("hp") is not None else raw.get("hp_remaining", 0)
    max_hp = raw.get("max_hp") or raw.get("max_hp", 80)
    deck = raw.get("deck_final") or raw.get("deck", [])
    relics = raw.get("relics_final") or raw.get("relics", [])
    decisions = raw.get("decisions") or raw.get("total_steps") or 0

    return {
        "seed": raw.get("seed", ""),
        "won": raw.get("won", False),
        "floor": floor,
        "hp": hp,
        "maxHp": max_hp,
        "decisions": decisions,
        "durationMs": int(duration_s * 1000),
        "totalReward": raw.get("total_reward", 0),
        "deckFinal": deck,
        "relicsFinal": relics,
        "deathEnemy": raw.get("death_enemy") or None,
        "deathRoom": raw.get("death_room") or None,
        "combats": combats,
        "cardPicks": raw.get("card_picks", []),
        "eventChoices": raw.get("event_choices", []),
        "pathChoices": path_choices,
        "deckTimeline": raw.get("deck_timeline", raw.get("deck_changes", [])),
        "timestamp": raw.get("timestamp", ""),
        "configName": raw.get("config_name", ""),
    }


def adapt_metrics(raw: dict) -> dict:
    return {
        "step": raw.get("train_steps", raw.get("step", 0)),
        "games": raw.get("games", raw.get("total_games", 0)),
        "avgFloor": raw.get("avg_floor", 0),
        "peakFloor": raw.get("peak_floor", 0),
        "winRate": raw.get("win_rate", 0),
        "loss": {
            "total": raw.get("total_loss", 0),
            "policy": raw.get("policy_loss", 0),
            "value": raw.get("value_loss", 0),
        },
        "entropy": raw.get("entropy", 0),
        "timestamp": raw.get("ts", raw.get("timestamp", "")),
    }


def adapt_worker(raw: dict) -> dict:
    return {
        "name": raw.get("name", ""),
        "seed": raw.get("seed", ""),
        "floor": raw.get("floor", 0),
        "phase": (raw.get("phase", "map") or "map").lower(),
        "hp": raw.get("hp", 0),
        "maxHp": raw.get("max_hp", 80),
        "enemy": raw.get("enemy", ""),
        "lastUpdate": int(raw.get("ts", 0) * 1000),
    }


# -- API Routes --


@app.get("/api/status")
def get_status():
    raw = read_json_cached(LOGS_DIR / "status.json")
    return adapt_status(raw) or {}


@app.get("/api/floor-curve")
def get_floor_curve():
    return read_json_cached(LOGS_DIR / "floor_curve.json") or []


@app.get("/api/metrics")
def get_metrics():
    raw = read_jsonl_cached(LOGS_DIR / "metrics_history.jsonl", max_age=10)
    return [adapt_metrics(r) for r in raw]


@app.get("/api/episodes")
def get_episodes(
    limit: int = Query(200, ge=1, le=5000),
    sort: str = Query("floor"),
    desc: bool = Query(True),
):
    # Prefer episodes.jsonl (has full combat detail), fall back to recent_episodes.json
    raw = read_jsonl_cached(LOGS_DIR / "episodes.jsonl", max_age=5)
    if not raw:
        raw = read_json_cached(LOGS_DIR / "recent_episodes.json", max_age=5)
        if isinstance(raw, dict):
            raw = raw.get("episodes", [])
    episodes = [adapt_episode(r) for r in (raw or [])]
    # Sort
    episodes.sort(key=lambda e: e.get(sort, 0), reverse=desc)
    return episodes[:limit]


@app.get("/api/episode/{seed}")
def get_episode(seed: str):
    # Search episodes.jsonl for full detail
    all_eps = read_jsonl_cached(LOGS_DIR / "episodes.jsonl", max_age=5)
    for r in all_eps:
        if r.get("seed") == seed:
            return adapt_episode(r)
    return {"error": "not found"}


@app.get("/api/workers")
def get_workers():
    workers_dir = LOGS_DIR / "workers"
    if not workers_dir.exists():
        return []
    result = []
    for f in workers_dir.glob("*.json"):
        raw = read_json_cached(f, max_age=2)
        if raw:
            result.append(adapt_worker(raw))
    return result


@app.get("/api/runs")
def get_runs():
    runs_dir = (
        LOGS_DIR.parent / "runs" if LOGS_DIR.name == "active" else LOGS_DIR.parent
    )
    if not runs_dir.exists():
        return []
    return sorted(
        [
            d.name
            for d in runs_dir.iterdir()
            if d.is_dir() and d.name.startswith("run_")
        ],
        reverse=True,
    )


# -- v2 format detection --


@app.get("/api/format")
def get_format():
    has_manifest = (LOGS_DIR / "manifest.json").exists()
    has_status = (LOGS_DIR / "status.json").exists()
    return {"format": "v2" if has_manifest else "v1" if has_status else "empty"}


# -- v2 artifact endpoints --


def _snake_to_camel(s: str) -> str:
    parts = s.split("_")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def _adapt_keys(obj: Any) -> Any:
    if isinstance(obj, dict):
        return {_snake_to_camel(k): _adapt_keys(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [_adapt_keys(i) for i in obj]
    return obj


@app.get("/api/manifest")
def get_manifest():
    raw = read_json_cached(LOGS_DIR / "manifest.json")
    if not raw:
        return {}
    return _adapt_keys(raw)


@app.get("/api/events")
def get_events():
    raw = read_jsonl_cached(LOGS_DIR / "events.jsonl", max_age=2)
    return [_adapt_keys(r) for r in raw]


@app.get("/api/training-metrics")
def get_training_metrics():
    raw = read_jsonl_cached(LOGS_DIR / "metrics.jsonl", max_age=5)
    return [_adapt_keys(r) for r in raw]


@app.get("/api/benchmark")
def get_benchmark():
    raw = read_json_cached(LOGS_DIR / "benchmark_report.json")
    if not raw:
        return {"slices": []}
    return _adapt_keys(raw)


@app.get("/api/frontier")
def get_frontier():
    raw = read_json_cached(LOGS_DIR / "frontier_report.json")
    if not raw:
        return {"points": [], "frontier": [], "ranking": [], "groups": []}
    return _adapt_keys(raw)


@app.get("/api/corpus-matrix")
def get_corpus_matrix():
    raw = read_jsonl_cached(LOGS_DIR / "metrics.jsonl", max_age=10)
    cells: dict[tuple[str, str], dict] = {}
    for m in raw:
        deck = m.get("deck_family") or "unknown"
        enemy = m.get("enemy") or "unknown"
        name = m.get("name", "")
        value = m.get("value", 0)
        key = (deck, enemy)
        if key not in cells:
            cells[key] = {"deck_family": deck, "enemy": enemy, "count": 0, "_solve": 0, "_hp": 0, "_turns": 0}
        c = cells[key]
        if name == "solve_probability":
            c["_solve"] += value
            c["count"] += 1
        elif name == "expected_hp_loss":
            c["_hp"] += value
        elif name == "expected_turns":
            c["_turns"] += value
    result = []
    for c in cells.values():
        n = max(c["count"], 1)
        result.append({
            "deckFamily": c["deck_family"],
            "enemy": c["enemy"],
            "solveRate": c["_solve"] / n,
            "avgHpLoss": c["_hp"] / n,
            "avgTurns": c["_turns"] / n,
            "count": c["count"],
        })
    return result


# Serve built frontend if dist/ exists — with SPA fallback for client-side routes
dist = Path(__file__).parent / "dist"
if dist.exists():
    from fastapi.responses import FileResponse

    app.mount("/assets", StaticFiles(directory=str(dist / "assets")), name="assets")

    @app.get("/{full_path:path}")
    def spa_fallback(full_path: str):
        # Serve index.html for all non-API routes (SPA client-side routing)
        return FileResponse(str(dist / "index.html"))
