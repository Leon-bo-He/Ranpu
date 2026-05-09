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
    <header className="flex h-14 shrink-0 select-none items-center justify-between border-b bg-background px-6">
      <div className="flex items-center gap-4">
        <Link
          to="/"
          className="flex items-center gap-3 rounded-md px-1 py-1 transition-colors hover:bg-accent/50"
          aria-label="返回主面板"
          title="返回主面板"
        >
          <RanpuLogo size={28} withText />
        </Link>
        <WorkspacePicker />
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={onLock} title="锁定">
          <Lock className="mr-1 h-4 w-4" />
          锁定
        </Button>
      </div>
    </header>
  );
}
