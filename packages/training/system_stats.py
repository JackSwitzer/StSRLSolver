"""Best-effort host/process system stats for training runs."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import os
from pathlib import Path
import re
import subprocess
from typing import Any

import psutil


_GPU_RESIDENCY_RE = re.compile(r"GPU active residency:\s*([0-9]+(?:\.[0-9]+)?)%")


@dataclass(frozen=True)
class SystemStatsSnapshot:
    timestamp: str
    phase: str
    step: int | None
    process_pid: int
    process_cpu_percent: float
    process_rss_gb: float
    process_memory_percent: float
    host_cpu_percent: float
    host_memory_used_gb: float
    host_memory_total_gb: float
    host_memory_percent: float
    host_swap_used_gb: float
    host_swap_percent: float
    gpu_percent: float | None
    gpu_sampler: str
    gpu_status: str
    note: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def _bytes_to_gb(value: int | float) -> float:
    return float(value) / float(1024**3)


def _sample_gpu_percent() -> tuple[float | None, str, str]:
    if os.uname().sysname != "Darwin":
        return None, "unsupported", "gpu sampler only implemented for macOS"

    try:
        result = subprocess.run(
            ["powermetrics", "-n", "1", "--samplers", "gpu_power"],
            capture_output=True,
            text=True,
            timeout=2.5,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return None, "powermetrics", "powermetrics timed out"
    except Exception as exc:  # pragma: no cover - defensive path
        return None, "powermetrics", f"powermetrics failed: {exc}"

    combined = "\n".join(part for part in (result.stdout, result.stderr) if part).strip()
    if "must be invoked as the superuser" in combined:
        return None, "powermetrics", "powermetrics requires superuser"

    match = _GPU_RESIDENCY_RE.search(combined)
    if match:
        return float(match.group(1)), "powermetrics", "ok"
    if result.returncode == 0:
        return None, "powermetrics", "powermetrics returned no GPU residency field"
    return None, "powermetrics", combined or f"powermetrics exited {result.returncode}"


class SystemStatsSampler:
    """Sample process and host stats for long-running training stages."""

    def __init__(self, pid: int | None = None) -> None:
        self.process = psutil.Process(pid or os.getpid())
        psutil.cpu_percent(interval=None)
        self.process.cpu_percent(interval=None)

    def sample(
        self,
        *,
        phase: str,
        step: int | None = None,
        note: str | None = None,
    ) -> SystemStatsSnapshot:
        host_cpu = float(psutil.cpu_percent(interval=None))
        process_cpu = float(self.process.cpu_percent(interval=None))
        process_memory = self.process.memory_info()
        virtual_memory = psutil.virtual_memory()
        swap_memory = psutil.swap_memory()
        gpu_percent, gpu_sampler, gpu_status = _sample_gpu_percent()

        return SystemStatsSnapshot(
            timestamp=datetime.now(timezone.utc).isoformat(),
            phase=phase,
            step=step,
            process_pid=int(self.process.pid),
            process_cpu_percent=process_cpu,
            process_rss_gb=_bytes_to_gb(process_memory.rss),
            process_memory_percent=float(self.process.memory_percent()),
            host_cpu_percent=host_cpu,
            host_memory_used_gb=_bytes_to_gb(virtual_memory.used),
            host_memory_total_gb=_bytes_to_gb(virtual_memory.total),
            host_memory_percent=float(virtual_memory.percent),
            host_swap_used_gb=_bytes_to_gb(swap_memory.used),
            host_swap_percent=float(swap_memory.percent),
            gpu_percent=gpu_percent,
            gpu_sampler=gpu_sampler,
            gpu_status=gpu_status,
            note=note,
        )


def write_system_stats_snapshot(path: Path, snapshot: SystemStatsSnapshot) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(f"{snapshot.to_dict()!r}\n")
