import { Loader2, Search } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { ApiError } from '@/api/invoke';
import type { WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
import { Input } from '@/components/ui/input';
import { useSessionStore } from '@/store/session';
import { useWorkspacesStore } from '@/store/workspaces';

const DEBOUNCE_MS = 300;
const MAX_RESULTS = 50;

/// 顶栏全局搜索: 按名字 / 描述模糊匹配工作区, 选中后切换并跳到工作区配方页.
/// 工作区列表已经被 useWorkspacesStore 缓存, 走客户端过滤即可, 不打 IPC.
export function WorkspaceSearch() {
  const session = useSessionStore((s) => s.session);
  const setActiveWorkspace = useSessionStore((s) => s.setActiveWorkspace);
  const workspaces = useWorkspacesStore((s) => s.list);
  const refresh = useWorkspacesStore((s) => s.refresh);
  const navigate = useNavigate();

  const [keyword, setKeyword] = useState('');
  const [debouncedKeyword, setDebouncedKeyword] = useState('');
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement | null>(null);

  // 进入应用首次拿一遍工作区列表; 之后改动由各页面 refresh.
  useEffect(() => {
    if (!session) return;
    refresh();
  }, [session, refresh]);

  // 防抖: 输入太快不重算结果.
  useEffect(() => {
    const t = setTimeout(() => setDebouncedKeyword(keyword.trim()), DEBOUNCE_MS);
    return () => clearTimeout(t);
  }, [keyword]);

  // 点击外部关下拉.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, [open]);

  // 大小写无关 substring 匹配 name / description; system_mirror 也参与命中,
  // 但视觉上加个 "系统" 标签提示.
  const hits: WorkspaceView[] = useMemo(() => {
    if (!debouncedKeyword) return [];
    const needle = debouncedKeyword.toLowerCase();
    return workspaces
      .filter((w) => {
        if (w.name.toLowerCase().includes(needle)) return true;
        if (w.description?.toLowerCase().includes(needle)) return true;
        return false;
      })
      .slice(0, MAX_RESULTS);
  }, [workspaces, debouncedKeyword]);

  const onPick = async (workspace: WorkspaceView) => {
    setOpen(false);
    setKeyword('');
    setDebouncedKeyword('');
    setError(null);
    if (session?.active_workspace_id !== workspace.id) {
      setBusy(true);
      try {
        await workspaceApi.switch(workspace.id);
        setActiveWorkspace(workspace.id);
      } catch (e) {
        setError(e instanceof ApiError ? e.message : String(e));
        setBusy(false);
        return;
      }
      setBusy(false);
    }
    navigate('/workspace-formulas');
  };

  if (!session) return null;
  const showDropdown = open && debouncedKeyword.length > 0;

  return (
    <div ref={containerRef} className="relative w-[280px]">
      <div className="relative">
        <Search className="pointer-events-none absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          className="pl-8"
          placeholder="搜索工作区: 名字 / 描述"
          value={keyword}
          onChange={(e) => {
            setKeyword(e.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          aria-label="搜索工作区"
        />
        {busy && (
          <Loader2 className="absolute right-2 top-2.5 h-4 w-4 animate-spin text-muted-foreground" />
        )}
      </div>
      {showDropdown && (
        <div className="absolute left-0 right-0 top-full z-50 mt-1 max-h-[420px] overflow-auto rounded-md border bg-popover text-popover-foreground shadow-lg">
          {error && (
            <div className="border-b border-destructive/30 bg-destructive/5 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          )}
          {hits.length === 0 && !error && (
            <div className="px-3 py-6 text-center text-xs text-muted-foreground">
              没有匹配的工作区
            </div>
          )}
          <ul>
            {hits.map((w) => {
              const isActive = session.active_workspace_id === w.id;
              return (
                <li key={w.id}>
                  <button
                    type="button"
                    className="flex w-full flex-col items-start gap-0.5 px-3 py-2 text-left text-sm hover:bg-accent"
                    onClick={() => onPick(w)}
                  >
                    <div className="flex w-full items-center gap-2">
                      <span className="truncate font-medium">{w.name}</span>
                      {w.kind === 'system_mirror' && (
                        <span className="rounded bg-muted px-1 text-[10px] text-muted-foreground">
                          系统
                        </span>
                      )}
                      {isActive && (
                        <span className="ml-auto text-[10px] text-primary">当前</span>
                      )}
                    </div>
                    {w.description && (
                      <span className="line-clamp-1 text-xs text-muted-foreground">
                        {w.description}
                      </span>
                    )}
                  </button>
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
}
