#!/bin/bash
# Common commands for STS RL project

# ==============================================================================
# VOD Analysis / Training Data
# ==============================================================================

# Process existing Baalorlord Spirelogs data
vod_baalorlord() {
    uv run python scripts/analyze_vods.py --baalorlord
}

# Analyze YouTube video with transcript (requires OPENROUTER_API_KEY)
vod_transcript() {
    local video_id=$1
    local streamer=$2
    uv run python scripts/analyze_vods.py --video "$video_id" --streamer "$streamer"
}

# Analyze YouTube video directly with Gemini (requires GOOGLE_API_KEY)
vod_gemini() {
    local url=$1
    uv run python scripts/analyze_vods.py --video-url "$url"
}

# List available streamers and videos
vod_list() {
    uv run python scripts/analyze_vods.py --list
}

# Test LLM connections
vod_test() {
    uv run python scripts/analyze_vods.py --test-connection
}

# ==============================================================================
# Combat Search Server
# ==============================================================================

# Start the combat search server
search_server() {
    uv run python run_search_server.py --workers 4
}

# ==============================================================================
# Java Mod
# ==============================================================================

# Build the mod
mod_build() {
    cd mod && mvn package -DskipTests && cd ..
}

# Run the game with mod
mod_run() {
    ./scripts/dev.sh --no-server
}

# Full dev environment (server + mod + game)
mod_dev() {
    ./scripts/dev.sh
}

# ==============================================================================
# Development
# ==============================================================================

# Run tests
test() {
    uv run pytest tests/ -v
}

# Install all dependencies
install() {
    uv pip install -e ".[dev,vod,data]"
}

# ==============================================================================
# Usage
# ==============================================================================

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "Source this file to use commands:"
    echo "  source scripts/commands.sh"
    echo ""
    echo "Available commands:"
    echo "  vod_baalorlord    - Process Baalorlord Spirelogs data"
    echo "  vod_transcript    - Analyze video with transcript"
    echo "  vod_gemini        - Analyze video with Gemini"
    echo "  vod_list          - List available videos"
    echo "  vod_test          - Test LLM connections"
    echo "  search_server     - Start combat search server"
    echo "  mod_build         - Build Java mod"
    echo "  mod_run           - Run game with mod"
    echo "  mod_dev           - Full dev environment"
    echo "  test              - Run tests"
    echo "  install           - Install dependencies"
fi
