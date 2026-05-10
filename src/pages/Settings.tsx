import { EditModeToggle } from '@/components/EditModeToggle';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useEditModeStore } from '@/store/editMode';
import { useSettingsStore, type IdleTimeoutMinutes } from '@/store/settings';

export function SettingsPage() {
  const idleMinutes = useSettingsStore((s) => s.idleTimeoutMinutes);
  const setIdleMinutes = useSettingsStore((s) => s.setIdleTimeoutMinutes);

  const formulaEdit = useEditModeStore((s) => s.formulaEditEnabled);
  const enableFormula = useEditModeStore((s) => s.enableFormulaEdit);
  const disableFormula = useEditModeStore((s) => s.disableFormulaEdit);

  const workspaceEdit = useEditModeStore((s) => s.workspaceEditEnabled);
  const enableWorkspace = useEditModeStore((s) => s.enableWorkspaceEdit);
  const disableWorkspace = useEditModeStore((s) => s.disableWorkspaceEdit);

  const auditDisplay = useEditModeStore((s) => s.auditDisplayEnabled);
  const enableAudit = useEditModeStore((s) => s.enableAuditDisplay);
  const disableAudit = useEditModeStore((s) => s.disableAuditDisplay);

  return (
    <div className="space-y-6 p-6">
      <h2 className="font-serif text-xl tracking-[2px]">设置</h2>

      <Card>
        <CardHeader>
          <CardTitle>管理模式</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <EditModeToggle
            label="配方管理"
            whenOffCanStill="计算配方 / 加入批次清单"
            enabled={formulaEdit}
            onEnable={enableFormula}
            onDisable={disableFormula}
          />
          <EditModeToggle
            label="工作区管理"
            whenOffCanStill="切换工作区 / 浏览"
            enabled={workspaceEdit}
            onEnable={enableWorkspace}
            onDisable={disableWorkspace}
          />
          <EditModeToggle
            label="审计日志显示"
            whenOffCanStill=""
            enabled={auditDisplay}
            onEnable={enableAudit}
            onDisable={disableAudit}
          />
          <p className="text-xs text-muted-foreground">
            「工作区管理」与「审计日志显示」关闭时，对应入口在侧栏隐藏。
            重新开启即可看见。
          </p>
        </CardContent>
      </Card>

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
