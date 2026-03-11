import { useEffect, useRef } from 'react';
import type { AgentEpisodeMsg } from '../types/training';
import { AGENT_NAMES } from '../types/training';

interface EventFeedProps {
  episodes: AgentEpisodeMsg[];
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, '0');
  const mm = String(d.getMinutes()).padStart(2, '0');
  const ss = String(d.getSeconds()).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

function agentName(id: number): string {
  return AGENT_NAMES[id] ?? `Agent${id}`;
}

export const EventFeed = ({ episodes }: EventFeedProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const prevLen = useRef(0);

  // Auto-scroll to right when new events arrive
  useEffect(() => {
    if (episodes.length !== prevLen.current && containerRef.current) {
      containerRef.current.scrollLeft = containerRef.current.scrollWidth;
      prevLen.current = episodes.length;
    }
  }, [episodes.length]);

  const recent = episodes.slice(0, 30).reverse();

  return (
    <div
      ref={containerRef}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        overflowX: 'auto',
        padding: '4px 8px',
        background: '#0d1117',
        borderTop: '1px solid #30363d',
        height: '28px',
        flexShrink: 0,
        scrollbarWidth: 'none',
      }}
    >
      <span style={{ fontSize: '9px', color: '#8b949e', flexShrink: 0, textTransform: 'uppercase', letterSpacing: '0.5px' }}>
        events
      </span>
      {recent.length === 0 && (
        <span style={{ fontSize: '9px', color: '#3d444d', fontFamily: 'monospace' }}>waiting...</span>
      )}
      {recent.map((ep, i) => {
        const won = ep.won;
        const color = won ? '#00ff41' : '#ff4444';
        const label = won ? 'WIN' : `F${ep.floors_reached}`;
        return (
          <span
            key={`${ep.agent_id}-${ep.episode}-${i}`}
            style={{
              fontSize: '9px',
              fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
              color: '#8b949e',
              whiteSpace: 'nowrap',
              flexShrink: 0,
            }}
          >
            <span style={{ color: '#3d444d' }}>[{formatTime(Date.now())}]</span>{' '}
            <span style={{ color: '#c9d1d9' }}>{agentName(ep.agent_id)}</span>{' '}
            <span style={{ color, fontWeight: 700 }}>{label}</span>
          </span>
        );
      })}
    </div>
  );
};
