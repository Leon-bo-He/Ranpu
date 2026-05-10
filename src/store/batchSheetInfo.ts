import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/// 一组纱支变体: 厂名 + 规格. 缸号 / 缸次 不在这里 — 同一个配方下所有
/// 变体共用 PerFormulaMeta 上的缸号 / 缸次.
export interface PerFormulaEntry {
  yarnMill: string;
  yarnSpec: string;
}

/// 一个配方在批次单 prompt 里的元信息. 缸号 + 缸次 跟着配方走 (一个配方
/// 一锅染液一对缸号 / 缸次), entries 是这一锅里要发的多份不同纱支.
export interface PerFormulaMeta {
  vat: string;
  batch: string;
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

interface LegacyV3Entry {
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
  return { yarnMill: '', yarnSpec: '' };
}

export function emptyMeta(): PerFormulaMeta {
  return { vat: '', batch: '', entries: [emptyEntry()] };
}

function fromV1(m: LegacyV1PerFormulaMeta): PerFormulaMeta {
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
  return { vat, batch, entries: [{ yarnMill, yarnSpec }] };
}

/// 把任何旧版本规格化到 v4: 缸号 / 缸次 提到 PerFormulaMeta, entries 只
/// 留厂名 / 规格. 多 entries 时取第一条的缸号 / 缸次作为整组共用值.
function migrateToV4(state: PersistedShape | undefined): BatchSheetInfoState['byWorkspace'] {
  const out: BatchSheetInfoState['byWorkspace'] = {};
  if (!state?.byWorkspace) return out;
  for (const [wsId, info] of Object.entries(state.byWorkspace)) {
    const newPerFormula: Record<string, PerFormulaMeta> = {};
    const legacy = info.perFormula ?? {};
    for (const [key, raw] of Object.entries(legacy)) {
      // v4 (新结构): 已经有 entries + 顶层 vat / batch.
      const v4 = raw as Partial<PerFormulaMeta>;
      if (Array.isArray(v4.entries) && ('vat' in v4 || 'batch' in v4)) {
        newPerFormula[key] = {
          vat: v4.vat ?? '',
          batch: v4.batch ?? '',
          entries:
            v4.entries.length > 0
              ? v4.entries.map((e) => ({
                  yarnMill: e.yarnMill ?? '',
                  yarnSpec: e.yarnSpec ?? '',
                }))
              : [emptyEntry()],
        };
        continue;
      }
      // v3: { entries: [{vat, batch, yarnMill, yarnSpec}, ...] }
      const v3 = raw as { entries?: LegacyV3Entry[] };
      if (Array.isArray(v3.entries)) {
        const first = v3.entries[0] ?? {};
        newPerFormula[key] = {
          vat: first.vat ?? '',
          batch: first.batch ?? '',
          entries:
            v3.entries.length > 0
              ? v3.entries.map((e) => ({
                  yarnMill: e.yarnMill ?? '',
                  yarnSpec: e.yarnSpec ?? '',
                }))
              : [emptyEntry()],
        };
        continue;
      }
      // v2: { vat, batch, yarnMill, yarnSpec }
      const v2 = raw as LegacyV2PerFormulaMeta;
      if ('batch' in v2 || 'yarnMill' in v2 || 'yarnSpec' in v2) {
        newPerFormula[key] = {
          vat: v2.vat ?? '',
          batch: v2.batch ?? '',
          entries: [{ yarnMill: v2.yarnMill ?? '', yarnSpec: v2.yarnSpec ?? '' }],
        };
        continue;
      }
      // v1 fallback.
      newPerFormula[key] = fromV1(raw as LegacyV1PerFormulaMeta);
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
      version: 4,
      migrate: (persisted, fromVersion) => {
        if (fromVersion < 4) {
          return {
            byWorkspace: migrateToV4(persisted as PersistedShape),
          } as BatchSheetInfoState;
        }
        return persisted as BatchSheetInfoState;
      },
    },
  ),
);

export const lineKey = (sourceKind: string, sourceFormulaId: number): string =>
  `${sourceKind}:${sourceFormulaId}`;
