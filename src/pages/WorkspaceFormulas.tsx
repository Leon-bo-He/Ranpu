import { Loader2, Lock, Plus, Search } from 'lucide-react';
import { useEffect, useState } from 'react';

import { cartApi } from '@/api/cart';
import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { FormulaView, WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
import { useCartStaleGuard } from '@/components/CartStaleGuard';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
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
import { formatAmount } from '@/lib/format';
import { useEditModeStore } from '@/store/editMode';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';

export function WorkspaceFormulasPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;

  const [keyword, setKeyword] = useState('');
  // 防抖关键词: 输入停 300ms 后才触发查询, 避免每个键击都打 IPC.
  const [debouncedKeyword, setDebouncedKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<FormulaView | null>(null);
  const [activeWorkspace, setActiveWorkspace] = useState<WorkspaceView | null>(null);
  const [colorFamilies, setColorFamilies] = useState<string[]>([]);
  const editModeOn = useEditModeStore((s) => s.formulaEditEnabled);
  const touchEdit = useEditModeStore((s) => s.touchFormulaActivity);
  const isSystemMirror = activeWorkspace?.kind === 'system_mirror';
  // canEdit = (1) 不是 system_mirror 工作区 + (2) 配方管理开关已开启.
  const canEdit = !isSystemMirror && editModeOn;

  // 加入批次清单流程: 打开 dialog → 输入 kg → 处理冲突 (累加 / 覆盖).
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
  // 跨日工作前提醒清空昨天残留的批次清单. dialog 在 JSX 末尾渲染.
  const { guard: cartStaleGuard, dialog: cartStaleDialog } = useCartStaleGuard({
    onError: setCartErr,
  });

  const load = (kw: string = debouncedKeyword) => {
    if (!hasWs) {
      setList([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    return formulaApi
      .listWorkspace({ keyword: kw })
      .then(setList)
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
  }, [activeWorkspaceId, debouncedKeyword]);

  useEffect(() => {
    if (!activeWorkspaceId) {
      setActiveWorkspace(null);
      return;
    }
    workspaceApi
      .list()
      .then((all) => setActiveWorkspace(all.find((w) => w.id === activeWorkspaceId) ?? null))
      .catch(() => setActiveWorkspace(null));
  }, [activeWorkspaceId]);

  // 编辑器打开时, 拉一份当前工作区已用过的色系喂进 dropdown.
  useEffect(() => {
    if (!editorOpen || !hasWs) return;
    formulaApi
      .listWorkspaceColorFamilies()
      .then(setColorFamilies)
      .catch(() => setColorFamilies([]));
  }, [editorOpen, hasWs]);

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区。
      </p>
    );
  }

  const askDelete = (formula: FormulaView) => setPendingDelete(formula);

  const confirmDelete = async () => {
    if (!pendingDelete) return;
    try {
      await formulaApi.deleteWorkspace(pendingDelete.id);
      touchEdit();
      setPendingDelete(null);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setPendingDelete(null);
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertWorkspace>[0]) => {
    await formulaApi.upsertWorkspace(payload);
    touchEdit();
    setEditorOpen(false);
    load();
  };

  const onOpenAddToCart = (formula: FormulaView) => {
    cartStaleGuard(() => {
      setCartTarget(formula);
      setCartKg('10');
      setCartErr(null);
    });
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
        (l) => l.source_kind === 'workspace' && l.source_formula_id === cartTarget.id,
      );
      if (existing) {
        setConflict({ formula: cartTarget, addKg: kg, existingKg: existing.target_kg });
        setCartTarget(null);
      } else {
        await cartApi.add('workspace', cartTarget.id, kg);
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
        await cartApi.updateKg('workspace', formula.id, sum);
        setCartMsg(
          `已累加到批次清单：${formula.internal_color_code} · ${formatAmount(existingKg)} + ${formatAmount(addKg)} = ${formatAmount(sum)} kg`,
        );
      } else {
        await cartApi.add('workspace', formula.id, addKg);
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

  return (
    <div className="space-y-4 p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">
          工作区配方
          {activeWorkspace && (
            <span className="ml-2 text-sm font-normal text-muted-foreground">
              · {activeWorkspace.name}
            </span>
          )}
        </h2>
        <div className="flex flex-wrap items-center gap-2">
          {canEdit && (
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

      {isSystemMirror && (
        <div className="flex items-start gap-2 rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
          <Lock className="mt-0.5 h-4 w-4" />
          <p>
            「{activeWorkspace?.name}」是系统内置工作区，配方与
            <strong className="mx-1">默认配方库</strong>
            自动同步，无法在此处直接新建 / 编辑 / 删除。如需修改，请到默认配方库页面操作。
          </p>
        </div>
      )}

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
              source="workspace"
              canManage={canEdit}
              hasActiveWorkspace={hasWs}
              onEdit={
                canEdit
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={canEdit ? askDelete : undefined}
              onAddToCart={onOpenAddToCart}
            />
          ))}
        </div>
      )}

      <FormulaEditor
        open={editorOpen}
        onClose={() => setEditorOpen(false)}
        initial={editing}
        scope="工作区"
        colorFamilies={colorFamilies}
        onSave={onSave}
      />

      <ConfirmDialog
        open={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title="确认删除配方？"
        description={
          pendingDelete && (
            <>
              将永久删除当前工作区中的{' '}
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

      {cartStaleDialog}
    </div>
  );
}

export default WorkspaceFormulasPage;
