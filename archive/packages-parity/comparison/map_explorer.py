#!/usr/bin/env python3
"""
Interactive Map Explorer

Generates an HTML/SVG visualization of a seeded map with dynamic RNG tracking.
Open the generated HTML in a browser to explore the map interactively.
"""

import sys
import os
import json

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict

from packages.engine.state.rng import Random, seed_to_long
from packages.engine.state.game_rng import GameRNGState, RNGStream
from packages.engine.generation.map import MapGenerator, MapGeneratorConfig, MapRoomNode, RoomType
from packages.engine.generation.rewards import generate_card_rewards, RewardState


def get_map_seed(base_seed: int, act: int) -> int:
    """Get the map RNG seed for an act."""
    # From AbstractDungeon.generateSeeds() - map uses offset
    return base_seed + act


def generate_map_for_seed(seed_str: str, act: int = 1, ascension: int = 20) -> List[List[MapRoomNode]]:
    """Generate the map for a given seed and act."""
    base_seed = seed_to_long(seed_str)
    map_seed = get_map_seed(base_seed, act)

    config = MapGeneratorConfig(ascension_level=ascension)
    rng = Random(map_seed)
    generator = MapGenerator(rng, config)

    return generator.generate()


def map_to_json(nodes: List[List[MapRoomNode]]) -> List[Dict]:
    """Convert map to JSON-serializable format."""
    result = []
    for row in nodes:
        row_data = []
        for node in row:
            if node.has_edges() or node.room_type is not None:
                node_data = {
                    "x": node.x,
                    "y": node.y,
                    "type": node.room_type.value if node.room_type else None,
                    "typeName": node.room_type.name if node.room_type else None,
                    "edges": [{"dx": e.dst_x, "dy": e.dst_y} for e in node.edges],
                    "hasEmeraldKey": node.has_emerald_key,
                }
                row_data.append(node_data)
        result.append(row_data)
    return result


def generate_card_predictions(seed_str: str, neow: str = "BOSS_SWAP", max_combats: int = 50) -> List[Dict]:
    """
    Pre-generate card reward predictions for all possible combat states.

    Returns predictions indexed by cardRng counter value.
    """
    rng = GameRNGState(seed_str)
    if neow != "NONE":
        rng.apply_neow_choice(neow)

    predictions = {}
    reward_state = RewardState()

    # Generate predictions for counter values 0 to ~600 (full run)
    for counter in range(0, 800, 9):  # Each combat uses ~9 calls
        card_rng = Random(rng.seed, counter)
        temp_reward = RewardState()  # Fresh for each prediction

        # Normal room prediction
        cards_normal = generate_card_rewards(
            rng=card_rng,
            reward_state=temp_reward,
            act=1 if counter < 250 else (2 if counter < 500 else 3),
            player_class="WATCHER",
            ascension=20,
            room_type="normal",
            num_cards=3,
        )

        # Elite room prediction (different rarity thresholds)
        card_rng_elite = Random(rng.seed, counter)
        temp_reward_elite = RewardState()
        cards_elite = generate_card_rewards(
            rng=card_rng_elite,
            reward_state=temp_reward_elite,
            act=1 if counter < 250 else (2 if counter < 500 else 3),
            player_class="WATCHER",
            ascension=20,
            room_type="elite",
            num_cards=3,
        )

        predictions[counter] = {
            "normal": [c.name for c in cards_normal],
            "elite": [c.name for c in cards_elite],
            "counterAfter": card_rng.counter,
        }

    return predictions


