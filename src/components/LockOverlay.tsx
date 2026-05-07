import { useState, type FormEvent } from 'react';

import { identityApi } from '@/api/identity';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { RanpuLogo } from '@/components/RanpuLogo';
import { useSessionStore } from '@/store/session';

export function LockOverlay() {
  const session = useSessionStore((s) => s.session);
  const setLocked = useSessionStore((s) => s.setLocked);
  const clearSession = useSessionStore((s) => s.clear);
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result = await identityApi.unlockSession(password);
      if (result.kind === 'unlocked') {
        setLocked(false);
        setPassword('');
      } else if (result.kind === 'still_locked') {
        setError(`密码不对，剩余 ${result.remaining ?? 0} 次机会`);
      } else if (result.kind === 'force_logged_out') {
        clearSession();
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const onSwitchUser = async () => {
    setBusy(true);
    setError(null);
    try {
      await identityApi.logout();
    } catch {
      // 即使后端 logout 出错, 前端也强制清 session 让用户能回登录页.
    } finally {
      clearSession();
      setBusy(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-[1000] flex flex-col items-center justify-center backdrop-blur-md"
      style={{ background: 'rgba(0, 0, 0, 0.55)' }}
    >
      <div className="flex flex-col items-center gap-3 rounded-lg bg-background/95 p-10 shadow-2xl">
        <RanpuLogo size={80} />
        <p className="font-serif text-xl tracking-[3px]">染谱</p>
        <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
          DYE FORMULA
        </p>
        <p className="mt-4 text-sm text-muted-foreground">会话已锁定，请输入用户密码继续</p>
        {session && (
          <p className="text-sm">
            当前用户:{' '}
            <span className="font-mono font-medium">{session.username}</span>
          </p>
        )}
        <form onSubmit={onSubmit} className="mt-2 flex w-72 flex-col gap-2">
          <Input
            type="password"
            autoFocus
            placeholder="密码"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={busy}
          />
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={busy || password.length === 0}>
            {busy ? '正在解锁…' : '解锁'}
          </Button>
        </form>
        <button
          type="button"
          onClick={onSwitchUser}
          disabled={busy}
          className="mt-1 text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline disabled:opacity-50"
        >
          不是这个账号？切换用户
        </button>
      </div>
    </div>
  );
}
