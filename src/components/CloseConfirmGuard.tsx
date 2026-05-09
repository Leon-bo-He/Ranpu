import { getCurrentWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';

/// 拦截主窗口关闭按钮 (X), 弹一个确认对话框. 用户确认后才真正销毁窗口,
/// 取消则什么都不做. Tauri 2 的 onCloseRequested 给的 event 上调
/// preventDefault() 就阻断默认关闭. confirm 之后用 window.destroy()
/// 直接销毁 (不再走 close 流, 否则又会触发 onCloseRequested 再次弹框).
export function CloseConfirmGuard() {
  const [open, setOpen] = useState(false);

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
    await getCurrentWindow().destroy();
  };

  return (
    <ConfirmDialog
      open={open}
      onClose={() => setOpen(false)}
      title="确认关闭染谱?"
      description="关闭后会退出当前会话, 下次打开需要重新输入启动口令解锁数据库."
      confirmLabel="关闭"
      cancelLabel="取消"
      onConfirm={onConfirm}
    />
  );
}
