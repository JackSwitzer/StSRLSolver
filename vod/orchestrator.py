"""
Main extraction orchestrator coordinating the full VOD extraction pipeline.

Supports both static and dynamic (scan-first) chunking strategies.

Coordinates:
1. Optional scan pass (with Pro model) to identify event boundaries
2. Dynamic chunk creation based on scan results
3. Multi-pass extraction per chunk (with Flash model)
4. Voting across passes
5. State handoff between chunks
6. Seed verification
"""

import json
import os
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, Literal

from vod.state import VODRunState, DecisionLog
from vod.tools import TOOLS
from vod.handlers import ToolHandler
from vod.voting import VotingEngine, VotingResult
from vod.chunker import (
    VideoChunker, DynamicChunker, Chunk, ScanResult,
    create_chunk_prompt, SCAN_PROMPT, SCAN_TOOL
)
from vod.micro_chunker import (
    MicroChunker, MicroChunk, create_micro_chunk_prompt,
    BOUNDARY_SCAN_PROMPT, parse_boundary_scan
)
from vod.extractor import GeminiExtractor, ExtractionResult, create_extractor
from vod.verification import SeedVerifier, SeedVerificationReport, verify_extraction


@dataclass
class ChunkExtractionResult:
    """Result of extracting a single chunk with multiple passes."""
    chunk: Chunk
    passes: list[ExtractionResult]
    voted_result: Optional[VotingResult] = None
    state_after: Optional[dict] = None
    duration_seconds: float = 0.0

    def to_dict(self) -> dict:
        return {
            "chunk": self.chunk.to_dict(),
            "pass_count": len(self.passes),
            "voted_decisions": self.voted_result.to_dict() if self.voted_result else None,
            "duration_seconds": self.duration_seconds,
        }


@dataclass
class FullExtractionResult:
    """Complete result of extracting an entire VOD."""
    video_id: str
    video_url: str
    state: VODRunState
    chunk_results: list[ChunkExtractionResult]
    scan_result: Optional[ScanResult] = None
    verification: Optional[SeedVerificationReport] = None
    overall_confidence: float = 0.0
    total_decisions: int = 0
    duration_seconds: float = 0.0
    extracted_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def to_dict(self) -> dict:
        return {
            "video_id": self.video_id,
            "video_url": self.video_url,
            "extracted_at": self.extracted_at,
            "state": self.state.to_dict(),
            "scan_result": {
                "events": [
                    {"type": e.event_type, "floor": e.floor, "start": e.timestamp_start, "end": e.timestamp_end}
                    for e in self.scan_result.events
                ],
                "final_floor": self.scan_result.final_floor,
                "victory": self.scan_result.victory,
            } if self.scan_result else None,
            "chunk_results": [c.to_dict() for c in self.chunk_results],
            "verification": self.verification.to_dict() if self.verification else None,
            "overall_confidence": self.overall_confidence,
            "total_decisions": self.total_decisions,
            "duration_seconds": self.duration_seconds,
        }

    def save(self, output_dir: str = "data/vod_extractions") -> str:
        """Save extraction result to JSON file."""
        os.makedirs(output_dir, exist_ok=True)
        output_path = os.path.join(output_dir, f"{self.video_id}_extraction.json")

        with open(output_path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)

        return output_path


ChunkingStrategy = Literal["static", "dynamic", "micro"]


