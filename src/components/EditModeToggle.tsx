import { Lock, Unlock } from 'lucide-react';

import { Button } from '@/components/ui/button';

/// 页面顶端 "管理模式" 开关条. 文案自带 30min 自动关说明,
/// 调用方只负责传当前开关状态 + on/off callback.
interface EditModeToggleProps {
  /// 模式名: "配方管理" / "工作区管理" / "审计日志显示"
  label: string;
  /// 关闭时仍可做的事 (UI 文案), 例如 "计算 / 加入批次清单"
  /// 或 "" (留空时不显示).
  whenOffCanStill: string;
  enabled: boolean;
  onEnable: () => void;
  onDisable: () => void;
}

export function EditModeToggle({
  label,
  whenOffCanStill,
  enabled,
  onEnable,
  onDisable,
}: EditModeToggleProps) {
  return (
    <div className="flex flex-col gap-1 rounded-md border bg-muted/30 px-3 py-2">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-sm">
          {enabled ? (
            <Unlock className="h-4 w-4 text-emerald-600" />
          ) : (
            <Lock className="h-4 w-4 text-muted-foreground" />
          )}
          <span className="font-medium">
            {label}：
            <span className={enabled ? 'text-emerald-700' : 'text-muted-foreground'}>
              {enabled ? '已开启' : '已关闭'}
            </span>
          </span>
        </div>
        {enabled ? (
          <Button size="sm" variant="outline" onClick={onDisable}>
            关闭
          </Button>
        ) : (
          <Button size="sm" onClick={onEnable}>
            开启
          </Button>
        )}
      </div>
      <p className="text-xs leading-5 text-muted-foreground">
        开启后才能进行写操作
        {whenOffCanStill && <>；关闭时只可以{whenOffCanStill}</>}。
        开启后 30 分钟内无操作会自动关闭。
      </p>
    </div>
  );
}
