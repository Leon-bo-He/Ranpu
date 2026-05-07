import { useState, type FormEvent } from 'react';

import { identityApi } from '@/api/identity';
import { ApiError } from '@/api/invoke';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useSettingsStore, type IdleTimeoutMinutes } from '@/store/settings';

export function SettingsPage() {
  const idleMinutes = useSettingsStore((s) => s.idleTimeoutMinutes);
  const setIdleMinutes = useSettingsStore((s) => s.setIdleTimeoutMinutes);

  return (
    <div className="space-y-6 p-6">
      <h2 className="font-serif text-xl tracking-[2px]">设置</h2>

      <Card>
        <CardHeader>
          <CardTitle>自动锁屏</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-2 max-w-md">
          <Label>空闲多久自动锁定</Label>
          <Select
            value={String(idleMinutes)}
            onValueChange={(v) => setIdleMinutes(Number(v) as IdleTimeoutMinutes)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">关闭自动锁屏</SelectItem>
              <SelectItem value="5">5 分钟</SelectItem>
              <SelectItem value="10">10 分钟</SelectItem>
              <SelectItem value="30">30 分钟</SelectItem>
              <SelectItem value="60">60 分钟</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>修改密码</CardTitle>
        </CardHeader>
        <CardContent>
          <ChangePasswordForm />
        </CardContent>
      </Card>
    </div>
  );
}

function ChangePasswordForm() {
  const [oldPw, setOldPw] = useState('');
  const [newPw, setNewPw] = useState('');
  const [newPw2, setNewPw2] = useState('');
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setMsg(null);
    if (newPw !== newPw2) {
      setError('两次输入的新密码不一致');
      return;
    }
    setBusy(true);
    try {
      await identityApi.changePassword(oldPw, newPw);
      setMsg('密码已修改');
      setOldPw('');
      setNewPw('');
      setNewPw2('');
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="grid max-w-md gap-3">
      <div className="grid gap-1">
        <Label>当前密码</Label>
        <Input
          type="password"
          value={oldPw}
          onChange={(e) => setOldPw(e.target.value)}
          disabled={busy}
        />
      </div>
      <div className="grid gap-1">
        <Label>新密码（≥ 8 位）</Label>
        <Input
          type="password"
          value={newPw}
          onChange={(e) => setNewPw(e.target.value)}
          disabled={busy}
        />
      </div>
      <div className="grid gap-1">
        <Label>再次输入新密码</Label>
        <Input
          type="password"
          value={newPw2}
          onChange={(e) => setNewPw2(e.target.value)}
          disabled={busy}
        />
      </div>
      {msg && <p className="text-sm text-emerald-600">{msg}</p>}
      {error && <p className="text-sm text-destructive">{error}</p>}
      <Button type="submit" disabled={busy || !oldPw || !newPw}>
        {busy ? '正在修改…' : '修改密码'}
      </Button>
    </form>
  );
}

export default SettingsPage;
