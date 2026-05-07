import { save } from '@tauri-apps/plugin-dialog';
import { Download, Trash, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { cartApi } from '@/api/cart';
import { ApiError } from '@/api/invoke';
import { workspaceApi } from '@/api/workspace';
import type { CartLineView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatGrams } from '@/lib/format';
import { hasActiveWorkspace, useSessionStore } from '@/store/session';

export function CartPage() {
  const session = useSessionStore((s) => s.session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;
  const [lines, setLines] = useState<CartLineView[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [workspaceName, setWorkspaceName] = useState<string>('');
  const [askClear, setAskClear] = useState(false);

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

  const onExport = async (format: 'csv' | 'html') => {
    try {
      const date = new Date().toISOString().slice(0, 10);
      const namePrefix = workspaceName
        ? `${sanitizeForFilename(workspaceName)}-批次单-${date}`
        : `批次单-${date}`;
      const out = await save({
        defaultPath: `${namePrefix}.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
      });
      if (!out) return;
      await cartApi.export(format, out);
      alert(
        format === 'html'
          ? '已导出 HTML。在浏览器打开后按 Ctrl+P 可另存为 PDF。'
          : '已导出 CSV。',
      );
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h2 className="font-serif text-xl tracking-[2px]">批次清单</h2>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => onExport('csv')}>
            <Download className="mr-1 h-4 w-4" /> 导出 CSV
          </Button>
          <Button
            variant="outline"
            onClick={() => onExport('html')}
            title="导出网页版批次单 (浏览器打开后 Ctrl+P 可另存为 PDF)"
          >
            <Download className="mr-1 h-4 w-4" /> 导出 HTML
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
              <TableHead>颜色俗称</TableHead>
              <TableHead>来源</TableHead>
              <TableHead>目标 kg</TableHead>
              <TableHead>染料明细 / 总克数</TableHead>
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
                <TableCell>{line.color_name ?? '—'}</TableCell>
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
                <TableCell className="max-w-md text-xs">
                  {line.calculation ? (
                    <div className="space-y-1">
                      {line.calculation.lines.map((cl, i) => (
                        <div key={i} className="flex justify-between gap-2">
                          <span className="truncate">
                            {cl.dye_name}
                            {cl.dye_code && (
                              <span className="ml-1 text-muted-foreground">
                                ({cl.dye_code})
                              </span>
                            )}
                          </span>
                          <span className="font-mono">{formatGrams(cl.grams)}</span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <span className="text-destructive">{line.error ?? '无结果'}</span>
                  )}
                </TableCell>
                <TableCell>
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
    </div>
  );
}

/** Windows 文件名禁用字符: \ / : * ? " < > | + 控制字符 → 用下划线代替. */
function sanitizeForFilename(s: string): string {
  // eslint-disable-next-line no-control-regex
  return s.replace(/[\\/:*?"<>|\x00-\x1f]/g, '_').trim();
}

export default CartPage;
