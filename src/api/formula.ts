import { invoke } from './invoke';
import type { FormulaView, Unit } from './types';

export interface FormulaItemDto {
  dye_name: string;
  dye_code: string | null;
  amount: number;
  unit: Unit;
  sort_order: number;
}

export interface UpsertFormulaPayload {
  id: number | null;
  internal_color_code: string;
  customer_color_code: string | null;
  color_name: string | null;
  description: string | null;
  base_weight_kg: number | null;
  liquor_ratio: number | null;
  notes: string | null;
  items: FormulaItemDto[];
}

export interface ListFormulasArgs {
  keyword?: string;
  limit?: number;
  offset?: number;
}

export const formulaApi = {
  listDefault: (args: ListFormulasArgs = {}) =>
    invoke<FormulaView[]>('cmd_list_default_formulas', {
      cmd: {
        keyword: args.keyword ?? null,
        limit: args.limit ?? null,
        offset: args.offset ?? null,
      },
    }),

  upsertDefault: (payload: UpsertFormulaPayload) =>
    invoke<number>('cmd_upsert_default_formula', { cmd: payload }),

  deleteDefault: (id: number) => invoke<void>('cmd_delete_default_formula', { id }),

  listWorkspace: (args: ListFormulasArgs = {}) =>
    invoke<FormulaView[]>('cmd_list_workspace_formulas', {
      cmd: {
        keyword: args.keyword ?? null,
        limit: args.limit ?? null,
        offset: args.offset ?? null,
      },
    }),

  upsertWorkspace: (payload: UpsertFormulaPayload) =>
    invoke<number>('cmd_upsert_workspace_formula', { cmd: payload }),

  deleteWorkspace: (id: number) => invoke<void>('cmd_delete_workspace_formula', { id }),

  copyDefaultToWorkspace: (defaultFormulaId: number) =>
    invoke<number>('cmd_copy_default_to_active_workspace', { defaultFormulaId }),
};
