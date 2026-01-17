"""
VOD Annotation Tool

Generates HTML pages for manual decision annotation:
- Embedded YouTube player
- Clickable transcript with timestamps
- Decision annotation interface
- Export to training data

Workflow:
1. Collect transcript from YouTube
2. Generate annotation HTML
3. Human annotates decisions (card picks, events, etc.)
4. Grab seed from victory screen
5. Use seed in sim to verify all possible options
6. Export verified training data
"""

import json
import os
from pathlib import Path
from typing import List, Dict, Optional
from dataclasses import dataclass, asdict
import html

# ============ DATA STRUCTURES ============

@dataclass
class AnnotatedDecision:
    """A human-verified decision."""
    timestamp: float
    decision_type: str  # card_pick, card_skip, rest_site, path, boss_relic, event, shop
    choice: str
    alternatives: List[str]  # Other options available
    reasoning: str  # Why they made this choice (from transcript)
    floor: Optional[int] = None
    verified: bool = False

@dataclass
class AnnotatedRun:
    """A fully annotated run."""
    video_id: str
    video_url: str
    streamer: str
    character: str = "Watcher"
    ascension: int = 20
    seed: str = ""  # From victory screen
    result: str = "win"
    decisions: List[AnnotatedDecision] = None
    transcript: List[Dict] = None

# ============ HTML GENERATION ============

