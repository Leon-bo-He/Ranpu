import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useRef, useState, type FormEvent } from 'react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { RanpuLogo } from '@/components/RanpuLogo';
import { useSessionStore } from '@/store/session';

export function LockOverlay() {
  const setSession = useSessionStore((s) => s.setSession);
  const [passphrase, setPassphrase] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 自动锁屏从后台 setInterval 触发, 这一刻 webview 很可能因为长时间
  // 没活动失去了焦点 (用户离开座位 / 切到其它窗口). 锁屏 div 渲染出来
  // 但窗口没焦点 → 鼠标事件被穿透到下面元素 (能选中文本 / 划选), 输入
  // 框 autoFocus 也拿不到焦点. 用户得鼠标移出再进来才让 webview 重获
  // 焦点, 一切才正常. 这里 mount 时主动调 setFocus() 强制窗口拿焦点,
  // 然后再把焦点扔到密码输入框上.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await getCurrentWindow().setFocus();
      } catch {
        // 非 Tauri 环境或权限 deny — 影响不大, 继续走 input.focus.
      }
      if (cancelled) return;
      // 顺便清掉锁屏前可能遗留的文本选区, 避免视觉杂乱.
      window.getSelection()?.removeAllRanges();
      inputRef.current?.focus();
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const session = await bootApi.unlockSession(passphrase);
      setSession(session);
      setPassphrase('');
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message || '解锁密码不对');
      } else {
        setError('解锁密码不对');
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      // user-select: none 配合 onMouseDown 把焦点抢回密码输入框, 避免锁屏
      // 状态下用户在 overlay 任意空白处 drag 划出选区或穿透到下面.
      className="fixed inset-0 z-[1000] flex select-none flex-col items-center justify-center backdrop-blur-md"
      style={{ background: 'rgba(0, 0, 0, 0.55)' }}
      onMouseDown={() => inputRef.current?.focus()}
    >
      <div className="flex flex-col items-center gap-3 rounded-lg bg-background/95 p-10 shadow-2xl">
        <RanpuLogo size={80} />
        <p className="font-serif text-xl tracking-[3px]">染谱</p>
        <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
          DYE FORMULA
        </p>
        <p className="mt-4 text-sm text-muted-foreground">
          会话已锁定，请输入启动口令继续
        </p>
        <form onSubmit={onSubmit} className="mt-2 flex w-72 flex-col gap-2">
          <Input
            ref={inputRef}
            type="password"
            placeholder="启动口令"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
          />
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={busy || passphrase.length === 0}>
            {busy ? '正在解锁…' : '解锁'}
          </Button>
        </form>
      </div>
    </div>
  );
}
