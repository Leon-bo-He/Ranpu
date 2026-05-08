import { Download, X } from 'lucide-react';
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';

/// 启动时静默查一次更新; 有新版本就右下角弹一个 toast, 点 "立即更新"
/// 走 downloadAndInstall + relaunch.
///
/// 检查失败 (无网 / endpoint 不通) 一律静默, 不打扰. 用户想主动看错误
/// 详情可去 "关于" 页手动点 "检查更新".
///
/// 一次会话只检查一次 — 用户 dismiss "稍后" 后, 本次会话不再骚扰.
/// 下次启动 / 重新登录会再 mount, 那时会再查一次.
export function UpdateNotifier() {
  const [pending, setPending] = useState<Update | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [dismissed, setDismissed] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    check()
      .then((u) => {
        if (!cancelled && u) setPending(u);
      })
      .catch(() => {
        // 静默: 启动时联网失败不该弹错误对话框打断用户.
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (!pending || dismissed) return null;

  const onUpdate = async () => {
    setDownloading(true);
    setError(null);
    try {
      await pending.downloadAndInstall();
      await relaunch();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setDownloading(false);
    }
  };

  return (
    <div
      role="status"
      aria-live="polite"
      className="fixed bottom-6 right-6 z-50 w-[340px] rounded-md border bg-background p-4 shadow-lg"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <p className="text-sm font-medium">发现新版本 {pending.version}</p>
          {pending.body && (
            <p className="line-clamp-3 break-words text-xs text-muted-foreground">
              {pending.body}
            </p>
          )}
        </div>
        <button
          type="button"
          onClick={() => setDismissed(true)}
          className="shrink-0 text-muted-foreground hover:text-foreground disabled:opacity-50"
          aria-label="稍后再说"
          disabled={downloading}
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      {error && <p className="mt-2 text-xs text-destructive">{error}</p>}
      <div className="mt-3 flex justify-end gap-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setDismissed(true)}
          disabled={downloading}
        >
          稍后
        </Button>
        <Button size="sm" onClick={onUpdate} disabled={downloading}>
          <Download className="mr-1 h-4 w-4" />
          {downloading ? '下载中…' : '立即更新'}
        </Button>
      </div>
    </div>
  );
}
