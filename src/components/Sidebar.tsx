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
import { hasActiveWorkspace, useSessionStore } from '@/store/session';
import { useUpdateStore } from '@/store/update';

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  /** 没激活 workspace 时禁用（仍显示，灰掉 + tooltip 提示）。 */
  needsActiveWorkspace?: boolean;
}

const NAV_ITEMS: NavItem[] = [
  { to: '/', label: '主面板', icon: Home },
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
  if (!session) return null;
  const hasWs = hasActiveWorkspace(session);

  return (
    <aside className="flex w-[200px] shrink-0 select-none flex-col border-r bg-card/30">
      <nav className="flex flex-1 flex-col gap-0.5 p-3">
        {NAV_ITEMS.map((item) => {
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
