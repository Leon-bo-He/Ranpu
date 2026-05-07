/**
 * 与后端 interfaces/tauri/dto.rs 对齐的 TypeScript 类型。
 * 命名保持与 Rust 一致以便对照。
 */

export interface SessionView {
  user_id: number;
  username: string;
  role: 'admin' | 'user';
  active_workspace_id: number | null;
  locked: boolean;
  last_activity_at: string;
}

export interface UserView {
  id: number;
  username: string;
  role: 'admin' | 'user';
  is_active: boolean;
  failed_attempts: number;
  locked_until: string | null;
  created_at: string;
  last_login: string | null;
}

export interface WorkspaceView {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
}

export interface FormulaItemView {
  id: number;
  dye_name: string;
  dye_code: string | null;
  amount: number;
  unit: 'pct_owf' | 'g_per_kg' | 'g_per_L';
  sort_order: number;
}

export interface FormulaView {
  id: number;
  internal_color_code: string;
  customer_color_code: string | null;
  color_name: string | null;
  description: string | null;
  base_weight_kg: number | null;
  liquor_ratio: number | null;
  notes: string | null;
  items: FormulaItemView[];
  source_default_id: number | null;
  created_at: string;
  updated_at: string;
}

export interface CalculationLineView {
  dye_name: string;
  dye_code: string | null;
  grams: number;
  unit_used: 'pct_owf' | 'g_per_kg' | 'g_per_L';
}

export interface CalculationResultView {
  source: 'current_workspace' | 'default_fallback';
  source_label: string;
  formula_id: number | null;
  internal_color_code: string;
  target_kg: number;
  lines: CalculationLineView[];
}

export interface CustomerCodeMatchView {
  source: 'current_workspace' | 'default_fallback';
  source_label: string;
  formula_id: number | null;
  internal_color_code: string;
  color_name: string | null;
  customer_color_code: string | null;
}

export interface BatchCopyOutcomeItemView {
  source_default_id: number;
  new_workspace_formula_id: number | null;
  error: string | null;
}

export interface BatchCopySummaryView {
  items: BatchCopyOutcomeItemView[];
  succeeded: number;
  failed: number;
}

export interface ImportItemOutcomeView {
  internal_color_code: string;
  status: 'imported' | 'skipped_duplicate' | 'failed';
  error: string | null;
}

export interface ImportSectionSummaryView {
  items: ImportItemOutcomeView[];
  imported: number;
  skipped: number;
  failed: number;
}

export type WorkspaceImportAction = 'skip' | 'merge' | 'create_new';

export interface WorkspaceImportPlanDto {
  name: string;
  action: WorkspaceImportAction;
}

export interface ExportLibraryArchiveView {
  default_count: number;
  workspace_count: number;
  workspace_formula_count: number;
}

export interface PreviewWorkspaceView {
  name: string;
  description: string | null;
  formula_count: number;
  already_exists: boolean;
}

export interface PreviewLibraryArchiveView {
  exported_at: string;
  default_count: number;
  has_default: boolean;
  workspaces: PreviewWorkspaceView[];
}

export interface ImportWorkspaceSummaryView {
  name: string;
  /** "skipped" | "merged" | "created" */
  action: string;
  summary: ImportSectionSummaryView;
}

export interface ImportLibraryArchiveView {
  default_summary: ImportSectionSummaryView | null;
  workspace_summaries: ImportWorkspaceSummaryView[];
}

export interface CartLineView {
  source_kind: 'default' | 'workspace';
  source_formula_id: number;
  target_kg: number;
  added_at: string;
  internal_color_code: string | null;
  color_name: string | null;
  customer_color_code: string | null;
  calculation: CalculationResultView | null;
  error: string | null;
}

export interface AuditEventView {
  id: number;
  event_uuid: string;
  user_id: number | null;
  workspace_context_id: number | null;
  action: string;
  target: string | null;
  details: string | null;
  occurred_at: string;
}

export interface UnlockOutcomeView {
  kind: 'unlocked' | 'still_locked' | 'force_logged_out';
  remaining: number | null;
}

export interface BootStatusView {
  keystore_exists: boolean;
  db_initialized: boolean;
  user_count: number;
}

export type Unit = 'pct_owf' | 'g_per_kg' | 'g_per_L';
