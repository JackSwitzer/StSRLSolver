import type { RoomType } from '../../types/engine';
import { ROOM_COLORS } from '../../types/engine';
import { theme } from '../../styles/theme';

interface RoomIconProps {
  roomType: RoomType;
  active?: boolean;
  size?: number;
}

function RoomSvgContent({ roomType }: { roomType: RoomType }) {
  switch (roomType) {
    case 'monster':
      // Crossed swords
      return (
        <g>
          <path d="M4,4 L12,12 M12,4 L4,12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        </g>
      );
    case 'elite':
      // Shield
      return (
        <path
          d="M8,3 C5,3 3,5 3,5 L3,8 C3,12 8,14 8,14 C8,14 13,12 13,8 L13,5 C13,5 11,3 8,3 Z"
          stroke="currentColor"
          strokeWidth="1.5"
          fill="none"
          strokeLinejoin="round"
        />
      );
    case 'boss':
      // Skull
      return (
        <g>
          <circle cx="8" cy="7" r="4.5" stroke="currentColor" strokeWidth="1.5" fill="none" />
          <circle cx="6.5" cy="6.5" r="1" fill="currentColor" />
          <circle cx="9.5" cy="6.5" r="1" fill="currentColor" />
          <path d="M6,10 L7,9 L8,10 L9,9 L10,10" stroke="currentColor" strokeWidth="1" fill="none" />
        </g>
      );
    case 'rest':
      // Fire
      return (
        <path
          d="M8,3 C6,6 4,8 4,10 C4,12.5 5.8,14 8,14 C10.2,14 12,12.5 12,10 C12,8 10,6 8,3 Z"
          stroke="currentColor"
          strokeWidth="1.5"
          fill="none"
          strokeLinejoin="round"
        />
      );
    case 'shop':
      // Dollar sign
      return (
        <g>
          <path d="M8,3 L8,14 M5.5,6 C5.5,4.5 7,4 8,4 C9,4 10.5,4.5 10.5,6 C10.5,7.5 5.5,8 5.5,10 C5.5,11.5 7,12 8,12 C9,12 10.5,11.5 10.5,10" stroke="currentColor" strokeWidth="1.5" fill="none" strokeLinecap="round" />
        </g>
      );
    case 'event':
      // Question mark
      return (
        <g>
          <path d="M6,6 C6,4 7,3 8,3 C9,3 10.5,4 10.5,5.5 C10.5,7 8,7.5 8,9.5" stroke="currentColor" strokeWidth="1.5" fill="none" strokeLinecap="round" />
          <circle cx="8" cy="12" r="1" fill="currentColor" />
        </g>
      );
    case 'treasure':
      // Chest
      return (
        <g>
          <rect x="4" y="6" width="8" height="6" rx="1" stroke="currentColor" strokeWidth="1.5" fill="none" />
          <path d="M4,9 L12,9" stroke="currentColor" strokeWidth="1.5" />
          <circle cx="8" cy="10.5" r="0.8" fill="currentColor" />
        </g>
      );
  }
}

export function RoomIcon({ roomType, active, size = 20 }: RoomIconProps) {
  const color = ROOM_COLORS[roomType];

  return (
    <div
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: size,
        height: size,
        borderRadius: '50%',
        background: active ? `${color}33` : theme.bg.tertiary,
        border: `1.5px solid ${active ? color : `${color}66`}`,
        color,
        boxShadow: active ? `0 0 6px ${color}44` : 'none',
        transition: 'all 200ms ease',
      }}
    >
      <svg width={size * 0.7} height={size * 0.7} viewBox="0 0 16 16">
        <RoomSvgContent roomType={roomType} />
      </svg>
    </div>
  );
}
