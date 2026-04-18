"""Replay a recorded `.run` combat-by-combat against the Rust engine.

Takes a parsed `RecordedRun`, runs PUCT search on each combat with the
recorded entry state (deck/relics/potions/HP), and reports per-combat
`solver_hp_loss` vs `recorded_hp_loss`. Encounters not in the engine's
encounter catalog are marked `unsupported` (skipped, not failed).

Pass criterion: `solver_hp_loss <= recorded_hp_loss + max(base_tol, 0.1 × max_hp)`.

Emits live events to `events.jsonl` (`combat_started`, `combat_solved`,
`combat_failed`, `combat_unsupported`, `combat_error`, `run_complete`)
for SpireMonitor to render in real time.
"""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from .contracts import parse_combat_puct_result
from .encounters import encounter_spec
from .engine_adapter import build_model_evaluator
from .engine_module import load_engine_module
from .inference_service import CombatInferenceService, CombatSearchConfig
from .manifests import (
    GitSnapshot,
    TrainingConfigSnapshot,
    TrainingRunManifest,
)
from .run_logging import TrainingArtifacts, TrainingRunLogger
from .run_parser import RecordedCombatCase, RecordedRun
from .stage2_pipeline import _config_for_room


def _capture_git_branch() -> str:
    """Best-effort current-branch lookup; falls back to 'unknown'."""
    try:
        result = subprocess.check_output(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=Path(__file__).resolve().parents[2],
            text=True,
        )
        return result.strip() or "unknown"
    except Exception:
        return "unknown"


def _result_from_case(
    case: RecordedCombatCase,
    *,
    status: str,
    tolerance: int,
    **extras,
) -> "CombatReplayResult":
    """Build a CombatReplayResult, copying the case-derived fields verbatim."""
    return CombatReplayResult(
        floor=case.floor,
        encounter=case.encounter,
        room_kind=case.room_kind,
        status=status,
        entry_hp=case.entry_hp,
        max_hp=case.max_hp,
        entry_deck=case.entry_deck,
        entry_relics=case.entry_relics,
        entry_potions=case.entry_potions,
        recorded_hp_loss=case.recorded_damage_taken,
        recorded_turns=case.recorded_turns,
        tolerance=tolerance,
        **extras,
    )


@dataclass
class CombatReplayResult:
    floor: int
    encounter: str
    room_kind: str
    status: str  # solved | failed | unsupported | error
    # Human side (from .run)
    entry_hp: int
    max_hp: int
    entry_deck: tuple[str, ...]
    entry_relics: tuple[str, ...]
    entry_potions: tuple[str, ...]
    recorded_hp_loss: int
    recorded_turns: int | None
    # Solver side (from PUCT)
    tolerance: int
    solver_hp_loss: float | None = None
    search_visits: int | None = None
    stop_reason: str | None = None
    error: str | None = None

    def to_dict(self) -> dict:
        out: dict = {
            "floor": self.floor,
            "encounter": self.encounter,
            "room_kind": self.room_kind,
            "status": self.status,
            "entry_hp": self.entry_hp,
            "max_hp": self.max_hp,
            "entry_deck_size": len(self.entry_deck),
            "entry_deck": list(self.entry_deck),
            "entry_relics": list(self.entry_relics),
            "entry_potions": list(self.entry_potions),
            "recorded_hp_loss": self.recorded_hp_loss,
            "tolerance": self.tolerance,
        }
        if self.recorded_turns is not None:
            out["recorded_turns"] = self.recorded_turns
        if self.solver_hp_loss is not None:
            out["solver_hp_loss"] = self.solver_hp_loss
        if self.search_visits is not None:
            out["search_visits"] = self.search_visits
        if self.stop_reason is not None:
            out["stop_reason"] = self.stop_reason
        if self.error is not None:
            out["error"] = self.error
        return out


@dataclass
class RecordedRunReplayReport:
    play_id: str
    seed_played: str
    total_combats: int
    solved: int
    failed: int
    unsupported: int
    error: int
    results: list[CombatReplayResult] = field(default_factory=list)

    def to_dict(self) -> dict:
        return {
            "play_id": self.play_id,
            "seed_played": self.seed_played,
            "total_combats": self.total_combats,
            "solved": self.solved,
            "failed": self.failed,
            "unsupported": self.unsupported,
            "error": self.error,
            "results": [r.to_dict() for r in self.results],
        }


def _compute_tolerance(case: RecordedCombatCase, base: int) -> int:
    return max(base, int(case.max_hp * 0.10))


def _stop_reason_str(reason) -> str:
    return reason.value if hasattr(reason, "value") else str(reason)


