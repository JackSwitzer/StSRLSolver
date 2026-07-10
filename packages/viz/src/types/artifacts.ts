export type RunFormat = 'v1' | 'v2' | 'empty';

export interface TrainingRunManifest {
  runId: string;
  createdAt: string;
  git: { commitSha: string; branch: string; dirty: boolean };
  config: { values: Record<string, unknown>; configHash: string };
  host: string;
  tags: string[];
  notes: string[];
  overnightSearch?: {
    sweepConfig: string;
    searchPolicy: string;
    plannedGames: number;
    workerCount: number;
    corpusName: string;
    corpusSlices: string[];
    budget: { frontierWidth: number; nodeBudget: number; rolloutBudget: number; timeLimitMs: number };
  };
}

export interface CombatOutcome {
  solveProbability: number;
  expectedHpLoss: number;
  expectedTurns: number;
  potionCost: number;
  setupValueDelta: number;
  persistentScalingDelta: number;
}

export interface FrontierLine {
  lineIndex: number;
  actionPrefix: number[];
  visits: number;
  expandedNodes: number;
  elapsedMs: number;
  outcome: CombatOutcome;
}

export interface FrontierPoint {
  label: string;
  winRate: number;
  avgFloor: number;
  throughputGpm: number;
  deckFamily: string;
  removeCount: number;
  potionSet: string;
  enemy: string;
}

export interface FrontierGroupSummary {
  key: { deckFamily: string; removeCount: number; potionSet: string; enemy: string };
  labels: string[];
  count: number;
  meanWinRate: number;
  meanAvgFloor: number;
  meanThroughputGpm: number;
}

export interface FrontierReport {
  points: FrontierPoint[];
  frontier: string[];
  ranking: string[];
  bestByMetric: Record<string, string>;
  weights: { winRate: number; avgFloor: number; throughputGpm: number };
  groups: FrontierGroupSummary[];
}

export interface BenchmarkSlice {
  sliceName: string;
  cases: number;
  solveRate: number;
  expectedHpLoss: number;
  expectedTurns: number;
  oracleTopKAgreement: number;
  p95ElapsedMs: number;
  p95RssGb: number;
}

export interface BenchmarkReport { slices: BenchmarkSlice[]; }

export interface TrainingMetric {
  timestamp: string;
  name: string;
  value: number;
  step: number;
  config: string;
  deckFamily?: string;
  removeCount?: number;
  potionSet?: string;
  enemy?: string;
  corpusSlice?: string;
}

export interface TrainingEvent {
  timestamp: string;
  eventType: string;
  phase?: string;
  epochIndex?: number;
  accuracy?: number;
  requestCount?: number;
  updatedExamples?: number;
}

export interface CorpusCell {
  deckFamily: string;
  enemy: string;
  solveRate: number;
  avgHpLoss: number;
  avgTurns: number;
  count: number;
}
