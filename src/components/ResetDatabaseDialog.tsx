import { useEffect, useRef, useState, type FormEvent } from 'react';

import { adminApi } from '@/api/admin';
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

const REQUIRED_CONFIRM_PHRASE = '重置数据库';

interface ResetDatabaseDialogProps {
  open: boolean;
  onClose: () => void;
}

/// 重置数据库的双重确认 dialog: 启动口令 + 明文 "重置数据库".
/// 提交成功后短暂内 app 会被重启 (后端 restart()); 前端不需要做导航.
export function ResetDatabaseDialog({ open, onClose }: ResetDatabaseDialogProps) {
  const [passphrase, setPassphrase] = useState('');
  const [confirmText, setConfirmText] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const passInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setPassphrase('');
      setConfirmText('');
      setError(null);
      const t = setTimeout(() => passInputRef.current?.focus(), 0);
      return () => clearTimeout(t);
    }
  }, [open]);

  const canSubmit =
    passphrase.length > 0 &&
    confirmText.trim() === REQUIRED_CONFIRM_PHRASE &&
    !busy;

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    setBusy(true);
    setError(null);
    try {
      await adminApi.resetDatabase(passphrase, confirmText);
      // 后端会异步 restart, 这里不做导航; 进程会被替换.
    } catch (err) {
      setError(err instanceof ApiError ? err.message : '重置失败');
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && !busy && onClose()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>重置数据库</DialogTitle>
          <DialogDescription>
            将清空整个数据目录（默认配方、工作区、批次清单、审计日志和启动口令），且不可恢复。
            重启后回到首次设置界面，需要重新设定启动口令。
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit} className="grid gap-3">
          <div className="grid gap-1">
            <Label htmlFor="reset-passphrase">启动口令</Label>
            <Input
              id="reset-passphrase"
              ref={passInputRef}
              type="password"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              disabled={busy}
              autoComplete="off"
            />
          </div>
          <div className="grid gap-1">
            <Label htmlFor="reset-confirm">
              明文确认 — 请输入「{REQUIRED_CONFIRM_PHRASE}」
            </Label>
            <Input
              id="reset-confirm"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              disabled={busy}
              placeholder={REQUIRED_CONFIRM_PHRASE}
              autoComplete="off"
            />
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <DialogFooter className="gap-2 pt-2">
            <Button type="button" variant="ghost" onClick={onClose} disabled={busy}>
              取消
            </Button>
            <Button type="submit" variant="destructive" disabled={!canSubmit}>
              {busy ? '重置中…' : '确认重置'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
