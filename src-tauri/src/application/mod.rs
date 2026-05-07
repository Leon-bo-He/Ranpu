//! Application 层：定义 ports（trait）+ 编排 use case。
//!
//! 严禁 import infrastructure；只能依赖 domain 与 std/chrono/thiserror/uuid。

pub mod errors;
pub mod ports;

pub use errors::{AppError, AppResult};
