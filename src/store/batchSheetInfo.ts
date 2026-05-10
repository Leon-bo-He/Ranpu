import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface PerFormulaMeta {
  vat: string;
  batch: string;
  yarnMill: string;
  yarnSpec: string;
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

interface LegacyPerFormulaMeta {
  vat?: string;
  yarn?: string;
}

interface LegacyBatchSheetInfo {
  customer?: string;
  perFormula?: Record<string, LegacyPerFormulaMeta>;
}

interface PersistedShape {
  byWorkspace?: Record<string, LegacyBatchSheetInfo | BatchSheetInfo>;
}

/// v1 → v2 迁移: vat ("5-2") → vat + batch; yarn ("博奥 30/2" / "30/2") →
/// yarnMill + yarnSpec. 拆不出来时尽量保留进 spec 字段, 让用户自己再调.
function migrateV1ToV2(state: PersistedShape | undefined): BatchSheetInfoState {
  const out: BatchSheetInfoState['byWorkspace'] = {};
  if (state?.byWorkspace) {
    for (const [wsId, info] of Object.entries(state.byWorkspace)) {
      const newPerFormula: Record<string, PerFormulaMeta> = {};
      const legacy = (info as LegacyBatchSheetInfo).perFormula ?? {};
      for (const [key, m] of Object.entries(legacy)) {
        const meta = m as LegacyPerFormulaMeta & Partial<PerFormulaMeta>;
        // 已经是新结构 (v2 重启) 就直接用.
        if (
          'batch' in meta ||
          'yarnMill' in meta ||
          'yarnSpec' in meta
        ) {
          newPerFormula[key] = {
            vat: meta.vat ?? '',
            batch: meta.batch ?? '',
            yarnMill: meta.yarnMill ?? '',
            yarnSpec: meta.yarnSpec ?? '',
          };
          continue;
        }
        let vat = '';
        let batch = '';
        if (meta.vat) {
          const dash = meta.vat.lastIndexOf('-');
          if (dash > 0) {
            vat = meta.vat.slice(0, dash);
            batch = meta.vat.slice(dash + 1);
          } else {
            vat = meta.vat;
          }
        }
        let yarnMill = '';
        let yarnSpec = '';
        if (meta.yarn) {
          const space = meta.yarn.indexOf(' ');
          if (space > 0) {
            yarnMill = meta.yarn.slice(0, space).trim();
            yarnSpec = meta.yarn.slice(space + 1).trim();
          } else {
            yarnSpec = meta.yarn;
          }
        }
        newPerFormula[key] = { vat, batch, yarnMill, yarnSpec };
      }
      out[Number(wsId)] = {
        customer: (info as LegacyBatchSheetInfo).customer ?? '',
        perFormula: newPerFormula,
      };
    }
  }
  return {
    byWorkspace: out,
    setInfo: (() => {
      // placeholder — real impl 在 store factory 里覆盖.
      throw new Error('not initialized');
    }) as BatchSheetInfoState['setInfo'],
  };
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
    {
      name: 'ranpu-batch-sheet-info',
      version: 2,
      migrate: (persisted, fromVersion) => {
        if (fromVersion < 2) {
          const migrated = migrateV1ToV2(persisted as PersistedShape);
          return { byWorkspace: migrated.byWorkspace } as BatchSheetInfoState;
        }
        return persisted as BatchSheetInfoState;
      },
    },
  ),
);

export const lineKey = (sourceKind: string, sourceFormulaId: number): string =>
  `${sourceKind}:${sourceFormulaId}`;
