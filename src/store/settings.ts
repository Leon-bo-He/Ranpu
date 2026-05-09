import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type IdleTimeoutMinutes = 5 | 10 | 30 | 60 | 0; // 0 = 关闭

interface SettingsState {
  idleTimeoutMinutes: IdleTimeoutMinutes;
  setIdleTimeoutMinutes: (m: IdleTimeoutMinutes) => void;
  /// 染厂染缸总数. 后续 "单日染缸批次重置" 用这个数加批次序号自动生成
  /// 缸号: 例 8 缸厂第一批 1-1, 1-2, ..., 1-8, 第二批 2-1, ..., 2-8.
  /// 当前先存值, 实际填缸号逻辑 follow-up PR 实现.
  vatCount: number;
  setVatCount: (n: number) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      idleTimeoutMinutes: 10,
      setIdleTimeoutMinutes: (m) => set({ idleTimeoutMinutes: m }),
      vatCount: 8,
      setVatCount: (n) => set({ vatCount: Math.max(1, Math.min(99, Math.floor(n))) }),
    }),
    { name: 'ranpu-settings' },
  ),
);
