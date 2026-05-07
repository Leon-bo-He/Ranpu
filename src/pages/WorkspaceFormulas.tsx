import { Loader2, Plus, Search } from 'lucide-react';
import { useEffect, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { FormulaView } from '@/api/types';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
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

  const load = () => {
    if (!hasWs) {
      setList([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    return formulaApi
      .listWorkspace({ keyword })
      .then(setList)
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)))
      .finally(() => setLoading(false));
  };

  // 依赖 active_workspace_id 而非 hasWs (boolean), 不然 A→B 切换不会触发刷新.
  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeWorkspaceId]);

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区。
      </p>
    );
  }

  const onDelete = async (formula: FormulaView) => {
    if (!confirm(`确认删除「${formula.internal_color_code}」？`)) return;
    try {
      await formulaApi.deleteWorkspace(formula.id);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertWorkspace>[0]) => {
    await formulaApi.upsertWorkspace(payload);
    setEditorOpen(false);
    load();
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">工作区配方</h2>
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
              onDelete={admin ? onDelete : undefined}
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
    </div>
  );
}

export default WorkspaceFormulasPage;
