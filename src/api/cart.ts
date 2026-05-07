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

  export: (format: 'csv' | 'pdf', outPath: string) =>
    invoke<void>('cmd_export_cart', { cmd: { format, out_path: outPath } }),
};
