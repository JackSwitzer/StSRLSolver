const API_BASE = import.meta.env.VITE_API_BASE ?? '';

export async function fetchJson<T>(url: string): Promise<T | null> {
  try {
    const fullUrl = API_BASE ? `${API_BASE}${url}` : url;
    const res = await fetch(fullUrl);
    if (!res.ok) return null;
    const ct = res.headers.get('content-type') ?? '';
    if (!ct.includes('application/json')) return null;
    return await res.json();
  } catch {
    return null;
  }
}
