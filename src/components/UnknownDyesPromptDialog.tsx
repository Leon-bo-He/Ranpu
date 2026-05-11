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

interface UnknownDyesPromptDialogProps {
  open: boolean;
  /// 保存配方时收集到的新染料名 (不在 dyeLibrary 里, 已去重 + 去首尾空白).
  unknowns: string[];
  /// 用户选好后, 把要加入库的染料名传出去; 空数组也算确认 (用户取消勾选).
  onConfirm: (toAdd: string[]) => void;
  onCancel: () => void;
}

/// 保存配方前的兜底: 检查 dye_name 有没有不在染料库的新词. 有则弹这个
/// dialog 让用户挑哪些加进库. 默认全勾, 跟 UnknownYarnPromptDialog 同款.
export function UnknownDyesPromptDialog({
  open,
  unknowns,
  onConfirm,
  onCancel,
}: UnknownDyesPromptDialogProps) {
  const [picked, setPicked] = useState<boolean[]>([]);

  useEffect(() => {
    if (open) {
      setPicked(unknowns.map(() => true));
    }
  }, [open, unknowns]);

  const onSubmit = () => {
    onConfirm(unknowns.filter((_, i) => picked[i]));
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onCancel()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>有新的染料名，加入到库里复用？</DialogTitle>
          <DialogDescription>
            勾选的名字会保存到「设置 → 染料库」，下次编辑配方可以直接挑。
            不想加入的取消勾选即可，本次仍会照原样保存。
          </DialogDescription>
        </DialogHeader>
        <ul className="space-y-1 max-h-[50vh] overflow-y-auto">
          {unknowns.map((name, i) => (
            <li key={`${i}-${name}`}>
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
                <span>{name}</span>
              </label>
            </li>
          ))}
        </ul>
        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={onCancel}>
            取消
          </Button>
          <Button onClick={onSubmit}>继续保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
