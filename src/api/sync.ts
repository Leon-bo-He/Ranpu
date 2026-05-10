import { invoke } from './invoke';

export interface SyncStatusView {
  running: boolean;
  /// 启动后这台机器在 mDNS 上的 instance_id (uuid). 没启动是 null.
  instance_id: string | null;
}

export interface SyncPeerView {
  instance_id: string;
  hostname: string;
  app_version: string;
  /// 该实例广播的 IP 字符串 (可能是 IPv4 / IPv6 混合).
  addresses: string[];
  port: number;
}

export const syncApi = {
  status: () => invoke<SyncStatusView>('cmd_sync_status'),
  enable: () => invoke<SyncStatusView>('cmd_sync_enable'),
  disable: () => invoke<SyncStatusView>('cmd_sync_disable'),
  listPeers: () => invoke<SyncPeerView[]>('cmd_sync_list_peers'),
};
