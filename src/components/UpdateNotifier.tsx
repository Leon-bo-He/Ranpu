import { Download, X } from 'lucide-react';
import { relaunch } from '@tauri-apps/plugin-process';
import { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { useUpdateStore } from '@/store/update';

/// 启动时静默查一次更新; 有新版本就右下角弹一个 toast, 点 "立即更新"
/// 走 downloadAndInstall + relaunch.
///
/// 检查失败 (无网 / endpoint 不通) 一律静默, 不打扰. 用户想主动看错误
/// 详情可去 "关于" 页手动点 "检查更新".
///
/// toast 不自动消失 — 自动隐藏会让用户错过, 用户必须显式 dismiss.
/// 用户点 "稍后" / X 后, 本会话 toast 不再出现, 但 About 页按钮仍会
/// 显示 "有新版本 + 红点" (那是用户主动查询入口).
export function UpdateNotifier() {
  const pending = useUpdateStore((s) => s.pending);
  const hasChecked = useUpdateStore((s) => s.hasChecked);
  const checking = useUpdateStore((s) => s.checking);
  const toastDismissed = useUpdateStore((s) => s.toastDismissed);
  const dismissToast = useUpdateStore((s) => s.dismissToast);
  const runCheck = useUpdateStore((s) => s.runCheck);

  const [downloading, setDownloading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!hasChecked && !checking) {
      runCheck();
    }
  }, [hasChecked, checking, runCheck]);

  if (!pending || toastDismissed) return null;

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
          onClick={dismissToast}
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
          onClick={dismissToast}
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
