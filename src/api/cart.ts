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
  /// perFormula 跟 list() 返回的购物车顺序对齐, 每条配方独立的缸号 / 纱支.
  /// layout: 'standard' (默认每条一段) 或 'grid' (A4 四宫格).
  previewHtml: (args: {
    customer?: string | null;
    perFormula?: Array<{
      vatNumber?: string | null;
      yarnCount?: string | null;
    }>;
    layout?: 'standard' | 'grid';
  } = {}) =>
    invoke<string>('cmd_preview_cart_as_batch_sheet_html', {
      cmd: {
        customer: args.customer ?? null,
        per_formula: (args.perFormula ?? []).map((m) => ({
          vat_number: m.vatNumber ?? null,
          yarn_count: m.yarnCount ?? null,
        })),
        layout: args.layout ?? null,
      },
    }),
};
