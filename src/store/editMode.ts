import { create } from 'zustand';

/// 三个 "管理模式" 开关 — 默认全部 OFF, 把改动 / 敏感数据藏在 toggle 后面.
///   formulaEdit:    开启后才能新建 / 编辑 / 删除配方 (默认库 + 工作区配方)
///   workspaceEdit:  开启后才能新建 / 改名 / 删除工作区
///   auditDisplay:   开启后才显示审计日志列表
///
/// 自动关: 开启后 30 分钟内无相应操作 (touch*Activity), 由 EditModeAutoOffGuard
/// 每分钟扫一次, 自动 disable* . 重启应用一律回到 OFF (内存 store 不持久化).
interface EditModeState {
  formulaEditEnabled: boolean;
  workspaceEditEnabled: boolean;
  auditDisplayEnabled: boolean;
  formulaLastActivity: number;
  workspaceLastActivity: number;
  auditLastActivity: number;

  enableFormulaEdit: () => void;
  disableFormulaEdit: () => void;
  touchFormulaActivity: () => void;

  enableWorkspaceEdit: () => void;
  disableWorkspaceEdit: () => void;
  touchWorkspaceActivity: () => void;

  enableAuditDisplay: () => void;
  disableAuditDisplay: () => void;
  touchAuditActivity: () => void;

  /// EditModeAutoOffGuard 用. 调用一次清掉所有超过 idle_ms 没活动的开关.
  sweepIdle: (idle_ms: number) => void;
}

export const IDLE_AUTO_OFF_MS = 30 * 60 * 1000;

export const useEditModeStore = create<EditModeState>()((set) => ({
  formulaEditEnabled: false,
  workspaceEditEnabled: false,
  auditDisplayEnabled: false,
  formulaLastActivity: 0,
  workspaceLastActivity: 0,
  auditLastActivity: 0,

  enableFormulaEdit: () =>
    set({ formulaEditEnabled: true, formulaLastActivity: Date.now() }),
  disableFormulaEdit: () => set({ formulaEditEnabled: false }),
  touchFormulaActivity: () => set({ formulaLastActivity: Date.now() }),

  enableWorkspaceEdit: () =>
    set({ workspaceEditEnabled: true, workspaceLastActivity: Date.now() }),
  disableWorkspaceEdit: () => set({ workspaceEditEnabled: false }),
  touchWorkspaceActivity: () => set({ workspaceLastActivity: Date.now() }),

  enableAuditDisplay: () =>
    set({ auditDisplayEnabled: true, auditLastActivity: Date.now() }),
  disableAuditDisplay: () => set({ auditDisplayEnabled: false }),
  touchAuditActivity: () => set({ auditLastActivity: Date.now() }),

  sweepIdle: (idle_ms) =>
    set((s) => {
      const now = Date.now();
      const patch: Partial<EditModeState> = {};
      if (s.formulaEditEnabled && now - s.formulaLastActivity > idle_ms) {
        patch.formulaEditEnabled = false;
      }
      if (s.workspaceEditEnabled && now - s.workspaceLastActivity > idle_ms) {
        patch.workspaceEditEnabled = false;
      }
      if (s.auditDisplayEnabled && now - s.auditLastActivity > idle_ms) {
        patch.auditDisplayEnabled = false;
      }
      return patch;
    }),
}));
