import { X } from 'lucide-react';
import { useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

interface StringListEditorProps {
  values: string[];
  onChange: (next: string[]) => void;
  /// 新建条目输入框的 placeholder.
  newPlaceholder?: string;
  /// 已有条目的 input 占位符, 用于 a11y / 视觉提示.
  itemPlaceholder?: string;
  /// md 屏及以上一排几列, 默认 3. 移动端固定 2 列.
  cols?: number;
  /// 只读模式: 已有条目 input + 删除 × + 新增 input + 添加按钮全部 disable.
  /// 设置页用这个让用户先点 "修改" 解锁才能动数据.
  readOnly?: boolean;
}

/// 通用字符串列表编辑器: 每条一行 (input + 删除 ×), 末尾一行 input + "添加".
/// - 编辑: 已有 input 失焦或回车提交; 改成空串视为 "想删除", 弹确认.
/// - 删除: 点 × 也弹确认; 用户确认后才真删. 误点不会丢数据.
/// - 添加: 末尾 input 写入后失焦 / 回车 / 点 "添加" 都生效.
/// - 去重: 同名 (trim 后) 已存在则忽略, 避免重复.
export function StringListEditor({
  values,
  onChange,
  newPlaceholder = '新增…',
  itemPlaceholder,
  cols = 3,
  readOnly = false,
}: StringListEditorProps) {
  const [draft, setDraft] = useState('');
  // 待删除的条目 idx; null 时 dialog 不显示.
  const [pendingDelete, setPendingDelete] = useState<number | null>(null);

  const commitEdit = (idx: number, raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed) {
      // 改空 = 想删. 弹确认; 用户取消时 input 通过 key 刷回原值.
      setPendingDelete(idx);
      return;
    }
    if (trimmed === values[idx]) return;
    // 去重: 别的位置已有同名 → 忽略, input 通过 key 重置回原值.
    if (values.some((v, i) => i !== idx && v === trimmed)) return;
    const next = [...values];
    next[idx] = trimmed;
    onChange(next);
  };

  const askRemove = (idx: number) => setPendingDelete(idx);

  const confirmRemove = () => {
    if (pendingDelete === null) return;
    onChange(values.filter((_, i) => i !== pendingDelete));
    setPendingDelete(null);
  };

  const onAdd = () => {
    const trimmed = draft.trim();
    if (!trimmed || values.includes(trimmed)) {
      setDraft('');
      return;
    }
    onChange([...values, trimmed]);
    setDraft('');
  };

  // md 屏及以上的列数. 枚举写法 (不是 `md:grid-cols-${n}`) 让 Tailwind purge
  // 留下这些 class. 目前只用到 3 (纱支厂名/规格) 和 6 (染料库).
  const mdGridClass = cols === 6 ? 'md:grid-cols-6' : 'md:grid-cols-3';
  return (
    <div className="space-y-2">
      <div className={`grid grid-cols-2 gap-2 ${mdGridClass}`}>
        {values.map((v, i) => (
          // key 包含 v: 列表 onChange 后该行的 defaultValue 跟着刷新.
          <div key={`${i}-${v}`} className="flex items-center gap-1">
            <Input
              defaultValue={v}
              placeholder={itemPlaceholder}
              onBlur={(e) => commitEdit(i, e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  (e.target as HTMLInputElement).blur();
                }
              }}
              disabled={readOnly}
              className="h-9 min-w-0"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              onClick={() => askRemove(i)}
              aria-label="删除"
              disabled={readOnly}
              className="h-8 w-8 shrink-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
      {!readOnly && (
        <div className="flex items-center gap-2 pt-1">
          <Input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                onAdd();
              }
            }}
            placeholder={newPlaceholder}
            className="h-9 max-w-xs"
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onAdd}
            disabled={!draft.trim()}
            className="shrink-0"
          >
            添加
          </Button>
        </div>
      )}

      <ConfirmDialog
        open={pendingDelete !== null}
        onClose={() => setPendingDelete(null)}
        title="确认删除？"
        description={
          pendingDelete !== null
            ? `将删除「${values[pendingDelete] ?? ''}」。`
            : ''
        }
        confirmLabel="删除"
        destructive
        onConfirm={confirmRemove}
      />
    </div>
  );
}
