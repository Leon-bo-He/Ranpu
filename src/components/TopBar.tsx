import { Home, Lock, LogOut } from 'lucide-react';
import { Link, useLocation } from 'react-router-dom';

import { identityApi } from '@/api/identity';
import { Button } from '@/components/ui/button';
import { RanpuLogo } from '@/components/RanpuLogo';
import { WorkspaceSwitcher } from '@/components/WorkspaceSwitcher';
import { useSessionStore } from '@/store/session';

export function TopBar() {
  const session = useSessionStore((s) => s.session);
  const clearSession = useSessionStore((s) => s.clear);
  const setLocked = useSessionStore((s) => s.setLocked);
  const location = useLocation();
  const atHome = location.pathname === '/';

  if (!session) return null;

  const onLock = async () => {
    await identityApi.lockSession();
    setLocked(true);
  };

  const onLogout = async () => {
    await identityApi.logout();
    clearSession();
  };

  return (
    <header className="flex h-14 items-center justify-between border-b bg-background px-6">
      <div className="flex items-center gap-3">
        <Link
          to="/"
          className="flex items-center gap-3 rounded-md px-1 py-1 transition-colors hover:bg-accent/50"
          aria-label="返回主面板"
          title="返回主面板"
        >
          <RanpuLogo size={28} withText />
        </Link>
        {!atHome && (
          <Button asChild variant="ghost" size="sm" title="返回主面板">
            <Link to="/">
              <Home className="mr-1 h-4 w-4" />
              主面板
            </Link>
          </Button>
        )}
      </div>
      <div className="flex items-center gap-4">
        <WorkspaceSwitcher />
      </div>
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">
          {session.username}
          <span className="ml-2 text-xs">
            ({session.role === 'admin' ? '管理员' : '普通用户'})
          </span>
        </span>
        <Button variant="ghost" size="sm" onClick={onLock} title="锁定">
          <Lock className="mr-1 h-4 w-4" />
          锁定
        </Button>
        <Button variant="ghost" size="sm" onClick={onLogout} title="登出">
          <LogOut className="mr-1 h-4 w-4" />
          登出
        </Button>
      </div>
    </header>
  );
}
