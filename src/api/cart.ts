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
  /// customer / vatNumber / yarnCount 为前端预览对话框收集到的元信息, 写到
  /// 批次单头部. 都可空, 后端 customer 为空时 fallback 到当前工作区名.
  previewHtml: (args: {
    customer?: string | null;
    vatNumber?: string | null;
    yarnCount?: string | null;
  } = {}) =>
    invoke<string>('cmd_preview_cart_as_batch_sheet_html', {
      cmd: {
        customer: args.customer ?? null,
        vat_number: args.vatNumber ?? null,
        yarn_count: args.yarnCount ?? null,
      },
    }),
};
