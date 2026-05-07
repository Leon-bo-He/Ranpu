import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { RanpuLogo } from '@/components/RanpuLogo';

export function AboutPage() {
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
        <CardContent className="space-y-2 text-sm">
          <p>版本：0.1.0</p>
          <p>
            染谱是一个面向印染车间的离线配方管理 + 染料计算软件，运行在 Windows 桌面，
            数据本地加密存储 (SQLCipher + DPAPI)。
          </p>
          <p>
            架构采用 DDD + Hexagonal/Ports-and-Adapters；前端 React + Tailwind +
            shadcn/ui；后端 Rust + Tauri 2。
          </p>
          <p className="text-muted-foreground">© 2026 染谱 Ranpu</p>
        </CardContent>
      </Card>
    </div>
  );
}

export default AboutPage;
