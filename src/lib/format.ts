/**
 * 染谱通用日期/数字格式化。
 */

/**
 * RFC3339 字符串 → "YYYY-MM-DD HH:mm" 本地时间。
 */
export function formatDateTime(input: string | Date | null | undefined): string {
  if (!input) return '';
  const d = typeof input === 'string' ? new Date(input) : input;
  if (Number.isNaN(d.getTime())) return '';
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const dd = String(d.getDate()).padStart(2, '0');
  const hh = String(d.getHours()).padStart(2, '0');
  const mi = String(d.getMinutes()).padStart(2, '0');
  return `${yyyy}-${mm}-${dd} ${hh}:${mi}`;
}

/**
 * 数字最多保留 N 位小数, 末尾零自动去掉. 默认 4 位 — 染料用量 / 计算结果
 * 经常出现 0.001 / 0.0001 这种, 只显示 2 位会被截成 0.00 看不出区别.
 *
 *   2          → "2"
 *   2.5        → "2.5"
 *   2.0        → "2"
 *   0.001      → "0.001"
 *   0.0001     → "0.0001"
 *   0.00001    → "0"   (低于精度被四舍五入掉)
 *   123.456789 → "123.4568"
 */
export function formatAmount(
  n: number | null | undefined,
  maxDecimals = 4,
): string {
  if (n === null || n === undefined || !Number.isFinite(n)) return '';
  // toFixed → parseFloat → toString 这一手把 "12.3400" 变 "12.34", "12.0000"
  // 变 "12", 比手写 regex 干净.
  return parseFloat(n.toFixed(maxDecimals)).toString();
}

export function formatGrams(n: number | null | undefined): string {
  return n === null || n === undefined ? '' : `${formatAmount(n)} g`;
}

export function formatKg(n: number | null | undefined): string {
  return n === null || n === undefined ? '' : `${formatAmount(n)} kg`;
}

export function unitLabel(unit: 'pct_owf' | 'g_per_kg'): string {
  switch (unit) {
    case 'pct_owf':
      return '%（owf）';
    case 'g_per_kg':
      return 'g/kg';
  }
}
