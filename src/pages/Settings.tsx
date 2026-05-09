import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
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
    </div>
  );
}

export default SettingsPage;
