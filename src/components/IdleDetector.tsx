import { useEffect, useRef } from 'react';

import { bootApi } from '@/api/boot';
import { useSessionStore } from '@/store/session';
import { useSettingsStore } from '@/store/settings';

/**
 * 监听全局鼠标 / 键盘活动；超时未活动 → 触发后端 lock_session + 前端 setLocked.
 *
 * 当 idleTimeoutMinutes 为 0 时关闭自动锁屏（PROMPT 第 83 行设置项）。
 */
export function IdleDetector() {
  const session = useSessionStore((s) => s.session);
  const setLocked = useSessionStore((s) => s.setLocked);
  const idleMinutes = useSettingsStore((s) => s.idleTimeoutMinutes);
  const lastActivity = useRef<number>(Date.now());

  useEffect(() => {
    const reset = () => {
      lastActivity.current = Date.now();
    };
    const events: (keyof WindowEventMap)[] = [
      'mousemove',
      'mousedown',
      'keydown',
      'wheel',
      'touchstart',
    ];
    events.forEach((e) => window.addEventListener(e, reset));
    return () => events.forEach((e) => window.removeEventListener(e, reset));
  }, []);

  useEffect(() => {
    if (!session || session.locked || idleMinutes === 0) return;
    const intervalMs = 30 * 1000; // 30 秒检查一次
    const idleMs = idleMinutes * 60 * 1000;
    const timer = window.setInterval(() => {
      if (Date.now() - lastActivity.current >= idleMs) {
        bootApi
          .lockSession()
          .then(() => setLocked(true))
          .catch(() => {
            /* 已经锁定 / 已登出，忽略 */
          });
      }
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [session, idleMinutes, setLocked]);

  return null;
}
