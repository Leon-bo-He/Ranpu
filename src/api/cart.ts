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
  /// layout: 'standard' 经典段落 | 'grid' A4 四宫格 | 'a6punch' 穿孔纸
  /// (一条配方一张, 默认) | 'label' 50×80mm 标签纸 (只 vat + 客户 + 纱支).
  /// 后端 None / 未识别值 也走 a6punch.
  /// colorCheck / dryCheck: 跟踪卡 (label) 上 对色 / 烘干 框是否预先打 ✓.
  /// 其他 layout 忽略这两个字段.
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
    layout?: 'standard' | 'grid' | 'a6punch' | 'label';
    colorCheck?: boolean;
    dryCheck?: boolean;
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
        color_check: args.colorCheck ?? null,
        dry_check: args.dryCheck ?? null,
      },
    }),
};
