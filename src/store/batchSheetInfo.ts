import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface PerFormulaMeta {
  vat: string;
  yarn: string;
}

export interface BatchSheetInfo {
  customer: string;
  /// key = `${source_kind}:${source_formula_id}`. 用 line 的稳定身份做键,
  /// 批次清单内容增删后旧值仍能按行匹配回填.
  perFormula: Record<string, PerFormulaMeta>;
}

interface BatchSheetInfoState {
  byWorkspace: Record<number, BatchSheetInfo>;
  setInfo: (workspaceId: number, info: BatchSheetInfo) => void;
}

export const useBatchSheetInfoStore = create<BatchSheetInfoState>()(
  persist(
    (set) => ({
      byWorkspace: {},
      setInfo: (workspaceId, info) =>
        set((s) => ({
          byWorkspace: { ...s.byWorkspace, [workspaceId]: info },
        })),
    }),
    { name: 'ranpu-batch-sheet-info' },
  ),
);

export const lineKey = (sourceKind: string, sourceFormulaId: number): string =>
  `${sourceKind}:${sourceFormulaId}`;
