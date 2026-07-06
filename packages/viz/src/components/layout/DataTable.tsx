import { useState, type ReactNode } from 'react';
import { theme } from '../../styles/theme';

export interface Column<T> {
  key: string;
  header: string;
  render?: (row: T) => ReactNode;
  sortable?: boolean;
  width?: string;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  onRowClick?: (row: T) => void;
  selectedIndex?: number;
}

export function DataTable<T extends Record<string, unknown>>({
  columns,
  data,
  onRowClick,
  selectedIndex,
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDesc, setSortDesc] = useState(true);

  const handleSort = (key: string) => {
    if (sortKey === key) {
      setSortDesc(!sortDesc);
    } else {
      setSortKey(key);
      setSortDesc(true);
    }
  };

  const sorted = [...data];
  if (sortKey) {
    sorted.sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (av == null && bv == null) return 0;
      if (av == null) return 1;
      if (bv == null) return -1;
      if (typeof av === 'number' && typeof bv === 'number') {
        return sortDesc ? bv - av : av - bv;
      }
      const cmp = String(av).localeCompare(String(bv));
      return sortDesc ? -cmp : cmp;
    });
  }

  return (
    <div style={{ overflowX: 'auto' }}>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            {columns.map(col => (
              <th
                key={col.key}
                onClick={col.sortable ? () => handleSort(col.key) : undefined}
                style={{
                  padding: '8px 12px',
                  textAlign: 'left',
                  fontSize: 11,
                  fontWeight: 600,
                  color: theme.text.secondary,
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px',
                  borderBottom: `1px solid ${theme.border}`,
                  cursor: col.sortable ? 'pointer' : 'default',
                  userSelect: 'none',
                  whiteSpace: 'nowrap',
                  width: col.width,
                }}
              >
                {col.header}
                {col.sortable && sortKey === col.key && (
                  <span style={{ marginLeft: 4, opacity: 0.7 }}>
                    {sortDesc ? '\u2193' : '\u2191'}
                  </span>
                )}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => (
            <tr
              key={i}
              onClick={onRowClick ? () => onRowClick(row) : undefined}
              style={{
                cursor: onRowClick ? 'pointer' : 'default',
                background: selectedIndex === i ? theme.bg.tertiary : 'transparent',
                transition: 'background 100ms ease',
              }}
              onMouseEnter={(e) => {
                if (selectedIndex !== i) {
                  e.currentTarget.style.background = theme.bg.hover;
                }
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background =
                  selectedIndex === i ? theme.bg.tertiary : 'transparent';
              }}
            >
              {columns.map(col => (
                <td
                  key={col.key}
                  style={{
                    padding: '8px 12px',
                    fontSize: 13,
                    color: theme.text.primary,
                    borderBottom: `1px solid ${theme.border}22`,
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    maxWidth: col.width || 'none',
                  }}
                >
                  {col.render
                    ? col.render(row)
                    : String(row[col.key] ?? '')}
                </td>
              ))}
            </tr>
          ))}
          {sorted.length === 0 && (
            <tr>
              <td
                colSpan={columns.length}
                style={{
                  padding: '24px 12px',
                  textAlign: 'center',
                  color: theme.text.muted,
                  fontSize: 13,
                }}
              >
                No data
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
