import { create } from 'zustand';

import { workspaceApi } from '@/api/workspace';
import type { WorkspaceView } from '@/api/types';

/**
 * 工作区列表的全局缓存. 任何会改动工作区集合的页面 (创建 / 重命名 / 删除 /
 * 通过归档导入) 在操作完成后调一次 refresh(), 顶栏 WorkspaceSwitcher 等订阅者
 * 自动同步.
 */
interface WorkspacesState {
  list: WorkspaceView[];
  loaded: boolean;
  refresh: () => Promise<void>;
  clear: () => void;
}

export const useWorkspacesStore = create<WorkspacesState>((set) => ({
  list: [],
  loaded: false,
  refresh: async () => {
    try {
      const all = await workspaceApi.list();
      set({ list: all, loaded: true });
    } catch {
      set({ list: [], loaded: true });
    }
  },
  clear: () => set({ list: [], loaded: false }),
}));
