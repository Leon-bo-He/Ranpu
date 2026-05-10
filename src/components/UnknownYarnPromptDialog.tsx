import { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

export interface UnknownYarnEntry {
  /// "mill" | "spec" — 用来分类显示 "厂名" / "规格".
  kind: 'mill' | 'spec';
  value: string;
}

interface UnknownYarnPromptDialogProps {
  open: boolean;
  unknowns: UnknownYarnEntry[];
  /// 用户选好后, 把要加入库的 (kind, value) 数组传出去, 调用方再写 store.
  /// 数组空也算确认 (用户选 "不加, 这次先用一下").
  onConfirm: (toAdd: UnknownYarnEntry[]) => void;
  onCancel: () => void;
}

/// 批次单 prompt 提交前的兜底: 检查 prompt 里的厂名 / 规格有没有不在
/// 设置库中的新词. 如果有, 弹这个 dialog 让用户挑哪些加进库. 默认全勾,
/// 用户可逐条取消.
export function UnknownYarnPromptDialog({
  open,
  unknowns,
  onConfirm,
  onCancel,
}: UnknownYarnPromptDialogProps) {
  // 每条 unknown 一个 boolean: 是否加入库. 默认 true.
  const [picked, setPicked] = useState<boolean[]>([]);

  useEffect(() => {
    if (open) {
      setPicked(unknowns.map(() => true));
    }
  }, [open, unknowns]);

  const onSubmit = () => {
    const toAdd = unknowns.filter((_, i) => picked[i]);
    onConfirm(toAdd);
  };

  const millEntries = unknowns
    .map((u, i) => ({ u, i }))
    .filter(({ u }) => u.kind === 'mill');
  const specEntries = unknowns
    .map((u, i) => ({ u, i }))
    .filter(({ u }) => u.kind === 'spec');

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onCancel()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>有新的纱支项，加入到库里复用？</DialogTitle>
          <DialogDescription>
            勾选的项会保存到「设置 → 纱支」，下次输入可以直接挑。
            不想加入的取消勾选即可，这次仍会按原样使用。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-3 max-h-[50vh] overflow-y-auto">
          {millEntries.length > 0 && (
            <section>
              <p className="mb-1 text-xs font-medium text-muted-foreground">厂名</p>
              <ul className="space-y-1">
                {millEntries.map(({ u, i }) => (
                  <li key={`mill-${i}`}>
                    <label className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm cursor-pointer hover:bg-accent">
                      <input
                        type="checkbox"
                        checked={picked[i] ?? false}
                        onChange={(e) =>
                          setPicked((prev) => {
                            const next = [...prev];
                            next[i] = e.target.checked;
                            return next;
                          })
                        }
                      />
                      <span>{u.value}</span>
                    </label>
                  </li>
                ))}
              </ul>
            </section>
          )}
          {specEntries.length > 0 && (
            <section>
              <p className="mb-1 text-xs font-medium text-muted-foreground">规格</p>
              <ul className="space-y-1">
                {specEntries.map(({ u, i }) => (
                  <li key={`spec-${i}`}>
                    <label className="flex items-center gap-2 rounded-md border px-3 py-2 text-sm cursor-pointer hover:bg-accent">
                      <input
                        type="checkbox"
                        checked={picked[i] ?? false}
                        onChange={(e) =>
                          setPicked((prev) => {
                            const next = [...prev];
                            next[i] = e.target.checked;
                            return next;
                          })
                        }
                      />
                      <span>{u.value}</span>
                    </label>
                  </li>
                ))}
              </ul>
            </section>
          )}
        </div>
        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={onCancel}>
            取消
          </Button>
          <Button onClick={onSubmit}>继续</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
