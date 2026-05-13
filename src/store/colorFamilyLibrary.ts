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
  /// 是否已经把 DB 历史色系一次性导入过. 升级到有色系库的版本时, 老用户
  /// 库为空, 直接看不到自己之前用过的色系, 体验差. App.tsx 在 session 建好
  /// 后检查此 flag, 为 false 就调一次 listAllColorFamilies 合并 + 置 true.
  imported: boolean;
  setColorFamilies: (list: string[]) => void;
  resetColorFamilies: () => void;
  markImported: () => void;
}

export const useColorFamilyLibraryStore = create<ColorFamilyLibraryState>()(
  persist(
    (set) => ({
      colorFamilies: [...DEFAULT_COLOR_FAMILIES],
      imported: false,
      setColorFamilies: (list) => set({ colorFamilies: sortByPinyin(list) }),
      resetColorFamilies: () => set({ colorFamilies: [...DEFAULT_COLOR_FAMILIES] }),
      markImported: () => set({ imported: true }),
    }),
    { name: 'ranpu-color-family-library' },
  ),
);
