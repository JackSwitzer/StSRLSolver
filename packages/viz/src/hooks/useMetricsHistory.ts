import { usePolling } from './usePolling';
import type { MetricsSnapshot } from '../types/training';
import { ENDPOINTS } from '../api/endpoints';

export function useMetricsHistory() {
  return usePolling<MetricsSnapshot[]>(ENDPOINTS.metrics, 30000);
}
