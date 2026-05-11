import { create } from 'zustand';
import { persist } from 'zustand/middleware';

import { sortByPinyin } from '@/lib/pinyinSearch';

/// 染料名称库. 用户在编辑配方时, 染料名 input 从这个库里挑候选;
/// 保存时如果有不在库里的新名字, 弹 dialog 询问是否加入复用.
/// 默认空 — 不内置任何染料名, 用户首次保存配方时按需加入.
const DEFAULT_DYES: string[] = [];

interface DyeLibraryState {
  dyes: string[];
  setDyes: (list: string[]) => void;
  resetDyes: () => void;
}

export const useDyeLibraryStore = create<DyeLibraryState>()(
  persist(
    (set) => ({
      dyes: [...DEFAULT_DYES],
      setDyes: (list) => set({ dyes: sortByPinyin(list) }),
      resetDyes: () => set({ dyes: [...DEFAULT_DYES] }),
    }),
    { name: 'ranpu-dye-library' },
  ),
);
