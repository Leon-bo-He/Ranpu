import { Copy, Pencil, Trash2 } from 'lucide-react';

import type { FormulaView } from '@/api/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { liquorRatioLabel, unitLabel } from '@/lib/format';

export interface FormulaCardActions {
  onCopyToWorkspace?: (formula: FormulaView) => void;
  onEdit?: (formula: FormulaView) => void;
  onDelete?: (formula: FormulaView) => void;
}

interface FormulaCardProps extends FormulaCardActions {
  formula: FormulaView;
  source: 'default' | 'workspace';
  /** 是否当前用户是 admin（控制编辑/删除按钮显示）。 */
  canManage: boolean;
  /** 当前是否选中了 workspace（控制「复制到当前工作区」是否禁用）。 */
  hasActiveWorkspace: boolean;
  /** 多选模式：传 onToggleSelected 即开启，左上角显示一个 checkbox。 */
  selected?: boolean;
  onToggleSelected?: (formula: FormulaView, next: boolean) => void;
}

export function FormulaCard({
  formula,
  source,
  canManage,
  hasActiveWorkspace,
  onCopyToWorkspace,
  onEdit,
  onDelete,
  selected,
  onToggleSelected,
}: FormulaCardProps) {
  const selectable = onToggleSelected !== undefined;
  return (
    <Card
      className={cn(
        'flex flex-col transition-colors',
        selected && 'ring-2 ring-primary',
      )}
    >
      <CardHeader className="space-y-1">
        <div className="flex items-start justify-between gap-2">
          <CardTitle className="flex flex-1 items-center gap-2">
            <span className="text-base font-bold">{formula.internal_color_code}</span>
            {/* 默认库的「客户色号」只是预设占位, 实际客户报色在工作区配方
                复制后才有意义, 所以仅在 workspace 卡片上展示这个 badge. */}
            {source === 'workspace' && formula.customer_color_code && (
              <Badge variant="secondary">
                客户色号：{formula.customer_color_code}
              </Badge>
            )}
          </CardTitle>
          {selectable && (
            <label
              className="flex cursor-pointer items-center"
              title="选择以批量复制"
            >
              <input
                type="checkbox"
                className="h-4 w-4 cursor-pointer accent-primary"
                checked={!!selected}
                onChange={(e) => onToggleSelected?.(formula, e.target.checked)}
              />
            </label>
          )}
        </div>
        {formula.color_name && (
          <CardDescription>{formula.color_name}</CardDescription>
        )}
      </CardHeader>
      <CardContent className="flex-1 space-y-2 text-sm">
        <ul className="space-y-1">
          {formula.items.map((it) => (
            <li
              key={`${it.dye_name}-${it.sort_order}`}
              className="flex justify-between gap-3"
            >
              <span className="truncate">
                {it.dye_name}
                {it.dye_code && (
                  <span className="ml-1 text-muted-foreground">({it.dye_code})</span>
                )}
              </span>
              <span className="whitespace-nowrap font-mono">
                {it.amount.toFixed(2)} {unitLabel(it.unit)}
              </span>
            </li>
          ))}
        </ul>
        {formula.liquor_ratio !== null && (
          <p className="text-xs text-muted-foreground">
            浴比 {liquorRatioLabel(formula.liquor_ratio)}
          </p>
        )}
        {formula.notes && (
          <p className="text-xs text-muted-foreground">{formula.notes}</p>
        )}
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        {source === 'default' && canManage && (
          <Button
            size="sm"
            variant="outline"
            disabled={!hasActiveWorkspace}
            title={hasActiveWorkspace ? '' : '请先选择工作区'}
            onClick={() => onCopyToWorkspace?.(formula)}
          >
            <Copy className="mr-1 h-4 w-4" /> 复制到当前工作区
          </Button>
        )}
        {canManage && onEdit && (
          <Button size="sm" variant="ghost" onClick={() => onEdit(formula)}>
            <Pencil className="mr-1 h-4 w-4" /> 编辑
          </Button>
        )}
        {canManage && onDelete && (
          <Button size="sm" variant="ghost" onClick={() => onDelete(formula)}>
            <Trash2 className="mr-1 h-4 w-4" /> 删除
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}
