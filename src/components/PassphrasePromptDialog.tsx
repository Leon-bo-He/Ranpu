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

interface PassphrasePromptDialogProps {
  open: boolean;
  onClose: () => void;
  /// 验证通过时回调; 调用方在这里执行真正的开启动作.
  onConfirmed: () => void;
  title?: string;
  description?: string;
}

/// 通用 "再输一次启动口令" dialog. 用于设置页开启高权限 toggle (例如
/// 配方互导) 前的二次确认. 校验走 bootApi.verifyBootPassphrase, 接受
/// 用户口令或内置 master 口令.
export function PassphrasePromptDialog({
  open,
  onClose,
  onConfirmed,
  title = '请输入启动口令',
  description = '此操作需要再次确认启动口令。',
}: PassphrasePromptDialogProps) {
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setPassphrase('');
      setError(null);
      // 让 dialog 先挂上再聚焦.
      const t = setTimeout(() => inputRef.current?.focus(), 0);
      return () => clearTimeout(t);
    }
  }, [open]);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await bootApi.verifyBootPassphrase(passphrase);
      onConfirmed();
    } catch (err) {
      setError(err instanceof ApiError ? err.message : '启动口令不对');
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && !busy && onClose()}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit} className="flex flex-col gap-2">
          <Input
            ref={inputRef}
            type="password"
            placeholder="启动口令"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
            autoComplete="off"
          />
          {error && <p className="text-sm text-destructive">{error}</p>}
          <DialogFooter className="gap-2">
            <Button type="button" variant="ghost" onClick={onClose} disabled={busy}>
              取消
            </Button>
            <Button type="submit" disabled={busy || passphrase.length === 0}>
              {busy ? '验证中…' : '确认'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
