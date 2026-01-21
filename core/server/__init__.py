"""
Combat Search Server for Java ↔ Python integration.

Provides real-time tree search for optimal combat plays by connecting
the Java mod to the Python simulation engine.

Modules:
- search_server: TCP server on port 9998
- protocol: JSON message parsing and validation
- state_converter: JSON → CombatState conversion
- verification: Mismatch detection and logging
"""

from .protocol import SearchRequest, SearchResponse, parse_message, create_response
from .state_converter import json_to_combat_state
from .search_server import CombatSearchServer

__all__ = [
    "CombatSearchServer",
    "SearchRequest",
    "SearchResponse",
    "parse_message",
    "create_response",
    "json_to_combat_state",
]
