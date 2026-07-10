import { theme } from '../../styles/theme';

interface HPBarProps {
  hp: number;
  maxHp: number;
  size?: 'sm' | 'md' | 'lg';
}

const SIZE_MAP = {
  sm: { height: 8, fontSize: 0, borderRadius: 4 },
  md: { height: 18, fontSize: 11, borderRadius: 6 },
  lg: { height: 24, fontSize: 13, borderRadius: 8 },
} as const;

function hpColor(ratio: number): string {
  if (ratio > 0.66) return theme.success;
  if (ratio > 0.33) return theme.warning;
  return theme.danger;
}

export function HPBar({ hp, maxHp, size = 'md' }: HPBarProps) {
  const ratio = maxHp > 0 ? Math.max(0, Math.min(1, hp / maxHp)) : 0;
  const { height, fontSize, borderRadius } = SIZE_MAP[size];
  const showText = size !== 'sm';

  return (
    <div
      style={{
        position: 'relative',
        width: '100%',
        height,
        borderRadius,
        background: theme.bg.tertiary,
        overflow: 'hidden',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          height: '100%',
          width: `${ratio * 100}%`,
          background: hpColor(ratio),
          borderRadius,
          transition: 'width 300ms ease, background 300ms ease',
        }}
      />
      {showText && (
        <span
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize,
            fontWeight: 600,
            color: theme.text.primary,
            textShadow: '0 1px 2px rgba(0,0,0,0.6)',
            lineHeight: 1,
          }}
        >
          {hp} / {maxHp}
        </span>
      )}
    </div>
  );
}
