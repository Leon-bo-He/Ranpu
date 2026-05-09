import { useState, type FormEvent } from 'react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { RanpuLogo } from '@/components/RanpuLogo';
import { useSessionStore } from '@/store/session';

export function LockOverlay() {
  const setSession = useSessionStore((s) => s.setSession);
  const [passphrase, setPassphrase] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const session = await bootApi.unlockSession(passphrase);
      setSession(session);
      setPassphrase('');
    } catch (err) {
      if (err instanceof ApiError) {
        // 后端把口令错误统一映射成中文 message; 兜底显示通用提示.
        setError(err.message || '解锁密码不对');
      } else {
        setError('解锁密码不对');
      }
    } finally {
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
        <p className="mt-4 text-sm text-muted-foreground">
          会话已锁定，请输入启动口令继续
        </p>
        <form onSubmit={onSubmit} className="mt-2 flex w-72 flex-col gap-2">
          <Input
            type="password"
            autoFocus
            placeholder="启动口令"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
          />
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={busy || passphrase.length === 0}>
            {busy ? '正在解锁…' : '解锁'}
          </Button>
        </form>
      </div>
    </div>
  );
}
