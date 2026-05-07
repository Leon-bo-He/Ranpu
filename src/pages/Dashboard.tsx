import { Link } from 'react-router-dom';
import {
  BookOpen,
  Calculator as CalcIcon,
  ClipboardList,
  Layers,
  Settings as Cog,
  ShoppingCart,
  Users as UsersIcon,
} from 'lucide-react';

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { hasActiveWorkspace, isAdmin, useSessionStore } from '@/store/session';

export function DashboardPage() {
  const session = useSessionStore((s) => s.session);
  if (!session) return null;
  const admin = isAdmin(session);
  const hasWs = hasActiveWorkspace(session);

  return (
    <div className="space-y-6 p-6">
      <h2 className="font-serif text-2xl tracking-[2px]">主面板</h2>
      {!hasWs && (
        <p className="rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
          当前未选择工作区。配方编辑、染料计算与批次清单都需要先在顶栏选一个工作区。
        </p>
      )}

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        <NavCard
          to="/default-library"
          title="默认配方库"
          desc="任何登录用户都能查看的全局配方"
          icon={<BookOpen className="h-5 w-5" />}
        />
        <NavCard
          to="/workspace-formulas"
          title="工作区配方"
          desc="当前工作区里的配方"
          icon={<Layers className="h-5 w-5" />}
        />
        <NavCard
          to="/calculator"
          title="染料计算器"
          desc="按内部色号 + kg 立刻得出克数"
          icon={<CalcIcon className="h-5 w-5" />}
        />
        <NavCard
          to="/cart"
          title="批次清单"
          desc="本次染浴的配方与计算结果汇总"
          icon={<ShoppingCart className="h-5 w-5" />}
        />
        {admin && (
          <NavCard
            to="/workspaces"
            title="工作区管理"
            desc="创建 / 重命名 / 删除工作区"
            icon={<Layers className="h-5 w-5" />}
          />
        )}
        {admin && (
          <NavCard
            to="/users"
            title="用户管理"
            desc="新建用户、停用用户"
            icon={<UsersIcon className="h-5 w-5" />}
          />
        )}
        {admin && (
          <NavCard
            to="/audit"
            title="审计日志"
            desc="查看 / 导出系统操作记录"
            icon={<ClipboardList className="h-5 w-5" />}
          />
        )}
        <NavCard
          to="/settings"
          title="设置"
          desc="自动锁屏、修改密码"
          icon={<Cog className="h-5 w-5" />}
        />
      </div>
    </div>
  );
}

function NavCard({
  to,
  title,
  desc,
  icon,
}: {
  to: string;
  title: string;
  desc: string;
  icon: React.ReactNode;
}) {
  return (
    <Link to={to}>
      <Card className="transition-colors hover:bg-accent/50">
        <CardHeader className="flex flex-row items-center gap-3 space-y-0">
          <div className="rounded-md bg-secondary p-2 text-secondary-foreground">
            {icon}
          </div>
          <div>
            <CardTitle className="text-base">{title}</CardTitle>
            <CardDescription className="text-xs">{desc}</CardDescription>
          </div>
        </CardHeader>
        <CardContent />
      </Card>
    </Link>
  );
}

export default DashboardPage;
