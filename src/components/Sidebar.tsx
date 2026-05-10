import {
  BookOpen,
  Calculator,
  ClipboardList,
  Home,
  Info,
  Layers,
  PackageOpen,
  Settings as Cog,
  ShoppingCart,
} from 'lucide-react';
import { NavLink } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';

import { cn } from '@/lib/utils';
import { useEditModeStore } from '@/store/editMode';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';
import { useUpdateStore } from '@/store/update';

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  /** 没激活 workspace 时禁用（仍显示，灰掉 + tooltip 提示）。 */
  needsActiveWorkspace?: boolean;
}

type NavEntry = NavItem | { divider: true };

const isDivider = (e: NavEntry): e is { divider: true } => 'divider' in e;

const NAV_ITEMS: NavEntry[] = [
  { to: '/', label: '主面板', icon: Home },
  { divider: true },
  { to: '/default-library', label: '默认配方库', icon: BookOpen },
  {
    to: '/workspace-formulas',
    label: '工作区配方',
    icon: Layers,
    needsActiveWorkspace: true,
  },
  {
    to: '/calculator',
    label: '染料计算器',
    icon: Calculator,
    needsActiveWorkspace: true,
  },
  {
    to: '/cart',
    label: '批次清单',
    icon: ShoppingCart,
    needsActiveWorkspace: true,
  },
  { divider: true },
  { to: '/workspaces', label: '工作区管理', icon: Layers },
  { to: '/audit', label: '审计日志', icon: ClipboardList },
  { to: '/library-transfer', label: '配方互导', icon: PackageOpen },
  { to: '/settings', label: '设置', icon: Cog },
  { to: '/about', label: '关于', icon: Info },
];

export function Sidebar() {
  const session = useSessionStore((s) => s.session);
  // 全局更新状态: 有 pending 就在 "关于" 项右边贴个红点提示新版本.
  const hasUpdate = useUpdateStore((s) => s.pending !== null);
  // 工作区管理 / 审计日志 / 配方互导 入口跟随对应 toggle 隐藏 — 关闭时
  // 直接从侧栏移除, 用户要重新看到入口需要去 "设置 → 管理模式" 打开
  // (配方互导额外要求输入启动口令).
  const workspaceEditOn = useEditModeStore((s) => s.workspaceEditEnabled);
  const auditDisplayOn = useEditModeStore((s) => s.auditDisplayEnabled);
  const libraryTransferOn = useEditModeStore((s) => s.libraryTransferEnabled);
  if (!session) return null;
  const hasWs = hasActiveWorkspace(session);

  // 过滤: 工作区管理 / 审计日志 toggle 关闭时移除对应入口. 同时合并连续的
  // divider — 比如 "工作区管理" 被隐藏后, 它前面的分界线和它后面那段如果
  // 都还在, 不会出现两条紧贴的分界线 (因为没有连续 divider 这种情况);
  // 但万一 divider 后面紧跟的整段全被隐藏, 会出现 "悬空" divider, 需要剔除.
  const visibleEntries: NavEntry[] = [];
  NAV_ITEMS.forEach((entry) => {
    if (isDivider(entry)) {
      visibleEntries.push(entry);
      return;
    }
    if (entry.to === '/workspaces' && !workspaceEditOn) return;
    if (entry.to === '/audit' && !auditDisplayOn) return;
    if (entry.to === '/library-transfer' && !libraryTransferOn) return;
    visibleEntries.push(entry);
  });
  // 剔除 "悬空" / 重复的 divider: 列表头尾的, 以及连续的两条.
  const cleaned: NavEntry[] = [];
  visibleEntries.forEach((entry, idx) => {
    if (isDivider(entry)) {
      const prev = cleaned[cleaned.length - 1];
      if (!prev || isDivider(prev)) return;
      // divider 后面如果只剩 divider / 没东西, 也丢掉.
      const tail = visibleEntries.slice(idx + 1).filter((e) => !isDivider(e));
      if (tail.length === 0) return;
    }
    cleaned.push(entry);
  });

  return (
    <aside className="flex w-[200px] shrink-0 select-none flex-col border-r bg-card/30">
      <nav className="flex flex-1 flex-col gap-0.5 p-3">
        {cleaned.map((entry, idx) => {
          if (isDivider(entry)) {
            return <div key={`d-${idx}`} className="my-1.5 border-t" />;
          }
          const item = entry;
          const disabled = item.needsActiveWorkspace === true && !hasWs;
          const Icon = item.icon;
          if (disabled) {
            return (
              <span
                key={item.to}
                className={cn(
                  'flex items-center gap-2 rounded-md px-3 py-2 text-sm',
                  'cursor-not-allowed text-muted-foreground/60',
                )}
                title="请先在顶栏选择工作区"
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </span>
            );
          }
          const showUpdateBadge = item.to === '/about' && hasUpdate;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors',
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent hover:text-accent-foreground',
                )
              }
              title={showUpdateBadge ? '有可用更新' : undefined}
            >
              <Icon className="h-4 w-4" />
              <span className="flex-1">{item.label}</span>
              {showUpdateBadge && (
                <span
                  aria-label="有可用更新"
                  className="h-2 w-2 shrink-0 rounded-full bg-red-500"
                />
              )}
            </NavLink>
          );
        })}
      </nav>
    </aside>
  );
}
