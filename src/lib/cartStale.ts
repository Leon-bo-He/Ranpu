import type { CartLineView } from '@/api/types';

/// 判定批次清单是否 "陈旧": 有数据且最近一条 added_at 的本地日期不是今
/// 天. 返回那个非今天的最大日期 (YYYY-MM-DD); 空清单或最近一条就是今
/// 天则返回 null. 用于 "加入配方到批次清单" 时提醒用户清掉上次留下的
/// 旧数据.
export function getCartStaleDate(lines: CartLineView[]): string | null {
  if (lines.length === 0) return null;
  let max = '';
  for (const l of lines) {
    const d = toLocalYMD(new Date(l.added_at));
    if (d > max) max = d;
  }
  const today = toLocalYMD(new Date());
  return max && max < today ? max : null;
}

function toLocalYMD(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}
