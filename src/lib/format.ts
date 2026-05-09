/**
 * 染谱通用日期/数字格式化（PROMPT 第 299-300 行）。
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
 * 数字保留 2 位小数（用于 kg、g 显示）。
 */
export function formatTwoDecimal(n: number | null | undefined): string {
  if (n === null || n === undefined || Number.isNaN(n)) return '';
  return n.toFixed(2);
}

export function formatGrams(n: number | null | undefined): string {
  return n === null || n === undefined ? '' : `${formatTwoDecimal(n)} g`;
}

export function formatKg(n: number | null | undefined): string {
  return n === null || n === undefined ? '' : `${formatTwoDecimal(n)} kg`;
}

export function unitLabel(unit: 'pct_owf' | 'g_per_kg'): string {
  switch (unit) {
    case 'pct_owf':
      return '% (owf)';
    case 'g_per_kg':
      return 'g/kg';
  }
}
