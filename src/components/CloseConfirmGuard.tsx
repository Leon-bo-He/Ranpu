import * as DialogPrimitive from '@radix-ui/react-dialog';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';

/// 拦截主窗口关闭按钮 (X), 弹一个确认对话框. 用户确认后才真正销毁窗口.
///
/// 实现细节:
/// - 用 destroy() 而不是 close(): destroy 直接销毁窗口, 不再触发
///   onCloseRequested, 避免和我们自己的拦截器死循环. core:window:default
///   capability 没带 allow-destroy / allow-close, 所以 capabilities/default.json
///   显式加了 core:window:allow-destroy.
/// - 自渲染 Radix DialogPrimitive (而不是项目里的 shadcn Dialog), 给 overlay +
///   content 都硬编 z-[1100], 保证盖在 LockOverlay (z-1000) 之上 — 锁定状态
///   下点 X 也能看到 / 操作确认窗口.
export function CloseConfirmGuard() {
  const [open, setOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const win = getCurrentWindow();
    win
      .onCloseRequested((event) => {
        event.preventDefault();
        setOpen(true);
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        // 注册失败 (例如非 Tauri 环境) 时, 用户点 X 走默认关闭, 影响不大.
      });
    return () => {
      unlisten?.();
    };
  }, []);

  const onConfirm = async () => {
    setError(null);
    try {
      await getCurrentWindow().destroy();
    } catch (e) {
      // 直接把错误显示在对话框里, 否则用户只看到 "点了没反应" 完全无诊断.
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <DialogPrimitive.Root open={open} onOpenChange={(o) => !o && setOpen(false)}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-[1100] bg-black/60 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
        <DialogPrimitive.Content className="fixed left-[50%] top-[50%] z-[1100] grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg sm:rounded-lg">
          <div className="flex flex-col space-y-1.5">
            <DialogPrimitive.Title className="text-lg font-semibold leading-none tracking-tight">
              确认关闭染谱?
            </DialogPrimitive.Title>
            <DialogPrimitive.Description className="text-sm text-muted-foreground">
              关闭后会退出当前会话, 下次打开需要重新输入启动口令解锁数据库.
            </DialogPrimitive.Description>
          </div>
          {error && (
            <p className="text-sm text-destructive">关闭失败: {error}</p>
          )}
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setOpen(false)}>
              取消
            </Button>
            <Button onClick={onConfirm}>关闭</Button>
          </div>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
