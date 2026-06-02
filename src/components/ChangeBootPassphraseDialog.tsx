import { useEffect, useRef, useState, type FormEvent } from 'react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface Props {
  open: boolean;
  onClose: () => void;
}

export function ChangeBootPassphraseDialog({ open, onClose }: Props) {
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [confirm, setConfirm] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [succeeded, setSucceeded] = useState(false);
  const currentRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setCurrent('');
      setNext('');
      setConfirm('');
      setError(null);
      setSucceeded(false);
      const t = setTimeout(() => currentRef.current?.focus(), 0);
      return () => clearTimeout(t);
    }
  }, [open]);

  // 成功后 1.2s 自动关闭
  useEffect(() => {
    if (!succeeded) return;
    const t = setTimeout(onClose, 1200);
    return () => clearTimeout(t);
  }, [succeeded, onClose]);

  const mismatch = next.length > 0 && confirm.length > 0 && next !== confirm;
  const canSubmit = current.length > 0 && next.length > 0 && next === confirm && !busy;

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setBusy(true);
    setError(null);
    try {
      await bootApi.changeBootPassphrase(current, next);
      setSucceeded(true);
    } catch (err) {
      if (err instanceof ApiError && err.code === 'boot_passphrase_incorrect') {
        setError('当前启动口令不正确');
      } else if (err instanceof ApiError) {
        setError(err.message);
      } else {
        setError(String(err));
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && !busy && onClose()}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>更改启动口令</DialogTitle>
          <DialogDescription>输入当前口令验证身份，再设置新口令。</DialogDescription>
        </DialogHeader>

        {succeeded ? (
          <p className="py-4 text-center text-sm text-green-600">启动口令已更新。</p>
        ) : (
          <form onSubmit={onSubmit} className="flex flex-col gap-3">
            <div className="grid gap-1">
              <Label htmlFor="cbp-current">当前启动口令</Label>
              <Input
                id="cbp-current"
                ref={currentRef}
                type="password"
                value={current}
                onChange={(e) => setCurrent(e.target.value)}
                disabled={busy}
                autoComplete="current-password"
              />
            </div>
            <div className="grid gap-1">
              <Label htmlFor="cbp-new">新启动口令</Label>
              <Input
                id="cbp-new"
                type="password"
                value={next}
                onChange={(e) => setNext(e.target.value)}
                disabled={busy}
                autoComplete="new-password"
              />
            </div>
            <div className="grid gap-1">
              <Label htmlFor="cbp-confirm">确认新口令</Label>
              <Input
                id="cbp-confirm"
                type="password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                disabled={busy}
                autoComplete="new-password"
              />
              {mismatch && (
                <p className="text-xs text-destructive">两次输入不一致</p>
              )}
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <DialogFooter className="gap-2">
              <Button type="button" variant="ghost" onClick={onClose} disabled={busy}>
                取消
              </Button>
              <Button type="submit" disabled={!canSubmit}>
                {busy ? '更新中…' : '确认更改'}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
