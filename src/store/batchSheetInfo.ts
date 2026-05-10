import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/// 单条 "纱支变体": 一个 (缸号, 缸次, 厂名, 规格) 组合.
export interface PerFormulaEntry {
  vat: string;
  batch: string;
  yarnMill: string;
  yarnSpec: string;
}

/// 一个配方在批次单 prompt 里的所有变体. 同一个配方可能要打多份不同纱支
/// 的批次单 (例如博奥 30/2 一份, 名仁 32/2 一份), 每份独立 vat / yarn.
export interface PerFormulaMeta {
  entries: PerFormulaEntry[];
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

interface LegacyV1PerFormulaMeta {
  vat?: string;
  yarn?: string;
}

interface LegacyV2PerFormulaMeta {
  vat?: string;
  batch?: string;
  yarnMill?: string;
  yarnSpec?: string;
}

interface LegacyBatchSheetInfo {
  customer?: string;
  perFormula?: Record<string, unknown>;
}

interface PersistedShape {
  byWorkspace?: Record<string, LegacyBatchSheetInfo>;
}

export function emptyEntry(): PerFormulaEntry {
  return { vat: '', batch: '', yarnMill: '', yarnSpec: '' };
}

function toEntryFromV1(m: LegacyV1PerFormulaMeta): PerFormulaEntry {
  let vat = '';
  let batch = '';
  if (m.vat) {
    const dash = m.vat.lastIndexOf('-');
    if (dash > 0) {
      vat = m.vat.slice(0, dash);
      batch = m.vat.slice(dash + 1);
    } else {
      vat = m.vat;
    }
  }
  let yarnMill = '';
  let yarnSpec = '';
  if (m.yarn) {
    const space = m.yarn.indexOf(' ');
    if (space > 0) {
      yarnMill = m.yarn.slice(0, space).trim();
      yarnSpec = m.yarn.slice(space + 1).trim();
    } else {
      yarnSpec = m.yarn;
    }
  }
  return { vat, batch, yarnMill, yarnSpec };
}

/// v1 → v3 / v2 → v3: 单条 meta 包成 entries 数组.
function migrateToV3(state: PersistedShape | undefined): BatchSheetInfoState['byWorkspace'] {
  const out: BatchSheetInfoState['byWorkspace'] = {};
  if (!state?.byWorkspace) return out;
  for (const [wsId, info] of Object.entries(state.byWorkspace)) {
    const newPerFormula: Record<string, PerFormulaMeta> = {};
    const legacy = info.perFormula ?? {};
    for (const [key, raw] of Object.entries(legacy)) {
      // v3 (新结构) 直接保留.
      const m = raw as { entries?: PerFormulaEntry[] };
      if (Array.isArray(m.entries)) {
        newPerFormula[key] = { entries: m.entries.length > 0 ? m.entries : [emptyEntry()] };
        continue;
      }
      const v2 = raw as LegacyV2PerFormulaMeta;
      if (
        'batch' in v2 ||
        'yarnMill' in v2 ||
        'yarnSpec' in v2
      ) {
        newPerFormula[key] = {
          entries: [
            {
              vat: v2.vat ?? '',
              batch: v2.batch ?? '',
              yarnMill: v2.yarnMill ?? '',
              yarnSpec: v2.yarnSpec ?? '',
            },
          ],
        };
        continue;
      }
      // v1 fallback.
      newPerFormula[key] = { entries: [toEntryFromV1(raw as LegacyV1PerFormulaMeta)] };
    }
    out[Number(wsId)] = {
      customer: info.customer ?? '',
      perFormula: newPerFormula,
    };
  }
  return out;
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
      version: 3,
      migrate: (persisted, fromVersion) => {
        if (fromVersion < 3) {
          return {
            byWorkspace: migrateToV3(persisted as PersistedShape),
          } as BatchSheetInfoState;
        }
        return persisted as BatchSheetInfoState;
      },
    },
  ),
);

export const lineKey = (sourceKind: string, sourceFormulaId: number): string =>
  `${sourceKind}:${sourceFormulaId}`;