class ExtractionOrchestrator:
    """
    Orchestrates the full VOD extraction pipeline.

    Supports two chunking strategies:
    - static: Fixed floor-based chunks (fast, no scan pass)
    - dynamic: Scan-first to identify event boundaries (more accurate)

    Usage:
        # Dynamic chunking (recommended)
        orchestrator = ExtractionOrchestrator(chunking_strategy="dynamic")
        result = orchestrator.extract(video_path="/path/to/video.mp4", video_id="abc123")

        # Static chunking (faster, no Pro model needed)
        orchestrator = ExtractionOrchestrator(chunking_strategy="static")
        result = orchestrator.extract(...)
    """

    def __init__(
        self,
        extractor: Optional[GeminiExtractor] = None,
        scan_extractor: Optional[GeminiExtractor] = None,
        passes_per_chunk: int = 3,
        voting_threshold: float = 0.5,
        chunking_strategy: ChunkingStrategy = "dynamic",
        extraction_model: str = "gemini-3-flash-preview",
        scan_model: str = "gemini-3-flash-preview",
        use_mock: bool = False,
    ):
        """
        Initialize the orchestrator.

        Args:
            extractor: Extractor for chunk extraction (uses extraction_model if not provided)
            scan_extractor: Extractor for scan pass (uses scan_model if not provided)
            passes_per_chunk: Number of extraction passes per chunk
            voting_threshold: Confidence threshold for flagging decisions
            chunking_strategy: "static" or "dynamic"
            extraction_model: Model for chunk extraction
            scan_model: Model for scan pass (can be Pro for better accuracy)
            use_mock: Use mock extractors for testing
        """
        self.passes_per_chunk = passes_per_chunk
        self.voting_threshold = voting_threshold
        self.chunking_strategy = chunking_strategy
        self.use_mock = use_mock

        # Create extractors
        if use_mock:
            self.extractor = create_extractor(use_mock=True)
            self.scan_extractor = create_extractor(use_mock=True)
        else:
            self.extractor = extractor or create_extractor(model_name=extraction_model)
            self.scan_extractor = scan_extractor or create_extractor(model_name=scan_model)

        # Chunkers
        self.static_chunker = VideoChunker()
        self.dynamic_chunker = DynamicChunker()
        self.micro_chunker: Optional[MicroChunker] = None  # Created with transcript path

    def extract(
        self,
        video_path: str,
        video_id: str,
        video_url: str = "",
        seed: Optional[str] = None,
        final_floor: Optional[int] = None,
        checkpoint_dir: Optional[str] = None,
        transcript_path: Optional[str] = None,
        video_duration_seconds: Optional[float] = None,
    ) -> FullExtractionResult:
        """
        Extract decisions from an entire VOD.

        Args:
            video_path: Path to video file
            video_id: Unique identifier for the video
            video_url: Original URL (optional)
            seed: Known seed (optional)
            final_floor: Last floor reached (optional, detected from scan)
            checkpoint_dir: Directory to save checkpoints
            transcript_path: Path to transcript JSON (for micro chunking)
            video_duration_seconds: Total video length (for micro chunking)

        Returns:
            Complete extraction result
        """
        import time
        start_time = time.time()

        # Initialize state
        state = VODRunState.create(
            video_id=video_id,
            video_url=video_url,
            seed=seed,
        )

        scan_result = None
        chunks = []
        micro_chunks = []

        # Get chunks based on strategy
        if self.chunking_strategy == "micro":
            # Micro-chunking: Use transcript for fine-grained boundaries
            if not transcript_path:
                print("Warning: micro chunking requested but no transcript provided")
                print("Falling back to static chunking")
                final_floor = final_floor or 56
                chunks = self.static_chunker.get_chunks(final_floor)
                chunks = self.static_chunker.estimate_timestamps(chunks)
            else:
                print("Creating micro-chunks from transcript...")
                self.micro_chunker = MicroChunker(
                    transcript_path=transcript_path,
                    min_chunk_seconds=20,
                    max_chunk_seconds=90,  # Keep chunks small
                    target_actions_per_chunk=2,
                    padding_seconds=5,
                )

                # Get video duration if not provided
                duration = video_duration_seconds
                if not duration:
                    import subprocess
                    try:
                        result = subprocess.run(
                            ["ffprobe", "-v", "error", "-show_entries",
                             "format=duration", "-of", "csv=p=0", video_path],
                            capture_output=True, text=True
                        )
                        duration = float(result.stdout.strip())
                    except Exception:
                        print("Warning: Could not detect video duration, using 55 min default")
                        duration = 55 * 60

                micro_chunks = self.micro_chunker.create_chunks(duration)
                print(f"Created {len(micro_chunks)} micro-chunks from transcript")

        elif self.chunking_strategy == "dynamic":
            print("Running scan pass to identify events...")
            scan_result = self._run_scan_pass(video_path)

            if scan_result and scan_result.events:
                print(f"Scan found {len(scan_result.events)} events, final floor: {scan_result.final_floor}")
                chunks = self.dynamic_chunker.create_chunks(scan_result)

                # Update state with scan results
                if scan_result.seed and not seed:
                    state.run.seed_string = scan_result.seed
                    state.seed_detected = True
            else:
                print("Scan pass returned no events, falling back to static chunking")
                final_floor = final_floor or 56
                chunks = self.static_chunker.get_chunks(final_floor)
                chunks = self.static_chunker.estimate_timestamps(chunks)
        else:
            # Static chunking
            final_floor = final_floor or 56
            chunks = self.static_chunker.get_chunks(final_floor)
            chunks = self.static_chunker.estimate_timestamps(chunks)

        # Handle micro-chunks vs regular chunks
        if micro_chunks:
            print(f"Extracting {len(micro_chunks)} micro-chunks")
        else:
            print(f"Created {len(chunks)} chunks for extraction")

        chunk_results = []

        # Process micro-chunks if available
        if micro_chunks:
            for i, mc in enumerate(micro_chunks):
                expected = [a.action_type.value for a in mc.expected_actions]
                print(f"\nChunk {i+1}/{len(micro_chunks)}: {mc.start_str}-{mc.end_str} "
                      f"({mc.duration_seconds:.0f}s) [{', '.join(expected)}]")

                # Extract micro-chunk
                chunk_result = self._extract_micro_chunk(
                    video_path=video_path,
                    micro_chunk=mc,
                    state=state,
                )
                chunk_results.append(chunk_result)

                # Apply voted decisions to state
                if chunk_result.voted_result:
                    self._apply_decisions_to_state(
                        state=state,
                        decisions=[d.to_decision_log() for d in chunk_result.voted_result.decisions],
                    )
                    print(f"  Applied {len(chunk_result.voted_result.decisions)} decisions")

                chunk_result.state_after = state.snapshot()
        else:
            # Process regular chunks
            for i, chunk in enumerate(chunks):
                print(f"\nExtracting chunk {i+1}/{len(chunks)}: {chunk}")

                # Extract chunk with multiple passes
                chunk_result = self._extract_chunk(
                    video_path=video_path,
                    chunk=chunk,
                    state=state,
                )
                chunk_results.append(chunk_result)

                # Apply voted decisions to state
                if chunk_result.voted_result:
                    self._apply_decisions_to_state(
                        state=state,
                        decisions=[d.to_decision_log() for d in chunk_result.voted_result.decisions],
                    )
                    print(f"  Applied {len(chunk_result.voted_result.decisions)} decisions, "
                          f"confidence: {chunk_result.voted_result.overall_confidence:.1%}")

                # Save checkpoint if requested
                if checkpoint_dir:
                    self._save_checkpoint(
                        checkpoint_dir=checkpoint_dir,
                        video_id=video_id,
                        chunk_index=i,
                        state=state,
                        chunk_result=chunk_result,
                    )

                chunk_result.state_after = state.snapshot()

        # Run seed verification if seed is known
        verification = None
        if state.seed_detected:
            verification = verify_extraction(state)

        # Calculate overall metrics
        total_decisions = sum(
            len(cr.voted_result.decisions) if cr.voted_result else 0
            for cr in chunk_results
        )

        confidences = [
            cr.voted_result.overall_confidence
            for cr in chunk_results
            if cr.voted_result
        ]
        overall_confidence = sum(confidences) / len(confidences) if confidences else 0.0

        duration = time.time() - start_time

        return FullExtractionResult(
            video_id=video_id,
            video_url=video_url,
            state=state,
            chunk_results=chunk_results,
            scan_result=scan_result,
            verification=verification,
            overall_confidence=overall_confidence,
            total_decisions=total_decisions,
            duration_seconds=duration,
        )

    def _run_scan_pass(self, video_path: str) -> Optional[ScanResult]:
        """Run the scan pass to identify events."""
        try:
            result = self.scan_extractor.extract_from_video(
                video_path=video_path,
                prompt=SCAN_PROMPT,
            )

            if result.error:
                print(f"Scan pass error: {result.error}")
                return None

            return self.dynamic_chunker._parse_scan_result(result.tool_calls)

        except Exception as e:
            print(f"Scan pass failed: {e}")
            return None

    def extract_single_chunk(
        self,
        video_path: str,
        chunk: Chunk,
        state: Optional[VODRunState] = None,
    ) -> ChunkExtractionResult:
        """Extract a single chunk (for testing or resuming)."""
        if state is None:
            state = VODRunState.create(
                video_id=Path(video_path).stem,
            )

        return self._extract_chunk(video_path, chunk, state)

    def _extract_chunk(
        self,
        video_path: str,
        chunk: Chunk,
        state: VODRunState,
    ) -> ChunkExtractionResult:
        """Extract a chunk with multiple passes and voting."""
        import time
        start_time = time.time()

        # Create prompt for this chunk
        prompt = create_chunk_prompt(chunk, state.snapshot())

        # Run multiple extraction passes
        passes: list[ExtractionResult] = []
        for pass_num in range(self.passes_per_chunk):
            state.current_pass = pass_num + 1
            result = self.extractor.extract_chunk(
                video_path=video_path,
                chunk_prompt=prompt,
                start_time=chunk.start_time,
                end_time=chunk.end_time,
            )
            passes.append(result)

            # Add pass number to decisions
            for decision in result.decisions:
                decision.pass_number = pass_num + 1

            print(f"  Pass {pass_num + 1}: {len(result.decisions)} decisions")

        # Vote across passes
        voting_engine = VotingEngine(confidence_threshold=self.voting_threshold)
        for pass_result in passes:
            voting_engine.add_pass(pass_result.decisions)

        voted_result = voting_engine.vote()

        duration = time.time() - start_time

        return ChunkExtractionResult(
            chunk=chunk,
            passes=passes,
            voted_result=voted_result,
            duration_seconds=duration,
        )

    def _extract_micro_chunk(
        self,
        video_path: str,
        micro_chunk: MicroChunk,
        state: VODRunState,
    ) -> ChunkExtractionResult:
        """Extract a micro-chunk with focused prompt."""
        import time
        start_time_ts = time.time()

        # Create focused prompt for micro-chunk
        state_context = {
            "hp": state.hp,
            "max_hp": state.max_hp,
            "gold": state.gold,
            "floor": state.floor,
            "deck_size": len(state.deck),
            "relics": state.relics[:5] if state.relics else [],  # Limit for prompt size
        }
        prompt = create_micro_chunk_prompt(micro_chunk, state_context)

        # Run extraction (single pass for micro-chunks - faster)
        passes: list[ExtractionResult] = []
        for pass_num in range(self.passes_per_chunk):
            state.current_pass = pass_num + 1
            result = self.extractor.extract_chunk(
                video_path=video_path,
                chunk_prompt=prompt,
                start_time=micro_chunk.start_str,
                end_time=micro_chunk.end_str,
            )
            passes.append(result)

            for decision in result.decisions:
                decision.pass_number = pass_num + 1

            if result.decisions:
                print(f"  Pass {pass_num + 1}: {len(result.decisions)} decisions")

        # Vote across passes
        voting_engine = VotingEngine(confidence_threshold=self.voting_threshold)
        for pass_result in passes:
            voting_engine.add_pass(pass_result.decisions)

        voted_result = voting_engine.vote()

        duration = time.time() - start_time_ts

        # Create a Chunk object for compatibility
        compat_chunk = Chunk(
            chunk_type="micro",
            floor_start=micro_chunk.floor_start,
            floor_end=micro_chunk.floor_end,
            start_time=micro_chunk.start_str,
            end_time=micro_chunk.end_str,
        )

        return ChunkExtractionResult(
            chunk=compat_chunk,
            passes=passes,
            voted_result=voted_result,
            duration_seconds=duration,
        )

    def _apply_decisions_to_state(
        self,
        state: VODRunState,
        decisions: list[DecisionLog],
    ) -> None:
        """Apply voted decisions to the state."""
        handler = ToolHandler(state)

        for decision in decisions:
            tool_name = self._decision_type_to_tool(decision.decision_type)
            if tool_name:
                handler.handle(tool_name, decision.data)

    def _decision_type_to_tool(self, dtype) -> Optional[str]:
        """Map decision type back to tool name."""
        from vod.state import DecisionType

        mapping = {
            DecisionType.SEED: "set_seed",
            DecisionType.NEOW: "neow",
            DecisionType.PATH: "path",
            DecisionType.COMBAT_START: "combat_start",
            DecisionType.COMBAT_TURN: "combat_turn",
            DecisionType.COMBAT_END: "combat_end",
            DecisionType.CARD_REWARD: "card_reward",
            DecisionType.RELIC_REWARD: "relic_reward",
            DecisionType.POTION_REWARD: "potion_reward",
            DecisionType.SHOP: "shop",
            DecisionType.REST: "rest",
            DecisionType.BOSS_RELIC: "boss_relic",
            DecisionType.EVENT: "event",
            DecisionType.RESULT: "result",
        }
        return mapping.get(dtype)

    def _save_checkpoint(
        self,
        checkpoint_dir: str,
        video_id: str,
        chunk_index: int,
        state: VODRunState,
        chunk_result: ChunkExtractionResult,
    ) -> None:
        """Save checkpoint after each chunk."""
        os.makedirs(checkpoint_dir, exist_ok=True)

        checkpoint = {
            "video_id": video_id,
            "chunk_index": chunk_index,
            "timestamp": datetime.now().isoformat(),
            "state": state.to_dict(),
            "chunk_result": chunk_result.to_dict(),
        }

        path = os.path.join(
            checkpoint_dir,
            f"{video_id}_checkpoint_{chunk_index:02d}.json"
        )

        with open(path, "w") as f:
            json.dump(checkpoint, f, indent=2)

    def resume_from_checkpoint(
        self,
        checkpoint_path: str,
        video_path: str,
        final_floor: int = 56,
    ) -> FullExtractionResult:
        """Resume extraction from a checkpoint."""
        with open(checkpoint_path) as f:
            checkpoint = json.load(f)

        state = VODRunState.from_dict(checkpoint["state"])
        start_chunk = checkpoint["chunk_index"] + 1

        # Get remaining chunks
        all_chunks = self.static_chunker.get_chunks(final_floor)
        remaining_chunks = all_chunks[start_chunk:]

        chunk_results = []
        for i, chunk in enumerate(remaining_chunks):
            chunk_result = self._extract_chunk(video_path, chunk, state)
            chunk_results.append(chunk_result)

            if chunk_result.voted_result:
                self._apply_decisions_to_state(
                    state=state,
                    decisions=[d.to_decision_log() for d in chunk_result.voted_result.decisions],
                )

        verification = verify_extraction(state) if state.seed_detected else None

        return FullExtractionResult(
            video_id=checkpoint["video_id"],
            video_url=state.video_url,
            state=state,
            chunk_results=chunk_results,
            verification=verification,
            overall_confidence=0.0,
            total_decisions=len(state.decisions),
            duration_seconds=0.0,
        )


