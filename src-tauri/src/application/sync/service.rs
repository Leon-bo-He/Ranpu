use std::collections::HashMap;
use std::sync::Arc;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use parking_lot::Mutex;
use thiserror::Error;
use uuid::Uuid;

use super::peer::Peer;

const SERVICE_TYPE: &str = "_ranpu-sync._tcp.local.";
/// 第一阶段还没有真的 HTTP 服务, 用一个固定端口作为 mDNS announce 的元数
/// 据. 后续 PR 加 HTTP 时再绑这个端口 (或换成动态分配).
const PLACEHOLDER_PORT: u16 = 8765;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("局域网同步初始化失败: {0}")]
    Init(String),
}

/// 局域网同步服务. start() 后:
/// - 在 mDNS 上广播自己 (TXT records 含 instance_id / version / hostname).
/// - 浏览同名服务, 用一个后台 std::thread 把发现的同伴塞进 peers map.
/// - 暴露 peers() 给前端轮询展示.
/// drop 时调 daemon.shutdown(), browse 线程因 channel 关闭自然退出.
pub struct SyncService {
    daemon: ServiceDaemon,
    instance_id: String,
    peers: Arc<Mutex<HashMap<String, Peer>>>,
}

impl SyncService {
    pub fn start(app_version: &str) -> Result<Self, SyncError> {
        let daemon = ServiceDaemon::new().map_err(|e| SyncError::Init(e.to_string()))?;
        let instance_id = Uuid::new_v4().to_string();
        let hostname = local_hostname();

        let mut props: HashMap<String, String> = HashMap::new();
        props.insert("instance_id".into(), instance_id.clone());
        props.insert("version".into(), app_version.to_owned());
        props.insert("hostname".into(), hostname.clone());

        // ip 给空字符串 + enable_addr_auto: 让 mdns-sd 自动选本机各网卡的 IP.
        let info = ServiceInfo::new(
            SERVICE_TYPE,
            &instance_id,
            &format!("{hostname}.local."),
            "",
            PLACEHOLDER_PORT,
            props,
        )
        .map_err(|e| SyncError::Init(e.to_string()))?
        .enable_addr_auto();

        daemon
            .register(info)
            .map_err(|e| SyncError::Init(e.to_string()))?;

        let receiver = daemon
            .browse(SERVICE_TYPE)
            .map_err(|e| SyncError::Init(e.to_string()))?;

        let peers: Arc<Mutex<HashMap<String, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
        let peers_for_task = peers.clone();
        let my_id = instance_id.clone();

        // Browse 循环单独一个 std::thread; daemon.shutdown() 会关闭 receiver,
        // 循环 recv() 拿到 Err 就自然退出.
        std::thread::Builder::new()
            .name("ranpu-sync-browse".to_owned())
            .spawn(move || {
                while let Ok(event) = receiver.recv() {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            if let Some(peer) = peer_from_info(&info, &my_id) {
                                peers_for_task
                                    .lock()
                                    .insert(info.get_fullname().to_owned(), peer);
                            }
                        }
                        ServiceEvent::ServiceRemoved(_, fullname) => {
                            peers_for_task.lock().remove(&fullname);
                        }
                        _ => {}
                    }
                }
            })
            .map_err(|e| SyncError::Init(e.to_string()))?;

        Ok(Self {
            daemon,
            instance_id,
            peers,
        })
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn peers(&self) -> Vec<Peer> {
        let mut out: Vec<Peer> = self.peers.lock().values().cloned().collect();
        // 按 hostname / instance_id 排序保证 UI 列表稳定.
        out.sort_by(|a, b| a.hostname.cmp(&b.hostname).then(a.instance_id.cmp(&b.instance_id)));
        out
    }
}

impl Drop for SyncService {
    fn drop(&mut self) {
        let _ = self.daemon.shutdown();
    }
}

/// 从一条 ServiceResolved 事件抠出 Peer; 是自己 (instance_id 相同) 时返回
/// None, 调用方就不会把自己塞进列表.
fn peer_from_info(info: &ServiceInfo, my_id: &str) -> Option<Peer> {
    let instance_id = info.get_property_val_str("instance_id")?.to_owned();
    if instance_id == my_id {
        return None;
    }
    let hostname = info
        .get_property_val_str("hostname")
        .map(str::to_owned)
        .unwrap_or_else(|| info.get_hostname().trim_end_matches('.').to_owned());
    let app_version = info
        .get_property_val_str("version")
        .map(str::to_owned)
        .unwrap_or_default();
    let addresses = info.get_addresses().iter().copied().collect();
    Some(Peer {
        instance_id,
        hostname,
        app_version,
        addresses,
        port: info.get_port(),
    })
}

/// 跨平台拿一份当前机器的 hostname. Windows 用 COMPUTERNAME, Unix 用
/// HOSTNAME, 都没有就 fallback "ranpu".
fn local_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "ranpu".to_owned())
}
