use std::net::IpAddr;

/// 局域网里发现的一个 染谱 实例. 通过 mDNS TXT records 拿到的元数据.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    /// 实例唯一 ID (uuid v4, 启动时生成, 同一进程生命周期内不变).
    /// 用来去重 / 排除自己.
    pub instance_id: String,
    /// 用户友好的机器名 (mDNS hostname 去掉 .local 后缀).
    pub hostname: String,
    /// 该实例的版本号 (例 "1.0.15"), 后续协议协商用.
    pub app_version: String,
    /// 服务监听的 IP 地址列表 (IPv4 / IPv6 都可能有).
    pub addresses: Vec<IpAddr>,
    /// 服务监听端口.
    pub port: u16,
}
