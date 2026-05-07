import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  CheckSquare,
  Loader2,
  PackageOpen,
  Square,
} from 'lucide-react';
import { useEffect, useState } from 'react';

import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import { workspaceApi } from '@/api/workspace';
import type {
  ExportLibraryArchiveView,
  ImportLibraryArchiveView,
  PreviewLibraryArchiveView,
  WorkspaceImportAction,
  WorkspaceImportPlanDto,
  WorkspaceView,
} from '@/api/types';
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
import { isAdmin, useSessionStore } from '@/store/session';

export function LibraryTransferPage() {
  const session = useSessionStore((s) => s.session);
  if (!session || !isAdmin(session)) {
    return (
      <p className="p-6 text-sm text-muted-foreground">
        只有管理员可以使用配方互导。
      </p>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center gap-2">
        <PackageOpen className="h-5 w-5" />
        <h2 className="font-serif text-xl tracking-[2px]">配方互导</h2>
      </div>
      <p className="text-sm text-muted-foreground">
        将默认配方库与一个或多个工作区一次性加密导出为 .ranpu
        文件，或在另一台机器导入。工作区按名称匹配 (找不到则新建，已存在可选合并或跳过)。
      </p>
      <ExportSection />
      <ImportSection />
    </div>
  );
}

// ---------- 导出 ----------

function ExportSection() {
  const [workspaces, setWorkspaces] = useState<WorkspaceView[]>([]);
  const [includeDefault, setIncludeDefault] = useState(true);
  const [selectedWsIds, setSelectedWsIds] = useState<Set<number>>(new Set());
  const [passphrase, setPassphrase] = useState('');
  const [passphrase2, setPassphrase2] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [done, setDone] = useState<{
    summary: ExportLibraryArchiveView;
    path: string;
  } | null>(null);

  useEffect(() => {
    workspaceApi
      .list()
      .then(setWorkspaces)
      .catch((e) => setErr(e instanceof ApiError ? e.message : String(e)));
  }, []);

  const toggleWs = (id: number) => {
    setSelectedWsIds((prev) => {
      const out = new Set(prev);
      if (out.has(id)) out.delete(id);
      else out.add(id);
      return out;
    });
  };

  const allSelected =
    workspaces.length > 0 && workspaces.every((w) => selectedWsIds.has(w.id));
  const toggleAll = () => {
    if (allSelected) setSelectedWsIds(new Set());
    else setSelectedWsIds(new Set(workspaces.map((w) => w.id)));
  };

  const onExport = async () => {
    setErr(null);
    setDone(null);
    if (!includeDefault && selectedWsIds.size === 0) {
      setErr('至少勾选默认库或一个工作区');
      return;
    }
    if (passphrase.length < 8) {
      setErr('导出口令至少 8 位');
      return;
    }
    if (passphrase !== passphrase2) {
      setErr('两次输入的口令不一致');
      return;
    }
    const out = await saveDialog({
      defaultPath: `配方库-${new Date().toISOString().slice(0, 10)}.ranpu`,
      filters: [{ name: 'Ranpu 加密包', extensions: ['ranpu'] }],
    });
    if (typeof out !== 'string') return;

    setBusy(true);
    try {
      const summary = await formulaApi.exportLibraryArchive({
        includeDefault,
        workspaceIds: [...selectedWsIds],
        passphrase,
        outPath: out,
      });
      setDone({ summary, path: out });
      setPassphrase('');
      setPassphrase2('');
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <ArrowUpFromLine className="h-4 w-4" /> 加密导出
        </CardTitle>
        <CardDescription>
          打包默认库 + 选中的工作区，AES-256-GCM 加密为 .ranpu 文件。
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={includeDefault}
              onChange={(e) => setIncludeDefault(e.target.checked)}
              disabled={busy}
            />
            包含默认配方库
          </label>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-sm">工作区</Label>
            <Button
              size="sm"
              variant="outline"
              onClick={toggleAll}
              disabled={busy || workspaces.length === 0}
            >
              {allSelected ? (
                <CheckSquare className="mr-1 h-4 w-4" />
              ) : (
                <Square className="mr-1 h-4 w-4" />
              )}
              {allSelected ? '取消全选' : '全选'}
            </Button>
          </div>
          {workspaces.length === 0 ? (
            <p className="text-xs text-muted-foreground">尚未创建任何工作区。</p>
          ) : (
            <div className="grid gap-1 max-h-60 overflow-auto rounded-md border p-2">
              {workspaces.map((w) => (
                <label
                  key={w.id}
                  className="flex items-center gap-2 rounded-sm px-1 py-0.5 text-sm hover:bg-accent/50"
                >
                  <input
                    type="checkbox"
                    checked={selectedWsIds.has(w.id)}
                    onChange={() => toggleWs(w.id)}
                    disabled={busy}
                  />
                  <span className="font-medium">{w.name}</span>
                  {w.description && (
                    <span className="text-xs text-muted-foreground">
                      — {w.description}
                    </span>
                  )}
                </label>
              ))}
            </div>
          )}
        </div>

        <div className="grid gap-2 md:grid-cols-2">
          <div className="grid gap-1">
            <Label className="text-sm">导出口令（≥ 8 位）</Label>
            <Input
              type="password"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              disabled={busy}
            />
          </div>
          <div className="grid gap-1">
            <Label className="text-sm">再次输入口令</Label>
            <Input
              type="password"
              value={passphrase2}
              onChange={(e) => setPassphrase2(e.target.value)}
              disabled={busy}
            />
          </div>
        </div>

        {err && <p className="text-sm text-destructive">{err}</p>}
        {done && (
          <div className="rounded-md border border-emerald-300 bg-emerald-50 p-3 text-sm text-emerald-900">
            已导出到 <span className="font-mono">{done.path}</span>。
            包含默认配方 <Badge variant="secondary">{done.summary.default_count}</Badge>{' '}
            条，工作区{' '}
            <Badge variant="secondary">{done.summary.workspace_count}</Badge>{' '}
            个，工作区配方{' '}
            <Badge variant="secondary">{done.summary.workspace_formula_count}</Badge>{' '}
            条。
          </div>
        )}

        <div className="flex justify-end">
          <Button onClick={onExport} disabled={busy}>
            {busy ? (
              <>
                <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                导出中…
              </>
            ) : (
              '选路径并导出'
            )}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

// ---------- 导入 ----------

interface PreviewState {
  inPath: string;
  passphrase: string;
  preview: PreviewLibraryArchiveView;
  /** 用户为每个工作区选择的动作；key = name */
  plans: Record<string, WorkspaceImportAction>;
  /** 是否导入默认库 (preview.has_default 为 true 时可勾) */
  includeDefault: boolean;
}

function ImportSection() {
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [state, setState] = useState<PreviewState | null>(null);
  const [result, setResult] = useState<ImportLibraryArchiveView | null>(null);

  const onPickAndPreview = async () => {
    setErr(null);
    setResult(null);
    if (passphrase.length === 0) {
      setErr('请输入导出时使用的口令');
      return;
    }
    const inPath = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: 'Ranpu 加密包', extensions: ['ranpu'] }],
    });
    if (typeof inPath !== 'string') return;

    setBusy(true);
    try {
      const preview = await formulaApi.previewLibraryArchive(passphrase, inPath);
      const initialPlans: Record<string, WorkspaceImportAction> = {};
      for (const w of preview.workspaces) {
        initialPlans[w.name] = w.already_exists ? 'merge' : 'create_new';
      }
      setState({
        inPath,
        passphrase,
        preview,
        plans: initialPlans,
        includeDefault: preview.has_default,
      });
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const setPlan = (name: string, action: WorkspaceImportAction) => {
    setState((prev) =>
      prev ? { ...prev, plans: { ...prev.plans, [name]: action } } : prev,
    );
  };

  const onConfirmImport = async () => {
    if (!state) return;
    setErr(null);
    setBusy(true);
    try {
      const workspacePlans: WorkspaceImportPlanDto[] = state.preview.workspaces.map(
        (w) => ({ name: w.name, action: state.plans[w.name] ?? 'skip' }),
      );
      const summary = await formulaApi.importLibraryArchive({
        passphrase: state.passphrase,
        inPath: state.inPath,
        includeDefault: state.includeDefault,
        workspacePlans,
      });
      setResult(summary);
      setState(null);
      setPassphrase('');
    } catch (e) {
      setErr(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <ArrowDownToLine className="h-4 w-4" /> 加密导入
        </CardTitle>
        <CardDescription>
          选择 .ranpu
          文件，按工作区名称匹配。已存在的工作区可选合并 / 跳过；不存在的将新建。
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {!state && (
          <>
            <div className="grid gap-1 max-w-md">
              <Label className="text-sm">解密口令</Label>
              <Input
                type="password"
                value={passphrase}
                onChange={(e) => setPassphrase(e.target.value)}
                disabled={busy}
              />
            </div>
            <div className="flex justify-end">
              <Button onClick={onPickAndPreview} disabled={busy}>
                {busy ? (
                  <>
                    <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                    解密中…
                  </>
                ) : (
                  '选文件并预览'
                )}
              </Button>
            </div>
          </>
        )}

        {state && (
          <div className="space-y-3">
            <p className="text-xs text-muted-foreground">
              文件: <span className="font-mono">{state.inPath}</span>
              {' · '}
              导出时间: {state.preview.exported_at}
            </p>

            <div className="rounded-md border p-3 space-y-2">
              <p className="text-sm font-medium">默认配方库</p>
              {state.preview.has_default ? (
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={state.includeDefault}
                    onChange={(e) =>
                      setState((prev) =>
                        prev ? { ...prev, includeDefault: e.target.checked } : prev,
                      )
                    }
                    disabled={busy}
                  />
                  导入默认库（{state.preview.default_count} 条；同内部色号自动跳过）
                </label>
              ) : (
                <p className="text-xs text-muted-foreground">
                  归档中没有默认配方。
                </p>
              )}
            </div>

            <div className="rounded-md border p-3 space-y-2">
              <p className="text-sm font-medium">工作区</p>
              {state.preview.workspaces.length === 0 ? (
                <p className="text-xs text-muted-foreground">
                  归档中没有工作区。
                </p>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>名称</TableHead>
                      <TableHead>状态</TableHead>
                      <TableHead>配方数</TableHead>
                      <TableHead>动作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {state.preview.workspaces.map((w) => (
                      <TableRow key={w.name}>
                        <TableCell className="font-medium">
                          {w.name}
                          {w.description && (
                            <div className="text-xs text-muted-foreground">
                              {w.description}
                            </div>
                          )}
                        </TableCell>
                        <TableCell>
                          {w.already_exists ? (
                            <Badge variant="secondary">已存在</Badge>
                          ) : (
                            <Badge variant="default">新建</Badge>
                          )}
                        </TableCell>
                        <TableCell>{w.formula_count}</TableCell>
                        <TableCell>
                          <PlanSelect
                            value={state.plans[w.name] ?? 'skip'}
                            alreadyExists={w.already_exists}
                            disabled={busy}
                            onChange={(v) => setPlan(w.name, v)}
                          />
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </div>

            {err && <p className="text-sm text-destructive">{err}</p>}

            <div className="flex justify-end gap-2">
              <Button
                variant="ghost"
                onClick={() => {
                  setState(null);
                  setErr(null);
                }}
                disabled={busy}
              >
                取消
              </Button>
              <Button onClick={onConfirmImport} disabled={busy}>
                {busy ? (
                  <>
                    <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                    导入中…
                  </>
                ) : (
                  '确认导入'
                )}
              </Button>
            </div>
          </div>
        )}

        {!state && err && <p className="text-sm text-destructive">{err}</p>}

        {result && <ImportResult result={result} />}
      </CardContent>
    </Card>
  );
}

function PlanSelect({
  value,
  alreadyExists,
  disabled,
  onChange,
}: {
  value: WorkspaceImportAction;
  alreadyExists: boolean;
  disabled: boolean;
  onChange: (v: WorkspaceImportAction) => void;
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value as WorkspaceImportAction)}
      disabled={disabled}
      className="rounded-md border bg-background px-2 py-1 text-sm"
    >
      <option value="skip">跳过</option>
      {alreadyExists ? (
        <option value="merge">合并到已存在</option>
      ) : (
        <option value="create_new">新建工作区</option>
      )}
    </select>
  );
}

function ImportResult({ result }: { result: ImportLibraryArchiveView }) {
  return (
    <div className="space-y-3 rounded-md border border-emerald-300 bg-emerald-50/40 p-3">
      <p className="text-sm font-medium text-emerald-900">导入完成</p>
      {result.default_summary && (
        <div className="text-sm">
          默认库：导入{' '}
          <Badge variant="default">{result.default_summary.imported}</Badge> 条，
          跳过{' '}
          <Badge variant="secondary">{result.default_summary.skipped}</Badge> 条，
          失败{' '}
          <Badge variant="destructive">{result.default_summary.failed}</Badge> 条。
        </div>
      )}
      {result.workspace_summaries.length > 0 && (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>工作区</TableHead>
              <TableHead>结果</TableHead>
              <TableHead>导入</TableHead>
              <TableHead>跳过</TableHead>
              <TableHead>失败</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {result.workspace_summaries.map((w) => (
              <TableRow key={w.name}>
                <TableCell className="font-medium">{w.name}</TableCell>
                <TableCell>
                  {w.action === 'created' && <Badge variant="default">新建</Badge>}
                  {w.action === 'merged' && <Badge variant="secondary">合并</Badge>}
                  {w.action === 'skipped' && <Badge variant="outline">跳过</Badge>}
                </TableCell>
                <TableCell>{w.summary.imported}</TableCell>
                <TableCell>{w.summary.skipped}</TableCell>
                <TableCell className="text-destructive">{w.summary.failed}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  );
}

export default LibraryTransferPage;
