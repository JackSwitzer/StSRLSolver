import { usePolling } from './usePolling';
import type { TrainingRunManifest, TrainingEvent, TrainingMetric, BenchmarkReport, FrontierReport, CorpusCell, RunFormat } from '../types/artifacts';

export function useRunFormat() {
  return usePolling<{ format: RunFormat }>('/api/format', 10000);
}
export function useManifest() {
  return usePolling<TrainingRunManifest>('/api/manifest', 30000);
}
export function useTrainingEvents() {
  return usePolling<TrainingEvent[]>('/api/events', 5000);
}
export function useTrainingMetrics() {
  return usePolling<TrainingMetric[]>('/api/training-metrics', 5000);
}
export function useBenchmark() {
  return usePolling<BenchmarkReport>('/api/benchmark', 30000);
}
export function useFrontier() {
  return usePolling<FrontierReport>('/api/frontier', 30000);
}
export function useCorpusMatrix() {
  return usePolling<CorpusCell[]>('/api/corpus-matrix', 10000);
}
