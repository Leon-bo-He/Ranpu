import { useState, type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RanpuLogo } from '@/components/RanpuLogo';
import { useSessionStore } from '@/store/session';

const MIN_PASSWORD_LEN = 8;

export function FirstRunSetup() {
  const setSession = useSessionStore((s) => s.setSession);
  const [bootPassphrase, setBootPassphrase] = useState('');
  const [bootPassphrase2, setBootPassphrase2] = useState('');
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [password2, setPassword2] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    if (bootPassphrase !== bootPassphrase2) {
      setError('两次输入的启动口令不一致');
      return;
    }
    if (password !== password2) {
      setError('两次输入的密码不一致');
      return;
    }
    if (password.length < MIN_PASSWORD_LEN) {
      setError(`密码至少 ${MIN_PASSWORD_LEN} 位`);
      return;
    }
    if (bootPassphrase.length < MIN_PASSWORD_LEN) {
      setError(`启动口令至少 ${MIN_PASSWORD_LEN} 位`);
      return;
    }
    setBusy(true);
    try {
      const session = await bootApi.setupFirstRun(bootPassphrase, username, password);
      setSession(session);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-background py-8">
      <form
        onSubmit={onSubmit}
        className="flex w-[440px] flex-col gap-4 rounded-lg border bg-card p-8 shadow-sm"
      >
        <div className="flex flex-col items-center gap-1">
          <RanpuLogo size={72} />
          <h1 className="mt-2 font-serif text-2xl tracking-[3px]">染谱</h1>
          <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
            首次启动设置
          </p>
        </div>

        <p className="text-sm text-muted-foreground">
          为这台电脑设置一个「启动口令」(用于解锁本机数据), 再创建第一个管理员账号。
          启动口令与登录密码独立, 请分别牢记。
        </p>

        <div className="grid gap-1">
          <Label>启动口令</Label>
          <Input
            type="password"
            value={bootPassphrase}
            onChange={(e) => setBootPassphrase(e.target.value)}
            disabled={busy}
          />
        </div>
        <div className="grid gap-1">
          <Label>再次输入启动口令</Label>
          <Input
            type="password"
            value={bootPassphrase2}
            onChange={(e) => setBootPassphrase2(e.target.value)}
            disabled={busy}
          />
        </div>

        <div className="my-2 border-t" />

        <div className="grid gap-1">
          <Label>管理员用户名</Label>
          <Input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            disabled={busy}
            autoComplete="username"
          />
        </div>
        <div className="grid gap-1">
          <Label>管理员密码</Label>
          <Input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={busy}
            autoComplete="new-password"
          />
        </div>
        <div className="grid gap-1">
          <Label>再次输入密码</Label>
          <Input
            type="password"
            value={password2}
            onChange={(e) => setPassword2(e.target.value)}
            disabled={busy}
            autoComplete="new-password"
          />
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}
        <Button type="submit" disabled={busy}>
          {busy ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              正在创建…
            </>
          ) : (
            '完成设置并登录'
          )}
        </Button>
      </form>
    </div>
  );
}

export default FirstRunSetup;
