import { create } from 'zustand';

type AuthState = {
  token: string | null;
  refreshToken: string | null;
  setTokens: (t: string | null, r: string | null) => void;
  clearTokens: () => void;
};

export const useAuthStore = create<AuthState>((set) => ({
  token: typeof window !== 'undefined' ? localStorage.getItem('authToken') : null,
  refreshToken: typeof window !== 'undefined' ? localStorage.getItem('refreshToken') : null,
  setTokens: (t: string | null, r: string | null) => {
    set({ token: t, refreshToken: r });
    if (typeof window !== 'undefined') {
      if (t) localStorage.setItem('authToken', t);
      else localStorage.removeItem('authToken');
      if (r) localStorage.setItem('refreshToken', r);
      else localStorage.removeItem('refreshToken');
    }
  },
  clearTokens: () => {
    set({ token: null, refreshToken: null });
    if (typeof window !== 'undefined') {
      localStorage.removeItem('authToken');
      localStorage.removeItem('refreshToken');
    }
  },
}));

export function getAuthTokenSync() {
  return useAuthStore.getState().token;
}

export function getRefreshTokenSync() {
  return useAuthStore.getState().refreshToken;
}

export function setTokensSync(t: string | null, r: string | null) {
  useAuthStore.getState().setTokens(t, r);
}

export function clearTokensSync() {
  useAuthStore.getState().clearTokens();
}
