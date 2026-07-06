import { useState, useMemo } from 'react';
import { theme } from '../../../styles/theme';
import { useEpisodeList } from '../../../hooks/useEpisodes';
import { EpisodeDetail } from './EpisodeDetail';
import type { Episode } from '../../../types/episode';

type SortKey = 'floor' | 'won' | 'seed' | 'deathEnemy' | 'durationMs';
type SortDir = 'asc' | 'desc';

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const s = ms / 1000;
  if (s < 60) return `${s.toFixed(1)}s`;
  const m = Math.floor(s / 60);
  const rem = Math.floor(s % 60);
  return `${m}m ${rem}s`;
}

export function EpisodesView() {
  const { data: episodes, loading, stale } = useEpisodeList();
  const [selectedSeed, setSelectedSeed] = useState<string | null>(null);
  const [sortKey, setSortKey] = useState<SortKey>('floor');
  const [sortDir, setSortDir] = useState<SortDir>('desc');

  const sorted = useMemo(() => {
    if (!episodes) return [];
    return [...episodes].sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case 'floor': cmp = a.floor - b.floor; break;
        case 'won': cmp = (a.won ? 1 : 0) - (b.won ? 1 : 0); break;
        case 'seed': cmp = a.seed.localeCompare(b.seed); break;
        case 'deathEnemy': cmp = (a.deathEnemy ?? '').localeCompare(b.deathEnemy ?? ''); break;
        case 'durationMs': cmp = a.durationMs - b.durationMs; break;
      }
      return sortDir === 'asc' ? cmp : -cmp;
    });
  }, [episodes, sortKey, sortDir]);

  function handleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    } else {
      setSortKey(key);
      setSortDir('desc');
    }
  }

  function SortHeader({ k, label, width }: { k: SortKey; label: string; width?: number }) {
    const active = sortKey === k;
    return (
      <th
        onClick={() => handleSort(k)}
        style={{
          cursor: 'pointer',
          color: active ? theme.text.primary : theme.text.secondary,
          fontSize: 12,
          fontWeight: 600,
          userSelect: 'none',
          width,
          padding: '8px 12px',
          borderBottom: `1px solid ${theme.border}`,
        }}
      >
        {label} {active ? (sortDir === 'asc' ? '\u25B2' : '\u25BC') : ''}
      </th>
    );
  }

  if (loading && !episodes) {
    return (
      <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
        Loading episodes...
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', gap: 16, height: 'calc(100vh - 96px)' }}>
      {/* Left: Episode list */}
      <div style={{
        flex: selectedSeed ? '0 0 420px' : '1',
        overflow: 'auto',
        background: theme.bg.secondary,
        border: `1px solid ${theme.border}`,
        borderRadius: 8,
        transition: 'flex 200ms ease',
      }}>
        {stale && (
          <div style={{
            padding: '6px 12px',
            fontSize: 11,
            color: theme.warning,
            background: theme.warning + '11',
            borderBottom: `1px solid ${theme.border}`,
          }}>
            Data may be stale
          </div>
        )}
        {!episodes || episodes.length === 0 ? (
          <div style={{ padding: 40, textAlign: 'center', color: theme.text.muted }}>
            No episodes recorded yet
          </div>
        ) : (
          <table>
            <thead>
              <tr>
                <SortHeader k="floor" label="Floor" width={60} />
                <SortHeader k="won" label="Result" width={70} />
                <SortHeader k="seed" label="Seed" />
                <SortHeader k="deathEnemy" label="Death Enemy" />
                <SortHeader k="durationMs" label="Duration" width={90} />
              </tr>
            </thead>
            <tbody>
              {sorted.map((ep: Episode) => (
                <tr
                  key={ep.seed}
                  onClick={() => setSelectedSeed(ep.seed === selectedSeed ? null : ep.seed)}
                  style={{
                    cursor: 'pointer',
                    background: ep.seed === selectedSeed ? theme.bg.tertiary : 'transparent',
                    borderBottom: `1px solid ${theme.border}`,
                  }}
                  onMouseEnter={e => {
                    if (ep.seed !== selectedSeed) {
                      (e.currentTarget as HTMLElement).style.background = theme.bg.hover;
                    }
                  }}
                  onMouseLeave={e => {
                    if (ep.seed !== selectedSeed) {
                      (e.currentTarget as HTMLElement).style.background = 'transparent';
                    }
                  }}
                >
                  <td style={{ fontSize: 13, fontWeight: 600, color: theme.text.primary, padding: '8px 12px' }}>
                    {ep.floor}
                  </td>
                  <td style={{ padding: '8px 12px' }}>
                    <span style={{
                      fontSize: 11,
                      fontWeight: 600,
                      padding: '2px 8px',
                      borderRadius: 4,
                      background: ep.won ? theme.success + '22' : theme.danger + '22',
                      color: ep.won ? theme.success : theme.danger,
                    }}>
                      {ep.won ? 'WIN' : 'LOSS'}
                    </span>
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.secondary, fontFamily: 'monospace', padding: '8px 12px' }}>
                    {ep.seed.slice(0, 10)}
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.secondary, padding: '8px 12px' }}>
                    {ep.deathEnemy ?? '-'}
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.muted, padding: '8px 12px' }}>
                    {formatDuration(ep.durationMs)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Right: Episode detail */}
      {selectedSeed && (
        <div style={{
          flex: 1,
          overflow: 'auto',
          background: theme.bg.secondary,
          border: `1px solid ${theme.border}`,
          borderRadius: 8,
        }}>
          <EpisodeDetail seed={selectedSeed} onClose={() => setSelectedSeed(null)} />
        </div>
      )}
    </div>
  );
}
