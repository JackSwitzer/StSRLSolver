import { useState } from 'react';
import type { GameObservation } from './types/game';
import { useGameState } from './hooks/useGameState';
import { MapView } from './components/MapView';
import { CombatView } from './components/CombatView';
import { Sidebar } from './components/Sidebar';

// ---------------------------------------------------------------------------
// Mock data so the app renders something without a WebSocket connection
// ---------------------------------------------------------------------------

const MOCK_COMBAT: GameObservation = {
  phase: 'combat',
  run: {
    hp: 55,
    max_hp: 72,
    gold: 142,
    floor: 7,
    act: 1,
    ascension: 20,
    deck: [
      { id: 'strike_1', name: 'Strike', cost: 1, type: 'attack', upgraded: false },
      { id: 'strike_2', name: 'Strike', cost: 1, type: 'attack', upgraded: false },
      { id: 'strike_3', name: 'Strike', cost: 1, type: 'attack', upgraded: false },
      { id: 'strike_4', name: 'Strike', cost: 1, type: 'attack', upgraded: false },
      { id: 'defend_1', name: 'Defend', cost: 1, type: 'skill', upgraded: false },
      { id: 'defend_2', name: 'Defend', cost: 1, type: 'skill', upgraded: false },
      { id: 'defend_3', name: 'Defend', cost: 1, type: 'skill', upgraded: false },
      { id: 'defend_4', name: 'Defend', cost: 1, type: 'skill', upgraded: false },
      { id: 'eruption', name: 'Eruption', cost: 2, type: 'attack', upgraded: false, description: 'Deal 9 damage. Enter Wrath.' },
      { id: 'vigilance', name: 'Vigilance', cost: 2, type: 'skill', upgraded: false, description: 'Gain 8 Block. Enter Calm.' },
      { id: 'tantrum', name: 'Tantrum', cost: 1, type: 'attack', upgraded: true, description: 'Deal 3 damage 4 times. Enter Wrath.' },
      { id: 'inner_peace', name: 'Inner Peace', cost: 1, type: 'skill', upgraded: false, description: 'If Calm: draw 3. Else: enter Calm.' },
      { id: 'mental_fortress', name: 'Mental Fortress', cost: 1, type: 'power', upgraded: false, description: 'Gain 4 Block on stance change.' },
    ],
    relics: [
      { id: 'pure_water', name: 'Pure Water' },
      { id: 'vajra', name: 'Vajra', counter: 0 },
      { id: 'bag_of_marbles', name: 'Bag of Marbles' },
    ],
    potions: [
      { id: 'block_potion', name: 'Block Potion' },
      { id: null, name: null },
      { id: 'fire_potion', name: 'Fire Potion' },
    ],
  },
  combat: {
    player: {
      hp: 55,
      max_hp: 72,
      block: 8,
      powers: [
        { id: 'strength', name: 'Strength', amount: 1 },
        { id: 'mental_fortress', name: 'Mental Fortress', amount: 4 },
      ],
    },
    enemies: [
      {
        id: 'gremlin_nob',
        name: 'Gremlin Nob',
        hp: 82,
        max_hp: 106,
        block: 0,
        size: 'large',
        intent: { type: 'attack', damage: 16 },
        powers: [
          { id: 'anger', name: 'Anger', amount: 3 },
        ],
      },
    ],
    hand: [
      { id: 'strike_1', name: 'Strike', cost: 1, type: 'attack', upgraded: false, playable: true, description: 'Deal 6 damage.' },
      { id: 'defend_2', name: 'Defend', cost: 1, type: 'skill', upgraded: false, playable: true, description: 'Gain 5 Block.' },
      { id: 'tantrum', name: 'Tantrum', cost: 1, type: 'attack', upgraded: true, playable: true, description: 'Deal 3 damage 4 times. Enter Wrath.' },
      { id: 'inner_peace', name: 'Inner Peace', cost: 1, type: 'skill', upgraded: false, playable: true, description: 'If Calm: draw 3. Else: enter Calm.' },
      { id: 'eruption', name: 'Eruption', cost: 2, type: 'attack', upgraded: false, playable: true, description: 'Deal 9 damage. Enter Wrath.' },
    ],
    draw_pile_count: 5,
    discard_pile_count: 3,
    exhaust_pile_count: 0,
    energy: 3,
    max_energy: 3,
    turn: 2,
    stance: 'calm',
  },
};

