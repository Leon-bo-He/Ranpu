import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface VatSlot {
  vat: number;
  batch: number;
}

interface VatSequenceState {
  /// 上一次发出的缸号. 0 表示从未发过 (或新一天首次).
  lastVat: number;
  /// 上一次发出的批次. 0 = 从未发过.
  lastBatch: number;
  /// 上次发缸号的日期 (本地时区 YYYY-MM-DD). '' = 从未发过.
  lastDate: string;
  /// 预览接下来的 count 个连续槽位 (不修改 state). 跨日自动从 1-1 重置,
  /// 缸号到 vatCount 自动进入下一批 (例: 4 缸厂 4-2 之后是 1-3).
  peek: (count: number, vatCount: number) => VatSlot[];
  /// 把 slot 设为最新已发位置 (打印时调用, 推进全局计数器). 仅当 slot
  /// 严格大于当前状态 (或跨日) 时才更新, 避免手填小号导致计数器回退.
  commit: (slot: VatSlot) => void;
}

export const useVatSequenceStore = create<VatSequenceState>()(
  persist(
    (set, get) => ({
      lastVat: 0,
      lastBatch: 0,
      lastDate: '',
      peek: (count, vatCount) => {
        if (count <= 0) return [];
        const safeVatCount = Math.max(1, Math.floor(vatCount));
        const today = todayLocal();
        const { lastVat, lastBatch, lastDate } = get();
        const sameDay = lastDate === today && lastBatch > 0;
        let vat = sameDay ? lastVat : 0;
        let batch = sameDay ? lastBatch : 1;
        const out: VatSlot[] = [];
        for (let i = 0; i < count; i++) {
          vat += 1;
          if (vat > safeVatCount) {
            vat = 1;
            batch += 1;
          }
          out.push({ vat, batch });
        }
        return out;
      },
      commit: (slot) => {
        const { lastVat, lastBatch, lastDate } = get();
        const today = todayLocal();
        const newer =
          lastDate !== today ||
          slot.batch > lastBatch ||
          (slot.batch === lastBatch && slot.vat > lastVat);
        if (newer) {
          set({ lastVat: slot.vat, lastBatch: slot.batch, lastDate: today });
        }
      },
    }),
    { name: 'ranpu-vat-sequence' },
  ),
);

/// 本地时区当天的 YYYY-MM-DD (而非 UTC), 染厂以本地工作日切换批次.
function todayLocal(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/// 解析 "X-Y" 格式的缸号字符串. X 和 Y 都必须是正整数, 否则返回 null.
export function parseVatSlot(s: string): VatSlot | null {
  const m = s.trim().match(/^(\d+)-(\d+)$/);
  if (!m) return null;
  const vat = Number(m[1]);
  const batch = Number(m[2]);
  if (!Number.isFinite(vat) || !Number.isFinite(batch)) return null;
  if (vat <= 0 || batch <= 0) return null;
  return { vat, batch };
}

/// 一组缸号里 (batch, vat) 字典序最大的那个. 用于打印时确定该把全局计
/// 数器推进到哪. 不能解析 / 空数组返回 null.
export function maxVatSlot(slots: VatSlot[]): VatSlot | null {
  let best: VatSlot | null = null;
  for (const s of slots) {
    if (
      !best ||
      s.batch > best.batch ||
      (s.batch === best.batch && s.vat > best.vat)
    ) {
      best = s;
    }
  }
  return best;
}
