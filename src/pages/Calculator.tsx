import { Calculator as CalcIcon, ShoppingCart } from 'lucide-react';
import { useState, type FormEvent } from 'react';

import { calculationApi } from '@/api/calculation';
import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import type { CalculationResultView } from '@/api/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
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
import { formatGrams, unitLabel } from '@/lib/format';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';

export function CalculatorPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const [code, setCode] = useState('');
  const [kg, setKg] = useState('10');
  const [result, setResult] = useState<CalculationResultView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [cartBusy, setCartBusy] = useState(false);
  const [cartMsg, setCartMsg] = useState<string | null>(null);

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区，再进行染料计算。
      </p>
    );
  }

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    setResult(null);
    setCartMsg(null);
    try {
      const r = await calculationApi.calculate(code.trim(), Number(kg));
      setResult(r);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const onAddToCart = async () => {
    if (!result || result.formula_id === null) return;
    const sourceKind: 'workspace' | 'default' =
      result.source === 'current_workspace' ? 'workspace' : 'default';
    setCartBusy(true);
    setError(null);
    setCartMsg(null);
    try {
      await cartApi.add(sourceKind, result.formula_id, result.target_kg);
      setCartMsg(
        `已加入购物车：${result.internal_color_code} · ${result.target_kg.toFixed(2)} kg`,
      );
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
          <CardDescription>
            会先在当前工作区查询，找不到再 fallback 到默认库。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={submit} className="grid gap-3 md:grid-cols-3">
            <div className="grid gap-1 md:col-span-2">
              <Label>内部色号</Label>
              <Input
                value={code}
                onChange={(e) => setCode(e.target.value)}
                disabled={busy}
              />
            </div>
            <div className="grid gap-1">
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
            <div className="md:col-span-3">
              <Button
                type="submit"
                disabled={busy || code.trim().length === 0 || !kg}
              >
                {busy ? '计算中…' : '计算'}
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
                  目标 {result.target_kg.toFixed(2)} kg
                </CardDescription>
              </div>
              <Button
                size="sm"
                onClick={onAddToCart}
                disabled={cartBusy || result.formula_id === null}
                title={
                  result.formula_id === null
                    ? '当前结果没有关联配方 ID，无法加车'
                    : '加入购物车'
                }
              >
                <ShoppingCart className="mr-1 h-4 w-4" />
                {cartBusy ? '加入中…' : '加入购物车'}
              </Button>
            </div>
            {cartMsg && (
              <p className="mt-2 text-sm text-emerald-600">{cartMsg}</p>
            )}
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
    </div>
  );
}

export default CalculatorPage;
