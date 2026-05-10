import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/// 纱支选项的两个维度: 厂名 + 规格. 都是用户可编辑的字符串列表, 持久化在
/// 本地. 后续批次单 prompt 的 "纱支" 输入会从这俩拼出候选 (例 "博奥 30/2"),
/// 但目前只负责存和编辑.
const DEFAULT_MILLS = ['博奥', '名仁', '妙虎', '弘曲'] as const;
const DEFAULT_SPECS = [
  '20/2',
  '20/3',
  '30/2',
  '30/3',
  '40/2',
  '40/3',
  '50/2',
  '50/3',
  '60/2',
  '60/3',
] as const;

interface YarnSettingsState {
  mills: string[];
  specs: string[];
  setMills: (list: string[]) => void;
  setSpecs: (list: string[]) => void;
  resetMills: () => void;
  resetSpecs: () => void;
}

export const useYarnSettingsStore = create<YarnSettingsState>()(
  persist(
    (set) => ({
      mills: [...DEFAULT_MILLS],
      specs: [...DEFAULT_SPECS],
      setMills: (list) => set({ mills: list }),
      setSpecs: (list) => set({ specs: list }),
      resetMills: () => set({ mills: [...DEFAULT_MILLS] }),
      resetSpecs: () => set({ specs: [...DEFAULT_SPECS] }),
    }),
    { name: 'ranpu-yarn-settings' },
  ),
);
