import { Lock, Pencil, Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ApiError } from '@/api/invoke';
import type { WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
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
import { Textarea } from '@/components/ui/textarea';
import { formatDateTime } from '@/lib/format';
import { useSessionStore } from '@/store/session';
import { useWorkspacesStore } from '@/store/workspaces';

export function WorkspaceManagementPage() {
  const session = useSessionStore((s) => s.session);
  const list = useWorkspacesStore((s) => s.list);
  const refresh = useWorkspacesStore((s) => s.refresh);
  const [editing, setEditing] = useState<WorkspaceView | 'new' | null>(null);
  const [pendingDelete, setPendingDelete] = useState<WorkspaceView | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    refresh().catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  }, [refresh]);

  if (session?.role !== 'admin') {
    return (
      <p className="p-6 text-sm text-muted-foreground">只有管理员能管理工作区。</p>
    );
  }

  const askDelete = (w: WorkspaceView) => setPendingDelete(w);

  const confirmDelete = async () => {
    if (!pendingDelete) return;
    try {
      await workspaceApi.remove(pendingDelete.id);
      setPendingDelete(null);
      await refresh();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
      setPendingDelete(null);
    }
  };

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h2 className="font-serif text-xl tracking-[2px]">工作区管理</h2>
        <Button onClick={() => setEditing('new')}>
          <Plus className="mr-1 h-4 w-4" /> 新建工作区
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>名称</TableHead>
            <TableHead>说明</TableHead>
            <TableHead>创建时间</TableHead>
            <TableHead>操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {list.map((w) => {
            const isSystem = w.kind === 'system_mirror';
            return (
              <TableRow key={w.id}>
                <TableCell className="font-medium">
                  <span className="inline-flex items-center gap-2">
                    {w.name}
                    {isSystem && (
                      <Badge variant="secondary" className="gap-1">
                        <Lock className="h-3 w-3" /> 系统内置
                      </Badge>
                    )}
                  </span>
                </TableCell>
                <TableCell>{w.description ?? '—'}</TableCell>
                <TableCell>{formatDateTime(w.created_at)}</TableCell>
                <TableCell className="flex gap-1">
                  {!isSystem && (
                    <>
                      <Button size="sm" variant="ghost" onClick={() => setEditing(w)}>
                        <Pencil className="mr-1 h-4 w-4" /> 重命名
                      </Button>
                      <Button size="sm" variant="ghost" onClick={() => askDelete(w)}>
                        <Trash2 className="mr-1 h-4 w-4" /> 删除
                      </Button>
                    </>
                  )}
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>

      <WorkspaceEditor
        target={editing}
        onClose={() => setEditing(null)}
        onSaved={() => {
          setEditing(null);
          refresh();
        }}
      />

      <ConfirmDialog
        open={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title="确认删除工作区？"
        description={
          pendingDelete && (
            <>
              将永久删除工作区{' '}
              <span className="font-mono">{pendingDelete.name}</span>{' '}
              及其下<strong>所有配方</strong>与<strong>批次清单记录</strong>，
              操作不可撤销。审计日志会保留。
            </>
          )
        }
        confirmLabel="删除工作区"
        destructive
        onConfirm={confirmDelete}
      />
    </div>
  );
}

function WorkspaceEditor({
  target,
  onClose,
  onSaved,
}: {
  target: WorkspaceView | 'new' | null;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (target === 'new') {
      setName('');
      setDescription('');
    } else if (target) {
      setName(target.name);
      setDescription(target.description ?? '');
    }
    setError(null);
  }, [target]);

  if (!target) return null;

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      if (target === 'new') {
        await workspaceApi.create(name, description);
      } else {
        await workspaceApi.rename(target.id, name);
      }
      onSaved();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{target === 'new' ? '新建工作区' : '重命名工作区'}</DialogTitle>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>名称</Label>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        {target === 'new' && (
          <div className="grid gap-2">
            <Label>说明（选填）</Label>
            <Textarea
              rows={2}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy || !name.trim()}>
            {busy ? '保存中…' : '保存'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default WorkspaceManagementPage;
