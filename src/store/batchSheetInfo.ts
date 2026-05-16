import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/// 一组纱支变体: 厂名 + 规格 + 个数. 缸号 / 缸次 不在这里 — 同一个配方
/// 下所有变体共用 PerFormulaMeta 上的缸号 / 缸次. count 默认由 总重量 /
/// 单个重量 算出, 用户可手改; 加一组纱支时新行的默认 = 总重 / 单个重 -
/// 已设置总和.
export interface PerFormulaEntry {
  yarnMill: string;
  yarnSpec: string;
  count: string;
}

/// 一个配方在批次单 prompt 里的元信息. 缸号 + 缸次 跟着配方走 (一个配方
/// 一锅染液一对缸号 / 缸次), entries 是这一锅里要发的多份不同纱支.
/// colorCheck / dryCheck: 该条配方的跟踪卡上 对色 / 烘干 框是否预先 ✓.
/// 老数据 / 新建配方默认 对色 ✓ / 烘干 ☐.
export interface PerFormulaMeta {
  vat: string;
  batch: string;
  entries: PerFormulaEntry[];
  colorCheck: boolean;
  dryCheck: boolean;
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

interface LegacyV4Entry {
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
  return { yarnMill: '', yarnSpec: '', count: '' };
}

export function emptyMeta(): PerFormulaMeta {
  return {
    vat: '',
    batch: '',
    entries: [emptyEntry()],
    colorCheck: true,
    dryCheck: false,
  };
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
  return {
    vat,
    batch,
    entries: [{ yarnMill, yarnSpec, count: '' }],
    colorCheck: true,
    dryCheck: false,
  };
}

/// 把任何旧版本规格化到 v5: 配方层 vat + batch + entries[厂名/规格/个数].
/// 多 entries 旧数据取第一条的缸号 / 缸次作为整组共用值; 个数全部置空,
/// 等用户重新打开 prompt 时自动算默认值.
function migrateToV5(state: PersistedShape | undefined): BatchSheetInfoState['byWorkspace'] {
  const out: BatchSheetInfoState['byWorkspace'] = {};
  if (!state?.byWorkspace) return out;
  for (const [wsId, info] of Object.entries(state.byWorkspace)) {
    const newPerFormula: Record<string, PerFormulaMeta> = {};
    const legacy = info.perFormula ?? {};
    for (const [key, raw] of Object.entries(legacy)) {
      // v5 (新结构): 已有 entries 含 count.
      const v5 = raw as Partial<PerFormulaMeta>;
      if (
        Array.isArray(v5.entries) &&
        ('vat' in v5 || 'batch' in v5) &&
        v5.entries.every((e) => typeof e?.count === 'string')
      ) {
        newPerFormula[key] = {
          vat: v5.vat ?? '',
          batch: v5.batch ?? '',
          entries:
            v5.entries.length > 0
              ? v5.entries.map((e) => ({
                  yarnMill: e.yarnMill ?? '',
                  yarnSpec: e.yarnSpec ?? '',
                  count: e.count ?? '',
                }))
              : [emptyEntry()],
          colorCheck: true,
          dryCheck: false,
        };
        continue;
      }
      // v4: { vat, batch, entries: [{yarnMill, yarnSpec}, ...] }
      const v4 = raw as { vat?: string; batch?: string; entries?: LegacyV4Entry[] };
      if (
        Array.isArray(v4.entries) &&
        v4.entries.length > 0 &&
        v4.entries.every((e) => 'yarnMill' in e || 'yarnSpec' in e)
      ) {
        newPerFormula[key] = {
          vat: v4.vat ?? '',
          batch: v4.batch ?? '',
          entries: v4.entries.map((e) => ({
            yarnMill: e.yarnMill ?? '',
            yarnSpec: e.yarnSpec ?? '',
            count: '',
          })),
          colorCheck: true,
          dryCheck: false,
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
                  count: '',
                }))
              : [emptyEntry()],
          colorCheck: true,
          dryCheck: false,
        };
        continue;
      }
      // v2: { vat, batch, yarnMill, yarnSpec }
      const v2 = raw as LegacyV2PerFormulaMeta;
      if ('batch' in v2 || 'yarnMill' in v2 || 'yarnSpec' in v2) {
        newPerFormula[key] = {
          vat: v2.vat ?? '',
          batch: v2.batch ?? '',
          entries: [
            {
              yarnMill: v2.yarnMill ?? '',
              yarnSpec: v2.yarnSpec ?? '',
              count: '',
            },
          ],
          colorCheck: true,
          dryCheck: false,
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

/// v6: 每条 PerFormulaMeta 增加 colorCheck / dryCheck. v6 早期开发版本曾经
/// 在 BatchSheetInfo 顶层加过同名字段 (整组通用), 这里把它分发到每条配方
/// 当作默认值 — 然后顶层字段直接忽略. v5 / 早期 v6 都用此路径补全.
function migrateToV6(state: BatchSheetInfoState): BatchSheetInfoState {
  const out: BatchSheetInfoState['byWorkspace'] = {};
  for (const [wsId, info] of Object.entries(state.byWorkspace ?? {})) {
    const globalColor = (info as unknown as { colorCheck?: boolean }).colorCheck;
    const globalDry = (info as unknown as { dryCheck?: boolean }).dryCheck;
    const perFormulaOut: Record<string, PerFormulaMeta> = {};
    for (const [key, m] of Object.entries(info.perFormula ?? {})) {
      perFormulaOut[key] = {
        ...m,
        colorCheck: m.colorCheck ?? globalColor ?? true,
        dryCheck: m.dryCheck ?? globalDry ?? false,
      };
    }
    out[Number(wsId)] = {
      customer: info.customer ?? '',
      perFormula: perFormulaOut,
    };
  }
  return { ...state, byWorkspace: out };
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
      version: 6,
      migrate: (persisted, fromVersion) => {
        let state: BatchSheetInfoState;
        if (fromVersion < 5) {
          state = {
            byWorkspace: migrateToV5(persisted as PersistedShape),
          } as BatchSheetInfoState;
        } else {
          state = persisted as BatchSheetInfoState;
        }
        if (fromVersion < 6) {
          state = migrateToV6(state);
        }
        return state;
      },
    },
  ),
);

export const lineKey = (sourceKind: string, sourceFormulaId: number): string =>
  `${sourceKind}:${sourceFormulaId}`;
