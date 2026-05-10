import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type IdleTimeoutMinutes = 5 | 10 | 30 | 60 | 0; // 0 = 关闭

interface SettingsState {
  idleTimeoutMinutes: IdleTimeoutMinutes;
  setIdleTimeoutMinutes: (m: IdleTimeoutMinutes) => void;
  /// 一个纱支包/筒的标准重量 (kg). 批次单 prompt 里用 总重量 / 单个重量
  /// 自动算每条配方的纱支个数. 默认 1.25 kg.
  singleYarnWeightKg: number;
  setSingleYarnWeightKg: (n: number) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      idleTimeoutMinutes: 10,
      setIdleTimeoutMinutes: (m) => set({ idleTimeoutMinutes: m }),
      singleYarnWeightKg: 1.25,
      setSingleYarnWeightKg: (n) =>
        set({
          // 限正数, 上限给个保守的 999 防误输; 非数字 / 0 / 负数都拒绝.
          singleYarnWeightKg:
            Number.isFinite(n) && n > 0 ? Math.min(n, 999) : 1.25,
        }),
    }),
    { name: 'ranpu-settings' },
  ),
);
