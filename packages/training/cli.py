"""Tiny CLI for the fresh training rebuild scaffolding."""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict

from .config import TrainingStackConfig
from .corpus import default_watcher_a0_act1_corpus_plan


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Combat-first training rebuild tools")
    parser.add_argument(
        "command",
        choices=("print-default-config", "print-corpus-plan"),
        help="Which scaffold artifact to print",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "print-default-config":
        print(json.dumps(asdict(TrainingStackConfig()), indent=2, sort_keys=True))
        return 0
    if args.command == "print-corpus-plan":
        print(json.dumps(asdict(default_watcher_a0_act1_corpus_plan()), indent=2, sort_keys=True))
        return 0
    raise AssertionError(f"unhandled command: {args.command}")
