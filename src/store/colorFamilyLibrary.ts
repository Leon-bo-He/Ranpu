import { create } from 'zustand';
import { persist } from 'zustand/middleware';

import { sortByPinyin } from '@/lib/pinyinSearch';

/// 色系库. 跟 dyeLibrary 同款: 用户编辑配方时色系下拉从这里挑候选;
/// 保存时如果填了不在库里的新色系, 弹 dialog 询问是否加入复用.
/// 跨工作区共享 (zustand persist 到 localStorage). 默认空 — 不内置,
/// 用户首次保存配方时按需加入.
const DEFAULT_COLOR_FAMILIES: string[] = [];

interface ColorFamilyLibraryState {
  colorFamilies: string[];
  setColorFamilies: (list: string[]) => void;
  resetColorFamilies: () => void;
}

export const useColorFamilyLibraryStore = create<ColorFamilyLibraryState>()(
  persist(
    (set) => ({
      colorFamilies: [...DEFAULT_COLOR_FAMILIES],
      setColorFamilies: (list) => set({ colorFamilies: sortByPinyin(list) }),
      resetColorFamilies: () => set({ colorFamilies: [...DEFAULT_COLOR_FAMILIES] }),
    }),
    { name: 'ranpu-color-family-library' },
  ),
);
