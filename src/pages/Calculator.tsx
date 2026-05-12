import { Calculator as CalcIcon, Loader2, ShoppingCart } from 'lucide-react';
import { useState, type FormEvent } from 'react';

import { calculationApi } from '@/api/calculation';
import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import type { CalculationResultView, CustomerCodeMatchView } from '@/api/types';
import { useCartStaleGuard } from '@/components/CartStaleGuard';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatAmount, formatGrams, unitLabel } from '@/lib/format';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';
import { useResetOnLock } from '@/hooks/useResetOnLock';

type SearchMode = 'internal' | 'customer';

export function CalculatorPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const [mode, setMode] = useState<SearchMode>('internal');
  const [code, setCode] = useState('');
  const [kg, setKg] = useState('10');
  const [result, setResult] = useState<CalculationResultView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [cartBusy, setCartBusy] = useState(false);
  const [cartMsg, setCartMsg] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<CustomerCodeMatchView[] | null>(null);
  // 命中已存在条目时的确认对话框上下文：existingKg 是当前批次清单里的 kg。
  const [conflict, setConflict] = useState<{
    sourceKind: 'workspace' | 'default';
    formulaId: number;
    internalCode: string;
    addKg: number;
    existingKg: number;
  } | null>(null);
  // 跨日工作前提醒清空昨天残留的批次清单. dialog 在 JSX 末尾渲染.
  const { guard: cartStaleGuard, dialog: cartStaleDialog } = useCartStaleGuard({
    onError: setError,
  });

  // 锁屏触发时关 conflict 确认 + 候选客户色号选择 Dialog, 不让 focus-scope
  // 卡 LockOverlay.
  useResetOnLock(() => {
    setConflict(null);
    setCandidates(null);
  });

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区，再进行染料计算。
      </p>
    );
  }

  const calcWithInternalCode = async (internalCode: string) => {
    const r = await calculationApi.calculate(internalCode, Number(kg));
    setResult(r);
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    setResult(null);
    setCartMsg(null);
    setCandidates(null);
    try {
      if (mode === 'internal') {
        await calcWithInternalCode(code.trim());
      } else {
        const list = await calculationApi.searchByCustomerCode(code.trim());
        if (list.length === 0) {
          setError('没找到客户色号匹配的配方');
        } else if (list.length === 1) {
          // 只有一条直接计算，省一次点击
          await calcWithInternalCode(list[0].internal_color_code);
        } else {
          // 多条让用户挑
          setCandidates(list);
        }
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const onPickCandidate = async (m: CustomerCodeMatchView) => {
    setCandidates(null);
    setBusy(true);
    setError(null);
    try {
      await calcWithInternalCode(m.internal_color_code);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const onAddToCart = () => {
    if (!result || result.formula_id === null) return;
    cartStaleGuard(() => {
      void doAddToCart();
    });
  };

  const doAddToCart = async () => {
    if (!result || result.formula_id === null) return;
    const sourceKind: 'workspace' | 'default' =
      result.source === 'current_workspace' ? 'workspace' : 'default';
    const formulaId = result.formula_id;
    const addKg = result.target_kg;
    const internalCode = result.internal_color_code;

    setCartBusy(true);
    setError(null);
    setCartMsg(null);
    // 加进清单前先看批次清单里有没有同 (source_kind, formula_id)
    try {
      const cart = await cartApi.list();
      const existing = cart.find(
        (l) => l.source_kind === sourceKind && l.source_formula_id === formulaId,
      );
      if (existing) {
        setConflict({
          sourceKind,
          formulaId,
          internalCode,
          addKg,
          existingKg: existing.target_kg,
        });
        setCartBusy(false);
        return;
      }
      // 没冲突 → 直接加
      await cartApi.add(sourceKind, formulaId, addKg);
      setCartMsg(`已加入批次清单：${internalCode} · ${formatAmount(addKg)} kg`);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setCartBusy(false);
    }
  };

  const resolveConflict = async (action: 'accumulate' | 'replace') => {
    if (!conflict) return;
    const { sourceKind, formulaId, internalCode, addKg, existingKg } = conflict;
    setCartBusy(true);
    setError(null);
    setCartMsg(null);
    try {
      if (action === 'accumulate') {
        const sum = Math.min(existingKg + addKg, 99999.99);
        await cartApi.updateKg(sourceKind, formulaId, sum);
        setCartMsg(
          `已累加到批次清单：${internalCode} · ${formatAmount(existingKg)} + ${formatAmount(addKg)} = ${formatAmount(sum)} kg`,
        );
      } else {
        // 替换为新的 kg：直接复用 add（后端 add 即 add_or_update，命中 → 覆盖 kg）。
        await cartApi.add(sourceKind, formulaId, addKg);
        setCartMsg(
          `已覆盖批次清单 kg：${internalCode} · ${formatAmount(existingKg)} → ${formatAmount(addKg)} kg`,
        );
      }
      setConflict(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setCartBusy(false);
    }
  };


  return (
    <div className="space-y-4 p-6">
      <h2 className="font-serif text-xl tracking-[2px]">染料计算器</h2>

      <Card className="max-w-2xl">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <CalcIcon className="h-5 w-5" /> 输入色号与目标重量
          </CardTitle>
          <CardDescription className="space-y-1">
            <div>内部色号：先在当前工作区查询，找不到再重定向到默认库。</div>
            <div>客户色号：跨当前工作区与默认库找匹配，多条候选时让你挑。</div>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={submit} className="grid gap-3 md:grid-cols-12">
            <div className="grid gap-1 md:col-span-3">
              <Label>查询模式</Label>
              <Select
                value={mode}
                onValueChange={(v) => {
                  setMode(v as SearchMode);
                  setResult(null);
                  setCandidates(null);
                  setError(null);
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="internal">内部色号</SelectItem>
                  <SelectItem value="customer">客户色号</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-1 md:col-span-6">
              <Label>{mode === 'internal' ? '内部色号' : '客户色号'}</Label>
              <Input
                value={code}
                onChange={(e) => setCode(e.target.value)}
                disabled={busy}
                placeholder={
                  mode === 'internal' ? '例如 N-2024' : '例如 CUST-NAVY-01'
                }
              />
            </div>
            <div className="grid gap-1 md:col-span-3">
              <Label>目标 kg</Label>
              <Input
                type="number"
                min={0.01}
                max={99999.99}
                step={0.01}
                value={kg}
                onChange={(e) => setKg(e.target.value)}
                disabled={busy}
              />
            </div>
            <div className="md:col-span-12">
              <Button
                type="submit"
                disabled={busy || code.trim().length === 0 || !kg}
              >
                {busy
                  ? mode === 'internal'
                    ? '计算中…'
                    : '搜索中…'
                  : mode === 'internal'
                    ? '计算'
                    : '搜索并计算'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {result && (
        <Card>
          <CardHeader>
            <div className="flex items-start justify-between gap-3">
              <div>
                <CardTitle className="flex items-center gap-3 text-base">
                  <span>{result.internal_color_code}</span>
                  <Badge
                    variant={
                      result.source === 'current_workspace'
                        ? 'default'
                        : 'secondary'
                    }
                  >
                    {result.source_label}
                  </Badge>
                </CardTitle>
                <CardDescription>
                  目标 {formatAmount(result.target_kg)} kg
                </CardDescription>
              </div>
              <Button
                size="sm"
                onClick={onAddToCart}
                disabled={cartBusy || result.formula_id === null}
                title={
                  result.formula_id === null
                    ? '当前结果没有关联配方 ID，无法加车'
                    : '加入批次清单'
                }
              >
                {cartBusy ? (
                  <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                ) : (
                  <ShoppingCart className="mr-1 h-4 w-4" />
                )}
                加入批次清单
              </Button>
            </div>
            {/* 永远占位 min-h-5, 防止消息出现/消失导致 CardHeader 高度跳动 */}
            <p
              aria-live="polite"
              className="mt-2 min-h-5 text-sm text-emerald-600"
            >
              {cartMsg ?? ' '}
            </p>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>染料</TableHead>
                  <TableHead>编号</TableHead>
                  <TableHead className="text-right">克数</TableHead>
                  <TableHead>原始单位</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {result.lines.map((l, idx) => (
                  <TableRow key={idx}>
                    <TableCell>{l.dye_name}</TableCell>
                    <TableCell>{l.dye_code ?? '—'}</TableCell>
                    <TableCell className="text-right font-mono">
                      {formatGrams(l.grams)}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {unitLabel(l.unit_used)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      <Dialog
        open={conflict !== null}
        onOpenChange={(o) => !o && setConflict(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>批次清单里已有这条配方</DialogTitle>
            <DialogDescription>
              {conflict && (
                <>
                  <span className="font-mono">{conflict.internalCode}</span>{' '}
                  当前批次清单记录{' '}
                  <span className="font-mono">{formatAmount(conflict.existingKg)}</span>{' '}
                  kg，本次想加的是{' '}
                  <span className="font-mono">{formatAmount(conflict.addKg)}</span> kg。
                </>
              )}
            </DialogDescription>
          </DialogHeader>
          <div className="rounded-md border bg-muted/30 p-3 text-sm">
            选择处理方式：
            <ul className="mt-2 space-y-1 text-xs text-muted-foreground">
              <li>
                · 累加：把这次的 kg 加到批次清单现有 kg 上（
                {conflict
                  ? `${formatAmount(conflict.existingKg)} + ${formatAmount(conflict.addKg)} = ${formatAmount(
                      Math.min(conflict.existingKg + conflict.addKg, 99999.99),
                    )} kg`
                  : ''}
                ）
              </li>
              <li>
                · 覆盖：用本次的 kg 直接替换掉批次清单里的 kg（
                {conflict ? `${formatAmount(conflict.addKg)} kg` : ''}）
              </li>
            </ul>
          </div>
          <DialogFooter className="gap-2">
            <Button variant="ghost" onClick={() => setConflict(null)}>
              取消
            </Button>
            <Button
              variant="outline"
              disabled={cartBusy}
              onClick={() => resolveConflict('replace')}
            >
              {cartBusy ? '处理中…' : '覆盖'}
            </Button>
            <Button
              disabled={cartBusy}
              onClick={() => resolveConflict('accumulate')}
            >
              {cartBusy ? '处理中…' : '累加'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={candidates !== null}
        onOpenChange={(o) => !o && setCandidates(null)}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>选择一条配方</DialogTitle>
            <DialogDescription>
              「{code}」匹配到 {candidates?.length ?? 0} 条配方，请挑一条进行计算。
            </DialogDescription>
          </DialogHeader>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>内部色号</TableHead>
                <TableHead>色系</TableHead>
                <TableHead>客户色号</TableHead>
                <TableHead>来源</TableHead>
                <TableHead></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {candidates?.map((m) => (
                <TableRow key={`${m.source}-${m.formula_id}`}>
                  <TableCell className="font-medium">
                    {m.internal_color_code}
                  </TableCell>
                  <TableCell>{m.color_family ?? '—'}</TableCell>
                  <TableCell>{m.customer_color_code ?? '—'}</TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        m.source === 'current_workspace' ? 'default' : 'secondary'
                      }
                    >
                      {m.source_label}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <Button size="sm" onClick={() => onPickCandidate(m)}>
                      选这条
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setCandidates(null)}>
              取消
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {cartStaleDialog}
    </div>
  );
}

export default CalculatorPage;