def extract_vod(
    video_path: str,
    video_id: Optional[str] = None,
    video_url: str = "",
    seed: Optional[str] = None,
    final_floor: Optional[int] = None,
    passes: int = 3,
    chunking: ChunkingStrategy = "dynamic",
    extraction_model: str = "gemini-3-flash-preview",
    scan_model: str = "gemini-3-flash-preview",
    use_mock: bool = False,
    output_dir: str = "data/vod_extractions",
    transcript_path: Optional[str] = None,
    video_duration_seconds: Optional[float] = None,
) -> FullExtractionResult:
    """
    Convenience function to extract a VOD.

    Args:
        video_path: Path to video file
        video_id: Unique ID (defaults to filename)
        video_url: Original URL
        seed: Known seed
        final_floor: Last floor reached (auto-detected with dynamic chunking)
        passes: Number of extraction passes per chunk
        chunking: "static", "dynamic", or "micro"
        extraction_model: Model for chunk extraction
        scan_model: Model for scan pass (can use Pro for better results)
        use_mock: Use mock extractor for testing
        output_dir: Directory to save results
        transcript_path: Path to transcript JSON (required for micro chunking)
        video_duration_seconds: Total video length (for micro chunking)

    Returns:
        Complete extraction result
    """
    if video_id is None:
        video_id = Path(video_path).stem

    orchestrator = ExtractionOrchestrator(
        passes_per_chunk=passes,
        chunking_strategy=chunking,
        extraction_model=extraction_model,
        scan_model=scan_model,
        use_mock=use_mock,
    )

    result = orchestrator.extract(
        video_path=video_path,
        video_id=video_id,
        video_url=video_url,
        seed=seed,
        final_floor=final_floor,
        transcript_path=transcript_path,
        video_duration_seconds=video_duration_seconds,
    )

    # Save result
    result.save(output_dir)

    return result


