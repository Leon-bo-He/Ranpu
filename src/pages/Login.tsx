import { useState, type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';

import { identityApi } from '@/api/identity';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RanpuLogo } from '@/components/RanpuLogo';
import { useSessionStore } from '@/store/session';

export function LoginPage() {
  const setSession = useSessionStore((s) => s.setSession);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const session = await identityApi.login(username, password);
      setSession(session);
    } catch (err) {
      // 后端已经把 InvalidCredentials / InvalidCredentialsWithRemaining /
      // AccountJustLocked / AccountLocked 都映射成中文 message，直接显示。
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-3 bg-background">
      <RanpuLogo size={120} animated />
      <h1 className="font-serif text-3xl tracking-[3px]">染谱</h1>
      <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
        DYE FORMULA
      </p>

      <form
        onSubmit={onSubmit}
        className="mt-10 flex w-80 flex-col gap-3 rounded-lg border bg-card p-6 shadow-sm"
      >
        <div className="grid gap-1">
          <Label htmlFor="username">用户名</Label>
          <Input
            id="username"
            autoFocus
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={busy}
            autoComplete="username"
          />
        </div>
        <div className="grid gap-1">
          <Label htmlFor="password">密码</Label>
          <Input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={busy}
            autoComplete="current-password"
          />
        </div>
        {error && <p className="text-sm text-destructive">{error}</p>}
        <Button
          type="submit"
          disabled={busy || username.length === 0 || password.length === 0}
        >
          {busy ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              正在登录…
            </>
          ) : (
            '登录'
          )}
        </Button>
      </form>
    </div>
  );
}

export default LoginPage;
