"""Push training metrics to a GitHub Gist for dashboard consumption.

Reads status.json from the active training run, formats metrics, and
updates a GitHub Gist via `gh gist edit`.

Usage:
    uv run python scripts/push_metrics.py [--gist-id ID]

The gist ID can also be set via TRAINING_GIST_ID env var.
"""

from __future__ import annotations

import argparse
import json
import logging
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [push_metrics] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("push_metrics")


def gather_metrics() -> dict:
    """Collect metrics from active training run + system stats."""
    metrics = {
        "timestamp": datetime.now().isoformat(),
        "training": {},
        "system": {},
    }

    # Training status
    status_path = Path("logs/active/status.json")
    if status_path.exists():
        try:
            metrics["training"] = json.loads(status_path.read_text())
        except Exception as e:
            logger.warning("Failed to read status.json: %s", e)

    # System stats
    disk = shutil.disk_usage(".")
    metrics["system"]["disk_free_gb"] = round(disk.free / (1024**3), 1)
    metrics["system"]["disk_used_gb"] = round(disk.used / (1024**3), 1)

    # Check if training is alive
    pid_path = Path(".run/training.pid")
    if pid_path.exists():
        try:
            pid = int(pid_path.read_text().strip())
            os.kill(pid, 0)
            metrics["system"]["training_alive"] = True
        except (ValueError, OSError):
            metrics["system"]["training_alive"] = False
    else:
        metrics["system"]["training_alive"] = False

    return metrics


def push_to_gist(metrics: dict, gist_id: str):
    """Update a GitHub Gist with metrics JSON."""
    tmp = Path("/tmp/sts_training_metrics.json")
    tmp.write_text(json.dumps(metrics, indent=2))

    result = subprocess.run(
        ["gh", "gist", "edit", gist_id, "-f", "metrics.json", str(tmp)],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        logger.error("gh gist edit failed: %s", result.stderr.strip())
        return False

    logger.info("Pushed metrics to gist %s", gist_id)
    return True


def main():
    parser = argparse.ArgumentParser(description="Push training metrics to GitHub Gist")
    parser.add_argument("--gist-id", type=str, default=os.environ.get("TRAINING_GIST_ID", ""),
                        help="GitHub Gist ID (or set TRAINING_GIST_ID env var)")
    parser.add_argument("--create", action="store_true", help="Create a new gist if no ID provided")
    args = parser.parse_args()

    metrics = gather_metrics()
    logger.info("Metrics: %d training keys, disk=%.1fGB free, alive=%s",
                len(metrics["training"]), metrics["system"]["disk_free_gb"],
                metrics["system"]["training_alive"])

    if not args.gist_id and args.create:
        tmp = Path("/tmp/sts_training_metrics.json")
        tmp.write_text(json.dumps(metrics, indent=2))
        result = subprocess.run(
            ["gh", "gist", "create", "--public", "-f", "metrics.json", str(tmp)],
            capture_output=True, text=True,
        )
        if result.returncode == 0:
            gist_url = result.stdout.strip()
            gist_id = gist_url.split("/")[-1]
            logger.info("Created gist: %s", gist_url)
            logger.info("Set TRAINING_GIST_ID=%s in ~/.claude/.credentials.env", gist_id)
        else:
            logger.error("Failed to create gist: %s", result.stderr.strip())
        return

    if not args.gist_id:
        logger.info("No gist ID. Use --create to create one, or --gist-id ID")
        print(json.dumps(metrics, indent=2))
        return

    push_to_gist(metrics, args.gist_id)


if __name__ == "__main__":
    main()
