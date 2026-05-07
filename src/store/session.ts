import { create } from 'zustand';

import type { SessionView } from '@/api/types';

interface SessionState {
  session: SessionView | null;
  setSession: (s: SessionView | null) => void;
  setLocked: (locked: boolean) => void;
  setActiveWorkspace: (id: number | null) => void;
  clear: () => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  session: null,
  setSession: (s) => set({ session: s }),
  setLocked: (locked) =>
    set((state) => ({
      session: state.session ? { ...state.session, locked } : null,
    })),
  setActiveWorkspace: (id) =>
    set((state) => ({
      session: state.session ? { ...state.session, active_workspace_id: id } : null,
    })),
  clear: () => set({ session: null }),
}));

export const isAdmin = (s: SessionView | null) => s?.role === 'admin';
export const hasActiveWorkspace = (s: SessionView | null) =>
  s !== null && s.active_workspace_id !== null;
