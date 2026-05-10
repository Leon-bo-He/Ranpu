import { useEffect, useMemo, useRef, useState } from 'react';

import { Input } from '@/components/ui/input';
import { matchOption } from '@/lib/pinyinSearch';
import { cn } from '@/lib/utils';

interface ComboboxInputProps {
  value: string;
  onChange: (next: string) => void;
  options: string[];
  placeholder?: string;
  className?: string;
}

const FILTER_DEBOUNCE_MS = 120;

/// 自由文本输入 + 候选下拉. 支持:
/// - 直接子串匹配 (大小写无关).
/// - 拼音首字母匹配 (例 "ba" → "博奥").
/// - 输入防抖 120ms 才过滤候选.
/// 用户既可挑下拉, 也可手填任意值, onChange 反映当前 input 内容.
export function ComboboxInput({
  value,
  onChange,
  options,
  placeholder,
  className,
}: ComboboxInputProps) {
  const [open, setOpen] = useState(false);
  const [debounced, setDebounced] = useState(value);
  // 下拉默认向下展开; 接近视口底部时翻转向上 (例如批次单 prompt 的最后
  // 一两条配方, 向下展开会被 DialogFooter 遮住).
  const [openUp, setOpenUp] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), FILTER_DEBOUNCE_MS);
    return () => clearTimeout(t);
  }, [value]);

  // 打开瞬间测一次空间, 决定上 / 下展开方向.
  useEffect(() => {
    if (!open) return;
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;
    // max-h-48 = 12rem ≈ 192px; 留点余量定 220.
    setOpenUp(spaceBelow < 220 && spaceAbove > spaceBelow);
  }, [open]);

  // 点外部关下拉.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, [open]);

  const filtered = useMemo(() => {
    const q = debounced.trim();
    if (!q) return options;
    return options.filter((o) => matchOption(q, o));
  }, [debounced, options]);

  const pickOption = (opt: string) => {
    onChange(opt);
    setOpen(false);
  };

  return (
    <div ref={containerRef} className={cn('relative', className)}>
      <Input
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          if (!open) setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault();
            setOpen(false);
          }
        }}
        placeholder={placeholder}
        autoComplete="off"
      />
      {open && filtered.length > 0 && (
        <div
          className={cn(
            'absolute left-0 right-0 z-50 max-h-48 overflow-auto rounded-md border bg-popover py-1 shadow-md',
            openUp ? 'bottom-full mb-1' : 'top-full mt-1',
          )}
        >
          {filtered.map((opt) => (
            <button
              key={opt}
              type="button"
              // 用 onMouseDown + preventDefault 避免 input blur 抢先关 popover.
              onMouseDown={(e) => {
                e.preventDefault();
                pickOption(opt);
              }}
              className="block w-full truncate px-3 py-1.5 text-left text-sm hover:bg-accent"
            >
              {opt}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
