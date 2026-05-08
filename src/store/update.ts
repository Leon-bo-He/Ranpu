import { check, type Update } from '@tauri-apps/plugin-updater';
import { create } from 'zustand';

/**
 * 应用更新状态. 进 app 主面板时由 UpdateNotifier 调一次 runCheck(),
 * 命中就把 pending 灌进来; About 页的按钮也订阅同一份, 显示 "有新版本"
 * + 红点, 不再额外打 IPC.
 *
 * - toastDismissed: 用户点过 "稍后" / X 之后, 本会话不再弹 toast,
 *   但不影响 About 按钮 (那是用户主动入口, 应该照常显示新版本提示).
 */
interface UpdateState {
  pending: Update | null;
  checking: boolean;
  hasChecked: boolean;
  error: string | null;
  toastDismissed: boolean;
  runCheck: () => Promise<Update | null>;
  dismissToast: () => void;
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  pending: null,
  checking: false,
  hasChecked: false,
  error: null,
  toastDismissed: false,
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
  dismissToast: () => set({ toastDismissed: true }),
}));
