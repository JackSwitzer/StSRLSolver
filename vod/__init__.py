"""
VOD Extraction System for Slay the Spire.

This package provides tools for extracting game state from video recordings
using multi-pass LLM analysis with voting for accuracy.
"""

from vod.state import VODRunState, CombatExtraction, MapPosition, DecisionLog
from vod.tools import TOOLS, get_tool_definitions
from vod.handlers import ToolHandler
from vod.voting import VotingEngine, vote_on_decisions
from vod.chunker import VideoChunker, DynamicChunker, Chunk, ChunkType, ScanResult, DetectedEvent
from vod.verified_extractor import VerifiedExtractor
from vod.verification import SeedVerifier
# Note: orchestrator.py has broken deps (vod.extractor) - disabled until fixed
# from vod.orchestrator import ExtractionOrchestrator, extract_vod

__all__ = [
    # State
    "VODRunState",
    "CombatExtraction",
    "MapPosition",
    "DecisionLog",
    # Tools
    "TOOLS",
    "get_tool_definitions",
    "ToolHandler",
    # Voting
    "VotingEngine",
    "vote_on_decisions",
    # Chunking
    "VideoChunker",
    "DynamicChunker",
    "Chunk",
    "ChunkType",
    "ScanResult",
    "DetectedEvent",
    # Extraction
    "VerifiedExtractor",
    "SeedVerifier",
    # Disabled until orchestrator.py is fixed:
    # "ExtractionOrchestrator",
    # "extract_vod",
]
