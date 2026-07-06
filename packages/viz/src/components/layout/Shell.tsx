import { type ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import { theme } from '../../styles/theme';
import { useTrainingStatus } from '../../hooks/useTrainingStatus';
import { useRunFormat } from '../../hooks/useArtifacts';

const navItems = [
  { to: '/', label: 'Dashboard' },
  { to: '/episodes', label: 'Episodes' },
  { to: '/analysis', label: 'Analysis' },
  { to: '/corpus', label: 'Corpus' },
];

export function Shell({ children }: { children: ReactNode }) {
  const { data: status, stale } = useTrainingStatus();
  const { data: formatData } = useRunFormat();

  return (
    <div style={{
      display: 'flex',
      height: '100vh',
      overflow: 'hidden',
    }}>
      {/* Sidebar */}
      <aside style={{
        width: 200,
        flexShrink: 0,
        display: 'flex',
        flexDirection: 'column',
        background: theme.bg.secondary,
        borderRight: `1px solid ${theme.border}`,
      }}>
        <div style={{
          padding: '16px 16px 12px',
          borderBottom: `1px solid ${theme.border}`,
        }}>
          <span style={{
            fontWeight: 700,
            fontSize: 15,
            color: theme.text.primary,
            letterSpacing: '-0.5px',
          }}>
            Spire Monitor
          </span>
        </div>
        <nav style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 2,
          padding: '8px',
        }}>
          {navItems.map(item => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              style={({ isActive }) => ({
                padding: '8px 12px',
                borderRadius: 6,
                fontSize: 13,
                fontWeight: 500,
                color: isActive ? theme.text.primary : theme.text.secondary,
                background: isActive ? theme.bg.tertiary : 'transparent',
                textDecoration: 'none',
                transition: 'background 150ms, color 150ms',
              })}
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>

      {/* Main area */}
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>
        {/* Header */}
        <header style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 24px',
          height: 48,
          borderBottom: `1px solid ${theme.border}`,
          background: theme.bg.secondary,
          flexShrink: 0,
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <span style={{
              fontWeight: 600,
              fontSize: 14,
              color: theme.text.primary,
            }}>
              Spire Monitor
            </span>
            <span
              style={{
                display: 'inline-block',
                width: 8,
                height: 8,
                borderRadius: '50%',
                background: stale ? theme.warning : theme.success,
                boxShadow: stale ? 'none' : `0 0 6px ${theme.success}88`,
              }}
              title={stale ? 'Data is stale' : 'Live'}
            />
            {formatData?.format && (
              <span style={{
                fontSize: 10,
                fontWeight: 600,
                padding: '2px 6px',
                borderRadius: 4,
                background: formatData.format === 'v2' ? theme.accent + '22' : theme.bg.tertiary,
                color: formatData.format === 'v2' ? theme.accent : theme.text.muted,
                border: `1px solid ${formatData.format === 'v2' ? theme.accent + '44' : theme.border}`,
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
              }}>
                {formatData.format}
              </span>
            )}
          </div>
          {status?.configName && (
            <span style={{
              fontSize: 12,
              color: theme.text.secondary,
              fontWeight: 500,
            }}>
              {status.configName}
            </span>
          )}
        </header>

        {/* Content */}
        <main style={{
          flex: 1,
          overflow: 'auto',
          padding: 24,
        }}>
          {children}
        </main>
      </div>
    </div>
  );
}
