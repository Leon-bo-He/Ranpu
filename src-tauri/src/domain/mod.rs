//! Domain 层：零外部依赖（仅 std + chrono + thiserror + uuid）。
//!
//! 各上下文按 PROMPT 第 91-142 行的限界上下文划分：
//! identity / workspace / formula / calculation / cart / audit。
//! 加上 shared/ 存放跨上下文的强类型 ID 与基础错误。

pub mod calculation;
pub mod formula;
pub mod identity;
pub mod shared;
pub mod workspace;
