const BASE = '/api';
export const ENDPOINTS = {
  status: `${BASE}/status`,
  floorCurve: `${BASE}/floor-curve`,
  metrics: `${BASE}/metrics`,
  episodes: `${BASE}/episodes`,
  episode: (seed: string) => `${BASE}/episode/${seed}`,
  workers: `${BASE}/workers`,
  runs: `${BASE}/runs`,
} as const;
