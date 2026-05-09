//! Domain 层：零外部依赖（仅 std + chrono + thiserror + uuid）。
//!
//! 各上下文按 PROMPT 第 91-142 行的限界上下文划分：
//! session / workspace / formula / calculation / cart / audit。
//! 加上 shared/ 存放跨上下文的强类型 ID 与基础错误。
//! 单用户解锁模型: 没有 identity 上下文 (没有用户表 / 角色 / 登录).

pub mod audit;
pub mod calculation;
pub mod cart;
pub mod formula;
pub mod session;
pub mod shared;
pub mod workspace;
