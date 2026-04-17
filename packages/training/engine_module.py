"""Helpers for building and loading the Rust PyO3 engine extension in-place."""

from __future__ import annotations

import importlib.machinery
import importlib.util
import os
import subprocess
import sys
from pathlib import Path
from types import ModuleType


ENGINE_MODULE_NAME = "sts_engine"


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _engine_target_path() -> Path:
    override = os.environ.get("STS_ENGINE_EXTENSION_PATH")
    if override:
        return Path(override).expanduser().resolve()
    return _repo_root() / "packages" / "engine-rs" / "target" / "debug" / "libsts_engine.dylib"


def build_engine_extension(*, force: bool = False) -> Path:
    target = _engine_target_path()
    if target.exists() and not force:
        return target

    repo_root = _repo_root()
    env = dict(os.environ)
    rustflags = env.get("RUSTFLAGS", "").strip()
    dynamic_lookup_flags = "-C link-arg=-undefined -C link-arg=dynamic_lookup"
    if dynamic_lookup_flags not in rustflags:
        env["RUSTFLAGS"] = f"{rustflags} {dynamic_lookup_flags}".strip()
    subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            "packages/engine-rs/Cargo.toml",
            "--features",
            "extension-module",
        ],
        cwd=repo_root,
        env=env,
        check=True,
    )
    if not target.exists():
        raise FileNotFoundError(f"expected engine extension at {target}")
    return target


def load_engine_module(
    *,
    build_if_missing: bool = True,
    force_rebuild: bool = False,
    force_reload: bool = False,
) -> ModuleType:
    target = _engine_target_path()
    if force_rebuild:
        target = build_engine_extension(force=True)
    elif not target.exists():
        if not build_if_missing:
            raise FileNotFoundError(f"missing engine extension at {target}")
        target = build_engine_extension()

    existing = None if force_reload else sys.modules.get(ENGINE_MODULE_NAME)
    if existing is not None:
        origin = getattr(existing, "__file__", None)
        if origin == str(target):
            return existing

    if force_reload:
        sys.modules.pop(ENGINE_MODULE_NAME, None)

    loader = importlib.machinery.ExtensionFileLoader(ENGINE_MODULE_NAME, str(target))
    spec = importlib.util.spec_from_loader(ENGINE_MODULE_NAME, loader)
    if spec is None:
        raise ImportError(f"unable to create import spec for {target}")
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    sys.modules[ENGINE_MODULE_NAME] = module
    return module
