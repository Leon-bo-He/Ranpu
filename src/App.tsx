import { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { Loader2 } from 'lucide-react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { IdleDetector } from '@/components/IdleDetector';
import { LockOverlay } from '@/components/LockOverlay';
import { Sidebar } from '@/components/Sidebar';
import { TopBar } from '@/components/TopBar';
import { UpdateNotifier } from '@/components/UpdateNotifier';
import { AboutPage } from '@/pages/About';
import { AuditLogPage } from '@/pages/AuditLog';
import { BootScreen } from '@/pages/BootScreen';
import { CalculatorPage } from '@/pages/Calculator';
import { CartPage } from '@/pages/Cart';
import { DashboardPage } from '@/pages/Dashboard';
import { DefaultLibraryPage } from '@/pages/DefaultLibrary';
import { FirstRunSetup } from '@/pages/FirstRunSetup';
import { LibraryTransferPage } from '@/pages/LibraryTransfer';
import { LoginPage } from '@/pages/Login';
import { PrintPreviewWindow } from '@/pages/PrintPreviewWindow';
import { SettingsPage } from '@/pages/Settings';
import { UserManagementPage } from '@/pages/UserManagement';
import { WorkspaceFormulasPage } from '@/pages/WorkspaceFormulas';
import { WorkspaceManagementPage } from '@/pages/WorkspaceManagement';
import { useSessionStore } from '@/store/session';

/// 打印预览子窗口标记: 后端创建窗口时把 ?ranpu-view=print-preview 拼到 URL,
/// 这边读 window.location.search 分流. 比 getCurrentWebviewWindow().label
/// 可靠 — 是浏览器原生同步 API, 不依赖 Tauri 内部 metadata 注入时机.
function isPrintPreviewWindow(): boolean {
  if (typeof window === 'undefined') return false;
  const params = new URLSearchParams(window.location.search);
  return params.get('ranpu-view') === 'print-preview';
}

type GateState =
  | { kind: 'checking' }
  | { kind: 'first-run' }
  | { kind: 'boot' }
  | { kind: 'login' }
  | { kind: 'app' }
  | { kind: 'error'; message: string };

function App() {
  // 主进程开 print-preview 子窗口时, 走的是同一个 SPA bundle. URL 带
  // ?ranpu-view=print-preview 区分: 子窗口完全跳过 boot/login/路由,
  // 渲染独立 UI. 拆成两个组件而不是 if-then-return 是为了不违反
  // rules-of-hooks (子窗口压根不调那些 hooks).
  if (isPrintPreviewWindow()) {
    return <PrintPreviewWindow />;
  }
  return <MainApp />;
}

function MainApp() {
  const session = useSessionStore((s) => s.session);
  const [gate, setGate] = useState<GateState>({ kind: 'checking' });

  useEffect(() => {
    bootApi
      .status()
      .then((s) => {
        if (!s.keystore_exists) {
          setGate({ kind: 'first-run' });
        } else if (!s.db_initialized) {
          setGate({ kind: 'boot' });
        } else if (s.user_count === 0) {
          // 极少出现：keystore 存在但 DB 没用户。引导走 FirstRunSetup。
          setGate({ kind: 'first-run' });
        } else {
          setGate(session ? { kind: 'app' } : { kind: 'login' });
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
  // 拿到 session（登录 / FirstRunSetup 完成）→ 进 app；
  // session 被清空（logout / 强制登出）→ 回 login；
  // 'checking' / 'error' 是初始 / 出错状态，不被 session 变化影响。
  useEffect(() => {
    setGate((g) => {
      if (g.kind === 'checking' || g.kind === 'error') return g;
      if (session) return { kind: 'app' };
      // session 没拿到但还在 first-run / boot 等启动阶段：保持原样
      if (g.kind === 'first-run' || g.kind === 'boot') return g;
      return { kind: 'login' };
    });
  }, [session]);

  if (gate.kind === 'checking') {
    return <SplashLoader />;
  }
  if (gate.kind === 'error') {
    return (
      <div className="flex min-h-screen items-center justify-center p-8">
        <div className="rounded-md border border-destructive/40 bg-destructive/5 p-6 text-sm text-destructive">
          启动出错：{gate.message}
        </div>
      </div>
    );
  }
  if (gate.kind === 'first-run') {
    return <FirstRunSetup />;
  }
  if (gate.kind === 'boot') {
    return (
      <BootScreen
        onBooted={(userCount) => {
          if (userCount === 0) setGate({ kind: 'first-run' });
          else setGate({ kind: 'login' });
        }}
      />
    );
  }
  if (gate.kind === 'login' || !session) {
    return <LoginPage />;
  }

  // App 主体: TopBar 横通栏 + 下方左 Sidebar 200px + 右侧路由内容
  return (
    <BrowserRouter>
      <IdleDetector />
      <UpdateNotifier />
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
              <Route path="/users" element={<UserManagementPage />} />
              <Route path="/audit" element={<AuditLogPage />} />
              <Route path="/library-transfer" element={<LibraryTransferPage />} />
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

export default App;
