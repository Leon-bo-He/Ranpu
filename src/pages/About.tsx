import { getVersion } from '@tauri-apps/api/app';
import { message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { useEffect, useState } from 'react';

import { ConfirmDialog } from '@/components/ConfirmDialog';
import { RanpuLogo } from '@/components/RanpuLogo';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export function AboutPage() {
  const [version, setVersion] = useState<string>('—');
  const [checking, setChecking] = useState(false);
  const [pending, setPending] = useState<Update | null>(null);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion('—'));
  }, []);

  const onCheckUpdate = async () => {
    setChecking(true);
    try {
      const update = await check();
      if (update) {
        setPending(update);
      } else {
        await message('当前已是最新版本。', { title: '检查更新', kind: 'info' });
      }
    } catch (e) {
      await message(`检查更新失败：${e instanceof Error ? e.message : String(e)}`, {
        title: '检查更新',
        kind: 'error',
      });
    } finally {
      setChecking(false);
    }
  };

  const onInstall = async () => {
    if (!pending) return;
    await pending.downloadAndInstall();
    await relaunch();
  };

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
            <Button size="sm" variant="outline" onClick={onCheckUpdate} disabled={checking}>
              {checking ? '检查中…' : '检查更新'}
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
        open={pending !== null}
        onClose={() => setPending(null)}
        title={`发现新版本 ${pending?.version ?? ''}`}
        description={
          <span>
            当前 {version} → 新版本 {pending?.version}。
            {pending?.body ? <><br />{pending.body}</> : null}
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
