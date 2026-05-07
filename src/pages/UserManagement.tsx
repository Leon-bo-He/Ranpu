import { useEffect, useState } from 'react';
import { UserPlus } from 'lucide-react';

import { identityApi } from '@/api/identity';
import { ApiError } from '@/api/invoke';
import type { UserView } from '@/api/types';
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
import { useSessionStore } from '@/store/session';

export function UserManagementPage() {
  const session = useSessionStore((s) => s.session);
  const [users, setUsers] = useState<UserView[]>([]);
  const [openCreate, setOpenCreate] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = () => identityApi.listUsers().then(setUsers).catch((e) => setError(String(e)));

  useEffect(() => {
    load();
  }, []);

  const onDeactivate = async (id: number) => {
    if (!confirm('确认停用该用户？')) return;
    try {
      await identityApi.deactivateUser(id);
      await load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  if (session?.role !== 'admin') {
    return <p className="p-6 text-sm text-muted-foreground">只有管理员可以管理用户。</p>;
  }

  return (
    <div className="space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h2 className="font-serif text-xl tracking-[2px]">用户管理</h2>
        <Button onClick={() => setOpenCreate(true)}>
          <UserPlus className="mr-1 h-4 w-4" /> 新建用户
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>用户名</TableHead>
            <TableHead>角色</TableHead>
            <TableHead>状态</TableHead>
            <TableHead>失败次数</TableHead>
            <TableHead>上次登录</TableHead>
            <TableHead>创建时间</TableHead>
            <TableHead>操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {users.map((u) => (
            <TableRow key={u.id}>
              <TableCell className="font-medium">{u.username}</TableCell>
              <TableCell>
                <Badge variant={u.role === 'admin' ? 'default' : 'secondary'}>
                  {u.role === 'admin' ? '管理员' : '普通用户'}
                </Badge>
              </TableCell>
              <TableCell>
                {u.is_active ? (
                  <span className="text-sm">启用</span>
                ) : (
                  <span className="text-sm text-muted-foreground">已停用</span>
                )}
              </TableCell>
              <TableCell>{u.failed_attempts}</TableCell>
              <TableCell>{formatDateTime(u.last_login)}</TableCell>
              <TableCell>{formatDateTime(u.created_at)}</TableCell>
              <TableCell>
                {u.is_active && u.id !== session.user_id && (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => onDeactivate(u.id)}
                  >
                    停用
                  </Button>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <CreateUserDialog
        open={openCreate}
        onClose={() => setOpenCreate(false)}
        onCreated={() => {
          setOpenCreate(false);
          load();
        }}
      />
    </div>
  );
}

function CreateUserDialog({
  open,
  onClose,
  onCreated,
}: {
  open: boolean;
  onClose: () => void;
  onCreated: () => void;
}) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [role, setRole] = useState<'admin' | 'user'>('user');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      await identityApi.createUser(username, password, role);
      setUsername('');
      setPassword('');
      onCreated();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>新建用户</DialogTitle>
        </DialogHeader>
        <div className="grid gap-2">
          <Label>用户名</Label>
          <Input value={username} onChange={(e) => setUsername(e.target.value)} />
        </div>
        <div className="grid gap-2">
          <Label>初始密码（≥ 8 位）</Label>
          <Input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <div className="grid gap-2">
          <Label>角色</Label>
          <Select value={role} onValueChange={(v) => setRole(v as 'admin' | 'user')}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="user">普通用户</SelectItem>
              <SelectItem value="admin">管理员</SelectItem>
            </SelectContent>
          </Select>
        </div>
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            取消
          </Button>
          <Button onClick={submit} disabled={busy || !username || !password}>
            {busy ? '正在创建…' : '创建'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default UserManagementPage;
