import { usePolling } from './usePolling';
import { useState, useEffect } from 'react';
import type { Episode } from '../types/episode';
import { ENDPOINTS } from '../api/endpoints';
import { fetchJson } from '../api/client';

export function useEpisodeList() {
  return usePolling<Episode[]>(ENDPOINTS.episodes, 10000);
}

export function useEpisodeDetail(seed: string | null) {
  const [episode, setEpisode] = useState<Episode | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!seed) {
      setEpisode(null);
      return;
    }
    setLoading(true);
    fetchJson<Episode>(ENDPOINTS.episode(seed)).then(ep => {
      setEpisode(ep);
      setLoading(false);
    });
  }, [seed]);

  return { episode, loading };
}
