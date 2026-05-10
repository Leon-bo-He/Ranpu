import { useEffect, useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';
import { EditModeToggle } from '@/components/EditModeToggle';
import { LanSyncCard } from '@/components/LanSyncCard';
import { StringListEditor } from '@/components/StringListEditor';
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
import { useEditModeStore } from '@/store/editMode';
import { useSettingsStore, type IdleTimeoutMinutes } from '@/store/settings';
import { useYarnSettingsStore } from '@/store/yarnSettings';

export function SettingsPage() {
  const idleMinutes = useSettingsStore((s) => s.idleTimeoutMinutes);
  const setIdleMinutes = useSettingsStore((s) => s.setIdleTimeoutMinutes);

  const [askResetMills, setAskResetMills] = useState(false);
  const [askResetSpecs, setAskResetSpecs] = useState(false);

  const formulaEdit = useEditModeStore((s) => s.formulaEditEnabled);
  const enableFormula = useEditModeStore((s) => s.enableFormulaEdit);
  const disableFormula = useEditModeStore((s) => s.disableFormulaEdit);

  const workspaceEdit = useEditModeStore((s) => s.workspaceEditEnabled);
  const enableWorkspace = useEditModeStore((s) => s.enableWorkspaceEdit);
  const disableWorkspace = useEditModeStore((s) => s.disableWorkspaceEdit);

  const auditDisplay = useEditModeStore((s) => s.auditDisplayEnabled);
  const enableAudit = useEditModeStore((s) => s.enableAuditDisplay);
  const disableAudit = useEditModeStore((s) => s.disableAuditDisplay);

  const yarnMills = useYarnSettingsStore((s) => s.mills);
  const setYarnMills = useYarnSettingsStore((s) => s.setMills);
  const resetYarnMills = useYarnSettingsStore((s) => s.resetMills);
  const yarnSpecs = useYarnSettingsStore((s) => s.specs);
  const setYarnSpecs = useYarnSettingsStore((s) => s.setSpecs);
  const resetYarnSpecs = useYarnSettingsStore((s) => s.resetSpecs);

  // 单个纱支重量 (kg). 用本地 state 暂存输入中态, 失焦再过 store 钳位
  // (拒非正数, 上限 999).
  const singleYarnWeight = useSettingsStore((s) => s.singleYarnWeightKg);
  const setSingleYarnWeight = useSettingsStore((s) => s.setSingleYarnWeightKg);
  const [singleYarnInput, setSingleYarnInput] = useState(String(singleYarnWeight));
  useEffect(() => {
    setSingleYarnInput(String(singleYarnWeight));
  }, [singleYarnWeight]);
  const commitSingleYarn = () => {
    const n = Number(singleYarnInput);
    if (Number.isFinite(n) && n > 0) {
      setSingleYarnWeight(n);
    } else {
      setSingleYarnInput(String(singleYarnWeight));
    }
  };

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

      <Card>
        <CardHeader>
          <CardTitle>纱支</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-2 max-w-md">
            <Label htmlFor="single-yarn-weight">单个纱支重量</Label>
            <div className="flex items-center gap-2">
              <Input
                id="single-yarn-weight"
                type="number"
                min={0.01}
                max={999}
                step={0.01}
                inputMode="decimal"
                value={singleYarnInput}
                onChange={(e) => setSingleYarnInput(e.target.value)}
                onBlur={commitSingleYarn}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                className="w-32"
              />
              <span className="text-sm text-muted-foreground">kg</span>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <Card className="min-w-0 w-full">
              <CardHeader className="flex flex-row items-center justify-between space-y-0">
                <CardTitle className="text-base">厂名</CardTitle>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setAskResetMills(true)}
                  title="还原默认: 博奥 / 名仁 / 妙虎 / 弘曲"
                >
                  还原默认
                </Button>
              </CardHeader>
              <CardContent className="grid gap-2">
                <StringListEditor
                  values={yarnMills}
                  onChange={setYarnMills}
                  newPlaceholder="新增厂名…"
                />
              </CardContent>
            </Card>
            <Card className="min-w-0 w-full">
              <CardHeader className="flex flex-row items-center justify-between space-y-0">
                <CardTitle className="text-base">规格</CardTitle>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setAskResetSpecs(true)}
                  title="还原默认: 20/2 至 60/3"
                >
                  还原默认
                </Button>
              </CardHeader>
              <CardContent className="grid gap-2">
                <StringListEditor
                  values={yarnSpecs}
                  onChange={setYarnSpecs}
                  newPlaceholder="新增规格 (例 32/2)…"
                />
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>

      <LanSyncCard />

      <ConfirmDialog
        open={askResetMills}
        onClose={() => setAskResetMills(false)}
        title="还原厂名默认列表？"
        description="当前厂名将被清空，恢复成内置的 博奥 / 名仁 / 妙虎 / 弘曲。已经手动加的会丢失。"
        confirmLabel="还原"
        destructive
        onConfirm={() => {
          resetYarnMills();
          setAskResetMills(false);
        }}
      />
      <ConfirmDialog
        open={askResetSpecs}
        onClose={() => setAskResetSpecs(false)}
        title="还原规格默认列表？"
        description="当前规格将被清空，恢复成内置的 20/2 至 60/3 共 10 条。已经手动加的会丢失。"
        confirmLabel="还原"
        destructive
        onConfirm={() => {
          resetYarnSpecs();
          setAskResetSpecs(false);
        }}
      />
    </div>
  );
}

export default SettingsPage;
