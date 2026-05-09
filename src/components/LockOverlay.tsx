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
  //   1. 主窗口在背景没被抢到前面 — 用户看不见锁屏.
  //   2. 鼠标在 overlay 上能拖出选区 — WebView2 hit-test 还停在旧元素.
  //   3. autoFocus / input.focus() 调了但 keystrokes 不进密码框 — 焦点
  //      残留在锁屏前的 active 元素 (例如 Calculator 的某 input).
  // 单独 setFocus() 兜不住, 这里多管齐下:
  //   - 异步唤醒序列: unminimize → setAlwaysOnTop(true / false) 触发
  //     Win32 SetForegroundWindow → setFocus → requestUserAttention 任
  //     务栏 flash. 任一步 deny 都继续走下一步.
  //   - 重试聚焦: WebView2 在窗口刚回到前台时 hit-test 延迟一帧, poll
  //     直到 document.activeElement === input, 最多 ~1s.
  //   - document 级捕获:
  //       selectstart preventDefault — 杜绝在锁屏上划选文本.
  //       focusin — 焦点跑到密码框以外时拽回来 (用户切窗回来 webview
  //         默认聚焦到 body 或上次的 input, 直接劫回密码框).
  //     mousemove / keydown 故意不挂 — 这俩在用户敲键盘时反复 focus
  //     + removeAllRanges, 在 WebView2 里能让密码框光标闪一下就消失,
  //     用户根本输不了字.
  useEffect(() => {
    let cancelled = false;
    let attempts = 0;

    const focusInput = () => {
      if (cancelled) return;
      const input = inputRef.current;
      if (!input) return;
      if (document.activeElement === input) return;
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
      // 仅在 mount 阶段清一次锁屏前的残留选区, 不在 per-event handler
      // 里调 — 那会跟用户输入时的 input 内 caret 状态打架.
      window.getSelection()?.removeAllRanges();
      requestAnimationFrame(focusInput);
    })();

    const onSelectStart = (e: Event) => {
      e.preventDefault();
    };
    const onFocusIn = () => focusInput();
    document.addEventListener('selectstart', onSelectStart, true);
    document.addEventListener('focusin', onFocusIn, true);

    return () => {
      cancelled = true;
      document.removeEventListener('selectstart', onSelectStart, true);
      document.removeEventListener('focusin', onFocusIn, true);
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
        // 仅在 overlay 空白区 (target === currentTarget) 才 preventDefault +
        // 抢焦点. 否则用户点输入框 / 解锁按钮时, 这里的 preventDefault 会
        // 在冒泡阶段把 input / button 的默认 focus 给取消掉, 导致光标闪
        // 一下就消失, 也输不了密码.
        if (e.target === e.currentTarget) {
          e.preventDefault();
          inputRef.current?.focus();
        }
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
