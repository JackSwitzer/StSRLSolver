import type { RunPhase } from './engine';

export interface TrainingStatus {
  timestamp: string;
  elapsedHours: number;
  totalGames: number;
  totalWins: number;
  winRate: number;
  avgFloor: number;
  peakFloor: number;
  gamesPerMin: number;
  trainSteps: number;
  loss: { total: number; policy: number; value: number };
  entropy: number;
  diagnostics: {
    explainedVariance: number;
    klDivergence: number;
    meanAdvantage: number;
    clipFraction: number;
  };
  configName: string;
  gpuPercent: number | null;
}

export interface MetricsSnapshot {
  step: number;
  games: number;
  avgFloor: number;
  peakFloor: number;
  winRate: number;
  loss: { total: number; policy: number; value: number };
  entropy: number;
  timestamp: string;
}

export interface WorkerStatus {
  name: string;
  seed: string;
  floor: number;
  phase: RunPhase;
  hp: number;
  maxHp: number;
  enemy: string;
  lastUpdate: number;
}
