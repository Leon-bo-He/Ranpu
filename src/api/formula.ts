import { invoke } from './invoke';
import type {
  BatchCopySummaryView,
  FormulaView,
  ImportFormulasSummaryView,
  Unit,
} from './types';

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

  batchCopyDefaultToWorkspace: (ids: number[]) =>
    invoke<BatchCopySummaryView>('cmd_batch_copy_default_to_active_workspace', {
      cmd: { default_formula_ids: ids },
    }),

  exportDefaultFormulas: (
    defaultFormulaIds: number[],
    passphrase: string,
    outPath: string,
  ) =>
    invoke<number>('cmd_export_default_formulas', {
      cmd: {
        default_formula_ids: defaultFormulaIds,
        passphrase,
        out_path: outPath,
      },
    }),

  importDefaultFormulas: (passphrase: string, inPath: string) =>
    invoke<ImportFormulasSummaryView>('cmd_import_default_formulas', {
      cmd: { passphrase, in_path: inPath },
    }),

  exportWorkspaceFormulas: (
    workspaceFormulaIds: number[],
    passphrase: string,
    outPath: string,
  ) =>
    invoke<number>('cmd_export_workspace_formulas', {
      cmd: {
        workspace_formula_ids: workspaceFormulaIds,
        passphrase,
        out_path: outPath,
      },
    }),

  importWorkspaceFormulas: (passphrase: string, inPath: string) =>
    invoke<ImportFormulasSummaryView>('cmd_import_workspace_formulas', {
      cmd: { passphrase, in_path: inPath },
    }),
};
