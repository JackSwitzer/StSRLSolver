import { usePolling } from './usePolling';
import type { TrainingStatus } from '../types/training';
import { ENDPOINTS } from '../api/endpoints';

export function useTrainingStatus() {
  return usePolling<TrainingStatus>(ENDPOINTS.status, 2500);
}
