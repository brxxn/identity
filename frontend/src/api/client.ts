export type ApiSuccess<T> = {
  success: true;
  data: T;
};

export type ApiError = {
  success: false;
  status: number;
  code: string;
  message: string;
};

export type ApiResult<T> = ApiSuccess<T> | ApiError;

const API_ORIGIN = import.meta.env.VITE_API_ORIGIN || '';
const API_BASE = `${API_ORIGIN}/v1`;

import { getAuthTokenSync, setTokensSync, getRefreshTokenSync, clearTokensSync } from '../store/auth';
import { refreshAuthToken } from './contollers/login';

// Keep exported helpers for other code to set/get token via the shared store
export function setAuthToken(token: string | null, refreshToken: string | null) {
  setTokensSync(token, refreshToken);
}

export function getAuthToken() {
  return getAuthTokenSync();
}

let isRefreshing = false;
let refreshPromise: Promise<ApiResult<void>> | null = null;

async function refreshToken(): Promise<ApiResult<void>> {
  if (isRefreshing && refreshPromise) return refreshPromise;

  isRefreshing = true;
  const refreshToken = getRefreshTokenSync();
  if (!refreshToken) {
    isRefreshing = false;
    clearTokensSync();
    return { success: false, status: 401, code: 'no_refresh_token', message: 'No refresh token available' };
  }

  refreshPromise = (async () => {
    const result = await refreshAuthToken({
      refresh_token: refreshToken
    });

    if (!result.success) {
      clearTokensSync();
      return result;
    }

    setTokensSync(result.data.access_token, result.data.refresh_token);
    return { success: true, data: undefined } as ApiSuccess<void>;
  })();

  const result = await refreshPromise;
  isRefreshing = false;
  refreshPromise = null;
  return result;
}

const isCrossOrigin = !!API_ORIGIN && typeof window !== 'undefined' && !API_ORIGIN.startsWith(window.location.origin);

function buildUrl(path: string, params?: Record<string, string | number | boolean | undefined | null>) {
  const normalized = path.startsWith('/') ? `${API_BASE}${path}` : `${API_BASE}/${path}`;
  if (typeof window === 'undefined') return normalized;
  const url = new URL(normalized, window.location.origin);
  if (params) {
    Object.keys(params).forEach((k) => {
      const v = params[k];
      if (v !== undefined && v !== null) url.searchParams.append(k, String(v));
    });
  }
  return url.toString();
}

export async function request<T>(
  method: string,
  path: string,
  options?: { params?: Record<string, any>; body?: any; headers?: Record<string, string>; credentials?: RequestCredentials }
): Promise<ApiResult<T>> {
  const doRequest = async (): Promise<ApiResult<T>> => {
    try {
      const url = typeof window !== 'undefined' ? buildUrl(path, options?.params) : `${API_BASE}${path}`;
      const headers: Record<string, string> = {
        Accept: 'application/json',
        ...(options?.headers || {}),
      };

      let body: string | undefined;
      if (options?.body !== undefined) {
        headers['Content-Type'] = 'application/json';
        body = JSON.stringify(options.body);
      }

      const token = getAuthToken();
      if (token) headers['Authorization'] = `Bearer ${token}`;

      const resp = await fetch(url, {
        method,
        headers,
        body,
        credentials: options?.credentials ?? (isCrossOrigin ? 'include' : 'same-origin'),
      });

      const contentType = resp.headers.get('content-type') || '';
      if (!resp.ok) {
        let body: any = undefined;
        try {
          if (contentType.includes('application/json')) body = await resp.json();
          else body = await resp.text();
        } catch (e) {
          body = undefined;
        }
        const code = body?.error?.code || 'unknown_error';
        const message = body?.error?.message || resp.statusText || 'API error';
        return { success: false, status: resp.status, code, message };
      }

      if (resp.status === 204) return { success: true, data: undefined as unknown as T };

      if (contentType.includes('application/json')) {
        const json = await resp.json();
        return { success: true, data: json.data as T };
      }
      return { success: true, data: (await resp.text()) as unknown as T };
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Network error occurred';
      return { success: false, status: 0, code: 'network_error', message };
    }
  };

  const result = await doRequest();

  if (!result.success && result.status === 401 && path !== '/auth/refresh') {
    const refreshResult = await refreshToken();
    if (refreshResult.success) {
      // If refresh succeeded, retry the original request
      return await doRequest();
    }
    // Refresh failed, return the original error
    return result;
  }

  return result;
}

export const api = {
  get: <T>(path: string, params?: Record<string, any>) => request<T>('GET', path, { params }),
  post: <T>(path: string, body?: any) => request<T>('POST', path, { body }),
  put: <T>(path: string, body?: any) => request<T>('PUT', path, { body }),
  patch: <T>(path: string, body?: any) => request<T>('PATCH', path, { body }),
  del: <T>(path: string) => request<T>('DELETE', path),
};

export default api;
