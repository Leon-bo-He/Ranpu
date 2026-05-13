import { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { Loader2 } from 'lucide-react';

import { bootApi } from '@/api/boot';
import { formulaApi } from '@/api/formula';
import { ApiError } from '@/api/invoke';
import { CloseConfirmGuard } from '@/components/CloseConfirmGuard';
import { EditModeAutoOffGuard } from '@/components/EditModeAutoOffGuard';
import { IdleDetector } from '@/components/IdleDetector';
import { LockOverlay } from '@/components/LockOverlay';
import { Sidebar } from '@/components/Sidebar';
import { TopBar } from '@/components/TopBar';
import { AboutPage } from '@/pages/About';
import { AuditLogPage } from '@/pages/AuditLog';
import { BootScreen } from '@/pages/BootScreen';
import { CalculatorPage } from '@/pages/Calculator';
import { CartPage } from '@/pages/Cart';
import { DashboardPage } from '@/pages/Dashboard';
import { DefaultLibraryPage } from '@/pages/DefaultLibrary';
import { FirstRunSetup } from '@/pages/FirstRunSetup';
import { LibraryTransferPage } from '@/pages/LibraryTransfer';
import { SettingsPage } from '@/pages/Settings';
import { WorkspaceFormulasPage } from '@/pages/WorkspaceFormulas';
import { WorkspaceManagementPage } from '@/pages/WorkspaceManagement';
import { useColorFamilyLibraryStore } from '@/store/colorFamilyLibrary';
import { useEditModeStore } from '@/store/editMode';
import { useSessionStore } from '@/store/session';
import { useUpdateStore } from '@/store/update';

type GateState =
  | { kind: 'checking' }
  | { kind: 'first-run' }
  | { kind: 'boot' }
  | { kind: 'app' }
  | { kind: 'error'; message: string };

function App() {
  const session = useSessionStore((s) => s.session);
  const [gate, setGate] = useState<GateState>({ kind: 'checking' });

  // 静默查一次更新; 命中 → useUpdateStore.pending 设上, 侧栏 "关于"
  // 项右边的红点 + About 页按钮 "有新版本 X.Y.Z" 自动显示. 不弹 toast.
  const runUpdateCheck = useUpdateStore((s) => s.runCheck);
  const updateChecked = useUpdateStore((s) => s.hasChecked);
  useEffect(() => {
    if (session && !updateChecked) {
      runUpdateCheck();
    }
  }, [session, updateChecked, runUpdateCheck]);

  // 色系库一次性导入: 升级到有色系库的版本后, session 建好就把 DB 里
  // 历史色系全部 merge 进库. flag 持久化 (zustand persist), 跑一次就置真
  // 不再触发. 失败 (例如 IPC 临时不可用) 不打 flag, 下次进 app 重试.
  const cfImported = useColorFamilyLibraryStore((s) => s.imported);
  const cfCurrent = useColorFamilyLibraryStore((s) => s.colorFamilies);
  const cfSet = useColorFamilyLibraryStore((s) => s.setColorFamilies);
  const cfMarkImported = useColorFamilyLibraryStore((s) => s.markImported);
  useEffect(() => {
    if (!session || cfImported) return;
    formulaApi
      .listAllColorFamilies()
      .then((fromDb) => {
        // 合并 + 大小写 / 首尾空白去重 (库优先, DB 历史追加).
        const seen = new Set<string>();
        const out: string[] = [];
        for (const v of [...cfCurrent, ...fromDb]) {
          const k = v.trim().toLowerCase();
          if (!k || seen.has(k)) continue;
          seen.add(k);
          out.push(v.trim());
        }
        cfSet(out);
        cfMarkImported();
      })
      .catch(() => {
        /* 静默失败, 下次进 app 再试 */
      });
    // 故意省 cfCurrent / cfSet / cfMarkImported 依赖: 只在 session / imported
    // 翻转时跑一次. cfCurrent 跟随 cfSet 变会触发再跑, 但 imported 也同步
    // 翻成 true, 守卫已经挡住, 多触发一次也无害 — 还是 eslint-disable 比较清楚.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [session, cfImported]);

  useEffect(() => {
    bootApi
      .status()
      .then((s) => {
        if (!s.keystore_exists) {
          setGate({ kind: 'first-run' });
        } else if (!s.db_initialized) {
          setGate({ kind: 'boot' });
        } else {
          // keystore 已存在, DB 已初始化 → 进 boot 流程让用户输入启动口令
          // 拿到 SessionView 后由 setSession 推到 'app'.
          setGate(session ? { kind: 'app' } : { kind: 'boot' });
        }
      })
      .catch((e: unknown) => {
        setGate({
          kind: 'error',
          message: e instanceof ApiError ? e.message : String(e),
        });
      });
    // 仅在挂载时探测一次启动状态，登录态变化由下面的 effect 处理
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 当 session 变化时同步 gate
  // 拿到 session（boot / FirstRunSetup 完成）→ 进 app；
  // session 被清空（锁定后强制清掉的极少数情况）→ 回 boot；
  // 'checking' / 'error' 是初始 / 出错状态，不被 session 变化影响。
  useEffect(() => {
    setGate((g) => {
      if (g.kind === 'checking' || g.kind === 'error') return g;
      if (session) return { kind: 'app' };
      // session 没拿到但还在 first-run 阶段：保持原样
      if (g.kind === 'first-run') return g;
      return { kind: 'boot' };
    });
  }, [session]);

  // CloseConfirmGuard 跨所有 gate 状态都要挂着, 否则 splash / boot / lock 时
  // 用户点 X 直接关掉就跳过确认了. 用 fragment 把它放进各分支的最外层.
  if (gate.kind === 'checking') {
    return (
      <>
        <CloseConfirmGuard />
        <SplashLoader />
      </>
    );
  }
  if (gate.kind === 'error') {
    return (
      <>
        <CloseConfirmGuard />
        <div className="flex min-h-screen items-center justify-center p-8">
          <div className="rounded-md border border-destructive/40 bg-destructive/5 p-6 text-sm text-destructive">
            启动出错：{gate.message}
          </div>
        </div>
      </>
    );
  }
  if (gate.kind === 'first-run') {
    return (
      <>
        <CloseConfirmGuard />
        <FirstRunSetup />
      </>
    );
  }
  if (gate.kind === 'boot' || !session) {
    return (
      <>
        <CloseConfirmGuard />
        <BootScreen />
      </>
    );
  }

  // App 主体: TopBar 横通栏 + 下方左 Sidebar 200px + 右侧路由内容
  return (
    <BrowserRouter>
      <CloseConfirmGuard />
      <IdleDetector />
      <EditModeAutoOffGuard />
      <div className="flex h-screen flex-col bg-background text-foreground">
        <TopBar />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />
          <main className="flex-1 overflow-auto">
            <Routes>
              <Route path="/" element={<DashboardPage />} />
              <Route path="/default-library" element={<DefaultLibraryPage />} />
              <Route path="/workspace-formulas" element={<WorkspaceFormulasPage />} />
              <Route path="/calculator" element={<CalculatorPage />} />
              <Route path="/cart" element={<CartPage />} />
              <Route path="/workspaces" element={<WorkspaceManagementPage />} />
              <Route path="/audit" element={<AuditLogPage />} />
              <Route
                path="/library-transfer"
                element={
                  <RequireLibraryTransfer>
                    <LibraryTransferPage />
                  </RequireLibraryTransfer>
                }
              />
              <Route path="/about" element={<AboutPage />} />
              <Route path="/settings" element={<SettingsPage />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </main>
        </div>
      </div>
      {session.locked && <LockOverlay />}
    </BrowserRouter>
  );
}

function SplashLoader() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-3 bg-background">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      <p className="text-sm text-muted-foreground">正在启动…</p>
    </div>
  );
}

/// "配方互导" 路由守卫. toggle 关时直接重定向回主面板, 不渲染页面 —
/// 防止用户从主面板卡片以外的入口 (旧书签 / 直接敲 URL) 绕过密码检查.
function RequireLibraryTransfer({ children }: { children: React.ReactNode }) {
  const enabled = useEditModeStore((s) => s.libraryTransferEnabled);
  if (!enabled) return <Navigate to="/" replace />;
  return <>{children}</>;
}

export default App;
