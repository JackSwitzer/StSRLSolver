export const theme = {
  bg: {
    primary: '#1a2332',      // soft navy blue
    secondary: '#1f2b3d',    // slightly lighter navy
    tertiary: '#253448',     // card/panel bg
    hover: '#2d3f55',        // hover state
  },
  text: {
    primary: '#ffffff',      // white for main text
    secondary: '#a8b8cc',    // muted blue-grey
    muted: '#5a6f88',        // subtle labels
  },
  border: '#2d3f55',
  accent: '#87ceeb',         // sky blue accent
  success: '#2ecc71',        // emerald
  warning: '#e67e22',        // burnt orange
  danger: '#8b0000',         // blood red
  chart: {
    blue: '#87ceeb',         // spring sky blue
    green: '#2ecc71',        // emerald green
    red: '#8b0000',          // blood red (deaths)
    yellow: '#daa520',       // gold
    purple: '#7b2d8e',       // royal purple
    orange: '#cc5500',       // burnt orange
    teal: '#20b2aa',         // turquoise
    forest: '#228b22',       // forest green
    gold: '#ffd700',         // treasure gold
    skyBlue: '#87ceeb',      // spring sky blue
  },
} as const;
