import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import {
  CheckSquare,
  Copy,
  Download,
  Loader2,
  Plus,
  Search,
  Square,
  Upload,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type {
  BatchCopySummaryView,
  FormulaView,
  ImportFormulasSummaryView,
} from '@/api/types';
import { FormulaCard } from '@/components/FormulaCard';
import { FormulaEditor } from '@/components/FormulaEditor';
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
import { hasActiveWorkspace, isAdmin, useSessionStore } from '@/store/session';

export function DefaultLibraryPage() {
  const session = useSessionStore((s) => s.session);
  const admin = isAdmin(session);
  const hasWs = hasActiveWorkspace(session);

  const [keyword, setKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  /** 第一次拉数据完成前为 true; 之后只在显式刷新时短暂为 true。 */
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 多选状态：仅 admin + 已激活 workspace 时启用
  const selectionEnabled = admin && hasWs;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [batchBusy, setBatchBusy] = useState(false);
  const [batchSummary, setBatchSummary] = useState<BatchCopySummaryView | null>(null);

  // 加密导入导出对话框状态
  const [exportOpen, setExportOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [importSummary, setImportSummary] = useState<ImportFormulasSummaryView | null>(
    null,
  );

  const load = (kw?: string) => {
    setLoading(true);
    return formulaApi
      .listDefault({ keyword: kw ?? keyword })
      .then((data) => {
        setList(data);
        // 列表刷新后清掉无效的 selected id
        setSelectedIds((prev) => {
          const allIds = new Set(data.map((f) => f.id));
          return new Set([...prev].filter((id) => allIds.has(id)));
        });
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 失去 selectionEnabled 资格时清空选择，避免状态不一致
  useEffect(() => {
    if (!selectionEnabled) setSelectedIds(new Set());
  }, [selectionEnabled]);

  const onCopyToWorkspace = async (formula: FormulaView) => {
    try {
      await formulaApi.copyDefaultToWorkspace(formula.id);
      alert('已复制到当前工作区');
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onDelete = async (formula: FormulaView) => {
    if (!confirm(`确认删除「${formula.internal_color_code}」？`)) return;
    try {
      await formulaApi.deleteDefault(formula.id);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertDefault>[0]) => {
    await formulaApi.upsertDefault(payload);
    setEditorOpen(false);
    load();
  };

  const onToggleSelected = (formula: FormulaView, next: boolean) => {
    setSelectedIds((prev) => {
      const out = new Set(prev);
      if (next) out.add(formula.id);
      else out.delete(formula.id);
      return out;
    });
  };

  const allSelected =
    list.length > 0 && list.every((f) => selectedIds.has(f.id));

  const onToggleSelectAll = () => {
    if (allSelected) setSelectedIds(new Set());
    else setSelectedIds(new Set(list.map((f) => f.id)));
  };

  const onBatchCopy = async () => {
    if (selectedIds.size === 0) return;
    setBatchBusy(true);
    setError(null);
    try {
      const summary = await formulaApi.batchCopyDefaultToWorkspace([...selectedIds]);
      setBatchSummary(summary);
      // 清空选择，方便下一轮
      setSelectedIds(new Set());
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBatchBusy(false);
    }
  };

  const failedItems = useMemo(
    () => batchSummary?.items.filter((i) => i.error !== null) ?? [],
    [batchSummary],
  );

  const onPickExportPath = async (): Promise<string | null> => {
    const out = await saveDialog({
      defaultPath: `默认配方-${new Date().toISOString().slice(0, 10)}.ydaexp`,
      filters: [{ name: 'YDAEXP', extensions: ['ydaexp'] }],
    });
    return typeof out === 'string' ? out : null;
  };

  const onPickImportPath = async (): Promise<string | null> => {
    const path = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: 'YDAEXP', extensions: ['ydaexp'] }],
    });
    return typeof path === 'string' ? path : null;
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">默认配方库</h2>
        <div className="flex flex-wrap items-center gap-2">
          {selectionEnabled && (
            <>
              <Button
                size="sm"
                variant="outline"
                onClick={onToggleSelectAll}
                disabled={list.length === 0}
              >
                {allSelected ? (
                  <CheckSquare className="mr-1 h-4 w-4" />
                ) : (
                  <Square className="mr-1 h-4 w-4" />
                )}
                {allSelected ? '取消全选' : '全选当前页'}
              </Button>
              <Button
                size="sm"
                disabled={selectedIds.size === 0 || batchBusy}
                onClick={onBatchCopy}
              >
                <Copy className="mr-1 h-4 w-4" />
                {batchBusy
                  ? '正在复制…'
                  : `批量复制到工作区${selectedIds.size > 0 ? ` (${selectedIds.size})` : ''}`}
              </Button>
            </>
          )}
          {admin && (
            <Button variant="outline" onClick={() => setExportOpen(true)}>
              <Upload className="mr-1 h-4 w-4" /> 加密导出
            </Button>
          )}
          {admin && (
            <Button variant="outline" onClick={() => setImportOpen(true)}>
              <Download className="mr-1 h-4 w-4" /> 加密导入
            </Button>
          )}
          {admin && (
            <Button
              onClick={() => {
                setEditing(null);
                setEditorOpen(true);
              }}
            >
              <Plus className="mr-1 h-4 w-4" /> 新建配方
            </Button>
          )}
        </div>
      </div>

      <div className="flex items-center gap-2 max-w-md">
        <div className="relative flex-1">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-8"
            placeholder="搜索内部色号 / 客户色号 / 颜色俗称"
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && load()}
          />
        </div>
        <Button variant="outline" onClick={() => load()}>
          搜索
        </Button>
      </div>

      {selectionEnabled && selectedIds.size > 0 && (
        <p className="text-xs text-muted-foreground">
          已选 <Badge variant="default">{selectedIds.size}</Badge> 条配方
        </p>
      )}
      {admin && !hasWs && (
        <p className="text-xs text-muted-foreground">
          想批量复制配方？请先在顶栏选择目标工作区。
        </p>
      )}

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading && list.length === 0 ? (
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          正在加载…
        </div>
      ) : list.length === 0 ? (
        <p className="text-sm text-muted-foreground">没有匹配的配方。</p>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {list.map((f) => (
            <FormulaCard
              key={f.id}
              formula={f}
              source="default"
              canManage={admin}
              hasActiveWorkspace={hasWs}
              onCopyToWorkspace={onCopyToWorkspace}
              onEdit={
                admin
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={admin ? onDelete : undefined}
              selected={selectionEnabled ? selectedIds.has(f.id) : undefined}
              onToggleSelected={selectionEnabled ? onToggleSelected : undefined}
            />
          ))}
        </div>
      )}

      <FormulaEditor
        open={editorOpen}
        onClose={() => setEditorOpen(false)}
        initial={editing}
        scope="默认"
        onSave={onSave}
      />

      <Dialog
        open={batchSummary !== null}
        onOpenChange={(o) => !o && setBatchSummary(null)}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>批量复制结果</DialogTitle>
          </DialogHeader>
          <div className="text-sm">
            成功 <Badge variant="default">{batchSummary?.succeeded ?? 0}</Badge>{' '}
            条，失败 <Badge variant="destructive">{batchSummary?.failed ?? 0}</Badge>{' '}
            条。
          </div>
          {failedItems.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>源配方 ID</TableHead>
                  <TableHead>错误</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {failedItems.map((i) => (
                  <TableRow key={i.source_default_id}>
                    <TableCell className="font-mono">{i.source_default_id}</TableCell>
                    <TableCell className="text-destructive">{i.error}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
          <DialogFooter>
            <Button onClick={() => setBatchSummary(null)}>知道了</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ExportFormulasDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        selectedIds={[...selectedIds]}
        totalCount={list.length}
        onPickPath={onPickExportPath}
        onSubmit={async (passphrase, ids, outPath) => {
          const count = await formulaApi.exportDefaultFormulas(ids, passphrase, outPath);
          setExportOpen(false);
          alert(`已加密导出 ${count} 条配方到\n${outPath}`);
        }}
      />

      <ImportFormulasDialog
        open={importOpen}
        onClose={() => setImportOpen(false)}
        onPickPath={onPickImportPath}
        onSubmit={async (passphrase, inPath) => {
          const summary = await formulaApi.importDefaultFormulas(passphrase, inPath);
          setImportOpen(false);
          setImportSummary(summary);
          load();
        }}
      />

      <Dialog
        open={importSummary !== null}
        onOpenChange={(o) => !o && setImportSummary(null)}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>导入结果</DialogTitle>
          </DialogHeader>
          <div className="text-sm">
            导入 <Badge variant="default">{importSummary?.imported ?? 0}</Badge>{' '}
            条，跳过{' '}
            <Badge variant="secondary">{importSummary?.skipped ?? 0}</Badge>{' '}
            条 (内部色号已存在)，失败{' '}
            <Badge variant="destructive">{importSummary?.failed ?? 0}</Badge>{' '}
            条。
          </div>
          {importSummary && importSummary.items.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>内部色号</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>说明</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {importSummary.items.map((i, idx) => (
                  <TableRow key={`${i.internal_color_code}-${idx}`}>
                    <TableCell className="font-mono">
                      {i.internal_color_code}
                    </TableCell>
                    <TableCell>
                      {i.status === 'imported' && (
                        <Badge variant="default">已导入</Badge>
                      )}
                      {i.status === 'skipped_duplicate' && (
                        <Badge variant="secondary">已跳过</Badge>
                      )}
                      {i.status === 'failed' && (
                        <Badge variant="destructive">失败</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-xs text-destructive">
                      {i.error ?? ''}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
          <DialogFooter>
            <Button onClick={() => setImportSummary(null)}>知道了</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

interface ExportDialogProps {
  open: boolean;
  onClose: () => void;
  selectedIds: number[];
  totalCount: number;
  onPickPath: () => Promise<string | null>;
  onSubmit: (passphrase: string, ids: number[], outPath: string) => Promise<void>;
}

function ExportFormulasDialog({
  open,
  onClose,
  selectedIds,
  totalCount,
  onPickPath,
  onSubmit,
}: ExportDialogProps) {
  const [passphrase, setPassphrase] = useState('');
  const [passphrase2, setPassphrase2] = useState('');
  const [scope, setScope] = useState<'selected' | 'all'>('selected');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setPassphrase('');
      setPassphrase2('');
      setErr(null);
      setScope(selectedIds.length > 0 ? 'selected' : 'all');
    }
  }, [open, selectedIds.length]);

  const submit = async () => {
    setErr(null);
    if (passphrase.length < 8) {
      setErr('导出口令至少 8 位');
      return;
    }
    if (passphrase !== passphrase2) {
      setErr('两次输入的口令不一致');
      return;
    }
    const ids = scope === 'selected' ? selectedIds : [];
    if (scope === 'selected' && ids.length === 0) {
      setErr('未选中任何配方，请先勾选或改成「导出全部」');
      return;
    }
    const out = await onPickPath();
    if (!out) return;
    setBusy(true);
    try {
      await onSubmit(passphrase, ids, out);
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>加密导出默认配方</DialogTitle>
          <DialogDescription>
            导出文件 (.ydaexp) 用 AES-256-GCM 加密，需要导出口令才能在另一台机器导入。
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>范围</Label>
          <div className="flex flex-col gap-1 text-sm">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="export-scope"
                checked={scope === 'selected'}
                onChange={() => setScope('selected')}
                disabled={selectedIds.length === 0}
              />
              已勾选 {selectedIds.length} 条
              {selectedIds.length === 0 && (
                <span className="text-xs text-muted-foreground">
                  （未勾选任何配方）
                </span>
              )}
            </label>
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="export-scope"
                checked={scope === 'all'}
                onChange={() => setScope('all')}
              />
              当前默认库全部 {totalCount} 条
            </label>
          </div>
        </div>
        <div className="grid gap-2">
          <Label>导出口令（≥ 8 位）</Label>
          <Input
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
          />
        </div>
        <div className="grid gap-2">
          <Label>再次输入口令</Label>
          <Input
            type="password"
            value={passphrase2}
            onChange={(e) => setPassphrase2(e.target.value)}
            disabled={busy}
          />
        </div>
        {err && <p className="text-sm text-destructive">{err}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy}>
            {busy ? '导出中…' : '选路径并导出'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface ImportDialogProps {
  open: boolean;
  onClose: () => void;
  onPickPath: () => Promise<string | null>;
  onSubmit: (passphrase: string, inPath: string) => Promise<void>;
}

function ImportFormulasDialog({
  open,
  onClose,
  onPickPath,
  onSubmit,
}: ImportDialogProps) {
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setPassphrase('');
      setErr(null);
    }
  }, [open]);

  const submit = async () => {
    setErr(null);
    if (passphrase.length === 0) {
      setErr('请输入导出时使用的口令');
      return;
    }
    const inPath = await onPickPath();
    if (!inPath) return;
    setBusy(true);
    try {
      await onSubmit(passphrase, inPath);
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>加密导入默认配方</DialogTitle>
          <DialogDescription>
            选择 .ydaexp 文件 + 导出时设置的口令；同内部色号的配方将被自动跳过。
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>解密口令</Label>
          <Input
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            disabled={busy}
          />
        </div>
        {err && <p className="text-sm text-destructive">{err}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy}>
            {busy ? '导入中…' : '选文件并导入'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default DefaultLibraryPage;
