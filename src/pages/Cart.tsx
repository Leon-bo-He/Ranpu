import { Printer, Trash, Trash2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import { workspaceApi } from '@/api/workspace';
import type { CartLineView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatGrams } from '@/lib/format';
import { cn } from '@/lib/utils';
import { ComboboxInput } from '@/components/ComboboxInput';
import {
  UnknownYarnPromptDialog,
  type UnknownYarnEntry,
} from '@/components/UnknownYarnPromptDialog';
import { useResetOnLock } from '@/hooks/useResetOnLock';
import {
  emptyEntry,
  emptyMeta,
  lineKey,
  useBatchSheetInfoStore,
  type PerFormulaEntry,
  type PerFormulaMeta,
} from '@/store/batchSheetInfo';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';
import { useSettingsStore } from '@/store/settings';
import { useYarnSettingsStore } from '@/store/yarnSettings';

export function CartPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;
  const setBatchSheetInfo = useBatchSheetInfoStore((s) => s.setInfo);
  const yarnMills = useYarnSettingsStore((s) => s.mills);
  const yarnSpecs = useYarnSettingsStore((s) => s.specs);
  const setYarnMills = useYarnSettingsStore((s) => s.setMills);
  const setYarnSpecs = useYarnSettingsStore((s) => s.setSpecs);
  const singleYarnWeight = useSettingsStore((s) => s.singleYarnWeightKg);
  const [lines, setLines] = useState<CartLineView[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [workspaceName, setWorkspaceName] = useState<string>('');
  const [askClear, setAskClear] = useState(false);
  const [previewHtml, setPreviewHtml] = useState<string | null>(null);
  const [previewBusy, setPreviewBusy] = useState(false);
  const previewIframeRef = useRef<HTMLIFrameElement | null>(null);
  // 预览前的元信息收集对话框: 客户 (默认当前工作区名) +
  // 每条配方独立的缸号 / 纱支. perFormula 数组与 lines 顺序对齐.
  const [promptOpen, setPromptOpen] = useState(false);
  const [promptCustomer, setPromptCustomer] = useState('');
  const [promptPerFormula, setPromptPerFormula] = useState<PerFormulaMeta[]>([]);
  // 当次预览用到的 customer (供打印 PDF 默认文件名用), 拿 prompt 提交时的值.
  const [printCustomer, setPrintCustomer] = useState('');
  // 预览版本 toggle: a6punch (穿孔纸 120×140mm, 默认) / grid (A4 四宫格) /
  // standard (每条一段) / label (50×80mm 标签纸, 只 vat + 客户 + 纱支).
  // 用户在预览框右上角切换, 切换时重新请求对应 HTML.
  const [previewLayout, setPreviewLayout] =
    useState<'standard' | 'grid' | 'a6punch' | 'label'>('a6punch');
  // 提交 prompt 时如果发现新词 (不在 yarnMills / yarnSpecs 里), 暂存 +
  // 弹 dialog 让用户决定加不加进库.
  const [unknownYarns, setUnknownYarns] = useState<UnknownYarnEntry[]>([]);

  const load = () => {
    if (!hasWs) {
      setLines([]);
      return;
    }
    cartApi
      .list()
      .then(setLines)
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  };

  // 切换工作区时同步刷新批次清单与名称缓存。
  useEffect(() => {
    load();
    if (activeWorkspaceId !== null) {
      workspaceApi
        .list()
        .then((ws) => {
          const found = ws.find((w) => w.id === activeWorkspaceId);
          setWorkspaceName(found?.name ?? '');
        })
        .catch(() => setWorkspaceName(''));
    } else {
      setWorkspaceName('');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeWorkspaceId]);

  // 锁屏触发时直接清空本页所有 state-driven Dialog. 走 setState 不走
  // onOpenChange 是有意的: 后者会触发 Cart 预览的级联 (关预览 → 弹 prompt),
  // 让另一个 Dialog 接力 trap 焦点, LockOverlay 就一直进不来.
  useResetOnLock(() => {
    setPromptOpen(false);
    setPreviewHtml(null);
    setAskClear(false);
    setUnknownYarns([]);
  });

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区，批次清单按工作区维护。
      </p>
    );
  }

  const onChangeKg = async (line: CartLineView, kg: number) => {
    if (!Number.isFinite(kg) || kg <= 0) return;
    try {
      await cartApi.updateKg(line.source_kind, line.source_formula_id, kg);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onRemove = async (line: CartLineView) => {
    try {
      await cartApi.remove(line.source_kind, line.source_formula_id);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const confirmClear = async () => {
    try {
      await cartApi.clear();
      setAskClear(false);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setAskClear(false);
    }
  };

  // 点 "预览/打印" 先开元信息对话框 (客户默认当前工作区名 + 每条配方
  // 独立的缸号 / 纱支), 用户填完再去后端拿渲染好的 HTML.
  const onOpenPreviewPrompt = () => {
    const saved =
      activeWorkspaceId !== null
        ? useBatchSheetInfoStore.getState().byWorkspace[activeWorkspaceId]
        : undefined;
    setPromptCustomer(saved?.customer || workspaceName);
    setPromptPerFormula(
      lines.map((line) => {
        const meta = saved?.perFormula[lineKey(line.source_kind, line.source_formula_id)];
        const totalMax = totalCountFor(line.target_kg, singleYarnWeight);
        if (!meta) {
          // 全新配方: 第一条 entry 用 总重 / 单个重 兜底, 用户可改.
          return {
            vat: '',
            batch: '',
            entries: [{ yarnMill: '', yarnSpec: '', count: formatCount(totalMax) }],
          };
        }
        // 每次开 prompt 都按当前 target_kg / 单个重 重算个数 (用户改了数
        // 量后能直接看到新值). 策略:
        // - 没存过的: 第一条 entry = totalMax, 其它 = 0.
        // - 存过的: 把保存的 count 按 totalMax / savedSum 比例缩放, 保留
        //   用户原本的分配比例 (例: 旧 {5, 3} 共 8, 新总 9.6 → {6, 3.6}).
        //   旧 sum = 0 (空白) → 走第一条逻辑.
        const savedSum = meta.entries.reduce((acc, e) => {
          const n = parseFloat(e.count);
          return acc + (Number.isFinite(n) && n > 0 ? n : 0);
        }, 0);
        const refreshed = meta.entries.map((e, idx) => {
          if (savedSum > 0) {
            const n = parseFloat(e.count);
            const scaled = Number.isFinite(n) ? (n * totalMax) / savedSum : 0;
            return { ...e, count: formatCount(scaled) };
          }
          return {
            ...e,
            count: idx === 0 ? formatCount(totalMax) : formatCount(0),
          };
        });
        return {
          vat: meta.vat ?? '',
          batch: meta.batch ?? '',
          entries: refreshed.length > 0 ? refreshed : [emptyEntry()],
        };
      }),
    );
    setPromptOpen(true);
  };

  // 改配方顶层字段 (缸号 / 缸次), 整组纱支共用.
  const updatePromptMeta = (
    formulaIdx: number,
    patch: Partial<Pick<PerFormulaMeta, 'vat' | 'batch'>>,
  ) =>
    setPromptPerFormula((prev) =>
      prev.map((m, i) => (i === formulaIdx ? { ...m, ...patch } : m)),
    );

  // 改某条纱支变体字段 (厂名 / 规格).
  const updatePromptEntry = (
    formulaIdx: number,
    entryIdx: number,
    patch: Partial<PerFormulaEntry>,
  ) =>
    setPromptPerFormula((prev) =>
      prev.map((m, i) =>
        i === formulaIdx
          ? {
              ...m,
              entries: m.entries.map((e, j) => (j === entryIdx ? { ...e, ...patch } : e)),
            }
          : m,
      ),
    );

  // "+ 加一组纱支": 在该配方末尾追加一条新 entry. 缸号 / 缸次不变 (跟着
  // 配方走). count 默认 = 总重/单个重 - 已设置 entries 的 count 之和;
  // 厂名 / 规格留空让用户填.
  const addPromptEntry = (formulaIdx: number) =>
    setPromptPerFormula((prev) =>
      prev.map((m, i) => {
        if (i !== formulaIdx) return m;
        const line = lines[formulaIdx];
        const totalMax = line ? totalCountFor(line.target_kg, singleYarnWeight) : 0;
        const usedSum = m.entries.reduce((acc, e) => {
          const n = parseFloat(e.count);
          return acc + (Number.isFinite(n) ? n : 0);
        }, 0);
        const remaining = Math.max(0, totalMax - usedSum);
        return {
          ...m,
          entries: [
            ...m.entries,
            { yarnMill: '', yarnSpec: '', count: formatCount(remaining) },
          ],
        };
      }),
    );

  // 删除该配方下某条 entry. 至少保留 1 条.
  const removePromptEntry = (formulaIdx: number, entryIdx: number) =>
    setPromptPerFormula((prev) =>
      prev.map((m, i) => {
        if (i !== formulaIdx) return m;
        if (m.entries.length <= 1) return { ...m, entries: [emptyEntry()] };
        return { ...m, entries: m.entries.filter((_, j) => j !== entryIdx) };
      }),
    );

  // 把当前 prompt 输入持久化到 batchSheetInfo store, 按工作区维度存. 入口:
  // confirmPreview / 取消按钮 / 生成缸号. 切工作区或重启后重开 prompt 时
  // 这些值会回填.
  const persistPromptInfo = (
    customer: string,
    perFormula: PerFormulaMeta[],
  ) => {
    if (activeWorkspaceId === null) return;
    const map: Record<string, PerFormulaMeta> = {};
    perFormula.forEach((m, i) => {
      const line = lines[i];
      if (!line) return;
      const nonEmptyEntries = m.entries.filter(
        (e) => e.yarnMill || e.yarnSpec || e.count.trim(),
      );
      const hasAny = m.vat || m.batch || nonEmptyEntries.length > 0;
      if (hasAny) {
        map[lineKey(line.source_kind, line.source_formula_id)] = {
          vat: m.vat,
          batch: m.batch,
          // 至少保留 1 条 entry (即便全空), 重开 prompt 还能渲染那一行
          // 输入框. 全空 entry 在生成预览时会过滤掉.
          entries: nonEmptyEntries.length > 0 ? nonEmptyEntries : [emptyEntry()],
        };
      }
    });
    setBatchSheetInfo(activeWorkspaceId, { customer, perFormula: map });
  };

  const onCancelPrompt = () => {
    persistPromptInfo(promptCustomer, promptPerFormula);
    setPromptOpen(false);
  };

  // 用最新 prompt + 指定 layout 拉一份预览 HTML. 提交 prompt / 切 tab 都走这里.
  const fetchPreview = async (layout: 'standard' | 'grid' | 'a6punch' | 'label') => {
    const customer = promptCustomer.trim();
    setPreviewBusy(true);
    try {
      const html = await cartApi.previewHtml({
        customer: customer || null,
        // 一份配方一格 (四宫格) / 一段 (标准), 不论多少组纱支变体. 缸号 /
        // 缸次 拼成单个 vat_number 整组共用; 多组纱支以 yarns 数组里的多
        // 条变体 (厂名 / 规格 / 个数) 展示成内联多行.
        perFormula: promptPerFormula.map((m) => ({
          vatNumber: combineDash(m.vat, m.batch),
          yarns: m.entries.map((e) => ({
            mill: e.yarnMill || null,
            spec: e.yarnSpec || null,
            count: e.count.trim() || null,
          })),
        })),
        layout,
      });
      setPreviewHtml(html);
      setPreviewLayout(layout);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setPreviewBusy(false);
    }
  };

  const onConfirmPreview = async () => {
    // 扫一遍 prompt 里的厂名 / 规格, 找出不在库里的新词 — 用户可能临时
    // 输入了新厂名 / 新规格. 同一个新词出现多次只问一次. 比对大小写 +
    // 首尾空白都忽略.
    const millSet = new Set(yarnMills.map((s) => s.trim().toLowerCase()));
    const specSet = new Set(yarnSpecs.map((s) => s.trim().toLowerCase()));
    const seen = new Set<string>();
    const unknowns: UnknownYarnEntry[] = [];
    for (const m of promptPerFormula) {
      for (const e of m.entries) {
        const mill = e.yarnMill.trim();
        if (mill) {
          const key = `mill:${mill.toLowerCase()}`;
          if (!millSet.has(mill.toLowerCase()) && !seen.has(key)) {
            seen.add(key);
            unknowns.push({ kind: 'mill', value: mill });
          }
        }
        const spec = e.yarnSpec.trim();
        if (spec) {
          const key = `spec:${spec.toLowerCase()}`;
          if (!specSet.has(spec.toLowerCase()) && !seen.has(key)) {
            seen.add(key);
            unknowns.push({ kind: 'spec', value: spec });
          }
        }
      }
    }
    if (unknowns.length > 0) {
      setUnknownYarns(unknowns);
      return; // dialog 流程会接着调 doProceedPreview
    }
    await doProceedPreview();
  };

  /// 实际跳预览: 把客户名 + 配方信息持久化, 关 prompt, 拉 HTML.
  const doProceedPreview = async () => {
    const customer = promptCustomer.trim();
    persistPromptInfo(promptCustomer, promptPerFormula);
    setPromptOpen(false);
    setPrintCustomer(customer || workspaceName);
    // 进预览默认 a6punch (A6 穿孔纸); 用户可在右上角切到四宫格 / 标准.
    await fetchPreview('a6punch');
  };

  /// UnknownYarnPromptDialog 确认回调: 把选中的新词加进 yarnSettings store,
  /// 然后继续跳预览. 取消 (cancel) 也跳预览 — 用户已经确认过 prompt 内容,
  /// 不让 "加不加进库" 的小决策挡住主流程.
  const onUnknownYarnsResolved = (toAdd: UnknownYarnEntry[]) => {
    if (toAdd.length > 0) {
      const newMills = toAdd
        .filter((u) => u.kind === 'mill')
        .map((u) => u.value);
      const newSpecs = toAdd
        .filter((u) => u.kind === 'spec')
        .map((u) => u.value);
      if (newMills.length > 0) {
        setYarnMills([...yarnMills, ...newMills]);
      }
      if (newSpecs.length > 0) {
        setYarnSpecs([...yarnSpecs, ...newSpecs]);
      }
    }
    setUnknownYarns([]);
    void doProceedPreview();
  };

  const onUnknownYarnsCancel = () => {
    // 用户点取消视为 "返回继续改 prompt", 不跳预览.
    setUnknownYarns([]);
  };

  const onPrintPreview = () => {
    const ifWin = previewIframeRef.current?.contentWindow;
    if (!ifWin) return;

    // Chrome / WebView2 给 iframe 调 print() 时, "Save as PDF" 默认文件名
    // 取的是 *主窗口* document.title (= "染谱 Ranpu") 而不是 iframe 自己
    // 的 <title>. 临时把主窗口 title 改成我们要的, 打印结束再还原.
    const date = new Date().toISOString().slice(0, 10);
    // 优先用 prompt 里填的客户名做文件名, 没填则 fallback 到工作区名.
    const customerForFilename = printCustomer || workspaceName;
    const printTitle = customerForFilename
      ? `${sanitizeForFilename(customerForFilename)}-批次单-${date}`
      : `批次单-${date}`;
    const originalTitle = document.title;
    document.title = printTitle;
    const restore = () => {
      document.title = originalTitle;
      ifWin.removeEventListener('afterprint', restore);
    };
    ifWin.addEventListener('afterprint', restore);

    ifWin.focus();
    ifWin.print();
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h2 className="font-serif text-xl tracking-[2px]">批次清单</h2>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={onOpenPreviewPrompt}
            disabled={previewBusy || lines.length === 0}
            title="弹出批次单预览，点打印可调起系统打印对话框（Windows 内置 Microsoft Print to PDF）"
          >
            <Printer className="mr-1 h-4 w-4" />
            {previewBusy ? '生成中…' : '预览 / 打印'}
          </Button>
          <Button
            variant="ghost"
            onClick={() => setAskClear(true)}
            disabled={lines.length === 0}
          >
            <Trash2 className="mr-1 h-4 w-4" /> 清空
          </Button>
        </div>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {lines.length === 0 ? (
        <p className="text-sm text-muted-foreground">批次清单为空。</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>内部色号</TableHead>
              <TableHead>客户色号</TableHead>
              <TableHead>色系</TableHead>
              <TableHead>来源</TableHead>
              <TableHead>目标 kg</TableHead>
              <TableHead>染料明细</TableHead>
              <TableHead className="text-right">克数</TableHead>
              <TableHead></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {lines.map((line, idx) => (
              <TableRow key={`${line.source_kind}-${line.source_formula_id}-${idx}`}>
                <TableCell className="font-medium">
                  {line.internal_color_code ?? '（已删除）'}
                </TableCell>
                <TableCell>{line.customer_color_code ?? '—'}</TableCell>
                <TableCell>{line.color_family ?? '—'}</TableCell>
                <TableCell>
                  <Badge
                    variant={line.source_kind === 'workspace' ? 'default' : 'secondary'}
                  >
                    {line.source_kind === 'workspace' ? '工作区' : '默认库'}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Input
                    type="number"
                    className="h-8 w-24"
                    min={0.01}
                    max={99999.99}
                    step={0.01}
                    defaultValue={line.target_kg}
                    onBlur={(e) => {
                      const v = Number(e.target.value);
                      if (v !== line.target_kg) onChangeKg(line, v);
                    }}
                  />
                </TableCell>
                {/* 染料明细 + 克数 拆成两列, 用 align-top 让多行染料从顶部
                    起算, 两列每一行严格对齐 (同 i 下标 = 同一染料行). */}
                <TableCell className="max-w-md align-top text-xs">
                  {line.calculation ? (
                    <div className="space-y-1">
                      {line.calculation.lines.map((cl, i) => (
                        <div key={i} className="truncate">
                          {cl.dye_name}
                          {cl.dye_code && (
                            <span className="ml-1 text-muted-foreground">
                              ({cl.dye_code})
                            </span>
                          )}
                        </div>
                      ))}
                    </div>
                  ) : (
                    <span className="text-destructive">{line.error ?? '无结果'}</span>
                  )}
                </TableCell>
                <TableCell className="align-top text-right text-xs">
                  {line.calculation && (
                    <div className="space-y-1">
                      {line.calculation.lines.map((cl, i) => (
                        <div key={i} className="font-mono">
                          {formatGrams(cl.grams)}
                        </div>
                      ))}
                    </div>
                  )}
                </TableCell>
                <TableCell className="align-top">
                  <Button size="icon" variant="ghost" onClick={() => onRemove(line)}>
                    <Trash className="h-4 w-4" />
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      <ConfirmDialog
        open={askClear}
        onClose={() => setAskClear(false)}
        title="清空批次清单？"
        description={
          <>
            将移除当前工作区批次清单里的全部 {lines.length} 条配方记录，
            操作不可撤销。被引用的配方本身不会被删除。
          </>
        }
        confirmLabel="清空"
        destructive
        onConfirm={confirmClear}
      />

      <Dialog
        open={promptOpen}
        onOpenChange={(o) => {
          if (!o) onCancelPrompt();
        }}
      >
        <DialogContent className="flex max-h-[85vh] max-w-3xl flex-col gap-0 p-0">
          <DialogHeader className="shrink-0 border-b px-6 py-4">
            <DialogTitle>批次单信息</DialogTitle>
          </DialogHeader>
          <div className="flex-1 space-y-4 overflow-y-auto px-6 py-4">
            <div className="grid gap-1">
              <Label>客户</Label>
              <Input
                value={promptCustomer}
                onChange={(e) => setPromptCustomer(e.target.value)}
                placeholder="默认当前工作区名"
                autoFocus
              />
            </div>

            <div className="space-y-2">
              <Label>每条配方的 缸号 · 缸次 + 纱支厂名 · 规格 · 个数（同一配方可加多份不同纱支）</Label>
              <div className="space-y-3">
                {lines.map((line, idx) => {
                  const meta = promptPerFormula[idx] ?? emptyMeta();
                  const entries = meta.entries.length > 0 ? meta.entries : [emptyEntry()];
                  return (
                    <div
                      key={`${line.source_kind}-${line.source_formula_id}-${idx}`}
                      className="space-y-2 rounded-md border p-3"
                    >
                      <div className="flex items-center gap-2">
                        <div className="flex-1 truncate text-sm">
                          <span className="font-medium">
                            {line.internal_color_code ?? '（已删除）'}
                          </span>
                          {line.color_family && (
                            <span className="ml-1 text-xs text-muted-foreground">
                              · {line.color_family}
                            </span>
                          )}
                        </div>
                        <Input
                          value={meta.vat}
                          onChange={(e) =>
                            updatePromptMeta(idx, { vat: e.target.value })
                          }
                          placeholder="缸号"
                          inputMode="numeric"
                          className="w-24"
                        />
                        <Input
                          value={meta.batch}
                          onChange={(e) =>
                            updatePromptMeta(idx, { batch: e.target.value })
                          }
                          placeholder="缸次"
                          inputMode="numeric"
                          className="w-24"
                        />
                      </div>
                      <div className="space-y-2">
                        {entries.map((entry, entryIdx) => (
                          <div key={entryIdx} className="flex items-center gap-2">
                            <div className="grid flex-1 grid-cols-[1fr_1fr_5rem] gap-2">
                              <ComboboxInput
                                value={entry.yarnMill}
                                onChange={(v) =>
                                  updatePromptEntry(idx, entryIdx, { yarnMill: v })
                                }
                                options={yarnMills}
                                placeholder="纱支厂名"
                              />
                              <ComboboxInput
                                value={entry.yarnSpec}
                                onChange={(v) =>
                                  updatePromptEntry(idx, entryIdx, { yarnSpec: v })
                                }
                                options={yarnSpecs}
                                placeholder="纱支规格"
                              />
                              <Input
                                value={entry.count}
                                onChange={(e) =>
                                  updatePromptEntry(idx, entryIdx, {
                                    count: e.target.value,
                                  })
                                }
                                placeholder="个数"
                                inputMode="decimal"
                              />
                            </div>
                            <Button
                              type="button"
                              variant="ghost"
                              size="icon"
                              onClick={() => removePromptEntry(idx, entryIdx)}
                              aria-label="删除这组纱支"
                              disabled={entries.length === 1}
                              className="shrink-0"
                            >
                              <Trash className="h-4 w-4" />
                            </Button>
                          </div>
                        ))}
                      </div>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => addPromptEntry(idx)}
                      >
                        + 加一组纱支
                      </Button>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
          <DialogFooter className="shrink-0 gap-2 border-t bg-background px-6 py-3">
            <Button variant="ghost" onClick={onCancelPrompt}>
              取消
            </Button>
            <Button onClick={onConfirmPreview}>生成预览</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={previewHtml !== null}
        onOpenChange={(o) => {
          // X / Esc / 点外面关闭都视为 "返回批次单信息" — 关掉预览 + 重开
          // prompt 让用户继续改 (而不是直接退出整个流程). promptCustomer
          // / promptPerFormula state 没动过, prompt 重开后值都还在.
          if (!o) {
            setPreviewHtml(null);
            setPromptOpen(true);
          }
        }}
      >
        <DialogContent className="flex h-[90vh] max-w-5xl flex-col gap-0 p-0">
          <DialogHeader className="shrink-0 flex-row items-center gap-4 border-b px-6 py-4 space-y-0">
            <DialogTitle>批次单预览</DialogTitle>
            <div className="inline-flex rounded-md border bg-background">
              <button
                type="button"
                disabled={previewBusy}
                onClick={() => previewLayout !== 'a6punch' && fetchPreview('a6punch')}
                className={cn(
                  'rounded-l-md px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'a6punch'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                穿孔纸
              </button>
              <button
                type="button"
                disabled={previewBusy}
                onClick={() => previewLayout !== 'label' && fetchPreview('label')}
                className={cn(
                  'border-l px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'label'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                标签纸
              </button>
              <button
                type="button"
                disabled={previewBusy}
                onClick={() => previewLayout !== 'grid' && fetchPreview('grid')}
                className={cn(
                  'border-l px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'grid'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                四宫格
              </button>
              <button
                type="button"
                disabled={previewBusy}
                onClick={() => previewLayout !== 'standard' && fetchPreview('standard')}
                className={cn(
                  'rounded-r-md border-l px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'standard'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                标准
              </button>
            </div>
          </DialogHeader>
          {previewHtml &&
            (previewLayout === 'a6punch' || previewLayout === 'label' ? (
              // 穿孔纸 / 标签纸都是窄页 (120×140mm / 50×80mm), 直接拉满 iframe
              // 看着挤. 包一层灰底 + 内距 + 阴影模拟 "桌上一张纸" 的视觉, 标签
              // 纸更窄, max-w 给小一档.
              <div className="flex-1 overflow-auto bg-muted/40 px-12 py-10">
                <iframe
                  ref={previewIframeRef}
                  srcDoc={previewHtml}
                  title="批次单预览"
                  className={cn(
                    'mx-auto block h-full w-full border bg-white shadow-md',
                    previewLayout === 'label' ? 'max-w-md' : 'max-w-2xl',
                  )}
                />
              </div>
            ) : (
              <iframe
                ref={previewIframeRef}
                srcDoc={previewHtml}
                title="批次单预览"
                className="flex-1 border-0 bg-white"
              />
            ))}
          <DialogFooter className="shrink-0 gap-2 border-t bg-background px-6 py-3">
            <Button
              variant="ghost"
              onClick={() => {
                setPreviewHtml(null);
                setPromptOpen(true);
              }}
            >
              返回修改
            </Button>
            <Button onClick={onPrintPreview}>
              <Printer className="mr-1 h-4 w-4" />
              打印 / 另存为 PDF
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <UnknownYarnPromptDialog
        open={unknownYarns.length > 0}
        unknowns={unknownYarns}
        onConfirm={onUnknownYarnsResolved}
        onCancel={onUnknownYarnsCancel}
      />
    </div>
  );
}

/** Windows 文件名禁用字符: \ / : * ? " < > | + 控制字符 → 用下划线代替. */
function sanitizeForFilename(s: string): string {
  // eslint-disable-next-line no-control-regex
  return s.replace(/[\\/:*?"<>|\x00-\x1f]/g, '_').trim();
}

/// 用 "-" 拼两个字符串, 任一为空则只返回非空那个; 全空返回 null. 用于
/// 缸号-缸次 → "5-2".
function combineDash(a: string, b: string): string | null {
  const x = a.trim();
  const y = b.trim();
  if (!x && !y) return null;
  if (x && y) return `${x}-${y}`;
  return x || y;
}

/// 总纱支个数 = 配方目标 kg / 单个纱支重量. 任一非法返回 0.
function totalCountFor(targetKg: number, singleWeightKg: number): number {
  if (!Number.isFinite(targetKg) || targetKg <= 0) return 0;
  if (!Number.isFinite(singleWeightKg) || singleWeightKg <= 0) return 0;
  return targetKg / singleWeightKg;
}

/// 个数显示成最多 2 位小数, 末尾零去掉; 非有限或负值显示空串.
function formatCount(n: number): string {
  if (!Number.isFinite(n) || n < 0) return '';
  return Number(n.toFixed(2)).toString();
}

export default CartPage;
