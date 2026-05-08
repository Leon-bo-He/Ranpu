import { check, type Update } from '@tauri-apps/plugin-updater';
import { create } from 'zustand';

/**
 * 应用更新状态. App 进主面板时静默调一次 runCheck(); 命中就把 pending 灌进来,
 * 侧栏 "关于" 项右侧红点 + About 页按钮 "有新版本 X.Y.Z" 自动展示. About 页
 * 用户也可以主动按 "检查更新" 重跑.
 */
interface UpdateState {
  pending: Update | null;
  checking: boolean;
  hasChecked: boolean;
  error: string | null;
  runCheck: () => Promise<Update | null>;
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  pending: null,
  checking: false,
  hasChecked: false,
  error: null,
  runCheck: async () => {
    if (get().checking) return get().pending;
    set({ checking: true, error: null });
    try {
      const u = await check();
      set({ pending: u, hasChecked: true, checking: false });
      return u;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ error: msg, hasChecked: true, checking: false });
      return null;
    }
  },
}));
