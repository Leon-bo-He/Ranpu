import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';
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

  // 自动锁屏从后台 setInterval 触发, 锁屏 div 渲染时 webview 很可能因为长
  // 时间没活动失去焦点 (用户切到别的窗口 / 鼠标静止). 常见症状:
  //   1. 主窗口还在背景, 没被抢到前面 — 用户看不见锁屏.
  //   2. 鼠标在 overlay 上能拖出文字选区 — WebView2 hit-test 还停在旧元素.
  //   3. autoFocus / input.focus() 调了但 keystrokes 不进密码框 — 焦点
  //      残留在锁屏前的 active 元素 (例如 Calculator 的某 input).
  // 单独 setFocus() 兜不住, 这里多管齐下:
  //   - 异步唤醒序列: unminimize → setAlwaysOnTop(true / false) 触发
  //     Win32 SetForegroundWindow → setFocus → requestUserAttention 任
  //     务栏 flash. 任一步 deny / 失败都继续走下一步.
  //   - 重试聚焦: WebView2 在窗口刚回到前台时 hit-test 延迟一帧, 用 poll
  //     比对 document.activeElement === input, 没拿到再来 (最多 ~1s).
  //   - document 级捕获: selectstart 一律 preventDefault 杜绝划选; mouse
  //     move / keydown / focusin 都把焦点强行送回密码输入框, 即便锁屏前
  //     的某个 input 抢着接 keystrokes 也会被夺回来.
  useEffect(() => {
    let cancelled = false;
    let attempts = 0;

    const focusInput = () => {
      if (cancelled) return;
      window.getSelection()?.removeAllRanges();
      const input = inputRef.current;
      if (!input) return;
      input.focus();
      if (document.activeElement !== input && attempts < 10) {
        attempts += 1;
        setTimeout(focusInput, 100);
      }
    };

    (async () => {
      const w = getCurrentWindow();
      try {
        await w.unminimize();
      } catch {
        /* 非 Tauri / 权限 deny */
      }
      try {
        await w.setAlwaysOnTop(true);
        await w.setAlwaysOnTop(false);
      } catch {
        /* deny — 继续 */
      }
      try {
        await w.setFocus();
      } catch {
        /* deny — 继续 */
      }
      try {
        await w.requestUserAttention(UserAttentionType.Critical);
      } catch {
        /* deny — 继续 */
      }
      if (cancelled) return;
      requestAnimationFrame(focusInput);
    })();

    const onSelectStart = (e: Event) => {
      e.preventDefault();
    };
    const onAnyInteraction = () => focusInput();
    document.addEventListener('selectstart', onSelectStart, true);
    document.addEventListener('mousemove', onAnyInteraction, true);
    document.addEventListener('keydown', onAnyInteraction, true);
    document.addEventListener('focusin', onAnyInteraction, true);

    return () => {
      cancelled = true;
      document.removeEventListener('selectstart', onSelectStart, true);
      document.removeEventListener('mousemove', onAnyInteraction, true);
      document.removeEventListener('keydown', onAnyInteraction, true);
      document.removeEventListener('focusin', onAnyInteraction, true);
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
      onMouseDown={(e) => {
        // 锁屏期间 overlay 上不应该出现新的选区 / 拖拽; 阻止默认行为顺便
        // 把焦点拽回密码框.
        e.preventDefault();
        inputRef.current?.focus();
      }}
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
