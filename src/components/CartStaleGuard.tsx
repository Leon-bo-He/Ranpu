import { useState, type ReactNode } from 'react';

import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { getCartStaleDate } from '@/lib/cartStale';

interface UseCartStaleGuardOptions {
  /// 清空 / 列清单时的错误反馈, 让调用页面 (setError / setCartErr) 弹出
  /// 给用户.
  onError?: (msg: string) => void;
}

interface UseCartStaleGuardResult {
  /// 在打开 "加入批次清单" 弹窗前调一遍, 异步检查清单是否陈旧:
  /// 不陈旧 → 立刻同步 proceed; 陈旧 → 弹三选 dialog (清空 / 保留 / 取消),
  /// 用户选完再决定是否调 proceed.
  guard: (proceed: () => void) => Promise<void>;
  /// 把这个节点放在页面 JSX 末尾.
  dialog: ReactNode;
}

/// 共享给 默认配方库 / 工作区配方 等加入批次清单的入口. 主要意图: 跨日工
/// 作前提醒用户清掉昨天的批次清单, 避免把今天的新批次和昨天的混在一起.
export function useCartStaleGuard(
  options: UseCartStaleGuardOptions = {},
): UseCartStaleGuardResult {
  const [ask, setAsk] = useState<{
    staleDate: string;
    proceed: () => void;
  } | null>(null);
  const [busy, setBusy] = useState(false);

  const guard = async (proceed: () => void) => {
    try {
      const lines = await cartApi.list();
      const staleDate = getCartStaleDate(lines);
      if (staleDate) {
        setAsk({ staleDate, proceed });
      } else {
        proceed();
      }
    } catch (e) {
      options.onError?.(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onClear = async () => {
    if (!ask) return;
    const cb = ask.proceed;
    setBusy(true);
    try {
      await cartApi.clear();
      setAsk(null);
      cb();
    } catch (e) {
      options.onError?.(e instanceof ApiError ? e.message : String(e));
      setAsk(null);
    } finally {
      setBusy(false);
    }
  };

  const onKeep = () => {
    if (!ask) return;
    const cb = ask.proceed;
    setAsk(null);
    cb();
  };

  const dialog = (
    <Dialog
      open={ask !== null}
      onOpenChange={(o) => !o && !busy && setAsk(null)}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>批次清单还有上次的数据</DialogTitle>
          <DialogDescription>
            清单里最近一条记录是 {ask?.staleDate}，不是今天。继续加入前要清空吗？
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={() => setAsk(null)} disabled={busy}>
            取消
          </Button>
          <Button variant="outline" onClick={onKeep} disabled={busy}>
            保留并继续
          </Button>
          <Button variant="destructive" onClick={onClear} disabled={busy}>
            {busy ? '清空中…' : '清空并继续'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );

  return { guard, dialog };
}