def print_extraction_summary(result: FullExtractionResult) -> None:
    """Print a human-readable summary of extraction results."""
    print(f"\n{'='*60}")
    print(f"VOD Extraction Summary: {result.video_id}")
    print(f"{'='*60}")
    print(f"Video URL: {result.video_url or 'N/A'}")
    print(f"Extracted at: {result.extracted_at}")
    print(f"Duration: {result.duration_seconds:.1f}s")
    print()

    if result.scan_result:
        print("Scan Results:")
        print(f"  Events detected: {len(result.scan_result.events)}")
        print(f"  Final floor: {result.scan_result.final_floor}")
        print(f"  Victory: {result.scan_result.victory}")
        print()

    print("State Summary:")
    print(f"  Floor: {result.state.floor}")
    print(f"  HP: {result.state.hp}/{result.state.max_hp}")
    print(f"  Gold: {result.state.gold}")
    print(f"  Deck size: {len(result.state.deck)}")
    print(f"  Relics: {len(result.state.relics)}")
    print(f"  Seed: {result.state.seed}")
    print()

    print("Extraction Metrics:")
    print(f"  Total decisions: {result.total_decisions}")
    print(f"  Chunks processed: {len(result.chunk_results)}")
    print(f"  Overall confidence: {result.overall_confidence:.1%}")
    print()

    if result.verification:
        v = result.verification
        print("Seed Verification:")
        print(f"  Card reward accuracy: {v.card_reward_accuracy:.1%}")
        print(f"  Path accuracy: {v.path_accuracy:.1%}")
        print(f"  Overall accuracy: {v.overall_accuracy:.1%}")
        print(f"  Mismatches: {len(v.mismatches)}")

    print(f"{'='*60}\n")
