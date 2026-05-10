import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type IdleTimeoutMinutes = 5 | 10 | 30 | 60 | 0; // 0 = 关闭

interface SettingsState {
  idleTimeoutMinutes: IdleTimeoutMinutes;
  setIdleTimeoutMinutes: (m: IdleTimeoutMinutes) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      idleTimeoutMinutes: 10,
      setIdleTimeoutMinutes: (m) => set({ idleTimeoutMinutes: m }),
    }),
    { name: 'ranpu-settings' },
  ),
);
