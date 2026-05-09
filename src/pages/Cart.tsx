import { save } from '@tauri-apps/plugin-dialog';
import { Printer, Trash, Trash2, Upload } from 'lucide-react';
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
  DialogDescription,
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
import {
  lineKey,
  useBatchSheetInfoStore,
  type PerFormulaMeta,
} from '@/store/batchSheetInfo';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';
import { useSettingsStore } from '@/store/settings';
import {
  maxVatSlot,
  nextSlotsFrom,
  parseVatSlot,
  useVatSequenceStore,
} from '@/store/vatSequence';

export function CartPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;
  const vatCount = useSettingsStore((s) => s.vatCount);
  const setBatchSheetInfo = useBatchSheetInfoStore((s) => s.setInfo);
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
  // 预览版本 toggle: standard (每条一段) 或 grid (A4 九宫格). 用户在
  // 预览框右上角切换, 切换时重新请求对应 HTML.
  const [previewLayout, setPreviewLayout] =
    useState<'standard' | 'grid'>('standard');
  // 当 prompt 里已经有缸号时, "生成缸号" 弹个三选 dialog: 覆盖全部 / 接
  // 续填空白 / 取消. 没填过则直接走覆盖逻辑.
  const [genChoiceOpen, setGenChoiceOpen] = useState(false);

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

  const onExportCsv = async () => {
    try {
      const date = new Date().toISOString().slice(0, 10);
      const namePrefix = workspaceName
        ? `${sanitizeForFilename(workspaceName)}-批次单-${date}`
        : `批次单-${date}`;
      const out = await save({
        defaultPath: `${namePrefix}.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (!out) return;
      await cartApi.export('csv', out);
      alert('已导出 CSV。');
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
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
        return { vat: meta?.vat ?? '', yarn: meta?.yarn ?? '' };
      }),
    );
    setPromptOpen(true);
  };

  const updatePromptMeta = (idx: number, patch: Partial<PerFormulaMeta>) =>
    setPromptPerFormula((prev) =>
      prev.map((m, i) => (i === idx ? { ...m, ...patch } : m)),
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
      if (m.vat || m.yarn) {
        map[lineKey(line.source_kind, line.source_formula_id)] = m;
      }
    });
    setBatchSheetInfo(activeWorkspaceId, { customer, perFormula: map });
  };

  // "生成缸号" 入口. 填一次就把全局计数器推进到本次最大缸号, 下次别的
  // 入口 (本工作区或其它) 生成会接着往后排. 跨日自动从 1-1 重置, 缸号
  // 到 vatCount 自动进入下一批 (例: 4 缸厂 4-2 之后是 1-3). 如果 prompt
  // 里已填了任何缸号, 弹三选 dialog 决定覆盖还是接续; 全空则直接覆盖.
  const onGenerateVats = () => {
    if (lines.length === 0) return;
    const hasFilled = promptPerFormula.some((m) => m.vat.trim() !== '');
    if (hasFilled) {
      setGenChoiceOpen(true);
    } else {
      doGenerateOverwrite();
    }
  };

  // 全部覆盖: 从全局计数器 (存好的号) 之后 peek N 个连续槽位, 重写所有
  // 行. 跟 "只填空白" 不同, 这里完全忽略表里的已填值 — 用户的语义是
  // "全部从头重排", 哪怕手填值更大也不沿用. 完成后 commit 推进计数器.
  const doGenerateOverwrite = () => {
    const store = useVatSequenceStore.getState();
    const slots = store.peek(lines.length, vatCount);
    const next = promptPerFormula.map((m, i) => {
      const slot = slots[i];
      if (!slot) return m;
      return { ...m, vat: `${slot.vat}-${slot.batch}` };
    });
    setPromptPerFormula(next);
    const last = slots[slots.length - 1];
    if (last) store.commit(last);
    persistPromptInfo(promptCustomer, next);
  };

  // 接续填写: 按当前 prompt 里能解析出的 (batch, vat) 字典序最大缸号
  // 往后排, 只填空白行. 没法解析出合法号时退回 peek 全局计数器. 完成
  // 后把计数器推到本次最大缸号.
  const doGenerateContinue = () => {
    const emptyIdx: number[] = [];
    promptPerFormula.forEach((m, i) => {
      if (!m.vat.trim()) emptyIdx.push(i);
    });
    if (emptyIdx.length === 0) return;
    const filledMax = maxVatSlot(
      promptPerFormula
        .map((m) => parseVatSlot(m.vat))
        .filter((s): s is NonNullable<typeof s> => s !== null),
    );
    const slots = filledMax
      ? nextSlotsFrom(filledMax, emptyIdx.length, vatCount)
      : useVatSequenceStore.getState().peek(emptyIdx.length, vatCount);
    const next = promptPerFormula.map((m, i) => {
      const slotIdx = emptyIdx.indexOf(i);
      if (slotIdx === -1) return m;
      const slot = slots[slotIdx];
      if (!slot) return m;
      return { ...m, vat: `${slot.vat}-${slot.batch}` };
    });
    setPromptPerFormula(next);
    const last = slots[slots.length - 1];
    if (last) useVatSequenceStore.getState().commit(last);
    persistPromptInfo(promptCustomer, next);
  };

  // dialog 文案要展示当前最大已填缸号, 让用户能预判 "接续" 起点. 计算
  // 在打开 dialog 时进行, 不读 store, 完全基于 prompt state.
  const filledMaxLabel = (() => {
    const max = maxVatSlot(
      promptPerFormula
        .map((m) => parseVatSlot(m.vat))
        .filter((s): s is NonNullable<typeof s> => s !== null),
    );
    return max ? `${max.vat}-${max.batch}` : null;
  })();

  const onCancelPrompt = () => {
    persistPromptInfo(promptCustomer, promptPerFormula);
    setPromptOpen(false);
  };

  // 用最新 prompt + 指定 layout 拉一份预览 HTML. 提交 prompt / 切 tab 都走这里.
  const fetchPreview = async (layout: 'standard' | 'grid') => {
    const customer = promptCustomer.trim();
    setPreviewBusy(true);
    try {
      const html = await cartApi.previewHtml({
        customer: customer || null,
        perFormula: promptPerFormula.map((m) => ({
          vatNumber: m.vat.trim() || null,
          yarnCount: m.yarn.trim() || null,
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
    const customer = promptCustomer.trim();
    persistPromptInfo(promptCustomer, promptPerFormula);
    setPromptOpen(false);
    setPrintCustomer(customer || workspaceName);
    // 进预览默认 standard 版本; 用户可在右上角切到 grid.
    await fetchPreview('standard');
  };

  const onPrintPreview = () => {
    const ifWin = previewIframeRef.current?.contentWindow;
    if (!ifWin) return;

    // 打印时再 commit 一次, 兜底用户在生成缸号之后手填了更大的号. commit
    // 内部仅当新值大于当前 (或跨日) 才推进, 重复 commit 没副作用.
    const printedSlots = promptPerFormula
      .map((m) => parseVatSlot(m.vat))
      .filter((s): s is NonNullable<typeof s> => s !== null);
    const last = maxVatSlot(printedSlots);
    if (last) {
      useVatSequenceStore.getState().commit(last);
    }

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
            onClick={onExportCsv}
            disabled={lines.length === 0}
          >
            <Upload className="mr-1 h-4 w-4" /> 导出 CSV
          </Button>
          <Button
            variant="outline"
            onClick={onOpenPreviewPrompt}
            disabled={previewBusy || lines.length === 0}
            title="弹出批次单预览, 点打印可调起系统打印对话框 (Windows 内置 Microsoft Print to PDF)"
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
        <DialogContent className="flex max-h-[85vh] max-w-2xl flex-col gap-0 p-0">
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
              <div className="flex items-center justify-between">
                <Label>每条配方的缸号 / 纱支 (留空则不显示)</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={onGenerateVats}
                  disabled={lines.length === 0}
                  title={`按设置中的染缸数量 (${vatCount}) 自动生成连续缸号. 跨日自动重置批次.`}
                >
                  生成缸号
                </Button>
              </div>
              <div className="space-y-2">
                {lines.map((line, idx) => (
                  <div
                    key={`${line.source_kind}-${line.source_formula_id}-${idx}`}
                    className="grid grid-cols-12 items-end gap-2"
                  >
                    <div className="col-span-4 truncate text-sm">
                      <span className="font-medium">
                        {line.internal_color_code ?? '（已删除）'}
                      </span>
                      {line.color_family && (
                        <span className="ml-1 text-xs text-muted-foreground">
                          · {line.color_family}
                        </span>
                      )}
                    </div>
                    <div className="col-span-4">
                      <Input
                        value={promptPerFormula[idx]?.vat ?? ''}
                        onChange={(e) =>
                          updatePromptMeta(idx, { vat: e.target.value })
                        }
                        placeholder="缸号 (例: 5-2)"
                      />
                    </div>
                    <div className="col-span-4">
                      <Input
                        value={promptPerFormula[idx]?.yarn ?? ''}
                        onChange={(e) =>
                          updatePromptMeta(idx, { yarn: e.target.value })
                        }
                        placeholder="纱支 (例: 32S/2)"
                      />
                    </div>
                  </div>
                ))}
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
        open={genChoiceOpen}
        onOpenChange={(o) => !o && setGenChoiceOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>如何生成缸号?</DialogTitle>
            <DialogDescription>
              {filledMaxLabel ? (
                <>
                  当前批次单已填到 {filledMaxLabel}. 「全部重新生成」会从系统最后一次发出的缸号之后给每一行重新编号 (已填值将被覆盖);
                  <br />
                  「只填空白」则保留已填行, 把空白行接在 {filledMaxLabel} 之后.
                </>
              ) : (
                <>
                  批次单已填了缸号. 「全部重新生成」会从系统最后一次发出的缸号之后给每一行重新编号;
                  <br />
                  「只填空白」则保留已填行, 只补空白行.
                </>
              )}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2">
            <Button variant="ghost" onClick={() => setGenChoiceOpen(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                setGenChoiceOpen(false);
                doGenerateOverwrite();
              }}
            >
              全部重新生成
            </Button>
            <Button
              onClick={() => {
                setGenChoiceOpen(false);
                doGenerateContinue();
              }}
            >
              只填空白
            </Button>
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
                onClick={() => previewLayout !== 'standard' && fetchPreview('standard')}
                className={cn(
                  'rounded-l-md px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'standard'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                标准
              </button>
              <button
                type="button"
                disabled={previewBusy}
                onClick={() => previewLayout !== 'grid' && fetchPreview('grid')}
                className={cn(
                  'rounded-r-md border-l px-5 py-2 text-sm font-medium transition-colors',
                  previewLayout === 'grid'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-foreground hover:bg-accent',
                )}
              >
                九宫格
              </button>
            </div>
          </DialogHeader>
          {previewHtml && (
            <iframe
              ref={previewIframeRef}
              srcDoc={previewHtml}
              title="批次单预览"
              className="flex-1 border-0 bg-white"
            />
          )}
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
    </div>
  );
}

/** Windows 文件名禁用字符: \ / : * ? " < > | + 控制字符 → 用下划线代替. */
function sanitizeForFilename(s: string): string {
  // eslint-disable-next-line no-control-regex
  return s.replace(/[\\/:*?"<>|\x00-\x1f]/g, '_').trim();
}

export default CartPage;
