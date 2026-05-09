import { Check, ChevronDown, FolderClosed, Loader2, Search } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { ApiError } from '@/api/invoke';
import type { WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useSessionStore } from '@/store/session';
import { useWorkspacesStore } from '@/store/workspaces';

const DEBOUNCE_MS = 300;

/// 顶栏单一入口: 显示当前工作区, 点开 popover 是搜索 + 全部工作区的过滤列表.
/// 合并了原来的 WorkspaceSwitcher (Select) 和 WorkspaceSearch — 它们功能重叠.
///
/// 选中后 cmd_switch_workspace + 跳到 /workspace-formulas. 选中"未选择工作区"
/// 则只切换不跳转, 让在其他页面 (设置 / 审计) 的用户能临时退出工作区上下文.
export function WorkspacePicker() {
  const session = useSessionStore((s) => s.session);
  const setActiveWorkspace = useSessionStore((s) => s.setActiveWorkspace);
  const workspaces = useWorkspacesStore((s) => s.list);
  const refresh = useWorkspacesStore((s) => s.refresh);
  const navigate = useNavigate();

  const [open, setOpen] = useState(false);
  const [keyword, setKeyword] = useState('');
  const [debouncedKeyword, setDebouncedKeyword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  // 进入应用首次拉一遍工作区列表; 之后改动由各页面触发 refresh.
  useEffect(() => {
    if (!session) return;
    refresh();
  }, [session, refresh]);

  // 防抖关键词.
  useEffect(() => {
    const t = setTimeout(() => setDebouncedKeyword(keyword.trim()), DEBOUNCE_MS);
    return () => clearTimeout(t);
  }, [keyword]);

  // 点外部关 popover.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
        setKeyword('');
        setDebouncedKeyword('');
        setError(null);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, [open]);

  // 打开时自动聚焦输入框.
  useEffect(() => {
    if (open) {
      // 让 popover 先挂载再聚焦.
      const t = setTimeout(() => inputRef.current?.focus(), 0);
      return () => clearTimeout(t);
    }
  }, [open]);

  const activeWorkspace =
    workspaces.find((w) => w.id === session?.active_workspace_id) ?? null;

  // 大小写无关 substring 匹配 name / description; 空关键词显示全部.
  const filtered: WorkspaceView[] = useMemo(() => {
    if (!debouncedKeyword) return workspaces;
    const needle = debouncedKeyword.toLowerCase();
    return workspaces.filter((w) => {
      if (w.name.toLowerCase().includes(needle)) return true;
      if (w.description?.toLowerCase().includes(needle)) return true;
      return false;
    });
  }, [workspaces, debouncedKeyword]);

  const closePopover = () => {
    setOpen(false);
    setKeyword('');
    setDebouncedKeyword('');
    setError(null);
  };

  const onPick = async (workspace: WorkspaceView | null) => {
    setError(null);
    const targetId = workspace?.id ?? null;
    if (session?.active_workspace_id !== targetId) {
      setBusy(true);
      try {
        await workspaceApi.switch(targetId);
        setActiveWorkspace(targetId);
      } catch (e) {
        setError(e instanceof ApiError ? e.message : String(e));
        setBusy(false);
        return;
      }
      setBusy(false);
    }
    closePopover();
    // 切到具体工作区跳到配方页 (search-and-jump 行为); 取消选择则停在当前页.
    if (workspace) navigate('/workspace-formulas');
  };

  if (!session) return null;

  return (
    <div ref={containerRef} className="relative">
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-9 min-w-[200px] justify-start gap-2 px-3"
        onClick={() => setOpen((v) => !v)}
        aria-haspopup="listbox"
        aria-expanded={open}
      >
        <FolderClosed className="h-4 w-4 text-muted-foreground" />
        <span className="flex-1 truncate text-left">
          {activeWorkspace ? (
            activeWorkspace.name
          ) : (
            <span className="text-muted-foreground">未选择工作区</span>
          )}
        </span>
        {busy ? (
          <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
        ) : (
          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        )}
      </Button>
      {open && (
        <div className="absolute left-0 top-full z-50 mt-1 w-[320px] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-lg">
          <div className="border-b p-2">
            <div className="relative">
              <Search className="pointer-events-none absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                ref={inputRef}
                className="h-9 pl-8"
                placeholder="搜索工作区：名字 / 描述"
                value={keyword}
                onChange={(e) => setKeyword(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Escape') closePopover();
                }}
                aria-label="搜索工作区"
              />
            </div>
          </div>
          {error && (
            <div className="border-b border-destructive/30 bg-destructive/5 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          )}
          <ul className="max-h-[360px] overflow-auto py-1">
            {/* 未选择工作区: 跟原 Select 的 __none__ 等价, 让用户能退出工作区上下文. */}
            <li>
              <button
                type="button"
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-accent"
                onClick={() => onPick(null)}
              >
                {activeWorkspace === null ? (
                  <Check className="h-4 w-4 text-primary" />
                ) : (
                  <span className="h-4 w-4" aria-hidden />
                )}
                <span className="text-muted-foreground">未选择工作区</span>
              </button>
            </li>
            {filtered.length === 0 && debouncedKeyword.length > 0 && (
              <li className="px-3 py-6 text-center text-xs text-muted-foreground">
                没有匹配的工作区
              </li>
            )}
            {filtered.map((w) => {
              const isActive = activeWorkspace?.id === w.id;
              return (
                <li key={w.id}>
                  <button
                    type="button"
                    className="flex w-full items-start gap-2 px-3 py-2 text-left text-sm hover:bg-accent"
                    onClick={() => onPick(w)}
                  >
                    {isActive ? (
                      <Check className="mt-0.5 h-4 w-4 text-primary" />
                    ) : (
                      <span className="mt-0.5 h-4 w-4" aria-hidden />
                    )}
                    <div className="flex min-w-0 flex-1 flex-col">
                      <div className="flex items-center gap-2">
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
                    </div>
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
