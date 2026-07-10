import { theme } from '../../styles/theme';
import type { Intent } from '../../types/engine';

interface IntentIconProps {
  intent: Intent;
}

function SwordSvg({ color = theme.danger }: { color?: string }) {
  return (
    <path
      d="M4,2 L10,8 M10,2 L4,8 M7,10 L7,16 M4,13 L10,13"
      stroke={color}
      strokeWidth="1.5"
      fill="none"
      strokeLinecap="round"
    />
  );
}

function ShieldSvg({ color = theme.chart.blue }: { color?: string }) {
  return (
    <path
      d="M7,2 C4,2 2,4 2,4 L2,9 C2,13 7,16 7,16 C7,16 12,13 12,9 L12,4 C12,4 10,2 7,2 Z"
      stroke={color}
      strokeWidth="1.5"
      fill={`${color}22`}
      strokeLinejoin="round"
    />
  );
}

function ArrowUpSvg({ color = theme.success }: { color?: string }) {
  return (
    <path
      d="M7,14 L7,4 M3,8 L7,4 L11,8"
      stroke={color}
      strokeWidth="1.5"
      fill="none"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}

function ArrowDownSvg({ color = theme.chart.purple }: { color?: string }) {
  return (
    <path
      d="M7,4 L7,14 M3,10 L7,14 L11,10"
      stroke={color}
      strokeWidth="1.5"
      fill="none"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  );
}

function DamageLabel({ damage, hits, x = 14 }: { damage: number; hits: number; x?: number }) {
  const text = hits > 1 ? `${damage}x${hits}` : `${damage}`;
  return (
    <text
      x={x}
      y={12}
      fill={theme.danger}
      fontSize="8"
      fontWeight="700"
      fontFamily="monospace"
      textAnchor="start"
    >
      {text}
    </text>
  );
}

function BlockLabel({ amount, x = 14 }: { amount: number; x?: number }) {
  return (
    <text
      x={x}
      y={12}
      fill={theme.chart.blue}
      fontSize="8"
      fontWeight="700"
      fontFamily="monospace"
      textAnchor="start"
    >
      {amount}
    </text>
  );
}

export function IntentIcon({ intent }: IntentIconProps) {
  const kind = intent.kind;
  let width = 20;

  if (kind === 'attack' || kind === 'attack_buff' || kind === 'attack_debuff' || kind === 'attack_block') {
    width = 36;
  } else if (kind === 'block' || kind === 'defend_buff') {
    width = 30;
  }

  return (
    <svg width={width} height={20} viewBox={`0 0 ${width} 18`} style={{ verticalAlign: 'middle' }}>
      {kind === 'attack' && (
        <>
          <g transform="translate(0,0)">
            <SwordSvg />
          </g>
          <DamageLabel damage={(intent as { damage: number }).damage} hits={(intent as { hits: number }).hits} />
        </>
      )}

      {kind === 'block' && (
        <>
          <g transform="translate(0,0)">
            <ShieldSvg />
          </g>
          <BlockLabel amount={(intent as { amount: number }).amount} />
        </>
      )}

      {kind === 'buff' && (
        <g transform="translate(3,0)">
          <ArrowUpSvg />
        </g>
      )}

      {kind === 'debuff' && (
        <g transform="translate(3,0)">
          <ArrowDownSvg />
        </g>
      )}

      {kind === 'attack_block' && (
        <>
          <g transform="translate(0,0)">
            <SwordSvg />
          </g>
          <g transform="translate(12,0)" opacity="0.7">
            <ShieldSvg />
          </g>
          <DamageLabel damage={(intent as { damage: number }).damage} hits={(intent as { hits: number }).hits} x={26} />
        </>
      )}

      {kind === 'attack_buff' && (
        <>
          <g transform="translate(0,0)">
            <SwordSvg />
          </g>
          <g transform="translate(12,0)" opacity="0.7">
            <ArrowUpSvg />
          </g>
          <DamageLabel damage={(intent as { damage: number }).damage} hits={(intent as { hits: number }).hits} x={26} />
        </>
      )}

      {kind === 'attack_debuff' && (
        <>
          <g transform="translate(0,0)">
            <SwordSvg />
          </g>
          <g transform="translate(12,0)" opacity="0.7">
            <ArrowDownSvg />
          </g>
          <DamageLabel damage={(intent as { damage: number }).damage} hits={(intent as { hits: number }).hits} x={26} />
        </>
      )}

      {kind === 'defend_buff' && (
        <>
          <g transform="translate(0,0)">
            <ShieldSvg />
          </g>
          <g transform="translate(12,0)" opacity="0.7">
            <ArrowUpSvg />
          </g>
        </>
      )}

      {kind === 'spawn' && (
        <text x="7" y="13" fill={theme.warning} fontSize="14" textAnchor="middle" fontWeight="700">+</text>
      )}

      {kind === 'escape' && (
        <path
          d="M5,9 L12,4 M12,4 L12,14 M12,14 L5,9"
          stroke={theme.text.muted}
          strokeWidth="1.5"
          fill="none"
          strokeLinejoin="round"
          transform="translate(0,0)"
        />
      )}

      {kind === 'sleep' && (
        <text x="7" y="13" fill={theme.text.muted} fontSize="12" textAnchor="middle" fontFamily="serif" fontStyle="italic">z</text>
      )}

      {kind === 'stun' && (
        <text x="7" y="14" fill={theme.warning} fontSize="14" textAnchor="middle" fontWeight="700">*</text>
      )}

      {kind === 'unknown' && (
        <text x="7" y="14" fill={theme.text.muted} fontSize="14" textAnchor="middle" fontWeight="700">?</text>
      )}
    </svg>
  );
}
