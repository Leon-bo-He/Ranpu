import { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { Loader2 } from 'lucide-react';

import { bootApi } from '@/api/boot';
import { ApiError } from '@/api/invoke';
import { IdleDetector } from '@/components/IdleDetector';
import { LockOverlay } from '@/components/LockOverlay';
import { TopBar } from '@/components/TopBar';
import { BootScreen } from '@/pages/BootScreen';
import { CalculatorPage } from '@/pages/Calculator';
import { CartPage } from '@/pages/Cart';
import { DashboardPage } from '@/pages/Dashboard';
import { DefaultLibraryPage } from '@/pages/DefaultLibrary';
import { FirstRunSetup } from '@/pages/FirstRunSetup';
import { LoginPage } from '@/pages/Login';
import { SettingsPage } from '@/pages/Settings';
import { UserManagementPage } from '@/pages/UserManagement';
import { WorkspaceFormulasPage } from '@/pages/WorkspaceFormulas';
import { useSessionStore } from '@/store/session';

type GateState =
  | { kind: 'checking' }
  | { kind: 'first-run' }
  | { kind: 'boot' }
  | { kind: 'login' }
  | { kind: 'app' }
  | { kind: 'error'; message: string };

function App() {
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
  useEffect(() => {
    setGate((g) => {
      if (g.kind === 'checking' || g.kind === 'first-run' || g.kind === 'error') return g;
      if (session) return { kind: 'app' };
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

  // App 主体
  return (
    <BrowserRouter>
      <IdleDetector />
      <div className="min-h-screen bg-background text-foreground">
        <TopBar />
        <Routes>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/default-library" element={<DefaultLibraryPage />} />
          <Route path="/workspace-formulas" element={<WorkspaceFormulasPage />} />
          <Route path="/calculator" element={<CalculatorPage />} />
          <Route path="/cart" element={<CartPage />} />
          <Route path="/users" element={<UserManagementPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
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
