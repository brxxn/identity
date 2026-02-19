import { useCallback, useState } from 'react';

/**
 * Simple hook to call an async API function and track state.
 * Usage: const { data, loading, error, call } = useApi(() => api.get('/whoami'))
 */
export function useApi<T>(fn: () => Promise<T>, deps: any[] = []) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<any>(null);

  const call = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fn();
      setData(res);
      return res;
    } catch (e) {
      setError(e);
      throw e;
    } finally {
      setLoading(false);
    }
  }, deps);

  return { data, loading, error, call } as const;
}

export default useApi;
