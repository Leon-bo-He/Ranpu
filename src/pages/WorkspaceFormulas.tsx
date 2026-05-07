import { CheckSquare, Loader2, Plus, Search, Square } from 'lucide-react';
import { useEffect, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { FormulaView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { hasActiveWorkspace, isAdmin, useSessionStore } from '@/store/session';

export function WorkspaceFormulasPage() {
  const session = useSessionStore((s) => s.session);
  const admin = isAdmin(session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;

  const [keyword, setKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<FormulaView | null>(null);

  const selectionEnabled = admin;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());

  const load = () => {
    if (!hasWs) {
      setList([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    return formulaApi
      .listWorkspace({ keyword })
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

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeWorkspaceId]);

  useEffect(() => {
    setSelectedIds(new Set());
  }, [activeWorkspaceId]);

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
      setPendingDelete(null);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setPendingDelete(null);
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertWorkspace>[0]) => {
    await formulaApi.upsertWorkspace(payload);
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

  return (
    <div className="space-y-4 p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">工作区配方</h2>
        <div className="flex flex-wrap items-center gap-2">
          {selectionEnabled && (
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
              canManage={admin}
              hasActiveWorkspace={hasWs}
              onEdit={
                admin
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={admin ? askDelete : undefined}
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
        scope="工作区"
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
              {pendingDelete.color_name && <> · {pendingDelete.color_name}</>}
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

export default WorkspaceFormulasPage;
