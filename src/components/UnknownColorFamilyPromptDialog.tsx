import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

interface UnknownColorFamilyPromptDialogProps {
  open: boolean;
  /// 保存配方时填的新色系名 (不在色系库里, 已去首尾空白).
  unknown: string;
  /// 用户确认加入库 → onConfirmAdd; 不加入但仍保存 → onConfirmSkip; 关闭 → onCancel.
  onConfirmAdd: () => void;
  onConfirmSkip: () => void;
  onCancel: () => void;
}

/// 保存配方前的兜底: 检查 color_family 有没有不在色系库的新词. 有则弹这个
/// dialog 让用户选要不要加进库. 单个色系所以不用 checkbox 列表, 两个按钮:
/// 加入并保存 / 不加入只保存.
export function UnknownColorFamilyPromptDialog({
  open,
  unknown,
  onConfirmAdd,
  onConfirmSkip,
  onCancel,
}: UnknownColorFamilyPromptDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onCancel()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>有新的色系，加入到库里复用？</DialogTitle>
          <DialogDescription>
            「{unknown}」不在色系库里。加入后会保存到「设置 → 色系库」，
            下次编辑配方可以直接挑（所有工作区共享）。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="gap-2">
          <Button variant="ghost" onClick={onCancel}>
            取消
          </Button>
          <Button variant="outline" onClick={onConfirmSkip}>
            不加入，只保存
          </Button>
          <Button onClick={onConfirmAdd}>加入并保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
