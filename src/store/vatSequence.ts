import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface VatSlot {
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
  /// 给 count 个配方分配连续缸号. 跨日重置为 1-1, 缸号到 vatCount 自动进
  /// 入下一批 (例: 4 缸厂 4-2 之后是 1-3). 全系统共享同一序列.
  reserve: (count: number, vatCount: number) => VatSlot[];
}

export const useVatSequenceStore = create<VatSequenceState>()(
  persist(
    (set, get) => ({
      lastVat: 0,
      lastBatch: 0,
      lastDate: '',
      reserve: (count, vatCount) => {
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
        const last = out[out.length - 1];
        set({ lastVat: last.vat, lastBatch: last.batch, lastDate: today });
        return out;
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
