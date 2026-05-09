import { CheckSquare, Copy, Loader2, Plus, Search, Square } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { cartApi } from '@/api/cart';
import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { BatchCopySummaryView, FormulaView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
import { Badge } from '@/components/ui/badge';
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
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatAmount } from '@/lib/format';
import { useEditModeStore } from '@/store/editMode';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';

export function DefaultLibraryPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const editEnabled = useEditModeStore((s) => s.formulaEditEnabled);
  const touchEdit = useEditModeStore((s) => s.touchFormulaActivity);

  const [keyword, setKeyword] = useState('');
  // 防抖关键词: 输入停 300ms 后才触发查询.
  const [debouncedKeyword, setDebouncedKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 批量选 + 复制到工作区 都是写操作 (会在目标工作区生成新配方),
  // 所以同时受 配方管理 toggle 限制 — 没开就不显示这一组按钮.
  const selectionEnabled = hasWs && editEnabled;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [batchBusy, setBatchBusy] = useState(false);
  const [batchSummary, setBatchSummary] = useState<BatchCopySummaryView | null>(null);
  const [colorFamilies, setColorFamilies] = useState<string[]>([]);

  // 加入批次清单流程: 跟 WorkspaceFormulas 一致的弹窗 → kg 输入 → 冲突 (累加 / 覆盖).
  // 默认配方加车走 source_kind='default', 仍记到当前工作区的 cart 下.
  const [cartTarget, setCartTarget] = useState<FormulaView | null>(null);
  const [cartKg, setCartKg] = useState('10');
  const [cartBusy, setCartBusy] = useState(false);
  const [cartErr, setCartErr] = useState<string | null>(null);
  const [cartMsg, setCartMsg] = useState<string | null>(null);
  const [conflict, setConflict] = useState<{
    formula: FormulaView;
    addKg: number;
    existingKg: number;
  } | null>(null);

  const onOpenAddToCart = (formula: FormulaView) => {
    setCartTarget(formula);
    setCartKg('10');
    setCartErr(null);
  };

  const onConfirmAddToCart = async () => {
    if (!cartTarget) return;
    const kg = Number(cartKg);
    if (!Number.isFinite(kg) || kg <= 0 || kg > 99999.99) {
      setCartErr('目标 kg 需在 0.01 – 99999.99 之间');
      return;
    }
    setCartBusy(true);
    setCartErr(null);
    try {
      const cart = await cartApi.list();
      const existing = cart.find(
        (l) => l.source_kind === 'default' && l.source_formula_id === cartTarget.id,
      );
      if (existing) {
        setConflict({ formula: cartTarget, addKg: kg, existingKg: existing.target_kg });
        setCartTarget(null);
      } else {
        await cartApi.add('default', cartTarget.id, kg);
        setCartMsg(
          `已加入批次清单：${cartTarget.internal_color_code} · ${formatAmount(kg)} kg`,
        );
        setCartTarget(null);
      }
    } catch (e) {
      setCartErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setCartBusy(false);
    }
  };

  const resolveConflict = async (action: 'accumulate' | 'replace') => {
    if (!conflict) return;
    const { formula, addKg, existingKg } = conflict;
    setCartBusy(true);
    setCartErr(null);
    try {
      if (action === 'accumulate') {
        const sum = Math.min(existingKg + addKg, 99999.99);
        await cartApi.updateKg('default', formula.id, sum);
        setCartMsg(
          `已累加到批次清单：${formula.internal_color_code} · ${formatAmount(existingKg)} + ${formatAmount(addKg)} = ${formatAmount(sum)} kg`,
        );
      } else {
        await cartApi.add('default', formula.id, addKg);
        setCartMsg(
          `已覆盖批次清单 kg：${formula.internal_color_code} · ${formatAmount(existingKg)} → ${formatAmount(addKg)} kg`,
        );
      }
      setConflict(null);
    } catch (e) {
      setCartErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setCartBusy(false);
    }
  };

  // 编辑器打开时, 拉一份已用过的色系喂进 dropdown.
  useEffect(() => {
    if (!editorOpen) return;
    formulaApi
      .listDefaultColorFamilies()
      .then(setColorFamilies)
      .catch(() => setColorFamilies([]));
  }, [editorOpen]);

  const load = (kw: string = debouncedKeyword) => {
    setLoading(true);
    return formulaApi
      .listDefault({ keyword: kw })
      .then((data) => {
        setList(data);
        setSelectedIds((prev) => {
          const allIds = new Set(data.map((f) => f.id));
          return new Set([...prev].filter((id) => allIds.has(id)));
        });
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)))
      .finally(() => setLoading(false));
  };

  // 把 keyword 防抖成 debouncedKeyword.
  useEffect(() => {
    const t = setTimeout(() => setDebouncedKeyword(keyword), 300);
    return () => clearTimeout(t);
  }, [keyword]);

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedKeyword]);

  useEffect(() => {
    if (!selectionEnabled) setSelectedIds(new Set());
  }, [selectionEnabled]);

  const onCopyToWorkspace = async (formula: FormulaView) => {
    try {
      await formulaApi.copyDefaultToWorkspace(formula.id);
      alert('已复制到当前工作区');
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const [pendingDelete, setPendingDelete] = useState<FormulaView | null>(null);
  const askDelete = (formula: FormulaView) => setPendingDelete(formula);
  const confirmDelete = async () => {
    if (!pendingDelete) return;
    try {
      await formulaApi.deleteDefault(pendingDelete.id);
      touchEdit();
      setPendingDelete(null);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setPendingDelete(null);
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertDefault>[0]) => {
    await formulaApi.upsertDefault(payload);
    touchEdit();
    setEditorOpen(false);
    load();
  };

  const onToggleSelected = (formula: FormulaView, next: boolean) => {
    setSelectedIds((prev) => {
      const out = new Set(prev);
      if (next) out.add(formula.id);
      else out.delete(formula.id);
      return out;
    });
  };

  const allSelected = list.length > 0 && list.every((f) => selectedIds.has(f.id));
  const onToggleSelectAll = () => {
    if (allSelected) setSelectedIds(new Set());
    else setSelectedIds(new Set(list.map((f) => f.id)));
  };

  const onBatchCopy = async () => {
    if (selectedIds.size === 0) return;
    setBatchBusy(true);
    setError(null);
    try {
      const summary = await formulaApi.batchCopyDefaultToWorkspace([...selectedIds]);
      setBatchSummary(summary);
      setSelectedIds(new Set());
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBatchBusy(false);
    }
  };

  const failedItems = useMemo(
    () => batchSummary?.items.filter((i) => i.error !== null) ?? [],
    [batchSummary],
  );

  return (
    <div className="space-y-4 p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">默认配方库</h2>
        <div className="flex flex-wrap items-center gap-2">
          {selectionEnabled && (
            <>
              <Button
                size="sm"
                variant="outline"
                onClick={onToggleSelectAll}
                disabled={list.length === 0}
              >
                {allSelected ? (
                  <CheckSquare className="mr-1 h-4 w-4" />
                ) : (
                  <Square className="mr-1 h-4 w-4" />
                )}
                {allSelected ? '取消全选' : '全选当前页'}
              </Button>
              <Button
                size="sm"
                disabled={selectedIds.size === 0 || batchBusy}
                onClick={onBatchCopy}
              >
                <Copy className="mr-1 h-4 w-4" />
                {batchBusy
                  ? '正在复制…'
                  : `批量复制到工作区${selectedIds.size > 0 ? ` (${selectedIds.size})` : ''}`}
              </Button>
            </>
          )}
          {editEnabled && (
            <Button
              onClick={() => {
                setEditing(null);
                setEditorOpen(true);
              }}
            >
              <Plus className="mr-1 h-4 w-4" /> 新建配方
            </Button>
          )}
        </div>
      </div>

      <div className="max-w-md">
        <div className="relative">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-8"
            placeholder="搜索内部色号 / 客户色号 / 色系"
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
          />
        </div>
      </div>

      {selectionEnabled && selectedIds.size > 0 && (
        <p className="text-xs text-muted-foreground">
          已选 <Badge variant="default">{selectedIds.size}</Badge> 条配方
        </p>
      )}
      {!hasWs && (
        <p className="text-xs text-muted-foreground">
          想批量复制配方？请先在顶栏选择目标工作区。
        </p>
      )}

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading && list.length === 0 ? (
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          正在加载…
        </div>
      ) : list.length === 0 ? (
        <p className="text-sm text-muted-foreground">没有匹配的配方。</p>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {list.map((f) => (
            <FormulaCard
              key={f.id}
              formula={f}
              source="default"
              canManage={editEnabled}
              hasActiveWorkspace={hasWs}
              onCopyToWorkspace={onCopyToWorkspace}
              onEdit={
                editEnabled
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={editEnabled ? askDelete : undefined}
              // 加入批次清单 不受 配方管理 toggle 限制 (跟工作区配方页一致),
              // 但批次清单按工作区维护, 没激活 workspace 就不显示按钮.
              onAddToCart={hasWs ? onOpenAddToCart : undefined}
              selected={selectionEnabled ? selectedIds.has(f.id) : undefined}
              onToggleSelected={selectionEnabled ? onToggleSelected : undefined}
            />
          ))}
        </div>
      )}

      <FormulaEditor
        open={editorOpen}
        onClose={() => setEditorOpen(false)}
        initial={editing}
        scope="默认"
        colorFamilies={colorFamilies}
        onSave={onSave}
      />

      <Dialog
        open={batchSummary !== null}
        onOpenChange={(o) => !o && setBatchSummary(null)}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>批量复制结果</DialogTitle>
          </DialogHeader>
          <div className="text-sm">
            成功 <Badge variant="default">{batchSummary?.succeeded ?? 0}</Badge>{' '}
            条，失败 <Badge variant="destructive">{batchSummary?.failed ?? 0}</Badge>{' '}
            条。
          </div>
          {failedItems.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>源配方 ID</TableHead>
                  <TableHead>错误</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {failedItems.map((i) => (
                  <TableRow key={i.source_default_id}>
                    <TableCell className="font-mono">{i.source_default_id}</TableCell>
                    <TableCell className="text-destructive">{i.error}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
          <DialogFooter>
            <Button onClick={() => setBatchSummary(null)}>知道了</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title="确认删除配方？"
        description={
          pendingDelete && (
            <>
              将永久删除默认库中的{' '}
              <span className="font-mono">{pendingDelete.internal_color_code}</span>
              {pendingDelete.color_family && <> · {pendingDelete.color_family}</>}
              {' '}及其所有染料明细，操作不可撤销。
            </>
          )
        }
        confirmLabel="删除"
        destructive
        onConfirm={confirmDelete}
      />

      <Dialog
        open={cartTarget !== null}
        onOpenChange={(o) => {
          if (!o) {
            setCartTarget(null);
            setCartErr(null);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>加入批次清单</DialogTitle>
            <DialogDescription>
              {cartTarget && (
                <>
                  <span className="font-mono">{cartTarget.internal_color_code}</span>
                  {cartTarget.color_family && <> · {cartTarget.color_family}</>}
                </>
              )}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-2">
            <Label>目标 kg</Label>
            <Input
              type="number"
              min={0.01}
              max={99999.99}
              step={0.01}
              value={cartKg}
              onChange={(e) => setCartKg(e.target.value)}
              disabled={cartBusy}
              autoFocus
            />
          </div>
          {cartErr && <p className="text-sm text-destructive">{cartErr}</p>}
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => setCartTarget(null)}
              disabled={cartBusy}
            >
              取消
            </Button>
            <Button onClick={onConfirmAddToCart} disabled={cartBusy || !cartKg}>
              {cartBusy ? '加入中…' : '加入批次清单'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={conflict !== null}
        onOpenChange={(o) => !o && setConflict(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>批次清单里已有这条配方</DialogTitle>
            <DialogDescription>
              {conflict && (
                <>
                  <span className="font-mono">
                    {conflict.formula.internal_color_code}
                  </span>{' '}
                  当前批次清单记录{' '}
                  <span className="font-mono">{formatAmount(conflict.existingKg)}</span>{' '}
                  kg，本次想加的是{' '}
                  <span className="font-mono">{formatAmount(conflict.addKg)}</span> kg。
                </>
              )}
            </DialogDescription>
          </DialogHeader>
          <div className="rounded-md border bg-muted/30 p-3 text-sm">
            选择处理方式：
            <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
              <li>
                · 累加：把这次的 kg 加到批次清单现有 kg 上（
                {conflict
                  ? `${formatAmount(conflict.existingKg)} + ${formatAmount(conflict.addKg)} = ${formatAmount(
                      Math.min(conflict.existingKg + conflict.addKg, 99999.99),
                    )} kg`
                  : ''}
                ）
              </li>
              <li>
                · 覆盖：用本次的 kg 直接替换掉批次清单里的 kg（
                {conflict ? `${formatAmount(conflict.addKg)} kg` : ''}）
              </li>
            </ul>
          </div>
          {cartErr && <p className="text-sm text-destructive">{cartErr}</p>}
          <DialogFooter className="gap-2">
            <Button variant="ghost" onClick={() => setConflict(null)}>
              取消
            </Button>
            <Button
              variant="outline"
              disabled={cartBusy}
              onClick={() => resolveConflict('replace')}
            >
              {cartBusy ? '处理中…' : '覆盖'}
            </Button>
            <Button
              disabled={cartBusy}
              onClick={() => resolveConflict('accumulate')}
            >
              {cartBusy ? '处理中…' : '累加'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {cartMsg && (
        <div
          aria-live="polite"
          className="fixed bottom-6 right-6 rounded-md border border-emerald-300 bg-emerald-50 px-4 py-2 text-sm text-emerald-900 shadow-md"
        >
          <div className="flex items-center gap-3">
            <span>{cartMsg}</span>
            <button
              className="text-emerald-900/60 hover:text-emerald-900"
              onClick={() => setCartMsg(null)}
              aria-label="关闭"
            >
              ×
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default DefaultLibraryPage;
