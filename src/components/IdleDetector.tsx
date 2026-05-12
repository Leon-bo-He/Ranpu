import { useEffect, useRef } from 'react';

import { bootApi } from '@/api/boot';
import { useSessionStore } from '@/store/session';
import { useSettingsStore } from '@/store/settings';

/**
 * 监听全局鼠标 / 键盘活动；超时未活动 → 触发后端 lock_session + 前端 setLocked.
 *
 * 当 idleTimeoutSeconds 为 0 时关闭自动锁屏。10 秒是测试用挡位 (复现锁屏
 * 焦点 bug); 检查间隔会按 idleMs / 3 缩短, 避免 30 秒固定间隔下 10 秒挡
 * 实际要等 ~30 秒才触发.
 */
export function IdleDetector() {
  const session = useSessionStore((s) => s.session);
  const setLocked = useSessionStore((s) => s.setLocked);
  const idleSeconds = useSettingsStore((s) => s.idleTimeoutSeconds);
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
    if (!session || session.locked || idleSeconds === 0) return;
    const idleMs = idleSeconds * 1000;
    // 短超时 (10 秒测试档) 要更密的检查, 否则触发延迟 ≈ 间隔本身. 上限
    // 30 秒避免长超时档 (1 小时) 每秒空跑.
    const intervalMs = Math.max(1000, Math.min(idleMs / 3, 30 * 1000));
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
  }, [session, idleSeconds, setLocked]);

  return null;
}
