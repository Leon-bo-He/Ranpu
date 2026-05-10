import { invoke } from './invoke';
import type { CartLineView } from './types';

export const cartApi = {
  list: () => invoke<CartLineView[]>('cmd_list_cart'),

  add: (sourceKind: 'default' | 'workspace', sourceFormulaId: number, targetKg: number) =>
    invoke<void>('cmd_add_to_cart', {
      cmd: {
        source_kind: sourceKind,
        source_formula_id: sourceFormulaId,
        target_kg: targetKg,
      },
    }),

  updateKg: (
    sourceKind: 'default' | 'workspace',
    sourceFormulaId: number,
    targetKg: number,
  ) =>
    invoke<void>('cmd_update_cart_kg', {
      cmd: {
        source_kind: sourceKind,
        source_formula_id: sourceFormulaId,
        target_kg: targetKg,
      },
    }),

  remove: (sourceKind: 'default' | 'workspace', sourceFormulaId: number) =>
    invoke<boolean>('cmd_remove_from_cart', {
      cmd: { source_kind: sourceKind, source_formula_id: sourceFormulaId },
    }),

  clear: () => invoke<void>('cmd_clear_cart'),

  export: (format: 'csv' | 'html', outPath: string) =>
    invoke<void>('cmd_export_cart', { cmd: { format, out_path: outPath } }),

  /// 渲染当前批次清单为 HTML 字符串, 不落盘. 用于 iframe 预览 / 打印.
  /// customer 写到批次单头部 (空则后端 fallback 当前工作区名).
  /// perFormula 跟 list() 返回的购物车顺序对齐. 每条 cart line 一份元信息:
  /// 单个 vat (整组共用) + 多条纱支变体 (厂名 / 规格 / 个数).
  /// layout: 'standard' 或 'grid' (A4 四宫格, 默认).
  previewHtml: (args: {
    customer?: string | null;
    perFormula?: Array<{
      vatNumber?: string | null;
      yarns?: Array<{
        mill?: string | null;
        spec?: string | null;
        count?: string | null;
      }>;
    }>;
    layout?: 'standard' | 'grid';
  } = {}) =>
    invoke<string>('cmd_preview_cart_as_batch_sheet_html', {
      cmd: {
        customer: args.customer ?? null,
        per_formula: (args.perFormula ?? []).map((m) => ({
          vat_number: m.vatNumber ?? null,
          yarns: (m.yarns ?? []).map((y) => ({
            mill: y.mill ?? null,
            spec: y.spec ?? null,
            count: y.count ?? null,
          })),
        })),
        layout: args.layout ?? null,
      },
    }),
};
