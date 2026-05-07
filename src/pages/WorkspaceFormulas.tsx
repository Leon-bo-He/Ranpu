import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import {
  CheckSquare,
  Download,
  Loader2,
  Plus,
  Search,
  Square,
  Upload,
} from 'lucide-react';
import { useEffect, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import type { FormulaView, ImportFormulasSummaryView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
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

export function WorkspaceFormulasPage() {
  const session = useSessionStore((s) => s.session);
  const admin = isAdmin(session);
  const hasWs = hasActiveWorkspace(session);
  const activeWorkspaceId = session?.active_workspace_id ?? null;

  const [keyword, setKeyword] = useState('');
  const [list, setList] = useState<FormulaView[]>([]);
  const [loading, setLoading] = useState(true);
  const [editorOpen, setEditorOpen] = useState(false);
  const [editing, setEditing] = useState<FormulaView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<FormulaView | null>(null);

  // 多选 + 加密导入导出 (admin)
  const selectionEnabled = admin;
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [exportOpen, setExportOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [importSummary, setImportSummary] = useState<ImportFormulasSummaryView | null>(
    null,
  );

  const load = () => {
    if (!hasWs) {
      setList([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    return formulaApi
      .listWorkspace({ keyword })
      .then((data) => {
        setList(data);
        setSelectedIds((prev) => {
          const allIds = new Set(data.map((f) => f.id));
          return new Set([...prev].filter((id) => allIds.has(id)));
        });
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)))
      .finally(() => setLoading(false));
  };

  // 依赖 active_workspace_id 而非 hasWs (boolean), 不然 A→B 切换不会触发刷新.
  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeWorkspaceId]);

  // 切换工作区清掉旧选择
  useEffect(() => {
    setSelectedIds(new Set());
  }, [activeWorkspaceId]);

  if (!hasWs) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        请先在顶栏选择一个工作区。
      </p>
    );
  }

  const askDelete = (formula: FormulaView) => setPendingDelete(formula);

  const confirmDelete = async () => {
    if (!pendingDelete) return;
    try {
      await formulaApi.deleteWorkspace(pendingDelete.id);
      setPendingDelete(null);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setPendingDelete(null);
    }
  };

  const onSave = async (payload: Parameters<typeof formulaApi.upsertWorkspace>[0]) => {
    await formulaApi.upsertWorkspace(payload);
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

  const allSelected = list.length > 0 && list.every((f) => selectedIds.has(f.id));
  const onToggleSelectAll = () => {
    if (allSelected) setSelectedIds(new Set());
    else setSelectedIds(new Set(list.map((f) => f.id)));
  };

  const onPickExportPath = async (): Promise<string | null> => {
    const out = await saveDialog({
      defaultPath: `工作区配方-${new Date().toISOString().slice(0, 10)}.ranpu`,
      filters: [{ name: 'Ranpu 加密包', extensions: ['ranpu'] }],
    });
    return typeof out === 'string' ? out : null;
  };

  const onPickImportPath = async (): Promise<string | null> => {
    const path = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: 'Ranpu 加密包', extensions: ['ranpu'] }],
    });
    return typeof path === 'string' ? path : null;
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-serif text-xl tracking-[2px]">工作区配方</h2>
        <div className="flex flex-wrap items-center gap-2">
          {selectionEnabled && (
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
              source="workspace"
              canManage={admin}
              hasActiveWorkspace={hasWs}
              onEdit={
                admin
                  ? (f) => {
                      setEditing(f);
                      setEditorOpen(true);
                    }
                  : undefined
              }
              onDelete={admin ? askDelete : undefined}
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
        scope="工作区"
        onSave={onSave}
      />

      <ConfirmDialog
        open={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title="确认删除配方？"
        description={
          pendingDelete && (
            <>
              将永久删除当前工作区中的{' '}
              <span className="font-mono">{pendingDelete.internal_color_code}</span>
              {pendingDelete.color_name && <> · {pendingDelete.color_name}</>}
              {' '}及其所有染料明细，操作不可撤销。
            </>
          )
        }
        confirmLabel="删除"
        destructive
        onConfirm={confirmDelete}
      />

      <ExportFormulasDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        selectedIds={[...selectedIds]}
        totalCount={list.length}
        onPickPath={onPickExportPath}
        onSubmit={async (passphrase, ids, outPath) => {
          const count = await formulaApi.exportWorkspaceFormulas(
            ids,
            passphrase,
            outPath,
          );
          setExportOpen(false);
          alert(`已加密导出 ${count} 条配方到\n${outPath}`);
        }}
      />

      <ImportFormulasDialog
        open={importOpen}
        onClose={() => setImportOpen(false)}
        onPickPath={onPickImportPath}
        onSubmit={async (passphrase, inPath) => {
          const summary = await formulaApi.importWorkspaceFormulas(passphrase, inPath);
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
          <DialogTitle>加密导出工作区配方</DialogTitle>
          <DialogDescription>
            导出文件 (.ranpu) 用 AES-256-GCM 加密，需要导出口令才能在另一台机器导入。
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>范围</Label>
          <div className="flex flex-col gap-1 text-sm">
            <label className="flex items-center gap-2">
              <input
                type="radio"
                name="ws-export-scope"
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
                name="ws-export-scope"
                checked={scope === 'all'}
                onChange={() => setScope('all')}
              />
              当前工作区全部 {totalCount} 条
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
          <DialogTitle>加密导入到当前工作区</DialogTitle>
          <DialogDescription>
            选择 .ranpu 文件 + 导出时设置的口令；同内部色号的配方将被自动跳过。
            默认库导出的 .ranpu 也支持导入到工作区。
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

export default WorkspaceFormulasPage;
