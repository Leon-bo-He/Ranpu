import { Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import type { FormulaItemDto, UpsertFormulaPayload } from '@/api/formula';
import type { FormulaView, Unit } from '@/api/types';
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
import { Textarea } from '@/components/ui/textarea';

interface FormulaEditorProps {
  open: boolean;
  onClose: () => void;
  initial: FormulaView | null;
  /** 标题文案，前缀，比如 "默认配方" 或 "工作区配方"。 */
  scope: string;
  /** 提交时调用，返回 Promise 让对话框知道何时关闭 / 显示错误。 */
  onSave: (payload: UpsertFormulaPayload) => Promise<void>;
}

export function FormulaEditor({
  open,
  onClose,
  initial,
  scope,
  onSave,
}: FormulaEditorProps) {
  const [internal, setInternal] = useState('');
  const [customer, setCustomer] = useState('');
  const [colorName, setColorName] = useState('');
  const [description, setDescription] = useState('');
  const [baseKg, setBaseKg] = useState('');
  const [ratio, setRatio] = useState('');
  const [notes, setNotes] = useState('');
  const [items, setItems] = useState<FormulaItemDto[]>([blankItem(0)]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    if (initial) {
      setInternal(initial.internal_color_code);
      setCustomer(initial.customer_color_code ?? '');
      setColorName(initial.color_name ?? '');
      setDescription(initial.description ?? '');
      setBaseKg(initial.base_weight_kg !== null ? String(initial.base_weight_kg) : '');
      setRatio(initial.liquor_ratio !== null ? String(initial.liquor_ratio) : '');
      setNotes(initial.notes ?? '');
      setItems(
        initial.items.map((i, idx) => ({
          dye_name: i.dye_name,
          dye_code: i.dye_code,
          amount: i.amount,
          unit: i.unit,
          sort_order: idx,
        })),
      );
    } else {
      setInternal('');
      setCustomer('');
      setColorName('');
      setDescription('');
      setBaseKg('');
      setRatio('');
      setNotes('');
      setItems([blankItem(0)]);
    }
    setError(null);
  }, [open, initial]);

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      const payload: UpsertFormulaPayload = {
        id: initial?.id ?? null,
        internal_color_code: internal.trim(),
        customer_color_code: customer.trim() ? customer.trim() : null,
        color_name: colorName.trim() ? colorName.trim() : null,
        description: description.trim() ? description.trim() : null,
        base_weight_kg: baseKg ? Number(baseKg) : null,
        liquor_ratio: ratio ? Number(ratio) : null,
        notes: notes.trim() ? notes.trim() : null,
        items: items.map((it, idx) => ({ ...it, sort_order: idx })),
      };
      await onSave(payload);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const addItem = () =>
    setItems((prev) => [...prev, blankItem(prev.length)]);

  const updateItem = (idx: number, patch: Partial<FormulaItemDto>) =>
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
            内部色号必填且唯一; g/L 单位的染料需要设置浴比。
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
          <Field label="颜色俗称">
            <Input value={colorName} onChange={(e) => setColorName(e.target.value)} />
          </Field>
          <Field label="基础重量 (kg)" hint="首次调色时所用的重量，仅作记录参考">
            <Input
              type="number"
              min={0.01}
              max={99999.99}
              step={0.01}
              value={baseKg}
              onChange={(e) => setBaseKg(e.target.value)}
            />
          </Field>
          <Field label="浴比 1:N">
            <Input
              type="number"
              min={0.1}
              step={0.1}
              value={ratio}
              onChange={(e) => setRatio(e.target.value)}
            />
          </Field>
        </div>

        <div className="grid gap-1">
          <Label>说明</Label>
          <Textarea
            rows={2}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>
        <div className="grid gap-1">
          <Label>备注</Label>
          <Textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
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
                  <Input
                    value={it.dye_name}
                    onChange={(e) => updateItem(idx, { dye_name: e.target.value })}
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
                    value={Number.isFinite(it.amount) ? String(it.amount) : ''}
                    onChange={(e) =>
                      updateItem(idx, { amount: Number(e.target.value) })
                    }
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
                      <SelectItem value="pct_owf">% (owf)</SelectItem>
                      <SelectItem value="g_per_kg">g/kg</SelectItem>
                      <SelectItem value="g_per_L">g/L</SelectItem>
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
    </Dialog>
  );
}

function blankItem(sort: number): FormulaItemDto {
  return {
    dye_name: '',
    dye_code: null,
    amount: 1,
    unit: 'pct_owf',
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