HTML_TEMPLATE = '''<!DOCTYPE html>
<html>
<head>
    <title>STS Decision Annotator - {video_id}</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0; padding: 20px;
            background: #1a1a2e; color: #eee;
        }}
        .container {{ display: flex; gap: 20px; max-width: 1800px; margin: 0 auto; }}
        .video-panel {{ flex: 1; min-width: 640px; }}
        .transcript-panel {{ flex: 1; max-height: 90vh; overflow-y: auto; }}
        .decisions-panel {{ flex: 1; max-height: 90vh; overflow-y: auto; }}

        h1 {{ color: #00d4ff; margin-bottom: 5px; }}
        h2 {{ color: #ff6b6b; margin-top: 20px; }}
        .meta {{ color: #888; margin-bottom: 20px; }}

        .video-container {{
            position: relative; padding-bottom: 56.25%; height: 0;
            background: #000; border-radius: 8px; overflow: hidden;
        }}
        .video-container iframe {{
            position: absolute; top: 0; left: 0; width: 100%; height: 100%;
        }}

        .transcript {{ background: #16213e; border-radius: 8px; padding: 15px; }}
        .segment {{
            padding: 8px 12px; margin: 4px 0; border-radius: 4px;
            cursor: pointer; transition: background 0.2s;
        }}
        .segment:hover {{ background: #1f4068; }}
        .segment.highlighted {{ background: #e94560; color: white; }}
        .timestamp {{
            color: #00d4ff; font-size: 12px; margin-right: 10px;
            font-family: monospace;
        }}

        .decisions {{ background: #16213e; border-radius: 8px; padding: 15px; }}
        .decision {{
            background: #1f4068; padding: 15px; margin: 10px 0;
            border-radius: 8px; border-left: 4px solid #00d4ff;
        }}
        .decision.card_pick {{ border-left-color: #4ade80; }}
        .decision.card_skip {{ border-left-color: #f87171; }}
        .decision.rest_site {{ border-left-color: #fbbf24; }}
        .decision.boss_relic {{ border-left-color: #a78bfa; }}
        .decision.event {{ border-left-color: #38bdf8; }}
        .decision.path {{ border-left-color: #fb923c; }}

        .decision-header {{ display: flex; justify-content: space-between; align-items: center; }}
        .decision-type {{
            background: #e94560; color: white; padding: 2px 8px;
            border-radius: 4px; font-size: 12px; text-transform: uppercase;
        }}
        .decision-time {{ color: #00d4ff; font-family: monospace; }}
        .decision-choice {{ font-size: 18px; font-weight: bold; margin: 10px 0; }}
        .decision-alts {{ color: #888; font-size: 14px; }}
        .decision-reasoning {{
            font-style: italic; color: #aaa; margin-top: 10px;
            padding-top: 10px; border-top: 1px solid #333;
        }}

        .add-decision {{
            background: #4ade80; color: #000; border: none;
            padding: 10px 20px; border-radius: 8px; cursor: pointer;
            font-size: 16px; width: 100%; margin-top: 15px;
        }}
        .add-decision:hover {{ background: #22c55e; }}

        .form-group {{ margin: 10px 0; }}
        .form-group label {{ display: block; margin-bottom: 5px; color: #888; }}
        .form-group input, .form-group select, .form-group textarea {{
            width: 100%; padding: 8px; border-radius: 4px;
            border: 1px solid #333; background: #0f0f23; color: #eee;
        }}

        .export-btn {{
            background: #00d4ff; color: #000; border: none;
            padding: 15px 30px; border-radius: 8px; cursor: pointer;
            font-size: 16px; font-weight: bold; margin-top: 20px;
        }}

        #seed-input {{
            font-family: monospace; font-size: 18px;
            text-align: center; letter-spacing: 2px;
        }}

        .controls {{ margin: 15px 0; display: flex; gap: 10px; }}
        .controls button {{
            background: #1f4068; color: #eee; border: none;
            padding: 8px 16px; border-radius: 4px; cursor: pointer;
        }}
        .controls button:hover {{ background: #2d5a87; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="video-panel">
            <h1>🎮 {streamer} - {video_id}</h1>
            <p class="meta">Watcher A20 | <span id="result">{result}</span></p>

            <div class="video-container">
                <iframe id="player"
                    src="https://www.youtube.com/embed/{video_id}?enablejsapi=1"
                    frameborder="0"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                    allowfullscreen>
                </iframe>
            </div>

            <div class="controls">
                <button onclick="seekRelative(-10)">⏪ -10s</button>
                <button onclick="seekRelative(-5)">◀ -5s</button>
                <button onclick="togglePlay()">⏯ Play/Pause</button>
                <button onclick="seekRelative(5)">▶ +5s</button>
                <button onclick="seekRelative(10)">⏩ +10s</button>
            </div>

            <div class="form-group" style="margin-top: 20px;">
                <label>🌱 Seed (from victory screen):</label>
                <input type="text" id="seed-input" placeholder="e.g. 4YUHY81W7GRHT" value="{seed}">
            </div>

            <button class="export-btn" onclick="exportData()">📥 Export Training Data</button>
        </div>

        <div class="transcript-panel">
            <h2>📝 Transcript</h2>
            <p class="meta">Click timestamp to jump • Highlight to mark decision</p>
            <div class="transcript" id="transcript">
                {transcript_html}
            </div>
        </div>

        <div class="decisions-panel">
            <h2>🎯 Decisions ({decision_count})</h2>
            <div class="decisions" id="decisions">
                {decisions_html}
            </div>
            <button class="add-decision" onclick="addDecision()">+ Add Decision</button>
        </div>
    </div>

    <script>
        // YouTube API
        var player;
        var currentTime = 0;

        function onYouTubeIframeAPIReady() {{
            player = new YT.Player('player', {{
                events: {{ 'onReady': onPlayerReady }}
            }});
        }}

        function onPlayerReady(event) {{
            setInterval(() => {{
                if (player && player.getCurrentTime) {{
                    currentTime = player.getCurrentTime();
                    highlightCurrentSegment();
                }}
            }}, 500);
        }}

        function seekTo(seconds) {{
            if (player && player.seekTo) {{
                player.seekTo(seconds, true);
            }}
        }}

        function seekRelative(delta) {{
            if (player && player.getCurrentTime) {{
                seekTo(player.getCurrentTime() + delta);
            }}
        }}

        function togglePlay() {{
            if (player) {{
                var state = player.getPlayerState();
                if (state === 1) player.pauseVideo();
                else player.playVideo();
            }}
        }}

        function highlightCurrentSegment() {{
            document.querySelectorAll('.segment').forEach(el => {{
                var start = parseFloat(el.dataset.start);
                var end = parseFloat(el.dataset.end);
                if (currentTime >= start && currentTime < end) {{
                    el.classList.add('highlighted');
                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                }} else {{
                    el.classList.remove('highlighted');
                }}
            }});
        }}

        // Decisions management
        var decisions = {decisions_json};

        function renderDecisions() {{
            var html = decisions.map((d, i) => `
                <div class="decision ${{d.decision_type}}" data-index="${{i}}">
                    <div class="decision-header">
                        <span class="decision-type">${{d.decision_type}}</span>
                        <span class="decision-time" onclick="seekTo(${{d.timestamp}})">${{formatTime(d.timestamp)}}</span>
                    </div>
                    <div class="decision-choice">${{d.choice}}</div>
                    <div class="decision-alts">vs: ${{d.alternatives.join(', ') || 'N/A'}}</div>
                    <div class="decision-reasoning">"${{d.reasoning}}"</div>
                    <button onclick="editDecision(${{i}})" style="margin-top:10px;background:#1f4068;color:#eee;border:none;padding:5px 10px;border-radius:4px;cursor:pointer;">Edit</button>
                    <button onclick="deleteDecision(${{i}})" style="margin-left:5px;background:#e94560;color:#fff;border:none;padding:5px 10px;border-radius:4px;cursor:pointer;">Delete</button>
                </div>
            `).join('');
            document.getElementById('decisions').innerHTML = html;
        }}

        function formatTime(seconds) {{
            var m = Math.floor(seconds / 60);
            var s = Math.floor(seconds % 60);
            return m + ':' + (s < 10 ? '0' : '') + s;
        }}

        function addDecision() {{
            var type = prompt('Decision type (card_pick, card_skip, rest_site, path, boss_relic, event, shop):');
            if (!type) return;

            var choice = prompt('Choice made:');
            if (!choice) return;

            var alts = prompt('Alternatives (comma separated):');
            var reasoning = prompt('Reasoning (from transcript):');

            decisions.push({{
                timestamp: currentTime,
                decision_type: type,
                choice: choice,
                alternatives: alts ? alts.split(',').map(s => s.trim()) : [],
                reasoning: reasoning || '',
                floor: null,
                verified: true
            }});

            decisions.sort((a, b) => a.timestamp - b.timestamp);
            renderDecisions();
        }}

        function editDecision(index) {{
            var d = decisions[index];
            var choice = prompt('Choice:', d.choice);
            if (choice !== null) d.choice = choice;

            var alts = prompt('Alternatives (comma separated):', d.alternatives.join(', '));
            if (alts !== null) d.alternatives = alts.split(',').map(s => s.trim());

            var reasoning = prompt('Reasoning:', d.reasoning);
            if (reasoning !== null) d.reasoning = reasoning;

            renderDecisions();
        }}

        function deleteDecision(index) {{
            if (confirm('Delete this decision?')) {{
                decisions.splice(index, 1);
                renderDecisions();
            }}
        }}

        function exportData() {{
            var data = {{
                video_id: '{video_id}',
                video_url: 'https://youtube.com/watch?v={video_id}',
                streamer: '{streamer}',
                character: 'Watcher',
                ascension: 20,
                seed: document.getElementById('seed-input').value,
                result: 'win',
                decisions: decisions,
                exported_at: new Date().toISOString()
            }};

            var blob = new Blob([JSON.stringify(data, null, 2)], {{type: 'application/json'}});
            var url = URL.createObjectURL(blob);
            var a = document.createElement('a');
            a.href = url;
            a.download = '{video_id}_annotated.json';
            a.click();
        }}

        // Initialize
        renderDecisions();

        // Load YouTube API
        var tag = document.createElement('script');
        tag.src = "https://www.youtube.com/iframe_api";
        var firstScriptTag = document.getElementsByTagName('script')[0];
        firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
    </script>
</body>
</html>
'''

