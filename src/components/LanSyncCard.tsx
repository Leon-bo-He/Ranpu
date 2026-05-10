import { useEffect, useState } from 'react';

import { ApiError } from '@/api/invoke';
import { syncApi, type SyncPeerView, type SyncStatusView } from '@/api/sync';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

const PEER_POLL_MS = 3_000;

/// 设置页 "局域网同步" 卡. 第一阶段只做服务发现:
/// - toggle 启动 / 停止 mDNS 服务.
/// - 启动后轮询同网段同伴列表展示.
/// - 还没有真正同步动作 — 后续 PR 加.
export function LanSyncCard() {
  const [status, setStatus] = useState<SyncStatusView>({ running: false, instance_id: null });
  const [peers, setPeers] = useState<SyncPeerView[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // mount 时拉一次状态; 后续靠 toggle 触发.
  useEffect(() => {
    syncApi.status().then(setStatus).catch(() => {
      // 后端没注册 / 加载失败时静默, 卡片仍会显示但 toggle 出错时提示.
    });
  }, []);

  // 启用时定时拉 peers (浏览有延迟, 单次拉不到, 必须轮询).
  useEffect(() => {
    if (!status.running) {
      setPeers([]);
      return;
    }
    let cancelled = false;
    const tick = () => {
      syncApi
        .listPeers()
        .then((list) => {
          if (!cancelled) setPeers(list);
        })
        .catch(() => {
          /* 不抢屏, 偶发失败下次再说 */
        });
    };
    tick();
    const id = setInterval(tick, PEER_POLL_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [status.running]);

  const onToggle = async () => {
    setBusy(true);
    setError(null);
    try {
      const next = status.running ? await syncApi.disable() : await syncApi.enable();
      setStatus(next);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0">
        <CardTitle>局域网同步</CardTitle>
        <Button
          variant={status.running ? 'destructive' : 'default'}
          size="sm"
          onClick={onToggle}
          disabled={busy}
        >
          {busy ? '处理中…' : status.running ? '关闭' : '开启'}
        </Button>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-muted-foreground">
          开启后这台染谱会在局域网广播自己，同时显示同网段其它在线实例。
          目前只做发现，下一步将加同步协议（端到端加密）。
        </p>
        {error && <p className="text-sm text-destructive">{error}</p>}
        {status.running && status.instance_id && (
          <p className="text-xs text-muted-foreground">
            本机 ID：
            <span className="ml-1 font-mono">
              {status.instance_id.slice(0, 8)}…
            </span>
          </p>
        )}
        {status.running && (
          <div className="space-y-1">
            <p className="text-sm font-medium">同网段其它实例</p>
            {peers.length === 0 ? (
              <p className="text-xs text-muted-foreground">暂未发现其它实例。</p>
            ) : (
              <ul className="space-y-1 text-sm">
                {peers.map((p) => (
                  <li key={p.instance_id} className="rounded-md border px-3 py-2">
                    <div className="font-medium">{p.hostname}</div>
                    <div className="text-xs text-muted-foreground">
                      v{p.app_version} · {p.addresses.join(', ') || '（解析中）'}
                      {p.port > 0 && <span className="ml-1">:{p.port}</span>}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
