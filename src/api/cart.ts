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

  /// 后端渲染 HTML 并通过自定义协议 ranpu-print:// 在新窗口加载.
  /// 新窗口里有内联工具栏调 window.print() / window.close(), 完全
  /// 不依赖前端 SPA / IPC.
  openPrintPreview: () => invoke<void>('cmd_open_print_preview'),
};
