import { theme } from '../../styles/theme';

export interface FilterDef {
  key: string;
  label: string;
  type: 'select' | 'range' | 'toggle';
  options?: string[];
}

interface FilterBarProps {
  filters: FilterDef[];
  values: Record<string, unknown>;
  onChange: (key: string, value: unknown) => void;
}

const inputStyle: React.CSSProperties = {
  padding: '4px 8px',
  borderRadius: 4,
  border: `1px solid ${theme.border}`,
  background: theme.bg.tertiary,
  color: theme.text.primary,
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
};

export function FilterBar({ filters, values, onChange }: FilterBarProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 16,
        padding: '8px 12px',
        background: theme.bg.secondary,
        border: `1px solid ${theme.border}`,
        borderRadius: 8,
        flexWrap: 'wrap',
      }}
    >
      {filters.map(f => (
        <div key={f.key} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <label
            style={{
              fontSize: 11,
              fontWeight: 600,
              color: theme.text.secondary,
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
            }}
          >
            {f.label}
          </label>

          {f.type === 'select' && (
            <select
              value={String(values[f.key] ?? '')}
              onChange={(e) => onChange(f.key, e.target.value)}
              style={{
                ...inputStyle,
                cursor: 'pointer',
              }}
            >
              <option value="">All</option>
              {f.options?.map(opt => (
                <option key={opt} value={opt}>
                  {opt}
                </option>
              ))}
            </select>
          )}

          {f.type === 'range' && (
            <input
              type="number"
              value={values[f.key] !== undefined ? String(values[f.key]) : ''}
              onChange={(e) => onChange(f.key, e.target.value ? Number(e.target.value) : undefined)}
              placeholder="--"
              style={{
                ...inputStyle,
                width: 64,
              }}
            />
          )}

          {f.type === 'toggle' && (
            <button
              onClick={() => onChange(f.key, !values[f.key])}
              style={{
                padding: '4px 10px',
                borderRadius: 4,
                border: `1px solid ${values[f.key] ? theme.accent : theme.border}`,
                background: values[f.key] ? `${theme.accent}22` : theme.bg.tertiary,
                color: values[f.key] ? theme.accent : theme.text.secondary,
                fontSize: 12,
                cursor: 'pointer',
                fontFamily: 'inherit',
                fontWeight: 500,
              }}
            >
              {values[f.key] ? 'On' : 'Off'}
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