def replay_recorded_run(
    run: RecordedRun,
    *,
    output_dir: Path,
    tolerance_base: int = 5,
    checkpoint_path: Path | None = None,
) -> RecordedRunReplayReport:
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    artifacts = TrainingArtifacts(root=output_dir)
    logger = TrainingRunLogger(artifacts)

    manifest = TrainingRunManifest.create(
        run_id=f"recorded-run-{run.play_id}",
        git=GitSnapshot(branch=_capture_git_branch()),
        config=TrainingConfigSnapshot.from_values(
            {
                "mode": "recorded_run_replay",
                "seed_played": run.seed_played,
                "character": run.character,
                "ascension": run.ascension_level,
                "total_combats": len(run.combat_cases),
                "tolerance_base": tolerance_base,
            }
        ),
        tags=("recorded_run", "watcher_a0_combat"),
        notes=(
            f"victory={run.victory}",
            f"floor_reached={run.floor_reached}",
            f"reconstruction_warnings={len(run.reconstruction_warnings)}",
        ),
    )
    logger.write_manifest(manifest)

    logger.append_event(
        "run_started",
        play_id=run.play_id,
        seed_played=run.seed_played,
        character=run.character,
        ascension=run.ascension_level,
        total_combats=len(run.combat_cases),
    )

    from .combat_model import MLXCombatModel

    engine_mod = load_engine_module()
    model = MLXCombatModel(checkpoint_path=str(checkpoint_path) if checkpoint_path else None)
    service = CombatInferenceService.build(model=model, config=CombatSearchConfig(top_k=8))

    counts = {"solved": 0, "failed": 0, "unsupported": 0, "error": 0}
    results: list[CombatReplayResult] = []

    for case in run.combat_cases:
        tolerance = _compute_tolerance(case, tolerance_base)

        try:
            spec = encounter_spec(case.encounter)
        except KeyError:
            result = _result_from_case(
                case,
                status="unsupported",
                tolerance=tolerance,
                error="encounter not in ENCOUNTER_CATALOG",
            )
            results.append(result)
            counts["unsupported"] += 1
            logger.append_event(
                "combat_unsupported",
                floor=case.floor,
                encounter=case.encounter,
                reason="encounter not in ENCOUNTER_CATALOG",
            )
            continue

        logger.append_event(
            "combat_started",
            floor=case.floor,
            encounter=case.encounter,
            room_kind=spec.room_kind,
            entry_hp=case.entry_hp,
            max_hp=case.max_hp,
            entry_deck_size=len(case.entry_deck),
            entry_relics=list(case.entry_relics),
            entry_potions=list(case.entry_potions),
            recorded_hp_loss=case.recorded_damage_taken,
        )

        try:
            engine = engine_mod.RustCombatEngine(
                case.entry_hp,
                case.max_hp,
                3,  # energy
                list(case.entry_deck),
                spec.to_engine_enemies(),
                7_000 + case.floor,
                list(case.entry_relics),
            )
            engine.start_combat()
            snapshot_json = engine.get_combat_snapshot_json()

            solver = engine_mod.CombatSolver.from_snapshot_json(snapshot_json)
            evaluator = build_model_evaluator(
                service,
                metadata_factory=lambda _ctx, _floor=case.floor: {
                    "request_id": f"recorded::{_floor}"
                },
            )
            config = _config_for_room(spec.room_kind, multiplier=1)
            puct_result_payload = solver.run_combat_puct(
                evaluator, json.dumps(config.to_dict())
            )
            parsed = parse_combat_puct_result(puct_result_payload)

            solver_hp_loss = parsed.root_outcome.expected_hp_loss
            status = (
                "solved"
                if solver_hp_loss <= case.recorded_damage_taken + tolerance
                else "failed"
            )

            result = _result_from_case(
                case,
                status=status,
                tolerance=tolerance,
                solver_hp_loss=solver_hp_loss,
                search_visits=parsed.root_total_visits,
                stop_reason=_stop_reason_str(parsed.stop_reason),
            )
            results.append(result)
            counts[status] += 1
            logger.append_event(
                f"combat_{status}",
                floor=case.floor,
                encounter=case.encounter,
                solver_hp_loss=solver_hp_loss,
                recorded_hp_loss=case.recorded_damage_taken,
                tolerance=tolerance,
                search_visits=parsed.root_total_visits,
                stop_reason=result.stop_reason,
            )
        except Exception as exc:  # noqa: BLE001 -- broad catch is intentional; we log+continue
            err_str = f"{type(exc).__name__}: {exc}"
            result = _result_from_case(
                case,
                status="error",
                tolerance=tolerance,
                error=err_str,
            )
            results.append(result)
            counts["error"] += 1
            logger.append_event(
                "combat_error",
                floor=case.floor,
                encounter=case.encounter,
                error=err_str,
            )

    report = RecordedRunReplayReport(
        play_id=run.play_id,
        seed_played=run.seed_played,
        total_combats=len(run.combat_cases),
        solved=counts["solved"],
        failed=counts["failed"],
        unsupported=counts["unsupported"],
        error=counts["error"],
        results=results,
    )

    report_path = output_dir / "recorded_run_replay_report.json"
    report_path.write_text(json.dumps(report.to_dict(), indent=2, sort_keys=True))

    logger.append_event(
        "run_complete",
        solved=counts["solved"],
        failed=counts["failed"],
        unsupported=counts["unsupported"],
        error=counts["error"],
        total_combats=len(run.combat_cases),
    )

    return report


__all__ = [
    "CombatReplayResult",
    "RecordedRunReplayReport",
    "replay_recorded_run",
]


def _print_summary(report: RecordedRunReplayReport) -> None:
    print(f"play_id={report.play_id}  seed={report.seed_played}")
    print(
        f"total={report.total_combats}  solved={report.solved}  "
        f"failed={report.failed}  unsupported={report.unsupported}  error={report.error}"
    )
    print()
    print("Per-combat:")
    for r in report.results:
        loss_str = f"{r.solver_hp_loss:5.1f}" if r.solver_hp_loss is not None else "  -  "
        visits_str = str(r.search_visits) if r.search_visits is not None else "-"
        stop_str = r.stop_reason or "-"
        print(
            f"  F{r.floor:2d}  {r.encounter:30s}  status={r.status:11s}  "
            f"rec_loss={r.recorded_hp_loss:2d}  solver_loss={loss_str}  "
            f"tol={r.tolerance:2d}  visits={visits_str}  stop={stop_str}"
        )


if __name__ == "__main__":
    import sys

    from .run_parser import parse_run_file

    if len(sys.argv) < 3:
        print(
            "usage: python -m packages.training.run_replay <path-to-.run> <output-dir>"
        )
        sys.exit(2)
    parsed_run = parse_run_file(sys.argv[1])
    out = Path(sys.argv[2])
    final_report = replay_recorded_run(parsed_run, output_dir=out)
    _print_summary(final_report)
