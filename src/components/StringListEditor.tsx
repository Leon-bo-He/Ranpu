import { X } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

interface StringListEditorProps {
  values: string[];
  onChange: (next: string[]) => void;
  /// 新建条目输入框的 placeholder.
  newPlaceholder?: string;
  /// 已有条目的 input 占位符, 用于 a11y / 视觉提示.
  itemPlaceholder?: string;
}

/// 通用字符串列表编辑器: 每条一行 (input + 删除 ×), 末尾一行 input + "添加".
/// - 编辑: 已有 input 失焦或回车提交; 改成空串则删除该条.
/// - 添加: 末尾 input 写入后失焦 / 回车 / 点 "添加" 都生效.
/// - 去重: 同名 (trim 后) 已存在则忽略, 避免重复.
export function StringListEditor({
  values,
  onChange,
  newPlaceholder = '新增…',
  itemPlaceholder,
}: StringListEditorProps) {
  const [draft, setDraft] = useState('');

  const commitEdit = (idx: number, raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed) {
      onChange(values.filter((_, i) => i !== idx));
      return;
    }
    if (trimmed === values[idx]) return;
    // 去重: 别的位置已有同名 → 忽略, input 通过 key 重置回原值.
    if (values.some((v, i) => i !== idx && v === trimmed)) return;
    const next = [...values];
    next[idx] = trimmed;
    onChange(next);
  };

  const onRemove = (idx: number) => {
    onChange(values.filter((_, i) => i !== idx));
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

  return (
    <div className="space-y-2">
      <div className="space-y-1">
        {values.map((v, i) => (
          // key 包含 v 让 onChange 把列表替换后, 对应行的 defaultValue 跟着刷.
          <div key={`${i}-${v}`} className="flex items-center gap-2">
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
              className="h-9"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              onClick={() => onRemove(i)}
              aria-label="删除"
              className="shrink-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
      <div className="flex items-center gap-2">
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
          className="h-9"
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
    </div>
  );
}
