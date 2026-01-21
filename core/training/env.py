"""Environment variable loading for API keys."""

import os
from pathlib import Path


def load_env(env_path: Path = None):
    """Load environment variables from .env file.

    Checks in order:
    1. Provided path
    2. Project .env
    3. Central ~/Desktop/Envs/.env
    """
    paths_to_try = []

    if env_path:
        paths_to_try.append(Path(env_path))

    # Project .env
    project_env = Path(__file__).parent.parent.parent / ".env"
    paths_to_try.append(project_env)

    # Central env location
    central_env = Path.home() / "Desktop/Envs/.env"
    paths_to_try.append(central_env)

    for path in paths_to_try:
        if path.exists():
            _load_env_file(path)
            return path

    return None


def _load_env_file(path: Path):
    """Parse and load a .env file."""
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, _, value = line.partition("=")
                key = key.strip()
                value = value.strip().strip('"').strip("'")
                if value and key not in os.environ:
                    os.environ[key] = value


def ensure_api_keys():
    """Ensure API keys are loaded, return status dict."""
    load_env()

    return {
        "openrouter": bool(os.environ.get("OPENROUTER_API_KEY") or
                          os.environ.get("OPEN_ROUTER_API_KEY")),
        "google": bool(os.environ.get("GOOGLE_API_KEY")),
    }


# Auto-load on import
_loaded_path = load_env()
if _loaded_path:
    print(f"[env] Loaded from {_loaded_path}")
