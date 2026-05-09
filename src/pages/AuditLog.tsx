import { save } from '@tauri-apps/plugin-dialog';
import { Upload } from 'lucide-react';
import { useEffect, useState } from 'react';

import { auditApi } from '@/api/audit';
import { ApiError } from '@/api/invoke';
import type { AuditEventView } from '@/api/types';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { EditModeToggle } from '@/components/EditModeToggle';
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
import { formatDateTime } from '@/lib/format';
import { useEditModeStore } from '@/store/editMode';

export function AuditLogPage() {
  const displayEnabled = useEditModeStore((s) => s.auditDisplayEnabled);
  const enableDisplay = useEditModeStore((s) => s.enableAuditDisplay);
  const disableDisplay = useEditModeStore((s) => s.disableAuditDisplay);
  const touchDisplay = useEditModeStore((s) => s.touchAuditActivity);

  const [events, setEvents] = useState<AuditEventView[]>([]);
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [exportOpen, setExportOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = () => {
    // 前端只展示最新 50 条 (按 occurred_at DESC). 全量审计仍由 "导出" 走加密包.
    auditApi
      .list({
        from: from ? new Date(from).toISOString() : undefined,
        to: to ? new Date(to).toISOString() : undefined,
        limit: 50,
      })
      .then((rows) => {
        setEvents(rows);
        touchDisplay();
      })
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  };

  useEffect(() => {
    if (displayEnabled) load();
    else setEvents([]);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [displayEnabled]);

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h2 className="font-serif text-xl tracking-[2px]">审计日志</h2>
        {displayEnabled && (
          <Button onClick={() => setExportOpen(true)}>
            <Upload className="mr-1 h-4 w-4" /> 导出
          </Button>
        )}
      </div>

      <EditModeToggle
        label="审计日志显示"
        whenOffCanStill=""
        enabled={displayEnabled}
        onEnable={enableDisplay}
        onDisable={disableDisplay}
      />

      {!displayEnabled ? (
        <p className="text-sm text-muted-foreground">
          审计日志包含敏感操作记录, 默认隐藏. 点上方 "开启" 加载最新 50 条.
        </p>
      ) : (
        <>
          <div className="flex flex-wrap items-end gap-3">
            <div className="grid gap-1">
              <Label className="text-xs">起始日期</Label>
              <Input
                type="date"
                className="w-40"
                value={from}
                onChange={(e) => setFrom(e.target.value)}
              />
            </div>
            <div className="grid gap-1">
              <Label className="text-xs">截止日期</Label>
              <Input
                type="date"
                className="w-40"
                value={to}
                onChange={(e) => setTo(e.target.value)}
              />
            </div>
            <Button variant="outline" onClick={load}>
              筛选
            </Button>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>时间</TableHead>
                <TableHead>工作区</TableHead>
                <TableHead>动作</TableHead>
                <TableHead>对象</TableHead>
                <TableHead>详情</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {events.map((e) => (
                <TableRow key={e.id}>
                  <TableCell className="whitespace-nowrap">
                    {formatDateTime(e.occurred_at)}
                  </TableCell>
                  <TableCell>{e.workspace_context_id ?? '—'}</TableCell>
                  <TableCell className="font-mono text-xs">{e.action}</TableCell>
                  <TableCell className="max-w-[180px] truncate">
                    {e.target ?? '—'}
                  </TableCell>
                  <TableCell className="max-w-[280px] truncate text-xs text-muted-foreground">
                    {e.details ?? '—'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </>
      )}

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        defaultFrom={from}
        defaultTo={to}
      />
    </div>
  );
}

function ExportDialog({
  open,
  onClose,
  defaultFrom,
  defaultTo,
}: {
  open: boolean;
  onClose: () => void;
  defaultFrom: string;
  defaultTo: string;
}) {
  const [from, setFrom] = useState(defaultFrom);
  const [to, setTo] = useState(defaultTo);
  const [format, setFormat] = useState<'encrypted' | 'csv'>('encrypted');
  const [passphrase, setPassphrase] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [askPlainCsv, setAskPlainCsv] = useState(false);

  useEffect(() => {
    setFrom(defaultFrom);
    setTo(defaultTo);
  }, [defaultFrom, defaultTo, open]);

  const submit = async () => {
    setError(null);
    if (!from || !to) {
      setError('起止日期都必填');
      return;
    }
    if (format === 'encrypted' && passphrase.length < 8) {
      setError('加密导出需要至少 8 位口令');
      return;
    }
    if (format === 'csv') {
      // 把明文导出二次确认抬到 ConfirmDialog 而不是 window.confirm.
      setAskPlainCsv(true);
      return;
    }
    await doExport();
  };

  const doExport = async () => {
    const ext = format === 'csv' ? 'csv' : 'ranpu';
    const filterName = format === 'csv' ? 'CSV' : 'Ranpu 加密包';
    const out = await save({
      defaultPath: `审计日志-${from}-${to}.${ext}`,
      filters: [{ name: filterName, extensions: [ext] }],
    });
    if (!out) return;
    setBusy(true);
    try {
      await auditApi.export({
        from: new Date(from).toISOString(),
        to: new Date(to + 'T23:59:59').toISOString(),
        format,
        passphrase: format === 'encrypted' ? passphrase : undefined,
        out_path: out,
      });
      onClose();
      alert('已导出');
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const onConfirmPlainCsv = async () => {
    setAskPlainCsv(false);
    await doExport();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>导出审计日志</DialogTitle>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3">
          <div className="grid gap-1">
            <Label>起始日期</Label>
            <Input type="date" value={from} onChange={(e) => setFrom(e.target.value)} />
          </div>
          <div className="grid gap-1">
            <Label>截止日期</Label>
            <Input type="date" value={to} onChange={(e) => setTo(e.target.value)} />
          </div>
        </div>
        <div className="grid gap-1">
          <Label>导出格式</Label>
          <Select value={format} onValueChange={(v) => setFormat(v as 'encrypted' | 'csv')}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="encrypted">加密 .ranpu（推荐）</SelectItem>
              <SelectItem value="csv">明文 CSV</SelectItem>
            </SelectContent>
          </Select>
        </div>
        {format === 'encrypted' && (
          <div className="grid gap-1">
            <Label>导出口令（≥ 8 位，与登录密码独立）</Label>
            <Input
              type="password"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
            />
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy}>
            {busy ? '导出中…' : '导出'}
          </Button>
        </DialogFooter>
      </DialogContent>

      <ConfirmDialog
        open={askPlainCsv}
        onClose={() => setAskPlainCsv(false)}
        title="确认明文导出审计日志？"
        description={
          <>
            日志包含敏感操作记录（动作、时间、对象），明文 CSV{' '}
            <strong>不会被加密</strong>，任何拿到文件的人都能直接读取。如需对外分发，
            建议改用 「加密 .ranpu」 格式。
          </>
        }
        confirmLabel="仍然明文导出"
        destructive
        onConfirm={onConfirmPlainCsv}
      />
    </Dialog>
  );
}

export default AuditLogPage;
