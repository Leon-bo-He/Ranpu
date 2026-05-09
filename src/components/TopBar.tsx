import { Lock } from 'lucide-react';
import { Link } from 'react-router-dom';

import { bootApi } from '@/api/boot';
import { Button } from '@/components/ui/button';
import { RanpuLogo } from '@/components/RanpuLogo';
import { WorkspacePicker } from '@/components/WorkspacePicker';
import { useSessionStore } from '@/store/session';

export function TopBar() {
  const session = useSessionStore((s) => s.session);
  const setLocked = useSessionStore((s) => s.setLocked);

  if (!session) return null;

  const onLock = async () => {
    await bootApi.lockSession();
    setLocked(true);
  };

  return (
    <header className="flex h-14 shrink-0 select-none items-center border-b bg-background">
      {/* Logo 区占 200px = 跟下方 Sidebar 同宽, 视觉上 sidebar 右边框
          直接和 logo 区右边对齐, 主内容区域 (含 WorkspacePicker) 起点
          就是 sidebar 边界. */}
      <Link
        to="/"
        className="flex h-full w-[200px] shrink-0 items-center gap-3 px-6 transition-colors hover:bg-accent/50"
        aria-label="返回主面板"
        title="返回主面板"
      >
        <RanpuLogo size={28} withText />
      </Link>
      <div className="flex flex-1 items-center justify-between px-6">
        <WorkspacePicker />
        <Button variant="ghost" size="sm" onClick={onLock} title="锁定">
          <Lock className="mr-1 h-4 w-4" />
          锁定
        </Button>
      </div>
    </header>
  );
}
