//! 局域网同步服务. 第一阶段 (本 PR) 只做服务发现:
//! - 在 mDNS / Bonjour 上广播自己 (`_ranpu-sync._tcp.local.`).
//! - 同时浏览同一服务名, 收集同网段其它实例的列表.
//! - 暴露 peer 列表给前端 (设置页 → 局域网同步卡显示).
//!
//! 后续 PR 会在这个 service 上面叠:
//! - HTTP 服务 (axum) 暴露 /info, /pull, /push 等.
//! - 配对握手 (PSK 或配对码), 防陌生人接入.
//! - 同步协议 (基于 last-write-wins / CRDT 二选一).
//! - 端到端加密 (用启动口令派生的对称密钥).
//!
//! 本期目标: 用户能在设置里看见局域网里运行的另一台染谱机器, 但还不能
//! 真正同步数据. 这是把架构骨架先立起来, 同步协议留给后续 PR.

pub mod peer;
pub mod service;

pub use peer::Peer;
pub use service::SyncService;
