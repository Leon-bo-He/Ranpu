import { Lock, Pencil, Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ApiError } from '@/api/invoke';
import type { WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { EditModeToggle } from '@/components/EditModeToggle';
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
import { useEditModeStore } from '@/store/editMode';
import { useWorkspacesStore } from '@/store/workspaces';

export function WorkspaceManagementPage() {
  const list = useWorkspacesStore((s) => s.list);
  const refresh = useWorkspacesStore((s) => s.refresh);
  const editEnabled = useEditModeStore((s) => s.workspaceEditEnabled);
  const enableEdit = useEditModeStore((s) => s.enableWorkspaceEdit);
  const disableEdit = useEditModeStore((s) => s.disableWorkspaceEdit);
  const touchEdit = useEditModeStore((s) => s.touchWorkspaceActivity);
  const [editing, setEditing] = useState<WorkspaceView | 'new' | null>(null);
  const [pendingDelete, setPendingDelete] = useState<WorkspaceView | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    refresh().catch((e) => setError(e instanceof ApiError ? e.message : String(e)));
  }, [refresh]);

  const askDelete = (w: WorkspaceView) => setPendingDelete(w);

  const confirmDelete = async () => {
    if (!pendingDelete) return;
    try {
      await workspaceApi.remove(pendingDelete.id);
      touchEdit();
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
        {editEnabled && (
          <Button onClick={() => setEditing('new')}>
            <Plus className="mr-1 h-4 w-4" /> 新建工作区
          </Button>
        )}
      </div>

      <EditModeToggle
        label="工作区管理"
        whenOffCanStill="切换工作区 / 浏览"
        enabled={editEnabled}
        onEnable={enableEdit}
        onDisable={disableEdit}
      />

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
                  {!isSystem && editEnabled && (
                    <>
                      <Button size="sm" variant="ghost" onClick={() => setEditing(w)}>
                        <Pencil className="mr-1 h-4 w-4" /> 编辑
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
          touchEdit();
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
        // 编辑: 名字 / 说明 各只在变化时调一次, 避免无意义的审计 + 写库.
        const trimmedName = name.trim();
        const trimmedDesc = description.trim();
        const prevDesc = target.description ?? '';
        if (trimmedName.length > 0 && trimmedName !== target.name) {
          await workspaceApi.rename(target.id, trimmedName);
        }
        if (trimmedDesc !== prevDesc) {
          await workspaceApi.updateDescription(
            target.id,
            trimmedDesc.length === 0 ? null : trimmedDesc,
          );
        }
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
          <DialogTitle>{target === 'new' ? '新建工作区' : '编辑工作区'}</DialogTitle>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>名称</Label>
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="grid gap-2">
          <Label>说明（选填）</Label>
          <Textarea
            rows={2}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>
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