def generate_html(seed_str: str, neow: str = "BOSS_SWAP", ascension: int = 20) -> str:
    """Generate interactive HTML map explorer."""

    # Generate map
    map_nodes = generate_map_for_seed(seed_str, act=1, ascension=ascension)
    map_json = map_to_json(map_nodes)

    # Generate card predictions
    predictions = generate_card_predictions(seed_str, neow)

    html = f'''<!DOCTYPE html>
<html>
<head>
    <title>STS Map Explorer - {seed_str}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            display: flex;
        }}
        #sidebar {{
            width: 350px;
            background: #16213e;
            padding: 20px;
            overflow-y: auto;
            border-right: 2px solid #0f3460;
        }}
        #map-container {{
            flex: 1;
            padding: 20px;
            overflow: auto;
        }}
        h1 {{ color: #e94560; font-size: 1.4em; margin-bottom: 10px; }}
        h2 {{ color: #0f3460; background: #e94560; padding: 8px; margin: 15px -20px; }}
        .info-row {{ display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #0f3460; }}
        .label {{ color: #888; }}
        .value {{ color: #fff; font-weight: bold; }}
        #rng-state {{ background: #0f3460; padding: 10px; border-radius: 5px; margin: 10px 0; }}
        #prediction {{ background: #1a1a2e; padding: 10px; border-radius: 5px; margin: 10px 0; }}
        #prediction h3 {{ color: #e94560; margin-bottom: 10px; }}
        .card {{ background: #0f3460; padding: 8px 12px; margin: 5px 0; border-radius: 3px; border-left: 3px solid #e94560; }}
        #path-log {{ max-height: 200px; overflow-y: auto; font-size: 0.9em; }}
        .log-entry {{ padding: 3px 0; border-bottom: 1px solid #0f3460; }}
        svg {{ display: block; }}
        .node {{ cursor: pointer; transition: all 0.2s; }}
        .node:hover {{ filter: brightness(1.3); }}
        .node.visited {{ opacity: 0.5; }}
        .node.current {{ filter: drop-shadow(0 0 8px #e94560); }}
        .edge {{ stroke: #444; stroke-width: 2; }}
        .edge.available {{ stroke: #e94560; stroke-width: 3; }}
        .room-label {{ font-size: 14px; font-weight: bold; fill: #fff; text-anchor: middle; pointer-events: none; }}
        #controls {{ margin-bottom: 15px; }}
        button {{
            background: #e94560;
            color: #fff;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 5px;
        }}
        button:hover {{ background: #ff6b6b; }}
        #neow-select {{ background: #0f3460; color: #fff; border: 1px solid #e94560; padding: 5px; }}
    </style>
</head>
<body>
    <div id="sidebar">
        <h1>STS Map Explorer</h1>

        <div class="info-row">
            <span class="label">Seed:</span>
            <span class="value">{seed_str}</span>
        </div>
        <div class="info-row">
            <span class="label">Ascension:</span>
            <span class="value">{ascension}</span>
        </div>

        <div id="controls">
            <label>Neow Choice:</label>
            <select id="neow-select" onchange="changeNeow(this.value)">
                <option value="NONE">None</option>
                <option value="BOSS_SWAP" selected>Boss Swap</option>
                <option value="HUNDRED_GOLD">100 Gold</option>
                <option value="RANDOM_COMMON_RELIC">Random Common Relic</option>
                <option value="THREE_CARDS">3 Card Choices</option>
                <option value="UPGRADE_CARD">Upgrade a Card</option>
                <option value="REMOVE_CARD">Remove a Card</option>
            </select>
            <button onclick="resetPath()">Reset</button>
        </div>

        <div id="rng-state">
            <div class="info-row">
                <span class="label">cardRng Counter:</span>
                <span class="value" id="card-counter">0</span>
            </div>
            <div class="info-row">
                <span class="label">Floor:</span>
                <span class="value" id="current-floor">0</span>
            </div>
            <div class="info-row">
                <span class="label">Act:</span>
                <span class="value" id="current-act">1</span>
            </div>
        </div>

        <div id="prediction">
            <h3>Next Combat Prediction</h3>
            <div id="prediction-cards">
                <em>Click a combat node to see prediction</em>
            </div>
        </div>

        <h2>Path History</h2>
        <div id="path-log"></div>
    </div>

    <div id="map-container">
        <svg id="map" width="600" height="900"></svg>
    </div>

    <script>
    const MAP_DATA = {json.dumps(map_json)};
    const PREDICTIONS = {json.dumps(predictions)};
    const SEED = "{seed_str}";

    // Room type colors
    const ROOM_COLORS = {{
        'M': '#4a90d9',  // Monster - blue
        'E': '#ffd700',  // Elite - gold
        'R': '#32cd32',  // Rest - green
        '$': '#ffb347',  // Shop - orange
        '?': '#9370db',  // Event - purple
        'T': '#daa520',  // Treasure - goldenrod
        'B': '#dc143c',  // Boss - crimson
    }};

    const ROOM_NAMES = {{
        'M': 'Monster',
        'E': 'Elite',
        'R': 'Rest',
        '$': 'Shop',
        '?': 'Event',
        'T': 'Treasure',
        'B': 'Boss',
    }};

    // Game state
    let state = {{
        cardRngCounter: 0,
        floor: 0,
        act: 1,
        currentNode: null,
        visitedNodes: new Set(),
        path: [],
        neow: 'BOSS_SWAP',
    }};

    function initMap() {{
        const svg = document.getElementById('map');
        svg.innerHTML = '';

        const nodeWidth = 50;
        const nodeHeight = 40;
        const colSpacing = 80;
        const rowSpacing = 55;
        const offsetX = 50;
        const offsetY = 50;

        // Draw edges first
        const edgeGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        edgeGroup.id = 'edges';

        MAP_DATA.forEach((row, rowIdx) => {{
            row.forEach(node => {{
                if (!node) return;
                const x1 = offsetX + node.x * colSpacing + nodeWidth/2;
                const y1 = offsetY + (14 - node.y) * rowSpacing + nodeHeight/2;

                node.edges.forEach(edge => {{
                    const x2 = offsetX + edge.dx * colSpacing + nodeWidth/2;
                    const y2 = offsetY + (14 - edge.dy) * rowSpacing + nodeHeight/2;

                    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                    line.setAttribute('x1', x1);
                    line.setAttribute('y1', y1);
                    line.setAttribute('x2', x2);
                    line.setAttribute('y2', y2);
                    line.setAttribute('class', 'edge');
                    line.setAttribute('data-from', `${{node.x}},${{node.y}}`);
                    line.setAttribute('data-to', `${{edge.dx}},${{edge.dy}}`);
                    edgeGroup.appendChild(line);
                }});
            }});
        }});
        svg.appendChild(edgeGroup);

        // Draw nodes
        const nodeGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        nodeGroup.id = 'nodes';

        MAP_DATA.forEach((row, rowIdx) => {{
            row.forEach(node => {{
                if (!node || !node.type) return;

                const x = offsetX + node.x * colSpacing;
                const y = offsetY + (14 - node.y) * rowSpacing;

                const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
                g.setAttribute('class', 'node');
                g.setAttribute('data-x', node.x);
                g.setAttribute('data-y', node.y);
                g.setAttribute('data-type', node.type);
                g.onclick = () => clickNode(node);

                const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
                rect.setAttribute('x', x);
                rect.setAttribute('y', y);
                rect.setAttribute('width', nodeWidth);
                rect.setAttribute('height', nodeHeight);
                rect.setAttribute('rx', 5);
                rect.setAttribute('fill', ROOM_COLORS[node.type] || '#666');
                g.appendChild(rect);

                const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                text.setAttribute('x', x + nodeWidth/2);
                text.setAttribute('y', y + nodeHeight/2 + 5);
                text.setAttribute('class', 'room-label');
                text.textContent = node.type;
                g.appendChild(text);

                // Floor number
                const floorText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                floorText.setAttribute('x', x + nodeWidth/2);
                floorText.setAttribute('y', y - 5);
                floorText.setAttribute('class', 'room-label');
                floorText.setAttribute('font-size', '10');
                floorText.setAttribute('fill', '#888');
                floorText.textContent = node.y + 1;
                g.appendChild(floorText);

                nodeGroup.appendChild(g);
            }});
        }});
        svg.appendChild(nodeGroup);

        // Add boss node
        const bossG = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        bossG.setAttribute('class', 'node');
        bossG.setAttribute('data-type', 'B');
        const bossX = offsetX + 3 * colSpacing;
        const bossY = offsetY - rowSpacing;

        const bossRect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        bossRect.setAttribute('x', bossX);
        bossRect.setAttribute('y', bossY);
        bossRect.setAttribute('width', nodeWidth);
        bossRect.setAttribute('height', nodeHeight);
        bossRect.setAttribute('rx', 5);
        bossRect.setAttribute('fill', ROOM_COLORS['B']);
        bossG.appendChild(bossRect);

        const bossText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        bossText.setAttribute('x', bossX + nodeWidth/2);
        bossText.setAttribute('y', bossY + nodeHeight/2 + 5);
        bossText.setAttribute('class', 'room-label');
        bossText.textContent = 'B';
        bossG.appendChild(bossText);
        nodeGroup.appendChild(bossG);

        updateAvailableNodes();
    }}

    function clickNode(node) {{
        if (!canVisit(node)) return;

        state.visitedNodes.add(`${{node.x}},${{node.y}}`);
        state.currentNode = node;
        state.floor = node.y + 1;

        // Handle room type effects on RNG
        const roomType = node.type;
        let prediction = null;

        if (roomType === 'M' || roomType === 'E' || roomType === 'B') {{
            // Combat - show card prediction
            const counterKey = Math.floor(state.cardRngCounter / 9) * 9;
            prediction = PREDICTIONS[counterKey] || PREDICTIONS[0];
            const cards = roomType === 'E' ? prediction.elite : prediction.normal;

            document.getElementById('prediction-cards').innerHTML =
                cards.map(c => `<div class="card">${{c}}</div>`).join('');

            addToLog(`Floor ${{state.floor}} [${{ROOM_NAMES[roomType]}}]: ${{cards.join(', ')}}`);

            // Advance counter
            state.cardRngCounter += 9;
        }} else if (roomType === '$') {{
            // Shop
            state.cardRngCounter += 12;
            document.getElementById('prediction-cards').innerHTML = '<em>Shop: +12 cardRng</em>';
            addToLog(`Floor ${{state.floor}} [Shop]: cardRng +12`);
        }} else if (roomType === '?') {{
            // Event
            document.getElementById('prediction-cards').innerHTML = '<em>Event: uses miscRng</em>';
            addToLog(`Floor ${{state.floor}} [Event]`);
        }} else if (roomType === 'R') {{
            document.getElementById('prediction-cards').innerHTML = '<em>Rest Site</em>';
            addToLog(`Floor ${{state.floor}} [Rest]`);
        }} else if (roomType === 'T') {{
            document.getElementById('prediction-cards').innerHTML = '<em>Treasure: uses treasureRng</em>';
            addToLog(`Floor ${{state.floor}} [Treasure]`);
        }}

        state.path.push(node);
        updateUI();
    }}

    function canVisit(node) {{
        if (state.floor === 0) {{
            // First move - can only visit floor 1 nodes
            return node.y === 0;
        }}

        // Check if current node has edge to target
        const current = state.path[state.path.length - 1];
        if (!current) return node.y === 0;

        return current.edges.some(e => e.dx === node.x && e.dy === node.y);
    }}

    function updateAvailableNodes() {{
        document.querySelectorAll('.node').forEach(n => {{
            const x = parseInt(n.getAttribute('data-x'));
            const y = parseInt(n.getAttribute('data-y'));
            const key = `${{x}},${{y}}`;

            if (state.visitedNodes.has(key)) {{
                n.classList.add('visited');
                n.classList.remove('current');
            }} else {{
                n.classList.remove('visited');
            }}
        }});

        // Highlight available edges
        document.querySelectorAll('.edge').forEach(e => {{
            e.classList.remove('available');
        }});

        if (state.currentNode) {{
            state.currentNode.edges.forEach(edge => {{
                const selector = `.edge[data-from="${{state.currentNode.x}},${{state.currentNode.y}}"][data-to="${{edge.dx}},${{edge.dy}}"]`;
                const edgeEl = document.querySelector(selector);
                if (edgeEl) edgeEl.classList.add('available');
            }});
        }} else {{
            // Highlight edges from start
            document.querySelectorAll('.edge[data-from$=",-1"]').forEach(e => {{
                e.classList.add('available');
            }});
        }}
    }}

    function updateUI() {{
        document.getElementById('card-counter').textContent = state.cardRngCounter;
        document.getElementById('current-floor').textContent = state.floor;
        document.getElementById('current-act').textContent = state.act;
        updateAvailableNodes();
    }}

    function addToLog(message) {{
        const log = document.getElementById('path-log');
        const entry = document.createElement('div');
        entry.className = 'log-entry';
        entry.textContent = message;
        log.appendChild(entry);
        log.scrollTop = log.scrollHeight;
    }}

    function resetPath() {{
        state = {{
            cardRngCounter: 0,
            floor: 0,
            act: 1,
            currentNode: null,
            visitedNodes: new Set(),
            path: [],
            neow: state.neow,
        }};
        document.getElementById('path-log').innerHTML = '';
        document.getElementById('prediction-cards').innerHTML = '<em>Click a combat node to see prediction</em>';
        updateUI();
        addToLog('Path reset - Neow: ' + state.neow);
    }}

    function changeNeow(value) {{
        state.neow = value;
        resetPath();
    }}

    // Initialize
    initMap();
    addToLog('Map generated for seed: ' + SEED);
    addToLog('Select Neow choice and click first floor node');
    </script>
</body>
</html>
'''
    return html


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Generate interactive map explorer")
    parser.add_argument("seed", nargs="?", default="33J85JVCVSPJY", help="Seed string")
    parser.add_argument("--neow", default="BOSS_SWAP", help="Neow choice")
    parser.add_argument("--ascension", type=int, default=20, help="Ascension level")
    parser.add_argument("-o", "--output", help="Output HTML file")

    args = parser.parse_args()

    output_path = args.output or f"map_explorer_{args.seed}.html"

    print(f"Generating map explorer for seed: {args.seed}")
    html = generate_html(args.seed, args.neow, args.ascension)

    with open(output_path, 'w') as f:
        f.write(html)

    print(f"Saved to: {output_path}")
    print(f"\nOpen in browser to explore the map interactively.")

    # Try to open in browser
    import webbrowser
    webbrowser.open(f"file://{os.path.abspath(output_path)}")


if __name__ == "__main__":
    main()
