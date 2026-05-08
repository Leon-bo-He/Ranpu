import { getVersion } from '@tauri-apps/api/app';
import { message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { useEffect, useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';
import { RanpuLogo } from '@/components/RanpuLogo';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useUpdateStore } from '@/store/update';

export function AboutPage() {
  const [version, setVersion] = useState<string>('—');
  const pending = useUpdateStore((s) => s.pending);
  const checking = useUpdateStore((s) => s.checking);
  const runCheck = useUpdateStore((s) => s.runCheck);

  const [installOpen, setInstallOpen] = useState(false);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion('—'));
  }, []);

  // 启动时若 UpdateNotifier 还没跑完检查, 这里点按钮也会触发 (store 内部
  // 自带 in-flight 守卫, 不会重复打 IPC). 已知有新版本就直接进 install 弹窗.
  const onClickButton = async () => {
    if (pending) {
      setInstallOpen(true);
      return;
    }
    const u = await runCheck();
    if (u) {
      setInstallOpen(true);
    } else {
      await message('当前已是最新版本。', { title: '检查更新', kind: 'info' });
    }
  };

  const onInstall = async () => {
    if (!pending) return;
    try {
      await pending.downloadAndInstall();
      await relaunch();
    } catch (e) {
      await message(`更新失败：${e instanceof Error ? e.message : String(e)}`, {
        title: '更新',
        kind: 'error',
      });
    }
  };

  const buttonLabel = checking
    ? '检查中…'
    : pending
      ? `有新版本 ${pending.version}`
      : '检查更新';

  return (
    <div className="flex flex-col items-center gap-4 p-8">
      <RanpuLogo size={64} withText />
      <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
        DYE FORMULA
      </p>

      <Card className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>关于染谱</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-sm">
          <div className="flex items-center justify-between">
            <span>版本：{version}</span>
            <Button
              size="sm"
              variant="outline"
              onClick={onClickButton}
              disabled={checking}
              className="relative"
            >
              {buttonLabel}
              {pending && (
                <span
                  aria-hidden
                  className="absolute -right-1 -top-1 h-2.5 w-2.5 rounded-full bg-red-500 ring-2 ring-background"
                />
              )}
            </Button>
          </div>
          <p>
            染谱是一个面向印染车间的离线配方管理 + 染料计算软件，运行在 Windows 桌面，
            数据本地加密存储 (SQLCipher + DPAPI)。
          </p>
          <p>
            架构采用 DDD + Hexagonal/Ports-and-Adapters；前端 React + Tailwind +
            shadcn/ui；后端 Rust + Tauri 2。
          </p>
          <p className="text-muted-foreground">© {new Date().getFullYear()} 染谱 Ranpu</p>
        </CardContent>
      </Card>

      <ConfirmDialog
        open={installOpen}
        onClose={() => setInstallOpen(false)}
        title={`发现新版本 ${pending?.version ?? ''}`}
        description={
          <span>
            当前 {version} → 新版本 {pending?.version}。
            <br />
            点击"立即更新"会下载并安装，然后自动重启应用。
          </span>
        }
        confirmLabel="立即更新"
        cancelLabel="稍后"
        onConfirm={onInstall}
      />
    </div>
  );
}

export default AboutPage;
