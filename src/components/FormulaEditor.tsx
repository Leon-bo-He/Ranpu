import { Check, ChevronsUpDown, Plus, Trash2 } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';

import type { UpsertFormulaPayload } from '@/api/formula';
import type { FormulaView, Unit } from '@/api/types';
import { ComboboxInput } from '@/components/ComboboxInput';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { UnknownDyesPromptDialog } from '@/components/UnknownDyesPromptDialog';
import { cn } from '@/lib/utils';
import { useDyeLibraryStore } from '@/store/dyeLibrary';

/// UI 内部用的 item 形态: amount 保持字符串, 让 "0." / "0.12" 这种
/// 输入中态能完整保留. 提交时再 parseFloat 转回 number 进 payload.
/// 用 number 状态会让 "0." → Number("0.") = 0 → renders "0", 把小数
/// 点吞掉, 用户敲 "0.12" 永远只能拿到 "012".
interface ItemForm {
  dye_name: string;
  dye_code: string | null;
  amount: string;
  unit: Unit;
  sort_order: number;
}

interface FormulaEditorProps {
  open: boolean;
  onClose: () => void;
  initial: FormulaView | null;
  /** 标题文案，前缀，比如 "默认配方" 或 "工作区配方"。 */
  scope: string;
  /** 已存在的色系列表 (从仓储 distinct 读), 喂进 dropdown. */
  colorFamilies: string[];
  /** 提交时调用，返回 Promise 让对话框知道何时关闭 / 显示错误。 */
  onSave: (payload: UpsertFormulaPayload) => Promise<void>;
}

