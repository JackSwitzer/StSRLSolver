import { useMemo } from 'react';

// ---- Types ----

interface ConfigEntry {
  label: string;
  config: Record<string, any>;
}

interface ConfigDiffProps {
  configs: ConfigEntry[];
}

// ---- Constants ----

const DISPLAY_KEYS = [
  'entropy_coeff',
  'temperature',
  'lr',
  'batch_size',
  'turn_solver_ms',
  'epsilon_mode',
  'gamma',
  'num_workers',
  'mcts_sims',
  'hidden_dim',
  'num_blocks',
  'clip_epsilon',
  'value_loss_coeff',
  'max_grad_norm',
];

const BORDER = '#21262d';
const ACCENT = '#00ff41';
const TEXT = '#c9d1d9';
const SECONDARY = '#8b949e';
const DIFF_BG = 'rgba(0,255,65,0.06)';

// ---- Helpers ----

function fmtValue(v: any): string {
  if (v === undefined || v === null) return '--';
  if (typeof v === 'number') {
    if (Number.isInteger(v)) return String(v);
    if (Math.abs(v) < 0.001) return v.toExponential(2);
    return v.toPrecision(4);
  }
  if (typeof v === 'boolean') return v ? 'true' : 'false';
  return String(v);
}

function valuesMatch(a: any, b: any): boolean {
  return fmtValue(a) === fmtValue(b);
}

// ---- Component ----

export const ConfigDiff = ({ configs }: ConfigDiffProps) => {
  // Gather all keys present in any config, filtered to display keys if they exist
  const rows = useMemo(() => {
    if (configs.length === 0) return [];

    // Collect all keys across all configs
    const allKeys = new Set<string>();
    for (const entry of configs) {
      for (const key of Object.keys(entry.config)) {
        allKeys.add(key);
      }
    }

    // Prefer display keys order, then alphabetical for extras
    const orderedKeys: string[] = [];
    for (const dk of DISPLAY_KEYS) {
      if (allKeys.has(dk)) orderedKeys.push(dk);
    }
    for (const k of [...allKeys].sort()) {
      if (!orderedKeys.includes(k)) orderedKeys.push(k);
    }

    return orderedKeys.map((key) => {
      const values = configs.map((c) => c.config[key]);
      // Check if values differ across configs
      const isDiff = configs.length >= 2 && !values.every((v) => valuesMatch(v, values[0]));
      return { key, values, isDiff };
    });
  }, [configs]);

  if (configs.length === 0) {
    return (
      <div style={{
        padding: '16px',
        textAlign: 'center',
        color: '#3d444d',
        fontSize: '11px',
        fontFamily: "'JetBrains Mono', monospace",
      }}>
        No config data
      </div>
    );
  }

  const isSingle = configs.length === 1;

  return (
    <div style={{ overflow: 'auto' }}>
      {/* Header */}
      <div style={{
        fontSize: '9px',
        color: SECONDARY,
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        fontWeight: 600,
        marginBottom: '6px',
      }}>
        Config {isSingle ? '' : 'Diff'}
      </div>

      <table style={{
        width: '100%',
        borderCollapse: 'collapse',
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        fontSize: '10px',
      }}>
        <thead>
          <tr>
            <th style={{
              textAlign: 'left',
              padding: '4px 8px',
              fontSize: '9px',
              color: SECONDARY,
              borderBottom: `1px solid ${BORDER}`,
              fontWeight: 600,
            }}>
              Key
            </th>
            {configs.map((c, i) => (
              <th key={i} style={{
                textAlign: 'right',
                padding: '4px 8px',
                fontSize: '9px',
                color: SECONDARY,
                borderBottom: `1px solid ${BORDER}`,
                fontWeight: 600,
                whiteSpace: 'nowrap',
              }}>
                {c.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr
              key={row.key}
              style={{
                background: row.isDiff ? DIFF_BG : 'transparent',
              }}
            >
              <td style={{
                padding: '3px 8px',
                color: row.isDiff ? ACCENT : SECONDARY,
                fontWeight: row.isDiff ? 600 : 400,
                borderBottom: `1px solid ${BORDER}`,
                whiteSpace: 'nowrap',
              }}>
                {row.isDiff && (
                  <span style={{ color: ACCENT, marginRight: '4px', fontSize: '8px' }}>*</span>
                )}
                {row.key}
              </td>
              {row.values.map((v, i) => (
                <td key={i} style={{
                  padding: '3px 8px',
                  textAlign: 'right',
                  color: row.isDiff ? ACCENT : TEXT,
                  fontWeight: row.isDiff ? 600 : 400,
                  borderBottom: `1px solid ${BORDER}`,
                  whiteSpace: 'nowrap',
                }}>
                  {fmtValue(v)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* Diff summary */}
      {!isSingle && (
        <div style={{
          marginTop: '6px',
          fontSize: '9px',
          color: '#3d444d',
          fontFamily: "'JetBrains Mono', monospace",
        }}>
          {rows.filter((r) => r.isDiff).length} differences found
        </div>
      )}
    </div>
  );
};
