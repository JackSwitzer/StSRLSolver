import { usePolling } from './usePolling';
import type { WorkerStatus } from '../types/training';
import { ENDPOINTS } from '../api/endpoints';

export function useWorkers() {
  return usePolling<WorkerStatus[]>(ENDPOINTS.workers, 2500);
}