def format_timestamp(seconds: float) -> str:
    """Format seconds as MM:SS."""
    m = int(seconds // 60)
    s = int(seconds % 60)
    return f"{m}:{s:02d}"

def generate_transcript_html(transcript: List[Dict]) -> str:
    """Generate clickable transcript HTML."""
    segments = []
    for seg in transcript:
        start = seg["start"]
        end = start + seg.get("duration", 3)
        text = html.escape(seg["text"])
        time_str = format_timestamp(start)
        segments.append(
            f'<div class="segment" data-start="{start}" data-end="{end}" onclick="seekTo({start})">'
            f'<span class="timestamp">{time_str}</span>{text}'
            f'</div>'
        )
    return "\n".join(segments)

def generate_decisions_html(decisions: List[Dict]) -> str:
    """Generate decisions HTML (initial render, JS will update)."""
    # Just placeholder - JS handles rendering
    return ""

def generate_annotation_html(
    video_id: str,
    streamer: str,
    transcript: List[Dict],
    decisions: List[Dict] = None,
    seed: str = "",
    output_path: Path = None,
) -> str:
    """Generate full annotation HTML page."""

    if decisions is None:
        decisions = []

    transcript_html = generate_transcript_html(transcript)
    decisions_html = generate_decisions_html(decisions)
    decisions_json = json.dumps(decisions)

    html_content = HTML_TEMPLATE.format(
        video_id=video_id,
        streamer=streamer,
        result="win",
        seed=seed,
        transcript_html=transcript_html,
        decisions_html=decisions_html,
        decisions_json=decisions_json,
        decision_count=len(decisions),
    )

    if output_path:
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(html_content)
        print(f"Generated annotation page: {output_path}")

    return html_content

# ============ PIPELINE ============

def create_annotation_page(
    video_id: str,
    streamer: str,
    output_dir: Path,
    transcript_path: Path = None,
):
    """
    Create annotation page for a video.

    If transcript_path provided, uses existing transcript.
    Otherwise fetches from YouTube.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load or fetch transcript
    if transcript_path and Path(transcript_path).exists():
        with open(transcript_path) as f:
            data = json.load(f)
            transcript = data.get("full_transcript", [])
            # Use pattern decisions as starting point
            decisions = [
                {
                    "timestamp": d["timestamp"],
                    "decision_type": d["type"],
                    "choice": d.get("choice", ""),
                    "alternatives": [],
                    "reasoning": d.get("raw_text", ""),
                    "floor": None,
                    "verified": False,
                }
                for d in data.get("pattern_decisions", [])
                if d.get("confidence", 0) > 0.6
            ]
    else:
        # Fetch transcript
        from transcript_extractor import get_transcript, extract_decisions_from_transcript
        transcript = get_transcript(video_id)
        raw_decisions = extract_decisions_from_transcript(transcript)
        decisions = [
            {
                "timestamp": d.timestamp,
                "decision_type": d.decision_type.value,
                "choice": d.extracted_choice or "",
                "alternatives": [],
                "reasoning": d.raw_text,
                "floor": None,
                "verified": False,
            }
            for d in raw_decisions
        ]

    # Generate HTML
    html_path = output_dir / f"{video_id}_annotate.html"
    generate_annotation_html(
        video_id=video_id,
        streamer=streamer,
        transcript=transcript,
        decisions=decisions,
        output_path=html_path,
    )

    return html_path

def batch_create_annotation_pages(
    video_ids: List[str],
    streamer: str,
    output_dir: Path,
):
    """Create annotation pages for multiple videos."""
    output_dir = Path(output_dir)

    pages = []
    for vid in video_ids:
        try:
            page = create_annotation_page(vid, streamer, output_dir)
            pages.append(page)
        except Exception as e:
            print(f"Failed to create page for {vid}: {e}")

    # Create index page
    index_html = f'''<!DOCTYPE html>
<html>
<head>
    <title>STS Annotation - {streamer}</title>
    <style>
        body {{ font-family: sans-serif; background: #1a1a2e; color: #eee; padding: 40px; }}
        h1 {{ color: #00d4ff; }}
        .video-list {{ list-style: none; padding: 0; }}
        .video-list li {{
            background: #16213e; margin: 10px 0; padding: 15px 20px;
            border-radius: 8px;
        }}
        .video-list a {{ color: #4ade80; text-decoration: none; font-size: 18px; }}
        .video-list a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>🎮 {streamer} - Annotation Pages</h1>
    <ul class="video-list">
        {"".join(f'<li><a href="{p.name}">{p.stem}</a></li>' for p in pages)}
    </ul>
</body>
</html>
'''

    with open(output_dir / "index.html", 'w') as f:
        f.write(index_html)

    print(f"Created {len(pages)} annotation pages in {output_dir}")
    return pages

# ============ CLI ============

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Generate VOD annotation pages")
    parser.add_argument("--video-id", help="YouTube video ID")
    parser.add_argument("--transcript", help="Path to existing transcript JSON")
    parser.add_argument("--streamer", default="unknown", help="Streamer name")
    parser.add_argument("--output", default="./annotations", help="Output directory")
    parser.add_argument("--batch", help="Comma-separated video IDs for batch processing")

    args = parser.parse_args()

    output_dir = Path(args.output)

    if args.batch:
        video_ids = [v.strip() for v in args.batch.split(",")]
        batch_create_annotation_pages(video_ids, args.streamer, output_dir)
    elif args.video_id:
        create_annotation_page(
            args.video_id,
            args.streamer,
            output_dir,
            transcript_path=args.transcript,
        )
    else:
        print("Provide --video-id or --batch")
