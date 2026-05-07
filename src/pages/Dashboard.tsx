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
          desc="全公司共享的染纱配方模板。admin 维护，所有人可查、可加密导出。"
          icon={<BookOpen className="h-5 w-5" />}
        />
        <NavCard
          to="/workspace-formulas"
          title="工作区配方"
          desc="当前客户 / 项目的专属配方，与默认库相互隔离，可加密互导。"
          icon={<Layers className="h-5 w-5" />}
        />
        <NavCard
          to="/calculator"
          title="染料计算器"
          desc="按内部色号或客户色号查配方，输入目标 kg 自动算每种染料的投料克数。"
          icon={<CalcIcon className="h-5 w-5" />}
        />
        <NavCard
          to="/cart"
          title="批次清单"
          desc="本缸要染的多条配方汇总，一键导出批次单 (CSV / 可打印 HTML) 交操作工。"
          icon={<ShoppingCart className="h-5 w-5" />}
        />
        {admin && (
          <NavCard
            to="/workspaces"
            title="工作区管理"
            desc="按客户或项目划分工作区。配方与批次清单按工作区隔离。"
            icon={<Layers className="h-5 w-5" />}
          />
        )}
        {admin && (
          <NavCard
            to="/users"
            title="用户管理"
            desc="新建 / 停用 / 启用登录账号，分配管理员或普通用户角色。"
            icon={<UsersIcon className="h-5 w-5" />}
          />
        )}
        {admin && (
          <NavCard
            to="/audit"
            title="审计日志"
            desc="查询全部用户操作记录。支持按日期 / 用户 / 动作筛选，加密 .ranpu 或明文 CSV 导出。"
            icon={<ClipboardList className="h-5 w-5" />}
          />
        )}
        <NavCard
          to="/settings"
          title="设置"
          desc="自动锁屏时长 (5/10/30/60 分钟) + 修改自己的登录密码。"
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
      <Card className="h-full transition-colors hover:bg-accent/50">
        <CardHeader className="flex flex-row items-start gap-3 space-y-0">
          <div className="shrink-0 rounded-md bg-secondary p-2 text-secondary-foreground">
            {icon}
          </div>
          <div className="space-y-1">
            <CardTitle className="text-base leading-tight">{title}</CardTitle>
            <CardDescription className="text-xs leading-relaxed">
              {desc}
            </CardDescription>
          </div>
        </CardHeader>
      </Card>
    </Link>
  );
}

export default DashboardPage;
