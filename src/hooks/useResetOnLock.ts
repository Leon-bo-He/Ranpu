import { useEffect } from 'react';

import { useSessionStore } from '@/store/session';

/// 当会话被锁定时跑一次 reset 回调. 主要用来把页面里 state-driven 的
/// Radix Dialog 立刻关掉 — 不关的话 Dialog 自带的 focus-scope 会一直
/// 把焦点 trap 住, LockOverlay 的密码框 / 解锁按钮抢不到焦点, 用户
/// 看得见但点不动.
///
/// 受控 Dialog (open prop 由外面 state 决定) 直接 set state → false 不会
/// 触发 onOpenChange 回调, 干净: 不会触发关闭时的 side effects (例如 Cart
/// 预览 Dialog 关闭时 onOpenChange 里 setPromptOpen(true) 把另一个 Dialog
/// 弹起来的级联).
export function useResetOnLock(reset: () => void) {
  const locked = useSessionStore((s) => s.session?.locked ?? false);
  useEffect(() => {
    if (locked) reset();
    // reset 故意不放依赖里: 每次 lock toggle 时跑一次最新 reset 就行,
    // 不需要 stable 引用. locked 变 true 触发 → 即可.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [locked]);
}
