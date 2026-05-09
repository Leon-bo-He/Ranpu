import { CheckSquare, Copy, Loader2, Plus, Search, Square } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { BatchCopySummaryView, FormulaView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { EditModeToggle } from '@/components/EditModeToggle';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useEditModeStore } from '@/store/editMode';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';

export function DefaultLibraryPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const editEnabled = useEditModeStore((s) => s.formulaEditEnabled);
  const enableEdit = useEditModeStore((s) => s.enableFormulaEdit);
  const disableEdit = useEditModeStore((s) => s.disableFormulaEdit);
  const touchEdit = useEditModeStore((s) => s.touchFormulaActivity);

  const [keyword, setKeyword] = useState('');
  // 防抖关键词: 输入停 300ms 后才触发查询.
  const [debouncedKeyword, setDebouncedKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectionEnabled = hasWs;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [batchBusy, setBatchBusy] = useState(false);
  const [batchSummary, setBatchSummary] = useState<BatchCopySummaryView | null>(null);
  const [colorFamilies, setColorFamilies] = useState<string[]>([]);

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

      <EditModeToggle
        label="配方管理"
        whenOffCanStill="计算配方 / 加入批次清单 / 复制到工作区"
        enabled={editEnabled}
        onEnable={enableEdit}
        onDisable={disableEdit}
      />

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
    </div>
  );
}

export default DefaultLibraryPage;
