import { CheckSquare, Copy, Loader2, Plus, Search, Square } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { BatchCopySummaryView, FormulaView } from '@/api/types';
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
import { hasActiveWorkspace, isAdmin, useSessionStore } from '@/store/session';

export function DefaultLibraryPage() {
  const session = useSessionStore((s) => s.session);
  const admin = isAdmin(session);
  const hasWs = hasActiveWorkspace(session);

  const [keyword, setKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  /** 第一次拉数据完成前为 true; 之后只在显式刷新时短暂为 true。 */
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 多选状态：仅 admin + 已激活 workspace 时启用
  const selectionEnabled = admin && hasWs;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [batchBusy, setBatchBusy] = useState(false);
  const [batchSummary, setBatchSummary] = useState<BatchCopySummaryView | null>(null);

  const load = (kw?: string) => {
    setLoading(true);
    return formulaApi
      .listDefault({ keyword: kw ?? keyword })
      .then((data) => {
        setList(data);
        // 列表刷新后清掉无效的 selected id
        setSelectedIds((prev) => {
          const allIds = new Set(data.map((f) => f.id));
          return new Set([...prev].filter((id) => allIds.has(id)));
        });
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 失去 selectionEnabled 资格时清空选择，避免状态不一致
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

  const onDelete = async (formula: FormulaView) => {
    if (!confirm(`确认删除「${formula.internal_color_code}」？`)) return;
    try {
      await formulaApi.deleteDefault(formula.id);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertDefault>[0]) => {
    await formulaApi.upsertDefault(payload);
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

  const allSelected =
    list.length > 0 && list.every((f) => selectedIds.has(f.id));

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
      // 清空选择，方便下一轮
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
          {admin && (
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

      <div className="flex items-center gap-2 max-w-md">
        <div className="relative flex-1">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-8"
            placeholder="搜索内部色号 / 客户色号 / 颜色俗称"
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && load()}
          />
        </div>
        <Button variant="outline" onClick={() => load()}>
          搜索
        </Button>
      </div>

      {selectionEnabled && selectedIds.size > 0 && (
        <p className="text-xs text-muted-foreground">
          已选 <Badge variant="default">{selectedIds.size}</Badge> 条配方
        </p>
      )}
      {admin && !hasWs && (
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
              canManage={admin}
              hasActiveWorkspace={hasWs}
              onCopyToWorkspace={onCopyToWorkspace}
              onEdit={
                admin
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={admin ? onDelete : undefined}
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
    </div>
  );
}

export default DefaultLibraryPage;
