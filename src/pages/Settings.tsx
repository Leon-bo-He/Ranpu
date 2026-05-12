import { useEffect, useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';
import { EditModeToggle } from '@/components/EditModeToggle';
import { PassphrasePromptDialog } from '@/components/PassphrasePromptDialog';
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
import { useSettingsStore, type IdleTimeoutSeconds } from '@/store/settings';
import { useResetOnLock } from '@/hooks/useResetOnLock';
import { useDyeLibraryStore } from '@/store/dyeLibrary';
import { useYarnSettingsStore } from '@/store/yarnSettings';

export function SettingsPage() {
  const idleSeconds = useSettingsStore((s) => s.idleTimeoutSeconds);
  const setIdleSeconds = useSettingsStore((s) => s.setIdleTimeoutSeconds);

  const [askResetMills, setAskResetMills] = useState(false);
  const [askResetSpecs, setAskResetSpecs] = useState(false);
  // 纱支 / 染料库的 "修改" 解锁状态; 默认 OFF, 防误改. 局部 state, 切页
  // 自动归位 — 防止用户离开后这个高权限模式还停在打开.
  const [yarnEditOn, setYarnEditOn] = useState(false);
  const [dyeEditOn, setDyeEditOn] = useState(false);

  const formulaEdit = useEditModeStore((s) => s.formulaEditEnabled);
  const enableFormula = useEditModeStore((s) => s.enableFormulaEdit);
  const disableFormula = useEditModeStore((s) => s.disableFormulaEdit);

  const workspaceEdit = useEditModeStore((s) => s.workspaceEditEnabled);
  const enableWorkspace = useEditModeStore((s) => s.enableWorkspaceEdit);
  const disableWorkspace = useEditModeStore((s) => s.disableWorkspaceEdit);

  const auditDisplay = useEditModeStore((s) => s.auditDisplayEnabled);
  const enableAudit = useEditModeStore((s) => s.enableAuditDisplay);
  const disableAudit = useEditModeStore((s) => s.disableAuditDisplay);

  const libraryTransfer = useEditModeStore((s) => s.libraryTransferEnabled);
  const enableLibraryTransfer = useEditModeStore((s) => s.enableLibraryTransfer);
  const disableLibraryTransfer = useEditModeStore((s) => s.disableLibraryTransfer);
  // 配方互导开启需要再次输入启动口令; 这里 toggle 的 onEnable 改成
  // 弹密码 dialog, 校验通过才真正打开开关.
  const [askLibraryTransferPwd, setAskLibraryTransferPwd] = useState(false);


  const yarnMills = useYarnSettingsStore((s) => s.mills);
  const setYarnMills = useYarnSettingsStore((s) => s.setMills);
  const resetYarnMills = useYarnSettingsStore((s) => s.resetMills);
  const yarnSpecs = useYarnSettingsStore((s) => s.specs);
  const setYarnSpecs = useYarnSettingsStore((s) => s.setSpecs);
  const resetYarnSpecs = useYarnSettingsStore((s) => s.resetSpecs);

  const dyes = useDyeLibraryStore((s) => s.dyes);
  const setDyes = useDyeLibraryStore((s) => s.setDyes);
  const resetDyes = useDyeLibraryStore((s) => s.resetDyes);
  const [askResetDyes, setAskResetDyes] = useState(false);

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

  // 锁屏触发时关掉所有还原确认 + 配方互导口令对话框, 不让 focus-scope
  // 卡 LockOverlay.
  useResetOnLock(() => {
    setAskResetMills(false);
    setAskResetSpecs(false);
    setAskResetDyes(false);
    setAskLibraryTransferPwd(false);
  });


  return (
    <div className="space-y-6 p-6">
      <h2 className="font-serif text-xl tracking-[2px]">设置</h2>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <CardTitle>纱支</CardTitle>
          <Button
            variant={yarnEditOn ? 'outline' : 'default'}
            size="sm"
            onClick={() => setYarnEditOn((v) => !v)}
          >
            {yarnEditOn ? '完成' : '修改'}
          </Button>
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
                disabled={!yarnEditOn}
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
                  disabled={!yarnEditOn}
                  title="还原默认: 博奥 / 弘曲 / 鸿泰 / 华盛 / 锦华 / 妙虎 / 名仁"
                >
                  还原默认
                </Button>
              </CardHeader>
              <CardContent className="grid gap-2">
                <StringListEditor
                  values={yarnMills}
                  onChange={setYarnMills}
                  newPlaceholder="新增厂名…"
                  readOnly={!yarnEditOn}
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
                  disabled={!yarnEditOn}
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
                  readOnly={!yarnEditOn}
                />
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <CardTitle>染料库</CardTitle>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setAskResetDyes(true)}
              disabled={!dyeEditOn}
              title="清空染料库"
            >
              清空
            </Button>
            <Button
              variant={dyeEditOn ? 'outline' : 'default'}
              size="sm"
              onClick={() => setDyeEditOn((v) => !v)}
            >
              {dyeEditOn ? '完成' : '修改'}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="grid gap-2">
          <StringListEditor
            values={dyes}
            onChange={setDyes}
            newPlaceholder="新增染料名…"
            cols={6}
            readOnly={!dyeEditOn}
          />
          <p className="text-xs text-muted-foreground">
            保存配方时若有不在库的染料名，会弹窗询问是否加入复用。
          </p>
        </CardContent>
      </Card>

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
          <EditModeToggle
            label="配方互导"
            whenOffCanStill=""
            enabled={libraryTransfer}
            onEnable={() => setAskLibraryTransferPwd(true)}
            onDisable={disableLibraryTransfer}
          />
          <p className="text-xs text-muted-foreground">
            「工作区管理」「审计日志显示」「配方互导」关闭时，对应入口在侧栏隐藏。
            重新开启即可看见。「配方互导」开启需再次输入启动口令。
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
            value={String(idleSeconds)}
            onValueChange={(v) => setIdleSeconds(Number(v) as IdleTimeoutSeconds)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="0">关闭自动锁屏</SelectItem>
              <SelectItem value="10">10 秒（测试）</SelectItem>
              <SelectItem value="300">5 分钟</SelectItem>
              <SelectItem value="600">10 分钟</SelectItem>
              <SelectItem value="1800">30 分钟</SelectItem>
              <SelectItem value="3600">60 分钟</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      {/* "重置数据库" 模块按用户要求暂时从设置页隐藏. 后端 cmd_reset_database
          / ResetDatabaseDialog / adminApi.resetDatabase 都保留, 后续需要时
          只在这里恢复 Card + Dialog mount 即可. */}

      <ConfirmDialog
        open={askResetMills}
        onClose={() => setAskResetMills(false)}
        title="还原厂名默认列表？"
        description="当前厂名将被清空，恢复成内置的 博奥 / 弘曲 / 鸿泰 / 华盛 / 锦华 / 妙虎 / 名仁。已经手动加的会丢失。"
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
      <ConfirmDialog
        open={askResetDyes}
        onClose={() => setAskResetDyes(false)}
        title="清空染料库？"
        description="所有手动加进来的染料名都会被清掉。下次保存配方有新名字时会重新弹窗询问。"
        confirmLabel="清空"
        destructive
        onConfirm={() => {
          resetDyes();
          setAskResetDyes(false);
        }}
      />
      <PassphrasePromptDialog
        open={askLibraryTransferPwd}
        onClose={() => setAskLibraryTransferPwd(false)}
        title="开启配方互导"
        description="此操作会在侧栏显示「配方互导」入口，需要再次输入启动口令。"
        onConfirmed={() => {
          setAskLibraryTransferPwd(false);
          enableLibraryTransfer();
        }}
      />
    </div>
  );
}

export default SettingsPage;