const MOCK_MAP: GameObservation = {
  phase: 'map',
  run: { ...MOCK_COMBAT.run, floor: 3 },
  map: {
    boss_name: 'Hexaghost',
    current_node: { x: 1, y: 2 },
    available_next: [
      { x: 1, y: 3 },
      { x: 2, y: 3 },
    ],
    nodes: [
      // Floor 0 (bottom)
      [
        { x: 0, y: 0, type: 'monster' },
        { x: 1, y: 0, type: 'monster' },
        { x: 2, y: 0, type: 'monster' },
        { x: 3, y: 0, type: 'monster' },
      ],
      // Floor 1
      [
        { x: 0, y: 1, type: 'event' },
        { x: 1, y: 1, type: 'monster' },
        { x: 2, y: 1, type: 'treasure' },
        { x: 3, y: 1, type: 'monster' },
      ],
      // Floor 2 (current)
      [
        { x: 0, y: 2, type: 'monster' },
        { x: 1, y: 2, type: 'elite' },
        { x: 2, y: 2, type: 'shop' },
      ],
      // Floor 3
      [
        { x: 0, y: 3, type: 'event' },
        { x: 1, y: 3, type: 'rest' },
        { x: 2, y: 3, type: 'monster' },
      ],
      // Floor 4
      [
        { x: 0, y: 4, type: 'monster' },
        { x: 1, y: 4, type: 'monster' },
        { x: 2, y: 4, type: 'elite' },
      ],
      // Floor 5
      [
        { x: 0, y: 5, type: 'treasure' },
        { x: 1, y: 5, type: 'event' },
      ],
      // Floor 6
      [
        { x: 0, y: 6, type: 'rest' },
        { x: 1, y: 6, type: 'monster' },
        { x: 2, y: 6, type: 'rest' },
      ],
      // Floor 7
      [
        { x: 0, y: 7, type: 'boss' },
      ],
    ],
    edges: [
      // Floor 0 -> 1
      { from: { x: 0, y: 0 }, to: { x: 0, y: 1 } },
      { from: { x: 1, y: 0 }, to: { x: 1, y: 1 } },
      { from: { x: 1, y: 0 }, to: { x: 0, y: 1 } },
      { from: { x: 2, y: 0 }, to: { x: 2, y: 1 } },
      { from: { x: 3, y: 0 }, to: { x: 3, y: 1 } },
      // Floor 1 -> 2
      { from: { x: 0, y: 1 }, to: { x: 0, y: 2 } },
      { from: { x: 1, y: 1 }, to: { x: 1, y: 2 } },
      { from: { x: 2, y: 1 }, to: { x: 2, y: 2 } },
      { from: { x: 3, y: 1 }, to: { x: 2, y: 2 } },
      // Floor 2 -> 3
      { from: { x: 0, y: 2 }, to: { x: 0, y: 3 } },
      { from: { x: 1, y: 2 }, to: { x: 1, y: 3 } },
      { from: { x: 1, y: 2 }, to: { x: 2, y: 3 } },
      { from: { x: 2, y: 2 }, to: { x: 2, y: 3 } },
      // Floor 3 -> 4
      { from: { x: 0, y: 3 }, to: { x: 0, y: 4 } },
      { from: { x: 1, y: 3 }, to: { x: 1, y: 4 } },
      { from: { x: 2, y: 3 }, to: { x: 2, y: 4 } },
      // Floor 4 -> 5
      { from: { x: 0, y: 4 }, to: { x: 0, y: 5 } },
      { from: { x: 1, y: 4 }, to: { x: 1, y: 5 } },
      { from: { x: 2, y: 4 }, to: { x: 1, y: 5 } },
      // Floor 5 -> 6
      { from: { x: 0, y: 5 }, to: { x: 0, y: 6 } },
      { from: { x: 1, y: 5 }, to: { x: 1, y: 6 } },
      { from: { x: 1, y: 5 }, to: { x: 2, y: 6 } },
      // Floor 6 -> 7 (boss)
      { from: { x: 0, y: 6 }, to: { x: 0, y: 7 } },
      { from: { x: 1, y: 6 }, to: { x: 0, y: 7 } },
      { from: { x: 2, y: 6 }, to: { x: 0, y: 7 } },
    ],
  },
};

type MockScene = 'combat' | 'map';

export const App = () => {
  const { state: liveState, connected } = useGameState();
  const [mockScene, setMockScene] = useState<MockScene>('combat');

  // Use live state if connected, otherwise use mock
  const state: GameObservation = liveState || (mockScene === 'combat' ? MOCK_COMBAT : MOCK_MAP);
  const isUsingMock = !liveState;

  return (
    <>
      {/* Header */}
      <header className="app-header">
        <h1>Slay the Spire RL</h1>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          {isUsingMock && (
            <div style={{ display: 'flex', gap: '4px' }}>
              <button
                onClick={() => setMockScene('combat')}
                style={{
                  padding: '2px 8px',
                  fontSize: '11px',
                  background: mockScene === 'combat' ? 'var(--accent)' : 'var(--border)',
                  color: 'var(--text)',
                  border: 'none',
                  borderRadius: '3px',
                  cursor: 'pointer',
                }}
              >
                Combat
              </button>
              <button
                onClick={() => setMockScene('map')}
                style={{
                  padding: '2px 8px',
                  fontSize: '11px',
                  background: mockScene === 'map' ? 'var(--accent)' : 'var(--border)',
                  color: 'var(--text)',
                  border: 'none',
                  borderRadius: '3px',
                  cursor: 'pointer',
                }}
              >
                Map
              </button>
            </div>
          )}
          <div className="connection-status">
            <span className={`status-dot ${connected ? 'connected' : isUsingMock ? 'mock' : 'disconnected'}`} />
            <span>{connected ? 'Connected' : isUsingMock ? 'Mock Data' : 'Disconnected'}</span>
          </div>
        </div>
      </header>

      {/* Body */}
      <div className="app-body">
        {/* Main view area */}
        <div className="main-view">
          <div className="phase-label">Phase: {state.phase}</div>

          {state.phase === 'combat' && state.combat && (
            <CombatView combat={state.combat} />
          )}

          {state.phase === 'map' && state.map && (
            <MapView map={state.map} />
          )}

          {state.phase !== 'combat' && state.phase !== 'map' && (
            <div style={{ textAlign: 'center', color: '#666', marginTop: '40px' }}>
              <p>Phase: {state.phase}</p>
              <p>No renderer for this phase yet.</p>
            </div>
          )}
        </div>

        {/* Right sidebar */}
        <Sidebar run={state.run} />
      </div>
    </>
  );
};