export function FormulaEditor({
  open,
  onClose,
  initial,
  scope,
  colorFamilies,
  onSave,
}: FormulaEditorProps) {
  const [internal, setInternal] = useState('');
  const [customer, setCustomer] = useState('');
  const [colorFamily, setColorFamily] = useState('');
  const [items, setItems] = useState<ItemForm[]>([blankItem(0)]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // 染料库: 用户编辑配方时下拉候选, 保存时不在库的新名字弹 dialog 询问.
  const dyeLibrary = useDyeLibraryStore((s) => s.dyes);
  const setDyeLibrary = useDyeLibraryStore((s) => s.setDyes);
  // 保存时收集到的新染料名 (排重 + 排首尾空白). 非空时弹
  // UnknownDyesPromptDialog 让用户选要不要加进库.
  const [unknownDyes, setUnknownDyes] = useState<string[]>([]);

  useEffect(() => {
    if (!open) return;
    if (initial) {
      setInternal(initial.internal_color_code);
      setCustomer(initial.customer_color_code ?? '');
      setColorFamily(initial.color_family ?? '');
      setItems(
        initial.items.map((i, idx) => ({
          dye_name: i.dye_name,
          dye_code: i.dye_code,
          amount: String(i.amount),
          unit: i.unit,
          sort_order: idx,
        })),
      );
    } else {
      setInternal('');
      setCustomer('');
      setColorFamily('');
      setItems([blankItem(0)]);
    }
    setError(null);
  }, [open, initial]);

  /// 点保存的入口: 先扫一遍 items 找不在染料库的新名字. 有 → 弹
  /// UnknownDyesPromptDialog; 无 → 直接 doSubmit.
  const submit = () => {
    const librarySet = new Set(dyeLibrary.map((s) => s.trim().toLowerCase()));
    const seen = new Set<string>();
    const unknowns: string[] = [];
    for (const it of items) {
      const name = it.dye_name.trim();
      if (!name) continue;
      const key = name.toLowerCase();
      if (librarySet.has(key) || seen.has(key)) continue;
      seen.add(key);
      unknowns.push(name);
    }
    if (unknowns.length > 0) {
      setUnknownDyes(unknowns);
      return;
    }
    void doSubmit();
  };

  const doSubmit = async () => {
    setBusy(true);
    setError(null);
    try {
      const payload: UpsertFormulaPayload = {
        id: initial?.id ?? null,
        internal_color_code: internal.trim(),
        customer_color_code: customer.trim() ? customer.trim() : null,
        color_family: colorFamily.trim() ? colorFamily.trim() : null,
        // 备注栏前端已删, 始终写 null. 老配方原本的备注会被这次保存清空.
        notes: null,
        items: items.map((it, idx) => ({
          dye_name: it.dye_name,
          dye_code: it.dye_code,
          amount: parseFloat(it.amount),
          unit: it.unit,
          sort_order: idx,
        })),
      };
      await onSave(payload);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  /// UnknownDyesPromptDialog 确认: 选中的写入染料库, 然后继续保存. 取消
  /// 视为返回编辑器 (不保存). 跟 Cart 里的 UnknownYarn 流程语义一致.
  const onUnknownDyesResolved = (toAdd: string[]) => {
    if (toAdd.length > 0) {
      setDyeLibrary([...dyeLibrary, ...toAdd]);
    }
    setUnknownDyes([]);
    void doSubmit();
  };

  const onUnknownDyesCancel = () => {
    setUnknownDyes([]);
  };

  const addItem = () =>
    setItems((prev) => [...prev, blankItem(prev.length)]);

  const updateItem = (idx: number, patch: Partial<ItemForm>) =>
    setItems((prev) => prev.map((it, i) => (i === idx ? { ...it, ...patch } : it)));

  const removeItem = (idx: number) =>
    setItems((prev) => (prev.length === 1 ? prev : prev.filter((_, i) => i !== idx)));

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="flex max-h-[90vh] max-w-3xl flex-col gap-0 p-0">
        <DialogHeader className="shrink-0 border-b px-6 py-4">
          <DialogTitle>
            {initial ? '编辑' : '新建'}
            {scope}配方
          </DialogTitle>
          <DialogDescription>
            内部色号必填且唯一。色系可从已用过的下拉里选，也可直接输入新的。
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 space-y-4 overflow-y-auto px-6 py-4">
          <div className="grid grid-cols-2 gap-3">
            <Field label="内部色号" required>
              <Input value={internal} onChange={(e) => setInternal(e.target.value)} />
            </Field>
            <Field label="客户色号">
              <Input value={customer} onChange={(e) => setCustomer(e.target.value)} />
            </Field>
            <Field label="色系">
              <ColorFamilyCombo
                value={colorFamily}
                onChange={setColorFamily}
                options={colorFamilies}
              />
            </Field>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>染料明细（至少 1 条）</Label>
              <Button size="sm" variant="outline" onClick={addItem}>
                <Plus className="mr-1 h-4 w-4" /> 添加
              </Button>
            </div>
            <div className="space-y-2">
              {items.map((it, idx) => (
                <div key={idx} className="grid grid-cols-12 items-end gap-2">
                  <div className="col-span-4 grid gap-1">
                    <Label className="text-xs">名称</Label>
                    <ComboboxInput
                      value={it.dye_name}
                      onChange={(v) => updateItem(idx, { dye_name: v })}
                      options={dyeLibrary}
                      placeholder="挑或输入新染料"
                    />
                  </div>
                  <div className="col-span-2 grid gap-1">
                    <Label className="text-xs">编号</Label>
                    <Input
                      value={it.dye_code ?? ''}
                      onChange={(e) =>
                        updateItem(idx, {
                          dye_code: e.target.value ? e.target.value : null,
                        })
                      }
                    />
                  </div>
                  <div className="col-span-2 grid gap-1">
                    <Label className="text-xs">数量</Label>
                    <Input
                      type="number"
                      min={0.0001}
                      step={0.01}
                      value={it.amount}
                      onChange={(e) => updateItem(idx, { amount: e.target.value })}
                    />
                  </div>
                  <div className="col-span-3 grid gap-1">
                    <Label className="text-xs">单位</Label>
                    <Select
                      value={it.unit}
                      onValueChange={(v) => updateItem(idx, { unit: v as Unit })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pct_owf">%（owf）</SelectItem>
                        <SelectItem value="g_per_kg">g/kg</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-1">
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => removeItem(idx)}
                      disabled={items.length === 1}
                      title="删除"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        <DialogFooter className="shrink-0 gap-2 border-t bg-background px-6 py-3">
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy || internal.trim().length === 0}>
            {busy ? '保存中…' : '保存'}
          </Button>
        </DialogFooter>
      </DialogContent>

      <UnknownDyesPromptDialog
        open={unknownDyes.length > 0}
        unknowns={unknownDyes}
        onConfirm={onUnknownDyesResolved}
        onCancel={onUnknownDyesCancel}
      />
    </Dialog>
  );
}

/// 色系下拉 + 直接编辑: 用户可以从已有列表点选, 也可以直接打字
/// 创建一个新色系. 点输入框右侧的箭头展开候选, 候选里支持模糊筛选;
/// 点候选项写回输入框, 失焦或回车关闭.
function ColorFamilyCombo({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (v: string) => void;
  options: string[];
}) {
  const [open, setOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  // 点其他地方关闭.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, [open]);

  const filtered = useMemo(() => {
    const q = value.trim();
    if (!q) return options;
    const lower = q.toLowerCase();
    return options.filter((o) => o.toLowerCase().includes(lower));
  }, [value, options]);

  return (
    <div ref={wrapperRef} className="relative">
      <div className="relative">
        <Input
          value={value}
          onChange={(e) => {
            onChange(e.target.value);
            if (!open) setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          placeholder="选已有色系或输入新的"
          className="pr-8"
        />
        <button
          type="button"
          onClick={() => setOpen((o) => !o)}
          className="absolute right-1 top-1/2 -translate-y-1/2 rounded p-1 text-muted-foreground hover:text-foreground"
          tabIndex={-1}
        >
          <ChevronsUpDown className="h-4 w-4" />
        </button>
      </div>
      {open && filtered.length > 0 && (
        <div className="absolute left-0 right-0 top-full z-50 mt-1 max-h-48 overflow-y-auto rounded-md border bg-popover shadow-md">
          {filtered.map((opt) => (
            <button
              key={opt}
              type="button"
              onClick={() => {
                onChange(opt);
                setOpen(false);
              }}
              className={cn(
                'flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-accent',
                opt === value && 'bg-accent',
              )}
            >
              <span>{opt}</span>
              {opt === value && <Check className="h-4 w-4" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function blankItem(sort: number): ItemForm {
  return {
    dye_name: '',
    dye_code: null,
    amount: '1',
    // 默认 g/kg: 染厂车间最常用的克 / 千克纤维, 比 % owf 更直观.
    unit: 'g_per_kg',
    sort_order: sort,
  };
}

function Field({
  label,
  required,
  hint,
  children,
}: {
  label: string;
  required?: boolean;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid gap-1">
      <Label>
        {label}
        {required && <span className="ml-1 text-destructive">*</span>}
      </Label>
      {children}
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  );
}

export default FormulaEditor;
