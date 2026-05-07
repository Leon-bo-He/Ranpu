import { Pencil, Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ApiError } from '@/api/invoke';
import type { WorkspaceView } from '@/api/types';
import { workspaceApi } from '@/api/workspace';
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

export function WorkspaceManagementPage() {
  const session = useSessionStore((s) => s.session);
  const [list, setList] = useState<WorkspaceView[]>([]);
  const [editing, setEditing] = useState<WorkspaceView | 'new' | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = () =>
    workspaceApi
      .list()
      .then(setList)
      .catch((e) => setError(e instanceof ApiError ? e.message : String(e)));

  useEffect(() => {
    load();
  }, []);

  if (session?.role !== 'admin') {
    return (
      <p className="p-6 text-sm text-muted-foreground">只有管理员能管理工作区。</p>
    );
  }

  const onDelete = async (w: WorkspaceView) => {
    if (
      !confirm(
        `确认删除工作区「${w.name}」？该操作会同时删除其下所有配方与购物车条目。`,
      )
    )
      return;
    try {
      await workspaceApi.remove(w.id);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
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
          {list.map((w) => (
            <TableRow key={w.id}>
              <TableCell className="font-medium">{w.name}</TableCell>
              <TableCell>{w.description ?? '—'}</TableCell>
              <TableCell>{formatDateTime(w.created_at)}</TableCell>
              <TableCell className="flex gap-1">
                <Button size="sm" variant="ghost" onClick={() => setEditing(w)}>
                  <Pencil className="mr-1 h-4 w-4" /> 重命名
                </Button>
                <Button size="sm" variant="ghost" onClick={() => onDelete(w)}>
                  <Trash2 className="mr-1 h-4 w-4" /> 删除
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <WorkspaceEditor
        target={editing}
        onClose={() => setEditing(null)}
        onSaved={() => {
          setEditing(null);
          load();
        }}
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
