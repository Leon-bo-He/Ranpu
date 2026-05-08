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

  /// 后端渲染 HTML, 弹出独立的 print-preview 窗口. 那个窗口 mount 时
  /// 自己调 consumePrintPreview 取走 HTML — 调用方不需要持有.
  openPrintPreview: () => invoke<void>('cmd_open_print_preview'),

  /// 仅 print-preview 窗口用: 取走主窗口刚 stash 的 HTML.
  consumePrintPreview: () => invoke<string | null>('cmd_consume_print_preview'),
};
