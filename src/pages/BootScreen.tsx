import { useState, type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RanpuLogo } from '@/components/RanpuLogo';

interface BootScreenProps {
  onBooted: (userCount: number) => void;
}

export function BootScreen({ onBooted }: BootScreenProps) {
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const status = await bootApi.bootApp(passphrase);
      onBooted(status.user_count);
    } catch (err) {
      if (err instanceof ApiError && err.code === 'boot_passphrase_incorrect') {
        setError('启动口令不正确');
      } else {
        setError(err instanceof ApiError ? err.message : String(err));
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-3 bg-background">
      <RanpuLogo size={100} animated />
      <h1 className="font-serif text-2xl tracking-[3px]">染谱</h1>
      <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
        DYE FORMULA
      </p>

      <form
        onSubmit={onSubmit}
        className="mt-8 flex w-80 flex-col gap-3 rounded-lg border bg-card p-6 shadow-sm"
      >
        <p className="text-sm text-muted-foreground">请输入启动口令解锁本机数据。</p>
        <div className="grid gap-1">
          <Label htmlFor="boot-pw">启动口令</Label>
          <Input
            id="boot-pw"
            type="password"
            autoFocus
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
          />
        </div>
        {error && <p className="text-sm text-destructive">{error}</p>}
        <Button type="submit" disabled={busy || passphrase.length === 0}>
          {busy ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              正在启动…
            </>
          ) : (
            '启动'
          )}
        </Button>
      </form>
    </div>
  );
}

export default BootScreen;
